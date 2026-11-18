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
//! **Presentation only.** The models stay in the world, simulated,
//! collided with and found by queries as before; only which instances the
//! frame holds changes. A proxy stands in for its models only while each
//! is as the cook read it: alive, visible, at its cooked place, with its
//! own model and look. One destroyed, hidden, moved, tinted or flashing
//! house returns its node, and every node above it, to their models.
//!
//! **World cells.** A cell's sub-scene has its own tree, kept in its declaring
//! bundle. While the cell is unloaded its proxies draw at the cell's pose
//! with nothing behind them, so the far world is there before it streams:
//! per branch the finest proxy that still shows every model of it.
//! (A `persist` cell's kept changes are not in its proxies.)

use crate::engine::Engine;
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
    /// Distinguishes its proxies' instances from every other place's.
    serial: u64,
}

/// What the last frame drew through HLOD (the probe's `dump_render`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Drawn {
    /// The trees walked.
    pub scenes: u32,
    /// Proxies drawn, and how many of them for cells not loaded.
    pub proxies: u32,
    pub unloaded: u32,
    /// Models a proxy drew in place of.
    pub replaced: u32,
}

#[derive(Default)]
pub(crate) struct HlodState {
    /// By bundle and scene id: its tree, or none when it has none.
    trees: BTreeMap<(usize, String), Option<Arc<Tree>>>,
    /// Unshared model resources owned by trees, also across a hot reload.
    pub proxies: IdMap<u64, Vec<og_render::TextureHandle>>,
    places: Vec<Place>,
    serial: u64,
    pub drawn: Drawn,
    /// The models the last frame's proxies drew in place of, by entity
    /// (the probe's `dump_entity`).
    pub replaced: IdMap<u64, ()>,
    /// Counts the changes to the places other than a frame's draw: a
    /// place added, or given its members.
    pub changes: u64,
    /// Whether the last draw kept every place and every node's choice as
    /// it found them: a frame with its rows, its view and [`inputs`] the
    /// same draws the same proxies again.
    pub settled: bool,
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

/// What [`draw`] reads of the frame.
pub(crate) struct View<'a> {
    pub models: &'a IdMap<u64, Parts>,
    /// The visible og.Model rows by entity.
    pub rows: &'a dyn Fn() -> IdMap<u64, Row>,
    pub view_height: f32,
    pub bias: f32,
    pub debug: bool,
}

/// A scene's proxy maps are presentation resources, just like its models'.
pub(crate) fn scene_ready(rs: &RenderState, bundle: usize, scene: &str) -> bool {
    rs.hlod
        .trees
        .get(&(bundle, scene.to_string()))
        .is_none_or(|tree| {
            tree.as_ref().is_none_or(|t| {
                t.proxies
                    .iter()
                    .all(|h| !rs.decoded.hidden_models.contains_key(h))
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
    let mut drawn = Drawn::default();
    let mut places = std::mem::take(&mut rs.hlod.places);
    let before = places.len();
    // A cell that is gone takes its place with it.
    places.retain(|p| !p.cell || p.root.is_some_and(|c| world.is_alive(Entity::from_bits(c))));
    let mut settled = places.len() == before;
    let rows = (!places.is_empty()).then(|| (view.rows)());
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
        // Per member, whether it is as the cook read it; asked once.
        let mut intact: Vec<Option<bool>> = vec![None; place.members.len()];
        let mut member_intact = |k: usize, h: &mut Hierarchy| -> bool {
            *intact[k].get_or_insert_with(|| {
                let (e, asset) = place.members[k];
                rows.get(&e).is_some_and(|r| r.plain && r.asset == asset)
                    && same_place(&h.world_of(e), &(root * tree.hlod.members[k].matrix))
            })
        };
        let mut stack: Vec<usize> = tree.hlod.children(None).collect();
        stack.reverse();
        while let Some(n) = stack.pop() {
            let node = &nodes[n];
            let parts = view
                .models
                .get(&tree.proxies[n])
                .filter(|_| !rs.decoded.hidden_models.contains_key(&tree.proxies[n]));
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
                Some(_) if loaded => !near && under.clone().all(|k| member_intact(k, h)),
                // Nothing else shows the models a node holds itself (a
                // leaf's are all its own).
                Some(_) => !near || own < node.members.1,
            };
            settled &= place.drew[n] == Some(proxy);
            place.drew[n] = Some(proxy);
            let Some(parts) = parts.filter(|_| proxy) else {
                stack.extend(children.into_iter().rev());
                continue;
            };
            settled &= place.drew[n + 1..node.end].iter().all(Option::is_none);
            place.drew[n + 1..node.end].fill(None);
            drawn.proxies += 1;
            if loaded {
                for k in under {
                    replaced.insert(place.members[k].0, ());
                }
            } else {
                drawn.unloaded += 1;
            }
            let fx = match view.debug {
                true => og_render::InstanceFx {
                    flash: 0.8,
                    flash_color: debug_color(node.level),
                    ..og_render::InstanceFx::NONE
                },
                false => og_render::InstanceFx::NONE,
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
    rs.hlod.places = places;
    rs.hlod.drawn = drawn;
    rs.hlod.settled = settled;
    replaced
}
