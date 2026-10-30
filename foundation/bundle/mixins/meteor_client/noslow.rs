// No Slow: the item-use slowdown left out only for this native client thread.
fn solstice_noslow_hook(
    input: [f32; 2],
    item_use: Option<f32>,
    sneaking: Option<f32>,
) -> std::ops::ControlFlow<[f32; 2]> {
    if item_use.is_some() && solstice_flags::enabled() {
        return std::ops::ControlFlow::Break(modify_input(input, None, sneaking));
    }
    std::ops::ControlFlow::Continue(())
}
