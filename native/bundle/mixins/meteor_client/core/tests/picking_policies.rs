use meteor_client::{
    colors::{Rainbow, SettingColor},
    modules::{
        air_place::{self, ItemKind, ShapeMode},
        liquid_interact, no_mining_trace,
    },
};
#[test]
fn trace_replaces_only_a_chosen_entity_with_real_block_and_either_pickaxe() {
    let mut s = no_mining_trace::Settings::default();
    assert!(s.replace_entity_hit(true, false, true, "minecraft:cow", true));
    assert!(!s.replace_entity_hit(true, false, true, "minecraft:cow", false));
    assert!(!s.replace_entity_hit(false, true, true, "minecraft:cow", true));
    assert!(!s.replace_entity_hit(true, false, false, "minecraft:cow", true));
    s.blacklisted_entities.push("minecraft:cow".into());
    assert!(!s.replace_entity_hit(true, true, false, "minecraft:cow", true));
    assert!(s.can_work(true, true, false, None));
    s.only_when_holding_a_pickaxe = false;
    assert!(s.replace_entity_hit(true, false, false, "minecraft:pig", true));
}
#[test]
fn liquid_retry_never_overrides_a_real_solid_pick() {
    assert!(liquid_interact::retry_with_source_fluids(true, true));
    assert!(!liquid_interact::retry_with_source_fluids(true, false));
    assert!(!liquid_interact::retry_with_source_fluids(false, true));
}
#[test]
fn air_place_retains_source_cached_hit_and_hand_specific_gate() {
    let mut state = air_place::State::default();
    let hit = air_place::Hit { pos: [1, 80, -3] };
    state.pre_tick(true, true, || Some(hit));
    state.pre_tick(false, true, || panic!("gated"));
    state.pre_tick(true, false, || panic!("gated"));
    assert_eq!(state.hit, Some(hit));
    assert!(
        state
            .interaction_hit(ItemKind::Block, false, |h, check| h == hit && check)
            .is_some()
    );
    assert!(
        state
            .interaction_hit(ItemKind::SpawnEgg, false, |_, check| !check)
            .is_some()
    );
    assert!(
        state
            .interaction_hit(ItemKind::FireworkRocket, true, |_, _| panic!("gliding"))
            .is_none()
    );
    assert!(
        state
            .interaction_hit(ItemKind::ArmorStand, false, |_, check| !check)
            .is_none()
    );
    assert!(
        state
            .render_hit(&Default::default(), false, true, true)
            .is_none()
    );
}
#[test]
fn ranges_shapes_and_rainbow_preserve_source_settings() {
    let mut s = air_place::Settings::default();
    assert_eq!(s.pick_range(8.0), 8.0);
    s.custom_range = true;
    assert_eq!(s.pick_range(8.0), 5.0);
    s.range = -1.0;
    assert!(s.validate().is_err());
    assert!(ShapeMode::Both.lines() && ShapeMode::Both.sides());
    assert!(!ShapeMode::Lines.sides());
    assert!(!ShapeMode::Sides.lines());
    let mut global = Rainbow::default();
    global.next(0.005, 1.0);
    assert_eq!(global.rgb, [255, 8, 0]);
    let mut c = SettingColor::new(1, 2, 3, 10);
    c.rainbow = true;
    c.update(&global);
    assert_eq!(c.rgba(), [255, 8, 0, 10]);
    global.next(0.0, 1.0);
    assert_eq!(global.rgb, [255, 8, 0]);
}

#[test]
fn rainbow_matches_java_awt_float_rounding_after_repeated_ticks() {
    // Generated with original RGBtoHSB/HSBtoRGB JDK21, speed0.5/100.
    let cases = [
        (1, [255, 8, 0]),
        (50, [110, 255, 0]),
        (100, [0, 220, 255]),
        (200, [255, 70, 0]),
        (500, [0, 80, 255]),
        (1000, [160, 255, 0]),
    ];
    let mut color = Rainbow::default();
    for tick in 1..=1000 {
        color.next(0.005, 1.0);
        if let Some((_, expected)) = cases.iter().find(|(t, _)| *t == tick) {
            assert_eq!(&color.rgb, expected, "tick {tick}");
        }
    }
}
