// SPDX-License-Identifier: GPL-3.0-only
// Generated original-layout NBT; stage as meteor_import/hud_tests.rs after freeze.
use crate::meteor_import;
use horizon_world::nbt::{self, Limits, NamedTag, Tag};
use meteor_client::Config;

fn s(v: &str) -> Tag {
    Tag::String(v.into())
}
fn c(v: impl IntoIterator<Item = (&'static str, Tag)>) -> Tag {
    Tag::compound(v)
}
fn setting(name: &str, value: Tag) -> Tag {
    c([("name", s(name)), ("value", value)])
}
fn group(name: &str, settings: Vec<Tag>) -> Tag {
    c([("name", s(name)), ("settings", Tag::List(settings))])
}
fn settings(groups: Vec<Tag>) -> Tag {
    c([("groups", Tag::List(groups))])
}
fn text(body: &str, x: i32) -> Tag {
    c([
        ("name", s("text")),
        ("active", Tag::Byte(1)),
        ("autoAnchors", Tag::Byte(0)),
        (
            "box",
            c([
                ("x-anchor", s("Right")),
                ("y-anchor", s("Center")),
                ("x", Tag::Int(x)),
                ("y", Tag::Int(-12)),
            ]),
        ),
        (
            "settings",
            settings(vec![group("General", vec![setting("text", s(body))])]),
        ),
    ])
}
fn bytes(font: Option<bool>, elements: Vec<Tag>) -> Vec<u8> {
    let general = font
        .map(|v| vec![setting("custom-font", Tag::Byte(v as i8))])
        .unwrap_or_default();
    nbt::write_named(
        &NamedTag {
            name: "".into(),
            tag: c([
                ("__version__", Tag::Int(1)),
                ("active", Tag::Byte(1)),
                ("settings", settings(vec![group("General", general)])),
                ("elements", Tag::List(elements)),
            ]),
        },
        Limits::default(),
    )
    .unwrap()
}

#[test]
fn repeated_original_text_preserves_order_coordinates_defaults_and_scope() {
    let mut base = Config::default();
    base.zoom.zoom = 7.;
    base.modules.get_mut("zoom").unwrap().active = true;
    base.friends.command(&["add", "KeepFriend"]).unwrap();
    let before = serde_json::to_value(&base).unwrap();
    let got = meteor_import::hud(
        &bytes(
            Some(false),
            vec![text("FPS {fps}", -20), text("Second", -40)],
        ),
        &base,
    )
    .unwrap();
    let hud = &got.candidate.hud;
    assert_eq!(hud.version, 2);
    assert!(hud.active);
    assert!(!hud.custom_font);
    assert_eq!(hud.elements.len(), 2);
    assert_ne!(hud.elements[0].id, hud.elements[1].id);
    assert_eq!(hud.elements[0].kind, "text");
    assert_eq!(hud.elements[0].text.as_ref().unwrap().text, "FPS {fps}");
    assert_eq!(hud.elements[1].text.as_ref().unwrap().text, "Second");
    assert_eq!(hud.elements[0].source_box.unwrap().x, -20);
    assert_eq!(hud.elements[1].source_box.unwrap().x, -40);
    assert!(!hud.elements[0].auto_anchors);
    assert_eq!(hud.elements[0].text.as_ref().unwrap().update_delay, 4);
    let mut after = serde_json::to_value(&got.candidate).unwrap();
    after["hud"] = before["hud"].clone();
    assert_eq!(before, after);
    assert_eq!(before, serde_json::to_value(base).unwrap());
}

#[test]
fn omitted_or_true_original_custom_font_is_rejected_without_native_fallback() {
    let base = Config::default();
    for font in [None, Some(true)] {
        let error = meteor_import::hud(&bytes(font, vec![text("Unsupported", 0)]), &base)
            .err()
            .unwrap();
        assert!(error.contains("custom-font=false"), "{error}");
    }
}

#[test]
fn empty_layout_stays_empty_and_unsupported_kinds_are_explicit() {
    let base = Config::default();
    let got = meteor_import::hud(&bytes(Some(false), vec![]), &base).unwrap();
    assert!(got.candidate.hud.elements.is_empty());
    let round = Config::from_json(&got.candidate.to_json().unwrap()).unwrap();
    assert!(round.hud.elements.is_empty());
    let got = meteor_import::hud(
        &bytes(
            Some(false),
            vec![c([("name", s("inventory"))]), text("Supported", 0)],
        ),
        &base,
    )
    .unwrap();
    assert_eq!(got.candidate.hud.elements.len(), 1);
    assert!(got.report.skipped.iter().any(|v| v.contains("inventory")));
}

#[test]
fn malformed_and_oversized_hud_reject_atomically() {
    let base = Config::default();
    let before = base.to_json().unwrap();
    let malformed = c([
        ("name", s("text")),
        ("box", c([("x-anchor", s("Diagonal"))])),
    ]);
    assert!(meteor_import::hud(&bytes(Some(false), vec![malformed]), &base).is_err());
    let oversized = vec![text("Too many", 0); 1025];
    assert!(meteor_import::hud(&bytes(Some(false), oversized), &base).is_err());
    assert_eq!(before, base.to_json().unwrap());
}
