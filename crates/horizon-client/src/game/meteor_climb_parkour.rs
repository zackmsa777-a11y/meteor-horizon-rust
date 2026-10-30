// SPDX-License-Identifier: GPL-3.0-only
// Native module event adapters. Copyright (c) Meteor Development.
use super::{Game, Result};
use horizon_client::motion::MotionWorld;
use horizon_core::geometry::Vec3;
use meteor_client::modules::{fast_climb, parkour, spider};

impl Game<'_> {
    /// Tick.Pre: returns a shared Timer override effect. The caller must apply it to real
    /// tick scheduling. Do not expose timer-mode as functional until that hook is connected.
    pub(super) fn meteor_fast_climb_pre(
        &mut self,
        settings: &fast_climb::Settings,
        state: &mut fast_climb::State,
    ) -> Result<Option<f64>> {
        if !settings.timer_mode {
            return Ok(None);
        }
        let entities = self.player_entity_collisions();
        let Some(player) = self.player.as_mut() else {
            return Ok(None);
        };
        let snapshot = MotionWorld {
            entities: &entities,
            player,
            scene: &mut self.world,
            assets: &self.pack.assets,
            collision_binding: self.collision,
            resources: self.motion,
        }
        .meteor_climbing_player()?;
        Ok(state.pre_tick(settings, snapshot))
    }
    /// Tick.Post for speed-mode FastClimb; no Timer or scheduler side effects.
    pub(super) fn meteor_fast_climb_post(&mut self, settings: &fast_climb::Settings) -> Result<()> {
        if settings.timer_mode {
            return Ok(());
        }
        let entities = self.player_entity_collisions();
        let Some(player) = self.player.as_mut() else {
            return Ok(());
        };
        let mut world = MotionWorld {
            entities: &entities,
            player,
            scene: &mut self.world,
            assets: &self.pack.assets,
            collision_binding: self.collision,
            resources: self.motion,
        };
        let snapshot = world.meteor_climbing_player()?;
        let v = world.player.velocity;
        let next = fast_climb::post_tick(settings, snapshot, [v.x, v.y, v.z]);
        world.player.velocity = Vec3::new(next[0], next[1], next[2]);
        Ok(())
    }
    pub(super) fn meteor_spider_post(&mut self, settings: &spider::Settings) {
        if let Some(player) = self.player.as_mut() {
            let v = player.velocity;
            let next = spider::post_tick(settings, player.collision.horizontal, [v.x, v.y, v.z]);
            player.velocity = Vec3::new(next[0], next[1], next[2]);
        }
    }
    pub(super) fn meteor_parkour_post(&mut self, settings: &parkour::Settings) -> Result<()> {
        let down = |mapping: &str| {
            self.physical_keys.iter().any(|key| {
                horizon_client::keybinds::key_name(*key) == Some(self.settings.keys.get(mapping))
            }) || self.physical_buttons.iter().any(|button| {
                horizon_client::keybinds::mouse_name(*button) == self.settings.keys.get(mapping)
            })
        };
        let jump = down("key.jump");
        let sneak = down("key.sneak");
        let entities = self.player_entity_collisions();
        if let Some(player) = self.player.as_mut() {
            MotionWorld {
                entities: &entities,
                player,
                scene: &mut self.world,
                assets: &self.pack.assets,
                collision_binding: self.collision,
                resources: self.motion,
            }
            .meteor_parkour_post(settings, jump, sneak)?;
        }
        Ok(())
    }
}
