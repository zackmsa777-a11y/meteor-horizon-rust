# Standard native Meteor migration proof

This normal Horizon `.hnmod` integrates **Zoom + AutoJump** and passed 50 isolated
runtime checks on the recorded client. It remains a partial prototype, not the
complete Meteor port. Of 30 retained core module policies, the other 28 are
inactive. HUD, custom fonts, the complete GUI and other host integrations remain
migration work.

[Download the source proof package](../packages/meteor-client-autojump-migration-proof.hnmod).
SHA-256: `b9a4fd4ffa3f0e23b6b1d9d44d88ab8788fae7c6c08bb9dff9a2724519799c40`.
The standard `horizon-native.json` schema 1 names `meteor_client`, version
`0.1.0-migration.2`, and pins:

- Horizon: `b76ae35f5b9b2d82c0e6818fc0ef93bb5277ec73`.
- horizon-mixins: `cb1ad23022392d88f006cf94cbf6552a73f5df12` (0.6.0).
- Meteor reference: `e3a2e77967629d216e127b5ceecf71fb15f9d2e5`.

## Implementation

Official Mixins stages the bundle's own crate and declared hooks into isolated
pristine Horizon source, then Cargo compiles that stage. This uses the existing
native mod format, not the legacy custom `horizon-native-source.json` loader.
Native code requires the ordinary explicit trust decision.

The 70 core files, four provenance/license data files and eight original helpers
are unchanged from the Zoom proof. All 69 Rust core files retain their bytes;
the core Cargo manifest's only prior change made edition `2024` explicit.
[Core provenance](bundle/mixins/meteor_client/data/core-provenance.json) records
those hashes and the historical initial Zoom migration scope.

AutoJump adds a raw HEAD hook at `horizon_client::game::Game<'_>::tick` for the
required Tick.Pre boundary. It calls the retained policy and existing native jump
path; LowHop writes only Y velocity. No Game or PlayerMotion fields are added,
and the adapter does not execute on the server's Game.

Use `/meteor-migration` for status and Zoom commands. AutoJump adds `auto-jump
toggle`, `auto-jump mode Jump|LowHop`, `auto-jump jump-if Always|Walking|Sprinting`
and `auto-jump velocity-height NUMBER`. Existing schema-1 Zoom settings gain
disabled AutoJump with source defaults. The separate `zoom-migration.json` does
not import the legacy prototype's state.

## Verification and limits

[Verification](evidence/verification.json) records nine staged HEAD hooks, host
checking, the linked fast build, two exact-source configuration tests, seven
packaging tests and stock-parser acceptance of the 86-entry archive. The content
parser correctly rejects native code.

The [50-check run](evidence/runtime-autojump.json) covered Jump and LowHop motion,
Always/Walking/Sprinting predicates, sneak suppression, restoration, invalid
settings, generated configuration and a targeted Zoom/mouse-sensitivity regression.
The frozen client and both harnesses stayed unchanged; shutdown exited with code 0.

Sampled normal Jump rise was about 1.252203 blocks; LowHop at 0.25 rose about
0.506239 blocks. Zero-height LowHop stayed at floor height while native ground
flags alternated and horizontal movement continued. It does not force ground
state or promise equal baseline speed. See the [zero-height explanation](evidence/lowhop-zero-height.md).
These samples are not exact mathematical apex values.

Tested client SHA-256:
`56e9ec8c64f46c06d972d88537e4251a98be7334ebd0fdb078185ee67cc1a42d`.
Control CLI SHA-256:
`a32f27d371eb24179c0d2f64b076156e46ab85b881f6204272fc3f16a401e7aa`.
The [earlier Zoom verification](evidence/verification-zoom.json) and
[32-check Zoom run](evidence/runtime-zoom.json) remain separate historical evidence.
`test_standard_zoom.py` expects migration.1's Zoom-only status text; use the
AutoJump harness for the combined client.

Zoom still has the native factor cap of 10 and turn range 0.05..1. Cinematic,
HUD and hand behavior is incomplete. Configuration was inspected without a client
restart. No full Zoom, AutoJump or Meteor parity claim is made. The tested build
used official Mixins plus pinned Cargo; **stock launcher end-to-end build/install
has not yet been tested**.

## Reproduce

Use Python 3.11 or later and pinned Rust/Mixins. Replace `PRISTINE_HORIZON`,
`PREPARED_RESOURCES` and `SCRATCH` with your own source, prepared Minecraft
resources and separate scratch paths. No custom `meteor` or `mods` feature is
needed for this pinned Horizon.

```sh
python native/tools/test_package_standard_hnmod.py
python native/tools/package_standard_hnmod.py --output dist/meteor-client-autojump-migration-proof.hnmod --report dist/package-report.json
horizon-mixins plan --workspace PRISTINE_HORIZON --mods native/bundle/mixins --out SCRATCH/plan.json
horizon-mixins apply --workspace PRISTINE_HORIZON --mods native/bundle/mixins --out SCRATCH/staged-horizon
cargo test --manifest-path SCRATCH/staged-horizon/Cargo.toml --locked -p meteor-client
cargo check --manifest-path SCRATCH/staged-horizon/Cargo.toml --locked -p horizon-client --bin horizon-client
cargo build --manifest-path SCRATCH/staged-horizon/Cargo.toml --locked --profile fast -p horizon-client --bin horizon-client --bin horizon-ctl
python native/tools/test_standard_autojump.py --client-binary SCRATCH/staged-horizon/target/fast/horizon-client --control-binary SCRATCH/staged-horizon/target/fast/horizon-ctl --resources PREPARED_RESOURCES --scratch-root SCRATCH/runtime
```

Append `.exe` on Windows. If `CARGO_TARGET_DIR` is set, use its linked binaries.
The harness requires explicit scratch paths and fresh generated singleplayer
worlds; existing worlds, real accounts and multiplayer are excluded. The package
builder itself never compiles or executes mod source.

The stock-parser validator accepts caller-owned source and target paths:

```sh
python native/tools/build_standard_validator.py --horizon-source PRISTINE_HORIZON --target-dir SCRATCH/validator-target
python native/tools/validate_standard_hnmod.py --package dist/meteor-client-autojump-migration-proof.hnmod --validator SCRATCH/validator-target/debug/meteor-standard-hnmod-validator --pristine PRISTINE_HORIZON --base-zip ORIGINAL_HORIZON_ZIP --report dist/validator-report.json
```

First builds may download dependencies; `--offline` requires a cache. Configure
absolute `RUSTC`/`RUSTDOC` paths and a pinned Cargo executable to avoid rustup
compiler changes during builds.

The ordinary launcher documents this optional entry point for a prepared base:

```sh
horizon-launcher native-build native/bundle --version PREPARED_BASE --trust-native-code
```

That full launcher build/install flow is not part of the evidence above. This
source subtree includes no game executable, Minecraft assets, prepared resources,
worlds or accounts.

## Licenses

Meteor-derived code remains GPL-3.0-only. Preserve the
[Meteor GPL notice](bundle/mixins/meteor_client/data/licenses/Meteor-GPL-3.0-only.txt),
[Starscript MIT notice](bundle/mixins/meteor_client/data/licenses/Starscript-MIT.txt)
and [Horizon MIT notice](bundle/mixins/meteor_client/data/licenses/Horizon-MIT.txt).
This is an unofficial, AI-assisted native port. The legacy prototype remains
separate from these standard native migration proofs.
