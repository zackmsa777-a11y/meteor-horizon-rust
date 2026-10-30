// SPDX-License-Identifier: GPL-3.0-only
// Engine adapter for SafeWalk.java. Copyright (c) Meteor Development.
//! Compile this module only with the `meteor` feature. Queries use live resident-world facts,
//! collider shapes and WATER fluid shapes; minimum-fall-distance is not approximated by air tests.
use super::{Aabb, BlockPos, MotionWorld, Result, Vec3, traverse_blocks};
use horizon_world::{heightmap::HeightmapType, worldgen::terrain::TerrainBlockStates};
use meteor_client::modules::safe_walk::{self, Bounds, Decision, FallProbe, Player, Settings};
use std::io;

impl MotionWorld<'_, '_, '_> {
    /// Called from staying_on_ground_surface, after edge_backoff's flying/upward/mover gates.
    /// The caller applies press_sneak to the input sneak bit and ORs force_clip with vanilla's
    /// staying-on-ground value. A false force_clip never cancels vanilla or another module's clip.
    pub(super) fn meteor_safe_walk_decision(
        &mut self,
        settings: &Settings,
        sprint_key_down: bool,
    ) -> Result<Decision> {
        let position = self.player.position;
        let block = BlockPos::containing(position);
        let player = Player {
            y: position.y,
            block_y: block.y,
            on_ground: self.player.ground.on_ground,
            shift_key_down: self.player.is_shift_key_down(),
            sprint_key_down,
        };
        // A distance of one bypasses both upstream queries. For larger distances, use the same
        // heightmap predicate as ClientLevel rather than collision occupancy or world-surface.
        let probe = if settings.minimum_fall_distance > 1 {
            let surface = self.scene.motion_blocking(block.x, block.z, |state| {
                HeightmapType::MotionBlocking
                    .is_opaque(self.assets.properties.heightmap_state(state))
            });
            FallProbe {
                motion_blocking_surface_y: surface,
                downward_hit_block_y: if block.y < surface {
                    self.meteor_safe_walk_hit_below(position)?
                } else {
                    None
                },
            }
        } else {
            FallProbe {
                motion_blocking_surface_y: 0,
                downward_hit_block_y: None,
            }
        };
        if !safe_walk::sufficient_drop(settings, player, probe) {
            return Ok(Decision::default());
        }
        let no_collision = if settings.sneak {
            let bounds = self.player.bounds();
            let min = bounds.min_position();
            let max = bounds.max_position();
            let original = Bounds::new([min.x, min.y, min.z], [max.x, max.y, max.z]);
            // LivingEntity.maxUpStep narrows the attribute to float before expansion.
            let step = f64::from(self.player.attribute("minecraft:step_height")? as f32);
            let adjusted = safe_walk::adjusted_box(original, step, settings.edge_distance);
            self.no_player_collision(Aabb::new(
                Vec3::new(adjusted.min[0], adjusted.min[1], adjusted.min[2]),
                Vec3::new(adjusted.max[0], adjusted.max[1], adjusted.max[2]),
            ))?
        } else {
            false
        };
        Ok(safe_walk::clip(settings, player, probe, no_collision))
    }

    /// ClipContext(COLLIDER, WATER) down to the resident world's minimum Y. Blocks are traversed
    /// in ray order; collider and water hits within a block share the block Y used by SafeWalk.
    fn meteor_safe_walk_hit_below(&mut self, from: Vec3) -> Result<Option<i32>> {
        let to = Vec3::new(from.x, f64::from(crate::chunk::MIN_Y), from.z);
        let binding = self.collision_binding;
        let context = self.context();
        let hit = traverse_blocks(from, to, 1_000_000, |position| {
            let result = (|| -> Result<bool> {
                let state = self.block(position);
                let collider = binding
                    .collision_shape_with_context(&mut self.collisions(), state, position, &context)
                    .map_err(io::Error::other)?;
                let block_hit = collider.clip(from, to, position).is_some();
                let water_hit = self.water_clip(from, to, position)?;
                Ok(block_hit || water_hit)
            })();
            match result {
                Ok(true) => Some(Ok(position.y)),
                Ok(false) => None,
                Err(error) => Some(Err(error)),
            }
        })?;
        hit.transpose()
    }
}
