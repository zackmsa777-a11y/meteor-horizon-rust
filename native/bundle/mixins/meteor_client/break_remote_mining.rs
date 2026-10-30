// SPDX-License-Identifier: GPL-3.0-only
// BreakDelay block-breaking boundary of the window client. Copyright (c) Meteor Development.
fn meteor_break_remote_mining(game: &mut crate::game::Game<'_>, start: bool) {
    crate::game::meteor_client_migration::on_remote_mining(game, start);
}
