// SPDX-License-Identifier: GPL-3.0-only
//! Adapter for the optional native Meteor port. No Java VM or Fabric JAR execution.
use super::{Game, native_hooks::NativeMod};
use std::time::Duration;

#[cfg(not(feature = "meteor"))]
#[derive(Default)]
pub(super) struct Bridge;

#[cfg(not(feature = "meteor"))]
impl Game<'_> {
    pub(super) fn meteor_fullbright_gamma(&self) -> bool {
        false
    }
    pub(super) fn meteor_break_mouse(&mut self, _: bool) {}
    pub(super) fn meteor_break_cooldown(&mut self) -> u32 {
        5
    }
    pub(super) fn meteor_break_progress(&mut self, progress: f32) -> (f32, Option<u32>) {
        (progress, None)
    }
    pub(super) fn meteor_fast_use_delay(&mut self, _: super::Hand) {}
    pub(super) fn meteor_auto_respawn_screen(&mut self) -> bool {
        false
    }
    pub(super) fn meteor_gui_bind(
        &mut self,
        _: &str,
        _: Option<&str>,
        _: u8,
    ) -> Result<(), String> {
        Err("Meteor feature unavailable".into())
    }
    pub(super) fn meteor_tick_multiplier(&self) -> f32 {
        1.0
    }
    pub(super) fn meteor_post_tick(&mut self) {}
    pub(super) fn meteor_player_replaced(&mut self) {}
    pub(super) fn meteor_safe_walk_render(
        &mut self,
        _: &mut Vec<horizon_client::gizmos::Line>,
    ) -> super::Result<()> {
        Ok(())
    }
    pub(super) fn meteor_hud_save_config(&mut self, _: serde_json::Value) -> Result<(), String> {
        Err("Meteor feature unavailable".into())
    }
    pub(super) fn meteor_frame(&mut self, _: Duration) {
        let _ = &self.meteor;
    }
    pub(super) fn meteor_pre_tick(&mut self) -> super::Result<()> {
        Ok(())
    }
    pub(super) fn meteor_tick(&mut self) {}
    pub(super) fn meteor_gui_module_option(&mut self, _: &str, _: &str, _: bool) {}
    pub(super) fn meteor_gui_set_setting(
        &mut self,
        _: &str,
        _: &str,
        _: &str,
    ) -> Result<(), String> {
        Err("Meteor feature unavailable".into())
    }
    pub(super) fn meteor_walk_keys(
        &self,
        keys: horizon_client::sprint::Keys,
    ) -> horizon_client::sprint::Keys {
        keys
    }
    pub(super) fn meteor_gui_input(&mut self, _: &str, _: bool, _: bool) {}
    pub(super) fn meteor_release_input(&mut self) {}
    pub(super) fn meteor_gui_keys(&self) -> Option<horizon_client::sprint::Keys> {
        None
    }
    pub(super) fn meteor_before_attack(&self) -> bool {
        false
    }
    pub(super) fn meteor_after_attack(&self) -> bool {
        false
    }
    pub(super) fn meteor_keep_sprint(&self) -> bool {
        false
    }
    pub(super) fn meteor_key(&mut self, _: &str, _: bool) {}
    pub(super) fn meteor_wheel(&mut self, _: f32, _: &mut bool) {}
    pub(super) fn meteor_command(&mut self, _: &str, _: &mut bool) {}
    pub(super) fn meteor_mods(&mut self, _: &mut Vec<NativeMod>) {}
    pub(super) fn meteor_settings(&mut self, _: &str, _: &mut Vec<String>) {}
    pub(super) fn meteor_settings_click(&mut self, _: &str, _: usize) {}
    pub(super) fn meteor_zoom(&self) -> Option<(f32, bool, f64)> {
        None
    }
    pub(super) fn meteor_look(&self, look: crate::game::LookOptions) -> crate::game::LookOptions {
        look
    }
    pub(super) fn meteor_hide_hud(&self) -> bool {
        false
    }
    pub(super) fn meteor_hide_hands(&self) -> bool {
        false
    }
    pub(super) fn meteor_status(&self) -> serde_json::Value {
        serde_json::Value::Null
    }
}

#[cfg(feature = "meteor")]
pub(super) use enabled::Bridge;

#[cfg(feature = "meteor")]
mod enabled {
    use super::*;
    use meteor_client::{Client, Config, registry::KeyEvent};
    use std::collections::BTreeSet;

    #[derive(Default)]
    pub(in crate::game) struct Bridge {
        pub(in crate::game) client: Client,
        loaded: bool,
        pub(in crate::game) load_failed: bool,
        pub(in crate::game) profiles_world: Option<String>,
        screen_open: bool,
        held: BTreeSet<String>,
        camera_baseline: Option<f64>,
        camera_override: Option<(f64, bool)>,
        air_jump: meteor_client::modules::air_jump::AirJump,
        gui_move: meteor_client::modules::gui_move::GuiMove,
        auto_walk_previous: Option<(bool, meteor_client::modules::auto_walk::Settings)>,
        step: super::super::meteor_step::State,
        fast_climb: meteor_client::modules::fast_climb::State,
        fast_climb_generation: Option<u64>,
        no_slow: meteor_client::modules::no_slow::State,
        no_slow_generation: Option<u64>,
        pub(in crate::game) last_sent_input: Option<u8>,
        timer: meteor_client::modules::timer::Timer,
        break_delay: meteor_client::modules::break_delay::State,
        fullbright_previous: Option<(bool, meteor_client::modules::fullbright::Settings)>,
        pub(in crate::game) flight: super::super::meteor_flight::State,
        pub(in crate::game) air_place: meteor_client::modules::air_place::State,
        pub(in crate::game) auto_clicker: super::super::meteor_auto_clicker::State,
        pub(in crate::game) air_place_preview: std::cell::Cell<[usize; 2]>,
        pub(in crate::game) rainbow: meteor_client::colors::Rainbow,
    }

    impl Game<'_> {
        pub(in crate::game) fn meteor_fullbright_gamma(&self) -> bool {
            self.meteor
                .client
                .config
                .fullbright
                .gamma(self.meteor_interaction_active("fullbright") && self.title.is_none())
        }
        fn meteor_fullbright_sync(&mut self) {
            use meteor_client::modules::fullbright;
            let active = self.meteor_interaction_active("fullbright") && self.title.is_none();
            let settings = self.meteor.client.config.fullbright.clone();
            let [block, sky] = settings.floors(active);
            let floors = horizon_client::render_light::Floors::new(block, sky);
            let changed = self.world.render_light_floors != floors;
            self.world.render_light_floors = floors;
            self.streaming.set_render_light_floors(floors);
            if let Some(lod) = self.lod.as_mut() {
                lod.set_render_light_floors(floors);
            }
            let mut effects = match self.meteor.fullbright_previous.as_ref() {
                Some((true, old)) if !active => fullbright::deactivate(old),
                Some((true, old)) if active => fullbright::settings_changed(old, &settings, true),
                _ if active => fullbright::activate(&settings),
                _ => Vec::new(),
            };
            if changed && !effects.contains(&fullbright::Effect::RebuildSections) {
                effects.push(fullbright::Effect::RebuildSections);
            }
            self.meteor.fullbright_previous = Some((active, settings));
            self.meteor_fullbright_effects(&effects);
        }
        pub(in crate::game) fn meteor_interaction_active(&self, id: &str) -> bool {
            self.native_window()
                && self.native_on("meteor-client")
                && self.player.is_some()
                && self.meteor.client.active(id)
        }
        pub(in crate::game) fn meteor_break_mouse(&mut self, pressed: bool) {
            if self.meteor_interaction_active("break-delay") {
                self.meteor
                    .break_delay
                    .mouse_click(&self.meteor.client.config.break_delay, pressed);
            }
        }
        pub(in crate::game) fn meteor_break_cooldown(&mut self) -> u32 {
            if self.meteor_interaction_active("break-delay") {
                self.meteor
                    .break_delay
                    .cooldown(&self.meteor.client.config.break_delay) as u32
            } else {
                5
            }
        }
        pub(in crate::game) fn meteor_break_progress(
            &mut self,
            progress: f32,
        ) -> (f32, Option<u32>) {
            if self.meteor_interaction_active("break-delay") {
                let (progress, delay) = self
                    .meteor
                    .break_delay
                    .continue_progress(&self.meteor.client.config.break_delay, progress);
                (progress, delay.map(|v| v as u32))
            } else {
                (progress, None)
            }
        }
        pub(in crate::game) fn meteor_fast_use_delay(&mut self, hand: super::super::Hand) {
            if self.meteor_interaction_active("fast-use") {
                let settings = self.meteor.client.config.fast_use.clone();
                if let Err(error) = self
                    .meteor_fast_use_validate(&settings)
                    .and_then(|()| self.meteor_fast_use_hand(&settings, hand))
                {
                    self.use_delay = 4;
                    self.meteor_disable_error("fast-use", error);
                }
            }
        }
        pub(in crate::game) fn meteor_auto_respawn_screen(&mut self) -> bool {
            if !self.meteor_interaction_active("auto-respawn") {
                return false;
            }
            match self.meteor_auto_respawn_open_death() {
                Ok(cancel) => cancel,
                Err(error) => {
                    self.meteor_disable_error("auto-respawn", error);
                    false
                }
            }
        }
        fn meteor_set_checked(
            &mut self,
            id: &str,
            setting: &str,
            value: &str,
        ) -> Result<(), String> {
            let mut candidate = self.meteor.client.config.clone();
            candidate.set(id, setting, value)?;
            if id == "fast-use" {
                self.meteor_fast_use_validate(&candidate.fast_use)
                    .map_err(|e| e.to_string())?;
            }
            if id == "slippy" {
                self.meteor_slippy_validate(&candidate.slippy)
                    .map_err(|e| e.to_string())?;
            }
            if id == "no-mining-trace" {
                self.meteor_trace_validate(&candidate.no_mining_trace)
                    .map_err(|e| e.to_string())?;
            }
            self.meteor.client.config = candidate;
            Ok(())
        }
        pub(in crate::game) fn meteor_hud_speed_scale(&self) -> (bool, f64) {
            let active = self.meteor.client.active("timer");
            (
                active,
                self.meteor
                    .timer
                    .multiplier(&self.meteor.client.config.timer, active),
            )
        }
        pub(in crate::game) fn meteor_gui_bind(
            &mut self,
            id: &str,
            key: Option<&str>,
            modifiers: u8,
        ) -> Result<(), String> {
            self.meteor_load();
            let module = self
                .meteor
                .client
                .config
                .modules
                .get_mut(id)
                .ok_or_else(|| format!("Module not implemented: {id}"))?;
            module.keybind.key = key.map(str::to_owned);
            module.keybind.modifiers = if key.is_some_and(|key| !key.starts_with("key.mouse.")) {
                modifiers & 15
            } else {
                0
            };
            self.meteor_save();
            Ok(())
        }
        pub(in crate::game) fn meteor_tick_multiplier(&self) -> f32 {
            if !self.native_window()
                || !self.native_on("meteor-client")
                || self.player.is_none()
                || self.title.is_some()
            {
                return 1.0;
            }
            self.meteor.timer.delta_multiplier(
                &self.meteor.client.config.timer,
                self.meteor.client.active("timer"),
            )
        }
        fn meteor_timer_pre(&mut self) {
            let in_world = self.native_window()
                && self.native_on("meteor-client")
                && self.player.is_some()
                && self.title.is_none();
            self.meteor.client.world_changed(in_world);
            self.meteor_auto_clicker_sync();
            if !in_world {
                return;
            }
            let listeners: Vec<_> = self
                .meteor
                .client
                .subscriptions()
                .iter()
                .filter(|entry| matches!(entry.id.as_str(), "fast-climb" | "no-slow"))
                .cloned()
                .collect();
            for listener in listeners {
                match listener.id.as_str() {
                    "fast-climb" => self.meteor_climb_pre(listener.generation),
                    "no-slow" => self.meteor_no_slow_pre(listener.generation),
                    _ => {}
                }
            }
        }
        fn meteor_no_slow_pre(&mut self, generation: u64) {
            let active = self.meteor_interaction_active("no-slow") && self.title.is_none();
            if active && self.meteor.no_slow_generation != Some(generation) {
                self.meteor.no_slow.activate();
            }
            self.meteor.no_slow_generation = active.then_some(generation);
            if !active {
                return;
            }
            let p = self.player.as_ref().unwrap();
            let b = [
                p.position.x.floor() as i32,
                p.position.y.floor() as i32,
                p.position.z.floor() as i32,
            ];
            let cobweb = self
                .pack
                .assets
                .registry
                .state(self.world.block(b[0], b[1], b[2]))
                .is_some_and(|s| s.name == "minecraft:cobweb");
            if let Some(value) = self.meteor.no_slow.pre_tick(
                &self.meteor.client.config.no_slow,
                cobweb,
                p.ground.on_ground,
            ) {
                self.meteor.timer.set_override(value);
            }
        }
        fn meteor_climb_pre(&mut self, generation: u64) {
            let active = self.native_window()
                && self.native_on("meteor-client")
                && self.player.is_some()
                && self.title.is_none()
                && self.meteor.client.active("fast-climb");
            if active && self.meteor.fast_climb_generation != Some(generation) {
                self.meteor.fast_climb.activate();
            }
            self.meteor.fast_climb_generation = active.then_some(generation);
            if !active {
                return;
            }
            let settings = self.meteor.client.config.fast_climb.clone();
            let mut state = std::mem::take(&mut self.meteor.fast_climb);
            let result = self.meteor_fast_climb_pre(&settings, &mut state);
            self.meteor.fast_climb = state;
            match result {
                Ok(Some(value)) => self.meteor.timer.set_override(value),
                Ok(None) => {}
                Err(error) => self.meteor_disable_error("fast-climb", error),
            }
        }
        pub(in crate::game) fn meteor_disable_error(
            &mut self,
            id: &str,
            error: Box<dyn std::error::Error>,
        ) {
            if self.meteor.client.active(id) {
                let _ = self.meteor.client.toggle(id);
                self.meteor_save();
            }
            self.native_chat(&format!("[Meteor] {id} disabled: {error}"));
        }
        pub(in crate::game) fn meteor_player_replaced(&mut self) {
            self.meteor.step.reset();
        }
        pub(in crate::game) fn meteor_post_tick(&mut self) {
            self.meteor_flight_post();
            self.meteor_auto_clicker_post();
            self.meteor_macros_post();
            self.meteor_hud_update_text(true);
            self.meteor
                .rainbow
                .next(self.meteor.client.config.rainbow_speed / 100.0, 1.0);
            for color in &mut self.meteor.client.config.hud.text_colors {
                color.update(&self.meteor.rainbow);
            }
            for element in &mut self.meteor.client.config.hud.elements {
                if let Some(text) = &mut element.text {
                    text.background_color.update(&self.meteor.rainbow);
                }
            }
            if self.meteor_interaction_active("air-place") {
                self.meteor
                    .client
                    .config
                    .air_place
                    .side_color
                    .update(&self.meteor.rainbow);
                self.meteor
                    .client
                    .config
                    .air_place
                    .line_color
                    .update(&self.meteor.rainbow);
            }
            self.meteor_fullbright_sync();
            let enabled = self.native_window()
                && self.native_on("meteor-client")
                && self.player.is_some()
                && self.title.is_none();
            let settings = (enabled && self.meteor.client.active("step"))
                .then(|| self.meteor.client.config.step.clone());
            let mut state = std::mem::take(&mut self.meteor.step);
            let result = self.meteor_step_post(&mut state, settings.as_ref());
            self.meteor.step = state;
            if let Err(error) = result {
                self.meteor.step.reset();
                if self.meteor.client.active("step") {
                    let _ = self.meteor.client.toggle("step");
                    self.meteor_save();
                }
                self.native_chat(&format!("[Meteor] Step disabled: {error}"));
            }
            if enabled && self.meteor.client.active("reverse-step") {
                let settings = self.meteor.client.config.reverse_step.clone();
                if let Err(error) = self.meteor_reverse_step_post(&settings) {
                    let _ = self.meteor.client.toggle("reverse-step");
                    self.meteor_save();
                    self.native_chat(&format!("[Meteor] Reverse Step disabled: {error}"));
                }
            }
            if enabled && self.meteor.client.active("fast-climb") {
                let settings = self.meteor.client.config.fast_climb.clone();
                if let Err(error) = self.meteor_fast_climb_post(&settings) {
                    self.meteor_disable_error("fast-climb", error);
                }
            }
            if enabled && self.meteor.client.active("spider") {
                let settings = self.meteor.client.config.spider.clone();
                self.meteor_spider_post(&settings);
            }
            if enabled && self.meteor.client.active("parkour") {
                let settings = self.meteor.client.config.parkour.clone();
                if let Err(error) = self.meteor_parkour_post(&settings) {
                    self.meteor_disable_error("parkour", error);
                }
            }
            if enabled && self.meteor.client.active("fullbright") {
                let settings = self.meteor.client.config.fullbright.clone();
                self.meteor_fullbright_potion_post(&settings);
            }
        }
        pub(in crate::game) fn meteor_safe_walk_render(
            &mut self,
            lines: &mut Vec<horizon_client::gizmos::Line>,
        ) -> crate::game::Result<()> {
            if self.native_window()
                && self.native_on("meteor-client")
                && self.meteor.client.active("safe-walk")
            {
                if let Some(player) = self.player.as_mut() {
                    super::super::meteor_safe_walk_render::append(
                        &self.meteor.client.config.safe_walk,
                        player,
                        lines,
                    )?;
                }
            }
            Ok(())
        }
        pub(in crate::game) fn meteor_hud_save_config(
            &mut self,
            value: serde_json::Value,
        ) -> Result<(), String> {
            let config: meteor_client::hud::Config =
                serde_json::from_value(value).map_err(|e| e.to_string())?;
            config.validate()?;
            self.meteor_load();
            self.meteor.client.config.hud = config;
            self.meteor_save();
            Ok(())
        }
        pub(in crate::game) fn meteor_gui_module_option(
            &mut self,
            id: &str,
            field: &str,
            value: bool,
        ) {
            self.meteor_load();
            let Some(module) = self.meteor.client.config.modules.get_mut(id) else {
                return;
            };
            match field {
                "favorite" => module.favorite = value,
                "toggle_on_bind_release" => module.toggle_on_bind_release = value,
                "chat_feedback" => module.chat_feedback = value,
                _ => return,
            }
            self.meteor_save();
        }
        pub(in crate::game) fn meteor_gui_set_setting(
            &mut self,
            id: &str,
            setting: &str,
            value: &str,
        ) -> Result<(), String> {
            self.meteor_load();
            self.meteor_set_checked(id, setting, value)?;
            self.meteor_fullbright_sync();
            self.meteor_gui_sync();
            self.meteor_auto_walk_sync();
            self.meteor_save();
            Ok(())
        }
        /// Read-only diagnostics for the control API, reporting the actual engine adapter inputs.
        pub(in crate::game) fn meteor_status(&self) -> serde_json::Value {
            let look = self.look_options();
            serde_json::json!({
                "enabled": self.native_on("meteor-client"),
                "gui": self.meteor_gui_status(),
                "macros": self.meteor_macros_status(),
                "tick_multiplier": self.meteor_tick_multiplier(),
                "last_sent_input": self.meteor.last_sent_input,
                "slow_movement": self.player.as_ref().map(|p|serde_json::json!({"raw_water":p.fluids.was_touching_water,"in_water":p.is_in_water(),"in_lava":p.is_in_lava(),"ability_walk_speed":p.ability_walk_speed,"moving_slowly":p.moving_slowly()})),
                "destroy_delay": self.meteor_destroy_delay(), "use_delay": self.use_delay,
                "fullbright": {"gamma": self.meteor_fullbright_gamma(),
                    "block_floor": self.world.render_light_floors.block,
                    "sky_floor": self.world.render_light_floors.sky,
                    "night_vision_ticks": self.potions.effects.get("night_vision").map(|effect|effect.duration)},
                "config": self.meteor.client.config,
                "zoom": {"subscribed": self.meteor.client.zoom.subscribed,
                    "factor": self.meteor_zoom().map_or(1.0, |z| z.0),
                    "hide_hud": self.meteor_hide_hud(), "hide_hands": self.meteor_hide_hands(),
                    "sensitivity": look.sensitivity, "cinematic": look.smooth_camera},
                "configuration_error": self.meteor.load_failed,
                "air_jump_level": self.meteor.air_jump.level,
                "air_place": {"cached_hit":self.meteor.air_place.hit.map(|h|h.pos), "preview_line_vertices":self.meteor.air_place_preview.get()[0],"preview_triangle_vertices":self.meteor.air_place_preview.get()[1]},
                "flight": self.meteor_flight_status(),
                "interaction_ranges": {"block":self.player_block_range(),"entity":self.player_entity_range()},
            })
        }
        fn meteor_sync_motion_mods(&mut self) {
            if self.meteor_interaction_active("slippy") {
                let settings = self.meteor.client.config.slippy.clone();
                if let Err(error) = self.meteor_slippy_validate(&settings) {
                    self.meteor_disable_error("slippy", error);
                }
            }
            let enabled =
                self.native_window() && self.native_on("meteor-client") && self.title.is_none();
            let no_slow = (enabled && self.meteor.client.active("no-slow"))
                .then(|| self.meteor.client.config.no_slow.clone());
            let sneak = (enabled && self.meteor.client.active("sneak"))
                .then(|| self.meteor.client.config.sneak.clone());
            let slippy = (enabled && self.meteor.client.active("slippy"))
                .then(|| self.meteor.client.config.slippy.clone());

            let sprint = (self.native_window()
                && self.native_on("meteor-client")
                && self.meteor.client.active("sprint"))
            .then(|| self.meteor.client.config.sprint.clone());
            let high_jump = (self.native_window()
                && self.native_on("meteor-client")
                && self.meteor.client.active("high-jump"))
            .then(|| self.meteor.client.config.high_jump.clone());
            let auto_walk = (self.native_window()
                && self.native_on("meteor-client")
                && self.meteor.client.active("auto-walk"))
            .then(|| self.meteor.client.config.auto_walk.clone());
            let safe_walk = (self.native_window()
                && self.native_on("meteor-client")
                && self.meteor.client.active("safe-walk"))
            .then(|| self.meteor.client.config.safe_walk.clone());
            let sprint_key = self.physical_keys.iter().any(|key| {
                horizon_client::keybinds::key_name(*key)
                    == Some(self.settings.keys.get("key.sprint"))
            }) || self.physical_buttons.iter().any(|button| {
                horizon_client::keybinds::mouse_name(*button)
                    == self.settings.keys.get("key.sprint")
            });
            if let Some(player) = self.player.as_mut() {
                player.meteor_sprint = sprint;
                player.meteor_no_slow = no_slow;
                player.meteor_sneak = sneak;
                player.meteor_slippy = slippy;
                player.meteor_high_jump = high_jump;
                player.meteor_auto_walk = auto_walk;
                player.meteor_safe_walk = safe_walk;
                player.meteor_safe_walk_sprint_key = sprint_key;
                player.meteor_safe_walk_sneak_input = false;
            }
        }
        /// Meteor Tick.Pre: acts on the local player's state from the preceding tick.
        pub(in crate::game) fn meteor_pre_tick(&mut self) -> crate::game::Result<()> {
            self.meteor_load();
            self.meteor_timer_pre();
            self.meteor_auto_walk_tick();
            self.meteor_sync_motion_mods();
            if !self.native_window() || !self.native_on("meteor-client") || self.title.is_some() {
                return Ok(());
            }
            self.meteor_load();
            self.meteor_flight_pre();
            self.meteor_picking_pre();
            self.meteor_sync_air_jump();
            if let Some(player) = self.meteor_air_player() {
                let jump_down = self.keys.contains(&winit::keyboard::KeyCode::Space);
                if self
                    .meteor
                    .air_jump
                    .tick(player, &self.meteor.client.config.air_jump, jump_down)
                {
                    self.meteor_jump_from_ground()?;
                }
            }
            let Some(player) = self.player.as_ref() else {
                return Ok(());
            };
            let snapshot = meteor_client::modules::auto_jump::Player {
                on_ground: player.ground.on_ground,
                sneaking: player.is_shift_key_down(),
                sprinting: player.sprinting,
                forward: player.move_vector[1],
                sideways: player.move_vector[0],
            };
            let Some(action) = self.meteor.client.auto_jump(&snapshot) else {
                return Ok(());
            };
            let Some(player) = self.player.as_mut() else {
                return Ok(());
            };
            match action {
                meteor_client::modules::auto_jump::Action::SetVerticalVelocity(y) => {
                    player.velocity.y = y
                }
                meteor_client::modules::auto_jump::Action::JumpFromGround => {
                    self.meteor_jump_from_ground()?;
                }
            }
            Ok(())
        }
        fn meteor_jump_from_ground(&mut self) -> crate::game::Result<()> {
            self.meteor_sync_motion_mods();
            let entities = self.player_entity_collisions();
            if let Some(player) = self.player.as_mut() {
                horizon_client::motion::MotionWorld {
                    entities: &entities,
                    player,
                    scene: &mut self.world,
                    assets: &self.pack.assets,
                    collision_binding: self.collision,
                    resources: self.motion,
                }
                .jump_from_ground()?;
            }
            Ok(())
        }
        fn meteor_clear_walk_directions(&mut self) {
            use meteor_client::modules::gui_move::Action;
            use winit::keyboard::KeyCode;
            for key in [KeyCode::KeyW, KeyCode::KeyS, KeyCode::KeyA, KeyCode::KeyD] {
                self.keys.remove(&key);
            }
            for action in [Action::Forward, Action::Back, Action::Left, Action::Right] {
                self.meteor.gui_move.input(
                    action,
                    false,
                    false,
                    &self.meteor.client.config.gui_move,
                );
            }
        }
        fn meteor_auto_walk_sync(&mut self) {
            use meteor_client::modules::auto_walk;
            let active = self.native_window()
                && self.native_on("meteor-client")
                && self.player.is_some()
                && self.title.is_none()
                && self.meteor.client.active("auto-walk");
            let next = self.meteor.client.config.auto_walk.clone();
            if let Some((was_active, previous)) = self.meteor.auto_walk_previous.take() {
                let release = (was_active
                    && !active
                    && matches!(
                        auto_walk::deactivate(&previous),
                        auto_walk::Effect::ReleaseDirections
                    ))
                    || auto_walk::settings_changed(
                        &previous,
                        &next,
                        active,
                        self.player.is_some(),
                        self.yaw,
                    )
                    .contains(&auto_walk::Effect::ReleaseDirections);
                if release {
                    self.meteor_clear_walk_directions();
                }
            }
            // The native Baritone/path-manager implementation is not present yet.
            self.meteor.auto_walk_previous = Some((active, next));
        }
        fn meteor_auto_walk_tick(&mut self) {
            use meteor_client::modules::auto_walk::{self, TickAction};
            self.meteor_auto_walk_sync();
            if !self
                .meteor
                .auto_walk_previous
                .as_ref()
                .is_some_and(|(active, _)| *active)
            {
                return;
            }
            let Some(player) = &self.player else {
                return;
            };
            if let TickAction::Disable(reason) = auto_walk::tick(
                &self.meteor.client.config.auto_walk,
                self.previous_position.y,
                player.position.y,
                false,
            ) {
                let _ = self.meteor.client.toggle("auto-walk");
                if let Some(message) = reason.message() {
                    self.native_chat(&format!("[Meteor] {message}"));
                }
                self.meteor_auto_walk_sync();
                self.meteor_save();
            }
        }
        pub(in crate::game) fn meteor_walk_keys(
            &self,
            mut keys: horizon_client::sprint::Keys,
        ) -> horizon_client::sprint::Keys {
            use meteor_client::modules::auto_walk::{Direction, Mode};
            if self.native_window()
                && self.native_on("meteor-client")
                && self.meteor.client.active("auto-walk")
                && self.meteor.client.config.auto_walk.mode == Mode::Simple
                && self.title.is_none()
            {
                match self.meteor.client.config.auto_walk.simple_direction {
                    Direction::Forwards => keys.forward = true,
                    Direction::Backwards => keys.backward = true,
                    Direction::Left => keys.left = true,
                    Direction::Right => keys.right = true,
                }
            }
            keys
        }
        fn meteor_air_player(&self) -> Option<meteor_client::modules::air_jump::Player> {
            self.player
                .as_ref()
                .filter(|_| self.title.is_none())
                .map(|p| meteor_client::modules::air_jump::Player {
                    block_y: p.position.y.floor() as i32,
                    on_ground: p.ground.on_ground,
                    freecam: self.meteor.client.active("freecam"),
                })
        }
        fn meteor_sync_air_jump(&mut self) {
            self.meteor_flight_sync();
            let active = self.native_on("meteor-client") && self.meteor.client.active("air-jump");
            let y = self.meteor_air_player().map(|p| p.block_y);
            self.meteor.air_jump.sync(active, y);
        }
        fn meteor_gui_context(&self) -> meteor_client::modules::gui_move::Context {
            use meteor_client::modules::gui_move::{Context, Screen};
            Context {
                screen: if self.meteor_gui.open {
                    Screen::Widget
                } else {
                    match self.screen {
                        super::super::Screen::Playing => Screen::None,
                        super::super::Screen::Inventory | super::super::Screen::Creative => {
                            Screen::Container
                        }
                        _ => Screen::Other,
                    }
                },
                can_use_keys: self.native_window()
                    && self.player.is_some()
                    && self.title.is_none()
                    && !self.dialog_open()
                    && self.book.is_none()
                    && self.sign_edit.is_none()
                    && !self.meteor_gui_wants_text(),
                creative_search: self.screen == super::super::Screen::Creative
                    && self.creative_screen.tab().search,
                text_editor: self.recipe_search_focused()
                    || self
                        .station
                        .as_ref()
                        .is_some_and(|s| s.kind == super::super::stations::Kind::Anvil),
            }
        }
        fn meteor_gui_sync(&mut self) {
            let active = self.native_on("meteor-client") && self.meteor.client.active("gui-move");
            let context = self.meteor_gui_context();
            self.meteor
                .gui_move
                .sync(active, context, &self.meteor.client.config.gui_move);
        }
        pub(in crate::game) fn meteor_release_input(&mut self) {
            self.meteor.gui_move.release_all();
            self.meteor.held.clear();
            let changed = self.meteor.client.screen_opened();
            if !changed.is_empty() {
                self.meteor_save();
            }
        }
        pub(in crate::game) fn meteor_gui_input(&mut self, key: &str, pressed: bool, repeat: bool) {
            self.meteor_gui_sync();
            for (action, mapping) in meteor_client::modules::gui_move::ACTIONS {
                if key == self.settings.keys.get(mapping) {
                    self.meteor.gui_move.input(
                        action,
                        pressed,
                        repeat,
                        &self.meteor.client.config.gui_move,
                    );
                }
            }
        }
        pub(in crate::game) fn meteor_gui_keys(&self) -> Option<horizon_client::sprint::Keys> {
            use meteor_client::modules::gui_move::{Action, skip};
            if !self.native_on("meteor-client")
                || !self.meteor.client.active("gui-move")
                || skip(
                    self.meteor_gui_context(),
                    &self.meteor.client.config.gui_move,
                )
            {
                return None;
            }
            let down = |action| {
                self.meteor.gui_move.down(action)
                    && self.meteor.client.config.gui_move.enabled(action)
            };
            Some(horizon_client::sprint::Keys {
                forward: down(Action::Forward),
                backward: down(Action::Back),
                left: down(Action::Left),
                right: down(Action::Right),
                jump: down(Action::Jump),
                shift: down(Action::Sneak),
                sprint: down(Action::Sprint),
                analog: None,
            })
        }
        fn meteor_load(&mut self) {
            if self.meteor.loaded || !self.native_window() {
                return;
            }
            self.meteor.loaded = true;
            let path = self.native_data_dir("meteor-client").join("config.json");
            let result = match std::fs::read_to_string(&path) {
                Ok(text) => Config::from_json(&text),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
                Err(e) => Err(e.to_string()),
            };
            match result {
                Ok(config) => self.meteor.client = Client::from_config(config),
                Err(e) => {
                    self.meteor.load_failed = true;
                    log::error!(
                        "Meteor settings {}: {e}; saving disabled to preserve original",
                        path.display()
                    );
                }
            }
        }
        pub(in crate::game) fn meteor_save(&self) {
            if self.meteor.load_failed {
                return;
            }
            let dir = self.native_data_dir("meteor-client");
            let result = (|| -> Result<(), Box<dyn std::error::Error>> {
                let text = self.meteor.client.config.to_json()?;
                std::fs::create_dir_all(&dir)?;
                // Keep the previous successful configuration before replacing it.
                let path = dir.join("config.json");
                if path.exists() {
                    std::fs::copy(&path, dir.join("config.previous.json"))?;
                }
                std::fs::write(&path, text)?;
                Ok(())
            })();
            if let Err(e) = result {
                log::error!("Meteor settings save failed: {e}");
            }
        }
        pub(in crate::game) fn meteor_frame(&mut self, elapsed: Duration) {
            if !self.native_window() {
                return;
            }
            self.meteor_load();
            self.meteor_fullbright_sync();
            let in_world =
                self.player.is_some() && self.title.is_none() && self.native_on("meteor-client");
            self.meteor.client.world_changed(in_world);
            self.meteor_profiles_join(in_world);
            self.meteor_auto_clicker_sync();
            self.meteor_sync_air_jump();
            self.meteor_gui_sync();
            self.meteor_auto_walk_sync();
            if self.meteor.gui_move.allowed && !self.meteor.client.active("freecam") {
                use winit::keyboard::KeyCode;
                let arrows = [
                    KeyCode::ArrowLeft,
                    KeyCode::ArrowRight,
                    KeyCode::ArrowUp,
                    KeyCode::ArrowDown,
                ]
                .map(|key| self.physical_keys.contains(&key));
                (self.yaw, self.pitch) = meteor_client::modules::gui_move::rotate(
                    self.yaw,
                    self.pitch,
                    elapsed.as_secs_f64(),
                    &self.meteor.client.config.gui_move,
                    arrows,
                );
            }
            if !self.meteor.client.zoom.subscribed {
                self.meteor.camera_baseline = None;
                self.meteor.camera_override = None;
            } else if self.meteor.camera_baseline.is_none() {
                let sensitivity = f64::from(self.settings.sensitivity);
                self.meteor.camera_baseline = Some(sensitivity);
                self.meteor.camera_override = Some((sensitivity, self.smooth_camera));
            }
            let screen_open = !self.native_playing();
            if screen_open && !self.meteor.screen_open {
                let changed = self.meteor.client.screen_opened();
                if !changed.is_empty() {
                    self.meteor_save();
                }
                self.meteor.held.clear();
            }
            self.meteor.screen_open = screen_open;
            self.meteor.client.frame(elapsed.as_secs_f64());
        }
        pub(in crate::game) fn meteor_tick(&mut self) {
            if self.native_window() {
                if let Some(snapshot) = self.meteor_sprint_player() {
                    if let Some(sprinting) = meteor_client::modules::sprint::tick(
                        self.native_on("meteor-client") && self.meteor.client.active("sprint"),
                        &snapshot,
                        &self.meteor.client.config.sprint,
                    ) && let Some(player) = self.player.as_mut()
                        && let Err(error) = player.set_sprinting(sprinting)
                    {
                        log::error!("Meteor Sprint: {error}");
                    }
                }
                if self.meteor.client.zoom.subscribed && self.native_on("meteor-client") {
                    let base = *self
                        .meteor
                        .camera_baseline
                        .get_or_insert(f64::from(self.settings.sensitivity));
                    let cinematic = self.meteor.client.config.zoom.cinematic;
                    let sensitivity = if cinematic {
                        self.meteor.camera_override.map_or(base, |camera| camera.0)
                    } else {
                        base * self.meteor.client.zoom.sensitivity_multiplier()
                    };
                    self.meteor.camera_override = Some((sensitivity, cinematic));
                }
                self.meteor.client.zoom.tick();
                if !self.meteor.client.zoom.subscribed {
                    self.meteor.camera_baseline = None;
                    self.meteor.camera_override = None;
                }
            }
        }
        fn meteor_sprint_player(&self) -> Option<meteor_client::modules::sprint::Player> {
            self.player
                .as_ref()
                .filter(|_| self.title.is_none())
                .map(|p| {
                    meteor_client::modules::sprint::Player {
                        screen_open: !self.native_playing(),
                        // Sprint consults this setting even when GUIMove is inactive.
                        gui_move_sprint: self.meteor.client.config.gui_move.sprint,
                        forward: p.move_vector[1],
                        sideways: p.move_vector[0],
                        under_water: p.is_under_water(),
                        in_water: p.is_in_water(),
                        shallow_water: p.is_in_water() && !p.is_under_water(),
                        mobility_restricted: self.sleeping.is_some() || p.health.health() <= 0.0,
                        passenger: self.riding.is_some(),
                        vehicle_can_sprint: false,
                        vehicle_local: false,
                        enough_food: p.movement_enough_food(),
                        horizontal_collision: p.collision.horizontal,
                        minor_horizontal_collision: p.collision.minor,
                    }
                })
        }
        pub(in crate::game) fn meteor_before_attack(&self) -> bool {
            self.native_window()
                && self.native_on("meteor-client")
                && meteor_client::modules::sprint::before_attack(
                    self.meteor.client.active("sprint"),
                    &self.meteor.client.config.sprint,
                )
        }
        pub(in crate::game) fn meteor_after_attack(&self) -> bool {
            self.native_window()
                && self.native_on("meteor-client")
                && self.meteor_sprint_player().is_some_and(|snapshot| {
                    meteor_client::modules::sprint::after_attack(
                        self.meteor.client.active("sprint"),
                        self.player.as_ref().is_some_and(|p| p.sprinting),
                        &snapshot,
                        &self.meteor.client.config.sprint,
                    )
                })
        }
        pub(in crate::game) fn meteor_keep_sprint(&self) -> bool {
            self.native_window()
                && self.native_on("meteor-client")
                && !meteor_client::modules::sprint::stop_after_knockback(
                    self.meteor.client.active("sprint"),
                    &self.meteor.client.config.sprint,
                )
        }
        pub(in crate::game) fn meteor_look(
            &self,
            mut look: crate::game::LookOptions,
        ) -> crate::game::LookOptions {
            if self.native_on("meteor-client")
                && let Some((sensitivity, cinematic)) = self.meteor.camera_override
            {
                // Apply before Minecraft's nonlinear sensitivity curve, not to its output.
                look.sensitivity = sensitivity;
                look.smooth_camera = cinematic;
            }
            look
        }
        pub(in crate::game) fn meteor_hide_hud(&self) -> bool {
            self.native_on("meteor-client")
                && meteor_client::modules::zoom::Zoom::hide_hud(
                    self.meteor.client.active("zoom"),
                    &self.meteor.client.config.zoom,
                )
        }
        pub(in crate::game) fn meteor_hide_hands(&self) -> bool {
            self.native_on("meteor-client")
                && !meteor_client::modules::zoom::Zoom::render_hands(
                    self.meteor.client.active("zoom"),
                    &self.meteor.client.config.zoom,
                )
        }
        pub(in crate::game) fn meteor_key(&mut self, key: &str, pressed: bool) {
            if !self.native_window() || !self.native_on("meteor-client") {
                return;
            }
            self.meteor_load();
            let repeat = pressed && self.meteor.held.contains(key);
            if pressed {
                self.meteor.held.insert(key.into());
            } else {
                self.meteor.held.remove(key);
            }
            let held = &self.meteor.held;
            let mut modifiers = 0;
            for (bit, left, right) in [
                (1, "key.keyboard.left.shift", "key.keyboard.right.shift"),
                (2, "key.keyboard.left.control", "key.keyboard.right.control"),
                (4, "key.keyboard.left.alt", "key.keyboard.right.alt"),
                (8, "key.keyboard.left.win", "key.keyboard.right.win"),
            ] {
                if self.physical_keys.iter().any(|key| {
                    horizon_client::keybinds::key_name(*key)
                        .is_some_and(|name| name == left || name == right)
                }) {
                    modifiers |= bit;
                }
            }
            let event = KeyEvent {
                key: key.into(),
                modifiers: if key.starts_with("key.mouse.") {
                    0
                } else {
                    modifiers
                },
                pressed,
                repeat,
                screen_open: !self.native_playing(),
                debug_key_down: held.contains("key.keyboard.f3"),
            };
            self.meteor_macros_input(&event);
            // Macros observes at EventPriority.HIGH; ordinary module listeners still
            // receive the same input afterward, including changes made by that macro.
            self.meteor_gui_input(key, pressed, false);
            if self.meteor.client.active("auto-walk")
                && meteor_client::modules::auto_walk::disable_for_input(
                    &self.meteor.client.config.auto_walk,
                    pressed && !repeat,
                    meteor_client::modules::auto_walk::MANUAL_MOVEMENT_MAPPINGS
                        .iter()
                        .any(|mapping| self.settings.keys.get(mapping) == key),
                    !self.native_playing(),
                    self.meteor.client.active("gui-move"),
                    meteor_client::modules::gui_move::skip(
                        self.meteor_gui_context(),
                        &self.meteor.client.config.gui_move,
                    ),
                )
            {
                let _ = self.meteor.client.toggle("auto-walk");
                self.meteor_auto_walk_sync();
                self.meteor_save();
            }
            let changed = self.meteor.client.key(&event);
            self.meteor_sync_air_jump();
            self.meteor_gui_sync();
            // Meteor's AirJump listens to keyboard Press events, not mouse button events.
            if key.starts_with("key.keyboard.")
                && let Some(player) = self.meteor_air_player()
            {
                let jump = key == self.settings.keys.get("key.jump");
                let sneak = key == self.settings.keys.get("key.sneak");
                if self.meteor.air_jump.key(
                    player,
                    pressed && !repeat,
                    event.screen_open,
                    jump,
                    sneak,
                ) && let Err(error) = self.meteor_jump_from_ground()
                {
                    log::error!("Meteor AirJump: {error}");
                }
            }
            for id in &changed {
                if self
                    .meteor
                    .client
                    .config
                    .modules
                    .get(id)
                    .is_some_and(|m| m.chat_feedback)
                {
                    self.native_chat(&format!("[Meteor] {id}: {}", self.meteor.client.active(id)));
                }
            }
            if !changed.is_empty() {
                self.meteor_save();
            }
        }
        pub(in crate::game) fn meteor_wheel(&mut self, steps: f32, claimed: &mut bool) {
            if self.native_on("meteor-client") && self.native_playing() && !*claimed {
                *claimed = self.meteor.client.scroll(f64::from(steps), false);
            }
        }
        pub(in crate::game) fn meteor_zoom(&self) -> Option<(f32, bool, f64)> {
            if !self.native_window()
                || !self.native_on("meteor-client")
                || self.title.is_some()
                || self.player.is_none()
                || !self.meteor.client.zoom.subscribed
            {
                return None;
            }
            // Meteor performs its own cubic animation, then divides the final FOV.
            Some((self.meteor.client.zoom.scaling() as f32, false, 1.0))
        }
        pub(in crate::game) fn meteor_apply_config(
            &mut self,
            config: Config,
        ) -> Result<(), String> {
            let config = Config::from_json(&config.to_json().map_err(|e| e.to_string())?)?;
            self.meteor_slippy_validate(&config.slippy)
                .map_err(|e| e.to_string())?;
            self.meteor_fast_use_validate(&config.fast_use)
                .map_err(|e| e.to_string())?;
            self.meteor_trace_validate(&config.no_mining_trace)
                .map_err(|e| e.to_string())?;
            let mut step = std::mem::take(&mut self.meteor.step);
            let restored = self.meteor_step_post(&mut step, None);
            self.meteor.step = step;
            restored.map_err(|e| format!("Cannot restore Step before import: {e}"))?;
            for module in &mut self.meteor.client.config.modules.modules {
                module.active = false;
            }
            self.meteor_fullbright_sync();
            self.meteor_auto_walk_sync();
            self.meteor_gui_sync();
            self.meteor_release_input();
            self.meteor_flight_sync();
            self.meteor.client = Client::from_config(config);
            self.meteor.air_jump = Default::default();
            self.meteor.fast_climb = Default::default();
            self.meteor.fast_climb_generation = None;
            self.meteor.no_slow = Default::default();
            self.meteor.no_slow_generation = None;
            self.meteor.camera_baseline = None;
            self.meteor.camera_override = None;
            // The source's shared Timer override survives disableAll; FastClimb will
            // update it on its next eligible tick. Do not silently invent a reset here.
            let in_world =
                self.player.is_some() && self.title.is_none() && self.native_on("meteor-client");
            self.meteor.client.world_changed(in_world);
            self.meteor_sync_air_jump();
            self.meteor_gui_sync();
            self.meteor_auto_walk_sync();
            self.meteor_sync_motion_mods();
            Ok(())
        }
        fn meteor_import_command(&mut self, text: &str) -> Result<String, String> {
            use std::io::Read;
            if self.meteor.load_failed {
                return Err("Existing Meteor configuration could not be read; resolve that error before importing.".into());
            }
            let (kind, path) = text
                .trim()
                .split_once(char::is_whitespace)
                .ok_or("Usage: /meteor import modules|friends|macros PATH_TO_NBT")?;
            if !matches!(kind, "modules" | "friends" | "macros") {
                return Err("Import kind must be modules, friends or macros.".into());
            }
            let path = path.trim();
            let path = if (path.starts_with('"') && path.ends_with('"'))
                || (path.starts_with('\'') && path.ends_with('\''))
            {
                path.get(1..path.len().saturating_sub(1)).unwrap_or("")
            } else {
                path
            };
            let file = std::fs::File::open(path).map_err(|e| format!("Cannot read NBT: {e}"))?;
            if !file.metadata().map_err(|e| e.to_string())?.is_file() {
                return Err("Choose a regular NBT file.".into());
            }
            let mut bytes = Vec::new();
            file.take(horizon_client::meteor_import::MAX_BYTES as u64 + 1)
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())?;
            let imported = match kind {
                "modules" => {
                    horizon_client::meteor_import::modules(&bytes, &self.meteor.client.config)
                }
                "macros" => {
                    horizon_client::meteor_import::macros(&bytes, &self.meteor.client.config)
                }
                _ => horizon_client::meteor_import::friends(&bytes, &self.meteor.client.config),
            }?;
            if kind == "modules" {
                self.meteor_slippy_validate(&imported.candidate.slippy)
                    .map_err(|e| e.to_string())?;
                self.meteor_fast_use_validate(&imported.candidate.fast_use)
                    .map_err(|e| e.to_string())?;
            }
            // Parsing and validation finish before any live state changes. Friends/macros
            // preserve running module state. Module import follows disableAll/reactivation.
            if kind == "friends" {
                self.meteor.client.config.friends = imported.candidate.friends;
            } else if kind == "macros" {
                self.meteor.client.config.macros = imported.candidate.macros;
                // Macros.fromTag replaces subscribed objects. MacroCommand's queued old
                // object references remain alive and keep their original execution state.
                self.meteor_macros_reload_from_config();
            } else {
                self.meteor_apply_config(imported.candidate)?;
            }
            for skipped in &imported.report.skipped {
                self.native_chat(&format!("[Meteor import] Skipped: {skipped}"));
            }
            for note in &imported.report.notes {
                self.native_chat(&format!("[Meteor import] {note}"));
            }
            Ok(format!(
                "Imported {} modules, {} settings, {} friends, {} macros; {} unsupported entries skipped.",
                imported.report.modules.len(),
                imported.report.settings,
                imported.report.friends,
                imported.report.macros,
                imported.report.skipped.len()
            ))
        }
        pub(in crate::game) fn meteor_command(&mut self, command: &str, ran: &mut bool) {
            let mut args = command.split_whitespace();
            if args.next() != Some("meteor") || !self.native_window() {
                return;
            }
            *ran = true;
            self.meteor_load();
            let words: Vec<_> = args.collect();
            let result = if matches!(words.first(), Some(&"macros") | Some(&"macro")) {
                let rest = command
                    .trim_start()
                    .strip_prefix("meteor")
                    .unwrap_or("")
                    .trim_start();
                if words.first() == Some(&"macro") {
                    let rest = rest.strip_prefix("macro").unwrap_or("").trim_start();
                    if rest == "clear" || rest.starts_with("clear ") {
                        self.meteor_macros_command(rest)
                    } else {
                        self.meteor_macros_command(&format!("run {rest}"))
                    }
                } else {
                    self.meteor_macros_command(rest.strip_prefix("macros").unwrap_or(""))
                }
            } else if words.first() == Some(&"profiles") {
                let rest = command
                    .trim_start()
                    .strip_prefix("meteor")
                    .unwrap_or("")
                    .trim_start();
                self.meteor_profiles_command(rest.strip_prefix("profiles").unwrap_or(""))
            } else if words.first() == Some(&"import") {
                let rest = command
                    .trim_start()
                    .strip_prefix("meteor")
                    .unwrap_or("")
                    .trim_start();
                self.meteor_import_command(rest.strip_prefix("import").unwrap_or(""))
            } else {
                match words.as_slice() {
                ["friends", rest @ ..] => self.meteor.client.config.friends.command(rest),
                ["set", module, setting, values @ ..] if !values.is_empty() => {
                    let value=values.join(" ");
                    self.meteor_set_checked(module,setting,&value).map(|()| format!("{module}.{setting}: {value}"))
                },
                ["toggle", id] => self.meteor.client.toggle(id).map(|on| format!("{id}: {on}")),
                ["bind", id, key] => match self.meteor.client.config.modules.get_mut(id) {
                    Some(module) if *key == "none" || key.starts_with("key.keyboard.") || key.starts_with("key.mouse.") => {
                        module.keybind.key = (*key != "none").then(|| (*key).to_string());
                        module.keybind.modifiers = 0;
                        Ok(format!("Bound {id} to {key}"))
                    }
                    _ => Err("Unknown module or invalid key name".into()),
                },
                ["zoom", value] => value.parse::<f64>().map_err(|e| e.to_string()).and_then(|value| {
                    if !value.is_finite() || value < 1.0 { return Err("Zoom must be finite and at least 1".into()); }
                    self.meteor.client.config.zoom.zoom = value;
                    Ok(format!("Zoom factor: {value} (applies next activation)"))
                }),
                _ => Err("Commands: /meteor toggle MODULE; /meteor bind MODULE KEY; /meteor set MODULE SETTING VALUE; /meteor zoom 6; /meteor friends add|remove NAME; /meteor friends list".into()),
            }
            };
            match result {
                Ok(message) => {
                    self.meteor_fullbright_sync();
                    self.meteor_sync_air_jump();
                    self.meteor_gui_sync();
                    self.meteor_auto_walk_sync();
                    self.meteor_save();
                    self.native_chat(&format!("[Meteor] {message}"));
                }
                Err(message) => self.native_chat(&format!("[Meteor] {message}")),
            }
        }
        pub(in crate::game) fn meteor_mods(&mut self, mods: &mut Vec<NativeMod>) {
            self.meteor_load();
            mods.push(NativeMod {
                id: "meteor-client".into(), title: "Meteor Rust — development".into(),
                description: "Native Meteor development port: modules, ClickGUI, HUD and friends. Full compatibility remains in progress.".into(),
                keys: vec![],
            });
        }
        pub(in crate::game) fn meteor_settings(&mut self, id: &str, buttons: &mut Vec<String>) {
            if id != "meteor-client" {
                return;
            }
            self.meteor_load();
            buttons.push(format!(
                "Zoom: {}",
                if self.meteor.client.active("zoom") {
                    "On"
                } else {
                    "Off"
                }
            ));
            buttons.push(format!(
                "Smooth zoom: {}",
                self.meteor.client.config.zoom.smooth
            ));
            buttons.push(format!(
                "Hold key to zoom: {}",
                self.meteor.client.config.modules.modules[0].toggle_on_bind_release
            ));
            buttons.push(format!(
                "Auto Jump: {}",
                self.meteor.client.active("auto-jump")
            ));
            buttons.push(format!(
                "Jump mode: {:?}",
                self.meteor.client.config.auto_jump.mode
            ));
            buttons.push(format!(
                "Jump when: {:?}",
                self.meteor.client.config.auto_jump.jump_if
            ));
            buttons.push(format!(
                "Air Jump: {}",
                self.meteor.client.active("air-jump")
            ));
            buttons.push(format!(
                "Maintain level: {}",
                self.meteor.client.config.air_jump.maintain_level
            ));
            let sprint = &self.meteor.client.config.sprint;
            buttons.push(format!("Sprint: {}", self.meteor.client.active("sprint")));
            buttons.push(format!("Sprint mode: {:?}", sprint.mode));
            buttons.push(format!("Keep sprint: {}", sprint.keep_sprint));
            buttons.push(format!("Unsprint on hit: {}", sprint.unsprint_on_hit));
            buttons.push(format!("Unsprint in water: {}", sprint.unsprint_in_water));
            buttons.push(format!(
                "Sprint while stationary: {}",
                sprint.sprint_while_stationary
            ));
        }
        pub(in crate::game) fn meteor_settings_click(&mut self, id: &str, button: usize) {
            if id != "meteor-client" {
                return;
            }
            self.meteor_load();
            match button {
                0 => {
                    let _ = self.meteor.client.toggle("zoom");
                }
                1 => self.meteor.client.config.zoom.smooth = !self.meteor.client.config.zoom.smooth,
                2 => {
                    let module = &mut self.meteor.client.config.modules.modules[0];
                    module.toggle_on_bind_release = !module.toggle_on_bind_release;
                }
                3 => {
                    let _ = self.meteor.client.toggle("auto-jump");
                }
                4 => {
                    use meteor_client::modules::auto_jump::Mode;
                    let value = &mut self.meteor.client.config.auto_jump.mode;
                    *value = match *value {
                        Mode::Jump => Mode::LowHop,
                        Mode::LowHop => Mode::Jump,
                    };
                }
                5 => {
                    use meteor_client::modules::auto_jump::JumpWhen;
                    let value = &mut self.meteor.client.config.auto_jump.jump_if;
                    *value = match *value {
                        JumpWhen::Always => JumpWhen::Walking,
                        JumpWhen::Walking => JumpWhen::Sprinting,
                        JumpWhen::Sprinting => JumpWhen::Always,
                    };
                }
                6 => {
                    let _ = self.meteor.client.toggle("air-jump");
                }
                7 => {
                    self.meteor.client.config.air_jump.maintain_level =
                        !self.meteor.client.config.air_jump.maintain_level;
                }
                8 => {
                    let _ = self.meteor.client.toggle("sprint");
                }
                9 => {
                    use meteor_client::modules::sprint::Mode;
                    let mode = &mut self.meteor.client.config.sprint.mode;
                    *mode = if *mode == Mode::Strict {
                        Mode::Rage
                    } else {
                        Mode::Strict
                    };
                }
                10 => {
                    let value = &mut self.meteor.client.config.sprint.keep_sprint;
                    *value = !*value;
                }
                11 => {
                    let value = &mut self.meteor.client.config.sprint.unsprint_on_hit;
                    *value = !*value;
                }
                12 => {
                    let value = &mut self.meteor.client.config.sprint.unsprint_in_water;
                    *value = !*value;
                }
                13 => {
                    let value = &mut self.meteor.client.config.sprint.sprint_while_stationary;
                    *value = !*value;
                }
                _ => return,
            }
            self.meteor_sync_air_jump();
            self.meteor_save();
        }
    }
}
