//! Cooked bundles, loads and scene spawning (IDL `load`, `World.instantiate`).
//!
//! The runtime reads cooked output through the host (`Hooks::read_asset`):
//! the native host reads files under the project, the web host answers from
//! the files it fetched before the engine started (and fetches a later
//! bundle's on request, which completes at a tick boundary). Textures are
//! decoded through `Hooks::texture`: og-cook's native transcoder, or the web
//! host's Basis transcoder.
//!
//! Handles are `index + 2^32` (generation 1), like materials, so 0 is never
//! a live handle and a stale number cannot alias a later asset.

use super::codec::Layout;
use super::engine::{before_structural, hooks, Engine};
use super::extract::add_font;
use og_cook::runtime::{Bundle, CookedSound, SceneDesc};
use og_core::reflect::{FieldType, Schema};
use og_ecs::Entity;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

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
    Gltf {
        scenes: u32,
        geometries: Vec<u64>,
        materials: Vec<u64>,
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
    pub warnings: Vec<String>,
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
    fn bundle_for(&self, id: &str) -> Result<(&Bundle, &str), String> {
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
            og_cook::runtime::asset_id(id)
        ))
    }

    /// The bytes of a cooked output, 16-aligned.
    pub(crate) fn blob(&self, id: &str) -> Result<(og_blob_bytes::Aligned, &Bundle), String> {
        let (bundle, dir) = self.bundle_for(id)?;
        let out = bundle.output(id).map_err(|e| e.to_string())?;
        let path = format!("{dir}/{}", out.file);
        match read(&path) {
            Fetch::Ready(bytes) => Ok((og_blob_bytes::Aligned::new(bytes), bundle)),
            Fetch::Pending => Err(format!(
                "{path} is still being fetched; load it once its bundle is ready"
            )),
            Fetch::Failed(e) => Err(format!("{path}: {e}")),
        }
    }
}

/// 16-aligned bytes for blob views.
mod og_blob_bytes {
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

/// `og.load.gltf`: the model's meshes, materials and textures join the
/// renderer once per asset id; the answer is the asset index.
pub(crate) fn load_gltf(eng: &Engine, path: &str) -> Result<usize, String> {
    let id = og_cook::runtime::asset_id(path).to_string();
    if let Some(&i) = eng.assets.borrow().by_path.get(&format!("gltf:{id}")) {
        return Ok(i);
    }
    let (model, textures) = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&id)?;
        let model = bundle
            .model(&id, bytes.bytes())
            .map_err(|e| e.to_string())?;
        let mut textures = Vec::with_capacity(model.textures.len());
        for t in &model.textures {
            let (tb, bundle) = assets.blob(&t.id)?;
            let tex = bundle
                .texture(&t.id, tb.bytes())
                .map_err(|e| e.to_string())?;
            let data = match hooks().map(|h| h.texture(&tex)) {
                Some(Fetch::Ready(d)) => d,
                Some(Fetch::Pending) => {
                    return Err(format!("{}: the texture is still being transcoded", t.id))
                }
                Some(Fetch::Failed(e)) => return Err(format!("{}: {e}", t.id)),
                None => return Err("no host is installed to decode textures".into()),
            };
            textures.push(data);
        }
        (model, textures)
    };
    let mut rs = eng.render.borrow_mut();
    let parts = model.add_to(&mut rs.scene, &textures);
    let mut geometries = Vec::new();
    let mut materials = Vec::new();
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
        }
        if !materials.contains(&handle(m)) {
            materials.push(handle(m));
        }
    }
    let mut assets = eng.assets.borrow_mut();
    let index = assets.push(
        &id,
        Status::Ready,
        Payload::Gltf {
            scenes: model.scenes.len() as u32,
            geometries,
            materials,
        },
    );
    rs.models.insert(handle(index), parts.parts);
    assets.by_path.insert(format!("gltf:{id}"), index);
    // Its geometries become physics meshes under their Geometry handles, for
    // convex and triangle-mesh colliders.
    let meshes: Vec<(u64, og_physics::MeshDesc)> = rs
        .geometries
        .iter()
        .enumerate()
        .filter_map(|(g, m)| Some((handle(g), super::modules::mesh_of(rs.scene.mesh(*m)?))))
        .collect();
    drop(assets);
    drop(rs);
    for (h, mesh) in meshes {
        if !eng.state.borrow().physics_meshes.contains_key(&h) {
            let _ = super::modules::set_mesh(eng, h, mesh);
        }
    }
    Ok(index)
}

/// `og.load.scene`: decoded and kept; `instantiate` spawns it.
pub(crate) fn load_scene(eng: &Engine, path: &str) -> Result<usize, String> {
    let id = og_cook::runtime::asset_id(path).to_string();
    if let Some(&i) = eng.assets.borrow().by_path.get(&format!("scene:{id}")) {
        return Ok(i);
    }
    let scene = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&id)?;
        bundle
            .scene(&id, bytes.bytes())
            .map_err(|e| e.to_string())?
    };
    let mut assets = eng.assets.borrow_mut();
    let index = assets.push(&id, Status::Ready, Payload::Scene(Box::new(scene)));
    assets.by_path.insert(format!("scene:{id}"), index);
    Ok(index)
}

pub(crate) fn load_font(eng: &Engine, path: &str) -> Result<usize, String> {
    let id = og_cook::runtime::asset_id(path).to_string();
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
    });
    let mut rs = eng.render.borrow_mut();
    let pages = add_font(&mut rs.scene, &font);
    rs.fonts.push(pages);
    let render = rs.fonts.len() - 1;
    let mut assets = eng.assets.borrow_mut();
    assets.fonts.push(LoadedFont { id, info, render });
    Ok(assets.fonts.len() - 1)
}

/// `og.load.texture`: a cooked texture added to the renderer once; the
/// answer is its index in the render state's textures.
pub(crate) fn load_texture(eng: &Engine, path: &str) -> Result<usize, String> {
    let id = og_cook::runtime::asset_id(path).to_string();
    if let Some(&i) = eng.assets.borrow().by_path.get(&format!("texture:{id}")) {
        return Ok(i);
    }
    let data = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&id)?;
        let tex = bundle
            .texture(&id, bytes.bytes())
            .map_err(|e| e.to_string())?;
        match hooks().map(|h| h.texture(&tex)) {
            Some(Fetch::Ready(d)) => d,
            Some(Fetch::Pending) => {
                return Err(format!("{id}: the texture is still being transcoded"))
            }
            Some(Fetch::Failed(e)) => return Err(format!("{id}: {e}")),
            None => return Err("no host is installed to decode textures".into()),
        }
    };
    let mut rs = eng.render.borrow_mut();
    let handle = rs.scene.add_texture(data);
    rs.textures.push(handle);
    let i = rs.textures.len() - 1;
    eng.assets
        .borrow_mut()
        .by_path
        .insert(format!("texture:{id}"), i);
    Ok(i)
}

pub(crate) fn load_sound(eng: &Engine, path: &str) -> Result<usize, String> {
    let id = og_cook::runtime::asset_id(path).to_string();
    if let Some(i) = eng.assets.borrow().sounds.iter().position(|s| s.id == id) {
        return Ok(i);
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
        Some(link) => link
            .bank()
            .insert_qoa(&sound.id, &sound.qoa)
            .map_err(|e| e.reason().to_string())?,
        None => return Err(format!("{id}: this engine has no audio")),
    };
    let mut assets = eng.assets.borrow_mut();
    assets.sounds.push(sound);
    assets.sound_handles.push(handle);
    Ok(assets.sounds.len() - 1)
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
    let lanes: Vec<f64> = (0..ty.lanes())
        .map(|i| {
            let at = i * ty.scalar().size();
            let b = &bytes[at..at + ty.scalar().size()];
            match ty.name() {
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
fn migrate(schema: &Schema, row: &[u8], layout: &Layout, dropped: &mut Vec<String>) -> Value {
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
        if f.ty.lanes() != target.ty.lanes() {
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

/// Spawns scene entity rows under `root` (top level when `None`). The whole
/// scene is refused before anything spawns when a name is taken.
pub(crate) fn spawn_scene(
    eng: &Engine,
    scene: &SceneDesc,
    root: Option<Entity>,
) -> Result<Vec<Entity>, String> {
    {
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
    let layouts = eng.state.borrow().layouts.clone();
    let transform = resolve(&layouts, "Transform").cloned();
    let parent_layout = resolve(&layouts, "Parent").cloned();
    let model_layout = resolve(&layouts, "Model").cloned();
    let mut dropped = Vec::new();
    let mut tables = Vec::new();
    for t in &scene.components {
        match resolve(&layouts, &t.name) {
            Some(l) => tables.push((t, l.clone())),
            None => dropped.push(format!(
                "{} (no registered component of that name; declare it with defineComponent before the scene loads)",
                t.name
            )),
        }
    }
    let mut spawned: Vec<Entity> = Vec::with_capacity(scene.entities.len());
    before_structural(eng);
    for (i, e) in scene.entities.iter().enumerate() {
        let world = eng.world()?;
        let mut components = Vec::new();
        let mut has_transform = false;
        for (t, layout) in &tables {
            if let Some(row) = t.get(i) {
                let value = migrate(&t.schema, row, layout, &mut dropped);
                let id = world
                    .id_by_name(&layout.schema)
                    .map_err(|e| e.reason().to_string())?;
                components.push((id, layout.encode(None, Some(&value))?));
                has_transform |= transform
                    .as_ref()
                    .is_some_and(|tl| tl.schema == layout.schema);
            }
        }
        if !has_transform {
            if let Some(tl) = &transform {
                let id = world
                    .id_by_name(&tl.schema)
                    .map_err(|e| e.reason().to_string())?;
                components.push((id, tl.encode(None, None)?));
            }
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
                    components.push((
                        id,
                        ml.encode(None, Some(&json!({ "asset": handle(index) as f64 })))?,
                    ));
                }
                None => dropped.push(format!("{}: og.Model is not registered", e.name)),
            }
        }
        let world = eng.world()?;
        let entity = world
            .spawn_bytes(components)
            .map_err(|e| e.reason().to_string())?;
        if !e.name.is_empty() {
            super::api::set_name(eng, entity, &e.name);
        }
        spawned.push(entity);
    }
    if !dropped.is_empty() {
        eng.assets.borrow_mut().warnings.push(format!(
            "{}: dropped {}",
            scene.id,
            dropped.join("; ")
        ));
    }
    Ok(spawned)
}
