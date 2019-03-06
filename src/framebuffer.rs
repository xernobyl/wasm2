use web_sys::{WebGlFramebuffer, WebGl2RenderingContext};
use wasm_bindgen::JsValue;

type GL = WebGl2RenderingContext;


pub struct Framebuffer {
	width: i32,
	height: i32,
	framebuffer: WebGlFramebuffer,
}


impl Framebuffer {
	fn js_to_str(value: JsValue) -> String {
		value.as_string().unwrap_or_else(|| "???".to_string())
	}

	pub fn new(gl: &GL, width: u32, height: u32) -> Result<Self, String> {
		let width = width as i32;
		let height = height as i32;

		// HDR texture
		let texture_hdr = gl.create_texture().ok_or("Can't create texture.")?;
		gl.bind_texture(GL::TEXTURE_2D, Some(&texture_hdr));
		gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
			GL::TEXTURE_2D, 0, GL::RGB10_A2 as i32, width, height, 0, GL::RGBA, GL::UNSIGNED_INT_2_10_10_10_REV, None)
			.map_err(Self::js_to_str)?;
		//gl.tex_image_2d(GL::TEXTURE_2D, 0, gl.RGBA8, width, height, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
		//gl.tex_image_2d(GL::TEXTURE_2D, 0, gl.RGB16F, width, height, 0, gl.RGB, gl.HALF_FLOAT, null);
		gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32);
		gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32);
		gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32);
		gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32);

		// Depth texture
		let texture_depth = gl.create_texture().ok_or("Can't create texture.")?;
		gl.bind_texture(GL::TEXTURE_2D, Some(&texture_depth));
		//gl.texImage2D(gl.TEXTURE_2D, 0, gl.DEPTH32F_STENCIL8, width, height, 0, gl.DEPTH_STENCIL, gl.FLOAT_32_UNSIGNED_INT_24_8_REV, null);
		gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
			GL::TEXTURE_2D, 0, GL::DEPTH_COMPONENT32F as i32, width as i32, height as i32, 0, GL::DEPTH_COMPONENT, GL::FLOAT, None)
			.map_err(Self::js_to_str)?;
		gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32);
		gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32);
		gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32);
		gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32);

		// create framebuffer
		let framebuffer = gl.create_framebuffer().ok_or("Can't create framebuffer.")?;
		gl.bind_framebuffer(GL::FRAMEBUFFER, Some(&framebuffer));
		gl.framebuffer_texture_2d(GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, Some(&texture_hdr), 0);
		//gl.framebufferTexture2D(gl.FRAMEBUFFER, GL::DEPTH_STENCIL_ATTACHMENT, GL::TEXTURE_2D, texture_depth, 0);
		gl.framebuffer_texture_2d(GL::FRAMEBUFFER, GL::DEPTH_ATTACHMENT, GL::TEXTURE_2D, Some(&texture_depth), 0);

		gl.draw_buffers(&js_sys::Array::of1(&JsValue::from(GL::COLOR_ATTACHMENT0)));
		gl.clear(GL::DEPTH_BUFFER_BIT | GL::COLOR_BUFFER_BIT);

		match gl.check_framebuffer_status(GL::FRAMEBUFFER) {
			GL::FRAMEBUFFER_COMPLETE => Err("FRAMEBUFFER_COMPLETE"),
			GL::FRAMEBUFFER_UNSUPPORTED => Err("FRAMEBUFFER_UNSUPPORTED"),
			GL::FRAMEBUFFER_INCOMPLETE_ATTACHMENT => Err("FRAMEBUFFER_INCOMPLETE_ATTACHMENT"),
			GL::FRAMEBUFFER_INCOMPLETE_DIMENSIONS => Err("FRAMEBUFFER_INCOMPLETE_DIMENSIONS"),
			GL::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => Err("FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT"),
			_ => Ok(())
		}?;

		gl.bind_texture(GL::TEXTURE_2D, None);
		gl.bind_framebuffer(GL::FRAMEBUFFER, None);

		Ok(Self {
			width,
			height,
			framebuffer,
		})
	}


	fn bind(&self, gl: &GL) {
		gl.bind_framebuffer(GL::FRAMEBUFFER, Some(&self.framebuffer));
		gl.draw_buffers(&js_sys::Array::of1(&JsValue::from(GL::COLOR_ATTACHMENT0)));	// needed?
		gl.viewport(0, 0, self.width as i32, self.height as i32);
	}
}
