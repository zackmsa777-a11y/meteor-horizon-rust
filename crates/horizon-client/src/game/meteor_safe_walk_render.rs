// SPDX-License-Identifier: GPL-3.0-only
// SafeWalk Render3DEvent adapter. Copyright (c) Meteor Development.
//! Include only with the `meteor` feature. Append these world-space lines to the frame's
//! on-top list before camera_relative. Meteor's event.renderer uses ALWAYS_PASS depth testing
//! with depth writes disabled; its separate depthRenderer is not used by SafeWalk.
use horizon_client::{
    gizmos::{Gizmos, Line},
    motion::PlayerMotion,
};
use meteor_client::modules::safe_walk::{self, Bounds, Settings};

/// The caller checks that SafeWalk is active and that a player/world is loaded. Unlike the
/// debug hitbox overlay, these helpers are independent of the F3 hitbox and F1 HUD flags.
pub(super) fn append(
    settings: &Settings,
    player: &mut PlayerMotion,
    on_top: &mut Vec<Line>,
) -> super::Result<()> {
    if !settings.sneak || !settings.render {
        return Ok(());
    }
    // Source draws getBoundingBox directly, without interpolating the player by partial ticks.
    let bounds = player.bounds();
    let min = bounds.min_position();
    let max = bounds.max_position();
    let bounds = Bounds::new([min.x, min.y, min.z], [max.x, max.y, max.z]);
    let step = f64::from(player.attribute("minecraft:step_height")? as f32);
    let mut geometry = Gizmos::default();
    for outline in safe_walk::outlines(settings, bounds, step) {
        let [r, g, b, a] = outline.line_rgba;
        let argb = u32::from_be_bytes([a, r, g, b]);
        // Meteor DEBUG_LINES uses the default one-pixel stroke. Horizon expands lines into
        // screen-space quads; its rasterized edge smoothing can differ from GL line smoothing.
        geometry.cuboid(outline.bounds.min, outline.bounds.max, argb, 1.0);
    }
    on_top.extend(geometry.lines);
    Ok(())
}
