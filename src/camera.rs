use glam::{Mat4, Vec3};

use crate::projection::perspective_infinite_reversed_z;
use crate::stereo_camera::Eye;
use crate::view::ViewState;

/// Camera with jitter support for TAA. Uses inverted infinite (reversed Z) projection; `far` is ignored.
pub struct Camera {
    jitter_x: f32,
    jitter_y: f32,
    aspect: f32,
    near: f32,
    #[allow(dead_code)]
    far: f32,
    fov: f32,  // horizontal FOV (radians)
    fovy: f32, // vertical FOV (radians), derived from fov and aspect
    projection: Mat4,
    projection_no_jitter: Mat4,
    view: Mat4,
    inverse_view: Mat4,
    view_projection: Mat4,
    previous_view_projection: Mat4,
    view_projection_no_jitter: Mat4,
    previous_view_projection_no_jitter: Mat4,
}

impl Camera {
    pub fn new(fov_rad: f32, aspect: f32, near: f32, far: f32) -> Self {
        let mut c = Camera {
            jitter_x: 0.0,
            jitter_y: 0.0,
            aspect,
            near,
            far,
            fov: fov_rad,
            fovy: 0.0,
            projection: Mat4::IDENTITY,
            projection_no_jitter: Mat4::IDENTITY,
            view: Mat4::IDENTITY,
            inverse_view: Mat4::IDENTITY,
            view_projection: Mat4::IDENTITY,
            previous_view_projection: Mat4::IDENTITY,
            view_projection_no_jitter: Mat4::IDENTITY,
            previous_view_projection_no_jitter: Mat4::IDENTITY,
        };
        c.set_fovy_from_fov();
        c.update_projection();
        c
    }

    fn set_fovy_from_fov(&mut self) {
        // Horizontal FOV fov; vertical FOV: tan(fovy/2) = tan(fov/2) / aspect.
        let half_fov = self.fov * 0.5;
        self.fovy = 2.0 * (half_fov.tan() / self.aspect).atan();
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
        self.set_fovy_from_fov();
        self.update_projection();
    }

    /// Kept for API / future use; do not remove.
    #[allow(dead_code)]
    pub fn set_fov(&mut self, fov: f32) {
        self.fov = fov;
        self.set_fovy_from_fov();
        self.update_projection();
    }

    /// Kept for API / future use; do not remove.
    #[allow(dead_code)]
    pub fn fov(&self) -> f32 {
        self.fov
    }

    /// Kept for API / view abstraction; do not remove.
    #[allow(dead_code)]
    pub fn fovy(&self) -> f32 {
        self.fovy
    }

    pub fn set_jitter(&mut self, x: f32, y: f32) {
        self.jitter_x = x;
        self.jitter_y = y;
        self.update_projection();
    }

    pub fn look_at(&mut self, position: Vec3, target: Vec3, up: Vec3) {
        self.view = Mat4::look_at_rh(position, target, up);
        self.inverse_view = self.view.inverse();
    }

    /// Build projection (inverted infinite, reversed Z). Jitter in [8],[9] for TAA.
    pub fn update_projection(&mut self) {
        self.projection_no_jitter =
            perspective_infinite_reversed_z(self.fovy, self.aspect, self.near);
        self.projection = self.projection_no_jitter;
        if self.jitter_x != 0.0 || self.jitter_y != 0.0 {
            let mut p = self.projection.to_cols_array();
            p[8] = self.jitter_x;
            p[9] = self.jitter_y;
            self.projection = Mat4::from_cols_array(&p);
        }
    }

    /// Call once per frame after setting view; copies previous VP and computes current VP.
    pub fn update(&mut self) {
        self.previous_view_projection = self.view_projection;
        self.previous_view_projection_no_jitter = self.view_projection_no_jitter;
        self.view_projection = self.projection * self.view;
        self.view_projection_no_jitter = self.projection_no_jitter * self.view;
    }

    /// Kept for API / future use; do not remove.
    #[allow(dead_code)]
    pub fn view(&self) -> Mat4 {
        self.view
    }

    /// Kept for API / view abstraction; do not remove.
    #[allow(dead_code)]
    pub fn inverse_view(&self) -> Mat4 {
        self.inverse_view
    }

    /// Kept for API / view abstraction; do not remove.
    #[allow(dead_code)]
    pub fn view_projection(&self) -> Mat4 {
        self.view_projection
    }

    /// Kept for API / view abstraction; do not remove.
    #[allow(dead_code)]
    pub fn previous_view_projection(&self) -> Mat4 {
        self.previous_view_projection
    }

    /// Kept for API / future use; do not remove.
    #[allow(dead_code)]
    pub fn view_projection_no_jitter(&self) -> Mat4 {
        self.view_projection_no_jitter
    }

    /// Kept for API / future use; do not remove.
    #[allow(dead_code)]
    pub fn previous_view_projection_no_jitter(&self) -> Mat4 {
        self.previous_view_projection_no_jitter
    }

    /// Forward direction (negative Z in view space transformed by inverse view).
    /// Kept for API / view abstraction; do not remove.
    #[allow(dead_code)]
    pub fn direction(&self) -> Vec3 {
        let m = self.inverse_view;
        Vec3::new(-m.col(2).x, -m.col(2).y, -m.col(2).z)
    }

    /// Build a single [`ViewState`] for the current frame (mono).
    pub fn to_view_state(&self, viewport: (i32, i32, i32, i32)) -> ViewState {
        ViewState::from_matrices(
            self.view,
            self.projection,
            self.previous_view_projection,
            self.view_projection_no_jitter,
            self.previous_view_projection_no_jitter,
            self.fovy,
            Eye::Mono,
            viewport,
        )
    }
}
