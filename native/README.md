# Standard native Meteor migration proof

This directory develops a normal Horizon `.hnmod` using horizon-mixins. It is a
**Zoom-only migration proof**, not a working release of the complete Meteor port.
The retained core contains 30 module policies; the other policies are not connected
to this native adapter. HUD, custom fonts, the complete GUI and the other host
integrations remain migration work.

The bundle uses the existing `horizon-native.json` schema 1, package ID
`meteor_client`, and version `0.1.0-migration.1`. It pins:

- Horizon: `b76ae35f5b9b2d82c0e6818fc0ef93bb5277ec73`.
- horizon-mixins: `cb1ad23022392d88f006cf94cbf6552a73f5df12` (0.6.0).
- Meteor reference: `e3a2e77967629d216e127b5ceecf71fb15f9d2e5`.

This follows Horizon's existing native-mod build path: Mixins adds the bundle's
crate and hooks to an isolated source stage, and Cargo compiles that stage. It
does not use the legacy `horizon-native-source.json` package or a custom loader.
Native code requires the normal explicit trust decision. Minecraft resources,
worlds, accounts, executables and prepared game files are not included.

## Source and current scope

`bundle/mixins/meteor_client/core/` retains the existing Meteor Rust core, including
its tests and source notices. Its copied source bytes are unchanged; the sole
Cargo change replaces inherited edition with explicit edition `2024`. Per-file
original and copied hashes are recorded in
[`core-provenance.json`](bundle/mixins/meteor_client/data/core-provenance.json).

The adapter uses eight named hooks: frame, tick, wheel, command, zoom, mods,
settings and settings-click. A module-local window table keeps proof state out of
Horizon structs. The command is `/meteor-migration`; its status/help and Options >
Mods describe the limited proof. Its separate `zoom-migration.json` settings do not
import or overwrite the legacy prototype's configuration.

Horizon's existing Zoom API clamps factors to 10 and turn scaling to 0.05–1. The
proof does not yet reproduce every Meteor Zoom setting, including cinematic and
HUD/hand visibility behavior. Keeping the core policy is not evidence that every
host operation has been ported.

## Verification

[`evidence/verification.json`](evidence/verification.json) records the exact source
snapshot and scoped checks. Mixins plan and apply succeeded against pristine
Horizon with eight named hooks, one support module and one bundle crate. The
retained core test suite and four exact-source window lifecycle tests passed.
The corrected R2 stage passed a full `horizon-client` binary `cargo check` in
22.60 seconds. Seven packaging checks passed, and the final proof archive passed
the stock native parser. The current runtime, all eight helpers, 70 core payloads
and four data payloads match that R2 stage exactly. Game runtime is **not tested**.

The local source proof archive is `meteor-client-zoom-migration-proof.hnmod`,
SHA-256 `4c8d2f238f8c9c184f96228f6c966d9a3945b07dffe65169fe438cf59fe70de3`.
It is not a complete or runtime-verified Meteor release.

The stock parser source used for validation was compared byte for byte with the
original pinned Horizon ZIP. Its ordinary content parser rejects this native
archive. Passing package checks or planning does not prove gameplay behavior.

## Reproduce

Use Python 3.11 or later and the pinned Rust/Mixins versions. Obtain your own
pristine Horizon source export and keep it separate from scratch build outputs.
Replace `PRISTINE_HORIZON` and `SCRATCH` below with those paths.

```sh
python native/tools/test_package_standard_hnmod.py
python native/tools/package_standard_hnmod.py --output dist/meteor-client-zoom-proof.hnmod --report dist/package-report.json
horizon-mixins plan --workspace PRISTINE_HORIZON --mods native/bundle/mixins --out SCRATCH/plan.json
horizon-mixins apply --workspace PRISTINE_HORIZON --mods native/bundle/mixins --out SCRATCH/staged-horizon
cargo test --manifest-path SCRATCH/staged-horizon/Cargo.toml --locked -p meteor-client
cargo check --manifest-path SCRATCH/staged-horizon/Cargo.toml --locked -p horizon-client --bin horizon-client
```

The package contains Rust/TOML source and license/provenance data. The packager
does not compile or run it. This pinned Horizon base has no `mods` Cargo feature;
the migration proof does not require a custom `meteor` feature.

To build the small stock-parser validator, supply your own pristine source and a
separate target directory. The helper creates a temporary Cargo manifest with a
quoted source path; no manual manifest edits or machine-specific paths are needed.

```sh
python native/tools/build_standard_validator.py --horizon-source PRISTINE_HORIZON --target-dir SCRATCH/validator-target
python native/tools/validate_standard_hnmod.py --package dist/meteor-client-zoom-proof.hnmod --validator SCRATCH/validator-target/debug/meteor-standard-hnmod-validator --pristine PRISTINE_HORIZON --base-zip ORIGINAL_HORIZON_ZIP --report dist/validator-report.json
```

On Windows, append `.exe` to the validator filename. Dependency downloads may be
needed on the first validator build; `--offline` requires an existing Cargo cache.
For a pinned toolchain, set absolute `RUSTC`/`RUSTDOC` and pass its Cargo executable
with `--cargo`; this avoids dependency-local rustup overrides changing compilers.

When runtime validation is ready, the ordinary launcher accepts `native/bundle`
as an unpacked native bundle, using its existing native-build trust flow and the
player's own prepared base version. No ready-to-play `.hnmod` release is claimed
here. Automated game tests use fresh isolated homes and generated singleplayer
fixtures; existing worlds, accounts and multiplayer are excluded.

## Licenses

Meteor-derived code is GPL-3.0-only. Preserve the
[Meteor GPL notice](bundle/mixins/meteor_client/data/licenses/Meteor-GPL-3.0-only.txt),
[Starscript MIT notice](bundle/mixins/meteor_client/data/licenses/Starscript-MIT.txt)
and [Horizon MIT notice](bundle/mixins/meteor_client/data/licenses/Horizon-MIT.txt).
This is an unofficial, AI-assisted port. The legacy prototype remains separate
from this migration proof.
