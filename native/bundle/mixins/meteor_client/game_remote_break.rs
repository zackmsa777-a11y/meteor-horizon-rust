// SPDX-License-Identifier: GPL-3.0-only
// BreakDelay access to the window client's block-breaking state. Copyright (c) Meteor Development.
use super::Game;

/// `MultiPlayerGameMode.destroyDelay`; zero without a server session.
pub(in crate::game) fn delay(game: &Game<'_>) -> u32 {
    game.remote.as_ref().map_or(0, |remote| remote.destroy_delay)
}

pub(in crate::game) fn set_delay(game: &mut Game<'_>, delay: u32) {
    if let Some(remote) = game.remote.as_mut() {
        remote.destroy_delay = delay;
    }
}

/// The survival block being broken and the progress accumulated on it.
pub(in crate::game) fn destroying(game: &Game<'_>) -> Option<([i32; 3], f32)> {
    let remote = game.remote.as_ref()?;
    remote.destroying.map(|(pos, _, done)| (pos, done))
}

pub(in crate::game) fn set_done(game: &mut Game<'_>, done: f32) {
    if let Some(destroying) = game.remote.as_mut().and_then(|remote| remote.destroying.as_mut()) {
        destroying.2 = done;
    }
}

/// `sameDestroyTarget`'s item half: the item the break began with is still held.
pub(in crate::game) fn same_item(game: &Game<'_>, held: &str) -> bool {
    game.remote.as_ref().is_some_and(|remote| remote.destroying_item == held)
}
