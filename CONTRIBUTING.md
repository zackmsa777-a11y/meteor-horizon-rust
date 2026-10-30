# Contributing to the Meteor mod for Horizon

This project ports Meteor Client features to a native Rust mod for Horizon. Contributions
focus on modules, GUI/HUD, commands, configuration import and the standard `.hnmod` / Horizon Mixins integration.
The port is in development; the feature inventory records implemented behavior and remaining gaps.

## Where to contribute

Use [this repository's issues](https://github.com/zackmsa777-a11y/meteor-horizon-rust/issues)
for bugs and proposals, and open pull requests here. Check existing work before starting a large
change. This project does not use Horizon's Discord claim bot or upstream release labels.

`main` preserves the verified legacy checkpoint. `next-source` preserves newer legacy work;
`hnmod-migration` develops the standard native integration in [`native/`](native/).
Coordinate ongoing port changes with the repository maintainer,
[@zackmsa777-a11y](https://github.com/zackmsa777-a11y).

## Current native integration coordination

New Solstice foundation PRs target **`solstice-integration`**. Meteor Horizon is adapting
Solstice's existing MIT native foundation using the project's pinned Horizon/Mixins pair.
The current allocation (2026-10-30) is:

- **AutoRespawn and BreakDelay:** reserved for a separate contributor's planned PR against
  `hnmod-migration` until an explicit handoff. The foundation must exclude upstream Auto Respawn
  and overlapping Fast Break registration/dispatch; do not silently replace the reserved work.
- **Foundation adaptation, GUI/HUD and validation:** active work by this team.
- **Previous snapshots:** migration.3 retains verified Zoom/AutoJump/AirJump/HighJump evidence;
  the separate four-module GUI migration.4 remains held unfinished.

Check assignments and existing PRs before changes. Keep the friend's source, tests and
attribution intact, and coordinate a foundation port when that work arrives. Update the
reservation when handed off or merged; do not infer a contributor identity or retarget an
existing PR without coordination.

Keep module-specific host logic in separate injection helpers or self-contained support payloads
under `foundation/bundle/mixins/meteor_client/` for the new foundation, using descriptive module
prefixes. Keep the entire existing `native/` tree and versioned archives unchanged. Prefer small
registration changes in shared `runtime.rs` and `horizon-mixin.toml`; preserve existing hooks and
avoid unrelated formatting or whole-file replacement. Rebase focused changes on the latest
appropriate target-branch head before integration to resolve shared dispatch or manifest edits.

Retain the reusable Meteor core/tests and explicit Solstice MIT / Meteor GPL component notices.
Do not globally relicense the retained Meteor-derived source. Generate pristine stages with the pinned
upstream Mixins tool; do not introduce a custom loader or manually edit generated host source.
Preserve versioned verified packages and their evidence while developing the next integration,
and record only checks actually run against the new source snapshot.

## Source layout and setup

Integration migration is underway. Preserve reusable module behavior and tests; move host glue
into a standard Horizon Mixins bundle. Prefer named hooks, and pin raw hooks to a tested Horizon
revision where needed. Do not expand the legacy custom source-patch loader as the distribution path.
The existing source layout and build commands below describe the legacy prototype during migration.
For the standard integration, start with [`native/README.md`](native/README.md). Keep bundle
crates and adapter helpers there, and generate a new source stage from pristine Horizon using
the pinned upstream Mixins tool. Do not edit generated or pristine host files manually.

- `crates/meteor-client/`: mod behavior, settings, persistence and core tests.
- `crates/horizon-client/src/game/meteor*.rs`: native host integration and GUI/HUD.
- `tools/test_meteor_runtime.py` and `tools/meteor_*_fixture.py`: isolated runtime checks.
- `tools/package_meteor_hmod.py`: source-based `.hmod` packaging.
- `launcher/` and `docs/native-source-hmod.md`: the required native package build/load support.

The repository includes a pinned Horizon host source snapshot to build and test the mod's
integration. [Upstream setup instructions](UPSTREAM-HORIZON-README.md) explain prerequisites
and preparing resources from your own Minecraft 26.3 installation. Build the mod with:

```sh
cargo build --profile fast -p horizon-client --features meteor
```

## Implementing a feature

Follow the supplied Meteor reference version documented in `METEOR-PORT.md`. Preserve its
defaults, settings names, lifecycle and event ordering where supported. Keep reusable policies
in `meteor-client` and actual game operations in the host adapter. Player-facing code is Rust;
Python and Java may be used for development tools and reference checks.

Document unsupported dependencies and behavior explicitly. Empty registrations do not count
as implemented modules, and passing a smoke test does not establish complete source parity.
Update the inventory when a feature's implementation or verification status changes.

## Validation

Run checks appropriate to the change and include their exact results in the pull request.
Core behavior changes should run `cargo test -p meteor-client`. Standard integration changes
must pass Mixins planning/apply and compile the resulting stage with its locked dependencies;
the standard proof does not use a custom `meteor` feature. Changes to the preserved legacy
adapter should compile with that feature enabled and disabled. Add meaningful coverage for
changed behavior. Documentation-only changes do not require rebuilding the game.

Runtime tests use fresh temporary singleplayer worlds and generated files. Set isolated
`HORIZON_HOME`, `HORIZON_LAUNCHER_HOME`, `XDG_DATA_HOME` and `XDG_CONFIG_HOME`; disable
self-updates and audio in agent runs. The existing runtime driver handles these settings.
Keep existing worlds, real accounts and multiplayer servers out of automated test runs.

Before pushing, run `python tools/check_public_tree.py --history`. Do not commit Minecraft
JARs/assets, decompiled game sources, prepared resources, worlds, account files or credentials.

## Pull requests and attribution

Keep changes focused. Explain the resulting behavior, the Meteor reference followed,
validation performed and remaining limitations. Include screenshots for visible UI changes.
Identify AI-assisted contributions and retain the applicable commit co-author attribution.

Meteor-derived contributions retain GPL-3.0-only attribution. Original Horizon files retain
their MIT notices; Starscript and other dependencies retain their own notices. See the
[source and license overview](README.md#source-and-licenses). Development artifacts and
checks in this repository do not imply an upstream Horizon release or review.

For vulnerabilities, follow [SECURITY.md](SECURITY.md).
