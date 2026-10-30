// SPDX-License-Identifier: GPL-3.0-only
fn meteor_settings_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    crate::game::meteor_client_migration::on_settings_click(game, mod_id, button);
}
