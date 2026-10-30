// SPDX-License-Identifier: GPL-3.0-only
fn meteor_wheel(game: &mut crate::game::Game<'_>, steps: f32, claimed: &mut bool) {
    crate::game::meteor_client_migration::on_wheel(game, steps, claimed);
}
