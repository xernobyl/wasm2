//! View state and camera abstraction for mono, stereo, and (future) XR.
//!
//! [`ViewState`] holds the matrices and parameters needed for one render pass
//! (one eye or mono). The app runs the pipeline once per [`ViewState`].

use glam::{Mat4, Vec3};

use crate::stereo_camera::Eye;

/// Immutable view state for one render pass (one eye or mono).
/// Passed to warehouse, scene, and any pass that needs view/projection.
/// Velocity and TAA reprojection use the no-jitter matrices so motion is not mixed with subpixel jitter.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct ViewState {
    pub view: Mat4,
    pub projection: Mat4,
    pub inverse_view: Mat4,
    pub view_projection: Mat4,
    pub previous_view_projection: Mat4,
    /// Same as view_projection but without subpixel jitter; use for velocity buffer and TAA.
    pub view_projection_no_jitter: Mat4,
    /// Same as previous_view_projection but without jitter; use for velocity buffer and TAA.
    pub previous_view_projection_no_jitter: Mat4,
    pub fovy: f32,
    pub eye: Eye,
    /// Viewport (x, y, width, height) for this eye; (0,0,full_w,full_h) for mono.
    pub viewport: (i32, i32, i32, i32),
}

impl ViewState {
    pub fn from_matrices(
        view: Mat4,
        projection: Mat4,
        previous_view_projection: Mat4,
        view_projection_no_jitter: Mat4,
        previous_view_projection_no_jitter: Mat4,
        fovy: f32,
        eye: Eye,
        viewport: (i32, i32, i32, i32),
    ) -> Self {
        let inverse_view = view.inverse();
        let view_projection = projection * view;
        ViewState {
            view,
            projection,
            inverse_view,
            view_projection,
            previous_view_projection,
            view_projection_no_jitter,
            previous_view_projection_no_jitter,
            fovy,
            eye,
            viewport,
        }
    }

    pub fn direction(&self) -> Vec3 {
        let m = self.inverse_view;
        Vec3::new(-m.col(2).x, -m.col(2).y, -m.col(2).z)
    }
}
