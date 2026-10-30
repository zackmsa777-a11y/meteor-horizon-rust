// SPDX-License-Identifier: GPL-3.0-only
// Port of HighJump.java and its Entity/LivingEntity mixins. Copyright (c) Meteor Development.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub jump_multiplier: f64,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            jump_multiplier: 1.0,
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if !self.jump_multiplier.is_finite() || self.jump_multiplier < 0.0 {
            Err("jump-multiplier must be finite and nonnegative".into())
        } else {
            Ok(())
        }
    }
    pub fn block_jump_factor(&self, original: f32) -> f32 {
        // The Java event's compound assignment narrows the double multiplier to float first.
        original * self.jump_multiplier as f32
    }
}
