// SPDX-License-Identifier: GPL-3.0-only
// FastClimb/Parkour host queries. Copyright (c) Meteor Development.
use super::{Aabb, BlockPos, MotionWorld, Result, Vec3};
use meteor_client::modules::{fast_climb, parkour};

impl MotionWorld<'_, '_, '_> {
    pub fn meteor_climbing_player(&mut self) -> Result<fast_climb::Player> {
        let mut player = fast_climb::Player {
            horizontal_collision: self.player.collision.horizontal,
            jumping: self.player.jumping,
            ..Default::default()
        };
        if !player.horizontal_collision && !player.jumping {
            return Ok(player);
        }
        player.on_climbable = self.climbable()?;
        if !player.on_climbable {
            let state = self.block(BlockPos::containing(self.player.position));
            player.in_powder_snow_block = self
                .assets
                .registry
                .state(state)
                .ok_or("FastClimb encountered an unknown block state")?
                .name
                == "minecraft:powder_snow";
            // PowderSnowBlock's entity-type whitelist does not include the local player.
            player.can_walk_powder_snow = self.player.leather_boots;
        }
        Ok(player)
    }

    pub fn meteor_parkour_post(
        &mut self,
        settings: &parkour::Settings,
        jump_down: bool,
        sneak_down: bool,
    ) -> Result<()> {
        let player = parkour::Player {
            on_ground: self.player.ground.on_ground,
            jump_key_down: jump_down,
            shift_key_down: self.player.is_shift_key_down(),
            sneak_key_down: sneak_down,
        };
        if !parkour::eligible(player) {
            return Ok(());
        }
        let bounds = self.player.bounds();
        let min = bounds.min_position();
        let max = bounds.max_position();
        let (min, max) =
            parkour::adjusted_box(settings, [min.x, min.y, min.z], [max.x, max.y, max.z]);
        let bounds = Aabb::new(
            Vec3::new(min[0], min[1], min[2]),
            Vec3::new(max[0], max[1], max[2]),
        );
        let context = self.context();
        let has_collision = {
            let mut scene = self.collisions();
            let mut scan = horizon_world::block_collision_scan::BlockCollisionScan::new(
                &mut scene, context, bounds, false, 1_000_000,
            )?;
            match scan.next() {
                Some(hit) => {
                    hit?;
                    true
                }
                None => false,
            }
        };
        if parkour::should_jump(player, has_collision) {
            self.jump_from_ground()?;
        }
        Ok(())
    }
}
