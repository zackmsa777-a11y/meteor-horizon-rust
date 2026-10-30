// SPDX-License-Identifier: GPL-3.0-only
// ReverseStep.java Tick.Post routing. Copyright (c) Meteor Development.
use super::{Game, Result};
use meteor_client::modules::reverse_step::Settings;

impl Game<'_> {
    /// Call only when the module/native parent is enabled and a world is loaded.
    /// Until all ridden entity fluid/noPhysics states are exposed, fail explicitly rather
    /// than silently applying the player's collision state to an unrelated vehicle.
    pub(super) fn meteor_reverse_step_post(&mut self, settings: &Settings) -> Result<()> {
        if self.riding.is_some() && settings.vehicles {
            return Err(
                "ReverseStep vehicles requires native ridden-entity fluid/collision integration"
                    .into(),
            );
        }
        let jump_key_down = self.physical_keys.iter().any(|key| {
            horizon_client::keybinds::key_name(*key) == Some(self.settings.keys.get("key.jump"))
        }) || self.physical_buttons.iter().any(|button| {
            horizon_client::keybinds::mouse_name(*button) == self.settings.keys.get("key.jump")
        });
        let entities = self.player_entity_collisions();
        if let Some(player) = self.player.as_mut() {
            horizon_client::motion::MotionWorld {
                entities: &entities,
                player,
                scene: &mut self.world,
                assets: &self.pack.assets,
                collision_binding: self.collision,
                resources: self.motion,
            }
            .meteor_reverse_step_player(settings, jump_key_down)?;
        }
        Ok(())
    }
}
