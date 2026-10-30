// SPDX-License-Identifier: GPL-3.0-only
// Flight.java event, packet and ability adapters. Copyright (c) Meteor Development.
use super::Game;
use horizon_core::geometry::Vec3;
use horizon_protocol::packets::play::{self, Serverbound as S};
use meteor_client::modules::flight::{self, AntiKick, Mode, PostAction};
use std::cell::RefCell;

#[derive(Default)]
pub(super) struct State {
    core: RefCell<flight::State>,
    previous: Option<(u64, Mode)>,
    sent: RefCell<std::collections::VecDeque<serde_json::Value>>,
}
impl Game<'_> {
    pub(super) fn meteor_flight_status(&self) -> serde_json::Value {
        let core = self.meteor.flight.core.borrow();
        serde_json::json!({"delay_left":core.delay_left,"off_left":core.off_left,"last_packet_y":core.last_packet_y,
            "abilities":self.player.as_ref().map(|p| serde_json::json!({"flying":p.flying,"may_fly":p.may_fly,"flying_speed":p.flying_speed,"walking_speed":p.ability_walk_speed,"invulnerable":p.invulnerable,"creative":self.creative,"shift_key_down":p.is_shift_key_down()})),
            "sent_movement":self.meteor.flight.sent.borrow().clone()})
    }
    /// Capture actual packets accepted by the connection send path, after transformation.
    pub(super) fn meteor_flight_record_sent(&self, packet: &S) {
        let value = match packet {
            S::MovePlayerPos(p) => {
                serde_json::json!({"kind":"Pos","position":[p.x,p.y,p.z],"flags":p.flags})
            }
            S::MovePlayerPosRot(p) => {
                serde_json::json!({"kind":"PosRot","position":[p.x,p.y,p.z],"yaw":p.yaw,"pitch":p.pitch,"flags":p.flags})
            }
            S::MovePlayerRot(p) => {
                serde_json::json!({"kind":"Rot","yaw":p.yaw,"pitch":p.pitch,"flags":p.flags})
            }
            S::MovePlayerStatus(p) => serde_json::json!({"kind":"Status","flags":p.flags}),
            _ => return,
        };
        let mut sent = self.meteor.flight.sent.borrow_mut();
        if sent.len() == 32 {
            sent.pop_front();
        }
        sent.push_back(value);
    }

    pub(super) fn meteor_flight_sync(&mut self) {
        let next = self
            .meteor_interaction_active("flight")
            .then(|| {
                self.meteor
                    .client
                    .subscriptions()
                    .iter()
                    .find(|s| s.id == "flight")
                    .map(|s| (s.generation, self.meteor.client.config.flight.mode))
            })
            .flatten();
        let old = self.meteor.flight.previous;
        let spectator = self.spectator();
        if let Some(player) = self.player.as_mut() {
            if old != next {
                // Mode.onChanged always clears abilities; deactivation only does so in Abilities.
                let mode_changed = old.zip(next).is_some_and(|(a, b)| a.1 != b.1);
                if mode_changed || (old.is_some_and(|s| s.1 == Mode::Abilities) && !spectator) {
                    player.flying = false;
                    player.flying_speed = 0.05;
                    if !self.creative {
                        player.may_fly = false;
                    }
                }
                if next.is_some_and(|s| s.1 == Mode::Abilities)
                    && !spectator
                    && (old.is_none() || old.map(|s| s.0) != next.map(|s| s.0))
                {
                    player.flying = true;
                    if !self.creative {
                        player.may_fly = true;
                    }
                }
            }
            player.meteor_flight = next.map(|_| self.meteor.client.config.flight.clone());
        }
        self.meteor.flight.previous = next;
    }
    pub(super) fn meteor_flight_pre(&mut self) {
        self.meteor_flight_sync();
        if self.meteor.flight.previous.is_none() {
            return;
        }
        if let Some(p) = &self.player {
            self.yaw = self.meteor.flight.core.borrow_mut().pre(
                self.yaw,
                p.fall_distance,
                [p.velocity.x, p.velocity.y, p.velocity.z],
            );
        }
    }
    pub(super) fn meteor_flight_post(&mut self) {
        self.meteor_flight_sync();
        if self.meteor.flight.previous.is_none() {
            return;
        }
        let settings = self.meteor.client.config.flight.clone();
        let action = self.meteor.flight.core.borrow_mut().post(&settings);
        if action.resend_position {
            self.remote_flight_remind();
        }
        let spectator = self.spectator();
        let down = |mapping: &str| {
            self.physical_keys.iter().any(|key| {
                horizon_client::keybinds::key_name(*key) == Some(self.settings.keys.get(mapping))
            }) || self.physical_buttons.iter().any(|button| {
                horizon_client::keybinds::mouse_name(*button) == self.settings.keys.get(mapping)
            })
        };
        let (jump, sneak) = (down("key.jump"), down("key.sneak"));
        let Some(p) = &mut self.player else {
            return;
        };
        if action.action == PostAction::AbilitiesOffAndReturn {
            p.flying = false;
            p.flying_speed = 0.05;
            if !self.creative {
                p.may_fly = false;
            }
            return; // Source deliberately skips yaw restoration during this interval.
        }
        self.yaw = self.meteor.flight.core.borrow().last_yaw;
        match settings.mode {
            Mode::Velocity => {
                p.flying = false;
                p.velocity = Vec3::new(0.0, settings.vertical_velocity(jump, sneak), 0.0);
                if settings.no_sneak {
                    p.ground.on_ground = false;
                }
            }
            Mode::Abilities if !spectator => {
                p.flying_speed = settings.speed as f32;
                p.flying = true;
                if !self.creative {
                    p.may_fly = true;
                }
            }
            _ => {}
        }
    }
    pub(super) fn meteor_flight_preserve_abilities(&self) -> bool {
        self.meteor_interaction_active("flight")
            && self.meteor.client.config.flight.mode == Mode::Abilities
    }
    /// Packet.Send sees every local movement variant, including teleport responses.
    pub(super) fn meteor_flight_packet(&self, packet: S) -> S {
        if !self.meteor_interaction_active("flight")
            || self.meteor.client.config.flight.anti_kick_mode != AntiKick::Packet
        {
            return packet;
        }
        if !matches!(
            &packet,
            S::MovePlayerPos(_)
                | S::MovePlayerPosRot(_)
                | S::MovePlayerRot(_)
                | S::MovePlayerStatus(_)
        ) {
            return packet;
        }
        let Some(p) = &self.player else {
            return packet;
        };
        let bounds = p.bounds();
        let min = bounds.min_position();
        let max = bounds.max_position();
        // EntityUtils.isOnAir uses all states, including noncolliding grass and fluids.
        let on_air = ((min.x - 0.0625).floor() as i32..=(max.x + 0.0625).floor() as i32).all(|x| {
            ((min.y - 0.0625 - 0.55).floor() as i32..=(max.y + 0.0625).floor() as i32).all(|y| {
                ((min.z - 0.0625).floor() as i32..=(max.z + 0.0625).floor() as i32)
                    .all(|z| self.res.facts().is_air(self.world.block(x, y, z)))
            })
        });
        let mut state = self.meteor.flight.core.borrow_mut();
        match packet {
            S::MovePlayerPos(mut v) => {
                v.y = state.packet_y(v.y, on_air);
                S::MovePlayerPos(v)
            }
            S::MovePlayerPosRot(mut v) => {
                v.y = state.packet_y(v.y, on_air);
                S::MovePlayerPosRot(v)
            }
            S::MovePlayerRot(v) => {
                let y = state.upgraded_packet_y(p.position.y, on_air);
                // Java cancels the original, modifies its replacement, then sends that
                // replacement through Packet.Send once more. Preserve both invocations.

                S::MovePlayerPosRot(play::MovePlayerPosRot {
                    x: p.position.x,
                    y,
                    z: p.position.z,
                    yaw: v.yaw,
                    pitch: v.pitch,
                    flags: (v.flags & 1) | (u8::from(p.collision.horizontal) << 1),
                })
            }
            S::MovePlayerStatus(v) => {
                let y = state.upgraded_packet_y(p.position.y, on_air);

                S::MovePlayerPos(play::MovePlayerPos {
                    x: p.position.x,
                    y,
                    z: p.position.z,
                    flags: (v.flags & 1) | (u8::from(p.collision.horizontal) << 1),
                })
            }
            other => other,
        }
    }
}
