// SPDX-License-Identifier: GPL-3.0-only
// Reach.java and PlayerMixin return-value hooks. Copyright (c) Meteor Development.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub extra_block_reach: f64,
    pub extra_entity_reach: f64,
}

impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if !self.extra_block_reach.is_finite() || !self.extra_entity_reach.is_finite() {
            return Err("Extra reach must be finite".into());
        }
        Ok(())
    }

    pub fn block(&self, active: bool, original: f64) -> f64 {
        (original + if active { self.extra_block_reach } else { 0.0 }).max(0.0)
    }

    pub fn entity(&self, active: bool, original: f64) -> f64 {
        (original + if active { self.extra_entity_reach } else { 0.0 }).max(0.0)
    }
}
