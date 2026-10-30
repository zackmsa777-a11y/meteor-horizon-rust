// SPDX-License-Identifier: GPL-3.0-only
// SettingColor/RainbowColor/RainbowColors native policies. Copyright (c) Meteor Development.
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
    #[serde(default)]
    pub rainbow: bool,
}
impl SettingColor {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r,
            g,
            b,
            a,
            rainbow: false,
        }
    }
    pub fn rgba(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
    pub fn update(&mut self, global: &Rainbow) {
        if self.rainbow {
            [self.r, self.g, self.b] = global.rgb;
        }
    }
}
#[derive(Clone, Debug)]
pub struct Rainbow {
    pub rgb: [u8; 3],
}
impl Default for Rainbow {
    fn default() -> Self {
        Self { rgb: [255; 3] }
    }
}
impl Rainbow {
    /// Java RGBtoHSB then HSBtoRGB, with source float precision and rounding.
    /// TickEvent.Post passes Config.rainbowSpeed / 100 as speed; delta is 1.
    pub fn next(&mut self, speed: f64, delta: f64) {
        if speed <= 0.0 {
            return;
        }
        let [r, g, b] = self.rgb.map(i32::from);
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let hue = if max == min {
            0.0
        } else {
            let span = (max - min) as f32;
            let (rc, gc, bc) = (
                (max - r) as f32 / span,
                (max - g) as f32 / span,
                (max - b) as f32 / span,
            );
            let h = if r == max {
                bc - gc
            } else if g == max {
                2.0 + rc - bc
            } else {
                4.0 + gc - rc
            };
            let h = h / 6.0;
            if h < 0.0 { h + 1.0 } else { h }
        };
        let hue = hue + (speed * delta) as f32;
        let h = (hue - hue.floor()) * 6.0;
        let f = h - h.floor();
        let q = 1.0 - f;
        let t = f;
        let rgb = match h as i32 {
            0 => [1.0, t, 0.0],
            1 => [q, 1.0, 0.0],
            2 => [0.0, 1.0, t],
            3 => [0.0, q, 1.0],
            4 => [t, 0.0, 1.0],
            _ => [1.0, 0.0, q],
        };
        self.rgb = rgb.map(|v| (v * 255.0 + 0.5) as u8);
    }
}
