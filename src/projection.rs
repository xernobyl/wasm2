//! Projection matrices: infinite far plane + reversed Z.
//!
//! Matches [pezcode's gist](https://gist.github.com/pezcode/1609b61a1eedd207ec8c5acf6f94f53a)
//! (adapted for RH: clip_w = -z). See also [wgpu Reverse Z discussion](https://github.com/gfx-rs/wgpu/discussions/1497)
//! and [WebGPU reversedZ sample](https://webgpu.github.io/webgpu-samples/?sample=reversedZ).
//! - **Infinite far**: only near plane. **Reversed Z**: depth near → 1, far → 0.
//! - Use Depth32Float, GREATER (or GreaterEqual), and clear depth to 0.

use glam::Mat4;

/// Right-handed symmetric perspective, infinite far, reversed Z.
/// - `fovy`: vertical FOV in radians.
/// - `aspect`: width / height.
/// - `near`: near plane distance (positive).
///
/// NDC z: near → 1, infinity → 0. clip_z = near, clip_w = -z so NDC_z = near/(-z).
/// With depth range [0,1], use GREATER and clear to 0.
#[inline]
pub fn perspective_infinite_reversed_z(fovy: f32, aspect: f32, near: f32) -> Mat4 {
    let tan_half_fovy = (fovy * 0.5).tan();
    let h = 1.0 / tan_half_fovy;
    let w = h / aspect;
    Mat4::from_cols_array(&[
        w,
        0.0,
        0.0,
        0.0,
        0.0,
        h,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        -1.0,
        0.0,
        0.0,
        near,
        0.0,
    ])
}

/// Right-handed off-axis perspective, infinite far, reversed Z.
/// Frustum in view space: left, right, bottom, top at near plane, and near (positive).
///
/// Use for stereo: asymmetric frusta with convergence shift.
#[inline]
pub fn perspective_off_axis_infinite_reversed_z(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
) -> Mat4 {
    let rl = right - left;
    let tb = top - bottom;
    let n2 = 2.0 * near;
    Mat4::from_cols_array(&[
        n2 / rl,
        0.0,
        0.0,
        0.0,
        0.0,
        n2 / tb,
        0.0,
        0.0,
        (right + left) / rl,
        (top + bottom) / tb,
        1.0,
        -1.0,
        0.0,
        0.0,
        2.0 * near,
        0.0,
    ])
}
