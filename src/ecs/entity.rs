//! Entity: lightweight handle for an object in the world.
//! No data stored here; components live in the World.

use std::num::NonZeroU32;

/// Opaque handle for an entity. Use `World::spawn_*` to create.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Entity(NonZeroU32);

impl Entity {
    #[inline]
    pub(crate) fn from_index(index: u32) -> Self {
        Self(NonZeroU32::new(index + 1).expect("entity index overflow"))
    }

    #[inline]
    pub(crate) fn to_index(self) -> u32 {
        self.0.get() - 1
    }
}
