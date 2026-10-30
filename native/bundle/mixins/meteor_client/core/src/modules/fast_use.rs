// SPDX-License-Identifier: GPL-3.0-only
// Port of FastUse.java. Copyright (c) Meteor Development.
use serde::{Deserialize, Serialize};

/// Native settings editor accepts a JSON string array; command users may use the source's
/// comma-separated IDs. Both canonicalize missing namespaces to minecraft.
pub fn parse_items(value: &str) -> Result<Vec<String>, String> {
    let value = value.trim();
    let ids: Vec<String> = if value.starts_with('[') {
        serde_json::from_str(value)
            .map_err(|_| "items must be a JSON array of strings".to_string())?
    } else if value.is_empty() {
        Vec::new()
    } else {
        value.split(',').map(|s| s.trim().to_owned()).collect()
    };
    let ids = ids
        .into_iter()
        .map(|id| {
            if id.contains(':') {
                id
            } else {
                format!("minecraft:{id}")
            }
        })
        .collect();
    let candidate = Settings {
        items: ids,
        ..Default::default()
    };
    candidate.validate()?;
    Ok(candidate.items)
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    All,
    Some,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub mode: Mode,
    pub items: Vec<String>,
    pub blocks: bool,
    pub cooldown: i32,
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if self.cooldown < 0 {
            return Err("cooldown must be nonnegative".into());
        }
        for id in &self.items {
            let Some((namespace, path)) = id.split_once(':') else {
                return Err(format!("Item identifier must include namespace: {id}"));
            };
            let legal = |b: u8| {
                b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b'.' | b'-')
            };
            if namespace.is_empty()
                || path.is_empty()
                || !namespace.bytes().all(legal)
                || !path.bytes().all(|b| legal(b) || b == b'/')
            {
                return Err(format!("Invalid item identifier: {id}"));
            }
        }
        Ok(())
    }
}
/// Called before isItemEnabled for EACH hand reached by startUseItem. Later passing hands
/// may overwrite the delay selected for an earlier hand. This does not alter item-specific
/// cooldown components (pearls/shields/etc) or the time needed to finish eating.
pub fn cooldown(settings: &Settings, item_id: &str, is_block_item: bool) -> i32 {
    if settings.mode == Mode::All
        || (settings.blocks && is_block_item)
        || settings.items.iter().any(|v| v == item_id)
    {
        settings.cooldown
    } else {
        4
    }
}
