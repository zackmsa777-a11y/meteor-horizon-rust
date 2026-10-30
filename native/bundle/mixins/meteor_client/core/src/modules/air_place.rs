// SPDX-License-Identifier: GPL-3.0-only
// AirPlace.java native policy. Copyright (c) Meteor Development.
use crate::colors::SettingColor;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShapeMode {
    Lines,
    Sides,
    #[default]
    Both,
}
impl ShapeMode {
    pub fn lines(self) -> bool {
        self != Self::Sides
    }
    pub fn sides(self) -> bool {
        self != Self::Lines
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub render: bool,
    pub shape_mode: ShapeMode,
    pub side_color: SettingColor,
    pub line_color: SettingColor,
    pub custom_range: bool,
    pub range: f64,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            render: true,
            shape_mode: ShapeMode::Both,
            side_color: SettingColor::new(204, 0, 0, 10),
            line_color: SettingColor::new(204, 0, 0, 255),
            custom_range: false,
            range: 5.0,
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if !self.range.is_finite() || self.range < 0.0 {
            Err("AirPlace range must be finite and nonnegative".into())
        } else {
            Ok(())
        }
    }
    pub fn pick_range(&self, block_range: f64) -> f64 {
        if self.custom_range {
            self.range
        } else {
            block_range
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemKind {
    Block,
    SpawnEgg,
    FireworkRocket,
    ArmorStand,
    Other,
}
impl ItemKind {
    pub fn placeable(self, fall_flying: bool) -> bool {
        match self {
            Self::Block | Self::SpawnEgg | Self::ArmorStand => true,
            Self::FireworkRocket => !fall_flying,
            Self::Other => false,
        }
    }
    pub fn check_entities(self) -> bool {
        matches!(self, Self::Block | Self::ArmorStand)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hit {
    pub pos: [i32; 3],
}
#[derive(Clone, Debug, Default)]
pub struct State {
    pub hit: Option<Hit>,
}
impl State {
    /// Source intentionally keeps the prior hit on gated ticks (and toggle cycles).
    pub fn pre_tick(
        &mut self,
        placeable_in_hands: bool,
        current_hit_is_miss: bool,
        ray_hit: impl FnOnce() -> Option<Hit>,
    ) {
        if placeable_in_hands && current_hit_is_miss {
            self.hit = ray_hit();
        }
    }
    pub fn render_hit(
        &self,
        settings: &Settings,
        current_hit_is_miss: bool,
        replaceable: bool,
        placeable_in_hands: bool,
    ) -> Option<Hit> {
        self.hit
            .filter(|_| settings.render && current_hit_is_miss && replaceable && placeable_in_hands)
    }
    pub fn interaction_hit(
        &self,
        item: ItemKind,
        fall_flying: bool,
        can_place: impl FnOnce(Hit, bool) -> bool,
    ) -> Option<Hit> {
        let hit = self.hit?;
        (item.placeable(fall_flying) && can_place(hit, item.check_entities())).then_some(hit)
    }
}
