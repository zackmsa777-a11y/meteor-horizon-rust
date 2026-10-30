// SPDX-License-Identifier: GPL-3.0-only
// Port of systems/modules/movement/AutoJump.java. Copyright (c) Meteor Development.
// The Horizon adapter applies these actions through its existing MotionWorld.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Jump,
    LowHop,
}
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum JumpWhen {
    Sprinting,
    Walking,
    #[default]
    Always,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub mode: Mode,
    pub jump_if: JumpWhen,
    pub velocity_height: f64,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: Mode::Jump,
            jump_if: JumpWhen::Always,
            velocity_height: 0.25,
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if !self.velocity_height.is_finite() || self.velocity_height < 0.0 {
            return Err("Velocity height must be finite and nonnegative".into());
        }
        Ok(())
    }
}
pub struct Player {
    pub on_ground: bool,
    pub sneaking: bool,
    pub sprinting: bool,
    pub forward: f32,
    pub sideways: f32,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    JumpFromGround,
    SetVerticalVelocity(f64),
}
pub fn tick(player: &Player, settings: &Settings) -> Option<Action> {
    if !player.on_ground || player.sneaking {
        return None;
    }
    let moving = player.forward != 0.0 || player.sideways != 0.0;
    let should_jump = match settings.jump_if {
        JumpWhen::Sprinting => player.sprinting && moving,
        JumpWhen::Walking => moving,
        JumpWhen::Always => true,
    };
    if !should_jump {
        return None;
    }
    match settings.mode {
        Mode::Jump => Some(Action::JumpFromGround),
        Mode::LowHop if settings.velocity_height.is_finite() && settings.velocity_height >= 0.0 => {
            Some(Action::SetVerticalVelocity(settings.velocity_height))
        }
        Mode::LowHop => None,
    }
}
