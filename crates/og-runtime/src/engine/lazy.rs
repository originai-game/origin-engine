//! Engine records registered with a world only once a game uses them
//! (G2-1's og.PrefabLink, G2-4's og.NavFilter, ...), so a world that never does keeps the type
//! table, saves and hash it had before they existed.
//!
//! Their layouts are made with the engine's records at assembly
//! ([`crate::engine::creation`] skips registering them), and [`ensure`]
//! registers one, from that layout, when the game first names it outside
//! a system: an insert, an emit, a system or hook that declares it, a save
//! that holds it. The module that reads it is then granted it
//! ([`og_ecs::World::grant`]): `enable` registers the module's own Rust
//! type (the same layout) and widens its system's declarations.

use crate::engine::{Engine, Mount};
use crate::registry::{ATTACH, LAZY, NEEDS, TOGETHER};
use og_ecs::World;

/// One lazily registered record and what its module does when it is.
pub(crate) struct Lazy {
    pub name: &'static str,
    /// Grants the record to its module on `world` of the engine it is
    /// mounted on (a module with per-engine state, og.motion, adds its
    /// state block here).
    pub enable: fn(&mut World, Mount<'_>) -> og_core::Result<()>,
}

/// The lazy records lazy record `name`'s module needs registered first.
pub(crate) fn needs(name: &str) -> &'static [&'static str] {
    NEEDS
        .iter()
        .find(|(group, _)| group.contains(&name))
        .map_or(&[], |(_, n)| n)
}

/// Runs lazy record `name`'s attach ([`ATTACH`]), if it has one.
fn attach(name: &str, mount: Mount<'_>, world: &mut World) -> Result<(), String> {
    match ATTACH.iter().find(|(n, _)| *n == name) {
        Some((_, attach)) => attach(mount, world),
        None => Ok(()),
    }
}

/// Lazy record `name` and the ones registered with it.
pub(crate) fn with_partners(name: &'static str) -> Vec<&'static str> {
    TOGETHER
        .iter()
        .find(|g| g.contains(&name))
        .map_or_else(|| vec![name], |g| g.to_vec())
}

/// Whether engine record `name` is registered lazily.
pub(crate) fn is_lazy(name: &str) -> bool {
    LAZY.iter().any(|l| l.name == name)
}

/// Registers lazy record `name` with `world` if it is one and is not yet
/// (its layout's schema, then its module's grant). Anything else is left
/// to the caller's own lookup.
pub(crate) fn ensure(eng: &Engine, world: &mut World, name: &str) -> Result<(), String> {
    let Some(lazy) = LAZY.iter().find(|l| l.name == name) else {
        return ensure_deferred(eng, world, name);
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
        .register_schema(crate::engine::versioned(layout.schema()))
        .map_err(|e| e.reason().to_string())?;
    // A resource holds its default from the moment it is registered.
    if layout.kind == crate::host::codec::RecordKind::Resource && !world.has_resource_id(id) {
        let bytes = layout.encode(None, None)?;
        world
            .insert_resource_bytes(id, &bytes)
            .map_err(|e| e.reason().to_string())?;
    }
    {
        // The runtime's table of names by id mirrors the registry's order.
        let mut st = eng.state.borrow_mut();
        if st.names_by_id.len() == id.index() as usize {
            st.names_by_id.push(name.to_string());
        }
    }
    for need in needs(name) {
        ensure(eng, world, need)?;
    }
    let mount = Mount {
        id: eng.id,
        modules: &eng.modules,
    };
    (lazy.enable)(world, mount).map_err(|e| format!("{name}: {}", e.reason()))?;
    // og.WorldOrigin is on once named: the resource at its defaults.
    if name == crate::world::origin::RESOURCE && !world.has_resource_id(id) {
        let bytes = layout.encode(None, None)?;
        world
            .insert_resource_bytes(id, &bytes)
            .map_err(|e| e.reason().to_string())?;
    }
    attach(name, mount, world)?;
    for partner in with_partners(lazy.name) {
        if partner != lazy.name {
            ensure(eng, world, partner)?;
        }
    }
    Ok(())
}

/// Whether `name` is a record `eng`'s world registers when first named:
/// an engine record in [`LAZY`] or a game record only other games of the
/// process name (assembly's `deferred`).
pub(crate) fn is_deferred(eng: &Engine, name: &str) -> bool {
    is_lazy(name) || eng.state.borrow().deferred.contains(name)
}

/// Registers game record `name` if assembly deferred it (it was only other
/// games' then): its layout's schema, a resource at its default.
fn ensure_deferred(eng: &Engine, world: &mut World, name: &str) -> Result<(), String> {
    let layout = {
        let mut st = eng.state.borrow_mut();
        if !st.deferred.remove(name) {
            return Ok(());
        }
        st.layouts
            .get(name)
            .cloned()
            .ok_or_else(|| format!("{name} has no layout"))?
    };
    let id = world
        .register_schema(layout.schema())
        .map_err(|e| e.reason().to_string())?;
    if layout.kind == crate::host::codec::RecordKind::Resource && !world.has_resource_id(id) {
        let bytes = layout.encode(None, None)?;
        world
            .insert_resource_bytes(id, &bytes)
            .map_err(|e| e.reason().to_string())?;
    }
    let mut st = eng.state.borrow_mut();
    if st.names_by_id.len() == id.index() as usize {
        st.names_by_id.push(name.to_string());
    }
    if layout.kind == crate::host::codec::RecordKind::Resource
        && !st.resources.iter().any(|r| r == name)
    {
        st.resources.push(name.to_string());
    }
    Ok(())
}

/// Runs lazy record `name`'s module grant on a world that has registered
/// it (assembly, once the modules are mounted).
pub(crate) fn enable(world: &mut World, mount: Mount<'_>, name: &str) -> Result<(), String> {
    match LAZY.iter().find(|l| l.name == name) {
        Some(l) => {
            (l.enable)(world, mount).map_err(|e| format!("{name}: {}", e.reason()))?;
            attach(name, mount, world)?;
            Ok(())
        }
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
        if is_deferred(eng, &n) {
            let _ = ensure(eng, world, &n);
        }
    }
}

/// Whether a snapshot names a lazy record `world` has not registered: a
/// restore asked for inside a system waits for the tick's end, where
/// [`before_restore`] can register it.
pub(crate) fn snapshot_needs(eng: &Engine, snapshot: &[u8]) -> bool {
    let has = |n: &str| eng.state.borrow().names_by_id.iter().any(|have| have == n);
    og_ecs::snapshot_type_names(snapshot)
        .is_ok_and(|names| names.iter().any(|n| is_deferred(eng, n) && !has(n)))
}
