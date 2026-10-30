// SPDX-License-Identifier: GPL-3.0-only
// Core decisions are tested independently of engine hooks, which need separate runtime checks.
use meteor_client::modules::{auto_walk, safe_walk};

#[test]
fn safe_walk_config_migrates_old_files_and_rejects_invalid_edits_atomically() {
    let mut config = meteor_client::Config::from_json("{}").unwrap();
    assert!(!config.modules.get("safe-walk").unwrap().active);
    let before = config.safe_walk.clone();
    assert!(
        config
            .set("safe-walk", "minimum-fall-distance", "0")
            .is_err()
    );
    assert!(config.set("safe-walk", "edge-distance", "NaN").is_err());
    assert_eq!(config.safe_walk, before);
    config
        .set("safe-walk", "minimum-fall-distance", "3")
        .unwrap();
    config.set("safe-walk", "edge-distance", "-0.2").unwrap();
    config.set("safe-walk", "sneak", "true").unwrap();
    let restored = meteor_client::Config::from_json(&config.to_json().unwrap()).unwrap();
    assert_eq!(restored.safe_walk, config.safe_walk);
}

#[test]
fn auto_walk_smart_requires_real_pathing_and_lifecycle_stops_it() {
    use auto_walk::*;
    let settings = Settings::default();
    assert_eq!(settings.mode, Mode::Smart);
    assert_eq!(
        activate(&settings, 37.5),
        vec![Effect::MovePathInDirection { yaw: 37.5 }]
    );
    assert_eq!(
        tick(&settings, 64.0, 65.0, false),
        TickAction::Disable(DisableReason::MissingPathManager)
    );
    assert_eq!(
        DisableReason::MissingPathManager.message(),
        Some("Smart mode requires Baritone")
    );
    assert_eq!(DisableReason::VerticalMovement.message(), None);
    assert_eq!(tick(&settings, 64.0, 65.0, true), TickAction::None);
    assert_eq!(deactivate(&settings), Effect::StopPath);
    let simple = Settings {
        mode: Mode::Simple,
        ..settings.clone()
    };
    assert_eq!(
        settings_changed(&settings, &simple, true, true, 10.0),
        vec![Effect::StopPath, Effect::ReleaseDirections]
    );
    assert_eq!(
        settings_changed(&simple, &settings, true, true, -90.0),
        vec![
            Effect::MovePathInDirection { yaw: -90.0 },
            Effect::ReleaseDirections
        ]
    );
    assert!(settings_changed(&settings, &simple, true, false, 0.0).is_empty());
    assert!(settings_changed(&settings, &simple, false, true, 0.0).is_empty());
    assert!(activate(&simple, 0.0).is_empty());
    assert_eq!(deactivate(&simple), Effect::ReleaseDirections);
}

#[test]
fn auto_walk_simple_directions_vertical_changes_and_input_gates() {
    use auto_walk::*;
    let mut settings = Settings {
        mode: Mode::Simple,
        ..Settings::default()
    };
    for (direction, mapping) in [
        (Direction::Forwards, "key.forward"),
        (Direction::Backwards, "key.back"),
        (Direction::Left, "key.left"),
        (Direction::Right, "key.right"),
    ] {
        settings.simple_direction = direction;
        assert_eq!(direction.mapping(), mapping);
        assert_eq!(
            tick(&settings, 64.0, 63.9, false),
            TickAction::Press(direction)
        );
    }
    settings.disable_on_y_change = true;
    assert_eq!(
        tick(&settings, 64.0, 64.0, false),
        TickAction::Press(Direction::Right)
    );
    assert_eq!(
        tick(&settings, 64.0, 64.00000001, false),
        TickAction::Disable(DisableReason::VerticalMovement)
    );
    let mut next = settings.clone();
    next.simple_direction = Direction::Left;
    // Direction's onChanged callback is not guarded by canUpdate, unlike mode's callback.
    assert_eq!(
        settings_changed(&settings, &next, true, false, 0.0),
        vec![Effect::ReleaseDirections]
    );
    assert!(!disable_for_input(
        &settings, true, true, false, false, true
    ));
    settings.disable_on_input = true;
    assert!(disable_for_input(&settings, true, true, false, false, true));
    assert!(!disable_for_input(
        &settings, false, true, false, false, true
    ));
    assert!(!disable_for_input(
        &settings, true, false, false, false, true
    ));
    assert!(!disable_for_input(
        &settings, true, true, true, false, false
    ));
    assert!(!disable_for_input(&settings, true, true, true, true, true));
    assert!(disable_for_input(&settings, true, true, true, true, false));
    assert!(MANUAL_MOVEMENT_MAPPINGS.contains(&"key.jump"));
    assert!(MANUAL_MOVEMENT_MAPPINGS.contains(&"key.sneak"));
    assert!(!MANUAL_MOVEMENT_MAPPINGS.contains(&"key.sprint"));
}

#[test]
fn auto_walk_chunk_guard_preserves_vertical_motion_and_java_truncation() {
    use auto_walk::*;
    assert_eq!(destination_chunk([-0.5, 0.0, -16.5], [0.0; 3]), [0, -1]);
    assert_eq!(
        destination_chunk([15.0, 0.0, 32.0], [0.6, -0.5, -0.1]),
        [1, 1]
    );
    let mut settings = Settings {
        mode: Mode::Simple,
        ..Settings::default()
    };
    let delta = [0.2, -0.0784, 0.1];
    assert_eq!(movement(&settings, delta, false), [0.0, -0.0784, 0.0]);
    assert_eq!(movement(&settings, delta, true), delta);
    settings.no_unloaded_chunks = false;
    assert_eq!(movement(&settings, delta, false), delta);
    settings.no_unloaded_chunks = true;
    settings.mode = Mode::Smart;
    assert_eq!(movement(&settings, delta, false), delta);
    let loaded: Settings =
        serde_json::from_str(&serde_json::to_string(&settings).unwrap()).unwrap();
    assert_eq!(loaded, settings);
}

#[test]
fn safe_walk_drop_policy_uses_heightmap_or_raycast_block_y() {
    use safe_walk::*;
    let mut settings = Settings::default();
    let player = Player {
        y: 64.7,
        block_y: 64,
        on_ground: true,
        ..Player::default()
    };
    let probe = FallProbe {
        motion_blocking_surface_y: 64,
        downward_hit_block_y: None,
    };
    assert!(sufficient_drop(&settings, player, probe));
    settings.minimum_fall_distance = 3;
    assert!(!sufficient_drop(
        &settings,
        player,
        FallProbe {
            motion_blocking_surface_y: 62,
            ..probe
        }
    ));
    assert!(sufficient_drop(
        &settings,
        player,
        FallProbe {
            motion_blocking_surface_y: 61,
            ..probe
        }
    ));
    let underground = FallProbe {
        motion_blocking_surface_y: 80,
        downward_hit_block_y: Some(61),
    };
    assert!(!sufficient_drop(&settings, player, underground)); // (int)(64.7 - 62) == 2
    assert!(sufficient_drop(
        &settings,
        player,
        FallProbe {
            downward_hit_block_y: Some(60),
            ..underground
        }
    ));
    assert!(sufficient_drop(
        &settings,
        player,
        FallProbe {
            downward_hit_block_y: None,
            ..underground
        }
    ));
}

#[test]
fn safe_walk_clipping_and_sneak_decisions_do_not_overwrite_existing_input() {
    use safe_walk::*;
    let mut settings = Settings::default();
    let mut player = Player {
        on_ground: true,
        ..Player::default()
    };
    let probe = FallProbe {
        motion_blocking_surface_y: 0,
        downward_hit_block_y: None,
    };
    assert_eq!(
        clip(&settings, player, probe, false),
        Decision {
            force_clip: true,
            press_sneak: false
        }
    );
    player.shift_key_down = true;
    assert_eq!(clip(&settings, player, probe, false), Decision::default());
    settings.sneak = true;
    assert_eq!(
        clip(&settings, player, probe, true),
        Decision {
            force_clip: false,
            press_sneak: true
        }
    );
    assert_eq!(
        clip(&settings, player, probe, false),
        Decision {
            force_clip: true,
            press_sneak: false
        }
    );
    player.on_ground = false;
    assert_eq!(
        clip(&settings, player, probe, true),
        Decision {
            force_clip: true,
            press_sneak: false
        }
    );
    settings.safe_sneak = false;
    assert_eq!(clip(&settings, player, probe, true), Decision::default());
    player.on_ground = true;
    player.sprint_key_down = true;
    settings.sneak_on_sprint = false;
    assert_eq!(clip(&settings, player, probe, true), Decision::default());
    settings.sneak_on_sprint = true;
    assert!(clip(&settings, player, probe, true).press_sneak);
}

#[test]
fn safe_walk_box_geometry_render_flags_and_settings_bounds() {
    use safe_walk::*;
    let player = Bounds::new([-0.3, 64.0, -0.3], [0.3, 65.8, 0.3]);
    let adjusted = adjusted_box(player, 0.6, 0.3);
    assert_eq!(adjusted.min, [0.0, 63.4, 0.0]);
    assert_eq!(adjusted.max, [0.0, 65.8, 0.0]);
    let mut settings = Settings::default();
    settings.validate().unwrap();
    assert!(outlines(&settings, player, 0.6).is_empty());
    settings.sneak = true;
    settings.render = true;
    assert_eq!(
        outlines(&settings, player, 0.6),
        vec![Outline {
            bounds: adjusted,
            line_rgba: [255, 0, 0, 255]
        }]
    );
    settings.render_player_box = true;
    let boxes = outlines(&settings, player, 0.6);
    assert_eq!(
        boxes[1],
        Outline {
            bounds: player,
            line_rgba: [0, 255, 0, 255]
        }
    );
    settings.edge_distance = -0.25;
    settings.validate().unwrap();
    settings.edge_distance = 1.0;
    settings.validate().unwrap(); // slider limits are not hard constraints
    settings.minimum_fall_distance = 0;
    assert!(settings.validate().is_err());
    settings.minimum_fall_distance = 1;
    settings.edge_distance = f64::INFINITY;
    assert!(settings.validate().is_err());
}
