// SPDX-License-Identifier: GPL-3.0-only
fn meteor_tick(game: &mut crate::game::Game<'_>) {
    crate::game::meteor_client_migration::on_tick(game);
}
