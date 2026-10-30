// SPDX-License-Identifier: GPL-3.0-only
// Modules/Module/Settings/Keybind/Friends/Macros NBT import. Copyright (c) Meteor Development.
//! Pure, bounded import: returns a candidate, never mutates a live client or reads files.
//! Include only with feature `meteor`. HUD, GUI expansion and unported settings are not imported.
use horizon_world::{
    compression::Compression,
    nbt::{self, Limits, Tag},
};
use meteor_client::{
    Config,
    friends::{Friend, Friends},
    registry::Keybind,
};
use serde::Serialize;
use std::collections::BTreeSet;

pub const MAX_BYTES: usize = 4 * 1024 * 1024;
#[cfg(test)]
mod macros_tests;
#[derive(Default, Debug, Serialize)]
pub struct Report {
    pub modules: Vec<String>,
    pub settings: usize,
    pub friends: usize,
    pub macros: usize,
    pub skipped: Vec<String>,
    pub notes: Vec<String>,
}
pub struct Import {
    pub candidate: Config,
    pub report: Report,
}

fn decode(bytes: &[u8]) -> Result<Tag, String> {
    if bytes.len() > MAX_BYTES {
        return Err("Meteor NBT input exceeds 4 MiB".into());
    }
    let compression = if bytes.starts_with(&[0x1f, 0x8b]) {
        Compression::Gzip
    } else {
        Compression::None
    };
    let decoded = compression
        .decode(bytes, MAX_BYTES)
        .map_err(|e| e.to_string())?;
    let (named, used) = nbt::read_named(
        &decoded,
        Limits {
            max_bytes: MAX_BYTES,
            max_nodes: 100_000,
            max_depth: 32,
        },
    )
    .map_err(|e| format!("Invalid Meteor NBT: {e:?}"))?;
    if used != decoded.len() {
        return Err("Trailing data after Meteor NBT root".into());
    }
    compound(&named.tag)?;
    Ok(named.tag)
}
fn compound(tag: &Tag) -> Result<(), String> {
    if matches!(tag, Tag::Compound(_)) {
        Ok(())
    } else {
        Err("Expected NBT compound".into())
    }
}
fn string(tag: &Tag) -> Result<String, String> {
    match tag {
        Tag::String(v) => v
            .to_string()
            .map_err(|_| "Unpaired UTF-16 in Meteor text".into()),
        _ => Err("Expected NBT string".into()),
    }
}
fn text(tag: &Tag, name: &str) -> Result<String, String> {
    string(tag.get(name).ok_or_else(|| format!("Missing {name}"))?)
}
fn list<'a>(tag: &'a Tag, name: &str, required: bool) -> Result<&'a [Tag], String> {
    match tag.get(name) {
        Some(Tag::List(v)) => Ok(v),
        None if !required => Ok(&[]),
        _ => Err(format!("Expected {name} list")),
    }
}
fn boolean(tag: &Tag, name: &str, default: bool) -> Result<bool, String> {
    match tag.get(name) {
        None => Ok(default),
        Some(Tag::Byte(v)) => Ok(*v != 0),
        _ => Err(format!("Expected {name} byte boolean")),
    }
}
fn integer(tag: &Tag, name: &str, default: i32) -> Result<i32, String> {
    match tag.get(name) {
        None => Ok(default),
        Some(Tag::Int(v)) => Ok(*v),
        _ => Err(format!("Expected {name} integer")),
    }
}

fn legacy_key(value: i32) -> Option<String> {
    let suffix = match value {
        48..=57 => (char::from_u32(value as u32)?).to_string(),
        65..=90 => (char::from_u32(value as u32)?.to_ascii_lowercase()).to_string(),
        290..=314 => format!("f{}", value - 289),
        320..=329 => format!("keypad.{}", value - 320),
        32 => "space".into(),
        39 => "apostrophe".into(),
        44 => "comma".into(),
        45 => "minus".into(),
        46 => "period".into(),
        47 => "slash".into(),
        59 => "semicolon".into(),
        61 => "equal".into(),
        91 => "left.bracket".into(),
        92 => "backslash".into(),
        93 => "right.bracket".into(),
        96 => "grave.accent".into(),
        256 => "escape".into(),
        257 => "enter".into(),
        258 => "tab".into(),
        259 => "backspace".into(),
        260 => "insert".into(),
        261 => "delete".into(),
        262 => "right".into(),
        263 => "left".into(),
        264 => "down".into(),
        265 => "up".into(),
        266 => "page.up".into(),
        267 => "page.down".into(),
        268 => "home".into(),
        269 => "end".into(),
        280 => "caps.lock".into(),
        281 => "scroll.lock".into(),
        282 => "num.lock".into(),
        283 => "print.screen".into(),
        284 => "pause".into(),
        330 => "keypad.decimal".into(),
        331 => "keypad.divide".into(),
        332 => "keypad.multiply".into(),
        333 => "keypad.subtract".into(),
        334 => "keypad.add".into(),
        335 => "keypad.enter".into(),
        336 => "keypad.equal".into(),
        340 => "left.shift".into(),
        341 => "left.control".into(),
        342 => "left.alt".into(),
        343 => "left.win".into(),
        344 => "right.shift".into(),
        345 => "right.control".into(),
        346 => "right.alt".into(),
        347 => "right.win".into(),
        348 => "menu".into(),
        _ => return None,
    };
    Some(format!("key.keyboard.{suffix}"))
}
fn keybind(tag: &Tag) -> Result<Result<Keybind, String>, String> {
    compound(tag)?;
    let (key, modifiers) = if tag.get("key").is_some() {
        let key = text(tag, "key")?;
        let mut modifiers = 0u8;
        for modifier in list(tag, "modifiers", false)? {
            modifiers |= match string(modifier)?.as_str() {
                "SHIFT" => 1,
                "CONTROL" => 2,
                "ALT" => 4,
                "SUPER" => 8,
                other => {
                    return Ok(Err(format!(
                        "modifier {other} is not supported by the native input bridge"
                    )));
                }
            };
        }
        (key, modifiers)
    } else {
        let is_key = boolean(tag, "isKey", true)?;
        let value = integer(tag, "value", -1)?;
        let mods = integer(tag, "modifiers", 0)?;
        if value == -1 {
            return Ok(Ok(Keybind::default()));
        }
        if is_key && mods & !15 != 0 {
            return Ok(Err("legacy lock/unknown modifiers are not supported".into()));
        }
        let key = if is_key {
            let Some(key) = legacy_key(value) else {
                return Ok(Err(format!("legacy GLFW key {value} is not mapped")));
            };
            key
        } else {
            match value {
                0 => "key.mouse.left".into(),
                1 => "key.mouse.right".into(),
                2 => "key.mouse.middle".into(),
                3..=7 => format!("key.mouse.{}", value + 1),
                _ => return Ok(Err(format!("legacy mouse button {value} is not supported"))),
            }
        };
        (key, if is_key { mods as u8 } else { 0 })
    };
    if key == "key.keyboard.unknown" {
        return Ok(Ok(Keybind::default()));
    }
    if crate::keybinds::key_code(&key).is_none() && crate::keybinds::mouse_button(&key).is_none() {
        return Ok(Err(format!("key {key} is not supported by native input")));
    }
    if key.starts_with("key.mouse.") && modifiers != 0 {
        return Ok(Err(
            "mouse modifiers are not supported by native input".into()
        ));
    }
    Ok(Ok(Keybind {
        key: Some(key),
        modifiers,
    }))
}

/// Modules.fromTag disables all modules, then applies each listed module. Settings.fromTag
/// resets each imported settings object first, so omitted changed-only values regain defaults.
/// Unlisted modules retain their settings but become inactive. HUD/friends remain untouched.
pub fn modules(bytes: &[u8], base: &Config) -> Result<Import, String> {
    let root = decode(bytes)?;
    let entries = list(&root, "modules", true)?;
    let mut candidate = base.clone();
    for module in &mut candidate.modules.modules {
        module.active = false;
    }
    let defaults = serde_json::to_value(Config::default()).map_err(|e| e.to_string())?;
    let mut report = Report::default();
    let mut seen = BTreeSet::new();
    for entry in entries {
        compound(entry)?;
        let name = text(entry, "name")?;
        if !seen.insert(name.clone()) {
            return Err(format!("Duplicate module {name}"));
        }
        if candidate.modules.get(&name).is_none() {
            report
                .skipped
                .push(format!("module {name}: not implemented"));
            continue;
        }
        let module = candidate.modules.get_mut(&name).unwrap();
        module.active = boolean(entry, "active", false)?;
        module.favorite = boolean(entry, "favorite", false)?;
        module.chat_feedback = boolean(entry, "chatFeedback", true)?;
        module.toggle_on_bind_release = boolean(entry, "toggleOnKeyRelease", false)?;
        module.keybind = Keybind::default();
        if let Some(bind) = entry.get("keybind") {
            match keybind(bind)? {
                Ok(bind) => module.keybind = bind,
                Err(reason) => report.skipped.push(format!("{name}.keybind: {reason}")),
            }
        }
        if let Some(settings) = entry.get("settings") {
            compound(settings)?;
            let field = name.replace('-', "_");
            let schema = defaults
                .get(&field)
                .ok_or_else(|| format!("No native settings schema for {name}"))?;
            let mut json = serde_json::to_value(&candidate).map_err(|e| e.to_string())?;
            json[&field] = schema.clone();
            candidate = serde_json::from_value(json).map_err(|e| e.to_string())?;
            let mut setting_seen = BTreeSet::new();
            for group in list(settings, "groups", false)? {
                compound(group)?;
                let group_name = text(group, "name")?;
                if group_name != "General"
                    && !(name == "safe-walk" && group_name == "Render")
                    && !(name == "air-place" && group_name == "Range")
                    && !(name == "flight" && group_name == "Anti Kick")
                {
                    report
                        .skipped
                        .push(format!("{name}/{group_name}: unknown setting group"));
                    continue;
                }
                for setting in list(group, "settings", false)? {
                    compound(setting)?;
                    let setting_name = text(setting, "name")?;
                    let native_name = if name == "flight"
                        && group_name == "Anti Kick"
                        && setting_name == "mode"
                    {
                        "anti-kick-mode"
                    } else {
                        &setting_name
                    };
                    if name == "flight"
                        && (group_name == "Anti Kick")
                            != matches!(native_name, "anti-kick-mode" | "delay" | "off-time")
                    {
                        report.skipped.push(format!(
                            "{name}/{group_name}/{setting_name}: wrong setting group"
                        ));
                        continue;
                    }
                    let field = if name == "sprint" && setting_name == "sprint-mode" {
                        "mode".into()
                    } else {
                        native_name.to_lowercase().replace('-', "_")
                    };
                    let Some(expected) = schema.get(&field) else {
                        report
                            .skipped
                            .push(format!("{name}.{setting_name}: not implemented"));
                        continue;
                    };
                    let render_setting =
                        matches!(setting_name.as_str(), "render" | "render-player-box");
                    if name == "safe-walk" && (group_name == "Render") != render_setting {
                        report
                            .skipped
                            .push(format!("{name}.{setting_name}: wrong setting group"));
                        continue;
                    }
                    if !setting_seen.insert((
                        if name == "flight" {
                            group_name.clone()
                        } else {
                            String::new()
                        },
                        setting_name.clone(),
                    )) {
                        return Err(format!("Duplicate setting {name}.{setting_name}"));
                    }
                    let value = setting
                        .get("value")
                        .ok_or_else(|| format!("Missing {name}.{setting_name}.value"))?;
                    let text = match (expected, value) {
                        (serde_json::Value::Bool(_), Tag::Byte(v)) => (*v != 0).to_string(),
                        (serde_json::Value::String(_), Tag::String(_)) => string(value)?,
                        (serde_json::Value::Number(n), Tag::Int(v)) if n.is_i64() => v.to_string(),
                        (serde_json::Value::Number(n), Tag::Double(v)) if n.is_f64() => {
                            f64::from_bits(*v).to_string()
                        }
                        (serde_json::Value::Array(_), Tag::List(values))
                            if (name == "no-mining-trace"
                                && setting_name == "blacklisted-entities")
                                || (name == "fast-use" && setting_name == "items")
                                || (name == "slippy"
                                    && matches!(
                                        setting_name.as_str(),
                                        "ignored-blocks" | "allowed-blocks"
                                    )) =>
                        {
                            let items = values.iter().map(string).collect::<Result<Vec<_>, _>>()?;
                            serde_json::to_string(&items).map_err(|e| e.to_string())?
                        }
                        (serde_json::Value::Object(_), Tag::Compound(_))
                            if name == "air-place"
                                && matches!(setting_name.as_str(), "side-color" | "line-color") =>
                        {
                            let color = meteor_client::colors::SettingColor {
                                r: integer(value, "r", 0)?.clamp(0, 255) as u8,
                                g: integer(value, "g", 0)?.clamp(0, 255) as u8,
                                b: integer(value, "b", 0)?.clamp(0, 255) as u8,
                                a: integer(value, "a", 0)?.clamp(0, 255) as u8,
                                rainbow: boolean(value, "rainbow", false)?,
                            };
                            serde_json::to_string(&color).map_err(|e| e.to_string())?
                        }
                        _ => return Err(format!("Wrong NBT value type for {name}.{setting_name}")),
                    };
                    candidate
                        .set(&name, native_name, &text)
                        .map_err(|e| format!("{name}.{setting_name}: {e}"))?;
                    report.settings += 1;
                }
            }
        }
        report.modules.push(name);
    }
    report.notes.push("GUI group expansion, HUD layout and unported modules are not imported; review skips before applying.".into());
    let candidate = Config::from_json(&candidate.to_json().map_err(|e| e.to_string())?)?;
    Ok(Import { candidate, report })
}

/// Names/UUIDs only. No profile requests or head downloads are made.
pub fn friends(bytes: &[u8], base: &Config) -> Result<Import, String> {
    let root = decode(bytes)?;
    let mut report = Report::default();
    let mut entries = Vec::new();
    for entry in list(&root, "friends", true)? {
        compound(entry)?;
        if entry.get("name").is_none() {
            report.skipped.push("friend without name".into());
            continue;
        }
        let name = text(entry, "name")?;
        let id = match entry.get("id") {
            None => None,
            Some(value) => {
                let value = string(value)?;
                if value.trim().is_empty() {
                    None
                } else {
                    let compact = value.replace('-', "");
                    if compact.len() != 32 || !compact.bytes().all(|b| b.is_ascii_hexdigit()) {
                        return Err(format!("Invalid friend UUID for {name}"));
                    }
                    Some(compact.to_ascii_lowercase())
                }
            }
        };
        entries.push(Friend { name, id });
    }
    let mut candidate = base.clone();
    let count = entries.len();
    candidate.friends = Friends::loaded(entries);
    report.friends = candidate.friends.friends.len();
    if count != report.friends {
        report
            .skipped
            .push(format!("{} duplicate friend names", count - report.friends));
    }
    report.notes.push("Imported stored names/UUIDs only; online profile refresh and head textures were not requested.".into());
    Ok(Import { candidate, report })
}

/// Original Macros.toTag -> Macro.settings -> General setting records. Replaces
/// definitions only; the Game adapter replaces live handles while preserving queues.
pub fn macros(bytes: &[u8], base: &Config) -> Result<Import, String> {
    use meteor_client::macros::{Macro, Macros};
    let root = decode(bytes)?;
    let mut report = Report::default();
    let source = list(&root, "macros", true)?;
    if source.len() > 1024 {
        return Err("Too many macros (native limit 1024)".into());
    }
    let mut entries = Vec::with_capacity(source.len());
    for (index, entry) in source.iter().enumerate() {
        compound(entry)?;
        let mut value = Macro::default();
        if let Some(settings) = entry.get("settings") {
            compound(settings)?;
            for group in list(settings, "groups", false)? {
                compound(group)?;
                let group_name = match group.get("name") {
                    Some(name) => string(name)?,
                    None => String::new(),
                };
                if group_name != "General" {
                    report.skipped.push(format!(
                        "macro[{index}]/{group_name}: unknown setting group"
                    ));
                    continue;
                }
                // SettingGroup.fromTag visits in source order: a later duplicate
                // setting replaces the earlier value. Macro names are not deduplicated.
                for setting in list(group, "settings", false)? {
                    compound(setting)?;
                    let name = match setting.get("name") {
                        Some(name) => string(name)?,
                        None => String::new(),
                    };
                    match name.as_str() {
                        "name" => {
                            value.name = match setting.get("value") {
                                Some(name) => string(name)?,
                                None => String::new(),
                            };
                        }
                        "messages" => {
                            value.messages = list(setting, "value", false)?
                                .iter()
                                .map(string)
                                .collect::<Result<_, _>>()?;
                        }
                        "keybind" => {
                            value.keybind = Keybind::default();
                            if let Some(bind) = setting.get("value") {
                                match keybind(bind)? {
                                    Ok(bind) => value.keybind = bind,
                                    Err(reason) => report
                                        .skipped
                                        .push(format!("macro[{index}].keybind: {reason}")),
                                }
                            }
                        }
                        _ => {
                            report
                                .skipped
                                .push(format!("macro[{index}].{name}: unknown setting"));
                            continue;
                        }
                    }
                    report.settings += 1;
                }
            }
        }
        entries.push(value);
    }
    let mut candidate = base.clone();
    candidate.macros = Macros { macros: entries };
    // Validate every definition and the full candidate before changing live handles.
    let candidate = Config::from_json(&candidate.to_json().map_err(|e| e.to_string())?)?;
    report.macros = candidate.macros.macros.len();
    report.notes.push("Imported macro definitions in source order, including duplicate names. Unsupported keybinds are left unset; review skips. No imported message was executed.".into());
    Ok(Import { candidate, report })
}
