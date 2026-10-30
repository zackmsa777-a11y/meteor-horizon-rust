// SPDX-License-Identifier: GPL-3.0-only
fn meteor_command(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) {
    crate::game::meteor_client_migration::on_command(game, command, ran);
}
