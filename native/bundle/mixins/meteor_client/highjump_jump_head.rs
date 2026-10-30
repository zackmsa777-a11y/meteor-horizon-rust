// SPDX-License-Identifier: GPL-3.0-only
fn meteor_highjump_jump(world: &mut crate::motion::MotionWorld<'_, '_, '_>) -> ::core::ops::ControlFlow<Result<()>> {
    crate::motion::meteor_highjump::jump(world)
}
