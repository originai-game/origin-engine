//! Engine records registered with a world only once a game uses them
//! (G2-1's og.PrefabLink, G2-4's og.NavFilter, ...), so a world that never does keeps the type
//! table, saves and hash it had before they existed.
//!
//! Their layouts are made with the engine's records at assembly
//! ([`super::engine::creation`] skips registering them), and [`ensure`]
//! registers one, from that layout, when the game first names it outside
//! a system: an insert, an emit, a system or hook that declares it, a save
//! that holds it. The module that reads it is then granted it
//! ([`og_ecs::World::grant`]): `enable` registers the module's own Rust
//! type (the same layout) and widens its system's declarations.

use crate::engine::Engine;
use og_ecs::World;

/// One lazily registered record and what its module does when it is.
pub(crate) struct Lazy {
    pub name: &'static str,
    pub enable: fn(&mut World) -> og_core::Result<()>,
}

/// Every lazily registered engine record.
pub(crate) const LAZY: &[Lazy] = &[
    // G2-1: the runtime itself writes and reads it; no module to grant.
    Lazy {
        name: crate::prefab::LINK,
        enable: |_| Ok(()),
    },
    // G2-4: og.nav reads it.
    Lazy {
        name: "og.NavFilter",
        enable: og_nav::enable_filters,
    },
    // G2-4: og.nav reads it (crowd avoidance).
    Lazy {
        name: "og.Crowd",
        enable: og_nav::enable_crowds,
    },
    // G2-4: og.nav sends it once a navmesh has an off-mesh link.
    Lazy {
        name: "og.NavLinkReached",
        enable: og_nav::enable_links,
    },
];

/// Whether engine record `name` is registered lazily.
pub(crate) fn is_lazy(name: &str) -> bool {
    LAZY.iter().any(|l| l.name == name)
}

/// Registers lazy record `name` with `world` if it is one and is not yet
/// (its layout's schema, then its module's grant). Anything else is left
/// to the caller's own lookup.
pub(crate) fn ensure(eng: &Engine, world: &mut World, name: &str) -> Result<(), String> {
    let Some(lazy) = LAZY.iter().find(|l| l.name == name) else {
        return Ok(());
    };
    if world.id_by_name(name).is_ok() {
        return Ok(());
    }
    let layout = eng
        .state
        .borrow()
        .layouts
        .get(name)
        .cloned()
        .ok_or_else(|| format!("{name} has no layout"))?;
    let id = world
        .register_schema(layout.schema())
        .map_err(|e| e.reason().to_string())?;
    {
        // The runtime's table of names by id mirrors the registry's order.
        let mut st = eng.state.borrow_mut();
        if st.names_by_id.len() == id.index() as usize {
            st.names_by_id.push(name.to_string());
        }
    }
    (lazy.enable)(world).map_err(|e| format!("{name}: {}", e.reason()))
}

/// Runs lazy record `name`'s module grant on a world that has registered
/// it (assembly, once the modules are mounted).
pub(crate) fn enable(world: &mut World, name: &str) -> Result<(), String> {
    match LAZY.iter().find(|l| l.name == name) {
        Some(l) => (l.enable)(world).map_err(|e| format!("{name}: {}", e.reason())),
        None => Ok(()),
    }
}

/// Before a restore: registers every lazy record the snapshot's type table
/// names and the world lacks (a save of a world that used one, restored
/// in a session that has not yet).
pub(crate) fn before_restore(eng: &Engine, world: &mut World, snapshot: &[u8]) {
    let Ok(names) = og_ecs::snapshot_type_names(snapshot) else {
        return;
    };
    for n in names {
        if is_lazy(&n) {
            let _ = ensure(eng, world, &n);
        }
    }
}

/// Whether a snapshot names a lazy record `world` has not registered: a
/// restore asked for inside a system waits for the tick's end, where
/// [`before_restore`] can register it.
pub(crate) fn snapshot_needs(world_has: impl Fn(&str) -> bool, snapshot: &[u8]) -> bool {
    og_ecs::snapshot_type_names(snapshot)
        .is_ok_and(|names| names.iter().any(|n| is_lazy(n) && !world_has(n)))
}
