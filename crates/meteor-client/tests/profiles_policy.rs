use meteor_client::Config;

#[test]
fn quoted_profile_arguments_preserve_spaces_and_reject_partial_quotes() {
    assert_eq!(
        meteor_client::profiles::arguments("create \"My Profile\" modules,hud").unwrap(),
        ["create", "My Profile", "modules,hud"]
    );
    assert_eq!(
        meteor_client::profiles::arguments("joins 'My Profile' 'World One' example.org:25565")
            .unwrap(),
        ["joins", "My Profile", "World One", "example.org:25565"]
    );
    for invalid in ["load \"Open", "load \"Closed\"tail", "load \"bad\\x\""] {
        assert!(meteor_client::profiles::arguments(invalid).is_err());
    }
}
use meteor_client::profiles;
use profiles::{Profiles, SaveSections};

#[test]
fn hud_runtime_reloads_only_when_selected_snapshot_exists() {
    let config = Config::default();
    let mut profiles = Profiles::default();
    profiles
        .create(
            "Hud lifecycle".into(),
            SaveSections {
                hud: true,
                ..Default::default()
            },
            vec![],
            &config,
        )
        .unwrap();
    assert!(profiles.load("Hud lifecycle", &config).unwrap().reload_hud);
    profiles
        .configure("Hud lifecycle", SaveSections::default(), vec![])
        .unwrap();
    assert!(!profiles.load("Hud lifecycle", &config).unwrap().reload_hud);
    profiles
        .configure(
            "Hud lifecycle",
            SaveSections {
                hud: true,
                ..Default::default()
            },
            vec![],
        )
        .unwrap();
    profiles.entries[0].snapshot.hud = None;
    assert!(!profiles.load("Hud lifecycle", &config).unwrap().reload_hud);
}

#[test]
fn macros_profile_restores_messages_and_bindings_without_restarting_modules() {
    let mut saved = zoom(3.0);
    saved.macros.macros.push(meteor_client::macros::Macro {
        name: "Greeting".into(),
        messages: vec!["hello {player.name}".into()],
        keybind: meteor_client::registry::Keybind {
            key: Some("key.keyboard.g".into()),
            modifiers: 2,
        },
    });
    let mut profiles = Profiles::default();
    profiles
        .create(
            "Macros".into(),
            SaveSections {
                macros: true,
                ..Default::default()
            },
            vec![],
            &saved,
        )
        .unwrap();
    let mut current = zoom(8.0);
    current.friends.command(&["add", "KeepMe"]).unwrap();
    let plan = profiles.load("Macros", &current).unwrap();
    assert!(plan.reload_macros);
    assert!(!plan.reload_hud);
    assert!(!plan.reload_modules);
    assert!(plan.deactivate.is_empty());
    assert_eq!(plan.candidate.zoom.zoom, 8.0);
    assert_eq!(
        plan.candidate.macros.macros[0].messages,
        ["hello {player.name}"]
    );
    assert_eq!(plan.candidate.macros.macros[0].keybind.modifiers, 2);
    assert_eq!(plan.candidate.friends.friends.len(), 1);
    profiles
        .configure("Macros", SaveSections::default(), vec![])
        .unwrap();
    profiles.save("Macros", &current).unwrap();
    assert!(!profiles.load("Macros", &current).unwrap().reload_macros);
    profiles
        .configure(
            "Macros",
            SaveSections {
                macros: true,
                ..Default::default()
            },
            vec![],
        )
        .unwrap();
    assert_eq!(
        profiles
            .load("Macros", &current)
            .unwrap()
            .candidate
            .macros
            .macros[0]
            .name,
        "Greeting"
    );
}

#[test]
fn missing_macro_snapshot_is_noop_and_invalid_macro_snapshot_is_atomic() {
    let current = zoom(6.0);
    let mut profiles = Profiles::default();
    profiles
        .create("Empty".into(), SaveSections::default(), vec![], &current)
        .unwrap();
    profiles
        .configure(
            "Empty",
            SaveSections {
                macros: true,
                ..Default::default()
            },
            vec![],
        )
        .unwrap();
    assert!(!profiles.load("Empty", &current).unwrap().reload_macros);
    profiles.entries[0].snapshot.macros = Some(
        serde_json::json!({"macros":[{"name":"bad","messages":["test"],"keybind":{"key":"invalid","modifiers":0}}]}),
    );
    assert!(profiles.load("Empty", &current).is_err());
    assert!(current.macros.macros.is_empty());
}

fn modules() -> SaveSections {
    SaveSections {
        modules: true,
        ..Default::default()
    }
}
fn zoom(value: f64) -> Config {
    let mut config = Config::default();
    config.zoom.zoom = value;
    config.modules.get_mut("zoom").unwrap().active = true;
    config
}

#[test]
fn save_load_restores_state_bindings_settings_and_hud_but_not_friends() {
    let mut saved = zoom(7.0);
    saved.modules.get_mut("zoom").unwrap().keybind.key = Some("key.keyboard.z".into());
    saved.hud.active = true;
    let mut profiles = Profiles::default();
    profiles
        .create(
            "Building".into(),
            SaveSections {
                hud: true,
                ..modules()
            },
            vec![],
            &saved,
        )
        .unwrap();
    let mut live = Config::default();
    live.modules.get_mut("sprint").unwrap().active = true;
    live.friends.command(&["add", "Alice"]).unwrap();
    let load = profiles.load("building", &live).unwrap();
    assert_eq!(load.name, "Building");
    assert_eq!(load.candidate.zoom.zoom, 7.0);
    assert_eq!(
        load.candidate
            .modules
            .get("zoom")
            .unwrap()
            .keybind
            .key
            .as_deref(),
        Some("key.keyboard.z")
    );
    assert!(load.candidate.hud.active);
    assert_eq!(load.candidate.friends.friends[0].name, "Alice");
    assert_eq!(load.deactivate, ["sprint"]);
    assert_eq!(load.activate, ["zoom"]);
    assert!(load.reload_modules);
}

#[test]
fn modules_load_restarts_even_an_unchanged_active_module() {
    let live = zoom(3.0);
    let mut profiles = Profiles::default();
    profiles
        .create("Same".into(), modules(), vec![], &live)
        .unwrap();
    let load = profiles.load("Same", &live).unwrap();
    assert_eq!(load.deactivate, ["zoom"]);
    assert_eq!(load.activate, ["zoom"]);
}

#[test]
fn hud_only_profile_preserves_modules_and_their_lifecycle() {
    let mut saved = Config::default();
    saved.hud.active = true;
    let mut profiles = Profiles::default();
    profiles
        .create(
            "Hud".into(),
            SaveSections {
                hud: true,
                ..Default::default()
            },
            vec![],
            &saved,
        )
        .unwrap();
    let load = profiles.load("Hud", &zoom(8.0)).unwrap();
    assert_eq!(load.candidate.zoom.zoom, 8.0);
    assert!(load.candidate.modules.get("zoom").unwrap().active);
    assert!(load.candidate.hud.active);
    assert!(!load.reload_modules && load.deactivate.is_empty() && load.activate.is_empty());
}

#[test]
fn unchecked_save_keeps_prior_snapshot_and_missing_sections_are_noops() {
    let mut profiles = Profiles::default();
    profiles
        .create("Keep".into(), modules(), vec![], &zoom(8.0))
        .unwrap();
    profiles
        .configure("Keep", SaveSections::default(), vec![])
        .unwrap();
    profiles.save("Keep", &zoom(2.0)).unwrap();
    profiles.configure("Keep", modules(), vec![]).unwrap();
    assert_eq!(
        profiles
            .load("Keep", &Config::default())
            .unwrap()
            .candidate
            .zoom
            .zoom,
        8.0
    );
    profiles
        .create(
            "Empty".into(),
            SaveSections::default(),
            vec![],
            &Config::default(),
        )
        .unwrap();
    profiles.configure("Empty", modules(), vec![]).unwrap();
    let load = profiles.load("Empty", &zoom(4.0)).unwrap();
    assert!(!load.reload_modules);
    assert_eq!(load.candidate.zoom.zoom, 4.0);
}

#[test]
fn join_rules_are_exact_ordered_and_failures_do_not_erase_other_loads() {
    let mut profiles = Profiles::default();
    for (name, amount) in [("First", 2.0), ("Broken", 4.0), ("Last", 6.0)] {
        profiles
            .create(
                name.into(),
                modules(),
                vec!["example.org:25565".into()],
                &zoom(amount),
            )
            .unwrap();
    }
    profiles.entries[1]
        .snapshot
        .modules
        .as_mut()
        .unwrap()
        .get_mut("zoom")
        .unwrap()["zoom"] = serde_json::json!(-5);
    assert!(
        profiles
            .on_join("EXAMPLE.org:25565", &Config::default())
            .loads
            .is_empty()
    );
    assert!(
        profiles
            .on_join("example.org", &Config::default())
            .loads
            .is_empty()
    );
    let out = profiles.on_join("example.org:25565", &Config::default());
    assert_eq!(out.loads.len(), 2);
    assert_eq!(out.errors.len(), 1);
    assert_eq!(out.errors[0].0, "Broken");
    assert_eq!(out.loads[0].name, "First");
    assert_eq!(out.loads[1].candidate.zoom.zoom, 6.0);
    assert_eq!(out.loads[1].deactivate, ["zoom"]);
}

#[test]
fn json_roundtrip_names_validation_and_delete_keep_saved_data_consistent() {
    let mut profiles = Profiles::default();
    profiles
        .create(
            "Mining 1".into(),
            modules(),
            vec!["My World".into(), "realms".into()],
            &zoom(9.0),
        )
        .unwrap();
    let json = profiles.to_json().unwrap();
    let mut restored = Profiles::from_json(&json).unwrap();
    assert_eq!(
        restored.on_join("My World", &Config::default()).loads[0]
            .candidate
            .zoom
            .zoom,
        9.0
    );
    assert_eq!(
        restored.on_join("realms", &Config::default()).loads.len(),
        1
    );
    assert!(
        restored
            .create("MINING 1".into(), modules(), vec![], &Config::default())
            .is_err()
    );
    assert!(
        restored
            .create("../escape".into(), modules(), vec![], &Config::default())
            .is_err()
    );
    assert!(
        restored
            .configure(
                "Mining 1",
                SaveSections {
                    waypoints: true,
                    ..Default::default()
                },
                vec![]
            )
            .is_err()
    );
    assert!(restored.get("Mining 1").unwrap().save.modules);
    assert_eq!(restored.remove("mining 1").unwrap().name, "Mining 1");
    assert!(restored.entries.is_empty());
    assert!(Profiles::from_json(&json.replace("\"schema\": 1", "\"schema\": 2")).is_err());
}

#[test]
fn rejected_load_cannot_change_live_configuration_or_smuggle_friends() {
    let mut profiles = Profiles::default();
    profiles
        .create("Invalid".into(), modules(), vec![], &zoom(2.0))
        .unwrap();
    profiles.entries[0]
        .snapshot
        .modules
        .as_mut()
        .unwrap()
        .insert("friends".into(), serde_json::json!({"friends":[]}));
    let live = zoom(3.0);
    let before = live.to_json().unwrap();
    assert!(profiles.load("Invalid", &live).is_err());
    assert_eq!(live.to_json().unwrap(), before);
}
