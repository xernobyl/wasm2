//! Per-frame resources passed into systems.
//! Read-only from the point of view of systems (time, view, camera position).

use glam::Vec3;

use crate::view::ViewState;

/// Resources available to systems for the current frame and view.
#[derive(Debug)]
pub struct FrameResources<'a> {
    /// Time in seconds (e.g. from app.current_timestamp).
    pub time_s: f32,
    /// Current view state (matrices, viewport, eye).
    pub view: &'a ViewState,
    /// Camera position in world space (from view.inverse_view).
    pub camera_position: Vec3,
}
