/*
Some of the some called "utilities"
*/

#![macro_use]
#![allow(dead_code)]

type Gl = web_sys::WebGl2RenderingContext;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
  ( $( $t:tt )* ) => {
    web_sys::console::log_1(&format!( $( $t )* ).into());
  }
}

#[allow(unused_macros)]
macro_rules! log_error {
  ( $( $t:tt )* ) => {
    web_sys::console::error_1(&format!( $( $t )* ).into());
  }
}

pub fn as_u8_slice<T>(v: &[T]) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(v.as_ptr() as *const u8, v.len() * std::mem::size_of::<T>())
    }
}

pub fn as_f32_slice<T>(v: &[T], size_mult: usize) -> &[f32] {
    unsafe { std::slice::from_raw_parts(v.as_ptr() as *const f32, v.len() * size_mult) }
}

pub fn fullscreen_quad(gl: &Gl) {
    gl.bind_vertex_array(None);
    gl.draw_arrays(Gl::TRIANGLES, 0, 3);
}
