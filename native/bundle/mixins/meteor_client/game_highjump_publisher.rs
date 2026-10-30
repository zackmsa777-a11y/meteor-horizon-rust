// SPDX-License-Identifier: GPL-3.0-only
// Standalone Game-side bridge for integration into the existing weak-window Runtime.
use super::Game;
use meteor_client::modules::high_jump::Settings;
use horizon_client::motion::meteor_highjump;

/// Call after dropping the Runtime borrow, immediately before each jump path,
/// as well as frame/Pre refresh, disable and player/world lifecycle transitions.
pub(super) fn publish(game: &mut Game<'_>, active: bool, settings: &Settings) -> Result<(),String> {
    // A server Game must not clear another window's policy if called on its thread.
    if !game.native_window() { return Ok(()); }
    meteor_highjump::clear();
    if !active || !game.native_on("meteor_client") || game.title.is_some() {
        return Ok(());
    }
    settings.validate()?;
    let Some(player)=game.player.as_mut() else {return Ok(());};
    let key=player.attributes.key("minecraft:jump_strength")
        .ok_or("Local player has no registered jump_strength attribute")?;
    let owner=match player.attributes.lookup(Some(&key)) {
        Some(handle)=>handle,
        None=>player.attributes.get_instance(Some(key))
            .map_err(|e|format!("HighJump identity realization failed: {e:?}"))?
            .ok_or("Local player jump_strength instance is unavailable")?,
    };
    // Realization never substitutes a raw pointer; Handle owns its Arc owner token.
    meteor_highjump::publish(owner,settings.clone())
}
