// Meteor Horizon native support; license/source attribution: data/licenses/THIRD_PARTY_NOTICES.txt.
fn meteor_horizon_xray_unpack(out: &mut [u32; 4096], value: ()) {
    crate::mesher::meteor_horizon_xray::filter_all(out);
    value
}
