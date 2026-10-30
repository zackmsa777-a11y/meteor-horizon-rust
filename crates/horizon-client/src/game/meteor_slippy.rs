// SPDX-License-Identifier: GPL-3.0-only
// Native Slippy settings registry validation. Copyright (c) Meteor Development.
use super::{Game, Result};
use meteor_client::modules::slippy;
impl Game<'_> {
    pub(super) fn meteor_slippy_validate(&self, settings: &slippy::Settings) -> Result<()> {
        settings.validate()?;
        for id in settings
            .ignored_blocks
            .iter()
            .chain(&settings.allowed_blocks)
        {
            if self.pack.assets.registry.default_state(id).is_none() {
                return Err(format!("Unknown Slippy block: {id}").into());
            }
        }
        Ok(())
    }
}
