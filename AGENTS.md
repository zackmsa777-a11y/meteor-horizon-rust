# Meteor mod project — agent guide

This repository develops a native Rust Meteor Client mod for Horizon. Read `CONTRIBUTING.md`,
`METEOR-PORT.md` and the feature inventory before changing mod behavior. A pinned Horizon host
source tree is included for native integration and builds.

- Target the standard `.hnmod` / Horizon Mixins integration. Preserve the Meteor core and tests;
  migrate the adapter and packaging. Do not present legacy `.hmod` source-patch packages as normal
  content mods or extend their custom launcher protocol as the new distribution architecture.
- Use GPT-6.1 Sol with high reasoning for delegated agents unless the user changes this preference.
  Do not switch agents to Astra.
- Develop the standard adapter in `native/bundle/mixins/meteor_client/`, with reusable policies
  in its `core/` crate and host operations in support modules and injection helpers. Generate
  stages using upstream Mixins; do not edit pristine or generated host source manually.
  `crates/meteor-client` and the host's `meteor*.rs` files preserve the legacy prototype.
  Do not register empty modules or claim full parity from partial checks.
- Native integration PRs target `hnmod-migration`. Current allocation (2026-10-30):
  AutoRespawn and BreakDelay are reserved for a separate contributor's planned PR. AirJump and
  HighJump were integrated by this team in verified migration.3. Recheck assignments and open
  PRs before starting another module, and update this note when work is handed off or merged.
- Prefer separate module-specific injection helpers and self-contained support payload files.
  Keep shared `runtime.rs` dispatch/configuration and `horizon-mixin.toml` registration changes
  small, preserve other modules' wiring, and avoid unrelated formatting or whole-file rewrites.
  Retain the reusable core and tests; use upstream Mixins, not a new custom loader.
- Preserve versioned verified packages and their evidence when developing the next integration.
  A new adapter or passing plan is not a runtime result for an earlier artifact.
- Preserve applicable Meteor GPL-3.0-only, Horizon MIT, Starscript MIT and third-party notices.
- Never commit Minecraft JARs/assets, decompiled game sources, prepared resources, credentials,
  accounts or player worlds. Run `python tools/check_public_tree.py --history` before pushing.
- Run games and launchers only with fresh isolated data homes. Agent runs set `HORIZON_HOME`,
  `HORIZON_LAUNCHER_HOME`, `XDG_DATA_HOME` and `XDG_CONFIG_HOME` to scratch directories, plus
  `HORIZON_LAUNCHER_NO_SELF_UPDATE=1` and `HORIZON_NO_AUDIO=1`. Launcher trash, clipboard and
  desktop paths must also point into scratch data. Use generated fixtures, not existing worlds,
  real accounts or multiplayer, for automated tests.
- Run focused checks appropriate to changes; record actual results and exact tested snapshots.
  Keep verified `main` evidence separate from unverified `next-source` changes.
- Do not create issues, send messages, request reviews or publish releases unless authorized.
  Keep AI attribution clear in authorized contributions and include commit co-author attribution.
- Repository ownership and security reporting are defined here and in `SECURITY.md`.
  Upstream Discord claims, maintainer identities and release automation do not govern this mod.

The [archived Horizon agent map](docs/upstream/HORIZON-AGENTS.md) is useful for navigating
host crates and engine tests. Its upstream project-management instructions are historical
context; this mod's contributing and agent guidance takes precedence in this repository.
