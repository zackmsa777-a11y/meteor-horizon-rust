// SPDX-License-Identifier: GPL-3.0-only
use meteor_client::modules::{fast_climb, timer};

#[test]
fn finite_double_overflow_is_rejected_before_it_can_poison_the_float_clock() {
    let too_large = f64::from(f32::MAX) * 2.0;
    assert!(too_large.is_finite());
    assert!(
        timer::Settings {
            multiplier: too_large
        }
        .validate()
        .is_err()
    );
    assert!(
        fast_climb::Settings {
            timer: too_large,
            ..Default::default()
        }
        .validate()
        .is_err()
    );
    assert!(
        timer::Settings {
            multiplier: f64::from(f32::MAX)
        }
        .validate()
        .is_ok()
    );
    let mut config = meteor_client::Config::default();
    let before = config.to_json().unwrap();
    assert!(
        config
            .set("timer", "multiplier", &too_large.to_string())
            .is_err()
    );
    assert!(
        config
            .set("fast-climb", "timer", &too_large.to_string())
            .is_err()
    );
    assert_eq!(config.to_json().unwrap(), before);
    config.timer.multiplier = too_large;
    assert!(meteor_client::Config::from_json(&config.to_json().unwrap()).is_err());
}

#[test]
fn timer_override_precedes_active_state_and_off_restores_configured_speed() {
    let mut timer = timer::Timer::default();
    let settings = timer::Settings { multiplier: 2.25 };
    settings.validate().unwrap();
    assert_eq!(timer.multiplier(&settings, false), 1.0);
    assert_eq!(timer.multiplier(&settings, true), 2.25);
    timer.set_override(1.436);
    assert_eq!(timer.multiplier(&settings, false), 1.436);
    assert_eq!(timer.multiplier(&settings, true), 1.436);
    assert_eq!(timer.delta_multiplier(&settings, true), 1.436f32);
    timer.set_override(timer::OFF);
    assert_eq!(timer.multiplier(&settings, true), 2.25);
    assert_eq!(timer.multiplier(&settings, false), 1.0);
    assert!(timer::Settings { multiplier: 0.099 }.validate().is_err());
    assert!(
        timer::Settings {
            multiplier: f64::NAN
        }
        .validate()
        .is_err()
    );
    assert_eq!(timer::Settings::default().multiplier, 1.0);
}

#[test]
fn fast_climb_override_resets_to_timer_module_not_unconditionally_normal_speed() {
    let mut timer = timer::Timer::default();
    let timer_settings = timer::Settings { multiplier: 0.5 };
    let mut climb = fast_climb::State::default();
    climb.activate();
    let settings = fast_climb::Settings {
        timer_mode: true,
        ..Default::default()
    };
    settings.validate().unwrap();
    let player = fast_climb::Player {
        jumping: true,
        on_climbable: true,
        ..Default::default()
    };
    timer.set_override(climb.pre_tick(&settings, player).unwrap());
    assert_eq!(timer.multiplier(&timer_settings, false), settings.timer);
    assert_eq!(
        fast_climb::post_tick(&settings, player, [0.1, 0.2, 0.3]),
        [0.1, 0.2, 0.3]
    );
    timer.set_override(
        climb
            .pre_tick(&settings, fast_climb::Player::default())
            .unwrap(),
    );
    assert_eq!(timer.multiplier(&timer_settings, true), 0.5);
    assert_eq!(timer.multiplier(&timer_settings, false), 1.0);
    assert!(
        climb
            .pre_tick(&settings, fast_climb::Player::default())
            .is_none()
    );
}
