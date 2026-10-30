"""Read/check a proof archive with a validator linked to pristine Horizon's native parser."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import zipfile


def sha(data):
    return hashlib.sha256(data).hexdigest()


def validate(package, validator, pristine, base_zip, *, expected_version=None, portable=False):
    parser_path = pristine / 'crates/horizon-mods/src/index/native.rs'
    parser_bytes = parser_path.read_bytes()
    with zipfile.ZipFile(base_zip) as original, zipfile.ZipFile(package) as archive:
        manifest = json.loads(archive.read('horizon-native.json'))
        if expected_version is not None and manifest.get('version') != expected_version:
            raise ValueError('Package differs from explicitly expected version')
        if original.comment.decode().strip() != manifest['horizon']:
            raise ValueError('Original source revision differs from package')
        prefix = next(iter({name.split('/')[0] for name in original.namelist()})) + '/'
        if parser_bytes != original.read(prefix + 'crates/horizon-mods/src/index/native.rs'):
            raise ValueError('Validator source is not the pristine stock native parser')
        if 'horizon-native-source.json' in archive.namelist():
            raise ValueError('Custom native-source package is not a standard .hnmod')
    result = subprocess.run([str(validator.resolve()), str(package.resolve())], capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError(result.stderr)
    native = json.loads(result.stdout)
    if not native['content_loader_rejected']:
        raise ValueError('Stock content parser accepted native code')
    return {
        'status': 'PASS: standard schema-1 .hnmod accepted by pristine native parser; content parser rejects it',
        'package': package.name if portable else str(package.resolve()), 'package_sha256': sha(package.read_bytes()),
        'manifest': manifest, 'base_archive_sha256': sha(base_zip.read_bytes()),
        'validator': validator.name if portable else str(validator.resolve()), 'validator_sha256': sha(validator.read_bytes()),
        'parser_source': 'crates/horizon-mods/src/index/native.rs' if portable else str(parser_path.resolve()),
        'parser_source_sha256': sha(parser_bytes),
        'parser_byte_identical_to_original_archive': True, 'native': native,
        'limits': 'Package validation only; this does not compile or execute mod source or establish gameplay parity.',
    }


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('package', 'validator', 'pristine', 'base-zip', 'report'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--expected-version', help='Reject a different version before invoking the stock validator')
    parser.add_argument('--portable-report', action='store_true', help='Use logical filenames and parser-relative path in report')
    args = parser.parse_args()
    report = validate(args.package, args.validator, args.pristine, args.base_zip,
                      expected_version=args.expected_version, portable=args.portable_report)
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(report, indent=2))
