# Standard native movement migration proof

This normal Horizon `.hnmod` integrates **Zoom, AutoJump, AirJump and HighJump**.
The recorded combined client passed **101 isolated runtime checks** and exited
gracefully with code 0. It remains a partial Meteor prototype: of 30 retained core
policies, the other 26 are inactive. Full per-module parity and the complete
Meteor port are not claimed.

[Download the source proof package](../packages/meteor-client-movement-migration-proof.hnmod).
SHA-256: `0062fdecefc1f530632cb69c28f8cab40629289bf6e55ace8e6fda0993dea34f`.
The 91-entry archive uses ordinary `horizon-native.json` schema 1, ID
`meteor_client`, version `0.1.0-migration.3`, with exact pins:

- Horizon `b76ae35f5b9b2d82c0e6818fc0ef93bb5277ec73`.
- Mixins `cb1ad23022392d88f006cf94cbf6552a73f5df12` (0.6.0).
- Meteor reference `e3a2e77967629d216e127b5ceecf71fb15f9d2e5`.

## Implementation and preserved source

Official Mixins stages 12 hooks, three support payloads and one core crate into
isolated pristine Horizon source. The Game runtime and HighJump publisher remain
separate from the public motion-library facade. That facade uses owned native
attribute identity, settings and callbacks across the library/binary boundary;
there are no added Game or PlayerMotion fields, raw actor pointers, extra core
crates or custom loader protocol.

AirJump reuses activation-height capture, physical jump/sneak matching and the
held maintain-level policy. HighJump uses concrete cancellable HEAD hooks rather
than RETURN/body rewrites or generic living_motion injection. Four production
HighJump files are included; spike-only shells are excluded.

All 70 core files and four data/provenance/license files are unchanged from the
verified migration.2 proof. The reserved AutoRespawn and BreakDelay policies are
also unchanged; their host integration remains the other contributor's work.
[Core provenance](bundle/mixins/meteor_client/data/core-provenance.json) retains
the exact copied-source hashes and original migration history.

## Verification and limits

[Verification](evidence/verification.json) separates the official plan/apply,
host check, linked build and runtime snapshots. Seven source-package tests passed,
the pristine stock native parser accepted the archive, and its content parser
rejected native code. All 3,567 pristine input files remained byte-identical to the
original pinned ZIP after R5.

The [101-check combined run](evidence/runtime-movement.json) includes the prior
Zoom/AutoJump regression path and new AirJump/HighJump checks. It exercised rebound
keyboard J, ignored keyboard repeats, sneak, GUI filtering, maintain-level,
disable/re-enable, and actual world leave/rejoin: captured level changed from 92
to 80. HighJump checks covered multiplier 2, zero, additive JumpBoost with zero,
tiny positive power, restoration, sprint impulse and composition with AutoJump
and AirJump. All three harnesses and frozen binaries stayed unchanged.

Sampled rises were about 1.176759 blocks normally, 4.336672 at multiplier 2,
zero at multiplier 0, 1.704257 with multiplier 0 plus JumpBoost 5, and 1.252203
after disabling. Tiny positive power produced about `4.2e-9` blocks of sampled
rise. These are observations from the fixture, not exact mathematical apex values.

Per-jump exhaustion, needs_sync and exact wrapper-tail counts remain dynamically
unverified. Native saved jump counters are observational and cannot attribute
client-wrapper counts. AirJump mouse jump bindings were not tested. Zoom's native
factor cap, cinematic/HUD/hands gaps, Freecam interaction and other unported
systems remain limitations. LowHop zero still does not force ground flags or
equal horizontal speed; see the [earlier explanation](evidence/lowhop-zero-height.md).

The [13 exact state tests](evidence/movement-state-units.json) and
[two native HighJump identity tests](evidence/highjump-component-units.json) retain
their narrower scopes. Historical [migration.2 evidence](evidence/verification-autojump.json),
[50-check AutoJump result](evidence/runtime-autojump.json),
[Zoom evidence](evidence/verification-zoom.json) and
[32-check Zoom result](evidence/runtime-zoom.json) remain separate.

The tested client SHA-256 is
`dffe550cd9a9716a906ea6b07230d7c664cf4cc8d63ad373dd65cba898864443`;
the control CLI SHA-256 is
`66798153f91d40246ced8440fcfe1ad10464874fcdb8d89fe93eed683347e626`.
The build used official Mixins R5 plus pinned Cargo. **Stock launcher end-to-end
build/install remains unverified.**

## Reproduce

Use Python 3.11 or later and pinned Rust/Mixins. Replace the source, resource and
scratch placeholders with your own paths. Keep outputs separate from pristine
source. No custom `meteor` or `mods` feature is needed for this pinned base.

```sh
python native/tools/test_package_standard_hnmod.py
python native/tools/package_standard_hnmod.py --output dist/meteor-client-movement-migration-proof.hnmod --report dist/package-report.json
horizon-mixins plan --workspace PRISTINE_HORIZON --mods native/bundle/mixins --out SCRATCH/plan.json
horizon-mixins apply --workspace PRISTINE_HORIZON --mods native/bundle/mixins --out SCRATCH/staged-horizon
cargo test --manifest-path SCRATCH/staged-horizon/Cargo.toml --locked -p meteor-client
cargo check --manifest-path SCRATCH/staged-horizon/Cargo.toml --locked -p horizon-client --bin horizon-client
cargo build --manifest-path SCRATCH/staged-horizon/Cargo.toml --locked --profile fast -p horizon-client --bin horizon-client --bin horizon-ctl
python native/tools/test_standard_movement.py --client-binary SCRATCH/staged-horizon/target/fast/horizon-client --control-binary SCRATCH/staged-horizon/target/fast/horizon-ctl --resources PREPARED_RESOURCES --scratch-root SCRATCH/runtime
```

Append `.exe` on Windows and adjust binary paths if `CARGO_TARGET_DIR` is set.
The movement harness imports both retained helper scripts; older standalone
harnesses expect their own snapshot's status text. Omit `--skip-lifecycle` to
exercise the leave/rejoin check. Every game run uses explicit paths, new UUID data
homes and fresh generated singleplayer fixtures, excluding existing worlds, real
accounts and multiplayer. The fixture alone disables fall damage.

The stock-parser helper builds a small validation tool, not the game:

```sh
python native/tools/build_standard_validator.py --horizon-source PRISTINE_HORIZON --target-dir SCRATCH/validator-target
python native/tools/validate_standard_hnmod.py --package dist/meteor-client-movement-migration-proof.hnmod --validator SCRATCH/validator-target/debug/meteor-standard-hnmod-validator --pristine PRISTINE_HORIZON --base-zip ORIGINAL_HORIZON_ZIP --report dist/validator-report.json
```

Use absolute `RUSTC`/`RUSTDOC` paths and pinned Cargo to avoid rustup compiler
changes. First builds may download dependencies; `--offline` requires a cache.

The ordinary launcher documents this optional native entry point for a prepared
installed base:

```sh
horizon-launcher native-build native/bundle --version PREPARED_BASE --trust-native-code
```

That complete launcher flow was not part of the recorded runtime verification.
No game executable, prepared Minecraft assets, world or account is included.

## Licenses

Preserve [Meteor GPL-3.0-only](bundle/mixins/meteor_client/data/licenses/Meteor-GPL-3.0-only.txt),
[Starscript MIT](bundle/mixins/meteor_client/data/licenses/Starscript-MIT.txt) and
[Horizon MIT](bundle/mixins/meteor_client/data/licenses/Horizon-MIT.txt) notices.
This is an unofficial, AI-assisted native port. The legacy prototype remains
separate from the standard native migration.
