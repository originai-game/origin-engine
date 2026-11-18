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

use crate::engine::{Engine, before_structural, hooks};
use crate::host::codec::Layout;
use crate::present::extract::add_font;
use og_bundle::runtime::{Bundle, CookedSound, SceneDesc};
use og_core::IdMap;
use og_core::reflect::{FieldType, Schema};
use og_ecs::Entity;
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet, HashMap};

mod models;
mod release;
mod sound;
mod spawn;
mod textures;

pub(crate) use models::*;
pub(crate) use release::release_bundle;
pub(crate) use release::resolve_releases;
pub(crate) use sound::*;
pub(crate) use spawn::*;
pub(crate) use textures::*;

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
    Scene(Box<LoadedScene>),
    /// A cooked scene whose ungrouped blob is being fetched by the host.
    ScenePending,
    /// A sub-scene whose blob is in a cell group's pack (G3-3), not yet
    /// landed: its handle exists (a cell names it), its scene once the
    /// pack's bytes are in ([`crate::world::stream::resolve_scenes`]).
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
    /// An uncooked load not yet in place (crate::world::uncooked).
    Uncooked,
    /// An uncooked environment in place: its index in the render state's
    /// environment maps (its EnvironmentMap handle's).
    Environment {
        map: usize,
        width: u32,
        height: u32,
        size: u32,
    },
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
    /// The cooked bundle this load resolved to (none for uncooked loads).
    pub bundle: Option<usize>,
    pub status: Status,
    pub payload: Payload,
}

/// A decoded scene keeps its source for every dependency it places.
#[derive(Clone)]
pub(crate) struct LoadedScene {
    pub bundle: usize,
    pub desc: SceneDesc,
}

impl std::ops::Deref for LoadedScene {
    type Target = SceneDesc;
    fn deref(&self) -> &SceneDesc {
        &self.desc
    }
}

pub(crate) struct LoadedFont {
    pub id: String,
    pub info: Value,
    /// Index into the render state's fonts.
    pub render: usize,
    /// The font's file, when it ships one (a shaped script, `runtime
    /// true`): og-ui shapes with it and measures characters outside the
    /// atlas, which `crate::present::glyphs` renders.
    pub file: Option<std::sync::Arc<og_ui::shape::FontFile>>,
    /// An icon set's names (glyph U+E000 + i is icon i); empty for a text font.
    pub icons: Vec<String>,
    /// A pixel font's grid (`font { pixel <grid> }`); 0 for an outline font.
    pub pixel: u32,
}

/// Everything an engine loaded.
#[derive(Default)]
pub(crate) struct Assets {
    /// The project directory (native) or URL base (web); empty: none.
    pub root: String,
    pub entries: Vec<Entry>,
    /// Bundle closes requested while systems hold the world.
    pub pending_releases: BTreeSet<usize>,
    /// The game's own bundle (`cooked`).
    pub default_bundle: Option<usize>,
    pub by_path: BTreeMap<String, usize>,
    pub fonts: Vec<LoadedFont>,
    pub sounds: Vec<CookedSound>,
    /// Each sound's handle in the engine's audio bank, by `sounds` index:
    /// what og.load.sound answers and AudioSource.sound names.
    pub sound_handles: Vec<u64>,
    /// og.load.style's styles, by handle index.
    pub styles: Vec<crate::bind::modules::LoadedStyle>,
    pub warnings: Vec<String>,
    /// The game bundle's streaming groups.
    pub stream: crate::world::stream::Streams,
    /// og.load.textureArray's answers, by asset id.
    pub texture_arrays: BTreeMap<String, Value>,
    /// Per glTF asset index, the GltfDesc shadow flags of its last
    /// og.load.gltf ([`Assets::model_shadows`]).
    pub model_shadows: IdMap<usize, ModelShadows>,
    /// Uncooked loads (og.load.imageData) and their journal.
    pub uncooked: crate::world::uncooked::Uncooked,
    /// The http(s) images documents name, by request key.
    pub remote: crate::world::remote::Remote,
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

pub(crate) fn join(root: &str, path: &str) -> String {
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
            bundle: None,
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
                    if let Err(e) = self.stream.add_bundle(i, &bundle, &dir) {
                        entry.status = Status::Failed(format!("{manifest}: {e}"));
                        return;
                    }
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
        let dir = join(&self.root, path);
        if let Some(i) = self.entries.iter().position(|e| {
            matches!(&e.payload, Payload::Bundle { dir: opened, .. } if opened == &dir)
                || (e.path == path && e.status == Status::Pending)
        }) {
            return i;
        }
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
            } else if matches!(self.entries[i].payload, Payload::ScenePending) {
                let entry = &self.entries[i];
                let decoded = decode_scene(
                    self,
                    &entry.path,
                    entry.bundle.expect("cooked scene's bundle"),
                );
                match decoded {
                    Ok(Some(scene)) => {
                        self.entries[i].payload = Payload::Scene(Box::new(scene));
                        self.entries[i].status = Status::Ready;
                    }
                    Ok(None) => {}
                    Err(e) => {
                        self.entries[i].payload = Payload::None;
                        self.entries[i].status = Status::Failed(e);
                    }
                }
            }
        }
    }

    /// The requester's bundle first, then the boot bundle and open order.
    pub(crate) fn source(&self, id: &str, owner: Option<usize>) -> Result<usize, String> {
        let order = owner
            .into_iter()
            .chain(self.default_bundle)
            .chain(0..self.entries.len());
        let mut open = 0;
        for i in order {
            if let Payload::Bundle { bundle, .. } = &self.entries[i].payload {
                open += 1;
                if bundle.output(id).is_ok() {
                    return Ok(i);
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

    pub(crate) fn bundle_from(
        &self,
        id: &str,
        owner: Option<usize>,
    ) -> Result<(&Bundle, &str), String> {
        let i = self.source(id, owner)?;
        let Payload::Bundle { bundle, dir } = &self.entries[i].payload else {
            unreachable!()
        };
        Ok((bundle, dir))
    }

    /// A direct load has no requester: boot first, then open order.
    pub(crate) fn bundle_for(&self, id: &str) -> Result<(&Bundle, &str), String> {
        self.bundle_from(id, None)
    }

    /// The bytes of a cooked output, 16-aligned.
    pub(crate) fn blob(&self, id: &str) -> Result<(og_blob_bytes::Aligned, &Bundle), String> {
        self.blob_from(id, None)
    }

    pub(crate) fn blob_from(
        &self,
        id: &str,
        owner: Option<usize>,
    ) -> Result<(og_blob_bytes::Aligned, &Bundle), String> {
        self.fetch_blob_from(id, owner)?
            .ok_or_else(|| format!("{id} is still being fetched; load it once its bundle is ready"))
    }

    /// A pending fetch is not a rejected blob: asynchronous scene loads
    /// keep their handle and retry only the pending read at a tick boundary.
    fn fetch_blob_from(
        &self,
        id: &str,
        owner: Option<usize>,
    ) -> Result<Option<(og_blob_bytes::Aligned, &Bundle)>, String> {
        let (bundle, dir) = self.bundle_from(id, owner)?;
        let out = bundle.output(id).map_err(|e| e.to_string())?;
        if let Some(group) = &out.group {
            // A streamed output is read from its group's pack, once resident.
            return Ok(Some((aligned(self.stream.file(group, &out.file)?), bundle)));
        }
        let path = format!("{dir}/{}", out.file);
        let start = crate::author::trace::start();
        match read(&path) {
            Fetch::Ready(bytes) => {
                crate::author::trace::load(id, start, bytes.len());
                Ok(Some((og_blob_bytes::Aligned::new(bytes), bundle)))
            }
            Fetch::Pending => Ok(None),
            Fetch::Failed(e) => Err(format!("{path}: {e}")),
        }
    }

    /// Whether a cooked output's file is still being fetched (the web
    /// reads some on demand: a font pack's faces). Asking starts the fetch.
    pub(crate) fn pending(&self, id: &str) -> bool {
        let Ok((bundle, dir)) = self.bundle_for(id) else {
            return false;
        };
        match bundle.output(id) {
            Ok(out) if out.group.is_none() => {
                matches!(read(&format!("{dir}/{}", out.file)), Fetch::Pending)
            }
            _ => false,
        }
    }

    /// The streaming group of a cooked output (`None`: loaded at boot).
    pub(crate) fn group_of(&self, id: &str) -> Option<String> {
        self.group_from(id, None)
    }

    pub(crate) fn group_from(&self, id: &str, owner: Option<usize>) -> Option<String> {
        let (bundle, _) = self.bundle_from(id, owner).ok()?;
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

/// After a hot reload (`og dev` cooks before it rebuilds the bundle): every
/// open bundle's manifest is read again (whatever is loaded: timelines
/// refresh against it too), and each glTF whose cooked outputs
/// changed (the model or its `#` textures, by hash) loads again into its own
/// asset index, so Model components keep their handle and draw the edit.
/// The old version's meshes and materials stay in the render scene, unused.
/// A manifest the host cannot read at once (a web fetch in flight) keeps its
/// bundle as it was. Answers the ids loaded again, and why any failed.
pub(crate) fn refresh(eng: &Engine) -> (Vec<String>, Vec<String>) {
    fn outputs(assets: &Assets, id: &str, owner: usize) -> Vec<u64> {
        let Ok((bundle, _)) = assets.bundle_from(id, Some(owner)) else {
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
    let models: Vec<(usize, usize, String, Vec<u64>)> = {
        let assets = eng.assets.borrow();
        assets
            .by_path
            .iter()
            .filter_map(|(key, &i)| {
                if !key.starts_with("gltf:") || !eng.render.borrow().models.contains_key(&handle(i))
                {
                    return None;
                }
                let entry = &assets.entries[i];
                let owner = entry.bundle?;
                Some((
                    i,
                    owner,
                    entry.path.clone(),
                    outputs(&assets, &entry.path, owner),
                ))
            })
            .collect()
    };
    // The textures models share (assets the models name, not their own
    // `#` outputs): decoded again in place when theirs changed, so every
    // model naming one draws the edit.
    let shared: Vec<(og_render::TextureHandle, usize, String, Vec<u64>)> = {
        let (rs, assets) = (eng.render.borrow(), eng.assets.borrow());
        rs.shared
            .iter()
            .flat_map(|(&owner, shared)| shared.textures().map(move |(id, h)| (owner, id, h)))
            .map(|(owner, id, h)| (h, owner, id.to_string(), outputs(&assets, id, owner)))
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
    for (h, owner, id, before) in shared {
        if outputs(&eng.assets.borrow(), &id, owner) == before {
            continue;
        }
        let decoded = decode_from(eng, &id, Some(owner));
        let mut rs = eng.render.borrow_mut();
        match decoded {
            Fetch::Ready((source, ..)) => {
                crate::world::stream::replace_texture(&mut rs.scene, h, source)
            }
            Fetch::Pending => {
                rs.scene.set_texture(h, crate::world::stream::placeholder());
                rs.decoded.pending.push((h, id.clone()));
            }
            Fetch::Failed(e) => failed.push(format!("{id}: {e}")),
        }
        drop(rs);
        crate::world::stream::mips::reset(eng, h);
    }
    for (index, owner, id, before) in models {
        let now = outputs(&eng.assets.borrow(), &id, owner);
        if now.is_empty() || now == before {
            continue;
        }
        let shared = !eng
            .render
            .borrow()
            .hlod
            .proxies
            .contains_key(&handle(index));
        match add_gltf(eng, &id, owner, Some(index), shared) {
            Ok(_) => loaded.push(id),
            Err(e) => failed.push(format!("{id}: {e}")),
        }
    }
    (loaded, failed)
}

/// `og.load.scene`: decoded and kept; `instantiate` spawns it.
pub(crate) fn load_scene(eng: &Engine, path: &str) -> Result<usize, String> {
    load_scene_from(eng, path, None)
}

pub(crate) fn load_scene_from(
    eng: &Engine,
    path: &str,
    owner: Option<usize>,
) -> Result<usize, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    let bundle = eng.assets.borrow().source(&id, owner)?;
    let key = format!("scene:{bundle}:{id}");
    if let Some(&i) = eng.assets.borrow().by_path.get(&key) {
        return Ok(i);
    }
    let group = eng.assets.borrow().group_from(&id, Some(bundle));
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
        assets.entries[index].bundle = Some(bundle);
        assets.by_path.insert(key, index);
        assets.stream.add_scene(&group, index);
        drop(assets);
        crate::world::stream::resolve_scenes(eng, Some(&group));
        return Ok(index);
    }
    let scene = decode_scene(&eng.assets.borrow(), &id, bundle);
    let mut assets = eng.assets.borrow_mut();
    let (status, payload) = match scene {
        Ok(Some(scene)) => (Status::Ready, Payload::Scene(Box::new(scene))),
        Ok(None) => (Status::Pending, Payload::ScenePending),
        Err(e) => (Status::Failed(e), Payload::None),
    };
    let index = assets.push(&id, status, payload);
    assets.entries[index].bundle = Some(bundle);
    assets.by_path.insert(key, index);
    Ok(index)
}

/// A cooked scene's blob decoded (from its group's pack when it streams).
pub(crate) fn decode_scene(
    assets: &Assets,
    id: &str,
    owner: usize,
) -> Result<Option<LoadedScene>, String> {
    let Some((bytes, bundle)) = assets.fetch_blob_from(id, Some(owner))? else {
        return Ok(None);
    };
    let desc = bundle.scene(id, bytes.bytes()).map_err(|e| e.to_string())?;
    Ok(Some(LoadedScene {
        bundle: owner,
        desc,
    }))
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
        "icons": font.names.len(), "pixel": font.pixel,
    });
    let mut rs = eng.render.borrow_mut();
    let pages = add_font(&mut rs.scene, &font);
    rs.fonts.push(pages);
    let render = rs.fonts.len() - 1;
    let file = font
        .ttf
        .clone()
        .and_then(|t| og_ui::shape::FontFile::new(std::sync::Arc::new(t)))
        .or_else(|| {
            font.shaping
                .clone()
                .and_then(|t| og_ui::shape::FontFile::for_shaping(std::sync::Arc::new(t)))
        });
    if font.ttf.is_some()
        && let Some(f) = &file
    {
        rs.runtime_fonts.insert(
            render,
            crate::present::glyphs::RuntimeFont::new(
                f.clone(),
                m.em_px,
                m.px_range,
                font.pixel,
                font.color_runtime.clone(),
            ),
        );
    }
    let mut assets = eng.assets.borrow_mut();
    assets.fonts.push(LoadedFont {
        id,
        info,
        render,
        file,
        icons: font.names.clone(),
        pixel: font.pixel,
    });
    Ok(assets.fonts.len() - 1)
}
