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
//! One walk ([`walk`]) answers both questions:
//!
//! - [`Assets::closure_fetching`]: whether a file of it is still on its
//!   way (the hold gate's question, `crate::world::stream::cells::waiting`);
//! - [`ready`]: whether all of it is decoded, prepared, shown and done
//!   fading (`StreamStatus.ready`'s, `crate::world::stream::cell_groups`).

use super::{Assets, LoadedScene, Payload, handle};
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
