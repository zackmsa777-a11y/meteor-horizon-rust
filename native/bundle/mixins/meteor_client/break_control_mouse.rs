// SPDX-License-Identifier: GPL-3.0-only
// BreakDelay control-socket mouse press. Copyright (c) Meteor Development.
fn meteor_break_control_mouse(game: &mut crate::game::Game<'_>, pressed: bool) {
    crate::game::meteor_client_migration::on_control_mouse(game, pressed);
}
