// SPDX-License-Identifier: GPL-3.0-only
fn meteor_highjump_block(world: &mut crate::motion::MotionWorld<'_, '_, '_>) -> ::core::ops::ControlFlow<Result<f32>> {
    crate::motion::meteor_highjump::block_factor(world)
}
