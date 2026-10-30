// SPDX-License-Identifier: GPL-3.0-only
// Port of GUIMove.java. Copyright (c) Meteor Development.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Screens {
    GUI,
    #[default]
    Inventory,
    Both,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub guis: Screens,
    pub jump: bool,
    pub sneak: bool,
    pub sprint: bool,
    pub arrows_rotate: bool,
    pub rotate_speed: f64,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            guis: Screens::Inventory,
            jump: true,
            sneak: true,
            sprint: true,
            arrows_rotate: true,
            rotate_speed: 4.0,
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if !self.rotate_speed.is_finite() || self.rotate_speed < 0.0 {
            Err("rotate-speed must be finite and nonnegative".into())
        } else {
            Ok(())
        }
    }
    pub fn enabled(&self, action: Action) -> bool {
        match action {
            Action::Jump => self.jump,
            Action::Sneak => self.sneak,
            Action::Sprint => self.sprint,
            _ => true,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Screen {
    #[default]
    None,
    Widget,
    Container,
    Other,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Context {
    pub screen: Screen,
    pub can_use_keys: bool,
    pub creative_search: bool,
    pub text_editor: bool,
}
pub fn skip(context: Context, settings: &Settings) -> bool {
    !context.can_use_keys
        || context.creative_search
        || context.text_editor
        || !match settings.guis {
            Screens::GUI => context.screen == Screen::Widget,
            Screens::Inventory => context.screen == Screen::Container,
            Screens::Both => matches!(context.screen, Screen::Widget | Screen::Container),
        }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    Forward,
    Back,
    Left,
    Right,
    Jump,
    Sneak,
    Sprint,
}
pub const ACTIONS: [(Action, &str); 7] = [
    (Action::Forward, "key.forward"),
    (Action::Back, "key.back"),
    (Action::Left, "key.left"),
    (Action::Right, "key.right"),
    (Action::Jump, "key.jump"),
    (Action::Sneak, "key.sneak"),
    (Action::Sprint, "key.sprint"),
];
#[derive(Default)]
pub struct GuiMove {
    held: BTreeSet<Action>,
    pub allowed: bool,
}
impl GuiMove {
    /// Release events can be lost when a window loses focus or an input driver stops.
    pub fn release_all(&mut self) {
        self.held.clear();
    }
    pub fn sync(&mut self, active: bool, context: Context, settings: &Settings) {
        self.allowed = active && !skip(context, settings);
        if !self.allowed {
            self.release_all();
        } else {
            self.held.retain(|a| settings.enabled(*a));
        }
    }
    pub fn input(&mut self, action: Action, pressed: bool, repeat: bool, settings: &Settings) {
        if !self.allowed || repeat || !settings.enabled(action) {
            return;
        }
        if pressed {
            self.held.insert(action);
        } else {
            self.held.remove(&action);
        }
    }
    pub fn down(&self, action: Action) -> bool {
        self.allowed && self.held.contains(&action)
    }
}
/// Frame-time-scaled rotation, matching the source's float conversion and 100-degree cap.
pub fn rotate(
    yaw: f32,
    pitch: f32,
    seconds: f64,
    settings: &Settings,
    arrows: [bool; 4],
) -> (f32, f32) {
    if !settings.arrows_rotate {
        return (yaw, pitch);
    }
    let delta = ((settings.rotate_speed * seconds * 20.0) as f32).min(100.0);
    let [left, right, up, down] = arrows;
    (
        yaw + if left { -delta } else { 0.0 } + if right { delta } else { 0.0 },
        (pitch + if up { -delta } else { 0.0 } + if down { delta } else { 0.0 }).clamp(-90.0, 90.0),
    )
}
