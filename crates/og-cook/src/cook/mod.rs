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
//!
//! `pipeline/` holds the cook's own stages, `formats/` the importers of
//! asset file formats and `systems/` the cooks of the world systems;
//! `conform/` is the art contract both share. Each directory's modules are
//! used back here, so a module is named `cook::<module>` wherever it lives.

mod pipeline;
pub use pipeline::{asset_move, assets, cache, stats, ts_refs};
use pipeline::{
    execute, import_defaults, index, kinds, options, output, plan, report, scan, stage, stream,
};
mod systems;
pub use systems::{animated, efk, look, probes, strings_extract, tilemap, vfx2d};
use systems::{
    autorig, behaviour, effect, environment, flipbook, fracture, fracture_cache, fx_field,
    lightmap, navmesh, rig2d, shader_check, story, story_functions, strings, terrain, vfx,
};
mod formats;
pub use formats::{
    anim, audio, bc6h, cube_source, icons, ktx2_source, model_text, prefab, sfx, texture_array,
    video, vox,
};
use formats::{fbx, font, font_pack, model, motion, normal2d, psd, scene, texture, timeline};
pub mod conform;
pub use conform::{ConformAction, ConformEntry};
pub mod riv;
#[cfg(test)]
#[path = "../../../../tests/temp.rs"]
mod test_dir;
pub mod ui_components;
pub mod ui_screens;
pub mod vector;

use execute::{read_sibling, run};
pub use options::{CookOptions, FontSettings, MeshOptions, Palette, TextureOptions};
use options::{
    anim_settings, conform_off, font_settings, mesh_for, mesh_texture, model_stream_textures,
    own_conform_off, settings_block, texture_kind, texture_stream,
};
pub use report::{AssetReport, COOK_TIMES, CookTimes, KindTotals, OutputReport, Report, StageTime};
use report::{CONFORM_LOG, ConformLog, conform_log_text, conform_note, read_conform_log};
pub use scan::own_lines;
use scan::{Scan, ext, output_group, plan_groups, scan};

use crate::format::{self, BlobKind, COOKER_VERSION};
use crate::runtime::{MANIFEST, Manifest, ManifestAsset, ManifestOutput};
use kinds::{Exec, Job, KindDef};
use og_core::Schema;
use og_core::hash::{StableHasher, hash_bytes};
use og_core::ogt::Document;
use output::*;
use plan::*;
use stage::*;
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
    /// A font pack's `extension.json` (`kind font`, docs/ui.md §3.4).
    FontPack,
    Rig,
    MotionAction,
    Icons,
    Look,
    Palette,
    TextureArray,
    Video,
    /// A kind its system's module declares (`pipeline/kinds.rs`).
    System(&'static KindDef),
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
            Kind::FontPack => "font-pack",
            Kind::Rig => "rig",
            Kind::MotionAction => "motion-action",
            Kind::Icons => "icons",
            Kind::Look => "look",
            Kind::Palette => "palette",
            Kind::TextureArray => "texture-array",
            Kind::Video => "video",
            Kind::System(k) => k.name,
        }
    }
}

pub(crate) struct Source {
    rel: String,
    path: PathBuf,
    kind: Kind,
    bytes: Vec<u8>,
    settings: Option<(String, Document)>,
    /// The files `settings` came from, as asset ids: the directories'
    /// `import.ogt` from `assets/` down, then its own `<file>.ogt`.
    settings_files: Vec<String>,
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
    /// Whether it has settings of its own (its `<file>.ogt`, or the cook's
    /// for a source it made), not only its directories'.
    fn own_settings(&self) -> bool {
        self.settings.is_some()
            && self
                .settings_files
                .last()
                .is_none_or(|f| f.strip_suffix(".ogt") == Some(&self.rel))
    }

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
    /// A glTF image cooked as a texture: its index, kind, cut-off as a
    /// cut-out ([`model::TextureUse::cutout`]) and how it splits.
    GltfTexture(&'a Source, usize, u32, Option<u8>, Split),
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
    /// A UI document and the ids of the documents defining the components
    /// it uses, which the runtime loads with it.
    Document(&'a Source, Vec<String>),
    Tileset(&'a Source),
    Tilemap(&'a Source, tilemap::MapSettings),
    Flipbook(&'a Source),
    /// An `array true` sheet's texture array blob, and its layer `n`.
    FlipbookArray(&'a Source, Box<og_anim::FlipbookSheet>),
    FlipbookLayer(&'a Source, Box<og_anim::FlipbookSheet>, usize),
    Rig2d(&'a Source),
    Rig2dAtlas(&'a Source),
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
    FontPack(&'a Source),
    Rig(&'a Source),
    MotionAction(&'a Source),
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
    /// Work its system's module plans and cooks (`pipeline/kinds.rs`).
    Job(Box<dyn Job + 'a>),
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

/// `assets/` of a source: its path less its id's components.
pub(crate) fn assets_root(s: &Source) -> PathBuf {
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
    let mut laps = Laps::new(started);
    let cook_index = index::Index::open(root, options, texture_cache);
    let unchanged = cook_index.unchanged();
    laps.lap("index");
    if let Some(mut report) = unchanged {
        report.elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
        report.stages = laps.stages;
        return Ok(report);
    }
    // Code alone changed: the assets are as cooked, and only what reads
    // code is done again.
    if let Some((mut report, before_code)) = cook_index.only_code_changed()
        && cook_code(root, &before_code)?
    {
        laps.lap("code");
        cook_index.save(&report, Some(before_code));
        laps.lap("save");
        report.elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
        report.stages = laps.stages;
        return Ok(report);
    }
    let project = Project::read(root)?;
    laps.lap("project");
    let assets_dir = &project.assets_dir;
    let cooked_dir = &project.cooked_dir;
    let threads = if options.threads == 0 {
        std::thread::available_parallelism().map_or(4, |n| n.get())
    } else {
        options.threads
    };
    let texture_encoder = texture::encoder_name(&options.texture);
    let Scanned {
        mut sources,
        mut ignored,
        mut early_errors,
        mut consumed,
        components,
    } = scan_sources(assets_dir, &cook_index, threads)?;
    laps.lap("scan");
    let imported = import(&mut sources, &mut consumed, texture_cache, threads);
    laps.lap("import");
    let Resolved {
        known,
        prefab_texts,
    } = resolve(assets_dir, &mut sources, &mut ignored, consumed);
    laps.lap("resolve");
    let Contracted {
        contract,
        conformed,
        options,
    } = contract(
        &project,
        &mut sources,
        &known,
        &imported.model_texts,
        options,
        &mut early_errors,
    )?;
    laps.lap("contract");
    let options = &options;
    let sources = sources;
    let Grouped {
        groups,
        group_of,
        pack_faces,
    } = group(&sources, &known, &mut early_errors, threads);
    laps.lap("group");
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
    let document_themes = vec![og_ui::Theme::default()];
    let mut style_tokens_hash = StableHasher::new();
    for source in sources.iter().filter(|s| s.kind == Kind::Style) {
        style_tokens_hash.write_str(&source.rel);
        style_tokens_hash.write(&source.hash.to_le_bytes());
    }
    let style_tokens_hash = style_tokens_hash.finish().to_le_bytes();
    let cx = Cx {
        root,
        assets_dir,
        cooked_dir,
        options,
        sources: &sources,
        known: &known,
        prefab_texts: &prefab_texts,
        group_of: &group_of,
        pack_faces: &pack_faces,
        old_outputs: &old_outputs,
        schemas: &schemas,
        budgets: &project.budgets,
        tex_desc: &tex_desc,
        mesh_desc: &mesh_desc,
        font_desc: &font_desc,
        budget_desc: &project.budget_desc,
        schema_hash,
        style_tokens_hash,
        story_functions: &project.story_functions,
        story_functions_stamp: &project.story_functions_stamp,
        document_themes: &document_themes,
        components: &components,
    };
    let (
        plan,
        PlanState {
            early_errors,
            parsed_refs,
            ..
        },
    ) = plan(&cx, early_errors, threads);
    laps.lap("plan");
    let project_key = project_key(&cook_index, &sources, texture_cache);
    let results = execute(
        &cx,
        &plan,
        threads,
        texture_cache,
        project_key.as_deref(),
        started,
    );
    laps.lap("execute");
    let Assembled {
        mut reports,
        new_outputs,
        log,
    } = assemble(
        &cx,
        &plan,
        results,
        early_errors,
        &imported.reused_fractures,
    )?;
    laps.lap("assemble");
    let mut written = write(&cx, new_outputs, &mut reports, &old)?;
    laps.lap("write");
    pack(&cx, &groups, &mut written, &mut reports)?;
    laps.lap("pack");
    let (removed, warnings, before_code) = finish(
        &cx,
        &old,
        written,
        &parsed_refs,
        &mut reports,
        &log,
        threads,
    )?;
    laps.lap("finish");
    let (conformed, warnings) = diagnose(
        &cx,
        &plan,
        &mut reports,
        &log,
        contract.as_ref(),
        conformed,
        warnings,
    );
    reports.sort_by(|a, b| a.source.cmp(&b.source));
    laps.lap("diagnose");
    let mut report = Report {
        conform: conformed,
        warnings,
        root: root.display().to_string(),
        cooker: COOKER_VERSION.to_string(),
        texture_encoder: texture_encoder.unwrap_or_else(|e| format!("none ({e})")),
        assets: reports,
        ignored,
        removed,
        elapsed_ms: started.elapsed().as_secs_f64() * 1000.0,
        stages: Vec::new(),
    };
    save(
        &cook_index,
        &report,
        before_code,
        &sources,
        assets_dir,
        texture_cache,
    );
    laps.lap("save");
    report.stages = laps.stages;
    Ok(report)
}

/// `f` of every item, on up to `threads` threads; the answers in the
/// items' order. For the cook's work that is one file's alone (reading it,
/// hashing it, parsing it), which a project has thousands of.
fn each<T: Sync, R: Send>(items: &[T], threads: usize, f: impl Fn(&T) -> R + Sync) -> Vec<R> {
    let threads = threads.min(items.len());
    if threads < 2 {
        return items.iter().map(f).collect();
    }
    let done: Vec<Mutex<Option<R>>> = items.iter().map(|_| Mutex::new(None)).collect();
    let next = std::sync::atomic::AtomicUsize::new(0);
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| {
                loop {
                    let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some(item) = items.get(i) else { break };
                    *done[i].lock().expect("no poisoning") = Some(f(item));
                }
            });
        }
    });
    done.into_iter()
        .map(|r| r.into_inner().expect("no poisoning").expect("every item"))
        .collect()
}

/// Times the cook's stages, each from the end of the one before.
struct Laps {
    last: Instant,
    stages: Vec<StageTime>,
}

impl Laps {
    fn new(started: Instant) -> Laps {
        Laps {
            last: started,
            stages: Vec::new(),
        }
    }

    fn lap(&mut self, name: &str) {
        let now = Instant::now();
        self.stages.push(StageTime {
            name: name.to_string(),
            ms: now.duration_since(self.last).as_secs_f64() * 1000.0,
        });
        self.last = now;
    }
}

type OutputBytes = std::sync::Arc<[u8]>;

/// A planned output's manifest entry, its new bytes (None: up to date) and report line.
type NewOutput = (ManifestOutput, Option<OutputBytes>, OutputReport);
