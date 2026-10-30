// SPDX-License-Identifier: GPL-3.0-only
// Derived from Meteor Client Module.java, Modules.java and Keybind.java.
// Copyright (c) Meteor Development.
use serde::{Deserialize, Serialize};

/// Engine-neutral key names; keyboard and mouse names occupy separate namespaces.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Keybind {
    pub key: Option<String>,
    pub modifiers: u8,
}

impl Keybind {
    pub fn matches(&self, event: &KeyEvent) -> bool {
        self.key.as_deref() == Some(event.key.as_str())
            && (self.modifiers == 0 || self.modifiers == event.modifiers)
    }
}

pub struct KeyEvent {
    pub key: String,
    pub modifiers: u8,
    pub pressed: bool,
    pub repeat: bool,
    pub screen_open: bool,
    pub debug_key_down: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Module {
    pub id: String,
    pub title: String,
    pub category: String,
    pub active: bool,
    pub keybind: Keybind,
    pub toggle_on_bind_release: bool,
    pub chat_feedback: bool,
    pub favorite: bool,
}

impl Default for Module {
    fn default() -> Self {
        Self {
            id: String::new(),
            title: String::new(),
            category: String::new(),
            active: false,
            keybind: Keybind::default(),
            toggle_on_bind_release: false,
            chat_feedback: true,
            favorite: false,
        }
    }
}

impl Module {
    pub fn new(id: &str, title: &str, category: &str) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            category: category.into(),
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Registry {
    pub modules: Vec<Module>,
}

impl Registry {
    pub fn new(modules: Vec<Module>) -> Self {
        Self { modules }
    }
    pub fn get(&self, id: &str) -> Option<&Module> {
        self.modules.iter().find(|m| m.id == id)
    }
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Module> {
        self.modules.iter_mut().find(|m| m.id == id)
    }
    pub fn matching(&self, event: &KeyEvent) -> Vec<String> {
        if event.repeat || event.screen_open || event.debug_key_down {
            return Vec::new();
        }
        let modified = self
            .modules
            .iter()
            .any(|m| m.keybind.modifiers != 0 && m.keybind.matches(event));
        self.modules
            .iter()
            .filter(|m| !modified || m.keybind.modifiers != 0)
            .filter(|m| m.keybind.matches(event))
            .filter(|m| event.pressed || (m.toggle_on_bind_release && m.active))
            .map(|m| m.id.clone())
            .collect()
    }
}
