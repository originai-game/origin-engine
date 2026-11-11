//! What each runtime module keeps for one engine beside its world: the
//! state its bindings share with its systems and state blocks, its
//! registries and its caches. It is made before the engine's world is
//! assembled (assembly mounts the modules' systems with it) and lives as
//! long as the engine, across a reload. What of it is world state is
//! saved by the module's state block, as before; the rest is the engine's,
//! never the process's.

use crate::{
    atmosphere, destruction, destruction2d, e6, i18n, invariants, modules, motion, net, origin,
    particle_events, platform, story, stream, terrain, timeline, vfx, video,
};

/// Each module's state on one engine (an optional module left out of the
/// build keeps an empty one).
#[derive(Default)]
#[cfg_attr(
    not(all(
        feature = "destruction",
        feature = "motion",
        feature = "video",
        feature = "story",
        not(feature = "profile2d")
    )),
    allow(dead_code)
)]
pub(crate) struct Modules {
    pub atmosphere: atmosphere::ModuleState,
    pub destruction: destruction::ModuleState,
    pub destruction2d: destruction2d::ModuleState,
    pub motion: motion::ModuleState,
    pub particles: particle_events::ModuleState,
    pub vfx: vfx::ModuleState,
    pub timeline: timeline::ModuleState,
    pub terrain: terrain::ModuleState,
    pub video: video::ModuleState,
    pub cells: stream::cells::ModuleState,
    pub stream: stream::ModuleState,
    pub e6: e6::ModuleState,
    pub story: story::ModuleState,
    pub platform: platform::ModuleState,
    pub net: net::ModuleState,
    pub i18n: i18n::ModuleState,
    pub docs: modules::docs::ModuleState,
    pub ui: modules::ModuleState,
    pub invariants: invariants::ModuleState,
    pub origin: origin::ModuleState,
}

/// The engine a world is assembled for, as its modules mount on it: its
/// id (what its systems name it by when they run) and its modules' state.
#[derive(Clone, Copy)]
pub(crate) struct Mount<'a> {
    pub id: u32,
    pub modules: &'a Modules,
}
