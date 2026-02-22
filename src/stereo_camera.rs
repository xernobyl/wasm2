//! Stereo camera for VR and stereo rendering.
//!
//! Provides left/right eye view and projection matrices with configurable
//! inter-pupillary distance (IPD) and convergence (distance or angle).
//! Uses inverted infinite (reversed Z) projection; no far plane.

use glam::{Mat4, Vec3};

use crate::projection::perspective_off_axis_infinite_reversed_z;
use crate::view::ViewState;

/// Which eye we are rendering for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Eye {
    /// Single view (non-stereo).
    Mono,
    /// Left eye.
    Left,
    /// Right eye.
    Right,
}

impl Eye {
    #[allow(dead_code)]
    pub fn is_stereo(self) -> bool {
        matches!(self, Eye::Left | Eye::Right)
    }
}

/// Stereo camera: two eye views with configurable IPD and convergence.
///
/// Uses the same world-space pose (position, target, up) as a mono camera;
/// left and right eyes are offset along the camera's right vector.
/// Convergence is the distance (world units) at which the two eye views align;
/// it can be set directly or derived from a convergence angle.
pub struct StereoCamera {
    /// Pose (shared by both eyes).
    position: Vec3,
    target: Vec3,
    up: Vec3,

    /// Vertical FOV (radians).
    fovy: f32,
    /// Aspect ratio (width/height) per eye.
    aspect: f32,
    near: f32,
    /// Kept for API compatibility; projection is infinite far (ignored).
    #[allow(dead_code)]
    far: f32,

    /// Inter-pupillary distance in world units (e.g. 0.1).
    ipd: f32,
    /// Convergence distance in world units. At this distance the two eyes' views align.
    convergence_distance: f32,

    /// Cached view matrices (updated when pose changes).
    view_left: Mat4,
    view_right: Mat4,
    inverse_view_left: Mat4,
    inverse_view_right: Mat4,

    /// Cached projection matrices (updated when projection params or convergence change).
    projection_left: Mat4,
    projection_right: Mat4,

    /// Cached view-projection and previous frame (for TAA velocity and reprojection).
    view_projection_left: Mat4,
    view_projection_right: Mat4,
    previous_view_projection_left: Mat4,
    previous_view_projection_right: Mat4,
}

impl StereoCamera {
    pub fn new(fovy: f32, aspect: f32, near: f32, far: f32) -> Self {
        let mut c = StereoCamera {
            position: Vec3::ZERO,
            target: Vec3::NEG_Z,
            up: Vec3::Y,
            fovy,
            aspect,
            near,
            far,
            ipd: 0.1,
            convergence_distance: 2.0,
            view_left: Mat4::IDENTITY,
            view_right: Mat4::IDENTITY,
            inverse_view_left: Mat4::IDENTITY,
            inverse_view_right: Mat4::IDENTITY,
            projection_left: Mat4::IDENTITY,
            projection_right: Mat4::IDENTITY,
            view_projection_left: Mat4::IDENTITY,
            view_projection_right: Mat4::IDENTITY,
            previous_view_projection_left: Mat4::IDENTITY,
            previous_view_projection_right: Mat4::IDENTITY,
        };
        c.update_view();
        c.update_projection();
        c.update();
        c
    }

    /// Set inter-pupillary distance in world units (default 0.1).
    pub fn set_eye_distance(&mut self, ipd: f32) {
        self.ipd = ipd.max(0.0);
        self.update_view();
        self.update_projection();
    }

    /// Inter-pupillary distance in world units.
    #[allow(dead_code)]
    pub fn eye_distance(&self) -> f32 {
        self.ipd
    }

    /// Set convergence by distance (world units). At this distance the two eyes align.
    pub fn set_convergence_distance(&mut self, distance: f32) {
        self.convergence_distance = distance.max(0.01);
        self.update_projection();
    }

    /// Set convergence by half-angle (radians). The angle between each eye's gaze and the center line.
    /// `convergence_distance = (ipd/2) / tan(angle)`.
    #[allow(dead_code)]
    pub fn set_convergence_angle(&mut self, angle_rad: f32) {
        let half = angle_rad.abs().min(std::f32::consts::FRAC_PI_2 - 0.01);
        self.convergence_distance = (self.ipd * 0.5) / half.tan();
        self.convergence_distance = self.convergence_distance.max(0.01);
        self.update_projection();
    }

    /// Convergence distance in world units.
    #[allow(dead_code)]
    pub fn convergence_distance(&self) -> f32 {
        self.convergence_distance
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
        self.update_projection();
    }

    #[allow(dead_code)]
    pub fn set_fovy(&mut self, fovy: f32) {
        self.fovy = fovy;
        self.update_projection();
    }

    #[allow(dead_code)]
    pub fn fovy(&self) -> f32 {
        self.fovy
    }

    pub fn look_at(&mut self, position: Vec3, target: Vec3, up: Vec3) {
        self.position = position;
        self.target = target;
        self.up = up;
        self.update_view();
    }

    /// Call once per frame after setting pose/aspect; copies current view-projection to previous and recomputes current (for TAA).
    pub fn update(&mut self) {
        self.previous_view_projection_left = self.view_projection_left;
        self.previous_view_projection_right = self.view_projection_right;
        self.view_projection_left = self.projection_left * self.view_left;
        self.view_projection_right = self.projection_right * self.view_right;
    }

    fn update_view(&mut self) {
        let forward = (self.target - self.position).normalize_or_zero();
        let right = forward.cross(self.up).normalize_or_zero();
        let half = self.ipd * 0.5;
        let pos_left = self.position - right * half;
        let pos_right = self.position + right * half;
        self.view_left = Mat4::look_at_rh(pos_left, self.target, self.up);
        self.view_right = Mat4::look_at_rh(pos_right, self.target, self.up);
        self.inverse_view_left = self.view_left.inverse();
        self.inverse_view_right = self.view_right.inverse();
    }

    fn update_projection(&mut self) {
        let half_h = self.near * (self.fovy * 0.5).tan();
        let half_w = half_h * self.aspect;
        let shift = (self.ipd * 0.5) * self.near / self.convergence_distance;
        self.projection_left = perspective_off_axis_infinite_reversed_z(
            -half_w + shift,
            half_w + shift,
            -half_h,
            half_h,
            self.near,
        );
        self.projection_right = perspective_off_axis_infinite_reversed_z(
            -half_w - shift,
            half_w - shift,
            -half_h,
            half_h,
            self.near,
        );
    }

    pub fn view(&self, eye: Eye) -> Mat4 {
        match eye {
            Eye::Mono => self.view_left,
            Eye::Left => self.view_left,
            Eye::Right => self.view_right,
        }
    }

    #[allow(dead_code)]
    pub fn inverse_view(&self, eye: Eye) -> Mat4 {
        match eye {
            Eye::Mono => self.inverse_view_left,
            Eye::Left => self.inverse_view_left,
            Eye::Right => self.inverse_view_right,
        }
    }

    pub fn projection(&self, eye: Eye) -> Mat4 {
        match eye {
            Eye::Mono => self.projection_left,
            Eye::Left => self.projection_left,
            Eye::Right => self.projection_right,
        }
    }

    pub fn view_projection(&self, eye: Eye) -> Mat4 {
        match eye {
            Eye::Mono => self.view_projection_left,
            Eye::Left => self.view_projection_left,
            Eye::Right => self.view_projection_right,
        }
    }

    #[allow(dead_code)]
    pub fn position(&self, eye: Eye) -> Vec3 {
        let inv = self.inverse_view(eye);
        Vec3::new(inv.col(3).x, inv.col(3).y, inv.col(3).z)
    }

    /// Forward direction (same for both eyes in world space).
    #[allow(dead_code)]
    pub fn direction(&self) -> Vec3 {
        (self.target - self.position).normalize_or_zero()
    }

    /// Build left and right [`ViewState`]s for stereo rendering.
    /// `viewport_full` is (width, height) of the full framebuffer; left uses left half, right uses right half.
    pub fn to_view_states(&self, viewport_full: (i32, i32)) -> Vec<ViewState> {
        let (w, h) = viewport_full;
        let half_w = w / 2;
        vec![
            ViewState::from_matrices(
                self.view_left,
                self.projection_left,
                self.previous_view_projection_left,
                self.view_projection_left,
                self.previous_view_projection_left,
                self.fovy,
                Eye::Left,
                (0, 0, half_w, h),
            ),
            ViewState::from_matrices(
                self.view_right,
                self.projection_right,
                self.previous_view_projection_right,
                self.view_projection_right,
                self.previous_view_projection_right,
                self.fovy,
                Eye::Right,
                (half_w, 0, w - half_w, h),
            ),
        ]
    }
}

