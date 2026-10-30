// SPDX-License-Identifier: GPL-3.0-only
fn meteor_frame(game: &mut crate::game::Game<'_>, elapsed: std::time::Duration) {
    crate::game::meteor_client_migration::on_frame(game, elapsed);
}
