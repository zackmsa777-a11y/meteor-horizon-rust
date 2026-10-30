// Meteor Horizon native support; license/source attribution: data/licenses/THIRD_PARTY_NOTICES.txt.
fn meteor_horizon_reach(game: &crate::game::Game<'_>) -> std::ops::ControlFlow<f64> {
    if !crate::game::meteor_horizon::client_on(game) || game.player.is_none() || game.title.is_some() {
        return std::ops::ControlFlow::Continue(());
    }
    match crate::game::meteor_horizon_player::reach_extra() {
        Some(extra) => std::ops::ControlFlow::Break(if game.creative { 5.0 } else { 4.5 } + extra),
        None => std::ops::ControlFlow::Continue(()),
    }
}
