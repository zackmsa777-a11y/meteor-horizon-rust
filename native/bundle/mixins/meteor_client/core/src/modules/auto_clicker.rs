// SPDX-License-Identifier: GPL-3.0-only
// AutoClicker.java native policies. Copyright (c) Meteor Development.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Disabled,
    Hold,
    #[default]
    Press,
}
impl Mode {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "Disabled" => Ok(Self::Disabled),
            "Hold" => Ok(Self::Hold),
            "Press" => Ok(Self::Press),
            _ => Err("Click mode must be Disabled, Hold or Press".into()),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub while_in_screens: bool,
    pub mode_left: Mode,
    pub delay_left: i32,
    pub mode_right: Mode,
    pub delay_right: i32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            while_in_screens: true,
            mode_left: Mode::Press,
            delay_left: 2,
            mode_right: Mode::Press,
            delay_right: 2,
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if self.delay_left < 0 || self.delay_right < 0 {
            Err("AutoClicker delays must be nonnegative".into())
        } else {
            Ok(())
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Action {
    #[default]
    None,
    Hold,
    Press,
}
#[derive(Clone, Debug, Default)]
pub struct State {
    pub left_timer: i32,
    pub right_timer: i32,
}
impl State {
    pub fn post(&mut self, settings: &Settings, screen_open: bool) -> [Action; 2] {
        if screen_open && !settings.while_in_screens {
            return [Action::None; 2];
        }
        fn side(timer: &mut i32, mode: Mode, delay: i32) -> Action {
            match mode {
                Mode::Disabled => Action::None,
                Mode::Hold => Action::Hold,
                Mode::Press => {
                    *timer = timer.wrapping_add(1); // Java int overflow, not saturation.
                    if *timer > delay {
                        *timer = 0;
                        Action::Press
                    } else {
                        Action::None
                    }
                }
            }
        }
        [
            side(
                &mut self.left_timer,
                settings.mode_left,
                settings.delay_left,
            ),
            side(
                &mut self.right_timer,
                settings.mode_right,
                settings.delay_right,
            ),
        ]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_delay_two_means_every_third_tick() {
        let mut state = State::default();
        let settings = Settings::default();
        assert_eq!(state.post(&settings, false), [Action::None; 2]);
        assert_eq!(state.post(&settings, false), [Action::None; 2]);
        assert_eq!(state.post(&settings, false), [Action::Press; 2]);
        assert_eq!(state.post(&settings, false), [Action::None; 2]);
    }
    #[test]
    fn screen_gate_pauses_timers_without_releasing_hold() {
        let mut state = State {
            left_timer: 2,
            right_timer: 1,
        };
        let settings = Settings {
            while_in_screens: false,
            ..Settings::default()
        };
        assert_eq!(state.post(&settings, true), [Action::None; 2]);
        assert_eq!((state.left_timer, state.right_timer), (2, 1));
        assert_eq!(state.post(&settings, false), [Action::Press, Action::None]);
    }
    #[test]
    fn channels_and_mode_changes_keep_independent_counters() {
        let mut state = State {
            left_timer: 2,
            right_timer: 1,
        };
        let settings = Settings {
            mode_left: Mode::Hold,
            mode_right: Mode::Disabled,
            ..Settings::default()
        };
        assert_eq!(state.post(&settings, false), [Action::Hold, Action::None]);
        assert_eq!((state.left_timer, state.right_timer), (2, 1));
        assert_eq!(
            state.post(&Settings::default(), false),
            [Action::Press, Action::None]
        );
    }
    #[test]
    fn zero_delay_runs_each_tick_and_max_delay_preserves_java_overflow() {
        let mut state = State {
            left_timer: i32::MAX,
            right_timer: 0,
        };
        let settings = Settings {
            delay_left: i32::MAX,
            delay_right: 0,
            ..Settings::default()
        };
        assert_eq!(state.post(&settings, false), [Action::None, Action::Press]);
        assert_eq!(state.left_timer, i32::MIN);
        assert_eq!(state.post(&settings, false)[1], Action::Press);
    }
}
