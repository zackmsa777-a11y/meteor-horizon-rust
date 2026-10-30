// SPDX-License-Identifier: GPL-3.0-only
// MeteorStarscript host bindings; Starscript language itself retains its MIT license.
use super::ValueMap;
use super::{Error, Value};
use crate::Config;

pub fn position([x, y, z]: [f64; 3]) -> Value {
    let map = ValueMap::default();
    for (name, n) in [("x", x), ("y", y), ("z", z)] {
        map.set(name, Value::Number(n)).unwrap();
    }
    map.set(
        "_toString",
        Value::String(format!(
            "X: {:.0} Y: {:.0} Z: {:.0}",
            x.round(),
            y.round(),
            z.round()
        )),
    )
    .unwrap();
    Value::Map(map)
}
pub fn direction(yaw: f32) -> Value {
    let name = crate::hud_metrics::direction(yaw);
    let axis = match name {
        "South" => "Z+",
        "South West" => "X- Z+",
        "West" => "X-",
        "North West" => "X- Z-",
        "North" => "Z-",
        "North East" => "X+ Z-",
        "East" => "X+",
        _ => "X+ Z+",
    };
    let map = ValueMap::default();
    map.set("name", Value::String(name.into())).unwrap();
    map.set("axis", Value::String(axis.into())).unwrap();
    map.set("_toString", Value::String(format!("{name} {axis}")))
        .unwrap();
    Value::Map(map)
}
/// The host supplies the real registry name and actual component-derived durability.
pub fn item(
    name: &str,
    id: &str,
    count: i32,
    damage: i32,
    max_damage: i32,
    damageable: bool,
) -> Value {
    let map = ValueMap::default();
    let empty = count <= 0 || id == "minecraft:air";
    let (name, id, count, max_damage) = if empty {
        ("", "minecraft:air", 0, 0)
    } else {
        (name, id, count, max_damage)
    };
    let durability = if empty || !damageable {
        0
    } else {
        max_damage.wrapping_sub(damage)
    };
    map.set(
        "_toString",
        Value::String(if count <= 1 {
            name.into()
        } else {
            format!("{name} {count}x")
        }),
    )
    .unwrap();
    map.set("name", Value::String(name.into())).unwrap();
    map.set("id", Value::String(id.into())).unwrap();
    for (key, n) in [
        ("count", count),
        ("durability", durability),
        ("max_durability", max_damage),
    ] {
        map.set(key, Value::Number(n as f64)).unwrap();
    }
    Value::Map(map)
}
pub fn effect(duration: i32, amplifier: i32) -> Value {
    let map = ValueMap::default();
    map.set("duration", Value::Number(duration as f64)).unwrap();
    map.set("level", Value::Number(amplifier.wrapping_add(1) as f64))
        .unwrap();
    Value::Map(map)
}
/// Production binding shared by native Game and portable regression tests.
pub fn get_module_setting(config: &Config, args: &[Value]) -> Result<Value, Error> {
    if args.len() != 2 {
        return Err(Error(format!(
            "meteor.get_module_setting() requires 2 arguments, got {}.",
            args.len()
        )));
    }
    let setting =
        args[1].string("Second argument to meteor.get_module_setting() needs to be a string.")?;
    let name =
        args[0].string("First argument to meteor.get_module_setting() needs to be a string.")?;
    let module = config
        .modules
        .modules
        .iter()
        .find(|m| m.id.eq_ignore_ascii_case(name))
        .ok_or_else(|| {
            Error(format!(
                "Unable to get module {name} for meteor.get_module_setting()"
            ))
        })?;
    let json = serde_json::to_value(config).map_err(|e| Error(e.to_string()))?;
    let value = json
        .get(module.id.replace('-', "_"))
        .and_then(serde_json::Value::as_object)
        .and_then(|values| {
            values
                .iter()
                .find(|(field, _)| Config::source_setting_name(&module.id, field) == setting)
                .map(|(_, value)| value)
        })
        .ok_or_else(|| {
            Error(format!(
                "Unable to get setting {setting} for module {name} for meteor.get_module_setting()"
            ))
        })?;
    match value {
        serde_json::Value::Bool(v) => Ok(Value::Bool(*v)),
        serde_json::Value::Number(v) => {
            Ok(Value::Number(v.as_f64().ok_or_else(|| {
                Error("Setting is not representable as a double".into())
            })?))
        }
        serde_json::Value::String(v) => Ok(Value::String(v.clone())),
        serde_json::Value::Array(v) => Ok(Value::Number(v.len() as f64)),
        _ => Err(Error(
            "This native setting type has no Starscript representation yet.".into(),
        )),
    }
}
