use meteor_client::{
    hud,
    hud_box::HudBox,
    hud_text::{Runtime, Shown, TextSettings},
    starscript::{Environment, Value},
};
#[test]
fn repeated_text_kinds_are_independent_instances_and_old_layout_survives() {
    let mut config = hud::Config::default();
    let mut element = config.elements[0].clone();
    element.id = "text-1".into();
    element.kind = "text".into();
    element.source_box = Some(HudBox::default());
    element.text = Some(TextSettings::default());
    config.elements.push(element.clone());
    element.id = "text-2".into();
    config.elements.push(element);
    config.validate().unwrap();
    let json = serde_json::to_value(&config).unwrap();
    let restored: hud::Config = serde_json::from_value(json).unwrap();
    assert_eq!(restored.elements[0].offset, [4., 4.]);
    assert_eq!(restored.elements[9].kind(), "text");
    assert_eq!(restored.elements[10].kind(), "text");
    let mut empty = hud::Config::default();
    empty.elements.clear();
    empty.merge_presets();
    assert!(empty.elements.is_empty());
}
#[test]
fn old_json_migrates_without_viewport_or_font_assumptions() {
    let mut config:hud::Config=serde_json::from_str(r#"{"active":true,"elements":[{"id":"fps","enabled":false,"anchor":[1,0],"offset":[-44,61]}]}"#).unwrap();
    assert_eq!(config.version, 1);
    config.merge_presets();
    assert_eq!(config.version, 2);
    assert_eq!(config.elements[0].offset, [-44., 61.]);
    assert!(!config.elements[0].enabled);
    assert_eq!(config.elements.len(), 9);
    config.validate().unwrap();
}
#[test]
fn runtime_refreshes_on_source_tick_cadence_and_retains_sections() {
    let env = Environment::default();
    env.globals.set("n", Value::Number(1.)).unwrap();
    let settings = TextSettings {
        text: "Count {n}".into(),
        ..Default::default()
    };
    let mut runtime = Runtime::default();
    runtime.update(&settings, &env, true);
    assert_eq!(runtime.output[0].text, "Count 1");
    env.globals.set("n", Value::Number(2.)).unwrap();
    for _ in 0..4 {
        runtime.update(&settings, &env, true);
        assert_eq!(runtime.output[0].text, "Count 1");
    }
    runtime.update(&settings, &env, true);
    assert_eq!(runtime.output[0].text, "Count 2");
    let mut settings = settings;
    settings.text = "{missing + 1}".into();
    runtime.update(&settings, &env, false);
    assert!(!runtime.output.is_empty());
    assert!(runtime.visible);
}
#[test]
fn text_condition_renders_boolean_string_and_edit_forces_refresh() {
    let env = Environment::default();
    let mut settings = TextSettings {
        text: "Live".into(),
        shown: Shown::WhenTrue,
        condition: "{false}".into(),
        ..Default::default()
    };
    let mut runtime = Runtime::default();
    runtime.update(&settings, &env, false);
    assert!(!runtime.visible);
    settings.condition = "{true}".into();
    runtime.update(&settings, &env, false);
    assert!(runtime.visible);
    settings.shown = Shown::WhenFalse;
    runtime.update(&settings, &env, false);
    assert!(!runtime.visible);
}
