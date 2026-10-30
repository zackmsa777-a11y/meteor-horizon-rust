// Meteor Horizon native support; license/source attribution: data/licenses/THIRD_PARTY_NOTICES.txt.
//
fn meteor_horizon_hud(
    game: &mut crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    corners: &mut horizon_client::extras_hud::Corners,
) {
    crate::game::meteor_horizon::hud(game, batch, corners);
}
