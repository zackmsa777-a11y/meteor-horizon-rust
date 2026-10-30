// SPDX-License-Identifier: GPL-3.0-only
// Port of world/Timer.java. Copyright (c) Meteor Development.
use serde::{Deserialize, Serialize};

pub const OFF: f64 = 1.0;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub multiplier: f64,
}
impl Default for Settings {
    fn default() -> Self {
        Self { multiplier: OFF }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        // Native boundary guard: Java's double-to-float infinity would poison clock residuals.
        if !(self.multiplier as f32).is_finite() || self.multiplier < 0.1 {
            Err("multiplier must fit a finite float and be at least 0.1".into())
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Timer {
    override_multiplier: f64,
}
impl Default for Timer {
    fn default() -> Self {
        Self {
            override_multiplier: OFF,
        }
    }
}
impl Timer {
    /// Overrides belong to the shared Timer instance, independent of its enabled state.
    /// Like upstream, the OFF sentinel (1.0) falls back to the module's configured multiplier.
    pub fn multiplier(&self, settings: &Settings, active: bool) -> f64 {
        if self.override_multiplier != OFF {
            self.override_multiplier
        } else if active {
            settings.multiplier
        } else {
            OFF
        }
    }
    pub fn set_override(&mut self, value: f64) {
        self.override_multiplier = value;
    }
    /// RenderTickCounterDynamicMixin narrows the selected double before multiplying deltaTicks.
    pub fn delta_multiplier(&self, settings: &Settings, active: bool) -> f32 {
        self.multiplier(settings, active) as f32
    }
}
