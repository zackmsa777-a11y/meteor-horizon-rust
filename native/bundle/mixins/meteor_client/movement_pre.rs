// SPDX-License-Identifier: GPL-3.0-only
// Source AutoJump Tick.Pre, before the pristine game tick body.
fn meteor_movement_pre(game: &mut crate::game::Game<'_>) {
    crate::game::meteor_client_migration::on_pre_tick(game);
}
