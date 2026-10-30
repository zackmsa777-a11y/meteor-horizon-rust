// SPDX-License-Identifier: GPL-3.0-only
// Port of Sprint.java. Copyright (c) Meteor Development.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Strict,
    Rage,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub mode: Mode,
    pub keep_sprint: bool,
    pub unsprint_on_hit: bool,
    pub unsprint_in_water: bool,
    pub sprint_while_stationary: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: Mode::Strict,
            keep_sprint: false,
            unsprint_on_hit: false,
            unsprint_in_water: true,
            sprint_while_stationary: false,
        }
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Player {
    pub screen_open: bool,
    pub gui_move_sprint: bool,
    pub forward: f32,
    pub sideways: f32,
    pub under_water: bool,
    pub in_water: bool,
    pub shallow_water: bool,
    pub mobility_restricted: bool,
    pub passenger: bool,
    pub vehicle_can_sprint: bool,
    pub vehicle_local: bool,
    pub enough_food: bool,
    pub horizontal_collision: bool,
    pub minor_horizontal_collision: bool,
}
pub fn should_sprint(active: bool, player: &Player, settings: &Settings) -> bool {
    if player.screen_open && !player.gui_move_sprint {
        return false;
    }
    let movement = if settings.mode == Mode::Rage {
        player.forward.abs() + player.sideways.abs()
    } else {
        player.forward
    };
    // Java promotes the dry threshold to double, but the underwater literal is a float.
    let threshold = if player.under_water {
        f64::from(1.0e-5_f32)
    } else {
        0.8
    };
    if f64::from(movement) <= threshold
        && (settings.mode == Mode::Strict || !settings.sprint_while_stationary)
    {
        return false;
    }
    // Preserve the source's ternary precedence, including its shallow-water fallback.
    let strict = if !player.shallow_water && !player.mobility_restricted && player.passenger {
        player.vehicle_can_sprint && player.vehicle_local
    } else {
        player.enough_food && (!player.horizontal_collision || player.minor_horizontal_collision)
    };
    active && (settings.mode == Mode::Rage || strict)
}
pub fn tick(active: bool, player: &Player, settings: &Settings) -> Option<bool> {
    if !active || (settings.unsprint_in_water && player.in_water) {
        None
    } else {
        Some(should_sprint(active, player, settings))
    }
}
pub fn before_attack(active: bool, settings: &Settings) -> bool {
    active && settings.unsprint_on_hit
}
pub fn after_attack(active: bool, sprinting: bool, player: &Player, settings: &Settings) -> bool {
    active
        && settings.unsprint_on_hit
        && settings.keep_sprint
        && !sprinting
        && should_sprint(active, player, settings)
}
pub fn stop_after_knockback(active: bool, settings: &Settings) -> bool {
    !active || !settings.keep_sprint
}
/// LivingEntityMixin's jump yaw, applied only for the local player in Rage mode.
pub fn jump_yaw(yaw: f32, forward: f32, sideways: f32) -> f32 {
    fn sign(value: f32) -> f32 {
        if value == 0.0 { value } else { value.signum() }
    }
    let forward = sign(forward);
    let mut strafe = 90.0 * sign(sideways);
    if forward != 0.0 {
        strafe *= forward * 0.5;
    }
    yaw - strafe - if forward < 0.0 { 180.0 } else { 0.0 }
}
