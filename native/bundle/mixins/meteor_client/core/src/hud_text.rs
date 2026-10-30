// SPDX-License-Identifier: GPL-3.0-only
// Source-derived Meteor TextHud policies.
use crate::colors::SettingColor;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Shown {
    #[default]
    Always,
    WhenTrue,
    WhenFalse,
}
impl Shown {
    pub fn visible(self, result: Option<&str>) -> bool {
        match self {
            Self::Always => true,
            Self::WhenTrue => result.is_some_and(|s| s.eq_ignore_ascii_case("true")),
            Self::WhenFalse => result.is_some_and(|s| s.eq_ignore_ascii_case("false")),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TextSettings {
    pub text: String,
    pub update_delay: i32,
    pub shadow: bool,
    pub border: i32,
    pub shown: Shown,
    pub condition: String,
    pub custom_scale: bool,
    pub scale: f64,
    pub background: bool,
    pub background_color: SettingColor,
}
impl Default for TextSettings {
    fn default() -> Self {
        Self {
            text: "Meteor".into(),
            update_delay: 4,
            shadow: true,
            border: 0,
            shown: Shown::Always,
            condition: String::new(),
            custom_scale: false,
            scale: 1.,
            background: false,
            background_color: SettingColor::new(25, 25, 25, 50),
        }
    }
}
impl TextSettings {
    pub fn validate(&self) -> Result<(), String> {
        if self.text.len() > 65536 || self.condition.len() > 65536 {
            return Err("HUD text/condition limit is65536 bytes".into());
        }
        if self.update_delay < 0 {
            return Err("HUD update-delay must be non-negative".into());
        }
        if !self.scale.is_finite() || self.scale < 0.5 {
            return Err("HUD scale must be finite and at least0.5".into());
        }
        // Geometry overflow/resource safety bounds, separate from source UI slider ranges.
        if self.scale > 10000. || self.border.abs_diff(0) > 1_000_000 {
            return Err("HUD geometry exceeds native resource bounds".into());
        }
        Ok(())
    }
    pub fn effective_scale(&self, global: f64) -> f64 {
        if self.custom_scale {
            self.scale
        } else {
            global
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct UpdateClock {
    timer: i32,
    pub first_tick: bool,
}
impl Default for UpdateClock {
    fn default() -> Self {
        Self {
            timer: 0,
            first_tick: true,
        }
    }
}
impl UpdateClock {
    pub fn tick(&mut self, delay: i32) -> bool {
        if self.timer <= 0 {
            self.timer = delay;
            true
        } else {
            self.timer -= 1;
            false
        }
    }
    pub fn delay_changed(&mut self, delay: i32) {
        if self.timer > delay {
            self.timer = delay
        }
    }
    pub fn recompile(&mut self) {
        self.first_tick = true
    }
    pub fn evaluated(&mut self) {
        self.first_tick = false
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_delay_is_skip_count_not_period() {
        let mut c = UpdateClock::default();
        let samples = (0..11).map(|_| c.tick(4)).collect::<Vec<_>>();
        assert_eq!(
            samples,
            vec![
                true, false, false, false, false, true, false, false, false, false, true
            ]
        );
        assert!(UpdateClock::default().tick(0));
    }
    #[test]
    fn changing_delay_clamps_remaining_wait_only() {
        let mut c = UpdateClock::default();
        assert!(c.tick(4));
        c.delay_changed(1);
        assert!(!c.tick(1));
        assert!(c.tick(1));
        c.evaluated();
        c.recompile();
        assert!(c.first_tick);
    }
    #[test]
    fn shown_uses_source_string_comparison_not_truthiness() {
        assert!(Shown::Always.visible(None));
        assert!(Shown::WhenTrue.visible(Some("TrUe")));
        assert!(!Shown::WhenTrue.visible(Some(" true ")));
        assert!(!Shown::WhenTrue.visible(Some("1")));
        assert!(Shown::WhenFalse.visible(Some("false")));
        assert!(!Shown::WhenFalse.visible(None));
    }
    #[test]
    fn defaults_and_invalid_geometry() {
        let mut s = TextSettings::default();
        s.validate().unwrap();
        assert_eq!(s.effective_scale(2.), 2.);
        s.custom_scale = true;
        assert_eq!(s.effective_scale(2.), 1.);
        s.scale = f64::NAN;
        assert!(s.validate().is_err());
        s.scale = 1.;
        s.update_delay = -1;
        assert!(s.validate().is_err());
    }
}

/// Per-instance text cache; editing presentation alone does not reset evaluation cadence.
#[derive(Default)]
pub struct Runtime {
    settings: Option<TextSettings>,
    script: Option<crate::starscript::Script>,
    condition: Option<crate::starscript::Script>,
    pub output: Vec<crate::starscript::Section>,
    pub visible: bool,
    clock: UpdateClock,
}
impl Runtime {
    pub fn update(
        &mut self,
        settings: &TextSettings,
        environment: &crate::starscript::Environment,
        tick: bool,
    ) {
        let recompile = self.settings.as_ref().is_none_or(|old| {
            old.text != settings.text
                || old.condition != settings.condition
                || old.shown != settings.shown
        });
        if recompile {
            self.script = match crate::starscript::compile(&settings.text) {
                Ok(script) => Some(script),
                Err(errors) => {
                    self.output = vec![crate::starscript::Section {
                        index: 0,
                        text: errors.first().map_or("Invalid text".to_owned(), |e| {
                            format!("{} at {}", e.message, e.offset)
                        }),
                    }];
                    None
                }
            };
            self.condition = if settings.shown != Shown::Always {
                crate::starscript::compile(&settings.condition).ok()
            } else {
                None
            };
            self.clock.recompile();
        }
        if self
            .settings
            .as_ref()
            .is_some_and(|s| s.update_delay != settings.update_delay)
        {
            self.clock.delay_changed(settings.update_delay)
        }
        self.settings = Some(settings.clone());
        let due = tick && self.clock.tick(settings.update_delay);
        if !due && !self.clock.first_tick {
            return;
        }
        if let Some(script) = &self.script {
            self.output = match environment.run(script) {
                Ok(result) => result.sections,
                Err(error) => vec![crate::starscript::Section {
                    index: 0,
                    text: error.to_string(),
                }],
            };
        }
        self.visible = if settings.shown == Shown::Always {
            true
        } else {
            let result = self
                .condition
                .as_ref()
                .and_then(|s| environment.run(s).ok())
                .map(|r| r.text());
            settings.shown.visible(result.as_deref())
        };
        self.clock.evaluated();
    }
}
