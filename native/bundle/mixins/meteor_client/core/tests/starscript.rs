use meteor_client::starscript;
use starscript::{Environment, Value, compile};
use std::rc::Rc;
#[path = "fixtures/starscript-0.2.5.rs"]
mod fixtures;
fn env() -> Environment {
    let e = Environment::standard(
        123,
        Rc::new(|| Value::String("12:34".into())),
        Rc::new(|| Value::String("30. 10. 2026".into())),
    );
    e.globals
        .set("player.name", Value::String("Zack".into()))
        .unwrap();
    e.globals.set("player.health", Value::Number(20.0)).unwrap();
    e
}
#[test]
fn real_module_setting_binding_uses_source_names_and_current_values() {
    use std::cell::RefCell;
    let config = Rc::new(RefCell::new(meteor_client::Config::default()));
    let captured = config.clone();
    let e = env();
    e.globals
        .set(
            "meteor.get_module_setting",
            Value::Function(Rc::new(move |args| {
                starscript::meteor::get_module_setting(&captured.borrow(), args)
            })),
        )
        .unwrap();
    let script=compile("{meteor.get_module_setting('zoom','hide-HUD')} {meteor.get_module_setting('sprint','sprint-mode')}").unwrap();
    config.borrow_mut().set("zoom", "hide-HUD", "true").unwrap();
    assert_eq!(e.run(&script).unwrap().text(), "true Strict");
    config
        .borrow_mut()
        .set("zoom", "hide-HUD", "false")
        .unwrap();
    config
        .borrow_mut()
        .set("sprint", "sprint-mode", "Rage")
        .unwrap();
    assert_eq!(e.run(&script).unwrap().text(), "false Rage");
    assert!(
        e.run(&compile("{meteor.get_module_setting('zoom','hide_hud')}").unwrap())
            .is_err()
    );
    assert!(
        e.run(&compile("{meteor.get_module_setting('missing','hide-HUD')}").unwrap())
            .is_err()
    );
}
#[test]
fn host_wrappers_preserve_item_effect_and_position_fields() {
    use starscript::meteor;
    let e = env();
    e.globals
        .set(
            "stack",
            meteor::item(
                "Diamond Pickaxe",
                "minecraft:diamond_pickaxe",
                2,
                7,
                1561,
                true,
            ),
        )
        .unwrap();
    e.globals
        .set(
            "empty",
            meteor::item("Stone", "minecraft:stone", 0, 0, 0, false),
        )
        .unwrap();
    e.globals
        .set(
            "unbreakable",
            meteor::item("Pickaxe", "minecraft:diamond_pickaxe", 1, 7, 1561, false),
        )
        .unwrap();
    e.globals.set("effect", meteor::effect(-1, 2)).unwrap();
    e.globals
        .set("position", meteor::position([1.5, -2.5, 3.25]))
        .unwrap();
    e.globals.set("direction", meteor::direction(90.)).unwrap();
    let script=compile("{stack}|{stack.id}|{stack.durability}/{stack.max_durability}|{empty.id}:{empty.count}|{unbreakable.durability}|{effect.duration}:{effect.level}|{position}|{position.z}|{direction}:{direction.axis}").unwrap();
    assert_eq!(
        e.run(&script).unwrap().text(),
        "Diamond Pickaxe 2x|minecraft:diamond_pickaxe|1554/1561|minecraft:air:0|0|-1:3|X: 2 Y: -3 Z: 3|3.25|West X-:X-"
    );
    assert_eq!(starscript::java_double_string(0.0001), "1.0E-4");
    assert_eq!(starscript::java_double_string(0.), "0.0");
}
#[test]
fn official_java_differential_fixtures() {
    let fixtures: serde_json::Value = serde_json::from_str(fixtures::JSON).unwrap();
    for row in fixtures.as_array().unwrap() {
        let source = row["source"].as_str().unwrap();
        match compile(source) {
            Err(e) => {
                assert_eq!(row["kind"], "parse", "{source}: {e:?}");
                assert_eq!(row["message"], e[0].message, "{source}");
            }
            Ok(script) => match env().run(&script) {
                Err(e) => {
                    assert_eq!(row["kind"], "error", "{source}: {e:?}");
                    assert_eq!(row["message"], e.0, "{source}");
                }
                Ok(output) => {
                    assert_eq!(row["kind"], "ok", "{source}: {output:?}");
                    assert_eq!(row["text"], output.text(), "{source}");
                    let sections = output
                        .sections
                        .iter()
                        .map(|s| serde_json::json!({"index":s.index,"text":s.text}))
                        .collect::<Vec<_>>();
                    assert_eq!(row["sections"], serde_json::json!(sections), "{source}");
                }
            },
        }
    }
}
#[test]
fn suppliers_and_compiled_scripts_stay_live() {
    let e = env();
    let state = Rc::new(std::cell::Cell::new(1.0));
    let copy = state.clone();
    e.globals
        .set_supplier("counter", Rc::new(move || Value::Number(copy.get())))
        .unwrap();
    let s = compile("{counter} {player.name}").unwrap();
    assert_eq!(e.run(&s).unwrap().text(), "1 Zack");
    state.set(2.0);
    assert_eq!(e.run(&s).unwrap().text(), "2 Zack");
    assert!(e.globals.set("false", Value::Null).is_err());
    e.globals.remove("player.name");
    assert_eq!(e.run(&s).unwrap().text(), "2 null");
}
#[test]
fn recursive_maps_and_excessive_depth_are_errors() {
    let e = env();
    let map = starscript::ValueMap::default();
    map.set("_toString", Value::Map(map.clone())).unwrap();
    e.globals.set("cycle", Value::Map(map)).unwrap();
    assert!(e.run(&compile("{cycle}").unwrap()).is_err());
    assert!(compile(&format!("{{{}1{}}}", "(".repeat(256), ")".repeat(256))).is_err());
    assert!(e.run(&compile("{pad('x', 10000000)}").unwrap()).is_err());
    assert!(
        e.run(&compile("#1{pad('x',600000)}#2{pad('x',600000)}").unwrap())
            .is_err()
    );
    assert!(
        e.run(&compile("{pad('x',600000) + pad('y',600000)}").unwrap())
            .is_err()
    );
}

#[test]
fn host_functions_maps_and_objects_are_real_values() {
    let e = env();
    let function: starscript::Function = Rc::new(|args| Ok(Value::Number(args.len() as f64)));
    e.globals.set("count", Value::Function(function)).unwrap();
    let object = Value::Object {
        identity: 42,
        text: "actual host object".into(),
    };
    e.globals.set("object", object.clone()).unwrap();
    e.globals.set("object2", object).unwrap();
    let map = starscript::ValueMap::default();
    map.set("_toString", Value::String("map text".into()))
        .unwrap();
    e.globals.set("custom", Value::Map(map)).unwrap();
    let script = compile("{count(1,2,3)} {object} {object==object2} {custom}").unwrap();
    assert_eq!(
        e.run(&script).unwrap().text(),
        "3 actual host object true map text"
    );
    assert_eq!(e.globals.get("player.health"), Value::Number(20.0));
    assert!(e.globals.keys().contains(&"count".to_string()));
    e.globals.clear();
    assert_eq!(e.globals.get("count"), Value::Null);
    assert_eq!(
        starscript::JavaRandom::new(123).next_double(),
        0.7231742029971469
    );
}
