// Meteor Horizon native support; license/source attribution: data/licenses/THIRD_PARTY_NOTICES.txt.
fn meteor_horizon_camera(game: &mut crate::game::Game<'_>, partial: f32) -> std::ops::ControlFlow<Option<(glam::DVec3, f32, f32)>> {
    match crate::game::meteor_horizon_render::camera(game, partial) {
        Some(view) => std::ops::ControlFlow::Break(view),
        None => std::ops::ControlFlow::Continue(()),
    }
}
