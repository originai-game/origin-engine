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
pub use pipeline::author::project_assets;
pub use pipeline::{asset_move, assets, cache, graph, stats, ts_refs};
use pipeline::{
    execute, import_defaults, index, kinds, options, output, plan, provided, report, scan, stage,
    stream,
};
mod systems;
pub use systems::shader_check::material_shader;
pub use systems::{animated, efk, look, probes, strings_extract, tilemap, vfx2d};
use systems::{
    animgraph, autorig, behaviour, district, effect, effect_texture, environment, field, flipbook,
    fracture, fracture_cache, hlod, lightmap, navmesh, rig2d, shader_check, story, story_functions,
    strings, table, terrain, vfx,
};
mod formats;
/// A source image decoded to RGBA8.
pub use formats::texture::{
    Image as DecodedImage, decode as decode_image, orientation as image_orientation,
};
pub use formats::{
    anim, audio, bc6h, cube_source, fbx, icons, ktx2_source, model_program, model_text, prefab,
    sfx, texture_array, timeline, video, vox,
};
use formats::{font, font_pack, model, motion, psd, scene, texture};
pub mod conform;
pub mod file_types;
pub mod flow;
pub mod generator;
pub mod materials;
pub mod source_content;
pub mod source_index;
pub mod source_metadata;
mod source_path;
pub mod source_project;
pub mod source_records;
pub mod source_schemas;
pub mod source_transaction;
pub mod source_types;
pub use conform::{ConformAction, ConformEntry};
pub mod riv;
mod schemas;
pub use schemas::engine_schemas;
#[cfg(test)]
#[path = "../../../../tests/temp.rs"]
mod test_dir;
pub mod ui_art;
mod ui_packs;
pub mod ui_themes;
pub mod vector;

use execute::run;
pub use graph::CodeGraph;
pub use options::{CookOptions, FontSettings, MeshOptions, TextureOptions};
use options::{
    anim_settings, conform_off, font_settings, mesh_for, mesh_texture, model_stream_textures,
    own_conform_off, settings_block, texture_kind, texture_stream,
};
pub use report::{AssetReport, COOK_TIMES, CookTimes, KindTotals, OutputReport, Report, StageTime};
use report::{CONFORM_LOG, ConformLog, conform_log_text, conform_note, read_conform_log};
pub use scan::own_lines;
use scan::{Scan, ext, output_group, plan_groups, scan};

use crate::format::{self, Area, BlobKind, COOKER};
use crate::runtime::{MANIFEST, Manifest, ManifestAsset, ManifestOutput};
use kinds::{Action, Exec, Job, KindDef};
use og_core::Schema;
use og_core::hash::{StableHasher, hash_bytes};

/// A cooked file's Subresource Integrity metadata, `sha384-<base64>`: the
/// manifest's `integrity`, which a loader checks a package from another
/// origin against (docs/web-export.md §Asset base).
pub fn integrity(bytes: &[u8]) -> String {
    let digest = ring::digest::digest(&ring::digest::SHA384, bytes);
    format!("sha384-{}", og_core::base64::encode(digest.as_ref()))
}
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

/// A source's kind: the declaration its system's module makes
/// (`pipeline/kinds.rs`).
type Kind = &'static KindDef;

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
    /// A model's cook (its blob, notes and LOD surfaces), made once by
    /// the first of its outputs that needs it.
    model: std::sync::OnceLock<Result<model::ModelOutput, CookError>>,
    /// One media snapshot and plan, from content discovery through execution.
    video: Option<video::Prepared>,
    /// Native tracks and captured captions, shared by planning and execution.
    timeline: Option<timeline::Prepared>,
    /// Checked reaction with references resolved to its producers' outputs.
    motion_action: Option<og_bundle::motion_action::MotionAction>,
    /// Compiled ICU messages and resolved locale font chain.
    strings: Option<og_i18n::StringTable>,
    /// Checked lattices shared with terrain and particle consumers for this cook.
    fields: std::sync::Arc<field::Fields>,
    /// Checked terrain surfaces shared by their writer, navigation and scatter.
    terrains: std::sync::Arc<terrain::Terrains>,
    /// Captured static geometry for the typed navigation bake.
    navmesh: Option<navmesh::Prepared>,
    /// The image the art contract conformed it to (`conform/`): a
    /// texture's own, a flipbook's texture's; read instead of the file,
    /// which is never written. `None`: as on disk.
    conformed: Option<std::sync::Arc<Vec<u8>>>,
    /// A model with a `fracture` block: its fracture asset
    /// (`fracture.rs`), cooked as `<model>#fracture`.
    fracture: Option<og_physics_data::fracture::FractureAsset>,
}

impl Source {
    /// Decode one already schema-checked TS declaration for its domain job.
    /// This reads canonical author IR, never an intermediate author text format.
    fn input<T: serde::de::DeserializeOwned>(&self) -> Result<T, String> {
        let data = generator::value::Data::from_bytes(self.bytes.clone())?;
        let file: source_schemas::File =
            serde_json::from_value(data.root().into_json()).map_err(|e| e.to_string())?;
        serde_json::from_value(file.value).map_err(|e| e.to_string())
    }

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
                let pcm = if self.path.extension().is_some_and(|ext| ext == "ts") {
                    self.input::<audio::PcmInput>()?.into_pcm()
                } else {
                    audio::decode(&self.rel, &self.bytes)
                };
                pcm.and_then(|pcm| match settings.codec {
                    audio::Codec::Opus => audio::encode_opus_chunked(
                        &pcm,
                        settings.opus_bitrate(pcm.channels),
                        per_chunk,
                    ),
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

struct Planned<'a> {
    source: usize,
    id: String,
    file: String,
    ty: &'static str,
    stamp: u64,
    work: Option<Box<dyn Job + 'a>>,
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

/// An output's cook stamp: the revisions of the `areas` whose code wrote
/// it ([`format::revisions`]), then its inputs.
fn stamp(areas: &[Area], parts: &[&[u8]]) -> u64 {
    let mut h = StableHasher::new();
    h.write_u64(format::revisions(areas));
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

/// A source's owning root. `@styles/` names the shipped styles root, not a
/// physical directory within it; other ids are relative to their module root.
pub(crate) fn assets_root(s: &Source) -> PathBuf {
    let mut root = s.path.clone();
    for _ in s.rel.strip_prefix("@styles/").unwrap_or(&s.rel).split('/') {
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
    let mut laps = Laps::new(started, options.progress);
    let cook_index = index::Index::open(root, options);
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
        && cook_code(root, &before_code, &options.code)?
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
        warnings: scan_warnings,
    } = scan_sources(&project, &cook_index, threads, &options.code)?;
    laps.lap("scan");
    let mut imported = import(&mut sources, &mut consumed, texture_cache, threads);
    laps.lap("import");
    let before_author = sources.len();
    let content = pipeline::author::sources(
        root,
        &project.source,
        &mut sources,
        &mut consumed,
        &project.provided,
    )?;
    // Districts expanded after TS terrain discovery can add model sources.
    // Import only those new sources; never re-import the captured originals.
    let added = import(
        &mut sources[before_author..],
        &mut consumed,
        texture_cache,
        threads,
    );
    imported.model_texts.extend(added.model_texts);
    imported.reused_fractures.extend(added.reused_fractures);
    // Projected scatter needs both resolved mesh operands and the checked
    // TS field snapshot, just like the terrain writer and navmesh baker.
    scene::expand(&mut sources);
    for s in sources.iter_mut().filter(|s| s.kind == &scene::SCENE) {
        s.hash = stage::source_hash(s);
    }
    navmesh::bind(&mut sources, &content)?;
    let boot = project
        .boot
        .as_ref()
        .map(|id| content.outputs[id].asset.clone());
    let flow = project
        .flow
        .as_ref()
        .map(|id| content.outputs[id].asset.clone());
    laps.lap("author");
    let Resolved {
        mut known,
        prefab_texts,
    } = resolve(assets_dir, &mut sources, &mut ignored, consumed);
    known.extend(content.outputs.values().map(|output| output.asset.clone()));
    laps.lap("resolve");
    let Contracted {
        contract,
        conformed,
        options,
    } = contract(
        &project,
        &content,
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
    let project_key = project_key(&cook_index, &sources, texture_cache);
    let cx = Cx {
        index: &cook_index,
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
        story_functions: &project.story_functions,
        story_functions_stamp: &project.story_functions_stamp,
        profile: project.profile,
        boot: boot.as_deref(),
        flow: flow.as_deref(),
        provided: &project.provided,
        texture_cache,
        project_key: project_key.as_deref(),
    };
    let (
        plan,
        PlanState {
            early_errors,
            mut parsed_refs,
            ..
        },
    ) = plan(&cx, early_errors, threads);
    for (id, node) in &content.nodes {
        parsed_refs.entry(id.clone()).or_default().extend(
            node.references
                .iter()
                .map(|edge| content.outputs[&edge.target].asset.clone()),
        );
    }
    laps.lap("plan");
    let results = execute(
        &cx,
        &plan,
        threads,
        texture_cache,
        project_key.as_deref(),
        started,
    );
    laps.lap("execute");
    // What only a cook could name (a model's LOD surfaces), in a round of
    // its own.
    let derived = derive(&cx, &plan, &mut parsed_refs, threads);
    let derived_results = execute(
        &cx,
        &derived,
        threads,
        texture_cache,
        project_key.as_deref(),
        started,
    );
    let (plan, results) = with_derived(plan, results, derived, derived_results);
    laps.lap("derived");
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
    let mut warnings = warnings;
    warnings.extend(diagnose_models(&cx, &mut reports, threads));
    let (conformed, mut warnings) = diagnose(
        &cx,
        &plan,
        &mut reports,
        &log,
        contract.as_ref(),
        conformed,
        warnings,
    );
    warnings.extend(scan_warnings);
    reports.sort_by(|a, b| a.source.cmp(&b.source));
    laps.lap("diagnose");
    let mut report = Report {
        conform: conformed,
        warnings,
        root: root.display().to_string(),
        cooker: COOKER.to_string(),
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
    /// `CookOptions::progress`: a slow stage says so on stderr.
    progress: bool,
}

impl Laps {
    fn new(started: Instant, progress: bool) -> Laps {
        Laps {
            last: started,
            stages: Vec::new(),
            progress,
        }
    }

    fn lap(&mut self, name: &str) {
        let now = Instant::now();
        let took = now.duration_since(self.last);
        if self.progress && took >= output::SLOW {
            eprintln!("og cook: {name} took {:.1} s", took.as_secs_f64());
        }
        self.stages.push(StageTime {
            name: name.to_string(),
            ms: took.as_secs_f64() * 1000.0,
        });
        self.last = now;
    }
}

type OutputBytes = std::sync::Arc<[u8]>;

/// A planned output's manifest entry, its new bytes (None: up to date) and report line.
type NewOutput = (ManifestOutput, Option<OutputBytes>, OutputReport);
