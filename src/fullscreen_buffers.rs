use web_sys::WebGl2RenderingContext;

type Gl = WebGl2RenderingContext;

pub struct ScreenBuffers {
    max_width: i32,
    max_height: i32,
    framebuffer: web_sys::WebGlFramebuffer,
    depth_texture: web_sys::WebGlTexture,
    pub color_texture: web_sys::WebGlTexture,
}

impl ScreenBuffers {
    pub fn init(gl: &Gl, max_width: &i32, max_height: &i32) -> Option<Self> {
        let depth_texture = gl.create_texture();
        gl.bind_texture(Gl::TEXTURE_2D, depth_texture.as_ref());
        #[allow(unused_must_use)] {
            gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                Gl::TEXTURE_2D,
                0,
                Gl::DEPTH_COMPONENT24 as i32,
                *max_width,
                *max_height,
                0,
                Gl::DEPTH_COMPONENT,
                Gl::UNSIGNED_INT,
                None,
            );
        }

        gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MAG_FILTER, Gl::NEAREST as i32);
        gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MIN_FILTER, Gl::NEAREST as i32);
        gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_S, Gl::CLAMP_TO_EDGE as i32);
        gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_T, Gl::CLAMP_TO_EDGE as i32);

        let color_texture = gl.create_texture();
        gl.bind_texture(Gl::TEXTURE_2D, color_texture.as_ref());
        #[allow(unused_must_use)] {
            gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                Gl::TEXTURE_2D,
                0,
                Gl::R11F_G11F_B10F as i32,
                *max_width,
                *max_height,
                0,
                Gl::RGB,
                Gl::UNSIGNED_INT_10F_11F_11F_REV,
                None,
            );
        }
        Self::set_linear_clamp(gl);

        let motion_texture = gl.create_texture();
        gl.bind_texture(Gl::TEXTURE_2D, motion_texture.as_ref());
        #[allow(unused_must_use)] {
            gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                Gl::TEXTURE_2D,
                0,
                Gl::RG16F as i32,
                *max_width,
                *max_height,
                0,
                Gl::RG,
                Gl::HALF_FLOAT,
                None,
            );
        }
        Self::set_linear_clamp(gl);

        /*
        Maybe do like doom and calculate this on the forward pass?

        let normal_roughness_texture = gl.create_texture();
        gl.bind_texture(Gl::TEXTURE_2D, normal_roughness_texture.as_ref());
        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
          Gl::TEXTURE_2D, 0, Gl::RGBA8 as i32, *max_width, *max_height, 0, Gl::RGBA, Gl::UNSIGNED_BYTE, None);
        Self::set_linear_clamp(gl);
        */

        let width = (*max_width >> 1) + (*max_width & 0x01 != 0) as i32;
        let height = (*max_height >> 1) + (*max_height & 0x01 != 0) as i32;
        let depth_min_max_1_texture = gl.create_texture();
        gl.bind_texture(Gl::TEXTURE_2D, depth_min_max_1_texture.as_ref());
        #[allow(unused_must_use)] {
            gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                Gl::TEXTURE_2D,
                0,
                Gl::RG16F as i32,
                width,
                height,
                0,
                Gl::RG,
                Gl::HALF_FLOAT,
                None,
            );
        }
        Self::set_linear_clamp(gl);

        let width = (*max_width >> 2) + (*max_width & 0x03 != 0) as i32;
        let height = (*max_height >> 2) + (*max_height & 0x03 != 0) as i32;
        let depth_min_max_2_texture = gl.create_texture();
        gl.bind_texture(Gl::TEXTURE_2D, depth_min_max_2_texture.as_ref());
        #[allow(unused_must_use)] {
            gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                Gl::TEXTURE_2D,
                0,
                Gl::RG16F as i32,
                width,
                height,
                0,
                Gl::RG,
                Gl::HALF_FLOAT,
                None,
            );
        }
        Self::set_linear_clamp(gl);

        let width = (*max_width >> 3) + (*max_width & 0x07 != 0) as i32;
        let height = (*max_height >> 3) + (*max_height & 0x07 != 0) as i32;
        let depth_min_max_3_texture = gl.create_texture();
        gl.bind_texture(Gl::TEXTURE_2D, depth_min_max_3_texture.as_ref());
        #[allow(unused_must_use)] {
            gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                Gl::TEXTURE_2D,
                0,
                Gl::RG16F as i32,
                width,
                height,
                0,
                Gl::RG,
                Gl::HALF_FLOAT,
                None,
            );
        }
        Self::set_linear_clamp(gl);

        let width = (*max_width >> 4) + (*max_width & 0x0f != 0) as i32;
        let height = (*max_height >> 4) + (*max_height & 0x0f != 0) as i32;
        let depth_min_max_4_texture = gl.create_texture();
        gl.bind_texture(Gl::TEXTURE_2D, depth_min_max_4_texture.as_ref());
        #[allow(unused_must_use)] {
            gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                Gl::TEXTURE_2D,
                0,
                Gl::RG16F as i32,
                width,
                height,
                0,
                Gl::RG,
                Gl::HALF_FLOAT,
                None,
            );
        }
        Self::set_linear_clamp(gl);

        let framebuffer = gl.create_framebuffer();
        gl.bind_framebuffer(Gl::FRAMEBUFFER, framebuffer.as_ref());
        gl.framebuffer_texture_2d(
            Gl::FRAMEBUFFER,
            Gl::DEPTH_ATTACHMENT,
            Gl::TEXTURE_2D,
            depth_texture.as_ref(),
            0,
        );
        gl.framebuffer_texture_2d(
            Gl::FRAMEBUFFER,
            Gl::COLOR_ATTACHMENT0,
            Gl::TEXTURE_2D,
            color_texture.as_ref(),
            0,
        );
        gl.framebuffer_texture_2d(
            Gl::FRAMEBUFFER,
            Gl::COLOR_ATTACHMENT1,
            Gl::TEXTURE_2D,
            motion_texture.as_ref(),
            0,
        );

        if gl.check_framebuffer_status(Gl::FRAMEBUFFER) != Gl::FRAMEBUFFER_COMPLETE {
            None
        } else {
            Some(Self {
                max_width: *max_width,
                max_height: *max_height,
                framebuffer: framebuffer.unwrap(),
                depth_texture: depth_texture.unwrap(),
                color_texture: color_texture.unwrap(),
            })
        }
    }

    /*
    pub fn generate_depth_mips(gl: &Gl) {
        // TODO
    }
    */

    pub fn bind(&self, gl: &Gl) {
        gl.bind_framebuffer(Gl::FRAMEBUFFER, Some(&self.framebuffer));
        gl.viewport(0, 0, self.max_width, self.max_height);
    }

    fn set_linear_clamp(gl: &Gl) {
        gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MAG_FILTER, Gl::LINEAR as i32);
        gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MIN_FILTER, Gl::LINEAR as i32);
        gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_S, Gl::CLAMP_TO_EDGE as i32);
        gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_T, Gl::CLAMP_TO_EDGE as i32);
    }
}
