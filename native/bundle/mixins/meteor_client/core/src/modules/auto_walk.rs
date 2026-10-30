// SPDX-License-Identifier: GPL-3.0-only
// Port of AutoWalk.java. Copyright (c) Meteor Development.
//! Decisions only: the engine adapter applies key states and dispatches path-manager requests.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Simple,
    #[default]
    Smart,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    #[default]
    Forwards,
    Backwards,
    Left,
    Right,
}

impl Direction {
    pub fn mapping(self) -> &'static str {
        match self {
            Self::Forwards => "key.forward",
            Self::Backwards => "key.back",
            Self::Left => "key.left",
            Self::Right => "key.right",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub mode: Mode,
    pub simple_direction: Direction,
    pub disable_on_input: bool,
    pub disable_on_y_change: bool,
    pub no_unloaded_chunks: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: Mode::Smart,
            simple_direction: Direction::Forwards,
            disable_on_input: false,
            disable_on_y_change: false,
            no_unloaded_chunks: true,
        }
    }
}

/// Lifecycle effects must be executed in returned order. ReleaseDirections clears only W/S/A/D
/// mappings, including their rebound keys, as upstream's unpress does.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Effect {
    ReleaseDirections,
    StopPath,
    MovePathInDirection { yaw: f32 },
}

pub fn activate(settings: &Settings, yaw: f32) -> Vec<Effect> {
    if settings.mode == Mode::Smart {
        vec![Effect::MovePathInDirection { yaw }]
    } else {
        Vec::new()
    }
}

pub fn deactivate(settings: &Settings) -> Effect {
    if settings.mode == Mode::Simple {
        Effect::ReleaseDirections
    } else {
        Effect::StopPath
    }
}

/// Mirrors the settings' onChanged callbacks; can_update gates only mode changes.
pub fn settings_changed(
    previous: &Settings,
    next: &Settings,
    active: bool,
    can_update: bool,
    yaw: f32,
) -> Vec<Effect> {
    if !active {
        return Vec::new();
    }
    let mut effects = Vec::new();
    if previous.mode != next.mode && can_update {
        effects.push(if next.mode == Mode::Simple {
            Effect::StopPath
        } else {
            Effect::MovePathInDirection { yaw }
        });
        effects.push(Effect::ReleaseDirections);
    }
    if previous.simple_direction != next.simple_direction {
        effects.push(Effect::ReleaseDirections);
    }
    effects
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisableReason {
    VerticalMovement,
    MissingPathManager,
}

impl DisableReason {
    /// Upstream emits feedback only for the unavailable Smart-mode dependency.
    pub fn message(self) -> Option<&'static str> {
        match self {
            Self::VerticalMovement => None,
            Self::MissingPathManager => Some("Smart mode requires Baritone"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TickAction {
    None,
    Press(Direction),
    Disable(DisableReason),
}

/// Tick.Pre with HIGH priority. previous_y must be Entity.yo, not an activation height.
pub fn tick(
    settings: &Settings,
    previous_y: f64,
    y: f64,
    path_manager_available: bool,
) -> TickAction {
    match settings.mode {
        Mode::Simple if settings.disable_on_y_change && previous_y != y => {
            TickAction::Disable(DisableReason::VerticalMovement)
        }
        Mode::Simple => TickAction::Press(settings.simple_direction),
        Mode::Smart if !path_manager_available => {
            TickAction::Disable(DisableReason::MissingPathManager)
        }
        Mode::Smart => TickAction::None,
    }
}

/// Root input adapters match physical keyboard OR mouse bindings to these mapping identities.
pub const MANUAL_MOVEMENT_MAPPINGS: [&str; 6] = [
    "key.forward",
    "key.back",
    "key.left",
    "key.right",
    "key.sneak",
    "key.jump",
];

pub fn disable_for_input(
    settings: &Settings,
    press: bool,
    matches_movement_binding: bool,
    screen_open: bool,
    gui_move_active: bool,
    gui_move_skips: bool,
) -> bool {
    settings.disable_on_input
        && press
        && matches_movement_binding
        && (!screen_open || (gui_move_active && !gui_move_skips))
}

/// The source casts the divided coordinate to int (truncates toward zero), not floor/chunk >> 4.
pub fn destination_chunk(position: [f64; 3], movement: [f64; 3]) -> [i32; 2] {
    [
        ((position[0] + movement[0] * 2.0) / 16.0) as i32,
        ((position[2] + movement[2] * 2.0) / 16.0) as i32,
    ]
}

pub fn movement(settings: &Settings, movement: [f64; 3], destination_loaded: bool) -> [f64; 3] {
    if settings.mode == Mode::Simple && settings.no_unloaded_chunks && !destination_loaded {
        [0.0, movement[1], 0.0]
    } else {
        movement
    }
}
