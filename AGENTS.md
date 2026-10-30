# Meteor mod project — agent guide

This repository develops a native Rust Meteor Client mod for Horizon. Read `CONTRIBUTING.md`,
`METEOR-PORT.md` and the feature inventory before changing mod behavior. A pinned Horizon host
source tree is included for native integration and builds.

- Target the standard `.hnmod` / Horizon Mixins integration. Preserve the Meteor core and tests;
  migrate the adapter and packaging. Do not present legacy `.hmod` source-patch packages as normal
  content mods or extend their custom launcher protocol as the new distribution architecture.
- Use GPT-6.1 Sol with high reasoning for delegated agents unless the user changes this preference.
  Do not switch agents to Astra.
- Put mod policies in `crates/meteor-client` and host operations in the corresponding
  `meteor*.rs` adapters. Do not register empty modules or claim full parity from partial checks.
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
