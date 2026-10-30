// SPDX-License-Identifier: GPL-3.0-only
// Sneak.java, KeyboardInputMixin and LocalPlayerMixin. Copyright (c) Meteor Development.
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Packet,
    #[default]
    Vanilla,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub mode: Mode,
}
impl Settings {
    pub fn vanilla(&self, active: bool, flying: bool) -> bool {
        active && !flying && self.mode == Mode::Vanilla
    }
    pub fn packet(&self, active: bool, flying: bool) -> bool {
        active && !flying && self.mode == Mode::Packet
    }
}
