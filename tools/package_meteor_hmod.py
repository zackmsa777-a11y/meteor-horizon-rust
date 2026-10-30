#!/usr/bin/env python3
"""Build the fork's source-only native .hmod against an original GitHub source ZIP."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import zipfile


def digest(data):
    return hashlib.sha256(data).hexdigest()


def allowed(name):
    if name in ("Cargo.toml", "Cargo.lock"):
        return True
    parts = name.split("/")
    return (len(parts) >= 3 and parts[0] == "crates"
            and parts[1] in ("meteor-client", "horizon-world", "horizon-client", "horizon-client-core", "horizon-client-input", "horizon-client-render")
            and name.endswith((".rs", ".toml")))


def build(root, base_zip, meteor_license, output):
    with zipfile.ZipFile(base_zip) as original:
        revision = original.comment.decode("ascii").strip()
        if not re.fullmatch("[0-9a-f]{40}", revision):
            raise ValueError("Original GitHub source ZIP must carry its exact commit in the ZIP comment")
        roots = {name.split("/")[0] for name in original.namelist()}
        if len(roots) != 1:
            raise ValueError("Expected a single original source root")
        prefix = next(iter(roots)) + "/"
        baseline = {name[len(prefix):]: original.read(name) for name in original.namelist()
                    if not name.endswith("/") and allowed(name[len(prefix):])}
    current = {}
    for path in root.joinpath("crates").glob("**/*"):
        if path.is_file() and not path.is_symlink():
            name = path.relative_to(root).as_posix()
            if allowed(name):
                current[name] = path.read_bytes()
    for name in ("Cargo.toml", "Cargo.lock"):
        current[name] = root.joinpath(name).read_bytes()
    removed = baseline.keys() - current.keys()
    if removed:
        raise ValueError("Schema 1 cannot remove source files: " + ", ".join(sorted(removed)))
    changes, entries = [], {}
    for name, data in sorted(current.items()):
        before = baseline.get(name)
        if data == before:
            continue
        changes.append({"path": name, "before": digest(before) if before is not None else None, "after": digest(data)})
        entries["source/" + name] = data
    if not changes:
        raise ValueError("No source changes found")
    manifest = {"schema": 1, "kind": "horizon-native-source", "id": "meteor-client",
                "version": "0.1.0-dev", "game": "26.3", "horizon": revision,
                "features": ["horizon-client/meteor"], "files": changes}
    entries["horizon-native-source.json"] = (json.dumps(manifest, indent=2) + "\n").encode()
    entries["notices/Meteor-GPL-3.0.txt"] = meteor_license.read_bytes()
    entries["notices/Horizon-license.txt"] = root.joinpath("LICENSE").read_bytes()
    starscript_license = root / "crates/meteor-client/STARSCRIPT-LICENSE.txt"
    if starscript_license.is_file():
        entries["notices/Starscript-MIT.txt"] = starscript_license.read_bytes()
    entries["notices/README.txt"] = (
        "Meteor native Rust port, development build. Requires the modified Horizon launcher\n"
        "with native-source-hmod/1 support; stock content-only .hmod loaders reject it.\n"
        "Native code has full computer access. Review the source and explicitly enable the\n"
        "launcher's native-code trust option before compiling. Includes source only, no game\n"
        "assets, accounts, worlds, executables or JARs. The original base is pinned by commit\n"
        "and every replaced source file is checked against its original SHA-256.\n"
        "Meteor-derived source is distributed under GPL-3.0; see the included license.\n"
    ).encode()
    if len(entries) > 2048 or sum(map(len, entries.values())) > 64 * 1024 * 1024:
        raise ValueError("Package exceeds native-source format limits")
    output.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as package:
        for name, data in sorted(entries.items()):
            info = zipfile.ZipInfo(name, date_time=(2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            package.writestr(info, data)
    print(json.dumps({"path": str(output.resolve()), "sha256": digest(output.read_bytes()),
                      "base": revision, "changed_source_files": len(changes), "bytes": output.stat().st_size}, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base-zip", type=Path, required=True)
    parser.add_argument("--meteor-license", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--source-root", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    build(args.source_root.resolve(), args.base_zip, args.meteor_license, args.output)
