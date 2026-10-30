import json
from pathlib import Path
import shutil
import tempfile
import unittest
import zipfile

import audit_native_bundle as audit
import package_native_hnmod as package


class FoundationPackageTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='meteor-foundation-package-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name) / 'bundle'
        shutil.copytree(Path(__file__).resolve().parents[1] / 'bundle', self.root)
        provenance = json.loads((self.root / package.PROVENANCE).read_text())
        if provenance.get('status') != 'frozen-source-awaiting-validation':
            notes = {p.relative_to(self.root).as_posix(): 'Temporary unit fixture only; not a production validation claim'
                     for p in self.root.rglob('*.rs')}
            audit.freeze(self.root, notes)

    def test_normal_deterministic_contract(self):
        one, two = Path(self.temp.name) / 'one.hnmod', Path(self.temp.name) / 'two.hnmod'
        package.build(self.root, one)
        package.build(self.root, two)
        self.assertEqual(one.read_bytes(), two.read_bytes())
        with zipfile.ZipFile(one) as archive:
            self.assertEqual(json.loads(archive.read('horizon-native.json')), package.EXPECTED_MANIFEST)
            self.assertEqual(package.VERSION, '0.1.0-native.1')
            self.assertNotIn('horizon-native-source.json', archive.namelist())
            self.assertIn(package.PREFIX + 'data/licenses/THIRD_PARTY_NOTICES.txt', archive.namelist())

    def test_requires_explicit_freeze_and_rejects_changed_source(self):
        provenance = self.root / package.PROVENANCE
        raw = provenance.read_bytes()
        value = json.loads(raw)
        value['status'] = 'initial copy'
        provenance.write_text(json.dumps(value))
        with self.assertRaisesRegex(ValueError, 'freeze'):
            package.collect(self.root)
        provenance.write_bytes(raw)
        rust = self.root / package.PREFIX / 'runtime.rs'
        rust.write_bytes(rust.read_bytes() + b'\n// changed after freeze\n')
        with self.assertRaisesRegex(ValueError, 'Frozen payload changed'):
            package.collect(self.root)

    def test_rejects_unlisted_source_and_binary_data(self):
        for name in ('unlisted.rs', 'data/unlisted.ttf'):
            path = self.root / package.PREFIX / name
            path.write_bytes(b'unlisted')
            with self.assertRaisesRegex(ValueError, 'Unlisted'):
                package.collect(self.root)
            path.unlink()

    def test_rejects_old_pins_or_extra_native_metadata(self):
        path = self.root / 'horizon-native.json'
        original = path.read_bytes()
        for key, value in (('horizon', 'ba6344a59cb9e755b730dcf02285fc0d33b97d3c'), ('hooks', 2)):
            manifest = json.loads(original)
            manifest[key] = value
            path.write_text(json.dumps(manifest))
            with self.assertRaisesRegex(ValueError, 'exact normal schema1'):
                package.collect(self.root)

    def test_preserves_exact_notices_and_original_provenance(self):
        path = self.root / package.PREFIX / 'data/licenses/THIRD_PARTY_NOTICES.txt'
        raw = path.read_bytes()
        path.write_bytes(raw + b'\nchanged')
        with self.assertRaisesRegex(ValueError, 'preserved component notice'):
            package.collect(self.root)
        path.write_bytes(raw)
        path = self.root / package.PROVENANCE
        value = json.loads(path.read_text())
        value['original_files'][0]['sha256'] = '0' * 64
        path.write_text(json.dumps(value))
        with self.assertRaisesRegex(ValueError, 'original source hashes differ'):
            package.collect(self.root)

    def test_rejects_custom_build_actions_and_unsafe_paths(self):
        for path in ('../outside.rs', '/absolute.rs', 'mixins/CON.rs', 'mixins/a:stream.rs',
                     'mixins/COM¹.rs', 'mixins/trailing.rs.'):
            self.assertFalse(package.safe_path(path), path)
        mixin = self.root / package.PREFIX / 'horizon-mixin.toml'
        mixin.write_bytes(mixin.read_bytes() + b'\n[[post_build]]\ncrate="x"\nbin="x"\n')
        with self.assertRaisesRegex(ValueError, 'no crate/dependency/post_build'):
            package.collect(self.root)

    def test_preserves_previous_archive(self):
        path = Path(self.temp.name) / 'held.hnmod'
        package.build(self.root, path)
        raw = path.read_bytes()
        with self.assertRaisesRegex(ValueError, 'Preserve existing archive'):
            package.build(self.root, path)
        self.assertEqual(path.read_bytes(), raw)


if __name__ == '__main__':
    unittest.main()
