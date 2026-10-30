// Solstice GUI input ownership, adapted with actual native client/owner gating.
fn solstice_buttons(game: &mut crate::game::Game<'_>) -> std::ops::ControlFlow<Result<()>> {
    if crate::game::solstice::client_on(game) && crate::game::solstice::gui_blocks_input() {
        return std::ops::ControlFlow::Break(Ok(()));
    }
    std::ops::ControlFlow::Continue(())
}
