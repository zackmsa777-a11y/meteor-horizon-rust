// SPDX-License-Identifier: GPL-3.0-only
// Source-derived subset of Meteor Hud/HudBox and MeteorTextHud presets.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    #[serde(default = "legacy_version")]
    pub version: u32,
    pub text_scale: f64,
    pub text_colors: Vec<crate::colors::SettingColor>,
    pub hide_in_menus: bool,
    pub custom_font: bool,
    pub border: i32,
    pub snapping_range: i32,
    pub active: bool,
    pub elements: Vec<Element>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Element {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub source_box: Option<crate::hud_box::HudBox>,
    #[serde(default = "yes")]
    pub auto_anchors: bool,
    #[serde(default)]
    pub text: Option<crate::hud_text::TextSettings>,
    pub id: String,
    pub enabled: bool,
    /// Fractional viewport anchor, 0 = left/top, 1 = right/bottom.
    pub anchor: [f32; 2],
    /// Logical GUI pixel offset from the anchored element corner.
    pub offset: [f32; 2],
}
fn legacy_version() -> u32 {
    1
}
fn yes() -> bool {
    true
}
impl Element {
    pub fn kind(&self) -> &str {
        if self.kind.is_empty() {
            &self.id
        } else {
            &self.kind
        }
    }
}
impl Default for Config {
    fn default() -> Self {
        Self {
            version: 2,
            text_scale: 1.,
            text_colors: vec![
                crate::colors::SettingColor::new(255, 255, 255, 255),
                crate::colors::SettingColor::new(175, 175, 175, 255),
                crate::colors::SettingColor::new(25, 225, 25, 255),
                crate::colors::SettingColor::new(225, 25, 25, 255),
            ],
            hide_in_menus: false,
            custom_font: false,
            border: 4,
            snapping_range: 10,
            active: false,
            elements: [
                ("watermark", [0., 0.], [4., 4.]),
                ("fps", [0., 0.], [4., 16.]),
                ("position", [1., 1.], [-4., -4.]),
                ("active-modules", [1., 0.], [-4., 4.]),
                ("tps", [0., 0.], [4., 28.]),
                ("ping", [0., 0.], [4., 40.]),
                ("speed", [0., 0.], [4., 52.]),
                ("rotation", [1., 1.], [-4., -28.]),
                ("game-mode", [0., 0.], [4., 64.]),
            ]
            .into_iter()
            .map(|(id, anchor, offset)| Element {
                kind: String::new(),
                source_box: None,
                auto_anchors: true,
                text: None,
                id: id.into(),
                enabled: id != "game-mode",
                anchor,
                offset,
            })
            .collect(),
        }
    }
}
impl Config {
    /// New supported presets become reachable while existing positions/toggles stay intact.
    pub fn merge_presets(&mut self) {
        if self.version >= 2 {
            return;
        }
        for preset in Self::default().elements {
            if !self.elements.iter().any(|e| e.id == preset.id) {
                self.elements.push(preset);
            }
        }
        self.version = 2;
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.elements.len() > 1024 {
            return Err("HUD element limit is1024".into());
        }
        if self.custom_font {
            return Err(
                "Custom HUD font renderer is not implemented; select vanilla font explicitly"
                    .into(),
            );
        }
        if !self.text_scale.is_finite()
            || !(0.5..=10000.).contains(&self.text_scale)
            || self.text_colors.len() > 256
            || self.border.abs_diff(0) > 1_000_000
            || self.snapping_range < 0
            || self.snapping_range > 1_000_000
        {
            return Err("Invalid HUD global settings".into());
        }
        let mut seen = std::collections::BTreeSet::new();
        for element in &self.elements {
            if ![
                "text",
                "watermark",
                "fps",
                "position",
                "active-modules",
                "tps",
                "ping",
                "speed",
                "rotation",
                "game-mode",
            ]
            .contains(&element.kind())
            {
                return Err(format!("HUD element not implemented: {}", element.id));
            }
            if element.id.is_empty() || element.id.len() > 128 {
                return Err("Invalid HUD instance id".into());
            }
            if element.kind() == "text" {
                element
                    .text
                    .as_ref()
                    .ok_or("Text HUD settings missing")?
                    .validate()?;
            }
            if let Some(b) = element.source_box {
                if b.x.abs_diff(0) > 1_000_000 || b.y.abs_diff(0) > 1_000_000 {
                    return Err("HUD box exceeds coordinate bounds".into());
                }
            }
            if !seen.insert(&element.id) {
                return Err("Duplicate HUD element".into());
            }
            if element
                .anchor
                .iter()
                .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
                || element
                    .offset
                    .iter()
                    .any(|v| !v.is_finite() || v.abs() > 100_000.)
            {
                return Err("Invalid HUD position".into());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn old_configs_receive_disabled_source_defaults() {
        let c: Config = serde_json::from_str("{}").unwrap();
        assert!(!c.active);
        assert_eq!(c.elements.len(), 9);
        c.validate().unwrap();
    }
    #[test]
    fn adding_presets_keeps_existing_placement_and_toggles() {
        let mut c = Config::default();
        c.elements.truncate(4);
        c.version = 1;
        c.elements[0].enabled = false;
        c.elements[0].offset = [30., 70.];
        c.merge_presets();
        c.merge_presets();
        assert_eq!(c.elements.len(), 9);
        assert!(!c.elements[0].enabled);
        assert_eq!(c.elements[0].offset, [30., 70.]);
        assert!(
            !c.elements
                .iter()
                .find(|e| e.id == "game-mode")
                .unwrap()
                .enabled
        );
    }
    #[test]
    fn rejects_ambiguous_or_invalid_placement() {
        let mut c = Config::default();
        c.elements.push(c.elements[0].clone());
        assert!(c.validate().is_err());
        c.elements.pop();
        c.elements[0].anchor[0] = 2.;
        assert!(c.validate().is_err());
        c.elements[0].anchor[0] = 0.;
        c.elements[0].offset[0] = f32::NAN;
        assert!(c.validate().is_err());
    }
    #[test]
    fn placement_and_enabled_flags_round_trip() {
        let mut c = Config::default();
        c.active = true;
        c.elements[1].enabled = false;
        c.elements[2].offset = [-33., -56.];
        let restored: Config = serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
        restored.validate().unwrap();
        assert!(restored.active);
        assert!(!restored.elements[1].enabled);
        assert_eq!(restored.elements[2].offset, [-33., -56.]);
    }
}
