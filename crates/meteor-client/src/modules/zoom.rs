// SPDX-License-Identifier: GPL-3.0-only
// Port of systems/modules/render/Zoom.java. Copyright (c) Meteor Development.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ZoomSettings {
    pub zoom: f64,
    pub scroll_sensitivity: f64,
    pub smooth: bool,
    pub cinematic: bool,
    pub hide_hud: bool,
    pub show_hands: bool,
}

impl Default for ZoomSettings {
    fn default() -> Self {
        Self {
            zoom: 6.0,
            scroll_sensitivity: 1.0,
            smooth: true,
            cinematic: false,
            hide_hud: false,
            show_hands: false,
        }
    }
}

impl ZoomSettings {
    pub fn validate(&self) -> Result<(), String> {
        if !self.zoom.is_finite() || self.zoom < 1.0 {
            return Err("Zoom must be finite and at least 1".into());
        }
        if !self.scroll_sensitivity.is_finite() || self.scroll_sensitivity < 0.0 {
            return Err("Scroll sensitivity must be finite and nonnegative".into());
        }
        Ok(())
    }
}

pub struct Zoom {
    pub subscribed: bool,
    value: f64,
    time: f64,
}

impl Default for Zoom {
    fn default() -> Self {
        Self {
            subscribed: false,
            value: 1.0,
            time: 0.0,
        }
    }
}

impl Zoom {
    pub fn activate(&mut self, settings: &ZoomSettings) {
        // A toggle during fade-out keeps the same animation and scroll-adjusted factor.
        if self.subscribed {
            return;
        }
        self.value = settings.zoom;
        self.time = 0.001;
        self.subscribed = true;
    }
    pub fn stop(&mut self) {
        self.subscribed = false;
        self.time = 0.0;
    }
    pub fn frame(&mut self, seconds: f64, active: bool, settings: &ZoomSettings) {
        if !self.subscribed || !seconds.is_finite() || seconds < 0.0 {
            return;
        }
        if !settings.smooth {
            self.time = if active { 1.0 } else { 0.0 };
        } else {
            self.time = (self.time + seconds * if active { 5.0 } else { -5.0 }).clamp(0.0, 1.0);
        }
    }
    /// The original unsubscribes on Tick.Post, after the fade has finished.
    pub fn tick(&mut self) {
        if self.time == 0.0 {
            self.subscribed = false;
        }
    }
    pub fn scaling(&self) -> f64 {
        let t = self.time;
        let delta = if t < 0.5 {
            4.0 * t * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
        };
        1.0 + delta * (self.value - 1.0)
    }
    pub fn sensitivity_multiplier(&self) -> f64 {
        1.0 / (self.scaling() * 0.5).max(1.0)
    }
    pub fn scroll(
        &mut self,
        amount: f64,
        active: bool,
        screen_open: bool,
        settings: &ZoomSettings,
    ) -> bool {
        if !self.subscribed
            || !active
            || screen_open
            || settings.scroll_sensitivity <= 0.0
            || !amount.is_finite()
        {
            return false;
        }
        let value = self.value + amount * 0.25 * settings.scroll_sensitivity * self.value;
        if value.is_finite() {
            self.value = value.max(1.0);
        }
        true
    }
    pub fn render_hands(active: bool, settings: &ZoomSettings) -> bool {
        !active || settings.show_hands
    }
    pub fn hide_hud(active: bool, settings: &ZoomSettings) -> bool {
        active && settings.hide_hud
    }
}
