// SPDX-License-Identifier: GPL-3.0-only
// Multitask.java and MinecraftMixin action gates. Copyright (c) Meteor Development.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub attacking_entities: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { attacking_entities: true }
    }
}

impl Settings {
    /// handleKeybinds' using-item branch alone respects attacking-entities.
    pub fn entity_attacks(&self, active: bool) -> bool {
        active && self.attacking_entities
    }
}

/// startUseItem's destroying-block check and continueAttack's using-item check
/// are bypassed whenever the module is active, independently of its setting.
pub fn exclusive_action(active: bool, original: bool) -> bool {
    !active && original
}
