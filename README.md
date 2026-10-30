> **Working branch:** this branch includes newer HUD NBT imports, HUD cache/editor fixes,
> and activation-ordered module Post callbacks. It has focused checks but has not yet passed
> a full rebuilt-client runtime suite. Use `main` for the verified 28-module checkpoint.

# Meteor Client for Horizon (native Rust port)

Unofficial work in progress: a native Rust port of Meteor Client for Horizon / Minecraft Java 26.3.

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

Follow [Horizon build prerequisites and resource preparation](UPSTREAM-HORIZON-README.md).
Build the optional native port with:

```sh
cargo build --profile fast -p horizon-client --features meteor
```

Launch using your own prepared Minecraft 26.3 resources. In a world, press **Right Shift**
to open the Meteor GUI. Modules start disabled. The HUD toolbar opens the HUD editor.

The source-based `.hmod` needs this fork's modified native-source-aware launcher; stock
content-only `.hmod` loaders cannot load it. See [native package documentation](docs/native-source-hmod.md).
The verified [28-module source package](packages/meteor-client-native-28-module.hmod) is included.
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

Changes were implemented with AI assistance. This repository is an independent development fork.
