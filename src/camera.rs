use glam::{Mat4, Vec3};

pub struct Camera {
    projection: Mat4,
    view: Mat4,
    view_projection: Mat4,
    field_of_view: f32, // diagonal, in radians
    aspect_ratio: f32,
    position: Vec3,
    // direction: Quaternion or a Matrix?
}

impl Camera {
    /// Creates a right-handed infininte inverted perspective projection matrix with [1, -1] depth range.
    fn perspective_infinite_inverted(fov_x_radians: f32, fov_y_radians: f32, z_near: f32) -> Mat4{
        let a = 1.0 / (0.5 * fov_x_radians).tan();
        let b = 1.0 / (0.5 * fov_y_radians).tan();
        let c = 2.0 * z_near;
        Mat4::from_cols(
            Vec4::new(a, 0.0, 0.0, 0.0),
            Vec4::new(0.0, b, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, -1.0),
            Vec4::new(0.0, 0.0, c, 0.0),
        )
    }

    pub fn new() -> Self {
        Camera {}
    }

    pub fn look_at(&mut self, position: Vec3, target: Vec3, up: Vec3) {
        let view = Mat4::look_at_rh(position, target, up);
        let perspective = perspective_infinite_inverted(self.fov_y, self.aspect_ratio, self.near);
    }

    pub fn fov_rad(&mut self, fov: f32) {
        let a = 1.0 / (self::aspect_ratio * self::aspect_ratio + 1.0);
        let aa = 1.0 - a;
        let fov2 = fov * fov;
        let fov_y = (fov2 * a).sqrt();
        let fov_x = (fov2 * aa).sqrt();
    }

    pub fn update(&mut self) {
        self::view_projection = self::projection * self::view;
    }
}
