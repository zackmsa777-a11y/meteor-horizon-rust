// Meteor Horizon native support; license/source attribution: data/licenses/THIRD_PARTY_NOTICES.txt.
fn meteor_horizon_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    crate::game::meteor_horizon::listed(mods);
}
