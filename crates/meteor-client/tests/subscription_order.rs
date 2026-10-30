// SPDX-License-Identifier: GPL-3.0-only
use meteor_client::subscriptions::Subscriptions;
#[test]
fn equal_priority_dispatch_follows_activation_and_resubscription_order() {
    let mut s = Subscriptions::default();
    s.activate("fast-climb");
    s.activate("no-slow");
    let original = s.generation("fast-climb").unwrap();
    assert_eq!(
        s.entries()
            .iter()
            .map(|e| e.id.as_str())
            .collect::<Vec<_>>(),
        ["fast-climb", "no-slow"]
    );
    s.deactivate("fast-climb");
    s.activate("fast-climb");
    assert_eq!(
        s.entries()
            .iter()
            .map(|e| e.id.as_str())
            .collect::<Vec<_>>(),
        ["no-slow", "fast-climb"]
    );
    assert_ne!(s.generation("fast-climb"), Some(original));
    s.clear();
    s.activate("fast-climb");
    assert_ne!(s.generation("fast-climb"), Some(original));
}
#[test]
fn source_timer_writers_observe_the_same_order_as_their_subscriptions() {
    use meteor_client::modules::{fast_climb, no_slow, timer};
    let mut subscriptions = Subscriptions::default();
    let climb_settings = fast_climb::Settings {
        timer_mode: true,
        timer: 2.0,
        ..Default::default()
    };
    let web_settings = no_slow::Settings {
        web: no_slow::WebMode::Timer,
        web_timer: 10.0,
        ..Default::default()
    };
    let dispatch = |subscriptions: &Subscriptions| {
        let mut climb = fast_climb::State::default();
        let mut web = no_slow::State::default();
        let mut timer = timer::Timer::default();
        for entry in subscriptions.entries() {
            let value = match entry.id.as_str() {
                "fast-climb" => climb.pre_tick(
                    &climb_settings,
                    fast_climb::Player {
                        horizontal_collision: true,
                        on_climbable: true,
                        ..Default::default()
                    },
                ),
                "no-slow" => web.pre_tick(&web_settings, true, false),
                _ => None,
            };
            if let Some(value) = value {
                timer.set_override(value);
            }
        }
        timer.multiplier(&Default::default(), false)
    };
    subscriptions.activate("fast-climb");
    subscriptions.activate("no-slow");
    assert_eq!(dispatch(&subscriptions), 10.0);
    subscriptions.deactivate("fast-climb");
    subscriptions.activate("fast-climb");
    assert_eq!(dispatch(&subscriptions), 2.0);
}

#[test]
fn client_toggle_menu_and_world_lifecycle_manage_real_subscriptions() {
    let mut client = meteor_client::Client::default();
    client.toggle("fast-climb").unwrap();
    assert!(client.subscriptions().is_empty());
    client.world_changed(true);
    let first = client
        .subscriptions()
        .iter()
        .find(|s| s.id == "fast-climb")
        .unwrap()
        .generation;
    client.toggle("no-slow").unwrap();
    client.toggle("fast-climb").unwrap();
    client.toggle("fast-climb").unwrap();
    assert_eq!(client.subscriptions().last().unwrap().id, "fast-climb");
    assert_ne!(client.subscriptions().last().unwrap().generation, first);
    client.world_changed(false);
    assert!(client.subscriptions().is_empty());
    client.world_changed(true);
    assert_eq!(
        client
            .subscriptions()
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        ["no-slow", "fast-climb"]
    );
}

#[test]
fn competing_post_movement_writers_follow_real_activation_order() {
    use meteor_client::{
        Client,
        modules::{flight, spider},
        subscriptions::{PostTick, ordered_post_tick},
    };
    let mut client = Client::default();
    client.world_changed(true);
    client.toggle("flight").unwrap();
    client.toggle("spider").unwrap();
    client.config.flight.mode = flight::Mode::Velocity;
    let dispatch = |client: &Client| {
        let mut velocity = [0.0; 3];
        for listener in ordered_post_tick(client.subscriptions()) {
            match listener {
                PostTick::Flight => {
                    velocity = [
                        0.0,
                        client.config.flight.vertical_velocity(false, false),
                        0.0,
                    ]
                }
                PostTick::Spider => {
                    velocity = spider::post_tick(&client.config.spider, true, velocity)
                }
                _ => {}
            }
        }
        velocity
    };
    assert_eq!(dispatch(&client)[1], 0.2);
    client.toggle("flight").unwrap();
    client.toggle("flight").unwrap();
    assert_eq!(dispatch(&client)[1], 0.0);
    client.world_changed(false);
    assert!(ordered_post_tick(client.subscriptions()).is_empty());
    client.world_changed(true);
    assert_eq!(dispatch(&client)[1], 0.0);
}
#[test]
fn post_dispatch_snapshot_is_stable_while_live_bus_unsubscribes() {
    use meteor_client::subscriptions::{PostTick, Subscriptions, ordered_post_tick};
    let mut subscriptions = Subscriptions::default();
    subscriptions.activate("auto-clicker");
    subscriptions.activate("zoom"); // no Post listener
    subscriptions.activate("step");
    let snapshot = ordered_post_tick(subscriptions.entries());
    subscriptions.deactivate("step");
    assert_eq!(snapshot, [PostTick::AutoClicker, PostTick::Step]);
    assert_eq!(
        ordered_post_tick(subscriptions.entries()),
        [PostTick::AutoClicker]
    );
}
