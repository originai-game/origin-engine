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

mod action_cache;
pub mod anim;
pub mod animated;
pub mod asset_move;
pub mod assets;
pub mod audio;
mod autorig;
pub mod bc6h;
pub mod cache;
pub mod conform;
pub use conform::{ConformAction, ConformEntry};
pub mod cube_source;
mod e6;
mod effect;
pub mod efk;
mod environment;
mod fbx;
mod flipbook;
mod font;
mod fracture;
mod fracture_cache;
mod fx_field;
pub mod icons;
mod index;
pub mod ktx2_source;
mod lightmap;
pub mod look;
mod look_assets;
mod look_styles;
mod model;
pub mod model_text;
mod motion;
mod normal2d;
pub mod prefab;
pub mod probes;
mod psd;
mod rig2d;
pub mod riv;
mod scene;
pub mod sfx;
mod shader_check;
pub mod stats;
mod story_functions;
mod stream;
pub mod strings_extract;
mod style_recolour;
mod terrain;
mod terrain_generate;
#[cfg(test)]
#[path = "../../../../tests/temp.rs"]
mod test_dir;
mod texture;
pub mod texture_array;
pub mod tilemap;
mod timeline;
pub mod ts_refs;
pub mod ui_screens;
pub mod vector;
mod vfx;
pub mod vfx2d;
pub mod video;
pub mod vox;

mod execute;
mod options;
mod report;
mod scan;

pub use execute::slice_tone;
use execute::{read_sibling, run, style_atlas};
pub use options::{CookOptions, FontSettings, MeshOptions, Palette, TextureOptions};
use options::{
    anim_settings, conform_off, font_settings, mesh_for, mesh_texture, model_stream_textures,
    own_conform_off, settings_block, texture_kind, texture_stream,
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
    /// An `.ogt` of `type prefab`: cooked as a scene, flattened.
    Prefab,
    Lut,
    Emitter,
    Effect,
    Vfx,
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
    Rive,
    Behaviour,
    Dialogue,
    Quest,
    NavMesh,
    Strings,
    Rig,
    MotionAction,
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
            Kind::Prefab => "prefab",
            Kind::Lut => "lut",
            Kind::Emitter => "emitter",
            Kind::Effect => "effect",
            Kind::Vfx => "vfx",
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
            Kind::Rive => "rive",
            Kind::Behaviour => "behaviour",
            Kind::Dialogue => "dialogue",
            Kind::Quest => "quest",
            Kind::NavMesh => "navmesh",
            Kind::Strings => "strings",
            Kind::Rig => "rig",
            Kind::MotionAction => "motion-action",
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
    /// A streamed texture's whole chain (its blob and the cook's notes),
    /// by glTF image (0 for a texture file) and the `#mips` output's
    /// stamp: encoded once by the first of its two outputs to cook.
    whole: Mutex<BTreeMap<(usize, u64), std::sync::Arc<WholeTexture>>>,
    /// The image the art contract conformed it to (`conform/`): a
    /// texture's own, a flipbook's texture's; read instead of the file,
    /// which is never written. `None`: as on disk.
    conformed: Option<std::sync::Arc<Vec<u8>>>,
    /// A model with a `fracture` block: its fracture asset
    /// (`fracture.rs`), cooked as `<model>#fracture`.
    fracture: Option<og_destruct::FractureAsset>,
}

impl Source {
    fn encoded(&self, per_chunk: Option<u32>) -> Result<std::sync::Arc<audio::Encoded>, CookError> {
        self.encoded
            .get_or_init(|| {
                let settings = audio::SoundSettings::parse(settings_block(self, "sound"))?;
                audio::decode(&self.rel, &self.bytes)
                    .and_then(|pcm| match settings.codec {
                        audio::Codec::Opus => {
                            audio::encode_opus(&pcm, settings.opus_bitrate(pcm.channels))
                        }
                        audio::Codec::Qoa => audio::encode(&pcm, per_chunk),
                    })
                    .map(std::sync::Arc::new)
            })
            .clone()
            .map_err(CookError)
    }
}

type WholeTexture = std::sync::OnceLock<Result<(Vec<u8>, Vec<String>), CookError>>;

impl Source {
    /// The whole chain of a streamed texture, cooked by `cook` once per
    /// image and stamp however many outputs ask (the tail and `#mips`).
    fn whole_texture(
        &self,
        image: usize,
        stamp: u64,
        cook: impl FnOnce() -> Result<(Vec<u8>, Vec<String>), CookError>,
    ) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let cell = self
            .whole
            .lock()
            .expect("no poisoning")
            .entry((image, stamp))
            .or_default()
            .clone();
        cell.get_or_init(cook).clone()
    }
}

/// How a texture output stands to its source's mip chain (G3-3).
#[derive(Clone, Debug)]
enum Split {
    /// The whole chain in one output, as ever; with the report's line on
    /// why a texture that asks to stream does not.
    Whole(Option<String>),
    /// A streamed texture's `<id>`: its mip tail, cut from the whole chain
    /// cooked under its `#mips` output's stamp (this one).
    Tail(u64),
    /// A streamed texture's `<id>#mips`: the whole chain.
    Mips,
}

enum Work<'a> {
    Model(&'a Source, Vec<String>),
    /// A streaming group's glTF stand-in: its coarsest LOD, no textures.
    Proxy(&'a Source),
    /// A model's impostor atlas: 0 the albedo, 1 the normal and depth, 2
    /// the material (roughness, translucency, metallic).
    Impostor(&'a Source, model::ImpostorBake, u8),
    Animation(&'a Source, anim::AnimSettings),
    GltfTexture(&'a Source, usize, u32, Split),
    Texture(&'a Source, u32, Split),
    /// A 2D texture's `#normal` (normal2d.rs): made from it, or from the
    /// file its `normal` line names (its bytes).
    TextureNormal(&'a Source, Option<Vec<u8>>),
    /// A sound, whole or (QOA frames per chunk, chunks) the first of its chunks.
    Sound(&'a Source, Option<(u32, u32)>),
    /// Chunk `n` (1 and up) of a sound cooked in chunks of so many QOA frames.
    SoundChunk(&'a Source, u32, u32),
    Lut(&'a Source),
    Emitter(&'a Source),
    Effect(&'a Source),
    Vfx(&'a Source),
    Style(&'a Source),
    Document(&'a Source),
    Tileset(&'a Source),
    Tilemap(&'a Source, tilemap::MapSettings),
    Flipbook(&'a Source),
    /// An `array true` sheet's texture array blob, and its layer `n`.
    FlipbookArray(&'a Source, Box<og_anim::FlipbookSheet>),
    FlipbookLayer(&'a Source, Box<og_anim::FlipbookSheet>, usize),
    Rig2d(&'a Source),
    Rig2dAtlas(&'a Source),
    /// A style's atlas at a density (1, 2 or 3) with its painted accents
    /// recoloured to the palette (`recolour`), of a texture kind.
    StyleAtlas(&'a Source, u32, u32),
    Shader(&'a Source),
    Post(&'a Source),
    UiMaterial(&'a Source),
    Vector(&'a Source),
    /// A `.riv`'s artboard, by name.
    Rive(&'a Source, String),
    /// A `.riv`'s embedded image, by asset index.
    RiveImage(&'a Source, usize),
    /// A font a `.riv`'s text uses.
    RiveFont(Box<riv::RivFont>),
    Behaviour(&'a Source),
    Dialogue(
        &'a Source,
        &'a BTreeMap<String, og_story::yarn::types::Function>,
    ),
    Quest(&'a Source),
    NavMesh(&'a Source),
    Strings(&'a Source),
    Rig(&'a Source),
    MotionAction(&'a Source),
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
    /// A streamed video's AV1 units and its poster frame.
    VideoStream(PathBuf, u64, Box<video::VideoPlan>),
    VideoPoster(PathBuf, u64, Box<video::VideoPlan>),
    Font(&'a Source, FontSettings),
    Scene(&'a Source),
    /// A fractured model's `#fracture`.
    Fracture(&'a Source),
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

/// A bundle-relative path with its `.` and `..` segments resolved
/// (`garden/cells/../textures/a.png` is `garden/textures/a.png`): how a
/// model's relative file names the asset it is (s59). A `..` past the
/// assets root is kept.
fn normalise_rel(path: &str) -> String {
    let mut out: Vec<&str> = Vec::new();
    for seg in path.split('/') {
        match seg {
            "" | "." => {}
            ".." if out.last().is_some_and(|l| *l != "..") => {
                out.pop();
            }
            s => out.push(s),
        }
    }
    out.join("/")
}

fn assets_root(s: &Source) -> PathBuf {
    let mut root = s.path.clone();
    for _ in s.rel.split('/') {
        root.pop();
    }
    root
}

/// Why a texture that asks to stream its levels above the mip tail
/// (`texture { stream true }`, `model { stream_textures true }`) is cooked
/// whole after all, or `None` when it is split into its tail and `#mips`:
/// a texture in a streaming group (the group's pack holds its whole chain),
/// one with no mips (pixel art), or one whose source's header says it is no
/// larger than the tail.
fn not_split(bytes: &[u8], k: u32, group: Option<&String>) -> Option<String> {
    if let Some(g) = group {
        return Some(format!(
            "not streamed by level: it is in streaming group \"{g}\", whose pack holds its whole chain"
        ));
    }
    if k == format::kind::PIXEL {
        return Some("not streamed by level: pixel art has no mips".into());
    }
    texture::longest_side(bytes)
        .filter(|&n| n <= crate::runtime::MIP_TAIL)
        .map(|n| {
            format!(
                "not streamed by level: its longest side, {n}, is within the mip tail ({})",
                crate::runtime::MIP_TAIL
            )
        })
}

/// Cooks `<root>/assets` into `<root>/cooked`. Per-asset failures are in the
/// report ([`Report::ok`]); an `Err` means the cook could not run at all.
pub fn cook_project(root: &Path, options: &CookOptions) -> Result<Report, CookError> {
    let cache = cache::default_path();
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
    let cook_index = index::Index::open(root, options, texture_cache);
    if let Some(mut report) = cook_index.unchanged() {
        report.elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
        return Ok(report);
    }
    let story_functions = story_functions::read(root).map_err(CookError)?;
    let story_functions_stamp =
        serde_json::to_vec(&story_functions).map_err(|e| CookError(e.to_string()))?;
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
    } = scan::scan_with(&assets_dir, &|path| cook_index.read(path))?;
    let mut consumed: BTreeSet<String> = BTreeSet::new();
    // Pixel effects and flipbooks of animated sources become ordinary
    // sheets (`vfx2d.rs`, `animated.rs`).
    vfx2d::expand(&mut sources, &mut consumed);
    animated::expand(&mut sources, &mut consumed);
    // Effekseer projects become composed effects and what they need (`efk/`).
    efk::expand(&mut sources, &mut consumed);
    // The `type model` texts, built again under the art contract's face
    // budget once the look is read (below).
    let mut model_texts: BTreeMap<String, String> = BTreeMap::new();
    let mut reused_fractures = BTreeSet::new();
    for s in sources.iter_mut() {
        // What a model says it is (`conform character`, `conform effect`),
        // read from its text or import settings before a text becomes a glb.
        let role = if s.kind == Kind::Model {
            match conform::model_role(s) {
                Ok(c) => c,
                Err(e) => {
                    s.error = Some(e);
                    continue;
                }
            }
        } else {
            conform::ModelRole::Prop
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
        // An FBX or OBJ is a model: read by ufbx and written as a glTF
        // binary here (`fbx.rs`), so the rest of the cook sees a .glb; the
        // files it read beside the glb's own (an OBJ's .mtl, the sources
        // of packed images) join its hash and are consumed with it.
        let mut imported = None;
        if s.kind == Kind::Model && fbx::handles(&ext(&s.rel)) {
            match fbx::to_glb(&s.bytes, &s.path, &ext(&s.rel)) {
                Ok(i) => {
                    s.bytes = i.glb;
                    imported = Some((i.read, i.notes));
                }
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
        // A `fracture` block: the glb cut into its chunks (`fracture.rs`).
        if s.kind == Kind::Model {
            let dir = s.path.parent().unwrap_or(Path::new("."));
            let text = model_texts.get(&s.rel).map(String::as_str);
            let input_stamp = stamp(&[
                &s.bytes,
                text.unwrap_or_default().as_bytes(),
                s.settings.as_ref().map_or("", |(text, _)| text).as_bytes(),
            ]);
            match fracture::Settings::of(text, s.settings.as_ref().map(|(_, d)| d)).and_then(|f| {
                f.map(|f| {
                    let key = cache::key(
                        "fracture",
                        &[
                            &s.bytes,
                            text.unwrap_or_default().as_bytes(),
                            s.settings.as_ref().map_or("", |(text, _)| text).as_bytes(),
                        ],
                    );
                    fracture_cache::run(texture_cache, &key, input_stamp, dir, || {
                        fracture::fracture_glb(&s.bytes, dir, &f)
                    })
                })
                .transpose()
            }) {
                Ok(Some((f, cached))) => {
                    s.bytes = f.glb;
                    s.fracture = Some(f.asset);
                    // Its images now live in the binary; the files stay
                    // part of the source (its hash, what it consumed).
                    let (read, notes) = imported.take().unwrap_or_default();
                    if cached {
                        reused_fractures.insert(s.rel.clone());
                    }
                    let mut all = read;
                    all.extend(f.read);
                    imported = Some((all, notes));
                }
                Ok(None) => {}
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
                    g.role = role;
                    if let Some((read, notes)) = imported {
                        g.external.extend(read);
                        g.warnings.extend(notes);
                    }
                    let base = s
                        .rel
                        .rsplit_once('/')
                        .map_or(String::new(), |(d, _)| format!("{d}/"));
                    for (p, _) in &g.external {
                        // As the asset it names: `cells/../textures/a.png` is
                        // `textures/a.png` (s59).
                        consumed.insert(normalise_rel(&format!("{base}{p}")));
                    }
                    s.gltf = Some(g);
                }
                Err(e) => s.error = Some(e),
            }
        }
        let mut h = StableHasher::new();
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
        if s.kind == Kind::Rig2d {
            consumed.extend(autorig::part_files(
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
    // A file a model references stays an asset of its own as well when it
    // has import settings of its own (which only a texture cooked on its own
    // reads) or a terrain names it as a layer (by asset id), s59.
    for s in &sources {
        if s.kind == Kind::Texture && s.settings.is_some() {
            consumed.remove(&s.rel);
        }
        if s.kind == Kind::Terrain {
            for layer in
                terrain::layer_files(&s.rel, &assets_dir, &String::from_utf8_lossy(&s.bytes))
            {
                consumed.remove(&layer);
            }
        }
    }
    // Files a .gltf references are part of that model, not assets of their own.
    sources.retain(|s| !consumed.contains(&s.rel));
    ignored.retain(|r| !consumed.contains(r));
    let known: BTreeSet<String> = sources.iter().map(|s| s.rel.clone()).collect();
    // Prefab and scene texts by asset id: scenes and prefabs flatten
    // through the prefabs, and a sub-scene (`scene "<id>"`) is checked
    // against the scenes (a scene, and no cycle).
    let prefab_texts: BTreeMap<String, String> = sources
        .iter()
        .filter(|s| matches!(s.kind, Kind::Prefab | Kind::Scene))
        .map(|s| {
            (
                s.rel.clone(),
                String::from_utf8_lossy(&s.bytes).into_owned(),
            )
        })
        .collect();
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
            // A fractured model keeps its chunks; the decimation that
            // follows holds it to the budget.
            if s.fracture.is_some() {
                continue;
            }
            if !c.applies(g) {
                continue;
            }
            let (budget, _) = c.budget(g);
            let role = g.role;
            let written = conform::mesh::drawn_triangles(g);
            let rebuilt = model_text::to_glb_within(text, budget).and_then(|(glb, detail)| {
                if detail >= 1.0 {
                    return Ok(None);
                }
                let dir = s.path.parent().unwrap_or(Path::new("."));
                let mut g = model::load(&glb, dir)?;
                g.role = role;
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
    // glTF textures of models outside streaming groups are cooked once per
    // image and use, whichever models use them: a scene exported in many
    // glTFs (cells, tiles) repeats its images in every file, and a copy per
    // model multiplied the cooked bytes, the load's transcoding and the
    // GPU's memory (og-runtime's add_gltf shares them by these ids).
    let mut planned_textures = std::collections::HashSet::new();
    let mut parsed_refs: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut document_themes = vec![og_ui::Theme::default()];
    let mut style_tokens_hash = StableHasher::new();
    for source in sources.iter().filter(|s| s.kind == Kind::Style) {
        style_tokens_hash.write_str(&source.rel);
        style_tokens_hash.write(&source.hash.to_le_bytes());
        if let Ok(style) = og_ui::parse_style(&String::from_utf8_lossy(&source.bytes)) {
            document_themes.push(style.theme);
        }
    }
    let style_tokens_hash = style_tokens_hash.finish().to_le_bytes();
    for (si, s) in sources.iter().enumerate() {
        if s.error.is_some() {
            continue;
        }
        // These cookers consume only bytes/options. The other kinds resolve
        // relative references or write source-derived names/sub-asset ids;
        // their location remains part of the action, not the content hash.
        let hash = match s.kind {
            Kind::Texture
            | Kind::Sound
            | Kind::Font
            | Kind::Lut
            | Kind::Emitter
            | Kind::Document
            | Kind::Dialogue
            | Kind::Quest
            | Kind::Strings
            | Kind::Shader
            | Kind::Post
            | Kind::Tileset
            | Kind::Rig
            | Kind::MotionAction
            | Kind::Palette => stamp(&[&s.hash.to_le_bytes(), ext(&s.rel).as_bytes()]),
            _ => stamp(&[&s.hash.to_le_bytes(), s.rel.as_bytes()]),
        }
        .to_le_bytes();
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
                        // An effect's mesh is held to no face budget.
                        if !d.is_empty() && g.role == conform::ModelRole::Effect {
                            d.push_str(" role effect");
                        }
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
                let stream_textures = match model_stream_textures(s) {
                    Ok(b) => b,
                    Err(e) => {
                        early_errors.push((s.rel.clone(), "model", e));
                        continue;
                    }
                };
                let mut ids = Vec::new();
                let grouped = group_of.contains_key(&s.rel);
                for (i, u) in g.textures.iter().enumerate() {
                    let image_hash = hash_bytes(&g.images[u.image]).to_le_bytes();
                    // One cooked texture per image and use: its kind, and
                    // whether it streams its levels (a model's
                    // `stream_textures` splits it into a tail and a chain).
                    // A model's own unless an art contract makes it depend
                    // on the model (a density is per metre of the model) or
                    // it is in a streaming group's pack.
                    let shared = !grouped && texture_desc.is_empty();
                    let id = if shared {
                        crate::runtime::gltf_texture_id(
                            u64::from_le_bytes(image_hash),
                            u.kind,
                            stream_textures,
                        )
                    } else {
                        format!("{}#tex{i}", s.rel)
                    };
                    parsed_refs
                        .entry(s.rel.clone())
                        .or_default()
                        .insert(id.clone());
                    if shared && !planned_textures.insert(id.clone()) {
                        ids.push(id);
                        continue;
                    }
                    // A density is per metre of the model: its hash too.
                    let kind = u.kind.to_le_bytes();
                    let st = |word: &[u8]| {
                        let mut parts: Vec<&[u8]> =
                            vec![b"texture", tex_desc.as_bytes(), &image_hash, &kind];
                        if !word.is_empty() {
                            parts.push(word);
                        }
                        with(&parts, &texture_desc, &hash)
                    };
                    let not_split = stream_textures
                        .then(|| not_split(&g.images[u.image], u.kind, group_of.get(&s.rel)));
                    match not_split {
                        Some(None) => {
                            let mips = st(b"mips");
                            outs.push((
                                id.clone(),
                                format::Texture::NAME,
                                st(b"tail"),
                                Work::GltfTexture(s, u.image, u.kind, Split::Tail(mips)),
                            ));
                            outs.push((
                                crate::runtime::mips_id(&id),
                                format::Texture::NAME,
                                mips,
                                Work::GltfTexture(s, u.image, u.kind, Split::Mips),
                            ));
                        }
                        why => outs.push((
                            id.clone(),
                            format::Texture::NAME,
                            st(b""),
                            Work::GltfTexture(s, u.image, u.kind, Split::Whole(why.flatten())),
                        )),
                    }
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
                if s.fracture.is_some() {
                    outs.push((
                        format!("{}#fracture", s.rel),
                        format::Fracture::NAME,
                        // The asset's layout too: a new og_destruct version
                        // recooks what the runtime would refuse.
                        stamp(&[
                            b"fracture",
                            &og_destruct::ASSET_VERSION.to_le_bytes(),
                            &hash,
                        ]),
                        Work::Fracture(s),
                    ));
                }
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
                // The impostor's three atlases (`model { impostor true }`),
                // baked from the model's source.
                if let Some(b) = mesh_for(s, options)
                    .ok()
                    .and_then(|m| m.impostor)
                    .map(|b| b.resolved(g))
                {
                    let size = [b.grid.to_le_bytes(), b.frame.to_le_bytes()].concat();
                    for (kind, id) in model::impostor_ids(&s.rel).into_iter().enumerate() {
                        // `3`: the atlas's layout version (2, the
                        // hemi-octahedron; 3, 16 samples a texel, filtered
                        // textures and the material atlas), so an atlas
                        // baked by an older layout is baked again.
                        let st = with(
                            &[
                                b"impostor",
                                &[3, kind as u8],
                                &size,
                                tex_desc.as_bytes(),
                                &hash,
                            ],
                            &model_desc,
                            &[],
                        );
                        outs.push((
                            id,
                            format::Texture::NAME,
                            st,
                            Work::Impostor(s, b, kind as u8),
                        ));
                    }
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
            Kind::Texture => match texture_kind(s).and_then(|k| Ok((k, texture_stream(s, k)?))) {
                Ok((k, stream)) => {
                    // A mesh's texture under the art contract: its desc too.
                    let mesh = mesh_texture(s, options)
                        .map(|c| c.texture_desc())
                        .unwrap_or_default();
                    let kind = k.to_le_bytes();
                    let st = |word: &[u8]| {
                        let mut parts: Vec<&[u8]> =
                            vec![b"texture", tex_desc.as_bytes(), &hash, &kind];
                        if !mesh.is_empty() {
                            parts.push(mesh.as_bytes());
                        }
                        if !word.is_empty() {
                            parts.push(word);
                        }
                        stamp(&parts)
                    };
                    let source = s.conformed.as_deref().map_or(&s.bytes[..], |b| &b[..]);
                    match stream.then(|| not_split(source, k, group_of.get(&s.rel))) {
                        Some(None) => {
                            let mips = st(b"mips");
                            outs.push((
                                s.rel.clone(),
                                format::Texture::NAME,
                                st(b"tail"),
                                Work::Texture(s, k, Split::Tail(mips)),
                            ));
                            outs.push((
                                crate::runtime::mips_id(&s.rel),
                                format::Texture::NAME,
                                mips,
                                Work::Texture(s, k, Split::Mips),
                            ));
                        }
                        why => outs.push((
                            s.rel.clone(),
                            format::Texture::NAME,
                            st(b""),
                            Work::Texture(s, k, Split::Whole(why.flatten())),
                        )),
                    }
                    // Its normal map for 2D lighting (`normal auto` or a file).
                    match normal2d::setting(settings_block(s, "texture")) {
                        Ok(None) => {}
                        Ok(Some(setting)) => {
                            let file = match &setting {
                                normal2d::NormalSetting::Auto => Ok(None),
                                normal2d::NormalSetting::File(name) => {
                                    let path = s.path.parent().unwrap_or(Path::new(".")).join(name);
                                    std::fs::read(&path)
                                        .map(Some)
                                        .map_err(|e| format!("normal \"{name}\": {e}"))
                                }
                            };
                            match file {
                                Ok(file) => {
                                    let st = stamp(&[
                                        b"texture normal",
                                        tex_desc.as_bytes(),
                                        &hash,
                                        file.as_deref().unwrap_or(b"auto"),
                                    ]);
                                    outs.push((
                                        format!("{}#normal", s.rel),
                                        format::Texture::NAME,
                                        st,
                                        Work::TextureNormal(s, file),
                                    ));
                                }
                                Err(e) => early_errors.push((s.rel.clone(), "texture", e)),
                            }
                        }
                        Err(e) => early_errors.push((s.rel.clone(), "texture", e)),
                    }
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
                    .inspect(|r| {
                        parsed_refs
                            .entry(s.rel.clone())
                            .or_default()
                            .insert(r.clone());
                    })
                    .map(|r| format!("{r}={}", known.contains(crate::runtime::asset_id(&r))))
                    .collect();
                let fields = fx_field::stamp(s, &text);
                let st = stamp(&[
                    b"emitter",
                    &hash,
                    refs.join("\n").as_bytes(),
                    fields.as_bytes(),
                ]);
                outs.push((s.rel.clone(), format::Emitter::NAME, st, Work::Emitter(s)));
            }
            Kind::Effect => {
                let text = String::from_utf8_lossy(&s.bytes);
                let refs: Vec<String> = og_bundle::effect::effect_references(&text)
                    .into_iter()
                    .map(|r| format!("{r}={}", known.contains(crate::runtime::asset_id(&r))))
                    .collect();
                let st = stamp(&[b"effect", &hash, refs.join("\n").as_bytes()]);
                outs.push((s.rel.clone(), format::Effect::NAME, st, Work::Effect(s)));
            }
            Kind::Vfx => {
                let st = vfx::stamp_inputs(s, &|id| known.contains(id));
                let st = stamp(&[b"vfx", &hash, st.as_bytes()]);
                outs.push((s.rel.clone(), format::Vfx::NAME, st, Work::Vfx(s)));
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
                    .map(|src| execute::read_beside(s, &src).unwrap_or_default())
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
            Kind::Rive => match riv::plan(s, &options.font) {
                Ok((names, images, fonts)) => {
                    for f in fonts {
                        // The font cook's version and settings, and the
                        // file's bytes (a referenced font's own).
                        let st = stamp(&[
                            b"rive font 1 font 7",
                            &hash,
                            &f.index.to_le_bytes(),
                            f.settings.describe().as_bytes(),
                            &hash_bytes(&f.bytes).to_le_bytes(),
                        ]);
                        outs.push((
                            format!("{}#font{}", s.rel, f.index),
                            format::Font::NAME,
                            st,
                            Work::RiveFont(Box::new(f)),
                        ));
                    }
                    for n in images {
                        let st = stamp(&[b"rive image", &hash, &n.to_le_bytes()]);
                        outs.push((
                            format!("{}#image{n}", s.rel),
                            format::Texture::NAME,
                            st,
                            Work::RiveImage(s, n),
                        ));
                    }
                    for name in names {
                        let st = stamp(&[b"rive 3", &hash, name.as_bytes()]);
                        outs.push((
                            format!("{}#{name}", s.rel),
                            format::RiveArtboard::NAME,
                            st,
                            Work::Rive(s, name),
                        ));
                    }
                }
                Err(e) => early_errors.push((s.rel.clone(), "rive", e)),
            },
            Kind::Style => {
                let text = String::from_utf8_lossy(&s.bytes);
                let refs: Vec<String> = og_ui::parse_style(&text)
                    .map(|mut f| {
                        f.resolve(&s.rel, |id| known.contains(crate::runtime::asset_id(id)));
                        f.references()
                    })
                    .unwrap_or_default()
                    .into_iter()
                    .inspect(|r| {
                        parsed_refs
                            .entry(s.rel.clone())
                            .or_default()
                            .insert(r.clone());
                    })
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
                // Its painted art recoloured to the palette, at each density
                // the atlas has (docs/art.md §2.7).
                for (density, id) in style_recolour::atlases(s, &known) {
                    let source = sources.iter().find(|x| x.rel == id);
                    let kind = source.map_or(Ok(format::kind::COLOR), texture_kind);
                    match kind {
                        Ok(k) => {
                            let bytes = source.map(|x| x.hash).unwrap_or_default().to_le_bytes();
                            let st = stamp(&[
                                b"style atlas",
                                tex_desc.as_bytes(),
                                &hash,
                                &bytes,
                                &k.to_le_bytes(),
                            ]);
                            outs.push((
                                style_recolour::atlas_id(&s.rel, density),
                                format::Texture::NAME,
                                st,
                                Work::StyleAtlas(s, density, k),
                            ));
                        }
                        Err(e) => early_errors.push((s.rel.clone(), "style", e)),
                    }
                }
            }
            Kind::Document => {
                let text = String::from_utf8_lossy(&s.bytes);
                if let Err(errors) = og_ui::UiDocument::parse(&s.rel, &text)
                    .and_then(|doc| doc.validate_tokens(&document_themes))
                {
                    early_errors.push((s.rel.clone(), "document", errors.join("\n")));
                    continue;
                }
                let st = stamp(&[b"document tokens", &hash, &style_tokens_hash]);
                outs.push((s.rel.clone(), format::Document::NAME, st, Work::Document(s)));
            }
            Kind::Tileset => {
                let deps = tilemap::references_of_tileset(&s.bytes, &|id| known.contains(id));
                assets::record_dependencies(&mut parsed_refs, &s.rel, &deps);
                let st = stamp(&[b"tileset", &hash, deps.as_bytes()]);
                outs.push((s.rel.clone(), format::Tileset::NAME, st, Work::Tileset(s)));
            }
            Kind::Tilemap => match tilemap::MapSettings::parse(settings_block(s, "tilemap")) {
                Ok(m) => {
                    let refs = tilemap::references(&s.rel, &s.bytes, &m, &|id| known.contains(id));
                    assets::record_dependencies(&mut parsed_refs, &s.rel, &refs.join("\n"));
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
                assets::record_dependencies(&mut parsed_refs, &s.rel, &deps);
                let st = stamp(&[b"rig2d", &hash, deps.as_bytes()]);
                outs.push((s.rel.clone(), format::Rig2D::NAME, st, Work::Rig2d(s)));
                // Rigged from layered art: its parts packed into a texture beside it.
                if Document::parse(&String::from_utf8_lossy(&s.bytes))
                    .is_ok_and(|d| autorig::is_autorig(&d))
                {
                    let st = stamp(&[b"rig2d atlas", tex_desc.as_bytes(), &hash, deps.as_bytes()]);
                    outs.push((
                        autorig::atlas_id(&s.rel),
                        format::Texture::NAME,
                        st,
                        Work::Rig2dAtlas(s),
                    ));
                }
            }
            Kind::Flipbook => {
                let deps = flipbook::dependency_hashes(s);
                assets::record_dependencies(&mut parsed_refs, &s.rel, &deps);
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
                // The stamp covers every subtree its `use` nodes reach.
                let deps = e6::behaviour_dependencies(s);
                let st = stamp(&[b"behaviour", &hash, deps.as_bytes()]);
                outs.push((
                    s.rel.clone(),
                    format::Behaviour::NAME,
                    st,
                    Work::Behaviour(s),
                ));
            }
            Kind::Dialogue => {
                let st = stamp(&[b"dialogue", &hash, &story_functions_stamp]);
                outs.push((
                    s.rel.clone(),
                    format::Dialogue::NAME,
                    st,
                    Work::Dialogue(s, &story_functions),
                ));
            }
            Kind::Quest => {
                let st = stamp(&[b"quest", &hash]);
                outs.push((s.rel.clone(), format::Quest::NAME, st, Work::Quest(s)));
            }
            Kind::Strings => {
                let st = stamp(&[b"strings", &hash]);
                outs.push((s.rel.clone(), format::Strings::NAME, st, Work::Strings(s)));
            }
            Kind::Rig => {
                let st = stamp(&[b"rig", &hash]);
                outs.push((s.rel.clone(), format::Rig::NAME, st, Work::Rig(s)));
            }
            Kind::MotionAction => {
                let st = stamp(&[b"motion-action", &hash]);
                outs.push((
                    s.rel.clone(),
                    format::MotionAction::NAME,
                    st,
                    Work::MotionAction(s),
                ));
            }
            Kind::Icons => {
                // The stamp covers every SVG the set names.
                let deps = icons::dependencies(s);
                let st = stamp(&[b"icons", &hash, deps.as_bytes()]);
                outs.push((s.rel.clone(), format::Font::NAME, st, Work::Icons(s)));
            }
            Kind::Terrain => {
                let deps = terrain::terrain_dependencies(s, &|id| known.contains(id));
                assets::record_dependencies(&mut parsed_refs, &s.rel, &deps);
                let st = stamp(&[b"terrain 2", &hash, deps.as_bytes()]);
                outs.push((s.rel.clone(), format::Terrain::NAME, st, Work::Terrain(s)));
            }
            Kind::Lightmap => {
                // The stamp covers its pages file.
                let deps = lightmap::dependencies(s);
                assets::record_dependencies(&mut parsed_refs, &s.rel, &deps);
                let st = stamp(&[b"lightmap 1", &hash, deps.as_bytes()]);
                outs.push((s.rel.clone(), format::Lightmap::NAME, st, Work::Lightmap(s)));
            }
            Kind::Timeline => {
                // The stamp covers every caption file it imports.
                let deps = timeline::dependencies(s);
                assets::record_dependencies(&mut parsed_refs, &s.rel, &deps);
                let st = stamp(&[b"timeline 1", &hash, deps.as_bytes()]);
                outs.push((s.rel.clone(), format::Timeline::NAME, st, Work::Timeline(s)));
            }
            Kind::TextureArray => {
                let text = String::from_utf8_lossy(&s.bytes);
                let root = texture_array::assets_dir(&s.path, &s.rel);
                match texture_array::parse(&s.rel, &text, &|p| root.join(p).is_file()) {
                    Ok(decl) => {
                        let deps = texture_array::dependencies(&root, &decl);
                        assets::record_dependencies(&mut parsed_refs, &s.rel, &deps);
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
                        let encoder = if d.stream {
                            video::stream_encoder()?
                        } else {
                            ""
                        };
                        let plan = video::plan(&d, video::probe(&src)?, encoder)?;
                        Ok((src, hash_bytes(&bytes), plan))
                    });
                match planned {
                    Ok((src, src_hash, plan)) => {
                        let described = plan.describe();
                        let base = [
                            b"video 2".as_slice(),
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
                        if plan.decl.stream {
                            outs.push((
                                video::stream_id(&s.rel),
                                format::VideoStream::NAME,
                                stamp(&[&base[..], &[b"stream".as_slice()]].concat()),
                                Work::VideoStream(src.clone(), src_hash, plan.clone()),
                            ));
                            if let Some(size) = og_video::stream::fallback_size(plan.size) {
                                outs.push((
                                    video::stream_fallback_id(&s.rel),
                                    format::VideoStream::NAME,
                                    stamp(&[&base[..], &[b"stream720".as_slice()]].concat()),
                                    Work::VideoStream(
                                        src.clone(),
                                        src_hash,
                                        Box::new(video::fallback_plan(&plan, size)),
                                    ),
                                ));
                            }
                            outs.push((
                                video::poster_id(&s.rel),
                                format::Texture::NAME,
                                stamp(&[&base[..], &[b"poster".as_slice()]].concat()),
                                Work::VideoPoster(src.clone(), src_hash, plan.clone()),
                            ));
                        } else {
                            let frames = video::frames_id(&s.rel);
                            outs.push((
                                frames.clone(),
                                format::TextureArray::NAME,
                                stamp(&[&base[..], &[b"array".as_slice()]].concat()),
                                Work::VideoArray(s, plan.clone()),
                            ));
                            for n in 0..plan.frames as usize {
                                let st =
                                    stamp(&[&base[..], &[&(n as u32).to_le_bytes()[..]]].concat());
                                outs.push((
                                    texture_array::layer_id(&frames, n),
                                    format::Texture::NAME,
                                    st,
                                    Work::VideoFrame(src.clone(), src_hash, plan.clone(), n),
                                ));
                            }
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
                assets::record_dependencies(&mut parsed_refs, &s.rel, &deps);
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
                assets::record_dependencies(&mut parsed_refs, &s.rel, &deps);
                let st = stamp(&[b"look 1", &hash, deps.as_bytes()]);
                outs.push((s.rel.clone(), format::Look::NAME, st, Work::Look(s)));
            }
            Kind::Palette => {
                let st = stamp(&[b"palette 1", &hash]);
                outs.push((s.rel.clone(), format::Palette::NAME, st, Work::Palette(s)));
            }
            Kind::NavMesh => {
                let deps = e6::navmesh_dependencies(s);
                assets::record_dependencies(&mut parsed_refs, &s.rel, &deps);
                // Re-bake cached terrain surfaces with adaptive diagonals.
                let st = stamp(&[b"navmesh 2", &hash, deps.as_bytes()]);
                outs.push((s.rel.clone(), format::NavMesh::NAME, st, Work::NavMesh(s)));
            }
            Kind::Font => match font_settings(s, &options.font) {
                Ok(f) => {
                    // "font 2": the Unicode spaces a font lacks are cooked as empty
                    // glyphs; "font 3": the field is of each glyph's outline
                    // union; "font 4": negative-bearing fill and post-sign error
                    // correction; "font 5": every face carries OpenType data
                    // for paragraph layout; "font 6": default substitutions
                    // and referenced contextual lookups; "font 7": layout-only
                    // tables without runtime outlines. Older fonts re-cook.
                    let st = stamp(&[b"font 7", f.describe().as_bytes(), &hash]);
                    outs.push((s.rel.clone(), format::Font::NAME, st, Work::Font(s, f)));
                }
                Err(e) => early_errors.push((s.rel.clone(), "font", e)),
            },
            Kind::Scene | Kind::Prefab => {
                let text = String::from_utf8_lossy(&s.bytes);
                // Every prefab it reaches, transitively, with its text's
                // hash: editing a prefab re-cooks what places it.
                let texts = |id: &str| prefab_texts.get(id).cloned();
                let reached = prefab::reached(&text, &texts);
                let mut refs: BTreeSet<String> = scene::references(&text).into_iter().collect();
                let mut placed = Vec::new();
                for id in &reached {
                    let t = prefab_texts.get(id);
                    if let Some(t) = t {
                        refs.extend(scene::references(t));
                    }
                    placed.push(format!(
                        "{id}#{}",
                        t.map_or(0, |t| hash_bytes(t.as_bytes()))
                    ));
                }
                parsed_refs
                    .entry(s.rel.clone())
                    .or_default()
                    .extend(refs.iter().chain(reached.iter()).cloned());
                let refs: Vec<String> = refs
                    .into_iter()
                    .map(|r| format!("{r}={}", known.contains(&r)))
                    .collect();
                // A scene that places no prefab keeps the stamp it had.
                let st = if placed.is_empty() && s.kind == Kind::Scene {
                    stamp(&[b"scene", &hash, &schema_hash, refs.join("\n").as_bytes()])
                } else {
                    stamp(&[
                        b"scene",
                        &hash,
                        &schema_hash,
                        refs.join("\n").as_bytes(),
                        placed.join("\n").as_bytes(),
                        &[options.publish as u8],
                    ])
                };
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
                    && std::fs::read(cooked_dir.join(&o.file)).is_ok_and(|b| {
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

    let project_key = texture_cache
        .and_then(|_| cook_index.content_key())
        .map(|input| {
            let dependencies: Vec<_> = sources
                .iter()
                .flat_map(|s| {
                    s.gltf.iter().flat_map(move |g| {
                        g.external.iter().map(move |(name, bytes)| {
                            (&s.rel, name, blake3::hash(bytes).to_hex().to_string())
                        })
                    })
                })
                .collect();
            cache::key(
                "outputs",
                &[
                    input.as_bytes(),
                    &serde_json::to_vec(&dependencies).expect("dependency strings"),
                ],
            )
        });
    // Cook what is stale, in parallel; results land by plan index.
    type Done = (Result<(Vec<u8>, Vec<String>), CookError>, f64, f64, u32);
    let results: Vec<Mutex<Option<Done>>> = plan.iter().map(|_| Mutex::new(None)).collect();
    let todo: Vec<usize> = (0..plan.len())
        .filter(|&i| plan[i].work.is_some())
        .collect();
    let next = std::sync::atomic::AtomicUsize::new(0);
    std::thread::scope(|scope| {
        for worker in 0..threads.min(todo.len()) {
            let (next, results, plan, schemas, known, todo, prefab_texts) = (
                &next,
                &results,
                &plan,
                &schemas,
                &known,
                &todo,
                &prefab_texts,
            );
            let budgets = &budgets;
            let project_key = project_key.as_deref();
            scope.spawn(move || {
                loop {
                    let k = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some(&i) = todo.get(k) else { break };
                    let p = &plan[i];
                    let t = Instant::now();
                    let start = t.duration_since(started).as_secs_f64() * 1000.0;
                    let r = action_cache::run(texture_cache, p, options, project_key, || {
                        run(p, options, threads, schemas, known, prefab_texts, budgets)
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
            notes: if reused_fractures.contains(&s.rel) {
                vec!["fracture: inputs unchanged, reused prefracture".into()]
            } else {
                Vec::new()
            },
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
                    (Some(OutputBytes::from(bytes)), true, ms, start, worker)
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
    // Resolve all aliases before writing: a changed canonical source may
    // overwrite the file an unchanged alias still reads. Compare the bytes,
    // not just their hash, and choose filenames in id order.
    let mut shared: BTreeMap<(u64, u64), Vec<(String, OutputBytes)>> = BTreeMap::new();
    let mut ordered: Vec<_> = new_outputs
        .iter_mut()
        .filter(|(si, _)| reports[**si].error.is_none())
        .flat_map(|(_, outs)| outs.iter_mut())
        .collect();
    ordered.sort_by(|a, b| a.0.id.cmp(&b.0.id));
    for (m, bytes, rep) in ordered {
        let data = match bytes.take() {
            Some(data) => data,
            None => {
                let file = &old_outputs[m.id.as_str()].file;
                std::fs::read(cooked_dir.join(file))
                    .map_err(|e| CookError(format!("cooked/{file}: {e}")))?
                    .into()
            }
        };
        let bucket = shared.entry((m.hash, m.bytes)).or_default();
        if let Some((file, _)) = bucket.iter().find(|(_, b)| *b == data) {
            m.file = file.clone();
        } else {
            bucket.push((m.file.clone(), data.clone()));
            *bytes = Some(data);
        }
        rep.file = m.file.clone();
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
        motion: None,
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
        files.dedup_by(|a, b| a.0 == b.0);
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
    // og.motion's model, for a project whose code uses it (in the manifest,
    // outside its outputs: never in the boot bundle).
    let mut motion_warnings = Vec::new();
    manifest.motion = motion::cook(root, &cooked_dir, &mut keep, &mut motion_warnings)?;
    let mut removed = Vec::new();
    let old_files = old
        .outputs()
        .map(|o| &o.file)
        .chain(old.streams.iter().map(|g| &g.file))
        .chain(old.motion.iter().map(|m| &m.file));
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
    let index = assets::build(root, &manifest, &sources, &parsed_refs)?;
    index.write(&cooked_dir)?;
    index.write_publish_plan(root, &manifest)?;
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
    let mut warnings = motion_warnings;
    // Diagnostics describe source assets, never change their cooked blobs.
    // Run on cached cooks as well so a warning does not disappear on rerun.
    for (source, report) in sources.iter().zip(&mut reports) {
        if report.error.is_some() {
            continue;
        }
        if let Some(gltf) = &source.gltf {
            let notes = anim_settings(source).and_then(|settings| {
                let mut notes =
                    model::skin_report::report(gltf, &assets_dir, settings.rig.as_deref())?;
                notes.extend(gltf.emissive_warnings()?);
                Ok(notes)
            });
            match notes {
                Ok(notes) => {
                    for note in notes {
                        if note.starts_with("warning:") {
                            warnings.push(format!("assets/{}: {note}", source.rel));
                        }
                        report.notes.push(note);
                    }
                }
                Err(error) => {
                    let note = format!("model diagnostics unavailable: {error}");
                    warnings.push(format!("assets/{}: {note}", source.rel));
                    report.notes.push(note);
                }
            }
        }
    }
    for (rel, what) in e6::check_strings(&tables) {
        warnings.push(format!("assets/{rel}: {what}"));
        if let Some(r) = reports.iter_mut().find(|r| r.source == rel) {
            r.notes.push(what);
        }
    }
    // Dialogues and quests: every key they show is in the base table.
    let stories: Vec<(&str, &[u8])> = sources
        .iter()
        .filter(|s| matches!(s.kind, Kind::Dialogue | Kind::Quest))
        .map(|s| (s.rel.as_str(), s.bytes.as_slice()))
        .collect();
    for (rel, what) in e6::check_story_keys(&tables, &stories, &story_functions) {
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
    // A prefab override whose target is gone: cooked (dropped), and said
    // (a publish's cook refuses it).
    for r in reports
        .iter()
        .filter(|r| r.kind == "scene" || r.kind == "prefab")
    {
        for n in r.notes.iter().filter(|n| n.contains("orphaned override")) {
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
    efk::warnings(&sources, &mut reports, &mut warnings);
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
    let report = Report {
        conform: conformed,
        warnings,
        root: root.display().to_string(),
        cooker: COOKER_VERSION.to_string(),
        texture_encoder: texture_encoder.unwrap_or_else(|e| format!("none ({e})")),
        assets: reports,
        ignored,
        removed,
        elapsed_ms: started.elapsed().as_secs_f64() * 1000.0,
    };
    let assets_path = assets_dir.canonicalize().ok();
    let bounded = sources.iter().all(|s| {
        s.kind != Kind::Video
            && s.gltf.as_ref().is_none_or(|g| {
                g.external.iter().all(|(name, _)| {
                    s.path
                        .parent()
                        .unwrap_or(&assets_dir)
                        .join(name)
                        .canonicalize()
                        .ok()
                        .zip(assets_path.as_ref())
                        .is_some_and(|(path, assets)| path.starts_with(assets))
                })
            })
    });
    cook_index.save(&report, bounded);
    if let Some(path) = texture_cache {
        let _ = cache::Store::new(path.to_owned()).gc_if_due(cache::max_bytes());
    }
    Ok(report)
}

type OutputBytes = std::sync::Arc<[u8]>;

/// A planned output's manifest entry, its new bytes (None: up to date) and report line.
type NewOutput = (ManifestOutput, Option<OutputBytes>, OutputReport);
