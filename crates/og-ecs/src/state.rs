//! State blocks: world state a module keeps outside archetypes and
//! resources, because it is neither fixed-layout nor per entity (a physics
//! engine's contact manifolds, solver warm starts and broad-phase tree, say).
//! A block is part of the world: [`World::snapshot`](crate::World::snapshot)
//! saves it, [`World::world_hash`](crate::World::world_hash) covers it, and a
//! restore brings it back. Like systems, blocks are code: restore into a
//! world that registered the same blocks.

use og_core::Result;
use std::any::Any;

/// One module's state, saved and restored with the world.
///
/// `save` must be a function of the state: equal state, equal bytes (the
/// hash compares them). A restore first calls `decode` on every block and
/// applies nothing unless the whole snapshot is valid, then `apply` on each;
/// `apply(None)` means the snapshot had no bytes for this block (an older
/// format, or a world that did not have it), and the block starts empty.
pub trait StateBlock: Send {
    /// Appends the block's state.
    fn save(&self, out: &mut Vec<u8>);
    /// Decodes saved bytes without changing the block.
    fn decode(&self, bytes: &[u8]) -> Result<Box<dyn Any + Send>>;
    /// Takes the decoded state (`Some`, from [`Self::decode`]) or resets.
    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>);
}

pub(crate) struct Block {
    pub name: String,
    pub block: Box<dyn StateBlock>,
}
