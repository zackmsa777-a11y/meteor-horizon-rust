# Native source `.hmod` in the Meteor development fork

This fork additionally supports **native-source-hmod/1**, a source-only ZIP with a
`horizon-native-source.json` manifest. This is executable native source, requiring
the same explicit trust as `.hnmod`. It is **not compatible with stock Horizon's
content-only `.hmod` loader**. Content loading and its no-code checks are unchanged.
The stock registry and browser upload/install contract are also unchanged; this
format currently uses the modified launcher's local native-build path.

The modified launcher accepts a package directly:

```sh
horizon-launcher native-build meteor-client-native.hmod --trust-native-code
```

Use a prepared base version, optionally selected with `--version`. This compiles
an isolated game version and selects its Native mods instance. It does not launch
the game. Review the source before using the native-code trust flag. Building
uses the launcher's portable Rust toolchain and can download dependencies.
Set `HORIZON_NATIVE_BUILD_HOME` to a spacious directory to store the portable Rust,
Cargo registry and native build caches separately from installed versions and
accounts. For example, PowerShell:

```powershell
$env:HORIZON_NATIVE_BUILD_HOME = 'D:\Codex-Meteor-Build\native-build-cache'
```

Leaving this variable unset preserves the launcher's default storage locations.
Do not place this package in the content `horizon-mods` directory. The modified
launcher can also collect it from `horizon-native-mods`; only one source package
may participate in a build, and combining it with mixin mods is rejected.

`cargo horizon-mod check meteor-client-native.hmod` recognizes and checks this
format in this fork. It hashes and validates source without compiling or executing
it. Explicitly checking it as a content mod still fails.

For development verification without running native code, use
`cargo horizon-mod stage-native-source package.hmod pristine-base fresh-output`.
The base must contain its exact commit in `.horizon-revision`; the output must
not exist. This exercises the same hash checks and staging code as the launcher.

## Contract

The manifest has `schema: 1`, `kind: "horizon-native-source"`, an ID, version,
game version, exact 40-character Horizon commit, Cargo feature list, and `files`.
Native-source IDs allow lowercase ASCII letters, digits, underscores and hyphens.
Meteor keeps its existing runtime ID `meteor-client` for enable/disable state,
installer metadata and saved settings; this does not change stock registry IDs.
Each change declares a relative path, SHA-256 `before` (null for an addition),
and SHA-256 `after`. Its bytes live at `source/<path>`. License notices live under
`notices/` as text. All other entries are rejected. Limits are 2,048 entries and
64 MiB of uncompressed source. Absolute paths, traversal, Windows aliases,
duplicate case-insensitive names and symbolic links are rejected.

Schema 1 permits root Cargo manifests/lockfile and Rust/TOML source under the
Meteor, Horizon client and Horizon world crates. It does not permit replacing
the launcher or package loader. These restrictions do not sandbox Rust source:
Cargo manifests and compiled code remain fully trusted native code.

Before staging, the loader checks the base commit marker and all affected base
file hashes. Additions must not exist. It copies the pristine source into fresh
isolated staging, applies validated files, then uses the existing native compiler
and version installer. Package bytes, pinned Rust version and build profile
determine the isolated version key. Existing worlds and original source are not
modified by staging. Packages cannot delete source files in schema 1.

## Rebuild the package

```sh
python tools/package_meteor_hmod.py --base-zip horizon-main.zip --meteor-license /path/to/meteor-client/LICENSE --output meteor-client-native.hmod
```

The original GitHub ZIP comment provides the exact baseline commit. The tool
compares all allowed source files against that ZIP on every run, including new
Meteor source modules, so the package tracks the current port. It includes the
Meteor GPL-3.0 and Horizon license notices. No JARs, binaries, accounts, game
assets, caches or worlds are included. Distribute the modified loader's source
alongside the package; a stock launcher cannot install this fork-specific format.
