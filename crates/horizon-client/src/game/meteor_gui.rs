// SPDX-License-Identifier: GPL-3.0-only
// Native port of Meteor's ModulesScreen / ModuleScreen interactions.
// Copyright (c) Meteor Development. Horizon rendering adapter is part of this port.
use super::{Game, Screen};
use horizon_client::{
    hud::{HudAtlas, HudBatch, rgba},
    keybinds,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use winit::{event::MouseButton, keyboard::KeyCode};

const ROW: f32 = 18.0;
const ACCENT: u32 = 0xffe23d91;
const WHITE: u32 = 0xffffffff;
const MUTED: u32 = 0xff969696;

#[derive(Clone)]
enum Action {
    Friends,
    #[cfg(feature = "meteor")]
    Macros,
    #[cfg(feature = "meteor")]
    Profiles,
    Hud,
    Module(String),
    Category(String),
    Back,
    Close,
    Search,
    Setting {
        module: String,
        key: String,
        value: Value,
    },
    Bind(String),
    Option {
        module: String,
        field: String,
        value: bool,
    },
}
#[derive(Clone)]
struct Hit {
    rect: [f32; 4],
    action: Action,
}
#[derive(Clone)]
struct Edit {
    module: String,
    key: String,
    text: String,
    replace_on_type: bool,
}

pub(super) struct State {
    pub(super) open: bool,
    query: String,
    query_focus: bool,
    selected: Option<String>,
    binding: Option<String>,
    edit: Option<Edit>,
    collapsed: BTreeSet<String>,
    hits: Vec<Hit>,
    scroll: f32,
    max_scroll: f32,
    error: Option<String>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            open: false,
            query: String::new(),
            query_focus: true,
            selected: None,
            binding: None,
            edit: None,
            collapsed: BTreeSet::new(),
            hits: Vec::new(),
            scroll: 0.0,
            max_scroll: 0.0,
            error: None,
        }
    }
}

fn text(batch: &mut HudBatch, atlas: &HudAtlas, s: &str, x: f32, y: f32, color: u32) {
    batch.text(atlas, s, x, y, color, false);
}
fn shorten(atlas: &HudAtlas, s: &str, width: f32) -> String {
    if HudBatch::text_width(atlas, s) <= width {
        return s.into();
    }
    let mut s = s.to_owned();
    while !s.is_empty() && HudBatch::text_width(atlas, &(s.clone() + "...")) > width {
        s.pop();
    }
    s + "..."
}
fn inside([x, y, w, h]: [f32; 4], cursor: (f32, f32)) -> bool {
    cursor.0 >= x && cursor.0 < x + w && cursor.1 >= y && cursor.1 < y + h
}
fn button(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    hits: &mut Vec<Hit>,
    cursor: (f32, f32),
    rect: [f32; 4],
    label: &str,
    active: bool,
    action: Action,
) {
    let [x, y, w, h] = rect;
    let color = if inside(rect, cursor) {
        rgba(40, 40, 40, 235)
    } else if active {
        rgba(50, 50, 50, 235)
    } else {
        rgba(20, 20, 20, 220)
    };
    batch.fill(atlas, x, y, w, h, color);
    if active {
        batch.fill(atlas, x, y, 2.0, h, ACCENT);
    }
    text(
        batch,
        atlas,
        &shorten(atlas, label, w - 10.0),
        x + 5.0,
        y + (h - 8.0) * 0.5,
        WHITE,
    );
    hits.push(Hit { rect, action });
}
fn title(key: &str) -> String {
    key.replace(['_', '-'], " ")
        .split_whitespace()
        .map(|word| {
            let mut c = word.chars();
            c.next()
                .map(|first| first.to_uppercase().collect::<String>() + c.as_str())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}
fn display(value: &Value) -> String {
    match value {
        Value::Bool(v) => if *v { "On" } else { "Off" }.into(),
        Value::String(v) => v.clone(),
        _ => value.to_string(),
    }
}
fn command_key(module: &str, key: &str) -> String {
    #[cfg(feature = "meteor")]
    {
        return meteor_client::Config::source_setting_name(module, key);
    }
    #[cfg(not(feature = "meteor"))]
    match (module, key) {
        ("zoom", "hide_hud") => "hide-HUD".into(),
        ("sprint", "mode") => "sprint-mode".into(),
        _ => key.replace('_', "-"),
    }
}
fn choices(module: &str, key: &str) -> &'static [&'static str] {
    match (module, key) {
        ("auto-jump", "mode") => &["Jump", "LowHop"],
        ("auto-jump", "jump_if") => &["Always", "Walking", "Sprinting"],
        ("sprint", "mode") => &["Strict", "Rage"],
        ("step", "active_when") => &["Always", "Sneaking", "NotSneaking"],
        ("fast-use", "mode") => &["All", "Some"],
        ("flight", "mode") => &["Abilities", "Velocity"],
        ("flight", "anti_kick_mode") => &["Normal", "Packet", "None"],
        ("auto-clicker", "mode_left" | "mode_right") => &["Disabled", "Hold", "Press"],
        ("sneak", "mode") => &["Packet", "Vanilla"],
        ("no-slow", "web") => &["Vanilla", "Timer", "None"],
        ("slippy", "list_mode") => &["Whitelist", "Blacklist"],
        ("fullbright", "mode") => &["Gamma", "Potion", "Luminance"],
        ("fullbright", "light_type") => &["BLOCK", "SKY"],
        ("air-place", "shape_mode") => &["Lines", "Sides", "Both"],
        ("gui-move", "guis") => &["GUI", "Inventory", "Both"],
        ("auto-walk", "mode") => &["Simple", "Smart"],
        ("auto-walk", "simple_direction") => &["Forwards", "Backwards", "Left", "Right"],
        _ => &[],
    }
}
pub(super) fn bind_display(bind: &Value) -> String {
    let Some(key) = bind["key"].as_str() else {
        return "None".into();
    };
    let mask = bind["modifiers"].as_u64().unwrap_or(0);
    let mut parts = Vec::new();
    for (bit, label) in [(1, "Shift"), (2, "Ctrl"), (4, "Alt"), (8, "Super")] {
        if mask & bit != 0 {
            parts.push(label);
        }
    }
    parts.push(key);
    parts.join(" + ")
}

impl Game<'_> {
    #[cfg(feature = "meteor")]
    pub(super) fn meteor_gui_status(&self) -> serde_json::Value {
        let tabs = self
            .meteor_gui
            .hits
            .iter()
            .filter_map(|hit| {
                let name = match hit.action {
                    Action::Friends => "friends",
                    Action::Macros => "macros",
                    Action::Profiles => "profiles",
                    Action::Hud => "hud",
                    _ => return None,
                };
                Some((name.to_owned(), serde_json::json!(hit.rect)))
            })
            .collect::<serde_json::Map<_, _>>();
        let settings = self
            .meteor_gui
            .hits
            .iter()
            .filter_map(|hit| {
                if let Action::Setting { module, key, .. } = &hit.action {
                    Some((format!("{module}/{key}"), serde_json::json!(hit.rect)))
                } else {
                    None
                }
            })
            .collect::<serde_json::Map<_, _>>();
        serde_json::json!({"open": self.meteor_gui.open, "hud_editor": self.meteor_hud.editor,
            "hud":self.meteor_hud_editor_status(),"tabs":tabs,"settings":settings,"macros":self.meteor_macros_gui_status(),"profiles":self.meteor_profiles_gui_status(),
            "query": self.meteor_gui.query, "query_focus": self.meteor_gui.query_focus,
            "selected": self.meteor_gui.selected, "binding": self.meteor_gui.binding,
            "editing": self.meteor_gui.edit.as_ref().map(|e| serde_json::json!({"module":e.module,"key":e.key,"text":e.text})),
            "error":self.meteor_gui.error,"cursor":self.cursor,"friends":self.meteor_friends_status()})
    }
    pub(super) fn meteor_gui_type(&mut self, text: &str) {
        #[cfg(feature = "meteor")]
        if self.meteor_macros_gui_binding() {
            return;
        }
        if self.meteor_gui.binding.is_some() {
            return;
        }
        if text == "\\b" {
            self.meteor_gui_key(KeyCode::Backspace, None, true, false);
        } else {
            self.meteor_gui_key(KeyCode::F24, Some(text), true, false);
        }
    }
    #[cfg(feature = "meteor")]
    pub(super) fn meteor_gui_wants_text(&self) -> bool {
        if !self.meteor_gui.open {
            return false;
        }
        if self.meteor_hud.editor {
            return self.meteor_hud.controls.panel && self.meteor_hud.controls.edit.is_some();
        }
        if self.meteor_friends.open {
            return self.meteor_friends_wants_text();
        }
        if self.meteor_macros_gui.open {
            return self.meteor_macros_gui_wants_text();
        }
        if self.meteor_profiles_gui.open {
            return self.meteor_profiles_gui_wants_text();
        }
        (self.meteor_gui.query_focus && self.meteor_gui.selected.is_none())
            || self.meteor_gui.edit.is_some()
            || self.meteor_gui.binding.is_some()
    }
    fn meteor_gui_close(&mut self) {
        #[cfg(feature = "meteor")]
        {
            self.meteor_macros_gui.open = false;
            self.meteor_profiles_gui.open = false;
        }
        self.meteor_friends.open = false;
        self.meteor_hud_editor_close();
        self.meteor_gui.open = false;
        self.meteor_gui.binding = None;
        self.meteor_gui.edit = None;
        self.meteor_gui.hits.clear();
        self.meteor_release_input();
        self.keys.clear();
        self.screen = Screen::Playing;
        self.capture(true);
    }
    fn meteor_gui_command(&mut self, command: String) {
        let mut claimed = false;
        self.meteor_command(&command, &mut claimed);
    }
    fn meteor_gui_set(&mut self, module: &str, key: &str, value: &str) {
        if let Some((field, component)) = key.split_once('.') {
            let mut color = self.meteor_status()["config"][module.replace('-', "_")][field].clone();
            if color.is_object() && matches!(component, "r" | "g" | "b" | "a" | "rainbow") {
                let candidate = if component == "rainbow" {
                    value
                        .parse::<bool>()
                        .map(Value::Bool)
                        .map_err(|_| "Rainbow must be on or off".to_owned())
                } else {
                    value.parse::<u8>().map(|v| Value::from(v)).map_err(|_| {
                        "Color channels must be whole numbers from 0 to 255".to_owned()
                    })
                };
                match candidate {
                    Ok(value) => {
                        color[component] = value;
                        self.meteor_gui.error = self
                            .meteor_gui_set_setting(
                                module,
                                &command_key(module, field),
                                &color.to_string(),
                            )
                            .err();
                    }
                    Err(error) => self.meteor_gui.error = Some(error),
                }
                return;
            }
        }
        self.meteor_gui.error = self
            .meteor_gui_set_setting(module, &command_key(module, key), value)
            .err();
    }
    /// Called before the normal screen consumes a keyboard event. Returns whether consumed.
    pub(super) fn meteor_gui_key(
        &mut self,
        key: KeyCode,
        input: Option<&str>,
        pressed: bool,
        repeat: bool,
    ) -> bool {
        let binding = self.meteor_gui.binding.is_some();
        #[cfg(feature = "meteor")]
        let binding = binding || self.meteor_macros_gui_binding();
        if pressed && !repeat && key == KeyCode::ShiftRight && !binding {
            if self.meteor_gui.open {
                self.meteor_gui_close();
                return true;
            }
            if self.native_playing()
                && self.player.is_some()
                && self.meteor_status()["enabled"].as_bool().unwrap_or(false)
            {
                self.meteor_release_input();
                self.meteor_gui = State {
                    open: true,
                    ..State::default()
                };
                self.screen = Screen::Options;
                self.keys.clear();
                self.left = false;
                self.right = false;
                self.mining = None;
                self.capture(false);
                return true;
            }
        }
        if !self.meteor_gui.open {
            return false;
        }
        if self.meteor_friends_key(key, input, pressed) {
            return true;
        }
        #[cfg(feature = "meteor")]
        if self.meteor_macros_gui_key(key, input, pressed)
            || self.meteor_profiles_gui_key(key, input, pressed)
        {
            return true;
        }
        if !pressed {
            return true;
        }
        if self.meteor_hud.editor {
            return self.meteor_hud_control_key(key, input);
        }
        if let Some(module) = self.meteor_gui.binding.clone() {
            if repeat {
                return true;
            }
            if key == KeyCode::Escape {
                self.meteor_gui.binding = None;
                return true;
            }
            if matches!(
                key,
                KeyCode::ShiftLeft
                    | KeyCode::ShiftRight
                    | KeyCode::ControlLeft
                    | KeyCode::ControlRight
                    | KeyCode::AltLeft
                    | KeyCode::AltRight
                    | KeyCode::SuperLeft
                    | KeyCode::SuperRight
            ) {
                return true;
            }
            let name = if matches!(key, KeyCode::Delete | KeyCode::Backspace) {
                Some("none")
            } else {
                keybinds::key_name(key)
            };
            if let Some(name) = name {
                let mut modifiers = 0;
                for (bit, left, right) in [
                    (1, KeyCode::ShiftLeft, KeyCode::ShiftRight),
                    (2, KeyCode::ControlLeft, KeyCode::ControlRight),
                    (4, KeyCode::AltLeft, KeyCode::AltRight),
                    (8, KeyCode::SuperLeft, KeyCode::SuperRight),
                ] {
                    if self.physical_keys.contains(&left) || self.physical_keys.contains(&right) {
                        modifiers |= bit;
                    }
                }
                self.meteor_gui.error = self
                    .meteor_gui_bind(&module, (name != "none").then_some(name), modifiers)
                    .err();
                if self.meteor_gui.error.is_none() {
                    self.meteor_gui.binding = None;
                }
            }
            return true;
        }
        if self.meteor_gui.edit.is_some() {
            let ctrl = self.physical_keys.contains(&KeyCode::ControlLeft)
                || self.physical_keys.contains(&KeyCode::ControlRight);
            if ctrl && key == KeyCode::KeyA {
                if let Some(edit) = &mut self.meteor_gui.edit {
                    edit.replace_on_type = true;
                }
                return true;
            }
            match key {
                KeyCode::Escape => {
                    self.meteor_gui.edit = None;
                    self.meteor_gui.error = None;
                }
                KeyCode::Enter | KeyCode::NumpadEnter => {
                    if let Some(edit) = self.meteor_gui.edit.clone() {
                        self.meteor_gui_set(&edit.module, &edit.key, edit.text.trim());
                        if self.meteor_gui.error.is_none() {
                            self.meteor_gui.edit = None;
                        }
                    }
                }
                KeyCode::Backspace => {
                    if let Some(edit) = &mut self.meteor_gui.edit {
                        if edit.replace_on_type {
                            edit.text.clear();
                            edit.replace_on_type = false;
                        } else {
                            edit.text.pop();
                        }
                    }
                }
                _ => {
                    if let (Some(edit), Some(input)) = (&mut self.meteor_gui.edit, input) {
                        if ctrl {
                            return true;
                        }
                        let incoming: String = input.chars().filter(|c| !c.is_control()).collect();
                        let limit = if (edit.module == "fast-use" && edit.key == "items")
                            || (edit.module == "slippy"
                                && matches!(edit.key.as_str(), "allowed_blocks" | "ignored_blocks"))
                        {
                            4096
                        } else {
                            128
                        };
                        let previous = if edit.replace_on_type {
                            0
                        } else {
                            edit.text.chars().count()
                        };
                        if previous + incoming.chars().count() > limit {
                            self.meteor_gui.error = Some(format!(
                                "Input limit: {limit} characters. Existing text kept."
                            ));
                        } else if !incoming.is_empty() {
                            if edit.replace_on_type {
                                edit.text.clear();
                                edit.replace_on_type = false;
                            }
                            edit.text.push_str(&incoming);
                            self.meteor_gui.error = None;
                        }
                    }
                }
            }
            return true;
        }
        if key == KeyCode::Escape {
            if self.meteor_gui.selected.take().is_some() {
                self.meteor_gui.scroll = 0.0;
            } else {
                self.meteor_gui_close();
            }
            return true;
        }
        let ctrl = self.physical_keys.contains(&KeyCode::ControlLeft)
            || self.physical_keys.contains(&KeyCode::ControlRight);
        if ctrl && key == KeyCode::KeyF {
            self.meteor_gui.selected = None;
            self.meteor_gui.query_focus = true;
            self.meteor_gui.scroll = 0.0;
            return true;
        }
        if self.meteor_gui.query_focus && self.meteor_gui.selected.is_none() {
            if key == KeyCode::Backspace {
                self.meteor_gui.query.pop();
            } else if key == KeyCode::Enter {
                self.meteor_gui.query_focus = false;
            } else if !ctrl && let Some(input) = input {
                for c in input.chars().filter(|c| !c.is_control()) {
                    if self.meteor_gui.query.len() < 128 {
                        self.meteor_gui.query.push(c);
                    }
                }
            }
            self.meteor_gui.scroll = 0.0;
        }
        true
    }
    pub(super) fn meteor_gui_mouse(&mut self, button: MouseButton, pressed: bool) -> bool {
        if !self.meteor_gui.open {
            return false;
        }
        if self.meteor_friends_mouse(button, pressed) {
            return true;
        }
        #[cfg(feature = "meteor")]
        if self.meteor_macros_gui_mouse(button, pressed)
            || self.meteor_profiles_gui_mouse(button, pressed)
        {
            return true;
        }
        if self.meteor_hud.editor {
            return self.meteor_hud_mouse(button, pressed);
        }
        if !pressed {
            return true;
        }
        if let Some(module) = self.meteor_gui.binding.take() {
            self.meteor_gui_command(format!(
                "meteor bind {module} {}",
                keybinds::mouse_name(button)
            ));
            return true;
        }
        if !matches!(button, MouseButton::Left | MouseButton::Right) {
            return true;
        }
        let hit = self
            .meteor_gui
            .hits
            .iter()
            .rev()
            .find(|h| inside(h.rect, self.cursor))
            .cloned();
        self.meteor_gui.query_focus = false;
        self.meteor_gui.error = None;
        match hit.map(|h| h.action) {
            #[cfg(feature = "meteor")]
            Some(Action::Macros) => self.meteor_macros_gui_open(),
            #[cfg(feature = "meteor")]
            Some(Action::Profiles) => self.meteor_profiles_gui_open(),
            Some(Action::Friends) => {
                self.meteor_gui.query_focus = false;
                self.meteor_friends_open();
            }
            Some(Action::Hud) => {
                self.meteor_gui.query_focus = false;
                self.meteor_hud_editor_open();
            }
            Some(Action::Close) => self.meteor_gui_close(),
            Some(Action::Back) => {
                self.meteor_gui.selected = None;
                self.meteor_gui.edit = None;
                self.meteor_gui.scroll = 0.0;
            }
            Some(Action::Search) => {
                self.meteor_gui.query_focus = true;
                self.meteor_gui.edit = None;
            }
            Some(Action::Category(category)) => {
                if !self.meteor_gui.collapsed.remove(&category) {
                    self.meteor_gui.collapsed.insert(category);
                }
            }
            Some(Action::Module(id)) => {
                if button == MouseButton::Right {
                    self.meteor_gui.selected = Some(id);
                    self.meteor_gui.scroll = 0.0;
                } else if button == MouseButton::Left {
                    self.meteor_gui_command(format!("meteor toggle {id}"));
                }
            }
            Some(Action::Bind(module)) => {
                self.meteor_gui.binding = Some(module);
                self.meteor_gui.edit = None;
            }
            Some(Action::Option {
                module,
                field,
                value,
            }) => {
                self.meteor_gui_module_option(&module, &field, !value);
            }
            Some(Action::Setting { module, key, value }) => {
                if let Some(value) = value.as_bool() {
                    self.meteor_gui_set(&module, &key, if value { "false" } else { "true" });
                } else if let Some(current) = value.as_str()
                    && !choices(&module, &key).is_empty()
                {
                    let choices = choices(&module, &key);
                    let index = choices.iter().position(|s| *s == current).unwrap_or(0);
                    let next = if button == MouseButton::Right {
                        (index + choices.len() - 1) % choices.len()
                    } else {
                        (index + 1) % choices.len()
                    };
                    self.meteor_gui_set(&module, &key, choices[next]);
                } else if value.is_number()
                    || value.is_string()
                    || value
                        .as_array()
                        .is_some_and(|values| values.iter().all(Value::is_string))
                {
                    self.meteor_gui.edit = Some(Edit {
                        module,
                        key,
                        text: display(&value),
                        replace_on_type: true,
                    });
                }
            }
            None => {}
        }
        true
    }
    pub(super) fn meteor_gui_wheel(&mut self, steps: f32) -> bool {
        if self.meteor_hud_control_wheel(steps) {
            return true;
        }
        if self.meteor_friends_wheel(steps) {
            return true;
        }
        #[cfg(feature = "meteor")]
        if self.meteor_macros_gui_wheel(steps) || self.meteor_profiles_gui_wheel(steps) {
            return true;
        }
        if !self.meteor_gui.open {
            return false;
        }
        self.meteor_gui.scroll =
            (self.meteor_gui.scroll - steps * ROW * 3.0).clamp(0.0, self.meteor_gui.max_scroll);
        true
    }
    pub(super) fn draw_meteor_gui(&mut self, batch: &mut HudBatch) {
        if !self.meteor_gui.open {
            return;
        }
        if self.meteor_friends.open {
            self.draw_meteor_friends(batch);
            return;
        }
        #[cfg(feature = "meteor")]
        {
            if self.meteor_macros_gui.open {
                self.draw_meteor_macros_gui(batch);
                return;
            }
            if self.meteor_profiles_gui.open {
                self.draw_meteor_profiles_gui(batch);
                return;
            }
        }
        if self.meteor_hud.editor {
            self.draw_meteor_hud(batch);
            return;
        }
        let snapshot = self.meteor_status();
        let config = &snapshot["config"];
        let modules = config["modules"]["modules"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let atlas = &self.gui.atlas;
        let w = batch.width as f32;
        let h = batch.height as f32;
        let mut hits = Vec::new();
        batch.fill(atlas, 0.0, 0.0, w, h, rgba(0, 0, 0, 110));
        batch.fill(atlas, 0.0, 0.0, w, 27.0, rgba(20, 20, 20, 235));
        text(batch, atlas, "Meteor", 9.0, 9.0, ACCENT);
        text(batch, atlas, "Modules", 58.0, 9.0, WHITE);
        button(
            batch,
            atlas,
            &mut hits,
            self.cursor,
            [165., 4., 60., 19.],
            "Friends",
            false,
            Action::Friends,
        );
        button(
            batch,
            atlas,
            &mut hits,
            self.cursor,
            [115.0, 4.0, 45.0, 19.0],
            "HUD",
            false,
            Action::Hud,
        );
        button(
            batch,
            atlas,
            &mut hits,
            self.cursor,
            [w - 25.0, 4.0, 20.0, 19.0],
            "X",
            false,
            Action::Close,
        );
        #[cfg(feature = "meteor")]
        let top = {
            let narrow = w < 390.0;
            let y = if narrow { 30.0 } else { 4.0 };
            let x = if narrow { 9.0 } else { 229.0 };
            if narrow {
                batch.fill(atlas, 0., 27., w, 27., rgba(20, 20, 20, 235));
            }
            button(
                batch,
                atlas,
                &mut hits,
                self.cursor,
                [x, y, 58., 19.],
                "Macros",
                false,
                Action::Macros,
            );
            button(
                batch,
                atlas,
                &mut hits,
                self.cursor,
                [x + 64., y, 62., 19.],
                "Profiles",
                false,
                Action::Profiles,
            );
            if narrow { 62.0 } else { 35.0 }
        };
        #[cfg(not(feature = "meteor"))]
        let top = 35.0;
        let bottom = (h - 27.0).max(top + ROW);
        let mut content_bottom = top;
        if let Some(id) = self.meteor_gui.selected.clone() {
            if let Some(module) = modules.iter().find(|m| m["id"].as_str() == Some(&id)) {
                let width = (w - 24.0).min(340.0);
                let x = (w - width) * 0.5;
                let title = module["title"].as_str().unwrap_or(&id);
                button(
                    batch,
                    atlas,
                    &mut hits,
                    self.cursor,
                    [x, top, width, ROW],
                    &format!("<  {title}"),
                    true,
                    Action::Back,
                );
                let mut rows = vec![
                    (
                        format!("Enabled: {}", display(&module["active"])),
                        Action::Module(id.clone()),
                    ),
                    (
                        format!("Keybind: {}", bind_display(&module["keybind"])),
                        Action::Bind(id.clone()),
                    ),
                ];
                for (field, label) in [
                    ("favorite", "Favorite"),
                    ("toggle_on_bind_release", "Toggle on bind release"),
                    ("chat_feedback", "Chat feedback"),
                ] {
                    let value = module[field].as_bool().unwrap_or(false);
                    rows.push((
                        format!("{label}: {}", if value { "On" } else { "Off" }),
                        Action::Option {
                            module: id.clone(),
                            field: field.into(),
                            value,
                        },
                    ));
                }
                if let Some(settings) = config[id.replace('-', "_")].as_object() {
                    for (key, value) in settings {
                        if value.is_object()
                            && ["r", "g", "b", "a"]
                                .iter()
                                .all(|channel| value[*channel].as_u64().is_some_and(|v| v <= 255))
                            && value["rainbow"].is_boolean()
                        {
                            for (channel, name) in [
                                ("r", "Red"),
                                ("g", "Green"),
                                ("b", "Blue"),
                                ("a", "Opacity"),
                                ("rainbow", "Rainbow"),
                            ] {
                                let field = format!("{key}.{channel}");
                                let shown = if let Some(edit) = &self.meteor_gui.edit
                                    && edit.module == id
                                    && edit.key == field
                                {
                                    format!("{}_", edit.text)
                                } else {
                                    display(&value[channel])
                                };
                                rows.push((
                                    format!("{} / {name}: {shown}", title_case(key)),
                                    Action::Setting {
                                        module: id.clone(),
                                        key: field,
                                        value: value[channel].clone(),
                                    },
                                ));
                            }
                            continue;
                        }
                        if value.is_boolean()
                            || value.is_number()
                            || value.is_string()
                            || value
                                .as_array()
                                .is_some_and(|values| values.iter().all(Value::is_string))
                        {
                            let label = if let Some(edit) = &self.meteor_gui.edit
                                && edit.module == id
                                && edit.key == *key
                            {
                                format!("{}: {}_", title_case(key), edit.text)
                            } else {
                                format!("{}: {}", title_case(key), display(value))
                            };
                            rows.push((
                                label,
                                Action::Setting {
                                    module: id.clone(),
                                    key: key.clone(),
                                    value: value.clone(),
                                },
                            ));
                        }
                    }
                }
                if id == "air-place" {
                    let shown = if let Some(edit) = &self.meteor_gui.edit
                        && edit.module == "config"
                        && edit.key == "rainbow_speed"
                    {
                        format!("{}_", edit.text)
                    } else {
                        display(&config["rainbow_speed"])
                    };
                    rows.push((
                        format!("Rainbow speed (global): {shown}"),
                        Action::Setting {
                            module: "config".into(),
                            key: "rainbow_speed".into(),
                            value: config["rainbow_speed"].clone(),
                        },
                    ));
                }
                let start = top + ROW;
                for (index, (label, action)) in rows.iter().enumerate() {
                    let y = start + index as f32 * ROW - self.meteor_gui.scroll;
                    if y >= start && y + ROW <= bottom {
                        let color = if let Action::Setting { key, .. } = action {
                            key.strip_suffix(".r")
                                .map(|field| &config[id.replace('-', "_")][field])
                        } else {
                            None
                        };
                        let label = if color.is_some() {
                            shorten(atlas, label, width - 30.)
                        } else {
                            label.clone()
                        };
                        button(
                            batch,
                            atlas,
                            &mut hits,
                            self.cursor,
                            [x, y, width, ROW],
                            &label,
                            false,
                            action.clone(),
                        );
                        if let Some(color) = color {
                            let sx = x + width - 19.;
                            let sy = y + 3.;
                            batch.fill(atlas, sx, sy, 12., 12., rgba(180, 180, 180, 255));
                            batch.fill(atlas, sx, sy, 6., 6., rgba(80, 80, 80, 255));
                            batch.fill(atlas, sx + 6., sy + 6., 6., 6., rgba(80, 80, 80, 255));
                            batch.fill(
                                atlas,
                                sx,
                                sy,
                                12.,
                                12.,
                                rgba(
                                    color["r"].as_u64().unwrap_or(0) as u8,
                                    color["g"].as_u64().unwrap_or(0) as u8,
                                    color["b"].as_u64().unwrap_or(0) as u8,
                                    color["a"].as_u64().unwrap_or(255) as u8,
                                ),
                            );
                        }
                    }
                }
                content_bottom = start + rows.len() as f32 * ROW;
                if id == "reach" {
                    let note = "Servers decide which actions are accepted. Extra client targeting does not guarantee attacks or placements. Vanilla may allow up to 4 additional blocks for block entities or vehicles; this does not work on Paper servers.";
                    let mut lines = vec![];
                    let mut line = String::new();
                    for word in note.split_whitespace() {
                        let candidate = if line.is_empty() {
                            word.to_owned()
                        } else {
                            format!("{line} {word}")
                        };
                        if !line.is_empty() && HudBatch::text_width(atlas, &candidate) > width - 10.
                        {
                            lines.push(std::mem::take(&mut line));
                            line = word.to_owned();
                        } else {
                            line = candidate;
                        }
                    }
                    if !line.is_empty() {
                        lines.push(line);
                    }
                    for (index, line) in lines.iter().enumerate() {
                        let y = content_bottom + 5. + index as f32 * 12. - self.meteor_gui.scroll;
                        if y >= start && y + 9. <= bottom {
                            text(batch, atlas, line, x + 5., y, MUTED);
                        }
                    }
                    content_bottom += lines.len() as f32 * 12. + 10.;
                }
            }
        } else {
            let search = if self.meteor_gui.query.is_empty() {
                "Search modules or settings...".into()
            } else {
                self.meteor_gui.query.clone() + if self.meteor_gui.query_focus { "_" } else { "" }
            };
            button(
                batch,
                atlas,
                &mut hits,
                self.cursor,
                [9.0, top, w - 18.0, ROW],
                &search,
                self.meteor_gui.query_focus,
                Action::Search,
            );
            let query = self.meteor_gui.query.to_lowercase();
            let mut categories: BTreeMap<String, Vec<Value>> = BTreeMap::new();
            for module in &modules {
                let id = module["id"].as_str().unwrap_or("");
                let searchable = format!(
                    "{} {} {}",
                    module["title"].as_str().unwrap_or(""),
                    id,
                    config[id.replace('-', "_")]
                )
                .to_lowercase();
                if !query.is_empty() && !searchable.contains(&query) {
                    continue;
                }
                categories
                    .entry(module["category"].as_str().unwrap_or("Misc").into())
                    .or_default()
                    .push(module.clone());
                if module["favorite"].as_bool().unwrap_or(false) {
                    categories
                        .entry("Favorites".into())
                        .or_default()
                        .push(module.clone());
                }
            }
            let columns = (((w - 18.0) / 135.0).floor() as usize).max(1);
            let width = ((w - 18.0 - (columns - 1) as f32 * 8.0) / columns as f32).min(180.0);
            let base = top + ROW + 9.0;
            let mut group_y = base;
            let mut group_height = 0.0;
            for (index, (category, mut list)) in categories.into_iter().enumerate() {
                if index > 0 && index % columns == 0 {
                    group_y += group_height + 9.0;
                    group_height = 0.0;
                }
                let x = 9.0 + (index % columns) as f32 * (width + 8.0);
                let collapsed = self.meteor_gui.collapsed.contains(&category);
                let header_y = group_y - self.meteor_gui.scroll;
                if header_y >= base && header_y + ROW <= bottom {
                    button(
                        batch,
                        atlas,
                        &mut hits,
                        self.cursor,
                        [x, header_y, width, ROW],
                        &format!("{} {category}", if collapsed { "+" } else { "-" }),
                        true,
                        Action::Category(category.clone()),
                    );
                }
                list.sort_by_key(|m| m["title"].as_str().unwrap_or("").to_lowercase());
                if !collapsed {
                    for (row, module) in list.iter().enumerate() {
                        let y = header_y + (row + 1) as f32 * ROW;
                        if y >= base && y + ROW <= bottom {
                            button(
                                batch,
                                atlas,
                                &mut hits,
                                self.cursor,
                                [x, y, width, ROW],
                                module["title"].as_str().unwrap_or("Module"),
                                module["active"].as_bool().unwrap_or(false),
                                Action::Module(module["id"].as_str().unwrap_or("").into()),
                            );
                        }
                    }
                }
                group_height = group_height.max(
                    ROW * if collapsed {
                        1.0
                    } else {
                        list.len() as f32 + 1.0
                    },
                );
                content_bottom = content_bottom.max(group_y + group_height);
            }
            if modules.is_empty() {
                text(
                    batch,
                    atlas,
                    "No native modules available.",
                    12.0,
                    base,
                    MUTED,
                );
            }
        }
        self.meteor_gui.max_scroll = (content_bottom - bottom).max(0.0);
        self.meteor_gui.scroll = self.meteor_gui.scroll.min(self.meteor_gui.max_scroll);
        let hint = if self.meteor_gui.binding.is_some() {
            "Press a key or mouse button. Delete clears; Esc cancels."
        } else if self.meteor_gui.edit.is_some() {
            "Enter saves the value. Esc cancels."
        } else if self.meteor_gui.selected.is_some() {
            "Click a value to edit. Right Shift closes."
        } else {
            "Left click: toggle  |  Right click: settings  |  Ctrl+F: search"
        };
        text(
            batch,
            atlas,
            &shorten(atlas, hint, w - 18.0),
            9.0,
            h - 18.0,
            MUTED,
        );
        if let Some(error) = &self.meteor_gui.error {
            batch.fill(atlas, 4.0, h - 34.0, w - 8.0, 14.0, rgba(20, 20, 20, 245));
            text(
                batch,
                atlas,
                &shorten(atlas, error, w - 18.0),
                9.0,
                h - 31.0,
                rgba(255, 100, 100, 255),
            );
        }
        self.meteor_gui.hits = hits;
    }
}

fn title_case(key: &str) -> String {
    title(key)
}
