// SPDX-License-Identifier: GPL-3.0-only
// Port of AirJump.java. Copyright (c) Meteor Development.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub maintain_level: bool,
}

#[derive(Clone, Copy)]
pub struct Player {
    pub block_y: i32,
    pub on_ground: bool,
    pub freecam: bool,
}

#[derive(Default)]
pub struct AirJump {
    pub level: Option<i32>,
}

impl AirJump {
    /// Capture the activation height once; leaving a world discards it.
    pub fn sync(&mut self, active: bool, block_y: Option<i32>) {
        if !active || block_y.is_none() {
            self.level = None;
        } else if self.level.is_none() {
            self.level = block_y;
        }
    }

    /// The caller supplies matched *physical* jump/sneak bindings, not fixed keys.
    pub fn key(
        &mut self,
        player: Player,
        press: bool,
        screen_open: bool,
        jump: bool,
        sneak: bool,
    ) -> bool {
        if self.level.is_none() || player.freecam || screen_open || player.on_ground || !press {
            return false;
        }
        if jump {
            self.level = Some(player.block_y);
            true
        } else {
            if sneak {
                self.level = self.level.map(|y| y.wrapping_sub(1));
            }
            false
        }
    }

    pub fn tick(&self, player: Player, settings: &Settings, jump_down: bool) -> bool {
        !player.freecam
            && !player.on_ground
            && settings.maintain_level
            && jump_down
            && self.level == Some(player.block_y)
    }
}
