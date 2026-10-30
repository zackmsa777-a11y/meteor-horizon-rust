// SPDX-License-Identifier: GPL-3.0-only
use meteor_client::modules::{auto_respawn, break_delay, fast_use};

#[test]
fn item_list_edits_normalize_identifiers_and_reject_invalid_candidates_atomically() {
    let mut config = meteor_client::Config::default();
    config
        .set("fast-use", "items", "stone,minecraft:experience_bottle")
        .unwrap();
    assert_eq!(
        config.fast_use.items,
        ["minecraft:stone", "minecraft:experience_bottle"]
    );
    config
        .set("fast-use", "items", r#"["dirt","minecraft:oak_planks"]"#)
        .unwrap();
    let before = config.to_json().unwrap();
    assert!(config.set("fast-use", "items", r#"["stone",3]"#).is_err());
    assert!(config.set("fast-use", "items", "minecraft:BadID").is_err());
    assert!(config.set("break-delay", "cooldown", "-1").is_err());
    assert_eq!(config.to_json().unwrap(), before);
    config.set("fast-use", "items", "[]").unwrap();
    assert!(config.fast_use.items.is_empty());
}

#[test]
fn auto_respawn_is_death_screen_event_with_ordered_waypoint_respawn_cancel() {
    use auto_respawn::*;
    let _settings = Settings::default();
    assert!(open_screen(false).is_empty());
    assert_eq!(
        open_screen(true),
        vec![
            Effect::AddDeathWaypoint,
            Effect::Respawn,
            Effect::CancelScreen
        ]
    );
}
#[test]
fn break_delay_press_is_single_use_and_survives_setting_change() {
    use break_delay::*;
    let mut settings = Settings {
        cooldown: 2,
        no_insta_break: true,
    };
    settings.validate().unwrap();
    let mut state = State::default();
    state.mouse_click(&settings, false);
    assert_eq!(state.cooldown(&settings), 2);
    state.mouse_click(&settings, true);
    state.mouse_click(&settings, true);
    settings.no_insta_break = false;
    assert_eq!(state.cooldown(&settings), 5);
    assert_eq!(state.cooldown(&settings), 2);
    settings.cooldown = 50;
    settings.validate().unwrap(); // sliderMax5 is not a hard limit
    settings.cooldown = -1;
    assert!(settings.validate().is_err());
}
#[test]
fn no_insta_break_only_intercepts_continue_progress_at_or_above_one() {
    use break_delay::*;
    let settings = Settings {
        cooldown: 0,
        no_insta_break: true,
    };
    let mut state = State::default();
    state.mouse_click(&settings, true);
    assert_eq!(state.continue_progress(&settings, 0.99), (0.99, None));
    assert_eq!(state.continue_progress(&settings, 1.0), (0.0, Some(5)));
    assert_eq!(state.continue_progress(&settings, 5.0), (0.0, Some(0)));
    assert_eq!(
        state.continue_progress(&Settings::default(), 1.5),
        (1.5, None)
    );
}
#[test]
fn fast_use_all_some_blocks_and_per_hand_fallback_preserve_source() {
    use fast_use::*;
    let mut settings = Settings::default();
    settings.validate().unwrap();
    assert_eq!(cooldown(&settings, "minecraft:air", false), 0);
    settings.mode = Mode::Some;
    settings.items = vec!["minecraft:experience_bottle".into()];
    settings.cooldown = 1;
    assert_eq!(cooldown(&settings, "minecraft:experience_bottle", false), 1);
    assert_eq!(cooldown(&settings, "minecraft:air", false), 4); // offhand can overwrite earlier1
    assert_eq!(cooldown(&settings, "minecraft:stone", true), 4);
    settings.blocks = true;
    assert_eq!(cooldown(&settings, "minecraft:stone", true), 1);
    settings.cooldown = 20;
    settings.validate().unwrap(); // sliderMax4 is not a hard limit
    settings.items.push("INVALID".into());
    assert!(settings.validate().is_err());
}
