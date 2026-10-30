// Meteor Horizon native support; license/source attribution: data/licenses/THIRD_PARTY_NOTICES.txt.
fn meteor_horizon_timer_hook(
    this: &mut FixedStep,
    elapsed: std::time::Duration,
) -> std::ops::ControlFlow<u32> {
    let multiplier = meteor_horizon_timer::multiplier();
    if multiplier == 1. {
        return std::ops::ControlFlow::Continue(());
    }
    let Some(_guard) = meteor_horizon_timer::enter() else {
        return std::ops::ControlFlow::Continue(());
    };
    let ticks = this.advance(elapsed.mul_f32(multiplier));
    std::ops::ControlFlow::Break(ticks)
}
