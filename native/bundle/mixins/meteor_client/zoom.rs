// SPDX-License-Identifier: GPL-3.0-only
fn meteor_zoom(
    game: &crate::game::Game<'_>,
    zoom: &mut Option<crate::game::native_hooks::NativeZoom>,
) {
    crate::game::meteor_client_migration::on_zoom(game, zoom);
}
