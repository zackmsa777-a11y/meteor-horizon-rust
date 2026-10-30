// SPDX-License-Identifier: GPL-3.0-only
// DamageUtils.java native crystal predictor. Copyright (c) Meteor Development.
use super::{Game, Result};
use horizon_client::collisions::SceneCollisions;
use horizon_core::geometry::{Vec3, traverse_blocks};
use horizon_world::{collision_context::CollisionContext, damage_source::Scaling};
use meteor_client::damage::{self, Bounds, Difficulty, Mitigation};
use std::io;

impl Game<'_> {
    /// Highest damage among resident live rendered crystals. Destroyed/despawned crystals are
    /// removed from Game.crystals by the entity handlers; no distance-based threat approximation.
    /// SafeStep passes translated bounds, while the player's position stays unchanged.
    pub(super) fn meteor_crystal_damage_at_box(&mut self, bounds: Bounds) -> Result<f64> {
        if self.creative {
            return Ok(0.0);
        }
        let catalog = &self.motion.health.damage;
        let id = catalog
            .id("minecraft:explosion")
            .ok_or("missing explosion damage definition")?;
        let source = catalog
            .source::<()>(id, None, None, None)
            .ok_or("missing explosion source")?;
        let mut protection = 0i32;
        // DamageUtils manually reads these levels, not datapack-driven damage_protection effects.
        for stack in self.armor.iter().flatten() {
            for (name, multiplier, applies) in [
                ("minecraft:protection", 1i32, true),
                (
                    "minecraft:fire_protection",
                    2,
                    source.in_tag("minecraft:is_fire"),
                ),
                (
                    "minecraft:blast_protection",
                    2,
                    source.in_tag("minecraft:is_explosion"),
                ),
                (
                    "minecraft:projectile_protection",
                    2,
                    source.in_tag("minecraft:is_projectile"),
                ),
                (
                    "minecraft:feather_falling",
                    3,
                    source.in_tag("minecraft:is_fall"),
                ),
            ] {
                let level = stack.enchantment_level(name) as i32;
                if applies && level > 0 {
                    protection = protection.wrapping_add(level.wrapping_mul(multiplier));
                }
            }
        }
        let player = self
            .player
            .as_mut()
            .ok_or("crystal prediction requires a player")?;
        let position = [player.position.x, player.position.y, player.position.z];
        let mitigation = Mitigation {
            difficulty: match self.ui.difficulty {
                0 => Difficulty::Peaceful,
                1 => Difficulty::Easy,
                3 => Difficulty::Hard,
                _ => Difficulty::Normal,
            },
            // explosion(null) has no causing entity, so LivingNonPlayer scaling is false.
            scales_with_difficulty: source.definition().scaling == Scaling::Always,
            armor: player.attribute("minecraft:armor")?,
            toughness: player.attribute("minecraft:armor_toughness")?,
            resistance_amplifier: player.status.resistance,
            protection,
            bypasses_invulnerability: source.in_tag("minecraft:bypasses_invulnerability"),
        };
        let mut maximum = 0.0f64;
        for crystal in &self.crystals {
            let center = [crystal.position.x, crystal.position.y, crystal.position.z];
            if damage::distance(position, center) > 12.0 {
                continue;
            }
            let exposure =
                damage::exposure(center, bounds, |from, to| self.meteor_crystal_ray(from, to))
                    .map_err(|e| io::Error::other(format!("Meteor crystal exposure: {e:?}")))?;
            maximum = maximum.max(f64::from(damage::crystal_damage(
                position, center, exposure, mitigation,
            )));
        }
        Ok(maximum)
    }

    fn meteor_crystal_ray(&self, from: [f64; 3], to: [f64; 3]) -> Result<bool> {
        let from = Vec3::new(from[0], from[1], from[2]);
        let to = Vec3::new(to[0], to[1], to[2]);
        let mut scene = SceneCollisions {
            world: &self.world,
            binding: self.collision,
            subject: None,
        };
        let context = CollisionContext::empty(false);
        let hit = traverse_blocks(from, to, 1_000_000, |position| {
            let state = self.world.block(position.x, position.y, position.z);
            if self.res.facts().get(state).explosion_resistance < 600.0 {
                return None;
            }
            let shape = self
                .collision
                .collision_shape_with_context(&mut scene, state, position, &context);
            match shape {
                Ok(shape) if shape.clip(from, to, position).is_some() => Some(Ok(true)),
                Ok(_) => None,
                Err(error) => Some(Err(io::Error::other(error))),
            }
        })?;
        Ok(hit.transpose()?.unwrap_or(false))
    }
}
