//! The IDL's `anim` module and `og.load.animation` over og-anim.
//!
//! Every world gets og-anim's `og.anim` system ([`install`]) after the game's
//! systems and before the UI's end and physics, so the players, graph
//! parameters and IK targets game code set this tick are posed this tick,
//! and root motion reaches og.Transform before the physics step reads it.
//! The components are the IDL's records: the world registers their layout
//! from `api.ogt` first and og-anim adopts it (`tests/modules.rs` keeps the
//! two equal); `og.AnimationEvent` carries a `str64` name, so og-anim
//! registers it and the runtime reads it through og-anim's layout.
//!
//! Assets (skeletons, clips, graphs) are not world state: they live per
//! engine in [`AnimAssets`] under handles minted here (generation 1, as the
//! load module's), and a hot reload registers them again in the new world's
//! system. A model loaded twice answers the same handles.
//!
//! The joint palettes and blendshape weights og-anim computes each tick are
//! drawn by extraction (`super::fx::posed_model`): an entity with og.Model
//! and og.Skeleton draws its skinned primitives as og-render skinned
//! instances under its palette, its morph targets with its og.Blendshapes
//! weights; the skinned copies of the primitives are made from the
//! `#anim` blob's streams the first time such an entity is drawn.

use super::engine::{Engine, engine};
use super::{ok, parse, refuse};
use og_anim::{AnimationSystem, ClipAsset, GraphAsset, HumanoidMap, RigAsset, SkeletonAsset};
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// The record og-anim registers itself (its `name` is a `str64`).
pub(crate) const REGISTERED_BY_ANIM: &[&str] = &["og.AnimationEvent"];

/// An engine's animation assets, by handle.
#[derive(Clone, Default)]
pub(crate) struct AnimAssets {
    next: u64,
    skeletons: BTreeMap<u64, SkeletonAsset>,
    clips: BTreeMap<u64, ClipAsset>,
    graphs: BTreeMap<u64, GraphAsset>,
    /// What og.load.animation answered per model id, so a second load answers the same handles.
    loaded: BTreeMap<String, Value>,
    /// Humanoid maps from og.load.rig, and their handles by asset id.
    rigs: BTreeMap<u64, RigAsset>,
    rig_ids: BTreeMap<String, u64>,
    /// What og.anim.retarget answered per (clip, source, target, source rig,
    /// target rig, foot IK), so asking again answers the same clip.
    retargeted: BTreeMap<(u64, u64, u64, u64, u64, bool), Value>,
}

impl AnimAssets {
    fn mint(&mut self) -> u64 {
        self.next += 1;
        self.next | (1u64 << 32)
    }

    /// Registers every asset with `system` (a reloaded world's).
    pub(crate) fn replay(&self, system: &AnimationSystem) {
        for (h, s) in &self.skeletons {
            let _ = system.add_skeleton(*h, s.clone());
        }
        for (h, c) in &self.clips {
            let _ = system.add_clip(*h, c.clone());
        }
        for (h, g) in &self.graphs {
            let _ = system.add_graph(*h, g.clone());
        }
    }
}

/// Registers the records og-anim owns the layout of (before the game's
/// systems, which may read `og.AnimationEvent`), as the runtime reads them.
pub(crate) fn register_records(
    world: &mut og_ecs::World,
) -> Result<Vec<super::codec::Layout>, String> {
    world
        .register::<og_anim::AnimationEvent>()
        .map_err(|e| e.reason().to_string())?;
    REGISTERED_BY_ANIM
        .iter()
        .map(|name| {
            let id = world.id_by_name(name).map_err(|e| e.reason().to_string())?;
            let schema = world.schema(id).map_err(|e| e.reason().to_string())?;
            Ok(super::codec::Layout::from_schema(
                schema,
                super::codec::RecordKind::Event,
            ))
        })
        .collect()
}

/// Puts og-anim on a world being assembled: after the game's systems.
pub(crate) fn install(world: &mut og_ecs::World) -> Result<AnimationSystem, String> {
    og_anim::install(world).map_err(|e| e.reason().to_string())
}

fn system(eng: &Engine) -> Result<AnimationSystem, String> {
    eng.state
        .borrow()
        .anim
        .clone()
        .ok_or_else(|| "this engine has no animation system".to_string())
}

fn wire(r: Result<Value, String>) -> String {
    match r {
        Ok(v) => ok(v),
        Err(e) => refuse(e),
    }
}

fn load_animation(eng: &Engine, path: &str) -> Result<Value, String> {
    let id = og_cook::runtime::anim_id(path);
    if let Some(v) = eng.state.borrow().anim_assets.loaded.get(&id) {
        return Ok(v.clone());
    }
    let cooked = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&id).map_err(|e| {
            if e.contains("not in the cooked bundle") {
                format!("{e} (the cook writes `#anim` only for a glTF with skins, clips or morph targets)")
            } else {
                e
            }
        })?;
        bundle
            .animation(&id, bytes.bytes())
            .map_err(|e| e.to_string())?
    };
    // The skin and morph streams, for drawing the model posed (og-render's
    // skinned path; the copies are made when an entity with og.Skeleton
    // and og.Model of it is first drawn).
    eng.render.borrow_mut().fx.skins.insert(
        og_cook::runtime::asset_id(path)
            .trim_end_matches("#anim")
            .to_string(),
        super::fx::SkinnedModel {
            streams: cooked.skin_streams.clone(),
            morphs: cooked.morphs.clone(),
            meshes: None,
        },
    );
    let sys = system(eng)?;
    let mut st = eng.state.borrow_mut();
    let assets = &mut st.anim_assets;
    let skeleton = assets.mint();
    sys.add_skeleton(skeleton, cooked.skeleton.clone())
        .map_err(|e| format!("{id}: {}", e.reason()))?;
    assets.skeletons.insert(skeleton, cooked.skeleton.clone());
    let mut clips = Vec::new();
    for c in &cooked.clips {
        let h = assets.mint();
        sys.add_clip(h, c.clone())
            .map_err(|e| format!("{id}: clip {}: {}", c.name, e.reason()))?;
        assets.clips.insert(h, c.clone());
        clips.push(json!({
            "clip": h as f64,
            "name": c.name,
            "duration": c.duration,
            "markers": c.events.iter().map(|e| e.name.clone()).collect::<Vec<_>>(),
        }));
    }
    let answer = json!({
        "skeleton": skeleton as f64,
        "clips": clips,
        "nodes": cooked.skeleton.nodes.iter().map(|n| n.name.clone()).collect::<Vec<_>>(),
        "skins": cooked.skeleton.skins.iter().map(|s| s.joints.len()).collect::<Vec<_>>(),
        "morphTargets": cooked.morphs.iter().map(|m| m.names.len()).sum::<usize>(),
    });
    assets.loaded.insert(id, answer.clone());
    Ok(answer)
}

pub fn og_load_animation(world: u32, path: String) -> String {
    wire(engine(world).and_then(|eng| load_animation(&eng, &path)))
}

fn graph(eng: &Engine, desc: &Value) -> Result<Value, String> {
    let text = desc["text"]
        .as_str()
        .ok_or("desc.text is the graph's .ogt text (`type animgraph`); it is missing")?;
    let mut by_name = BTreeMap::new();
    for c in desc["clips"].as_array().into_iter().flatten() {
        let (Some(name), Some(h)) = (c["name"].as_str(), c["clip"].as_f64()) else {
            return Err(
                "desc.clips holds ClipInfo rows ({clip, name}), as og.load.animation answers them"
                    .into(),
            );
        };
        by_name.insert(name.to_string(), h as u64);
    }
    let sys = system(eng)?;
    let known = eng
        .state
        .borrow()
        .anim_assets
        .clips
        .keys()
        .copied()
        .collect::<Vec<_>>();
    let missing = std::cell::RefCell::new(Vec::new());
    let parsed = GraphAsset::parse(text, &|name| {
        let h = by_name.get(name).copied().filter(|h| known.contains(h));
        if h.is_none() {
            missing.borrow_mut().push(name.to_string());
        }
        h
    });
    let graph = match parsed {
        Ok(g) => g,
        Err(e) => {
            let missing = missing.into_inner();
            return Err(if missing.is_empty() {
                format!("the graph: {}", e.reason())
            } else {
                format!(
                    "the graph names clip(s) {} that desc.clips lacks; pass og.load.animation's clips ({})",
                    missing.join(", "),
                    e.reason()
                )
            });
        }
    };
    let mut st = eng.state.borrow_mut();
    let h = st.anim_assets.mint();
    sys.add_graph(h, graph.clone())
        .map_err(|e| format!("the graph: {}", e.reason()))?;
    st.anim_assets.graphs.insert(h, graph);
    Ok(json!(h as f64))
}

fn load_rig(eng: &Engine, path: &str) -> Result<Value, String> {
    let id = og_cook::runtime::asset_id(path).to_string();
    if let Some(&h) = eng.state.borrow().anim_assets.rig_ids.get(&id) {
        return Ok(json!(h as f64));
    }
    let rig = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&id)?;
        bundle.rig(&id, bytes.bytes()).map_err(|e| e.to_string())?
    };
    let mut st = eng.state.borrow_mut();
    let assets = &mut st.anim_assets;
    let h = assets.mint();
    assets.rigs.insert(h, rig);
    assets.rig_ids.insert(id, h);
    Ok(json!(h as f64))
}

pub fn og_load_rig(world: u32, path: String) -> String {
    wire(engine(world).and_then(|eng| load_rig(&eng, &path)))
}

/// A handle field of a desc (0 when absent).
fn handle(desc: &Value, key: &str) -> u64 {
    desc[key].as_f64().map_or(0, |h| h as u64)
}

/// The humanoid map of skeleton `h` by rig `rig` (0: detected).
fn humanoid_of(
    eng: &Engine,
    sys: &AnimationSystem,
    h: u64,
    rig: u64,
    what: &str,
) -> Result<(std::sync::Arc<SkeletonAsset>, HumanoidMap), String> {
    let skeleton = sys
        .skeleton(h)
        .ok_or_else(|| format!("desc.{what} {h} is not a loaded skeleton (og.load.animation)"))?;
    let map = match rig {
        0 => HumanoidMap::detect(&skeleton),
        r => {
            let st = eng.state.borrow();
            let rig = st
                .anim_assets
                .rigs
                .get(&r)
                .ok_or_else(|| format!("{r} is not a loaded rig (og.load.rig)"))?;
            rig.map(&skeleton)
                .map_err(|e| format!("the {what}'s rig: {}", e.reason()))?
        }
    };
    Ok((skeleton, map))
}

fn humanoid(eng: &Engine, desc: &Value) -> Result<Value, String> {
    let sys = system(eng)?;
    let (skeleton, map) = humanoid_of(
        eng,
        &sys,
        handle(desc, "skeleton"),
        handle(desc, "rig"),
        "skeleton",
    )?;
    Ok(json!({
        "scheme": map.scheme,
        "bones": map
            .bones(&skeleton)
            .into_iter()
            .map(|(bone, node)| json!({ "bone": bone, "node": node }))
            .collect::<Vec<_>>(),
    }))
}

pub fn og_anim_humanoid(world: u32, desc: Option<String>) -> String {
    wire(engine(world).and_then(|eng| {
        let desc = parse(desc.as_deref())?;
        humanoid(&eng, &desc)
    }))
}

fn retarget(eng: &Engine, desc: &Value) -> Result<Value, String> {
    let sys = system(eng)?;
    let clip_h = handle(desc, "clip");
    let (source_h, target_h) = (handle(desc, "source"), handle(desc, "target"));
    let (source_rig, target_rig) = (handle(desc, "sourceRig"), handle(desc, "targetRig"));
    let foot_ik = desc["footIk"].as_bool().unwrap_or(true);
    let key = (clip_h, source_h, target_h, source_rig, target_rig, foot_ik);
    if let Some(v) = eng.state.borrow().anim_assets.retargeted.get(&key) {
        return Ok(v.clone());
    }
    let clip = sys
        .clip(clip_h)
        .ok_or_else(|| format!("desc.clip {clip_h} is not a loaded clip (og.load.animation)"))?;
    let (source, source_map) = humanoid_of(eng, &sys, source_h, source_rig, "source")?;
    let (target, target_map) = humanoid_of(eng, &sys, target_h, target_rig, "target")?;
    let out = og_anim::retarget(
        &source,
        &source_map,
        &target,
        &target_map,
        &clip,
        &og_anim::RetargetOptions { foot_ik },
    )
    .map_err(|e| format!("og.anim.retarget: {}", e.reason()))?;
    let mut st = eng.state.borrow_mut();
    let assets = &mut st.anim_assets;
    let h = assets.mint();
    sys.add_clip(h, out.clip.clone())
        .map_err(|e| format!("og.anim.retarget: {}", e.reason()))?;
    let answer = json!({
        "clip": h as f64,
        "name": out.clip.name,
        "duration": out.clip.duration,
        "markers": out.clip.events.iter().map(|e| e.name.clone()).collect::<Vec<_>>(),
    });
    assets.clips.insert(h, out.clip);
    assets.retargeted.insert(key, answer.clone());
    Ok(answer)
}

pub fn og_anim_retarget(world: u32, desc: Option<String>) -> String {
    wire(engine(world).and_then(|eng| {
        let desc = parse(desc.as_deref())?;
        retarget(&eng, &desc)
    }))
}

pub fn og_anim_graph(world: u32, desc: Option<String>) -> String {
    wire(engine(world).and_then(|eng| {
        let desc = parse(desc.as_deref())?;
        graph(&eng, &desc)
    }))
}
