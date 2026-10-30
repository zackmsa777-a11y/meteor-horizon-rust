// SPDX-License-Identifier: GPL-3.0-only
// Port of Parkour.java. Copyright (c) Meteor Development.
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub edge_distance: f64,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            edge_distance: 0.001,
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if !self.edge_distance.is_finite() || !(0.001..=0.1).contains(&self.edge_distance) {
            Err("edge-distance must be between 0.001 and 0.1".into())
        } else {
            Ok(())
        }
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Player {
    pub on_ground: bool,
    pub jump_key_down: bool,
    pub shift_key_down: bool,
    pub sneak_key_down: bool,
}
pub fn eligible(player: Player) -> bool {
    player.on_ground && !player.jump_key_down && !player.shift_key_down && !player.sneak_key_down
}
pub fn adjusted_box(settings: &Settings, min: [f64; 3], max: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    let a = [
        min[0] + settings.edge_distance,
        min[1] - 0.5,
        min[2] + settings.edge_distance,
    ];
    let b = [
        max[0] - settings.edge_distance,
        max[1] - 0.5,
        max[2] - settings.edge_distance,
    ];
    (
        std::array::from_fn(|i| a[i].min(b[i])),
        std::array::from_fn(|i| a[i].max(b[i])),
    )
}
/// Only getBlockCollisions counts; nearby entity boxes/borders cannot prevent this jump.
pub fn should_jump(player: Player, has_block_collision: bool) -> bool {
    eligible(player) && !has_block_collision
}
