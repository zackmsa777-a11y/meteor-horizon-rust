// SPDX-License-Identifier: GPL-3.0-only
// Port of SafeWalk.java. Copyright (c) Meteor Development.
//! Ledge-event decisions and render geometry. Collision, heightmap and raycast queries are engine-owned.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub minimum_fall_distance: i32,
    pub sneak: bool,
    pub safe_sneak: bool,
    pub sneak_on_sprint: bool,
    pub edge_distance: f64,
    pub render: bool,
    pub render_player_box: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            minimum_fall_distance: 1,
            sneak: false,
            safe_sneak: true,
            sneak_on_sprint: true,
            edge_distance: 0.30,
            render: false,
            render_player_box: false,
        }
    }
}

impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if self.minimum_fall_distance < 1 {
            return Err("minimum-fall-distance must be at least 1".into());
        }
        // The source's slider range is not a hard range: negative/greater-than-.30 values are valid.
        if !self.edge_distance.is_finite() {
            return Err("edge-distance must be finite".into());
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Player {
    pub y: f64,
    pub block_y: i32,
    pub on_ground: bool,
    pub shift_key_down: bool,
    pub sprint_key_down: bool,
}

/// Raycast uses COLLIDER blocks and WATER fluids, down to the world's min Y. A hit is represented
/// by its *block* Y; the policy subtracts above().Y rather than the intersection's fractional Y.
#[derive(Clone, Copy, Debug)]
pub struct FallProbe {
    pub motion_blocking_surface_y: i32,
    pub downward_hit_block_y: Option<i32>,
}

pub fn sufficient_drop(settings: &Settings, player: Player, probe: FallProbe) -> bool {
    if settings.minimum_fall_distance <= 1 {
        return true;
    }
    if player.block_y >= probe.motion_blocking_surface_y {
        player.block_y.wrapping_sub(probe.motion_blocking_surface_y)
            >= settings.minimum_fall_distance
    } else {
        probe.downward_hit_block_y.is_none_or(|hit_y| {
            (player.y - f64::from(hit_y.wrapping_add(1))) as i32 >= settings.minimum_fall_distance
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Decision {
    /// Set ClipAtLedgeEvent.clip to true, leaving its previous value alone when false.
    pub force_clip: bool,
    /// Set only the sneak bit in Input; preserve every other movement/jump/sprint bit.
    pub press_sneak: bool,
}

pub fn clip(
    settings: &Settings,
    player: Player,
    probe: FallProbe,
    adjusted_box_has_no_collision: bool,
) -> Decision {
    if !sufficient_drop(settings, player, probe) {
        return Decision::default();
    }
    if !settings.sneak {
        return Decision {
            force_clip: !player.shift_key_down,
            press_sneak: false,
        };
    }
    if !settings.sneak_on_sprint && player.sprint_key_down {
        return Decision::default();
    }
    if adjusted_box_has_no_collision && player.on_ground {
        Decision {
            force_clip: false,
            press_sneak: true,
        }
    } else {
        Decision {
            force_clip: settings.safe_sneak,
            press_sneak: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub min: [f64; 3],
    pub max: [f64; 3],
}

impl Bounds {
    /// Matches AABB's constructor normalization, including inflated boxes that cross their center.
    pub fn new(a: [f64; 3], b: [f64; 3]) -> Self {
        Self {
            min: std::array::from_fn(|i| a[i].min(b[i])),
            max: std::array::from_fn(|i| a[i].max(b[i])),
        }
    }
}

pub fn adjusted_box(player: Bounds, max_up_step: f64, edge_distance: f64) -> Bounds {
    let mut min = player.min;
    let mut max = player.max;
    let vertical = -max_up_step;
    if vertical < 0.0 {
        min[1] += vertical;
    } else if vertical > 0.0 {
        max[1] += vertical;
    }
    min[0] += edge_distance;
    min[2] += edge_distance;
    max[0] -= edge_distance;
    max[2] -= edge_distance;
    Bounds::new(min, max)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Outline {
    pub bounds: Bounds,
    pub line_rgba: [u8; 4],
}

/// Render3D: outlines only (ShapeMode.Lines), no filled faces, exclude-direction mask zero.
pub fn outlines(settings: &Settings, player: Bounds, max_up_step: f64) -> Vec<Outline> {
    if !settings.sneak || !settings.render {
        return Vec::new();
    }
    let mut result = vec![Outline {
        bounds: adjusted_box(player, max_up_step, settings.edge_distance),
        line_rgba: [255, 0, 0, 255],
    }];
    if settings.render_player_box {
        result.push(Outline {
            bounds: player,
            line_rgba: [0, 255, 0, 255],
        });
    }
    result
}
