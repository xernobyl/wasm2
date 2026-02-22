use crate::app::App;
use crate::chunk::{Chunk, ChunkMesh};
use crate::ecs::components::{BasePosition, HalfCube, OscillateMotion};
use crate::ecs::systems::half_cube_render_system;
use crate::ecs::{FrameResources, World};
use crate::fast_rand::FastRand;
use crate::line_2d_strip::Line2DStrip;
use crate::particles::Particles;
use crate::scene::Scene;
use crate::view::ViewState;
use glam::{Mat4, Vec3};

const CHUNK_N: usize = 16;

/// Cube count; VP is a single uniform, only model matrices are per-instance.
const N_CUBES: usize = 200;
/// Scale applied to each cube (1.0 = original size); 0.2 = 20% the size.
const CUBE_SCALE: f32 = 0.4;
/// Amplitude of back-and-forth motion along each cube's random axis (world units).
const MOTION_AMPLITUDE: f32 = 0.5;
/// Angular frequency for motion (radians per second).
const MOTION_SPEED: f32 = 2.0;

pub struct Scene1 {
    #[allow(dead_code)]
    line_strip: Line2DStrip,
    #[allow(dead_code)]
    particles: Particles,
    #[allow(dead_code)]
    particle_positions: Vec<f32>,
    #[allow(dead_code)]
    rng: FastRand,
    #[allow(dead_code)]
    chunk_mesh: ChunkMesh,

    /// ECS world: entities and components (moving half-cubes).
    world: World,
    /// Reused every frame for instanced draw (filled by half_cube_render_system).
    model_matrices: Vec<Mat4>,
}

impl Scene1 {
    pub fn new(app: &App) -> Self {
        let mut particle_positions = Vec::new();
        let mut rng = FastRand::new(3464357);

        for _ in 0..32 {
            particle_positions.push(rng.rand());
            particle_positions.push(rng.rand());
            particle_positions.push(rng.rand());
        }

        let mut chunk = Chunk::new(CHUNK_N, CHUNK_N, CHUNK_N);
        chunk.fill_hollow_box();
        let chunk_mesh = ChunkMesh::from_chunk(&chunk);

        let mut world = World::new();

        let mut pos_rng = FastRand::new(453455);
        let mut motion_rng = FastRand::new(789012);
        for _ in 0..N_CUBES {
            let base_position = BasePosition(Vec3::new(
                4.0 * (pos_rng.urand() * 2.0 - 1.0),
                2.0 * (pos_rng.urand() * 2.0 - 1.0) - 4.0,
                4.0 * (pos_rng.urand() * 2.0 - 1.0),
            ));
            let mut axis = Vec3::new(
                motion_rng.urand() * 2.0 - 1.0,
                motion_rng.urand() * 2.0 - 1.0,
                motion_rng.urand() * 2.0 - 1.0,
            )
            .normalize_or_zero();
            if axis.length_squared() < 0.01 {
                axis = Vec3::X;
            }
            let motion = OscillateMotion {
                axis,
                phase: motion_rng.urand() * std::f32::consts::TAU,
                amplitude: MOTION_AMPLITUDE,
                speed: MOTION_SPEED,
            };
            let half_cube = HalfCube {
                scale: CUBE_SCALE,
            };
            world.spawn_moving_half_cube(base_position, motion, half_cube);
        }

        let model_matrices = Vec::with_capacity(N_CUBES);

        Self {
            line_strip: Line2DStrip::new(),
            particles: Particles::new(),
            particle_positions,
            rng,
            chunk_mesh,
            world,
            model_matrices,
        }
    }
}

impl Scene for Scene1 {
    fn on_frame(
        &mut self,
        app: &mut App,
        view: &ViewState,
        pass: Option<&mut wgpu::RenderPass<'_>>,
        is_gbuffer: bool,
    ) {
        let time_s = (app.current_timestamp / 1000.0) as f32;
        let camera_position = Vec3::new(
            view.inverse_view.col(3).x,
            view.inverse_view.col(3).y,
            view.inverse_view.col(3).z,
        );

        let resources = FrameResources {
            time_s,
            view,
            camera_position,
        };

        half_cube_render_system(
            &self.world,
            &resources,
            app,
            &mut self.model_matrices,
            pass,
            is_gbuffer,
        );
    }
}

impl Drop for Scene1 {
    fn drop(&mut self) {}
}
