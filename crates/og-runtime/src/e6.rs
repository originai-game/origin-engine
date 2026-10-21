//! The E6 modules on the runtime: navigation (`og.nav`, over og-nav) and AI
//! (`og.ai`, over og-ai), mounted on every world. Their registries (grids,
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

use og_ai::{AiSystem, Behaviours};
use og_ecs::{Entity, World};
use og_nav::{NavSystem, Plane, SharedNavs};
use og_physics::{PhysicsSystem, QueryFilter, RaycastDesc};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub(crate) struct Modules {
    pub navs: SharedNavs,
    pub behaviours: Arc<Mutex<Behaviours>>,
    pub nav: Option<NavSystem>,
    pub ai: Option<AiSystem>,
    pending_nav: Option<NavSystem>,
    pending_ai: Option<AiSystem>,
    pub inbox: Option<og_probe::Inbox>,
    pub profile: u32,
}

thread_local! {
    static MODULES: RefCell<BTreeMap<u32, Modules>> = const { RefCell::new(BTreeMap::new()) };
}

/// Runs `f` with an engine's modules.
pub(crate) fn with<T>(engine: u32, f: impl FnOnce(&mut Modules) -> T) -> Option<T> {
    MODULES.with(|m| m.borrow_mut().get_mut(&engine).map(f))
}

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
pub(crate) fn mount_front(
    world: &mut World,
    engine: u32,
    profile: u32,
    physics: &PhysicsSystem,
    records: &[String],
) -> Result<(), String> {
    let behaviours = MODULES.with(|m| m.borrow_mut().entry(engine).or_default().behaviours.clone());
    let p = physics.clone();
    let sight: og_ai::SightRay = Arc::new(move |o, d, max, mask, ex| {
        ray(&p, o, d, max, mask, ex).map(|h| (h.distance, h.entity))
    });
    let sink: og_ai::CommandSink = Arc::new(move |name: &str, args: Vec<String>| {
        if let Some(Some(inbox)) = with(engine, |m| m.inbox.clone()) {
            inbox.push(name, args);
        }
    });
    super::platform::mount(world, engine)?;
    super::stream::mount(world, engine)?;
    let ai = og_ai::install(
        world,
        records,
        og_ai::AiDesc {
            plane: plane_of(profile),
            ray: Some(sight),
            sink: Some(sink),
            behaviours: Some(behaviours),
        },
    )
    .map_err(|e| e.reason().to_string())?;
    MODULES.with(|m| {
        let mut m = m.borrow_mut();
        let entry = m.entry(engine).or_default();
        entry.profile = profile;
        entry.pending_ai = Some(ai);
    });
    Ok(())
}

/// og.nav, before physics.
pub(crate) fn mount_back(
    world: &mut World,
    engine: u32,
    profile: u32,
    physics: &PhysicsSystem,
) -> Result<(), String> {
    let navs = MODULES.with(|m| m.borrow_mut().entry(engine).or_default().navs.clone());
    let p = physics.clone();
    let avoid: og_nav::Raycaster = Arc::new(move |o, d, max, mask, ex| {
        ray(&p, o, d, max, mask, ex).map(|h| (h.distance, h.normal))
    });
    let nav = og_nav::install(
        world,
        navs,
        og_nav::NavDesc {
            plane: plane_of(profile),
            raycast: Some(avoid),
            plans_per_tick: og_nav::PLANS_PER_TICK,
        },
    )
    .map_err(|e| e.reason().to_string())?;
    MODULES.with(|m| m.borrow_mut().entry(engine).or_default().pending_nav = Some(nav));
    Ok(())
}

/// The world assembled last is the one that runs: its systems become the
/// engine's, and its probe's inbox takes og.ai's commands.
pub(crate) fn commit(engine: u32, inbox: og_probe::Inbox) {
    with(engine, |m| {
        m.nav = m.pending_nav.take().or(m.nav.take());
        m.ai = m.pending_ai.take().or(m.ai.take());
        m.inbox = Some(inbox);
    });
}

/// After a reload: the rebuilt world's systems when it was swapped in.
pub(crate) fn after_reload(engine: u32, swapped: bool) {
    with(engine, |m| {
        let (nav, ai) = (m.pending_nav.take(), m.pending_ai.take());
        if swapped {
            m.nav = nav.or(m.nav.take());
            m.ai = ai.or(m.ai.take());
        }
    });
}

/// og.ai's warnings since the last call (each once per engine).
pub(crate) fn take_warnings(engine: u32) -> Vec<String> {
    with(engine, |m| {
        m.ai.as_ref().map(|a| a.take_warnings()).unwrap_or_default()
    })
    .unwrap_or_default()
}

/// The engine is gone.
pub(crate) fn retire(engine: u32) {
    MODULES.with(|m| m.borrow_mut().remove(&engine));
    super::platform::retire(engine);
    super::stream::retire(engine);
    super::terrain::retire(engine);
    super::i18n::retire(engine);
}

/// The dump actions (`dump_ai:<entity>[:path]`, `dump_nav:<entity>[:path]`),
/// the probe's `listen`, over the engine's audio link, and its `memory`.
pub(crate) fn register_commands(probe: &mut og_probe::Probe, engine: u32) {
    super::polish::register(probe, engine);
    super::lanes::register(probe, engine);
    probe.set_memory(move |world| super::memory::report(engine, world));
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
            super::ai::describe_json(engine, world, e).ok_or_else(|| format!("{id} has not thought: it needs an og.AiAgent with a loaded behaviour"))
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
            super::i18n::og_i18n_set_locale(engine, tag.clone());
            Ok(())
        },
    );
    let _ = probe.register_dump(
        "dump_platform",
        og_probe::CommandDesc {
            usage: "dump_platform[:path]".into(),
            doc: "every platform call (store, leaderboard, telemetry, point, AI mark): its method, arguments, tick, state and answer, and what answers them".into(),
        },
        0,
        move |_world, _args| Ok(super::platform::dump(engine)),
    );
    let _ = probe.register_dump(
        "dump_stream",
        og_probe::CommandDesc {
            usage: "dump_stream[:path]".into(),
            doc: "the streaming groups: the memory budget, what is used and its peak, every group's trigger, state, why and since which tick and what it keeps for the frame after unloading, and the recent state changes; and the terrain streamer: its mode, tiles resident, waiting, decoding and cached".into(),
        },
        0,
        move |_world, _args| Ok(super::stream::dump(engine)),
    );
    let _ = probe.register_command(
        "terrain_stream",
        og_probe::CommandDesc {
            usage: "terrain_stream:<stream|resident>[:<MiB>]".into(),
            doc: "stream terrain tiles as the view needs them (the default) or decode and show every wanted tile each frame (resident), with each terrain's tile budget in MiB".into(),
        },
        move |_world, args| super::terrain::set_streaming(engine, args),
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
            super::nav::path_json(engine, world, e)
                .ok_or_else(|| format!("{id} has not planned: it needs an og.NavAgent that seeks"))
        },
    );
}
