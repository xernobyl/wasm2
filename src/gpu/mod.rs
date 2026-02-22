//! WebGPU backend (wgpu). Device, queue, surface, pipelines, and targets.

mod context;
mod targets;
mod warehouse;

pub use context::GpuContext;
pub use context::init_gpu;
pub use targets::GbufferSet;
pub use warehouse::{WarehouseUniforms, FULLSCREEN_TRIANGLE};
