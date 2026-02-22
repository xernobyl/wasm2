//! Component types: data only, no behavior.
//! Systems read/write these via the World.

use glam::Vec3;

/// Base (rest) position in world space. Used with [OscillateMotion] to derive current position.
#[derive(Clone, Debug)]
pub struct BasePosition(pub Vec3);

/// Back-and-forth motion along an axis. Current offset = axis * (amplitude * sin(time * speed + phase)).
#[derive(Clone, Debug)]
pub struct OscillateMotion {
    pub axis: Vec3,
    pub phase: f32,
    pub amplitude: f32,
    pub speed: f32,
}

/// Renders as an instanced half-cube with the given scale.
#[derive(Clone, Copy, Debug)]
pub struct HalfCube {
    pub scale: f32,
}
