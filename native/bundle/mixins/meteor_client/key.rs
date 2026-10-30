// SPDX-License-Identifier: GPL-3.0-only
fn meteor_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    crate::game::meteor_client_migration::on_key(game, key, pressed);
}
