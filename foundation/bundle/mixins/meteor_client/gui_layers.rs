// SPDX-License-Identifier: MIT
// Meteor Horizon GUI adapter. Upstream implementation: PointOfPressure/Solstice.
// Full upstream notice is retained in data/licenses/Solstice-MIT.txt.
fn meteor_gui_layers(
    game: &mut crate::game::Game<'_>,
    size: [u32; 2],
) -> ::core::ops::ControlFlow<HudBatch> {
    match crate::game::solstice::on_gui_render(game, size) {
        Some(batch) => std::ops::ControlFlow::Break(batch),
        None => std::ops::ControlFlow::Continue(()),
    }
}
