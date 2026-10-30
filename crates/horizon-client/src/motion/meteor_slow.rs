// SPDX-License-Identifier: GPL-3.0-only
// NoSlow/Slippy native block-factor queries. Copyright (c) Meteor Development.
use super::{BlockPos, MotionWorld, Result, movement_support};
use meteor_client::modules::{no_slow, slippy};
impl MotionWorld<'_, '_, '_> {
    /// Entity.getBlockSpeedFactor with NoSlow's honey/soul-sand block substitution.
    /// Keep support resolution and movement_efficiency interpolation unchanged.
    pub fn meteor_no_slow_speed_factor(&mut self, settings: &no_slow::Settings) -> Result<f32> {
        let location = self.location();
        let current_id = self.block(location.block_position);
        let current = self.resources.blocks.get(current_id)?;
        let current_name = self
            .assets
            .registry
            .state(current_id)
            .ok_or("Unknown movement block")?
            .name
            .as_str();
        let factor = if settings.neutral_speed_block(current_name) {
            1.0
        } else {
            current.coefficients.speed_factor
        };
        let block = if current.water || current.bubble_column || factor != 1.0 {
            factor
        } else {
            let below = self.support(movement_support::MOVEMENT_SUPPORT_OFFSET)?;
            let below_id = self.block(below);
            let below_name = self
                .assets
                .registry
                .state(below_id)
                .ok_or("Unknown support block")?
                .name
                .as_str();
            if settings.neutral_speed_block(below_name) {
                1.0
            } else {
                self.resources
                    .blocks
                    .get(below_id)?
                    .coefficients
                    .speed_factor
            }
        };
        let efficiency = self.player.attribute("minecraft:movement_efficiency")? as f32;
        Ok(block + efficiency * (1.0 - block))
    }
    pub fn meteor_block_friction(
        &self,
        pos: BlockPos,
        settings: Option<&slippy::Settings>,
        no_slow_slime: bool,
    ) -> Result<f32> {
        let id = self.block(pos);
        let name = self
            .assets
            .registry
            .state(id)
            .ok_or("Unknown friction block")?
            .name
            .as_str();
        let original = self.resources.blocks.get(id)?.coefficients.friction;
        Ok(slippy::friction(settings, no_slow_slime, name, original))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn no_slow_keeps_raw_fluid_samples_and_restores_ordinary_player_queries() {
        use super::super::{Assets, MotionResources, PlayerMotion, Vec3};
        let root = std::env::var_os("HORIZON_TEST_RESOURCES")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../.local/horizon-client")
                    .join(horizon_core::VANILLA_VERSION)
            });
        if !root.join("movement-properties.nbt").exists() {
            eprintln!(
                "Skipped NoSlow motion regression: set HORIZON_TEST_RESOURCES to prepared resources"
            );
            return;
        }
        let assets = Assets::load(&root).expect("prepared test assets");
        let resources = MotionResources::load(&root, &assets).unwrap();
        let mut p = PlayerMotion::new(&resources, Vec3::ZERO).unwrap();
        p.fluids.was_touching_water = true;
        p.fluids.was_eye_in_water = true;
        p.health.food.food = 0;
        p.local_crouching = true;
        assert!(p.is_in_water());
        assert!(!p.movement_enough_food());
        assert!(p.moving_slowly());
        p.meteor_no_slow = Some(no_slow::Settings {
            fluid_drag: true,
            sneaking: true,
            hunger: true,
            ..Default::default()
        });
        assert!(!p.is_in_water());
        assert!(p.is_under_water());
        assert!(p.fluids.was_touching_water);
        assert!(p.movement_enough_food());
        assert!(!p.moving_slowly());
        p.ability_walk_speed = 0.2;
        p.meteor_no_slow.as_mut().unwrap().slowness = true;
        assert_eq!(p.movement_speed().unwrap(), 0.2);
        p.meteor_no_slow = None;
        assert_eq!(p.movement_speed().unwrap(), 0.1);
        assert!(p.is_in_water());
        assert!(!p.movement_enough_food());
        assert!(p.moving_slowly());
    }
}
