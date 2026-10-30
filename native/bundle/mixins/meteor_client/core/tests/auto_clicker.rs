// SPDX-License-Identifier: GPL-3.0-only
use meteor_client::{Config, modules::auto_clicker::Mode};

#[test]
fn source_settings_round_trip_and_reject_invalid_edits_atomically() {
    let mut config = Config::default();
    config.set("auto-clicker", "mode-left", "Hold").unwrap();
    config.set("auto-clicker", "delay-right", "0").unwrap();
    let before = serde_json::to_string(&config).unwrap();
    assert!(config.set("auto-clicker", "delay-right", "-1").is_err());
    assert!(config.set("auto-clicker", "mode-left", "Fake").is_err());
    assert_eq!(serde_json::to_string(&config).unwrap(), before);
    let restored = Config::from_json(&before).unwrap();
    assert_eq!(restored.auto_clicker.mode_left, Mode::Hold);
    assert_eq!(restored.auto_clicker.delay_right, 0);
    let mut bad = serde_json::to_value(&restored).unwrap();
    bad["auto_clicker"]["delay_left"] = (-5).into();
    assert!(Config::from_json(&bad.to_string()).is_err());
}
