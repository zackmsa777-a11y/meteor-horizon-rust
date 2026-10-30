// SPDX-License-Identifier: GPL-3.0-only
// Port of Step.java. Copyright (c) Meteor Development.
//! Attribute lifecycle and SafeStep sampling. The host must supply real crystal damage
//! at each candidate bounding box; returning zero without inspecting crystals is not a port.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActiveWhen {
    #[default]
    Always,
    Sneaking,
    NotSneaking,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub height: f64,
    pub active_when: ActiveWhen,
    pub safe_step: bool,
    pub step_health: i32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            height: 1.25,
            active_when: ActiveWhen::Always,
            safe_step: false,
            step_health: 5,
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if !self.height.is_finite() || self.height < 0.0 {
            return Err("height must be finite and nonnegative".into());
        }
        if !(1..=36).contains(&self.step_health) {
            return Err("step-health must be between 1 and 36".into());
        }
        Ok(())
    }
}

/// Activation captures effective maxUpStep (a float), not the underlying attribute base.
#[derive(Clone, Copy, Debug)]
pub struct Snapshot {
    pub previous_step_height: f32,
    pub previous_path_step: bool,
}
impl Snapshot {
    /// Host enables path-manager step on activation and restores this flag on deactivation.
    pub fn capture(effective_step_height: f64, path_step: bool) -> Self {
        Self {
            previous_step_height: effective_step_height as f32,
            previous_path_step: path_step,
        }
    }
    /// TickEvent.Post writes the returned value to the STEP_HEIGHT attribute base.
    pub fn tick_base(self, settings: &Settings, shift_key_down: bool, max_safe_height: f64) -> f64 {
        let work = match settings.active_when {
            ActiveWhen::Always => true,
            ActiveWhen::Sneaking => shift_key_down,
            ActiveWhen::NotSneaking => !shift_key_down,
        };
        if work && max_safe_height > 0.0 {
            max_safe_height
        } else {
            self.restore_base()
        }
    }
    pub fn restore_base(self) -> f64 {
        f64::from(self.previous_step_height)
    }
}

/// Input is [left, forward]. Preserve the source's same-sign cross terms (not a conventional
/// yaw rotation). The horizontal look is normalized before multiplying by 1.2.
pub fn input_offset(look: [f64; 3], input: [f32; 2]) -> [f64; 3] {
    let length = (look[0] * look[0] + look[2] * look[2]).sqrt();
    if length < 1.0e-5 {
        return [0.0; 3];
    }
    let x = look[0] / length * 1.2;
    let z = look[2] / length * 1.2;
    [
        f64::from(input[1]) * x + f64::from(input[0]) * z,
        0.0,
        f64::from(input[0]) * x + f64::from(input[1]) * z,
    ]
}

/// Damage is the maximum DamageUtils.crystalDamage among living EndCrystals in
/// entitiesForRendering. The closure evaluates with only the bounding box offset; player
/// position remains unchanged, matching the Java temporary setBoundingBox calls.
/// No damage queries are made when SafeStep is disabled.
#[derive(Debug, PartialEq)]
pub enum SafeHeightError<E> {
    Probe(E),
    /// Native resource guard; Java wraps its integer counter and can hang for huge heights.
    SweepLimit,
}

pub const MAX_SAFE_SWEEP_STEPS: i32 = 256;

pub fn max_safe_height<E>(
    settings: &Settings,
    health: f32,
    absorption: f32,
    offset: [f64; 3],
    mut damage_at: impl FnMut([f64; 3]) -> Result<f64, E>,
) -> Result<f64, SafeHeightError<E>> {
    if !settings.safe_step {
        return Ok(settings.height);
    }
    if !settings.height.is_finite()
        || settings.height < 0.0
        || settings.height.ceil() - 1.0 > f64::from(MAX_SAFE_SWEEP_STEPS)
    {
        return Err(SafeHeightError::SweepLimit);
    }
    let current_damage = damage_at([0.0; 3]).map_err(SafeHeightError::Probe)?;
    let total_health = f64::from(health + absorption);
    let threshold = f64::from(settings.step_health);
    let mut safer = |translation| -> Result<bool, SafeHeightError<E>> {
        let damage = damage_at(translation).map_err(SafeHeightError::Probe)?;
        Ok(
            (total_health > threshold && total_health - damage > threshold)
                || damage - current_damage <= 0.0,
        )
    };
    let mut h = 0.0;
    let mut i: i32 = 1;
    while f64::from(i) < settings.height {
        let y = f64::from(i);
        if !safer([0.0, y, 0.0])? || !safer([offset[0], y, offset[2]])? {
            return Ok(h);
        }
        h += 1.0;
        i += 1;
    }
    // Source performs no forward-offset query at the final (possibly fractional) height.
    if safer([0.0, settings.height, 0.0])? {
        h = settings.height;
    }
    Ok(h)
}
