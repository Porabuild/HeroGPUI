"""Pin and cache isolation regressions; no network or shared-cache mutations."""
import importlib.util
import io
import os
from pathlib import Path
import shutil
import tarfile
import tempfile
import unittest
from unittest.mock import patch


class BundleTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        shutil.copyfile(Path(__file__).with_name('bundle.py'), self.root / 'bundle.py')
        self.env = patch.dict(os.environ, {'TEMP': str(self.root / 'cache')})
        self.env.start()
        self.addCleanup(self.env.stop)

    def archive(self, body):
        with tarfile.open(self.root / 'heroui-css-v3.2.5.tar.gz', 'w:gz') as archive:
            entry = tarfile.TarInfo('tag.css')
            entry.size = len(body)
            archive.addfile(entry, io.BytesIO(body))

    def load(self):
        spec = importlib.util.spec_from_file_location('fixture_bundle', self.root / 'bundle.py')
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return module

    def test_old_cache_and_changed_archive_cannot_supply_styles(self):
        stale = self.root / 'cache' / 'heroui-css'
        stale.mkdir(parents=True)
        (stale / 'tag.css').write_bytes(b'old release')
        self.archive(b'new release')
        first = self.load()
        self.assertTrue(first.css_cache())
        self.assertEqual((Path(first.CSS_CACHE) / 'tag.css').read_bytes(), b'new release')
        self.archive(b'corrected archive')
        second = self.load()
        self.assertNotEqual(first.CSS_CACHE, second.CSS_CACHE)
        self.assertTrue(second.css_cache())
        self.assertEqual((Path(second.CSS_CACHE) / 'tag.css').read_bytes(), b'corrected archive')
        self.assertEqual((stale / 'tag.css').read_bytes(), b'old release')

    def test_missing_archive_is_not_a_cache_hit(self):
        module = self.load()
        self.assertFalse(module.css_cache())

    def test_bundle_rejects_previous_release(self):
        module = self.load()
        path = self.root / 'docs.txt'
        with patch.dict(os.environ):
            os.environ.pop('HEROUI_BUNDLE_UNPINNED', None)
            for release in ('v3.2.4', 'v3.2.5'):
                path.write_text('## Latest Release\n\n### ' + release)
                if release == module.PINNED_RELEASE:
                    self.assertEqual(module._verify(str(path)), str(path))
                else:
                    with self.assertRaises(SystemExit) as error:
                        module._verify(str(path))
                    self.assertEqual(error.exception.code, 2)


if __name__ == '__main__':
    unittest.main()
