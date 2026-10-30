// Meteor Horizon native support; license/source attribution: data/licenses/THIRD_PARTY_NOTICES.txt.
fn meteor_horizon_turn(game: &mut crate::game::Game<'_>, xo: f64, yo: f64) -> std::ops::ControlFlow<()> {
    if crate::game::meteor_horizon_render::turn(game, xo, yo) {
        return std::ops::ControlFlow::Break(());
    }
    std::ops::ControlFlow::Continue(())
}
