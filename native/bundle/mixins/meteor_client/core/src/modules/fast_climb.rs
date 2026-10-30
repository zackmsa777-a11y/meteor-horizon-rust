// SPDX-License-Identifier: GPL-3.0-only
// Port of FastClimb.java. Copyright (c) Meteor Development.
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub timer_mode: bool,
    pub climb_speed: f64,
    pub timer: f64,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            timer_mode: false,
            climb_speed: 0.2872,
            timer: 1.436,
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if !self.climb_speed.is_finite() || self.climb_speed < 0.0 {
            return Err("climb-speed must be finite and nonnegative".into());
        }
        // Timer's native f32 clock must never receive infinity after narrowing.
        if !(self.timer as f32).is_finite() || self.timer < 1.0 {
            return Err("timer must fit a finite float and be at least 1".into());
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Player {
    pub horizontal_collision: bool,
    pub jumping: bool,
    pub on_climbable: bool,
    pub in_powder_snow_block: bool,
    pub can_walk_powder_snow: bool,
}
pub fn climbing(player: Player) -> bool {
    (player.horizontal_collision || player.jumping)
        && (player.on_climbable || (player.in_powder_snow_block && player.can_walk_powder_snow))
}
#[derive(Default, Debug)]
pub struct State {
    reset_timer: bool,
}
impl State {
    pub fn activate(&mut self) {
        self.reset_timer = false;
    }
    /// Tick.Pre effect for the shared Timer override; 1.0 is Timer.OFF, not module disable.
    pub fn pre_tick(&mut self, settings: &Settings, player: Player) -> Option<f64> {
        if !settings.timer_mode {
            return None;
        }
        if climbing(player) {
            self.reset_timer = false;
            Some(settings.timer)
        } else if !self.reset_timer {
            self.reset_timer = true;
            Some(1.0)
        } else {
            None
        }
    }
    // Upstream has no onDeactivate timer reset. The host must not imply this resets Timer.
}
pub fn post_tick(settings: &Settings, player: Player, velocity: [f64; 3]) -> [f64; 3] {
    if !settings.timer_mode && climbing(player) {
        [velocity[0], settings.climb_speed, velocity[2]]
    } else {
        velocity
    }
}
