use glam::{Mat4, Vec3};

pub struct Camera {
    projection_matrix: Mat4,
    view_matrix: Mat4,
    view_projection_matrix: Mat4,
    field_of_view: f32, // diagonal, in radians
    aspect_ratio: f32,
    position: Vec3,
    // direction: Quaternion or a Matrix?
}

impl Camera {
    pub fn new() -> Self {
        Camera {}
    }

    pub fn look_at(position: Vec3, target: Vec3, up: Vec3) {
        Mat4::
    }
}
