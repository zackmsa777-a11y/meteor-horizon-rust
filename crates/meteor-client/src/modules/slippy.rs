// SPDX-License-Identifier: GPL-3.0-only
// Slippy.java and BlockMixin.getFriction. Copyright (c) Meteor Development.
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ListMode {
    Whitelist,
    #[default]
    Blacklist,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub friction: f64,
    pub list_mode: ListMode,
    pub ignored_blocks: Vec<String>,
    pub allowed_blocks: Vec<String>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            friction: 1.0,
            list_mode: ListMode::Blacklist,
            ignored_blocks: Vec::new(),
            allowed_blocks: Vec::new(),
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if !self.friction.is_finite() || !(0.01..=1.10).contains(&self.friction) {
            return Err("friction must be between 0.01 and 1.10".into());
        }
        for id in self.ignored_blocks.iter().chain(&self.allowed_blocks) {
            let Some((namespace, path)) = id.split_once(':') else {
                return Err(format!("Block identifier must include namespace: {id}"));
            };
            let legal = |b: u8| {
                b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b'.' | b'-')
            };
            if namespace.is_empty()
                || path.is_empty()
                || !namespace.bytes().all(legal)
                || !path.bytes().all(|b| legal(b) || b == b'/')
            {
                return Err(format!("Invalid block identifier: {id}"));
            }
        }
        Ok(())
    }
    pub fn allows(&self, id: &str) -> bool {
        match self.list_mode {
            ListMode::Whitelist => self.allowed_blocks.iter().any(|v| v == id),
            ListMode::Blacklist => !self.ignored_blocks.iter().any(|v| v == id),
        }
    }
}
/// Slippy wins over NoSlow's slime-friction fallback. Call only for local prediction.
pub fn friction(
    settings: Option<&Settings>,
    no_slow_slime: bool,
    block: &str,
    original: f32,
) -> f32 {
    if let Some(settings) = settings.filter(|s| s.allows(block)) {
        settings.friction as f32
    } else if no_slow_slime && block == "minecraft:slime_block" {
        0.6
    } else {
        original
    }
}
