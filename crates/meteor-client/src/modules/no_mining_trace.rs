// SPDX-License-Identifier: GPL-3.0-only
// NoMiningTrace.java + LocalPlayerMixin. Copyright (c) Meteor Development.
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub blacklisted_entities: Vec<String>,
    pub only_when_holding_a_pickaxe: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            blacklisted_entities: Vec::new(),
            only_when_holding_a_pickaxe: true,
        }
    }
}
impl Settings {
    pub fn can_work(
        &self,
        active: bool,
        main_pickaxe: bool,
        offhand_pickaxe: bool,
        entity: Option<&str>,
    ) -> bool {
        active
            && (!self.only_when_holding_a_pickaxe || main_pickaxe || offhand_pickaxe)
            && entity.is_none_or(|e| !self.blacklisted_entities.iter().any(|x| x == e))
    }
    /// Called only after selecting the original nearest entity. A MISS must never erase it.
    pub fn replace_entity_hit(
        &self,
        active: bool,
        main_pickaxe: bool,
        offhand_pickaxe: bool,
        entity: &str,
        has_block: bool,
    ) -> bool {
        has_block && self.can_work(active, main_pickaxe, offhand_pickaxe, Some(entity))
    }
}
