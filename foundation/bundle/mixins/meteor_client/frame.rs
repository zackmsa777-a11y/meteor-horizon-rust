// Meteor Horizon native support; license/source attribution: data/licenses/THIRD_PARTY_NOTICES.txt.
fn meteor_horizon_frame(game: &mut crate::game::Game<'_>) {
    crate::game::meteor_horizon_render::frame(game);
}
