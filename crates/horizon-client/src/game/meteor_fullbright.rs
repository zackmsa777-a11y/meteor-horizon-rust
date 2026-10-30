// SPDX-License-Identifier: GPL-3.0-only
// Fullbright lifecycle/Potion adapter. Copyright (c) Meteor Development.
use super::Game;
use horizon_client::effects::EffectInstance;
use meteor_client::modules::fullbright::{self, Effect, NightVision, Settings};

impl Game<'_> {
    /// Sync render-only floors before calling lifecycle effects; invalidate in-flight meshes
    /// as well as loaded sections in the renderer/streaming integration.
    pub(super) fn meteor_fullbright_effects(&mut self, effects: &[Effect]) {
        for effect in effects {
            match effect {
                Effect::RebuildSections => self.all_sections_changed(),
                Effect::RemoveNightVision if self.player.is_some() => {
                    // Deliberately removes pre-existing legitimate night vision too: source
                    // does not remember ownership or restore the prior effect on disable.
                    self.potions
                        .effects
                        .list
                        .retain(|effect| effect.id != "night_vision");
                    self.apply_effect_status();
                }
                Effect::RemoveNightVision => {}
            }
        }
    }
    pub(super) fn meteor_fullbright_potion_post(&mut self, settings: &Settings) {
        let duration = self
            .potions
            .effects
            .get("night_vision")
            .map(|effect| effect.duration);
        match fullbright::post_tick(settings, self.player.is_some(), duration) {
            NightVision::None => {}
            NightVision::Add420 => {
                self.add_player_effect(EffectInstance::new("night_vision", 420, 0));
            }
            NightVision::SetDuration420 => {
                if let Some(effect) = self
                    .potions
                    .effects
                    .list
                    .iter_mut()
                    .find(|e| e.id == "night_vision")
                {
                    effect.duration = 420;
                }
            }
        }
    }
}
