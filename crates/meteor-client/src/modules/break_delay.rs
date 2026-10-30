// SPDX-License-Identifier: GPL-3.0-only
// Port of BreakDelay.java + MultiPlayerGameModeMixin. Copyright (c) Meteor Development.
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub cooldown: i32,
    pub no_insta_break: bool,
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if self.cooldown < 0 {
            Err("cooldown must be nonnegative".into())
        } else {
            Ok(())
        }
    }
}
#[derive(Default, Debug)]
pub struct State {
    break_block_cooldown: bool,
}
impl State {
    /// ANY physical mouse button press, including GUI clicks; release/repeat are not Press.
    /// Call only while subscribed/active. Source has no activation/deactivation reset.
    pub fn mouse_click(&mut self, settings: &Settings, is_press: bool) {
        if is_press && settings.no_insta_break {
            self.break_block_cooldown = true;
        }
    }
    /// BlockBreakingCooldownEvent at creative start/continue and completed survival break.
    pub fn cooldown(&mut self, settings: &Settings) -> i32 {
        if std::mem::take(&mut self.break_block_cooldown) {
            5
        } else {
            settings.cooldown
        }
    }
    /// Only continueDestroyBlock's progress calculation is intercepted. startDestroyBlock's
    /// instant survival break remains untouched. Returns progress plus an optional delay write.
    pub fn continue_progress(&mut self, settings: &Settings, original: f32) -> (f32, Option<i32>) {
        if settings.no_insta_break && original >= 1.0 {
            (0.0, Some(self.cooldown(settings)))
        } else {
            (original, None)
        }
    }
}
