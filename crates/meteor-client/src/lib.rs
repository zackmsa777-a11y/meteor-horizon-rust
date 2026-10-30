// SPDX-License-Identifier: GPL-3.0-only
// Derived from Meteor Client, Copyright (c) Meteor Development.
//! Engine-independent behavior. The Horizon adapter owns engine mutations.
//! Unported modules are tracked in port-inventory.json, not registered as empty implementations.
pub mod colors;
pub mod damage;
pub mod friends;
pub mod hud;
pub mod hud_box;
pub mod hud_metrics;
pub mod hud_snapper;
pub mod hud_text;
pub mod macros;
pub mod modules;
pub mod profiles;
pub mod registry;
pub mod starscript;
pub mod subscriptions;

use modules::zoom::{Zoom, ZoomSettings};
use modules::{
    air_jump, auto_jump, auto_respawn, auto_walk, break_delay, fast_climb, fast_use, fullbright,
    gui_move, high_jump, parkour, reverse_step, safe_walk, spider, sprint, step, timer,
};
use modules::{no_slow, slippy, sneak};
use registry::{Module, Registry};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub schema: u32,
    pub modules: Registry,
    pub hud: hud::Config,
    pub friends: friends::Friends,
    pub macros: macros::Macros,
    pub zoom: ZoomSettings,
    pub auto_jump: auto_jump::Settings,
    pub air_jump: air_jump::Settings,
    pub sprint: sprint::Settings,
    pub gui_move: gui_move::Settings,
    pub high_jump: high_jump::Settings,
    pub auto_walk: auto_walk::Settings,
    pub safe_walk: safe_walk::Settings,
    pub step: step::Settings,
    pub reverse_step: reverse_step::Settings,
    pub fast_climb: fast_climb::Settings,
    pub spider: spider::Settings,
    pub parkour: parkour::Settings,
    pub timer: timer::Settings,
    pub auto_respawn: auto_respawn::Settings,
    pub flight: modules::flight::Settings,
    pub auto_clicker: modules::auto_clicker::Settings,
    pub break_delay: break_delay::Settings,
    pub fast_use: fast_use::Settings,
    pub fullbright: fullbright::Settings,
    pub sneak: sneak::Settings,
    pub no_slow: no_slow::Settings,
    pub slippy: slippy::Settings,
    pub reach: modules::reach::Settings,
    pub multitask: modules::multitask::Settings,
    pub air_place: modules::air_place::Settings,
    pub no_mining_trace: modules::no_mining_trace::Settings,
    pub liquid_interact: modules::liquid_interact::Settings,
    pub rainbow_speed: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            schema: 1,
            hud: hud::Config::default(),
            friends: friends::Friends::default(),
            macros: macros::Macros::default(),
            modules: Registry::new(vec![
                Module::new("zoom", "Zoom", "Render"),
                Module::new("fullbright", "Fullbright", "Render"),
                Module::new("sneak", "Sneak", "Movement"),
                Module::new("no-slow", "No Slow", "Movement"),
                Module::new("slippy", "Slippy", "Movement"),
                Module::new("auto-jump", "Auto Jump", "Movement"),
                Module::new("air-jump", "Air Jump", "Movement"),
                Module::new("sprint", "Sprint", "Movement"),
                Module::new("gui-move", "GUI Move", "Movement"),
                Module::new("high-jump", "High Jump", "Movement"),
                Module::new("auto-walk", "Auto Walk", "Movement"),
                Module::new("safe-walk", "Safe Walk", "Movement"),
                Module::new("step", "Step", "Movement"),
                Module::new("reverse-step", "Reverse Step", "Movement"),
                Module::new("fast-climb", "Fast Climb", "Movement"),
                Module::new("spider", "Spider", "Movement"),
                Module::new("parkour", "Parkour", "Movement"),
                Module::new("timer", "Timer", "World"),
                Module::new("auto-respawn", "Auto Respawn", "Player"),
                Module::new("flight", "Flight", "Movement"),
                Module::new("auto-clicker", "Auto Clicker", "Player"),
                Module::new("break-delay", "Break Delay", "Player"),
                Module::new("fast-use", "Fast Use", "Player"),
                Module::new("reach", "Reach", "Player"),
                Module::new("multitask", "Multitask", "Player"),
                Module::new("air-place", "Air Place", "Player"),
                Module::new("liquid-interact", "Liquid Interact", "Player"),
                Module::new("no-mining-trace", "No Mining Trace", "Player"),
            ]),
            zoom: ZoomSettings::default(),
            auto_jump: auto_jump::Settings::default(),
            air_jump: air_jump::Settings::default(),
            sprint: sprint::Settings::default(),
            gui_move: gui_move::Settings::default(),
            high_jump: high_jump::Settings::default(),
            auto_walk: auto_walk::Settings::default(),
            safe_walk: safe_walk::Settings::default(),
            step: step::Settings::default(),
            reverse_step: reverse_step::Settings::default(),
            fast_climb: fast_climb::Settings::default(),
            spider: spider::Settings::default(),
            parkour: parkour::Settings::default(),
            timer: timer::Settings::default(),
            auto_respawn: auto_respawn::Settings::default(),
            flight: Default::default(),
            auto_clicker: Default::default(),
            break_delay: break_delay::Settings::default(),
            fast_use: fast_use::Settings::default(),
            fullbright: fullbright::Settings::default(),
            sneak: Default::default(),
            no_slow: Default::default(),
            slippy: Default::default(),
            reach: Default::default(),
            multitask: Default::default(),
            air_place: Default::default(),
            no_mining_trace: Default::default(),
            liquid_interact: Default::default(),
            rainbow_speed: 0.5,
        }
    }
}

impl Config {
    /// Exact upstream setting names shared by ClickGUI commands and Starscript lookup.
    pub fn source_setting_name(module: &str, field: &str) -> String {
        match (module, field) {
            ("zoom", "hide_hud") => "hide-HUD".into(),
            ("sprint", "mode") => "sprint-mode".into(),
            _ => field.replace('_', "-"),
        }
    }
    pub fn from_json(text: &str) -> Result<Self, String> {
        let mut config: Self = serde_json::from_str(text).map_err(|e| e.to_string())?;
        if config.schema != 1 {
            return Err("Unsupported Meteor settings schema".into());
        }
        config.zoom.validate()?;
        config.auto_jump.validate()?;
        config.gui_move.validate()?;
        config.high_jump.validate()?;
        config.safe_walk.validate()?;
        config.step.validate()?;
        config.reverse_step.validate()?;
        config.fast_climb.validate()?;
        config.spider.validate()?;
        config.parkour.validate()?;
        config.timer.validate()?;
        config.break_delay.validate()?;
        config.flight.validate()?;
        config.auto_clicker.validate()?;
        config.fast_use.validate()?;
        config.fullbright.validate()?;
        config.no_slow.validate()?;
        config.slippy.validate()?;
        config.reach.validate()?;
        config.air_place.validate()?;
        if !config.rainbow_speed.is_finite() || !(0.0..=10.0).contains(&config.rainbow_speed) {
            return Err("Rainbow speed must be between 0 and 10".into());
        }
        config.hud.validate()?;
        config.macros.validate_storage()?;
        config.hud.merge_presets();
        config.friends = friends::Friends::loaded(std::mem::take(&mut config.friends.friends));
        // Merge saved state into the implemented catalog. A new module is disabled by default;
        // stale display metadata must not replace the source-derived identity.
        let mut catalog = Self::default().modules;
        let mut seen = std::collections::BTreeSet::new();
        for saved in config.modules.modules {
            if !seen.insert(saved.id.clone()) {
                return Err(format!("Duplicate module: {}", saved.id));
            }
            let module = catalog
                .get_mut(&saved.id)
                .ok_or_else(|| format!("Module not implemented: {}", saved.id))?;
            module.active = saved.active;
            module.keybind = saved.keybind;
            module.toggle_on_bind_release = saved.toggle_on_bind_release;
            module.chat_feedback = saved.chat_feedback;
            module.favorite = saved.favorite;
        }
        config.modules = catalog;
        Ok(config)
    }
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
    /// Validate a candidate before committing it, so rejected edits leave the config intact.
    pub fn set(&mut self, module: &str, setting: &str, value: &str) -> Result<(), String> {
        fn boolean(value: &str) -> Result<bool, String> {
            value.parse().map_err(|_| "Expected true or false".into())
        }
        fn number(value: &str) -> Result<f64, String> {
            value.parse().map_err(|_| "Expected a finite number".into())
        }
        match module {
            "air-place" => {
                let mut next = self.air_place.clone();
                match setting {
                    "render" => next.render = boolean(value)?,
                    "custom-range" => next.custom_range = boolean(value)?,
                    "range" => next.range = number(value)?,
                    "shape-mode" => {
                        next.shape_mode = serde_json::from_str(&format!("\"{}\"", value))
                            .map_err(|_| "Expected Lines, Sides, or Both")?
                    }
                    "side-color" => {
                        next.side_color = serde_json::from_str(value)
                            .map_err(|e| format!("Expected RGBA color object: {e}"))?
                    }
                    "line-color" => {
                        next.line_color = serde_json::from_str(value)
                            .map_err(|e| format!("Expected RGBA color object: {e}"))?
                    }
                    _ => return Err(format!("Unknown AirPlace setting: {setting}")),
                }
                next.validate()?;
                self.air_place = next;
            }
            "no-mining-trace" => match setting {
                "only-when-holding-a-pickaxe" => {
                    self.no_mining_trace.only_when_holding_a_pickaxe = boolean(value)?
                }
                "blacklisted-entities" => {
                    self.no_mining_trace.blacklisted_entities = serde_json::from_str(value)
                        .map_err(|e| format!("Expected entity identifier list: {e}"))?
                }
                _ => return Err(format!("Unknown NoMiningTrace setting: {setting}")),
            },
            "config" if setting == "rainbow-speed" => {
                let speed = number(value)?;
                if !speed.is_finite() || !(0.0..=10.0).contains(&speed) {
                    return Err("Rainbow speed must be between 0 and 10".into());
                }
                self.rainbow_speed = speed;
            }
            "reach" => {
                let mut next = self.reach.clone();
                match setting {
                    "extra-block-reach" => next.extra_block_reach = number(value)?,
                    "extra-entity-reach" => next.extra_entity_reach = number(value)?,
                    _ => return Err(format!("Unknown Reach setting: {setting}")),
                }
                next.validate()?;
                self.reach = next;
            }
            "multitask" => {
                if setting != "attacking-entities" {
                    return Err(format!("Unknown Multitask setting: {setting}"));
                }
                self.multitask.attacking_entities = boolean(value)?;
            }
            "sneak" => {
                if setting != "mode" {
                    return Err(format!("Unknown Sneak setting: {setting}"));
                }
                self.sneak.mode = match value {
                    "Packet" => sneak::Mode::Packet,
                    "Vanilla" => sneak::Mode::Vanilla,
                    _ => return Err("mode must be Packet or Vanilla".into()),
                };
            }
            "no-slow" => {
                let mut next = self.no_slow.clone();
                match setting {
                    "items" => next.items = boolean(value)?,
                    "web" => {
                        next.web = match value {
                            "Vanilla" => no_slow::WebMode::Vanilla,
                            "Timer" => no_slow::WebMode::Timer,
                            "None" => no_slow::WebMode::None,
                            _ => return Err("web must be Vanilla, Timer or None".into()),
                        }
                    }
                    "web-timer" => next.web_timer = number(value)?,
                    "honey-block" => next.honey_block = boolean(value)?,
                    "soul-sand" => next.soul_sand = boolean(value)?,
                    "slime-block" => next.slime_block = boolean(value)?,
                    "berry-bush" => next.berry_bush = boolean(value)?,
                    "air-strict" => next.air_strict = boolean(value)?,
                    "fluid-drag" => next.fluid_drag = boolean(value)?,
                    "sneaking" => next.sneaking = boolean(value)?,
                    "hunger" => next.hunger = boolean(value)?,
                    "slowness" => next.slowness = boolean(value)?,
                    _ => return Err(format!("Unknown NoSlow setting: {setting}")),
                }
                next.validate()?;
                self.no_slow = next;
            }
            "slippy" => {
                let mut next = self.slippy.clone();
                match setting {
                    "friction" => next.friction = number(value)?,
                    "list-mode" => {
                        next.list_mode = match value {
                            "Whitelist" => slippy::ListMode::Whitelist,
                            "Blacklist" => slippy::ListMode::Blacklist,
                            _ => return Err("list-mode must be Whitelist or Blacklist".into()),
                        }
                    }
                    "ignored-blocks" => next.ignored_blocks = fast_use::parse_items(value)?,
                    "allowed-blocks" => next.allowed_blocks = fast_use::parse_items(value)?,
                    _ => return Err(format!("Unknown Slippy setting: {setting}")),
                }
                next.validate()?;
                self.slippy = next;
            }

            "fullbright" => {
                let mut next = self.fullbright.clone();
                match setting {
                    "mode" => {
                        next.mode = match value {
                            "Gamma" => fullbright::Mode::Gamma,
                            "Potion" => fullbright::Mode::Potion,
                            "Luminance" => fullbright::Mode::Luminance,
                            _ => return Err("mode must be Gamma, Potion or Luminance".into()),
                        }
                    }
                    "light-type" => {
                        next.light_type = match value {
                            "BLOCK" => fullbright::LightType::BLOCK,
                            "SKY" => fullbright::LightType::SKY,
                            _ => return Err("light-type must be BLOCK or SKY".into()),
                        }
                    }
                    "minimum-light-level" => {
                        next.minimum_light_level =
                            value.parse().map_err(|_| "Expected an integer")?
                    }
                    _ => return Err(format!("Unknown Fullbright setting: {setting}")),
                }
                next.validate()?;
                self.fullbright = next;
            }

            "zoom" => {
                let mut next = self.zoom.clone();
                match setting {
                    "zoom" => next.zoom = number(value)?,
                    "scroll-sensitivity" => next.scroll_sensitivity = number(value)?,
                    "smooth" => next.smooth = boolean(value)?,
                    "cinematic" => next.cinematic = boolean(value)?,
                    "hide-HUD" => next.hide_hud = boolean(value)?,
                    "show-hands" => next.show_hands = boolean(value)?,
                    _ => return Err(format!("Unknown Zoom setting: {setting}")),
                }
                next.validate()?;
                self.zoom = next;
            }
            "auto-jump" => {
                let mut next = self.auto_jump.clone();
                match setting {
                    "mode" => {
                        next.mode = match value {
                            "Jump" => auto_jump::Mode::Jump,
                            "LowHop" => auto_jump::Mode::LowHop,
                            _ => return Err("Mode must be Jump or LowHop".into()),
                        }
                    }
                    "jump-if" => {
                        next.jump_if = match value {
                            "Always" => auto_jump::JumpWhen::Always,
                            "Walking" => auto_jump::JumpWhen::Walking,
                            "Sprinting" => auto_jump::JumpWhen::Sprinting,
                            _ => return Err("jump-if must be Always, Walking or Sprinting".into()),
                        }
                    }
                    "velocity-height" => next.velocity_height = number(value)?,
                    _ => return Err(format!("Unknown AutoJump setting: {setting}")),
                }
                next.validate()?;
                self.auto_jump = next;
            }
            "air-jump" => match setting {
                "maintain-level" => self.air_jump.maintain_level = boolean(value)?,
                _ => return Err(format!("Unknown AirJump setting: {setting}")),
            },
            "sprint" => match setting {
                "sprint-mode" => {
                    self.sprint.mode = match value {
                        "Strict" => sprint::Mode::Strict,
                        "Rage" => sprint::Mode::Rage,
                        _ => return Err("sprint-mode must be Strict or Rage".into()),
                    }
                }
                "keep-sprint" => self.sprint.keep_sprint = boolean(value)?,
                "unsprint-on-hit" => self.sprint.unsprint_on_hit = boolean(value)?,
                "unsprint-in-water" => self.sprint.unsprint_in_water = boolean(value)?,
                "sprint-while-stationary" => self.sprint.sprint_while_stationary = boolean(value)?,
                _ => return Err(format!("Unknown Sprint setting: {setting}")),
            },
            "gui-move" => {
                let mut next = self.gui_move.clone();
                match setting {
                    "guis" => {
                        next.guis = match value {
                            "GUI" => gui_move::Screens::GUI,
                            "Inventory" => gui_move::Screens::Inventory,
                            "Both" => gui_move::Screens::Both,
                            _ => return Err("guis must be GUI, Inventory or Both".into()),
                        }
                    }
                    "jump" => next.jump = boolean(value)?,
                    "sneak" => next.sneak = boolean(value)?,
                    "sprint" => next.sprint = boolean(value)?,
                    "arrows-rotate" => next.arrows_rotate = boolean(value)?,
                    "rotate-speed" => next.rotate_speed = number(value)?,
                    _ => return Err(format!("Unknown GUI Move setting: {setting}")),
                }
                next.validate()?;
                self.gui_move = next;
            }
            "high-jump" => {
                if setting != "jump-multiplier" {
                    return Err(format!("Unknown HighJump setting: {setting}"));
                }
                let next = high_jump::Settings {
                    jump_multiplier: number(value)?,
                };
                next.validate()?;
                self.high_jump = next;
            }
            "auto-walk" => match setting {
                "mode" => {
                    self.auto_walk.mode = match value {
                        "Simple" => auto_walk::Mode::Simple,
                        "Smart" => auto_walk::Mode::Smart,
                        _ => return Err("mode must be Simple or Smart".into()),
                    }
                }
                "simple-direction" => {
                    self.auto_walk.simple_direction = match value {
                        "Forwards" => auto_walk::Direction::Forwards,
                        "Backwards" => auto_walk::Direction::Backwards,
                        "Left" => auto_walk::Direction::Left,
                        "Right" => auto_walk::Direction::Right,
                        _ => {
                            return Err(
                                "simple-direction must be Forwards, Backwards, Left or Right"
                                    .into(),
                            );
                        }
                    }
                }
                "disable-on-input" => self.auto_walk.disable_on_input = boolean(value)?,
                "disable-on-y-change" => self.auto_walk.disable_on_y_change = boolean(value)?,
                "no-unloaded-chunks" => self.auto_walk.no_unloaded_chunks = boolean(value)?,
                _ => return Err(format!("Unknown AutoWalk setting: {setting}")),
            },
            "safe-walk" => {
                let mut next = self.safe_walk.clone();
                match setting {
                    "minimum-fall-distance" => {
                        next.minimum_fall_distance = value
                            .parse()
                            .map_err(|_| "Expected an integer".to_string())?
                    }
                    "sneak" => next.sneak = boolean(value)?,
                    "safe-sneak" => next.safe_sneak = boolean(value)?,
                    "sneak-on-sprint" => next.sneak_on_sprint = boolean(value)?,
                    "edge-distance" => next.edge_distance = number(value)?,
                    "render" => next.render = boolean(value)?,
                    "render-player-box" => next.render_player_box = boolean(value)?,
                    _ => return Err(format!("Unknown SafeWalk setting: {setting}")),
                }
                next.validate()?;
                self.safe_walk = next;
            }
            "step" => {
                let mut next = self.step.clone();
                match setting {
                    "height" => next.height = number(value)?,
                    "active-when" => {
                        next.active_when = match value {
                            "Always" => step::ActiveWhen::Always,
                            "Sneaking" => step::ActiveWhen::Sneaking,
                            "NotSneaking" => step::ActiveWhen::NotSneaking,
                            _ => {
                                return Err(
                                    "active-when must be Always, Sneaking or NotSneaking".into()
                                );
                            }
                        }
                    }
                    "safe-step" => next.safe_step = boolean(value)?,
                    "step-health" => {
                        next.step_health = value
                            .parse()
                            .map_err(|_| "Expected an integer".to_string())?
                    }
                    _ => return Err(format!("Unknown Step setting: {setting}")),
                }
                next.validate()?;
                self.step = next;
            }
            "reverse-step" => {
                let mut next = self.reverse_step.clone();
                match setting {
                    "fall-speed" => next.fall_speed = number(value)?,
                    "fall-distance" => next.fall_distance = number(value)?,
                    "vehicles" => next.vehicles = boolean(value)?,
                    _ => return Err(format!("Unknown ReverseStep setting: {setting}")),
                }
                next.validate()?;
                self.reverse_step = next;
            }
            "fast-climb" => {
                let mut next = self.fast_climb.clone();
                match setting {
                    "timer-mode" => next.timer_mode = boolean(value)?,
                    "climb-speed" => next.climb_speed = number(value)?,
                    "timer" => next.timer = number(value)?,
                    _ => return Err(format!("Unknown FastClimb setting: {setting}")),
                }
                next.validate()?;
                self.fast_climb = next;
            }
            "spider" => {
                if setting != "climb-speed" {
                    return Err(format!("Unknown Spider setting: {setting}"));
                }
                let next = spider::Settings {
                    climb_speed: number(value)?,
                };
                next.validate()?;
                self.spider = next;
            }
            "parkour" => {
                if setting != "edge-distance" {
                    return Err(format!("Unknown Parkour setting: {setting}"));
                }
                let next = parkour::Settings {
                    edge_distance: number(value)?,
                };
                next.validate()?;
                self.parkour = next;
            }
            "timer" => {
                if setting != "multiplier" {
                    return Err(format!("Unknown Timer setting: {setting}"));
                }
                let next = timer::Settings {
                    multiplier: number(value)?,
                };
                next.validate()?;
                self.timer = next;
            }
            "auto-respawn" => return Err("AutoRespawn has no settings".into()),
            "flight" => {
                let mut next = self.flight.clone();
                match setting {
                    "mode" => {
                        next.mode = match value {
                            "Abilities" => modules::flight::Mode::Abilities,
                            "Velocity" => modules::flight::Mode::Velocity,
                            _ => return Err("Flight mode must be Abilities or Velocity".into()),
                        }
                    }
                    "anti-kick-mode" => {
                        next.anti_kick_mode = match value {
                            "Normal" => modules::flight::AntiKick::Normal,
                            "Packet" => modules::flight::AntiKick::Packet,
                            "None" => modules::flight::AntiKick::None,
                            _ => return Err("Anti Kick mode must be Normal, Packet or None".into()),
                        }
                    }
                    "speed" => next.speed = number(value)?,
                    "vertical-speed-match" => next.vertical_speed_match = boolean(value)?,
                    "no-sneak" => next.no_sneak = boolean(value)?,
                    "delay" => next.delay = value.parse().map_err(|_| "Expected an integer")?,
                    "off-time" => {
                        next.off_time = value.parse().map_err(|_| "Expected an integer")?
                    }
                    _ => return Err(format!("Unknown Flight setting: {setting}")),
                }
                next.validate()?;
                self.flight = next;
            }
            "auto-clicker" => {
                let mut next = self.auto_clicker.clone();
                match setting {
                    "while-in-screens" => next.while_in_screens = boolean(value)?,
                    "mode-left" => next.mode_left = modules::auto_clicker::Mode::parse(value)?,
                    "mode-right" => next.mode_right = modules::auto_clicker::Mode::parse(value)?,
                    "delay-left" => {
                        next.delay_left = value.parse().map_err(|_| "Expected an integer")?
                    }
                    "delay-right" => {
                        next.delay_right = value.parse().map_err(|_| "Expected an integer")?
                    }
                    _ => return Err(format!("Unknown AutoClicker setting: {setting}")),
                }
                next.validate()?;
                self.auto_clicker = next;
            }
            "break-delay" => {
                let mut next = self.break_delay.clone();
                match setting {
                    "cooldown" => {
                        next.cooldown = value
                            .parse()
                            .map_err(|_| "Expected an integer".to_string())?
                    }
                    "no-insta-break" => next.no_insta_break = boolean(value)?,
                    _ => return Err(format!("Unknown BreakDelay setting: {setting}")),
                }
                next.validate()?;
                self.break_delay = next;
            }
            "fast-use" => {
                let mut next = self.fast_use.clone();
                match setting {
                    "mode" => {
                        next.mode = match value {
                            "All" => fast_use::Mode::All,
                            "Some" => fast_use::Mode::Some,
                            _ => return Err("mode must be All or Some".into()),
                        }
                    }
                    "items" => next.items = fast_use::parse_items(value)?,
                    "blocks" => next.blocks = boolean(value)?,
                    "cooldown" => {
                        next.cooldown = value
                            .parse()
                            .map_err(|_| "Expected an integer".to_string())?
                    }
                    _ => return Err(format!("Unknown FastUse setting: {setting}")),
                }
                next.validate()?;
                self.fast_use = next;
            }
            _ => return Err(format!("Unknown module: {module}")),
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct Client {
    pub config: Config,
    pub zoom: Zoom,
    in_world: bool,
    subscriptions: subscriptions::Subscriptions,
}

impl Client {
    pub fn auto_jump(&self, player: &auto_jump::Player) -> Option<auto_jump::Action> {
        self.active("auto-jump")
            .then(|| auto_jump::tick(player, &self.config.auto_jump))
            .flatten()
    }
    pub fn from_config(config: Config) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }
    pub fn active(&self, id: &str) -> bool {
        self.config.modules.get(id).is_some_and(|m| m.active)
    }
    pub fn toggle(&mut self, id: &str) -> Result<bool, String> {
        let module = self
            .config
            .modules
            .get_mut(id)
            .ok_or_else(|| format!("Unknown module: {id}"))?;
        module.active = !module.active;
        let active = module.active;
        if self.in_world && active {
            self.subscriptions.activate(id);
        } else if !active {
            self.subscriptions.deactivate(id);
        }

        if id == "zoom" && active && self.in_world {
            self.zoom.activate(&self.config.zoom);
        }
        Ok(active)
    }
    pub fn world_changed(&mut self, in_world: bool) {
        if in_world == self.in_world {
            return;
        }
        self.in_world = in_world;
        self.subscriptions.clear();
        if in_world {
            // Source getAll uses identity-hash iteration, which has no portable order.
            // Native catalog order defines the initial subscriptions on world join.
            for module in &self.config.modules.modules {
                if module.active {
                    self.subscriptions.activate(&module.id);
                }
            }
        }

        if !in_world {
            self.zoom.stop();
        } else if self.active("zoom") {
            self.zoom.activate(&self.config.zoom);
        }
    }
    pub fn subscriptions(&self) -> &[subscriptions::Subscription] {
        self.subscriptions.entries()
    }
    pub fn frame(&mut self, seconds: f64) {
        if self.in_world {
            let active = self.active("zoom");
            self.zoom.frame(seconds, active, &self.config.zoom);
        }
    }
    pub fn key(&mut self, event: &registry::KeyEvent) -> Vec<String> {
        let ids = self.config.modules.matching(event);
        for id in &ids {
            let _ = self.toggle(id);
        }
        ids
    }
    pub fn screen_opened(&mut self) -> Vec<String> {
        let ids: Vec<_> = self
            .config
            .modules
            .modules
            .iter()
            .filter(|m| m.active && m.toggle_on_bind_release)
            .map(|m| m.id.clone())
            .collect();
        for id in &ids {
            let _ = self.toggle(id);
        }
        ids
    }
    pub fn scroll(&mut self, amount: f64, screen_open: bool) -> bool {
        let active = self.active("zoom");
        self.zoom
            .scroll(amount, active, screen_open, &self.config.zoom)
    }
}
