//! HLOD at run time (docs/design.md §10.6): a scene's distant static
//! models drawn as the proxies its cook merged them into.
//!
//! A scene with an `hlod { }` block cooks to a tree of proxies beside it
//! (`og_bundle::hlod`), each an ordinary model. When the scene is spawned
//! the tree and its proxies are loaded ([`scene_spawned`]); every frame
//! [`draw`] walks each tree from its roots and draws a node's proxy in
//! place of every model under it when the proxy's error, projected at the
//! node's distance, stays under a pixel: the rule a model's own levels
//! are drawn by (`og_render::lod::select`, with its hysteresis and
//! `RenderSettings.lodBias`). A node nearer than that hands the choice to
//! its children, and its own models draw themselves.
//!
//! **Cross-fade.** Where a node's choice turns, the proxy and what was
//! drawn in its place (its children's proxies or the models) are both
//! drawn for [`crate::present::reveal::SECONDS`], dithered against each
//! other: the side fading in draws a share of the pixels, the side fading
//! out exactly the others (`og_render::InstanceFx::fade_out`), so no
//! pixel is drawn twice or left empty, and a temporal resolve sees one
//! surface turn into the other rather than a pop (the dithered LOD
//! transition of Unity's LOD cross-fade and Unreal's dithered LOD
//! transition). A turn back during the fade reverses it from where it
//! is. Only a node whose models are all as the cook read it fades: one
//! changed, a cell that is not loaded, or a proxy still fading in after
//! its maps arrived switches at once, as before.
//!
//! **Presentation only.** The models stay in the world, simulated,
//! collided with and found by queries as before; only which instances the
//! frame holds changes. A proxy stands in for its models only while each
//! is as the cook read it: alive, visible, at its cooked place, with its
//! own model and look. One destroyed, hidden, moved, tinted or flashing
//! house returns its node, and every node above it, to their models.
//!
//! **Baked lighting.** `og bake lightmaps` lights a scene's proxies by
//! its static models' bake and writes each into the scene's lightmap as an
//! entry of its own asset. When a tree loads, every proxy whose entry
//! matches its geometry gets its lightmapped parts ([`Lit`]), drawn while
//! RenderSettings.bakedLighting is on, as the models' are; a stale bake
//! leaves the proxies unbaked, as it leaves the models. A cell's proxies
//! are lit while the cell is not loaded too.
//!
//! **World cells.** A cell's sub-scene has its own tree, kept in its declaring
//! bundle. While the cell is unloaded its proxies draw at the cell's pose
//! with nothing behind them, so the far world is there before it streams:
//! per branch the finest proxy that still shows every model of it.
//! (A `persist` cell's kept changes are not in its proxies.) Once it has
//! loaded, a model hidden while its textures are prepared
//! (`crate::world::assets::Decoded::hidden_models`) is still shown by its
//! node's proxy, at any distance, until it can draw itself.

use crate::engine::Engine;
use crate::host::wire::record;
use crate::present::extract::{Hierarchy, Parts, RenderState};
use crate::world::assets::LoadedScene;
use crate::world::assets::{Placement, handle};
use glam::{Mat4, Vec3};
use og_bundle::hlod::Hlod;
use og_core::IdMap;
use og_ecs::{Entity, World};
use std::collections::BTreeMap;
use std::sync::Arc;

/// A cooked tree with its proxies loaded.
struct Tree {
    hlod: Hlod,
    /// Per node, its proxy model's asset handle.
    proxies: Vec<u64>,
}

/// One place a tree draws: a spawned scene, or a cell's sub-scene.
struct Place {
    scene: String,
    tree: Arc<Tree>,
    /// What the scene's space hangs under: the entity it was spawned
    /// under, or its cell (none: the world).
    root: Option<u64>,
    cell: bool,
    /// Per member of the tree, its entity and the model it was spawned
    /// with; empty for a cell that has not loaded.
    members: Vec<(u64, u64)>,
    /// Per node, whether its proxy drew last frame (the hysteresis); none
    /// for a node not asked.
    drew: Vec<Option<bool>>,
    /// Per node, the frame time in ticks its cross-fade towards `drew`
    /// started at; none when it is not fading.
    fades: Vec<Option<f64>>,
    /// Distinguishes its proxies' instances from every other place's.
    serial: u64,
}

/// A proxy drawn with its bake: its parts re-indexed by the lightmap's
/// vertices, and each new mesh's lightmap coordinates and pages.
pub(crate) struct Lit {
    pub parts: Parts,
    pub meshes: Vec<(usize, og_render::BakedMesh)>,
}

/// What the last frame drew through HLOD (the probe's `dump_render`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Drawn {
    /// The trees walked.
    pub scenes: u32,
    /// Proxies drawn, and how many of them for cells not loaded.
    pub proxies: u32,
    pub unloaded: u32,
    /// Of them, drawn with their models' bake.
    pub lit: u32,
    /// Models a proxy drew in place of.
    pub replaced: u32,
}

#[derive(Default)]
pub(crate) struct HlodState {
    /// By bundle and scene id: its tree, or none when it has none.
    trees: BTreeMap<(usize, String), Option<Arc<Tree>>>,
    /// Unshared model resources owned by trees, also across a hot reload.
    pub proxies: IdMap<u64, Vec<og_render::TextureHandle>>,
    /// The proxies lit by their models' bake, by handle.
    pub lit: IdMap<u64, Lit>,
    places: Vec<Place>,
    serial: u64,
    pub drawn: Drawn,
    /// The models the last frame's proxies drew in place of, by entity
    /// (the probe's `dump_entity`).
    pub replaced: IdMap<u64, ()>,
    /// The models cross-fading with a proxy in the frame being built: how
    /// much of each is drawn and whether it fades out
    /// (`og_render::InstanceFx::reveal`, `fade_out`). The frame's model
    /// rows take it.
    pub crossing: IdMap<u64, (f32, bool)>,
    /// Counts the changes to the places other than a frame's draw: a
    /// place added, or given its members.
    pub changes: u64,
    /// Whether the last draw kept every place and every node's choice as
    /// it found them: a frame with its rows, its view and [`inputs`] the
    /// same draws the same proxies again.
    pub settled: bool,
}

impl HlodState {
    /// The entities of the places a proxy was showing (`showing`: its
    /// handle is neither hidden nor fading in): where a cell's models
    /// start being drawn there, they were already shown, so they never
    /// fade in (`crate::present::reveal`).
    pub(crate) fn stood(&self, showing: impl Fn(u64) -> bool) -> IdMap<u64, ()> {
        let mut stood = IdMap::new();
        for place in self.places.iter().filter(|p| !p.members.is_empty()) {
            if place.tree.proxies.iter().any(|&h| showing(h)) {
                stood.extend(place.members.iter().map(|&(e, _)| (e, ())));
            }
        }
        stood
    }

    /// Of `shown` (the handles that stopped being hidden, ascending), the
    /// proxies with nothing drawn in their place before, each with the
    /// cell it stands for: none of its tree's proxies was showing, and its
    /// cell was not loaded or the models of its members were hidden too
    /// (`was_hidden`). Those fade in (`crate::present::reveal`); a proxy
    /// that takes over from what was drawn does not.
    pub(crate) fn fresh(
        &self,
        world: &World,
        shown: &[u64],
        was_hidden: impl Fn(u64) -> bool,
    ) -> Vec<(u64, Option<u64>)> {
        let mut fresh = Vec::new();
        if shown.is_empty() {
            return fresh;
        }
        for place in &self.places {
            let proxies = &place.tree.proxies;
            if !proxies.iter().any(|h| shown.binary_search(h).is_ok())
                || proxies.iter().any(|&h| !was_hidden(h))
            {
                continue;
            }
            let loaded = match (place.cell, place.root) {
                (true, Some(cell)) => cell_loaded(world, cell).unwrap_or(false),
                _ => true,
            };
            // A loaded cell draws its models, unless they were hidden.
            let models = &place.members;
            if loaded && (models.is_empty() || models.iter().any(|&(_, m)| !was_hidden(m))) {
                continue;
            }
            let cell = place.root.filter(|_| place.cell);
            let shown = proxies.iter().filter(|h| shown.binary_search(h).is_ok());
            fresh.extend(shown.map(|&h| (h, cell)));
        }
        fresh
    }
}

/// The tree of scene `id`, loaded with its proxies the first time.
fn tree(eng: &Engine, id: &str, owner: usize) -> Option<Arc<Tree>> {
    let key = (owner, id.to_string());
    if let Some(t) = eng.render.borrow().hlod.trees.get(&key) {
        return t.clone();
    }
    let hlod_id = og_bundle::hlod::hlod_id(id);
    let loaded = (|| -> Result<Option<Tree>, String> {
        let hlod = {
            let assets = eng.assets.borrow();
            let Ok((bundle, _)) = assets.bundle_from(&hlod_id, Some(owner)) else {
                return Ok(None);
            };
            if bundle.output(&hlod_id).is_err() {
                return Ok(None);
            }
            let (bytes, bundle) = assets.blob_from(&hlod_id, Some(owner))?;
            bundle
                .hlod(&hlod_id, bytes.bytes())
                .map_err(|e| e.to_string())?
        };
        let mut tree = Tree {
            hlod,
            proxies: Vec::new(),
        };
        for n in 0..tree.hlod.nodes.len() {
            let loaded =
                crate::world::assets::load_hlod_proxy(eng, &og_bundle::hlod::node_id(id, n), owner);
            match loaded {
                Ok(index) => tree.proxies.push(handle(index)),
                Err(e) => {
                    release_tree(&mut eng.render.borrow_mut(), &tree);
                    return Err(e);
                }
            }
        }
        light(eng, id, owner, &tree);
        Ok(Some(tree))
    })();
    let tree = match loaded {
        Ok(t) => t.map(Arc::new),
        // Its models draw themselves at every distance.
        Err(e) => {
            eng.assets
                .borrow_mut()
                .warnings
                .push(format!("{id}: its HLOD is not drawn ({e})"));
            None
        }
    };
    eng.render.borrow_mut().hlod.trees.insert(key, tree.clone());
    tree
}

/// Gives the proxies of `tree` (scene `scene` of bundle `owner`) their
/// lightmapped parts from the scene's bake, when it has one that matches
/// the scene and the proxy (`og bake lightmaps` writes an entry per proxy,
/// keyed by its asset).
fn light(eng: &Engine, scene: &str, owner: usize, tree: &Tree) {
    let lightmap_id = og_bundle::lightmap::lightmap_id(scene);
    let lm = {
        let assets = eng.assets.borrow();
        let Ok((bundle, _)) = assets.bundle_from(&lightmap_id, Some(owner)) else {
            return;
        };
        if bundle.output(&lightmap_id).is_err() {
            return;
        }
        let Ok((bytes, bundle)) = assets.blob_from(&lightmap_id, Some(owner)) else {
            return;
        };
        let Ok(lm) = bundle.lightmap(&lightmap_id, bytes.bytes()) else {
            return;
        };
        let scene_hash = bundle.asset(scene).map(|a| a.source_hash).unwrap_or(0);
        if lm.file.scene_hash != scene_hash {
            return;
        }
        lm
    };
    let by_asset: BTreeMap<&str, &og_bundle::lightmap::BakedModel> = lm
        .file
        .models
        .iter()
        .map(|m| (m.asset.as_str(), m))
        .collect();
    let mut rs = eng.render.borrow_mut();
    // Each proxy's entry, when its geometry is what the bake read.
    let mut entries = Vec::new();
    for (n, &h) in tree.proxies.iter().enumerate() {
        let Some(&m) = by_asset.get(og_bundle::hlod::node_id(scene, n).as_str()) else {
            continue;
        };
        let Some(parts) = rs.models.get(&h) else {
            continue;
        };
        let geometry = og_bundle::lightmap::geometry_hash(
            parts.iter().filter_map(|&(mesh, _, _)| rs.scene.mesh(mesh)),
        );
        if geometry == m.hash && parts.len() == m.parts.len() {
            entries.push((h, parts.clone(), m));
        }
    }
    // The pages they read, alone: the models' stay the scene's own.
    let mut used: Vec<u32> = entries
        .iter()
        .flat_map(|(_, _, m)| {
            m.parts
                .iter()
                .flat_map(|p| p.uv.iter().map(|uv| uv[2] as u32))
        })
        .collect();
    used.sort_unstable();
    used.dedup();
    if used.is_empty() {
        return;
    }
    let to_f32 = og_bundle::lightmap::f16_to_f32;
    let pages = Arc::new(og_render::LightmapPages {
        width: lm.width,
        height: lm.height,
        pages: used
            .iter()
            .filter_map(|&p| lm.pages.get(p as usize))
            .map(|p| {
                p.as_chunks::<4>()
                    .0
                    .iter()
                    .map(|t| [to_f32(t[0]), to_f32(t[1]), to_f32(t[2]), to_f32(t[3])])
                    .collect()
            })
            .collect(),
    });
    for (h, parts, m) in entries {
        let mut lit = Lit {
            parts: Vec::with_capacity(parts.len()),
            meshes: Vec::new(),
        };
        for (&(mesh, material, local), bp) in parts.iter().zip(&m.parts) {
            let Some(data) = rs.scene.mesh(mesh) else {
                continue;
            };
            if data.vertices.len() != bp.vertices as usize
                || bp.xref.iter().any(|&x| x as usize >= data.vertices.len())
                || bp
                    .uv
                    .iter()
                    .any(|uv| used.binary_search(&(uv[2] as u32)).is_err())
            {
                continue;
            }
            let baked = og_render::MeshData {
                vertices: bp.xref.iter().map(|&x| data.vertices[x as usize]).collect(),
                indices: bp.indices.clone(),
            };
            let uvs = bp
                .uv
                .iter()
                .map(|uv| {
                    let page = used.binary_search(&(uv[2] as u32)).unwrap_or(0);
                    [uv[0], uv[1], page as f32]
                })
                .collect();
            let handle = rs.scene.add_mesh(baked);
            lit.meshes.push((
                handle.index(),
                og_render::BakedMesh {
                    pages: pages.clone(),
                    uvs: Arc::new(og_render::LightmapUvs { uvs }),
                },
            ));
            lit.parts.push((handle, material, local));
        }
        if lit.parts.len() == parts.len() {
            rs.hlod.lit.insert(h, lit);
        } else {
            for (mesh, _, _) in lit.parts {
                rs.scene.remove_mesh(mesh);
            }
        }
    }
}

/// A bundle's tree is still owned by a live placement, including an
/// unloaded cell whose proxies draw the distant world.
pub(crate) fn bundle_in_use(world: &World, rs: &RenderState, bundle: usize) -> bool {
    rs.hlod.trees.iter().any(|((owner, _), tree)| {
        *owner == bundle
            && tree.as_ref().is_some_and(|tree| {
                rs.hlod.places.iter().any(|p| {
                    Arc::ptr_eq(&p.tree, tree)
                        && match p.root {
                            Some(root) => world.is_alive(Entity::from_bits(root)),
                            None => p
                                .members
                                .iter()
                                .any(|&(e, _)| world.is_alive(Entity::from_bits(e))),
                        }
                })
            })
    })
}

/// A released bundle has no live placement. Collect its dead placements
/// before forgetting even the negative tree lookups.
pub(crate) fn release_bundle(world: &World, rs: &mut RenderState, bundle: usize) {
    collect(world, rs);
    rs.hlod.trees.retain(|(owner, _), _| *owner != bundle);
}

/// Dead placements no longer own their trees or their decoded proxies.
/// A cell unloading is not a dead placement: its distant proxies must stay.
pub(crate) fn collect(world: &World, rs: &mut RenderState) {
    let before = rs.hlod.places.len();
    rs.hlod.places.retain(|p| match p.root {
        Some(root) => world.is_alive(Entity::from_bits(root)),
        None => p
            .members
            .iter()
            .any(|&(e, _)| world.is_alive(Entity::from_bits(e))),
    });
    if rs.hlod.places.len() == before {
        return;
    }
    rs.hlod.changes += 1;
    rs.kept_models.forget();
    let mut trees = std::mem::take(&mut rs.hlod.trees);
    trees.retain(|_, tree| {
        let Some(tree) = tree else { return true };
        if Arc::strong_count(tree) != 1 {
            return true;
        }
        release_tree(rs, tree);
        false
    });
    rs.hlod.trees = trees;
}

fn release_tree(rs: &mut RenderState, tree: &Tree) {
    for &key in &tree.proxies {
        release_proxy(rs, key);
    }
}

pub(crate) fn release_proxy(rs: &mut RenderState, key: u64) {
    if let Some(lit) = rs.hlod.lit.remove(&key) {
        for (mesh, _, _) in lit.parts {
            crate::present::geometry::forget_mesh(rs, mesh);
            rs.scene.remove_mesh(mesh);
        }
    }
    if let Some(textures) = rs.hlod.proxies.remove(&key) {
        rs.decoded.hidden_models.remove(&key);
        rs.decoded.forget_textures(&textures);
        // Cooked HLOD nodes have one mesh level and no impostor. Unlike
        // ordinary models, their resources are not shared with the bundle.
        rs.models.remove(&key);
        rs.model_lods.remove(&key);
        rs.wind.tops.remove(&key);
        if let Some(model) = rs.fx.models.remove(&key) {
            for mesh in model.meshes {
                crate::present::geometry::forget_mesh(rs, mesh);
                rs.scene.remove_mesh(mesh);
            }
        }
        if let Some(materials) = rs.fx.model_materials.remove(&key) {
            for material in materials {
                rs.scene.remove_material(material);
            }
        }
        for texture in textures {
            rs.scene.remove_texture(texture);
            rs.decoded.ids.remove(&texture);
            rs.decoded.bundles.remove(&texture);
        }
    }
}

/// A cell was given sub-scene `scene`: its proxies draw while it is not
/// loaded.
pub(crate) fn cell_placed(eng: &Engine, cell: Entity, index: usize) {
    let (scene, bundle) = {
        let a = eng.assets.borrow();
        let e = &a.entries[index];
        (e.path.clone(), e.bundle.expect("cooked scene's bundle"))
    };
    let Some(tree) = tree(eng, &scene, bundle) else {
        return;
    };
    let mut rs = eng.render.borrow_mut();
    let state = &mut rs.hlod;
    state.serial += 1;
    state.changes += 1;
    state.places.push(Place {
        scene,
        drew: vec![None; tree.hlod.nodes.len()],
        fades: vec![None; tree.hlod.nodes.len()],
        tree,
        root: Some(cell.to_bits()),
        cell: true,
        members: Vec::new(),
        serial: state.serial,
    });
}

/// `scene` was just spawned as `spawned` (one entity per scene entity, in
/// order).
pub(crate) fn scene_spawned(
    eng: &Engine,
    scene: &LoadedScene,
    placement: Placement,
    spawned: &[Entity],
) {
    let Some(tree) = tree(eng, &scene.id, scene.bundle) else {
        return;
    };
    let mut members = Vec::with_capacity(tree.hlod.members.len());
    for m in &tree.hlod.members {
        let entity = spawned.get(m.entity).map_or(0, |e| e.to_bits());
        let asset = scene
            .entities
            .get(m.entity)
            .and_then(|e| e.instance.as_deref())
            .and_then(|model| {
                crate::world::assets::load_gltf_from(eng, model, Some(scene.bundle)).ok()
            })
            .map_or(0, handle);
        members.push((entity, asset));
    }
    let (root, cell) = match placement {
        Placement::Under(root) => (root.map(Entity::to_bits), false),
        Placement::Cell { cell, .. } => (Some(cell.to_bits()), true),
    };
    let alive = |e: u64| eng.world().is_ok_and(|w| w.is_alive(Entity::from_bits(e)));
    let mut rs = eng.render.borrow_mut();
    let state = &mut rs.hlod;
    state.changes += 1;
    // A cell's place is its own from when the cell was placed.
    if cell
        && let Some(p) = state
            .places
            .iter_mut()
            .find(|p| p.cell && p.root == root && p.scene == scene.id)
    {
        p.members = members;
        p.drew.fill(None);
        p.fades.fill(None);
        return;
    }
    // An earlier spawn of it here whose models are all gone draws nothing more.
    state.places.retain(|p| {
        p.cell || p.root != root || p.scene != scene.id || p.members.iter().any(|&(e, _)| alive(e))
    });
    state.serial += 1;
    state.places.push(Place {
        scene: scene.id.clone(),
        drew: vec![None; tree.hlod.nodes.len()],
        fades: vec![None; tree.hlod.nodes.len()],
        tree,
        root,
        cell,
        members,
        serial: state.serial,
    });
}

/// How far a member's matrix may be from the cook's and still be the
/// model the proxy was made of: a hundredth of a unit in its axes, a
/// centimetre and the float's own step in its place.
const MOVED: f32 = 0.01;

/// Whether `m` is `expected` within [`MOVED`].
fn same_place(m: &Mat4, expected: &Mat4) -> bool {
    let at = expected.w_axis.truncate();
    let step = MOVED + 8.0 * f32::EPSILON * at.abs().max_element();
    m.x_axis.abs_diff_eq(expected.x_axis, MOVED)
        && m.y_axis.abs_diff_eq(expected.y_axis, MOVED)
        && m.z_axis.abs_diff_eq(expected.z_axis, MOVED)
        && m.w_axis.truncate().abs_diff_eq(at, step)
}

/// The colour `RenderOptions.debugView lod` gives a proxy: blue, towards
/// violet the higher its node. It shows where the regular path draws the
/// proxy: the cluster path's level view writes its own colour over an
/// instance's flash (og-render's `meshlet.wgsl`), green for a proxy's one
/// level.
fn debug_color(level: u32) -> Vec3 {
    let t = (level as f32 / 4.0).min(1.0);
    Vec3::new(0.15 + 0.6 * t, 0.45 - 0.3 * t, 1.0)
}

/// What [`draw`] reads of one place besides the models' rows.
#[derive(Clone, PartialEq)]
pub(crate) struct PlaceIn {
    alive: bool,
    /// Whether its cell is loaded (true for a place of no cell).
    loaded: bool,
    /// Pending proxies must invalidate kept draws when their maps arrive.
    pending: Vec<u64>,
    /// Its proxies fading in and how much of each is drawn
    /// (`crate::present::reveal`): they are built again while they fade.
    fading: Vec<(u64, u32)>,
    /// Its root's world matrix, bit for bit.
    root: [u32; 16],
}

/// [`PlaceIn`] of every place.
pub(crate) fn inputs(world: &World, rs: &RenderState, h: &mut Hierarchy) -> Vec<PlaceIn> {
    let places = rs.hlod.places.iter().map(|p| {
        let alive = p.root.is_some_and(|r| world.is_alive(Entity::from_bits(r)));
        let loaded = match (p.cell, p.root) {
            (true, Some(cell)) => cell_loaded(world, cell).unwrap_or(false),
            _ => true,
        };
        let root = match p.root {
            Some(r) if alive => h.world_of(r),
            _ => Mat4::IDENTITY,
        };
        let root = root.to_cols_array().map(f32::to_bits);
        PlaceIn {
            alive,
            loaded,
            pending: p
                .tree
                .proxies
                .iter()
                .filter(|h| rs.decoded.hidden_models.contains_key(h))
                .copied()
                .collect(),
            fading: rs.reveal.proxy_bits(&p.tree.proxies),
            root,
        }
    });
    places.collect()
}

/// og.StreamCell's `loaded` of `cell`; none when it is no cell (despawned).
fn cell_loaded(world: &World, cell: u64) -> Option<bool> {
    let id = world.id_by_name(crate::world::stream::cells::CELL).ok()?;
    let schema = world.schema(id).ok()?;
    let field = schema.fields.iter().position(|f| f.name == "loaded")?;
    let at = og_core::reflect::Layout::of(schema).range(field).start;
    let bytes = world.get_bytes(Entity::from_bits(cell), id).ok()?;
    bytes.get(at).map(|&b| b != 0)
}

/// One og.Model row as [`draw`] reads it: its model, and whether it draws
/// its own look (no tint, flash or variant).
#[derive(Clone, Copy)]
pub(crate) struct Row {
    pub asset: u64,
    pub plain: bool,
}

/// How a member of a loaded place stands to what the cook read.
#[derive(Clone, Copy, PartialEq)]
enum Member {
    /// Drawn as the cook read it: the proxy may stand in for it.
    Intact,
    /// As the cook read it, but not drawn while its model's textures are
    /// prepared: only the proxy shows it.
    Waiting,
    /// Gone, hidden, moved or given a look of its own.
    Changed,
}

/// What [`draw`] reads of the frame.
pub(crate) struct View<'a> {
    pub models: &'a IdMap<u64, Parts>,
    /// The visible og.Model rows by entity.
    pub rows: &'a dyn Fn() -> IdMap<u64, Row>,
    pub view_height: f32,
    pub bias: f32,
    pub debug: bool,
    /// RenderSettings.bakedLighting: lit proxies draw their [`Lit`] parts.
    pub baked: bool,
}

/// A scene's proxy maps are presentation resources, just like its models'.
/// `now`: the world's tick.
pub(crate) fn scene_ready(rs: &RenderState, bundle: usize, scene: &str, now: f64) -> bool {
    rs.hlod
        .trees
        .get(&(bundle, scene.to_string()))
        .is_none_or(|tree| {
            tree.as_ref().is_none_or(|t| {
                // Shown, and no longer fading in (`crate::present::reveal`).
                t.proxies.iter().all(|h| {
                    !rs.decoded.hidden_models.contains_key(h) && !rs.reveal.pending(*h, now)
                })
            })
        })
}

/// Draws the proxies of this frame and answers the models they stand in
/// for, which the frame then leaves out.
pub(crate) fn draw(
    world: &World,
    rs: &mut RenderState,
    h: &mut Hierarchy,
    view: &View<'_>,
    keys: &mut Vec<(u64, u32)>,
) -> IdMap<u64, ()> {
    let mut replaced = IdMap::new();
    let mut crossed = IdMap::new();
    let mut drawn = Drawn::default();
    // A cross-fade's length in ticks, and how far one started at `at` is
    // at this frame's time.
    let now = rs.reveal.now();
    let dt = world.time().dt;
    let ticks = f64::from(crate::present::reveal::SECONDS) / f64::from(dt.max(1e-6));
    let progress = |at: f64| ((now - at) / ticks).clamp(0.0, 1.0) as f32;
    let mut places = std::mem::take(&mut rs.hlod.places);
    let lit = std::mem::take(&mut rs.hlod.lit);
    let before = places.len();
    // A cell that is gone takes its place with it.
    places.retain(|p| !p.cell || p.root.is_some_and(|c| world.is_alive(Entity::from_bits(c))));
    let mut settled = places.len() == before;
    let rows = (!places.is_empty()).then(|| (view.rows)());
    let model = rs
        .draw
        .model
        .as_ref()
        .and_then(|layout| Some((layout, world.id_by_name(&layout.schema).ok()?)));
    for place in &mut places {
        let Some(rows) = &rows else { break };
        let loaded = match (place.cell, place.root) {
            (true, Some(cell)) => cell_loaded(world, cell).unwrap_or(false),
            _ => true,
        };
        if !loaded {
            place.members.clear();
        }
        // A loaded cell whose models this place does not know (brought
        // back from what it kept) draws its models.
        if loaded && place.members.is_empty() {
            continue;
        }
        let root = match place.root {
            Some(r) if place.cell || world.is_alive(Entity::from_bits(r)) => h.world_of(r),
            Some(_) => continue,
            None => Mat4::IDENTITY,
        };
        drawn.scenes += 1;
        let scale = root.to_scale_rotation_translation().0.abs().max_element();
        let tree = place.tree.clone();
        let nodes = &tree.hlod.nodes;
        // Per member, how it stands to the cook's; asked once.
        let mut members: Vec<Option<Member>> = vec![None; place.members.len()];
        let mut member = |k: usize, h: &mut Hierarchy| -> Member {
            *members[k].get_or_insert_with(|| {
                let (e, asset) = place.members[k];
                let mut cooked =
                    || same_place(&h.world_of(e), &(root * tree.hlod.members[k].matrix));
                // The frame leaves out a row whose model's textures are
                // being prepared: it is waiting when it is visible with its
                // cooked model, at its cooked place.
                let waiting = || {
                    rs.decoded.hidden_models.contains_key(&asset)
                        && model.is_some_and(|(layout, id)| {
                            let row = world.get_bytes(Entity::from_bits(e), id).ok();
                            let row = row.and_then(|b| layout.read::<record::Model>(&b).ok());
                            row.is_some_and(|m| m.visible && m.asset == asset)
                        })
                };
                match rows.get(&e) {
                    Some(r) if r.plain && r.asset == asset && cooked() => Member::Intact,
                    None if waiting() && cooked() => Member::Waiting,
                    _ => Member::Changed,
                }
            })
        };
        // Each node with the fade an ancestor draws it under, if any.
        let mut stack: Vec<(usize, Option<(f32, bool)>)> =
            tree.hlod.children(None).map(|n| (n, None)).collect();
        stack.reverse();
        while let Some((n, crossing)) = stack.pop() {
            let node = &nodes[n];
            // With its bake when it has one and baked lighting is on.
            let handle = tree.proxies[n];
            let lit_parts = lit.get(&handle).filter(|_| view.baked).map(|l| &l.parts);
            let parts = lit_parts
                .or_else(|| view.models.get(&handle))
                .filter(|_| !rs.decoded.hidden_models.contains_key(&handle));
            // Every member under it: its children's, then its own.
            let under = node.members.0..node.members.1;
            let children: Vec<usize> = tree.hlod.children(Some(n)).collect();
            let own = children
                .iter()
                .map(|&c| nodes[c].members.1)
                .max()
                .unwrap_or(node.members.0);
            let per_unit = og_render::lod::pixels_per_unit(
                &rs.scene.frame.camera,
                root.transform_point3(node.center),
                node.radius * scale,
                view.view_height,
            );
            let near = og_render::lod::select(
                &[0.0, node.error],
                per_unit * scale,
                view.bias,
                place.drew[n].map(usize::from),
            ) == 0;
            // A node whose proxy is not loaded hands on to its children.
            let proxy = match parts {
                None => false,
                // Near, it still stands in while a model it holds itself
                // waits (its children's wait under their own proxies).
                Some(_) if loaded => {
                    under.clone().all(|k| member(k, h) != Member::Changed)
                        && (!near || (own..node.members.1).any(|k| member(k, h) == Member::Waiting))
                }
                // Nothing else shows the models a node holds itself (a
                // leaf's are all its own).
                Some(_) => !near || own < node.members.1,
            };
            // Under an ancestor fading out, what was drawn goes on being
            // drawn as it was until the fade ends.
            let before = place.drew[n];
            let proxy = match (crossing, before) {
                (Some((_, true)), Some(b)) if parts.is_some() || !b => b,
                _ => proxy,
            };
            // Fading in, when its maps arrived with nothing drawn in its
            // place (`crate::present::reveal`).
            let reveal = rs.reveal.proxy(tree.proxies[n]);
            // The cross-fade: started where the choice turned, reversed
            // from where it was when it turns back during one.
            let fades = crossing.is_none()
                && loaded
                && parts.is_some()
                && reveal == 1.0
                && under.clone().all(|k| member(k, h) == Member::Intact);
            if !fades {
                place.fades[n] = None;
            } else if before.is_some_and(|b| b != proxy) {
                let shown = place.fades[n].map_or(1.0, progress);
                place.fades[n] = Some(now - f64::from(1.0 - shown) * ticks);
            }
            let t = place.fades[n].map(progress).filter(|&t| t < 1.0);
            if t.is_none() {
                place.fades[n] = None;
            }
            settled &= before == Some(proxy) && t.is_none();
            place.drew[n] = Some(proxy);
            // What its children and its own models draw with: an
            // ancestor's fade, else this node's other side.
            let other = match t {
                Some(t) if proxy => Some((1.0 - t, true)),
                Some(t) => Some((t, false)),
                None => crossing,
            };
            let own_models = |crossed: &mut IdMap<u64, (f32, bool)>| {
                if let Some(c) = other {
                    for &(entity, _) in &place.members[own..node.members.1] {
                        crossed.insert(entity, c);
                    }
                }
            };
            let (Some(parts), true) = (parts, proxy || t.is_some()) else {
                own_models(&mut crossed);
                stack.extend(children.into_iter().rev().map(|c| (c, other)));
                continue;
            };
            // This node's side of its own fade, or of an ancestor's.
            let (reveal, fade_out) = match (t, crossing) {
                (Some(t), _) if proxy => (t, false),
                (Some(t), _) => (1.0 - t, true),
                (None, Some(c)) => c,
                (None, None) => (reveal, false),
            };
            if t.is_some() {
                // The other side draws too: its walk is kept as it was.
                own_models(&mut crossed);
                stack.extend(children.into_iter().rev().map(|c| (c, other)));
            } else {
                settled &= place.drew[n + 1..node.end].iter().all(Option::is_none);
                place.drew[n + 1..node.end].fill(None);
                place.fades[n + 1..node.end].fill(None);
            }
            drawn.proxies += 1;
            drawn.lit += u32::from(lit_parts.is_some());
            if !loaded {
                drawn.unloaded += 1;
            } else if t.is_none() && crossing.is_none() {
                for k in under {
                    replaced.insert(place.members[k].0, ());
                }
            }
            let fx = og_render::InstanceFx {
                reveal,
                fade_out,
                ..match view.debug {
                    true => og_render::InstanceFx {
                        flash: 0.8,
                        flash_color: debug_color(node.level),
                        ..og_render::InstanceFx::NONE
                    },
                    false => og_render::InstanceFx::NONE,
                }
            };
            let key = (1 << 63) | (place.serial << 24) | n as u64;
            for (k, &(mesh, material, local)) in parts.iter().enumerate() {
                if rs.scene.meshlet_drawable(mesh, material) {
                    let at = rs.scene.objects.instances.len();
                    rs.scene.objects.instance_levels.resize(at + 1, u8::MAX);
                    rs.scene.objects.instance_levels[at] = 0;
                }
                rs.scene.objects.instances.push(og_render::Instance {
                    mesh,
                    material,
                    transform: root * local,
                    material_layer: None,
                    fx,
                    cast_shadows: true,
                    receive_shadows: true,
                    sway: 0.0,
                });
                keys.push((key, k as u32));
            }
        }
    }
    drawn.replaced = replaced.len() as u32;
    rs.hlod.crossing = crossed;
    rs.hlod.places = places;
    rs.hlod.lit = lit;
    rs.hlod.drawn = drawn;
    rs.hlod.settled = settled;
    replaced
}
