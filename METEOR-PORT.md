# Meteor Client native Rust port

This is a native Meteor mod project for Horizon, currently in development.
The integration target is now a standard `.hnmod` bundle using Horizon Mixins. Existing core
behavior and tests are retained; the host adapter and packaging are being migrated. Recorded
28-module runtime evidence belongs to the legacy source-patched prototype and does not certify
a standard `.hnmod` build. The experimental standard proof is scoped separately below.
The objective remains full native behavior, including modules, GUI/HUD, configuration, commands,
utilities, integrations and lifecycle. Java JARs are reference artifacts, not executable plugins here.

The [standard native proof](native/README.md) retains 30 module policies and currently connects
Zoom, AutoJump, AirJump and HighJump through upstream hooks. It passes 101 isolated checks, with exact
package and client hashes in its evidence. The other 26 policies remain inactive in that proof. Its evidence is separate
from the legacy inventory below; retaining a policy does not count as migrating its host integration.

Migration.4 adds AutoRespawn and BreakDelay to the same standard `.hnmod` (six policies integrated, 24
inactive). Its 39 player-interaction checks and the 101-check movement regression, with package and
client hashes, are in `native/evidence/runtime-player-interactions.json`.

The authoritative feature inventory is `crates/meteor-client/port-inventory.json`.
It enumerates registered module classes and every Java source file from the supplied archive.
Missing implementations must remain explicitly missing; empty module registrations do not count as ports.
The source ZIP's Git archive commit marker matches the reference JAR's metadata:
`e3a2e77967629d216e127b5ceecf71fb15f9d2e5`. File hashes are retained in the inventory.

## First implementation

- The 26-module source checkpoint adds AirPlace, LiquidInteract and NoMiningTrace.
  AirPlace uses real item interaction and a separate translucent overlay pass; LiquidInteract
  retries source-fluid picking only when the ordinary block ray misses; NoMiningTrace applies
  pickaxe and entity blacklist rules to the selected target. Core, importer, renderer and
  feature-on/off compile checks passed. The frozen 26-module preview passed 86 live
  picking, placement, mining, Starscript and color GUI checks, plus 102 earlier
  regression checks. Reports identify the exact tested client and package hashes.
  Horizon's baseline picker differs from vanilla when an entity outside entity reach precedes
  a block inside the larger block reach. The integrated server also retains its own attack range;
  client Reach settings do not override server authority.
- The next 28-module source adds Flight and AutoClicker, original macro NBT import,
  repeated HUD instances, source physical HudBox anchors and configurable Starscript
  Text elements. Its 130 core tests passed; live verification is pending. HUD supports
  vanilla text rendering, single-selection snapping and nine legacy native presets.
  Bundled custom fonts, HUD-wide binding, multi-selection, palette add/remove UI and
  original HUD NBT import remain incomplete.

- Native ClickGUI foundations: Right Shift, module categories/search, toggle/settings/bindings,
  favorites and hold settings. HUD editor supports watermark, FPS, coordinates and active modules,
  global/per-element visibility and persisted positioning. These are partial ports; fonts, themes,
  other HUD elements, Starscript and the complete original widget system remain.
- AutoWalk Simple directions, manual-input cancellation, vertical-change cancellation and unloaded
  chunk avoidance. Smart mode explicitly disables with the source's missing-path-manager message;
  native Baritone/pathing is not implemented.
- SafeWalk source policy, real collision/heightmap/raycast probes, ledge clipping, late sneak input
  and red/green outline rendering. Terrain/fluid combinations still need broader parity validation.
- Step attribute capture/restoration and SafeStep sampling with native crystal exposure and damage
  prediction. Basic ascent/restoration and SafeStep without crystals passed isolated checks;
  populated crystal scenarios and Baritone's separate step setting remain. SafeStep rejects huge
  sweeps instead of reproducing the source's potential integer-wrap hang.
- ReverseStep player downward velocity and real collision probes. Isolated velocity/restoration
  checks passed; enabling vehicle handling while riding explicitly disables with an unsupported message.
- FastClimb speed/timer modes, Spider wall-climb policy and Parkour ledge jump policy now have
  native collision/input adapters. Timer multiplies the client's delta ticks at the source hook
  point; real elapsed time and separate server pacing remain unaffected. Shared overrides follow
  the original sticky behavior (FastClimb has no deactivate reset). Isolated runtime checks passed.
- Friends model and native add/remove/list commands, JSON persistence and a Friends GUI page.
  Name membership is case-insensitive and unresolved/offline identities are retained. Online
  profile/head refresh and complete upstream Friends UI fidelity remain; stored names/UUIDs can
  now be imported from original NBT without fetching online profiles.
- AutoRespawn, BreakDelay and FastUse now have real death-screen, block-breaking and per-hand
  use-delay hooks. Actual breaking/bottle-use rates, item filtering, automatic respawn and restored
  death-screen behavior passed isolated checks. Nondefault Waypoints death behavior remains unavailable.
- Fullbright Gamma, Potion and Luminance have native renderer/effect hooks. Isolated checks passed
  for visible lighting, independent block/sky floors, unchanged gameplay light, Night Vision refresh
  and disabling/restoration. Smooth-light floors apply before averaging; stale meshes are invalidated.
- HUD text presets expanded with real speed, rotation, ping, server TPS and game-mode data.
  TPS uses incoming time-packet intervals; it is independent of the client's Timer pacing.
- New `meteor-client` crate with module state, modifier-aware keybind precedence, hold-release
  handling, world lifecycle and versioned Rust JSON configuration.
- Zoom core: source defaults, cubic easing, scroll adjustment, sensitivity factor and fade-out lifecycle.
- AutoJump: source movement conditions, jump/low-hop action decisions, and a local-client Tick.Pre
  adapter using Horizon's MotionWorld jump routine. The server thread does not execute the mod.
- AirJump: activation height, airborne keyboard press behavior, maintain-level and sneak-to-descend
  policy, with rebound key matching and Horizon's jump routine. Freecam interaction awaits Freecam.
- Sprint: Strict/Rage policy, source thresholds and water behavior, optional omnidirectional motion,
  Rage jump yaw/boost, attack packet stop/restart ordering, keep-sprint hook, commands and settings.
  Runtime validation and vehicle sprint capability bindings remain incomplete.
- GUIMove: inventory/GUI screen policy, text-entry exclusions, per-action controls, lifecycle clearing,
  keyboard/mouse binding matching, arrow rotation and movement input adapter. Horizon inventories
  are mapped; Meteor's original WidgetScreen and Freecam behavior are still missing.
- HighJump: scales the block jump factor (before additive potion jump boosts) and preserves the
  source's zero-power jump behavior. Uses the same local-player jump path as AirJump/AutoJump.
  Normal/doubled/zero/restored jump-height smoke checks passed; no claim of full HighJump parity is made.
- Optional Horizon `meteor` feature and per-Game adapter, disabled in ordinary builds.
- Horizon integration for Zoom's FOV, scroll, keybinds, commands and existing Mods settings page.
- Configuration merges newly implemented modules into older saved state. Duplicate and unknown module
  identities are rejected, and invalid setting edits leave existing values intact.
- Behavioral tests cover the module policies and lifecycle. These are focused Rust checks,
  not Java-vs-Rust differential proof; see validation.json for executed counts.

The current adapter is partial. An approved isolated singleplayer smoke test passed 46 checks:
Zoom factor, sensitivity, HUD/hands flags, cinematic mode, restoration, AutoJump movement/start-stop,
AirJump airborne rebound-key input, captured height, held maintain-level and disable behavior,
Sprint forward/sideways speed, stationary mode and disable behavior, and GUIMove inventory
movement, release/repress, arrow rotation and screen filtering. Focus-loss and control-release
paths now clear the mod's separate held-key state; the latter was exercised in the runtime test.
HighJump's measured rises were about 1.166 blocks disabled, 4.306 with multiplier 2, zero with
multiplier 0, and 1.166 after disabling it again. These are sampled heights, not exact apex values.
Measured forward and sideways speed increased by approximately 30% with the corresponding Sprint
mode. Sprint attack packet behavior, water behavior and vehicles still need runtime verification.
Screenshots visually confirmed magnification and hidden hands; HUD and cinematic checks inspected
the actual adapter options. This is not Java-vs-Rust differential proof. The test client exited cleanly.
The control API now routes client-only commands locally and exposes read-only Meteor status.
Later independent runs are retained in `crates/meteor-client/runtime-*.json`: GUI/walking,
Step/ReverseStep, climbing/Timer/Friends, generated NBT/chords, and player actions/Fullbright.
The earlier tested checkpoint has 18 registered modules out of the 169 source module classes.
Its 47-check player-action/Fullbright run used binary SHA256
`638bc94a47b11120083eb2c61873dfc2b8e8a389146204ac1ae517bd206d45fa` and exited cleanly.
These results do not verify later work or establish full differential parity.
The subsequent 21-module checkpoint adds Sneak, NoSlow and Slippy, plus native Profiles commands.
It passed 86 core tests, one resource-backed player-physics regression and 61 isolated runtime checks.
Runtime checks include actual crouch/packet input, shield and terrain slowdown, coasting, retained
raw fluid state, profile persistence and ordered loading after reopening the generated test world.
Its binary SHA256 is `140a3f91f3f427120b2a4390d23326149201ca470313e6429ae5b51edc61792e`;
its matching source-package SHA256 is `431125e5252ee0c3cc99a54bb8ea1c56b0edab6cb2ead460f6fe81702fe81db0`.
See `runtime-slow-profiles.json` for evidence. Later source changes are separate from this checkpoint.
Camera baseline restoration if options change while active, complete HUD-layer
parity, key lock modifiers, focus-loss handling, original Meteor NBT settings, the complete GUI and
all other modules remain. Settings are in the instance's native-mod data directory under `meteor-client`;
the original Meteor configuration is never overwritten. A read error prevents saves over that file.

## Validation authorization

Current source additions after the proven 21-module snapshot include Reach and Multitask,
native Macros/Starscript, Profiles/Macros GUI pages, macro profile sections, and activation-ordered
shared Timer writers. The 23-module source passes 95 core tests, both client compile configurations,
a real input-queue regression and two ray-interval regressions. Its comprehensive isolated runtime
run passed 102 checks, including Macros/Profiles UI and scheduling, profile macro queue identity,
real distant chest interaction, actual entity occlusion and simultaneous shield/mining behavior.
The tested binary is `5cab6b39eb2692cf7a3edcd026815ae5853ca2a884de532a18b85b6cb39da3de`,
matching source package `b43ec50bd60740f2daa36be5917d4382ca5b9b371f843cab8800cd3b12692333`.
See `runtime-macros-gui-reach.json`. Horizon's existing server attack picker retains its fixed
three-block survival check; Reach adds client targeting and does not bypass server authority.
Starscript has 1,029 official Java-reference cases, but host bindings remain a documented subset.
Original macro/profile NBT, wider HUD/Starscript integration, and HighwayBuilder's drawing-bow
release exemption remain incomplete. Source-only additions do not change older checkpoint evidence.

Testing is authorized within the isolated scope recorded below. The user approved Rust unit tests and Horizon compile checks
in this chat. They also approved building and launching the development client in a new temporary
singleplayer world to check Zoom and AutoJump, followed by approval for isolated AirJump runtime
checks (airborne press, rebound key, maintain-level and disable). This does not authorize multiplayer or existing worlds.
The user subsequently approved isolated Sprint checks for forward/sideways speed, stationary mode
and disabling it. GUIMove inventory movement, release, arrow rotation and screen filtering
runtime checks were subsequently approved as isolated singleplayer tests.
HighJump normal/doubled/zero/restored jump-height checks were also approved for a fresh isolated world.
GUI/HUD opening/search/activation and AutoWalk/SafeWalk movement, cancellation, ledge and outline
checks were approved subsequently. Prepared flags are `--native-gui --walk`; approval alone is not
evidence that these checks passed.
Step/ReverseStep and then FastClimb/Spider/Parkour/Timer/Friends GUI received separate runtime
approval. Flags are `--step --climb --friends`; each remains opt-in in the harness.
Generated-file original-NBT import checks were separately approved and passed with `--nbt-import`.
The user subsequently approved AutoRespawn/BreakDelay/FastUse, Fullbright, and **all future
full-port runtime tests restricted to fresh temporary singleplayer worlds and generated files**.
No further per-module approval is needed within that scope. Existing worlds, real accounts and
multiplayer servers remain excluded. Continue announcing meaningful test batches and retaining
evidence. The earlier Rust unit-test/compile/dependency-download approval also remains in force.
All game tests must use fresh scratch homes as described in AGENTS.md.

Approved commands:

```
cargo test -p meteor-client
cargo check -p horizon-client --bin horizon-client --features meteor
```

The initial controls are available through Options > Mods >
Meteor Rust or `/meteor toggle zoom`, `/meteor bind zoom key.keyboard.c`, and `/meteor zoom 6`.
AutoJump uses `/meteor toggle auto-jump`; settings can be edited with
`/meteor set auto-jump mode LowHop`, `/meteor set auto-jump jump-if Walking` and
`/meteor set auto-jump velocity-height 0.25`. Zoom settings use the same `set` command,
for example `/meteor set zoom cinematic true` and `/meteor set zoom hide-HUD true`.
AirJump uses `/meteor toggle air-jump` and `/meteor set air-jump maintain-level true`.
Sprint uses `/meteor toggle sprint` and `/meteor set sprint sprint-mode Strict` (or `Rage`).
Its boolean settings are `keep-sprint`, `unsprint-on-hit`, `unsprint-in-water`, and
`sprint-while-stationary`. They are also exposed on the native Mods settings page.
GUIMove uses `/meteor toggle gui-move`; settings include `guis` (`GUI`, `Inventory`, `Both`),
`jump`, `sneak`, `sprint`, `arrows-rotate`, and nonnegative `rotate-speed` (default 4).
`GUI` awaits the original Meteor widget GUI; vanilla options screens are not silently substituted.
HighJump uses `/meteor toggle high-jump` and `/meteor set high-jump jump-multiplier 2`.
The multiplier defaults to 1 and accepts nonnegative finite values.
Press **Right Shift** in a world for the native module GUI. Left-click toggles; right-click edits.
Use **HUD** at the top and enable it in the editor. Drag elements to move them, right-click to
change element visibility, Escape to return, and Right Shift to close.
The build needs `--features meteor`. Do not install this development fork over the working game.

Additional commands: `/meteor friends add NAME`, `/meteor friends remove NAME`,
`/meteor friends list`. The Friends button opens the corresponding native page.
For Step use `/meteor toggle step`; for ReverseStep use `/meteor toggle reverse-step`.
Original Meteor settings are never automatically read from another installation.
Use `/meteor import modules "PATH_TO_modules.nbt"` or `/meteor import friends "PATH_TO_friends.nbt"`
to intentionally import a selected file. Raw/gzip NBT, implemented primitive settings, supported
keybinds/modifiers, and friend names/UUIDs are supported; unsupported modules/settings are reported.
Parsing and candidate validation precede live mutation. Generated-fixture live import checks passed,
including Zoom behavior, modifier binding, friend deduplication, preservation of active modules during
friends-only import, and rejection of truncated NBT without changing configuration.

## Native `.hmod` distribution

`tools/package_meteor_hmod.py` creates the source-only `native-source-hmod/1` package described in
`docs/native-source-hmod.md`. It pins the original Horizon revision and hashes every changed file.
It requires the modified launcher in this source fork; stock content-only `.hmod` loaders reject it.
The package validator and source staging are separate from compiling and installing a native build.
The 14-module package snapshot with archive SHA256
`eda8ec04e70700bcbd851a3dd3c95afa30e3d1c5770cac2e3cb02f64ce25241a` passed the complete
isolated launcher build/install pipeline using pinned Rust 1.99.1 GNU. A cached rerun using the
updated launcher passed installation checks with synthetic private files excluded. No game was
launched during those packaging checks. This package evidence is separate from newer preview builds.
Neither the package extension nor passing unit tests proves complete Meteor parity.

The runtime driver is `tools/test_meteor_runtime.py`. After building, run it with
`--resources PATH_TO_PREPARED_RESOURCES --air-jump`; it creates a fresh scratch world under `work/`
and closes only its own test client. It requires local prepared Minecraft resources, which are not
included. The recorded smoke test used the same driver before its local resource paths were
converted to command-line arguments. Runtime results are in `crates/meteor-client/runtime-smoke.json`.

## Shared Timer event order

FastClimb and NoSlow share Timer's override. Their Tick.Pre handlers now run in actual module
subscription order, including disabling/re-enabling within one frame. Each fresh subscription
restarts its module's activation state; disabling does not invent an override reset. World leave
clears listeners, and world join resubscribes active modules in native catalog order (Meteor's
identity-hash-map traversal has no portable ordering). The earlier 21-module checkpoint used fixed
FastClimb-before-NoSlow order; use the subsequent source revision for activation-order parity.

## Attribution and license

The Meteor-derived Rust code is GPL-3.0-only, with the upstream copyright notices preserved.
The full GPL v3 text is `crates/meteor-client/COPYING`. Meteor's source:
https://github.com/MeteorDevelopment/meteor-client . This is an unofficial port, not an upstream release.
The existing Horizon and third-party files retain their license notices. Do not describe a combined
Meteor-enabled distribution as MIT-only. Include corresponding source and applicable notices when
distributing a combined build. Minecraft-derived resources remain local and outside source bundles.

Post-checkpoint 28 dispatcher change: Step, ReverseStep, FastClimb, Spider, Parkour, Flight, AutoClicker and Fullbright now receive Tick.Post in activation subscription order, including reactivation moving a callback to the end. Step deactivation restoration remains independent of active event dispatch. Core regressions verify competing Flight/Spider velocity writes and event snapshot behavior; the --post-order isolated runtime fixture is prepared but has not yet run against a rebuilt checkpoint. Immutable checkpoint 28 retains the earlier fixed host ordering.
