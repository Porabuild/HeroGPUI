"""macOS native-surface capture driver: build, launch, drive, capture.

    python3 .shots/native_capture.py --page Button --section Usage \
        --preview component --specimen btn-usage \
        --theme light --theme dark \
        --out-template 'docs/parity/evidence/native-button-usage/native-usage-{theme}.png'

The capture and smoke drivers in this directory are Windows-only: they post
input to an off-screen window and photograph it with `PrintWindow`. This is
the host-specific counterpart (plan section 5.2) with the same
reset/state/ack semantics, and it owns the macOS half of the protocol:

1. Build `herogpui-gallery` through Cargo when stale (Cargo itself is the
   staleness oracle; a fresh build is a no-op) and resolve the image from
   `gallery/Cargo.toml`'s `[[bin]]` plus `cargo metadata`'s target directory
   -- never from an assumed path.
2. Launch the gallery detached with `HEROGPUI_UNFOCUSED=1` and a per-run
   `HEROGPUI_CONTROL` file, without touching the caller's environment.
3. Publish complete UTF-8 requests by atomic replacement -- the exact
   semantics of `.shots/control.ps1` -- and wait for the case-sensitive
   `seq` in `.ack`, or surface the matching `seq`/`error` record from
   `.error`.
4. Capture the window through `screencapture -l<window-id> -x -o`, which
   reads the window's own backing store rather than the screen composition --
   the closest macOS relative of `PrintWindow`. The window is found by owner
   process id through CGWindowList, never by title. Every capture is checked
   for the blank or uniform frame a Screen Recording permission denial
   produces, and its provenance (window id and bounds, PNG pixel size, DPR,
   binary sha256, control transcript) is written next to the image.
5. Stop the child and remove the control, result and log files before exit,
   whether the run passed or failed.

Requires macOS and Screen Recording permission for the process that runs
this script (the permission belongs to the host terminal or app). See
`.shots/native_capture.md` for prerequisites, the computer-use fallback when
permission cannot be granted, and what an acknowledgement does not prove.
"""
from __future__ import annotations

import argparse
import ctypes
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import sys
import tempfile
import time
import tomllib
import uuid
import zlib

from gpui_patches import materialize_all
from interaction_inventory import ROOT, write_json_atomic

DRIVER = ".shots/native_capture.py"
PROVENANCE_SCHEMA = "herogpui-native-capture/1"
DEFAULT_OUT_TEMPLATE = ".shots/~native-{n}.png"
# Matches Wait-GalleryControl's default in .shots/control.ps1.
DEFAULT_ACK_TIMEOUT_MS = 3600
WINDOW_TIMEOUT_S = 20.0

# One gallery package, one binary image; the name comes from the manifest so
# a rename cannot leave a driver launching a stale path behind.
GALLERY_MANIFEST = ROOT / "gallery" / "Cargo.toml"


class DriverError(RuntimeError):
    """A loud, named failure -- missing input is not an empty passing result."""


class CaptureError(DriverError):
    """The gallery rendered, but the window image could not be trusted."""


# -- the control protocol (port of .shots/control.ps1) ------------------------


def control_lines(seq, page=None, section=None, specimen=None, theme=None,
                  overlays=None, reset=None, preview=None, motion=None):
    """One complete request in the field order the PowerShell drivers use.

    Omitted fields stay omitted: the gallery preserves an absent `page`,
    `preview` and `motion`, and resets `section`, `theme` and `overlays` to
    their defaults. `seq` must be nonempty and new for every request.
    """
    lines = [f"seq={seq}"]
    for key, value in (
        ("page", page),
        ("section", section),
        ("specimen", specimen),
        ("theme", theme),
        ("overlays", overlays),
        ("reset", reset),
        ("preview", preview),
        ("motion", motion),
    ):
        if value is not None:
            lines.append(f"{key}={value}")
    return lines


def result_paths(path):
    """The sibling result files: `control.txt` produces `control.ack`/`control.error`."""
    path = Path(path)
    return path.with_suffix(".ack"), path.with_suffix(".error")


def write_control(path, lines):
    """Publish a complete UTF-8 request by atomic replacement.

    A sibling temporary file keeps readers from seeing a partial sequence
    number with the previous page's defaults; the previous `.ack`/`.error`
    files are retired before the new request appears, so a result can only
    ever answer the sequence currently published.
    """
    path = Path(path)
    ack, error = result_paths(path)
    temporary = path.with_name(f"{path.name}.{uuid.uuid4().hex}.tmp")
    try:
        # UTF-8 without a BOM: the reader is byte-exact, and a BOM would
        # become part of the first key.
        temporary.write_text("\n".join(lines), encoding="utf-8")
        ack.unlink(missing_ok=True)
        error.unlink(missing_ok=True)
        temporary.replace(path)
    finally:
        temporary.unlink(missing_ok=True)


def wait_ack(path, seq, still_running, timeout_ms=DEFAULT_ACK_TIMEOUT_MS,
             poll_ms=50):
    """Wait for the case-sensitive sequence in `.ack`, or surface `.error`.

    The acknowledgement is scheduled from the requested render and delivered
    on a later frame, so polling is the correct way to observe it; a fixed
    timer cannot establish that ordering. `still_running` is a callable so
    tests can stand in for the child process.
    """
    ack, error = result_paths(path)
    deadline = time.monotonic() + timeout_ms / 1000.0
    while time.monotonic() < deadline:
        if not still_running():
            return False, "gallery exited before acknowledging"
        reported = []
        try:
            reported = error.read_text(encoding="utf-8").splitlines()
        except OSError:
            pass
        if reported and reported[0] == f"seq={seq}":
            return False, " ".join(reported)
        try:
            seen = ack.read_text(encoding="utf-8")
        except OSError:
            seen = ""
        if seen and seen.strip() == seq:
            return True, ""
        time.sleep(poll_ms / 1000.0)
    return False, f"no rendered-frame acknowledgement for {seq}"


# -- binary resolution and build ------------------------------------------------


def gallery_bin_name(manifest=GALLERY_MANIFEST):
    """The gallery's binary image name, read from its `[[bin]]` table."""
    parsed = tomllib.loads(Path(manifest).read_text(encoding="utf-8"))
    names = sorted({entry["name"] for entry in (parsed.get("bin") or [])})
    if len(names) != 1:
        raise DriverError(
            f"{manifest} must declare exactly one [[bin]] (found {names or 'none'})")
    return names[0]


def cargo_target_dir(repo=ROOT):
    result = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=repo, capture_output=True, text=True,
    )
    if result.returncode != 0:
        raise DriverError(f"cargo metadata failed: {result.stderr.strip()}")
    return Path(json.loads(result.stdout)["target_directory"])


def resolve_binary(repo=ROOT, release=False, manifest=GALLERY_MANIFEST):
    """Where Cargo writes the gallery image for this profile."""
    profile = "release" if release else "debug"
    binary = cargo_target_dir(repo) / profile / gallery_bin_name(manifest)
    if not binary.is_file():
        raise DriverError(
            f"gallery binary missing at {binary} -- build it first: "
            "cargo build -p herogpui-gallery")
    return binary


def build_gallery(repo=ROOT, release=False):
    """Build the gallery package; Cargo decides what is stale."""
    command = ["cargo", "build", "-p", "herogpui-gallery"]
    if release:
        command.append("--release")
    result = subprocess.run(command, cwd=repo)
    if result.returncode != 0:
        raise DriverError(
            f"build failed ({result.returncode}) -- do not trust captures taken after this")
    return resolve_binary(repo, release=release)


# -- window discovery (the framework load lives in the function, so importing
#    this module and running its unit tests works on any host) -------------------


def onscreen_windows():
    """Every normal on-screen window of every process, via CGWindowList.

    Reads CoreGraphics through ctypes because pyobjc is not a dependency of
    this repository. Titles are only populated when the calling process holds
    Screen Recording permission; discovery never relies on them.
    """
    if sys.platform != "darwin":
        raise DriverError("window discovery requires macOS (this is a native driver)")

    class CGRect(ctypes.Structure):
        _fields_ = [
            ("x", ctypes.c_double), ("y", ctypes.c_double),
            ("width", ctypes.c_double), ("height", ctypes.c_double),
        ]

    cg = ctypes.cdll.LoadLibrary(
        "/System/Library/Frameworks/CoreGraphics.framework/CoreGraphics")
    cf = ctypes.cdll.LoadLibrary(
        "/System/Library/Frameworks/CoreFoundation.framework/CoreFoundation")
    cg.CGWindowListCopyWindowInfo.restype = ctypes.c_void_p
    cg.CGWindowListCopyWindowInfo.argtypes = [ctypes.c_uint32, ctypes.c_uint32]
    cf.CFArrayGetCount.restype = ctypes.c_long
    cf.CFArrayGetCount.argtypes = [ctypes.c_void_p]
    cf.CFArrayGetValueAtIndex.restype = ctypes.c_void_p
    cf.CFArrayGetValueAtIndex.argtypes = [ctypes.c_void_p, ctypes.c_long]
    cf.CFDictionaryGetValue.restype = ctypes.c_void_p
    cf.CFDictionaryGetValue.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
    cf.CFStringGetLength.restype = ctypes.c_long
    cf.CFStringGetLength.argtypes = [ctypes.c_void_p]
    cf.CFStringGetCString.restype = ctypes.c_bool
    cf.CFStringGetCString.argtypes = [
        ctypes.c_void_p, ctypes.c_char_p, ctypes.c_long, ctypes.c_uint32]
    cf.CFNumberGetValue.restype = ctypes.c_bool
    cf.CFNumberGetValue.argtypes = [ctypes.c_void_p, ctypes.c_long, ctypes.c_void_p]
    cf.CFRelease.argtypes = [ctypes.c_void_p]
    cg.CGRectMakeWithDictionaryRepresentation.restype = ctypes.c_bool
    cg.CGRectMakeWithDictionaryRepresentation.argtypes = [
        ctypes.c_void_p, ctypes.POINTER(CGRect)]

    def constant(name):
        return ctypes.c_void_p.in_dll(cg, name).value

    def cfstring(pointer):
        if not pointer:
            return None
        length = cf.CFStringGetLength(pointer)
        buffer = ctypes.create_string_buffer(length * 4 + 1)
        if cf.CFStringGetCString(pointer, buffer, len(buffer), 0x08000100):
            return buffer.value.decode("utf-8")
        return None

    def cfnumber(pointer):
        if not pointer:
            return None
        value = ctypes.c_int64(0)
        if cf.CFNumberGetValue(pointer, 4, ctypes.byref(value)):
            return value.value
        return None

    # kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements
    array = cg.CGWindowListCopyWindowInfo(1 | 16, 0)
    windows = []
    try:
        for index in range(cf.CFArrayGetCount(array)):
            entry = cf.CFArrayGetValueAtIndex(array, index)
            bounds_rect = CGRect()
            bounds_value = cf.CFDictionaryGetValue(
                entry, constant("kCGWindowBounds"))
            if not bounds_value or not cg.CGRectMakeWithDictionaryRepresentation(
                    bounds_value, ctypes.byref(bounds_rect)):
                continue
            windows.append({
                "id": cfnumber(cf.CFDictionaryGetValue(entry, constant("kCGWindowNumber"))),
                "pid": cfnumber(cf.CFDictionaryGetValue(entry, constant("kCGWindowOwnerPID"))),
                "owner": cfstring(cf.CFDictionaryGetValue(entry, constant("kCGWindowOwnerName"))),
                "title": cfstring(cf.CFDictionaryGetValue(entry, constant("kCGWindowName"))),
                "layer": cfnumber(cf.CFDictionaryGetValue(entry, constant("kCGWindowLayer"))),
                "bounds": {
                    "x": bounds_rect.x, "y": bounds_rect.y,
                    "w": bounds_rect.width, "h": bounds_rect.height,
                },
            })
    finally:
        cf.CFRelease(array)
    return windows


def choose_window(windows, pid):
    """The gallery's own content window: matching pid, normal layer.

    The pid is authoritative because the driver launched the process; a title
    match is neither required (titles are redacted without permission) nor
    safe (another app's window can carry the same words).
    """
    candidates = [
        window for window in windows
        if window["pid"] == pid and window["layer"] == 0 and window["id"]
    ]
    if not candidates:
        return None
    return sorted(candidates, key=lambda window: window["id"])[0]


# -- PNG verification (stdlib: this repository has no PIL dependency) -----------


def png_decoded_prefixes(path, sample_pixels=96):
    """Decode just enough of a PNG to return a prefix of every scanline.

    Returns `(width, height, rows)` where each row is the filter-reconstructed
    first `min(width, sample_pixels)` pixels. Scanline filters are undone over
    that prefix only, which is enough to judge a capture without decoding
    megapixels. Interlaced PNGs are rejected: `screencapture` never writes
    them, and guessing at one would be worse than failing.
    """
    data = Path(path).read_bytes()
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise CaptureError(f"{path} is not a PNG")
    offset = 8
    header = None
    compressed = b""
    while offset + 8 <= len(data):
        length = int.from_bytes(data[offset:offset + 4], "big")
        kind = data[offset + 4:offset + 8]
        body = data[offset + 8:offset + 8 + length]
        offset += 12 + length
        if kind == b"IHDR":
            header = body
        elif kind == b"IDAT":
            compressed += body
        elif kind == b"IEND":
            break
    if header is None:
        raise CaptureError(f"{path} has no IHDR")
    width = int.from_bytes(header[0:4], "big")
    height = int.from_bytes(header[4:8], "big")
    bit_depth, color_type, interlace = header[8], header[9], header[12]
    if interlace != 0:
        raise CaptureError(f"{path} is interlaced; refusing to guess at it")
    if bit_depth != 8 or color_type not in (0, 2, 3, 4, 6):
        raise CaptureError(
            f"{path} uses bit depth {bit_depth}, color type {color_type}; unsupported")
    channels = {0: 1, 2: 3, 3: 1, 4: 2, 6: 4}[color_type]
    try:
        raw = zlib.decompress(compressed)
    except zlib.error as decode_error:
        raise CaptureError(f"{path} could not be decoded: {decode_error}") from decode_error

    stride = width * channels
    if len(raw) < height * (stride + 1):
        raise CaptureError(f"{path} image data is truncated")
    span = min(width, sample_pixels) * channels
    bpp = channels
    rows = []
    previous = bytearray(span)
    for y in range(height):
        row = y * (stride + 1)
        filter_kind = raw[row]
        line = bytearray(raw[row + 1:row + 1 + span])
        # Undo the filter over the sampled prefix. Every predictor needs only
        # the left, upper and upper-left bytes, all inside the prefix.
        if filter_kind == 1:
            for i in range(bpp, span):
                line[i] = (line[i] + line[i - bpp]) & 0xFF
        elif filter_kind == 2:
            for i in range(span):
                line[i] = (line[i] + previous[i]) & 0xFF
        elif filter_kind == 3:
            for i in range(span):
                left = line[i - bpp] if i >= bpp else 0
                line[i] = (line[i] + (left + previous[i]) // 2) & 0xFF
        elif filter_kind == 4:
            for i in range(span):
                a = line[i - bpp] if i >= bpp else 0
                b = previous[i]
                c = previous[i - bpp] if i >= bpp else 0
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                predictor = a if (pa <= pb and pa <= pc) else (b if pb <= pc else c)
                line[i] = (line[i] + predictor) & 0xFF
        elif filter_kind != 0:
            raise CaptureError(f"{path} row {y} has unknown filter {filter_kind}")
        rows.append(bytes(line))
        previous = line
    return width, height, rows


def png_summary(path, sample_pixels=96, distinct_cap=8):
    """Dimensions and uniformity of a PNG, decoded just far enough to judge.

    A Screen Recording denial does not fail `screencapture` -- it produces a
    uniform frame or the desktop wallpaper instead of the window -- so the
    capture is checked against exactly that before it can become evidence.
    """
    width, height, rows = png_decoded_prefixes(path, sample_pixels=sample_pixels)
    seen = set()
    for row in rows:
        seen.add(row)
        if len(seen) > distinct_cap:
            return {"width": width, "height": height, "uniform": False}
    return {"width": width, "height": height, "uniform": len(seen) <= 1}


# -- capture --------------------------------------------------------------------


def capture_window(window, out_path):
    """Photograph one window with `screencapture -l<id> -x -o` and verify it.

    `-l` reads the window's own backing store rather than the screen, `-x`
    silences the capture sound, and `-o` drops the drop shadow so the PNG is
    exactly the window. The verification exists because a Screen Recording
    denial does not fail the command -- it silently produces a uniform frame
    or the desktop wallpaper instead of the window.
    """
    out_path = Path(out_path)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    result = subprocess.run(
        ["screencapture", "-x", "-o", f"-l{window['id']}", str(out_path)],
        capture_output=True, text=True,
    )
    if result.returncode != 0 or not out_path.is_file():
        detail = (result.stderr or result.stdout).strip() or "no output"
        raise CaptureError(
            f"screencapture -l{window['id']} failed ({result.returncode}): {detail}")
    summary = png_summary(out_path)
    if summary["uniform"]:
        raise CaptureError(
            f"capture of window {window['id']} is uniform/blank -- Screen Recording "
            "permission is likely missing for this process; grant it to the host "
            "terminal or app, or fall back to the computer-use screenshot path "
            "(see .shots/native_capture.md)")
    bounds = window["bounds"]
    if bounds["w"] <= 0 or bounds["h"] <= 0:
        raise CaptureError(f"window {window['id']} reports degenerate bounds {bounds}")
    png_ratio = summary["width"] / summary["height"]
    bounds_ratio = bounds["w"] / bounds["h"]
    if abs(png_ratio - bounds_ratio) / bounds_ratio > 0.02:
        raise CaptureError(
            f"capture {summary['width']}x{summary['height']} does not match window "
            f"bounds {bounds['w']:.0f}x{bounds['h']:.0f} -- the frame may not be "
            "this window's content")
    dpr = summary["width"] / bounds["w"]
    return {
        "png": summary,
        "dpr": dpr,
        "logical": {"width": summary["width"] / dpr, "height": summary["height"] / dpr},
    }


# -- provenance -----------------------------------------------------------------


def git_state(repo=ROOT):
    def git(*arguments):
        result = subprocess.run(
            ["git", *arguments], cwd=repo, capture_output=True, text=True)
        return result.stdout.strip() if result.returncode == 0 else None

    commit = git("rev-parse", "HEAD")
    return {"commit": commit, "dirty": bool(git("status", "--porcelain"))}


def provenance(request_lines, window, captured, binary_sha256, binary_path,
               profile, host, git_info):
    return {
        "schema": PROVENANCE_SCHEMA,
        "driver": DRIVER,
        "captured_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "host": host,
        "git": git_info,
        "binary": {
            "path": str(binary_path),
            "sha256": binary_sha256,
            "profile": profile,
            "package": "herogpui-gallery",
        },
        "window": {
            "cg_window_id": window["id"],
            "title": window["title"],
            "owner_pid": window["pid"],
            "owner_name": window["owner"],
            "bounds_pt": window["bounds"],
        },
        "capture": {
            "method": "screencapture -l<window-id> -x -o",
            "png_width_px": captured["png"]["width"],
            "png_height_px": captured["png"]["height"],
            "dpr": round(captured["dpr"], 4),
            "logical_size_px": {
                "width": round(captured["logical"]["width"], 2),
                "height": round(captured["logical"]["height"], 2),
            },
            "uniform": captured["png"]["uniform"],
            "retina_note": "PNG pixels are dpr x the window's logical points; "
                           "logical px = png px / dpr",
        },
        "request": {"lines": request_lines},
    }


# -- the session -----------------------------------------------------------------


def slug(value):
    return re.sub(r"[^A-Za-z0-9]+", "-", value or "").strip("-").lower()


def launch_gallery(binary, repo, control_path, window_size, log_file):
    """Start the gallery detached; its environment overrides are the child's own.

    Every HEROGPUI_* variable of the caller is stripped first, so a stray
    HEROGPUI_SECTION in the shell cannot quietly win over the control file.
    """
    environment = {
        name: value for name, value in os.environ.items()
        if not name.startswith("HEROGPUI_")
    }
    environment["HEROGPUI_CONTROL"] = str(control_path)
    environment["HEROGPUI_UNFOCUSED"] = "1"
    if window_size:
        environment["HEROGPUI_WINDOW_SIZE"] = window_size
    return subprocess.Popen(
        [str(binary)], cwd=str(repo), env=environment,
        stdin=subprocess.DEVNULL, stdout=log_file, stderr=log_file,
        start_new_session=True,
    )


def stop_gallery(child):
    """SIGTERM, then SIGKILL; the drivers stop the child before cleaning files."""
    if child is None or child.poll() is not None:
        return
    child.terminate()
    try:
        child.wait(timeout=5)
    except subprocess.TimeoutExpired:
        child.kill()
        child.wait()


def wait_for_window(pid, timeout_s=WINDOW_TIMEOUT_S):
    """Poll CGWindowList until the gallery's content window exists."""
    deadline = time.monotonic() + timeout_s
    while time.monotonic() < deadline:
        window = choose_window(onscreen_windows(), pid)
        if window is not None:
            return window
        time.sleep(0.25)
    raise DriverError(f"gallery process {pid} opened no capturable window in {timeout_s:.0f}s")


def tail(path, limit=1500):
    """The child's captured output, for a loud failure that names the cause."""
    try:
        return "\n" + Path(path).read_text(encoding="utf-8", errors="replace")[-limit:]
    except OSError:
        return ""


def parse_args(argv=None):
    parser = argparse.ArgumentParser(
        description="macOS native-surface gallery capture driver",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )
    parser.add_argument("--page", help="exact navigation title, e.g. 'Combo Box'")
    parser.add_argument("--section", help="case-insensitive substring/CSV section filter")
    parser.add_argument("--specimen", help="stable specimen key; requires --preview component")
    parser.add_argument("--theme", action="append", choices=("light", "dark"),
                        help="one capture step per theme (repeatable; default light)")
    parser.add_argument("--overlays", choices=("0", "1"))
    parser.add_argument("--reset", choices=("0", "1"))
    parser.add_argument("--preview", choices=("gallery", "component"))
    parser.add_argument("--motion", choices=("full", "reduce"))
    parser.add_argument("--out-template", default=DEFAULT_OUT_TEMPLATE,
                        help="path template with {n}, {theme}, {page}, {section} "
                             f"(default {DEFAULT_OUT_TEMPLATE})")
    parser.add_argument("--window-size", metavar="WxH",
                        help="HEROGPUI_WINDOW_SIZE; macOS may clamp a size larger "
                             "than the display (unlike the Windows drivers)")
    parser.add_argument("--ack-timeout-ms", type=int, default=DEFAULT_ACK_TIMEOUT_MS)
    parser.add_argument("--settle-ms", type=int, default=500,
                        help="delay between acknowledgement and capture, so a short "
                             "exit animation can finish")
    parser.add_argument("--release", action="store_true",
                        help="build and launch the release profile")
    parser.add_argument("--skip-build", action="store_true",
                        help="launch the existing image without invoking Cargo")
    args = parser.parse_args(argv)
    args.theme = args.theme or ["light"]
    return args


def main(argv=None):
    args = parse_args(argv)
    if sys.platform != "darwin":
        print("FAIL: native_capture.py is the macOS driver; "
              "use .shots/drive.ps1 or .shots/batch.ps1 on Windows", file=sys.stderr)
        return 2
    if args.specimen and args.preview != "component":
        print("FAIL: --specimen requires --preview component", file=sys.stderr)
        return 2

    fields = dict(
        page=args.page, section=args.section, specimen=args.specimen,
        overlays=args.overlays, reset=args.reset, preview=args.preview,
        motion=args.motion,
    )
    try:
        # Both branches below shell out to cargo, and cargo resolves
        # `[patch.crates-io]` at manifest load -- so the gitignored `.vendor/`
        # forks have to exist before either one runs. A warm call is a hash
        # comparison over the materialization stamps and prints nothing.
        materialize_all(quiet=True)
        binary = resolve_binary() if args.skip_build else build_gallery(release=args.release)
    except (DriverError, ValueError, KeyError, OSError) as error:
        print(f"FAIL: {error}", file=sys.stderr)
        return 1
    binary_sha256 = hashlib.sha256(binary.read_bytes()).hexdigest()
    profile = "release" if args.release else "debug"

    control = Path(tempfile.gettempdir()) / f"herogpui-native-{uuid.uuid4().hex}.txt"
    ack, error = result_paths(control)
    log_path = Path(tempfile.gettempdir()) / f"herogpui-native-{uuid.uuid4().hex}.log"
    child = None
    started = False
    captures = []
    try:
        with log_path.open("w", encoding="utf-8") as log_file:
            # The seed request renders the subject during startup, like the
            # PowerShell drivers' `seq=0`; the per-theme steps follow.
            write_control(control, control_lines("0", page=args.page))
            child = launch_gallery(binary, ROOT, control, args.window_size, log_file)
            started = True
            window = wait_for_window(child.pid)
            print(f"gallery pid {child.pid}, window {window['id']} "
                  f"({window['bounds']['w']:.0f}x{window['bounds']['h']:.0f} pt)")

            git_info = git_state()
            host = {
                "system": platform.system(),
                "release": platform.release(),
                "mac_ver": platform.mac_ver()[0],
                "machine": platform.machine(),
            }
            for n, theme in enumerate(args.theme, start=1):
                if child.poll() is not None:
                    raise DriverError(f"gallery exited before step {n}{tail(log_path)}")
                lines = control_lines(str(n), theme=theme, **fields)
                write_control(control, lines)
                ok, failure = wait_ack(
                    control, str(n), still_running=lambda: child.poll() is None,
                    timeout_ms=args.ack_timeout_ms)
                if child.poll() is not None:
                    raise DriverError(f"gallery died rendering step {n}{tail(log_path)}")
                if not ok:
                    raise DriverError(
                        f"step {n} was not acknowledged: {failure}{tail(log_path)}")
                time.sleep(args.settle_ms / 1000.0)

                # Re-query: the window can move or resize between steps, and
                # the provenance must describe the frame that was captured.
                window = choose_window(onscreen_windows(), child.pid)
                if window is None:
                    raise DriverError(
                        f"gallery window vanished before capture {n}{tail(log_path)}")
                out_path = Path(args.out_template.format(
                    n=n, theme=slug(theme), page=slug(args.page),
                    section=slug(args.section)))
                captured = capture_window(window, out_path)
                write_json_atomic(out_path.with_suffix(".json"), provenance(
                    lines, window, captured, binary_sha256, binary, profile,
                    host, git_info))
                captures.append(out_path)
                print(
                    f"{args.page or '(current page)'} {args.section or ''} {theme}: "
                    f"{captured['png']['width']}x{captured['png']['height']} px "
                    f"@ {captured['dpr']:.2f}x -> {out_path}")
    except (DriverError, OSError) as failure:
        print(f"FAIL: {failure}", file=sys.stderr)
        return 1
    finally:
        if started:
            stop_gallery(child)
        for leftover in (control, ack, error, log_path):
            Path(leftover).unlink(missing_ok=True)

    print(f"{len(captures)} capture(s); provenance JSON beside each PNG")
    return 0


if __name__ == "__main__":
    sys.exit(main())
