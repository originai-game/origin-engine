//! Cooked bundles, loads and scene spawning (IDL `load`, `World.instantiate`).
//!
//! The runtime reads cooked output through the host (`Hooks::read_asset`):
//! the native host reads files under the project, the web host answers from
//! the files it fetched before the engine started (and fetches a later
//! bundle's on request, which completes at a tick boundary). Textures are
//! decoded through `Hooks::texture`: og-bundle's native transcoder, or the web
//! host's Basis transcoder.
//!
//! Handles are `index + 2^32` (generation 1), like materials, so 0 is never
//! a live handle and a stale number cannot alias a later asset.

use super::codec::Layout;
use super::engine::{Engine, before_structural, hooks};
use super::extract::add_font;
use og_bundle::runtime::{Bundle, CookedSound, SceneDesc};
use og_core::IdMap;
use og_core::reflect::{FieldType, Schema};
use og_ecs::Entity;
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, HashMap};

/// What a host answers for a read or a decode.
pub enum Fetch<T> {
    Ready(T),
    /// Not there yet; the host is fetching it. Ask again at a tick boundary.
    Pending,
    Failed(String),
}

/// The manifest every bundle starts with.
pub const MANIFEST: &str = ".cook.ogt";

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Status {
    Pending,
    Ready,
    Failed(String),
}

pub(crate) enum Payload {
    Bundle {
        bundle: Box<Bundle>,
        dir: String,
    },
    Scene(Box<SceneDesc>),
    /// A sub-scene whose blob is in a cell group's pack (G3-3), not yet
    /// landed: its handle exists (a cell names it), its scene once the
    /// pack's bytes are in ([`super::stream::resolve_scenes`]).
    SceneWaiting {
        group: String,
    },
    Gltf {
        scenes: u32,
        geometries: Vec<u64>,
        materials: Vec<u64>,
        /// `KHR_materials_variants`' names (og.Model.variant indexes them).
        variants: Vec<String>,
        /// `KHR_lights_punctual`: what each instance spawns ([`spawn_lights`]).
        lights: Vec<ModelLight>,
    },
    /// An uncooked load not yet in place (super::uncooked).
    Uncooked,
    /// An uncooked image in place: its index in the render state's textures.
    Image {
        texture: usize,
        width: u32,
        height: u32,
    },
    None,
}

pub(crate) struct Entry {
    pub path: String,
    pub status: Status,
    pub payload: Payload,
}

pub(crate) struct LoadedFont {
    pub id: String,
    pub info: Value,
    /// Index into the render state's fonts.
    pub render: usize,
    /// The font's file, when it ships one (a shaped script, `runtime
    /// true`): og-ui shapes with it and measures characters outside the
    /// atlas, which `super::glyphs` renders.
    pub file: Option<std::sync::Arc<og_ui::shape::FontFile>>,
    /// An icon set's names (glyph U+E000 + i is icon i); empty for a text font.
    pub icons: Vec<String>,
}

/// Everything an engine loaded.
#[derive(Default)]
pub(crate) struct Assets {
    /// The project directory (native) or URL base (web); empty: none.
    pub root: String,
    pub entries: Vec<Entry>,
    /// The game's own bundle (`cooked`).
    pub default_bundle: Option<usize>,
    pub by_path: BTreeMap<String, usize>,
    pub fonts: Vec<LoadedFont>,
    pub sounds: Vec<CookedSound>,
    /// Each sound's handle in the engine's audio bank, by `sounds` index:
    /// what og.load.sound answers and AudioSource.sound names.
    pub sound_handles: Vec<u64>,
    /// og.load.style's styles, by handle index.
    pub styles: Vec<super::modules::LoadedStyle>,
    pub warnings: Vec<String>,
    /// The game bundle's streaming groups.
    pub stream: super::stream::Streams,
    /// og.load.textureArray's answers, by asset id.
    pub texture_arrays: BTreeMap<String, Value>,
    /// Per glTF asset index, the GltfDesc shadow flags of its last
    /// og.load.gltf ([`Assets::model_shadows`]).
    pub model_shadows: IdMap<usize, ModelShadows>,
    /// Uncooked loads (og.load.imageData) and their journal.
    pub uncooked: super::uncooked::Uncooked,
}

/// GltfDesc's `castShadow` and `receiveShadow`: what world.instantiate of
/// the glTF and a scene's `model` line give the og.Model they spawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ModelShadows {
    pub cast: bool,
    pub receive: bool,
}

impl ModelShadows {
    /// A model loaded without a desc casts and receives.
    pub const ON: ModelShadows = ModelShadows {
        cast: true,
        receive: true,
    };

    /// A GltfDesc's flags (each on when omitted); refused when one is not a
    /// boolean.
    pub fn from_desc(desc: &Value) -> Result<ModelShadows, String> {
        let flag = |k: &str| match &desc[k] {
            Value::Null => Ok(true),
            Value::Bool(b) => Ok(*b),
            other => Err(format!("GltfDesc.{k} is a boolean, got {other}")),
        };
        Ok(ModelShadows {
            cast: flag("castShadow")?,
            receive: flag("receiveShadow")?,
        })
    }

    /// The og.Model fields they set.
    pub fn model_fields(self, asset: u64) -> Value {
        json!({ "asset": asset as f64, "castShadow": self.cast, "receiveShadow": self.receive })
    }
}

/// One `og.load.textureArray` (by load index, the handle's): its renderer
/// array while resident.
pub(crate) struct LoadedArray {
    pub id: String,
    /// `None` while its streaming group is not resident: released when the
    /// group unloaded, added again once it is resident (`resolve_arrays`).
    pub handle: Option<og_render::TextureArrayHandle>,
    pub layers: u32,
    /// Each layer's width and height in texels.
    pub size: (u32, u32),
}

pub(crate) fn handle(index: usize) -> u64 {
    index as u64 + (1u64 << 32)
}

pub(crate) fn index_of(handle: f64) -> Option<usize> {
    let bits = handle as u64;
    (handle >= 0.0 && handle.fract() == 0.0 && bits >> 32 == 1)
        .then_some((bits & 0xffff_ffff) as usize)
}

fn join(root: &str, path: &str) -> String {
    if root.is_empty() || path.starts_with('/') || path.contains("://") {
        path.to_string()
    } else {
        format!("{}/{}", root.trim_end_matches('/'), path)
    }
}

fn read(path: &str) -> Fetch<Vec<u8>> {
    match hooks() {
        Some(h) => h.read_asset(path),
        None => Fetch::Failed("no host is installed to read assets".into()),
    }
}

impl Assets {
    /// The shadow flags the models instantiated from glTF asset `index`
    /// take: its last og.load.gltf's, on when it never had a desc.
    pub fn model_shadows(&self, index: usize) -> ModelShadows {
        self.model_shadows
            .get(&index)
            .copied()
            .unwrap_or(ModelShadows::ON)
    }

    fn push(&mut self, path: &str, status: Status, payload: Payload) -> usize {
        self.entries.push(Entry {
            path: path.to_string(),
            status,
            payload,
        });
        self.entries.len() - 1
    }

    /// Opens (or retries) bundle entry `i`.
    fn poll_bundle(&mut self, i: usize) {
        let dir = join(&self.root, &self.entries[i].path);
        let manifest = format!("{dir}/{MANIFEST}");
        let entry = &mut self.entries[i];
        match read(&manifest) {
            Fetch::Pending => entry.status = Status::Pending,
            Fetch::Failed(e) => {
                entry.status = Status::Failed(format!(
                    "{manifest}: {e} (run `og cook` to write the cooked bundle)"
                ))
            }
            Fetch::Ready(bytes) => match String::from_utf8(bytes)
                .map_err(|e| e.to_string())
                .and_then(|t| Bundle::from_manifest(&t).map_err(|e| e.to_string()))
            {
                Ok(bundle) => {
                    entry.status = Status::Ready;
                    entry.payload = Payload::Bundle {
                        bundle: Box::new(bundle),
                        dir,
                    };
                }
                Err(e) => entry.status = Status::Failed(format!("{manifest}: {e}")),
            },
        }
    }

    /// Opens a bundle directory (relative to the root).
    pub fn open_bundle(&mut self, path: &str) -> usize {
        let i = self.push(path, Status::Pending, Payload::None);
        self.poll_bundle(i);
        i
    }

    /// Retries pending bundles; called at every tick boundary.
    pub fn poll(&mut self) {
        for i in 0..self.entries.len() {
            if self.entries[i].status == Status::Pending
                && matches!(self.entries[i].payload, Payload::None)
            {
                self.poll_bundle(i);
            }
        }
    }

    /// The ready bundle holding `id` (the default bundle first).
    pub(crate) fn bundle_for(&self, id: &str) -> Result<(&Bundle, &str), String> {
        let order = self.default_bundle.into_iter().chain(0..self.entries.len());
        let mut open = 0;
        for i in order {
            if let Payload::Bundle { bundle, dir } = &self.entries[i].payload {
                open += 1;
                if bundle.output(id).is_ok() {
                    return Ok((bundle, dir));
                }
            }
        }
        if open == 0 {
            let why = self
                .default_bundle
                .and_then(|i| match &self.entries[i].status {
                    Status::Failed(e) => Some(e.clone()),
                    Status::Pending => Some("the bundle is still loading".into()),
                    Status::Ready => None,
                })
                .unwrap_or_else(|| "no cooked bundle is open".into());
            return Err(format!("{id}: {why}"));
        }
        Err(format!(
            "{}: not in the cooked bundle; put it under assets/ and run `og cook`",
            og_bundle::runtime::asset_id(id)
        ))
    }

    /// The bytes of a cooked output, 16-aligned.
    pub(crate) fn blob(&self, id: &str) -> Result<(og_blob_bytes::Aligned, &Bundle), String> {
        let (bundle, dir) = self.bundle_for(id)?;
        let out = bundle.output(id).map_err(|e| e.to_string())?;
        if let Some(group) = &out.group {
            // A streamed output is read from its group's pack, once resident.
            return Ok((aligned(self.stream.file(group, &out.file)?), bundle));
        }
        let path = format!("{dir}/{}", out.file);
        let start = super::trace::start();
        match read(&path) {
            Fetch::Ready(bytes) => {
                super::trace::load(id, start, bytes.len());
                Ok((og_blob_bytes::Aligned::new(bytes), bundle))
            }
            Fetch::Pending => Err(format!(
                "{path} is still being fetched; load it once its bundle is ready"
            )),
            Fetch::Failed(e) => Err(format!("{path}: {e}")),
        }
    }

    /// The streaming group of a cooked output (`None`: loaded at boot).
    pub(crate) fn group_of(&self, id: &str) -> Option<String> {
        let (bundle, _) = self.bundle_for(id).ok()?;
        bundle.output(id).ok()?.group.clone()
    }
}

/// Bytes copied 16-aligned, for blob views.
pub(crate) fn aligned(bytes: &[u8]) -> og_blob_bytes::Aligned {
    og_blob_bytes::Aligned::new(bytes.to_vec())
}

/// 16-aligned bytes for blob views.
pub(crate) mod og_blob_bytes {
    pub struct Aligned {
        words: Vec<u128>,
        len: usize,
    }
    impl Aligned {
        pub fn new(bytes: Vec<u8>) -> Aligned {
            let mut words = vec![0u128; bytes.len().div_ceil(16)];
            og_core::bytemuck::cast_slice_mut::<u128, u8>(&mut words)[..bytes.len()]
                .copy_from_slice(&bytes);
            Aligned {
                words,
                len: bytes.len(),
            }
        }
        pub fn bytes(&self) -> &[u8] {
            &og_core::bytemuck::cast_slice::<u128, u8>(&self.words)[..self.len]
        }
    }
}

// ── loads ───────────────────────────────────────────────────────────────────

/// A glTF's `KHR_lights_punctual` light as the entity each instance of the
/// model spawns under it: the light component and its fields, at the
/// light's node's transform in the model.
#[derive(Clone, Debug)]
pub(crate) struct ModelLight {
    pub component: &'static str,
    pub fields: Value,
    pub transform: Value,
}

/// Where a light of glTF's infinite range (no `range`) reaches zero: the
/// distance its inverse-square radiance falls to 0.01 (the engine's lights
/// always end; 10 m for intensity 1, 100 m for 100).
fn light_reach(intensity: f32) -> f32 {
    (intensity.max(0.0) / 0.01).sqrt().max(0.01)
}

/// The model's lights in the default scene, as [`ModelLight`]s: glTF's
/// point and spot lights as og.PointLight and og.SpotLight (intensity in
/// candela as the radiance at one metre, `range` as distance, the spot's
/// outer cone as its angle and `1 - inner / outer` as its penumbra, as
/// three.js GLTFLoader), a directional one as og.DirectionalLight.
fn model_lights(model: &og_bundle::runtime::CookedModel) -> Vec<ModelLight> {
    use og_bundle::format::light;
    let world = model.node_world();
    let in_scene: std::collections::BTreeSet<usize> = model
        .default_scene
        .map(|s| model.scene_nodes(s).into_iter().collect())
        .unwrap_or_default();
    model
        .lights
        .iter()
        .filter(|l| in_scene.contains(&l.node))
        .map(|l| {
            let color = json!({ "r": l.color[0], "g": l.color[1], "b": l.color[2] });
            let distance = if l.range > 0.0 {
                l.range
            } else {
                light_reach(l.intensity)
            };
            let (component, fields) = match l.kind {
                light::DIRECTIONAL => (
                    "og.DirectionalLight",
                    json!({ "color": color, "intensity": l.intensity }),
                ),
                light::SPOT => {
                    let [inner, outer] = l.cone;
                    let penumbra = if outer > 0.0 {
                        (1.0 - inner / outer).clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    (
                        "og.SpotLight",
                        json!({ "color": color, "intensity": l.intensity, "distance": distance,
                                "angle": outer, "penumbra": penumbra }),
                    )
                }
                _ => (
                    "og.PointLight",
                    json!({ "color": color, "intensity": l.intensity, "distance": distance }),
                ),
            };
            let (s, r, t) = world[l.node].to_scale_rotation_translation();
            ModelLight {
                component,
                fields,
                transform: json!({
                    "position": { "x": t.x, "y": t.y, "z": t.z },
                    "quaternion": { "x": r.x, "y": r.y, "z": r.z, "w": r.w },
                    "scale": { "x": s.x, "y": s.y, "z": s.z },
                }),
            }
        })
        .collect()
}

/// Spawns the lights of glTF asset `index` (its `KHR_lights_punctual`)
/// as children of `root`, the entity drawing the model: each an entity
/// with og.Transform at its node's place in the model, og.Parent and its
/// light component, so the lights move, hide and despawn with the model.
pub(crate) fn spawn_lights(eng: &Engine, index: usize, root: Entity) -> Result<(), String> {
    let lights = match eng.assets.borrow().entries.get(index).map(|e| &e.payload) {
        Some(Payload::Gltf { lights, .. }) => lights.clone(),
        _ => return Ok(()),
    };
    for l in lights {
        let mut desc = l.transform.clone();
        desc["parent"] = json!(root.to_bits() as f64);
        let e = super::api::spawn_entity(eng, "a glTF's light", &desc)?;
        let world = eng.world()?;
        let (layout, id) = super::api::component(eng, l.component)?;
        let bytes = layout.encode(None, Some(&l.fields))?;
        before_structural(eng);
        world
            .insert_bytes(e, id, &bytes)
            .map_err(|e| e.reason().to_string())?;
    }
    Ok(())
}

/// `og.load.gltf`: the model's meshes, materials and textures join the
/// renderer once per asset id; the answer is the asset index.
pub(crate) fn load_gltf(eng: &Engine, path: &str) -> Result<usize, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    if let Some(&i) = eng.assets.borrow().by_path.get(&format!("gltf:{id}")) {
        return Ok(i);
    }
    add_gltf(eng, &id, None)
}

/// Adds glTF `id` to the renderer as a new asset, or `into` an existing
/// asset index (a refresh: its handle then names the new meshes).
fn add_gltf(eng: &Engine, id: &str, into: Option<usize>) -> Result<usize, String> {
    let id = id.to_string();
    // A streamed model draws its proxy until its group is resident.
    let group = eng.assets.borrow().group_of(&id);
    let source = match &group {
        Some(_) => format!("{id}#proxy"),
        None => id.clone(),
    };
    let formats = texture_formats(eng);
    let (model, textures, impostor) = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&source)?;
        let model = bundle
            .model(&source, bytes.bytes())
            .map_err(|e| e.to_string())?;
        let texture = |id: &str| {
            let (tb, bundle) = assets.blob(id)?;
            let tex = bundle.texture(id, tb.bytes()).map_err(|e| e.to_string())?;
            match hooks().map(|h| h.texture(&tex, formats.as_ref())) {
                Some(Fetch::Ready(d)) => Ok(d),
                Some(Fetch::Pending) => Err(format!("{id}: the texture is still being transcoded")),
                Some(Fetch::Failed(e)) => Err(format!("{id}: {e}")),
                None => Err("no host is installed to decode textures".to_string()),
            }
        };
        let mut textures = Vec::with_capacity(model.textures.len());
        for t in &model.textures {
            textures.push(texture(&t.id)?);
        }
        // The impostor's atlases: a model whose atlases are missing draws
        // its meshes at every distance.
        let impostor = match &model.impostor {
            Some(i) => Some([texture(&i.albedo)?, texture(&i.normal)?]),
            None => None,
        };
        (model, textures, impostor)
    };
    insert_model(eng, &id, group, model, &textures, impostor, into, true).map(|(i, _)| i)
}

/// Adds a decoded model and its textures (in [`CookedModel::textures`]
/// order) to the renderer as asset `id`: a new asset, or `into` an existing
/// index. A cooked model is `registered` under its path (og.load.gltf of it
/// answers this asset, a hot reload refreshes it); an uncooked one
/// (og.load.gltfData) is not. A streamed model (`group`) is its proxy.
/// `impostor` holds its impostor's two atlases (albedo, normal), when it
/// has one. Answers the asset index and the model's textures in the scene.
#[allow(clippy::too_many_arguments)]
pub(crate) fn insert_model(
    eng: &Engine,
    id: &str,
    group: Option<String>,
    model: og_bundle::runtime::CookedModel,
    textures: &[og_render::TextureSource],
    impostor: Option<[og_render::TextureSource; 2]>,
    into: Option<usize>,
    registered: bool,
) -> Result<(usize, Vec<og_render::TextureHandle>), String> {
    let id = id.to_string();
    let source = match &group {
        Some(_) => format!("{id}#proxy"),
        None => id.clone(),
    };
    let mut rs = eng.render.borrow_mut();
    // The model's primitives become consecutive scene meshes from here.
    let base = rs.scene.mesh_count();
    let mut prim_mesh = vec![0u32; model.primitives.len()];
    for (m, mesh) in model.meshes.iter().enumerate() {
        for p in mesh.primitives.clone() {
            if let Some(slot) = prim_mesh.get_mut(p) {
                *slot = m as u32;
            }
        }
    }
    let material_base = rs.scene.material_count();
    let parts = model.add_to_with(&mut rs.scene, textures);
    for (h, t) in parts.textures.iter().zip(&model.textures) {
        rs.decoded.ids.insert(*h, t.id.clone());
    }
    let texture_handles = parts.textures.clone();
    // Each primitive's raycast BVH, cooked into the blob: loaded with the
    // mesh, so the first og.render.raycast builds nothing.
    for (p, prim) in model.primitives.iter().enumerate() {
        if let Some(h) = rs.scene.mesh_handle(base + p) {
            super::geometry::adopt_cooked_bvh(&mut rs, h, prim)
                .map_err(|e| format!("{source}: primitive {p}: {e}"))?;
        }
    }
    // The coarser levels of detail join the scene after the full meshes, so
    // the model's primitives stay consecutive from `base`. A streamed model
    // draws its proxy (one level) and then its full meshes without levels.
    let lods = match group {
        Some(_) => None,
        None => {
            let levels = model.add_lod_levels(&mut rs.scene);
            let prim_of = |mesh| {
                (0..model.primitives.len()).find(|&p| rs.scene.mesh_handle(base + p) == Some(mesh))
            };
            // Each part's levels with their errors in the model's units: a
            // part's node scale is folded in.
            let parts: Vec<Vec<(og_render::MeshHandle, f32)>> = parts
                .parts
                .iter()
                .map(|&(mesh, _, local)| {
                    let s = local.to_scale_rotation_translation().0.abs().max_element();
                    prim_of(mesh)
                        .map(|p| levels[p].iter().map(|&(m, e)| (m, e * s)).collect())
                        .unwrap_or_default()
                })
                .collect();
            // The impostor: the model's last level, a card drawn from
            // its atlases as a StandardMaterial (`map` the albedo,
            // `normalMap` the normal and depth; og-render's `impostor`).
            let impostor = match (impostor, &model.impostor) {
                (Some([albedo, normal]), Some(i)) => {
                    let albedo = rs.scene.add_texture_source(albedo);
                    let normal = rs.scene.add_texture_source(normal);
                    let material = rs.scene.add_standard_material(og_render::StandardMaterial {
                        map: albedo.slot(),
                        normalMap: normal.slot(),
                        alphaTest: 0.5,
                        side: 2,
                        roughness: 0.85,
                        ..Default::default()
                    });
                    let grid = i.grid.clamp(2, og_render::MAX_IMPOSTOR_GRID);
                    Some((
                        og_render::Impostor {
                            material,
                            grid,
                            center: i.center,
                            radius: i.radius,
                        },
                        i.error,
                    ))
                }
                _ => None,
            };
            (parts.iter().any(|l| !l.is_empty()) || impostor.is_some()).then(|| {
                super::extract::ModelLods {
                    parts,
                    center: (model.min + model.max) * 0.5,
                    radius: ((model.max - model.min) * 0.5).length().max(1e-3),
                    impostor,
                }
            })
        }
    };
    let stand_in = group.as_ref().map(|_| {
        let meshes: Vec<_> = (0..model.primitives.len())
            .filter_map(|p| {
                let h = rs.scene.mesh_handle(base + p)?;
                Some((h, rs.scene.mesh(h)?.clone()))
            })
            .collect();
        let materials: Vec<_> = rs
            .scene
            .material_handles()
            .skip(material_base)
            .take(model.materials.len())
            .filter_map(|h| Some((h, *rs.scene.standard_material(h)?)))
            .collect();
        (meshes, materials)
    });
    let mut geometries = Vec::new();
    let mut materials = Vec::new();
    // The colliders the cook baked (`collider true`), by Geometry handle.
    let mut baked: Vec<(u64, Vec<u8>)> = Vec::new();
    for &(mesh, material, _) in &parts.parts {
        if !rs.geometries.contains(&mesh) {
            rs.geometries.push(mesh);
        }
        if !rs.materials.contains(&material) {
            rs.materials.push(material);
        }
        let g = rs.geometries.iter().position(|m| *m == mesh).unwrap_or(0);
        let m = rs
            .materials
            .iter()
            .position(|x| *x == material)
            .unwrap_or(0);
        if !geometries.contains(&handle(g)) {
            geometries.push(handle(g));
            let bake = (0..model.primitives.len())
                .find(|&p| rs.scene.mesh_handle(base + p) == Some(mesh))
                .and_then(|p| model.primitives[p].collider.clone());
            baked.extend(bake.map(|b| (handle(g), b)));
        }
        if !materials.contains(&handle(m)) {
            materials.push(handle(m));
        }
    }
    let mut assets = eng.assets.borrow_mut();
    let payload = Payload::Gltf {
        scenes: model.scenes.len() as u32,
        geometries: geometries.clone(),
        materials,
        variants: model.variants.clone(),
        lights: model_lights(&model),
    };
    let index = match into {
        Some(i) => {
            assets.entries[i].status = Status::Ready;
            assets.entries[i].payload = payload;
            i
        }
        None => assets.push(&id, Status::Ready, payload),
    };
    rs.models.insert(handle(index), parts.parts);
    // Its top above its origin: how high og.Model.sway's wind reaches.
    rs.wind.tops.insert(handle(index), model.max.y);
    match lods {
        Some(l) => rs.model_lods.insert(handle(index), l),
        None => rs.model_lods.remove(&handle(index)),
    };
    rs.fx.models.insert(
        handle(index),
        super::fx::ModelInfo {
            id: id.clone(),
            base,
            prim_mesh,
            part_nodes: parts.nodes,
            part_shown: parts.shown,
            part_variants: parts.variants,
        },
    );
    if registered {
        assets.by_path.insert(format!("gltf:{id}"), index);
    }
    // Its geometries are physics meshes under their Geometry handles, for
    // convex and triangle-mesh colliders: built when a collider first names
    // one (modules::geometry_mesh, physics' mesh source), not copied for
    // every geometry here. A streamed model's are registered now, while they
    // are its proxy's, since physics collides with the proxy.
    let meshes: Vec<(u64, og_physics::MeshDesc)> = if group.is_some() {
        geometries
            .iter()
            .filter_map(|&h| {
                let m = *rs.geometries.get(index_of(h as f64)?)?;
                Some((h, super::modules::mesh_of(rs.scene.mesh(m)?)))
            })
            .collect()
    } else {
        Vec::new()
    };
    drop(assets);
    drop(rs);
    for (h, mesh) in meshes {
        if !eng.state.borrow().physics_meshes.contains_key(&h) {
            let _ = super::modules::set_mesh(eng, h, mesh);
        }
    }
    // Decoded now, between ticks: the step that first needs one builds
    // nothing.
    for (h, bytes) in baked {
        super::modules::set_baked_trimesh(eng, h, bytes)
            .map_err(|e| format!("{id}: geometry {h}: {e}"))?;
    }
    // Physics keeps the proxy's meshes; the renderer's take the full model
    // whenever its group is resident.
    if let (Some(group), Some((meshes, materials))) = (group, stand_in) {
        super::stream::add_model(eng, &group, &id, meshes, materials);
    }
    Ok((index, texture_handles))
}

/// After a hot reload (`og dev` cooks before it rebuilds the bundle): every
/// open bundle's manifest is read again (whatever is loaded: timelines
/// refresh against it too), and each glTF whose cooked outputs
/// changed (the model or its `#` textures, by hash) loads again into its own
/// asset index, so Model components keep their handle and draw the edit.
/// The old version's meshes and materials stay in the render scene, unused.
/// A manifest the host cannot read at once (a web fetch in flight) keeps its
/// bundle as it was. Answers the ids loaded again, and why any failed.
pub(crate) fn refresh(eng: &Engine) -> (Vec<String>, Vec<String>) {
    fn outputs(assets: &Assets, id: &str) -> Vec<u64> {
        let Ok((bundle, _)) = assets.bundle_for(id) else {
            return Vec::new();
        };
        bundle
            .manifest()
            .outputs()
            .filter(|o| {
                o.id == id
                    || o.id
                        .strip_prefix(id)
                        .is_some_and(|rest| rest.starts_with('#'))
            })
            .map(|o| o.hash)
            .collect()
    }
    let models: Vec<(usize, String, Vec<u64>)> = {
        let assets = eng.assets.borrow();
        assets
            .by_path
            .iter()
            .filter_map(|(key, &i)| {
                let id = key.strip_prefix("gltf:")?;
                Some((i, id.to_string(), outputs(&assets, id)))
            })
            .collect()
    };
    // Every manifest is read again, models or not: timelines and the
    // other assets read again after this compare against it.
    {
        let mut assets = eng.assets.borrow_mut();
        for entry in &mut assets.entries {
            let Payload::Bundle { bundle, dir } = &mut entry.payload else {
                continue;
            };
            if let Fetch::Ready(bytes) = read(&format!("{dir}/{MANIFEST}"))
                && let Ok(fresh) = String::from_utf8(bytes)
                    .map_err(|e| e.to_string())
                    .and_then(|t| Bundle::from_manifest(&t).map_err(|e| e.to_string()))
            {
                **bundle = fresh;
            }
        }
    }
    let (mut loaded, mut failed) = (Vec::new(), Vec::new());
    for (index, id, before) in models {
        let now = outputs(&eng.assets.borrow(), &id);
        if now.is_empty() || now == before {
            continue;
        }
        match add_gltf(eng, &id, Some(index)) {
            Ok(_) => loaded.push(id),
            Err(e) => failed.push(format!("{id}: {e}")),
        }
    }
    (loaded, failed)
}

/// `og.load.scene`: decoded and kept; `instantiate` spawns it.
pub(crate) fn load_scene(eng: &Engine, path: &str) -> Result<usize, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    if let Some(&i) = eng.assets.borrow().by_path.get(&format!("scene:{id}")) {
        return Ok(i);
    }
    let group = eng.assets.borrow().group_of(&id);
    // A cell group's sub-scene: its handle now, its scene once its pack
    // has landed (at once when it has).
    if let Some(group) = group {
        let mut assets = eng.assets.borrow_mut();
        let index = assets.push(
            &id,
            Status::Pending,
            Payload::SceneWaiting {
                group: group.clone(),
            },
        );
        assets.by_path.insert(format!("scene:{id}"), index);
        assets.stream.add_scene(&group, index);
        drop(assets);
        super::stream::resolve_scenes(eng, Some(&group));
        return Ok(index);
    }
    let scene = decode_scene(&eng.assets.borrow(), &id)?;
    let mut assets = eng.assets.borrow_mut();
    let index = assets.push(&id, Status::Ready, Payload::Scene(Box::new(scene)));
    assets.by_path.insert(format!("scene:{id}"), index);
    Ok(index)
}

/// A cooked scene's blob decoded (from its group's pack when it streams).
pub(crate) fn decode_scene(assets: &Assets, id: &str) -> Result<SceneDesc, String> {
    let (bytes, bundle) = assets.blob(id)?;
    bundle.scene(id, bytes.bytes()).map_err(|e| e.to_string())
}

pub(crate) fn load_font(eng: &Engine, path: &str) -> Result<usize, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    if let Some(i) = eng.assets.borrow().fonts.iter().position(|f| f.id == id) {
        return Ok(i);
    }
    let font = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&id)?;
        bundle.font(&id, bytes.bytes()).map_err(|e| e.to_string())?
    };
    let m = &font.meta;
    let info = json!({
        "glyphs": m.glyph_count, "pages": m.page_count, "emPx": m.em_px as u32,
        "pxRange": m.px_range, "ascender": m.ascender, "descender": m.descender,
        "lineHeight": m.ascender - m.descender + m.line_gap,
        "icons": font.names.len(),
    });
    let mut rs = eng.render.borrow_mut();
    let pages = add_font(&mut rs.scene, &font);
    rs.fonts.push(pages);
    let render = rs.fonts.len() - 1;
    let file = font
        .ttf
        .clone()
        .and_then(|t| og_ui::shape::FontFile::new(std::sync::Arc::new(t)));
    if let Some(f) = &file {
        rs.runtime_fonts.insert(
            render,
            super::glyphs::RuntimeFont::new(f.clone(), m.em_px, m.px_range),
        );
    }
    let mut assets = eng.assets.borrow_mut();
    assets.fonts.push(LoadedFont {
        id,
        info,
        render,
        file,
        icons: font.names.clone(),
    });
    Ok(assets.fonts.len() - 1)
}

/// The encodings the attached renderer takes (`None` before one is attached:
/// textures then load as RGBA8).
pub(crate) fn texture_formats(eng: &Engine) -> Option<og_render::TextureFormats> {
    eng.render
        .borrow()
        .backend
        .as_ref()
        .and_then(|b| b.texture_formats())
}

/// `og.load.texture`: a cooked texture added to the renderer once; the
/// answer is its index in the render state's textures.
pub(crate) fn load_texture(eng: &Engine, path: &str) -> Result<usize, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    if let Some(&i) = eng.assets.borrow().by_path.get(&format!("texture:{id}")) {
        return Ok(i);
    }
    let group = eng.assets.borrow().group_of(&id);
    if let Some(group) = group {
        // A placeholder until its group is resident, then the texture.
        let mut rs = eng.render.borrow_mut();
        let handle = rs.scene.add_texture(super::stream::placeholder());
        rs.textures.push(handle);
        let i = rs.textures.len() - 1;
        drop(rs);
        eng.assets
            .borrow_mut()
            .by_path
            .insert(format!("texture:{id}"), i);
        super::stream::add_texture(eng, &group, &id, handle);
        return Ok(i);
    }
    let (data, pixel) = decode_texture(eng, &id)?;
    let mut rs = eng.render.borrow_mut();
    let handle = rs.scene.add_texture_source(data);
    rs.decoded.ids.insert(handle, id.clone());
    if pixel {
        rs.scene
            .set_texture_filter(handle, og_render::TextureFilter::Nearest);
    }
    rs.textures.push(handle);
    let i = rs.textures.len() - 1;
    drop(rs);
    eng.assets
        .borrow_mut()
        .by_path
        .insert(format!("texture:{id}"), i);
    link_index(eng, &id, handle);
    link_normal(eng, &id, handle);
    Ok(i)
}

/// A 2D texture's normal map: the cook's `<id>#normal` beside it (a
/// `normal` line in its import settings, og-cook's normal2d.rs), added to
/// the renderer only and named its normal, so 2D lighting shades every
/// sprite, tile and 2D rig drawn with it in relief. None without one.
fn link_normal(eng: &Engine, id: &str, texture: og_render::TextureHandle) {
    let name = format!("{id}#normal");
    if eng.assets.borrow().blob(&name).is_err() {
        return;
    }
    let decoded = decode_texture(eng, &name);
    let mut rs = eng.render.borrow_mut();
    match decoded {
        Ok((data, _)) => {
            let h = rs.scene.add_texture_source(data);
            rs.decoded.ids.insert(h, name);
            rs.scene.set_texture_normal(texture, Some(h));
        }
        Err(e) => rs
            .notes
            .push(format!("{name}: not loaded ({e}); the texture is lit flat")),
    }
}

/// A cooked texture decoded through the host's texture hook, and whether
/// its kind is pixel art (sampled nearest).
fn decode_texture(eng: &Engine, id: &str) -> Result<(og_render::TextureSource, bool), String> {
    match decode(eng, id) {
        Fetch::Ready(d) => Ok(d),
        Fetch::Pending => Err(format!("{id}: the texture is still being transcoded")),
        Fetch::Failed(e) => Err(e),
    }
}

/// [`decode_texture`], with a texture the host is still transcoding
/// answered `Pending`.
fn decode(eng: &Engine, id: &str) -> Fetch<(og_render::TextureSource, bool)> {
    let formats = texture_formats(eng);
    let assets = eng.assets.borrow();
    let tex = match assets
        .blob(id)
        .and_then(|(bytes, bundle)| bundle.texture(id, bytes.bytes()).map_err(|e| e.to_string()))
    {
        Ok(t) => t,
        Err(e) => return Fetch::Failed(e),
    };
    let pixel = tex.is_pixel_art();
    match hooks().map(|h| h.texture(&tex, formats.as_ref())) {
        Some(Fetch::Ready(d)) => Fetch::Ready((d, pixel)),
        Some(Fetch::Pending) => Fetch::Pending,
        Some(Fetch::Failed(e)) => Fetch::Failed(format!("{id}: {e}")),
        None => Fetch::Failed("no host is installed to decode textures".into()),
    }
}

/// The cooked textures in the renderer and the device encodings they
/// were last checked against. A renderer made again after a lost device
/// may take other encodings than the old one did (another adapter, a
/// browser that came back without BC): what it does not take is decoded
/// again from the bundle for it ([`retranscode`]), never uploaded as the
/// old device's encoding.
#[derive(Default)]
pub(crate) struct Decoded {
    /// Each scene texture decoded from a cooked texture, with its asset id.
    pub ids: HashMap<og_render::TextureHandle, String>,
    /// The encodings of the last check.
    formats: Option<og_render::TextureFormats>,
    /// Textures the host is still transcoding for the new device, drawn
    /// as the streaming placeholder meanwhile.
    pending: Vec<(og_render::TextureHandle, String)>,
    /// Texture arrays (by load index) let go of until the host has
    /// transcoded their layers, added again by [`resolve_texture_arrays`].
    arrays_pending: Vec<usize>,
}

/// Before a frame, after the backend recovered from a lost device: when
/// the device's texture encodings changed since the last frame, every
/// cooked texture and texture array the renderer holds in an encoding the
/// device does not take is decoded again from the bundle through the
/// host's texture hook, for the new encodings. Natively the hook
/// transcodes at once, so the frame draws as it did before the loss; the
/// web host's Worker transcodes in the background, and those textures
/// draw as the placeholder (arrays not at all) until they land.
pub(crate) fn retranscode(eng: &Engine) {
    let Some(formats) = texture_formats(eng) else {
        return;
    };
    let takes_upload = |u: &og_render::TextureUpload| formats.supported.contains(&u.desc.format);
    let takes = |s: &og_render::TextureSource| match s {
        og_render::TextureSource::Encoded(u) => takes_upload(u),
        _ => true,
    };
    let (stale, arrays) = {
        let mut rs = eng.render.borrow_mut();
        let rs = &mut *rs;
        let mut stale = std::mem::take(&mut rs.decoded.pending);
        let mut arrays = std::mem::take(&mut rs.decoded.arrays_pending);
        if rs.decoded.formats.as_ref() != Some(&formats) {
            rs.decoded.formats = Some(formats.clone());
            let scene = &rs.scene;
            rs.decoded.ids.retain(|h, _| scene.contains_texture(*h));
            stale.extend(
                rs.decoded
                    .ids
                    .iter()
                    .filter(|(h, _)| scene.texture_upload(**h).is_some_and(|u| !takes_upload(u)))
                    .map(|(h, id)| (*h, id.clone())),
            );
            for (i, a) in rs.texture_arrays.iter().enumerate() {
                let old = a
                    .handle
                    .and_then(|h| scene.texture_array(h))
                    .is_some_and(|d| !d.layers.iter().all(takes));
                if old {
                    arrays.push(i);
                }
            }
        }
        (stale, arrays)
    };
    let mut pending = Vec::new();
    for (h, id) in stale {
        let decoded = decode(eng, &id);
        let mut rs = eng.render.borrow_mut();
        match decoded {
            Fetch::Ready((source, _)) => {
                super::stream::replace_texture(&mut rs.scene, h, source);
            }
            Fetch::Pending => {
                rs.scene.set_texture(h, super::stream::placeholder());
                pending.push((h, id));
            }
            Fetch::Failed(e) => {
                rs.scene.set_texture(h, super::stream::placeholder());
                rs.notes.push(format!(
                    "{id}: not decoded again for the new GPU device ({e}); drawn as a placeholder"
                ));
            }
        }
        // A texture streamed by its mips no longer holds its whole chain.
        drop(rs);
        super::stream::mips::reset(eng, h);
    }
    let mut arrays_pending = Vec::new();
    for i in arrays {
        let Some((id, handle)) = eng
            .render
            .borrow()
            .texture_arrays
            .get(i)
            .map(|a| (a.id.clone(), a.handle))
        else {
            continue;
        };
        let Some(h) = handle else {
            // Let go of below on an earlier frame: added again once its
            // layers decode.
            if let Ok((added, _)) = add_texture_array(eng, &id) {
                eng.render.borrow_mut().texture_arrays[i].handle = Some(added);
            } else {
                arrays_pending.push(i);
            }
            continue;
        };
        let data = texture_array_data(eng, &id);
        let mut rs = eng.render.borrow_mut();
        let replaced = data.and_then(|(d, _)| rs.scene.set_texture_array(h, d));
        if replaced.is_err() {
            rs.scene.remove_texture_array(h);
            rs.texture_arrays[i].handle = None;
            arrays_pending.push(i);
        }
    }
    let mut rs = eng.render.borrow_mut();
    rs.decoded.pending = pending;
    rs.decoded.arrays_pending = arrays_pending;
}

/// A palette swap's index texture (docs/art.md §2.4,
/// `super::art::ArtState::index_of`): `texture` itself when its id is an
/// index texture (`<id>#index`), else the cook's `<id>#index` beside it,
/// added to the renderer only (never a Texture handle of the game's, so
/// the handles a game holds are the same with and without the contract).
fn link_index(eng: &Engine, id: &str, texture: og_render::TextureHandle) {
    let index = if id.ends_with("#index") {
        Some(texture)
    } else {
        let name = format!("{id}#index");
        let known = eng.assets.borrow().blob(&name).is_ok();
        match known.then(|| decode_texture(eng, &name)) {
            Some(Ok((data, pixel))) => {
                let mut rs = eng.render.borrow_mut();
                let h = rs.scene.add_texture_source(data);
                if pixel {
                    rs.scene
                        .set_texture_filter(h, og_render::TextureFilter::Nearest);
                }
                rs.decoded.ids.insert(h, name);
                Some(h)
            }
            _ => None,
        }
    };
    if let Some(index) = index {
        eng.render.borrow_mut().art.index_of.insert(texture, index);
    }
}

/// Reads and decodes a cooked texture array's layers through the host's
/// texture hook, and adds them to the renderer as one array (sampled
/// nearest when its kind is pixel art).
fn add_texture_array(
    eng: &Engine,
    id: &str,
) -> Result<(og_render::TextureArrayHandle, og_render::TextureArrayInfo), String> {
    let (data, pixel) = texture_array_data(eng, id)?;
    let info = data.info().map_err(|e| format!("{id}: {e}"))?;
    let mut rs = eng.render.borrow_mut();
    let added = rs
        .scene
        .add_texture_array(data)
        .map_err(|e| format!("{id}: {e}"))?;
    if pixel {
        rs.scene
            .set_texture_array_filter(added, og_render::TextureFilter::Nearest);
    }
    Ok((added, info))
}

/// A cooked texture array's layers, decoded through the host's texture
/// hook into the encodings the device takes, and whether its kind is pixel
/// art (sampled nearest).
pub(crate) fn texture_array_data(
    eng: &Engine,
    id: &str,
) -> Result<(og_render::TextureArrayData, bool), String> {
    let formats = texture_formats(eng);
    let (array, layers, pixel) = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(id)?;
        let array = bundle
            .texture_array(id, bytes.bytes())
            .map_err(|e| e.to_string())?;
        let mut layers = Vec::with_capacity(array.layers.len());
        let mut pixel = false;
        for l in &array.layers {
            let (bytes, bundle) = assets.blob(l)?;
            let tex = bundle
                .texture(l, bytes.bytes())
                .map_err(|e| e.to_string())?;
            pixel = tex.is_pixel_art();
            match hooks().map(|h| h.texture(&tex, formats.as_ref())) {
                Some(Fetch::Ready(d)) => layers.push(d),
                Some(Fetch::Pending) => {
                    return Err(format!("{l}: the layer is still being transcoded"));
                }
                Some(Fetch::Failed(e)) => return Err(format!("{l}: {e}")),
                None => return Err("no host is installed to decode textures".into()),
            }
        }
        (array, layers, pixel)
    };
    if layers.len() != array.meta.layers as usize {
        return Err(format!(
            "{id}: {} layers, its meta says {}",
            layers.len(),
            array.meta.layers
        ));
    }
    Ok((og_render::TextureArrayData { layers }, pixel))
}

/// `og.load.textureArray`: a cooked `type texture-array`, its layers
/// decoded through the host's texture hook (each an `og.Texture` the web
/// Worker transcoded with the bundle) and uploaded as one 2D-array
/// texture. Loading a path again answers the same array. One in a
/// streaming group's pack loads once its group is resident; when the
/// group unloads the renderer lets the array go (`super::stream`) and the
/// handle draws nothing until the group is resident again.
pub(crate) fn load_texture_array(eng: &Engine, path: &str) -> Result<Value, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    if let Some(v) = eng.assets.borrow().texture_arrays.get(&id) {
        return Ok(v.clone());
    }
    load_new_texture_array(eng, &id)
}

/// [`load_texture_array`]'s load index (what a TextureArray handle and
/// RenderState::texture_arrays index): for the engine's own loads of an
/// array (a flipbook's, an emitter's).
pub(crate) fn load_texture_array_index(eng: &Engine, path: &str) -> Result<usize, String> {
    let v = load_texture_array(eng, path)?;
    v["array"]
        .as_f64()
        .and_then(index_of)
        .ok_or_else(|| format!("{path}: no texture array handle"))
}

fn load_new_texture_array(eng: &Engine, id: &str) -> Result<Value, String> {
    let id = id.to_string();
    let (added, info) = add_texture_array(eng, &id)?;
    let index = {
        let mut rs = eng.render.borrow_mut();
        rs.texture_arrays.push(LoadedArray {
            id: id.clone(),
            handle: Some(added),
            layers: info.layers,
            size: (info.width, info.height),
        });
        rs.texture_arrays.len() - 1
    };
    let value = json!({
        "array": handle(index) as f64,
        "width": info.width,
        "height": info.height,
        "layers": info.layers,
        "levels": info.levels,
        "format": info.format.name(),
        "bytes": info.bytes as f64,
    });
    eng.assets
        .borrow_mut()
        .texture_arrays
        .insert(id, value.clone());
    Ok(value)
}

/// At a tick boundary: adds again the texture arrays whose streaming group
/// is resident again after an unload released them.
pub(crate) fn resolve_texture_arrays(eng: &Engine) {
    let pending: Vec<(usize, String)> = eng
        .render
        .borrow()
        .texture_arrays
        .iter()
        .enumerate()
        .filter(|(_, a)| a.handle.is_none())
        .map(|(i, a)| (i, a.id.clone()))
        .collect();
    for (i, id) in pending {
        if let Ok((h, _)) = add_texture_array(eng, &id) {
            eng.render.borrow_mut().texture_arrays[i].handle = Some(h);
        }
    }
}

pub(crate) fn load_sound(eng: &Engine, path: &str) -> Result<usize, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    if let Some(i) = eng.assets.borrow().sounds.iter().position(|s| s.id == id) {
        return Ok(i);
    }
    let group = eng.assets.borrow().group_of(&id);
    if let Some(group) = group {
        // Silence of its shape until its group is resident.
        let (sound, handle) = super::stream::add_sound(eng, &group, &id)?;
        let mut assets = eng.assets.borrow_mut();
        assets.sounds.push(sound);
        assets.sound_handles.push(handle);
        return Ok(assets.sounds.len() - 1);
    }
    let sound = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&id)?;
        bundle
            .sound(&id, bytes.bytes())
            .map_err(|e| e.to_string())?
    };
    // The engine's audio bank holds the sound the mixer plays; its handle is
    // the one AudioSource.sound names.
    let handle = match og_audio::link(eng.id) {
        // Cooked in chunks: the first is here, the others are read as the
        // playheads near them (stream::sound_chunks).
        Some(link) if sound.chunked() => link
            .bank()
            .insert_chunked(
                &sound.id,
                &sound.qoa,
                sound.meta.chunk_frames,
                sound.meta.chunk_count,
            )
            .map_err(|e| e.reason().to_string())?,
        Some(link) if sound.is_opus() => link
            .bank()
            .insert_opus(&sound.id, &sound.opus)
            .map_err(|e| e.reason().to_string())?,
        Some(link) => link
            .bank()
            .insert_qoa(&sound.id, &sound.qoa)
            .map_err(|e| e.reason().to_string())?,
        None => return Err(format!("{id}: this engine has no audio")),
    };
    #[cfg(not(target_arch = "wasm32"))]
    if sound.chunked()
        && let Some(link) = og_audio::link(eng.id)
    {
        link.bank()
            .set_chunk_loader(handle, chunk_loader(&eng.assets.borrow(), &sound));
    }
    let mut assets = eng.assets.borrow_mut();
    assets.sounds.push(sound);
    assets.sound_handles.push(handle);
    Ok(assets.sounds.len() - 1)
}

/// Natively, a chunk a voice reaches before stream::sound_chunks put it in
/// the bank (a long advance mixed at once) is read from its file there
/// and then, on whichever thread mixes, its type and cook stamp checked.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn chunk_loader(assets: &Assets, sound: &CookedSound) -> og_audio::ChunkLoader {
    let files: Vec<Option<(String, u64)>> = (0..sound.meta.chunk_count)
        .map(|n| {
            let id = og_bundle::runtime::chunk_id(&sound.id, n);
            let (bundle, dir) = assets.bundle_for(&id).ok()?;
            let out = bundle.output(&id).ok()?;
            Some((format!("{dir}/{}", out.file), out.stamp))
        })
        .collect();
    std::sync::Arc::new(move |n| {
        let (path, stamp) = files.get(n as usize)?.as_ref()?;
        let bytes = std::fs::read(path).ok()?;
        match og_bundle::runtime::sound_chunk_of(&bytes, path, *stamp) {
            Ok((k, qoa)) if k == n => Some(qoa),
            _ => None,
        }
    })
}

/// The `sounds` index of a sound handle.
pub(crate) fn sound_index(eng: &Engine, handle: f64) -> Option<usize> {
    let bits = handle as u64;
    (handle >= 0.0 && handle.fract() == 0.0)
        .then(|| {
            eng.assets
                .borrow()
                .sound_handles
                .iter()
                .position(|h| *h == bits)
        })
        .flatten()
}

// ── scene spawning ──────────────────────────────────────────────────────────

/// One field of a packed row as the JSON the codec reads, shaped for the
/// target field's kind.
fn field_json(ty: FieldType, bytes: &[u8], target: &super::codec::FieldLayout) -> Value {
    if ty.is_string() {
        return json!(og_core::unpack_str(bytes));
    }
    if let Some((element, _)) = ty.list_of() {
        // A list moves by its elements; the codec refuses more than fit.
        let one = element.field_type();
        return Value::Array(
            og_core::list_elements(ty, bytes)
                .map(|e| field_json(one, e, target))
                .collect(),
        );
    }
    let lanes: Vec<f64> = (0..ty.lanes())
        .map(|i| {
            let at = i * ty.scalar().size();
            let b = &bytes[at..at + ty.scalar().size()];
            match &*ty.name() {
                "f64" => f64::from_le_bytes(b.try_into().unwrap_or([0; 8])),
                "i32" => i32::from_le_bytes(b.try_into().unwrap_or([0; 4])) as f64,
                "u32" | "enum" => u32::from_le_bytes(b.try_into().unwrap_or([0; 4])) as f64,
                "i64" => i64::from_le_bytes(b.try_into().unwrap_or([0; 8])) as f64,
                "u64" => u64::from_le_bytes(b.try_into().unwrap_or([0; 8])) as f64,
                _ if b.len() == 1 => f64::from(b[0]),
                _ => f64::from(f32::from_le_bytes(b.try_into().unwrap_or([0; 4]))),
            }
        })
        .collect();
    use super::codec::Kind;
    match target.kind {
        Kind::Vector2 | Kind::Vector3 | Kind::Quaternion | Kind::Color => {
            let keys: &[&str] = match target.kind {
                Kind::Vector2 => &["x", "y"],
                Kind::Vector3 => &["x", "y", "z"],
                Kind::Quaternion => &["x", "y", "z", "w"],
                _ => &["r", "g", "b"],
            };
            let mut m = Map::new();
            for (i, k) in keys.iter().enumerate() {
                m.insert((*k).into(), json!(lanes.get(i).copied().unwrap_or(0.0)));
            }
            Value::Object(m)
        }
        Kind::Matrix4 => json!({ "elements": lanes }),
        Kind::Bool => json!(lanes.first().copied().unwrap_or(0.0) != 0.0),
        _ => json!(lanes.first().copied().unwrap_or(0.0)),
    }
}

/// A scene table's row as a value of the registered layout, migrated by
/// field name: missing fields keep their defaults, extra ones are reported.
pub(crate) fn migrate(
    schema: &Schema,
    row: &[u8],
    layout: &Layout,
    dropped: &mut Vec<String>,
) -> Value {
    let packed = og_core::reflect::Layout::of(schema);
    let mut out = Map::new();
    for (i, f) in schema.fields.iter().enumerate() {
        let Some((_, target)) = layout.field(&f.name) else {
            let name = format!("{}.{}", layout.schema, f.name);
            if !dropped.contains(&name) {
                dropped.push(name);
            }
            continue;
        };
        let range = packed.range(i);
        // A string moves by its text and a list by its elements; the codec
        // refuses one over the capacity.
        let strings = f.ty.is_string() && target.ty.is_string();
        let lists = f.ty.is_list() && target.ty.is_list();
        if !strings && !lists && f.ty.lanes() != target.ty.lanes() {
            dropped.push(format!(
                "{}.{} ({} lanes in the scene, {} registered)",
                layout.schema,
                f.name,
                f.ty.lanes(),
                target.ty.lanes()
            ));
            continue;
        }
        out.insert(f.name.clone(), field_json(f.ty, &row[range], target));
    }
    Value::Object(out)
}

/// The registered component a scene table names: exact, else the engine's
/// `og.<name>`.
fn resolve<'a>(layouts: &'a BTreeMap<String, Layout>, name: &str) -> Option<&'a Layout> {
    layouts
        .get(name)
        .or_else(|| layouts.get(&format!("og.{name}")))
        .filter(|l| l.kind == super::codec::RecordKind::Component)
}

/// Where a scene's entities go ([`spawn_placed`]).
#[derive(Clone, Copy, Debug)]
pub(crate) enum Placement {
    /// Its top-level entities under this root (at the top level when `None`).
    Under(Option<Entity>),
    /// A world cell's sub-scene (G3-3, [`super::stream::cells`]): its
    /// top-level entities are roots placed at the cell's world pose after
    /// their own transforms, and every entity it spawns carries an
    /// `og.CellMember` naming the cell.
    Cell {
        cell: Entity,
        pose: og_core::math::Pose,
    },
}

/// Spawns scene entity rows under `root` (top level when `None`). The whole
/// scene is refused before anything spawns when a name is taken.
pub(crate) fn spawn_scene(
    eng: &Engine,
    scene: &SceneDesc,
    root: Option<Entity>,
) -> Result<Vec<Entity>, String> {
    spawn_placed(eng, scene, Placement::Under(root))
}

/// [`spawn_scene`] with a [`Placement`]. An entity with a sub-scene
/// (`scene "<id>"`) gets it spawned under it, or, when it is a world cell
/// (`og.StreamCell`), its `scene` set to the sub-scene's handle for the
/// cell to load.
pub(crate) fn spawn_placed(
    eng: &Engine,
    scene: &SceneDesc,
    placement: Placement,
) -> Result<Vec<Entity>, String> {
    let mut spawned = Vec::with_capacity(scene.entities.len());
    spawn_part(eng, scene, placement, &mut spawned, scene.entities.len())?;
    Ok(spawned)
}

/// [`spawn_placed`] in parts (a world cell's `budget`, G3-3): spawns the
/// scene's entities from `spawned.len()` up to `end`, pushing each onto
/// `spawned` (the entities earlier parts spawned, in order: parents and
/// prefab roots are found there), and once the last one is in, what
/// follows the entities (sub-scenes, resources, the instance record).
/// The name check runs with the first part. On `Err`, `spawned` holds
/// what spawned before the refusal.
pub(crate) fn spawn_part(
    eng: &Engine,
    scene: &SceneDesc,
    placement: Placement,
    spawned: &mut Vec<Entity>,
    end: usize,
) -> Result<(), String> {
    let end = end.min(scene.entities.len());
    let root = match placement {
        Placement::Under(root) => root,
        Placement::Cell { .. } => None,
    };
    // A prefab's own blob (og.load.prefab) spawns unnamed: it is placed
    // many times.
    let prefab = scene.prefabs.first() == Some(&scene.id);
    if !prefab && spawned.is_empty() {
        let st = eng.state.borrow();
        let mut seen = std::collections::BTreeSet::new();
        for e in &scene.entities {
            if e.name.is_empty() {
                continue;
            }
            if st.names.contains_key(&e.name) || !seen.insert(e.name.clone()) {
                return Err(format!(
                    "{}: an entity named '{}' already exists; instantiate the scene under a root, or rename it in the scene",
                    scene.id, e.name
                ));
            }
        }
    }
    if spawned.is_empty() {
        super::timeline::register_scene_paths(eng.id, &scene.paths);
    }
    let layouts = eng.state.borrow().layouts.clone();
    let transform = resolve(&layouts, "Transform").cloned();
    let parent_layout = resolve(&layouts, "Parent").cloned();
    let model_layout = resolve(&layouts, "Model").cloned();
    let mut dropped = Vec::new();
    let mut tables = Vec::new();
    for t in &scene.components {
        match resolve(&layouts, &t.name) {
            Some(l) => {
                // A lazy engine record (og.StreamCell, ...) is registered
                // once a scene names it.
                super::lazy::ensure(eng, eng.world()?, &l.schema)?;
                tables.push((t, l.clone()))
            }
            None => dropped.push(format!(
                "{} (no registered component of that name; declare it with defineComponent before the scene loads)",
                t.name
            )),
        }
    }
    // The prefabs its entities came from, each loaded once (the links'
    // asset handles), and og.PrefabLink registered: before anything spawns.
    let mut prefab_assets = Vec::with_capacity(scene.prefabs.len());
    for id in &scene.prefabs {
        prefab_assets.push(if *id == scene.id {
            load_scene(eng, id)?
        } else {
            super::prefab::load(eng, id).map_err(|r| format!("{}: {r}", scene.id))?
        });
    }
    let link = if scene.prefabs.is_empty() {
        None
    } else {
        let id = super::prefab::register(eng, eng.world()?)?;
        let layout = eng.state.borrow().layouts.get(super::prefab::LINK).cloned();
        layout.map(|l| (id, l))
    };
    // A cell's members: og.CellMember, registered with the cell.
    let member = match placement {
        Placement::Cell { cell, .. } => {
            let name = super::stream::cells::MEMBER;
            super::lazy::ensure(eng, eng.world()?, name)?;
            let layout = layouts
                .get(name)
                .ok_or_else(|| format!("{name} has no layout"))?;
            let id = eng
                .world()?
                .id_by_name(name)
                .map_err(|e| e.reason().to_string())?;
            let bytes = layout.encode(None, Some(&json!({ "cell": cell.to_bits() as f64 })))?;
            Some((id, bytes))
        }
        Placement::Under(_) => None,
    };
    let cell_layout = resolve(&layouts, super::stream::cells::CELL).cloned();
    before_structural(eng);
    for i in spawned.len()..end {
        let e = &scene.entities[i];
        let world = eng.world()?;
        let mut components = Vec::new();
        // Its og.PrefabLink, spawned with it (an instance's root gets its
        // own once its id is known, below).
        let origin = e.prefab.zip(link.as_ref());
        if let Some((o, (id, layout))) = origin
            && o.root != i
        {
            let root = spawned.get(o.root).map_or(0, |r| r.to_bits());
            let value = super::prefab::link_value(prefab_assets[o.prefab], o.node, root);
            components.push((*id, layout.encode(None, Some(&value))?));
        }
        let mut has_transform = false;
        // A `Model` table beside a `model` line: its fields, with the
        // line's asset (`model_table`).
        let mut model_table = None;
        // A cell's top-level entity: placed at the cell's pose after its own.
        let place = match placement {
            Placement::Cell { pose, .. } if e.parent.is_none() => Some(pose),
            _ => None,
        };
        for (t, layout) in &tables {
            if let Some(row) = t.get(i) {
                let mut value = migrate(&t.schema, row, layout, &mut dropped);
                if let (Some(pose), Some(tl)) = (place, &transform)
                    && tl.schema == layout.schema
                {
                    value = super::stream::cells::placed(pose, &value);
                }
                if e.instance.is_some()
                    && model_layout
                        .as_ref()
                        .is_some_and(|ml| ml.schema == layout.schema)
                {
                    model_table = Some(value);
                    continue;
                }
                let id = world
                    .id_by_name(&layout.schema)
                    .map_err(|e| e.reason().to_string())?;
                components.push((id, layout.encode(None, Some(&value))?));
                has_transform |= transform
                    .as_ref()
                    .is_some_and(|tl| tl.schema == layout.schema);
            }
        }
        if !has_transform && let Some(tl) = &transform {
            let id = world
                .id_by_name(&tl.schema)
                .map_err(|e| e.reason().to_string())?;
            let value = place.map(|pose| super::stream::cells::placed(pose, &json!({})));
            components.push((id, tl.encode(None, value.as_ref())?));
        }
        if let Some((id, bytes)) = &member {
            components.push((*id, bytes.clone()));
        }
        let parent = e.parent.and_then(|p| spawned.get(p).copied()).or(root);
        if let (Some(p), Some(pl)) = (parent, &parent_layout) {
            let id = world
                .id_by_name(&pl.schema)
                .map_err(|e| e.reason().to_string())?;
            components.push((
                id,
                pl.encode(None, Some(&json!({ "entity": p.to_bits() as f64 })))?,
            ));
        }
        if let Some(model) = &e.instance {
            let index = load_gltf(eng, model)
                .map_err(|r| format!("{}: entity '{}': {r}", scene.id, e.name))?;
            match &model_layout {
                Some(ml) => {
                    let world = eng.world()?;
                    let id = world
                        .id_by_name(&ml.schema)
                        .map_err(|e| e.reason().to_string())?;
                    // The scene's own Model fields, else the load's shadow
                    // flags (GltfDesc).
                    let value = match model_table.take() {
                        Some(mut v) => {
                            v["asset"] = json!(handle(index) as f64);
                            v
                        }
                        None => eng
                            .assets
                            .borrow()
                            .model_shadows(index)
                            .model_fields(handle(index)),
                    };
                    components.push((id, ml.encode(None, Some(&value))?));
                }
                None => dropped.push(format!("{}: og.Model is not registered", e.name)),
            }
        }
        let world = eng.world()?;
        let entity = world
            .spawn_bytes(components)
            .map_err(|e| e.reason().to_string())?;
        if let Some((o, (id, layout))) = origin
            && o.root == i
        {
            let value =
                super::prefab::link_value(prefab_assets[o.prefab], o.node, entity.to_bits());
            eng.world()?
                .insert_bytes(entity, *id, &layout.encode(None, Some(&value))?)
                .map_err(|e| e.reason().to_string())?;
        }
        if !e.name.is_empty() && !prefab {
            super::api::set_name(eng, entity, &e.name);
        }
        spawned.push(entity);
        if let Some(model) = &e.instance {
            let index = load_gltf(eng, model)?;
            spawn_lights(eng, index, entity)
                .map_err(|r| format!("{}: entity '{}': {r}", scene.id, e.name))?;
        }
    }
    if spawned.len() < scene.entities.len() {
        return Ok(());
    }
    // Sub-scenes, once the scene's own entities are in: a cell's handle
    // set for it to load; any other spawned under its entity now.
    for (i, e) in scene.entities.iter().enumerate() {
        let Some(sub) = &e.subscene else { continue };
        let entity = spawned[i];
        let is_cell = tables.iter().any(|(t, l)| {
            t.get(i).is_some() && cell_layout.as_ref().is_some_and(|cl| cl.schema == l.schema)
        });
        let index =
            load_scene(eng, sub).map_err(|r| format!("{}: sub-scene {sub}: {r}", scene.id))?;
        if is_cell {
            super::stream::cells::set_scene(eng, entity, handle(index))?;
            continue;
        }
        // Whether its pack has landed or not: only a cell loads a cell
        // group's sub-scene.
        if let Some(group) = eng.assets.borrow().stream.group_of_scene(index) {
            return Err(format!(
                "{}: entity '{}': sub-scene {sub} is in cell group '{group}', which only a cell loads; give the entity an og.StreamCell",
                scene.id, e.name
            ));
        }
        let desc = match &eng.assets.borrow().entries[index].payload {
            Payload::Scene(s) => s.clone(),
            _ => return Err(format!("{}: sub-scene {sub} is not a scene", scene.id)),
        };
        spawn_placed(eng, &desc, Placement::Under(Some(entity)))?;
    }
    apply_resources(eng, scene, &layouts, &mut dropped)?;
    if !dropped.is_empty() {
        eng.assets.borrow_mut().warnings.push(format!(
            "{}: dropped {}",
            scene.id,
            dropped.join("; ")
        ));
    }
    super::baked::scene_spawned(eng, scene, spawned);
    eng.state
        .borrow_mut()
        .scene_instances
        .push(super::engine::SceneInstance {
            id: scene.id.clone(),
            root: match placement {
                Placement::Under(root) => root.map(Entity::to_bits),
                Placement::Cell { cell, .. } => Some(cell.to_bits()),
            },
            entities: spawned.iter().map(|e| e.to_bits()).collect(),
        });
    Ok(())
}

/// A scene's `resource` blocks: each field it lists set on the world's
/// resource of that name (the others keep their values), migrated by field
/// name to the registered layout.
fn apply_resources(
    eng: &Engine,
    scene: &SceneDesc,
    layouts: &BTreeMap<String, Layout>,
    dropped: &mut Vec<String>,
) -> Result<(), String> {
    for r in &scene.resources {
        let Some(layout) = layouts
            .get(&r.name)
            .or_else(|| layouts.get(&format!("og.{}", r.name)))
            .filter(|l| l.kind == super::codec::RecordKind::Resource)
        else {
            dropped.push(format!(
                "resource {} (no registered resource of that name; list it in GameDesc.resources)",
                r.name
            ));
            continue;
        };
        let value = migrate(&r.schema, &r.bytes, layout, dropped);
        let world = eng.world()?;
        // A lazy resource (og.Lighting2D) a scene sets is registered then.
        super::lazy::ensure(eng, world, &layout.schema)?;
        let id = world
            .id_by_name(&layout.schema)
            .map_err(|e| e.reason().to_string())?;
        let base = world.resource_bytes(id).ok();
        let bytes = layout.encode(base.as_deref(), Some(&value))?;
        world
            .insert_resource_bytes(id, &bytes)
            .map_err(|e| e.reason().to_string())?;
    }
    Ok(())
}
