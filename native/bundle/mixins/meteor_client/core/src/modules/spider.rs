// SPDX-License-Identifier: GPL-3.0-only
// Port of Spider.java. Copyright (c) Meteor Development.
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub climb_speed: f64,
}
impl Default for Settings {
    fn default() -> Self {
        Self { climb_speed: 0.2 }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if !self.climb_speed.is_finite() || self.climb_speed < 0.0 {
            Err("climb-speed must be finite and nonnegative".into())
        } else {
            Ok(())
        }
    }
}
/// Tick.Post threshold stays 0.2 even when configured climb_speed differs.
pub fn post_tick(settings: &Settings, horizontal_collision: bool, velocity: [f64; 3]) -> [f64; 3] {
    if horizontal_collision && !(velocity[1] >= 0.2) {
        [velocity[0], settings.climb_speed, velocity[2]]
    } else {
        velocity
    }
}
