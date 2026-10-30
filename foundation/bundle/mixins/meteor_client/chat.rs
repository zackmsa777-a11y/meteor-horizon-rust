// Meteor Horizon native support; license/source attribution: data/licenses/THIRD_PARTY_NOTICES.txt.
fn meteor_horizon_chat(game: &mut crate::game::Game<'_>, message: &str) -> std::ops::ControlFlow<()> {
    if crate::game::meteor_horizon::chat(game, message) {
        return std::ops::ControlFlow::Break(());
    }
    std::ops::ControlFlow::Continue(())
}
