//! og.ReflectionProbe into the frame (`og_render::ReflectionProbe`,
//! `docs/design.md` §2.6), the baked captures `og bake probes` wrote into
//! the cook cache, and the `bake_probes` dump that command reads the
//! captures back with.
//!
//! A probe's box is the og.Volume on its entity, as the world's axes bound
//! it (`og_env::volume::Volume::bounds`); a probe without one reaches
//! everywhere ([`EVERYWHERE`]). A light volume's box is its og.Volume's
//! the same way, and without one it has none.
//!
//! A probe's bake is keyed by what it would capture: its position (to the
//! millimetre), box and face size, and its room's content
//! (`og_render::ProbeContent`: the meshes, materials, textures, terrain,
//! scatter and lights in its box, by value) (`probes/<16 hex digits>`, an
//! `og.EnvironmentMap` output of the manifest). A baked probe whose key is
//! in the bundle samples that map; one whose key is not (never baked,
//! moved, or its room changed since) draws its own capture on its first
//! frame, as a realtime probe does every `intervalTicks` ticks.

use crate::engine::Engine;
use crate::host::wire::{FromJson, ReflectionProbeMode, record};
use crate::present::extract::{Hierarchy, RenderState};
use og_core::IdMap;
use og_core::hash::StableHasher;
use og_ecs::World;
use og_render::{MAX_REFLECTION_PROBES, ProbeMode, ReflectionProbe};
use serde_json::json;
use std::collections::BTreeMap;

/// The box of a probe without an og.Volume (m a side): everywhere a game
/// draws, its reflections as good as at infinity.
const EVERYWHERE: f32 = 1e6;

/// Where `e` puts a region and the og.Volume on it, if any.
fn region(
    world: &World,
    volume: Option<&crate::host::codec::Layout>,
    h: &mut Hierarchy,
    e: u64,
) -> (og_env::volume::Placement, Option<og_env::volume::Volume>) {
    let (scale, quaternion, position) = h.world_of(e).to_scale_rotation_translation();
    (
        og_env::volume::Placement {
            position,
            quaternion,
            scale,
        },
        crate::world::region::volume_of(world, volume, og_ecs::Entity::from_bits(e)),
    )
}

/// The baked captures tried, by key: the map once loaded (`None`: not in
/// the bundle, or not loadable).
#[derive(Default)]
pub(crate) struct ProbeBakes {
    pub tried: BTreeMap<String, Option<og_render::EnvironmentMapHandle>>,
    /// The rooms' content hashes, kept across frames.
    pub content: og_render::ProbeContent,
    /// The last frame's baked probes, by entity: their key and room.
    pub rooms: IdMap<u64, (String, og_render::ProbeRoom)>,
}

/// The asset id of `p`'s bake, whose room hashes to `room`.
pub(crate) fn key(p: &ReflectionProbe, room: u64) -> String {
    let mut h = StableHasher::new();
    for v in p.position.to_array().into_iter().chain(p.size.to_array()) {
        h.write_u64((v * 1000.0).round() as i64 as u64);
    }
    h.write_u64(p.face_size() as u64);
    h.write_u64(room);
    format!("probes/{:016x}", h.finish())
}

/// og.ReflectionProbe rows into `rs.scene.reflection_probes`: the visible
/// ones nearest the camera first (at most [`MAX_REFLECTION_PROBES`]), each
/// with the box and fade of the og.Volume on its entity and each baked one
/// with its loaded bake.
pub(crate) fn extract(world: &World, rs: &mut RenderState, h: &mut Hierarchy) {
    rs.scene.lighting.reflection_probes.clear();
    let Some(layout) = rs.draw.probe.clone() else {
        return;
    };
    let volume = rs.draw.volume.clone();
    let mut probes: Vec<ReflectionProbe> = Vec::new();
    for (e, v) in crate::present::extract::rows(world, &layout) {
        let Ok(v) = record::ReflectionProbe::from_json(&v) else {
            continue;
        };
        if !v.visible {
            continue;
        }
        let (at, shape) = region(world, volume.as_ref(), h, e);
        let (size, fade) = shape.map_or((glam::Vec3::splat(EVERYWHERE), 0.0), |s| {
            let (lo, hi) = s.bounds(&at);
            (hi - lo, s.fade.max(0.0))
        });
        let mut p = ReflectionProbe::new(e, at.position, size);
        p.fade = fade;
        p.resolution = v.resolution;
        p.mode = match v.mode {
            ReflectionProbeMode::Baked => ProbeMode::Baked,
            ReflectionProbeMode::Realtime => ProbeMode::Realtime,
        };
        p.interval = v.interval_ticks.max(1);
        p.intensity = v.intensity.max(0.0);
        probes.push(p);
    }
    let eye = rs.scene.frame.camera.position;
    og_core::sort::sort_by(&mut probes, |a, b| {
        a.position
            .distance_squared(eye)
            .total_cmp(&b.position.distance_squared(eye))
            .then(a.id.cmp(&b.id))
    });
    probes.truncate(MAX_REFLECTION_PROBES);
    rs.scene.lighting.reflection_probes = probes;
}

/// The lazy record that relights a LightVolume at run time: a world that
/// never names it keeps its type table and hash.
pub(crate) const REALTIME_LIGHT_VOLUME: &str = "og.RealtimeLightVolume";

/// og.RealtimeLightVolume rows, each with the og.LightVolume of its entity
/// (its spacing) and the og.Volume there (its box and fade; none: no box,
/// so nothing), into `rs.scene.light_volumes`: the nearest the
/// camera first, at most [`og_render::PROBE_VOLUMES`].
pub(crate) fn extract_light_volumes(world: &World, rs: &mut RenderState, h: &mut Hierarchy) {
    rs.scene.lighting.light_volumes.clear();
    let (Some(realtime), Some(volume)) = (
        rs.draw.realtime_volume.clone(),
        rs.draw.light_volume.clone(),
    ) else {
        return;
    };
    let live = crate::present::extract::rows(world, &realtime);
    if live.is_empty() {
        return;
    }
    let shape_layout = rs.draw.volume.clone();
    let boxes: BTreeMap<u64, record::LightVolume> = crate::present::extract::rows(world, &volume)
        .into_iter()
        .filter_map(|(e, v)| Some((e, record::LightVolume::from_json(&v).ok()?)))
        .collect();
    let mut volumes = Vec::new();
    for (e, v) in live {
        let Some(b) = boxes.get(&e) else {
            continue;
        };
        let Ok(v) = record::RealtimeLightVolume::from_json(&v) else {
            continue;
        };
        let (at, Some(shape)) = region(world, shape_layout.as_ref(), h, e) else {
            continue;
        };
        let (min, max) = shape.bounds(&at);
        volumes.push(og_render::RealtimeLightVolume {
            id: e,
            min,
            max,
            fade: shape.fade.max(0.0),
            spacing: b.spacing.max(0.1),
            updates_per_frame: v.updates_per_frame,
            intensity: v.intensity.max(0.0),
        });
    }
    let eye = rs.scene.frame.camera.position;
    let distance =
        |v: &og_render::RealtimeLightVolume| eye.clamp(v.min, v.max).distance_squared(eye);
    og_core::sort::sort_by(&mut volumes, |a, b| {
        distance(a).total_cmp(&distance(b)).then(a.id.cmp(&b.id))
    });
    volumes.truncate(og_render::PROBE_VOLUMES);
    rs.scene.lighting.light_volumes = volumes;
}

/// After extraction (the room complete): each baked probe's key, from its
/// room's content, and its bake, loaded from the bundle the first time the
/// key is seen (the probe samples it from this frame on; none: it draws its
/// own capture).
pub(crate) fn load_bakes(eng: &Engine, rs: &mut RenderState) {
    let wanted = apply_bakes(rs);
    load(eng, rs, wanted);
}

/// Each baked probe's key and the bake already tried under it (a capture
/// between frames uses these without loading); answers the probes whose
/// keys were never tried, with their keys.
pub(crate) fn apply_bakes(rs: &mut RenderState) -> Vec<(usize, String)> {
    rs.probe_bakes.rooms.clear();
    let mut wanted = Vec::new();
    for i in 0..rs.scene.lighting.reflection_probes.len() {
        let p = rs.scene.lighting.reflection_probes[i];
        if p.mode != ProbeMode::Baked {
            continue;
        }
        let room = rs.probe_bakes.content.room(&rs.scene, &p);
        // Keyed by its place in the world, wherever the frame is drawn from.
        let mut at = p;
        if rs.scene.frame.origin != glam::Vec3::ZERO {
            at.position += rs.scene.frame.origin;
        }
        let k = key(&at, room.hash);
        match rs.probe_bakes.tried.get(&k) {
            Some(&h) => rs.scene.lighting.reflection_probes[i].baked = h,
            None => wanted.push((i, k.clone())),
        }
        rs.probe_bakes.rooms.insert(p.id, (k, room));
    }
    wanted
}

fn load(eng: &Engine, rs: &mut RenderState, wanted: Vec<(usize, String)>) {
    for (i, k) in wanted {
        let size = rs.scene.lighting.reflection_probes[i].face_size();
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
        rs.scene.lighting.reflection_probes[i].baked = handle;
    }
}

/// The model assets whose meshes or materials a room draws (sorted), and
/// whether some of its meshes are not a model's (built by the game's code).
fn room_assets(
    rs: &RenderState,
    assets: &crate::world::assets::Assets,
    room: &og_render::ProbeRoom,
) -> (Vec<String>, bool) {
    let mut found = std::collections::BTreeSet::new();
    let mut covered = std::collections::BTreeSet::new();
    for (&model, parts) in &rs.models {
        let lods = rs.model_lods.get(&model);
        let meshes = parts.iter().map(|p| p.0).chain(
            lods.into_iter()
                .flat_map(|l| l.parts.iter().flatten().map(|x| x.0)),
        );
        let mut drawn = false;
        for m in meshes {
            if room.meshes.contains(&m) {
                covered.insert(m.index());
                drawn = true;
            }
        }
        drawn |= parts.iter().any(|p| room.materials.contains(&p.1));
        if drawn && let Some(e) = assets.entries.get((model & 0xffff_ffff) as usize) {
            found.insert(e.path.clone());
        }
    }
    let built = room.meshes.iter().any(|m| !covered.contains(&m.index()));
    (found.into_iter().collect(), built)
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
            let eng = crate::engine::engine(engine)?;
            let rs = eng.render.borrow();
            let backend = rs
                .backend
                .as_ref()
                .ok_or("no renderer: bake_probes reads the probes' captures back")?;
            let mut out = Vec::new();
            let assets = eng.assets.borrow();
            for p in &rs.scene.lighting.reflection_probes {
                let Some((k, room)) = rs.probe_bakes.rooms.get(&p.id) else {
                    continue;
                };
                let Some(read) = backend.read_reflection_probe(p.id) else {
                    continue;
                };
                let (size, levels) = read?;
                let (models, built) = room_assets(&rs, &assets, room);
                out.push(json!({
                    "key": k,
                    "entity": p.id,
                    "size": size,
                    "position": p.position.to_array(),
                    "box": p.size.to_array(),
                    "room": format!("{:016x}", room.hash),
                    // The model assets drawn in the room, and whether the
                    // game's code built some of it (og doctor's staleness).
                    "assets": models,
                    "built": built,
                    "levels": levels,
                }));
            }
            Ok(json!({ "kind": "probes", "probes": out }))
        },
    );
}
