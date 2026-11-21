//! What a decoded scene's presentation depends on: its dependency closure,
//! and the two questions asked of it.
//!
//! A scene's [`Closure`] is read once, when its blob decodes:
//!
//! - the models its entities place (`model` lines);
//! - the scenes that spawn with it: its sub-scenes that are not cells (a
//!   nested cell loads on its own, with its own closure), each with its
//!   own closure in turn;
//! - the prefabs its entities were placed from: blobs the spawn reads,
//!   whose models are the placing scene's own.
//!
//! The HLOD proxies over a scene are drawn for it and belong to it too
//! (the render state's trees, `crate::present::hlod::scene_ready`). Every
//! id resolves as the spawn resolves it ([`Assets::source`] from the
//! placing scene's bundle).
//!
//! One walk ([`walk`]) answers the questions asked of it:
//!
//! - [`Assets::closure_fetching`]: whether a file of it is still on its
//!   way (the hold gate's question, `crate::world::stream::cells::waiting`);
//! - [`ready`]: whether all of it is decoded, prepared, shown and done
//!   fading (`StreamStatus.ready`'s, `crate::world::stream::cell_groups`);
//! - [`prepared`]: whether a scene not in the world yet would be drawn
//!   whole the frame it is placed (`ScenePreload.prepared`'s), after
//!   [`load`] loaded it ahead (og.scene.preload, a timeline's `scene`
//!   track: `crate::world::scenes`).

use super::{Assets, LoadedScene, Payload, Status, handle};
use crate::engine::Engine;
use crate::present::extract::RenderState;
use og_bundle::runtime::SceneDesc;
use std::collections::BTreeSet;

/// A scene's direct dependencies; [`walk`] follows the nested scenes'.
#[derive(Clone, Debug, Default)]
pub(crate) struct Closure {
    /// The models its entities place.
    pub models: Vec<String>,
    /// Its sub-scenes that are not cells: they spawn with it.
    pub scenes: Vec<String>,
    /// The prefabs its entities came from (not itself).
    pub prefabs: Vec<String>,
}

impl Closure {
    pub(crate) fn of(desc: &SceneDesc) -> Closure {
        let cells: BTreeSet<usize> = desc
            .components
            .iter()
            .filter(|t| crate::world::stream::cells::is_cell(&t.name))
            .flat_map(|t| t.rows.iter().map(|r| r.0))
            .collect();
        let mut c = Closure::default();
        for (row, e) in desc.entities.iter().enumerate() {
            if let Some(model) = &e.instance
                && !c.models.contains(model)
            {
                c.models.push(model.clone());
            }
            if let Some(scene) = &e.subscene
                && !cells.contains(&row)
                && *scene != desc.id
                && !c.scenes.contains(scene)
            {
                c.scenes.push(scene.clone());
            }
        }
        c.prefabs = desc
            .prefabs
            .iter()
            .filter(|p| **p != desc.id)
            .cloned()
            .collect();
        c
    }
}

/// One part of a closure, as [`walk`] meets it.
pub(crate) enum Part<'a> {
    /// A decoded scene of it, the root first.
    Scene(&'a LoadedScene),
    /// A model a scene of it places, by the placing scene's bundle.
    Model { owner: usize, id: &'a str },
    /// A nested scene or a prefab not decoded yet.
    Waiting { owner: usize, id: &'a str },
}

/// Visits every part of `root`'s closure, each once, until `visit`
/// answers false (then false).
pub(crate) fn walk<'a>(
    a: &'a Assets,
    root: &'a LoadedScene,
    mut visit: impl FnMut(Part<'a>) -> bool,
) -> bool {
    let mut seen = BTreeSet::new();
    let mut scenes = vec![root];
    while let Some(scene) = scenes.pop() {
        if !visit(Part::Scene(scene)) {
            return false;
        }
        let owner = scene.bundle;
        let c = &scene.closure;
        for id in &c.models {
            if seen.insert(("gltf", owner, id.as_str())) && !visit(Part::Model { owner, id }) {
                return false;
            }
        }
        let nested = c.scenes.iter().map(|id| (id, true));
        for (id, spawns) in nested.chain(c.prefabs.iter().map(|id| (id, false))) {
            if !seen.insert(("scene", owner, id.as_str())) {
                continue;
            }
            let decoded = a.index_of_kind("scene", id, Some(owner)).and_then(|i| {
                match &a.entries[i].payload {
                    Payload::Scene(s) => Some(s.as_ref()),
                    _ => None,
                }
            });
            match decoded {
                Some(s) if spawns => scenes.push(s),
                Some(_) => {}
                None => {
                    if !visit(Part::Waiting { owner, id }) {
                        return false;
                    }
                }
            }
        }
    }
    true
}

impl Assets {
    /// The load index of `kind` (`gltf`, `scene`) `id`, as placed from
    /// bundle `owner`.
    pub(crate) fn index_of_kind(
        &self,
        kind: &str,
        id: &str,
        owner: Option<usize>,
    ) -> Option<usize> {
        let bundle = self.source(id, owner).ok()?;
        self.by_path.get(&format!("{kind}:{bundle}:{id}")).copied()
    }

    /// Whether a file of `scene`'s closure is still on its way, outside
    /// streaming groups (the web reads a runtime bundle's on demand): a
    /// model not loaded yet (its model or proxy blob, its textures), or a
    /// nested scene's or a prefab's blob not decoded yet. Asking starts
    /// every such fetch at once, so a cell holds its tick once for them
    /// all (`crate::world::stream::cells::waiting`). A nested scene's
    /// models are asked for once its blob has decoded.
    pub(crate) fn closure_fetching(&self, scene: &LoadedScene) -> bool {
        let Some(h) = crate::engine::hooks() else {
            return false;
        };
        let mut fetching = false;
        let mut ask = |id: &str, owner: usize, kind: &str| {
            let Ok((bundle, dir)) = self.bundle_from(id, Some(owner)) else {
                return;
            };
            let Ok(asset) = bundle.asset(id) else {
                return;
            };
            for out in asset.outputs.iter().filter(|o| {
                o.group.is_none()
                    && (o.ty == kind || (o.ty == "og.Texture" && !o.id.ends_with("#mips")))
            }) {
                fetching |= h.fetching(&format!("{dir}/{}", out.file));
            }
        };
        walk(self, scene, |part| {
            match part {
                Part::Scene(_) => {}
                Part::Model { owner, id } => {
                    if self.index_of_kind("gltf", id, Some(owner)).is_none() {
                        ask(id, owner, "og.Model");
                    }
                }
                Part::Waiting { owner, id } => {
                    if self.group_from(id, Some(owner)).is_none() {
                        ask(id, owner, "og.Scene");
                    }
                }
            }
            true
        });
        fetching
    }
}

/// Whether model or proxy `h` is drawn whole at tick `now`: not hidden
/// for its textures, not fading in (`crate::present::reveal`).
pub(crate) fn presented(rs: &RenderState, h: u64, now: f64) -> bool {
    !rs.decoded.hidden_models.contains_key(&h) && !rs.reveal.pending(h, now)
}

/// Whether all of `scene`'s closure is presented at tick `now`: every
/// scene of it decoded, with its HLOD proxies shown; every model loaded,
/// its streaming group resident and upgraded, and presented. Presentation
/// only, never what a tick does.
pub(crate) fn ready(a: &Assets, rs: &RenderState, scene: &LoadedScene, now: f64) -> bool {
    walk(a, scene, |part| match part {
        Part::Scene(s) => crate::present::hlod::scene_ready(rs, s.bundle, &s.id, now),
        Part::Model { owner, id } => {
            if let Some(group) = a.group_from(id, Some(owner))
                && !a.stream.upgraded(&group)
            {
                return false;
            }
            a.index_of_kind("gltf", id, Some(owner))
                .is_some_and(|i| presented(rs, handle(i), now))
        }
        Part::Waiting { .. } => false,
    })
}

/// What [`prepared`] finds of a scene not in the world yet.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Prepared {
    /// Placed now, it is drawn whole.
    Yes,
    /// Work on it is under way and ends by itself: a file fetched, a
    /// streaming group read or put in place of its stand-ins, textures
    /// prepared, a pipeline compiled.
    Busy,
    /// It is not, and nothing under way changes that (why).
    No(String),
}

/// Loads what the closure of the scene at load index `root` spawns with,
/// as its spawn would and in its order: each model (its meshes, materials
/// and textures join the renderer, and its pipelines start on the compile
/// workers without a frame drawing it, `og_render::SceneObjects::prepare`),
/// each nested scene and prefab, and theirs in turn. Nothing spawns.
///
/// Only what the boot bundle holds is loaded: its files are on every host
/// when this is called, so the asset handles made here are the same on
/// every host. A runtime bundle's files are asked for instead (the web
/// reads them on demand) and load when the scene is placed, at a tick
/// every host holds for them. Answers the loads refused (a missing or
/// broken file), which placing the scene reports too.
pub(crate) fn load(eng: &Engine, root: usize) -> Vec<String> {
    let mut failed = Vec::new();
    let mut tried = BTreeSet::new();
    loop {
        let (models, scenes) = {
            let a = eng.assets.borrow();
            let Some(Payload::Scene(scene)) = a.entries.get(root).map(|e| &e.payload) else {
                return failed;
            };
            // Asking starts the fetch of every file still on its way.
            a.closure_fetching(scene);
            let boot = |owner: usize, id: &str| a.source(id, Some(owner)).ok() == a.default_bundle;
            let (mut models, mut scenes) = (Vec::new(), Vec::new());
            walk(&a, scene, |part| {
                match part {
                    Part::Scene(_) => {}
                    Part::Model { owner, id } => {
                        if boot(owner, id) && a.index_of_kind("gltf", id, Some(owner)).is_none() {
                            models.push((owner, id.to_string()));
                        }
                    }
                    Part::Waiting { owner, id } => {
                        if boot(owner, id) {
                            scenes.push((owner, id.to_string()));
                        }
                    }
                }
                true
            });
            (models, scenes)
        };
        // A nested scene decoded now names models of its own: walk again.
        let mut again = false;
        for (owner, id) in scenes {
            if !tried.insert(("scene", owner, id.clone())) {
                continue;
            }
            let loaded = super::load_scene_from(eng, &id, Some(owner));
            let status = loaded.map(|i| eng.assets.borrow().entries[i].status.clone());
            match status {
                Ok(Status::Failed(e)) | Err(e) => failed.push(format!("{id}: {e}")),
                Ok(_) => again = true,
            }
        }
        for (owner, id) in models {
            if !tried.insert(("gltf", owner, id.clone())) {
                continue;
            }
            match super::load_gltf_from(eng, &id, Some(owner)) {
                Ok(i) => {
                    // A streamed model's pipelines started as it decoded.
                    if eng.assets.borrow().group_from(&id, Some(owner)).is_some() {
                        continue;
                    }
                    let mut rs = eng.render.borrow_mut();
                    let RenderState { models, scene, .. } = &mut *rs;
                    if let Some(parts) = models.get(&handle(i)) {
                        let pairs = parts.iter().map(|&(mesh, material, _)| (mesh, material));
                        scene.objects.prepare.extend(pairs);
                    }
                }
                Err(e) => failed.push(format!("{id}: {e}")),
            }
        }
        if !again {
            return failed;
        }
    }
}

/// Whether `scene`, not in the world yet, would be drawn whole the frame
/// it is placed, after [`load`]: every scene and prefab of its closure
/// decoded, every model loaded with its textures in and its streaming
/// group in place of its stand-ins, and no pipeline started ahead of it
/// still to be made. A runtime bundle's part is prepared once its file has
/// arrived (it loads when the scene is placed). Presentation only, never
/// what a tick does.
pub(crate) fn prepared(a: &Assets, rs: &RenderState, scene: &LoadedScene) -> Prepared {
    // Pipelines: the pairs loaded ahead stay listed until a frame finds
    // them made (`crate::engine::tick`). No frame is drawn where only
    // captures are, and a capture waits for what it draws: none waits there.
    let compiling = !rs.scene.objects.prepare.is_empty() && !rs.display.capture_only;
    let mut busy = a.closure_fetching(scene) || (rs.backend.is_some() && compiling);
    let mut no = None;
    let boot = |owner: usize, id: &str| a.source(id, Some(owner)).ok() == a.default_bundle;
    walk(a, scene, |part| {
        match part {
            Part::Scene(_) => {}
            Part::Model { owner, id } => {
                let group = a
                    .group_from(id, Some(owner))
                    .filter(|g| !a.stream.upgraded(g));
                if let Some(g) = &group {
                    match a.stream.arriving(g) {
                        true => busy = true,
                        false => no = Some(format!("streaming group '{g}' is not loaded")),
                    }
                }
                match a.index_of_kind("gltf", id, Some(owner)) {
                    // Hidden for its group's maps (above), or for textures
                    // being prepared for this device.
                    Some(i) => {
                        busy |=
                            group.is_none() && rs.decoded.hidden_models.contains_key(&handle(i));
                    }
                    None if boot(owner, id) => no = Some(format!("{id} is not loaded")),
                    None => {}
                }
            }
            Part::Waiting { owner, id } => {
                if boot(owner, id) {
                    no = Some(format!("{id} is not loaded"));
                }
            }
        }
        no.is_none()
    });
    match (no, busy) {
        (Some(why), _) => Prepared::No(why),
        (None, true) => Prepared::Busy,
        (None, false) => Prepared::Yes,
    }
}
