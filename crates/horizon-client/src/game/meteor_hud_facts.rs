// SPDX-License-Identifier: GPL-3.0-only
// Native MeteorTextHud presets: Speed, Rotation, Ping, TPS and Game mode.
// Copyright (c) Meteor Development. Values come from live engine/network state.
use super::Game;
use std::collections::BTreeMap;

pub(super) struct State {
    #[cfg(feature = "meteor")]
    origin: std::time::Instant,
    #[cfg(feature = "meteor")]
    rate: meteor_client::hud_metrics::TickRate,
}
impl Default for State {
    fn default() -> Self {
        Self {
            #[cfg(feature = "meteor")]
            origin: std::time::Instant::now(),
            #[cfg(feature = "meteor")]
            rate: Default::default(),
        }
    }
}
impl Game<'_> {
    #[cfg(feature = "meteor")]
    pub(super) fn meteor_hud_tps(&self) -> f32 {
        self.meteor_hud_facts.rate.rate(
            self.meteor_hud_facts.origin.elapsed().as_millis() as u64,
            self.player.is_some(),
        )
    }
    /// GameJoinedEvent equivalent: call on login, not respawn/dimension changes.
    pub(super) fn meteor_hud_server_join(&mut self) {
        self.meteor_hud_facts = State::default();
    }
    /// PacketEvent.Receive for ClientboundSetTimePacket, independent of client Timer pacing.
    pub(super) fn meteor_hud_server_time(&mut self) {
        #[cfg(feature = "meteor")]
        self.meteor_hud_facts
            .rate
            .packet(self.meteor_hud_facts.origin.elapsed().as_millis() as u64);
    }
    #[cfg(not(feature = "meteor"))]
    pub(super) fn meteor_hud_facts(&mut self) -> BTreeMap<&'static str, String> {
        BTreeMap::new()
    }
    #[cfg(feature = "meteor")]
    pub(super) fn meteor_hud_facts(&mut self) -> BTreeMap<&'static str, String> {
        use meteor_client::hud_metrics;
        let mut facts = BTreeMap::new();
        let Some(player) = self.player.as_ref() else {
            return facts;
        };
        let current = [player.position.x, player.position.y, player.position.z];
        let previous = [
            self.previous_position.x,
            self.previous_position.y,
            self.previous_position.z,
        ];
        let (timer_active, timer_multiplier) = self.meteor_hud_speed_scale();
        let speed = hud_metrics::speed(current, previous, timer_active, timer_multiplier);
        facts.insert("speed", format!("Speed: {speed:.1}"));
        let (_, yaw, pitch) = self
            .camera_view(self.clock.camera_fraction() as f32)
            .unwrap_or((glam::DVec3::ZERO, self.yaw, self.pitch));
        facts.insert(
            "rotation",
            format!(
                "{} ({:.1}, {:.1})",
                hud_metrics::direction(yaw),
                hud_metrics::angle(yaw),
                hud_metrics::angle(pitch)
            ),
        );
        facts.insert("game-mode", format!("Game mode: {}", self.mode.name()));
        facts.insert("ping", format!("Ping: {}", self.meteor_hud_ping()));
        let tps = self.meteor_hud_facts.rate.rate(
            self.meteor_hud_facts.origin.elapsed().as_millis() as u64,
            true,
        );
        facts.insert("tps", format!("TPS: {tps:.1}"));
        facts
    }
}
