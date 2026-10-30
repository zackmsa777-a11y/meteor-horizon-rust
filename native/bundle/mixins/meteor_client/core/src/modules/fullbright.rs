// SPDX-License-Identifier: GPL-3.0-only
// Fullbright.java + brightness/entity mixins. Copyright (c) Meteor Development.
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Gamma,
    Potion,
    Luminance,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum LightType {
    #[default]
    BLOCK,
    SKY,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub mode: Mode,
    pub light_type: LightType,
    pub minimum_light_level: i32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: Mode::Gamma,
            light_type: LightType::BLOCK,
            minimum_light_level: 8,
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if !(0..=15).contains(&self.minimum_light_level) {
            Err("minimum-light-level must be between 0 and 15".into())
        } else {
            Ok(())
        }
    }
    pub fn gamma(&self, active: bool) -> bool {
        active && self.mode == Mode::Gamma
    }
    /// [minimum block, minimum sky], for raw RENDER samples before any averaging.
    pub fn floors(&self, active: bool) -> [u8; 2] {
        if !active || self.mode != Mode::Luminance {
            return [0, 0];
        }
        match self.light_type {
            LightType::BLOCK => [self.minimum_light_level as u8, 0],
            LightType::SKY => [0, self.minimum_light_level as u8],
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    RebuildSections,
    RemoveNightVision,
}
pub fn activate(settings: &Settings) -> Vec<Effect> {
    if settings.mode == Mode::Luminance {
        vec![Effect::RebuildSections]
    } else {
        Vec::new()
    }
}
pub fn deactivate(settings: &Settings) -> Vec<Effect> {
    match settings.mode {
        Mode::Luminance => vec![Effect::RebuildSections],
        Mode::Potion => vec![Effect::RemoveNightVision],
        Mode::Gamma => Vec::new(),
    }
}
pub fn settings_changed(old: &Settings, new: &Settings, active: bool) -> Vec<Effect> {
    if !active {
        return Vec::new();
    }
    let mut effects = Vec::new();
    if old.mode != new.mode && new.mode != Mode::Potion {
        effects.push(Effect::RemoveNightVision);
    }
    if old != new {
        effects.push(Effect::RebuildSections);
    }
    effects
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NightVision {
    None,
    Add420,
    SetDuration420,
}
/// Source compares getDuration directly: even -1 (an infinite effect) becomes 420.
/// Extend duration in place, preserving amplifier/ambient/particles/hidden effects.
pub fn post_tick(settings: &Settings, player_exists: bool, duration: Option<i32>) -> NightVision {
    if !player_exists || settings.mode != Mode::Potion {
        return NightVision::None;
    }
    match duration {
        None => NightVision::Add420,
        Some(d) if d < 420 => NightVision::SetDuration420,
        _ => NightVision::None,
    }
}
