// SPDX-License-Identifier: GPL-3.0-only
// Native MacrosTab/EditMacroScreen. Copyright (c) Meteor Development.
use super::Game;
use horizon_client::{
    hud::{HudAtlas, HudBatch, rgba},
    keybinds,
};
use meteor_client::{macros::Macro, registry::Keybind};
use winit::{event::MouseButton, keyboard::KeyCode};
#[derive(Default)]
pub(super) struct State {
    pub(super) open: bool,
    edit: Option<Edit>,
    rows: Vec<Macro>,
    hits: Vec<([f32; 4], Action)>,
    scroll: usize,
    max_scroll: usize,
    confirmation: Option<usize>,
    error: Option<String>,
}
struct Edit {
    index: Option<usize>,
    value: Macro,
    focus: Option<usize>,
    replace: bool,
    binding: bool,
    line_scroll: usize,
}
#[derive(Clone)]
enum Action {
    Back,
    Create,
    Edit(usize),
    Remove(usize),
    Name,
    Bind,
    Line(usize),
    AddLine,
    RemoveLine,
    Save,
    Cancel,
}
fn inside(r: [f32; 4], p: (f32, f32)) -> bool {
    p.0 >= r[0] && p.0 < r[0] + r[2] && p.1 >= r[1] && p.1 < r[1] + r[3]
}
fn clipped(atlas: &HudAtlas, text: &str, width: f32) -> String {
    let mut s = text.to_owned();
    if HudBatch::text_width(atlas, &s) <= width {
        return s;
    }
    while !s.is_empty() && HudBatch::text_width(atlas, &(s.clone() + "...")) > width {
        s.pop();
    }
    s + "..."
}
impl Game<'_> {
    pub(super) fn meteor_macros_gui_open(&mut self) {
        let rows = self.meteor_macros_rows();
        self.meteor_macros_gui = State {
            open: true,
            rows,
            ..Default::default()
        };
    }
    pub(super) fn meteor_macros_gui_wants_text(&self) -> bool {
        self.meteor_macros_gui.open
            && self
                .meteor_macros_gui
                .edit
                .as_ref()
                .is_some_and(|e| e.focus.is_some() || e.binding)
    }
    pub(super) fn meteor_macros_gui_binding(&self) -> bool {
        self.meteor_macros_gui.open
            && self
                .meteor_macros_gui
                .edit
                .as_ref()
                .is_some_and(|e| e.binding)
    }
    pub(super) fn meteor_macros_gui_status(&self) -> serde_json::Value {
        let targets = self
            .meteor_macros_gui
            .hits
            .iter()
            .map(|(rect, action)| {
                let name = match action {
                    Action::Back => "back".into(),
                    Action::Create => "create".into(),
                    Action::Edit(i) => format!("edit/{i}"),
                    Action::Remove(i) => format!("remove/{i}"),
                    Action::Name => "name".into(),
                    Action::Bind => "bind".into(),
                    Action::Line(i) => format!("line/{i}"),
                    Action::AddLine => "add-line".into(),
                    Action::RemoveLine => "remove-line".into(),
                    Action::Save => "save".into(),
                    Action::Cancel => "cancel".into(),
                };
                (name, serde_json::json!(rect))
            })
            .collect::<serde_json::Map<_, _>>();
        serde_json::json!({"open":self.meteor_macros_gui.open,"targets":targets,"error":self.meteor_macros_gui.error,"scroll":self.meteor_macros_gui.scroll,"editing":self.meteor_macros_gui.edit.as_ref().map(|e|serde_json::json!({"index":e.index,"draft":e.value,"focus":e.focus,"binding":e.binding}))})
    }
    pub(super) fn meteor_macros_gui_key(
        &mut self,
        key: KeyCode,
        input: Option<&str>,
        pressed: bool,
    ) -> bool {
        if !self.meteor_macros_gui.open {
            return false;
        }
        if !pressed {
            return true;
        }
        let ctrl = self.physical_keys.contains(&KeyCode::ControlLeft)
            || self.physical_keys.contains(&KeyCode::ControlRight);
        if let Some(edit) = self.meteor_macros_gui.edit.as_mut() {
            if edit.binding {
                if key == KeyCode::Escape {
                    edit.binding = false;
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
                if matches!(key, KeyCode::Delete | KeyCode::Backspace) {
                    edit.value.keybind = Keybind::default();
                    edit.binding = false;
                    return true;
                }
                if let Some(name) = keybinds::key_name(key) {
                    let mut modifiers = 0;
                    for (bit, left, right) in [
                        (1, KeyCode::ShiftLeft, KeyCode::ShiftRight),
                        (2, KeyCode::ControlLeft, KeyCode::ControlRight),
                        (4, KeyCode::AltLeft, KeyCode::AltRight),
                        (8, KeyCode::SuperLeft, KeyCode::SuperRight),
                    ] {
                        if self.physical_keys.contains(&left) || self.physical_keys.contains(&right)
                        {
                            modifiers |= bit;
                        }
                    }
                    edit.value.keybind = Keybind {
                        key: Some(name.into()),
                        modifiers,
                    };
                    edit.binding = false;
                }
                return true;
            }
            if key == KeyCode::Escape {
                self.meteor_macros_gui.edit = None;
                return true;
            }
            if key == KeyCode::Tab {
                edit.focus = Some(match edit.focus {
                    Some(0) => 1,
                    _ => 0,
                });
                edit.replace = false;
                return true;
            }
            if key == KeyCode::Enter {
                if edit.focus == Some(0) {
                    edit.focus = Some(1);
                } else {
                    let at = edit
                        .focus
                        .unwrap_or(edit.value.messages.len())
                        .min(edit.value.messages.len());
                    edit.value.messages.insert(at, String::new());
                    edit.focus = Some(at + 1);
                    edit.line_scroll = at.saturating_sub(2);
                }
                return true;
            }
            if let Some(focus) = edit.focus {
                let target = if focus == 0 {
                    Some(&mut edit.value.name)
                } else {
                    edit.value.messages.get_mut(focus - 1)
                };
                if let Some(target) = target {
                    if ctrl && key == KeyCode::KeyA {
                        edit.replace = true;
                    } else if key == KeyCode::Backspace {
                        if edit.replace {
                            target.clear();
                            edit.replace = false;
                        } else {
                            target.pop();
                        }
                    } else if !ctrl && let Some(input) = input {
                        let clean: String = input.chars().filter(|c| !c.is_control()).collect();
                        let current = if edit.replace { 0 } else { target.len() };
                        if current + clean.len() > 65536 {
                            self.meteor_macros_gui.error =
                                Some("Text exceeds the native 64 KiB line limit".into());
                        } else {
                            if edit.replace {
                                target.clear();
                                edit.replace = false;
                            }
                            target.push_str(&clean);
                        }
                    }
                }
            }
        } else if key == KeyCode::Escape {
            self.meteor_macros_gui.open = false;
        }
        true
    }
    pub(super) fn meteor_macros_gui_mouse(&mut self, button: MouseButton, pressed: bool) -> bool {
        if !self.meteor_macros_gui.open {
            return false;
        }
        if !pressed {
            return true;
        }
        if let Some(edit) = self.meteor_macros_gui.edit.as_mut()
            && edit.binding
        {
            {
                let name = keybinds::mouse_name(button);
                edit.value.keybind = Keybind {
                    key: Some(name.into()),
                    modifiers: 0,
                };
                edit.binding = false;
            }
            return true;
        }
        if button != MouseButton::Left {
            return true;
        }
        let action = self
            .meteor_macros_gui
            .hits
            .iter()
            .rev()
            .find(|(r, _)| inside(*r, self.cursor))
            .map(|(_, a)| a.clone());
        match action {
            Some(Action::Back) => self.meteor_macros_gui.open = false,
            Some(Action::Create) => {
                self.meteor_macros_gui.edit = Some(Edit {
                    index: None,
                    value: Macro {
                        name: String::new(),
                        messages: vec![String::new()],
                        keybind: Keybind::default(),
                    },
                    focus: Some(0),
                    replace: false,
                    binding: false,
                    line_scroll: 0,
                });
                self.meteor_macros_gui.error = None;
            }
            Some(Action::Edit(index)) => {
                if let Some(value) = self.meteor_macros_gui.rows.get(index).cloned() {
                    self.meteor_macros_gui.edit = Some(Edit {
                        index: Some(index),
                        value,
                        focus: Some(0),
                        replace: false,
                        binding: false,
                        line_scroll: 0,
                    });
                    self.meteor_macros_gui.error = None;
                }
            }
            Some(Action::Remove(index)) => {
                if self.meteor_macros_gui.confirmation == Some(index) {
                    self.meteor_macros_gui.error = self.meteor_macro_remove(index).err();
                    self.meteor_macros_gui.rows = self.meteor_macros_rows();
                    self.meteor_macros_gui.confirmation = None;
                } else {
                    self.meteor_macros_gui.confirmation = Some(index);
                }
            }
            Some(Action::Save) => {
                if let Some(edit) = self.meteor_macros_gui.edit.as_ref() {
                    let index = edit.index;
                    let value = edit.value.clone();
                    match self.meteor_macro_edit(index, value) {
                        Ok(()) => {
                            self.meteor_macros_gui.edit = None;
                            self.meteor_macros_gui.rows = self.meteor_macros_rows();
                            self.meteor_macros_gui.error = None;
                        }
                        Err(e) => self.meteor_macros_gui.error = Some(e),
                    }
                }
            }
            Some(Action::Cancel) => {
                self.meteor_macros_gui.edit = None;
                self.meteor_macros_gui.error = None;
            }
            action => {
                if let Some(edit) = self.meteor_macros_gui.edit.as_mut() {
                    match action {
                        Some(Action::Name) => {
                            edit.focus = Some(0);
                            edit.replace = false;
                        }
                        Some(Action::Bind) => {
                            edit.binding = true;
                            edit.focus = None;
                        }
                        Some(Action::Line(index)) => {
                            edit.focus = Some(index + 1);
                            edit.replace = false;
                        }
                        Some(Action::AddLine) => {
                            edit.value.messages.push(String::new());
                            edit.focus = Some(edit.value.messages.len());
                            edit.line_scroll = edit.value.messages.len().saturating_sub(3);
                        }
                        Some(Action::RemoveLine) => {
                            if let Some(focus) = edit.focus
                                && focus > 0
                                && focus <= edit.value.messages.len()
                            {
                                edit.value.messages.remove(focus - 1);
                                edit.focus = None;
                            }
                        }
                        _ => edit.focus = None,
                    }
                }
            }
        }
        true
    }
    pub(super) fn meteor_macros_gui_wheel(&mut self, steps: f32) -> bool {
        if !self.meteor_macros_gui.open {
            return false;
        }
        let amount = (steps.abs().ceil() as usize).max(1);
        if let Some(edit) = self.meteor_macros_gui.edit.as_mut() {
            edit.line_scroll = if steps > 0.0 {
                edit.line_scroll.saturating_sub(amount)
            } else {
                (edit.line_scroll + amount).min(edit.value.messages.len().saturating_sub(1))
            };
        } else {
            self.meteor_macros_gui.scroll = if steps > 0.0 {
                self.meteor_macros_gui.scroll.saturating_sub(amount)
            } else {
                (self.meteor_macros_gui.scroll + amount).min(self.meteor_macros_gui.max_scroll)
            };
        }
        true
    }
    pub(super) fn draw_meteor_macros_gui(&mut self, batch: &mut HudBatch) {
        if !self.meteor_macros_gui.open {
            return;
        }
        let atlas = &self.gui.atlas;
        let w = batch.width as f32;
        let h = batch.height as f32;
        let width = (w - 24.0).min(580.0);
        let x = (w - width) * 0.5;
        batch.fill(atlas, 0., 0., w, h, rgba(0, 0, 0, 165));
        batch.fill(atlas, 0., 0., w, 27., rgba(20, 20, 20, 245));
        let mut hits = vec![];
        fn button(
            batch: &mut HudBatch,
            atlas: &HudAtlas,
            hits: &mut Vec<([f32; 4], Action)>,
            rect: [f32; 4],
            label: &str,
            active: bool,
            action: Action,
        ) {
            batch.fill(
                atlas,
                rect[0],
                rect[1],
                rect[2],
                rect[3],
                if active {
                    rgba(75, 40, 85, 245)
                } else {
                    rgba(30, 30, 30, 240)
                },
            );
            batch.text(
                atlas,
                &clipped(atlas, label, rect[2] - 10.),
                rect[0] + 5.,
                rect[1] + 5.,
                0xffffffff,
                false,
            );
            hits.push((rect, action));
        }
        button(
            batch,
            atlas,
            &mut hits,
            [6., 4., 68., 19.],
            "< Modules",
            false,
            Action::Back,
        );
        batch.text(atlas, "Meteor / Macros", 86., 9., 0xffe23d91, false);
        if let Some(edit) = self.meteor_macros_gui.edit.as_ref() {
            button(
                batch,
                atlas,
                &mut hits,
                [x, 37., width, 20.],
                &format!(
                    "Name: {}{}",
                    edit.value.name,
                    if edit.focus == Some(0) { "_" } else { "" }
                ),
                edit.focus == Some(0),
                Action::Name,
            );
            let bind = if edit.binding {
                "Press a key or mouse button (Esc cancels)".into()
            } else {
                format!(
                    "Keybind: {}",
                    super::meteor_gui::bind_display(&serde_json::json!(edit.value.keybind))
                )
            };
            button(
                batch,
                atlas,
                &mut hits,
                [x, 62., width, 20.],
                &bind,
                edit.binding,
                Action::Bind,
            );
            batch.text(
                atlas,
                "Messages / commands (Starscript)",
                x + 3.,
                89.,
                0xffbbbbbb,
                false,
            );
            let visible = ((h - 176.).max(18.) / 21.).floor() as usize;
            for (row, (index, line)) in edit
                .value
                .messages
                .iter()
                .enumerate()
                .skip(edit.line_scroll)
                .take(visible)
                .enumerate()
            {
                let focus = edit.focus == Some(index + 1);
                button(
                    batch,
                    atlas,
                    &mut hits,
                    [x, 103. + row as f32 * 21., width, 19.],
                    &format!("{}: {}{}", index + 1, line, if focus { "_" } else { "" }),
                    focus,
                    Action::Line(index),
                );
            }
            let y = (h - 65.).max(124.);
            button(
                batch,
                atlas,
                &mut hits,
                [x, y, 88., 19.],
                "+ Message",
                false,
                Action::AddLine,
            );
            button(
                batch,
                atlas,
                &mut hits,
                [x + 94., y, 110., 19.],
                "- Selected line",
                false,
                Action::RemoveLine,
            );
            button(
                batch,
                atlas,
                &mut hits,
                [x + width - 130., y, 62., 19.],
                "Save",
                true,
                Action::Save,
            );
            button(
                batch,
                atlas,
                &mut hits,
                [x + width - 62., y, 62., 19.],
                "Cancel",
                false,
                Action::Cancel,
            );
        } else {
            let visible = ((h - 113.).max(21.) / 23.).floor() as usize;
            self.meteor_macros_gui.max_scroll =
                self.meteor_macros_gui.rows.len().saturating_sub(visible);
            self.meteor_macros_gui.scroll = self
                .meteor_macros_gui
                .scroll
                .min(self.meteor_macros_gui.max_scroll);
            for (row, (index, value)) in self
                .meteor_macros_gui
                .rows
                .iter()
                .enumerate()
                .skip(self.meteor_macros_gui.scroll)
                .take(visible)
                .enumerate()
            {
                let y = 38. + row as f32 * 23.;
                button(
                    batch,
                    atlas,
                    &mut hits,
                    [x, y, width - 156., 20.],
                    &format!(
                        "{} ({})",
                        value.name,
                        super::meteor_gui::bind_display(&serde_json::json!(value.keybind))
                    ),
                    false,
                    Action::Edit(index),
                );
                button(
                    batch,
                    atlas,
                    &mut hits,
                    [x + width - 150., y, 56., 20.],
                    "Edit",
                    false,
                    Action::Edit(index),
                );
                let confirm = self.meteor_macros_gui.confirmation == Some(index);
                button(
                    batch,
                    atlas,
                    &mut hits,
                    [x + width - 88., y, 88., 20.],
                    if confirm { "Confirm remove" } else { "Remove" },
                    confirm,
                    Action::Remove(index),
                );
            }
            button(
                batch,
                atlas,
                &mut hits,
                [x, h - 65., width, 20.],
                "Create",
                false,
                Action::Create,
            );
            if self.meteor_macros_gui.rows.is_empty() {
                batch.text(
                    atlas,
                    "No macros. Create one, then set its keybind.",
                    x + 5.,
                    45.,
                    0xffbbbbbb,
                    false,
                );
            }
        }
        if let Some(error) = &self.meteor_macros_gui.error {
            batch.text(
                atlas,
                &clipped(atlas, error, w - 20.),
                10.,
                h - 35.,
                0xff7777ff,
                false,
            );
        }
        batch.text(
            atlas,
            "Enter adds a line | Ctrl+A replaces text | Keybind: Delete clears",
            10.,
            h - 17.,
            0xff969696,
            false,
        );
        self.meteor_macros_gui.hits = hits;
    }
}
