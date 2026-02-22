//! World: entity storage and component data.
//! Uses a single archetype for "moving half-cubes" (BasePosition + OscillateMotion + HalfCube).
//! Extensible by adding more archetypes or switching to a generic ECS later.

use glam::Vec3;

use super::components::{BasePosition, HalfCube, OscillateMotion};
use super::entity::Entity;

/// Component storage for entities that have BasePosition + OscillateMotion + HalfCube.
/// SoA for cache-friendly iteration.
#[derive(Default)]
pub struct MovingCubeArchetype {
    pub base_positions: Vec<Vec3>,
    pub motion_axes: Vec<Vec3>,
    pub motion_phases: Vec<f32>,
    pub motion_amplitudes: Vec<f32>,
    pub motion_speeds: Vec<f32>,
    pub scales: Vec<f32>,
}

impl MovingCubeArchetype {
    #[inline]
    pub fn len(&self) -> usize {
        self.base_positions.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.base_positions.is_empty()
    }

    /// Spawn one moving half-cube entity. Returns its [Entity] id.
    pub fn spawn(
        &mut self,
        base_position: BasePosition,
        motion: OscillateMotion,
        half_cube: HalfCube,
    ) -> Entity {
        let index = self.base_positions.len() as u32;
        self.base_positions.push(base_position.0);
        self.motion_axes.push(motion.axis);
        self.motion_phases.push(motion.phase);
        self.motion_amplitudes.push(motion.amplitude);
        self.motion_speeds.push(motion.speed);
        self.scales.push(half_cube.scale);
        Entity::from_index(index)
    }

    /// Iterate over (base_position, motion_axis, motion_phase, amplitude, speed, scale) for each entity.
    pub fn iter(
        &self,
    ) -> impl ExactSizeIterator<Item = (Vec3, Vec3, f32, f32, f32, f32)> + '_ {
        self.base_positions
            .iter()
            .zip(self.motion_axes.iter())
            .zip(self.motion_phases.iter())
            .zip(self.motion_amplitudes.iter())
            .zip(self.motion_speeds.iter())
            .zip(self.scales.iter())
            .map(|(((((bp, ax), ph), amp), sp), sc)| (*bp, *ax, *ph, *amp, *sp, *sc))
    }
}

/// World holds all component storages. Currently one archetype; add more as needed.
pub struct World {
    pub moving_cubes: MovingCubeArchetype,
}

impl Default for World {
    fn default() -> Self {
        Self {
            moving_cubes: MovingCubeArchetype::default(),
        }
    }
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    /// Spawn an entity with BasePosition + OscillateMotion + HalfCube (moving half-cube).
    pub fn spawn_moving_half_cube(
        &mut self,
        base_position: BasePosition,
        motion: OscillateMotion,
        half_cube: HalfCube,
    ) -> Entity {
        self.moving_cubes.spawn(base_position, motion, half_cube)
    }

    /// Number of moving half-cube entities (for instanced draw count).
    pub fn moving_half_cube_count(&self) -> usize {
        self.moving_cubes.len()
    }
}
