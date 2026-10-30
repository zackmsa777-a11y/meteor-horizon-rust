// Meteor Horizon native support; license/source attribution: data/licenses/THIRD_PARTY_NOTICES.txt.
//
//
fn meteor_gui_layers(
    game: &mut crate::game::Game<'_>,
    size: [u32; 2],
) -> ::core::ops::ControlFlow<HudBatch> {
    match crate::game::meteor_horizon::on_gui_render(game, size) {
        Some(batch) => std::ops::ControlFlow::Break(batch),
        None => std::ops::ControlFlow::Continue(()),
    }
}
