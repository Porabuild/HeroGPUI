"""Regression tests for the macOS native capture driver's pure helpers.

These cover the control-file protocol port, binary resolution, the CGWindow
choice filter and the stdlib PNG verification. Nothing here launches the
gallery, Cargo, or any GUI process.
"""
from pathlib import Path
import shutil
import tempfile
import unittest
import zlib

import native_capture as driver


def png_chunk(kind, body):
    return (len(body).to_bytes(4, "big") + kind + body
            + zlib.crc32(kind + body).to_bytes(4, "big"))


def write_png(path, width, height, rows, color_type=6):
    """A minimal 8-bit PNG writer so uniformity checks have real input."""
    ihdr = (width.to_bytes(4, "big") + height.to_bytes(4, "big")
            + bytes([8, color_type, 0, 0, 0]))
    channels = {0: 1, 2: 3, 3: 1, 4: 2, 6: 4}[color_type]
    raw = b"".join(b"\x00" + row for row in rows)
    assert all(len(row) == width * channels for row in rows)
    path.write_bytes(
        b"\x89PNG\r\n\x1a\n" + png_chunk(b"IHDR", ihdr)
        + png_chunk(b"IDAT", zlib.compress(raw)) + png_chunk(b"IEND", b""))


class ControlPublicationTests(unittest.TestCase):
    def setUp(self):
        self.directory = Path(tempfile.mkdtemp(prefix="herogpui-native-test-"))
        self.control = self.directory / "control.txt"
        self.ack, self.error = driver.result_paths(self.control)

    def tearDown(self):
        shutil.rmtree(self.directory, ignore_errors=True)

    def test_result_paths_replace_the_last_extension(self):
        base = Path("/tmp/one/two.3.txt")
        ack, error = driver.result_paths(base)
        self.assertEqual(ack, Path("/tmp/one/two.3.ack"))
        self.assertEqual(error, Path("/tmp/one/two.3.error"))

    def test_publish_is_complete_utf8_and_retires_old_results(self):
        self.ack.write_text("old", encoding="utf-8")
        self.error.write_text("old error", encoding="utf-8")
        driver.write_control(self.control, ["seq=1", "page=Button", "section=État"])
        self.assertEqual(self.control.read_bytes(), "seq=1\npage=Button\nsection=État".encode())
        self.assertFalse(self.ack.exists() or self.error.exists())
        self.assertEqual(list(self.directory.glob("*.tmp")), [])

    def test_rejected_publication_cleans_up_and_retires_results_first(self):
        # A directory at the destination makes the replacing move fail on
        # every host; the PowerShell writer retires the result files *before*
        # the move, so a rejected publication must look the same here.
        driver.write_control(self.control, ["seq=1"])
        self.ack.write_text("1", encoding="utf-8")
        self.error.write_text("seq=0\nerror=stale", encoding="utf-8")
        blocked = self.directory / "blocked.txt"
        blocked.mkdir()
        with self.assertRaises(OSError):
            driver.write_control(blocked, ["seq=2", "page=Select"])
        self.assertEqual(list(self.directory.glob("*.tmp")), [])
        self.assertFalse(any(self.directory.glob("blocked.txt.ack")))
        self.assertFalse(any(self.directory.glob("blocked.txt.error")))
        self.assertEqual(self.control.read_text(encoding="utf-8"), "seq=1")

    def test_lines_keep_field_order_and_omit_unset_fields(self):
        self.assertEqual(
            driver.control_lines(
                "4", page="Combo Box", section="Usage", specimen="cb-main",
                theme="dark", overlays="1", reset="1", preview="component",
                motion="reduce"),
            ["seq=4", "page=Combo Box", "section=Usage", "specimen=cb-main",
             "theme=dark", "overlays=1", "reset=1", "preview=component",
             "motion=reduce"])
        # The gallery preserves an absent page/preview/motion and defaults the
        # rest, so the writer must not invent lines for omitted fields.
        self.assertEqual(driver.control_lines("2", page="Button"),
                         ["seq=2", "page=Button"])


class AcknowledgementTests(unittest.TestCase):
    def setUp(self):
        self.directory = Path(tempfile.mkdtemp(prefix="herogpui-native-test-"))
        self.control = self.directory / "control.txt"
        self.ack, self.error = driver.result_paths(self.control)
        self.alive = lambda: True

    def tearDown(self):
        shutil.rmtree(self.directory, ignore_errors=True)

    def test_matching_sequence_acknowledges_even_with_surrounding_space(self):
        self.ack.write_text(" 1\n", encoding="utf-8")
        ok, failure = driver.wait_ack(self.control, "1", self.alive, timeout_ms=300)
        self.assertTrue(ok, failure)

    def test_a_different_sequence_cannot_acknowledge(self):
        self.ack.write_text("other", encoding="utf-8")
        ok, failure = driver.wait_ack(self.control, "1", self.alive, timeout_ms=120,
                                      poll_ms=20)
        self.assertFalse(ok)
        self.assertEqual(failure, "no rendered-frame acknowledgement for 1")

    def test_sequences_match_case_sensitively(self):
        self.ack.write_text("RUN-A", encoding="utf-8")
        self.error.write_text("seq=RUN-A\nerror=wrong case", encoding="utf-8")
        ok, failure = driver.wait_ack(self.control, "run-a", self.alive, timeout_ms=120,
                                      poll_ms=20)
        self.assertFalse(ok)
        self.assertFalse(failure and "wrong case" in failure)

    def test_a_matching_error_surfaces_even_when_a_stale_ack_exists(self):
        self.ack.write_text("1", encoding="utf-8")
        self.error.write_text("seq=1\nerror=unknown page", encoding="utf-8")
        ok, failure = driver.wait_ack(self.control, "1", self.alive, timeout_ms=300)
        self.assertFalse(ok)
        self.assertIn("unknown page", failure)

    def test_an_error_for_a_different_sequence_is_ignored(self):
        self.error.write_text("seq=0\nerror=stale", encoding="utf-8")
        ok, failure = driver.wait_ack(self.control, "1", self.alive, timeout_ms=120,
                                      poll_ms=20)
        self.assertFalse(ok)
        self.assertEqual(failure, "no rendered-frame acknowledgement for 1")

    def test_a_dead_process_reports_before_any_result_file(self):
        ok, failure = driver.wait_ack(self.control, "1", lambda: False, timeout_ms=300)
        self.assertFalse(ok)
        self.assertEqual(failure, "gallery exited before acknowledging")


class BinaryResolutionTests(unittest.TestCase):
    def test_gallery_manifest_declares_one_binary(self):
        self.assertEqual(driver.gallery_bin_name(), "herogpui-gallery")

    def test_a_manifest_without_a_bin_fails_loudly(self):
        with tempfile.TemporaryDirectory() as directory:
            manifest = Path(directory) / "Cargo.toml"
            manifest.write_text('[package]\nname = "other"\n', encoding="utf-8")
            with self.assertRaises(driver.DriverError):
                driver.gallery_bin_name(manifest)


class WindowChoiceTests(unittest.TestCase):
    def window(self, window_id, pid, layer=0, title=None):
        return {"id": window_id, "pid": pid, "layer": layer, "title": title,
                "owner": "Gallery", "bounds": {"x": 0, "y": 0, "w": 1200, "h": 800}}

    def test_chooses_the_launched_process_normal_layer_window(self):
        windows = [
            self.window(99, 4242, layer=-1, title="Dock-adjacent"),
            self.window(20, 777, title="HeroGPUI"),  # another app can own the words
            self.window(30, 4242, title="HeroGPUI — Gallery"),
        ]
        chosen = driver.choose_window(windows, 4242)
        self.assertEqual(chosen["id"], 30)

    def test_no_candidate_is_none_not_a_guess(self):
        self.assertIsNone(driver.choose_window([self.window(1, 5)], 4242))
        self.assertIsNone(driver.choose_window([], 4242))

    def test_titles_do_not_drive_the_choice(self):
        # A redacted title (no Screen Recording permission) must still be
        # chosen, and a matching title on a foreign pid must never be.
        self.assertEqual(driver.choose_window(
            [self.window(7, 4242, title=None)], 4242)["id"], 7)
        self.assertIsNone(driver.choose_window(
            [self.window(8, 9, title="HeroGPUI — Gallery")], 4242))


class PngSummaryTests(unittest.TestCase):
    def setUp(self):
        self.directory = Path(tempfile.mkdtemp(prefix="herogpui-native-test-"))

    def tearDown(self):
        shutil.rmtree(self.directory, ignore_errors=True)

    def row(self, width, pixel):
        return bytes(pixel) * width

    def test_dimensions_and_uniformity(self):
        path = self.directory / "uniform.png"
        write_png(path, 5, 3, [self.row(5, (10, 20, 30, 255))] * 3)
        summary = driver.png_summary(path)
        self.assertEqual((summary["width"], summary["height"]), (5, 3))
        self.assertTrue(summary["uniform"])

    def test_one_different_pixel_makes_it_non_uniform(self):
        path = self.directory / "varied.png"
        rows = [bytearray(self.row(5, (10, 20, 30, 255))) for _ in range(3)]
        rows[2][4 * 4] = 200  # last pixel's red channel
        write_png(path, 5, 3, rows)
        self.assertFalse(driver.png_summary(path)["uniform"])

    def test_filtered_rows_are_unfiltered_before_comparison(self):
        # The same content encoded once unfiltered and once with Sub-filtered
        # rows must decode to identical scanline prefixes; a decoder that
        # skipped the filter would see shifted bytes instead.
        width, height = 4, 2
        plain = []
        for y in range(height):
            row = bytearray()
            for x in range(width):
                red = (40 + y * 10) if x % 2 else (x + y)
                row += bytes((red, 20, 30, 255))
            plain.append(bytes(row))
        self.assertNotEqual(plain[0], plain[1], "rows must differ for this test")

        path = self.directory / "plain.png"
        write_png(path, width, height, plain)
        plain_rows = driver.png_decoded_prefixes(path)
        self.assertEqual((plain_rows[0], plain_rows[1]), (width, height))
        self.assertEqual(plain_rows[2][0], plain[0][:width * 4])

        ihdr = (width.to_bytes(4, "big") + height.to_bytes(4, "big")
                + bytes([8, 6, 0, 0, 0]))
        # Filter type 1 (Sub) on every row: each byte delta from its left.
        raw = b"".join(
            b"\x01" + bytes((plain[y][i] - plain[y][i - 4]) & 0xFF
                            if i >= 4 else plain[y][i]
                            for i in range(width * 4))
            for y in range(height))
        filtered = self.directory / "filtered.png"
        filtered.write_bytes(
            b"\x89PNG\r\n\x1a\n" + png_chunk(b"IHDR", ihdr)
            + png_chunk(b"IDAT", zlib.compress(raw)) + png_chunk(b"IEND", b""))
        filtered_rows = driver.png_decoded_prefixes(filtered)
        self.assertEqual(filtered_rows, plain_rows)
        # And the decoded difference must still read as a real capture.
        self.assertFalse(driver.png_summary(path)["uniform"])
        self.assertFalse(driver.png_summary(filtered)["uniform"])

    def test_non_png_and_truncated_data_fail_loudly(self):
        not_png = self.directory / "not.png"
        not_png.write_bytes(b"GIF89a")
        with self.assertRaises(driver.CaptureError):
            driver.png_summary(not_png)
        truncated = self.directory / "truncated.png"
        write_png(truncated, 4, 4, [self.row(4, (1, 2, 3, 255))] * 4)
        body = truncated.read_bytes()
        # Locate the IDAT chunk by its kind field and keep only half the
        # compressed payload: an incomplete deflate stream must be reported,
        # never silently judged.
        idat_at = body.index(b"IDAT")
        length = int.from_bytes(body[idat_at - 4:idat_at], "big")
        cut = idat_at + 4 + length // 2
        truncated.write_bytes(body[:cut] + png_chunk(b"IEND", b""))
        with self.assertRaises(driver.CaptureError):
            driver.png_summary(truncated)


class ProvenanceTests(unittest.TestCase):
    def test_provenance_records_window_capture_and_request(self):
        window = {"id": 42, "pid": 4242, "owner": "herogpui-gallery",
                  "title": "HeroGPUI — Gallery",
                  "bounds": {"x": 0.0, "y": 33.0, "w": 1200.0, "h": 800.0}}
        captured = {
            "png": {"width": 2400, "height": 1600, "uniform": False},
            "dpr": 2.0,
            "logical": {"width": 1200.0, "height": 800.0},
        }
        record = driver.provenance(
            ["seq=1", "page=Button", "theme=dark"], window, captured,
            "deadbeef", Path("/target/debug/herogpui-gallery"), "debug",
            {"system": "Darwin"}, {"commit": "abc", "dirty": False})
        self.assertEqual(record["schema"], driver.PROVENANCE_SCHEMA)
        self.assertEqual(record["binary"]["sha256"], "deadbeef")
        self.assertEqual(record["window"]["cg_window_id"], 42)
        self.assertEqual(record["window"]["bounds_pt"]["w"], 1200.0)
        self.assertEqual(record["capture"]["png_width_px"], 2400)
        self.assertEqual(record["capture"]["dpr"], 2.0)
        self.assertEqual(record["capture"]["logical_size_px"], {"width": 1200.0, "height": 800.0})
        self.assertEqual(record["request"]["lines"], ["seq=1", "page=Button", "theme=dark"])

    def test_dpr_fractional_scales_round_for_the_record(self):
        window = {"id": 1, "pid": 1, "owner": None, "title": None,
                  "bounds": {"x": 0.0, "y": 0.0, "w": 1512.0, "h": 982.0}}
        captured = {
            "png": {"width": 2268, "height": 1473, "uniform": False},
            "dpr": 1.5,
            "logical": {"width": 1512.0, "height": 982.0},
        }
        record = driver.provenance(["seq=1"], window, captured, "h", Path("b"),
                                   "debug", {}, {})
        self.assertEqual(record["capture"]["dpr"], 1.5)
        self.assertEqual(record["capture"]["png_height_px"], 1473)


class SlugTests(unittest.TestCase):
    def test_slug_matches_the_driver_naming_style(self):
        self.assertEqual(driver.slug("Label & Messages"), "label-messages")
        self.assertEqual(driver.slug("Combo Box"), "combo-box")
        self.assertEqual(driver.slug(None), "")


if __name__ == "__main__":
    unittest.main()
