// SPDX-License-Identifier: GPL-3.0-only
// Native Macros/MacroCommand adapter. Copyright (c) Meteor Development.
use super::Game;
use meteor_client::{
    macros::{Macro, Macros, Schedule},
    registry::KeyEvent,
    starscript::{self, Environment, Script, Value},
};
use std::{cell::RefCell, rc::Rc};
type Handle = Rc<RefCell<Entry>>;
struct Entry {
    value: Macro,
    scripts: Option<Vec<Script>>,
}
pub(super) struct State {
    loaded: bool,
    entries: Vec<Handle>,
    schedule: Schedule<Handle>,
    pub(super) environment: Environment,
    pub(super) executions: u64,
    pub(super) dispatched: u64,
    pub(super) last_error: Option<String>,
}
impl Default for State {
    fn default() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |v| v.as_nanos() as u64);
        Self {
            loaded: false,
            entries: vec![],
            schedule: Default::default(),
            environment: Environment::standard(
                seed,
                Rc::new(|| Value::String(chrono::Local::now().format("%H:%M").to_string())),
                Rc::new(|| Value::String(chrono::Local::now().format("%d. %m. %Y").to_string())),
            ),
            executions: 0,
            dispatched: 0,
            last_error: None,
        }
    }
}
impl Game<'_> {
    /// Macros.fromTag clears/recreates live entries, but MacroCommand retains scheduled
    /// references to old objects. Do not reset the independent queue or shared RNG.
    pub(super) fn meteor_macros_reload_from_config(&mut self) {
        self.meteor_macros.loaded = true;
        self.meteor_macros.entries = self
            .meteor
            .client
            .config
            .macros
            .macros
            .iter()
            .cloned()
            .map(|value| {
                Rc::new(RefCell::new(Entry {
                    value,
                    scripts: None,
                }))
            })
            .collect();
    }
    fn meteor_macros_load(&mut self) {
        if self.meteor_macros.loaded {
            return;
        }
        self.meteor_macros_reload_from_config();
    }
    fn meteor_macros_save(&mut self) {
        self.meteor.client.config.macros = Macros {
            macros: self
                .meteor_macros
                .entries
                .iter()
                .map(|entry| entry.borrow().value.clone())
                .collect(),
        };
        self.meteor_save();
    }
    pub(super) fn meteor_macros_rows(&mut self) -> Vec<Macro> {
        self.meteor_macros_load();
        self.meteor_macros
            .entries
            .iter()
            .map(|entry| entry.borrow().value.clone())
            .collect()
    }
    pub(super) fn meteor_macro_edit(
        &mut self,
        index: Option<usize>,
        value: Macro,
    ) -> Result<(), String> {
        self.meteor_macros_load();
        value.validate_editor()?;
        let mut candidate = self.meteor.client.config.macros.clone();
        if let Some(index) = index {
            *candidate
                .macros
                .get_mut(index)
                .ok_or("Macro no longer exists")? = value.clone();
        } else {
            candidate.macros.push(value.clone());
        }
        candidate.validate_storage()?;
        if value.name.len() > 4096
            || value.messages.len() > 1024
            || value.messages.iter().any(|s| s.len() > 65536)
        {
            return Err("Macro exceeds the native storage limit".into());
        }
        if let Some(index) = index {
            let mut entry = self
                .meteor_macros
                .entries
                .get(index)
                .ok_or("Macro no longer exists")?
                .borrow_mut();
            if entry.value.messages != value.messages {
                entry.scripts = None;
            }
            entry.value = value;
        } else {
            if self
                .meteor_macros
                .entries
                .iter()
                .any(|e| e.borrow().value == value)
            {
                return Err("A macro with that exact name already exists".into());
            }
            self.meteor_macros.entries.push(Rc::new(RefCell::new(Entry {
                value,
                scripts: None,
            })));
        }
        self.meteor_macros_save();
        Ok(())
    }
    pub(super) fn meteor_macro_remove(&mut self, index: usize) -> Result<(), String> {
        self.meteor_macros_load();
        if index >= self.meteor_macros.entries.len() {
            return Err("Macro no longer exists".into());
        }
        let name = self.meteor_macros.entries[index]
            .borrow()
            .value
            .name
            .clone();
        let index = self
            .meteor_macros
            .entries
            .iter()
            .position(|entry| entry.borrow().value.name == name)
            .ok_or("Macro no longer exists")?;
        // Queued handles survive list removal, exactly as ScheduledMacro's object reference.
        self.meteor_macros.entries.remove(index);
        self.meteor_macros_save();
        Ok(())
    }
    pub(super) fn meteor_macros_input(&mut self, event: &KeyEvent) {
        if !event.pressed || event.screen_open || self.player.is_none() {
            return;
        }
        self.meteor_macros_load();
        let selected = self
            .meteor_macros
            .entries
            .iter()
            .find(|entry| entry.borrow().value.keybind.matches(event))
            .cloned();
        if let Some(entry) = selected {
            self.meteor_macro_run(entry);
        }
    }
    pub(super) fn meteor_macros_post(&mut self) {
        if !self.native_window() || !self.native_on("meteor-client") {
            return;
        }
        self.meteor_macros_load();
        let due = self.meteor_macros.schedule.post_tick(self.player.is_some());
        for entry in due {
            self.meteor_macro_run(entry);
        }
    }
    fn meteor_macro_error(&mut self, message: String) {
        self.meteor_macros.last_error = Some(message.clone());
        self.native_chat(&format!("[Starscript] {message}"));
    }
    fn meteor_macro_run(&mut self, handle: Handle) {
        if self.player.is_none() {
            return;
        }
        self.meteor_macros.executions += 1;
        let mut errors = vec![];
        let scripts = {
            let mut entry = handle.borrow_mut();
            if entry.scripts.is_none() {
                let mut scripts = vec![];
                for (index, message) in entry.value.messages.iter().enumerate() {
                    match starscript::compile(message) {
                        Ok(script) => scripts.push(script),
                        Err(diagnostics) => {
                            for d in diagnostics {
                                errors.push(format!(
                                    "{} line {}, character {}: {}",
                                    entry.value.name,
                                    index + 1,
                                    d.offset,
                                    d.message
                                ));
                            }
                        }
                    }
                }
                entry.scripts = Some(scripts);
            }
            entry.scripts.clone().unwrap_or_default()
        };
        for error in errors {
            self.meteor_macro_error(error);
        }
        // Refresh before EACH line: an earlier generated command can mutate game state.
        for script in scripts {
            self.meteor_starscript_refresh();
            match self.meteor_macros.environment.run(&script) {
                Ok(output) => {
                    self.meteor_macros.dispatched += 1;
                    self.meteor_macro_dispatch(&output.text());
                }
                Err(error) => self.meteor_macro_error(error.to_string()),
            }
        }
    }
    fn meteor_macro_dispatch(&mut self, message: &str) {
        // ChatUtils.sendPlayerMsg(message,false): use actual network/local command paths,
        // without adding to chat history. Runtime tests only use fresh singleplayer worlds.
        // Honor Horizon's actual native command aliases before server dispatch.
        if let Some(command) = message.strip_prefix('/')
            && self.client_mod_command(command)
        {
            return;
        }
        if self.remote.is_some() {
            self.send_chat(message);
        } else if let Some(command) = message.strip_prefix('/') {
            self.run_command(command);
        } else {
            self.player_message(horizon_client::text::Text::translate(
                "chat.type.text",
                &[
                    self.player_display_name(),
                    horizon_client::text::Text::literal(message),
                ],
            ));
        }
    }
    pub(super) fn meteor_starscript_refresh(&mut self) {
        let globals = self.meteor_macros.environment.globals.clone();
        let set = |name: &str, value: Value| {
            let _ = globals.set(name, value);
        };
        set("fps", Value::Number(self.fps as i32 as f64));
        set("ping", Value::Number(self.meteor_hud_ping() as f64));
        set("meteor.name", Value::String("Meteor".into()));
        set("meteor.version", Value::String("native-development".into()));
        set(
            "meteor.modules",
            Value::Number(self.meteor.client.config.modules.modules.len() as f64),
        );
        set(
            "meteor.active_modules",
            Value::Number(
                self.meteor
                    .client
                    .config
                    .modules
                    .modules
                    .iter()
                    .filter(|m| m.active)
                    .count() as f64,
            ),
        );
        set(
            "player._toString",
            Value::String(self.options.player_name()),
        );
        if let Some(player) = self.player.as_ref() {
            for (name, n) in [
                ("health", player.health.health() as f64),
                ("absorption", player.health.absorption() as f64),
                ("hunger", player.health.food.food as f64),
                ("saturation", player.health.food.saturation as f64),
                ("pos.x", player.position.x),
                ("pos.y", player.position.y),
                ("pos.z", player.position.z),
            ] {
                set(&format!("player.{name}"), Value::Number(n));
            }
            set(
                "player.pos._toString",
                Value::String(format!(
                    "X: {:.0} Y: {:.0} Z: {:.0}",
                    player.position.x.round(),
                    player.position.y.round(),
                    player.position.z.round()
                )),
            );
            set(
                "player.yaw",
                Value::Number(meteor_client::hud_metrics::angle(self.yaw) as f64),
            );
            set(
                "player.pitch",
                Value::Number(meteor_client::hud_metrics::angle(self.pitch) as f64),
            );
            let direction = meteor_client::hud_metrics::direction(self.yaw);
            let axis = match direction {
                "South" => "Z+",
                "South West" => "X- Z+",
                "West" => "X-",
                "North West" => "X- Z-",
                "North" => "Z-",
                "North East" => "X+ Z-",
                "East" => "X+",
                _ => "X+ Z+",
            };
            set("player.direction.name", Value::String(direction.into()));
            set("player.direction.axis", Value::String(axis.into()));
            set(
                "player.direction._toString",
                Value::String(format!("{direction} {axis}")),
            );
            let mode = self.mode.name();
            let mut chars = mode.chars();
            let mode = chars
                .next()
                .map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
                .unwrap_or_default();
            set("player.gamemode", Value::String(mode));
            set(
                "player.xp.level",
                Value::Number(self.experience.level as f64),
            );
            set(
                "player.xp.progress",
                Value::Number(self.experience.progress as f64),
            );
            set(
                "player.xp.total",
                Value::Number(self.experience.total as f64),
            );
            let (active, multiplier) = self.meteor_hud_speed_scale();
            let speed = meteor_client::hud_metrics::speed(
                [player.position.x, player.position.y, player.position.z],
                [
                    self.previous_position.x,
                    self.previous_position.y,
                    self.previous_position.z,
                ],
                active,
                multiplier,
            );
            set("player.speed", Value::Number(speed));
        }
        let modules = self.meteor.client.config.modules.modules.clone();
        set(
            "meteor.is_module_active",
            Value::Function(Rc::new(move |args| {
                if args.len() != 1 {
                    return Err(starscript::Error(format!(
                        "meteor.is_module_active() requires 1 argument, got {}.",
                        args.len()
                    )));
                }
                let name = args[0]
                    .string("First argument to meteor.is_module_active() needs to be a string.")?;
                Ok(Value::Bool(
                    modules
                        .iter()
                        .any(|m| m.id.eq_ignore_ascii_case(name) && m.active),
                ))
            })),
        );
        let config = self.meteor.client.config.clone();
        set(
            "meteor.get_module_setting",
            Value::Function(Rc::new(move |args| {
                starscript::meteor::get_module_setting(&config, args)
            })),
        );
        self.meteor_starscript_extra(&globals);
        // Unsupported host functions/variables remain absent (Starscript null), rather
        // than fabricating values for Baritone, skins, account services or statistics.
    }
    pub(super) fn meteor_macros_status(&self) -> serde_json::Value {
        serde_json::json!({"executions":self.meteor_macros.executions,"dispatched":self.meteor_macros.dispatched,"last_error":self.meteor_macros.last_error})
    }
    pub(super) fn meteor_macros_command(&mut self, source: &str) -> Result<String, String> {
        self.meteor_macros_load();
        let source = source.trim();
        let (verb, rest) = source
            .split_once(char::is_whitespace)
            .unwrap_or((source, ""));
        let rest = rest.trim();
        match verb {
            "list"=>Ok(self.meteor_macros.entries.iter().enumerate().map(|(i,e)|format!("{i}: {}",e.borrow().value.name)).collect::<Vec<_>>().join(", ")),
            "add"=>{let value=serde_json::from_str::<Macro>(rest).map_err(|e|format!("Expected a macro JSON object: {e}"))?;self.meteor_macro_edit(None,value)?;Ok("Created macro".into())},
            "edit"=>{let (index,json)=rest.split_once(char::is_whitespace).ok_or("Expected INDEX followed by macro JSON")?;let index=index.parse().map_err(|_|"Invalid macro index")?;let value=serde_json::from_str::<Macro>(json.trim()).map_err(|e|e.to_string())?;self.meteor_macro_edit(Some(index),value)?;Ok("Saved macro".into())},
            "remove"=>{self.meteor_macro_remove(rest.parse().map_err(|_|"Expected macro index")?)?;Ok("Removed macro".into())},
            "clear" if rest.is_empty()=>{if !self.meteor_macros.schedule.contains(|_|true){return Err("No macros are currently scheduled".into());}self.meteor_macros.schedule.clear_all();Ok("Cleared scheduled macros".into())},
            "run"|"clear"=>{
                let (name,delay)=parse_name_delay(rest)?;
                let model=self.meteor.client.config.macros.get(&name).ok_or("Unknown macro")?;
                let handle=self.meteor_macros.entries.iter().find(|entry|entry.borrow().value.name==model.name).cloned().ok_or("Unknown macro")?;
                if verb=="clear" {if !self.meteor_macros.schedule.contains(|entry|Rc::ptr_eq(entry,&handle)){return Err("This macro is not currently scheduled".into());}self.meteor_macros.schedule.clear(|entry|Rc::ptr_eq(entry,&handle));Ok("Cleared scheduled macro".into())}
                else {self.meteor_macros.schedule.queue(delay,handle);Ok(format!("Scheduled {name} in {delay} ticks"))}
            },
            _=>Err("Macros: list; add JSON; edit INDEX JSON; remove INDEX; run NAME [DELAY]; clear [NAME]".into())
        }
    }
}
fn parse_name_delay(source: &str) -> Result<(String, u32), String> {
    let (name, rest) = if source.starts_with('"') {
        let mut stream = serde_json::Deserializer::from_str(source).into_iter::<String>();
        let name = stream
            .next()
            .ok_or("Expected macro name")?
            .map_err(|e| e.to_string())?;
        (name, source[stream.byte_offset()..].trim())
    } else {
        let (name, rest) = source
            .split_once(char::is_whitespace)
            .unwrap_or((source, ""));
        (name.to_owned(), rest.trim())
    };
    if name.is_empty() {
        return Err("Expected macro name".into());
    }
    let delay = if rest.is_empty() {
        0
    } else {
        let (number, scale) = if let Some(n) = rest.strip_suffix('s') {
            (n, 20.0)
        } else if let Some(n) = rest.strip_suffix('d') {
            (n, 24000.0)
        } else {
            (rest.strip_suffix('t').unwrap_or(rest), 1.0)
        };
        let number = number
            .parse::<f32>()
            .map_err(|_| "Invalid delay; use ticks, s, or d")?;
        if !number.is_finite() || number < 0.0 {
            return Err("Delay must be finite and non-negative".into());
        }
        (number.mul_add(scale, 0.5).floor() as i32) as u32
    };
    Ok((name, delay))
}
