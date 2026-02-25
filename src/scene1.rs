use crate::app::App;
use crate::chunk::{Chunk, ChunkMesh};
use crate::ecs::components::{BasePosition, HalfCube, OscillateMotion};
use crate::ecs::systems::half_cube_render_system;
use crate::ecs::{FrameResources, World};
use crate::fast_rand::FastRand;
use crate::line_2d_strip::Line2DStrip;
use crate::particles::Particles;
use crate::scene::{CameraDescriptor, FrameInput, Scene, SceneDescriptor};
use crate::view::ViewState;
use glam::Vec3;
use std::f32::consts::FRAC_PI_2;
use std::f32::consts::FRAC_PI_4;

const MOUSE_SENSITIVITY: f32 = 0.002;
const MOVE_SPEED: f32 = 5.0;
const MAX_PITCH: f32 = 1.5;

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
    /// Reused every frame for instanced draw (filled by half_cube_render_system). Packed [x,y,z,scale] per instance.
    instance_data: Vec<f32>,

    descriptor: SceneDescriptor,

    yaw: f32,
    pitch: f32,
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
                4.0 * (pos_rng.urand() * 2.0 - 1.0) + 2.5,
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

        let instance_data = Vec::with_capacity(N_CUBES * 4);

        Self {
            line_strip: Line2DStrip::new(),
            particles: Particles::new(),
            particle_positions,
            rng,
            chunk_mesh,
            world,
            instance_data,
            descriptor: SceneDescriptor {
                camera: CameraDescriptor {
                    position: Vec3::new(-10.0, 1.7, -10.0),
                    target: Vec3::new(0.0, 1.7, 0.0),
                    up: Vec3::Y,
                    fov: FRAC_PI_2,
                },
            },
            yaw: FRAC_PI_4,
            pitch: 0.0,
        }
    }
}

impl Scene for Scene1 {
    fn descriptor(&self) -> &SceneDescriptor {
        &self.descriptor
    }

    fn update(&mut self, input: &FrameInput) {
        self.yaw -= input.mouse_dx * MOUSE_SENSITIVITY;
        self.pitch = (self.pitch - input.mouse_dy * MOUSE_SENSITIVITY).clamp(-MAX_PITCH, MAX_PITCH);

        let forward = Vec3::new(self.yaw.sin(), 0.0, self.yaw.cos());
        let right = Vec3::new(-self.yaw.cos(), 0.0, self.yaw.sin());

        let dt = (input.delta_time / 1000.0) as f32;
        let speed = MOVE_SPEED * dt;
        let mut movement = Vec3::ZERO;
        if input.key(FrameInput::KEY_W) { movement += forward * speed; }
        if input.key(FrameInput::KEY_S) { movement -= forward * speed; }
        if input.key(FrameInput::KEY_D) { movement += right * speed; }
        if input.key(FrameInput::KEY_A) { movement -= right * speed; }
        if input.key(FrameInput::KEY_SPACE) { movement.y += speed; }
        if input.key(FrameInput::KEY_SHIFT) { movement.y -= speed; }
        self.descriptor.camera.position += movement;

        let dir = Vec3::new(
            self.pitch.cos() * self.yaw.sin(),
            self.pitch.sin(),
            self.pitch.cos() * self.yaw.cos(),
        );
        self.descriptor.camera.target = self.descriptor.camera.position + dir;
    }

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
            &mut self.instance_data,
            pass,
            is_gbuffer,
        );
    }
}

impl Drop for Scene1 {
    fn drop(&mut self) {}
}
