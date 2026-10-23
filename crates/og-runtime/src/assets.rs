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
use super::engine::{Engine, before_structural, hooks};
use super::extract::add_font;
use og_cook::runtime::{Bundle, CookedSound, SceneDesc};
use og_core::reflect::{FieldType, Schema};
use og_ecs::Entity;
use serde_json::{Map, Value, json};
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
    pub model_shadows: BTreeMap<usize, ModelShadows>,
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
            og_cook::runtime::asset_id(id)
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

/// `og.load.gltf`: the model's meshes, materials and textures join the
/// renderer once per asset id; the answer is the asset index.
pub(crate) fn load_gltf(eng: &Engine, path: &str) -> Result<usize, String> {
    let id = og_cook::runtime::asset_id(path).to_string();
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
    let (model, textures) = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&source)?;
        let model = bundle
            .model(&source, bytes.bytes())
            .map_err(|e| e.to_string())?;
        let mut textures = Vec::with_capacity(model.textures.len());
        for t in &model.textures {
            let (tb, bundle) = assets.blob(&t.id)?;
            let tex = bundle
                .texture(&t.id, tb.bytes())
                .map_err(|e| e.to_string())?;
            let data = match hooks().map(|h| h.texture(&tex, formats.as_ref())) {
                Some(Fetch::Ready(d)) => d,
                Some(Fetch::Pending) => {
                    return Err(format!("{}: the texture is still being transcoded", t.id));
                }
                Some(Fetch::Failed(e)) => return Err(format!("{}: {e}", t.id)),
                None => return Err("no host is installed to decode textures".into()),
            };
            textures.push(data);
        }
        (model, textures)
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
    let parts = model.add_to_with(&mut rs.scene, &textures);
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
    let payload = Payload::Gltf {
        scenes: model.scenes.len() as u32,
        geometries,
        materials,
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
    rs.fx.models.insert(
        handle(index),
        super::fx::ModelInfo {
            id: id.clone(),
            base,
            prim_mesh,
        },
    );
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
    // Physics keeps the proxy's meshes; the renderer's take the full model
    // whenever its group is resident.
    if let (Some(group), Some((meshes, materials))) = (group, stand_in) {
        super::stream::add_model(eng, &group, &id, meshes, materials);
    }
    Ok(index)
}

/// After a hot reload (`og dev` cooks before it rebuilds the bundle): every
/// open bundle's manifest is read again, and each glTF whose cooked outputs
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
    if models.is_empty() {
        return (Vec::new(), Vec::new());
    }
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
    let id = og_cook::runtime::asset_id(path).to_string();
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
    let pixel;
    let formats = texture_formats(eng);
    let data = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&id)?;
        let tex = bundle
            .texture(&id, bytes.bytes())
            .map_err(|e| e.to_string())?;
        pixel = tex.is_pixel_art();
        match hooks().map(|h| h.texture(&tex, formats.as_ref())) {
            Some(Fetch::Ready(d)) => d,
            Some(Fetch::Pending) => {
                return Err(format!("{id}: the texture is still being transcoded"));
            }
            Some(Fetch::Failed(e)) => return Err(format!("{id}: {e}")),
            None => return Err("no host is installed to decode textures".into()),
        }
    };
    let mut rs = eng.render.borrow_mut();
    let handle = rs.scene.add_texture_source(data);
    if pixel {
        rs.scene
            .set_texture_filter(handle, og_render::TextureFilter::Nearest);
    }
    rs.textures.push(handle);
    let i = rs.textures.len() - 1;
    eng.assets
        .borrow_mut()
        .by_path
        .insert(format!("texture:{id}"), i);
    Ok(i)
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
    let id = og_cook::runtime::asset_id(path).to_string();
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
    let id = og_cook::runtime::asset_id(path).to_string();
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
            let id = og_cook::runtime::chunk_id(&sound.id, n);
            let (bundle, dir) = assets.bundle_for(&id).ok()?;
            let out = bundle.output(&id).ok()?;
            Some((format!("{dir}/{}", out.file), out.stamp))
        })
        .collect();
    std::sync::Arc::new(move |n| {
        let (path, stamp) = files.get(n as usize)?.as_ref()?;
        let bytes = std::fs::read(path).ok()?;
        match og_cook::runtime::sound_chunk_of(&bytes, path, *stamp) {
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
        // A string moves by its text; the codec refuses one over the capacity.
        let strings = f.ty.is_string() && target.ty.is_string();
        if !strings && f.ty.lanes() != target.ty.lanes() {
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
    super::timeline::register_scene_paths(eng.id, &scene.paths);
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
        // A `Model` table beside a `model` line: its fields, with the
        // line's asset (`model_table`).
        let mut model_table = None;
        for (t, layout) in &tables {
            if let Some(row) = t.get(i) {
                let value = migrate(&t.schema, row, layout, &mut dropped);
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
            components.push((id, tl.encode(None, None)?));
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
