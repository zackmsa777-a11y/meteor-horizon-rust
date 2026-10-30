// SPDX-License-Identifier: GPL-3.0-only
use meteor_client::modules::{reverse_step, step};

#[test]
fn step_attribute_snapshot_and_activation_modes() {
    use step::*;
    let snapshot = Snapshot::capture(0.6, false);
    assert_eq!(snapshot.restore_base(), f64::from(0.6f32));
    assert!(!snapshot.previous_path_step);
    let mut settings = Settings::default();
    settings.validate().unwrap();
    assert_eq!(snapshot.tick_base(&settings, false, 1.25), 1.25);
    assert_eq!(
        snapshot.tick_base(&settings, false, 0.0),
        snapshot.restore_base()
    );
    settings.active_when = ActiveWhen::Sneaking;
    assert_eq!(
        snapshot.tick_base(&settings, false, 2.0),
        snapshot.restore_base()
    );
    assert_eq!(snapshot.tick_base(&settings, true, 2.0), 2.0);
    settings.active_when = ActiveWhen::NotSneaking;
    assert_eq!(
        snapshot.tick_base(&settings, true, 2.0),
        snapshot.restore_base()
    );
    assert_eq!(snapshot.tick_base(&settings, false, 2.0), 2.0);
    settings.height = -0.1;
    assert!(settings.validate().is_err());
}

#[test]
fn step_safe_sampling_preserves_fractional_and_directional_source_behavior() {
    use step::*;
    let mut settings = Settings::default();
    assert_eq!(
        max_safe_height(&settings, 0.0, 0.0, [0.0; 3], |_| -> Result<f64, ()> {
            panic!("disabled safe-step must not query crystals")
        })
        .unwrap(),
        1.25
    );
    settings.safe_step = true;
    settings.height = 2.5;
    let offset = input_offset([0.0, 0.8, 0.6], [1.0, 0.0]);
    assert_eq!(offset, [1.2, 0.0, 0.0]);
    let mut probes = Vec::new();
    let result = max_safe_height(&settings, 3.0, 0.0, offset, |at| {
        probes.push(at);
        Ok::<_, ()>(if at[0] != 0.0 && at[1] == 2.0 {
            8.0
        } else {
            4.0
        })
    })
    .unwrap();
    assert_eq!(result, 1.0);
    assert_eq!(
        probes,
        vec![
            [0.0; 3],
            [0.0, 1.0, 0.0],
            [1.2, 1.0, 0.0],
            [0.0, 2.0, 0.0],
            [1.2, 2.0, 0.0]
        ]
    );
    probes.clear();
    let result = max_safe_height(&settings, 20.0, 0.0, offset, |at| {
        probes.push(at);
        Ok::<_, ()>(4.0)
    })
    .unwrap();
    assert_eq!(result, 2.5);
    assert_eq!(probes.last(), Some(&[0.0, 2.5, 0.0]));
    assert!(!probes.contains(&[1.2, 2.5, 0.0]));
    assert_eq!(input_offset([0.0, 1.0, 0.0], [1.0, 1.0]), [0.0; 3]);
}

#[test]
fn step_safe_threshold_includes_absorption_and_propagates_missing_damage_queries() {
    use step::*;
    let settings = Settings {
        safe_step: true,
        height: 0.5,
        ..Settings::default()
    };
    // Current damage zero, proposed damage one: health 6 lands exactly on threshold 5, unsafe.
    for (absorption, expected) in [(0.0, 0.0), (1.0, 0.5)] {
        let result = max_safe_height(&settings, 6.0, absorption, [0.0; 3], |at| {
            Ok::<_, ()>(if at[1] == 0.0 { 0.0 } else { 1.0 })
        })
        .unwrap();
        assert_eq!(result, expected);
    }
    assert_eq!(
        max_safe_height(&settings, 20.0, 0.0, [0.0; 3], |_| Err::<f64, _>(
            "missing damage world"
        )),
        Err(SafeHeightError::Probe("missing damage world"))
    );
    let huge = Settings {
        height: f64::from(i32::MAX) + 1.0,
        ..settings
    };
    assert_eq!(
        max_safe_height(&huge, 20.0, 0.0, [0.0; 3], |_| -> Result<f64, ()> {
            panic!("oversized sweep must fail before probing")
        }),
        Err(SafeHeightError::SweepLimit)
    );
}

#[test]
fn reverse_step_vehicle_selection_and_collision_velocity() {
    use reverse_step::*;
    let mut settings = Settings::default();
    settings.validate().unwrap();
    assert_eq!(settings.probe_y(), f64::from(-3.01f32));
    assert_eq!(
        target(&settings, true, false, 1.0, 0.0, false),
        Some(Target::Player)
    );
    assert_eq!(target(&settings, false, false, 0.0, 0.0, false), None);
    assert_eq!(target(&settings, false, true, 1.0, 0.0, false), None);
    assert_eq!(target(&settings, false, false, 1.0, 0.0, true), None);
    settings.vehicles = true;
    assert_eq!(
        target(&settings, true, true, 0.0, 0.0, true),
        Some(Target::Vehicle)
    );
    let entity = Entity {
        on_ground: true,
        ..Entity::default()
    };
    let before = [0.2, -0.0784, -0.1];
    assert_eq!(
        velocity(&settings, entity, false, false, before),
        [0.2, -3.0, -0.1]
    );
    assert_eq!(velocity(&settings, entity, true, false, before), before);
    assert_eq!(velocity(&settings, entity, false, true, before), before);
    for blocked in [
        Entity {
            on_ground: false,
            ..entity
        },
        Entity {
            under_water: true,
            ..entity
        },
        Entity {
            in_lava: true,
            ..entity
        },
        Entity {
            no_physics: true,
            ..entity
        },
    ] {
        assert_eq!(velocity(&settings, blocked, false, false, before), before);
    }
}

#[test]
fn reverse_step_bed_queries_use_same_y_and_negative_coordinate_edges() {
    use reverse_step::*;
    let mut queried = Vec::new();
    assert!(on_bed([-0.2, 64.5625, -0.9], |at| {
        queried.push(at);
        at == [0, 64, -2]
    }));
    assert_eq!(
        queried,
        vec![[-1, 64, -1], [0, 64, -1], [-1, 64, -2], [0, 64, -2]]
    );
    let mut queried = Vec::new();
    assert!(!on_bed([0.5, 64.0, 0.5], |at| {
        queried.push(at);
        false
    }));
    assert_eq!(queried, vec![[0, 64, 0]]);
    let mut settings = Settings::default();
    settings.fall_distance = f64::INFINITY;
    assert!(settings.validate().is_err());
}
