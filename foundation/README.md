# Meteor Horizon

An unofficial native Rust utility mod for Horizon / Minecraft Java 26.3.
The standard `.hnmod` contains modules, a Right Shift menu, settings, keyboard
binds, chat commands and a native-font HUD. Full Meteor parity is unfinished.

[Download the native source package](../packages/meteor-horizon-native.hnmod)
(`0.1.0-native.1`). SHA-256:
`cb8d5981f60d67e6e5fb592fab53252ef8a7ca22ec19ef4d26062c3d72e8f68a`.

Development branch: `native-development`. The package uses ID `meteor_client`,
Horizon `b76ae35f5b9b2d82c0e6818fc0ef93bb5277ec73` and Mixins 0.6
`cb1ad23022392d88f006cf94cbf6552a73f5df12`. It adds 24 hooks and 10 support
modules through upstream Mixins, with no custom loader or manually patched host.

## Use

Enable Meteor Horizon under the game's Options > Mods after the standard native
build. Right Shift opens the menu. Left click toggles modules; right click opens
settings. Type to search; Escape clears the search and then closes the menu.
Numeric settings use sliders; keyboard binds and settings persist.

Production commands include `.help`, `.modules`, `.t`, `.settings`, `.set` and
`.bind`. Text-list settings use `.set`. Search and binds retain their current
ASCII/simple-key limitations; full keyboard/controller navigation is unfinished.

Use one version of the `meteor_client` native package per instance. This is a
native source mod and requires the experimental native build flow. The complete
stock-launcher build/install flow is not yet verified for this snapshot.

## Scope and validation

The catalog has 87 entries. AutoRespawn and BreakDelay remain reserved, so the
overlapping Auto Respawn/Fast Break entries are excluded. Catalog size does not
mean full source fidelity or 87 tested ports. Earlier movement policy differences
and architecture limits remain recorded in the historical evidence.

This update changes branding and internal names while retaining module behavior,
defaults and layout. Seven package checks passed and the pristine native parser
accepted the archive. Official plan/apply, compilation and all nine existing Rust regression tests
passed. The linked build is in progress. Interactive checks remain scoped separately.

The earlier complete `native/` tree and its 101-check four-module proof are
preserved. Historical archives and evidence keep their original bytes and scope.

## Reproduce

Use pristine Horizon at the pin above and the pinned Mixins CLI, with separate
fresh stage/output paths and an already pinned Rust compiler:

```sh
python foundation/tools/test_package_native_hnmod.py
python foundation/tools/package_native_hnmod.py --bundle foundation/bundle --output DIST/meteor-horizon-native.hnmod --report DIST/package.json
horizon-mixins plan --workspace PRISTINE_HORIZON --mods foundation/bundle/mixins --out SCRATCH/plan.json
horizon-mixins apply --workspace PRISTINE_HORIZON --mods foundation/bundle/mixins --out SCRATCH/staged-horizon
cargo check --manifest-path SCRATCH/staged-horizon/Cargo.toml --locked -p horizon-client --bin horizon-client
cargo test --manifest-path SCRATCH/staged-horizon/Cargo.toml --locked --profile fast -p horizon-client --lib --bin horizon-client meteor_horizon
cargo test --manifest-path SCRATCH/staged-horizon/Cargo.toml --locked --profile fast -p horizon-client-input --lib input::meteor_horizon_flags::tests
cargo build --manifest-path SCRATCH/staged-horizon/Cargo.toml --locked --profile fast -p horizon-client --bin horizon-client --bin horizon-ctl
```

Keep Minecraft resources local and use generated worlds/fresh homes for tests.
Build prerequisites remain in [the host setup guide](../UPSTREAM-HORIZON-README.md).

## Legal notices

Original copyright and permission notices remain in
[third-party notices](bundle/mixins/meteor_client/data/licenses/THIRD_PARTY_NOTICES.txt)
and the [component license records](bundle/mixins/meteor_client/data/component-licenses.json).
[Source provenance](bundle/mixins/meteor_client/data/source-provenance.json)
records original sources and modifications. Meteor-derived code keeps GPL-3.0-only;
other components keep their applicable notices. No original-authorship claim or
global relicensing is made.
