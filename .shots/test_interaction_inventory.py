"""Evidence freshness regressions, using isolated source/artifact fixtures."""
from copy import deepcopy
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import interaction_inventory as inventory

SOURCE_SNAPSHOT = inventory.source_snapshot


class InventoryTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.examples = {'button': [{'heading': 'Usage', 'code': 'Button::new("a")', 'description': 'One'}]}
        for path, data in ((inventory.EXAMPLES, self.examples),
                           (inventory.REFERENCE, {'button': {'version': '3.2.5'}}),
                           (inventory.CATALOG, {'components': {'button': {}}})):
            self.write_json(path, data)
        self.snapshot = {'sha256': 'initial', 'files': {'button.rs': 'first'}}
        self.mock = patch.object(inventory, 'source_snapshot', side_effect=lambda root: deepcopy(self.snapshot))
        self.mock.start()
        self.addCleanup(self.mock.stop)
        self.old = {'schema_version': 1, 'gallery_sections': {}, 'upstream': {'target_tag': 'v3.2.5'}}

    def write_json(self, path, data):
        path = self.root / path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(data))

    def current(self):
        return inventory.refresh_gallery(self.old, self.root)

    def test_hashed_driver_files_have_identical_checkout_bytes(self):
        checkout = self.root / 'checkout'
        checkout.mkdir()
        attributes = Path(__file__).resolve().parents[1] / '.gitattributes'
        (checkout / '.gitattributes').write_bytes(attributes.read_bytes())
        (checkout / 'control.ps1').write_bytes(b'Write-Host "control"\r\n')
        (checkout / 'run-tests.sh').write_bytes(b'#!/bin/bash\nexit 0\n')

        def git(*args):
            return subprocess.run(['git', '-C', str(checkout), *args], check=True,
                                  stdout=subprocess.PIPE, stderr=subprocess.PIPE).stdout

        git('init', '-q')
        git('-c', 'core.autocrlf=false', 'add', '.')
        for path in ('.gitattributes', 'control.ps1', 'run-tests.sh'):
            with self.subTest(path=path):
                unix = git('-c', 'core.autocrlf=false', 'cat-file', '--filters', ':' + path)
                windows = git('-c', 'core.autocrlf=true', 'cat-file', '--filters', ':' + path)
                self.assertEqual(unix, windows)
                self.assertEqual(unix, (checkout / path).read_bytes())

    def test_refresh_is_idempotent_and_keeps_reviewed_sections(self):
        self.old = self.current()
        self.old['gallery_sections']['button'][0]['status'] = 'specified'
        self.assertEqual(self.current(), self.old)

    def test_refresh_seeds_each_gallery_section_as_unreviewed_specimen(self):
        current = self.current()
        self.assertEqual(len(current['specimens']), 1)
        specimen = current['specimens'][0]
        self.assertEqual(specimen['id'], 'gallery/button/usage')
        self.assertEqual(specimen['status'], 'unreviewed')
        self.assertEqual(specimen['scope'], 'gallery-section')
        self.assertEqual(specimen['port']['gallerySection'], 'Usage')
        self.assertIsNone(specimen['upstream']['source'])
        self.assertEqual(specimen['evidence'], {
            'upstream': [], 'native': [], 'wasm': [], 'tests': [],
        })

    def test_manual_specimens_survive_gallery_refresh(self):
        self.old = self.current()
        manual = {'id': 'button/primary/keyboard-focus', 'status': 'specified'}
        self.old['specimens'].insert(0, manual)
        current = self.current()
        self.assertEqual(current['specimens'][0], manual)
        self.assertEqual(len(current['specimens']), 2)

    def test_description_or_import_changes_invalidate_even_with_same_code(self):
        self.old = self.current()
        self.old['gallery_sections']['button'][0]['status'] = 'verified'
        for key, value in [('description', 'Changed'), ('imports', 'use herogpui::Button;')]:
            rows = deepcopy(self.examples)
            rows['button'][0][key] = value
            self.write_json(inventory.EXAMPLES, rows)
            row = self.current()['gallery_sections']['button'][0]
            self.assertEqual(row['status'], 'unreviewed')
            self.assertEqual(row['history'][-1]['status'], 'verified')
            self.assertEqual(row['code_sha256'], self.old['gallery_sections']['button'][0]['code_sha256'])

    def test_source_change_invalidates_specimen_and_retains_evidence(self):
        self.old = self.current()
        self.old['specimens'] = [{'id': 'button/light', 'status': 'verified', 'evidence': {'native': ['retained']}}]
        self.snapshot['sha256'] = 'changed'
        current = self.current()
        self.assertEqual(current['specimens'][0]['status'], 'implemented-unverified')
        self.assertEqual(current['specimens'][0]['review_history'][0]['status'], 'verified')
        self.assertEqual(current['specimens'][0]['evidence']['native'], ['retained'])
        self.assertEqual(inventory.refresh_gallery(current, self.root), current)

    def test_removed_sections_are_retained_once_as_history(self):
        self.old = self.current()
        self.write_json(inventory.EXAMPLES, {'button': [{'heading': 'Replacement', 'code': 'new'}]})
        current = self.current()
        self.assertEqual(current['retired_gallery_sections']['button'][0]['heading'], 'Usage')
        self.assertEqual(inventory.refresh_gallery(current, self.root), current)

    def test_empty_missing_and_duplicate_sections_fail(self):
        for data in ({}, {'other': self.examples['button']}, {'button': []},
                     {'button': self.examples['button'] * 2}):
            self.write_json(inventory.EXAMPLES, data)
            with self.assertRaises(ValueError):
                self.current()

    def test_wrong_reference_version_fails(self):
        self.write_json(inventory.REFERENCE, {'button': {'version': '3.2.4'}})
        with self.assertRaisesRegex(ValueError, 'version'):
            self.current()

    def test_verified_requires_current_files_for_all_surfaces(self):
        current = self.current()
        evidence_file = self.root / 'capture.png'
        evidence_file.write_bytes(b'fixture evidence')
        record = {'path': 'capture.png', 'sha256': inventory.digest(evidence_file.read_bytes())}
        specimen = {'id': 'button/light', 'status': 'verified', 'verified_snapshot': current['verification_sha256'],
                    'evidence': {name: [record] for name in ('upstream', 'native', 'wasm', 'tests')}}
        current['specimens'] = [specimen]
        inventory.validate_specimens(current, self.root)
        for surface in specimen['evidence']:
            broken = deepcopy(current)
            broken['specimens'][0]['evidence'][surface] = []
            with self.assertRaisesRegex(ValueError, 'missing'):
                inventory.validate_specimens(broken, self.root)
        evidence_file.write_bytes(b'changed')
        with self.assertRaisesRegex(ValueError, 'stale'):
            inventory.validate_specimens(current, self.root)

    def test_duplicate_unknown_status_and_wrong_snapshot_fail(self):
        current = self.current()
        for rows in ([{'id': 'same', 'status': 'specified'}] * 2,
                     [{'id': 'a', 'status': 'looks-fine'}],
                     [{'id': 'a', 'status': 'verified', 'verified_snapshot': 'wrong'}]):
            current['specimens'] = rows
            with self.assertRaises(ValueError):
                inventory.validate_specimens(current, self.root)

    def test_real_snapshot_includes_gallery_manifest_preview_host_and_assets(self):
        for name in inventory.INPUT_FILES:
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text('fixture')
        paths = ('crates/herogpui-components/src/button.rs',
                 'web/src/components/preview/gallery-frame.tsx',
                 'gallery/assets/icon.svg', 'crates/herogpui-web/fonts/font.ttf',
                 'gallery/Cargo.toml', '.shots/control.ps1', '.shots/run-tests.sh',
                 '.shots/parity_report.py')
        for name in paths:
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text('original')
        previous = SOURCE_SNAPSHOT(self.root)
        for name in paths:
            (self.root / name).write_text('modified')
            current = SOURCE_SNAPSHOT(self.root)
            self.assertNotEqual(current['sha256'], previous['sha256'], name)
            previous = current

    def test_write_failure_preserves_original_inventory(self):
        path = self.root / 'evidence.json'
        path.write_bytes(b'{"original": true}')
        with patch.object(inventory.os, 'fsync', side_effect=OSError('simulated full disk')):
            with self.assertRaises(OSError):
                inventory.write_json_atomic(path, {'new': 'content'})
        self.assertEqual(path.read_bytes(), b'{"original": true}')
        self.assertEqual(list(self.root.glob('.evidence.json.*')), [])
        inventory.write_json_atomic(path, {'new': 'content'})
        self.assertEqual(inventory.read_json(path), {'new': 'content'})

    def upstream_fixture(self):
        directories = [self.root / name for name in ('target', 'baseline')]
        for directory, version in zip(directories, ('3.2.5', '3.2.4')):
            package = directory / 'packages/react/package.json'
            package.parent.mkdir(parents=True)
            package.write_text(json.dumps({'version': version}))
            source = directory / 'packages/react/src/button.tsx'
            source.parent.mkdir(parents=True)
            source.write_text(version)
        target, baseline = directories
        demo = target / 'apps/docs/src/demos/en/button/basic.tsx'
        demo.parent.mkdir(parents=True)
        demo.write_text('default example')
        current = self.current()
        current['upstream']['baseline_tag'] = 'v3.2.4'
        current['source_change_roots'] = ['packages/react/src']
        current['source_changes'] = []
        current['upstream_demos'] = {}
        return inventory.refresh_upstream(current, target, baseline), target, baseline

    def test_upstream_change_invalidates_review_even_with_unchanged_local_sources(self):
        current, target, baseline = self.upstream_fixture()
        current['specimens'] = [{'id': 'button/light', 'status': 'verified',
                                'verified_snapshot': current['verification_sha256'], 'evidence': {}}]
        original = current['verification_sha256']
        (target / 'packages/react/src/button.tsx').write_text('changed upstream')
        changed = inventory.refresh_upstream(current, target, baseline)
        self.assertNotEqual(changed['verification_sha256'], original)
        self.assertEqual(changed['specimens'][0]['status'], 'implemented-unverified')
        self.assertEqual(changed['specimens'][0]['review_history'][0]['status'], 'verified')
        inventory.validate_specimens(changed, self.root)

    def test_removed_upstream_records_keep_review_history_once(self):
        current, target, baseline = self.upstream_fixture()
        current['source_changes'][0]['evidence'] = 'source review'
        current['upstream_demos']['button'][0]['evidence'] = 'demo review'
        (target / 'packages/react/src/button.tsx').write_text('3.2.4')
        (target / 'apps/docs/src/demos/en/button/basic.tsx').unlink()
        (target / 'apps/docs/src/demos/en/button/replacement.tsx').write_text('replacement')
        changed = inventory.refresh_upstream(current, target, baseline)
        self.assertEqual(changed['retired_source_changes'][0]['evidence'], 'source review')
        self.assertEqual(changed['retired_upstream_demos']['button'][0]['evidence'], 'demo review')
        self.assertEqual(inventory.refresh_upstream(changed, target, baseline), changed)


if __name__ == '__main__':
    unittest.main()
