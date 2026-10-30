// SPDX-License-Identifier: GPL-3.0-only
// Native AutoRespawn/FastUse helpers. Copyright (c) Meteor Development.
use super::{Game, Hand, Result};
use meteor_client::modules::fast_use;

impl Game<'_> {
    /// Before assigning Screen::Dead/capture(false), while AutoRespawn is enabled. Returning
    /// true means cancel the pending death screen. Use the actual respawn operation; never heal
    /// the existing entity or bypass the server's respawn protocol.
    ///
    /// WaypointsModule.addDeath is a source dependency: default maxDeathPositions=0,dpChat=false
    /// produce no output in a fresh native client. Nondefault Meteor waypoint persistence/chat
    /// require that module's port and are NOT represented by vanilla last_death/locator markers.
    pub(super) fn meteor_auto_respawn_open_death(&mut self) -> Result<bool> {
        if self.player.is_none() {
            return Ok(false);
        }
        self.respawn_player()?;
        Ok(true)
    }

    /// Run when a FastUse settings candidate is edited/imported; registry validation prevents
    /// typos/unavailable IDs from becoming a silently ineffective Some-mode allowlist.
    pub(super) fn meteor_fast_use_validate(&self, settings: &fast_use::Settings) -> Result<()> {
        settings.validate()?;
        for id in &settings.items {
            if self.items().get(id).is_none() {
                return Err(format!("Unknown FastUse item: {id}").into());
            }
        }
        Ok(())
    }

    /// startUseItem's per-hand entry, before item enabled/interaction checks. Main is called
    /// before its entity/block interaction, Off only if the main turn passes. A passing Off
    /// turn can replace the main hand's delay; do not just select the successful item's delay.
    pub(super) fn meteor_fast_use_hand(
        &mut self,
        settings: &fast_use::Settings,
        hand: Hand,
    ) -> Result<()> {
        let stack = match hand {
            Hand::Main => self.stack_in(self.selected),
            Hand::Off => self.offhand.as_ref(),
        };
        let item = stack.map_or("minecraft:air", |s| s.item.as_ref());
        let is_block = if item == "minecraft:air" {
            false
        } else {
            self.items()
                .get(item)
                .ok_or_else(|| format!("Unknown held item: {item}"))?
                .block
                .is_some()
        };
        self.use_delay = fast_use::cooldown(settings, item, is_block) as u32;
        Ok(())
    }
}
