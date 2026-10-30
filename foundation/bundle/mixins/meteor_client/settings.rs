// Meteor Horizon native support; license/source attribution: data/licenses/THIRD_PARTY_NOTICES.txt.
fn meteor_horizon_settings(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    if mod_id == crate::game::meteor_horizon::ID {
        crate::game::meteor_horizon::buttons(game, buttons);
    }
}
