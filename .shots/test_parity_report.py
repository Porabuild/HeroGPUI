"""Prove that legacy zero exits and missing output cannot produce a green report."""
from pathlib import Path
from contextlib import redirect_stdout
import hashlib
import io
import os
import re
import tarfile
import tempfile
import unittest
from unittest.mock import patch

import parity_report as report
import api_audit
import demo_audit


class ReportTests(unittest.TestCase):
    def passing_output(self, name):
        positive, zero = report.CONTRACTS[name]
        result = '\n'.join(f'{label}: 1' for label in positive)
        result += '\n' + '\n'.join(f'{label}: 0' for label in zero)
        return result + '\n' + {
            'reason_audit.py': 'REASON: browser-only\n  Input.autoComplete\n',
            'reference_audit.py': 'PASS\n',
            'write_only.py': 'no write-only fields\n',
        }.get(name, '')

    def test_every_contract_accepts_positive_and_rejects_empty_output(self):
        for name in report.CONTRACTS:
            with self.subTest(name=name):
                self.assertEqual(report.analyze(name, self.passing_output(name), 0)['status'], 'passed')
                self.assertEqual(report.analyze(name, '', 0)['status'], 'failed')

    def test_every_failure_counter_fails_despite_success_exit(self):
        for name, (_, counters) in report.CONTRACTS.items():
            for label in counters:
                with self.subTest(name=name, label=label):
                    output = self.passing_output(name).replace(f'{label}: 0', f'{label}: 2')
                    self.assertEqual(report.analyze(name, output, 0)['status'], 'failed')

    def test_missing_duplicate_and_zero_input_counters_fail(self):
        for name, (counters, _) in report.CONTRACTS.items():
            for label in counters:
                output = self.passing_output(name)
                for broken in (output.replace(f'{label}: 1', ''),
                               output + f'\n{label}: 1',
                               output.replace(f'{label}: 1', f'{label}: 0')):
                    self.assertEqual(report.analyze(name, broken, 0)['status'], 'failed')

    def test_unowned_table_and_subcheck_cannot_hide_behind_zero_totals(self):
        for name, diagnostic in [('api_audit.py', 'PART TABLE UNOWNED: Select.ClearButton'),
                                 ('api_audit.py', 'API SECTION AMBIGUOUS: Button matched 0 sections'),
                                 ('anim_audit.py', 'DRAWER MISMATCHES: 1'),
                                 ('inert_audit.py', 'FROZEN   Slider  example')]:
            row = report.analyze(name, self.passing_output(name) + '\n' + diagnostic, 0)
            self.assertEqual(row['status'], 'failed')

    def test_actual_api_reader_missing_section_cannot_pass(self):
        stream = io.StringIO()
        sections = [section for section in api_audit.api_sections()
                    if '\n### Button\n' not in section]
        with patch.object(api_audit, 'api_sections', return_value=sections), redirect_stdout(stream):
            api_audit.main()
        output = stream.getvalue()
        self.assertIn('API SECTION AMBIGUOUS: Button matched 0 sections', output)
        self.assertEqual(report.analyze('api_audit.py', output, 0)['status'], 'failed')

    def test_fallback_demo_sources_are_pinned_and_use_distinct_cache_keys(self):
        moving = 'https://raw.githubusercontent.com/heroui-inc/heroui/v3/apps/docs/src/demos/en/tabs/basic.tsx'
        pinned = moving.replace('/v3/', '/v3.2.6/')
        self.assertEqual(demo_audit.pinned_source_url(moving), pinned)
        self.assertEqual(demo_audit.pinned_source_url(pinned), pinned)
        self.assertEqual(demo_audit.preview_cache_path(moving), demo_audit.preview_cache_path(pinned))
        old = moving.replace('/v3/', '/v3.2.5/')
        self.assertNotEqual(demo_audit.preview_cache_path(moving), demo_audit.preview_cache_path(old))
        for page in ('Link', 'Tabs'):
            source = re.search(r'^\*\*Source\*\*: (https://\S+)', demo_audit.bundle_page(page), re.M)
            self.assertIsNotNone(source, page)
            url = demo_audit.pinned_source_url(source.group(1))
            self.assertIn('/heroui-inc/heroui/v3.2.6/', url)
            self.assertNotIn('/refs/heads/', url)

    def test_stale_reason_and_nonzero_exit_fail(self):
        output = self.passing_output('reason_audit.py')
        for suffix in ('  unknown -- STALE: implemented', '  (no matching doc row -- stale entry?)'):
            self.assertEqual(report.analyze('reason_audit.py', output + suffix, 0)['status'], 'failed')
        self.assertEqual(report.analyze('reason_audit.py', output, 1)['status'], 'failed')

    def test_discovery_refuses_new_unaccounted_and_missing_audits(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / '.shots').mkdir()
            for name in report.CONTRACTS:
                (root / '.shots' / name).touch()
            self.assertEqual(set(report.discover(root)), set(report.CONTRACTS))
            extra = root / '.shots/new_audit.py'
            extra.touch()
            with self.assertRaises(ValueError):
                report.discover(root)
            extra.unlink()
            (root / '.shots/api_audit.py').unlink()
            with self.assertRaises(ValueError):
                report.discover(root)

    def test_pinned_runner_rejects_inherited_bundle_overrides(self):
        with patch.dict(os.environ):
            for key in ('HEROUI_BUNDLE', 'HEROUI_BUNDLE_UNPINNED'):
                os.environ.pop(key, None)
            report.require_pinned_environment()
            for key in ('HEROUI_BUNDLE', 'HEROUI_BUNDLE_UNPINNED'):
                with patch.dict(os.environ, {key: 'override'}):
                    with self.assertRaisesRegex(ValueError, key):
                        report.require_pinned_environment()

    def test_effective_cache_and_packaging_inputs_change_report_fingerprint(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / '.shots').mkdir()
            (root / '.shots/reader.py').write_text('reader')
            names = ('web/src/data/reference.json', 'web/src/data/rust-examples.json', 'llms.txt',
                     'README.md', 'RELEASING.md', 'NOTICE', 'LICENSE', 'deny.toml',
                     '.github/workflows/ci.yml', '.github/workflows/release.yml')
            for name in names:
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text('original')
            bundle = root / 'bundle.txt'
            bundle.write_text('pinned docs')
            css = root / 'css'
            css.mkdir()
            (css / 'button.css').write_text('pinned style')
            with patch.object(report, 'source_snapshot', side_effect=lambda root: {'files': {}}), \
                 patch.object(report, 'resolve', return_value=str(bundle)), \
                 patch.object(report, 'css_cache', return_value=True), \
                 patch.object(report, 'CSS_CACHE', str(css)):
                previous = report.inputs(root)
                for name in (*names, 'bundle.txt', 'css/button.css'):
                    (root / name).write_text('changed')
                    current = report.inputs(root)
                    self.assertNotEqual(previous, current, name)
                    previous = current


class DemoSourceTests(unittest.TestCase):
    URL = ('https://raw.githubusercontent.com/heroui-inc/heroui/v3.2.6'
           '/apps/docs/src/demos/en/link/render-function.tsx')

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        root = Path(self.temp.name)
        self.archive = root / 'heroui-demos-v3.2.6.tar.gz'
        self.cache = root / 'cache'
        self.env = patch.multiple(
            demo_audit,
            DEMO_ARCHIVE=str(self.archive),
            PREVIEW_CACHE=str(self.cache),
        )
        self.env.start()
        self.addCleanup(self.env.stop)

    def member_name(self, url=None):
        return hashlib.sha256(demo_audit.pinned_source_url(url or self.URL)
                              .encode('utf-8')).hexdigest() + '.txt'

    def write_archive(self, body='export function RenderFunction() {}\n', extra=None):
        members = {self.member_name(): body}
        if extra:
            members.update(extra)
        with tarfile.open(self.archive, 'w:gz') as archive:
            for name, text in members.items():
                data = text.encode('utf-8')
                info = tarfile.TarInfo(name)
                info.size = len(data)
                archive.addfile(info, io.BytesIO(data))

    def test_cold_cache_reads_the_pin_without_opening_a_socket(self):
        self.write_archive('pinned render-function\n')
        with patch.object(demo_audit.urllib.request, 'urlopen') as opener:
            text = demo_audit.fetch_text(self.URL)
        self.assertEqual(text, 'pinned render-function\n')
        opener.assert_not_called()

    def test_missing_member_fails_without_opening_a_socket(self):
        self.write_archive()
        missing = self.URL.replace('render-function', 'does-not-exist')
        with patch.object(demo_audit.urllib.request, 'urlopen') as opener:
            with self.assertRaisesRegex(RuntimeError, 'not in the pinned demo archive'):
                demo_audit.fetch_text(missing)
        opener.assert_not_called()

    def test_missing_archive_fails_without_opening_a_socket(self):
        with patch.object(demo_audit.urllib.request, 'urlopen') as opener:
            with self.assertRaisesRegex(RuntimeError, 'pinned demo archive is missing'):
                demo_audit.fetch_text(self.URL)
        opener.assert_not_called()

    def test_refresh_retries_a_connection_reset_then_succeeds(self):
        calls = {'n': 0}

        def opener(url, timeout=20):
            calls['n'] += 1
            if calls['n'] < 3:
                raise OSError(104, 'Connection reset by peer')
            return io.BytesIO(b'recovered from reset\n')

        with patch.object(demo_audit.urllib.request, 'urlopen', side_effect=opener), \
             patch.object(demo_audit.time, 'sleep') as sleeper:
            text = demo_audit.fetch_text(self.URL, refresh=True)
        self.assertEqual(text, 'recovered from reset\n')
        self.assertEqual(calls['n'], 3)
        self.assertEqual(sleeper.call_count, 2)
        self.assertEqual((self.cache / self.member_name()).read_text(encoding='utf-8'),
                         'recovered from reset\n')

    def test_unexpected_archive_member_is_rejected(self):
        with tarfile.open(self.archive, 'w:gz') as archive:
            data = b'not a cache key'
            info = tarfile.TarInfo('../escape.txt')
            info.size = len(data)
            archive.addfile(info, io.BytesIO(data))
        with self.assertRaisesRegex(RuntimeError, 'unexpected demo archive member'):
            demo_audit.demo_cache()


if __name__ == '__main__':
    unittest.main()
