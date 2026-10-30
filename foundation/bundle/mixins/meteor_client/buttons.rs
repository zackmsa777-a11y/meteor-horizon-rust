// Meteor Horizon native support; license/source attribution: data/licenses/THIRD_PARTY_NOTICES.txt.
fn meteor_horizon_buttons(game: &mut crate::game::Game<'_>) -> std::ops::ControlFlow<Result<()>> {
    if crate::game::meteor_horizon::client_on(game) && crate::game::meteor_horizon::gui_blocks_input() {
        return std::ops::ControlFlow::Break(Ok(()));
    }
    std::ops::ControlFlow::Continue(())
}
