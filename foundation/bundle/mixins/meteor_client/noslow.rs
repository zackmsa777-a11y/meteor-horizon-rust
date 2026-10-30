// Meteor Horizon native support; license/source attribution: data/licenses/THIRD_PARTY_NOTICES.txt.
fn meteor_horizon_noslow_hook(
    input: [f32; 2],
    item_use: Option<f32>,
    sneaking: Option<f32>,
) -> std::ops::ControlFlow<[f32; 2]> {
    if item_use.is_some() && meteor_horizon_flags::enabled() {
        return std::ops::ControlFlow::Break(modify_input(input, None, sneaking));
    }
    std::ops::ControlFlow::Continue(())
}
