// SPDX-License-Identifier: GPL-3.0-only
// Native adapter for Meteor Profiles/Profile and ProfilesCommand.
use super::Game;
use meteor_client::profiles::{LoadPlan, Profiles, SaveSections};
use std::io::Read;

const HELP: &str = "Profiles: list; key; create NAME [modules,hud,macros]; save|load|delete|show NAME; sections NAME modules,hud,macros|none; joins NAME [EXACT_WORLD_OR_SERVER ...]. Quote names containing spaces. Native JSON only; waypoints and Java profile NBT are not supported yet.";
const MAX_STORE: u64 = 8 * 1024 * 1024;

impl Game<'_> {
    pub(super) fn meteor_profiles_read(&self) -> Result<Profiles, String> {
        let path = self.native_data_dir("meteor-client").join("profiles.json");
        let file = match std::fs::File::open(&path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Profiles::default());
            }
            Err(error) => return Err(format!("Cannot read profiles: {error}")),
        };
        let mut text = String::new();
        file.take(MAX_STORE + 1)
            .read_to_string(&mut text)
            .map_err(|e| e.to_string())?;
        if text.len() as u64 > MAX_STORE {
            return Err("Profiles store exceeds 8 MiB".into());
        }
        Profiles::from_json(&text)
    }
    pub(super) fn meteor_profiles_write(&self, profiles: &Profiles) -> Result<(), String> {
        let text = profiles.to_json()?;
        if text.len() as u64 > MAX_STORE {
            return Err("Profiles store exceeds 8 MiB".into());
        }
        let dir = self.native_data_dir("meteor-client");
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let path = dir.join("profiles.json");
        let temp = dir.join("profiles.pending.json");
        std::fs::write(&temp, text).map_err(|e| e.to_string())?;
        if path.exists() {
            std::fs::copy(&path, dir.join("profiles.previous.json")).map_err(|e| e.to_string())?;
        }
        std::fs::rename(temp, path).map_err(|e| e.to_string())
    }
    fn meteor_profiles_apply(&mut self, plan: LoadPlan) -> Result<(), String> {
        if self.meteor.load_failed {
            return Err("Existing Meteor configuration is unreadable; profile loading is disabled to preserve it.".into());
        }
        if plan.reload_modules {
            self.meteor_apply_config(plan.candidate)?;
        } else {
            // HUD-only profiles must not restart Zoom or other active modules.
            self.meteor.client.config.hud = plan.candidate.hud;
            self.meteor.client.config.macros = plan.candidate.macros;
        }
        if plan.reload_macros {
            self.meteor_macros_reload_from_config();
        }
        self.meteor_save();
        Ok(())
    }
    fn meteor_profiles_key(&self) -> Option<String> {
        if let Some(server) = &self.integrated {
            return Some(server.world_folder.clone());
        }
        if let Some(remote) = &self.remote {
            return Some(remote.session.address.clone());
        }
        if self.player.is_some() && self.title.is_none() {
            return self
                .save
                .root
                .file_name()
                .map(|s| s.to_string_lossy().into_owned());
        }
        None
    }
    pub(super) fn meteor_profiles_join(&mut self, in_world: bool) {
        if !in_world {
            self.meteor.profiles_world = None;
            return;
        }
        let Some(key) = self.meteor_profiles_key() else {
            return;
        };
        if self.meteor.profiles_world.as_ref() == Some(&key) {
            return;
        }
        self.meteor.profiles_world = Some(key.clone());
        let profiles = match self.meteor_profiles_read() {
            Ok(profiles) => profiles,
            Err(error) => {
                self.native_chat(&format!("[Meteor profiles] {error}"));
                return;
            }
        };
        // Resolve each candidate against the last successfully applied live state. Engine
        // registry validation can reject a profile even after pure settings validation.
        for profile in profiles
            .entries
            .iter()
            .filter(|p| p.load_on_join.contains(&key))
        {
            match profile
                .load(&self.meteor.client.config)
                .and_then(|p| self.meteor_profiles_apply(p))
            {
                Ok(()) => self.native_chat(&format!("[Meteor profiles] Loaded {}", profile.name)),
                Err(error) => {
                    self.native_chat(&format!("[Meteor profiles] {}: {error}", profile.name))
                }
            }
        }
    }
    pub(super) fn meteor_profiles_command(&mut self, text: &str) -> Result<String, String> {
        let args = meteor_client::profiles::arguments(text)?;
        let words: Vec<_> = args.iter().map(String::as_str).collect();
        if words == ["key"] {
            return self
                .meteor_profiles_key()
                .map(|key| format!("Exact join key: {key}"))
                .ok_or("Join a world first".into());
        }
        let mut profiles = self.meteor_profiles_read()?;
        let message = match words.as_slice() {
            [] | ["help"] => return Ok(HELP.into()),
            ["list"] => {
                return Ok(format!(
                    "Profiles: {}",
                    profiles
                        .entries
                        .iter()
                        .map(|p| p.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            ["show", name] => {
                return serde_json::to_string_pretty(profiles.get(name).ok_or("Unknown profile")?)
                    .map_err(|e| e.to_string());
            }
            ["create", name] | ["create", name, _] => {
                if self.meteor.load_failed {
                    return Err("Cannot capture an unreadable configuration".into());
                }
                let save = sections(words.get(2).copied().unwrap_or("none"))?;
                profiles.create((*name).into(), save, Vec::new(), &self.meteor.client.config)?;
                format!("Created profile {name}")
            }
            ["save", name] => {
                if self.meteor.load_failed {
                    return Err("Cannot capture an unreadable configuration".into());
                }
                profiles.save(name, &self.meteor.client.config)?;
                format!("Saved profile {name}")
            }
            ["load", name] => {
                let plan = profiles.load(name, &self.meteor.client.config)?;
                self.meteor_profiles_apply(plan)?;
                return Ok(format!("Loaded profile {name}"));
            }
            ["delete", name] => {
                profiles.remove(name)?;
                format!("Deleted profile {name}")
            }
            ["sections", name, flags] => {
                let old = profiles.get(name).ok_or("Unknown profile")?;
                profiles.configure(name, sections(flags)?, old.load_on_join.clone())?;
                format!("Updated profile {name} sections; use save to recapture")
            }
            ["joins", name, keys @ ..] => {
                let old = profiles.get(name).ok_or("Unknown profile")?;
                profiles.configure(
                    name,
                    old.save.clone(),
                    keys.iter().map(|s| (*s).to_owned()).collect(),
                )?;
                format!("Updated profile {name} exact join keys")
            }
            _ => return Err(HELP.into()),
        };
        self.meteor_profiles_write(&profiles)?;
        Ok(message)
    }
}

fn sections(text: &str) -> Result<SaveSections, String> {
    let mut save = SaveSections::default();
    for name in text.split(',') {
        match name {
            "none" if text == "none" => {}
            "modules" => save.modules = true,
            "hud" => save.hud = true,
            "macros" => save.macros = true,
            "waypoints" => {
                return Err("Profile waypoints are not ported yet".into());
            }
            _ => return Err("Sections must be modules,hud,macros or none".into()),
        }
    }
    Ok(save)
}
