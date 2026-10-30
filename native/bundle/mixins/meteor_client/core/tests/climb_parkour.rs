// SPDX-License-Identifier: GPL-3.0-only
use meteor_client::modules::{fast_climb, parkour, spider};

#[test]
fn fast_climb_requires_collision_or_jump_and_real_climbable_surface() {
    use fast_climb::*;
    let settings = Settings::default();
    settings.validate().unwrap();
    let mut player = Player {
        on_climbable: true,
        ..Default::default()
    };
    let v = [0.4, -0.1, -0.2];
    assert_eq!(post_tick(&settings, player, v), v);
    player.jumping = true;
    assert_eq!(post_tick(&settings, player, v), [0.4, 0.2872, -0.2]);
    player.on_climbable = false;
    player.in_powder_snow_block = true;
    assert!(!climbing(player));
    player.can_walk_powder_snow = true;
    assert!(climbing(player));
    player.jumping = false;
    player.horizontal_collision = true;
    assert!(climbing(player));
    assert_eq!(
        post_tick(
            &Settings {
                timer_mode: true,
                ..settings
            },
            player,
            v
        ),
        v
    );
}

#[test]
fn fast_climb_timer_resets_once_and_can_reenter() {
    use fast_climb::*;
    let settings = Settings {
        timer_mode: true,
        ..Default::default()
    };
    let mut state = State::default();
    state.activate();
    assert_eq!(state.pre_tick(&settings, Player::default()), Some(1.0));
    assert_eq!(state.pre_tick(&settings, Player::default()), None);
    let climbing = Player {
        horizontal_collision: true,
        on_climbable: true,
        ..Default::default()
    };
    assert_eq!(state.pre_tick(&settings, climbing), Some(1.436));
    assert_eq!(state.pre_tick(&settings, climbing), Some(1.436));
    assert_eq!(state.pre_tick(&settings, Player::default()), Some(1.0));
    assert_eq!(state.pre_tick(&Settings::default(), climbing), None);
    assert!(
        Settings {
            timer: 0.9,
            ..settings
        }
        .validate()
        .is_err()
    );
}

#[test]
fn spider_threshold_is_fixed_not_configured_speed() {
    use spider::*;
    let settings = Settings { climb_speed: 0.9 };
    settings.validate().unwrap();
    assert_eq!(Settings::default().climb_speed, 0.2);
    assert_eq!(
        post_tick(&settings, true, [0.3, 0.199, -0.4]),
        [0.3, 0.9, -0.4]
    );
    assert_eq!(
        post_tick(&settings, true, [0.3, 0.2, -0.4]),
        [0.3, 0.2, -0.4]
    );
    assert_eq!(
        post_tick(&settings, false, [0.3, -1.0, -0.4]),
        [0.3, -1.0, -0.4]
    );
    assert!(Settings { climb_speed: -1.0 }.validate().is_err());
}

#[test]
fn parkour_shift_and_physical_keys_gate_block_only_probe() {
    use parkour::*;
    let settings = Settings::default();
    settings.validate().unwrap();
    let player = Player {
        on_ground: true,
        ..Default::default()
    };
    assert!(should_jump(player, false));
    assert!(!should_jump(player, true));
    for excluded in [
        Player {
            on_ground: false,
            ..player
        },
        Player {
            jump_key_down: true,
            ..player
        },
        Player {
            shift_key_down: true,
            ..player
        },
        Player {
            sneak_key_down: true,
            ..player
        },
    ] {
        assert!(!eligible(excluded));
        assert!(!should_jump(excluded, false));
    }
    let (min, max) = adjusted_box(&settings, [-0.3, 64.0, -0.3], [0.3, 65.8, 0.3]);
    assert_eq!(min, [-0.299, 63.5, -0.299]);
    assert_eq!(max, [0.299, 65.3, 0.299]);
    assert!(Settings { edge_distance: 0.1 }.validate().is_ok());
    assert!(
        Settings {
            edge_distance: 0.1001
        }
        .validate()
        .is_err()
    );
}
