//! What each runtime module keeps for one engine beside its world: the
//! state its bindings share with its systems and state blocks, its
//! registries and its caches. It is made before the engine's world is
//! assembled (assembly mounts the modules' systems with it) and lives as
//! long as the engine, across a reload. What of it is world state is
//! saved by the module's state block, as before; the rest is the engine's,
//! never the process's.

use crate::registry::Modules;

/// The engine a world is assembled for, as its modules mount on it: its
/// id (what its systems name it by when they run) and its modules' state.
#[derive(Clone, Copy)]
pub(crate) struct Mount<'a> {
    pub id: u32,
    pub modules: &'a Modules,
}
