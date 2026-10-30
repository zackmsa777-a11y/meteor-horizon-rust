// SPDX-License-Identifier: GPL-3.0-only
#[path = "../src/damage.rs"]
mod damage;
use damage::*;

fn bare() -> Mitigation {
    Mitigation {
        difficulty: Difficulty::Normal,
        scales_with_difficulty: true,
        armor: 0.0,
        toughness: 0.0,
        resistance_amplifier: None,
        protection: 0,
        bypasses_invulnerability: false,
    }
}

#[test]
fn crystal_distance_integer_damage_and_difficulty_match_source() {
    assert_eq!(crystal_damage([0.0; 3], [0.0; 3], 1.0, bare()), 85.0);
    assert_eq!(crystal_damage([0.0; 3], [6.0, 0.0, 0.0], 1.0, bare()), 32.0);
    assert_eq!(crystal_damage([0.0; 3], [12.0, 0.0, 0.0], 1.0, bare()), 1.0);
    assert_eq!(
        crystal_damage([0.0; 3], [12.0001, 0.0, 0.0], 1.0, bare()),
        0.0
    );
    assert_eq!(crystal_damage([0.0; 3], [0.0; 3], 0.0, bare()), 1.0);
    assert_eq!(
        reduce(
            10.0,
            Mitigation {
                difficulty: Difficulty::Easy,
                ..bare()
            }
        ),
        6.0
    );
    assert_eq!(
        reduce(
            10.0,
            Mitigation {
                difficulty: Difficulty::Hard,
                ..bare()
            }
        ),
        15.0
    );
    // Source has no Peaceful branch, and performs no difficulty scaling when source says false.
    assert_eq!(
        reduce(
            10.0,
            Mitigation {
                difficulty: Difficulty::Peaceful,
                ..bare()
            }
        ),
        10.0
    );
    assert_eq!(
        reduce(
            10.0,
            Mitigation {
                difficulty: Difficulty::Hard,
                scales_with_difficulty: false,
                ..bare()
            }
        ),
        10.0
    );
}

#[test]
fn mitigation_floors_armor_orders_resistance_and_caps_protection() {
    let a = Mitigation {
        armor: 20.9,
        toughness: 8.0,
        resistance_amplifier: Some(0),
        protection: 8,
        ..bare()
    };
    let armor_only = 20.0f32 * (1.0 - 15.0 / 25.0);
    assert_eq!(
        reduce(20.0, a),
        armor_only * (1.0 - 0.2) * (1.0 - 8.0 / 25.0)
    );
    assert_eq!(
        reduce(
            20.0,
            Mitigation {
                protection: 100,
                ..bare()
            }
        ),
        20.0 * (1.0 - 20.0 / 25.0)
    );
    assert_eq!(
        reduce(
            20.0,
            Mitigation {
                resistance_amplifier: Some(4),
                ..bare()
            }
        ),
        0.0
    );
    assert_eq!(
        reduce(
            20.0,
            Mitigation {
                protection: 20,
                bypasses_invulnerability: true,
                ..bare()
            }
        ),
        20.0
    );
}

#[test]
fn exposure_additive_grid_and_query_errors_are_not_hidden() {
    let bounds = Bounds {
        min: [0.0; 3],
        max: [1.0; 3],
    };
    let mut probes = Vec::new();
    let fraction = exposure([4.0, 2.0, 4.0], bounds, |from, to| {
        probes.push((from, to));
        Ok::<_, ()>(from[0] == 0.0)
    })
    .unwrap();
    assert_eq!(probes.len(), 64);
    assert_eq!(fraction, 0.75);
    assert_eq!(probes[0], ([0.0; 3], [4.0, 2.0, 4.0]));
    assert_eq!(probes.last().unwrap().0, [1.0; 3]);
    assert_eq!(
        exposure([0.0; 3], bounds, |_, _| Err::<bool, _>("ray failed")),
        Err(ExposureError::Ray("ray failed"))
    );
    assert_eq!(
        exposure(
            [0.0; 3],
            Bounds {
                min: [0.0; 3],
                max: [0.0, 1.0, 1.0]
            },
            |_, _| Ok::<_, ()>(false)
        ),
        Err(ExposureError::InvalidBounds)
    );
}
