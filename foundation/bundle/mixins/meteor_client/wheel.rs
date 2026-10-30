// Meteor Horizon native support; license/source attribution: data/licenses/THIRD_PARTY_NOTICES.txt.
fn meteor_horizon_wheel(game: &crate::game::Game<'_>, steps: f32, claimed: &mut bool) {
    if crate::game::meteor_horizon::client_on(game) && crate::game::meteor_horizon::gui_blocks_input() {
        crate::game::meteor_horizon::gui_wheel(game, steps);
        *claimed = true;
    }
}
