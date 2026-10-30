// Meteor Horizon native support; license/source attribution: data/licenses/THIRD_PARTY_NOTICES.txt.
fn meteor_horizon_xray_get(value: u32) -> u32 {
    crate::mesher::meteor_horizon_xray::filter(value)
}
