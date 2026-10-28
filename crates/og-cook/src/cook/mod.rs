//! The cook: `assets/` → `cooked/`, incremental by hash.
//!
//! [`cook_project`] walks `<root>/assets`, classifies each file by extension
//! (`.glb`/`.gltf` models, `.png`/`.jpg` textures, `.wav` sounds,
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
mod conform;
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
mod model;
pub mod model_text;
pub mod probes;
mod rig2d;
mod scene;
pub mod sfx;
pub mod stats;
mod stream;
mod terrain;
mod terrain_generate;
mod texture;
pub mod texture_array;
mod texture_cache;
pub mod tilemap;
mod timeline;
pub mod video;
pub mod vox;

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

/// Mesh processing (meshoptimizer).
#[derive(Clone, Debug, PartialEq)]
pub struct MeshOptions {
    /// LOD levels including the full mesh (LOD 0).
    pub lod_levels: u32,
    /// Each LOD targets this fraction of the previous one's triangles.
    pub lod_ratio: f32,
    /// The largest simplification error allowed, relative to the mesh extent.
    pub lod_max_error: f32,
    /// How much vertex-cache efficiency overdraw ordering may give up (1.05 = 5 %).
    pub overdraw_threshold: f32,
    /// Generate MikkTSpace tangents for normal-mapped primitives that lack them.
    pub tangents: bool,
    /// Each level's fraction of the full mesh's triangles, the first 1: a
    /// model's import settings (`model { lods 1 0.5 0.25 }`). `None`: the
    /// levels `lod_levels` and `lod_ratio` give (1, 0.5, 0.25 by default).
    pub lods: Option<Vec<f32>>,
    /// Cook each primitive's raycast BVH (og.render.raycast) into the
    /// model's blob, so a raycast never builds one at run time: a model's
    /// import settings turn it off (`model { raycast false }`) for a model
    /// no ray is cast at, saving its bytes (about 13 per triangle).
    pub raycast: bool,
}

impl Default for MeshOptions {
    fn default() -> Self {
        MeshOptions {
            lod_levels: 3,
            lod_ratio: 0.5,
            lod_max_error: 0.05,
            overdraw_threshold: 1.05,
            tangents: true,
            lods: None,
            raycast: true,
        }
    }
}

impl MeshOptions {
    /// Each level's fraction of the full mesh's triangles, level 0 first (1).
    pub fn fractions(&self) -> Vec<f32> {
        match &self.lods {
            Some(l) => l.clone(),
            None => (0..self.lod_levels.max(1))
                .map(|k| self.lod_ratio.powi(k as i32))
                .collect(),
        }
    }

    /// These options with a model's import settings (`model { lods 1 0.5
    /// 0.25 lodError 0.05 }`) over them: `lods` the levels' triangle
    /// fractions (the first 1, each below the one before; `lods 1` keeps the
    /// full mesh alone), `lodError` the largest simplification error
    /// relative to the mesh's extent.
    pub fn for_model(&self, block: Option<&og_core::ogt::Block>) -> Result<MeshOptions, String> {
        let mut o = self.clone();
        let Some(b) = block else {
            return Ok(o);
        };
        if let Some(e) = b.get("lods") {
            let mut f = Vec::new();
            for i in 0..e.values.len() {
                f.push(
                    e.f32(i)
                        .map_err(|x| format!("line {}: lods: {x}", e.line))?,
                );
            }
            let ok = f.first() == Some(&1.0)
                && f.windows(2).all(|w| w[1] < w[0] && w[1] > 0.0)
                && f.len() <= 8;
            if !ok {
                return Err(format!(
                    "line {}: lods takes each level's fraction of the full mesh's triangles, the first 1 and each below the one before, at most 8 (`lods 1 0.5 0.25`)",
                    e.line
                ));
            }
            o.lods = Some(f);
        }
        if let Some(e) = b.get("lodError") {
            let v = e
                .f32(0)
                .map_err(|x| format!("line {}: lodError: {x}", e.line))?;
            if !(v > 0.0 && v <= 1.0) {
                return Err(format!(
                    "line {}: lodError is the largest error relative to the mesh's extent, above 0 and at most 1 (default 0.05)",
                    e.line
                ));
            }
            o.lod_max_error = v;
        }
        if let Some(e) = b.get("raycast") {
            o.raycast = e.values.first().and_then(|v| v.as_bool()).ok_or_else(|| {
                format!(
                    "line {}: raycast takes true (cook the raycast BVH, the default) or false (build it at the first og.render.raycast that reaches the model)",
                    e.line
                )
            })?;
        }
        Ok(o)
    }

    fn describe(&self) -> String {
        format!(
            "lods {} ratio {} error {} overdraw {} tangents {} (bitangent up the image) quantize 16/oct16/uv16/rgba8 codec meshopt-v1 flat levels by position (S6)",
            self.lod_levels,
            self.lod_ratio,
            self.lod_max_error,
            self.overdraw_threshold,
            self.tangents
        )
    }
}

/// Texture compression (Basis Universal).
#[derive(Clone, Debug, PartialEq)]
pub struct TextureOptions {
    /// ETC1S quality 1..=255 (colour).
    pub etc1s_quality: u32,
    /// UASTC pack level 0..=4 (normals, linear data).
    pub uastc_level: u32,
    /// UASTC rate-distortion optimisation quality scalar (lower = higher quality), or none.
    pub uastc_rdo: Option<f32>,
    /// Zstandard supercompression of UASTC levels.
    pub zstd: bool,
    pub mipmaps: bool,
    /// Encode with `toktx` even when the basis-universal crate is built in.
    pub force_toktx: bool,
    /// Encode colour textures as UASTC too (about 4x the size of ETC1S, without its 4x4 block colour loss on foliage,
    /// bark and stone detail). Off by default; project.ogt's `cook { color_textures uastc }` turns it on and
    /// `publish { color_textures etc1s }` off again for the web export (og-cli's `cook::texture_options`).
    pub color_uastc: bool,
}

impl Default for TextureOptions {
    fn default() -> Self {
        TextureOptions {
            etc1s_quality: 128,
            uastc_level: 1,
            uastc_rdo: Some(1.0),
            zstd: true,
            mipmaps: true,
            force_toktx: false,
            color_uastc: false,
        }
    }
}

impl TextureOptions {
    fn describe(&self) -> String {
        format!(
            "etc1s {} uastc {} rdo {} zstd {} mips {} normal y in alpha{}",
            self.etc1s_quality,
            self.uastc_level,
            self.uastc_rdo.map_or("off".to_string(), |r| r.to_string()),
            self.zstd,
            self.mipmaps,
            if self.color_uastc {
                " colour uastc"
            } else {
                ""
            }
        )
    }
}

/// Font atlas settings; a font's import settings (`<font>.ogt`) override them.
#[derive(Clone, Debug, PartialEq)]
pub struct FontSettings {
    /// Named code point sets: `ascii`, `latin` (ASCII + Latin-1), `punct`,
    /// `cjk-punct`, `kana`, `all` (every code point the font maps).
    pub charset: Vec<String>,
    /// Extra characters to include (UI strings, a game's CJK text).
    pub chars: String,
    /// Atlas pixels per em.
    pub em_px: f32,
    /// Distance range in pixels.
    pub px_range: f32,
    /// Atlas page edge in pixels.
    pub page_size: u32,
    /// Ship the font file with the atlas, so characters outside the charset
    /// are rendered from its outlines when a text needs them (CJK UIs that
    /// show player text).
    pub runtime: bool,
}

impl Default for FontSettings {
    fn default() -> Self {
        FontSettings {
            charset: vec!["latin".into()],
            chars: String::new(),
            em_px: 32.0,
            px_range: 4.0,
            page_size: 1024,
            runtime: false,
        }
    }
}

impl FontSettings {
    fn describe(&self) -> String {
        format!(
            "charset {} chars {:?} em {} range {} page {} mtsdf ec kern{}",
            self.charset.join(" "),
            self.chars,
            self.em_px,
            self.px_range,
            self.page_size,
            if self.runtime { " runtime" } else { "" }
        )
    }
}

/// Everything the cook's output depends on besides the sources.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CookOptions {
    /// Worker threads (0 = the machine's parallelism). Does not change output.
    pub threads: usize,
    pub mesh: MeshOptions,
    pub texture: TextureOptions,
    pub font: FontSettings,
    /// Reflected component schemas scene tables are packed with (the engine's
    /// and the game's, as `og schema` exports them). Components without one
    /// get a layout inferred from the scene text.
    pub schemas: Vec<Schema>,
}

/// One output in the report.
#[derive(Clone, Debug, PartialEq)]
pub struct OutputReport {
    pub id: String,
    pub file: String,
    pub ty: String,
    pub bytes: u64,
    /// Cooked in this run (false: the manifest's output was up to date).
    pub cooked: bool,
    /// The file on disk changed.
    pub written: bool,
    pub ms: f64,
    /// When its cook began, in milliseconds after the cook's start, and
    /// the cook thread that ran it (0 to the thread count less one): the
    /// trace lays each output where it ran (`og run --trace`). 0 and 0 for
    /// an output that did not cook.
    pub start_ms: f64,
    pub worker: u32,
}

/// One source asset in the report.
#[derive(Clone, Debug, PartialEq)]
pub struct AssetReport {
    pub source: String,
    pub kind: String,
    pub source_bytes: u64,
    pub outputs: Vec<OutputReport>,
    pub notes: Vec<String>,
    pub error: Option<String>,
    /// What its last cook took, in milliseconds (its outputs' times summed):
    /// this run's when any output cooked, else the time recorded in
    /// `cooked/.cook-times.ogt` ([`CookTimes`]; 0 when none is).
    pub cook_ms: f64,
}

/// `cooked/.cook-times.ogt`: beside the manifest, the time each asset's
/// last cook took. The manifest itself holds no times, so the same sources
/// still cook to byte-identical `cooked/` (§Determinism); this file is the
/// one that differs, is written only when an asset cooked, and is not
/// shipped (og build leaves it out).
pub const COOK_TIMES: &str = ".cook-times.ogt";

/// [`COOK_TIMES`]: per source path, its last cook's milliseconds.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CookTimes {
    pub assets: BTreeMap<String, f64>,
}

impl CookTimes {
    /// Reads `cooked/.cook-times.ogt` (empty when missing or unreadable).
    pub fn read(cooked: &Path) -> CookTimes {
        std::fs::read_to_string(cooked.join(COOK_TIMES))
            .ok()
            .and_then(|t| CookTimes::parse(&t).ok())
            .unwrap_or_default()
    }

    /// Parses the text: `asset "<source>" <ms>` lines.
    pub fn parse(text: &str) -> Result<CookTimes, String> {
        let doc = Document::parse(text).map_err(|e| e.to_string())?;
        if doc.ty != "cook-times" || doc.version != 1 {
            return Err(format!(
                "expected a 'cook-times' version 1 document, got '{}' version {}",
                doc.ty, doc.version
            ));
        }
        let mut assets = BTreeMap::new();
        for e in doc.body.get_all("asset") {
            let source = e.string(0).map_err(|x| format!("line {}: {x}", e.line))?;
            let ms = e.f32(1).map_err(|x| format!("line {}: {x}", e.line))?;
            assets.insert(source.to_string(), f64::from(ms));
        }
        Ok(CookTimes { assets })
    }

    pub fn to_text(&self) -> String {
        use og_core::ogt::{Entry, Number, Value};
        let mut doc = Document::new("cook-times", 1);
        for (source, ms) in &self.assets {
            // To the microsecond.
            let ms = ((ms * 1000.0).round() / 1000.0) as f32;
            doc.body.push(
                Entry::new("asset")
                    .with(Value::String(source.clone()))
                    .with(Value::Number(
                        Number::from_f32(ms).unwrap_or_else(|_| Number::from(0u32)),
                    )),
            );
        }
        doc.to_text().expect("cook times are strings and numbers")
    }

    /// The `n` slowest assets, slowest first (then by path).
    pub fn slowest(&self, n: usize) -> Vec<(&str, f64)> {
        let mut v: Vec<(&str, f64)> = self.assets.iter().map(|(k, v)| (k.as_str(), *v)).collect();
        v.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(b.0)));
        v.truncate(n);
        v
    }
}

/// What a cook did.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Report {
    pub root: String,
    pub cooker: String,
    pub texture_encoder: String,
    pub assets: Vec<AssetReport>,
    /// Files under `assets/` that are not cooked (licences, notes).
    pub ignored: Vec<String>,
    /// Outputs deleted because their source is gone or failed.
    pub removed: Vec<String>,
    /// Problems that do not stop the cook: a locale's missing string keys,
    /// a Tiled map's custom property the importer does not read.
    pub warnings: Vec<String>,
    /// What the art contract changed in each asset it conformed, and every
    /// asset that opted out (docs/art.md §2.3), by asset id.
    pub conform: Vec<conform::ConformEntry>,
    pub elapsed_ms: f64,
}

/// Bytes in and out per asset kind.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct KindTotals {
    pub assets: usize,
    pub source_bytes: u64,
    pub cooked_bytes: u64,
}

impl Report {
    /// No asset failed.
    pub fn ok(&self) -> bool {
        self.assets.iter().all(|a| a.error.is_none())
    }

    pub fn errors(&self) -> Vec<String> {
        self.assets
            .iter()
            .filter_map(|a| {
                a.error
                    .as_ref()
                    .map(|e| format!("assets/{}: {e}", a.source))
            })
            .collect()
    }

    pub fn source_bytes(&self) -> u64 {
        self.assets.iter().map(|a| a.source_bytes).sum()
    }

    pub fn cooked_bytes(&self) -> u64 {
        self.assets
            .iter()
            .flat_map(|a| &a.outputs)
            .map(|o| o.bytes)
            .sum()
    }

    /// Outputs cooked in this run.
    pub fn cooked_outputs(&self) -> Vec<&OutputReport> {
        self.assets
            .iter()
            .flat_map(|a| &a.outputs)
            .filter(|o| o.cooked)
            .collect()
    }

    pub fn by_kind(&self) -> BTreeMap<String, KindTotals> {
        let mut m: BTreeMap<String, KindTotals> = BTreeMap::new();
        for a in &self.assets {
            let t = m.entry(a.kind.clone()).or_default();
            t.assets += 1;
            t.source_bytes += a.source_bytes;
            t.cooked_bytes += a.outputs.iter().map(|o| o.bytes).sum::<u64>();
        }
        m
    }

    /// The `n` assets whose last cook took longest ([`AssetReport::cook_ms`],
    /// this run's or the recorded one), slowest first.
    pub fn slowest(&self, n: usize) -> Vec<&AssetReport> {
        let mut v: Vec<&AssetReport> = self
            .assets
            .iter()
            .filter(|a| a.error.is_none() && !a.outputs.is_empty())
            .collect();
        v.sort_by(|a, b| {
            b.cook_ms
                .total_cmp(&a.cook_ms)
                .then(a.source.cmp(&b.source))
        });
        v.truncate(n);
        v
    }

    /// The assets cooked in this run with their times, slowest first.
    pub fn cooked_assets(&self) -> Vec<&AssetReport> {
        let mut v: Vec<&AssetReport> = self
            .assets
            .iter()
            .filter(|a| a.outputs.iter().any(|o| o.cooked))
            .collect();
        v.sort_by(|a, b| {
            b.cook_ms
                .total_cmp(&a.cook_ms)
                .then(a.source.cmp(&b.source))
        });
        v
    }

    /// The slowest ten as a Markdown table (`og cook --report`).
    pub fn slowest_markdown(&self) -> String {
        let mut s = String::from(
            "| # | asset | kind | last cook ms | this run |\n|---:|---|---|---:|---|\n",
        );
        for (i, a) in self.slowest(10).iter().enumerate() {
            let now = a.outputs.iter().any(|o| o.cooked);
            s.push_str(&format!(
                "| {} | {} | {} | {:.3} | {} |\n",
                i + 1,
                a.source,
                a.kind,
                a.cook_ms,
                if now { "cooked" } else { "up to date" }
            ));
        }
        s
    }

    /// A Markdown summary: per class and per asset sizes, what was cooked.
    pub fn to_markdown(&self) -> String {
        let pct = |a: u64, b: u64| {
            if b == 0 {
                "-".to_string()
            } else {
                format!("{:.1} %", a as f64 * 100.0 / b as f64)
            }
        };
        let mut s = format!(
            "# Cook report\n\n`{}` with {}; textures by {}. {} asset(s), {} output(s) cooked this run, {:.0} ms.\n\n",
            self.root,
            self.cooker,
            self.texture_encoder,
            self.assets.len(),
            self.cooked_outputs().len(),
            self.elapsed_ms
        );
        s.push_str("| class | assets | source bytes | cooked bytes | cooked / source |\n|---|---:|---:|---:|---:|\n");
        for (k, t) in self.by_kind() {
            s.push_str(&format!(
                "| {k} | {} | {} | {} | {} |\n",
                t.assets,
                t.source_bytes,
                t.cooked_bytes,
                pct(t.cooked_bytes, t.source_bytes)
            ));
        }
        s.push_str(&format!(
            "| **total** | {} | {} | {} | **{}** |\n\n",
            self.assets.len(),
            self.source_bytes(),
            self.cooked_bytes(),
            pct(self.cooked_bytes(), self.source_bytes())
        ));
        s.push_str(
            "## The slowest ten\n\nEach asset's last cook (`cooked/.cook-times.ogt`), its outputs' times summed.\n\n",
        );
        s.push_str(&self.slowest_markdown());
        s.push('\n');
        s.push_str("| asset | output | type | bytes | cooked this run | ms |\n|---|---|---|---:|---|---:|\n");
        for a in &self.assets {
            for o in &a.outputs {
                s.push_str(&format!(
                    "| {} | {} | {} | {} | {} | {:.0} |\n",
                    a.source,
                    o.file,
                    o.ty,
                    o.bytes,
                    if o.cooked { "yes" } else { "up to date" },
                    o.ms
                ));
            }
        }
        s.push('\n');
        for a in &self.assets {
            for n in &a.notes {
                s.push_str(&format!("- `{}`: {n}\n", a.source));
            }
            if let Some(e) = &a.error {
                s.push_str(&format!("- `{}`: **error**: {e}\n", a.source));
            }
        }
        for i in &self.ignored {
            s.push_str(&format!("- `{i}`: not an asset, ignored\n"));
        }
        if !self.conform.is_empty() {
            s.push_str("\n## The art contract\n\nWhat conforming to the project's look changed, and what opted out (docs/art.md §2.3).\n\n| asset | | what |\n|---|---|---|\n");
            for e in &self.conform {
                s.push_str(&format!(
                    "| {} | {} | {} |\n",
                    e.source,
                    e.action.name(),
                    e.what
                ));
            }
        }
        for r in &self.removed {
            s.push_str(&format!("- `cooked/{r}`: removed (stale)\n"));
        }
        s
    }

    /// The report as JSON.
    pub fn to_json(&self) -> String {
        use serde_json::{Value, json};
        let assets: Vec<Value> = self
            .assets
            .iter()
            .map(|a| {
                json!({
                    "source": a.source, "kind": a.kind, "source_bytes": a.source_bytes,
                    "error": a.error, "notes": a.notes, "cook_ms": a.cook_ms,
                    "outputs": a.outputs.iter().map(|o| json!({
                        "id": o.id, "file": o.file, "type": o.ty, "bytes": o.bytes,
                        "cooked": o.cooked, "written": o.written, "ms": o.ms
                    })).collect::<Vec<_>>(),
                })
            })
            .collect();
        let kinds: serde_json::Map<String, Value> = self
            .by_kind()
            .into_iter()
            .map(|(k, t)| {
                (k, json!({"assets": t.assets, "source_bytes": t.source_bytes, "cooked_bytes": t.cooked_bytes}))
            })
            .collect();
        serde_json::to_string_pretty(&json!({
            "root": self.root, "cooker": self.cooker, "texture_encoder": self.texture_encoder,
            "elapsed_ms": self.elapsed_ms, "source_bytes": self.source_bytes(), "cooked_bytes": self.cooked_bytes(),
            "by_kind": kinds, "assets": assets, "ignored": self.ignored, "removed": self.removed,
            "warnings": self.warnings,
            "conform": self.conform.iter().map(|e| json!({
                "source": e.source, "action": e.action.name(), "what": e.what
            })).collect::<Vec<_>>(),
        }))
        .expect("the report serializes")
    }
}

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
    Tileset,
    Tilemap,
    Flipbook,
    Rig2d,
    Shader,
    Post,
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
            Kind::Tileset => "tileset",
            Kind::Tilemap => "tilemap",
            Kind::Flipbook => "flipbook",
            Kind::Rig2d => "rig2d",
            Kind::Shader => "shader",
            Kind::Post => "post",
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
    /// A WAV's QOA encoding, made once by the first of its outputs to cook
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
            .get_or_init(|| audio::encode(&self.bytes, per_chunk).map(std::sync::Arc::new))
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
    Tileset(&'a Source),
    Tilemap(&'a Source, tilemap::MapSettings),
    Flipbook(&'a Source),
    /// An `array true` sheet's texture array blob, and its layer `n`.
    FlipbookArray(&'a Source, Box<og_anim::FlipbookSheet>),
    FlipbookLayer(&'a Source, Box<og_anim::FlipbookSheet>, usize),
    Rig2d(&'a Source),
    Shader(&'a Source),
    Post(&'a Source),
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

fn walk(dir: &Path, rel: &str, out: &mut Vec<(String, PathBuf)>) -> std::io::Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)?.collect::<Result<_, _>>()?;
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let name = e.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let r = if rel.is_empty() {
            name.clone()
        } else {
            format!("{rel}/{name}")
        };
        if e.file_type()?.is_dir() {
            walk(&e.path(), &r, out)?;
        } else {
            out.push((r, e.path()));
        }
    }
    Ok(())
}

fn ext(rel: &str) -> String {
    rel.rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .unwrap_or_default()
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

fn settings_block<'a>(s: &'a Source, key: &str) -> Option<&'a og_core::ogt::Block> {
    s.settings.as_ref()?.1.body.get(key)?.block.as_ref()
}

fn texture_kind(s: &Source) -> Result<u32, String> {
    match settings_block(s, "texture").and_then(|b| b.get("kind")) {
        None => Ok(format::kind::COLOR),
        Some(e) => match e.word(0).map_err(|x| x.to_string())? {
            "color" => Ok(format::kind::COLOR),
            "normal" => Ok(format::kind::NORMAL),
            "linear" => Ok(format::kind::LINEAR),
            "pixel" => Ok(format::kind::PIXEL),
            w => Err(format!(
                "line {}: texture kind '{w}' (expected color, normal, linear or pixel)",
                e.line
            )),
        },
    }
}

fn anim_settings(s: &Source) -> Result<anim::AnimSettings, String> {
    anim::AnimSettings::parse(settings_block(s, "animation"))
}

/// `assets/`, from a source's path and its path under it.
fn assets_root(s: &Source) -> PathBuf {
    let mut root = s.path.clone();
    for _ in s.rel.split('/') {
        root.pop();
    }
    root
}

fn font_settings(s: &Source, base: &FontSettings) -> Result<FontSettings, String> {
    let mut f = base.clone();
    if let Some(b) = settings_block(s, "font") {
        for e in &b.entries {
            let err = |x: og_core::Error| format!("line {}: {x}", e.line);
            match e.key.as_str() {
                "charset" => {
                    f.charset = e
                        .values
                        .iter()
                        .map(|v| {
                            v.as_word()
                                .map(str::to_string)
                                .ok_or_else(|| format!("line {}: charset takes words", e.line))
                        })
                        .collect::<Result<_, _>>()?
                }
                "chars" => f.chars = e.string(0).map_err(err)?.to_string(),
                "em_px" => f.em_px = e.f32(0).map_err(err)?,
                "px_range" => f.px_range = e.f32(0).map_err(err)?,
                "runtime" => {
                    f.runtime =
                        e.values.first().and_then(|v| v.as_bool()).ok_or_else(|| {
                            format!("line {}: runtime takes true or false", e.line)
                        })?
                }
                "page_size" => {
                    f.page_size = e.number(0).map_err(err)?.as_u64().map_err(err)? as u32
                }
                k => return Err(format!("line {}: unknown font setting '{k}'", e.line)),
            }
        }
    }
    if !(4.0..=256.0).contains(&f.em_px) || !(1.0..=32.0).contains(&f.px_range) {
        return Err("font em_px must be 4..=256 and px_range 1..=32".into());
    }
    Ok(f)
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
    let mut files = Vec::new();
    walk(&assets_dir, "", &mut files)
        .map_err(|e| CookError(format!("{}: {e}", assets_dir.display())))?;
    let threads = if options.threads == 0 {
        std::thread::available_parallelism().map_or(4, |n| n.get())
    } else {
        options.threads
    };
    let texture_encoder = texture::encoder_name(&options.texture);
    // The project's budgets (project.ogt's `budgets { }`, the defaults
    // without one): a `type model` past them is refused, not cooked.
    let budgets = std::fs::read_to_string(root.join("project.ogt"))
        .ok()
        .and_then(|t| Document::parse(&t).ok())
        .and_then(|d| stats::Budgets::from_project(&d.body).ok())
        .unwrap_or_default();
    let budget_desc = format!(
        "budgets triangles {} vertices {}",
        budgets.triangles, budgets.vertices
    );

    // The engine's built-in assets the looks name join the bundle under
    // their reserved ids (`@canvas/paper.png`, every file of
    // `@styles/pixel/`), once each, read from the cook itself.
    let mut builtins: BTreeMap<String, &'static [u8]> = BTreeMap::new();
    for (rel, path) in &files {
        if ext(rel) == "ogt"
            && let Ok(text) = std::fs::read_to_string(path)
            && Document::parse(&text).is_ok_and(|d| d.ty == crate::look::LOOK_TYPE)
        {
            for (id, bytes) in look::builtin_files(rel, &text, &assets_dir) {
                builtins.insert(id.to_string(), bytes);
            }
        }
    }
    for id in builtins.keys() {
        files.push((id.clone(), assets_dir.join(id)));
    }
    let read = |rel: &str, path: &Path| -> Result<Vec<u8>, CookError> {
        match builtins.get(rel) {
            Some(b) => Ok(b.to_vec()),
            None => std::fs::read(path).map_err(|e| CookError(format!("{}: {e}", path.display()))),
        }
    };

    // Classify; import settings attach to the file they are named after.
    let names: BTreeSet<&str> = files.iter().map(|(r, _)| r.as_str()).collect();
    let mut sources: Vec<Source> = Vec::new();
    let mut settings: BTreeMap<String, (String, Document)> = BTreeMap::new();
    let mut ignored = Vec::new();
    let mut early_errors: Vec<(String, &'static str, String)> = Vec::new();
    for (rel, path) in &files {
        if rel.starts_with('@') && !builtins.contains_key(rel) {
            early_errors.push((
                rel.clone(),
                "asset",
                "paths starting with '@' are the engine's built-in assets (a look names them: \"@canvas/<name>\", \"@palettes/<name>\", a built-in style by its slug); rename the file or its folder".into(),
            ));
            continue;
        }
        let kind = match ext(rel).as_str() {
            "glb" | "gltf" | "vox" => Kind::Model,
            "png" | "jpg" | "jpeg" | "ktx2" => Kind::Texture,
            "wav" => Kind::Sound,
            "ttf" | "otf" => Kind::Font,
            "cube" => Kind::Lut,
            "tmj" | "ldtk" => Kind::Tilemap,
            "hex" | "gpl" => Kind::Palette,
            "ogt" => {
                let text = String::from_utf8(read(rel, path)?)
                    .map_err(|_| CookError(format!("{}: not UTF-8 text", path.display())))?;
                match Document::parse(&text) {
                    Ok(doc) if doc.ty == "import" => {
                        let target = rel.trim_end_matches(".ogt");
                        if names.contains(target) {
                            settings.insert(target.to_string(), (text, doc));
                        } else {
                            early_errors.push((
                                rel.clone(),
                                "import",
                                format!("import settings for '{target}', which does not exist"),
                            ));
                        }
                        continue;
                    }
                    Ok(doc) if doc.ty == "scene" => Kind::Scene,
                    Ok(doc) if doc.ty == crate::fx::EMITTER_TYPE => Kind::Emitter,
                    Ok(doc) if doc.ty == og_ui::style_file::STYLE_TYPE => Kind::Style,
                    Ok(doc) if doc.ty == "tileset" => Kind::Tileset,
                    Ok(doc) if doc.ty == "tilemap" => Kind::Tilemap,
                    Ok(doc) if doc.ty == "flipbook" => Kind::Flipbook,
                    Ok(doc) if doc.ty == og_anim::RIG2D_TYPE => Kind::Rig2d,
                    Ok(doc) if doc.ty == crate::shader::SHADER_TYPE => Kind::Shader,
                    Ok(doc) if doc.ty == crate::shader::POST_TYPE => Kind::Post,
                    Ok(doc) if doc.ty == sfx::SFX_TYPE => Kind::Sound,
                    Ok(doc) if doc.ty == e6::BEHAVIOUR_TYPE => Kind::Behaviour,
                    Ok(doc) if doc.ty == og_i18n::STRINGS_TYPE => Kind::Strings,
                    Ok(doc) if doc.ty == e6::NAVMESH_TYPE => Kind::NavMesh,
                    Ok(doc) if doc.ty == anim::RIG_TYPE => Kind::Rig,
                    Ok(doc) if doc.ty == terrain::TERRAIN_TYPE => Kind::Terrain,
                    Ok(doc) if doc.ty == timeline::TIMELINE_TYPE => Kind::Timeline,
                    Ok(doc) if doc.ty == lightmap::LIGHTMAP_TYPE => Kind::Lightmap,
                    Ok(doc) if doc.ty == environment::ENVIRONMENT_TYPE => Kind::Environment,
                    Ok(doc) if doc.ty == icons::ICONS_TYPE => Kind::Icons,
                    Ok(doc) if doc.ty == crate::look::LOOK_TYPE => Kind::Look,
                    Ok(doc) if doc.ty == texture_array::ARRAY_TYPE => Kind::TextureArray,
                    Ok(doc) if doc.ty == video::VIDEO_TYPE => Kind::Video,
                    Ok(doc) if doc.ty == model_text::MODEL_TYPE => Kind::Model,
                    Ok(doc) => {
                        early_errors.push((
                            rel.clone(),
                            "scene",
                            format!(
                                "an .ogt of type '{}' is not a cookable asset (scene, model, particles, style, icons, tileset, tilemap, flipbook, rig2d, shader, behaviour, navmesh, terrain, timeline, environment, texture-array, video, look, sfx, strings, rig or import)",
                                doc.ty
                            ),
                        ));
                        continue;
                    }
                    Err(e) => {
                        early_errors.push((rel.clone(), "scene", e.to_string()));
                        continue;
                    }
                }
            }
            _ => {
                ignored.push(rel.clone());
                continue;
            }
        };
        let bytes = read(rel, path)?;
        sources.push(Source {
            rel: rel.clone(),
            path: path.clone(),
            kind,
            bytes,
            settings: None,
            hash: 0,
            gltf: None,
            error: None,
            encoded: std::sync::OnceLock::new(),
            conformed: None,
        });
    }
    let mut consumed: BTreeSet<String> = BTreeSet::new();
    for s in sources.iter_mut() {
        s.settings = settings.remove(&s.rel);
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
            match model_text::to_glb(&String::from_utf8_lossy(&s.bytes)) {
                Ok(glb) => s.bytes = glb,
                Err(e) => {
                    s.error = Some(e);
                    continue;
                }
            }
        }
        if s.kind == Kind::Model {
            let dir = s.path.parent().unwrap_or(Path::new("."));
            match model::load(&s.bytes, dir) {
                Ok(g) => {
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
    // Conformance (docs/art.md §2.3): the 2D textures, to the contract.
    let conformed =
        conform::texture2d::conform(root, &assets_dir, &mut sources, &known, contract.as_ref())
            .map_err(CookError)?;
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
                let mut ids = Vec::new();
                for (i, u) in g.textures.iter().enumerate() {
                    let id = format!("{}#tex{i}", s.rel);
                    let image_hash = hash_bytes(&g.images[u.image]).to_le_bytes();
                    let st = stamp(&[
                        b"texture",
                        tex_desc.as_bytes(),
                        &image_hash,
                        &u.kind.to_le_bytes(),
                    ]);
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
                    stamp(&[
                        b"model",
                        NODE_ROWS,
                        mesh_desc.as_bytes(),
                        &hash,
                        joined.as_bytes(),
                        budget_desc.as_bytes(),
                    ])
                } else {
                    stamp(&[
                        b"model",
                        NODE_ROWS,
                        mesh_desc.as_bytes(),
                        &hash,
                        joined.as_bytes(),
                    ])
                };
                outs.insert(
                    0,
                    (s.rel.clone(), format::Model::NAME, st, Work::Model(s, ids)),
                );
                if group_of.contains_key(&s.rel) {
                    let st = stamp(&[b"proxy", NODE_ROWS, mesh_desc.as_bytes(), &hash]);
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
                            let st = stamp(&[
                                b"animation",
                                NODE_ROWS,
                                mesh_desc.as_bytes(),
                                a.describe().as_bytes(),
                                &hash,
                                deps.as_bytes(),
                            ]);
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
                    let st = stamp(&[b"texture", tex_desc.as_bytes(), &hash, &k.to_le_bytes()]);
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
                // An unreadable header fails in the cook, with its reason.
                let chunking = audio::shape(&s.bytes)
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
                let st = stamp(&[b"style", &hash, refs.join("\n").as_bytes()]);
                outs.push((s.rel.clone(), format::Style::NAME, st, Work::Style(s)));
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
                let st = stamp(&[b"flipbook", &hash, deps.as_bytes()]);
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
    for (i, p) in plan.iter().enumerate() {
        let r = &mut reports[p.source];
        let (bytes, cooked, ms, start_ms, worker) =
            match results[i].lock().expect("no poisoning").take() {
                Some((Ok((bytes, notes)), ms, start, worker)) => {
                    r.notes.extend(notes);
                    (Some(bytes), true, ms, start, worker)
                }
                Some((Err(e), ..)) => {
                    r.error = Some(e.0);
                    continue;
                }
                None => (None, false, 0.0, 0.0, 0),
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

fn run(
    p: &Planned,
    options: &CookOptions,
    threads: usize,
    schemas: &BTreeMap<String, Schema>,
    known: &BTreeSet<String>,
    budgets: &stats::Budgets,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    match p.work.as_ref().expect("planned work") {
        Work::Model(s, ids) => {
            let g = s.gltf.as_ref().expect("loaded");
            let mesh = options
                .mesh
                .for_model(settings_block(s, "model"))
                .map_err(CookError)?;
            let (blob, n) = model::cook(g, ids, &mesh, p.stamp, false)?;
            // A model written as text is held to the project's budgets by
            // the cook itself: past them it is refused, not flagged.
            if ext(&s.rel) == "ogt" {
                for (what, value, limit, key) in [
                    (
                        "LOD 0 triangles",
                        n.triangles,
                        budgets.triangles,
                        "triangles",
                    ),
                    ("vertices", n.vertices, budgets.vertices, "vertices"),
                ] {
                    if value as u64 > limit {
                        return Err(CookError(format!(
                            "{what}: {value} is over the {key} budget of {limit} (project.ogt's budgets.{key}): lower the parts' segments, curveSegments, steps or detail, or raise the budget"
                        )));
                    }
                }
            }
            Ok((blob, n.notes))
        }
        Work::Proxy(s) => {
            let g = s.gltf.as_ref().expect("loaded");
            let mesh = options
                .mesh
                .for_model(settings_block(s, "model"))
                .map_err(CookError)?;
            model::cook(g, &[], &mesh, p.stamp, true).map(|(b, mut n)| {
                n.notes.retain(|x| x.contains("triangles"));
                (
                    b,
                    n.notes.into_iter().map(|x| format!("proxy: {x}")).collect(),
                )
            })
        }
        Work::Animation(s, a) => {
            let g = s.gltf.as_ref().expect("loaded");
            anim::cook(g, &s.bytes, &options.mesh, a, p.stamp, &assets_root(s))
        }
        Work::GltfTexture(s, image, k) => {
            let g = s.gltf.as_ref().expect("loaded");
            let img = texture::decode(&g.images[*image])
                .map_err(|e| CookError(format!("{}: image {image}: {e}", p.id)))?;
            let note = format!("{}: {}x{} image {image}", p.id, img.width, img.height);
            texture::cook(&img, *k, &options.texture, p.stamp).map(|b| (b, vec![note]))
        }
        Work::Texture(s, k) if s.rel.ends_with(".ktx2") => {
            let explicit = settings_block(s, "texture")
                .and_then(|b| b.get("kind"))
                .map(|_| *k);
            ktx2_source::cook(&s.bytes, explicit, &options.texture, p.stamp)
        }
        Work::Texture(s, k) => {
            let img =
                texture::decode(s.conformed.as_deref().unwrap_or(&s.bytes)).map_err(CookError)?;
            let note = format!(
                "{}x{} {}",
                img.width,
                img.height,
                match *k {
                    format::kind::NORMAL => "normal (UASTC)",
                    format::kind::LINEAR => "linear (UASTC)",
                    format::kind::PIXEL => "pixel art (RGBA8, no mips)",
                    _ => "colour (ETC1S)",
                }
            );
            texture::cook(&img, *k, &options.texture, p.stamp).map(|b| (b, vec![note]))
        }
        Work::Sound(s, _) if s.rel.ends_with(".ogt") => {
            sfx::cook(&s.bytes, p.stamp).map(|(b, n)| (b, vec![n]))
        }
        Work::Sound(s, chunking) => {
            let e = s.encoded(chunking.map(|c| c.0))?;
            audio::cook(&e, *chunking, p.stamp).map(|(b, n)| (b, vec![n]))
        }
        Work::SoundChunk(s, per, n) => {
            let e = s.encoded(Some(*per))?;
            audio::cook_chunk(&e, *n, p.stamp).map(|b| (b, Vec::new()))
        }
        Work::Lut(s) => {
            let text = std::str::from_utf8(&s.bytes)
                .map_err(|_| CookError("a .cube LUT is text (UTF-8)".into()))?;
            let lut = crate::fx::parse_cube(text).map_err(CookError)?;
            let mut w = og_blob::BlobWriter::new(
                format::Lut::type_hash(),
                format::Lut::schema_version(),
                p.stamp,
            );
            w.table(
                "meta",
                &[format::LutMeta { size: lut.size }],
                format::row_hash::<format::LutMeta>(),
            )
            .raw("rgba", lut.rgba, 16);
            Ok((w.finish(), vec![format!("{0}³ colour table", lut.size)]))
        }
        Work::Emitter(s) => {
            let text = std::str::from_utf8(&s.bytes)
                .map_err(|_| CookError("an emitter is text (UTF-8)".into()))?;
            let missing = std::cell::RefCell::new(Vec::new());
            let check = |id: &str| {
                if !known.contains(crate::runtime::asset_id(id)) {
                    missing.borrow_mut().push(id.to_string());
                }
            };
            let em = crate::fx::parse_emitter_with(
                text,
                &mut |id| {
                    check(id);
                    Ok(None)
                },
                &mut |id| {
                    check(id);
                    Ok(None)
                },
            )
            .map_err(CookError)?;
            let missing = missing.into_inner();
            if !missing.is_empty() {
                return Err(CookError(format!(
                    "the emitter names texture(s) or texture array(s) not under assets/: {}",
                    missing.join(", ")
                )));
            }
            let mut w = og_blob::BlobWriter::new(
                format::Emitter::type_hash(),
                format::Emitter::schema_version(),
                p.stamp,
            );
            w.table(
                "meta",
                &[format::EmitterMeta {
                    text_bytes: text.len() as u32,
                }],
                format::row_hash::<format::EmitterMeta>(),
            )
            .raw("text", text.as_bytes().to_vec(), 16);
            Ok((
                w.finish(),
                vec![format!(
                    "{} particles, {} burst(s)",
                    em.max_particles,
                    em.bursts.len()
                )],
            ))
        }
        Work::Shader(s) => {
            let text = std::str::from_utf8(&s.bytes)
                .map_err(|_| CookError("a shader is text (UTF-8)".into()))?;
            let name = s.rel.trim_end_matches(".ogt");
            let shader = crate::shader::parse_shader(name, text, &mut |src| read_sibling(s, src))
                .map_err(CookError)?;
            let compiled = crate::shader::validate_shader(&shader)
                .map_err(|e| CookError(format!("the shader does not compile: {e}")))?;
            let canonical = crate::shader::shader_text(&shader);
            let mut w = og_blob::BlobWriter::new(
                format::Shader::type_hash(),
                format::Shader::schema_version(),
                p.stamp,
            );
            w.table(
                "meta",
                &[format::ShaderMeta {
                    text_bytes: canonical.len() as u32,
                }],
                format::row_hash::<format::ShaderMeta>(),
            )
            .raw("text", canonical.into_bytes(), 16);
            Ok((
                w.finish(),
                vec![format!(
                    "{} parameter(s); compiled for {}",
                    shader.params.len(),
                    compiled.join(", ")
                )],
            ))
        }
        Work::Post(s) => {
            let text = std::str::from_utf8(&s.bytes)
                .map_err(|_| CookError("a post pass is text (UTF-8)".into()))?;
            let name = s.rel.trim_end_matches(".ogt");
            let post = crate::shader::parse_post(name, text, &mut |src| read_sibling(s, src))
                .map_err(CookError)?;
            let compiled = crate::shader::validate_post(&post)
                .map_err(|e| CookError(format!("the post pass does not compile: {e}")))?;
            let canonical = crate::shader::post_text(&post);
            let mut w = og_blob::BlobWriter::new(
                format::Post::type_hash(),
                format::Post::schema_version(),
                p.stamp,
            );
            w.table(
                "meta",
                &[format::PostMeta {
                    text_bytes: canonical.len() as u32,
                }],
                format::row_hash::<format::PostMeta>(),
            )
            .raw("text", canonical.into_bytes(), 16);
            Ok((
                w.finish(),
                vec![format!(
                    "{} parameter(s); compiled for {}",
                    post.params.len(),
                    compiled.join(", ")
                )],
            ))
        }
        Work::Style(s) => {
            let text = std::str::from_utf8(&s.bytes)
                .map_err(|_| CookError("a style is text (UTF-8)".into()))?;
            let mut file = og_ui::parse_style(text).map_err(CookError)?;
            file.resolve(&s.rel, |id| known.contains(crate::runtime::asset_id(id)));
            let missing: Vec<String> = file
                .references()
                .into_iter()
                .filter(|r| !known.contains(crate::runtime::asset_id(r)))
                .collect();
            if !missing.is_empty() {
                let dir = s.rel.rsplit_once('/').map_or("", |(d, _)| d);
                return Err(CookError(format!(
                    "the style names asset(s) neither beside it in {dir}/ nor under assets/: {} (an id is read relative to the style file first, then from assets/; a `fallback` font is the project's own: point it at a CJK font the game has, or remove the line, og-taste §1)",
                    missing.join(", ")
                )));
            }
            let mut w = og_blob::BlobWriter::new(
                format::Style::type_hash(),
                format::Style::schema_version(),
                p.stamp,
            );
            w.table(
                "meta",
                &[format::StyleMeta {
                    text_bytes: text.len() as u32,
                }],
                format::row_hash::<format::StyleMeta>(),
            )
            .raw("text", text.as_bytes().to_vec(), 16);
            let parts = file.theme.art.slices.iter().flatten().count();
            Ok((
                w.finish(),
                vec![format!("style '{}', {parts} frame(s)", file.name)],
            ))
        }
        Work::Tileset(s) => {
            let text = std::str::from_utf8(&s.bytes)
                .map_err(|_| CookError("a tileset is text (UTF-8)".into()))?;
            tilemap::cook_tileset(text, &|id| known.contains(id), p.stamp)
        }
        Work::Tilemap(s, m) => {
            tilemap::cook_tilemap(&s.rel, &s.bytes, m, &|id| known.contains(id), p.stamp)
        }
        Work::Flipbook(s) => flipbook::cook(s, p.stamp),
        Work::Rig2d(s) => rig2d::cook(s, p.stamp),
        Work::FlipbookArray(s, sheet) => flipbook::cook_array(s, sheet, &options.texture, p.stamp),
        Work::FlipbookLayer(s, sheet, n) => {
            flipbook::cook_layer(s, sheet, *n, &options.texture, p.stamp).map(|b| (b, Vec::new()))
        }
        Work::Icons(s) => icons::cook(s, p.stamp),
        Work::Behaviour(s) => e6::cook_behaviour(s, p.stamp),
        Work::Strings(s) => e6::cook_strings(s, p.stamp),
        Work::Rig(s) => anim::cook_rig(&s.bytes, p.stamp),
        Work::NavMesh(s) => e6::cook_navmesh(s, p.stamp),
        Work::Terrain(s) => terrain::cook_terrain(s, &|id| known.contains(id), p.stamp),
        Work::Timeline(s) => timeline::cook_timeline(s, &|id| known.contains(id), p.stamp),
        Work::Lightmap(s) => lightmap::cook_lightmap(s, p.stamp),
        Work::Environment(s) => environment::cook_environment(s, p.stamp),
        Work::Look(s) => look::cook_look(s, &assets_root(s), &|id| known.contains(id), p.stamp),
        Work::Palette(s) => look::cook_palette(s, p.stamp),
        Work::TextureArray(s, decl) => {
            let root = texture_array::assets_dir(&s.path, &s.rel);
            texture_array::cook(&s.rel, &root, decl, &options.texture, p.stamp)
        }
        Work::ArrayLayer(s, decl, n) => {
            let root = texture_array::assets_dir(&s.path, &s.rel);
            texture_array::cook_layer(&root, decl, *n, &options.texture, p.stamp)
                .map(|b| (b, Vec::new()))
        }
        Work::VideoMeta(plan) => video::cook_meta(plan, p.stamp),
        Work::VideoArray(s, plan) => Ok((
            video::cook_array(&s.rel, plan, &options.texture, p.stamp),
            Vec::new(),
        )),
        Work::VideoFrame(src, h, plan, n) => {
            video::cook_frame(src, *h, plan, *n, &options.texture, p.stamp).map(|b| (b, Vec::new()))
        }
        Work::VideoAudio(src, plan) => {
            let wav = video::audio_wav(src, plan)?;
            let e = audio::encode(&wav, None).map_err(CookError)?;
            audio::cook(&e, None, p.stamp)
                .map(|(b, n)| (b, vec![format!("the video's sound: {n}")]))
        }
        Work::Font(s, f) => {
            font::cook(&s.bytes, f, threads, p.stamp).map(|(b, n)| (b, vec![n.note]))
        }
        Work::Scene(s) => {
            let text = std::str::from_utf8(&s.bytes)
                .map_err(|_| CookError("scene is not UTF-8".into()))?;
            let doc = Document::parse(text).map_err(|e| CookError(e.to_string()))?;
            scene::cook(&doc, schemas, &|id| known.contains(id), p.stamp)
        }
    }
}

/// The text of `rel` (an asset path) under the assets directory `s` came from.
fn read_sibling(s: &Source, rel: &str) -> Result<String, String> {
    let mut root = s.path.clone();
    for _ in s.rel.split('/') {
        root.pop();
    }
    if rel.split('/').any(|c| c == ".." || c.is_empty()) {
        return Err(format!("'{rel}' is not a path under assets/"));
    }
    std::fs::read_to_string(root.join(rel)).map_err(|e| format!("{e}"))
}

/// The group of a planned output: its asset's, except a glTF's `#proxy`
/// and `#anim`, which stay in the first group (the proxy draws while the
/// group is away; clips are small and bound at load), and the chunks of a
/// sound cooked in chunks after its first (`#chunk<n>`): its head goes in
/// its group's pack and each other chunk stays a file of its own, read
/// ahead of the sound's playheads once the group is resident, as a chunked
/// sound of the first group's are.
fn output_group(id: &str, rel: &str, group_of: &BTreeMap<String, String>) -> Option<String> {
    let chunk = id
        .rsplit_once("#chunk")
        .is_some_and(|(_, n)| n.parse::<u32>().is_ok());
    if id.ends_with("#proxy") || id.ends_with("#anim") || chunk {
        return None;
    }
    group_of.get(rel).cloned()
}

/// Every scene's streaming groups, checked across scenes: a group's name is
/// taken once, an asset is in at most one group, and a group holds glTF
/// models without skins or morph targets, textures and sounds (each swapped
/// in behind a stand-in's handle), and texture arrays, flipbooks (an `array
/// true` sheet with its array and layers), environment maps, palettes,
/// looks and LUTs (loaded once the group is resident, let go when it
/// unloads). A refusal is
/// the scene's error, and its scene adds none of its groups.
fn plan_groups(
    sources: &[Source],
    known: &BTreeSet<String>,
    errors: &mut Vec<(String, &'static str, String)>,
) -> stream::Groups {
    let mut groups: BTreeMap<String, stream::GroupDecl> = BTreeMap::new();
    let mut group_of: BTreeMap<String, String> = BTreeMap::new();
    for s in sources.iter().filter(|s| s.kind == Kind::Scene) {
        let text = String::from_utf8_lossy(&s.bytes);
        let decls = match stream::groups_of(&s.rel, &text, &|id| known.contains(id)) {
            Ok(d) => d,
            Err(e) => {
                errors.push((s.rel.clone(), "scene", e));
                continue;
            }
        };
        // A scene with a refused group adds none of its groups.
        let mut refused = None;
        let mut mine: BTreeMap<String, stream::GroupDecl> = BTreeMap::new();
        let mut mine_of: BTreeMap<String, String> = BTreeMap::new();
        'groups: for g in &decls {
            if let Some(other) = groups.get(&g.name).or_else(|| mine.get(&g.name)) {
                refused = Some(format!(
                    "stream \"{}\" is declared by {} too; a group's name is taken once",
                    g.name, other.scene
                ));
                break;
            }
            for a in &g.assets {
                if let Some(other) = group_of.get(a).or_else(|| mine_of.get(a)) {
                    refused = Some(format!(
                        "stream \"{}\": {a} is in stream \"{other}\" already; an asset is in one group",
                        g.name
                    ));
                    break 'groups;
                }
                let Some(src) = sources.iter().find(|x| &x.rel == a) else {
                    continue;
                };
                // A skinned or morphed mesh cannot be swapped in place.
                let skinned = src.gltf.as_ref().is_some_and(|m| {
                    m.doc.skins().next().is_some()
                        || m.doc
                            .meshes()
                            .any(|x| x.primitives().any(|p| p.morph_targets().next().is_some()))
                });
                let fits = matches!(
                    src.kind,
                    Kind::Model
                        | Kind::Texture
                        | Kind::Sound
                        | Kind::TextureArray
                        | Kind::Flipbook
                        | Kind::Environment
                        | Kind::Palette
                        | Kind::Look
                        | Kind::Lut
                );
                if !fits || skinned {
                    refused = Some(format!(
                        "stream \"{}\": {a} is {}; a group holds glTF models without skins or morph targets, textures, texture arrays, flipbooks, sounds, environment maps, palettes, looks and colour LUTs",
                        g.name,
                        if skinned {
                            "a skinned or morphed model".to_string()
                        } else {
                            format!("a {}", src.kind.name())
                        }
                    ));
                    break 'groups;
                }
            }
            for a in &g.assets {
                mine_of.insert(a.clone(), g.name.clone());
            }
            mine.insert(g.name.clone(), g.clone());
        }
        if refused.is_none() {
            groups.append(&mut mine);
            group_of.append(&mut mine_of);
        }
        if let Some(e) = refused {
            errors.push((s.rel.clone(), "scene", e));
        }
    }
    (groups, group_of)
}
