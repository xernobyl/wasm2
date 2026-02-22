use crate::app::App;
use crate::view::ViewState;
use wgpu::RenderPass;

pub trait Scene {
    /// When `pass` is `Some`, the scene may record draws (e.g. cubes) into it. When `is_gbuffer` is true, use [crate::half_cube::HalfCube::draw_instanced_gbuffer].
    fn on_frame(&mut self, app: &mut App, view: &ViewState, pass: Option<&mut RenderPass<'_>>, is_gbuffer: bool);
}
