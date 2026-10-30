// Solstice Timer: scale only this client thread's elapsed time.
fn solstice_timer_hook(
    this: &mut FixedStep,
    elapsed: std::time::Duration,
) -> std::ops::ControlFlow<u32> {
    let multiplier = solstice_timer::multiplier();
    if multiplier == 1. {
        return std::ops::ControlFlow::Continue(());
    }
    let Some(_guard) = solstice_timer::enter() else {
        return std::ops::ControlFlow::Continue(());
    };
    let ticks = this.advance(elapsed.mul_f32(multiplier));
    std::ops::ControlFlow::Break(ticks)
}
