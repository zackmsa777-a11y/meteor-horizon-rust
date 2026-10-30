use meteor_client::modules::{multitask, reach};
#[test]
fn independent_offsets_clamp_and_preserve_inactive_attributes() {
    let s = reach::Settings {
        extra_block_reach: 8.0,
        extra_entity_reach: -5.0,
    };
    assert_eq!(s.block(true, 4.5), 12.5);
    assert_eq!(s.entity(true, 3.0), 0.0);
    assert_eq!(s.block(false, 7.0), 7.0);
    assert_eq!(s.entity(false, 8.0), 8.0);
    assert!(
        reach::Settings {
            extra_block_reach: f64::MAX,
            extra_entity_reach: -f64::MAX
        }
        .validate()
        .is_ok()
    );
    assert!(
        reach::Settings {
            extra_block_reach: f64::INFINITY,
            ..Default::default()
        }
        .validate()
        .is_err()
    );
}
#[test]
fn entity_setting_does_not_restore_block_action_gates() {
    let s = multitask::Settings {
        attacking_entities: false,
    };
    assert!(!s.entity_attacks(true));
    assert!(!multitask::exclusive_action(true, true));
    assert!(multitask::exclusive_action(false, true));
    assert!(multitask::Settings::default().entity_attacks(true));
}
