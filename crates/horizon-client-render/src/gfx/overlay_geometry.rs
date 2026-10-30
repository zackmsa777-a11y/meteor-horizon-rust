// SPDX-License-Identifier: GPL-3.0-only
// Derived from Meteor Renderer3D.boxSides/boxLines and its position-color shader.
//! Unlit world overlays: straight RGBA, no fog, no alpha cutoff, no depth writes.
//! Lines precede faces, matching Renderer3D.render. Native wgpu lines do not promise
//! the OpenGL line-smoothing behavior requested by Meteor's original pipeline.

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColoredVertex {
    pub position: [f32; 3],
    pub color: [u8; 4],
}

#[derive(Default)]
pub struct Geometry {
    pub lines: Vec<ColoredVertex>,
    pub triangles: Vec<ColoredVertex>,
}

/// `excludeDir == 0`, as used by AirPlace. Source face/edge order is retained.
/// Camera subtraction precedes f32 conversion, preserving sub-block precision at
/// large world coordinates. `sides`/`lines` implement the caller's ShapeMode.
pub fn box_geometry(
    min: [f64; 3],
    max: [f64; 3],
    camera: [f64; 3],
    side_color: [u8; 4],
    line_color: [u8; 4],
    sides: bool,
    lines: bool,
) -> Geometry {
    let mut out = Geometry::default();
    if !(0..3).all(|i| {
        min[i].is_finite() && max[i].is_finite() && camera[i].is_finite() && min[i] <= max[i]
    }) {
        return out;
    }
    // blb, blf, brb, brf, tlb, tlf, trb, trf.
    let corners: [[f32; 3]; 8] = std::array::from_fn(|i| {
        let world = [
            if i & 2 == 0 { min[0] } else { max[0] },
            if i & 4 == 0 { min[1] } else { max[1] },
            if i & 1 == 0 { min[2] } else { max[2] },
        ];
        std::array::from_fn(|axis| (world[axis] - camera[axis]) as f32)
    });
    if corners.iter().flatten().any(|v| !v.is_finite()) {
        return out;
    }
    if lines {
        for [a, b] in [
            [0, 4],
            [1, 5],
            [2, 6],
            [3, 7],
            [0, 1],
            [2, 3],
            [0, 2],
            [1, 3],
            [4, 5],
            [6, 7],
            [4, 6],
            [5, 7],
        ] {
            out.lines.extend([a, b].map(|i| ColoredVertex {
                position: corners[i],
                color: line_color,
            }));
        }
    }
    if sides {
        for [a, b, c, d] in [
            [0, 1, 5, 4],
            [2, 6, 7, 3],
            [0, 4, 6, 2],
            [1, 3, 7, 5],
            [0, 2, 3, 1],
            [4, 5, 7, 6],
        ] {
            out.triangles
                .extend([a, b, c, c, d, a].map(|i| ColoredVertex {
                    position: corners[i],
                    color: side_color,
                }));
        }
    }
    out
}

// Appended to terrain.wgsl, reusing its exact Globals block and pipeline layout.
// Rust embedding keeps this code inside the native-source .hmod source contract.
pub(super) const SHADER: &str = r#"
struct WorldOverlayOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) color: vec4<f32>,
}
@vertex fn vs_world_overlay(@location(0) position: vec3<f32>, @location(1) color: vec4<f32>) -> WorldOverlayOut {
    var out: WorldOverlayOut;
    out.clip = globals.view_proj * vec4(position, 1.0);
    out.color = color;
    return out;
}
@fragment fn fs_world_overlay(in: WorldOverlayOut) -> @location(0) vec4<f32> {
    return in.color;
}
"#;

pub(super) fn shader() -> String {
    include_str!("../terrain.wgsl").to_owned() + SHADER
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn combined_shader_validates_overlay_entry_points() {
        let module = naga::front::wgsl::parse_str(&shader()).unwrap();
        let info = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .unwrap();
        for name in ["vs_world_overlay", "fs_world_overlay"] {
            let entry = module.entry_points.iter().find(|e| e.name == name).unwrap();
            let (module, info) = naga::back::pipeline_constants::process_overrides(
                &module,
                &info,
                Some((entry.stage, &entry.name)),
                &Default::default(),
            )
            .unwrap();
            let mut hlsl = String::new();
            naga::back::hlsl::Writer::new(
                &mut hlsl,
                &naga::back::hlsl::Options::default(),
                &naga::back::hlsl::PipelineOptions {
                    entry_point: Some((entry.stage, entry.name.clone())),
                },
            )
            .write(&module, &info, None)
            .unwrap();
            assert!(!hlsl.is_empty());
        }
    }
    #[test]
    fn box_has_source_edges_faces_order_and_preserves_low_alpha() {
        let mesh = box_geometry(
            [0.; 3],
            [1.; 3],
            [0.; 3],
            [204, 0, 0, 10],
            [204, 0, 0, 255],
            true,
            true,
        );
        assert_eq!(mesh.lines.len(), 24);
        assert_eq!(mesh.triangles.len(), 36);
        assert_eq!(mesh.lines[0].position, [0., 0., 0.]);
        assert_eq!(mesh.lines[1].position, [0., 1., 0.]);
        assert_eq!(mesh.triangles[0].position, [0., 0., 0.]);
        assert_eq!(mesh.triangles[1].position, [0., 0., 1.]);
        assert_eq!(mesh.triangles[2].position, [0., 1., 1.]);
        assert!(mesh.triangles.iter().all(|v| v.color == [204, 0, 0, 10]));
        assert!(mesh.lines.iter().all(|v| v.color == [204, 0, 0, 255]));
        assert_eq!(std::mem::size_of::<ColoredVertex>(), 16);
    }
    #[test]
    fn camera_subtraction_retains_fractional_edges_at_world_border() {
        let mesh = box_geometry(
            [30_000_000., 70., 30_000_000.],
            [30_000_001., 71., 30_000_001.],
            [30_000_000.25, 70.5, 30_000_000.75],
            [1; 4],
            [2; 4],
            true,
            true,
        );
        assert_eq!(mesh.lines[0].position, [-0.25, -0.5, -0.75]);
        assert_eq!(mesh.lines[1].position, [-0.25, 0.5, -0.75]);
    }
    #[test]
    fn shape_modes_and_invalid_geometry_do_not_emit_other_primitives() {
        for (sides, lines) in [(true, false), (false, true), (false, false)] {
            let mesh = box_geometry([0.; 3], [1.; 3], [0.; 3], [0; 4], [0; 4], sides, lines);
            assert_eq!(mesh.triangles.len(), if sides { 36 } else { 0 });
            assert_eq!(mesh.lines.len(), if lines { 24 } else { 0 });
        }
        for min in [[f64::NAN, 0., 0.], [2., 0., 0.]] {
            let mesh = box_geometry(min, [1.; 3], [0.; 3], [0; 4], [0; 4], true, true);
            assert!(mesh.triangles.is_empty() && mesh.lines.is_empty());
        }
    }
}
