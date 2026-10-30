// SPDX-License-Identifier: GPL-3.0-only
// Derived from Meteor Client Profile.java, Profiles.java and Modules.fromTag.
// Copyright (c) Meteor Development.
//! Native profile persistence and load plans. The adapter writes the returned JSON and
//! applies module lifecycle operations; this model never touches worlds or connects to servers.
use crate::Config;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Brigadier-like quoted names for the native profile command. No shell expansion.
pub fn arguments(text: &str) -> Result<Vec<String>, String> {
    let mut args = Vec::new();
    let mut chars = text.chars().peekable();
    while chars.peek().is_some() {
        while chars.peek().is_some_and(|c| c.is_whitespace()) {
            chars.next();
        }
        let Some(first) = chars.next() else {
            break;
        };
        let mut value = String::new();
        if first == '"' || first == '\'' {
            let mut closed = false;
            while let Some(c) = chars.next() {
                if c == first {
                    closed = true;
                    break;
                }
                if c == '\\' {
                    let escaped = chars.next().ok_or("Unterminated profile argument escape")?;
                    if escaped != first && escaped != '\\' {
                        return Err("Only quote/backslash escapes are supported".into());
                    }
                    value.push(escaped);
                } else {
                    value.push(c);
                }
            }
            if !closed {
                return Err("Unterminated quoted profile argument".into());
            }
            if chars.peek().is_some_and(|c| !c.is_whitespace()) {
                return Err("Expected whitespace after quoted argument".into());
            }
        } else {
            value.push(first);
            while chars.peek().is_some_and(|c| !c.is_whitespace()) {
                value.push(chars.next().unwrap());
            }
        }
        args.push(value);
    }
    Ok(args)
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct SaveSections {
    pub modules: bool,
    pub hud: bool,
    pub macros: bool,
    pub waypoints: bool,
}
impl SaveSections {
    fn validate(&self) -> Result<(), String> {
        if self.waypoints {
            return Err("Profile waypoints are not ported yet".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Snapshot {
    /// Includes the module registry (active state, keys, favorites) and each module's settings.
    pub modules: Option<Map<String, Value>>,
    pub hud: Option<Value>,
    pub macros: Option<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub name: String,
    #[serde(default)]
    pub load_on_join: Vec<String>,
    #[serde(default)]
    pub save: SaveSections,
    #[serde(default)]
    pub snapshot: Snapshot,
}

/// Every active module must deactivate before reactivating saved modules, including
/// those active before AND after loading. This is Modules.fromTag's disableAll lifecycle.
pub struct LoadPlan {
    pub name: String,
    pub candidate: Config,
    pub reload_modules: bool,
    /// Replace live macro objects, while existing delayed command references remain alive.
    pub reload_macros: bool,
    pub deactivate: Vec<String>,
    pub activate: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profiles {
    pub schema: u32,
    pub entries: Vec<Profile>,
}
impl Default for Profiles {
    fn default() -> Self {
        Self {
            schema: 1,
            entries: Vec::new(),
        }
    }
}

/// Successful loads remain ordered; an invalid profile is reported without losing other loads.
#[derive(Default)]
pub struct Autoload {
    pub loads: Vec<LoadPlan>,
    pub errors: Vec<(String, String)>,
}

fn module_sections(config: &Config) -> std::collections::BTreeSet<String> {
    let mut names = std::collections::BTreeSet::from(["modules".to_owned()]);
    for module in &config.modules.modules {
        names.insert(module.id.clone());
        names.insert(module.id.replace('-', "_"));
    }
    names
}
fn config_object(config: &Config) -> Result<Map<String, Value>, String> {
    serde_json::to_value(config)
        .map_err(|e| e.to_string())?
        .as_object()
        .cloned()
        .ok_or_else(|| "Invalid native configuration".into())
}
fn active(config: &Config) -> Vec<String> {
    config
        .modules
        .modules
        .iter()
        .filter(|module| module.active)
        .map(|module| module.id.clone())
        .collect()
}

impl Profile {
    pub fn validate(&self) -> Result<(), String> {
        // Upstream's fileNameFilter; native profiles use a single JSON store instead of folders.
        if self.name.trim().is_empty()
            || !self
                .name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_- ".contains(&b))
        {
            return Err("Profile names need letters, digits, spaces, '_' or '-'".into());
        }
        if self
            .load_on_join
            .iter()
            .any(|key| key.is_empty() || key.chars().any(char::is_control))
        {
            return Err(
                "Profile join keys must be nonempty world names or server addresses".into(),
            );
        }
        self.save.validate()
    }

    /// Save only checked sections. Unchecked sections keep their earlier snapshots, like
    /// upstream leaving the corresponding .nbt files on disk when a save checkbox is off.
    pub fn capture(&mut self, current: &Config) -> Result<(), String> {
        self.validate()?;
        let current = Config::from_json(&current.to_json().map_err(|e| e.to_string())?)?;
        let object = config_object(&current)?;
        let module_keys = module_sections(&current);
        let mut next = self.snapshot.clone();
        if self.save.modules {
            next.modules = Some(
                object
                    .iter()
                    .filter(|(name, _)| module_keys.contains(*name))
                    .map(|(name, value)| (name.clone(), value.clone()))
                    .collect(),
            );
        }
        if self.save.hud {
            next.hud = object.get("hud").cloned();
        }
        if self.save.macros {
            next.macros = object.get("macros").cloned();
        }
        self.snapshot = next;
        Ok(())
    }

    /// Validate the complete candidate before an adapter mutates live state. Missing saved
    /// sections are no-ops, just as System.load returns when its selected file is absent.
    pub fn load(&self, current: &Config) -> Result<LoadPlan, String> {
        self.validate()?;
        let mut object = config_object(current)?;
        let module_keys = module_sections(current);
        let reload_modules = self.save.modules && self.snapshot.modules.is_some();
        if let Some(saved) = self.snapshot.modules.as_ref().filter(|_| self.save.modules) {
            if !saved.contains_key("modules") {
                return Err("Profile module snapshot has no registry".into());
            }
            for (name, value) in saved {
                if !module_keys.contains(name) || !object.contains_key(name) {
                    return Err(format!("Unknown or non-module profile section: {name}"));
                }
                object.insert(name.clone(), value.clone());
            }
        }
        if let Some(saved) = self.snapshot.hud.as_ref().filter(|_| self.save.hud) {
            object.insert("hud".into(), saved.clone());
        }
        let reload_macros = self.save.macros && self.snapshot.macros.is_some();
        if let Some(saved) = self.snapshot.macros.as_ref().filter(|_| self.save.macros) {
            object.insert("macros".into(), saved.clone());
        }
        let candidate = Config::from_json(&Value::Object(object).to_string())?;
        Ok(LoadPlan {
            name: self.name.clone(),
            deactivate: if reload_modules {
                active(current)
            } else {
                Vec::new()
            },
            activate: if reload_modules {
                active(&candidate)
            } else {
                Vec::new()
            },
            candidate,
            reload_modules,
            reload_macros,
        })
    }
}

impl Profiles {
    pub fn get(&self, name: &str) -> Option<&Profile> {
        self.entries
            .iter()
            .find(|profile| profile.name.eq_ignore_ascii_case(name))
    }
    fn index(&self, name: &str) -> Result<usize, String> {
        self.entries
            .iter()
            .position(|profile| profile.name.eq_ignore_ascii_case(name))
            .ok_or_else(|| format!("Unknown profile: {name}"))
    }
    pub fn create(
        &mut self,
        name: String,
        save: SaveSections,
        load_on_join: Vec<String>,
        current: &Config,
    ) -> Result<(), String> {
        if self.get(&name).is_some() {
            return Err(format!("Profile already exists: {name}"));
        }
        let mut profile = Profile {
            name,
            save,
            load_on_join,
            snapshot: Snapshot::default(),
        };
        profile.capture(current)?;
        self.entries.push(profile);
        Ok(())
    }
    /// Edit metadata without overwriting saved state. Save/capture is an explicit separate action.
    pub fn configure(
        &mut self,
        name: &str,
        save: SaveSections,
        load_on_join: Vec<String>,
    ) -> Result<(), String> {
        let index = self.index(name)?;
        let mut next = self.entries[index].clone();
        next.save = save;
        next.load_on_join = load_on_join;
        next.validate()?;
        self.entries[index] = next;
        Ok(())
    }
    pub fn save(&mut self, name: &str, current: &Config) -> Result<(), String> {
        let index = self.index(name)?;
        self.entries[index].capture(current)
    }
    pub fn remove(&mut self, name: &str) -> Result<Profile, String> {
        let index = self.index(name)?;
        Ok(self.entries.remove(index))
    }
    pub fn load(&self, name: &str, current: &Config) -> Result<LoadPlan, String> {
        self.entries[self.index(name)?].load(current)
    }
    /// Pass Utils.getWorldName's equivalent: singleplayer folder, exact server address,
    /// or "realms". Matching is case-sensitive and exact; no globbing or implicit port changes.
    pub fn on_join(&self, world_name: &str, current: &Config) -> Autoload {
        let mut out = Autoload::default();
        let mut candidate = current.clone();
        for profile in &self.entries {
            if !profile.load_on_join.iter().any(|key| key == world_name) {
                continue;
            }
            match profile.load(&candidate) {
                Ok(load) => {
                    candidate = load.candidate.clone();
                    out.loads.push(load);
                }
                Err(error) => out.errors.push((profile.name.clone(), error)),
            }
        }
        out
    }
    pub fn to_json(&self) -> Result<String, String> {
        self.validate()?;
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }
    pub fn from_json(text: &str) -> Result<Self, String> {
        let profiles: Self = serde_json::from_str(text).map_err(|e| e.to_string())?;
        profiles.validate()?;
        Ok(profiles)
    }
    fn validate(&self) -> Result<(), String> {
        if self.schema != 1 {
            return Err("Unsupported native profiles schema".into());
        }
        let mut names = std::collections::BTreeSet::new();
        for profile in &self.entries {
            profile.validate()?;
            if !names.insert(profile.name.to_ascii_lowercase()) {
                return Err(format!("Duplicate profile name: {}", profile.name));
            }
        }
        Ok(())
    }
}
