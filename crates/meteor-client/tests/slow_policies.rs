// SPDX-License-Identifier: GPL-3.0-only
#[path = "../src/modules/no_slow.rs"]
mod no_slow;
#[path = "../src/modules/slippy.rs"]
mod slippy;
#[path = "../src/modules/sneak.rs"]
mod sneak;
#[test]
fn sneak_preserves_packet_vs_vanilla_and_flight_gate() {
    let mut s = sneak::Settings::default();
    assert!(s.vanilla(true, false));
    assert!(!s.packet(true, false));
    s.mode = sneak::Mode::Packet;
    assert!(s.packet(true, false));
    assert!(!s.vanilla(true, false));
    assert!(!s.packet(true, true));
    assert!(!s.packet(false, false));
}
#[test]
fn no_slow_input_does_not_confuse_crawling_or_packet_spoof_with_pose() {
    let mut s = no_slow::Settings::default();
    s.validate().unwrap();
    assert!(!s.packet_sneak(true));
    s.air_strict = true;
    assert!(s.packet_sneak(true));
    assert!(!s.packet_sneak(false));
    assert!(s.moving_slowly(true, false));
    s.sneaking = true;
    assert!(!s.moving_slowly(true, false));
    assert!(s.moving_slowly(false, true));
    assert!(!s.enough_food(6));
    s.hunger = true;
    assert!(s.enough_food(0));
}
#[test]
fn no_slow_speed_floor_does_not_replace_effects_or_boost_normal_speed() {
    let mut s = no_slow::Settings::default();
    s.slowness = true;
    assert_eq!(s.speed(0.05, 0.1, false), 0.1);
    assert_eq!(
        s.speed(0.05, 0.1, true),
        (f64::from(0.1f32) * 1.30000001192092896) as f32
    );
    assert_eq!(s.speed(0.11, 0.1, true), 0.11);
    assert!(s.neutral_speed_block("minecraft:honey_block"));
    assert!(!s.neutral_speed_block("minecraft:slime_block"));
    assert!(s.cancel_inside("minecraft:cobweb"));
    s.web = no_slow::WebMode::Timer;
    assert!(!s.cancel_inside("minecraft:cobweb"));
    assert!(s.cancel_inside("minecraft:sweet_berry_bush"));
}
#[test]
fn no_slow_web_timer_resets_once_and_mode_change_does_not_reset_override() {
    let mut s = no_slow::Settings::default();
    s.web = no_slow::WebMode::Timer;
    let mut state = no_slow::State::default();
    state.activate();
    assert_eq!(state.pre_tick(&s, false, false), Some(1.0));
    assert_eq!(state.pre_tick(&s, false, false), None);
    assert_eq!(state.pre_tick(&s, true, false), Some(10.0));
    s.web = no_slow::WebMode::None;
    assert_eq!(state.pre_tick(&s, false, true), None);
    s.web = no_slow::WebMode::Timer;
    assert_eq!(state.pre_tick(&s, true, true), Some(1.0));
    s.web_timer = f64::MAX;
    assert!(s.validate().is_err());
}
#[test]
fn slippy_lists_take_precedence_over_no_slow_slime_fallback() {
    let mut s = slippy::Settings::default();
    s.validate().unwrap();
    assert_eq!(
        slippy::friction(Some(&s), true, "minecraft:slime_block", 0.8),
        1.0
    );
    s.ignored_blocks.push("minecraft:slime_block".into());
    assert_eq!(
        slippy::friction(Some(&s), true, "minecraft:slime_block", 0.8),
        0.6
    );
    s.list_mode = slippy::ListMode::Whitelist;
    assert_eq!(
        slippy::friction(Some(&s), false, "minecraft:stone", 0.6),
        0.6
    );
    s.allowed_blocks.push("minecraft:stone".into());
    assert_eq!(
        slippy::friction(Some(&s), false, "minecraft:stone", 0.6),
        1.0
    );
    s.friction = f64::NAN;
    assert!(s.validate().is_err());
}

#[test]
fn slow_module_config_edits_and_block_lists_validate_atomically() {
    let mut c = meteor_client::Config::default();
    for id in ["sneak", "no-slow", "slippy"] {
        assert!(c.modules.get(id).is_some());
    }
    c.set("slippy", "allowed-blocks", "stone,minecraft:ice")
        .unwrap();
    c.set("slippy", "list-mode", "Whitelist").unwrap();
    assert_eq!(
        c.slippy.allowed_blocks,
        ["minecraft:stone", "minecraft:ice"]
    );
    assert!(c.set("slippy", "friction", "1.11").is_err());
    assert_eq!(c.slippy.friction, 1.0);
    assert!(c.set("no-slow", "web-timer", "1e300").is_err());
    assert_eq!(c.no_slow.web_timer, 10.0);
    c.set("sneak", "mode", "Packet").unwrap();
    let loaded = meteor_client::Config::from_json(&c.to_json().unwrap()).unwrap();
    assert!(loaded.sneak.packet(true, false));
}
