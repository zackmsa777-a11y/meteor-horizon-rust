// Meteor Horizon native support; license/source attribution: data/licenses/THIRD_PARTY_NOTICES.txt.
fn meteor_horizon_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    crate::game::meteor_horizon::key(game, key, pressed);
}
