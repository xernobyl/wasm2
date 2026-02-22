//! ECS-style layer: entities, components, world, and systems.
//!
//! Structured so that:
//! - **Entities** are lightweight IDs.
//! - **Components** are data only (no logic).
//! - **World** holds component storage (SoA or archetype-style).
//! - **Systems** are functions that query the world and read/write resources.
//!
//! This keeps the codebase ready to swap in a full ECS crate (e.g. hecs) later
//! or to add more archetypes and systems without changing the app loop.

pub mod components;
mod entity;
mod resources;
pub mod systems;
mod world;

pub use components::{BasePosition, HalfCube, OscillateMotion};
pub use entity::Entity;
pub use resources::FrameResources;
pub use systems::half_cube_render_system;
pub use world::World;
