// SPDX-License-Identifier: GPL-3.0-only
// Flight.java native policies. Copyright (c) Meteor Development.
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Abilities,
    Velocity,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AntiKick {
    Normal,
    Packet,
    None,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub mode: Mode,
    pub speed: f64,
    pub vertical_speed_match: bool,
    pub no_sneak: bool,
    pub anti_kick_mode: AntiKick,
    pub delay: i32,
    pub off_time: i32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: Mode::Abilities,
            speed: 0.1,
            vertical_speed_match: false,
            no_sneak: false,
            anti_kick_mode: AntiKick::Packet,
            delay: 20,
            off_time: 1,
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !self.speed.is_finite() || self.speed < 0.0 || !(self.speed as f32 * 15.0).is_finite() {
            return Err(
                "Flight speed must be finite, nonnegative and representable at the native flight-speed boundary",
            );
        }
        if self.delay < 1 || self.off_time < 1 {
            return Err("Flight delays must be positive");
        }
        Ok(())
    }
    pub fn flying_speed(&self, sprint: bool) -> Option<f32> {
        (self.mode == Mode::Velocity).then(|| self.speed as f32 * if sprint { 15.0 } else { 10.0 })
    }
    pub fn suppress_sneak(&self) -> bool {
        self.mode == Mode::Velocity && self.no_sneak
    }
    pub fn vertical_velocity(&self, jump: bool, sneak: bool) -> f64 {
        let step = self.speed * if self.vertical_speed_match { 10.0 } else { 5.0 };
        // Preserve the source's sequential add/subtract, including both keys.
        let mut y = 0.0;
        if jump {
            y += step;
        }
        if sneak {
            y -= step;
        }
        y
    }
}
#[derive(Clone, Debug)]
pub struct State {
    pub delay_left: i32,
    pub off_left: i32,
    pub flip: bool,
    pub last_yaw: f32,
    pub last_packet_y: f64,
}
impl Default for State {
    fn default() -> Self {
        Self {
            delay_left: 20,
            off_left: 1,
            flip: false,
            last_yaw: 0.0,
            last_packet_y: f64::MAX,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PostAction {
    RunMode,
    AbilitiesOffAndReturn,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Post {
    pub action: PostAction,
    pub resend_position: bool,
}
impl State {
    // No activation reset: source counters and packet history survive toggle cycles.
    pub fn pre(&mut self, yaw: f32, fall_distance: f64, velocity: [f64; 3]) -> f32 {
        let mut result = yaw;
        if fall_distance >= 3.0
            && yaw == self.last_yaw
            && (velocity[0] * velocity[0] + velocity[1] * velocity[1] + velocity[2] * velocity[2])
                .sqrt()
                < 0.003
        {
            result += if self.flip { 1.0 } else { -1.0 };
            self.flip = !self.flip;
        }
        self.last_yaw = yaw;
        result
    }
    pub fn post(&mut self, s: &Settings) -> Post {
        let mut result = Post {
            action: PostAction::RunMode,
            resend_position: false,
        };
        if self.delay_left > 0 {
            self.delay_left -= 1;
        }
        if self.off_left <= 0 && self.delay_left <= 0 {
            self.delay_left = s.delay;
            self.off_left = s.off_time;
            result.resend_position = s.anti_kick_mode == AntiKick::Packet;
        } else if self.delay_left <= 0 {
            if s.anti_kick_mode == AntiKick::Normal && s.mode == Mode::Abilities {
                result.action = PostAction::AbilitiesOffAndReturn;
            } else if s.anti_kick_mode == AntiKick::Packet && self.off_left == s.off_time {
                result.resend_position = true;
            }
            self.off_left -= 1;
        }
        result
    }
    /// Rot/Status upgrade is modified before its recursive Packet.Send dispatch.
    pub fn upgraded_packet_y(&mut self, current: f64, on_air: bool) -> f64 {
        let first = self.packet_y(current, on_air);
        self.packet_y(first, on_air)
    }
    // One source antiKickPacket invocation. Non-position packets are upgraded by
    // the adapter; its recursive send invokes this again like the Java event bus.
    pub fn packet_y(&mut self, current: f64, on_air: bool) -> f64 {
        if self.delay_left <= 0
            && self.last_packet_y != f64::MAX
            && (current >= self.last_packet_y || self.last_packet_y - current < 0.03130)
            && on_air
        {
            self.last_packet_y - 0.03130
        } else {
            self.last_packet_y = current;
            current
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn packet_cycle_has_both_resend_edges() {
        let mut state = State::default();
        let settings = Settings::default();
        for _ in 0..19 {
            assert_eq!(
                state.post(&settings),
                Post {
                    action: PostAction::RunMode,
                    resend_position: false
                }
            );
        }
        assert!(state.post(&settings).resend_position);
        assert_eq!((state.delay_left, state.off_left), (0, 0));
        assert!(state.post(&settings).resend_position);
        assert_eq!((state.delay_left, state.off_left), (20, 1));
    }
    #[test]
    fn normal_pause_only_abilities_and_early_returns() {
        let settings = Settings {
            anti_kick_mode: AntiKick::Normal,
            ..Settings::default()
        };
        let mut state = State {
            delay_left: 1,
            ..State::default()
        };
        assert_eq!(
            state.post(&settings).action,
            PostAction::AbilitiesOffAndReturn
        );
        assert_eq!(state.post(&settings).action, PostAction::RunMode);
        state.delay_left = 1;
        assert_eq!(
            state
                .post(&Settings {
                    mode: Mode::Velocity,
                    ..settings
                })
                .action,
            PostAction::RunMode
        );
    }
    #[test]
    fn changed_delays_apply_only_when_cycle_reloads() {
        let settings = Settings {
            delay: 3,
            off_time: 2,
            ..Settings::default()
        };
        let mut state = State::default();
        for _ in 0..19 {
            assert!(!state.post(&settings).resend_position);
        }
        assert!(!state.post(&settings).resend_position); // old offLeft1 != newly configured2
        assert!(state.post(&settings).resend_position);
        assert_eq!((state.delay_left, state.off_left), (3, 2));
        state.post(&settings);
        state.post(&settings);
        assert!(state.post(&settings).resend_position);
        assert!(!state.post(&settings).resend_position);
        assert!(state.post(&settings).resend_position);
    }
    #[test]
    fn jitter_alternates_and_retains_pre_jitter_yaw_for_restore() {
        let mut state = State::default();
        assert_eq!(state.pre(0.0, 3.0, [0.0; 3]), -1.0);
        assert_eq!(state.last_yaw, 0.0);
        assert_eq!(state.pre(0.0, 3.0, [0.0; 3]), 1.0);
        assert_eq!(state.pre(0.0, 3.0, [0.003, 0.0, 0.0]), 0.0);
        assert_eq!(state.pre(2.0, 3.0, [0.0; 3]), 2.0);
    }
    #[test]
    fn packet_history_stays_fixed_during_synthetic_descent() {
        let mut state = State {
            delay_left: 0,
            ..State::default()
        };
        assert_eq!(state.packet_y(64.0, true), 64.0);
        assert_eq!(state.packet_y(65.0, true), 64.0 - 0.03130);
        assert_eq!(state.last_packet_y, 64.0);
        assert_eq!(state.packet_y(63.0, true), 63.0);
        assert_eq!(state.last_packet_y, 63.0);
        assert_eq!(state.packet_y(64.0, false), 64.0);
    }
    #[test]
    fn real_descent_threshold_and_cycle_gate() {
        let mut state = State {
            delay_left: 0,
            last_packet_y: 0.0,
            ..State::default()
        };
        assert_eq!(state.packet_y(-0.03, true), -0.03130);
        assert_eq!(state.packet_y(-0.04, true), -0.04);
        state.delay_left = 1;
        assert_eq!(state.packet_y(5.0, true), 5.0);
        assert_eq!(state.last_packet_y, 5.0);
    }
    #[test]
    fn velocity_ratios_keys_and_sneak_mode() {
        let mut settings = Settings {
            mode: Mode::Velocity,
            no_sneak: true,
            ..Settings::default()
        };
        assert_eq!(settings.flying_speed(false), Some(1.0));
        assert_eq!(settings.flying_speed(true), Some(1.5));
        assert_eq!(settings.vertical_velocity(true, false), 0.5);
        assert_eq!(settings.vertical_velocity(true, true), 0.0);
        assert!(settings.suppress_sneak());
        settings.vertical_speed_match = true;
        assert_eq!(settings.vertical_velocity(false, true), -1.0);
        settings.mode = Mode::Abilities;
        assert!(!settings.suppress_sneak());
        assert_eq!(settings.flying_speed(false), None);
    }
    #[test]
    fn setting_validation_matches_positive_source_bounds() {
        assert!(Settings::default().validate().is_ok());
        assert!(
            Settings {
                speed: f64::NAN,
                ..Settings::default()
            }
            .validate()
            .is_err()
        );
        assert!(
            Settings {
                speed: -1.0,
                ..Settings::default()
            }
            .validate()
            .is_err()
        );
        assert!(
            Settings {
                delay: 0,
                ..Settings::default()
            }
            .validate()
            .is_err()
        );
        assert!(
            Settings {
                speed: 0.0,
                ..Settings::default()
            }
            .validate()
            .is_ok()
        );
    }
}

#[cfg(test)]
mod boundary_tests {
    use super::*;
    #[test]
    fn upgraded_packet_reenters_source_callback() {
        let mut s = State {
            delay_left: 0,
            last_packet_y: 64.0,
            ..State::default()
        };
        let first = 64.0 - 0.03130;
        assert_eq!(s.upgraded_packet_y(65.0, true), first);
        // Floating-point subtraction at this coordinate is slightly above threshold:
        // the second invocation records its synthetic y, unlike a position packet.
        assert_eq!(s.last_packet_y, first);
        assert_eq!(s.upgraded_packet_y(66.0, true), first - 0.03130);
    }
    #[test]
    fn rejects_native_speed_overflow_atomically() {
        let mut c = crate::Config::default();
        let before = c.flight.speed;
        assert!(c.set("flight", "speed", "1e300").is_err());
        assert_eq!(c.flight.speed, before);
        assert!(
            Settings {
                speed: f64::from(f32::MAX) / 10.0,
                ..Settings::default()
            }
            .validate()
            .is_err()
        );
    }
}
