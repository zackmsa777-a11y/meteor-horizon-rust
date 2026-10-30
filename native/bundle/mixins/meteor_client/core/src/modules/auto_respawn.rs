// SPDX-License-Identifier: GPL-3.0-only
// Port of AutoRespawn.java. Copyright (c) Meteor Development.
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Settings {}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    AddDeathWaypoint,
    Respawn,
    CancelScreen,
}
/// OpenScreenEvent HIGH priority. DeathScreen specifically, not an arbitrary zero-health tick.
/// Preserve order: Waypoints.addDeath runs before respawn, and cancellation follows it.
pub fn open_screen(is_death_screen: bool) -> Vec<Effect> {
    if is_death_screen {
        vec![
            Effect::AddDeathWaypoint,
            Effect::Respawn,
            Effect::CancelScreen,
        ]
    } else {
        Vec::new()
    }
}
