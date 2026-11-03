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
use og_core::Reflect;
use og_ecs::World;

/// One lazily registered record and what its module does when it is.
pub(crate) struct Lazy {
    pub name: &'static str,
    /// Grants the record to its module on `world` of the engine whose id
    /// it is given (a module with per-engine state, og.motion, adds its
    /// state block here).
    pub enable: fn(&mut World, u32) -> og_core::Result<()>,
}

/// Every lazily registered engine record.
pub(crate) const LAZY: &[Lazy] = &[
    // G2-1: the runtime itself writes and reads it; no module to grant.
    Lazy {
        name: crate::prefab::LINK,
        enable: |_, _| Ok(()),
    },
    // G2-4: og.nav reads it.
    Lazy {
        name: "og.NavFilter",
        enable: |w, _| og_nav::enable_filters(w),
    },
    // G2-4: og.nav reads it (crowd avoidance).
    Lazy {
        name: "og.Crowd",
        enable: |w, _| og_nav::enable_crowds(w),
    },
    // G2-4: og.ai sends them once a behaviour with a game action is loaded.
    Lazy {
        name: "og.AiActionStarted",
        enable: |w, _| og_ai::enable_action_started(w),
    },
    Lazy {
        name: "og.AiActionStopped",
        enable: |w, _| og_ai::enable_action_stopped(w),
    },
    // G2-4: og.nav sends it once a navmesh has an off-mesh link.
    Lazy {
        name: "og.NavLinkReached",
        enable: |w, _| og_nav::enable_links(w),
    },
    // G2-3: og.physics sends them and reads the hooks once a game names
    // them; a world that never does keeps its type table and hash.
    Lazy {
        name: "og.BodySlept",
        enable: |w, _| og_physics::enable(w, "og.BodySlept"),
    },
    Lazy {
        name: "og.BodyWoke",
        enable: |w, _| og_physics::enable(w, "og.BodyWoke"),
    },
    Lazy {
        name: "og.JointBroken",
        enable: |w, _| og_physics::enable(w, "og.JointBroken"),
    },
    Lazy {
        name: "og.ContactHook",
        enable: |w, _| og_physics::enable(w, "og.ContactHook"),
    },
    Lazy {
        name: "og.VehicleWheel",
        enable: |w, _| og_physics::enable(w, "og.VehicleWheel"),
    },
    // G2-7: og.net sends them once a world runs lockstep in a room.
    Lazy {
        name: "og.SeatDropped",
        enable: |w, _| w.grant(crate::net::NET_SYSTEM, &["og.SeatDropped"]),
    },
    Lazy {
        name: "og.SeatRejoined",
        enable: |w, _| w.grant(crate::net::NET_SYSTEM, &["og.SeatRejoined"]),
    },
    Lazy {
        name: "og.PeerStalled",
        enable: |w, _| w.grant(crate::net::NET_SYSTEM, &["og.PeerStalled"]),
    },
    // G3-6: og.platform sends it once a game names it; the grants are the
    // runtime's own (og.platform.grant writes them).
    Lazy {
        name: crate::platform::NOTICE,
        enable: |w, _| w.grant(crate::platform::PLATFORM_SYSTEM, &[crate::platform::NOTICE]),
    },
    Lazy {
        name: crate::platform::GRANTS,
        enable: |w, _| crate::platform::enable_grants(w),
    },
    Lazy {
        name: "og.SoftBody",
        enable: |w, _| og_physics::enable(w, "og.SoftBody"),
    },
    // R9: GPU crowds, read by extraction alone (no system is granted it).
    Lazy {
        name: "og.CrowdAnimation",
        enable: |_, _| Ok(()),
    },
    // R9: og.physics steps it; the garment system poses it.
    Lazy {
        name: "og.Garment",
        enable: |w, _| {
            og_physics::enable(w, "og.Garment")?;
            w.grant(crate::garment::SYSTEM, &["og.Garment"])
        },
    },
    // og.motion: its system and state block (goals) once a game names it.
    Lazy {
        name: crate::motion::AGENT,
        enable: crate::motion::enable,
    },
    // G2-6: time groups, read by the modules that scale an entity's dt.
    // Naming either registers both (TOGETHER).
    Lazy {
        name: "og.TimeGroup",
        enable: |w, _| enable_time_groups(w, og_core::TimeGroup::NAME),
    },
    Lazy {
        name: "og.TimeGroups",
        enable: |w, _| enable_time_groups(w, og_core::TimeGroups::NAME),
    },
    // G2-8: og.story's resource and events, registered at the first
    // og.story.load (together, TOGETHER); og.story sends the events and
    // writes the resource.
    Lazy {
        name: crate::story::STORY,
        enable: |w, _| w.grant(crate::story::SYSTEM, &[crate::story::STORY]),
    },
    Lazy {
        name: crate::story::SHOWN,
        enable: |w, _| w.grant(crate::story::SYSTEM, &[crate::story::SHOWN]),
    },
    Lazy {
        name: crate::story::ENDED,
        enable: |w, _| w.grant(crate::story::SYSTEM, &[crate::story::ENDED]),
    },
    Lazy {
        name: crate::story::CHECK,
        enable: |w, _| w.grant(crate::story::SYSTEM, &[crate::story::CHECK]),
    },
    Lazy {
        name: crate::story::QUEST_UPDATED,
        enable: |w, _| w.grant(crate::story::SYSTEM, &[crate::story::QUEST_UPDATED]),
    },
    Lazy {
        name: crate::story::STORY_EVENT,
        enable: |w, _| w.grant(crate::story::SYSTEM, &[crate::story::STORY_EVENT]),
    },
    Lazy {
        name: crate::story::BARK_SAID,
        enable: |w, _| w.grant(crate::story::SYSTEM, &[crate::story::BARK_SAID]),
    },
    // G3-3: world cells, read and written by the runtime's before-tick
    // hook (stream::cells); no module to grant. Naming any of the cell's
    // records registers them together; an anchor alone is its own.
    Lazy {
        name: crate::stream::cells::CELL,
        enable: |_, _| Ok(()),
    },
    Lazy {
        name: crate::stream::cells::MEMBER,
        enable: |_, _| Ok(()),
    },
    Lazy {
        name: crate::stream::cells::EVENT,
        enable: |_, _| Ok(()),
    },
    Lazy {
        name: crate::stream::cells::ANCHOR,
        enable: |_, _| Ok(()),
    },
    // R11: 2D lighting, read by extraction alone (no system is granted
    // them): a world without 2D lights keeps its type table, hash and
    // frames whatever these records become.
    Lazy {
        name: "og.Lighting2D",
        enable: |_, _| Ok(()),
    },
    Lazy {
        name: "og.Light2D",
        enable: |_, _| Ok(()),
    },
    Lazy {
        name: "og.Occluder2D",
        enable: |_, _| Ok(()),
    },
    Lazy {
        name: "og.LightMask2D",
        enable: |_, _| Ok(()),
    },
    // R3: screen-space GI's settings, read by extraction alone: a world
    // that never names them keeps its type table and hash.
    Lazy {
        name: crate::fx::GLOBAL_ILLUMINATION,
        enable: |_, _| Ok(()),
    },
    // R12: the runtime's before-tick hook reads and writes them
    // (crate::origin); no module to grant. Naming either registers both.
    Lazy {
        name: crate::origin::RESOURCE,
        enable: |_, _| Ok(()),
    },
    Lazy {
        name: crate::origin::EVENT,
        enable: |_, _| Ok(()),
    },
    // G3-2: og.video plays it; a world without footage keeps its hash
    // whatever the player's layout becomes.
    Lazy {
        name: crate::video::PLAYER,
        enable: |w, _| w.grant(crate::video::VIDEO_SYSTEM, &[crate::video::PLAYER]),
    },
    // R7: og.particles simulates the systems whose emitters send events
    // and sends them, once a game names the event.
    Lazy {
        name: crate::particle_events::EVENT,
        enable: crate::particle_events::enable,
    },
    // R10: virtual cameras; og.camera's brain writes them (and
    // og.PostEffects, for a shot's focus). Naming one registers all three.
    Lazy {
        name: crate::feel::vcam::VCAM,
        enable: |w, _| crate::feel::vcam::enable(w, crate::feel::vcam::VCAM),
    },
    Lazy {
        name: crate::feel::vcam::BRAIN,
        enable: |w, _| crate::feel::vcam::enable(w, crate::feel::vcam::BRAIN),
    },
    Lazy {
        name: crate::feel::vcam::CUT,
        enable: |w, _| crate::feel::vcam::enable(w, crate::feel::vcam::CUT),
    },
    // R4: the physical sky and the clouds, read by extraction alone (no
    // system is granted them): a world without them keeps its type table
    // and hash.
    Lazy {
        name: "og.PhysicalSky",
        enable: |_, _| Ok(()),
    },
    Lazy {
        name: "og.Clouds",
        enable: |_, _| Ok(()),
    },
    // R5: the 3D water surface, read by extraction (and og.physics.waterHeight's
    // sea) alone: a world without water keeps its type table and hash as the
    // surface grows (the sea, the flow map).
    Lazy {
        name: "og.Water",
        enable: |_, _| Ok(()),
    },
    // R4: the weather's simulated state and its strikes; og.weather writes
    // and sends them. Naming either registers both.
    Lazy {
        name: crate::weather::STATE,
        enable: |w, _| crate::weather::enable(w, crate::weather::STATE),
    },
    Lazy {
        name: crate::weather::STRIKE,
        enable: |w, _| crate::weather::enable(w, crate::weather::STRIKE),
    },
];

/// Lazy records registered together: naming one registers the rest.
const TOGETHER: &[&[&str]] = &[
    &["og.TimeGroup", "og.TimeGroups"],
    &[crate::origin::RESOURCE, crate::origin::EVENT],
    &crate::story::RECORDS,
    &crate::feel::vcam::RECORDS,
    &crate::weather::RECORDS,
    &[
        crate::stream::cells::CELL,
        crate::stream::cells::MEMBER,
        crate::stream::cells::EVENT,
    ],
];

/// Lazy record `name` and the ones registered with it.
pub(crate) fn with_partners(name: &'static str) -> Vec<&'static str> {
    TOGETHER
        .iter()
        .find(|g| g.contains(&name))
        .map_or_else(|| vec![name], |g| g.to_vec())
}

/// Registers og.TimeGroup or og.TimeGroups (the resource at its default,
/// every group at 1). The modules that scale an entity's dt read them as
/// an engine service, as they read og.TimeScale, so no system is granted.
fn enable_time_groups(world: &mut World, name: &str) -> og_core::Result<()> {
    if name == og_core::TimeGroups::NAME {
        world.register::<og_core::TimeGroups>()?;
        if !world.has_resource::<og_core::TimeGroups>() {
            world.insert_resource(og_core::TimeGroups::default())?;
        }
    } else {
        world.register::<og_core::TimeGroup>()?;
    }
    Ok(())
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
    if layout.kind == crate::codec::RecordKind::Resource && !world.has_resource_id(id) {
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
    (lazy.enable)(world, eng.id).map_err(|e| format!("{name}: {}", e.reason()))?;
    // og.WorldOrigin is on once named: the resource at its defaults.
    if name == crate::origin::RESOURCE && !world.has_resource_id(id) {
        let bytes = layout.encode(None, None)?;
        world
            .insert_resource_bytes(id, &bytes)
            .map_err(|e| e.reason().to_string())?;
    }
    if name == crate::story::STORY {
        crate::story::attach(eng.id, world)?;
    }
    if name == crate::stream::cells::CELL {
        crate::stream::cells::attach(eng.id, world)?;
    }
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
    if layout.kind == crate::codec::RecordKind::Resource && !world.has_resource_id(id) {
        let bytes = layout.encode(None, None)?;
        world
            .insert_resource_bytes(id, &bytes)
            .map_err(|e| e.reason().to_string())?;
    }
    let mut st = eng.state.borrow_mut();
    if st.names_by_id.len() == id.index() as usize {
        st.names_by_id.push(name.to_string());
    }
    if layout.kind == crate::codec::RecordKind::Resource && !st.resources.iter().any(|r| r == name)
    {
        st.resources.push(name.to_string());
    }
    Ok(())
}

/// Runs lazy record `name`'s module grant on a world that has registered
/// it (assembly, once the modules are mounted).
pub(crate) fn enable(world: &mut World, engine: u32, name: &str) -> Result<(), String> {
    match LAZY.iter().find(|l| l.name == name) {
        Some(l) => {
            (l.enable)(world, engine).map_err(|e| format!("{name}: {}", e.reason()))?;
            if name == crate::story::STORY {
                crate::story::attach(engine, world)?;
            }
            if name == crate::stream::cells::CELL {
                crate::stream::cells::attach(engine, world)?;
            }
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
