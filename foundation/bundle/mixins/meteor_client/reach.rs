// Reach: the block reach, plus Reach's extra.
fn solstice_reach(game: &crate::game::Game<'_>) -> std::ops::ControlFlow<f64> {
    if !crate::game::solstice::client_on(game) || game.player.is_none() || game.title.is_some() {
        return std::ops::ControlFlow::Continue(());
    }
    match crate::game::solstice_player::reach_extra() {
        Some(extra) => std::ops::ControlFlow::Break(if game.creative { 5.0 } else { 4.5 } + extra),
        None => std::ops::ControlFlow::Continue(()),
    }
}
