struct Camera {
  projection_matrix: Mat4,
  view_matrix: Mat4,
  view_projection_matrix: Mat4,
  field_of_view: f32,
  aspect_ratio: f32,
  position: Vec3,
  // direction: Quaternion or a Matrix?
}

impl Camera {
  fn new() -> Self {
    Camera {

    }
  }
}
