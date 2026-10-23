//! og.ReflectionProbe into the frame (`og_render::ReflectionProbe`,
//! `docs/design.md` §2.6), the baked captures `og bake probes` wrote into
//! the cook cache, and the `bake_probes` dump that command reads the
//! captures back with.
//!
//! A probe's bake is keyed by what it would capture: its position (to the
//! millimetre), box and face size (`probes/<16 hex digits>`, an
//! `og.EnvironmentMap` output of the manifest). A baked probe whose key is
//! in the bundle samples that map; one whose key is not (never baked, or
//! moved since) draws its own capture on its first frame, as a realtime
//! probe does every `interval` ticks.

use super::engine::Engine;
use super::extract::{Hierarchy, RenderState};
use og_core::hash::StableHasher;
use og_ecs::World;
use og_render::{MAX_REFLECTION_PROBES, ProbeMode, ReflectionProbe};
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// The baked captures tried, by key: the map once loaded (`None`: not in
/// the bundle, or not loadable).
#[derive(Default)]
pub(crate) struct ProbeBakes {
    pub tried: BTreeMap<String, Option<og_render::EnvironmentMapHandle>>,
}

/// The asset id of `p`'s bake.
pub(crate) fn key(p: &ReflectionProbe) -> String {
    let mut h = StableHasher::new();
    for v in p.position.to_array().into_iter().chain(p.size.to_array()) {
        h.write_u64((v * 1000.0).round() as i64 as u64);
    }
    h.write_u64(p.face_size() as u64);
    format!("probes/{:016x}", h.finish())
}

/// og.ReflectionProbe rows into `rs.scene.reflection_probes`: the visible
/// ones nearest the camera first (at most [`MAX_REFLECTION_PROBES`]), each
/// baked one with its loaded bake.
pub(crate) fn extract(world: &World, rs: &mut RenderState, h: &mut Hierarchy) {
    rs.scene.reflection_probes.clear();
    let Some(layout) = rs.draw.probe.clone() else {
        return;
    };
    let f = |v: &Value, k: &str, d: f32| v[k].as_f64().map_or(d, |x| x as f32);
    let mut probes: Vec<ReflectionProbe> = Vec::new();
    for (e, v) in super::extract::rows(world, &layout) {
        if !v["visible"].as_bool().unwrap_or(true) {
            continue;
        }
        let m = h.world_of(e);
        let size = &v["size"];
        let mut p = ReflectionProbe::new(
            e,
            m.transform_point3(glam::Vec3::ZERO),
            glam::Vec3::new(f(size, "x", 10.0), f(size, "y", 10.0), f(size, "z", 10.0)),
        );
        p.resolution = v["resolution"].as_u64().unwrap_or(128) as u32;
        p.mode = match v["mode"].as_u64().unwrap_or(0) {
            1 => ProbeMode::Realtime,
            _ => ProbeMode::Baked,
        };
        p.interval = v["interval"].as_u64().unwrap_or(30).max(1) as u32;
        p.intensity = f(&v, "intensity", 1.0).max(0.0);
        p.blend_distance = f(&v, "blendDistance", 1.0).max(0.0);
        if p.mode == ProbeMode::Baked {
            p.baked = rs.probe_bakes.tried.get(&key(&p)).copied().flatten();
        }
        probes.push(p);
    }
    let eye = rs.scene.camera.position;
    probes.sort_by(|a, b| {
        a.position
            .distance_squared(eye)
            .total_cmp(&b.position.distance_squared(eye))
            .then(a.id.cmp(&b.id))
    });
    probes.truncate(MAX_REFLECTION_PROBES);
    rs.scene.reflection_probes = probes;
}

/// After extraction: the bakes of the frame's baked probes not tried yet,
/// loaded from the bundle when it has them (their probes sample them from
/// this frame on).
pub(crate) fn load_bakes(eng: &Engine, rs: &mut RenderState) {
    let wanted: Vec<(usize, String)> = rs
        .scene
        .reflection_probes
        .iter()
        .enumerate()
        .filter(|(_, p)| p.mode == ProbeMode::Baked && p.baked.is_none())
        .map(|(i, p)| (i, key(p)))
        .filter(|(_, k)| !rs.probe_bakes.tried.contains_key(k))
        .collect();
    for (i, k) in wanted {
        let size = rs.scene.reflection_probes[i].face_size();
        let map = {
            let assets = eng.assets.borrow();
            assets
                .blob(&k)
                .ok()
                .and_then(|(bytes, bundle)| bundle.environment(&k, bytes.bytes()).ok())
        };
        let handle = map
            .filter(|m| m.size == size)
            .map(|m| rs.scene.add_environment_map(m));
        rs.probe_bakes.tried.insert(k, handle);
        rs.scene.reflection_probes[i].baked = handle;
    }
}

/// The `bake_probes` dump: every probe of the last frame whose capture the
/// renderer drew, read back (native hosts), with its key and face size —
/// what `og bake probes` writes into the cook cache.
pub(crate) fn register_bake_dump(probe: &mut og_probe::Probe, engine: u32) {
    let _ = probe.register_dump(
        "bake_probes",
        og_probe::CommandDesc {
            usage: "bake_probes[:path]".into(),
            doc: "the captures of the frame's og.ReflectionProbe components read back (native hosts): per probe its bake key, face size and prefiltered levels, for `og bake probes`".into(),
        },
        0,
        move |_world, _args| {
            let eng = super::engine::engine(engine)?;
            let rs = eng.render.borrow();
            let backend = rs
                .backend
                .as_ref()
                .ok_or("no renderer: bake_probes reads the probes' captures back")?;
            let mut out = Vec::new();
            for p in &rs.scene.reflection_probes {
                let Some(read) = backend.read_reflection_probe(p.id) else {
                    continue;
                };
                let (size, levels) = read?;
                out.push(json!({
                    "key": key(p),
                    "entity": p.id,
                    "size": size,
                    "levels": levels,
                }));
            }
            Ok(json!({ "kind": "probes", "probes": out }))
        },
    );
}
