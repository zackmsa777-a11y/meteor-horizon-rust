// SPDX-License-Identifier: GPL-3.0-only
#[path = "../src/modules/fullbright.rs"]
mod fullbright;
use fullbright::*;
#[test]
fn gamma_and_luminance_select_exact_channel_only_when_active() {
    let mut settings = Settings::default();
    settings.validate().unwrap();
    assert!(settings.gamma(true));
    assert!(!settings.gamma(false));
    assert_eq!(settings.floors(true), [0, 0]);
    settings.mode = Mode::Luminance;
    assert!(!settings.gamma(true));
    assert_eq!(settings.floors(true), [8, 0]);
    settings.light_type = LightType::SKY;
    assert_eq!(settings.floors(true), [0, 8]);
    assert_eq!(settings.floors(false), [0, 0]);
    settings.minimum_light_level = 16;
    assert!(settings.validate().is_err());
}
#[test]
fn lifecycle_rebuilds_and_removes_night_vision_on_source_callbacks() {
    let gamma = Settings::default();
    let potion = Settings {
        mode: Mode::Potion,
        ..gamma.clone()
    };
    let luminance = Settings {
        mode: Mode::Luminance,
        ..gamma.clone()
    };
    assert!(activate(&gamma).is_empty());
    assert_eq!(activate(&luminance), vec![Effect::RebuildSections]);
    assert!(deactivate(&gamma).is_empty());
    assert_eq!(deactivate(&potion), vec![Effect::RemoveNightVision]);
    assert_eq!(deactivate(&luminance), vec![Effect::RebuildSections]);
    assert_eq!(
        settings_changed(&gamma, &luminance, true),
        vec![Effect::RemoveNightVision, Effect::RebuildSections]
    );
    assert_eq!(
        settings_changed(&gamma, &potion, true),
        vec![Effect::RebuildSections]
    );
    assert!(settings_changed(&potion, &gamma, false).is_empty());
}
#[test]
fn potion_refreshes_short_and_infinite_duration_but_never_shortens_long_effects() {
    let settings = Settings {
        mode: Mode::Potion,
        ..Default::default()
    };
    assert_eq!(post_tick(&settings, true, None), NightVision::Add420);
    for duration in [-1, 0, 419] {
        assert_eq!(
            post_tick(&settings, true, Some(duration)),
            NightVision::SetDuration420
        );
    }
    for duration in [420, 1000] {
        assert_eq!(
            post_tick(&settings, true, Some(duration)),
            NightVision::None
        );
    }
    assert_eq!(post_tick(&settings, false, None), NightVision::None);
    assert_eq!(
        post_tick(&Settings::default(), true, None),
        NightVision::None
    );
}

#[test]
fn config_validates_fullbright_edits_atomically() {
    let mut config = meteor_client::Config::default();
    assert!(config.modules.get("fullbright").is_some());
    config.set("fullbright", "mode", "Luminance").unwrap();
    config.set("fullbright", "light-type", "SKY").unwrap();
    config
        .set("fullbright", "minimum-light-level", "12")
        .unwrap();
    assert!(
        config
            .set("fullbright", "minimum-light-level", "16")
            .is_err()
    );
    assert_eq!(config.fullbright.minimum_light_level, 12);
    let loaded = meteor_client::Config::from_json(&config.to_json().unwrap()).unwrap();
    assert_eq!(loaded.fullbright.floors(true), [0, 12]);
}
