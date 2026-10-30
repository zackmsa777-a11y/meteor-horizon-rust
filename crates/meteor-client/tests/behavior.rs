// SPDX-License-Identifier: GPL-3.0-only
// These cover the initial slice, not full Meteor parity. Results: validation.json.
use meteor_client::{
    Client, Config,
    modules::{
        auto_jump,
        zoom::{Zoom, ZoomSettings},
    },
    registry::*,
};

fn close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-9, "{actual} != {expected}");
}

#[test]
fn zoom_eases_to_target_and_unsubscribes_after_fade_out() {
    let settings = ZoomSettings::default();
    let mut zoom = Zoom::default();
    zoom.activate(&settings);
    zoom.frame(0.0998, true, &settings); // 0.001 + 5 * 0.0998 = 0.5
    close(zoom.scaling(), 3.5);
    zoom.frame(0.1, true, &settings);
    close(zoom.scaling(), 6.0);
    close(zoom.sensitivity_multiplier(), 1.0 / 3.0);
    zoom.frame(0.2, false, &settings);
    close(zoom.scaling(), 1.0);
    assert!(zoom.subscribed);
    zoom.tick();
    assert!(!zoom.subscribed);
}

#[test]
fn scroll_is_multiplicative_and_does_not_apply_in_menus() {
    let settings = ZoomSettings {
        smooth: false,
        ..ZoomSettings::default()
    };
    let mut zoom = Zoom::default();
    zoom.activate(&settings);
    zoom.frame(0.0, true, &settings);
    assert!(zoom.scroll(1.0, true, false, &settings));
    close(zoom.scaling(), 7.5);
    assert!(!zoom.scroll(1.0, true, true, &settings));
    close(zoom.scaling(), 7.5);
    assert!(zoom.scroll(-100.0, true, false, &settings));
    close(zoom.scaling(), 1.0);
}

#[test]
fn reactivation_during_fade_preserves_scrolled_zoom() {
    let settings = ZoomSettings::default();
    let mut zoom = Zoom::default();
    zoom.activate(&settings);
    zoom.scroll(1.0, true, false, &settings);
    zoom.frame(0.2, true, &settings);
    zoom.frame(0.05, false, &settings);
    let before = zoom.scaling();
    zoom.activate(&settings);
    close(zoom.scaling(), before);
    zoom.frame(0.2, true, &settings);
    close(zoom.scaling(), 7.5);
}

fn key(modifiers: u8) -> KeyEvent {
    KeyEvent {
        key: "key.keyboard.g".into(),
        modifiers,
        pressed: true,
        repeat: false,
        screen_open: false,
        debug_key_down: false,
    }
}

#[test]
fn modifier_binds_take_precedence_but_plain_binds_allow_sneaking() {
    let mut plain = Module::new("plain", "Plain", "Test");
    plain.keybind.key = Some("key.keyboard.g".into());
    let mut ctrl = Module::new("ctrl", "Control", "Test");
    ctrl.keybind = Keybind {
        key: Some("key.keyboard.g".into()),
        modifiers: 2,
    };
    let registry = Registry::new(vec![plain, ctrl]);
    assert_eq!(registry.matching(&key(2)), vec!["ctrl"]);
    assert_eq!(registry.matching(&key(1)), vec!["plain"]);
    let mut repeat = key(0);
    repeat.repeat = true;
    assert!(registry.matching(&repeat).is_empty());
    let mut debug = key(0);
    debug.debug_key_down = true;
    assert!(registry.matching(&debug).is_empty());
}

#[test]
fn held_bind_deactivates_on_release_and_screen_open() {
    let mut client = Client::default();
    let zoom = client.config.modules.get_mut("zoom").unwrap();
    zoom.keybind.key = Some("key.keyboard.g".into());
    zoom.toggle_on_bind_release = true;
    client.world_changed(true);
    client.key(&key(0));
    assert!(client.active("zoom"));
    let mut release = key(0);
    release.pressed = false;
    client.key(&release);
    assert!(!client.active("zoom"));
    client.key(&key(0));
    assert_eq!(client.screen_opened(), vec!["zoom"]);
    assert!(!client.active("zoom"));
}

#[test]
fn config_rejects_unsupported_schema_and_preserves_module_options() {
    let mut config = Config::default();
    let zoom = config.modules.get_mut("zoom").unwrap();
    zoom.favorite = true;
    zoom.active = true;
    zoom.keybind.key = Some("key.keyboard.c".into());
    let restored = Config::from_json(&config.to_json().unwrap()).unwrap();
    assert!(restored.modules.get("zoom").unwrap().favorite);
    assert!(restored.modules.get("zoom").unwrap().active);
    config.schema = 2;
    assert!(Config::from_json(&config.to_json().unwrap()).is_err());
    assert!(Config::from_json("broken json").is_err());
}

#[test]
fn leaving_world_clears_runtime_but_retains_enabled_configuration() {
    let mut client = Client::default();
    client.world_changed(true);
    client.toggle("zoom").unwrap();
    assert!(client.zoom.subscribed);
    client.world_changed(false);
    assert!(!client.zoom.subscribed);
    assert!(client.active("zoom"));
    client.world_changed(true);
    assert!(client.zoom.subscribed);
}

#[test]
fn auto_jump_respects_ground_sneak_and_movement_conditions() {
    use auto_jump::*;
    let mut player = Player {
        on_ground: true,
        sneaking: false,
        sprinting: true,
        forward: 0.0,
        sideways: 0.0,
    };
    let mut settings = Settings {
        jump_if: JumpWhen::Sprinting,
        ..Settings::default()
    };
    assert_eq!(tick(&player, &settings), None);
    player.forward = 1.0;
    assert_eq!(tick(&player, &settings), Some(Action::JumpFromGround));
    settings.mode = Mode::LowHop;
    assert_eq!(
        tick(&player, &settings),
        Some(Action::SetVerticalVelocity(0.25))
    );
    player.sneaking = true;
    assert_eq!(tick(&player, &settings), None);
    player.sneaking = false;
    player.on_ground = false;
    assert_eq!(tick(&player, &settings), None);
}

#[test]
fn older_configuration_gains_new_modules_without_losing_user_state() {
    let mut config = Config::default();
    config.modules.modules.retain(|m| m.id == "zoom");
    config.modules.get_mut("zoom").unwrap().favorite = true;
    config.modules.get_mut("zoom").unwrap().keybind.key = Some("key.keyboard.c".into());
    let loaded = Config::from_json(&config.to_json().unwrap()).unwrap();
    assert!(loaded.modules.get("zoom").unwrap().favorite);
    assert_eq!(
        loaded.modules.get("zoom").unwrap().keybind.key.as_deref(),
        Some("key.keyboard.c")
    );
    assert!(!loaded.modules.get("auto-jump").unwrap().active);
    assert!(!loaded.modules.get("air-jump").unwrap().active);
}

#[test]
fn air_jump_tracks_activation_presses_and_integer_levels() {
    use meteor_client::modules::air_jump::*;
    let mut air = AirJump::default();
    let mut player = Player {
        block_y: -2,
        on_ground: false,
        freecam: false,
    };
    let settings = Settings {
        maintain_level: true,
    };
    air.sync(true, Some(-2));
    air.sync(true, Some(10));
    assert_eq!(air.level, Some(-2)); // activation, not every frame
    assert!(!air.tick(player, &Settings::default(), true));
    assert!(air.tick(player, &settings, true));
    assert!(!air.tick(player, &settings, false));
    assert!(!air.key(player, false, false, true, false));
    assert!(!air.key(player, true, true, true, false));
    assert!(!air.key(player, true, false, false, true));
    assert_eq!(air.level, Some(-3));
    assert!(!air.tick(player, &settings, true));
    player.block_y = -3;
    assert!(air.tick(player, &settings, true));
    player.block_y = 5;
    assert!(air.key(player, true, false, true, true)); // jump has priority
    assert_eq!(air.level, Some(5));
    player.freecam = true;
    assert!(!air.key(player, true, false, true, false));
    assert!(!air.tick(player, &settings, true));
    player.freecam = false;
    player.on_ground = true;
    assert!(!air.key(player, true, false, true, false));
    assert!(!air.tick(player, &settings, true));
    air.sync(false, Some(5));
    assert_eq!(air.level, None);
    air.sync(true, Some(8));
    assert_eq!(air.level, Some(8));
    air.sync(true, None);
    assert_eq!(air.level, None);
}

#[test]
fn air_jump_settings_round_trip_and_reject_invalid_edits() {
    let mut config = Config::default();
    config.set("air-jump", "maintain-level", "true").unwrap();
    assert!(config.set("air-jump", "maintain-level", "yes").is_err());
    assert!(config.set("air-jump", "unknown", "false").is_err());
    let loaded = Config::from_json(&config.to_json().unwrap()).unwrap();
    assert!(loaded.air_jump.maintain_level);
    assert!(!loaded.modules.get("air-jump").unwrap().active);
}

#[test]
fn sprint_thresholds_water_and_source_conditional_are_preserved() {
    use meteor_client::modules::sprint::*;
    let settings = Settings::default();
    let mut player = Player {
        forward: 1.0,
        enough_food: true,
        ..Player::default()
    };
    assert_eq!(tick(false, &player, &settings), None);
    assert_eq!(tick(true, &player, &settings), Some(true));
    player.forward = 0.79;
    assert!(!should_sprint(true, &player, &settings));
    player.under_water = true;
    assert!(should_sprint(true, &player, &settings));
    player.in_water = true;
    assert_eq!(tick(true, &player, &settings), None); // returns, does not force false
    player.in_water = false;
    player.horizontal_collision = true;
    assert!(!should_sprint(true, &player, &settings));
    player.minor_horizontal_collision = true;
    assert!(should_sprint(true, &player, &settings));
    player.passenger = true;
    assert!(!should_sprint(true, &player, &settings));
    player.vehicle_can_sprint = true;
    player.vehicle_local = true;
    player.enough_food = false;
    assert!(should_sprint(true, &player, &settings));
    player.shallow_water = true;
    assert!(!should_sprint(true, &player, &settings));
    player.enough_food = true;
    assert!(should_sprint(true, &player, &settings)); // source ternary fallback
}

#[test]
fn sprint_rage_stationary_and_screen_rules() {
    use meteor_client::modules::sprint::*;
    let mut settings = Settings {
        mode: Mode::Rage,
        ..Settings::default()
    };
    let mut player = Player {
        sideways: -1.0,
        ..Player::default()
    };
    assert!(should_sprint(true, &player, &settings));
    player.sideways = 0.0;
    assert!(!should_sprint(true, &player, &settings));
    settings.sprint_while_stationary = true;
    assert!(should_sprint(true, &player, &settings));
    player.screen_open = true;
    assert!(!should_sprint(true, &player, &settings));
    player.gui_move_sprint = true;
    assert!(should_sprint(true, &player, &settings));
    assert!(!should_sprint(false, &player, &settings));
}

#[test]
fn sprint_attack_decisions_and_directional_jump_yaw() {
    use meteor_client::modules::sprint::*;
    let mut settings = Settings::default();
    let player = Player {
        forward: 1.0,
        enough_food: true,
        ..Player::default()
    };
    assert!(!before_attack(true, &settings));
    assert!(stop_after_knockback(true, &settings));
    settings.unsprint_on_hit = true;
    assert!(before_attack(true, &settings));
    assert!(!after_attack(true, false, &player, &settings));
    settings.keep_sprint = true;
    assert!(after_attack(true, false, &player, &settings));
    assert!(!after_attack(true, true, &player, &settings));
    assert!(!after_attack(false, false, &player, &settings));
    assert!(!stop_after_knockback(true, &settings));
    assert!(stop_after_knockback(false, &settings));
    assert_eq!(jump_yaw(30.0, 0.0, 0.0), 30.0);
    assert_eq!(jump_yaw(30.0, 1.0, 1.0), -15.0);
    assert_eq!(jump_yaw(30.0, -1.0, 1.0), -105.0);
    assert_eq!(jump_yaw(30.0, 0.0, -1.0), 120.0);
}

#[test]
fn sprint_settings_persist_and_reject_invalid_edits() {
    let mut config = Config::default();
    config.set("sprint", "sprint-mode", "Rage").unwrap();
    config.set("sprint", "keep-sprint", "true").unwrap();
    assert!(config.set("sprint", "sprint-mode", "invalid").is_err());
    assert!(config.set("sprint", "keep-sprint", "invalid").is_err());
    let config = Config::from_json(&config.to_json().unwrap()).unwrap();
    assert_eq!(
        config.sprint.mode,
        meteor_client::modules::sprint::Mode::Rage
    );
    assert!(config.sprint.keep_sprint);
    assert!(!config.modules.get("sprint").unwrap().active);
}

#[test]
fn gui_move_screen_filters_and_typing_exclusions() {
    use meteor_client::modules::gui_move::*;
    let mut settings = Settings::default();
    let mut context = Context {
        screen: Screen::Container,
        can_use_keys: true,
        ..Context::default()
    };
    assert!(!skip(context, &settings));
    context.creative_search = true;
    assert!(skip(context, &settings));
    context.creative_search = false;
    context.text_editor = true;
    assert!(skip(context, &settings));
    context.text_editor = false;
    context.screen = Screen::Widget;
    assert!(skip(context, &settings));
    settings.guis = Screens::Both;
    assert!(!skip(context, &settings));
    context.can_use_keys = false;
    assert!(skip(context, &settings));
    context.can_use_keys = true;
    for screen in [Screen::None, Screen::Other] {
        context.screen = screen;
        assert!(skip(context, &settings));
    }
}

#[test]
fn gui_move_releases_keys_when_skipped_disabled_or_setting_changes() {
    use meteor_client::modules::gui_move::*;
    let mut gui = GuiMove::default();
    let mut settings = Settings::default();
    let context = Context {
        screen: Screen::Container,
        can_use_keys: true,
        ..Context::default()
    };
    gui.sync(true, context, &settings);
    gui.input(Action::Forward, true, true, &settings);
    assert!(!gui.down(Action::Forward));
    gui.input(Action::Forward, true, false, &settings);
    gui.input(Action::Jump, true, false, &settings);
    assert!(gui.down(Action::Forward));
    assert!(gui.down(Action::Jump));
    gui.release_all();
    assert!(!gui.down(Action::Forward));
    assert!(!gui.down(Action::Jump));
    gui.sync(true, context, &settings);
    assert!(!gui.down(Action::Forward)); // focus regain does not invent a press
    gui.input(Action::Forward, true, false, &settings);
    gui.input(Action::Jump, true, false, &settings);
    settings.jump = false;
    gui.sync(true, context, &settings);
    assert!(!gui.down(Action::Jump));
    assert!(gui.down(Action::Forward));
    gui.sync(
        true,
        Context {
            text_editor: true,
            ..context
        },
        &settings,
    );
    assert!(!gui.down(Action::Forward));
    gui.sync(true, context, &settings);
    assert!(!gui.down(Action::Forward)); // does not synthesize a new press
    gui.input(Action::Left, true, false, &settings);
    gui.sync(false, context, &settings);
    assert!(!gui.down(Action::Left));
}

#[test]
fn gui_move_rotation_is_frame_scaled_capped_and_validated() {
    use meteor_client::modules::gui_move::*;
    let settings = Settings::default();
    assert_eq!(
        rotate(0.0, 0.0, 0.05, &settings, [false, true, false, true]),
        (4.0, 4.0)
    );
    assert_eq!(
        rotate(0.0, 85.0, 10.0, &settings, [true, false, false, true]),
        (-100.0, 90.0)
    );
    assert_eq!(
        rotate(0.0, 0.0, 0.05, &settings, [true, true, true, true]),
        (0.0, 0.0)
    );
    let mut config = Config::default();
    assert!(config.set("gui-move", "rotate-speed", "NaN").is_err());
    assert!(config.set("gui-move", "rotate-speed", "-1").is_err());
    config.set("gui-move", "guis", "Both").unwrap();
    config.set("gui-move", "rotate-speed", "8").unwrap();
    let loaded = Config::from_json(&config.to_json().unwrap()).unwrap();
    assert_eq!(loaded.gui_move.guis, Screens::Both);
    assert_eq!(loaded.gui_move.rotate_speed, 8.0);
}

#[test]
fn high_jump_scales_block_factor_before_boost_and_validates_settings() {
    use meteor_client::modules::high_jump::Settings;
    assert_eq!(Settings::default().block_jump_factor(0.5), 0.5);
    let settings = Settings {
        jump_multiplier: 2.0,
    };
    let strength = 0.42_f32;
    let boost = 0.1_f32;
    let power = strength * settings.block_jump_factor(0.5) + boost;
    assert_eq!(power, 0.52_f32);
    assert_eq!(
        Settings {
            jump_multiplier: 0.0
        }
        .block_jump_factor(0.5),
        0.0
    );
    let mut config = Config::default();
    config.set("high-jump", "jump-multiplier", "2").unwrap();
    assert!(config.set("high-jump", "jump-multiplier", "-1").is_err());
    assert!(config.set("high-jump", "jump-multiplier", "NaN").is_err());
    let loaded = Config::from_json(&config.to_json().unwrap()).unwrap();
    assert_eq!(loaded.high_jump.jump_multiplier, 2.0);
    assert!(!loaded.modules.get("high-jump").unwrap().active);
}

#[test]
fn registry_restores_canonical_order_and_rejects_duplicates() {
    let mut config = Config::default();
    config.modules.modules.reverse();
    config.modules.get_mut("zoom").unwrap().title = "stale".into();
    let loaded = Config::from_json(&config.to_json().unwrap()).unwrap();
    assert_eq!(loaded.modules.modules[0].id, "zoom");
    assert_eq!(loaded.modules.get("zoom").unwrap().title, "Zoom");
    config
        .modules
        .modules
        .push(config.modules.modules[0].clone());
    assert!(Config::from_json(&config.to_json().unwrap()).is_err());
}

#[test]
fn settings_edits_validate_before_commit_and_drive_auto_jump() {
    use auto_jump::*;
    let mut client = Client::default();
    client.config.set("auto-jump", "mode", "LowHop").unwrap();
    client
        .config
        .set("auto-jump", "velocity-height", "0.75")
        .unwrap();
    assert!(
        client
            .config
            .set("auto-jump", "velocity-height", "-1")
            .is_err()
    );
    assert!(
        client
            .config
            .set("auto-jump", "velocity-height", "NaN")
            .is_err()
    );
    close(client.config.auto_jump.velocity_height, 0.75);
    let player = Player {
        on_ground: true,
        sneaking: false,
        sprinting: false,
        forward: 0.0,
        sideways: 0.0,
    };
    assert_eq!(client.auto_jump(&player), None);
    client.toggle("auto-jump").unwrap();
    assert_eq!(
        client.auto_jump(&player),
        Some(Action::SetVerticalVelocity(0.75))
    );
    client
        .config
        .set("auto-jump", "jump-if", "Walking")
        .unwrap();
    assert_eq!(client.auto_jump(&player), None);
    assert!(
        client
            .config
            .set("zoom", "scroll-sensitivity", "-2")
            .is_err()
    );
    close(client.config.zoom.scroll_sensitivity, 1.0);
}
