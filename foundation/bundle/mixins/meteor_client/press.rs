// Meteor Horizon native support; license/source attribution: data/licenses/THIRD_PARTY_NOTICES.txt.
fn meteor_horizon_press(
    game: &mut crate::game::Game<'_>,
    button: winit::event::MouseButton,
) -> std::ops::ControlFlow<bool> {
    if crate::game::meteor_horizon::client_on(game) && crate::game::meteor_horizon::gui_blocks_input() {
        if crate::game::meteor_horizon::gui_open() {
            crate::game::meteor_horizon::gui_press(game, button);
        }
        return std::ops::ControlFlow::Break(true);
    }
    std::ops::ControlFlow::Continue(())
}
