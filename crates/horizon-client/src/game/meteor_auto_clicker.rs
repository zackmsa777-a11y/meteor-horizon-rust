// SPDX-License-Identifier: GPL-3.0-only
// AutoClicker + Utils click helpers + MinecraftMixin's deferred right click.
// Copyright (c) Meteor Development.
use super::Game;

#[cfg(not(feature = "meteor"))]
impl Game<'_> {
    pub(super) fn meteor_auto_clicker_head(&mut self) {}
}

#[cfg(feature = "meteor")]
#[derive(Default)]
pub(super) struct State {
    core: meteor_client::modules::auto_clicker::State,
    generation: Option<u64>,
    pending_right: bool,
}

#[cfg(feature = "meteor")]
impl Game<'_> {
    pub(super) fn meteor_auto_clicker_sync(&mut self) {
        let generation = self
            .meteor_interaction_active("auto-clicker")
            .then(|| {
                self.meteor
                    .client
                    .subscriptions()
                    .iter()
                    .find(|s| s.id == "auto-clicker")
                    .map(|s| s.generation)
            })
            .flatten();
        if generation != self.meteor.auto_clicker.generation {
            self.left = false;
            self.right = false;
            if generation.is_some() {
                self.meteor.auto_clicker.core = Default::default();
            }
            self.meteor.auto_clicker.generation = generation;
        }
    }
    pub(super) fn meteor_auto_clicker_head(&mut self) {
        self.meteor_auto_clicker_sync();
        // This global pending click survives a module toggle, as MinecraftMixin's
        // rightClick flag does. It is consumed after Tick.Pre, before vanilla input.
        if std::mem::take(&mut self.meteor.auto_clicker.pending_right)
            && self.native_window()
            && self.native_on("meteor-client")
            && self.remote.is_some()
            && self.player.is_some()
        {
            self.remote_use();
        }
    }
    pub(super) fn meteor_auto_clicker_post(&mut self) {
        use meteor_client::modules::auto_clicker::Action;
        self.meteor_auto_clicker_sync();
        if self.meteor.auto_clicker.generation.is_none() || self.remote.is_none() {
            return;
        }
        let screen = self.screen != super::Screen::Playing || self.meteor_gui.open;
        let actions = self
            .meteor
            .auto_clicker
            .core
            .post(&self.meteor.client.config.auto_clicker, screen);
        match actions[0] {
            Action::None => {}
            Action::Hold => self.left = true,
            Action::Press => {
                if self.clicks.miss_time == 10000 {
                    self.clicks.miss_time = 0;
                }
                self.left = true;
                self.remote_start_attack();
                self.left = false;
            }
        }
        match actions[1] {
            Action::None => {}
            Action::Hold => self.right = true,
            Action::Press => self.meteor.auto_clicker.pending_right = true,
        }
    }
}
