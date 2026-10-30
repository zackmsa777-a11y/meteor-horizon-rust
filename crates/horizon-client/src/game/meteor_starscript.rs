// SPDX-License-Identifier: GPL-3.0-only
// Source-derived MeteorStarscript item/effect/camera/world bindings. Copyright Meteor Development.
use super::{Dim, Game};
use horizon_client::items::ItemStack;
use horizon_world::{identifier::Identifier, nbt::Tag};
use meteor_client::starscript::{Error, Value, ValueMap, meteor};
use std::{collections::BTreeMap, rc::Rc};

impl Game<'_> {
    /// Refresh from actual client state before each script line. No network/profile
    /// lookup or fabricated world facts; getters operate on this coherent snapshot.
    pub(super) fn meteor_starscript_extra(&mut self, globals: &ValueMap) {
        let set = |name: &str, value: Value| {
            let _ = globals.set(name, value);
        };
        let exists = self.player.is_some();
        set(
            "mc_version",
            Value::String(horizon_core::VANILLA_VERSION.into()),
        );
        let (dimension, opposite, scale) = match self.dim {
            Dim::Overworld => ("Overworld", "Nether", 0.125),
            Dim::Nether => ("Nether", "Overworld", 8.0),
            Dim::End => ("End", "End", 1.0),
        };
        set("player.dimension", Value::String(dimension.into()));
        set("player.opposite_dimension", Value::String(opposite.into()));
        if let Some((camera, yaw, pitch)) = self.camera_view(self.clock.camera_fraction() as f32) {
            set("camera.pos", meteor::position(camera.to_array()));
            set(
                "camera.opposite_dim_pos",
                meteor::position([camera.x * scale, camera.y, camera.z * scale]),
            );
            set(
                "camera.yaw",
                Value::Number(meteor_client::hud_metrics::angle(yaw) as f64),
            );
            set(
                "camera.pitch",
                Value::Number(meteor_client::hud_metrics::angle(pitch) as f64),
            );
            set("camera.direction", meteor::direction(yaw));
        }
        if let Some(player) = self.player.as_ref() {
            let p = player.position;
            set("player.pos", meteor::position([p.x, p.y, p.z]));
            set(
                "player.opposite_dim_pos",
                meteor::position([p.x * scale, p.y, p.z * scale]),
            );
            set("player.direction", meteor::direction(self.yaw));
            let (timer, multiplier) = self.meteor_hud_speed_scale();
            let factor = 20.0 * if timer { multiplier } else { 1.0 };
            let speed = [
                (p.x - self.previous_position.x) * factor,
                (p.y - self.previous_position.y) * factor,
                (p.z - self.previous_position.z) * factor,
            ];
            let vector = ValueMap::default();
            for (key, v) in ["x", "y", "z"].into_iter().zip(speed) {
                vector.set(key, Value::Number(v)).unwrap();
            }
            // Vec3.toString uses Java Double formatting. Values remain numeric as well.
            vector
                .set(
                    "_toString",
                    Value::String(format!(
                        "({}, {}, {})",
                        meteor_client::starscript::java_double_string(speed[0]),
                        meteor_client::starscript::java_double_string(speed[1]),
                        meteor_client::starscript::java_double_string(speed[2])
                    )),
                )
                .unwrap();
            set("player.speed_all", Value::Map(vector));
            let [x, y, z] = [p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32];
            if let Some(chunk) = self.world.chunks.get(&(x >> 4, z >> 4)) {
                let name = &self.res.biomes().get(chunk.biome(x, y, z)).name;
                let name = name
                    .split_once(':')
                    .map_or(name.as_str(), |(_, s)| s)
                    .split('_')
                    .map(|word| {
                        let mut c = word.chars();
                        c.next()
                            .map(|v| v.to_uppercase().collect::<String>() + c.as_str())
                            .unwrap_or_default()
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                set("player.biome", Value::String(name));
            } else {
                set("player.biome", Value::String("Unknown".into()));
            }
        }
        set(
            "player.breaking_progress",
            Value::Number(self.meteor_starscript_breaking_progress() as f64),
        );
        let hand =
            self.meteor_starscript_item(self.inventory.get(self.selected).and_then(Option::as_ref));
        let offhand = self.meteor_starscript_item(self.offhand.as_ref());
        set(
            "player.hand",
            if exists { hand.clone() } else { Value::Null },
        );
        set(
            "player.offhand",
            if exists { offhand.clone() } else { Value::Null },
        );
        let hand_empty = self
            .inventory
            .get(self.selected)
            .and_then(Option::as_ref)
            .is_none_or(|s| s.count == 0 || s.item.as_ref() == "minecraft:air");
        set(
            "player.hand_or_offhand",
            if !exists {
                Value::Null
            } else if hand_empty {
                offhand
            } else {
                hand
            },
        );
        let mut inventory = Vec::with_capacity(41);
        let mut counts = BTreeMap::<String, i32>::new();
        for index in 0..41 {
            let stack = match index {
                0..=35 => self.inventory.get(index).and_then(Option::as_ref),
                36..=39 => self.armor[39 - index].as_ref(),
                _ => self.offhand.as_ref(),
            };
            inventory.push(self.meteor_starscript_item(stack));
            if let Some(stack) = stack
                && stack.item.as_ref() != "minecraft:air"
                && self.items().items.contains_key(stack.item.as_ref())
            {
                let entry = counts.entry(stack.item.to_string()).or_default();
                *entry = entry.wrapping_add(stack.count as i32);
            }
        }
        let empty = meteor::item("", "minecraft:air", 0, 0, 0, false);
        set(
            "player.get_item",
            Value::Function(Rc::new(move |args| {
                arity(args, "player.get_item", 1)?;
                let index = args[0]
                    .number("First argument to player.get_item() needs to be a number.")?
                    as i32;
                if index < 0 {
                    return Err(Error(
                        "First argument to player.get_item() needs to be a non-negative integer."
                            .into(),
                    ));
                }
                Ok(if !exists {
                    Value::Null
                } else {
                    inventory.get(index as usize).unwrap_or(&empty).clone()
                })
            })),
        );
        set(
            "player.count_items",
            Value::Function(Rc::new(move |args| {
                arity(args, "player.count_items", 1)?;
                let id = args[0]
                    .string("First argument to player.count_items() needs to be a string.")?;
                let Ok(id) = Identifier::parse(id) else {
                    return Ok(Value::Number(0.0));
                };
                Ok(Value::Number(
                    counts.get(&id.to_string()).copied().unwrap_or(0) as f64,
                ))
            })),
        );
        let effects = self.potions.effects.list.clone();
        for name in ["has_potion_effect", "get_potion_effect"] {
            let effects = effects.clone();
            set(
                &format!("player.{name}"),
                Value::Function(Rc::new(move |args| {
                    if args.is_empty() {
                        return Err(Error(format!(
                            "player.{name}() requires 1 argument, got 0."
                        )));
                    }
                    if !exists {
                        return Ok(if name == "has_potion_effect" {
                            Value::Bool(false)
                        } else {
                            Value::Null
                        });
                    }
                    let id = args.last().unwrap().string(&format!(
                        "First argument to player.{name}() needs to a string."
                    ))?;
                    let id = Identifier::parse(id).map_err(|e| Error(e.to_string()))?;
                    let effect = if id.namespace() == "minecraft" {
                        effects.iter().find(|e| e.id == id.path())
                    } else {
                        None
                    };
                    Ok(if name == "has_potion_effect" {
                        Value::Bool(effect.is_some())
                    } else {
                        effect.map_or(Value::Null, |e| meteor::effect(e.duration, e.amplifier))
                    })
                })),
            );
        }
        let world_name = if let Some(server) = &self.integrated {
            server.world_folder.clone()
        } else if let Some(remote) = &self.remote {
            remote.session.address.clone()
        } else if exists {
            self.save
                .root
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default()
        } else {
            String::new()
        };
        set("server._toString", Value::String(world_name));
        set("server.tps", Value::Number(self.meteor_hud_tps() as f64));
        let time = self.world.day_time.wrapping_add(6000).rem_euclid(24000);
        set(
            "server.time",
            Value::String(if exists {
                format!("{:02}:{:02}", time / 1000, (time % 1000 * 60) / 1000)
            } else {
                "00:00".into()
            }),
        );
        let (rain, thunder) = self.world.weather.levels(1.0);
        set(
            "server.weather",
            Value::String(
                if !exists {
                    ""
                } else if thunder > 0.9 {
                    "Thunder"
                } else if rain > 0.2 {
                    "Rain"
                } else {
                    "Clear"
                }
                .into(),
            ),
        );
        set(
            "server.player_count",
            Value::Number(self.meteor_starscript_player_count() as f64),
        );
        set(
            "server.difficulty",
            Value::String(
                if !exists {
                    ""
                } else {
                    match self.ui.difficulty {
                        0 => "PEACEFUL",
                        1 => "EASY",
                        2 => "NORMAL",
                        _ => "HARD",
                    }
                }
                .into(),
            ),
        );
    }
    fn meteor_starscript_item(&self, stack: Option<&ItemStack>) -> Value {
        let Some(stack) = stack.filter(|s| s.count > 0 && s.item.as_ref() != "minecraft:air")
        else {
            return meteor::item("", "minecraft:air", 0, 0, 0, false);
        };
        let info = self.items().items.get(stack.item.as_ref());
        let (namespace, path) = stack
            .item
            .split_once(':')
            .unwrap_or(("minecraft", stack.item.as_ref()));
        let key = format!(
            "{}.{namespace}.{path}",
            if info.is_some_and(|i| i.block.is_some()) {
                "block"
            } else {
                "item"
            }
        );
        let name = self
            .pack
            .names
            .get(&key)
            .map(|v| v.to_string())
            .unwrap_or(key);
        let max_damage = match stack.data().and_then(|d| d.other("minecraft:max_damage")) {
            Some(Tag::Int(n)) => *n,
            _ => info.map_or(0, |i| i.max_damage as i32),
        };
        let unbreakable = stack
            .data()
            .is_some_and(|d| d.other("minecraft:unbreakable").is_some());
        // Horizon retains custom component values, but not every removal's presence bit.
        // Standard registry damageability and explicit unbreakable are fully represented.
        let damageable = max_damage > 0 && !unbreakable;
        meteor::item(
            &strip_color(&name),
            stack.item.as_ref(),
            stack.count as i32,
            stack.damage as i32,
            max_damage,
            damageable,
        )
    }
}
fn arity(args: &[Value], name: &str, n: usize) -> Result<(), Error> {
    if args.len() == n {
        Ok(())
    } else {
        Err(Error(format!(
            "{name}() requires {n} argument{}, got {}.",
            if n == 1 { "" } else { "s" },
            args.len()
        )))
    }
}
fn strip_color(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '§'
            && chars
                .peek()
                .is_some_and(|c| matches!(c.to_ascii_lowercase(),'0'..='9'|'a'..='f'|'k'..='o'|'r'))
        {
            chars.next();
        } else {
            out.push(c);
        }
    }
    out
}
