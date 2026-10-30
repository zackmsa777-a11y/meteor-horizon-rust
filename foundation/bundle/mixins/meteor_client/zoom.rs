// Meteor Horizon native support; license/source attribution: data/licenses/THIRD_PARTY_NOTICES.txt.
fn meteor_horizon_zoom(game: &crate::game::Game<'_>, zoom: &mut Option<crate::game::native_hooks::NativeZoom>) {
    crate::game::meteor_horizon::zoom(game, zoom);
}
