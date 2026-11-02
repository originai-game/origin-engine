//! The cook: `assets/` → `cooked/`, incremental by hash.
//!
//! [`cook_project`] walks `<root>/assets`, classifies each file by extension
//! (`.glb`/`.gltf` models, `.png`/`.jpg` textures, `.wav`/`.mp3`/`.ogg`/`.flac` sounds,
//! `.ttf`/`.otf` fonts, `.ogt` scenes; `<file>.ogt` of type `import` holds a
//! file's import settings), computes every output's cook stamp (FNV-1a of the
//! cooker version, the options that kind of output depends on and its own
//! inputs) and cooks only the outputs whose stamp the manifest
//! (`cooked/.cook.ogt`) does not already hold with an intact file. Outputs
//! are written only when their bytes change; outputs of removed or failed
//! sources are deleted. The same inputs give byte-identical `cooked/`.

pub mod anim;
pub mod audio;
pub mod bc6h;
pub mod conform;
pub use conform::{ConformAction, ConformEntry};
pub mod cube_source;
mod e6;
mod environment;
mod flipbook;
mod font;
pub mod icons;
pub mod ktx2_source;
mod lightmap;
pub mod look;
mod look_assets;
mod look_licences;
mod look_styles;
mod model;
pub mod model_text;
pub mod probes;
mod psd;
mod rig2d;
mod scene;
pub mod sfx;
mod shader_check;
pub mod stats;
mod stream;
mod terrain;
mod terrain_generate;
mod texture;
pub mod texture_array;
mod texture_cache;
pub mod tilemap;
mod timeline;
pub mod ui_screens;
pub mod vector;
pub mod video;
pub mod vox;

mod execute;
mod options;
mod report;
mod scan;

pub use execute::slice_tone;
use execute::{read_sibling, run, style_atlas};
pub use options::{CookOptions, FontSettings, MeshOptions, TextureOptions};
use options::{
    anim_settings, conform_off, font_settings, mesh_for, mesh_texture, own_conform_off,
    settings_block, texture_kind,
};
pub use report::{AssetReport, COOK_TIMES, CookTimes, KindTotals, OutputReport, Report};
use report::{CONFORM_LOG, ConformLog, conform_log_text, conform_note, read_conform_log};
use scan::{Scan, ext, output_group, plan_groups, scan};

use crate::format::{self, BlobKind, COOKER_VERSION};
use crate::runtime::{MANIFEST, Manifest, ManifestAsset, ManifestOutput};
use og_core::Schema;
use og_core::hash::{StableHasher, hash_bytes};
use og_core::ogt::Document;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Instant;

/// A cook failure, written for the person reading it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CookError(pub String);

impl fmt::Display for CookError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for CookError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Model,
    Texture,
    Sound,
    Font,
    Scene,
    Lut,
    Emitter,
    Style,
    Document,
    Tileset,
    Tilemap,
    Flipbook,
    Rig2d,
    Shader,
    Post,
    UiMaterial,
    Vector,
    Behaviour,
    NavMesh,
    Strings,
    Rig,
    Terrain,
    Timeline,
    Lightmap,
    Environment,
    Icons,
    Look,
    Palette,
    TextureArray,
    Video,
}

impl Kind {
    fn name(self) -> &'static str {
        match self {
            Kind::Model => "model",
            Kind::Texture => "texture",
            Kind::Sound => "sound",
            Kind::Font => "font",
            Kind::Scene => "scene",
            Kind::Lut => "lut",
            Kind::Emitter => "emitter",
            Kind::Style => "style",
            Kind::Document => "document",
            Kind::Tileset => "tileset",
            Kind::Tilemap => "tilemap",
            Kind::Flipbook => "flipbook",
            Kind::Rig2d => "rig2d",
            Kind::Shader => "shader",
            Kind::Post => "post",
            Kind::UiMaterial => "ui-material",
            Kind::Vector => "vector",
            Kind::Behaviour => "behaviour",
            Kind::NavMesh => "navmesh",
            Kind::Strings => "strings",
            Kind::Rig => "rig",
            Kind::Terrain => "terrain",
            Kind::Timeline => "timeline",
            Kind::Lightmap => "lightmap",
            Kind::Environment => "environment",
            Kind::Icons => "icons",
            Kind::Look => "look",
            Kind::Palette => "palette",
            Kind::TextureArray => "texture-array",
            Kind::Video => "video",
        }
    }
}

struct Source {
    rel: String,
    path: PathBuf,
    kind: Kind,
    bytes: Vec<u8>,
    settings: Option<(String, Document)>,
    hash: u64,
    gltf: Option<model::GltfSource>,
    error: Option<String>,
    /// A sound file's QOA encoding, made once by the first of its outputs to cook
    /// (a sound cooked in chunks has one output per chunk).
    encoded: std::sync::OnceLock<Result<std::sync::Arc<audio::Encoded>, String>>,
    /// The image the art contract conformed it to (`conform/`): a
    /// texture's own, a flipbook's texture's; read instead of the file,
    /// which is never written. `None`: as on disk.
    conformed: Option<std::sync::Arc<Vec<u8>>>,
}

impl Source {
    fn encoded(&self, per_chunk: Option<u32>) -> Result<std::sync::Arc<audio::Encoded>, CookError> {
        self.encoded
            .get_or_init(|| {
                audio::decode(&self.rel, &self.bytes)
                    .and_then(|pcm| audio::encode(&pcm, per_chunk))
                    .map(std::sync::Arc::new)
            })
            .clone()
            .map_err(CookError)
    }
}

enum Work<'a> {
    Model(&'a Source, Vec<String>),
    /// A streaming group's glTF stand-in: its coarsest LOD, no textures.
    Proxy(&'a Source),
    Animation(&'a Source, anim::AnimSettings),
    GltfTexture(&'a Source, usize, u32),
    Texture(&'a Source, u32),
    /// A sound, whole or (QOA frames per chunk, chunks) the first of its chunks.
    Sound(&'a Source, Option<(u32, u32)>),
    /// Chunk `n` (1 and up) of a sound cooked in chunks of so many QOA frames.
    SoundChunk(&'a Source, u32, u32),
    Lut(&'a Source),
    Emitter(&'a Source),
    Style(&'a Source),
    Document(&'a Source),
    Tileset(&'a Source),
    Tilemap(&'a Source, tilemap::MapSettings),
    Flipbook(&'a Source),
    /// An `array true` sheet's texture array blob, and its layer `n`.
    FlipbookArray(&'a Source, Box<og_anim::FlipbookSheet>),
    FlipbookLayer(&'a Source, Box<og_anim::FlipbookSheet>, usize),
    Rig2d(&'a Source),
    Shader(&'a Source),
    Post(&'a Source),
    UiMaterial(&'a Source),
    Vector(&'a Source),
    Behaviour(&'a Source),
    NavMesh(&'a Source),
    Strings(&'a Source),
    Rig(&'a Source),
    Terrain(&'a Source),
    Timeline(&'a Source),
    Lightmap(&'a Source),
    Environment(&'a Source),
    Icons(&'a Source),
    Look(&'a Source),
    Palette(&'a Source),
    /// A texture array's blob, and layer `n`'s texture.
    TextureArray(&'a Source, texture_array::ArrayDecl),
    ArrayLayer(&'a Source, texture_array::ArrayDecl, usize),
    /// A video's blob, its frames' texture array, frame `n` (with the
    /// source's path and hash) and its sound.
    VideoMeta(Box<video::VideoPlan>),
    VideoArray(&'a Source, Box<video::VideoPlan>),
    VideoFrame(PathBuf, u64, Box<video::VideoPlan>, usize),
    VideoAudio(PathBuf, Box<video::VideoPlan>),
    Font(&'a Source, FontSettings),
    Scene(&'a Source),
}

struct Planned<'a> {
    source: usize,
    id: String,
    file: String,
    ty: &'static str,
    stamp: u64,
    work: Option<Work<'a>>,
}

fn output_file(id: &str) -> String {
    match id.split_once('#') {
        Some((base, sub)) => format!("{base}.{sub}.ogb"),
        None => format!("{id}.ogb"),
    }
}

/// The node rows' layout, in the stamp of every blob that has them (a model,
/// its proxy, its `#anim`): `NodeRow` version 2 carries `visible`.
const NODE_ROWS: &[u8] = b"nodes-v2";

fn stamp(parts: &[&[u8]]) -> u64 {
    let mut h = StableHasher::new();
    h.write_str(COOKER_VERSION);
    for p in parts {
        h.write_u64(p.len() as u64);
        h.write(p);
    }
    h.finish()
}

fn assets_root(s: &Source) -> PathBuf {
    let mut root = s.path.clone();
    for _ in s.rel.split('/') {
        root.pop();
    }
    root
}

/// Cooks `<root>/assets` into `<root>/cooked`. Per-asset failures are in the
/// report ([`Report::ok`]); an `Err` means the cook could not run at all.
pub fn cook_project(root: &Path, options: &CookOptions) -> Result<Report, CookError> {
    let cache = texture_cache::default_path();
    cook_project_with_texture_cache(root, options, cache.as_deref())
}

/// Cook with an explicit shared texture cache directory, or `None` to disable it.
/// This does not read cache environment variables; useful for isolated callers/tests.
pub fn cook_project_with_texture_cache(
    root: &Path,
    options: &CookOptions,
    texture_cache: Option<&Path>,
) -> Result<Report, CookError> {
    let started = Instant::now();
    let assets_dir = root.join("assets");
    let cooked_dir = root.join("cooked");
    let threads = if options.threads == 0 {
        std::thread::available_parallelism().map_or(4, |n| n.get())
    } else {
        options.threads
    };
    let texture_encoder = texture::encoder_name(&options.texture);
    // The project's budgets (project.ogt's `budgets { }`, the defaults
    // without one): a `type model` past them is cooked with a warning.
    let budgets = std::fs::read_to_string(root.join("project.ogt"))
        .ok()
        .and_then(|t| Document::parse(&t).ok())
        .and_then(|d| stats::Budgets::from_project(&d.body).ok())
        .unwrap_or_default();
    let budget_desc = format!(
        "budgets triangles {} vertices {}",
        budgets.triangles, budgets.vertices
    );

    let Scan {
        mut sources,
        mut ignored,
        mut early_errors,
    } = scan(&assets_dir)?;
    let mut consumed: BTreeSet<String> = BTreeSet::new();
    // The `type model` texts, built again under the art contract's face
    // budget once the look is read (below).
    let mut model_texts: BTreeMap<String, String> = BTreeMap::new();
    for s in sources.iter_mut() {
        // A model that says it is a character (`conform character`), read
        // from its text or import settings before a text becomes a glb.
        let character = if s.kind == Kind::Model {
            match conform::character(s) {
                Ok(c) => c,
                Err(e) => {
                    s.error = Some(e);
                    continue;
                }
            }
        } else {
            false
        };
        // A MagicaVoxel file is a model: meshed into a glTF binary here (its
        // import settings' `model { voxScale voxOrigin }` applied), so the
        // rest of the cook sees a .glb.
        if s.kind == Kind::Model && ext(&s.rel) == "vox" {
            match vox::VoxSettings::parse(settings_block(s, "model"))
                .and_then(|v| vox::to_glb(&s.bytes, &v))
            {
                Ok(glb) => s.bytes = glb,
                Err(e) => {
                    s.error = Some(e);
                    continue;
                }
            }
        }
        // A model written as text (`type model`): its parts built and
        // written as a glTF binary here, so the rest of the cook sees a .glb.
        if s.kind == Kind::Model && ext(&s.rel) == "ogt" {
            let text = String::from_utf8_lossy(&s.bytes).to_string();
            match model_text::to_glb(&text) {
                Ok(glb) => {
                    s.bytes = glb;
                    model_texts.insert(s.rel.clone(), text);
                }
                Err(e) => {
                    s.error = Some(e);
                    continue;
                }
            }
        }
        if s.kind == Kind::Model {
            let dir = s.path.parent().unwrap_or(Path::new("."));
            match model::load(&s.bytes, dir) {
                Ok(mut g) => {
                    g.character = character;
                    let base = s
                        .rel
                        .rsplit_once('/')
                        .map_or(String::new(), |(d, _)| format!("{d}/"));
                    for (p, _) in &g.external {
                        consumed.insert(format!("{base}{p}"));
                    }
                    s.gltf = Some(g);
                }
                Err(e) => s.error = Some(e),
            }
        }
        let mut h = StableHasher::new();
        h.write_str(&s.rel);
        h.write(&s.bytes);
        if let Some((text, _)) = &s.settings {
            h.write_str(text);
        }
        if let Some(g) = &s.gltf {
            let mut ext: Vec<_> = g.external.iter().collect();
            ext.sort_by(|a, b| a.0.cmp(&b.0));
            for (p, b) in ext {
                h.write_str(p);
                h.write(b);
            }
        }
        s.hash = h.finish();
    }
    // A terrain's heightmap and control map are part of the terrain, an
    // environment's image part of the environment.
    for s in &sources {
        if s.kind == Kind::Terrain {
            consumed.extend(terrain::terrain_files(
                &s.rel,
                &assets_dir,
                &String::from_utf8_lossy(&s.bytes),
            ));
        }
        if s.kind == Kind::Lightmap
            && let Some(p) = lightmap::pages_file(&s.rel, &String::from_utf8_lossy(&s.bytes))
        {
            consumed.insert(p);
        }
        if s.kind == Kind::Timeline {
            consumed.extend(timeline::caption_files(
                &s.rel,
                &assets_dir,
                &String::from_utf8_lossy(&s.bytes),
            ));
        }
        if s.kind == Kind::Environment {
            consumed.extend(environment::environment_files(
                &s.rel,
                &assets_dir,
                &String::from_utf8_lossy(&s.bytes),
            ));
        }
    }
    // Files a .gltf references are part of that model, not assets of their own.
    sources.retain(|s| !consumed.contains(&s.rel));
    ignored.retain(|r| !consumed.contains(r));
    let known: BTreeSet<String> = sources.iter().map(|s| s.rel.clone()).collect();
    // The project's look (project.ogt's `look "looks/<id>.ogt"`): its art
    // contract (docs/art.md §2.1), read before anything cooks. A line that
    // names no look of the bundle is that asset's error.
    let mut contract: Option<(String, crate::look::LookFile)> = None;
    // Its `Stylise` paint fields, which the 3D pass paints a model's colour
    // textures with in texel space (painterly 3D).
    let mut mesh_paint = None;
    let project_look = std::fs::read_to_string(root.join("project.ogt"))
        .ok()
        .and_then(|t| Document::parse(&t).ok())
        .map(|d| crate::look::project_look(&d.body))
        .transpose()
        .map_err(|e| CookError(format!("project.ogt:{e}")))?
        .flatten();
    if let Some(id) = project_look {
        match sources.iter().find(|s| s.rel == id) {
            Some(s) if s.kind == Kind::Look => {
                if let Ok(c) = look::contract(s, &assets_dir, &|id| known.contains(id)) {
                    if let Ok(full) = look::flattened(s, &assets_dir, &|id| known.contains(id)) {
                        match conform::texture2d::paint_of(&full, &assets_dir) {
                            Ok(p) => mesh_paint = p.active().then_some(p),
                            Err(e) => early_errors.push((id.clone(), "look", e)),
                        }
                    }
                    contract = Some((id, c));
                }
            }
            Some(_) => early_errors.push((
                id,
                "look",
                "project.ogt's `look` names it, and it is not an .ogt of `type look`".into(),
            )),
            None => early_errors.push((
                id.clone(),
                "look",
                format!("project.ogt's `look` names assets/{id}, which does not exist"),
            )),
        }
    }
    // What the 3D pass keeps as made (docs/art.md §2.3): read before the
    // 2D pass, which takes a sheet's own `conform` line out of its text.
    let project_conform = std::fs::read_to_string(root.join("project.ogt"))
        .ok()
        .and_then(|t| Document::parse(&t).ok())
        .map(|d| conform::ProjectConform::parse(&d.body))
        .transpose()
        .map_err(|e| CookError(format!("project.ogt:{e}")))?
        .unwrap_or_default();
    // The screens `og ui check` visits (project.ogt's `ui { screen ... }`):
    // a malformed block fails the cook with its line and fix.
    if let Some(d) = std::fs::read_to_string(root.join("project.ogt"))
        .ok()
        .and_then(|t| Document::parse(&t).ok())
    {
        ui_screens::screens(&d.body).map_err(|e| CookError(format!("project.ogt:{e}")))?;
    }
    let mut mesh_off: BTreeMap<String, String> = BTreeMap::new();
    for s in sources
        .iter()
        .filter(|s| matches!(s.kind, Kind::Model | Kind::Flipbook))
    {
        let why = own_conform_off(s).or_else(|| {
            project_conform
                .off_by(&s.rel)
                .map(|p| format!("project.ogt's conform block: off \"{p}\""))
        });
        if let Some(why) = why {
            mesh_off.insert(s.rel.clone(), why);
        }
    }
    // Conformance (docs/art.md §2.3): the 2D textures, to the contract.
    let (mut conformed, mesh_textures) =
        conform::texture2d::conform(root, &assets_dir, &mut sources, &known, contract.as_ref())
            .map_err(CookError)?;
    // In a 3D project a texture nothing else claims is a mesh's
    // (`conform::TextureRole::Mesh`): the mesh pass holds it, unless it or
    // project.ogt opts it out.
    for s in sources.iter().filter(|s| mesh_textures.contains(&s.rel)) {
        let why = own_conform_off(s).or_else(|| {
            project_conform
                .off_by(&s.rel)
                .map(|p| format!("project.ogt's conform block: off \"{p}\""))
        });
        if let Some(why) = why {
            mesh_off.insert(s.rel.clone(), why);
        }
    }
    // The 3D conformance reads the contract (docs/art.md §2.3): every
    // model, the textures it uses, clips and flipbooks are held to it.
    let mut with_contract = options.clone();
    if let Some((id, c)) = &contract {
        let palette = |p: &str| -> Result<Vec<[u8; 3]>, String> {
            let bytes = match look::builtin(p) {
                Some((_, b)) => b.to_vec(),
                None => std::fs::read(assets_dir.join(crate::runtime::asset_id(p)))
                    .map_err(|e| e.to_string())?,
            };
            crate::look::parse_palette(p, &String::from_utf8_lossy(&bytes))
        };
        match conform::mesh::Conform::of(c, &palette) {
            Ok(m) => {
                with_contract.mesh.conform = m.map(|mut m| {
                    m.paint = mesh_paint.clone();
                    m.off = mesh_off.clone();
                    m.textures = mesh_textures.clone();
                    m
                })
            }
            Err(e) => early_errors.push((id.clone(), "look", e)),
        }
    }
    // A `type model` past its face budget is built again at the style's
    // segment counts (`model_text::to_glb_within`) before the 3D pass
    // decimates what is left. Its stamp already carries the budget
    // (`Conform::model_desc`).
    if let Some(c) = &with_contract.mesh.conform {
        for s in sources
            .iter_mut()
            .filter(|s| s.error.is_none() && !c.off.contains_key(&s.rel))
        {
            let (Some(text), Some(g)) = (model_texts.get(&s.rel), &s.gltf) else {
                continue;
            };
            if !c.applies(g) {
                continue;
            }
            let (budget, _) = c.budget(g);
            let character = g.character;
            let written = conform::mesh::drawn_triangles(g);
            let rebuilt = model_text::to_glb_within(text, budget).and_then(|(glb, detail)| {
                if detail >= 1.0 {
                    return Ok(None);
                }
                let dir = s.path.parent().unwrap_or(Path::new("."));
                let mut g = model::load(&glb, dir)?;
                g.character = character;
                g.segments = Some((detail, written));
                Ok(Some((glb, g)))
            });
            match rebuilt {
                Ok(Some((glb, g))) => {
                    s.bytes = glb;
                    s.gltf = Some(g);
                }
                Ok(None) => {}
                Err(e) => s.error = Some(e),
            }
        }
    }
    let options = &with_contract;
    // Streaming groups (phase 3): every scene's, read before anything cooks.
    let (groups, group_of) = plan_groups(&sources, &known, &mut early_errors);
    let schemas: BTreeMap<String, Schema> = options
        .schemas
        .iter()
        .map(|s| (s.name.clone(), s.clone()))
        .collect();
    let mut schema_hash = StableHasher::new();
    for s in schemas.values() {
        schema_hash.write_str(&s.to_json());
    }
    let schema_hash = schema_hash.finish().to_le_bytes();
    let old = std::fs::read_to_string(cooked_dir.join(MANIFEST))
        .ok()
        .and_then(|t| Manifest::parse(&t).ok())
        .unwrap_or_default();
    let old_outputs: BTreeMap<&str, &ManifestOutput> =
        old.outputs().map(|o| (o.id.as_str(), o)).collect();
    let mesh_desc = options.mesh.describe();
    let font_desc = options.font.describe();
    let tex_desc = format!(
        "{} encoder {}",
        options.texture.describe(),
        texture_encoder.as_deref().unwrap_or("none")
    );

    // Plan every output with its stamp.
    let mut plan: Vec<Planned> = Vec::new();
    for (si, s) in sources.iter().enumerate() {
        if s.error.is_some() {
            continue;
        }
        let hash = s.hash.to_le_bytes();
        let mut outs: Vec<(String, &'static str, u64, Work)> = Vec::new();
        match s.kind {
            Kind::Model => {
                let g = s.gltf.as_ref().expect("loaded");
                // The art contract's part of each output's stamp: empty
                // (and left out) when it changes nothing here.
                let contract = options
                    .mesh
                    .conform
                    .as_ref()
                    .filter(|_| !conform_off(s, options));
                let model_desc = contract
                    .map(|c| {
                        let mut d = if c.applies(g) {
                            c.model_desc()
                        } else {
                            String::new()
                        };
                        // Its colour textures' kind is in the blob.
                        if c.filter == conform::mesh::Filter::Nearest {
                            d.push_str(" textures nearest");
                        }
                        d
                    })
                    .unwrap_or_default();
                let texture_desc = contract.map(|c| c.texture_desc()).unwrap_or_default();
                let with = |parts: &[&[u8]], desc: &str, extra: &[u8]| {
                    let mut v: Vec<&[u8]> = parts.to_vec();
                    if !desc.is_empty() {
                        v.push(desc.as_bytes());
                        v.push(extra);
                    }
                    stamp(&v)
                };
                let mut ids = Vec::new();
                for (i, u) in g.textures.iter().enumerate() {
                    let id = format!("{}#tex{i}", s.rel);
                    let image_hash = hash_bytes(&g.images[u.image]).to_le_bytes();
                    // A density is per metre of the model: its hash too.
                    let st = with(
                        &[
                            b"texture",
                            tex_desc.as_bytes(),
                            &image_hash,
                            &u.kind.to_le_bytes(),
                        ],
                        &texture_desc,
                        &hash,
                    );
                    outs.push((
                        id.clone(),
                        format::Texture::NAME,
                        st,
                        Work::GltfTexture(s, u.image, u.kind),
                    ));
                    ids.push(id);
                }
                let joined = ids.join("\n");
                // A model written as text is refused past the budgets, so
                // they are among its inputs.
                let st = if ext(&s.rel) == "ogt" {
                    with(
                        &[
                            b"model",
                            NODE_ROWS,
                            mesh_desc.as_bytes(),
                            &hash,
                            joined.as_bytes(),
                            budget_desc.as_bytes(),
                        ],
                        &model_desc,
                        &[],
                    )
                } else {
                    with(
                        &[
                            b"model",
                            NODE_ROWS,
                            mesh_desc.as_bytes(),
                            &hash,
                            joined.as_bytes(),
                        ],
                        &model_desc,
                        &[],
                    )
                };
                outs.insert(
                    0,
                    (s.rel.clone(), format::Model::NAME, st, Work::Model(s, ids)),
                );
                if group_of.contains_key(&s.rel) {
                    let st = with(
                        &[b"proxy", NODE_ROWS, mesh_desc.as_bytes(), &hash],
                        &model_desc,
                        &[],
                    );
                    outs.push((
                        format!("{}#proxy", s.rel),
                        format::Model::NAME,
                        st,
                        Work::Proxy(s),
                    ));
                }
                if anim::wanted(g) {
                    match anim_settings(s) {
                        Ok(a) => {
                            let deps = anim::dependencies(&assets_root(s), &a);
                            let motion = contract
                                .and_then(|c| c.motion.as_ref())
                                .map(|m| m.describe())
                                .unwrap_or_default();
                            let st = with(
                                &[
                                    b"animation",
                                    NODE_ROWS,
                                    mesh_desc.as_bytes(),
                                    a.describe().as_bytes(),
                                    &hash,
                                    deps.as_bytes(),
                                ],
                                &format!("{model_desc}{motion}"),
                                &[],
                            );
                            outs.insert(
                                1,
                                (
                                    format!("{}#anim", s.rel),
                                    format::Animation::NAME,
                                    st,
                                    Work::Animation(s, a),
                                ),
                            );
                        }
                        Err(e) => early_errors.push((s.rel.clone(), "model", e)),
                    }
                }
            }
            Kind::Texture => match texture_kind(s) {
                Ok(k) => {
                    // A mesh's texture under the art contract: its desc too.
                    let mesh = mesh_texture(s, options)
                        .map(|c| c.texture_desc())
                        .unwrap_or_default();
                    let st = if mesh.is_empty() {
                        stamp(&[b"texture", tex_desc.as_bytes(), &hash, &k.to_le_bytes()])
                    } else {
                        stamp(&[
                            b"texture",
                            tex_desc.as_bytes(),
                            &hash,
                            &k.to_le_bytes(),
                            mesh.as_bytes(),
                        ])
                    };
                    outs.push((
                        s.rel.clone(),
                        format::Texture::NAME,
                        st,
                        Work::Texture(s, k),
                    ));
                }
                Err(e) => early_errors.push((s.rel.clone(), "texture", e)),
            },
            Kind::Sound if s.rel.ends_with(".ogt") => {
                // A preset's stamp moves with the synthesiser.
                let st = stamp(&[b"sfx", sfx::SYNTH_VERSION.as_bytes(), &hash]);
                outs.push((s.rel.clone(), format::Sound::NAME, st, Work::Sound(s, None)));
            }
            Kind::Sound => {
                let settings = match audio::SoundSettings::parse(settings_block(s, "sound")) {
                    Ok(x) => x,
                    Err(e) => {
                        early_errors.push((s.rel.clone(), "sound", e));
                        continue;
                    }
                };
                // An unreadable file fails in the cook, with its reason.
                let chunking = audio::shape(&s.rel, &s.bytes)
                    .ok()
                    .and_then(|(rate, ch, frames)| settings.chunking(rate, ch, frames));
                let shape =
                    chunking.map_or("whole".to_string(), |(p, n)| format!("{n} chunks of {p}"));
                let st = stamp(&[
                    b"sound",
                    &hash,
                    settings.describe().as_bytes(),
                    shape.as_bytes(),
                ]);
                outs.push((
                    s.rel.clone(),
                    format::Sound::NAME,
                    st,
                    Work::Sound(s, chunking),
                ));
                if let Some((per, count)) = chunking {
                    for n in 1..count {
                        let st =
                            stamp(&[b"sound chunk", &hash, shape.as_bytes(), &n.to_le_bytes()]);
                        outs.push((
                            format!("{}#chunk{n}", s.rel),
                            format::SoundChunk::NAME,
                            st,
                            Work::SoundChunk(s, per, n),
                        ));
                    }
                }
            }
            Kind::Lut => {
                let st = stamp(&[b"lut", &hash]);
                outs.push((s.rel.clone(), format::Lut::NAME, st, Work::Lut(s)));
            }
            Kind::Emitter => {
                let text = String::from_utf8_lossy(&s.bytes);
                let refs: Vec<String> = crate::fx::emitter_references(&text)
                    .into_iter()
                    .map(|r| format!("{r}={}", known.contains(crate::runtime::asset_id(&r))))
                    .collect();
                let st = stamp(&[b"emitter", &hash, refs.join("\n").as_bytes()]);
                outs.push((s.rel.clone(), format::Emitter::NAME, st, Work::Emitter(s)));
            }
            Kind::Shader => {
                // The stamp covers the WGSL file the shader names.
                let text = String::from_utf8_lossy(&s.bytes);
                let wgsl = crate::shader::shader_source(&text)
                    .map(|src| read_sibling(s, &src).unwrap_or_default())
                    .unwrap_or_default();
                let st = stamp(&[b"shader", &hash, wgsl.as_bytes()]);
                outs.push((s.rel.clone(), format::Shader::NAME, st, Work::Shader(s)));
            }
            Kind::Post => {
                let text = String::from_utf8_lossy(&s.bytes);
                let wgsl = crate::shader::shader_source(&text)
                    .map(|src| read_sibling(s, &src).unwrap_or_default())
                    .unwrap_or_default();
                let st = stamp(&[b"post", &hash, wgsl.as_bytes()]);
                outs.push((s.rel.clone(), format::Post::NAME, st, Work::Post(s)));
            }
            Kind::UiMaterial => {
                let text = String::from_utf8_lossy(&s.bytes);
                let wgsl = crate::shader::shader_source(&text)
                    .map(|src| read_sibling(s, &src).unwrap_or_default())
                    .unwrap_or_default();
                let st = stamp(&[b"ui-material", &hash, wgsl.as_bytes()]);
                outs.push((
                    s.rel.clone(),
                    format::UiMaterial::NAME,
                    st,
                    Work::UiMaterial(s),
                ));
            }
            Kind::Vector => {
                // The stamp covers the SVG the vector names.
                let svg = vector::dependencies(s);
                let st = stamp(&[b"vector", &hash, svg.as_bytes()]);
                outs.push((s.rel.clone(), format::Vector::NAME, st, Work::Vector(s)));
            }
            Kind::Style => {
                let text = String::from_utf8_lossy(&s.bytes);
                let refs: Vec<String> = og_ui::parse_style(&text)
                    .map(|mut f| {
                        f.resolve(&s.rel, |id| known.contains(crate::runtime::asset_id(id)));
                        f.references()
                    })
                    .unwrap_or_default()
                    .into_iter()
                    .map(|r| format!("{r}={}", known.contains(crate::runtime::asset_id(&r))))
                    .collect();
                let atlas = og_ui::parse_style(&text)
                    .ok()
                    .and_then(|mut f| {
                        f.resolve(&s.rel, |id| known.contains(crate::runtime::asset_id(id)));
                        f.atlas
                    })
                    .and_then(|id| style_atlas(crate::runtime::asset_id(&id), &assets_root(s)))
                    .map(|b| hash_bytes(&b).to_le_bytes())
                    .unwrap_or_default();
                let st = stamp(&[b"style", &hash, refs.join("\n").as_bytes(), &atlas]);
                outs.push((s.rel.clone(), format::Style::NAME, st, Work::Style(s)));
            }
            Kind::Document => {
                let st = stamp(&[b"document", &hash]);
                outs.push((s.rel.clone(), format::Document::NAME, st, Work::Document(s)));
            }
            Kind::Tileset => {
                let st = stamp(&[
                    b"tileset",
                    &hash,
                    tilemap::references_of_tileset(&s.bytes, &|id| known.contains(id)).as_bytes(),
                ]);
                outs.push((s.rel.clone(), format::Tileset::NAME, st, Work::Tileset(s)));
            }
            Kind::Tilemap => match tilemap::MapSettings::parse(settings_block(s, "tilemap")) {
                Ok(m) => {
                    let refs = tilemap::references(&s.rel, &s.bytes, &m, &|id| known.contains(id));
                    let st = stamp(&[
                        b"tilemap",
                        &hash,
                        m.describe().as_bytes(),
                        refs.join("\n").as_bytes(),
                    ]);
                    outs.push((
                        s.rel.clone(),
                        format::Tilemap::NAME,
                        st,
                        Work::Tilemap(s, m),
                    ));
                }
                Err(e) => early_errors.push((s.rel.clone(), "tilemap", e)),
            },
            Kind::Rig2d => {
                let deps = rig2d::dependency_hashes(s);
                let st = stamp(&[b"rig2d", &hash, deps.as_bytes()]);
                outs.push((s.rel.clone(), format::Rig2D::NAME, st, Work::Rig2d(s)));
            }
            Kind::Flipbook => {
                let deps = flipbook::dependency_hashes(s);
                let motion = options
                    .mesh
                    .conform
                    .as_ref()
                    .and_then(|c| c.motion.as_ref())
                    .filter(|_| !conform_off(s, options))
                    .map(|m| m.describe());
                let st = match &motion {
                    Some(m) => stamp(&[b"flipbook", &hash, deps.as_bytes(), m.as_bytes()]),
                    None => stamp(&[b"flipbook", &hash, deps.as_bytes()]),
                };
                outs.push((s.rel.clone(), format::Flipbook::NAME, st, Work::Flipbook(s)));
                // `array true`: the frames as the layers of a texture array.
                if let Ok(sheet) = flipbook::sheet(s)
                    && sheet.array
                {
                    let base = [
                        b"flipbook array 1".as_slice(),
                        tex_desc.as_bytes(),
                        &hash,
                        deps.as_bytes(),
                    ];
                    let sheet = Box::new(sheet);
                    for n in 0..sheet.frames.len() {
                        let st = stamp(&[&base[..], &[&(n as u32).to_le_bytes()[..]]].concat());
                        outs.push((
                            texture_array::layer_id(&flipbook::array_id(&s.rel), n),
                            format::Texture::NAME,
                            st,
                            Work::FlipbookLayer(s, sheet.clone(), n),
                        ));
                    }
                    outs.push((
                        flipbook::array_id(&s.rel),
                        format::TextureArray::NAME,
                        stamp(&base),
                        Work::FlipbookArray(s, sheet),
                    ));
                }
            }
            Kind::Behaviour => {
                let st = stamp(&[b"behaviour", &hash]);
                outs.push((
                    s.rel.clone(),
                    format::Behaviour::NAME,
                    st,
                    Work::Behaviour(s),
                ));
            }
            Kind::Strings => {
                let st = stamp(&[b"strings", &hash]);
                outs.push((s.rel.clone(), format::Strings::NAME, st, Work::Strings(s)));
            }
            Kind::Rig => {
                let st = stamp(&[b"rig", &hash]);
                outs.push((s.rel.clone(), format::Rig::NAME, st, Work::Rig(s)));
            }
            Kind::Icons => {
                // The stamp covers every SVG the set names.
                let deps = icons::dependencies(s);
                let st = stamp(&[b"icons", &hash, deps.as_bytes()]);
                outs.push((s.rel.clone(), format::Font::NAME, st, Work::Icons(s)));
            }
            Kind::Terrain => {
                let deps = terrain::terrain_dependencies(s, &|id| known.contains(id));
                let st = stamp(&[b"terrain 2", &hash, deps.as_bytes()]);
                outs.push((s.rel.clone(), format::Terrain::NAME, st, Work::Terrain(s)));
            }
            Kind::Lightmap => {
                // The stamp covers its pages file.
                let deps = lightmap::dependencies(s);
                let st = stamp(&[b"lightmap 1", &hash, deps.as_bytes()]);
                outs.push((s.rel.clone(), format::Lightmap::NAME, st, Work::Lightmap(s)));
            }
            Kind::Timeline => {
                // The stamp covers every caption file it imports.
                let deps = timeline::dependencies(s);
                let st = stamp(&[b"timeline 1", &hash, deps.as_bytes()]);
                outs.push((s.rel.clone(), format::Timeline::NAME, st, Work::Timeline(s)));
            }
            Kind::TextureArray => {
                let text = String::from_utf8_lossy(&s.bytes);
                let root = texture_array::assets_dir(&s.path, &s.rel);
                match texture_array::parse(&s.rel, &text, &|p| root.join(p).is_file()) {
                    Ok(decl) => {
                        let deps = texture_array::dependencies(&root, &decl);
                        let base = [
                            b"texture-array 1".as_slice(),
                            tex_desc.as_bytes(),
                            &hash,
                            deps.as_bytes(),
                        ];
                        outs.push((
                            s.rel.clone(),
                            format::TextureArray::NAME,
                            stamp(&base),
                            Work::TextureArray(s, decl.clone()),
                        ));
                        for n in 0..decl.layers.len() {
                            let st = stamp(&[&base[..], &[&(n as u32).to_le_bytes()[..]]].concat());
                            outs.push((
                                texture_array::layer_id(&s.rel, n),
                                format::Texture::NAME,
                                st,
                                Work::ArrayLayer(s, decl.clone(), n),
                            ));
                        }
                    }
                    Err(e) => early_errors.push((s.rel.clone(), "texture-array", e)),
                }
            }
            Kind::Video => {
                let text = String::from_utf8_lossy(&s.bytes);
                let root = texture_array::assets_dir(&s.path, &s.rel);
                let planned =
                    video::parse(&s.rel, &text, &|p| root.join(p).is_file()).and_then(|d| {
                        let src = root.join(&d.source);
                        let bytes =
                            std::fs::read(&src).map_err(|e| format!("{}: {e}", src.display()))?;
                        let plan = video::plan(&d, video::probe(&src)?)?;
                        Ok((src, hash_bytes(&bytes), plan))
                    });
                match planned {
                    Ok((src, src_hash, plan)) => {
                        let described = plan.describe();
                        let base = [
                            b"video 1".as_slice(),
                            tex_desc.as_bytes(),
                            &hash,
                            &src_hash.to_le_bytes(),
                            described.as_bytes(),
                        ];
                        let plan = Box::new(plan);
                        outs.push((
                            s.rel.clone(),
                            format::Video::NAME,
                            stamp(&base),
                            Work::VideoMeta(plan.clone()),
                        ));
                        let frames = video::frames_id(&s.rel);
                        outs.push((
                            frames.clone(),
                            format::TextureArray::NAME,
                            stamp(&[&base[..], &[b"array".as_slice()]].concat()),
                            Work::VideoArray(s, plan.clone()),
                        ));
                        for n in 0..plan.frames as usize {
                            let st = stamp(&[&base[..], &[&(n as u32).to_le_bytes()[..]]].concat());
                            outs.push((
                                texture_array::layer_id(&frames, n),
                                format::Texture::NAME,
                                st,
                                Work::VideoFrame(src.clone(), src_hash, plan.clone(), n),
                            ));
                        }
                        if plan.audio {
                            outs.push((
                                video::audio_id(&s.rel),
                                format::Sound::NAME,
                                stamp(&[&base[..], &[b"audio".as_slice()]].concat()),
                                Work::VideoAudio(src.clone(), plan.clone()),
                            ));
                        }
                    }
                    Err(e) => early_errors.push((s.rel.clone(), "video", e)),
                }
            }
            Kind::Environment => {
                let deps = environment::environment_dependencies(s);
                let st = stamp(&[b"environment 1", &hash, deps.as_bytes()]);
                outs.push((
                    s.rel.clone(),
                    format::EnvironmentMap::NAME,
                    st,
                    Work::Environment(s),
                ));
            }
            Kind::Look => {
                let deps = look::look_dependencies(s, &assets_dir, &|id| known.contains(id));
                let st = stamp(&[b"look 1", &hash, deps.as_bytes()]);
                outs.push((s.rel.clone(), format::Look::NAME, st, Work::Look(s)));
            }
            Kind::Palette => {
                let st = stamp(&[b"palette 1", &hash]);
                outs.push((s.rel.clone(), format::Palette::NAME, st, Work::Palette(s)));
            }
            Kind::NavMesh => {
                let deps = e6::navmesh_dependencies(s);
                let st = stamp(&[b"navmesh", &hash, deps.as_bytes()]);
                outs.push((s.rel.clone(), format::NavMesh::NAME, st, Work::NavMesh(s)));
            }
            Kind::Font => match font_settings(s, &options.font) {
                Ok(f) => {
                    let st = stamp(&[b"font", f.describe().as_bytes(), &hash]);
                    outs.push((s.rel.clone(), format::Font::NAME, st, Work::Font(s, f)));
                }
                Err(e) => early_errors.push((s.rel.clone(), "font", e)),
            },
            Kind::Scene => {
                let text = String::from_utf8_lossy(&s.bytes);
                let refs: Vec<String> = scene::references(&text)
                    .into_iter()
                    .map(|r| format!("{r}={}", known.contains(&r)))
                    .collect();
                let st = stamp(&[b"scene", &hash, &schema_hash, refs.join("\n").as_bytes()]);
                outs.push((s.rel.clone(), format::Scene::NAME, st, Work::Scene(s)));
            }
        }
        for (id, ty, st, work) in outs {
            let file = output_file(&id);
            // Up to date: the manifest's stamp and file agree, and the file
            // still holds the type and schema version the loader expects
            // today (a schema change without a cooker bump would otherwise
            // keep a blob the loader refuses).
            let want = format::expected_header(ty);
            let fresh = old_outputs.get(id.as_str()).is_some_and(|o| {
                o.stamp == st
                    && o.ty == ty
                    && o.file == file
                    && std::fs::read(cooked_dir.join(&file)).is_ok_and(|b| {
                        b.len() as u64 == o.bytes
                            && hash_bytes(&b) == o.hash
                            && og_blob::Blob::parse(&b, &file).is_ok_and(|blob| {
                                let h = blob.header();
                                want == Some((h.type_hash, h.schema_version))
                            })
                    })
            });
            plan.push(Planned {
                source: si,
                id,
                file,
                ty,
                stamp: st,
                work: (!fresh).then_some(work),
            });
        }
    }

    // Cook what is stale, in parallel; results land by plan index.
    type Done = (Result<(Vec<u8>, Vec<String>), CookError>, f64, f64, u32);
    let results: Vec<Mutex<Option<Done>>> = plan.iter().map(|_| Mutex::new(None)).collect();
    let todo: Vec<usize> = (0..plan.len())
        .filter(|&i| plan[i].work.is_some())
        .collect();
    let next = std::sync::atomic::AtomicUsize::new(0);
    std::thread::scope(|scope| {
        for worker in 0..threads.min(todo.len()) {
            let (next, results, plan, schemas, known, todo) =
                (&next, &results, &plan, &schemas, &known, &todo);
            let budgets = &budgets;
            scope.spawn(move || {
                loop {
                    let k = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some(&i) = todo.get(k) else { break };
                    let p = &plan[i];
                    let t = Instant::now();
                    let start = t.duration_since(started).as_secs_f64() * 1000.0;
                    let r = texture_cache::run(texture_cache, p, || {
                        run(p, options, threads, schemas, known, budgets)
                    });
                    *results[i].lock().expect("no poisoning") =
                        Some((r, t.elapsed().as_secs_f64() * 1000.0, start, worker as u32));
                }
            });
        }
    });

    // Assemble the report and the manifest; write what changed.
    let mut reports: Vec<AssetReport> = sources
        .iter()
        .map(|s| AssetReport {
            source: s.rel.clone(),
            kind: s.kind.name().to_string(),
            source_bytes: s.bytes.len() as u64
                + s.settings.as_ref().map_or(0, |(t, _)| t.len() as u64)
                + s.gltf
                    .as_ref()
                    .map_or(0, |g| g.external.iter().map(|e| e.1.len() as u64).sum()),
            outputs: Vec::new(),
            notes: s.gltf.as_ref().map(|_| Vec::new()).unwrap_or_default(),
            error: s.error.clone(),
            cook_ms: 0.0,
        })
        .collect();
    for (rel, kind, e) in early_errors {
        match reports.iter_mut().find(|r| r.source == rel) {
            Some(r) => r.error = Some(e),
            None => reports.push(AssetReport {
                source: rel,
                kind: kind.to_string(),
                source_bytes: 0,
                outputs: Vec::new(),
                notes: Vec::new(),
                error: Some(e),
                cook_ms: 0.0,
            }),
        }
    }
    let mut new_outputs: BTreeMap<usize, Vec<NewOutput>> = BTreeMap::new();
    // The 3D pass's lines per output: this cook's, or for an output that
    // was up to date the ones its last cook recorded.
    let old_log = read_conform_log(&cooked_dir);
    let mut log = ConformLog::new();
    for (i, p) in plan.iter().enumerate() {
        let r = &mut reports[p.source];
        let key = (p.id.clone(), p.stamp);
        let (bytes, cooked, ms, start_ms, worker) =
            match results[i].lock().expect("no poisoning").take() {
                Some((Ok((bytes, notes)), ms, start, worker)) => {
                    let mut mine = Vec::new();
                    for n in notes {
                        match conform_note(&p.id, &n) {
                            Some(what) => mine.push(what.to_string()),
                            None => r.notes.push(n),
                        }
                    }
                    if !mine.is_empty() {
                        log.insert(key, mine);
                    }
                    (Some(bytes), true, ms, start, worker)
                }
                Some((Err(e), ..)) => {
                    r.error = Some(e.0);
                    continue;
                }
                None => {
                    if let Some(w) = old_log.get(&key) {
                        log.insert(key, w.clone());
                    }
                    (None, false, 0.0, 0.0, 0)
                }
            };
        let (size, hash) = match &bytes {
            Some(b) => (b.len() as u64, hash_bytes(b)),
            None => {
                let o = old_outputs[p.id.as_str()];
                (o.bytes, o.hash)
            }
        };
        new_outputs.entry(p.source).or_default().push((
            ManifestOutput {
                id: p.id.clone(),
                ty: p.ty.to_string(),
                file: p.file.clone(),
                stamp: p.stamp,
                hash,
                bytes: size,
                group: output_group(&p.id, &sources[p.source].rel, &group_of),
            },
            bytes,
            OutputReport {
                id: p.id.clone(),
                file: p.file.clone(),
                ty: p.ty.to_string(),
                bytes: size,
                cooked,
                written: false,
                ms,
                start_ms,
                worker,
            },
        ));
    }
    let mut manifest = Manifest {
        cooker: COOKER_VERSION.to_string(),
        options: BTreeMap::from([
            ("font".to_string(), font_desc),
            ("mesh".to_string(), mesh_desc),
            ("texture".to_string(), tex_desc),
        ]),
        assets: Vec::new(),
        streams: Vec::new(),
    };
    let mut keep: BTreeSet<String> = BTreeSet::new();
    for (si, outs) in new_outputs {
        let s = &sources[si];
        if reports[si].error.is_some() {
            continue;
        }
        let mut asset = ManifestAsset {
            source: s.rel.clone(),
            kind: s.kind.name().to_string(),
            source_hash: s.hash,
            source_bytes: reports[si].source_bytes,
            outputs: Vec::new(),
        };
        for (m, bytes, mut rep) in outs {
            if let Some(b) = bytes {
                let path = cooked_dir.join(&m.file);
                if std::fs::read(&path).ok().as_deref() != Some(&b[..]) {
                    if let Some(dir) = path.parent() {
                        std::fs::create_dir_all(dir)
                            .map_err(|e| CookError(format!("{}: {e}", dir.display())))?;
                    }
                    std::fs::write(&path, &b)
                        .map_err(|e| CookError(format!("{}: {e}", path.display())))?;
                    rep.written = true;
                }
            }
            keep.insert(m.file.clone());
            asset.outputs.push(m);
            reports[si].outputs.push(rep);
        }
        manifest.assets.push(asset);
    }
    // Baked reflection probes (`og bake probes`, probes.rs) have no source:
    // kept while their files are.
    for a in old.assets.iter().filter(|a| a.kind == probes::KIND) {
        if a.outputs.iter().all(|o| cooked_dir.join(&o.file).is_file()) {
            keep.extend(a.outputs.iter().map(|o| o.file.clone()));
            manifest.assets.push(a.clone());
        }
    }
    manifest.assets.sort_by(|a, b| a.source.cmp(&b.source));
    for r in reports.iter_mut().filter(|r| r.error.is_some()) {
        r.outputs.clear();
    }
    // Each group's outputs, packed into one file the runtime fetches.
    for (name, g) in &groups {
        let mut files: Vec<(String, Vec<u8>)> = Vec::new();
        let mut memory = 0u64;
        let mut sounds = Vec::new();
        for o in manifest
            .outputs()
            .filter(|o| o.group.as_deref() == Some(name))
        {
            let path = cooked_dir.join(&o.file);
            let bytes =
                std::fs::read(&path).map_err(|e| CookError(format!("{}: {e}", path.display())))?;
            memory += stream::memory_of(&o.ty, &bytes);
            if o.ty == "og.Sound" {
                sounds.extend(stream::sound_of(&o.id, &bytes));
            }
            files.push((o.file.clone(), bytes));
        }
        files.sort_by(|a, b| a.0.cmp(&b.0));
        let mut h = StableHasher::new();
        h.write_str(name);
        for (f, b) in &files {
            h.write_str(f);
            h.write(&hash_bytes(b).to_le_bytes());
        }
        let pack = stream::pack(&files, h.finish());
        let file = format!("stream/{name}.ogb");
        let path = cooked_dir.join(&file);
        if std::fs::read(&path).ok().as_deref() != Some(&pack[..]) {
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir)
                    .map_err(|e| CookError(format!("{}: {e}", dir.display())))?;
            }
            std::fs::write(&path, &pack)
                .map_err(|e| CookError(format!("{}: {e}", path.display())))?;
        }
        keep.insert(file.clone());
        let entry = stream::manifest_entry(
            g,
            file,
            pack.len() as u64,
            hash_bytes(&pack),
            memory,
            sounds,
        );
        if let Some(r) = reports.iter_mut().find(|r| r.source == g.scene) {
            r.notes.push(format!(
                "stream group '{name}': {} asset(s), {} file(s), {} bytes packed, {} bytes once loaded",
                g.assets.len(),
                files.len(),
                entry.bytes,
                memory
            ));
        }
        manifest.streams.push(entry);
    }
    let mut removed = Vec::new();
    let old_files = old
        .outputs()
        .map(|o| &o.file)
        .chain(old.streams.iter().map(|g| &g.file));
    for f in old_files {
        if !keep.contains(f) && cooked_dir.join(f).exists() {
            std::fs::remove_file(cooked_dir.join(f))
                .map_err(|e| CookError(format!("cooked/{f}: {e}")))?;
            removed.push(f.clone());
        }
    }
    std::fs::create_dir_all(&cooked_dir)
        .map_err(|e| CookError(format!("{}: {e}", cooked_dir.display())))?;
    let text = manifest.to_text();
    let manifest_path = cooked_dir.join(MANIFEST);
    if std::fs::read_to_string(&manifest_path).ok().as_deref() != Some(text.as_str()) {
        std::fs::write(&manifest_path, &text)
            .map_err(|e| CookError(format!("{}: {e}", manifest_path.display())))?;
    }
    // Each asset's cook time: this run's (its outputs' summed) when it
    // cooked, else the one recorded; written only when one changed.
    let old_times = CookTimes::read(&cooked_dir);
    let mut times = CookTimes::default();
    for r in reports.iter_mut().filter(|r| r.error.is_none()) {
        let now: Vec<f64> = r
            .outputs
            .iter()
            .filter(|o| o.cooked)
            .map(|o| o.ms)
            .collect();
        r.cook_ms = if now.is_empty() {
            old_times.assets.get(&r.source).copied().unwrap_or(0.0)
        } else {
            now.iter().sum()
        };
        if !r.outputs.is_empty() {
            times.assets.insert(r.source.clone(), r.cook_ms);
        }
    }
    let log_path = cooked_dir.join(CONFORM_LOG);
    if log.is_empty() {
        if log_path.exists() {
            std::fs::remove_file(&log_path)
                .map_err(|e| CookError(format!("{}: {e}", log_path.display())))?;
        }
    } else {
        let text = conform_log_text(&log);
        if std::fs::read_to_string(&log_path).ok().as_deref() != Some(text.as_str()) {
            std::fs::write(&log_path, text)
                .map_err(|e| CookError(format!("{}: {e}", log_path.display())))?;
        }
    }
    if times != old_times {
        let path = cooked_dir.join(COOK_TIMES);
        std::fs::write(&path, times.to_text())
            .map_err(|e| CookError(format!("{}: {e}", path.display())))?;
    }
    // Across the string tables: every locale has every key, with its arguments.
    let tables: Vec<(&str, &[u8])> = sources
        .iter()
        .filter(|s| s.kind == Kind::Strings)
        .map(|s| (s.rel.as_str(), s.bytes.as_slice()))
        .collect();
    let mut warnings = Vec::new();
    for (rel, what) in e6::check_strings(&tables) {
        warnings.push(format!("assets/{rel}: {what}"));
        if let Some(r) = reports.iter_mut().find(|r| r.source == rel) {
            r.notes.push(what);
        }
    }
    // A text model past the project's triangle or vertex budget: cooked, and said.
    for r in reports.iter().filter(|r| r.kind == "model") {
        for n in r
            .notes
            .iter()
            .filter(|n| n.starts_with("warning: over budget: "))
        {
            warnings.push(format!("assets/{}: {n}", r.source));
        }
    }
    // A video's frames past the budget: footage is the heaviest asset a game has.
    for r in reports.iter_mut().filter(|r| r.kind == "video") {
        let frames = format!("{}#frames#layer", r.source);
        let bytes: u64 = r
            .outputs
            .iter()
            .filter(|o| o.id.starts_with(&frames))
            .map(|o| o.bytes)
            .sum();
        if bytes > video::BUDGET_BYTES {
            let what = format!(
                "its {} frames take {:.1} MiB cooked, past the {} MiB budget for a video: lower its size, fps or maxSeconds",
                r.outputs
                    .iter()
                    .filter(|o| o.id.starts_with(&frames))
                    .count(),
                bytes as f64 / f64::from(1u32 << 20),
                video::BUDGET_BYTES >> 20
            );
            warnings.push(format!("assets/{}: {what}", r.source));
            r.notes.push(what);
        }
    }
    // A Tiled map's custom properties the importer does not read.
    for s in sources
        .iter()
        .filter(|s| s.kind == Kind::Tilemap && s.rel.to_ascii_lowercase().ends_with(".tmj"))
    {
        for what in tilemap::tiled_unread(&String::from_utf8_lossy(&s.bytes)) {
            warnings.push(format!("assets/{}: {what}", s.rel));
            if let Some(r) = reports.iter_mut().find(|r| r.source == s.rel) {
                r.notes.push(what);
            }
        }
    }
    // The project's look carries the art contract: say which of its
    // resources it sets (conformance to them is docs/art.md §2.3).
    if let Some((id, c)) = &contract
        && let Some(r) = reports.iter_mut().find(|r| &r.source == id)
    {
        let set = c.resources();
        r.notes.push(if set.is_empty() {
            "the project's look (project.ogt): it sets no art resource, so the art contract constrains nothing".into()
        } else {
            format!("the project's look (project.ogt): its art contract sets {}", set.join(", "))
        });
    }
    // The 3D pass's entries (docs/art.md §2.3): per asset, what its
    // outputs' lines say, and what it keeps as made; its warnings (frames
    // dropped past ArtMotion.maxFrames) are the cook's too.
    let failed = |rel: &str| reports.iter().any(|r| r.source == rel && r.error.is_some());
    let mut mesh: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for p in &plan {
        if let Some(w) = log.get(&(p.id.clone(), p.stamp)) {
            mesh.entry(sources[p.source].rel.as_str())
                .or_default()
                .extend(w.iter().map(String::as_str));
        }
    }
    let mut mine: Vec<ConformEntry> = Vec::new();
    for (rel, lines) in mesh {
        if failed(rel) {
            continue;
        }
        let mut changed: Vec<&str> = Vec::new();
        for l in lines {
            if l != conform::mesh::UNCHANGED && !changed.contains(&l) {
                changed.push(l);
            }
            if l.starts_with("warning: ") {
                warnings.push(format!("assets/{rel}: {l}"));
            }
        }
        mine.push(ConformEntry {
            source: rel.to_string(),
            action: if changed.is_empty() {
                ConformAction::Unchanged
            } else {
                ConformAction::Changed
            },
            what: if changed.is_empty() {
                conform::mesh::UNCHANGED.to_string()
            } else {
                changed.join("; ")
            },
        });
    }
    if let Some(c) = &options.mesh.conform {
        for (rel, why) in &c.off {
            let applies = sources.iter().any(|s| {
                &s.rel == rel
                    && match s.kind {
                        Kind::Model => true,
                        Kind::Flipbook => c.motion.is_some(),
                        _ => !c.texture_desc().is_empty(),
                    }
            });
            let listed = conformed
                .iter()
                .any(|e| &e.source == rel && e.action == ConformAction::Off);
            if applies && !listed && !failed(rel) {
                mine.push(ConformEntry {
                    source: rel.clone(),
                    action: ConformAction::Off,
                    what: why.clone(),
                });
            }
        }
    }
    mine.sort_by(|a, b| a.source.cmp(&b.source));
    conformed.extend(mine);
    // What conformance changed and what opted out, on each asset too.
    for e in &conformed {
        if let Some(r) = reports.iter_mut().find(|r| r.source == e.source) {
            r.notes
                .push(format!("art contract: {}: {}", e.action.name(), e.what));
        }
    }
    reports.sort_by(|a, b| a.source.cmp(&b.source));
    Ok(Report {
        conform: conformed,
        warnings,
        root: root.display().to_string(),
        cooker: COOKER_VERSION.to_string(),
        texture_encoder: texture_encoder.unwrap_or_else(|e| format!("none ({e})")),
        assets: reports,
        ignored,
        removed,
        elapsed_ms: started.elapsed().as_secs_f64() * 1000.0,
    })
}

/// A planned output's manifest entry, its new bytes (None: up to date) and report line.
type NewOutput = (ManifestOutput, Option<Vec<u8>>, OutputReport);
