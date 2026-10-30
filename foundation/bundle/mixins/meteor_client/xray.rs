// Meteor Horizon native support; license/source attribution: data/licenses/THIRD_PARTY_NOTICES.txt.
fn meteor_horizon_xray_hook(this: &mut Mesher<'_>, region: &[[Option<&Chunk>; 3]; 3], sy: i32) -> std::ops::ControlFlow<SectionMesh> {
    use std::sync::atomic::Ordering::Relaxed;
    if !meteor_horizon_xray::ON.load(Relaxed) || meteor_horizon_xray::meshing() {
        return std::ops::ControlFlow::Continue(());
    }
    meteor_horizon_xray::set_meshing(true);
    let mesh = this.mesh(region, sy);
    meteor_horizon_xray::set_meshing(false);
    std::ops::ControlFlow::Break(mesh)
}
