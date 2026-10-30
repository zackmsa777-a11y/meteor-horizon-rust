// SPDX-License-Identifier: GPL-3.0-only
fn meteor_settings(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    crate::game::meteor_client_migration::on_settings(game, mod_id, buttons);
}
