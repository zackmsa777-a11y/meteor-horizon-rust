#![cfg(feature = "meteor")]
// SPDX-License-Identifier: GPL-3.0-only
use horizon_client::meteor_import;
use horizon_world::{
    compression::Compression,
    nbt::{self, Limits, NamedTag, Tag},
};
use meteor_client::Config;

fn s(value: &str) -> Tag {
    Tag::String(value.into())
}
fn bytes(root: Tag) -> Vec<u8> {
    nbt::write_named(
        &NamedTag {
            name: "".into(),
            tag: root,
        },
        Limits::default(),
    )
    .unwrap()
}
fn setting(name: &str, value: Tag) -> Tag {
    Tag::compound([("name", s(name)), ("value", value)])
}
fn module(name: &str, settings: Vec<Tag>) -> Tag {
    Tag::compound([
        ("name", s(name)),
        ("active", Tag::Byte(1)),
        (
            "settings",
            Tag::compound([(
                "groups",
                Tag::List(vec![Tag::compound([
                    ("name", s("General")),
                    ("sectionExpanded", Tag::Byte(1)),
                    ("settings", Tag::List(settings)),
                ])]),
            )]),
        ),
    ])
}
fn modules(entries: Vec<Tag>) -> Vec<u8> {
    bytes(Tag::compound([("modules", Tag::List(entries))]))
}

#[test]
fn source_item_list_setting_imports_string_list_and_rejects_wrong_entries_atomically() {
    let base = Config::default();
    let input = modules(vec![module(
        "fast-use",
        vec![
            setting("mode", s("Some")),
            setting(
                "items",
                Tag::List(vec![s("minecraft:stone"), s("minecraft:experience_bottle")]),
            ),
            setting("blocks", Tag::Byte(1)),
        ],
    )]);
    let result = meteor_import::modules(&input, &base).unwrap();
    assert_eq!(
        result.candidate.fast_use.items,
        ["minecraft:stone", "minecraft:experience_bottle"]
    );
    assert_eq!(
        result.candidate.fast_use.mode,
        meteor_client::modules::fast_use::Mode::Some
    );
    assert!(result.candidate.fast_use.blocks);
    assert_eq!(result.report.settings, 3);
    let bad = modules(vec![module(
        "fast-use",
        vec![setting("items", Tag::List(vec![Tag::Int(1)]))],
    )]);
    assert!(meteor_import::modules(&bad, &base).is_err());
    assert!(base.fast_use.items.is_empty());
}

#[test]
fn source_modules_fixture_imports_changed_values_metadata_keys_and_reports_unported() {
    let mut entry = module(
        "zoom",
        vec![
            setting("zoom", Tag::Double(4.5f64.to_bits())),
            setting("smooth", Tag::Byte(0)),
            setting("future-option", Tag::Int(1)),
        ],
    );
    let Tag::Compound(ref mut map) = entry else {
        unreachable!()
    };
    map.insert("favorite".into(), Tag::Byte(1));
    map.insert("toggleOnKeyRelease".into(), Tag::Byte(1));
    map.insert(
        "keybind".into(),
        Tag::compound([
            ("key", s("key.keyboard.r")),
            ("modifiers", Tag::List(vec![s("CONTROL"), s("SHIFT")])),
        ]),
    );
    let mut base = Config::default();
    base.zoom.scroll_sensitivity = 9.0;
    base.modules.get_mut("sprint").unwrap().active = true;
    let result =
        meteor_import::modules(&modules(vec![entry, module("elytra-fly", vec![])]), &base).unwrap();
    assert_eq!(result.candidate.zoom.zoom, 4.5);
    assert!(!result.candidate.zoom.smooth);
    assert_eq!(
        result.candidate.zoom.scroll_sensitivity,
        Config::default().zoom.scroll_sensitivity
    );
    let zoom = result.candidate.modules.get("zoom").unwrap();
    assert!(zoom.active && zoom.favorite && zoom.toggle_on_bind_release);
    assert_eq!(zoom.keybind.key.as_deref(), Some("key.keyboard.r"));
    assert_eq!(zoom.keybind.modifiers, 3);
    assert!(!result.candidate.modules.get("sprint").unwrap().active);
    assert_eq!(result.report.settings, 2);
    assert_eq!(result.report.skipped.len(), 2);
    assert_eq!(base.zoom.scroll_sensitivity, 9.0);
    assert!(base.modules.get("sprint").unwrap().active);
}

#[test]
fn malformed_or_invalid_values_never_mutate_base_and_gzip_is_bounded() {
    let base = Config::default();
    let before = base.to_json().unwrap();
    let valid = modules(vec![module(
        "high-jump",
        vec![setting("jump-multiplier", Tag::Double(2.0f64.to_bits()))],
    )]);
    let compressed = Compression::Gzip.encode(&valid).unwrap();
    assert_eq!(
        meteor_import::modules(&compressed, &base)
            .unwrap()
            .candidate
            .high_jump
            .jump_multiplier,
        2.0
    );
    assert!(meteor_import::modules(&valid[..valid.len() - 1], &base).is_err());
    let mut extra = valid.clone();
    extra.push(0);
    assert!(meteor_import::modules(&extra, &base).is_err());
    assert!(
        meteor_import::modules(
            &modules(vec![module(
                "high-jump",
                vec![setting("jump-multiplier", Tag::Double(f64::NAN.to_bits()))]
            )]),
            &base
        )
        .is_err()
    );
    assert!(
        meteor_import::modules(
            &modules(vec![module("zoom", vec![setting("smooth", s("false"))])]),
            &base
        )
        .is_err()
    );
    let bomb = Compression::Gzip
        .encode(&vec![0; meteor_import::MAX_BYTES + 1])
        .unwrap();
    assert!(meteor_import::modules(&bomb, &base).is_err());
    assert_eq!(base.to_json().unwrap(), before);
}

#[test]
fn legacy_glfw_keys_convert_but_unsupported_modifiers_are_reported_not_discarded() {
    let mut legacy = module("zoom", vec![]);
    let Tag::Compound(ref mut map) = legacy else {
        unreachable!()
    };
    map.insert(
        "keybind".into(),
        Tag::compound([
            ("isKey", Tag::Byte(1)),
            ("value", Tag::Int(82)),
            ("modifiers", Tag::Int(2)),
        ]),
    );
    let result = meteor_import::modules(&modules(vec![legacy]), &Config::default()).unwrap();
    assert_eq!(
        result
            .candidate
            .modules
            .get("zoom")
            .unwrap()
            .keybind
            .key
            .as_deref(),
        Some("key.keyboard.r")
    );
    assert_eq!(
        result
            .candidate
            .modules
            .get("zoom")
            .unwrap()
            .keybind
            .modifiers,
        2
    );
    let mut unsupported = module("zoom", vec![]);
    let Tag::Compound(ref mut map) = unsupported else {
        unreachable!()
    };
    map.insert(
        "keybind".into(),
        Tag::compound([
            ("key", s("key.keyboard.r")),
            ("modifiers", Tag::List(vec![s("CAPS_LOCK")])),
        ]),
    );
    let result = meteor_import::modules(&modules(vec![unsupported]), &Config::default()).unwrap();
    assert!(
        result
            .candidate
            .modules
            .get("zoom")
            .unwrap()
            .keybind
            .key
            .is_none()
    );
    assert!(result.report.skipped[0].contains("CAPS_LOCK"));
}

#[test]
fn friends_fixture_retains_uuid_and_sorts_deduplicated_names_without_network() {
    let input = bytes(Tag::compound([(
        "friends",
        Tag::List(vec![
            Tag::compound([
                ("name", s("Zack")),
                ("id", s("01234567-89ab-cdef-0123-456789abcdef")),
            ]),
            Tag::compound([("name", s("alex"))]),
            Tag::compound([("name", s("zAcK"))]),
        ]),
    )]));
    let imported = meteor_import::friends(&input, &Config::default()).unwrap();
    assert_eq!(imported.report.friends, 2);
    assert_eq!(imported.candidate.friends.friends[0].name, "alex");
    assert_eq!(
        imported.candidate.friends.friends[1].id.as_deref(),
        Some("0123456789abcdef0123456789abcdef")
    );
    assert_eq!(imported.report.skipped.len(), 1);
    let bad = bytes(Tag::compound([(
        "friends",
        Tag::List(vec![Tag::compound([
            ("name", s("Zack")),
            ("id", s("not-a-uuid")),
        ])]),
    )]));
    assert!(meteor_import::friends(&bad, &Config::default()).is_err());
}

#[test]
fn color_and_entity_list_source_tags_import_without_losing_rainbow_or_alpha() {
    let base = Config::default();
    let input = modules(vec![
        module(
            "air-place",
            vec![
                setting(
                    "side-color",
                    Tag::compound([
                        ("r", Tag::Int(300)),
                        ("g", Tag::Int(-1)),
                        ("b", Tag::Int(12)),
                        ("a", Tag::Int(10)),
                        ("rainbow", Tag::Byte(1)),
                    ]),
                ),
                setting("shape-mode", s("Sides")),
            ],
        ),
        module(
            "no-mining-trace",
            vec![setting(
                "blacklisted-entities",
                Tag::List(vec![s("minecraft:cow")]),
            )],
        ),
        module("liquid-interact", vec![]),
    ]);
    let result = meteor_import::modules(&input, &base).unwrap();
    assert_eq!(
        result.candidate.air_place.side_color.rgba(),
        [255, 0, 12, 10]
    );
    assert!(result.candidate.air_place.side_color.rainbow);
    assert_eq!(
        result.candidate.no_mining_trace.blacklisted_entities,
        ["minecraft:cow"]
    );
    assert_eq!(result.report.settings, 3);
    assert!(result.report.skipped.is_empty());
}

#[test]
fn flight_import_disambiguates_source_groups_and_is_atomic() {
    let group = |name: &str, values: Vec<Tag>| {
        Tag::compound([("name", s(name)), ("settings", Tag::List(values))])
    };
    let flight = |groups| {
        Tag::compound([
            ("name", s("flight")),
            ("active", Tag::Byte(1)),
            ("settings", Tag::compound([("groups", Tag::List(groups))])),
        ])
    };
    let base = Config::default();
    let input = modules(vec![flight(vec![
        group(
            "General",
            vec![
                setting("mode", s("Velocity")),
                setting("speed", Tag::Double(0.2f64.to_bits())),
            ],
        ),
        group(
            "Anti Kick",
            vec![setting("mode", s("Normal")), setting("delay", Tag::Int(9))],
        ),
    ])]);
    let result = meteor_import::modules(&input, &base).unwrap();
    assert_eq!(
        result.candidate.flight.mode,
        meteor_client::modules::flight::Mode::Velocity
    );
    assert_eq!(
        result.candidate.flight.anti_kick_mode,
        meteor_client::modules::flight::AntiKick::Normal
    );
    assert_eq!(result.candidate.flight.delay, 9);
    assert_eq!(result.report.settings, 4);
    let duplicate = modules(vec![flight(vec![group(
        "Anti Kick",
        vec![setting("mode", s("Normal")), setting("mode", s("None"))],
    )])]);
    assert!(meteor_import::modules(&duplicate, &base).is_err());
    let invalid = modules(vec![flight(vec![group(
        "General",
        vec![setting("speed", Tag::Double(1e300f64.to_bits()))],
    )])]);
    assert!(meteor_import::modules(&invalid, &base).is_err());
    assert_eq!(base.flight.speed, 0.1);
}
