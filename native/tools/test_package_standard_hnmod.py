import json
from pathlib import Path
import shutil
import tempfile
import unittest
import zipfile

import package_standard_hnmod as package


class StandardPackageTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='meteor-hnmod-proof-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name) / 'bundle'
        shutil.copytree(Path(__file__).resolve().parents[1] / 'bundle', self.root)
        self.output = Path(self.temp.name) / 'meteor.hnmod'

    def test_deterministic_normal_contract_and_provenance(self):
        first = package.build(self.root, self.output)
        raw = self.output.read_bytes()
        self.assertEqual(first, package.build(self.root, self.output))
        self.assertEqual(raw, self.output.read_bytes())
        with zipfile.ZipFile(self.output) as archive:
            self.assertIn('horizon-native.json', archive.namelist())
            self.assertNotIn('horizon-native-source.json', archive.namelist())
            manifest = json.loads(archive.read('horizon-native.json'))
            self.assertEqual(manifest['id'], 'meteor_client')
            self.assertNotIn('hooks', manifest)
            self.assertIn(b'edition = "2024"', archive.read('mixins/meteor_client/core/Cargo.toml'))

    def test_custom_native_source_contract_is_rejected(self):
        path = self.root / 'horizon-native-source.json'
        path.write_text('{"schema":2,"kind":"horizon-native-source"}')
        with self.assertRaisesRegex(ValueError, 'standard .hnmod'):
            package.collect(self.root)

    def test_binary_data_must_be_in_standard_top_level_data(self):
        path = self.root / 'mixins/meteor_client/core/data/unsupported.ttf'
        path.parent.mkdir()
        path.write_bytes(b'\0fontfixture')
        with self.assertRaisesRegex(ValueError, 'standard .hnmod'):
            package.collect(self.root)
        path.unlink()
        path = self.root / 'mixins/meteor_client/data/synthetic-font-fixture.ttf'
        path.write_bytes(b'\0fontfixture')
        self.assertIn(path.relative_to(self.root).as_posix(), package.collect(self.root))

    def test_changed_copied_core_is_rejected_by_provenance(self):
        path = self.root / 'mixins/meteor_client/core/src/lib.rs'
        path.write_bytes(path.read_bytes() + b'\n// changed\n')
        with self.assertRaisesRegex(ValueError, 'provenance'):
            package.collect(self.root)

    def test_metadata_hooks_and_post_build_are_not_silently_added(self):
        path = self.root / 'horizon-native.json'
        original = path.read_bytes()
        manifest = json.loads(original)
        manifest['hooks'] = 2
        path.write_text(json.dumps(manifest))
        with self.assertRaisesRegex(ValueError, 'schema-1 contract'):
            package.collect(self.root)
        path.write_bytes(original)
        mixin = self.root / 'mixins/meteor_client/horizon-mixin.toml'
        mixin.write_bytes(mixin.read_bytes() + b'\n[[post_build]]\ncrate="core"\nbin="unexpected"\n')
        with self.assertRaisesRegex(ValueError, 'omit post_build'):
            package.collect(self.root)

    def test_missing_notice_is_rejected(self):
        (self.root / 'mixins/meteor_client/data/licenses/Meteor-GPL-3.0-only.txt').unlink()
        with self.assertRaisesRegex(ValueError, 'Missing preserved notice'):
            package.collect(self.root)

    def test_windows_alias_and_traversal_names_are_rejected(self):
        for path in ('../outside.rs', '/absolute.rs', 'mixins/CON.rs',
                     'mixins/a:stream.rs', 'mixins/COM¹.rs', 'mixins/trailing.rs.'):
            self.assertFalse(package.safe_path(path), path)


if __name__ == '__main__':
    unittest.main()
