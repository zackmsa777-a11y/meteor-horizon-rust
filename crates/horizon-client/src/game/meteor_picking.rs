// SPDX-License-Identifier: GPL-3.0-only
// AirPlace/NoMiningTrace/LiquidInteract native adapters. Copyright (c) Meteor Development.
use super::{Game, Hand, Hit, KeyCode};
use horizon_core::{
    geometry::{Aabb, Vec3},
    position::BlockPos,
};
use horizon_world::level::LevelAccess;
use meteor_client::modules::{
    air_place::{self, ItemKind},
    no_mining_trace,
};
impl Game<'_> {
    pub(super) fn meteor_trace_validate(&self, s: &no_mining_trace::Settings) -> super::Result<()> {
        for id in &s.blacklisted_entities {
            if !self
                .ui
                .registries
                .entity_types
                .iter()
                .any(|known| known == id)
            {
                return Err(format!("Unknown NoMiningTrace entity: {id}").into());
            }
        }
        Ok(())
    }
    /// Select first, then apply NoMiningTrace. Filtering individual candidates
    /// would expose a blacklisted entity behind the actual selected entity.
    pub(super) fn meteor_camera_block_candidate(
        &self,
        origin: [f64; 3],
        dir: [f64; 3],
    ) -> Option<Hit> {
        let reach = self.reach().max(self.player_entity_range());
        let solid = self.clip_ray(origin, dir, reach);
        if meteor_client::modules::liquid_interact::retry_with_source_fluids(
            self.meteor_interaction_active("liquid-interact"),
            solid.is_none(),
        ) {
            self.meteor_pick_source_fluid(origin, dir, reach)
        } else {
            solid
        }
    }
    pub(super) fn meteor_trace_replaces_entity(&self) -> bool {
        if !self.meteor_interaction_active("no-mining-trace") {
            return false;
        }
        let Some((eye, dir)) = self.aim() else {
            return false;
        };
        self.meteor_camera_block_candidate(eye, dir)
            .is_some_and(|hit| self.meteor_trace_replaces_with(hit))
    }
    /// The source mixin returns the raw blockHitResult, even when it lies
    /// beyond block reach (Reach can make entity reach larger than block reach).
    pub(super) fn meteor_trace_replaces_with(&self, block: Hit) -> bool {
        if !self.meteor_interaction_active("no-mining-trace") {
            return false;
        }
        let settings = &self.meteor.client.config.no_mining_trace;
        let pickaxe = |stack: Option<&horizon_client::items::ItemStack>| {
            stack.is_some_and(|s| {
                self.items()
                    .item_tags
                    .get("minecraft:pickaxes")
                    .is_some_and(|tag| tag.contains(&*s.item))
            })
        };
        let main = pickaxe(self.stack_in(self.selected));
        let off = pickaxe(self.offhand.as_ref());
        if !settings.can_work(true, main, off, None) {
            return false;
        }
        let Some((eye, dir)) = self.aim() else {
            return false;
        };
        let block_distance = (0..3)
            .map(|i| (block.point[i] - eye[i]).powi(2))
            .sum::<f64>()
            .sqrt();
        let mut best: Option<(f64, String)> = None;
        let mut offer = |distance: f64, kind: String| {
            if distance < block_distance
                && distance < self.player_entity_range()
                && best.as_ref().is_none_or(|b| distance < b.0)
            {
                best = Some((
                    distance,
                    if kind.contains(':') {
                        kind
                    } else {
                        format!("minecraft:{kind}")
                    },
                ));
            }
        };
        if let Some((i, d)) = self.mob_hit() {
            offer(d, self.mobs[i].kind.id.to_string());
        }
        if let Some((_, d)) = self.guest_hit() {
            offer(d, "minecraft:player".into());
        }
        if let Some((i, d)) = self.boat_hit() {
            offer(d, self.boats[i].entity_type());
        }
        if let Some((i, d)) = self.cart_hit() {
            offer(d, self.carts[i].kind.id().to_owned());
        }
        if let Some((_, d, _)) = self.armor_stand_hit() {
            offer(d, "minecraft:armor_stand".into());
        }
        if let Some((_, d, _)) = self.cushion_hit() {
            offer(d, "minecraft:cushion".into());
        }
        if let Some((i, d)) = self.frame_hit() {
            offer(d, self.item_frames[i].kind().into());
        }
        if let Some((_, d)) = self.painting_hit() {
            offer(d, "minecraft:painting".into());
        }
        if let Some((i, d)) = self.redirectable_hit() {
            offer(d, self.creatures.projectiles[i].kind.id().into());
        }
        let reach = self.player_entity_range();
        for c in self.crystals.iter() {
            let p = c.position;
            let bounds = Aabb::new(
                Vec3::new(p.x - 1.0, p.y, p.z - 1.0),
                Vec3::new(p.x + 1.0, p.y + 2.0, p.z + 1.0),
            );
            if let Some(d) = super::ray_box(eye, dir, reach, bounds) {
                offer(d, "minecraft:end_crystal".into());
            }
        }
        if let Some(dragon) = self.dragon.as_ref().filter(|d| d.alive()) {
            for i in 0..8 {
                if let Some(d) = super::ray_box(eye, dir, reach, dragon.part_box(i)) {
                    offer(d, "minecraft:ender_dragon".into());
                }
            }
        }
        best.is_some_and(|(_, kind)| settings.replace_entity_hit(true, main, off, &kind, true))
    }
    fn meteor_air_item(&self, stack: Option<&horizon_client::items::ItemStack>) -> ItemKind {
        let Some(stack) = stack else {
            return ItemKind::Other;
        };
        let Some(info) = self.items().get(&stack.item) else {
            return ItemKind::Other;
        };
        if info.block.is_some() {
            ItemKind::Block
        } else if stack.item.ends_with("_spawn_egg") {
            ItemKind::SpawnEgg
        } else if &*stack.item == "minecraft:firework_rocket" {
            ItemKind::FireworkRocket
        } else if &*stack.item == "minecraft:armor_stand" {
            ItemKind::ArmorStand
        } else {
            ItemKind::Other
        }
    }
    fn meteor_air_hands(&self) -> bool {
        let flying = self.player.as_ref().is_some_and(|p| p.fall_flying);
        [self.stack_in(self.selected), self.offhand.as_ref()]
            .into_iter()
            .any(|s| self.meteor_air_item(s).placeable(flying))
    }
    pub(super) fn meteor_picking_pre(&mut self) {
        if self.meteor_interaction_active("no-mining-trace") {
            if let Err(error) =
                self.meteor_trace_validate(&self.meteor.client.config.no_mining_trace)
            {
                self.meteor_disable_error("no-mining-trace", error);
            }
        }
        if !self.meteor_interaction_active("air-place")
            || !self.meteor_air_hands()
            || self.pick().is_some()
            || self.entity_targeted()
        {
            return;
        }
        let Some(player) = self.player.as_ref() else {
            return;
        };
        // cameraEntity.pick(..., 0, false): previous feet, current pose eye-height;
        // Entity.turn updates both rotations, except a boat's post-save yaw turn.
        let origin = [
            self.previous_position.x,
            self.previous_position.y + f64::from(player.dimensions.eye_height),
            self.previous_position.z,
        ];
        let ride_turn = self
            .riding
            .and_then(|id| self.boats.iter().find(|b| b.id == id))
            .map_or(0.0, |b| b.delta_rotation);
        let dir = super::view_vector(self.yaw - ride_turn, self.pitch);
        let range = self
            .meteor
            .client
            .config
            .air_place
            .pick_range(self.player_block_range());
        let pos = self.clip_ray(origin, dir, range).map_or_else(
            || std::array::from_fn(|i| (origin[i] + dir[i] * range).floor() as i32),
            |hit| hit.pos,
        );
        self.meteor.air_place.hit = Some(air_place::Hit { pos });
    }
    pub(super) fn meteor_air_use(&mut self, hand: Hand) -> bool {
        if !self.meteor_interaction_active("air-place") {
            return false;
        }
        let Some(hit) = self.meteor.air_place.hit else {
            return false;
        };
        let held = self.with_hand(hand, |g| g.held().cloned());
        let item = self.meteor_air_item(held.as_ref());
        if !item.placeable(self.player.as_ref().is_some_and(|p| p.fall_flying)) {
            return false;
        }
        let [x, y, z] = hit.pos;
        if !(-30_000_000..30_000_000).contains(&x)
            || !(-30_000_000..30_000_000).contains(&z)
            || !(-20_000_000..20_000_000).contains(&y)
        {
            return false;
        }
        if !self.res.facts().get(self.world.block(x, y, z)).replaceable {
            return false;
        }
        if item.check_entities() {
            let block = held
                .as_ref()
                .and_then(|s| self.items().get(&s.item))
                .and_then(|i| i.block)
                .or_else(|| self.items().get("minecraft:obsidian").and_then(|i| i.block));
            let Some(block) = block else {
                return false;
            };
            let clear = self.with_level(|_, level, _| {
                level.player_collision = None;
                level.is_unobstructed(BlockPos::new(x, y, z), block)
            });
            if !clear {
                return false;
            }
        }
        // Entity.getMotionDirection is the horizontal facing; use its opposite.
        let face = match ((self.yaw / 90.0 + 0.5).floor() as i32).rem_euclid(4) {
            0 => [0, 0, -1],
            1 => [1, 0, 0],
            2 => [0, 0, 1],
            _ => [-1, 0, 0],
        };
        let forced = Hit {
            inside: false,
            pos: hit.pos,
            face,
            point: hit.pos.map(|v| f64::from(v) + 0.5),
        };
        let prior = self.forced_hit.replace(forced);
        let shift = self.keys.remove(&KeyCode::ShiftLeft);
        let _ = self.remote_use_item_on(hand);
        if shift {
            self.keys.insert(KeyCode::ShiftLeft);
        }
        self.forced_hit = prior;
        true // source event SUCCESS even when useItemOn returns PASS/FAIL
    }
    pub(super) fn meteor_air_overlay(
        &self,
        camera: [f64; 3],
    ) -> horizon_client::gfx::overlay_geometry::Geometry {
        let geometry = self.meteor_air_overlay_geometry(camera);
        self.meteor
            .air_place_preview
            .set([geometry.lines.len(), geometry.triangles.len()]);
        geometry
    }
    fn meteor_air_overlay_geometry(
        &self,
        camera: [f64; 3],
    ) -> horizon_client::gfx::overlay_geometry::Geometry {
        use horizon_client::gfx::overlay_geometry::{Geometry, box_geometry};
        if !self.meteor_interaction_active("air-place")
            || self.pick().is_some()
            || self.entity_targeted()
            || !self.meteor_air_hands()
        {
            return Geometry::default();
        }
        let Some(hit) = self.meteor.air_place.hit else {
            return Geometry::default();
        };
        let s = &self.meteor.client.config.air_place;
        let [x, y, z] = hit.pos;
        if !s.render || !self.res.facts().get(self.world.block(x, y, z)).replaceable {
            return Geometry::default();
        }
        let min = hit.pos.map(f64::from);
        let max = min.map(|v| v + 1.0);
        box_geometry(
            min,
            max,
            camera,
            s.side_color.rgba(),
            s.line_color.rgba(),
            s.shape_mode.sides(),
            s.shape_mode.lines(),
        )
    }
}
