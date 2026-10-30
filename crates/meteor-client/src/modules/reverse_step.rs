// SPDX-License-Identifier: GPL-3.0-only
// Port of ReverseStep.java. Copyright (c) Meteor Development.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub fall_speed: f64,
    pub fall_distance: f64,
    pub vehicles: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            fall_speed: 3.0,
            fall_distance: 3.0,
            vehicles: false,
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        for (name, value) in [
            ("fall-speed", self.fall_speed),
            ("fall-distance", self.fall_distance),
        ] {
            if !value.is_finite() || value < 0.0 {
                return Err(format!("{name} must be finite and nonnegative"));
            }
        }
        Ok(())
    }
    /// Source casts the entire negative sum to float before moving the bounding box.
    pub fn probe_y(&self) -> f64 {
        f64::from(-(self.fall_distance + 0.01) as f32)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Entity {
    pub on_ground: bool,
    pub under_water: bool,
    pub in_lava: bool,
    pub no_physics: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Player,
    Vehicle,
}

/// Vehicles bypass the player's ladder, idle and bed exclusions. If vehicles is false,
/// the player branch still runs while riding, as in the source.
pub fn target(
    settings: &Settings,
    has_vehicle: bool,
    suppressing_ladder_slide: bool,
    forward: f32,
    sideways: f32,
    on_bed: bool,
) -> Option<Target> {
    if has_vehicle && settings.vehicles {
        return Some(Target::Vehicle);
    }
    if suppressing_ladder_slide || (forward == 0.0 && sideways == 0.0) || on_bed {
        None
    } else {
        Some(Target::Player)
    }
}

pub fn eligible(entity: Entity, jump_key_down: bool) -> bool {
    entity.on_ground
        && !entity.under_water
        && !entity.in_lava
        && !jump_key_down
        && !entity.no_physics
}

/// Call after testing the target's box moved by probe_y against world noCollision(box).
/// Despite the setting description's blocks/second wording, Java writes -fallSpeed directly
/// into velocity Y; it neither divides by 20 nor changes horizontal velocity.
pub fn velocity(
    settings: &Settings,
    entity: Entity,
    jump_key_down: bool,
    probe_has_no_collision: bool,
    previous: [f64; 3],
) -> [f64; 3] {
    if eligible(entity, jump_key_down) && !probe_has_no_collision {
        [previous[0], -settings.fall_speed, previous[2]]
    } else {
        previous
    }
}

/// Beds are checked at blockPosition.Y, not below it; only the adjacent cells covered by
/// the player's fractional X/Z edge thresholds are inspected, in source order.
pub fn on_bed(position: [f64; 3], mut is_bed: impl FnMut([i32; 3]) -> bool) -> bool {
    let base = position.map(|v| v.floor() as i32);
    let x = position[0] - f64::from(base[0]);
    let z = position[2] - f64::from(base[2]);
    let low_x = (0.0..=0.3).contains(&x);
    let low_z = (0.0..=0.3).contains(&z);
    let high_x = x >= 0.7;
    let high_z = z >= 0.7;
    for (enabled, dx, dz) in [
        (true, 0, 0),
        (low_x, -1, 0),
        (high_x, 1, 0),
        (low_z, 0, -1),
        (high_z, 0, 1),
        (low_x && low_z, -1, -1),
        (low_x && high_z, -1, 1),
        (high_x && low_z, 1, -1),
        (high_x && high_z, 1, 1),
    ] {
        if enabled && is_bed([base[0].wrapping_add(dx), base[1], base[2].wrapping_add(dz)]) {
            return true;
        }
    }
    false
}
