// SPDX-License-Identifier: GPL-3.0-only
// Native ProfilesTab: explicit capture/load, edit metadata, confirmed save/delete.
use super::Game;
use horizon_client::hud::{HudAtlas, HudBatch, rgba};
use meteor_client::profiles::{Profile, SaveSections};
use winit::{event::MouseButton, keyboard::KeyCode};

#[derive(Default)]
pub(super) struct State {
    pub(super) open: bool,
    rows: Vec<Profile>,
    edit: Option<Edit>,
    confirmation: Option<(String, String)>,
    message: Option<String>,
    failed: bool,
    scroll: usize,
    max_scroll: usize,
    hits: Vec<([f32; 4], Action)>,
}
struct Edit {
    original: Option<String>,
    name: String,
    keys: String,
    modules: bool,
    hud: bool,
    macros: bool,
    focus: u8,
    replace: bool,
}
#[derive(Clone)]
enum Action {
    Back,
    New,
    Edit(String),
    Run(String, String),
    Name,
    Keys,
    Modules,
    Hud,
    Macros,
    Apply,
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
fn quote(text: &str) -> String {
    serde_json::to_string(text).unwrap_or_default()
}

impl Game<'_> {
    pub(super) fn meteor_profiles_gui_status(&self) -> serde_json::Value {
        let state = &self.meteor_profiles_gui;
        let targets: std::collections::BTreeMap<String, [f32; 4]> = state
            .hits
            .iter()
            .map(|(rect, action)| {
                let key = match action {
                    Action::Back => "back".into(),
                    Action::New => "create".into(),
                    Action::Edit(name) => format!("edit/{name}"),
                    Action::Run(command, name) => format!("{command}/{name}"),
                    Action::Name => "name".into(),
                    Action::Keys => "keys".into(),
                    Action::Modules => "modules".into(),
                    Action::Hud => "hud".into(),
                    Action::Macros => "macros".into(),
                    Action::Apply => "apply".into(),
                    Action::Cancel => "cancel".into(),
                };
                (key, *rect)
            })
            .collect();
        serde_json::json!({"open":state.open,"names":state.rows.iter().map(|p| &p.name).collect::<Vec<_>>(),
            "targets":targets,
            "message":state.message,"failed":state.failed,"confirmation":state.confirmation,
            "edit":state.edit.as_ref().map(|e| serde_json::json!({"name":e.name,"keys":e.keys,"modules":e.modules,"hud":e.hud,"macros":e.macros,"focus":e.focus}))})
    }
    fn meteor_profiles_gui_refresh(&mut self) {
        match self.meteor_profiles_read() {
            Ok(profiles) => self.meteor_profiles_gui.rows = profiles.entries,
            Err(error) => {
                self.meteor_profiles_gui.rows.clear();
                self.meteor_profiles_gui.message = Some(error);
                self.meteor_profiles_gui.failed = true;
            }
        }
    }
    pub(super) fn meteor_profiles_gui_open(&mut self) {
        self.meteor_profiles_gui = State {
            open: true,
            ..Default::default()
        };
        self.meteor_profiles_gui_refresh();
    }
    pub(super) fn meteor_profiles_gui_wants_text(&self) -> bool {
        self.meteor_profiles_gui.open
            && self
                .meteor_profiles_gui
                .edit
                .as_ref()
                .is_some_and(|e| e.focus != 0)
    }
    pub(super) fn meteor_profiles_gui_key(
        &mut self,
        key: KeyCode,
        text: Option<&str>,
        pressed: bool,
    ) -> bool {
        if !self.meteor_profiles_gui.open {
            return false;
        }
        if !pressed {
            return true;
        }
        if key == KeyCode::Escape {
            if self.meteor_profiles_gui.edit.take().is_none() {
                self.meteor_profiles_gui.open = false;
            }
            self.meteor_profiles_gui.confirmation = None;
            return true;
        }
        if key == KeyCode::Enter && self.meteor_profiles_gui.edit.is_some() {
            self.meteor_profiles_gui_apply();
            return true;
        }
        let ctrl = self.physical_keys.contains(&KeyCode::ControlLeft)
            || self.physical_keys.contains(&KeyCode::ControlRight);
        if let Some(edit) = &mut self.meteor_profiles_gui.edit {
            if key == KeyCode::Tab {
                edit.focus = if edit.focus == 1 { 2 } else { 1 };
                edit.replace = true;
                return true;
            }
            let value = match edit.focus {
                1 => &mut edit.name,
                2 => &mut edit.keys,
                _ => return true,
            };
            if ctrl && key == KeyCode::KeyA {
                edit.replace = true;
            } else if key == KeyCode::Backspace {
                if edit.replace {
                    value.clear();
                } else {
                    value.pop();
                }
                edit.replace = false;
            } else if !ctrl && let Some(text) = text {
                if edit.replace {
                    value.clear();
                    edit.replace = false;
                }
                for c in text.chars().filter(|c| !c.is_control()) {
                    if value.len() < 2048 {
                        value.push(c);
                    }
                }
            }
        }
        true
    }
    fn meteor_profiles_gui_apply(&mut self) {
        let result = (|| -> Result<String, String> {
            let edit = self
                .meteor_profiles_gui
                .edit
                .as_ref()
                .ok_or("No profile selected")?;
            let keys = meteor_client::profiles::arguments(&edit.keys)?;
            let save = SaveSections {
                modules: edit.modules,
                hud: edit.hud,
                macros: edit.macros,
                ..Default::default()
            };
            let mut profiles = self.meteor_profiles_read()?;
            if let Some(original) = &edit.original {
                let profile = profiles
                    .entries
                    .iter_mut()
                    .find(|p| p.name.eq_ignore_ascii_case(original))
                    .ok_or("Profile no longer exists")?;
                profile.name = edit.name.clone();
                profile.save = save;
                profile.load_on_join = keys;
            } else {
                if self.meteor.load_failed {
                    return Err("Cannot capture an unreadable configuration".into());
                }
                profiles.create(edit.name.clone(), save, keys, &self.meteor.client.config)?;
            }
            self.meteor_profiles_write(&profiles)?;
            Ok(if edit.original.is_some() {
                "Metadata saved. Use Save to recapture selected sections."
            } else {
                "Created profile and captured selected sections."
            }
            .into())
        })();
        self.meteor_profiles_gui.failed = result.is_err();
        match result {
            Ok(message) => {
                self.meteor_profiles_gui.message = Some(message);
                self.meteor_profiles_gui.edit = None;
                self.meteor_profiles_gui_refresh();
            }
            Err(error) => self.meteor_profiles_gui.message = Some(error),
        }
    }
    pub(super) fn meteor_profiles_gui_mouse(&mut self, button: MouseButton, pressed: bool) -> bool {
        if !self.meteor_profiles_gui.open {
            return false;
        }
        if !pressed || button != MouseButton::Left {
            return true;
        }
        let action = self
            .meteor_profiles_gui
            .hits
            .iter()
            .rev()
            .find(|(r, _)| inside(*r, self.cursor))
            .map(|(_, a)| a.clone());
        match action {
            Some(Action::Back) => self.meteor_profiles_gui.open = false,
            Some(Action::Cancel) => self.meteor_profiles_gui.edit = None,
            Some(Action::New) => {
                self.meteor_profiles_gui.edit = Some(Edit {
                    original: None,
                    name: String::new(),
                    keys: String::new(),
                    modules: false,
                    hud: false,
                    macros: false,
                    focus: 1,
                    replace: false,
                });
                self.meteor_profiles_gui.message = None;
            }
            Some(Action::Edit(name)) => {
                if let Some(profile) = self
                    .meteor_profiles_gui
                    .rows
                    .iter()
                    .find(|p| p.name == name)
                {
                    self.meteor_profiles_gui.edit = Some(Edit {
                        original: Some(name),
                        name: profile.name.clone(),
                        keys: profile
                            .load_on_join
                            .iter()
                            .map(|s| quote(s))
                            .collect::<Vec<_>>()
                            .join(" "),
                        modules: profile.save.modules,
                        hud: profile.save.hud,
                        macros: profile.save.macros,
                        focus: 1,
                        replace: true,
                    });
                    self.meteor_profiles_gui.message = None;
                }
            }
            Some(Action::Name) => {
                if let Some(e) = &mut self.meteor_profiles_gui.edit {
                    e.focus = 1;
                    e.replace = false;
                }
            }
            Some(Action::Keys) => {
                if let Some(e) = &mut self.meteor_profiles_gui.edit {
                    e.focus = 2;
                    e.replace = false;
                }
            }
            Some(Action::Modules) => {
                if let Some(e) = &mut self.meteor_profiles_gui.edit {
                    e.modules = !e.modules;
                }
            }
            Some(Action::Hud) => {
                if let Some(e) = &mut self.meteor_profiles_gui.edit {
                    e.hud = !e.hud;
                }
            }
            Some(Action::Macros) => {
                if let Some(e) = &mut self.meteor_profiles_gui.edit {
                    e.macros = !e.macros;
                }
            }
            Some(Action::Apply) => self.meteor_profiles_gui_apply(),
            Some(Action::Run(command, name)) => {
                let pair = (command.clone(), name.clone());
                if command != "load"
                    && self.meteor_profiles_gui.confirmation.as_ref() != Some(&pair)
                {
                    self.meteor_profiles_gui.confirmation = Some(pair);
                    return true;
                }
                self.meteor_profiles_gui.confirmation = None;
                let result = self.meteor_profiles_command(&format!("{command} {}", quote(&name)));
                self.meteor_profiles_gui.failed = result.is_err();
                self.meteor_profiles_gui.message = Some(result.unwrap_or_else(|e| e));
                self.meteor_profiles_gui_refresh();
            }
            None => {
                if let Some(e) = &mut self.meteor_profiles_gui.edit {
                    e.focus = 0;
                }
            }
        }
        true
    }
    pub(super) fn meteor_profiles_gui_wheel(&mut self, steps: f32) -> bool {
        if !self.meteor_profiles_gui.open {
            return false;
        }
        let state = &mut self.meteor_profiles_gui;
        state.scroll = (state.scroll as i64 - (steps * 3.).round() as i64)
            .clamp(0, state.max_scroll as i64) as usize;
        true
    }
    pub(super) fn draw_meteor_profiles_gui(&mut self, batch: &mut HudBatch) {
        let atlas = &self.gui.atlas;
        let w = batch.width as f32;
        let h = batch.height as f32;
        let width = (w - 20.).clamp(120., 530.);
        let x = (w - width) * 0.5;
        let editing = self.meteor_profiles_gui.edit.is_some();
        let footer = if editing { 151. } else { 61. };
        let rows = ((h - footer - 36.) / 26.).floor().max(0.) as usize;
        let state = &mut self.meteor_profiles_gui;
        state.max_scroll = state.rows.len().saturating_sub(rows);
        state.scroll = state.scroll.min(state.max_scroll);
        let mut hits = Vec::new();
        batch.fill(atlas, 0., 0., w, h, rgba(0, 0, 0, 155));
        batch.fill(atlas, 0., 0., w, 27., rgba(20, 20, 20, 245));
        button(
            batch,
            atlas,
            &mut hits,
            [0., 0., 80., 27.],
            "< Modules",
            Action::Back,
            self.cursor,
            false,
        );
        batch.text(atlas, "Profiles", 90., 9., rgba(190, 130, 255, 255), false);
        for (index, profile) in state.rows.iter().skip(state.scroll).take(rows).enumerate() {
            let y = 34. + index as f32 * 26.;
            batch.fill(atlas, x, y, width, 24., rgba(20, 20, 20, 240));
            batch.text(
                atlas,
                &clipped(atlas, &profile.name, width - 184.),
                x + 5.,
                y + 4.,
                rgba(255, 255, 255, 255),
                false,
            );
            let scope = format!(
                "{}{}{}",
                if profile.save.modules { "Modules " } else { "" },
                if profile.save.hud { "HUD " } else { "" },
                if profile.save.macros { "Macros" } else { "" }
            );
            batch.text(
                atlas,
                &clipped(
                    atlas,
                    if scope.is_empty() {
                        "No sections"
                    } else {
                        &scope
                    },
                    width - 184.,
                ),
                x + 5.,
                y + 14.,
                rgba(175, 175, 175, 255),
                false,
            );
            for (i, (label, command)) in [
                ("Save", "save"),
                ("Load", "load"),
                ("Edit", "edit"),
                ("Delete", "delete"),
            ]
            .into_iter()
            .enumerate()
            {
                let confirming =
                    state.confirmation.as_ref() == Some(&(command.into(), profile.name.clone()));
                button(
                    batch,
                    atlas,
                    &mut hits,
                    [x + width - 180. + i as f32 * 45., y, 44., 24.],
                    if confirming { "Confirm" } else { label },
                    if command == "edit" {
                        Action::Edit(profile.name.clone())
                    } else {
                        Action::Run(command.into(), profile.name.clone())
                    },
                    self.cursor,
                    confirming,
                );
            }
        }
        let y = (h - footer).max(34.);
        if let Some(edit) = &state.edit {
            batch.fill(atlas, x, y, width, 116., rgba(20, 20, 20, 250));
            for (label, value, focus, dy, action) in [
                ("Name", &edit.name, 1, 4., Action::Name),
                (
                    "Exact join keys (quote spaces)",
                    &edit.keys,
                    2,
                    39.,
                    Action::Keys,
                ),
            ] {
                batch.text(
                    atlas,
                    label,
                    x + 5.,
                    y + dy,
                    rgba(200, 200, 200, 255),
                    false,
                );
                let rect = [x + 4., y + dy + 11., width - 8., 20.];
                batch.fill(
                    atlas,
                    rect[0],
                    rect[1],
                    rect[2],
                    rect[3],
                    if edit.focus == focus {
                        rgba(65, 45, 85, 255)
                    } else {
                        rgba(35, 35, 35, 255)
                    },
                );
                batch.text(
                    atlas,
                    &clipped(atlas, value, width - 18.),
                    x + 8.,
                    y + dy + 17.,
                    rgba(255, 255, 255, 255),
                    false,
                );
                hits.push((rect, action));
            }
            button(
                batch,
                atlas,
                &mut hits,
                [x + 4., y + 77., 86., 22.],
                if edit.modules {
                    "[x] Modules"
                } else {
                    "[ ] Modules"
                },
                Action::Modules,
                self.cursor,
                edit.modules,
            );
            button(
                batch,
                atlas,
                &mut hits,
                [x + 94., y + 77., 65., 22.],
                if edit.hud { "[x] HUD" } else { "[ ] HUD" },
                Action::Hud,
                self.cursor,
                edit.hud,
            );
            button(
                batch,
                atlas,
                &mut hits,
                [x + 163., y + 77., 80., 22.],
                if edit.macros {
                    "[x] Macros"
                } else {
                    "[ ] Macros"
                },
                Action::Macros,
                self.cursor,
                edit.macros,
            );
            button(
                batch,
                atlas,
                &mut hits,
                [x + width - 115., y + 77., 60., 22.],
                if edit.original.is_some() {
                    "Apply"
                } else {
                    "Create"
                },
                Action::Apply,
                self.cursor,
                true,
            );
            button(
                batch,
                atlas,
                &mut hits,
                [x + width - 51., y + 77., 47., 22.],
                "Cancel",
                Action::Cancel,
                self.cursor,
                false,
            );
            batch.text(
                atlas,
                "Tab: next field | Enter: apply | Esc: cancel",
                x + 5.,
                y + 104.,
                rgba(180, 180, 180, 255),
                false,
            );
        } else {
            button(
                batch,
                atlas,
                &mut hits,
                [x, y, 90., 24.],
                "Create profile",
                Action::New,
                self.cursor,
                true,
            );
        }
        let message = state
            .message
            .as_deref()
            .unwrap_or("Save captures checked sections; Load restores them.");
        batch.text(
            atlas,
            &clipped(atlas, message, width),
            x,
            h - 27.,
            if state.failed {
                rgba(255, 145, 145, 255)
            } else {
                rgba(215, 215, 215, 255)
            },
            false,
        );
        batch.text(
            atlas,
            &clipped(
                atlas,
                "Native profiles. Java NBT and waypoints pending.",
                width,
            ),
            x,
            h - 14.,
            rgba(170, 170, 170, 255),
            false,
        );
        state.hits = hits;
    }
}
#[allow(clippy::too_many_arguments)]
fn button(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    hits: &mut Vec<([f32; 4], Action)>,
    r: [f32; 4],
    text: &str,
    action: Action,
    cursor: (f32, f32),
    accent: bool,
) {
    let color = if inside(r, cursor) {
        rgba(90, 65, 115, 255)
    } else if accent {
        rgba(90, 40, 145, 255)
    } else {
        rgba(40, 35, 45, 255)
    };
    batch.fill(atlas, r[0], r[1], r[2], r[3], color);
    batch.text(
        atlas,
        &clipped(atlas, text, r[2] - 6.),
        r[0] + 3.,
        r[1] + (r[3] - 8.) * 0.5,
        rgba(255, 255, 255, 255),
        false,
    );
    hits.push((r, action));
}
