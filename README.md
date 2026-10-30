# Meteor mod for Horizon

> **Installation status:** migration to the standard Horizon `.hnmod` / `horizon-mixins`
> integration is in progress. A standard installable Meteor `.hnmod` is not released yet.
> The existing `.hmod` is a legacy source-patch prototype for a custom build path. It cannot
> be loaded through Horizon's ordinary content Mods menu; renaming it will not convert it.

A native Rust Meteor Client mod for Horizon / Minecraft Java 26.3, targeting the standard `.hnmod` format through Horizon Mixins.
The port is unofficial and still in development.

The existing module logic, settings, configuration and tests are being retained. Migration focuses
on the Horizon adapter, state handling and packaging. The repository still includes the pinned
Horizon source snapshot used to reproduce the verified legacy prototype; attribution is retained below.

Follow the [standard `.hnmod` migration branch](https://github.com/zackmsa777-a11y/meteor-horizon-rust/tree/hnmod-migration/native)
for the current native bundle. Its first proof connects Zoom; the remaining retained modules
are still being migrated. This branch preserves the verified legacy checkpoint.

[Legacy prototype package — custom build path only](packages/meteor-client-native-28-module.hmod) ·
[Contributing](CONTRIBUTING.md) · [Security](SECURITY.md)

The verified checkpoint on `main` implements 28 of the reference client's 169 registered modules.
The complete Meteor port is not finished. See [METEOR-PORT.md](METEOR-PORT.md) and
[the feature inventory](crates/meteor-client/port-inventory.json) for partial behavior and remaining work.
`next-source` contains newer work that is not included in this verified checkpoint.

## Verification

- 130 core tests passed; seven original module/friends importer fixtures and four macro importer fixtures passed.
- The exact 28-module development client passed 71 isolated runtime checks for Flight, AutoClicker,
  macro NBT imports, repeated HUD instances and Starscript text. Earlier checkpoints have separate evidence.
- The 26-module source `.hmod` also passed the modified launcher's actual native build/install pipeline
  and 128 isolated runtime checks on its installed GNU client.

These are scoped checks, not complete parity certification. Runtime tests use fresh temporary
singleplayer worlds and generated data. Game resources and player data are not included.

## Build and use

The instructions below reproduce the **legacy prototype**. The replacement integration will use
Horizon's [standard native-mod build flow](https://playhorizon.dev/docs/mod-manifest), with a
`horizon-native.json`, a `horizon-mixin.toml`, pinned revisions and a normal `.hnmod` bundle.
That flow still compiles a modded game; it should not require a Meteor-specific launcher format.

Follow [Horizon build prerequisites and resource preparation](UPSTREAM-HORIZON-README.md).
Build the optional native port with:

```sh
cargo build --profile fast -p horizon-client --features meteor
```

Launch using your own prepared Minecraft 26.3 resources. In a world, press **Right Shift**
to open the Meteor GUI. Modules start disabled. The HUD toolbar opens the HUD editor.

The legacy source-patch `.hmod` needs the prototype's modified launcher; stock
content-only `.hmod` loaders cannot load it. See [native package documentation](docs/native-source-hmod.md).
The legacy, prototype-tested [28-module source package](packages/meteor-client-native-28-module.hmod) is included.
The original Java Meteor JAR cannot execute as a native Rust plugin.

## Source and licenses

Horizon base snapshot: `b76ae35f5b9b2d82c0e6818fc0ef93bb5277ec73`.
Meteor reference source/JAR commit: `e3a2e77967629d216e127b5ceecf71fb15f9d2e5`.

- Meteor-derived code: GPL-3.0-only; [full license](crates/meteor-client/COPYING).
- Original Horizon files: retain their [MIT license](LICENSE).
- Starscript-derived code: [MIT notice](crates/meteor-client/STARSCRIPT-LICENSE.txt).
- Other third-party notices remain with their source files.

Upstream projects: [Horizon](https://github.com/horizon-rust/horizon),
[Meteor Client](https://github.com/MeteorDevelopment/meteor-client),
[Starscript](https://github.com/MeteorDevelopment/starscript).

Changes were implemented with AI assistance. This is an independent, unofficial Meteor mod project.
