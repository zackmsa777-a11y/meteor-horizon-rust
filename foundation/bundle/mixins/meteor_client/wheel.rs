// The wheel scrolls a click GUI panel, and stays off the hotbar while it is open.
fn solstice_wheel(game: &crate::game::Game<'_>, steps: f32, claimed: &mut bool) {
    if crate::game::solstice::client_on(game) && crate::game::solstice::gui_blocks_input() {
        crate::game::solstice::gui_wheel(game, steps);
        *claimed = true;
    }
}
