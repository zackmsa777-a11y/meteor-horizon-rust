// SPDX-License-Identifier: GPL-3.0-only
// LiquidInteract.java + LocalPlayerMixin.modifyPick. Copyright (c) Meteor Development.
/// Retry SOURCE_ONLY fluid picking only when the ordinary block pick missed.
pub fn retry_with_source_fluids(active: bool, original_missed: bool) -> bool {
    active && original_missed
}

#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Settings {}
