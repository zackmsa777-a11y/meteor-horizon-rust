// SPDX-License-Identifier: GPL-3.0-only
// BreakDelay physical mouse press. Copyright (c) Meteor Development.
fn meteor_break_window_mouse(game: &mut crate::game::Game<'_>, event: &winit::event::WindowEvent) {
    crate::game::meteor_client_migration::on_window_mouse(game, event);
}
