// SPDX-License-Identifier: GPL-3.0-only
// LiquidInteract camera source-fluid retry using native fluid shape cache.
use super::{Game, Hit};
use horizon_core::{
    geometry::{Direction, Vec3, traverse_blocks},
    position::BlockPos,
};
use horizon_world::fluid::{FluidHeightWorld, FluidState};
use std::convert::Infallible;
struct Height<'g, 'a>(&'g Game<'a>);
impl FluidHeightWorld for Height<'_, '_> {
    type Error = Infallible;
    fn fluid(&mut self, p: [i32; 3]) -> Result<FluidState, Self::Error> {
        Ok(self
            .0
            .motion
            .fluids
            .fluid(self.0.world.block(p[0], p[1], p[2])))
    }
    fn sturdy_down(&mut self, p: [i32; 3]) -> Result<bool, Self::Error> {
        Ok(self
            .0
            .motion
            .fluids
            .sturdy(self.0.world.block(p[0], p[1], p[2]), Direction::Down))
    }
}
impl Game<'_> {
    pub(super) fn meteor_pick_source_fluid(
        &self,
        origin: [f64; 3],
        dir: [f64; 3],
        reach: f64,
    ) -> Option<Hit> {
        let intervals = self.loaded_ray_intervals(origin, dir, reach);
        let max_end = intervals.last()?.1;
        let eye = Vec3::new(origin[0], origin[1], origin[2]);
        let at = |t: f64| {
            Vec3::new(
                origin[0] + dir[0] * t,
                origin[1] + dir[1] * t,
                origin[2] + dir[2] * t,
            )
        };
        // Reach can overflow positively: clip only its undefined infinite endpoint
        // to the final resident column while preserving all finite-ray distances.
        let end = at(if reach.is_finite() { reach } else { max_end });
        let mut cache = self
            .motion
            .fluids
            .shapes
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut height = Height(self);
        for (start, finish) in intervals {
            let from = at(start);
            let to = at(finish);
            let span = to.subtract(from);
            let visits = (span.x.abs() + span.y.abs() + span.z.abs()).ceil() as usize + 8;
            let hit = traverse_blocks(from, to, visits, |pos: BlockPos| {
                let fluid = self
                    .motion
                    .fluids
                    .fluid(self.world.block(pos.x, pos.y, pos.z));
                if !fluid.is_source() {
                    return None;
                }
                let shape = match cache.shape(&mut height, fluid, pos) {
                    Ok(s) => s,
                    Err(e) => match e {},
                };
                shape.clip(eye, end, pos)
            })
            .ok()
            .flatten();
            if let Some(hit) = hit {
                let face = match hit.direction {
                    Direction::Down => [0, -1, 0],
                    Direction::Up => [0, 1, 0],
                    Direction::North => [0, 0, -1],
                    Direction::South => [0, 0, 1],
                    Direction::West => [-1, 0, 0],
                    Direction::East => [1, 0, 0],
                };
                return Some(Hit {
                    inside: hit.inside,
                    pos: [hit.block.x, hit.block.y, hit.block.z],
                    face,
                    point: [hit.location.x, hit.location.y, hit.location.z],
                });
            }
        }
        None
    }
}
