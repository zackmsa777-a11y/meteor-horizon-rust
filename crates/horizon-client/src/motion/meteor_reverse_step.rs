// SPDX-License-Identifier: GPL-3.0-only
// ReverseStep.java host collision adapter. Copyright (c) Meteor Development.
use super::{Aabb, BlockPos, MotionWorld, Result, Vec3};
use crate::collisions::{EntityCollisions, SceneCollisions};
use horizon_world::entity_obstruction::EntityIdentity;
use horizon_world::{collision_context::CollisionContext, world_movement};
use meteor_client::modules::reverse_step::{self, Settings};

impl MotionWorld<'_, '_, '_> {
    /// Tick.Post player branch; caller handles vehicle routing and actual physical jump binding.
    pub fn meteor_reverse_step_player(
        &mut self,
        settings: &Settings,
        jump_key_down: bool,
    ) -> Result<()> {
        let p = &self.player;
        if p.is_shift_key_down() || (p.move_vector[0] == 0.0 && p.move_vector[1] == 0.0) {
            return Ok(());
        }
        let entity = reverse_step::Entity {
            on_ground: p.ground.on_ground,
            under_water: p.is_under_water(),
            in_lava: p.is_in_lava(),
            no_physics: p.spectator,
        };
        let position = [p.position.x, p.position.y, p.position.z];
        // BedBlock is the sixteen colored beds; AbstractBedBlock's straw bed is not BedBlock.
        let mut missing = false;
        let on_bed = reverse_step::on_bed(position, |at| {
            let state = self.block(BlockPos { x: at[0], y: at[1], z: at[2] });
            let Some(state) = self.assets.registry.state(state) else {
                missing = true;
                return false;
            };
            let Some(color) = state
                .name
                .strip_prefix("minecraft:")
                .and_then(|v| v.strip_suffix("_bed"))
            else {
                return false;
            };
            matches!(
                color,
                "white"
                    | "orange"
                    | "magenta"
                    | "light_blue"
                    | "yellow"
                    | "lime"
                    | "pink"
                    | "gray"
                    | "light_gray"
                    | "cyan"
                    | "purple"
                    | "blue"
                    | "brown"
                    | "green"
                    | "red"
                    | "black"
            )
        });
        if missing {
            return Err("ReverseStep encountered an unknown block state".into());
        }
        if on_bed || !reverse_step::eligible(entity, jump_key_down) {
            return Ok(());
        }
        let bounds = self.player.bounds();
        let min = bounds.min_position();
        let max = bounds.max_position();
        let dy = settings.probe_y();
        let shifted = Aabb::new(
            Vec3::new(min.x, min.y + dy, min.z),
            Vec3::new(max.x, max.y + dy, max.z),
        );
        // Level.noCollision(AABB) uses an empty collision context, unlike SafeWalk's
        // noCollision(player, AABB); do not accidentally supply leather boots/descending state.
        let mut collisions = EntityCollisions {
            blocks: SceneCollisions {
                world: self.scene,
                binding: self.collision_binding,
                subject: None,
            },
            entities: self.entities,
            border: None,
        };
        let clear = world_movement::no_world_collision(
            &mut collisions,
            world_movement::CollisionCheckRequest {
                entity: EntityIdentity(0),
                context: CollisionContext::empty(false),
                bounds: shifted,
                candidate_budget: 1_000_000,
            },
        )?;
        let v = self.player.velocity;
        let next = reverse_step::velocity(settings, entity, jump_key_down, clear, [v.x, v.y, v.z]);
        self.player.velocity = Vec3::new(next[0], next[1], next[2]);
        Ok(())
    }
}
