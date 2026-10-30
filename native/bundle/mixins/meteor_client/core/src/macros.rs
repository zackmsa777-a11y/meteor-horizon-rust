// SPDX-License-Identifier: GPL-3.0-only
// Source-derived Macros.java, Macro.java, MacrosTab.java and MacroCommand.java behavior.
// Copyright (c) Meteor Development.
//! This system is not a module. Input selection and scheduling do not send messages.
//! The host must supply real Starscript compilation/evaluation and ChatUtils-style dispatch.
use crate::registry::{KeyEvent, Keybind};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Macro {
    pub name: String,
    pub messages: Vec<String>,
    pub keybind: Keybind,
}
impl PartialEq for Macro {
    // Upstream Macro.equals deliberately compares only the case-sensitive name.
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl Eq for Macro {}
fn simple_fold(c: char) -> char {
    let mut upper = c.to_uppercase();
    let first = upper.next().unwrap_or(c);
    let c = if upper.next().is_some() { c } else { first };
    c.to_lowercase().next().unwrap_or(c)
}
fn blank(s: &str) -> bool {
    // Java String.isBlank uses Character.isWhitespace, excluding non-breaking spaces.
    s.chars().all(|c| {
        matches!(c, '\u{0009}'..='\u{000d}' | '\u{001c}'..='\u{0020}' |
        '\u{1680}' | '\u{2000}'..='\u{2006}' | '\u{2008}'..='\u{200a}' | '\u{2028}' |
        '\u{2029}' | '\u{205f}' | '\u{3000}')
    })
}
impl Macro {
    /// The editor validates only blank names and an empty message list, not empty lines.
    pub fn validate_editor(&self) -> Result<(), String> {
        if blank(&self.name) {
            return Err("Macro name cannot be blank".into());
        }
        if self.messages.is_empty() {
            return Err("Macro needs at least one message".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Macros {
    pub macros: Vec<Macro>,
}
impl Macros {
    /// Native storage/input bounds. Loading retains source-permitted empty names and
    /// message lists; editor-only semantic validation is intentionally separate.
    pub fn validate_storage(&self) -> Result<(), String> {
        if self.macros.len() > 1024 {
            return Err("Too many macros (native limit 1024)".into());
        }
        let mut bytes = 0usize;
        for value in &self.macros {
            if value.name.len() > 4096
                || value.messages.len() > 1024
                || value.messages.iter().any(|s| s.len() > 65536)
            {
                return Err("Macro exceeds native storage limits".into());
            }
            if value.keybind.modifiers > 15
                || value.keybind.key.as_ref().is_some_and(|key| {
                    key.len() > 128
                        || !(key.starts_with("key.keyboard.") || key.starts_with("key.mouse."))
                })
            {
                return Err("Macro has an invalid native keybind".into());
            }
            bytes = bytes
                .saturating_add(value.name.len())
                .saturating_add(value.messages.iter().map(String::len).sum::<usize>());
            if bytes > 4 * 1024 * 1024 {
                return Err("Macros exceed the native 4 MiB storage limit".into());
            }
        }
        Ok(())
    }
    pub fn get(&self, name: &str) -> Option<&Macro> {
        self.macros.iter().find(|m| {
            m.name
                .chars()
                .map(simple_fold)
                .eq(name.chars().map(simple_fold))
        })
    }
    /// Source add itself permits duplicates; only the NEW macro editor rejects exact names.
    pub fn add_from_editor(&mut self, candidate: Macro) -> Result<(), String> {
        candidate.validate_editor()?;
        if self.macros.contains(&candidate) {
            return Err("A macro with that exact name already exists".into());
        }
        self.macros.push(candidate);
        Ok(())
    }
    /// Existing edits do not run the new-name uniqueness check in MacrosTab.
    pub fn edit_from_editor(&mut self, index: usize, candidate: Macro) -> Result<(), String> {
        candidate.validate_editor()?;
        *self.macros.get_mut(index).ok_or("Macro no longer exists")? = candidate;
        Ok(())
    }
    pub fn remove(&mut self, target: &Macro) -> bool {
        let Some(index) = self.macros.iter().position(|m| m == target) else {
            return false;
        };
        self.macros.remove(index);
        true
    }
    /// Macros are high-priority observers, not input cancellers. Run this before module binds
    /// but still deliver the event afterward. Unlike modules: repeats are accepted, no F3
    /// exclusion exists, and FIRST list match wins without modified-bind precedence.
    pub fn input(&self, event: &KeyEvent) -> Option<&Macro> {
        if !event.pressed || event.screen_open {
            return None;
        }
        let event = KeyEvent {
            key: event.key.clone(),
            modifiers: if event.key.starts_with("key.mouse.") {
                0
            } else {
                event.modifiers
            },
            pressed: event.pressed,
            repeat: event.repeat,
            screen_open: false,
            debug_key_down: event.debug_key_down,
        };
        self.macros.iter().find(|m| m.keybind.matches(&event))
    }
}

/// A host-supplied compiled Starscript cache. Compile errors skip only that line; runtime
/// errors skip only that execution. No literal-only evaluator or fake Starscript fallback.
pub struct Compiled<S> {
    scripts: Vec<S>,
    dirty: bool,
}
impl<S> Default for Compiled<S> {
    fn default() -> Self {
        Self {
            scripts: Vec::new(),
            dirty: true,
        }
    }
}
#[derive(Default, Debug, PartialEq, Eq)]
pub struct Evaluation {
    pub messages: Vec<String>,
    pub errors: Vec<String>,
}
impl<S> Compiled<S> {
    /// Call whenever messages are assigned, including deserialization; name/bind edits do not.
    pub fn messages_changed(&mut self) {
        self.dirty = true;
    }
    pub fn evaluate(
        &mut self,
        messages: &[String],
        mut compile: impl FnMut(&str) -> Result<S, String>,
        mut run: impl FnMut(&S) -> Result<String, String>,
    ) -> Evaluation {
        let mut result = Evaluation::default();
        if self.dirty {
            self.scripts.clear();
            for message in messages {
                match compile(message) {
                    Ok(script) => self.scripts.push(script),
                    Err(error) => result.errors.push(error),
                }
            }
            self.dirty = false;
        }
        for script in &self.scripts {
            match run(script) {
                Ok(message) => result.messages.push(message),
                Err(error) => result.errors.push(error),
            }
        }
        result
    }
}

/// MacroCommand schedules object REFERENCES. T must preserve live identity (e.g. an Rc
/// handle), including after list removal; do not substitute a snapshot of the message text.
pub struct Schedule<T> {
    queued: Vec<(u32, T)>,
    active: Vec<(u32, T)>,
}
impl<T> Default for Schedule<T> {
    fn default() -> Self {
        Self {
            queued: Vec::new(),
            active: Vec::new(),
        }
    }
}
impl<T> Schedule<T> {
    pub fn queue(&mut self, delay: u32, value: T) {
        self.queued.push((delay, value));
    }
    pub fn contains(&self, mut same_identity: impl FnMut(&T) -> bool) -> bool {
        self.queued
            .iter()
            .chain(&self.active)
            .any(|(_, value)| same_identity(value))
    }
    pub fn clear(&mut self, mut same_identity: impl FnMut(&T) -> bool) {
        self.queued.retain(|(_, value)| !same_identity(value));
        self.active.retain(|(_, value)| !same_identity(value));
    }
    pub fn clear_all(&mut self) {
        self.queued.clear();
        self.active.clear();
    }
    /// Tick.Post: enqueue, run due items, THEN decrement remaining delays. A due task with
    /// no player is consumed without execution, matching ScheduledMacro.run/runMacro.
    pub fn post_tick(&mut self, player_exists: bool) -> Vec<T> {
        self.active.append(&mut self.queued);
        let mut due = Vec::new();
        let mut next = Vec::new();
        for (delay, value) in self.active.drain(..) {
            if delay == 0 {
                if player_exists {
                    due.push(value);
                }
            } else {
                next.push((delay - 1, value));
            }
        }
        self.active = next;
        due
    }
}
