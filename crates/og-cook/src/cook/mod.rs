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

mod anim;
mod audio;
mod flipbook;
mod font;
mod model;
mod scene;
mod texture;
pub mod tilemap;

use crate::format::{self, BlobKind, COOKER_VERSION};
use crate::runtime::{Manifest, ManifestAsset, ManifestOutput, MANIFEST};
use og_core::hash::{hash_bytes, StableHasher};
use og_core::ogt::Document;
use og_core::Schema;
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
}

impl Default for MeshOptions {
    fn default() -> Self {
        MeshOptions {
            lod_levels: 3,
            lod_ratio: 0.5,
            lod_max_error: 0.05,
            overdraw_threshold: 1.05,
            tangents: true,
        }
    }
}

impl MeshOptions {
    fn describe(&self) -> String {
        format!(
            "lods {} ratio {} error {} overdraw {} tangents {} quantize 16/oct16/uv16/rgba8 codec meshopt-v1",
            self.lod_levels, self.lod_ratio, self.lod_max_error, self.overdraw_threshold, self.tangents
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
        }
    }
}

impl TextureOptions {
    fn describe(&self) -> String {
        format!(
            "etc1s {} uastc {} rdo {} zstd {} mips {}",
            self.etc1s_quality,
            self.uastc_level,
            self.uastc_rdo.map_or("off".to_string(), |r| r.to_string()),
            self.zstd,
            self.mipmaps
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
}

impl Default for FontSettings {
    fn default() -> Self {
        FontSettings {
            charset: vec!["latin".into()],
            chars: String::new(),
            em_px: 32.0,
            px_range: 4.0,
            page_size: 1024,
        }
    }
}

impl FontSettings {
    fn describe(&self) -> String {
        format!(
            "charset {} chars {:?} em {} range {} page {} mtsdf",
            self.charset.join(" "),
            self.chars,
            self.em_px,
            self.px_range,
            self.page_size
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
        for r in &self.removed {
            s.push_str(&format!("- `cooked/{r}`: removed (stale)\n"));
        }
        s
    }

    /// The report as JSON.
    pub fn to_json(&self) -> String {
        use serde_json::{json, Value};
        let assets: Vec<Value> = self
            .assets
            .iter()
            .map(|a| {
                json!({
                    "source": a.source, "kind": a.kind, "source_bytes": a.source_bytes,
                    "error": a.error, "notes": a.notes,
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
    Skin,
    Tileset,
    Tilemap,
    Flipbook,
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
            Kind::Skin => "skin",
            Kind::Tileset => "tileset",
            Kind::Tilemap => "tilemap",
            Kind::Flipbook => "flipbook",
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
}

enum Work<'a> {
    Model(&'a Source, Vec<String>),
    Animation(&'a Source, anim::AnimSettings),
    GltfTexture(&'a Source, usize, u32),
    Texture(&'a Source, u32),
    Sound(&'a Source),
    Lut(&'a Source),
    Emitter(&'a Source),
    Skin(&'a Source),
    Tileset(&'a Source),
    Tilemap(&'a Source, tilemap::MapSettings),
    Flipbook(&'a Source),
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
    let mut a = anim::AnimSettings::default();
    if let Some(b) = settings_block(s, "animation") {
        for e in &b.entries {
            let err = |x: og_core::Error| format!("line {}: {x}", e.line);
            match e.key.as_str() {
                "rate" => {
                    let r = e.f32(0).map_err(err)?;
                    if !(1.0..=120.0).contains(&r) {
                        return Err(format!(
                            "line {}: animation rate must be 1..=120 Hz",
                            e.line
                        ));
                    }
                    a.rate = Some(r);
                }
                "event" => a.events.push((
                    e.string(0).map_err(err)?.to_string(),
                    e.string(1).map_err(err)?.to_string(),
                    e.f32(2).map_err(err)?,
                )),
                k => {
                    return Err(format!(
                        "line {}: unknown animation setting '{k}' (rate, event)",
                        e.line
                    ))
                }
            }
        }
    }
    Ok(a)
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

    // Classify; import settings attach to the file they are named after.
    let names: BTreeSet<&str> = files.iter().map(|(r, _)| r.as_str()).collect();
    let mut sources: Vec<Source> = Vec::new();
    let mut settings: BTreeMap<String, (String, Document)> = BTreeMap::new();
    let mut ignored = Vec::new();
    let mut early_errors: Vec<(String, &'static str, String)> = Vec::new();
    for (rel, path) in &files {
        let kind = match ext(rel).as_str() {
            "glb" | "gltf" => Kind::Model,
            "png" | "jpg" | "jpeg" => Kind::Texture,
            "wav" => Kind::Sound,
            "ttf" | "otf" => Kind::Font,
            "cube" => Kind::Lut,
            "tmj" | "ldtk" => Kind::Tilemap,
            "ogt" => {
                let text = std::fs::read_to_string(path)
                    .map_err(|e| CookError(format!("{}: {e}", path.display())))?;
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
                    Ok(doc) if doc.ty == og_ui::skin::SKIN_TYPE => Kind::Skin,
                    Ok(doc) if doc.ty == "tileset" => Kind::Tileset,
                    Ok(doc) if doc.ty == "tilemap" => Kind::Tilemap,
                    Ok(doc) if doc.ty == "flipbook" => Kind::Flipbook,
                    Ok(doc) => {
                        early_errors.push((
                            rel.clone(),
                            "scene",
                            format!(
                                "an .ogt of type '{}' is not a cookable asset (scene, particles, skin, tileset, tilemap, flipbook or import)",
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
        let bytes =
            std::fs::read(path).map_err(|e| CookError(format!("{}: {e}", path.display())))?;
        sources.push(Source {
            rel: rel.clone(),
            path: path.clone(),
            kind,
            bytes,
            settings: None,
            hash: 0,
            gltf: None,
            error: None,
        });
    }
    let mut consumed: BTreeSet<String> = BTreeSet::new();
    for s in sources.iter_mut() {
        s.settings = settings.remove(&s.rel);
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
    // Files a .gltf references are part of that model, not assets of their own.
    sources.retain(|s| !consumed.contains(&s.rel));
    let known: BTreeSet<String> = sources.iter().map(|s| s.rel.clone()).collect();
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
                let st = stamp(&[
                    b"model",
                    mesh_desc.as_bytes(),
                    &hash,
                    ids.join("\n").as_bytes(),
                ]);
                outs.insert(
                    0,
                    (s.rel.clone(), format::Model::NAME, st, Work::Model(s, ids)),
                );
                if anim::wanted(g) {
                    match anim_settings(s) {
                        Ok(a) => {
                            let st = stamp(&[
                                b"animation",
                                mesh_desc.as_bytes(),
                                a.describe().as_bytes(),
                                &hash,
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
            Kind::Sound => {
                let st = stamp(&[b"sound", &hash]);
                outs.push((s.rel.clone(), format::Sound::NAME, st, Work::Sound(s)));
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
            Kind::Skin => {
                let text = String::from_utf8_lossy(&s.bytes);
                let refs: Vec<String> = og_ui::parse_skin(&text)
                    .map(|f| f.references())
                    .unwrap_or_default()
                    .into_iter()
                    .map(|r| format!("{r}={}", known.contains(crate::runtime::asset_id(&r))))
                    .collect();
                let st = stamp(&[b"skin", &hash, refs.join("\n").as_bytes()]);
                outs.push((s.rel.clone(), format::Skin::NAME, st, Work::Skin(s)));
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
            Kind::Flipbook => {
                let deps = flipbook::dependency_hashes(s);
                let st = stamp(&[b"flipbook", &hash, deps.as_bytes()]);
                outs.push((s.rel.clone(), format::Flipbook::NAME, st, Work::Flipbook(s)));
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
            let fresh = old_outputs.get(id.as_str()).is_some_and(|o| {
                o.stamp == st
                    && o.ty == ty
                    && o.file == file
                    && std::fs::read(cooked_dir.join(&file))
                        .is_ok_and(|b| b.len() as u64 == o.bytes && hash_bytes(&b) == o.hash)
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
    type Done = (Result<(Vec<u8>, Vec<String>), CookError>, f64);
    let results: Vec<Mutex<Option<Done>>> = plan.iter().map(|_| Mutex::new(None)).collect();
    let todo: Vec<usize> = (0..plan.len())
        .filter(|&i| plan[i].work.is_some())
        .collect();
    let next = std::sync::atomic::AtomicUsize::new(0);
    std::thread::scope(|scope| {
        for _ in 0..threads.min(todo.len()) {
            scope.spawn(|| loop {
                let k = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let Some(&i) = todo.get(k) else { break };
                let p = &plan[i];
                let t = Instant::now();
                let r = run(p, options, threads, &schemas, &known);
                *results[i].lock().expect("no poisoning") =
                    Some((r, t.elapsed().as_secs_f64() * 1000.0));
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
            }),
        }
    }
    let mut new_outputs: BTreeMap<usize, Vec<NewOutput>> = BTreeMap::new();
    for (i, p) in plan.iter().enumerate() {
        let r = &mut reports[p.source];
        let (bytes, cooked, ms) = match results[i].lock().expect("no poisoning").take() {
            Some((Ok((bytes, notes)), ms)) => {
                r.notes.extend(notes);
                (Some(bytes), true, ms)
            }
            Some((Err(e), _)) => {
                r.error = Some(e.0);
                continue;
            }
            None => (None, false, 0.0),
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
    for r in reports.iter_mut().filter(|r| r.error.is_some()) {
        r.outputs.clear();
    }
    let mut removed = Vec::new();
    for o in old.outputs() {
        if !keep.contains(&o.file) && cooked_dir.join(&o.file).exists() {
            std::fs::remove_file(cooked_dir.join(&o.file))
                .map_err(|e| CookError(format!("cooked/{}: {e}", o.file)))?;
            removed.push(o.file.clone());
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
    reports.sort_by(|a, b| a.source.cmp(&b.source));
    Ok(Report {
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
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    match p.work.as_ref().expect("planned work") {
        Work::Model(s, ids) => {
            let g = s.gltf.as_ref().expect("loaded");
            model::cook(g, ids, &options.mesh, p.stamp).map(|(b, n)| (b, n.notes))
        }
        Work::Animation(s, a) => {
            let g = s.gltf.as_ref().expect("loaded");
            anim::cook(g, &s.bytes, &options.mesh, a, p.stamp)
        }
        Work::GltfTexture(s, image, k) => {
            let g = s.gltf.as_ref().expect("loaded");
            let img = texture::decode(&g.images[*image])
                .map_err(|e| CookError(format!("{}: image {image}: {e}", p.id)))?;
            let note = format!("{}: {}x{} image {image}", p.id, img.width, img.height);
            texture::cook(&img, *k, &options.texture, p.stamp).map(|b| (b, vec![note]))
        }
        Work::Texture(s, k) => {
            let img = texture::decode(&s.bytes).map_err(CookError)?;
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
        Work::Sound(s) => audio::cook(&s.bytes, p.stamp).map(|(b, n)| (b, vec![n])),
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
            let mut missing = Vec::new();
            let em = crate::fx::parse_emitter(text, &mut |id| {
                if !known.contains(crate::runtime::asset_id(id)) {
                    missing.push(id.to_string());
                }
                Ok(None)
            })
            .map_err(CookError)?;
            if !missing.is_empty() {
                return Err(CookError(format!(
                    "the emitter names texture(s) not under assets/: {}",
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
        Work::Skin(s) => {
            let text = std::str::from_utf8(&s.bytes)
                .map_err(|_| CookError("a skin is text (UTF-8)".into()))?;
            let file = og_ui::parse_skin(text).map_err(CookError)?;
            let missing: Vec<String> = file
                .references()
                .into_iter()
                .filter(|r| !known.contains(crate::runtime::asset_id(r)))
                .collect();
            if !missing.is_empty() {
                return Err(CookError(format!(
                    "the skin names asset(s) not under assets/: {}",
                    missing.join(", ")
                )));
            }
            let mut w = og_blob::BlobWriter::new(
                format::Skin::type_hash(),
                format::Skin::schema_version(),
                p.stamp,
            );
            w.table(
                "meta",
                &[format::SkinMeta {
                    text_bytes: text.len() as u32,
                }],
                format::row_hash::<format::SkinMeta>(),
            )
            .raw("text", text.as_bytes().to_vec(), 16);
            let parts = file.theme.skin.slices.iter().flatten().count();
            Ok((
                w.finish(),
                vec![format!("skin '{}', {parts} frame(s)", file.name)],
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
