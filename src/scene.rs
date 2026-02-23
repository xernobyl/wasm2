use crate::app::App;
use crate::view::ViewState;
use glam::Vec3;
use wgpu::RenderPass;

pub struct CameraDescriptor {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    /// Horizontal FOV in radians.
    pub fov: f32,
}

impl Default for CameraDescriptor {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, -5.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            fov: std::f32::consts::FRAC_PI_2,
        }
    }
}

pub struct SceneDescriptor {
    pub camera: CameraDescriptor,
}

pub struct FrameInput {
    pub timestamp: f64,
    pub delta_time: f64,
    pub mouse_dx: f32,
    pub mouse_dy: f32,
    pub keys_held: u32,
}

impl FrameInput {
    pub const KEY_W: u32 = 1 << 0;
    pub const KEY_A: u32 = 1 << 1;
    pub const KEY_S: u32 = 1 << 2;
    pub const KEY_D: u32 = 1 << 3;
    pub const KEY_SPACE: u32 = 1 << 4;
    pub const KEY_SHIFT: u32 = 1 << 5;

    pub fn key(&self, mask: u32) -> bool {
        self.keys_held & mask != 0
    }
}

pub trait Scene {
    fn descriptor(&self) -> &SceneDescriptor;

    /// Called once per frame before the engine reads the descriptor.
    fn update(&mut self, input: &FrameInput);

    /// When `pass` is `Some`, the scene may record draws (e.g. cubes) into it.
    /// When `is_gbuffer` is true, use [crate::half_cube::HalfCube::draw_instanced_gbuffer].
    fn on_frame(&mut self, app: &mut App, view: &ViewState, pass: Option<&mut RenderPass<'_>>, is_gbuffer: bool);
}
