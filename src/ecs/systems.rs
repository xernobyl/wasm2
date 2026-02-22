//! Systems: logic that runs over world and resources.
//! Each system is a pure function (world, resources, app) for testability and clarity.

use glam::{Mat4, Vec3};
use wgpu::RenderPass;

use crate::app::App;
use crate::ecs::resources::FrameResources;
use crate::ecs::world::World;
use crate::utils;

/// Builds model matrices for moving half-cubes. Each instance is translation + scale only;
/// the vertex shader flips the 3 stored faces by camera octant so they look like a full cube.
pub fn half_cube_render_system(
    world: &World,
    resources: &FrameResources<'_>,
    app: &mut App,
    model_matrices: &mut Vec<Mat4>,
    pass: Option<&mut RenderPass<'_>>,
    is_gbuffer: bool,
) {
    let view = resources.view;
    let time_s = resources.time_s;

    model_matrices.clear();
    model_matrices.reserve(world.moving_half_cube_count());

    for (base_pos, axis, phase, amplitude, speed, scale) in world.moving_cubes.iter() {
        let offset = axis * (amplitude * (time_s * speed + phase).sin());
        let pos = base_pos + offset;
        let scale_mat = Mat4::from_scale(Vec3::splat(scale));
        let model = Mat4::from_translation(pos) * scale_mat;
        model_matrices.push(model);
    }

    if model_matrices.is_empty() {
        return;
    }

    app.cube
        .update_model(utils::as_f32_slice(model_matrices.as_slice(), 4 * 4));
    if is_gbuffer {
        if let Some(p) = pass {
            app.cube
                .draw_instanced_gbuffer(p, view, model_matrices.len() as i32);
        }
    } else {
        app.cube
            .draw_instanced(model_matrices.len() as i32, pass, view);
    }
}
