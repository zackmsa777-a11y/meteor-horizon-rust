// SPDX-License-Identifier: GPL-3.0-only
use macros::{Compiled, Macro, Macros, Schedule};
use meteor_client::macros;
use meteor_client::registry;
fn named(name: &str, mask: u8) -> Macro {
    Macro {
        name: name.into(),
        messages: vec!["hello".into()],
        keybind: registry::Keybind {
            key: Some("key.keyboard.h".into()),
            modifiers: mask,
        },
    }
}
#[test]
fn storage_bounds_preserve_source_editor_distinction() {
    let empty = Macros {
        macros: vec![Macro::default()],
    };
    assert!(empty.validate_storage().is_ok());
    assert!(empty.macros[0].validate_editor().is_err());
    let mut bad = Macros {
        macros: vec![named("valid", 16)],
    };
    assert!(bad.validate_storage().is_err());
    bad.macros[0].keybind.modifiers = 0;
    bad.macros[0].messages = vec!["x".repeat(65537)];
    assert!(bad.validate_storage().is_err());
}
#[test]
fn input_preserves_first_match_repeats_and_screen_release_gates() {
    let mut list = Macros::default();
    list.add_from_editor(named("plain", 0)).unwrap();
    list.add_from_editor(named("modified", 2)).unwrap();
    let mut event = registry::KeyEvent {
        key: "key.keyboard.h".into(),
        modifiers: 2,
        pressed: true,
        repeat: true,
        screen_open: false,
        debug_key_down: true,
    };
    assert_eq!(list.input(&event).unwrap().name, "plain");
    event.screen_open = true;
    assert!(list.input(&event).is_none());
    event.screen_open = false;
    event.pressed = false;
    assert!(list.input(&event).is_none());
}
#[test]
fn editor_equality_and_lookup_follow_different_source_case_rules() {
    let mut list = Macros::default();
    list.add_from_editor(named("Test", 0)).unwrap();
    assert!(list.add_from_editor(named("Test", 2)).is_err());
    list.add_from_editor(named("test", 0)).unwrap();
    assert_eq!(list.get("TEST").unwrap().name, "Test");
    list.edit_from_editor(1, named("Test", 0)).unwrap();
    assert!(list.remove(&named("Test", 2)));
    assert_eq!(list.macros.len(), 1);
    assert!(named(" \t", 0).validate_editor().is_err());
    assert!(named("\u{00a0}", 0).validate_editor().is_ok());
    let restored: Macros = serde_json::from_str(&serde_json::to_string(&list).unwrap()).unwrap();
    assert_eq!(restored.macros[0], list.macros[0]);
}
#[test]
fn compiler_cache_skips_bad_lines_and_recompiles_only_after_messages_change() {
    let mut cache = Compiled::default();
    let lines = vec!["ok".into(), "broken".into(), "runtime".into()];
    let mut calls = 0;
    let mut compile = |s: &str| {
        calls += 1;
        if s == "broken" {
            Err("compile error".into())
        } else {
            Ok(s.to_owned())
        }
    };
    let run = |s: &String| {
        if s == "runtime" {
            Err("runtime error".into())
        } else {
            Ok(s.clone())
        }
    };
    let first = cache.evaluate(&lines, &mut compile, run);
    assert_eq!(first.messages, ["ok"]);
    assert_eq!(first.errors.len(), 2);
    let second = cache.evaluate(&lines, &mut compile, run);
    assert_eq!(second.errors.len(), 1);
    cache.messages_changed();
    cache.evaluate(&lines, &mut compile, run);
    assert_eq!(calls, 6);
}
#[test]
fn scheduler_defers_zero_until_post_and_counts_delay_after_dispatch() {
    let mut schedule = Schedule::default();
    schedule.queue(0, 1);
    schedule.queue(1, 2);
    schedule.queue(2, 3);
    assert!(schedule.contains(|v| *v == 1));
    assert_eq!(schedule.post_tick(true), [1]);
    assert_eq!(schedule.post_tick(true), [2]);
    schedule.clear(|v| *v == 3);
    assert!(schedule.post_tick(true).is_empty());
    schedule.queue(0, 4);
    assert!(schedule.post_tick(false).is_empty());
    assert!(!schedule.contains(|v| *v == 4));
    schedule.queue(100, 5);
    schedule.clear_all();
    assert!(!schedule.contains(|_| true));
}
#[test]
fn schedule_handles_retain_live_object_identity_after_list_removal() {
    use std::{cell::RefCell, rc::Rc};
    let original = Rc::new(RefCell::new(named("editable", 0)));
    let distinct = Rc::new(RefCell::new(named("editable", 0)));
    let mut schedule = Schedule::default();
    schedule.queue(1, original.clone());
    schedule.clear(|m| Rc::ptr_eq(m, &distinct));
    assert!(schedule.post_tick(true).is_empty());
    original.borrow_mut().messages[0] = "changed".into();
    let due = schedule.post_tick(true);
    assert_eq!(due[0].borrow().messages, ["changed"]);
}
