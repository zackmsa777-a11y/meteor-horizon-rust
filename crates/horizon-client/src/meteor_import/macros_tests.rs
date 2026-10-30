#![cfg(feature = "meteor")]
// SPDX-License-Identifier: GPL-3.0-only
// Synthetic fixtures follow original Macro/Settings/SettingGroup serializers.
use crate::meteor_import;
use horizon_world::{
    compression::Compression,
    nbt::{self, Limits, NamedTag, Tag},
};
use meteor_client::{Config, macros::Macro};

fn s(value: &str) -> Tag {
    Tag::String(value.into())
}
fn setting(name: &str, value: Tag) -> Tag {
    Tag::compound([("name", s(name)), ("value", value)])
}
fn entry(settings: Vec<Tag>) -> Tag {
    Tag::compound([(
        "settings",
        Tag::compound([(
            "groups",
            Tag::List(vec![Tag::compound([
                ("name", s("General")),
                ("sectionExpanded", Tag::Byte(1)),
                ("settings", Tag::List(settings)),
            ])]),
        )]),
    )])
}
fn bytes(entries: Vec<Tag>) -> Vec<u8> {
    nbt::write_named(
        &NamedTag {
            name: "".into(),
            tag: Tag::compound([("macros", Tag::List(entries))]),
        },
        Limits::default(),
    )
    .unwrap()
}
fn bind(key: &str) -> Tag {
    Tag::compound([
        ("key", s(key)),
        ("modifiers", Tag::List(vec![s("CONTROL"), s("SHIFT")])),
    ])
}

#[test]
fn original_settings_layout_imports_chords_messages_duplicates_and_scopes_atomically() {
    let first = entry(vec![
        setting("name", s("Greeting")),
        setting(
            "messages",
            Tag::List(vec![s("hello {player.name}"), s("/meteor zoom 4")]),
        ),
        setting("keybind", bind("key.keyboard.g")),
    ]);
    let second = entry(vec![
        setting("name", s("Greeting")),
        setting("messages", Tag::List(vec![s("second")])),
    ]);
    let mut base = Config::default();
    base.zoom.zoom = 9.;
    base.modules.get_mut("zoom").unwrap().active = true;
    base.hud.active = false;
    base.friends.command(&["add", "KeepMe"]).unwrap();
    base.macros.macros.push(Macro {
        name: "Old".into(),
        messages: vec!["old".into()],
        ..Default::default()
    });
    let imported = meteor_import::macros(&bytes(vec![first, second]), &base).unwrap();
    assert_eq!(imported.report.macros, 2);
    assert_eq!(imported.report.settings, 5);
    let got = &imported.candidate.macros.macros;
    assert_eq!(got[0].name, "Greeting");
    assert_eq!(got[1].name, "Greeting");
    assert_eq!(got[0].messages, ["hello {player.name}", "/meteor zoom 4"]);
    assert_eq!(got[0].keybind.key.as_deref(), Some("key.keyboard.g"));
    assert_eq!(got[0].keybind.modifiers, 3);
    assert_eq!(got[1].keybind.key, None);
    assert_eq!(imported.candidate.zoom.zoom, 9.);
    assert!(imported.candidate.modules.get("zoom").unwrap().active);
    assert!(!imported.candidate.hud.active);
    assert_eq!(imported.candidate.friends.friends.len(), 1);
    assert_eq!(base.macros.macros[0].name, "Old");
}

#[test]
fn missing_settings_defaults_and_later_duplicate_setting_follow_source_order() {
    let input = bytes(vec![
        Tag::compound([] as [(&str, Tag); 0]),
        entry(vec![
            setting("name", s("Old")),
            setting("name", s("New")),
            setting("messages", Tag::List(vec![s("")])),
        ]),
    ]);
    let result = meteor_import::macros(&input, &Config::default()).unwrap();
    assert_eq!(result.candidate.macros.macros[0].name, "");
    assert!(result.candidate.macros.macros[0].messages.is_empty());
    assert_eq!(result.candidate.macros.macros[1].name, "New");
    assert_eq!(result.candidate.macros.macros[1].messages, [""]);
}

#[test]
fn legacy_mouse_and_unsupported_modifiers_are_explicitly_mapped_or_unset() {
    let legacy = Tag::compound([
        ("isKey", Tag::Byte(0)),
        ("value", Tag::Int(2)),
        ("modifiers", Tag::Int(99)),
    ]);
    let unsupported = Tag::compound([
        ("key", s("key.keyboard.g")),
        ("modifiers", Tag::List(vec![s("CAPS_LOCK")])),
    ]);
    let input = bytes(vec![
        entry(vec![setting("keybind", legacy)]),
        entry(vec![
            setting("keybind", unsupported),
            setting("future-setting", Tag::Int(1)),
        ]),
    ]);
    let imported = meteor_import::macros(&input, &Config::default()).unwrap();
    assert_eq!(
        imported.candidate.macros.macros[0].keybind.key.as_deref(),
        Some("key.mouse.middle")
    );
    assert_eq!(imported.candidate.macros.macros[0].keybind.modifiers, 0);
    assert_eq!(imported.candidate.macros.macros[1].keybind.key, None);
    assert_eq!(imported.report.skipped.len(), 2);
}

#[test]
fn gzip_malformed_trailing_and_oversized_data_never_mutate_the_base() {
    let base = Config::default();
    let input = bytes(vec![entry(vec![setting("name", s("Compressed"))])]);
    let gz = Compression::Gzip.encode(&input).unwrap();
    assert_eq!(meteor_import::macros(&gz, &base).unwrap().report.macros, 1);
    let bad = bytes(vec![entry(vec![setting(
        "messages",
        Tag::List(vec![Tag::Int(4)]),
    )])]);
    assert!(meteor_import::macros(&bad, &base).is_err());
    let mut trailing = input;
    trailing.push(0);
    assert!(meteor_import::macros(&trailing, &base).is_err());
    assert!(meteor_import::macros(&vec![0; meteor_import::MAX_BYTES + 1], &base).is_err());
    assert!(
        meteor_import::macros(
            &bytes(vec![Tag::compound([] as [(&str, Tag); 0]); 1025]),
            &base
        )
        .is_err()
    );
    assert!(base.macros.macros.is_empty());
}
