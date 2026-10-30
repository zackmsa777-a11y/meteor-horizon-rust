// SPDX-License-Identifier: GPL-3.0-only
// Source-derived AutoEat.java policies. Copyright (c) Meteor Development.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThresholdMode {
    Health,
    Hunger,
    #[default]
    Any,
    Both,
}
impl ThresholdMode {
    pub fn test(self, health: bool, hunger: bool) -> bool {
        match self {
            Self::Health => health,
            Self::Hunger => hunger,
            Self::Any => health || hunger,
            Self::Both => health && hunger,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    Combined,
    Hunger,
    #[default]
    Saturation,
}
impl Priority {
    pub fn value(self, food: &Food) -> f32 {
        match self {
            Self::Combined => food.nutrition as f32 + food.saturation,
            Self::Hunger => food.nutrition as f32,
            Self::Saturation => food.saturation,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub blacklist: Vec<String>,
    pub pause_auras: bool,
    pub pause_baritone: bool,
    pub search_inventory: bool,
    pub food_priority: Priority,
    pub threshold_mode: ThresholdMode,
    pub health_threshold: f64,
    pub hunger_threshold: i32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            blacklist: [
                "enchanted_golden_apple",
                "golden_apple",
                "chorus_fruit",
                "poisonous_potato",
                "pufferfish",
                "chicken",
                "rotten_flesh",
                "spider_eye",
                "suspicious_stew",
            ]
            .into_iter()
            .map(|id| format!("minecraft:{id}"))
            .collect(),
            pause_auras: true,
            pause_baritone: true,
            search_inventory: false,
            food_priority: Priority::Saturation,
            threshold_mode: ThresholdMode::Any,
            health_threshold: 10.0,
            hunger_threshold: 16,
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if !self.health_threshold.is_finite() || !(1.0..=19.0).contains(&self.health_threshold) {
            return Err("AutoEat health threshold must be between 1 and 19".into());
        }
        if !(1..=19).contains(&self.hunger_threshold) {
            return Err("AutoEat hunger threshold must be between 1 and 19".into());
        }
        crate::modules::fast_use::Settings { items:self.blacklist.clone(), ..Default::default() }.validate()?;
        Ok(())
    }
    pub fn edible(&self, food: &Food, hunger: i32) -> bool {
        // Utils.isFood checks the ITEM prototype's FOOD and CONSUMABLE, then
        // AutoEat reads the stack's FOOD. The adapter preserves that distinction.
        food.prototype_is_food
            && !self.blacklist.contains(&food.item)
            && (hunger < 20 || food.can_always_eat)
    }
    pub fn find_slot(&self, hunger: i32, foods: &[Food]) -> Option<i32> {
        if foods
            .iter()
            .any(|food| food.slot == 40 && self.edible(food, hunger))
        {
            return Some(40);
        }
        let best = |start, end| {
            let mut selected = None;
            let mut best = -1.0_f32;
            // Source iterates inventory indices, not the caller's collection order.
            for slot in start..=end {
                if let Some(food) = foods.iter().find(|food| food.slot == slot)
                    && self.edible(food, hunger)
                {
                    let value = self.food_priority.value(food);
                    if value > best {
                        best = value;
                        selected = Some(slot);
                    }
                }
            }
            selected
        };
        best(0, 8).or_else(|| self.search_inventory.then(|| best(9, 35)).flatten())
    }
    pub fn should_eat(&self, health: f32, hunger: i32, foods: &[Food]) -> Option<i32> {
        if !self.threshold_mode.test(
            f64::from(health) <= self.health_threshold,
            hunger <= self.hunger_threshold,
        ) {
            return None;
        }
        self.find_slot(hunger, foods)
    }
}
#[derive(Clone, Debug)]
pub struct Food {
    pub slot: i32,
    pub item: String,
    pub prototype_is_food: bool,
    pub nutrition: i32,
    pub saturation: f32,
    pub can_always_eat: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotPlan {
    Offhand,
    Hotbar(i32),
    Move { from: i32, to: i32 },
}
pub fn slot_plan(slot: i32, empty_hotbar: Option<i32>) -> Option<SlotPlan> {
    match slot {
        40 => Some(SlotPlan::Offhand),
        0..=8 => Some(SlotPlan::Hotbar(slot)),
        9..=35 => empty_hotbar
            .filter(|slot| (0..=8).contains(slot))
            .map(|to| SlotPlan::Move { from: slot, to }),
        _ => None,
    }
}
