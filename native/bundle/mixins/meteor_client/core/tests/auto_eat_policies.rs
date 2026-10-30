// SPDX-License-Identifier: GPL-3.0-only
use meteor_client::modules::auto_eat::*;
use meteor_client::Config;
fn food(slot: i32, nutrition: i32, saturation: f32) -> Food {
    Food {
        slot,
        item: "minecraft:bread".into(),
        prototype_is_food: true,
        nutrition,
        saturation,
        can_always_eat: false,
    }
}
#[test]
fn thresholds_are_inclusive_and_all_four_modes_are_distinct() {
    let foods = [food(0, 5, 6.0)];
    let mut s = Settings::default();
    assert_eq!(s.should_eat(10.0, 17, &foods), Some(0));
    assert_eq!(s.should_eat(11.0, 16, &foods), Some(0));
    assert_eq!(s.should_eat(11.0, 17, &foods), None);
    s.threshold_mode = ThresholdMode::Both;
    assert_eq!(s.should_eat(10.0, 17, &foods), None);
    assert_eq!(s.should_eat(10.0, 16, &foods), Some(0));
    s.threshold_mode = ThresholdMode::Health;
    assert_eq!(s.should_eat(11.0, 1, &foods), None);
    s.threshold_mode = ThresholdMode::Hunger;
    assert_eq!(s.should_eat(1.0, 17, &foods), None);
}
#[test]
fn offhand_wins_over_better_food_and_main_inventory_is_fallback_only() {
    let mut s = Settings::default();
    let mut foods = vec![food(12, 20, 20.0), food(4, 2, 1.0), food(40, 1, 0.0)];
    assert_eq!(s.find_slot(10, &foods), Some(40));
    foods.pop();
    s.search_inventory = true;
    assert_eq!(s.find_slot(10, &foods), Some(4));
    foods.pop();
    assert_eq!(s.find_slot(10, &foods), Some(12));
    s.search_inventory = false;
    assert_eq!(s.find_slot(10, &foods), None);
}
#[test]
fn priorities_and_slot_order_ties_follow_source_float_scores() {
    let mut s = Settings::default();
    let foods = [food(6, 2, 9.0), food(3, 8, 3.0), food(1, 8, 3.0)];
    assert_eq!(s.find_slot(10, &foods), Some(6));
    s.food_priority = Priority::Hunger;
    assert_eq!(s.find_slot(10, &foods), Some(1));
    s.food_priority = Priority::Combined;
    assert_eq!(s.find_slot(10, &foods), Some(1));
}
#[test]
fn blacklist_prototype_food_and_full_hunger_are_separate_gates() {
    let s = Settings::default();
    let mut f = food(0, 20, 20.0);
    f.item = "minecraft:golden_apple".into();
    f.can_always_eat = true;
    assert!(!s.edible(&f, 1));
    f.item = "minecraft:bread".into();
    f.prototype_is_food = false;
    assert!(!s.edible(&f, 1));
    f.prototype_is_food = true;
    f.can_always_eat = false;
    assert!(!s.edible(&f, 20));
    f.can_always_eat = true;
    assert!(s.edible(&f, 20));
}
#[test]
fn inventory_moves_require_an_empty_hotbar_but_hotbar_and_offhand_do_not() {
    assert_eq!(slot_plan(40, None), Some(SlotPlan::Offhand));
    assert_eq!(slot_plan(7, None), Some(SlotPlan::Hotbar(7)));
    assert_eq!(slot_plan(12, None), None);
    assert_eq!(
        slot_plan(12, Some(2)),
        Some(SlotPlan::Move { from: 12, to: 2 })
    );
    assert_eq!(slot_plan(12, Some(40)), None);
}
#[test]
fn persisted_threshold_validation_rejects_nonfinite_and_out_of_range() {
    let mut s = Settings::default();
    assert!(s.validate().is_ok());
    s.health_threshold = f64::NAN;
    assert!(s.validate().is_err());
    s.health_threshold = 20.0;
    assert!(s.validate().is_err());
    s.health_threshold = 10.0;
    s.hunger_threshold = 0;
    assert!(s.validate().is_err());
}

#[test]
fn commands_are_atomic_and_source_blacklist_bypasses_food_filter_on_load() {
    let mut config=Config::default();
    config.set("auto-eat","blacklist","stone,bread").unwrap();
    config.set("auto-eat","food-priority","Combined").unwrap();
    assert_eq!(config.auto_eat.blacklist,["minecraft:stone","minecraft:bread"]);
    let before=config.to_json().unwrap();
    assert!(config.set("auto-eat","health-threshold","NaN").is_err());
    assert!(config.set("auto-eat","hunger-threshold","20").is_err());
    assert!(config.set("auto-eat","food-priority","Fake").is_err());
    assert_eq!(config.to_json().unwrap(),before);
    let restored=Config::from_json(&before).unwrap();
    assert_eq!(restored.auto_eat.food_priority,Priority::Combined);
    assert_eq!(restored.auto_eat.blacklist,config.auto_eat.blacklist);
}
