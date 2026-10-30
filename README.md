<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/meteor-horizon/logo-dark.svg">
    <img src="docs/meteor-horizon/logo.svg" alt="Meteor Horizon meteor and horizon emblem" width="112" height="112">
  </picture>
</p>

<h1 align="center">Meteor Horizon</h1>

<p align="center">
  A native Rust utility mod for Horizon / Minecraft Java 26.3.
</p>

<p align="center">
  <img src="docs/meteor-horizon/badges/rust.svg" alt="Written in Rust" height="26">
  <img src="docs/meteor-horizon/badges/minecraft.svg" alt="Minecraft Java 26.3" height="26">
  <img src="docs/meteor-horizon/badges/native.svg" alt="Native .hnmod package" height="26">
</p>

<p align="center">
  <a href="https://github.com/zackmsa777-a11y/meteor-horizon-rust/raw/refs/heads/main/packages/meteor-horizon-native.hnmod"><strong>Download .hnmod</strong></a>
  &nbsp;·&nbsp; <a href="foundation/README.md">Setup</a>
  &nbsp;·&nbsp; <a href="CONTRIBUTING.md">Contribute</a>
</p>

## Built for Horizon

Meteor Horizon brings a category-based module menu, persistent settings and
keyboard binds, chat commands and a native-font HUD to Horizon. It uses the
standard Horizon Mixins build flow and `.hnmod` source-package format.

- **Modules across five categories** — Combat, Movement, Player, Render and World.
- **Click GUI** — search modules, toggle features, adjust sliders and set binds.
- **HUD** — frame rate, coordinates, active modules and other configurable readouts.
- **Persistent configuration** — keep settings, binds and panel positions between sessions.
- **Client commands** — inspect and configure modules directly from chat.

> [!NOTE]
> **In development.** The current catalog has 87 entries. This is not a claim of
> 87 fully verified Meteor ports; the complete port and behavior parity remain unfinished.

## Get started

1. **[Download the native package](https://github.com/zackmsa777-a11y/meteor-horizon-rust/raw/refs/heads/main/packages/meteor-horizon-native.hnmod)** — current version: `0.1.0-native.1`.
2. **Build it through Horizon's experimental native-mod flow.** Follow the
   [native setup guide](foundation/README.md) and [host build documentation](docs/native-mods.md#what-the-launcher-does).
3. **Enable Meteor Horizon in Options > Mods**, then press <kbd>Right Shift</kbd>
   in a world to open the menu.

The `.hnmod` contains source that is compiled into the game. The current build
pins Horizon `b76ae35` and Mixins `cb1ad23`; use the
[full revisions and reproduction steps](foundation/README.md#reproduce).
The complete stock-launcher build/install flow remains unverified for this snapshot.

<details>
<summary>Verify the package checksum</summary>

SHA-256 for `packages/meteor-horizon-native.hnmod`:

```text
cb8d5981f60d67e6e5fb592fab53252ef8a7ca22ec19ef4d26062c3d72e8f68a
```

</details>

## Controls and commands

- <kbd>Right Shift</kbd> opens the menu.
- **Left click** toggles a module; **right click** opens its settings.
- **Type** to search. <kbd>Esc</kbd> clears the search, then closes the menu.
- **Scroll** through long panels. Use the bind row to assign a keyboard key.
- **Right click a numeric slider** to restore its default.

```text
.help                 List client commands
.modules              List modules and their state
.t zoom               Toggle Zoom
.set zoom factor 4    Change a setting
.bind zoom r          Bind Zoom to R
```

Text-list settings use `.set`. Search currently uses ASCII key names and binds
use simple keyboard keys; full Unicode editing and keyboard/controller menu
navigation remain unfinished.

## Validation

The `0.1.0-native.1` checkpoint passed:

- **9 Rust regression checks** for catalog boundaries, state cleanup, input
  ownership, narrow-layout hit boxes and thread isolation.
- **7 package checks**, plus the pristine Horizon native parser.
- **Official Mixins plan/apply, compile and linked-build checks** for the pinned host.
- **Branding checks** against the source and compiled executable.

[Recorded verification](foundation/evidence/branding/verification.json) includes
the exact package and binary hashes. These results cover that checkpoint;
interactive runtime verification and full per-module parity are separate work.

## Development

Pull requests target **`main`**. Start with [Contributing](CONTRIBUTING.md),
[native build instructions](foundation/README.md#reproduce) and the
[port status and known gaps](METEOR-PORT.md).

```text
foundation/              Current native mod, tooling and verification
native/                  Preserved four-module migration proof
crates/meteor-client/    Retained Meteor policies and reference tests
packages/                Source packages
docs/meteor-horizon/     Editable SVG logo and README assets
```

AutoRespawn and BreakDelay remain reserved for a contributor's planned work.
Check the [current allocation](CONTRIBUTING.md#current-native-integration-coordination)
before starting a module. Report vulnerabilities through [Security](SECURITY.md).
Game resources, accounts and player worlds are not included in this repository.

<details>
<summary>Earlier checkpoints and legacy packages</summary>

These records are preserved with their original scope; they do not certify the
current 87-entry catalog.

- The [four-module native proof](native/README.md) connects Zoom, AutoJump,
  AirJump and HighJump and passed **101 isolated runtime checks**.
- The legacy 28-module prototype recorded **71 isolated runtime checks**,
  **130 core tests** and separate importer fixtures. See its
  [validation records](crates/meteor-client/validation.json) and
  [checkpoint metadata](PORT-CHECKPOINT.json).
- The 26-module legacy source-patch package passed its modified launcher's
  build/install pipeline and **128 isolated runtime checks**. This is a
  different integration path; see [legacy package documentation](docs/native-source-hmod.md).

The old `.hmod` packages and their evidence remain available for reproduction.
Current development uses the standard `.hnmod` package linked above.

</details>

## Licenses and notices

Meteor Horizon is licensed under [GPL-3.0-only](LICENSE).
Original host files and dependencies retain their applicable notices; see the
[license scope](docs/licensing.md).
Required [third-party notices](foundation/bundle/mixins/meteor_client/data/licenses/THIRD_PARTY_NOTICES.txt),
[component records](foundation/bundle/mixins/meteor_client/data/component-licenses.json)
and [source provenance](foundation/bundle/mixins/meteor_client/data/source-provenance.json)
are preserved.

This is an independent, unofficial project with AI-assisted code contributions.
The [logo assets](docs/meteor-horizon/README.md) are editable SVG geometry;
no image-generation model was used. This project is not affiliated with Horizon
or MeteorDevelopment.
