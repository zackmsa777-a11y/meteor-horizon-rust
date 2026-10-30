// SPDX-License-Identifier: GPL-3.0-only
// Native FriendsTab/FriendsScreen. Copyright (c) Meteor Development.
// Uses the real Friends model/commands. Profile fetching/head textures remain unported.
use super::Game;
use horizon_client::hud::{HudAtlas, HudBatch, rgba};
use winit::{event::MouseButton, keyboard::KeyCode};

#[derive(Default)]
pub(super) struct State {
    pub(super) open: bool,
    name: String,
    focus: bool,
    replace: bool,
    scroll: usize,
    max_scroll: usize,
    hits: Vec<([f32; 4], Action)>,
    error: Option<String>,
}
#[derive(Clone)]
enum Action {
    Back,
    Input,
    Add,
    Remove(String),
}
fn inside(r: [f32; 4], p: (f32, f32)) -> bool {
    p.0 >= r[0] && p.0 < r[0] + r[2] && p.1 >= r[1] && p.1 < r[1] + r[3]
}
fn clipped(atlas: &HudAtlas, value: &str, width: f32) -> String {
    let mut text = value.to_owned();
    if HudBatch::text_width(atlas, &text) <= width {
        return text;
    }
    while !text.is_empty() && HudBatch::text_width(atlas, &(text.clone() + "...")) > width {
        text.pop();
    }
    text + "..."
}
impl Game<'_> {
    #[cfg(feature = "meteor")]
    pub(super) fn meteor_friends_status(&self) -> serde_json::Value {
        serde_json::json!({"open":self.meteor_friends.open,"name":self.meteor_friends.name,"focus":self.meteor_friends.focus,"error":self.meteor_friends.error,"scroll":self.meteor_friends.scroll})
    }
    pub(super) fn meteor_friends_open(&mut self) {
        self.meteor_friends = State {
            open: true,
            focus: true,
            ..State::default()
        };
    }
    #[cfg(feature = "meteor")]
    pub(super) fn meteor_friends_wants_text(&self) -> bool {
        self.meteor_friends.open && self.meteor_friends.focus
    }
    pub(super) fn meteor_friends_key(
        &mut self,
        key: KeyCode,
        input: Option<&str>,
        pressed: bool,
    ) -> bool {
        if !self.meteor_friends.open {
            return false;
        }
        if !pressed {
            return true;
        }
        if key == KeyCode::Escape {
            self.meteor_friends.open = false;
            return true;
        }
        if key == KeyCode::Enter {
            self.meteor_friends_add();
            return true;
        }
        let ctrl = self.physical_keys.contains(&KeyCode::ControlLeft)
            || self.physical_keys.contains(&KeyCode::ControlRight);
        if self.meteor_friends.focus {
            if ctrl && key == KeyCode::KeyA {
                self.meteor_friends.replace = true;
            } else if key == KeyCode::Backspace {
                if self.meteor_friends.replace {
                    self.meteor_friends.name.clear();
                    self.meteor_friends.replace = false;
                } else {
                    self.meteor_friends.name.pop();
                }
            } else if !ctrl && let Some(input) = input {
                if self.meteor_friends.replace {
                    self.meteor_friends.name.clear();
                    self.meteor_friends.replace = false;
                }
                for c in input
                    .chars()
                    .filter(|c| !c.is_whitespace() && !c.is_control())
                {
                    if self.meteor_friends.name.len() < 128 {
                        self.meteor_friends.name.push(c);
                    }
                }
            }
        }
        true
    }
    fn meteor_friends_add(&mut self) {
        let name = self.meteor_friends.name.trim().to_owned();
        if name.is_empty() {
            self.meteor_friends.error = Some("Enter a player name.".into());
            return;
        }
        let before = self.meteor_status()["config"]["friends"]["friends"]
            .as_array()
            .map_or(0, Vec::len);
        let mut claimed = false;
        self.meteor_command(&format!("meteor friends add {name}"), &mut claimed);
        let after = self.meteor_status()["config"]["friends"]["friends"]
            .as_array()
            .map_or(0, Vec::len);
        if claimed && after > before {
            self.meteor_friends.name.clear();
            self.meteor_friends.error = None;
        } else {
            self.meteor_friends.error =
                Some("Already a friend, or this name could not be added.".into());
        }
        self.meteor_friends.focus = true;
    }
    pub(super) fn meteor_friends_mouse(&mut self, button: MouseButton, pressed: bool) -> bool {
        if !self.meteor_friends.open {
            return false;
        }
        if !pressed || button != MouseButton::Left {
            return true;
        }
        let action = self
            .meteor_friends
            .hits
            .iter()
            .rev()
            .find(|(r, _)| inside(*r, self.cursor))
            .map(|(_, a)| a.clone());
        match action {
            Some(Action::Back) => self.meteor_friends.open = false,
            Some(Action::Input) => self.meteor_friends.focus = true,
            Some(Action::Add) => self.meteor_friends_add(),
            Some(Action::Remove(name)) => {
                let mut claimed = false;
                self.meteor_command(&format!("meteor friends remove {name}"), &mut claimed);
                self.meteor_friends.error = None;
            }
            None => self.meteor_friends.focus = false,
        }
        true
    }
    pub(super) fn meteor_friends_wheel(&mut self, steps: f32) -> bool {
        if !self.meteor_friends.open {
            return false;
        }
        self.meteor_friends.scroll =
            (self.meteor_friends.scroll as i64 - (steps * 3.).round() as i64)
                .clamp(0, self.meteor_friends.max_scroll as i64) as usize;
        true
    }
    pub(super) fn draw_meteor_friends(&mut self, batch: &mut HudBatch) {
        let status = self.meteor_status();
        let friends = status["config"]["friends"]["friends"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let atlas = &self.gui.atlas;
        let w = batch.width as f32;
        let h = batch.height as f32;
        let width = (w - 24.).min(400.).max(100.);
        let x = (w - width) * 0.5;
        let bottom = (h - 52.).max(62.);
        let rows = ((bottom - 36.) / 26.).floor().max(0.) as usize;
        self.meteor_friends.max_scroll = friends.len().saturating_sub(rows);
        self.meteor_friends.scroll = self
            .meteor_friends
            .scroll
            .min(self.meteor_friends.max_scroll);
        let mut hits = vec![
            ([0., 0., 80., 27.], Action::Back),
            ([x, bottom + 5., width - 48., 21.], Action::Input),
            ([x + width - 44., bottom + 5., 44., 21.], Action::Add),
        ];
        batch.fill(atlas, 0., 0., w, h, rgba(0, 0, 0, 110));
        batch.fill(atlas, 0., 0., w, 27., rgba(20, 20, 20, 240));
        batch.text(atlas, "< Modules", 7., 9., rgba(255, 255, 255, 255), false);
        batch.text(
            atlas,
            &format!("Friends ({})", friends.len()),
            90.,
            9.,
            rgba(190, 130, 255, 255),
            false,
        );
        if friends.is_empty() {
            batch.text(
                atlas,
                "No friends yet. Add a player below.",
                x + 6.,
                45.,
                rgba(175, 175, 175, 255),
                false,
            );
        }
        // Preserve the model's source order: sorted on load, insertion order after adding.
        for (row, friend) in friends
            .iter()
            .skip(self.meteor_friends.scroll)
            .take(rows)
            .enumerate()
        {
            let y = 34. + row as f32 * 26.;
            let name = friend["name"].as_str().unwrap_or("");
            batch.fill(atlas, x, y, width, 24., rgba(20, 20, 20, 230));
            batch.text(
                atlas,
                &clipped(atlas, name, width - 62.),
                x + 6.,
                y + 3.,
                rgba(255, 255, 255, 255),
                false,
            );
            batch.text(
                atlas,
                if friend["id"].is_string() {
                    "Stored profile identity"
                } else {
                    "Offline / profile unresolved"
                },
                x + 6.,
                y + 14.,
                rgba(150, 150, 150, 255),
                false,
            );
            let rect = [x + width - 48., y, 48., 24.];
            batch.fill(
                atlas,
                rect[0],
                rect[1],
                rect[2],
                rect[3],
                if inside(rect, self.cursor) {
                    rgba(60, 40, 55, 255)
                } else {
                    rgba(40, 30, 40, 255)
                },
            );
            batch.text(
                atlas,
                "Remove",
                x + width - 43.,
                y + 8.,
                rgba(235, 180, 200, 255),
                false,
            );
            hits.push((rect, Action::Remove(name.to_owned())));
        }
        batch.fill(
            atlas,
            x,
            bottom + 5.,
            width - 48.,
            21.,
            if self.meteor_friends.focus {
                rgba(50, 40, 65, 255)
            } else {
                rgba(25, 25, 25, 255)
            },
        );
        let prompt = if self.meteor_friends.name.is_empty() {
            "Player name"
        } else {
            &self.meteor_friends.name
        };
        batch.text(
            atlas,
            &clipped(atlas, prompt, width - 58.),
            x + 5.,
            bottom + 12.,
            rgba(230, 230, 230, 255),
            false,
        );
        batch.fill(
            atlas,
            x + width - 44.,
            bottom + 5.,
            44.,
            21.,
            rgba(95, 40, 155, 255),
        );
        batch.text(
            atlas,
            "Add +",
            x + width - 37.,
            bottom + 12.,
            rgba(255, 255, 255, 255),
            false,
        );
        batch.text(
            atlas,
            self.meteor_friends
                .error
                .as_deref()
                .unwrap_or("Enter: add | Esc: Modules"),
            x,
            bottom + 33.,
            rgba(190, 190, 190, 255),
            false,
        );
        self.meteor_friends.hits = hits;
    }
}
