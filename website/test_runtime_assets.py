"""Integrity checks for the release overlay used by both static hosts."""
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import runtime_assets as runtime


class RuntimeAssetsTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.name = 'catalog/data/meshes/test.igmpz'
        self.payload = b'verified packet'
        self.item = {'url': 'https://github.com/zerobytes/ignis-catalog/releases/download/v1/test.igmpz',
                     'bytes': len(self.payload), 'sha256': hashlib.sha256(self.payload).hexdigest()}
        self.path = self.root / self.name
        self.path.parent.mkdir(parents=True)
        self.manifest = self.root / runtime.MANIFEST
        self.write()

    def write(self):
        self.manifest.write_text(json.dumps({'version': 1, 'files': {self.name: self.item}}))

    def test_verified_cache_needs_no_network(self):
        self.path.write_bytes(self.payload)
        with patch('runtime_assets.subprocess.run') as fetch:
            runtime.materialize(self.root)
            fetch.assert_not_called()

    def test_bad_download_preserves_existing_file(self):
        self.path.write_bytes(b'previous')
        def corrupt(args, **kwargs):
            Path(args[-1]).write_bytes(b'tampered')
        with patch('runtime_assets.subprocess.run', side_effect=corrupt):
            with self.assertRaisesRegex(RuntimeError, 'hash/size mismatch'):
                runtime.materialize(self.root)
        self.assertEqual(self.path.read_bytes(), b'previous')
        self.assertFalse(self.path.with_name('.' + self.path.name + '.part').exists())

    def test_escaping_path_rejected(self):
        self.name = 'catalog/data/meshes/../../escape.igmpz'
        self.write()
        with self.assertRaisesRegex(RuntimeError, 'declaration'):
            runtime.declarations(self.root)

    def test_oversized_preview_rejected(self):
        self.item['bytes'] = 65 * 1024 * 1024
        self.write()
        with self.assertRaisesRegex(RuntimeError, 'declaration'):
            runtime.declarations(self.root)

    def test_symlink_rejected_before_network(self):
        outside = self.root / 'outside'
        outside.write_bytes(self.payload)
        self.path.symlink_to(outside)
        with self.assertRaisesRegex(RuntimeError, 'symlink'):
            runtime.materialize(self.root)


if __name__ == '__main__':
    unittest.main()
