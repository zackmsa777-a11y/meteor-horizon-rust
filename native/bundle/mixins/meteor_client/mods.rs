// SPDX-License-Identifier: GPL-3.0-only
fn meteor_mods(
    game: &mut crate::game::Game<'_>,
    mods: &mut Vec<crate::game::native_hooks::NativeMod>,
) {
    crate::game::meteor_client_migration::on_mods(game, mods);
}
