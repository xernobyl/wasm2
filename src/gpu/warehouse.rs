//! Warehouse raymarch pass: fullscreen triangle + WGSL, single color output.

use bytemuck::{Pod, Zeroable};

use crate::view::ViewState;

/// Must match WGSL WarehouseUniforms layout (column-major matrices as [f32; 16]).
/// Padding fields align Rust offsets to WGSL rules: vec3 align 16, vec2 align 8.
/// resolution.z = 1/sqrt(width^2+height^2) for UV scaling (match webgl2/webgpu reference).
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct WarehouseUniforms {
    pub inverse_view: [f32; 16],                       // offset   0
    pub view_projection: [f32; 16],                    // offset  64
    pub inverse_view_projection: [f32; 16],            // offset 128
    pub inverse_projection: [f32; 16],                 // offset 192
    pub view_projection_no_jitter: [f32; 16],          // offset 256
    pub previous_view_projection_no_jitter: [f32; 16], // offset 320
    pub time: f32,                                     // offset 384
    _pad0: f32,                                        // offset 388
    _align_resolution: [f32; 2],                       // offset 392  (pad to reach 400 for vec3 align 16)
    /// (width, height, 1/sqrt(w*w+h*h))
    pub resolution: [f32; 3],                          // offset 400
    _pad_after_resolution: f32,                        // offset 412  (pad to 416 for vec2 align 8)
    pub viewport_origin: [f32; 2],                     // offset 416
    pub viewport_size: [f32; 2],                       // offset 424
    _pad1: [f32; 2],                                   // offset 432
    /// Diagonal FOV (radians) matching reference UV convention; zoom = 1/tan(fov/2).
    pub fov: f32,                                      // offset 440
    _struct_end_pad: f32,                              // offset 444  (round struct to 448 = 16*28)
}

impl WarehouseUniforms {
    /// When `background_only` is true, shader outputs solid gray (no raymarch). Use when warehouse is disabled so cubes draw on top.
    pub fn from_view(
        view: &ViewState,
        time_s: f32,
        fb_width: u32,
        fb_height: u32,
        background_only: bool,
    ) -> Self {
        let (vx, vy, vw, vh) = view.viewport;
        let w = fb_width as f32;
        let h = fb_height as f32;
        let inv_len = 1.0 / (w * w + h * h).sqrt();
        let aspect = vw as f32 / vh.max(1) as f32;
        let fov = 2.0 * ((view.fovy * 0.5).tan() * (1.0 + aspect * aspect).sqrt()).atan();
        Self {
            inverse_view: view.inverse_view.to_cols_array(),
            view_projection: view.view_projection.to_cols_array(),
            inverse_view_projection: view.view_projection.inverse().to_cols_array(),
            inverse_projection: view.projection.inverse().to_cols_array(),
            view_projection_no_jitter: view.view_projection_no_jitter.to_cols_array(),
            previous_view_projection_no_jitter: view.previous_view_projection_no_jitter.to_cols_array(),
            time: time_s,
            _pad0: 0.0,
            _align_resolution: [0.0; 2],
            resolution: [w, h, inv_len],
            _pad_after_resolution: 0.0,
            viewport_origin: [vx as f32, vy as f32],
            viewport_size: [vw as f32, vh as f32],
            _pad1: [if background_only { 1.0 } else { 0.0 }, 0.0],
            fov,
            _struct_end_pad: 0.0,
        }
    }
}

/// Fullscreen triangle in NDC: (-1,-1), (3,-1), (-1,3). One triangle covers the screen.
pub const FULLSCREEN_TRIANGLE: [f32; 6] = [-1.0, -1.0, 3.0, -1.0, -1.0, 3.0];
