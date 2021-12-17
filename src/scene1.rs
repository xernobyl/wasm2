use crate::app::App;
use crate::fast_rand::FastRand;
use crate::line_2d_strip::Line2DStrip;
use crate::particles::Particles;
use crate::scene::Scene;
use crate::shaders::Programs;
use crate::{fast_rand, utils};
use glam::{Mat4, Vec3};
use web_sys::WebGl2RenderingContext;

type Gl = WebGl2RenderingContext;

pub struct Scene1 {
    line_strip: Line2DStrip,
    particles: Particles,
    particle_positions: Vec<f32>,
    rng: FastRand,
}

impl Scene1 {
    pub fn new(app: &App) -> Self {
        let mut particle_positions = Vec::new();
        let mut rng = fast_rand::FastRand::new(3464357);

        for _ in 0..32 {
            particle_positions.push(rng.rand());
            particle_positions.push(rng.rand());
            particle_positions.push(rng.rand());
        }

        Self {
            line_strip: Line2DStrip::new(&app.context),
            particles: Particles::new(&app.context),
            particle_positions,
            rng,
        }
    }
}

fn iq_palette(
    t: f32,
    a: (f32, f32, f32),
    b: (f32, f32, f32),
    c: (f32, f32, f32),
    d: (f32, f32, f32),
) -> (f32, f32, f32) {
    const TAU: f32 = std::f32::consts::TAU;

    (
        a.0 + b.0 * f32::cos(TAU * (c.0 * t + d.0)),
        a.1 + b.1 * f32::cos(TAU * (c.1 * t + d.1)),
        a.2 + b.2 * f32::cos(TAU * (c.2 * t + d.2)),
    )
}

impl Scene for Scene1 {
    fn on_frame(&mut self, app: &App) {
        /*
        Use infinite inverted depth buffer because of the better precision
        */

        let gl = &app.context;

        // log!0"Frame: {}\nTimestamp: {}", self.current_frame, self.current_timestamp);

        app.fullscreen_buffers.bind(gl);
        let mut rng = fast_rand::FastRand::new(453455);

        let bg_col = iq_palette(
            app.current_timestamp as f32 / 5000.0,
            (0.5, 0.5, 0.5),
            (0.5, 0.5, 0.5),
            (1.0, 1.0, 0.5),
            (0.80, 0.90, 0.30),
        );

        gl.clear_color(bg_col.0, bg_col.1, bg_col.2, 0.0);
        gl.clear(Gl::DEPTH_BUFFER_BIT | Gl::COLOR_BUFFER_BIT);

        let camera_position = Vec3::new(
            15.0 * f32::cos(app.current_timestamp as f32 / 2000.0),
            10.0 * f32::sin(app.current_timestamp as f32 / 2000.0),
            15.0 * f32::sin(app.current_timestamp as f32 / 2000.0),
        );
        let projection = Mat4::perspective_infinite_reverse_rh(
            std::f32::consts::PI / 2.0,
            app.aspect_ratio,
            0.1,
        );
        let view = Mat4::look_at_rh(
            camera_position,
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );

        let view_projection = projection * view;

        let mut view_projection_array = Vec::new();

        for _ in 0..128 {
            let cube_pos = Vec3::new(5.0 * rng.rand(), 5.0 * rng.rand(), 5.0 * rng.rand());
            let mv = Mat4::from_translation(cube_pos);

            // camera aligned cubes... save this for later, probably do it in shader
            let mut dir = camera_position - cube_pos;
            dir.x = if dir.x >= 0.0 { 1.0 } else { -1.0 };
            dir.y = if dir.y >= 0.0 { 1.0 } else { -1.0 };
            dir.z = if dir.z >= 0.0 { 1.0 } else { -1.0 };
            let scale = Mat4::from_scale(dir);

            view_projection_array.push(view_projection * mv * scale);
        }

        app.cube
            .update_mvp(utils::as_f32_slice(view_projection_array.as_slice(), 4 * 4));

        gl.use_program(app.programs[Programs::Cube as usize].as_ref());
        let location = gl.get_uniform_location(
            app.programs[Programs::Cube as usize].as_ref().unwrap(),
            "camera_position",
        );
        gl.uniform3f(
            location.as_ref(),
            camera_position.x,
            camera_position.y,
            camera_position.z,
        );

        gl.depth_func(Gl::GREATER);
        gl.clear_depth(0.0);
        gl.enable(Gl::DEPTH_TEST);
        app.cube.draw_instanced(view_projection_array.len() as i32);
        gl.disable(Gl::DEPTH_TEST);

        let mut lines = Vec::new();
        const N_SEGMENTS: i32 = 37;

        for i in 0..N_SEGMENTS {
            let t = i as f32 / (N_SEGMENTS as f32) * std::f32::consts::TAU
                - app.current_timestamp as f32 / 4096.0;
            lines.push(0.75 * f32::sin(t));
            lines.push(0.75 * f32::cos(t));
            lines.push(
                rng.urand() * 0.25
                    + 0.25
                        * (0.75 + 0.25 * f32::sin(i as f32 + app.current_timestamp as f32 / 500.0)),
            );
        }

        // reconnect to first point, and tail data stuff
        lines.push(lines[0]);
        lines.push(lines[1]);
        lines.push(lines[2]);

        lines.push(lines[3]);
        lines.push(lines[4]);
        lines.push(lines[5]);

        // dummy data
        lines.push(lines[6]);
        lines.push(lines[7]);
        lines.push(lines[8]);

        self.line_strip.update_points(gl, lines.as_slice());
        gl.use_program(app.programs[Programs::Line2DStrip as usize].as_ref());
        self.line_strip.draw(gl, lines.len() as i32 / 3 - 3);

        // screen pass
        gl.bind_framebuffer(Gl::FRAMEBUFFER, None);
        gl.viewport(0, 0, app.width as i32, app.height as i32);

        gl.use_program(app.programs[Programs::Screen as usize].as_ref());
        let location = gl.get_uniform_location(
            app.programs[Programs::Cube as usize].as_ref().unwrap(),
            "color_texture",
        );
        gl.active_texture(Gl::TEXTURE0);
        gl.bind_texture(Gl::TEXTURE_2D, Some(&app.fullscreen_buffers.color_texture));
        gl.uniform1i(location.as_ref(), 0);
        utils::fullscreen_quad(gl);
    }
}

impl Drop for Scene1 {
    fn drop(&mut self) {
        // let _gl: &Gl = self.app.context.borrow();
        // TODO: destroy stuff
    }
}
