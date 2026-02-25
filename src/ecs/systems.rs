//! Systems: logic that runs over world and resources.
//! Each system is a pure function (world, resources, app) for testability and clarity.

use glam::Vec3;
use wgpu::RenderPass;

use crate::app::App;
use crate::ecs::resources::FrameResources;
use crate::ecs::world::World;

/// Builds packed instance data (x, y, z, scale) for moving half-cubes and draws.
/// The vertex shader flips the 3 stored faces by camera octant so they look like a full cube.
pub fn half_cube_render_system(
    world: &World,
    resources: &FrameResources<'_>,
    app: &mut App,
    instance_data: &mut Vec<f32>,
    pass: Option<&mut RenderPass<'_>>,
    is_gbuffer: bool,
) {
    let view = resources.view;
    let time_s = resources.time_s;

    instance_data.clear();
    instance_data.reserve(world.moving_half_cube_count() * 4);

    for (base_pos, axis, phase, amplitude, speed, scale) in world.moving_cubes.iter() {
        let offset = axis * (amplitude * (time_s * speed + phase).sin());
        let pos = base_pos + offset;
        instance_data.push(pos.x);
        instance_data.push(pos.y);
        instance_data.push(pos.z);
        instance_data.push(scale);
    }

    if instance_data.is_empty() {
        return;
    }

    app.cube.update_instances(instance_data);
    let count = instance_data.len() / 4;
    if is_gbuffer {
        if let Some(p) = pass {
            app.cube.draw_instanced_gbuffer(p, view, count as i32);
        }
    } else {
        app.cube.draw_instanced(count as i32, pass, view);
    }
}
