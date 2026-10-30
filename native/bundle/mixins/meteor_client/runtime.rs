// SPDX-License-Identifier: GPL-3.0-only

// Native Meteor migration proof. Reuses the existing Meteor-derived Zoom, AutoJump, AirJump and HighJump core.

// This candidate integrates Zoom, AutoJump, AirJump and HighJump only; other retained core modules are inactive.

use super::{
    Game,
    native_hooks::{NativeMod, NativeZoom},
};

use meteor_client::modules::{
    air_jump, auto_jump, high_jump,
    zoom::{Zoom, ZoomSettings},
};

use meteor_client::subscriptions::Subscriptions;

use std::{
    cell::RefCell,
    collections::HashMap,
    fs,
    path::PathBuf,
    rc::Rc,
    sync::{Arc, Weak},
    time::Duration,
};

use winit::window::{Window, WindowId};

const MOD_ID: &str = "meteor_client";

const COMMAND: &str = "meteor-migration";

const HELP: &str = "Zoom+AutoJump+AirJump+HighJump .hnmod migration candidate: /meteor-migration status | zoom toggle | zoom factor NUMBER | zoom smooth true|false | zoom scroll-sensitivity NUMBER | auto-jump toggle | auto-jump mode Jump|LowHop | auto-jump jump-if Always|Walking|Sprinting | auto-jump velocity-height NUMBER | air-jump toggle | air-jump maintain-level true|false | high-jump toggle | high-jump jump-multiplier NUMBER. AirJump keyboard presses use configured jump/sneak bindings. Native API caps visible Zoom at 10. Other retained Meteor modules are not integrated.";

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]

struct Saved {
    schema: u32,

    enabled: bool,

    zoom: ZoomSettings,
    auto_jump_enabled: bool,
    auto_jump: auto_jump::Settings,
    air_jump_enabled: bool,
    air_jump: air_jump::Settings,
    high_jump_enabled: bool,
    high_jump: high_jump::Settings,
}

impl Default for Saved {
    fn default() -> Self {
        Self {
            schema: 1,
            enabled: false,
            zoom: ZoomSettings::default(),
            auto_jump_enabled: false,
            auto_jump: auto_jump::Settings::default(),
            air_jump_enabled: false,
            air_jump: air_jump::Settings::default(),
            high_jump_enabled: false,
            high_jump: high_jump::Settings::default(),
        }
    }
}

impl Saved {
    fn validate(&self) -> Result<(), String> {
        if self.schema != 1 {
            return Err("Unsupported Meteor migration proof schema".into());
        }
        self.zoom.validate()?;
        self.auto_jump.validate()?;
        self.high_jump.validate()?;
        Ok(())
    }
}

struct Runtime {
    saved: Saved,

    zoom: Zoom,
    air_jump: air_jump::AirJump,
    movement_subscriptions: Subscriptions,
    auto_jump_subscribed: bool,
    air_jump_subscribed: bool,

    in_world: bool,

    path: PathBuf,

    save_allowed: bool,
}

impl Runtime {
    fn load(path: PathBuf) -> Self {
        let loaded = match fs::read_to_string(&path) {
            Ok(text) => serde_json::from_str::<Saved>(&text)
                .map_err(|e| e.to_string())
                .and_then(|value| {
                    value.validate()?;
                    Ok(value)
                }),

            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Saved::default()),

            Err(error) => Err(error.to_string()),
        };

        let (saved, save_allowed) = match loaded {
            Ok(value) => (value, true),

            Err(error) => {
                log::error!(
                    "Meteor Zoom+AutoJump+AirJump+HighJump migration candidate could not read {}: {error}; saving disabled",
                    path.display()
                );
                (Saved::default(), false)
            }
        };

        Self {
            saved,
            zoom: Zoom::default(),
            air_jump: air_jump::AirJump::default(),
            movement_subscriptions: Subscriptions::default(),
            auto_jump_subscribed: false,
            air_jump_subscribed: false,
            in_world: false,
            path,
            save_allowed,
        }
    }

    fn world(&mut self, in_world: bool) {
        if in_world == self.in_world {
            return;
        }

        self.in_world = in_world;

        if in_world && self.saved.enabled {
            self.zoom.activate(&self.saved.zoom);
        } else if !in_world {
            self.zoom.stop();
        }
    }

    fn toggle(&mut self) {
        self.saved.enabled = !self.saved.enabled;

        if self.saved.enabled && self.in_world {
            self.zoom.activate(&self.saved.zoom);
        }
    }

    fn sync_movement(&mut self, in_world: bool, block_y: Option<i32>) {
        if !in_world {
            self.movement_subscriptions.clear();
            self.auto_jump_subscribed = false;
            self.air_jump_subscribed = false;
            self.air_jump.sync(false, None);
            return;
        }
        // Fresh world joins use the retained native catalog fallback. Java joins
        // traverse Reference2ReferenceOpenHashMap.values(), whose class-identity
        // hash order is not portable across processes. No exact join-order claim.
        // Later enable/re-enable events append in their actual activation order.
        if self.saved.auto_jump_enabled != self.auto_jump_subscribed {
            if self.saved.auto_jump_enabled {
                self.movement_subscriptions.activate("auto-jump");
            } else {
                self.movement_subscriptions.deactivate("auto-jump");
            }
            self.auto_jump_subscribed = self.saved.auto_jump_enabled;
        }
        if self.saved.air_jump_enabled != self.air_jump_subscribed {
            if self.saved.air_jump_enabled {
                self.movement_subscriptions.activate("air-jump");
                self.air_jump.sync(false, None);
            } else {
                self.movement_subscriptions.deactivate("air-jump");
            }
            self.air_jump_subscribed = self.saved.air_jump_enabled;
        }
        self.air_jump.sync(self.saved.air_jump_enabled, block_y);
    }

    fn status(&self) -> String {
        format!(
            "Zoom+AutoJump+AirJump+HighJump .hnmod migration candidate: Zoom {}; requested {}, current {:.3} (native API clamps visible factor to 10); smoothing {}; AutoJump {} ({:?}, {:?}, LowHop height {}); AirJump {} (maintain-level {}, captured level {:?}); HighJump {} (multiplier {}); Pre order {:?}; other retained modules are not integrated. Zoom cinematic/hide-HUD/show-hands are not wired in this proof.",
            if self.saved.enabled {
                "enabled"
            } else {
                "disabled"
            },
            self.saved.zoom.zoom,
            self.zoom.scaling(),
            self.saved.zoom.smooth,
            if self.saved.auto_jump_enabled {
                "enabled"
            } else {
                "disabled"
            },
            self.saved.auto_jump.mode,
            self.saved.auto_jump.jump_if,
            self.saved.auto_jump.velocity_height,
            if self.saved.air_jump_enabled {
                "enabled"
            } else {
                "disabled"
            },
            self.saved.air_jump.maintain_level,
            self.air_jump.level,
            if self.saved.high_jump_enabled {
                "enabled"
            } else {
                "disabled"
            },
            self.saved.high_jump.jump_multiplier,
            self.movement_subscriptions
                .entries()
                .iter()
                .map(|entry| entry.id.as_str())
                .collect::<Vec<_>>()
        )
    }
}

struct Entry<W, S> {
    window: Weak<W>,
    runtime: Rc<RefCell<S>>,
}
struct WindowTable<K, W, S> {
    entries: HashMap<K, Entry<W, S>>,
}
impl<K: std::hash::Hash + Eq, W, S> WindowTable<K, W, S> {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }
    fn find(&mut self, key: &K, window: &Arc<W>) -> Option<Rc<RefCell<S>>> {
        self.entries
            .retain(|_, entry| entry.window.strong_count() > 0);
        self.entries.get(key).and_then(|entry| {
            entry
                .window
                .upgrade()
                .filter(|old| Arc::ptr_eq(old, window))
                .map(|_| Rc::clone(&entry.runtime))
        })
    }
    fn insert(&mut self, key: K, window: &Arc<W>, runtime: Rc<RefCell<S>>) {
        self.entries.insert(
            key,
            Entry {
                window: Arc::downgrade(window),
                runtime,
            },
        );
    }
}
thread_local! {
    static WINDOWS: RefCell<WindowTable<WindowId,Window,Runtime>> = RefCell::new(WindowTable::new());
}
/// Window's Arc allocation is stable if Game moves. Weak ownership avoids keeping

/// windows alive; pruning plus ptr_eq handles a recycled WindowId safely.

fn runtime(game: &Game<'_>) -> Option<Rc<RefCell<Runtime>>> {
    if !game.native_window() {
        return None;
    }

    let window = game.window.as_ref()?;

    let key = window.id();

    let existing = WINDOWS.with(|windows| windows.borrow_mut().find(&key, window));

    if existing.is_some() {
        return existing;
    }

    // No TLS table borrow exists while invoking even the stable host data helper.

    let path = game.native_data_dir(MOD_ID).join("zoom-migration.json");

    let runtime = Rc::new(RefCell::new(Runtime::load(path)));

    WINDOWS.with(|windows| {
        windows
            .borrow_mut()
            .insert(key, window, Rc::clone(&runtime));
    });

    Some(runtime)
}

/// Serialize while borrowed, then release the state before filesystem work or chat.

fn save(runtime: &Rc<RefCell<Runtime>>) -> Result<(), String> {
    let (path, text) = {
        let runtime = runtime.borrow();

        if !runtime.save_allowed {
            return Err("Saving disabled to preserve an unreadable migration-proof config".into());
        }

        (
            runtime.path.clone(),
            serde_json::to_string_pretty(&runtime.saved).map_err(|e| e.to_string())?,
        )
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    fs::write(path, text).map_err(|e| e.to_string())
}

fn refresh_high_jump(game: &mut Game<'_>) {
    if !game.native_window() {
        return;
    }
    let Some(runtime) = runtime(game) else {
        horizon_client::motion::meteor_highjump::clear();
        return;
    };
    let (active, settings) = {
        let runtime = runtime.borrow();
        (
            runtime.saved.high_jump_enabled,
            runtime.saved.high_jump.clone(),
        )
    };
    // The sibling publisher clears first and checks current actor identity. No
    // Runtime borrow is held while it accesses the host's attribute instance.
    if let Err(error) = super::meteor_highjump_publisher::publish(game, active, &settings) {
        horizon_client::motion::meteor_highjump::clear();
        log::error!("Meteor HighJump movement candidate policy refresh failed: {error}");
    }
}

fn air_player(game: &Game<'_>) -> Option<air_jump::Player> {
    game.player
        .as_ref()
        .filter(|_| game.title.is_none())
        .map(|player| air_jump::Player {
            block_y: player.position.y.floor() as i32,
            on_ground: player.ground.on_ground,
            // Freecam is not integrated/registered in this limited standard bundle.
            freecam: false,
        })
}
fn sync_movement(game: &Game<'_>, runtime: &Rc<RefCell<Runtime>>) {
    let player = air_player(game);
    let in_world = game.native_on(MOD_ID) && player.is_some();
    let y = player.map(|player| player.block_y);
    runtime.borrow_mut().sync_movement(in_world, y);
}
/// One source Tick.Pre dispatcher. Equal-priority subscribers run in their real
/// activation order, using a snapshot so subscriptions changed by callbacks wait.
pub(super) fn on_pre_tick(game: &mut Game<'_>) {
    refresh_high_jump(game);
    if !game.native_window() {
        return;
    }
    let Some(runtime) = runtime(game) else {
        return;
    };
    sync_movement(game, &runtime);
    if !game.native_on(MOD_ID)
        || game.agent.hold_server
        || game.title.is_some()
        || game.player.is_none()
    {
        return;
    }
    let listeners = { runtime.borrow().movement_subscriptions.entries().to_vec() };
    for listener in listeners {
        match listener.id.as_str() {
            "auto-jump" => {
                let Some(player) = game.player.as_ref() else {
                    continue;
                };
                let player = auto_jump::Player {
                    on_ground: player.ground.on_ground,
                    sneaking: player.shift_key_down,
                    sprinting: player.sprinting,
                    forward: player.move_vector[1],
                    sideways: player.move_vector[0],
                };
                let action = {
                    let runtime = runtime.borrow();
                    auto_jump::tick(&player, &runtime.saved.auto_jump)
                };
                match action {
                    Some(auto_jump::Action::SetVerticalVelocity(y)) => {
                        if let Some(player) = game.player.as_mut() {
                            player.velocity.y = y;
                        }
                    }
                    Some(auto_jump::Action::JumpFromGround) => {
                        if let Err(error) = jump_from_ground(game) {
                            log::error!(
                                "Meteor AutoJump migration candidate native jump failed: {error}"
                            );
                        }
                    }
                    None => {}
                }
            }
            "air-jump" => {
                let Some(player) = air_player(game) else {
                    continue;
                };
                // Stable native mapping queries the translated action, including
                // keyboard/mouse rebinding; it is not physical Space detection.
                let jump_down = game
                    .native_mapping("key.jump")
                    .is_some_and(|(down, _)| down);
                let should_jump = {
                    let runtime = runtime.borrow();
                    runtime
                        .air_jump
                        .tick(player, &runtime.saved.air_jump, jump_down)
                };
                if should_jump && let Err(error) = jump_from_ground(game) {
                    log::error!("Meteor AirJump maintain-level native jump failed: {error}");
                }
            }
            _ => {}
        }
    }
}
/// Native key@1's keyboard wrapper drops Repeat. AirJump's Java KeyInputEvent
/// handler observes keyboard Press only, so mouse names never trigger this arm.
pub(super) fn on_key(game: &mut Game<'_>, key: &str, pressed: bool) {
    refresh_high_jump(game);
    if !game.native_window() {
        return;
    }
    let Some(runtime) = runtime(game) else {
        return;
    };
    sync_movement(game, &runtime);
    if !game.native_on(MOD_ID) || !key.starts_with("key.keyboard.") {
        return;
    }
    let Some(player) = air_player(game) else {
        return;
    };
    let screen_open = !game.native_playing();
    let jump = key == game.settings.keys.get("key.jump");
    let sneak = key == game.settings.keys.get("key.sneak");
    let should_jump = {
        let mut runtime = runtime.borrow_mut();
        if !runtime.saved.air_jump_enabled {
            false
        } else {
            runtime
                .air_jump
                .key(player, pressed, screen_open, jump, sneak)
        }
    };
    // Every state/table borrow ends before this potentially reentrant host call.
    if should_jump && let Err(error) = jump_from_ground(game) {
        log::error!("Meteor AirJump key native jump failed: {error}");
    }
}
fn jump_from_ground(game: &mut Game<'_>) -> horizon_client::Result<()> {
    refresh_high_jump(game);
    let entities = game.player_entity_collisions();
    if let Some(player) = game.player.as_mut() {
        horizon_client::motion::MotionWorld {
            entities: &entities,
            player,
            scene: &mut game.world,
            assets: &game.pack.assets,
            collision_binding: game.collision,
            resources: game.motion,
        }
        .jump_from_ground()?;
    }
    Ok(())
}

pub(super) fn on_frame(game: &mut Game<'_>, elapsed: Duration) {
    refresh_high_jump(game);
    let Some(runtime) = runtime(game) else {
        return;
    };

    sync_movement(game, &runtime);
    let in_world = game.native_on(MOD_ID) && game.player.is_some() && game.title.is_none();

    {
        let mut runtime = runtime.borrow_mut();
        runtime.world(in_world);
        let active = runtime.saved.enabled;
        let settings = runtime.saved.zoom.clone();
        runtime.zoom.frame(elapsed.as_secs_f64(), active, &settings);
    }
}

pub(super) fn on_tick(game: &mut Game<'_>) {
    refresh_high_jump(game);
    let Some(runtime) = runtime(game) else {
        return;
    };

    sync_movement(game, &runtime);
    let in_world = game.native_on(MOD_ID) && game.player.is_some() && game.title.is_none();

    {
        let mut runtime = runtime.borrow_mut();
        runtime.world(in_world);
        runtime.zoom.tick();
    }
}

pub(super) fn on_zoom(game: &Game<'_>, output: &mut Option<NativeZoom>) {
    if output.is_some() || !game.native_on(MOD_ID) {
        return;
    }

    let Some(runtime) = runtime(game) else {
        return;
    };

    let value = {
        let runtime = runtime.borrow();
        runtime.zoom.subscribed.then(|| NativeZoom {
            factor: runtime.zoom.scaling() as f32,
            smooth: false,
            turn: runtime.zoom.sensitivity_multiplier() as f32,
        })
    };

    *output = value;
}

pub(super) fn on_wheel(game: &mut Game<'_>, steps: f32, claimed: &mut bool) {
    if *claimed || !game.native_on(MOD_ID) {
        return;
    }

    let Some(runtime) = runtime(game) else {
        return;
    };

    let screen_open = !game.native_playing();

    let handled = {
        let mut runtime = runtime.borrow_mut();
        let settings = runtime.saved.zoom.clone();
        let active = runtime.saved.enabled;
        runtime
            .zoom
            .scroll(f64::from(steps), active, screen_open, &settings)
    };

    if handled {
        *claimed = true;
    }
}

pub(super) fn on_command(game: &mut Game<'_>, command: &str, ran: &mut bool) {
    refresh_high_jump(game);
    if *ran || !game.native_on(MOD_ID) || !game.native_window() {
        return;
    }

    let words = command.split_whitespace().collect::<Vec<_>>();

    if words.first().copied() != Some(COMMAND) {
        return;
    }

    *ran = true;

    let Some(runtime) = runtime(game) else {
        game.native_chat(
            "Zoom+AutoJump+AirJump+HighJump migration candidate: no window runtime is available yet.",
        );
        return;
    };

    sync_movement(game, &runtime);
    let result = (|| -> Result<String, String> {
        if words.len() == 1 || words.get(1) == Some(&"help") {
            return Ok(HELP.into());
        }

        if words.as_slice() == [COMMAND, "status"] {
            return Ok(runtime.borrow().status());
        }

        {
            let mut runtime = runtime.borrow_mut();

            match &words[1..] {
                ["zoom", "toggle"] => runtime.toggle(),

                ["zoom", "factor", value] => {
                    let mut settings = runtime.saved.zoom.clone();
                    settings.zoom = value.parse().map_err(|_| "Expected a finite zoom number")?;
                    settings.validate()?;
                    runtime.saved.zoom = settings;
                }

                ["zoom", "smooth", value] => {
                    runtime.saved.zoom.smooth =
                        value.parse().map_err(|_| "Expected true or false")?;
                }

                ["zoom", "scroll-sensitivity", value] => {
                    let mut settings = runtime.saved.zoom.clone();
                    settings.scroll_sensitivity = value
                        .parse()
                        .map_err(|_| "Expected a nonnegative scroll sensitivity")?;
                    settings.validate()?;
                    runtime.saved.zoom = settings;
                }

                ["auto-jump", "toggle"] => {
                    runtime.saved.auto_jump_enabled = !runtime.saved.auto_jump_enabled
                }
                ["auto-jump", "mode", value] => {
                    runtime.saved.auto_jump.mode = match *value {
                        "Jump" => auto_jump::Mode::Jump,
                        "LowHop" => auto_jump::Mode::LowHop,
                        _ => return Err("AutoJump mode must be Jump or LowHop".into()),
                    };
                }
                ["auto-jump", "jump-if", value] => {
                    runtime.saved.auto_jump.jump_if = match *value {
                        "Always" => auto_jump::JumpWhen::Always,
                        "Walking" => auto_jump::JumpWhen::Walking,
                        "Sprinting" => auto_jump::JumpWhen::Sprinting,
                        _ => {
                            return Err(
                                "AutoJump jump-if must be Always, Walking or Sprinting".into()
                            );
                        }
                    };
                }
                ["auto-jump", "velocity-height", value] => {
                    let mut settings = runtime.saved.auto_jump.clone();
                    settings.velocity_height = value
                        .parse()
                        .map_err(|_| "Expected a finite nonnegative LowHop velocity height")?;
                    settings.validate()?;
                    runtime.saved.auto_jump = settings;
                }
                ["air-jump", "toggle"] => {
                    runtime.saved.air_jump_enabled = !runtime.saved.air_jump_enabled
                }
                ["air-jump", "maintain-level", value] => {
                    runtime.saved.air_jump.maintain_level =
                        value.parse().map_err(|_| "Expected true or false")?
                }
                ["high-jump", "toggle"] => {
                    runtime.saved.high_jump_enabled = !runtime.saved.high_jump_enabled
                }
                ["high-jump", "jump-multiplier", value] => {
                    let mut settings = runtime.saved.high_jump.clone();
                    settings.jump_multiplier = value
                        .parse()
                        .map_err(|_| "Expected a finite nonnegative jump multiplier")?;
                    settings.validate()?;
                    runtime.saved.high_jump = settings;
                }
                _ => return Err(HELP.into()),
            }
        }

        sync_movement(game, &runtime);
        refresh_high_jump(game);
        save(&runtime)?;

        Ok(runtime.borrow().status())
    })();

    // Host chat may trigger another injected helper; no Runtime/TLS borrow is held.

    match result {
        Ok(message) => game.native_chat(&message),
        Err(error) => game.native_chat(&format!(
            "Zoom+AutoJump+AirJump+HighJump migration candidate: {error}"
        )),
    }
}

pub(super) fn on_mods(_game: &mut Game<'_>, mods: &mut Vec<NativeMod>) {
    if mods.iter().any(|module| module.id == MOD_ID) {
        return;
    }

    mods.push(NativeMod {id:MOD_ID.into(),title:"Meteor Rust - Zoom+AutoJump+AirJump+HighJump migration candidate".into(),description:"Normal .hnmod candidate using the existing Meteor Rust core. Zoom, AutoJump, AirJump and HighJump only; other retained modules are inactive. HighJump uses current native actor identity, source block factor and additive boost/zero-power sequencing. Movement integration uses ordered Pre and real native jump paths. Native API caps visible Zoom at 10. Runtime validation pending.".into(),keys:Vec::new()});
}

pub(super) fn on_settings(game: &mut Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    refresh_high_jump(game);
    if mod_id != MOD_ID {
        return;
    }

    let Some(runtime) = runtime(game) else {
        buttons.push(
            "Zoom+AutoJump+AirJump+HighJump migration candidate: open the game window first".into(),
        );
        return;
    };

    sync_movement(game, &runtime);
    let labels = {
        let runtime = runtime.borrow();
        vec![
            format!(
                "Zoom proof: {}",
                if runtime.saved.enabled {
                    "enabled"
                } else {
                    "disabled"
                }
            ),
            format!(
                "Zoom factor: {} (visible API cap 10)",
                runtime.saved.zoom.zoom
            ),
            format!("Meteor smoothing: {}", runtime.saved.zoom.smooth),
            format!(
                "AutoJump: {}",
                if runtime.saved.auto_jump_enabled {
                    "enabled"
                } else {
                    "disabled"
                }
            ),
            format!("AutoJump mode: {:?}", runtime.saved.auto_jump.mode),
            format!("AutoJump jump if: {:?}", runtime.saved.auto_jump.jump_if),
            format!(
                "AutoJump LowHop velocity: {}",
                runtime.saved.auto_jump.velocity_height
            ),
            format!(
                "AirJump: {}",
                if runtime.saved.air_jump_enabled {
                    "enabled"
                } else {
                    "disabled"
                }
            ),
            format!(
                "AirJump maintain level: {}",
                runtime.saved.air_jump.maintain_level
            ),
            format!(
                "HighJump: {}",
                if runtime.saved.high_jump_enabled {
                    "enabled"
                } else {
                    "disabled"
                }
            ),
            format!(
                "HighJump multiplier: {}",
                runtime.saved.high_jump.jump_multiplier
            ),
        ]
    };

    buttons.extend(labels);
}

pub(super) fn on_settings_click(game: &mut Game<'_>, mod_id: &str, button: usize) {
    refresh_high_jump(game);
    if mod_id != MOD_ID || !game.native_on(MOD_ID) {
        return;
    }

    let Some(runtime) = runtime(game) else {
        return;
    };

    {
        let mut runtime = runtime.borrow_mut();

        match button {
            0 => runtime.toggle(),
            1 => {
                let current = runtime.saved.zoom.zoom;
                runtime.saved.zoom.zoom = [2.0, 4.0, 6.0, 8.0, 10.0]
                    .into_iter()
                    .find(|factor| *factor > current)
                    .unwrap_or(2.0);
            }
            2 => runtime.saved.zoom.smooth = !runtime.saved.zoom.smooth,
            3 => runtime.saved.auto_jump_enabled = !runtime.saved.auto_jump_enabled,
            4 => {
                runtime.saved.auto_jump.mode = match runtime.saved.auto_jump.mode {
                    auto_jump::Mode::Jump => auto_jump::Mode::LowHop,
                    auto_jump::Mode::LowHop => auto_jump::Mode::Jump,
                }
            }
            5 => {
                runtime.saved.auto_jump.jump_if = match runtime.saved.auto_jump.jump_if {
                    auto_jump::JumpWhen::Always => auto_jump::JumpWhen::Walking,
                    auto_jump::JumpWhen::Walking => auto_jump::JumpWhen::Sprinting,
                    auto_jump::JumpWhen::Sprinting => auto_jump::JumpWhen::Always,
                }
            }
            6 => {
                let height = runtime.saved.auto_jump.velocity_height;
                runtime.saved.auto_jump.velocity_height = [0.0, 0.25, 0.5, 1.0]
                    .into_iter()
                    .find(|value| *value > height)
                    .unwrap_or(0.0);
            }
            7 => runtime.saved.air_jump_enabled = !runtime.saved.air_jump_enabled,
            8 => runtime.saved.air_jump.maintain_level = !runtime.saved.air_jump.maintain_level,
            9 => runtime.saved.high_jump_enabled = !runtime.saved.high_jump_enabled,
            10 => {
                let factor = runtime.saved.high_jump.jump_multiplier;
                runtime.saved.high_jump.jump_multiplier = [0.0, 0.5, 1.0, 2.0, 3.0]
                    .into_iter()
                    .find(|value| *value > factor)
                    .unwrap_or(0.0);
            }
            _ => return,
        }
    }

    sync_movement(game, &runtime);
    refresh_high_jump(game);
    if let Err(error) = save(&runtime) {
        game.native_chat(&format!(
            "Zoom+AutoJump+AirJump+HighJump migration candidate: {error}"
        ));
    }
}

#[cfg(test)]
mod migration_lifecycle_tests {
    use super::*;
    #[test]
    fn state_follows_stable_window_anchor_when_game_owner_moves() {
        struct Owner {
            window: Arc<()>,
        }
        let owner = Owner {
            window: Arc::new(()),
        };
        let state = Rc::new(RefCell::new(7_u32));
        let mut table = WindowTable::new();
        table.insert(1_u8, &owner.window, Rc::clone(&state));
        let relocated = Box::new(owner);
        let found = table.find(&1, &relocated.window).unwrap();
        assert!(Rc::ptr_eq(&found, &state));
        *found.borrow_mut() = 12;
        assert_eq!(*state.borrow(), 12);
    }
    #[test]
    fn recycled_window_id_cannot_inherit_another_live_window_state() {
        let old = Arc::new(());
        let replacement = Arc::new(());
        let mut table = WindowTable::new();
        table.insert(2_u8, &old, Rc::new(RefCell::new(9_u32)));
        assert!(table.find(&2, &replacement).is_none());
        table.insert(2, &replacement, Rc::new(RefCell::new(0)));
        assert_eq!(*table.find(&2, &replacement).unwrap().borrow(), 0);
    }
    #[test]
    fn dropped_window_is_pruned_without_retaining_window_or_runtime() {
        let window = Arc::new(());
        let weak = Arc::downgrade(&window);
        let state = Rc::new(RefCell::new(1_u32));
        let state_weak = Rc::downgrade(&state);
        let mut table = WindowTable::new();
        table.insert(3_u8, &window, state);
        drop(window);
        assert!(weak.upgrade().is_none());
        assert!(table.find(&4, &Arc::new(())).is_none());
        assert!(table.entries.is_empty());
        assert!(state_weak.upgrade().is_none());
    }
    #[test]
    fn host_callback_can_reenter_after_table_and_runtime_guards_end() {
        let window = Arc::new(());
        let table = RefCell::new(WindowTable::new());
        table
            .borrow_mut()
            .insert(4_u8, &window, Rc::new(RefCell::new(2_u32)));
        let state = { table.borrow_mut().find(&4, &window).unwrap() };
        let effect = {
            let mut state = state.borrow_mut();
            *state += 1;
            *state
        };
        let callback = || {
            let nested = { table.borrow_mut().find(&4, &window).unwrap() };
            let value = *nested.borrow();
            value
        };
        assert_eq!(effect, 3);
        assert_eq!(callback(), 3);
    }
}

#[cfg(test)]
mod migration_autojump_config_tests {
    use super::*;
    #[test]
    fn older_zoom_schema_one_config_gains_disabled_source_default_autojump() {
        let saved: Saved =
            serde_json::from_str(r#"{"schema":1,"enabled":true,"zoom":{"zoom":4.0}}"#).unwrap();
        saved.validate().unwrap();
        assert!(saved.enabled);
        assert_eq!(saved.zoom.zoom, 4.0);
        assert!(!saved.auto_jump_enabled);
        assert!(matches!(saved.auto_jump.mode, auto_jump::Mode::Jump));
        assert!(matches!(
            saved.auto_jump.jump_if,
            auto_jump::JumpWhen::Always
        ));
        assert_eq!(saved.auto_jump.velocity_height, 0.25);
    }
    #[test]
    fn combined_config_validation_rejects_negative_autojump_without_changing_zoom() {
        let saved:Saved=serde_json::from_str(r#"{"schema":1,"enabled":true,"zoom":{"zoom":4.0},"auto_jump":{"velocity_height":-0.1}}"#).unwrap();
        assert!(saved.validate().is_err());
        assert_eq!(saved.zoom.zoom, 4.0);
    }
}

#[cfg(test)]
mod migration_airjump_state_tests {
    use super::*;
    fn state() -> Runtime {
        Runtime {
            saved: Saved::default(),
            zoom: Zoom::default(),
            air_jump: air_jump::AirJump::default(),
            movement_subscriptions: Subscriptions::default(),
            auto_jump_subscribed: false,
            air_jump_subscribed: false,
            in_world: false,
            path: PathBuf::from("unused-airjump-state-test"),
            save_allowed: false,
        }
    }
    fn order(state: &Runtime) -> Vec<&str> {
        state
            .movement_subscriptions
            .entries()
            .iter()
            .map(|entry| entry.id.as_str())
            .collect()
    }
    #[test]
    fn older_autojump_config_gains_disabled_airjump_and_source_default() {
        let saved:Saved=serde_json::from_str(r#"{"schema":1,"auto_jump_enabled":true,"auto_jump":{"mode":"LowHop","velocity_height":0.0}}"#).unwrap();
        saved.validate().unwrap();
        assert!(saved.auto_jump_enabled);
        assert!(!saved.air_jump_enabled);
        assert!(!saved.air_jump.maintain_level);
    }
    #[test]
    fn enabling_airjump_captures_height_before_first_frame_or_pre_callback() {
        let mut state = state();
        state.saved.air_jump_enabled = true;
        state.sync_movement(true, Some(74));
        assert_eq!(state.air_jump.level, Some(74));
        assert_eq!(order(&state), ["air-jump"]);
        assert!(state.air_jump.key(
            air_jump::Player {
                block_y: 77,
                on_ground: false,
                freecam: false
            },
            true,
            false,
            true,
            false
        ));
        assert_eq!(state.air_jump.level, Some(77));
    }
    #[test]
    fn active_sync_does_not_recapture_sneak_lowered_level() {
        let mut state = state();
        state.saved.air_jump_enabled = true;
        state.sync_movement(true, Some(74));
        assert!(!state.air_jump.key(
            air_jump::Player {
                block_y: 74,
                on_ground: false,
                freecam: false
            },
            true,
            false,
            false,
            true
        ));
        assert_eq!(state.air_jump.level, Some(73));
        state.sync_movement(true, Some(74));
        assert_eq!(state.air_jump.level, Some(73));
    }
    #[test]
    fn every_disable_reenable_restarts_activation_and_moves_pre_listener_to_end() {
        let mut state = state();
        state.saved.air_jump_enabled = true;
        state.sync_movement(true, Some(74));
        state.saved.auto_jump_enabled = true;
        state.sync_movement(true, Some(74));
        assert_eq!(order(&state), ["air-jump", "auto-jump"]);
        let old = state.movement_subscriptions.generation("air-jump").unwrap();
        state.saved.air_jump_enabled = false;
        state.sync_movement(true, Some(74));
        assert_eq!(state.air_jump.level, None);
        state.saved.air_jump_enabled = true;
        state.sync_movement(true, Some(81));
        assert_eq!(state.air_jump.level, Some(81));
        assert_eq!(order(&state), ["auto-jump", "air-jump"]);
        assert!(state.movement_subscriptions.generation("air-jump").unwrap() > old);
    }
    #[test]
    fn world_leave_clears_runtime_and_rejoin_recaptures_without_losing_enabled_settings() {
        let mut state = state();
        state.saved.air_jump_enabled = true;
        state.saved.auto_jump_enabled = true;
        state.sync_movement(true, Some(74));
        state.sync_movement(false, None);
        assert!(order(&state).is_empty());
        assert_eq!(state.air_jump.level, None);
        assert!(state.saved.air_jump_enabled);
        state.sync_movement(true, Some(-12));
        assert_eq!(state.air_jump.level, Some(-12));
        assert_eq!(order(&state), ["auto-jump", "air-jump"]);
    }
}

#[cfg(test)]
mod migration_highjump_config_tests {
    use super::*;
    #[test]
    fn old_airjump_config_defaults_highjump_disabled_and_multiplier_one() {
        let saved:Saved=serde_json::from_str(r#"{"schema":1,"air_jump_enabled":true,"air_jump":{"maintain_level":true},"auto_jump_enabled":true}"#).unwrap();
        saved.validate().unwrap();
        assert!(saved.air_jump_enabled);
        assert!(saved.air_jump.maintain_level);
        assert!(saved.auto_jump_enabled);
        assert!(!saved.high_jump_enabled);
        assert_eq!(saved.high_jump.jump_multiplier, 1.0);
    }
    #[test]
    fn saved_validation_checks_negative_and_nonfinite_highjump_multipliers() {
        for multiplier in [-0.01, f64::NAN, f64::INFINITY] {
            let mut saved = Saved::default();
            saved.zoom.zoom = 4.0;
            saved.high_jump.jump_multiplier = multiplier;
            assert!(saved.validate().is_err());
            assert_eq!(saved.zoom.zoom, 4.0);
        }
    }
}
