// Meteor Horizon native support; license/source attribution: data/licenses/THIRD_PARTY_NOTICES.txt.
fn meteor_horizon_send(
    game: &crate::game::Game<'_>,
    packet: &horizon_protocol::packets::play::Serverbound,
) -> std::ops::ControlFlow<()> {
    use crate::game::meteor_horizon::Out;
    match crate::game::meteor_horizon::outgoing(game, packet) {
        Out::Pass => std::ops::ControlFlow::Continue(()),
        Out::Drop => std::ops::ControlFlow::Break(()),
        Out::Replace(changed) => {
            if let Some(r) = &game.remote {
                r.session.send(changed);
            }
            std::ops::ControlFlow::Break(())
        }
    }
}
