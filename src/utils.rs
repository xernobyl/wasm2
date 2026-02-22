//! Shared utilities: logging, buffer views, Halton sequence.

#![allow(dead_code)]

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

/// Reinterprets a slice of `T` as bytes. Use only when `T` is `repr(C)` and the
/// buffer is used as raw bytes (e.g. for WebGL buffer upload).
#[inline]
pub fn as_u8_slice<T>(v: &[T]) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(
            v.as_ptr().cast::<u8>(),
            v.len().saturating_mul(std::mem::size_of::<T>()),
        )
    }
}

/// Reinterprets a slice of `T` as f32s. Caller must ensure alignment and that
/// `size_mult` matches the logical f32 count per element (e.g. 16 for Mat4).
#[inline]
pub fn as_f32_slice<T>(v: &[T], size_mult: usize) -> &[f32] {
    let len = v.len().saturating_mul(size_mult);
    unsafe { std::slice::from_raw_parts(v.as_ptr().cast::<f32>(), len) }
}

/// Halton sequence 2D; values in [-0.5, 0.5]. Returns flat array [x0,y0, x1,y1, ...].
pub fn halton_sequence_2d(count: usize, base1: u32, base2: u32) -> Vec<f32> {
    fn halton(index: u32, base: u32) -> f32 {
        let mut result = 0.0f32;
        let mut f = 1.0f32 / (base as f32);
        let mut i = index;
        while i > 0 {
            result += f * ((i % base) as f32);
            i /= base;
            f /= base as f32;
        }
        result - 0.5
    }
    let mut points = Vec::with_capacity(count * 2);
    for i in 1..=count {
        let i = i as u32;
        points.push(halton(i, base1));
        points.push(halton(i, base2));
    }
    points
}
