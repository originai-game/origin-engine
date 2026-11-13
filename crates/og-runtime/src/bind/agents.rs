//! The agents' modules on the runtime: navigation (`og.nav`, over og-ai's nav)
//! and AI (`og.ai`, over og-ai), mounted on every world. Their registries (grids,
//! navmeshes, flow fields, behaviours) are assets of the engine and outlive
//! a hot reload; their systems belong to a world, so `assemble` mounts new
//! ones as pending and the engine commits them when the world is the one
//! that runs (creation, or a reload the probe swapped in).
//!
//! The schedule they add: `og.ai` right after the UI's begin (before the
//! game's systems, so the game reads the tick's decisions) and `og.nav`
//! right before physics (after the game's systems, so it moves agents the
//! game retargeted). og.ai's game commands go to the probe's inbox and run
//! at the start of the next tick, as a scripted command would.

use crate::engine::Mount;
#[cfg(feature = "nav")]
use og_ai::nav::{NavSystem, Plane, SharedNavs};
#[cfg(feature = "ai")]
use og_ai::{AiSystem, Behaviours};
use og_ecs::{Entity, World};
use og_physics::{PhysicsSystem, QueryFilter, RaycastDesc};
use std::cell::RefCell;
use std::sync::Arc;
#[cfg(feature = "ai")]
use std::sync::Mutex;

#[derive(Default)]
pub(crate) struct Modules {
    #[cfg(feature = "nav")]
    pub navs: SharedNavs,
    #[cfg(feature = "ai")]
    pub behaviours: Arc<Mutex<Behaviours>>,
    #[cfg(feature = "nav")]
    pub nav: Option<NavSystem>,
    #[cfg(feature = "ai")]
    pub ai: Option<AiSystem>,
    #[cfg(feature = "nav")]
    pending_nav: Option<NavSystem>,
    #[cfg(feature = "ai")]
    pending_ai: Option<AiSystem>,
    pub inbox: Option<og_probe::Inbox>,
    pub profile: u32,
}

/// og.nav and og.ai on one engine.
pub(crate) type ModuleState = RefCell<Modules>;

/// Runs `f` with an engine's modules (none for an engine that is gone).
pub(crate) fn with<T>(engine: u32, f: impl FnOnce(&mut Modules) -> T) -> Option<T> {
    let eng = crate::engine::engine(engine).ok()?;
    Some(f(&mut eng.modules.agents.borrow_mut()))
}

#[cfg(feature = "nav")]
pub(crate) fn plane_of(profile: u32) -> Plane {
    if profile == og_physics::PROFILE_2D {
        Plane::Xy
    } else {
        Plane::Xz
    }
}

/// The physics ray og.nav's avoidance and og.ai's sight cast, over the
/// colliders recorded at the tick's start.
fn ray(
    physics: &PhysicsSystem,
    origin: og_core::glam::Vec3,
    dir: og_core::glam::Vec3,
    max: f32,
    mask: u32,
    exclude: Entity,
) -> Option<og_physics::RaycastHit> {
    if dir.length_squared() < 1e-12 {
        return None;
    }
    let q = physics.tick_query().ok()?;
    q.raycast(&RaycastDesc {
        origin,
        direction: dir,
        max_distance: max,
        filter: QueryFilter {
            mask,
            include_sensors: false,
            exclude: Some(exclude),
        },
    })
    .ok()
    .flatten()
}

/// og.audio's occlusion: whether a collider stands between a source and
/// the listener (a physics ray between them, a quarter metre clear of the
/// source and half a metre of the listener, so neither's own body counts).
pub(crate) fn occluder(physics: &PhysicsSystem) -> og_audio::Occluder {
    let p = physics.clone();
    Arc::new(move |from, to| {
        let d = to - from;
        let dist = d.length();
        if dist < 0.8 {
            return false;
        }
        let dir = d / dist;
        ray(
            &p,
            from + dir * 0.25,
            dir,
            dist - 0.75,
            0,
            Entity::from_bits(0),
        )
        .is_some()
    })
}

/// og.ai, before the game's systems. `records` are every component and
/// resource the world has (what a behaviour's checks may read).
pub(crate) fn mount_ai(
    world: &mut World,
    mount: Mount<'_>,
    profile: u32,
    physics: &PhysicsSystem,
    records: &[String],
) -> Result<(), String> {
    mount.modules.agents.borrow_mut().profile = profile;
    ai_system(world, mount, profile, physics, records)
}

/// What og.ai hears of og.audio: the spatial sources that started this
/// tick, each as loud as its volume. The one place AI and audio meet.
#[cfg(feature = "ai")]
pub(crate) fn hearing() -> og_ai::Hearing {
    use og_audio::{AudioSource, schema::SoundStarted};
    use og_core::Reflect;
    og_ai::Hearing {
        reads: vec![
            SoundStarted::NAME.to_string(),
            AudioSource::NAME.to_string(),
        ],
        sounds: Arc::new(|ctx| {
            let mut out = Vec::new();
            for s in ctx.events::<SoundStarted>()? {
                let e = og_ecs::Entity::from_bits(s.entity);
                if let Ok(src) = ctx.get::<AudioSource>(e)
                    && src.spatial
                {
                    out.push((e, src.volume));
                }
            }
            Ok(out)
        }),
    }
}

/// og.ai over og-ai.
#[cfg(feature = "ai")]
fn ai_system(
    world: &mut World,
    mount: Mount<'_>,
    profile: u32,
    physics: &PhysicsSystem,
    records: &[String],
) -> Result<(), String> {
    let engine = mount.id;
    let behaviours = mount.modules.agents.borrow().behaviours.clone();
    let p = physics.clone();
    let sight: og_ai::SightRay = Arc::new(move |o, d, max, mask, ex| {
        ray(&p, o, d, max, mask, ex).map(|h| (h.distance, h.entity))
    });
    let sink: og_ai::CommandSink = Arc::new(move |name: &str, args: Vec<String>| {
        if let Some(Some(inbox)) = with(engine, |m| m.inbox.clone()) {
            inbox.push(name, args);
        }
    });
    let ai = og_ai::install(
        world,
        records,
        og_ai::AiDesc {
            plane: plane_of(profile),
            ray: Some(sight),
            sink: Some(sink),
            behaviours: Some(behaviours),
            hearing: Some(hearing()),
        },
    )
    .map_err(|e| e.reason().to_string())?;
    mount.modules.agents.borrow_mut().pending_ai = Some(ai);
    Ok(())
}

/// og.ai idle (src/bind/agents_idle.rs): the system and block of a world that never
/// thinks.
#[cfg(not(feature = "ai"))]
fn ai_system(
    world: &mut World,
    _mount: Mount<'_>,
    _profile: u32,
    _physics: &PhysicsSystem,
    records: &[String],
) -> Result<(), String> {
    crate::bind::agents_idle::mount_ai(world, records, crate::bind::ai::ABSENT)
        .map_err(|e| e.reason().to_string())
}

/// og.nav idle (src/bind/agents_idle.rs), before physics.
#[cfg(not(feature = "nav"))]
pub(crate) fn mount_nav(
    world: &mut World,
    _mount: Mount<'_>,
    _profile: u32,
    _physics: &PhysicsSystem,
) -> Result<(), String> {
    crate::bind::agents_idle::mount_nav(world, crate::bind::nav::ABSENT)
        .map_err(|e| e.reason().to_string())
}

/// og.nav, before physics.
#[cfg(feature = "nav")]
pub(crate) fn mount_nav(
    world: &mut World,
    mount: Mount<'_>,
    profile: u32,
    physics: &PhysicsSystem,
) -> Result<(), String> {
    let navs = mount.modules.agents.borrow().navs.clone();
    let p = physics.clone();
    let avoid: og_ai::nav::Raycaster = Arc::new(move |o, d, max, mask, ex| {
        ray(&p, o, d, max, mask, ex).map(|h| (h.distance, h.normal))
    });
    let nav = og_ai::nav::install(
        world,
        navs,
        og_ai::nav::NavDesc {
            plane: plane_of(profile),
            raycast: Some(avoid),
            plans_per_tick: og_ai::nav::PLANS_PER_TICK,
        },
    )
    .map_err(|e| e.reason().to_string())?;
    mount.modules.agents.borrow_mut().pending_nav = Some(nav);
    Ok(())
}

/// The world assembled last is the one that runs: its systems become the
/// engine's, and its probe's inbox takes og.ai's commands.
pub(crate) fn commit(eng: &crate::engine::Engine, inbox: og_probe::Inbox) {
    {
        let m = &mut *eng.modules.agents.borrow_mut();
        #[cfg(feature = "nav")]
        {
            m.nav = m.pending_nav.take().or(m.nav.take());
        }
        #[cfg(feature = "ai")]
        {
            m.ai = m.pending_ai.take().or(m.ai.take());
        }
        m.inbox = Some(inbox);
    }
}

/// After a reload: the rebuilt world's systems when it was swapped in.
pub(crate) fn after_reload(engine: u32, swapped: bool) {
    with(engine, |_m| {
        #[cfg(feature = "nav")]
        {
            let nav = _m.pending_nav.take();
            if swapped {
                _m.nav = nav.or(_m.nav.take());
            }
        }
        #[cfg(feature = "ai")]
        {
            let ai = _m.pending_ai.take();
            if swapped {
                _m.ai = ai.or(_m.ai.take());
            }
        }
        let _ = swapped;
    });
}

/// og.ai's warnings since the last call (each once per engine).
pub(crate) fn take_warnings(engine: u32) -> Vec<String> {
    #[cfg(feature = "ai")]
    return with(engine, |m| {
        m.ai.as_ref().map(|a| a.take_warnings()).unwrap_or_default()
    })
    .unwrap_or_default();
    #[cfg(not(feature = "ai"))]
    {
        let _ = engine;
        Vec::new()
    }
}

/// The dump actions (`dump_ai:<entity>[:path]`, `dump_nav:<entity>[:path]`),
/// the probe's `listen`, over the engine's audio link, and its `memory`.
pub(crate) fn register_commands(probe: &mut og_probe::Probe, engine: u32) {
    crate::author::pulse::register(probe, engine);
    crate::author::lanes::register(probe, engine);
    crate::bind::net::register(probe, engine);
    crate::present::geometry::register(probe, engine);
    #[cfg(not(feature = "publish"))]
    crate::author::review::register(probe, engine);
    probe.set_memory(move |world| crate::author::memory::report(engine, world));
    probe.set_listen(move |seconds| {
        let link = og_audio::link(engine)?;
        let (pcm, rate) = link.listen().last(seconds)?;
        let peak = pcm.iter().fold(0.0f32, |m, x| m.max(x.abs()));
        let rms = (pcm.iter().map(|x| x * x).sum::<f32>() / pcm.len().max(1) as f32).sqrt();
        Some(og_probe::Heard {
            wav: og_audio::wav(&pcm, 2, rate),
            rate,
            frames: (pcm.len() / 2) as u64,
            peak,
            rms,
        })
    });
    let _ = probe.register_dump(
        "dump_ai",
        og_probe::CommandDesc {
            usage: "dump_ai:<entity>[:path]".into(),
            doc: "an AI agent's behaviour, state, running node and the path to it, status, blackboard and perception".into(),
        },
        1,
        move |world, args| {
            let id = args.first().ok_or("write dump_ai:<entity>[:path]")?;
            let e = og_probe::Names.resolve(world, id).ok_or_else(|| format!("no live entity '{id}'"))?;
            crate::bind::ai::describe_json(engine, world, e).ok_or_else(|| format!("{id} has not thought: it needs an og.AiAgent with a loaded behaviour"))
        },
    );
    let _ = probe.register_command(
        "locale",
        og_probe::CommandDesc {
            usage: "locale:<tag>".into(),
            doc: "ask for a locale (og.i18n.setLocale): ja, zh-CN, ar".into(),
        },
        move |_world, args| {
            let tag = args
                .first()
                .filter(|a| !a.is_empty())
                .ok_or("write locale:<tag>, such as locale:ja")?;
            crate::bind::i18n::og_i18n_set_locale(engine, tag.clone());
            Ok(())
        },
    );
    #[cfg(not(feature = "publish"))]
    let _ = probe.register_command(
        "cover",
        og_probe::CommandDesc {
            usage: "cover:<x>,<y>,<width>,<height>|none".into(),
            doc: "where a host's own control covers the play surface (Viewport.covered, CSS pixels), as the published page's Back does: cover:12,12,40,40 lays a native run out as the page; none clears it".into(),
        },
        move |_world, args| crate::bind::modules::set_cover(engine, args),
    );
    let _ = probe.register_dump(
        "dump_story",
        og_probe::CommandDesc {
            usage: "dump_story[:path]".into(),
            doc: "og.story's whole state: variables, seen nodes, every conversation's node, line, voice, timer and last roll, quests with their counts and the events they count, barks being said, ambient barkers, bark cooldowns, voices playing and the story's random stream".into(),
        },
        0,
        move |_world, _args| Ok(crate::bind::story::dump(engine)),
    );
    let _ = probe.register_dump(
        "dump_platform",
        og_probe::CommandDesc {
            usage: "dump_platform[:path]".into(),
            doc: "every platform call (store, leaderboard, telemetry, point, AI mark): its method, arguments, tick, state and answer, and what answers them".into(),
        },
        0,
        move |_world, _args| Ok(crate::host::platform::dump(engine)),
    );
    let _ = probe.register_dump(
        "dump_memory",
        og_probe::CommandDesc {
            usage: "dump_memory[:path]".into(),
            doc: "what the engine holds, as the probe's memory: the process's resident memory (the wasm linear memory on the web), the counted heap's live bytes and peak, the world's entities, the streaming budget, the audio bank, and the renderer's GPU memory by textures, meshes, render targets and buffers".into(),
        },
        0,
        move |world, _args| Ok(crate::author::memory::report(engine, world)),
    );
    let _ = probe.register_dump(
        "dump_stream",
        og_probe::CommandDesc {
            usage: "dump_stream[:path]".into(),
            doc: "the streaming groups: the memory budget, what is used and its peak, every group's trigger, state, why and since which tick and what it keeps for the frame after unloading, and the recent state changes; the terrain streamer: its mode, tiles resident, waiting, decoding and cached; the world cells: each cell's entity, sub-scene, whether it is loaded, its entities and when it last loaded and unloaded; and the textures streamed by their mips: the mode, the mip budget, what is used, and each texture's state, footprint and sizes".into(),
        },
        0,
        move |_world, _args| Ok(crate::world::stream::dump(engine)),
    );
    let _ = probe.register_command(
        "terrain_stream",
        og_probe::CommandDesc {
            usage: "terrain_stream:<stream|resident>[:<MiB>]".into(),
            doc: "stream terrain tiles as the view needs them (the default) or decode and show every wanted tile each frame (resident), with each terrain's tile budget in MiB".into(),
        },
        move |_world, args| crate::bind::terrain::set_streaming(engine, args),
    );
    let _ = probe.register_command(
        "mip_stream",
        og_probe::CommandDesc {
            usage: "mip_stream:<stream|resident>[:<MiB>]".into(),
            doc: "stream the textures cooked with `stream true` by their mips, whole while big on screen within the budget (the default), or read and show every one whole before each frame (resident), with the mip budget in MiB".into(),
        },
        move |_world, args| crate::world::stream::mips::set_mode(engine, args),
    );
    let _ = probe.register_dump(
        "dump_nav",
        og_probe::CommandDesc {
            usage: "dump_nav:<entity>[:path]".into(),
            doc: "a navigation agent's path, the corner it heads for and its status".into(),
        },
        1,
        move |world, args| {
            let id = args.first().ok_or("write dump_nav:<entity>[:path]")?;
            let e = og_probe::Names
                .resolve(world, id)
                .ok_or_else(|| format!("no live entity '{id}'"))?;
            crate::bind::nav::path_json(engine, world, e)
                .ok_or_else(|| format!("{id} has not planned: it needs an og.NavAgent that seeks"))
        },
    );
}
