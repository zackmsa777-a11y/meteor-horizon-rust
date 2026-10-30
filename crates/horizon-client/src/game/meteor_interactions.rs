// SPDX-License-Identifier: GPL-3.0-only
// Reach/Multitask native local-player hooks. Copyright (c) Meteor Development.
use super::Game;
impl Game<'_> {
    pub(super) fn meteor_multitask_active(&self) -> bool {
        #[cfg(feature = "meteor")]
        {
            return self.native_window()
                && self.native_on("meteor-client")
                && self.meteor.client.active("multitask");
        }
        #[cfg(not(feature = "meteor"))]
        false
    }
    pub(super) fn meteor_multitask_entities(&self) -> bool {
        #[cfg(feature = "meteor")]
        {
            return self
                .meteor
                .client
                .config
                .multitask
                .entity_attacks(self.meteor_multitask_active());
        }
        #[cfg(not(feature = "meteor"))]
        false
    }
    /// Compute an attribute instance on a clone so picking never edits authoritative attributes.
    fn local_range_attribute(&self, name: &str, fallback: f64) -> f64 {
        if !self.native_window() {
            return fallback;
        }
        let Some(player) = self.player.as_ref() else {
            return fallback;
        };
        let Some(key) = player.attributes.key(name) else {
            return fallback;
        };
        let Some(handle) = player.attributes.lookup(Some(&key)) else {
            return fallback;
        };
        player
            .attributes
            .instance(&handle)
            .map_or(fallback, |instance| instance.clone().value())
    }
    pub(super) fn player_block_range(&self) -> f64 {
        let original = self.local_range_attribute(
            "minecraft:block_interaction_range",
            if self.creative { 5.0 } else { 4.5 },
        );
        #[cfg(feature = "meteor")]
        if self.native_window() && self.native_on("meteor-client") {
            return self
                .meteor
                .client
                .config
                .reach
                .block(self.meteor.client.active("reach"), original);
        }
        original
    }
    pub(super) fn player_entity_range(&self) -> f64 {
        let original = self.local_range_attribute(
            "minecraft:entity_interaction_range",
            if self.creative { 5.0 } else { 3.0 },
        );
        #[cfg(feature = "meteor")]
        if self.native_window() && self.native_on("meteor-client") {
            return self
                .meteor
                .client
                .config
                .reach
                .entity(self.meteor.client.active("reach"), original);
        }
        original
    }
}

impl Game<'_> {
    /// Missing columns and out-of-height samples are air. Long picks only visit
    /// loaded column intersections; distance remains measured from the eye.
    pub(super) fn loaded_ray_intervals(
        &self,
        origin: [f64; 3],
        dir: [f64; 3],
        reach: f64,
    ) -> Vec<(f64, f64)> {
        if origin.iter().chain(dir.iter()).any(|v| !v.is_finite())
            || dir == [0.0; 3]
            || reach.is_nan()
            || reach < 0.0
        {
            return Vec::new();
        }
        if reach <= 64.0 {
            return vec![(0.0, reach)];
        }
        debug_assert!(self.res.facts().is_air(self.world.air));
        debug_assert!(self.res.facts().is_air(super::BlockStateId(0)));
        ray_intervals::columns(
            origin,
            dir,
            reach,
            self.world.chunks.keys().copied(),
            horizon_client::chunk::MIN_Y,
            horizon_client::chunk::HEIGHT,
        )
    }
}
#[path = "ray_intervals.rs"]
mod ray_intervals;
