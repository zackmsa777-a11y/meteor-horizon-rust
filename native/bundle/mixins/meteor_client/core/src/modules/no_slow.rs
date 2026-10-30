// SPDX-License-Identifier: GPL-3.0-only
// NoSlow.java and its input/entity/block/player mixins. Copyright (c) Meteor Development.
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WebMode {
    #[default]
    Vanilla,
    Timer,
    None,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub items: bool,
    pub web: WebMode,
    pub web_timer: f64,
    pub honey_block: bool,
    pub soul_sand: bool,
    pub slime_block: bool,
    pub berry_bush: bool,
    pub air_strict: bool,
    pub fluid_drag: bool,
    pub sneaking: bool,
    pub hunger: bool,
    pub slowness: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            items: true,
            web: WebMode::Vanilla,
            web_timer: 10.0,
            honey_block: true,
            soul_sand: true,
            slime_block: true,
            berry_bush: true,
            air_strict: false,
            fluid_drag: false,
            sneaking: false,
            hunger: false,
            slowness: false,
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if !(self.web_timer as f32).is_finite() || self.web_timer < 1.0 {
            Err("web-timer must fit a finite float and be at least 1".into())
        } else {
            Ok(())
        }
    }
    /// Source has no airborne test despite the setting's description.
    pub fn packet_sneak(&self, using_item: bool) -> bool {
        self.air_strict && using_item
    }
    pub fn moving_slowly(&self, original: bool, visually_crawling: bool) -> bool {
        if self.sneaking {
            visually_crawling
        } else {
            original
        }
    }
    pub fn enough_food(&self, food: i32) -> bool {
        food as f32 > if self.hunger { -1.0 } else { 6.0 }
    }
    /// Player.getSpeed return override; it does not remove status effects or modifiers.
    pub fn speed(&self, original: f32, ability_walk_speed: f32, sprinting: bool) -> f32 {
        if self.slowness && original < ability_walk_speed {
            if sprinting {
                (f64::from(ability_walk_speed) * 1.30000001192092896) as f32
            } else {
                ability_walk_speed
            }
        } else {
            original
        }
    }
    pub fn neutral_speed_block(&self, id: &str) -> bool {
        (self.honey_block && id == "minecraft:honey_block")
            || (self.soul_sand && id == "minecraft:soul_sand")
    }
    pub fn cancel_inside(&self, id: &str) -> bool {
        (self.web == WebMode::Vanilla && id == "minecraft:cobweb")
            || (self.berry_bush && id == "minecraft:sweet_berry_bush")
    }
}
#[derive(Default, Debug)]
pub struct State {
    reset_timer: bool,
}
impl State {
    pub fn activate(&mut self) {
        self.reset_timer = false;
    }
    pub fn pre_tick(
        &mut self,
        settings: &Settings,
        at_cobweb: bool,
        on_ground: bool,
    ) -> Option<f64> {
        if settings.web != WebMode::Timer {
            return None;
        }
        if at_cobweb && !on_ground {
            self.reset_timer = false;
            Some(settings.web_timer)
        } else if !self.reset_timer {
            self.reset_timer = true;
            Some(1.0)
        } else {
            None
        }
    }
    // Source has no onDeactivate reset and changing web mode does not reset the override.
}
