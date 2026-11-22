//! The runtime's modules: every place one takes part in an engine, each
//! a list in the order the engine runs it. The engine's core
//! (`engine/creation.rs`, `engine/lazy.rs`, `engine/module_state.rs`,
//! `lib.rs`) walks these lists and names no module, so a new module is
//! its own files, its `mod` line in `bind.rs` and its lines here.
//!
//! The lists are separate because their orders are: the order lazy
//! records are registered in is a world's type table (so its hash), the
//! mounts' order is the systems' schedule and the before-tick hooks' is
//! what each reads of the others. Each is written out, never sorted.

use crate::engine::lazy::Lazy;
use crate::engine::{Engine, Mount};
use crate::host::codec::Layout;
use crate::{
    bind::agents, bind::destruction, bind::destruction2d, bind::flow, bind::forces, bind::i18n,
    bind::modules, bind::motion, bind::net, bind::particles, bind::shared, bind::sight,
    bind::story, bind::terrain, bind::timeline, bind::vfx, bind::video, bind::wind,
    engine::invariants, host::platform, present::atmosphere, present::loading, world::origin,
    world::scenes, world::stream,
};
use og_core::Reflect;
use og_ecs::World;
use og_physics::PhysicsSystem;
use serde_json::Value;
use std::collections::BTreeMap;

/// Each module's state on one engine.
#[derive(Default)]
pub(crate) struct Modules {
    pub atmosphere: atmosphere::ModuleState,
    pub destruction: destruction::ModuleState,
    pub destruction2d: destruction2d::ModuleState,
    pub motion: motion::ModuleState,
    pub particles: particles::ModuleState,
    pub vfx: vfx::ModuleState,
    pub timeline: timeline::ModuleState,
    pub terrain: terrain::ModuleState,
    pub video: video::ModuleState,
    pub cells: stream::cells::ModuleState,
    pub stream: stream::ModuleState,
    pub agents: agents::ModuleState,
    pub story: story::ModuleState,
    pub flow: flow::ModuleState,
    pub platform: platform::ModuleState,
    pub net: net::ModuleState,
    pub i18n: i18n::ModuleState,
    pub docs: modules::docs::ModuleState,
    pub ui: modules::ModuleState,
    pub invariants: invariants::ModuleState,
    pub origin: origin::ModuleState,
    pub air: wind::ModuleState,
    pub forces: forces::ModuleState,
    pub sight: sight::ModuleState,
    pub shared: shared::ModuleState,
    pub scenes: scenes::ModuleState,
    pub loading: loading::ModuleState,
}

/// Every lazily registered engine record.
pub(crate) const LAZY: &[Lazy] = &[
    // G2-1: the runtime itself writes and reads it; no module to grant.
    Lazy {
        name: crate::world::prefab::LINK,
        enable: |_, _| Ok(()),
    },
    // G2-4: og.nav reads it.
    Lazy {
        name: "og.NavFilter",
        enable: |w, _| crate::bind::nav::enable_filters(w),
    },
    // G2-4: og.nav reads it (crowd avoidance).
    Lazy {
        name: "og.Crowd",
        enable: |w, _| crate::bind::nav::enable_crowds(w),
    },
    // G2-4: og.ai sends them once a behaviour with a game action is loaded.
    Lazy {
        name: "og.AiActionStarted",
        enable: |w, _| crate::bind::ai::enable_action_started(w),
    },
    Lazy {
        name: "og.AiActionStopped",
        enable: |w, _| crate::bind::ai::enable_action_stopped(w),
    },
    // G2-4: og.nav sends it once a navmesh has an off-mesh link.
    Lazy {
        name: "og.NavLinkReached",
        enable: |w, _| crate::bind::nav::enable_links(w),
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
    // R5: og.physics floats the bodies with it on og.Water and
    // og.WaterSurface once a game names it.
    Lazy {
        name: "og.Buoyancy",
        enable: |w, _| og_physics::enable(w, "og.Buoyancy"),
    },
    // G2-7: og.net sends them once a world runs lockstep in a room.
    Lazy {
        name: "og.SeatDropped",
        enable: |w, _| w.grant(crate::bind::net::NET_SYSTEM, &["og.SeatDropped"]),
    },
    Lazy {
        name: "og.SeatRejoined",
        enable: |w, _| w.grant(crate::bind::net::NET_SYSTEM, &["og.SeatRejoined"]),
    },
    Lazy {
        name: "og.PeerStalled",
        enable: |w, _| w.grant(crate::bind::net::NET_SYSTEM, &["og.PeerStalled"]),
    },
    // G3-6: og.platform sends it once a game names it; the grants are the
    // runtime's own (og.platform.grant writes them).
    Lazy {
        name: crate::host::platform::NOTICE,
        enable: |w, _| {
            w.grant(
                crate::host::platform::PLATFORM_SYSTEM,
                &[crate::host::platform::NOTICE],
            )
        },
    },
    Lazy {
        name: crate::host::platform::GRANTS,
        enable: |w, _| crate::host::platform::enable_grants(w),
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
            w.grant(crate::bind::garment::SYSTEM, &["og.Garment"])
        },
    },
    // og.motion: its system and state block (goals) once a game names it.
    Lazy {
        name: crate::bind::motion::AGENT,
        enable: crate::bind::motion::enable,
    },
    // og.destruction: its system and state block once a game names any of
    // its records (TOGETHER), the last one registered enabling it.
    Lazy {
        name: crate::bind::destruction::DESTRUCTIBLE,
        enable: crate::bind::destruction::enable,
    },
    Lazy {
        name: crate::bind::destruction::CHUNK,
        enable: crate::bind::destruction::enable,
    },
    Lazy {
        name: crate::bind::destruction::FRACTURED,
        enable: crate::bind::destruction::enable,
    },
    Lazy {
        name: crate::bind::destruction::RESOURCE,
        enable: crate::bind::destruction::enable,
    },
    // og.destruction2d: its system and state block once a game names any
    // of og.PixelTerrain, og.PixelPiece or og.PixelDetached.
    Lazy {
        name: crate::bind::destruction2d::TERRAIN,
        enable: crate::bind::destruction2d::enable,
    },
    Lazy {
        name: crate::bind::destruction2d::PIECE,
        enable: crate::bind::destruction2d::enable,
    },
    Lazy {
        name: crate::bind::destruction2d::DETACHED,
        enable: crate::bind::destruction2d::enable,
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
        name: crate::bind::story::STORY,
        enable: |w, _| w.grant(crate::bind::story::SYSTEM, &[crate::bind::story::STORY]),
    },
    Lazy {
        name: crate::bind::story::SHOWN,
        enable: |w, _| w.grant(crate::bind::story::SYSTEM, &[crate::bind::story::SHOWN]),
    },
    Lazy {
        name: crate::bind::story::ENDED,
        enable: |w, _| w.grant(crate::bind::story::SYSTEM, &[crate::bind::story::ENDED]),
    },
    Lazy {
        name: crate::bind::story::CHECK,
        enable: |w, _| w.grant(crate::bind::story::SYSTEM, &[crate::bind::story::CHECK]),
    },
    Lazy {
        name: crate::bind::story::QUEST_UPDATED,
        enable: |w, _| {
            w.grant(
                crate::bind::story::SYSTEM,
                &[crate::bind::story::QUEST_UPDATED],
            )
        },
    },
    Lazy {
        name: crate::bind::story::STORY_EVENT,
        enable: |w, _| {
            w.grant(
                crate::bind::story::SYSTEM,
                &[crate::bind::story::STORY_EVENT],
            )
        },
    },
    Lazy {
        name: crate::bind::story::BARK_SAID,
        enable: |w, _| w.grant(crate::bind::story::SYSTEM, &[crate::bind::story::BARK_SAID]),
    },
    // og.flow: its system once a game loads a flow (og.flow.load registers
    // its records together, TOGETHER).
    Lazy {
        name: crate::bind::flow::RECORDS[0],
        enable: crate::bind::flow::enable,
    },
    Lazy {
        name: crate::bind::flow::RECORDS[1],
        enable: crate::bind::flow::enable,
    },
    Lazy {
        name: crate::bind::flow::RECORDS[2],
        enable: crate::bind::flow::enable,
    },
    Lazy {
        name: crate::bind::flow::RECORDS[3],
        enable: crate::bind::flow::enable,
    },
    Lazy {
        name: crate::bind::flow::RECORDS[4],
        enable: crate::bind::flow::enable,
    },
    Lazy {
        name: crate::bind::flow::RECORDS[5],
        enable: crate::bind::flow::enable,
    },
    // G3-3: world cells, read and written by the runtime's before-tick
    // hook (stream::cells); no module to grant. Naming any of the cell's
    // records registers them together; an anchor alone is its own.
    Lazy {
        name: crate::world::stream::cells::CELL,
        enable: |_, _| Ok(()),
    },
    Lazy {
        name: crate::world::stream::cells::MEMBER,
        enable: |_, _| Ok(()),
    },
    Lazy {
        name: crate::world::stream::cells::EVENT,
        enable: |_, _| Ok(()),
    },
    Lazy {
        name: crate::world::stream::cells::ANCHOR,
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
        name: crate::present::fx::GLOBAL_ILLUMINATION,
        enable: |_, _| Ok(()),
    },
    // The lens flare's settings, read by extraction alone.
    Lazy {
        name: crate::present::fx::LENS_FLARE,
        enable: |_, _| Ok(()),
    },
    // R3: realtime light volumes, read by extraction alone.
    Lazy {
        name: crate::present::probes::REALTIME_LIGHT_VOLUME,
        enable: |_, _| Ok(()),
    },
    // R12: the runtime's before-tick hook reads and writes them
    // (crate::world::origin); no module to grant. Naming either registers both.
    Lazy {
        name: crate::world::origin::RESOURCE,
        enable: |_, _| Ok(()),
    },
    Lazy {
        name: crate::world::origin::EVENT,
        enable: |_, _| Ok(()),
    },
    // og.assert's bounds: the default invariants read them (crate::engine::invariants).
    Lazy {
        name: crate::engine::invariants::BOUNDS,
        enable: |_, _| Ok(()),
    },
    // G3-2: og.video plays it; a world without footage keeps its hash
    // whatever the player's layout becomes.
    Lazy {
        name: crate::bind::video::PLAYER,
        enable: |w, _| {
            w.grant(
                crate::bind::video::VIDEO_SYSTEM,
                &[crate::bind::video::PLAYER],
            )
        },
    },
    // og.MediaIntent asks a player to change at the tick boundary, once a
    // game names it.
    Lazy {
        name: crate::bind::video::MEDIA_INTENT,
        enable: |w, _| {
            w.grant(
                crate::bind::video::VIDEO_SYSTEM,
                &[crate::bind::video::MEDIA_INTENT],
            )?;
            w.grant(
                crate::bind::modules::UI_END,
                &[crate::bind::video::MEDIA_INTENT],
            )
        },
    },
    // 3D VFX: og.vfx plays composed effects, sends og.VfxEvent and
    // despawns them, once a game names og.Vfx or og.VfxEvent.
    Lazy {
        name: crate::bind::vfx::COMPONENT,
        enable: crate::bind::vfx::enable,
    },
    Lazy {
        name: crate::bind::vfx::EVENT,
        enable: crate::bind::vfx::enable,
    },
    // R7: og.particles simulates the systems whose emitters send events
    // and sends them, once a game names the event.
    Lazy {
        name: crate::bind::particles::EVENT,
        enable: crate::bind::particles::enable,
    },
    // R10: virtual cameras; og.camera's brain writes them (and
    // og.PostEffects, for a shot's focus). Naming one registers all three.
    Lazy {
        name: crate::present::feel::vcam::VCAM,
        enable: |w, _| crate::present::feel::vcam::enable(w, crate::present::feel::vcam::VCAM),
    },
    Lazy {
        name: crate::present::feel::vcam::BRAIN,
        enable: |w, _| crate::present::feel::vcam::enable(w, crate::present::feel::vcam::BRAIN),
    },
    Lazy {
        name: crate::present::feel::vcam::CUT,
        enable: |w, _| crate::present::feel::vcam::enable(w, crate::present::feel::vcam::CUT),
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
    // The aurora: read by extraction alone, as the clouds.
    Lazy {
        name: "og.Aurora",
        enable: |_, _| Ok(()),
    },
    // R5: the 3D water surface, read by extraction (and og.physics.waterHeight's
    // sea) alone: a world without water keeps its type table and hash as the
    // surface grows (the sea, the flow map).
    Lazy {
        name: "og.Water",
        enable: |_, _| Ok(()),
    },
    // R6: the grass, read by extraction alone: a world without grass keeps
    // its type table and hash.
    Lazy {
        name: "og.Grass",
        enable: |_, _| Ok(()),
    },
    // The terrain component, read by extraction (and describe) alone: a
    // world without a terrain keeps its type table and hash as it grows
    // fields (T5's horizonShadows).
    Lazy {
        name: "og.Terrain",
        enable: |_, _| Ok(()),
    },
    // T3: terrain stamps, read by extraction alone: a world without one
    // keeps its type table and hash.
    Lazy {
        name: "og.TerrainStamp",
        enable: |_, _| Ok(()),
    },
    // Area 4: the scattered copies and the wind, read by extraction alone:
    // a world without them keeps its type table and hash as they grow
    // (G4's placement rules, G2's simulated field).
    Lazy {
        name: "og.TerrainScatter",
        enable: |_, _| Ok(()),
    },
    Lazy {
        name: "og.Wind",
        enable: |_, _| Ok(()),
    },
    // Area 4 (G2, G3): the wind's motors and the grass's pushers, read by
    // extraction alone: a world without them keeps its type table and hash.
    Lazy {
        name: "og.WindMotor",
        enable: |_, _| Ok(()),
    },
    Lazy {
        name: "og.GrassPusher",
        enable: |_, _| Ok(()),
    },
    // What nothing grows in, read by extraction alone: a world without one
    // keeps its type table and hash.
    Lazy {
        name: "og.ScatterClearing",
        enable: |_, _| Ok(()),
    },
    // A region's acceleration, read as the tick starts
    // (bind::forces): a world without one keeps its type table and hash.
    Lazy {
        name: "og.ForceField",
        enable: |_, _| Ok(()),
    },
    // Fog inside a region, read by extraction alone.
    Lazy {
        name: "og.FogVolume",
        enable: |_, _| Ok(()),
    },
    // The shape of a region on its entity: og.audio bounds an
    // og.ReverbZone by it, and the cells read it as the tick starts.
    Lazy {
        name: "og.Volume",
        enable: |w, _| og_audio::enable_volumes(w),
    },
    // G4: the kinds of blade og.Grass fields list.
    Lazy {
        name: "og.GrassType",
        enable: |_, _| Ok(()),
    },
    // R4: the weather's simulated state and its strikes; og.weather writes
    // and sends them. Naming either registers both.
    Lazy {
        name: crate::bind::weather::STATE,
        enable: |w, _| crate::bind::weather::enable(w, crate::bind::weather::STATE),
    },
    Lazy {
        name: crate::bind::weather::STRIKE,
        enable: |w, _| crate::bind::weather::enable(w, crate::bind::weather::STRIKE),
    },
    // The time of year; og.season writes the date it draws and reads.
    Lazy {
        name: crate::bind::season::STATE,
        enable: |w, _| crate::bind::season::enable(w),
    },
    // og.sight: read by the runtime's before-tick hook (bind::sight); no
    // module to grant. Naming any registers the three and the state block.
    Lazy {
        name: crate::bind::sight::RECORDS[0],
        enable: |_, _| Ok(()),
    },
    Lazy {
        name: crate::bind::sight::RECORDS[1],
        enable: |_, _| Ok(()),
    },
    Lazy {
        name: crate::bind::sight::RECORDS[2],
        enable: |_, _| Ok(()),
    },
    // og.shared: its system once a game names a record or joins (TOGETHER).
    Lazy {
        name: crate::bind::shared::RECORDS[0],
        enable: crate::bind::shared::enable,
    },
    Lazy {
        name: crate::bind::shared::RECORDS[1],
        enable: crate::bind::shared::enable,
    },
    Lazy {
        name: crate::bind::shared::RECORDS[2],
        enable: crate::bind::shared::enable,
    },
    Lazy {
        name: crate::bind::shared::RECORDS[3],
        enable: crate::bind::shared::enable,
    },
    Lazy {
        name: crate::bind::shared::RECORDS[4],
        enable: crate::bind::shared::enable,
    },
    Lazy {
        name: crate::bind::shared::RECORDS[5],
        enable: crate::bind::shared::enable,
    },
    Lazy {
        name: crate::bind::shared::RECORDS[6],
        enable: crate::bind::shared::enable,
    },
    Lazy {
        name: crate::bind::shared::RECORDS[7],
        enable: crate::bind::shared::enable,
    },
    Lazy {
        name: crate::bind::shared::RECORDS[8],
        enable: crate::bind::shared::enable,
    },
    Lazy {
        name: crate::bind::shared::RECORDS[9],
        enable: crate::bind::shared::enable,
    },
    Lazy {
        name: crate::bind::shared::RECORDS[10],
        enable: crate::bind::shared::enable,
    },
    Lazy {
        name: crate::bind::shared::RECORDS[11],
        enable: crate::bind::shared::enable,
    },
    // og.scene: written and sent by the runtime's before-tick hook
    // (world::scenes) once a game first changes scene; no module to grant.
    Lazy {
        name: crate::world::scenes::RECORDS[0],
        enable: |_, _| Ok(()),
    },
    Lazy {
        name: crate::world::scenes::RECORDS[1],
        enable: |_, _| Ok(()),
    }, // og.play: its system and state block once a game names any of its
    // records (TOGETHER), the last one registered enabling it.
    Lazy {
        name: crate::bind::play::RECORDS[0],
        enable: crate::bind::play::enable,
    },
    Lazy {
        name: crate::bind::play::RECORDS[1],
        enable: crate::bind::play::enable,
    },
    Lazy {
        name: crate::bind::play::RECORDS[2],
        enable: crate::bind::play::enable,
    },
];

/// Lazy records registered together: naming one registers the rest.
pub(crate) const TOGETHER: &[&[&str]] = &[
    &["og.TimeGroup", "og.TimeGroups"],
    &[crate::world::origin::RESOURCE, crate::world::origin::EVENT],
    &crate::bind::story::RECORDS,
    &crate::present::feel::vcam::RECORDS,
    &crate::bind::weather::RECORDS,
    &crate::bind::sight::RECORDS,
    &crate::bind::destruction::RECORDS,
    &crate::bind::destruction2d::RECORDS,
    &crate::bind::vfx::RECORDS,
    &crate::bind::flow::RECORDS,
    &crate::bind::shared::RECORDS,
    &crate::world::scenes::RECORDS,
    &crate::bind::play::RECORDS,
    &[
        crate::world::stream::cells::CELL,
        crate::world::stream::cells::MEMBER,
        crate::world::stream::cells::EVENT,
    ],
];

/// Lazy records a module's system declares beside its own, registered
/// before its records enable it: og.flow's system writes its decks'
/// og.VideoPlayer.
pub(crate) const NEEDS: &[(&[&str], &[&str])] =
    &[(&crate::bind::flow::RECORDS, &[crate::bind::video::PLAYER])];

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

/// What attaches to a lazy record: see [`ATTACH`].
pub(crate) type AttachFn = fn(Mount<'_>, &mut World) -> Result<(), String>;

/// Run on a lazy record's world once its module is granted it, as the
/// record is registered or at assembly: the state the module keeps of it.
pub(crate) const ATTACH: &[(&str, AttachFn)] = &[
    (crate::bind::story::STORY, story::attach),
    (crate::world::stream::cells::CELL, stream::cells::attach),
    (crate::bind::sight::MAP, sight::attach),
];

/// What a module mounts on a world being assembled with.
pub(crate) struct Assembly<'a> {
    pub mount: Mount<'a>,
    /// og-physics' profile (2D or 3D).
    pub profile: u32,
    pub physics: &'a PhysicsSystem,
    /// Every component and resource the world has registered.
    pub records: &'a [String],
    pub layouts: &'a BTreeMap<String, Layout>,
}

/// A module's mount: its systems and state blocks on a world.
pub(crate) type MountFn = fn(&mut World, &Assembly<'_>) -> Result<(), String>;

/// Mounted after physics' and the UI's front, before the game's systems.
pub(crate) const FRONT: &[MountFn] = &[
    |w, a| terrain::mount(w, a.mount),
    |w, a| platform::mount(w, a.mount),
    |w, a| stream::mount(w, a.mount),
    |w, a| net::mount(w, a.mount),
    |w, a| story::mount(w, a.mount),
    |w, a| agents::mount_ai(w, a.mount, a.profile, a.physics, a.records),
];

/// Mounted after the game's systems and tweens, before og-anim.
pub(crate) const MIDDLE: &[MountFn] = &[
    |w, a| video::install(w, a.mount, a.layouts),
    |w, a| crate::bind::weather::install(w, a.layouts),
    |w, a| crate::bind::season::install(w, a.layouts),
    // og.path and og.timeline after tweens, before og-anim (an animation
    // track sets players) and physics.
    |w, a| timeline::install(w, a.mount, a.layouts),
];

/// Mounted after og-anim and game feel's middle, before physics' back.
pub(crate) const BACK: &[MountFn] = &[
    |w, a| agents::mount_nav(w, a.mount, a.profile, a.physics),
    |_, a| {
        destruction::set_physics(&a.mount.modules.destruction, a.physics);
        Ok(())
    },
];

/// Run as each tick starts, after a pending restore and the save's own
/// hook, before the debug geometry and interpolation.
pub(crate) const BEFORE_TICK: &[fn(&Engine, &mut World)] = &[
    // The world's zero follows its focus (R12), before anything reads a
    // place: cells, the raycast record, interpolation.
    origin::before_tick,
    // The water og.Buoyancy floats on this tick, at the new zero.
    |e, w| crate::bind::physics_ext::before_tick(e, w),
    // The air this tick's bodies and particles move through.
    |e, w| wind::before_tick(e, w),
    // The force fields this tick's bodies are accelerated by.
    |e, w| forces::before_tick(e, w),
    // What each team sees this tick (og.sight), from where things stand.
    |e, w| sight::before_tick(e, w),
    // A scene change asked for puts its scene in place, before the cells
    // it places are seen.
    scenes::before_tick,
    // The scenes timelines' scene tracks cut to and from.
    timeline::before_tick,
    // World cells load and unload as the tick starts (G3-3).
    stream::cells::before_tick,
];

/// A module's probe actions and the names it registers them under, which
/// are reserved for the engine whatever the build or the order (a
/// publish build registers fewer of them, never more).
pub(crate) struct Commands {
    pub register: fn(&mut og_probe::Probe, u32),
    pub actions: &'static [&'static str],
}

/// The engine's own probe actions, registered in this order.
pub(crate) const COMMANDS: &[Commands] = &[
    Commands {
        register: agents::register_commands,
        actions: &[
            // author::pulse, author::lanes, og.net, the debug geometry and
            // author::review, which og.ai's registers first.
            "dump_pulse",
            "dump_lanes",
            "lanes",
            "dump_desync",
            "dump_room",
            "debug",
            "view",
            "turntable",
            "dump_memory",
            "dump_ai",
            "dump_nav",
            "locale",
            "cover",
            "dump_story",
            "dump_stream",
            "dump_platform",
            "mip_stream",
            "terrain_stream",
        ],
    },
    Commands {
        register: timeline::register_commands,
        actions: &["timeline"],
    },
    Commands {
        register: flow::register_commands,
        actions: &["dump_flow", "flow"],
    },
    Commands {
        register: motion::register_commands,
        actions: &["dump_motion", "motion_backend"],
    },
    Commands {
        register: modules::docs::register_commands,
        actions: &["ui"],
    },
    Commands {
        register: crate::author::probe::register_render_dump,
        actions: &["dump_render"],
    },
    Commands {
        register: crate::present::probes::register_bake_dump,
        actions: &["bake_probes"],
    },
    Commands {
        register: |_probe, _| {
            #[cfg(not(feature = "publish"))]
            crate::author::determinism::register(_probe);
        },
        actions: &["dump_determinism"],
    },
    Commands {
        register: |_probe, _engine| {
            #[cfg(not(feature = "publish"))]
            crate::author::inspector::register(_probe, _engine);
        },
        actions: &["dump_inspector", "inspect"],
    },
];

/// Run once the probe's world is the engine's (at creation): each takes
/// the probe's inbox for its commands.
pub(crate) const COMMIT: &[fn(&Engine, og_probe::Inbox)] = &[agents::commit, timeline::commit];

/// Run once an engine is made, with the host's options.
pub(crate) const CREATED: &[fn(&Engine, &Value)] = &[
    // The host's locale (og run --locale, the browser's language).
    |e, options| i18n::request(e, options["locale"].as_str().unwrap_or("")),
];

/// GameDesc.start waits while one of these holds, as it waits for the
/// first scene.
pub(crate) const HOLD_START: &[fn(&Engine) -> bool] = &[
    // The locale's font-pack faces, which the web fetches after the boot
    // (docs/ui.md §3.4): an og.load.font of one in start finds it, and
    // the first frame needs them anyway.
    i18n::faces_pending,
];

/// A module's start: see [`BEFORE_START`].
pub(crate) type StartFn = fn(&Engine) -> Result<(), String>;

/// Run before GameDesc.start, after GameDesc.look: a failure is the
/// engine's start failure, in the words returned.
pub(crate) const BEFORE_START: &[StartFn] = &[
    // project.ogt's `flow`, which start may read (og.flow.current) or
    // pause behind a title screen. Taken here, so a game without one never
    // calls into the story module (wasm-modules.json).
    |e| {
        let flow = e.state.borrow_mut().boot_flow.take();
        flow.map_or(Ok(()), |dialogue| flow::start_project_flow(e, dialogue))
    },
];

#[cfg(test)]
mod tests {
    /// The engine's core walks the lists above and names no binding: a
    /// module that adds itself by a call in one of these files fails here.
    #[test]
    fn the_core_names_no_module() {
        let bind = include_str!("bind.rs");
        let modules: Vec<&str> = bind
            .lines()
            .filter_map(|l| l.trim().strip_prefix("pub(crate) mod "))
            .filter_map(|l| l.strip_suffix(';'))
            .collect();
        assert!(modules.contains(&"sight"), "bind.rs lists the bindings");
        let core = [
            ("engine/lazy.rs", include_str!("engine/lazy.rs")),
            (
                "engine/module_state.rs",
                include_str!("engine/module_state.rs"),
            ),
            ("engine/creation.rs", include_str!("engine/creation.rs")),
            ("lib.rs", include_str!("lib.rs")),
        ];
        let mut named = Vec::new();
        for (file, text) in core {
            for (n, line) in text.lines().enumerate() {
                for m in &modules {
                    // `super::` is engine.rs's alias of a binding.
                    for path in [format!("bind::{m}"), format!("super::{m}::")] {
                        let at = line.match_indices(&path).any(|(i, _)| {
                            let rest = &line[i + path.len()..];
                            !rest.starts_with(|c: char| c.is_alphanumeric() || c == '_')
                                || path.ends_with("::")
                        });
                        if at {
                            named.push(format!("{file}:{}: {}", n + 1, line.trim()));
                        }
                    }
                }
            }
        }
        assert!(
            named.is_empty(),
            "the core names a module:\n{}",
            named.join("\n")
        );
    }
}
