//! The runtime loaders: `cooked/` blobs into the renderer's input types and
//! plain scene data. Nothing here touches the ECS or the GPU; the host spawns
//! what [`SceneDesc`] describes and adds what [`CookedModel::add_to`] returns.
//!
//! A [`Bundle`] is the cook manifest (`cooked/.cook.ogt`) read as an index:
//! asset id to output file, type and cook stamp. Every load names an asset id
//! (the source path under `assets/`, `assets/` prefix optional; a glTF's
//! textures are `<model>#tex<n>`, or [`gltf_texture_id`] where the cook
//! writes one per image and use), finds the output in the manifest and
//! refuses a blob whose type, schema version or cook stamp disagrees with it
//! (`docs/design.md` §6). Bytes come from the host (`fetch` on the web); on
//! native targets [`Bundle::open`] and the `load_*` methods read files.

use crate::format::{self, *};
use crate::{ktx2, meshopt_codec, qoa};
use glam::{Mat4, Quat, Vec3};
use og_blob::{Blob, BlobError, Expect};
use og_core::ogt::{Block, Document};
use og_render_data::{
    AlphaMode, ColorSpace, Material, MaterialHandle, MeshData, MeshHandle, Mips,
    PhysicalExtensions, SceneResources, StandardMaterial, TextureData, TextureDesc, TextureFormat,
    TextureFormats, TextureHandle, TextureSource, TextureUpload, Vertex,
};
use std::collections::BTreeMap;
use std::fmt;

mod hdr;
mod media;
mod model;
mod scene;
mod terrain;
mod uncooked;

impl Bundle {
    /// A lossless scalar/vector lattice; no GPU allocation or normalization.
    pub fn field(&self, id: &str, bytes: &[u8]) -> Result<crate::field::FieldData, LoadError> {
        let blob = self.blob::<format::Field>(id, bytes)?;
        let meta = one::<format::FieldMeta>(&blob)?;
        let raw = blob.raw("values")?;
        if !raw.len().is_multiple_of(4) {
            return Err(invalid(&blob, "field values are cut short"));
        }
        let field = crate::field::FieldData {
            dimensions: [meta.width, meta.height, meta.depth],
            channels: meta.channels,
            origin: [meta.origin_x, meta.origin_y, meta.origin_z],
            spacing: [meta.spacing_x, meta.spacing_y, meta.spacing_z],
            values: raw
                .as_chunks::<4>()
                .0
                .iter()
                .map(|v| f32::from_le_bytes(*v))
                .collect(),
        };
        field.validate().map_err(|e| invalid(&blob, e))?;
        Ok(field)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_field(&self, id: &str) -> Result<crate::field::FieldData, LoadError> {
        self.field(id, &self.read(id)?)
    }
}

pub use hdr::{HdrRows, exr_size, hdr_size, is_exr, is_hdr};
pub use media::*;
pub use model::*;
pub use scene::*;
pub use terrain::*;
pub use uncooked::{UncookedGltf, leaf_season};

/// The manifest's file name inside `cooked/`.
pub const MANIFEST: &str = ".cook.ogt";

/// The cooked file suffix of a font pack's face (`<id>.face.ogb`, docs/ui.md
/// §3.4): the web boot leaves these out and the engine reads one when the
/// locale's chain or a frame's text first needs it.
pub const FONT_PACK_FACE: &str = "face.ogb";

/// Why a load was refused. Converts into `og_core::Error` for the API.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadError {
    /// The manifest has no such asset: it was never cooked or failed to cook.
    NotCooked { id: String },
    /// The blob was refused (stale stamp, wrong type, damaged).
    Blob(BlobError),
    /// Reading a file failed.
    Io { path: String, reason: String },
    /// The blob is structurally valid but its content is not.
    Invalid { path: String, reason: String },
    /// The manifest could not be read.
    Manifest(String),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::NotCooked { id } => write!(
                f,
                "{id}: not in cooked/ (run og cook; check the cook report for its error)"
            ),
            LoadError::Blob(e) => e.fmt(f),
            LoadError::Io { path, reason } => write!(f, "{path}: {reason}"),
            LoadError::Invalid { path, reason } => write!(f, "{path}: {reason}"),
            LoadError::Manifest(reason) => write!(f, "cooked/{MANIFEST}: {reason}"),
        }
    }
}

impl std::error::Error for LoadError {}

impl From<BlobError> for LoadError {
    fn from(e: BlobError) -> LoadError {
        LoadError::Blob(e)
    }
}

impl From<LoadError> for og_core::Error {
    fn from(e: LoadError) -> og_core::Error {
        let kind = match &e {
            LoadError::NotCooked { .. } => og_core::ErrorKind::NotFound,
            LoadError::Blob(BlobError::StampMismatch { .. })
            | LoadError::Blob(BlobError::TypeMismatch { .. })
            | LoadError::Blob(BlobError::SchemaVersion { .. }) => og_core::ErrorKind::Schema,
            _ => og_core::ErrorKind::Parse,
        };
        og_core::Error::new(kind, e.to_string())
    }
}

/// One cooked output as the manifest records it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifestOutput {
    /// The asset id (`models/ship.glb`, `models/ship.glb#tex0`).
    pub id: String,
    /// The blob type (`og.Model`, `og.Texture`, ...).
    pub ty: String,
    /// The file under `cooked/`.
    pub file: String,
    pub stamp: u64,
    /// FNV-1a of the file's bytes.
    pub hash: u64,
    /// The file's Subresource Integrity metadata (`sha384-<base64>`): what a
    /// loader checks bytes from another origin against (docs/web-export.md
    /// §Asset base).
    pub integrity: String,
    pub bytes: u64,
    /// The streaming group its asset belongs to (`None`: the first group,
    /// loaded at boot).
    pub group: Option<String>,
}

/// What loads a streaming group.
#[derive(Clone, Debug, PartialEq)]
pub enum StreamTrigger {
    /// Only `og.stream.request`.
    Manual,
    /// The camera within `radius` of `anchor`; unloaded past `unload`.
    Distance {
        anchor: [f32; 3],
        radius: f32,
        unload: f32,
    },
    /// The camera inside the named entity's box (its position ± half its scale).
    Volume(String),
    /// The world cells that place the declaring scene (G3-3): the group
    /// holds that sub-scene's blob and the assets it names, and loads ahead
    /// of the anchors nearing those cells (og-runtime's `stream::cells`).
    Cell,
}

/// One streaming group as the manifest records it.
#[derive(Clone, Debug, PartialEq)]
pub struct ManifestStream {
    pub name: String,
    /// The scene that declares it.
    pub scene: String,
    pub trigger: StreamTrigger,
    /// Higher loads first and unloads last.
    pub priority: i32,
    /// Its pack under `cooked/` (`stream/<name>.ogb`), its size and hash.
    pub file: String,
    pub bytes: u64,
    pub hash: u64,
    /// Its pack's Subresource Integrity metadata, as an output's.
    pub integrity: String,
    /// What its assets take once loaded (GPU textures and meshes, audio).
    pub memory: u64,
    /// Its asset ids.
    pub assets: Vec<String>,
    /// Its sounds' lengths, so a sound plays as silence of the same length
    /// (and the same ticks see it end) while the group is not resident.
    pub sounds: Vec<StreamSound>,
}

/// A grouped sound's shape: what plays as silence until its group loads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StreamSound {
    pub id: String,
    pub sample_rate: u32,
    pub channels: u32,
    pub frames: u64,
}

/// One source asset as the manifest records it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifestAsset {
    /// The path under `assets/`.
    pub source: String,
    pub kind: String,
    /// FNV-1a over the source and its import settings.
    pub source_hash: u64,
    pub source_bytes: u64,
    /// The engine wasm modules loading it needs (`modules nav physics`,
    /// sorted; empty: none, and no line): its records' (the IDL's
    /// `split`) and those of every asset it references, a scene's
    /// sub-scenes and prefabs included. og.scene.preload, og.load.scene
    /// and a timeline's pre-roll fetch them ahead of their first call
    /// (docs/web-export.md §Modules).
    pub modules: Vec<String>,
    /// Direct cooked content dependencies, including exact subresource IDs.
    /// Hosts fetch their closure before relocating a scene's typed columns.
    pub dependencies: Vec<String>,
    pub outputs: Vec<ManifestOutput>,
}

/// `cooked/.cook.ogt`: what was cooked from what.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Manifest {
    pub cooker: String,
    /// The project's profile: project.ogt's creation form as
    /// `look::project_profile` reads it (`profile "3d"` or `"2d"` in the
    /// text). What the build, publish and the play page take the
    /// profile from.
    pub profile: crate::look::Profile,
    /// The engine wasm modules opening the bundle needs (`modules nav
    /// physics`, sorted; empty: none, and no line): its boot scene's
    /// (project.ogt `scene`, as [`ManifestAsset::modules`]) and its
    /// project.ogt `flow` dialogue's, or every
    /// asset's when the cook cannot name that scene. og build loads them
    /// beside the primary; og.load.bundle fetches them once the bundle
    /// opens.
    pub modules: Vec<String>,
    /// Per asset kind, the option text its stamps cover.
    pub options: BTreeMap<String, String>,
    /// Sorted by source path.
    pub assets: Vec<ManifestAsset>,
    /// The streaming groups, by name.
    pub streams: Vec<ManifestStream>,
    /// Required bundle manifests, relative to this bundle's directory and
    /// pinned by integrity. Opened before this bundle becomes ready.
    pub requires: Vec<ManifestFile>,
    /// og.motion's model (`motion/model.ogb`), for a project whose code
    /// uses og.motion: not an output, so the boot bundle never loads it;
    /// the runtime reads it in the background once the bundle is open.
    pub motion: Option<ManifestFile>,
    /// Font-pack face files explicitly reachable outside the pack's index.
    /// Optional in older manifests; the web fetches these before start.
    /// These are cooked output paths (bytes and integrity in `assets`),
    /// not engine module names in `modules`: they must already be resident
    /// when a synchronous loader runs, not best-effort code prefetch hints.
    pub preload: Vec<String>,
    /// Actual scene output ID to its unique lightmap output ID.
    pub lightmaps: BTreeMap<String, String>,
}

/// A file of the bundle outside its outputs and groups.
#[derive(Clone, Debug, PartialEq)]
pub struct ManifestFile {
    /// Under `cooked/`.
    pub file: String,
    pub bytes: u64,
    /// FNV-1a of its bytes.
    pub hash: u64,
    /// Its Subresource Integrity metadata, as an output's.
    pub integrity: String,
}

fn hex(v: u64) -> String {
    format!("{v:016x}")
}

fn unhex(s: &str) -> Option<u64> {
    (s.len() == 16)
        .then(|| u64::from_str_radix(s, 16).ok())
        .flatten()
}

impl Manifest {
    /// Parses the manifest text.
    pub fn parse(text: &str) -> Result<Manifest, LoadError> {
        let bad = |r: String| LoadError::Manifest(r);
        let doc = Document::parse(text).map_err(|e| bad(e.to_string()))?;
        if doc.ty != "cook" || doc.version != 1 {
            return Err(bad(format!(
                "expected a 'cook' version 1 document, got '{}' version {}",
                doc.ty, doc.version
            )));
        }
        let mut m = Manifest {
            cooker: doc
                .body
                .get("cooker")
                .and_then(|e| e.string(0).ok())
                .unwrap_or_default()
                .to_string(),
            profile: match doc.body.get("profile") {
                Some(e) => e
                    .string(0)
                    .ok()
                    .and_then(crate::look::Profile::from_name)
                    .ok_or_else(|| {
                        bad(format!(
                            "line {}: profile is one of {}",
                            e.line,
                            crate::look::Profile::ALL
                                .map(|p| format!("\"{}\"", p.name()))
                                .join(", ")
                        ))
                    })?,
                None => return Err(bad("no profile line: cook the project again".into())),
            },
            ..Default::default()
        };
        let words = |e: &og_core::ogt::Entry| -> Result<Vec<String>, LoadError> {
            e.values
                .iter()
                .map(|v| v.as_word().map(str::to_string))
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| {
                    bad(format!(
                        "line {}: modules names engine modules, written bare",
                        e.line
                    ))
                })
        };
        if let Some(e) = doc.body.get("modules") {
            m.modules = words(e)?;
        }
        if let Some(block) = doc.body.get("options").and_then(|e| e.block.as_ref()) {
            for e in &block.entries {
                m.options.insert(
                    e.key.clone(),
                    e.string(0).map_err(|e| bad(e.to_string()))?.to_string(),
                );
            }
        }
        for e in doc.body.get_all("preload") {
            m.preload
                .push(e.string(0).map_err(|e| bad(e.to_string()))?.to_string());
        }
        for e in doc.body.get_all("lightmap") {
            let scene = e.string(0).map_err(|e| bad(e.to_string()))?.to_string();
            let id = e.string(1).map_err(|e| bad(e.to_string()))?.to_string();
            if m.lightmaps.insert(scene, id).is_some() {
                return Err(bad("a scene has more than one lightmap".into()));
            }
        }
        let field = |b: &Block, key: &str, line: usize| -> Result<String, LoadError> {
            let e = b
                .get(key)
                .ok_or_else(|| bad(format!("line {line}: missing '{key}'")))?;
            e.values
                .first()
                .and_then(|v| {
                    v.as_str()
                        .or(v.as_word())
                        .map(str::to_string)
                        .or(v.as_number().map(|n| n.text().to_string()))
                })
                .ok_or_else(|| bad(format!("line {}: '{key}' has no value", e.line)))
        };
        let hexf = |b: &Block, key: &str, line: usize| -> Result<u64, LoadError> {
            let s = field(b, key, line)?;
            unhex(&s).ok_or_else(|| bad(format!("line {line}: '{key}' is not 16 hex digits")))
        };
        let num = |b: &Block, key: &str, line: usize| -> Result<u64, LoadError> {
            field(b, key, line)?
                .parse()
                .map_err(|_| bad(format!("line {line}: '{key}' is not a number")))
        };
        for a in doc.body.get_all("asset") {
            let block = a
                .block
                .as_ref()
                .ok_or_else(|| bad(format!("line {}: asset without a block", a.line)))?;
            let mut asset = ManifestAsset {
                source: a.string(0).map_err(|e| bad(e.to_string()))?.to_string(),
                kind: field(block, "kind", a.line)?,
                source_hash: hexf(block, "source", a.line)?,
                source_bytes: num(block, "bytes", a.line)?,
                modules: block
                    .get("modules")
                    .map(words)
                    .transpose()?
                    .unwrap_or_default(),
                dependencies: block
                    .get("dependencies")
                    .map(|e| {
                        e.values
                            .iter()
                            .map(|v| {
                                v.as_str().map(str::to_owned).ok_or_else(|| {
                                    bad(format!(
                                        "line {}: dependencies names quoted content IDs",
                                        e.line
                                    ))
                                })
                            })
                            .collect::<Result<_, _>>()
                    })
                    .transpose()?
                    .unwrap_or_default(),
                outputs: Vec::new(),
            };
            for o in block.get_all("output") {
                let ob = o
                    .block
                    .as_ref()
                    .ok_or_else(|| bad(format!("line {}: output without a block", o.line)))?;
                asset.outputs.push(ManifestOutput {
                    id: o.string(0).map_err(|e| bad(e.to_string()))?.to_string(),
                    ty: field(ob, "type", o.line)?,
                    file: field(ob, "file", o.line)?,
                    stamp: hexf(ob, "stamp", o.line)?,
                    hash: hexf(ob, "hash", o.line)?,
                    integrity: field(ob, "integrity", o.line)?,
                    bytes: num(ob, "bytes", o.line)?,
                    group: ob
                        .get("group")
                        .map(|_| field(ob, "group", o.line))
                        .transpose()?,
                });
            }
            m.assets.push(asset);
        }
        for st in doc.body.get_all("stream") {
            let b = st
                .block
                .as_ref()
                .ok_or_else(|| bad(format!("line {}: stream without a block", st.line)))?;
            let t = b
                .get("trigger")
                .ok_or_else(|| bad(format!("line {}: stream without a trigger", st.line)))?;
            let nums = |from: usize| -> Vec<f32> {
                t.values[from.min(t.values.len())..]
                    .iter()
                    .filter_map(|v| v.as_number().map(|n| n.as_f32()))
                    .collect()
            };
            let trigger = match t.values.first().and_then(|v| v.as_word()) {
                Some("manual") => StreamTrigger::Manual,
                Some("cell") => StreamTrigger::Cell,
                Some("volume") => StreamTrigger::Volume(
                    t.string(1)
                        .map_err(|_| bad(format!("line {}: volume without a name", t.line)))?
                        .to_string(),
                ),
                Some("distance") => match nums(1)[..] {
                    [x, y, z, radius, unload] => StreamTrigger::Distance {
                        anchor: [x, y, z],
                        radius,
                        unload,
                    },
                    _ => {
                        return Err(bad(format!(
                            "line {}: distance x y z radius unload",
                            t.line
                        )));
                    }
                },
                _ => return Err(bad(format!("line {}: an unknown trigger", t.line))),
            };
            m.streams.push(ManifestStream {
                name: st.string(0).map_err(|e| bad(e.to_string()))?.to_string(),
                scene: field(b, "scene", st.line)?,
                trigger,
                priority: field(b, "priority", st.line)?
                    .parse()
                    .map_err(|_| bad(format!("line {}: 'priority' is not a number", st.line)))?,
                file: field(b, "file", st.line)?,
                bytes: num(b, "bytes", st.line)?,
                hash: hexf(b, "hash", st.line)?,
                integrity: field(b, "integrity", st.line)?,
                memory: num(b, "memory", st.line)?,
                assets: b
                    .get_all("asset")
                    .map(|a| {
                        a.string(0)
                            .map(str::to_string)
                            .map_err(|e| bad(e.to_string()))
                    })
                    .collect::<Result<_, _>>()?,
                sounds: b
                    .get_all("sound")
                    .map(|a| {
                        let n = |i: usize| {
                            a.values
                                .get(i)
                                .and_then(|v| v.as_number())
                                .map(|n| n.as_f64())
                                .filter(|v| *v >= 0.0 && v.fract() == 0.0)
                                .ok_or_else(|| {
                                    bad(format!(
                                        "line {}: sound \"<id>\" rate channels frames",
                                        a.line
                                    ))
                                })
                        };
                        Ok(StreamSound {
                            id: a.string(0).map_err(|e| bad(e.to_string()))?.to_string(),
                            sample_rate: n(1)? as u32,
                            channels: n(2)? as u32,
                            frames: n(3)? as u64,
                        })
                    })
                    .collect::<Result<_, LoadError>>()?,
            });
        }
        for e in doc.body.get_all("requires") {
            let b = e
                .block
                .as_ref()
                .ok_or_else(|| bad(format!("line {}: requires without a block", e.line)))?;
            m.requires.push(ManifestFile {
                file: e.string(0).map_err(|e| bad(e.to_string()))?.to_string(),
                bytes: num(b, "bytes", e.line)?,
                hash: hexf(b, "hash", e.line)?,
                integrity: field(b, "integrity", e.line)?,
            });
        }
        if let Some(e) = doc.body.get("motion") {
            let b = e
                .block
                .as_ref()
                .ok_or_else(|| bad(format!("line {}: motion without a block", e.line)))?;
            m.motion = Some(ManifestFile {
                file: e.string(0).map_err(|e| bad(e.to_string()))?.to_string(),
                bytes: num(b, "bytes", e.line)?,
                hash: hexf(b, "hash", e.line)?,
                integrity: field(b, "integrity", e.line)?,
            });
        }
        Ok(m)
    }

    /// The canonical text (what the cook writes).
    pub fn to_text(&self) -> String {
        use og_core::ogt::{Entry, Number, Value};
        let s = |v: &str| Value::String(v.to_string());
        let w = |v: &str| Value::Word(v.to_string());
        let n = |v: u64| Value::Number(Number::from(v));
        let mut doc = Document::new("cook", 1);
        doc.body.push(Entry::new("cooker").with(s(&self.cooker)));
        doc.body
            .push(Entry::new("profile").with(s(self.profile.name())));
        if !self.modules.is_empty() {
            doc.body.push(
                self.modules
                    .iter()
                    .fold(Entry::new("modules"), |e, m| e.with(w(m))),
            );
        }
        let mut options = Block::new();
        for (k, v) in &self.options {
            options.push(Entry::new(k.clone()).with(s(v)));
        }
        doc.body.push(Entry::new("options").with_block(options));
        for file in &self.preload {
            doc.body.push(Entry::new("preload").with(s(file)));
        }
        for (scene, id) in &self.lightmaps {
            doc.body
                .push(Entry::new("lightmap").with(s(scene)).with(s(id)));
        }
        for a in &self.assets {
            let mut b = Block::new()
                .with(Entry::new("kind").with(w(&a.kind)))
                .with(Entry::new("source").with(s(&hex(a.source_hash))))
                .with(Entry::new("bytes").with(n(a.source_bytes)));
            if !a.modules.is_empty() {
                b.push(
                    a.modules
                        .iter()
                        .fold(Entry::new("modules"), |e, m| e.with(w(m))),
                );
            }
            if !a.dependencies.is_empty() {
                b.push(
                    a.dependencies
                        .iter()
                        .fold(Entry::new("dependencies"), |e, id| e.with(s(id))),
                );
            }
            for o in &a.outputs {
                let ob = Block::new()
                    .with(Entry::new("type").with(w(&o.ty)))
                    .with(Entry::new("file").with(s(&o.file)))
                    .with(Entry::new("stamp").with(s(&hex(o.stamp))))
                    .with(Entry::new("hash").with(s(&hex(o.hash))))
                    .with(Entry::new("integrity").with(s(&o.integrity)))
                    .with(Entry::new("bytes").with(n(o.bytes)));
                let ob = match &o.group {
                    Some(g) => ob.with(Entry::new("group").with(s(g))),
                    None => ob,
                };
                b.push(Entry::new("output").with(s(&o.id)).with_block(ob));
            }
            doc.body
                .push(Entry::new("asset").with(s(&a.source)).with_block(b));
        }
        let f = |v: f32| Value::Number(Number::from_f32(v).unwrap_or_else(|_| Number::from(0u32)));
        for g in &self.streams {
            let trigger = match &g.trigger {
                StreamTrigger::Manual => Entry::new("trigger").with(w("manual")),
                StreamTrigger::Cell => Entry::new("trigger").with(w("cell")),
                StreamTrigger::Volume(e) => Entry::new("trigger").with(w("volume")).with(s(e)),
                StreamTrigger::Distance {
                    anchor,
                    radius,
                    unload,
                } => Entry::new("trigger")
                    .with(w("distance"))
                    .with(f(anchor[0]))
                    .with(f(anchor[1]))
                    .with(f(anchor[2]))
                    .with(f(*radius))
                    .with(f(*unload)),
            };
            let mut b = Block::new()
                .with(Entry::new("scene").with(s(&g.scene)))
                .with(trigger)
                .with(Entry::new("priority").with(Value::Number(Number::from(g.priority))))
                .with(Entry::new("file").with(s(&g.file)))
                .with(Entry::new("bytes").with(n(g.bytes)))
                .with(Entry::new("hash").with(s(&hex(g.hash))))
                .with(Entry::new("integrity").with(s(&g.integrity)))
                .with(Entry::new("memory").with(n(g.memory)));
            for a in &g.assets {
                b.push(Entry::new("asset").with(s(a)));
            }
            for x in &g.sounds {
                b.push(
                    Entry::new("sound")
                        .with(s(&x.id))
                        .with(n(u64::from(x.sample_rate)))
                        .with(n(u64::from(x.channels)))
                        .with(n(x.frames)),
                );
            }
            doc.body
                .push(Entry::new("stream").with(s(&g.name)).with_block(b));
        }
        for (key, m) in self
            .requires
            .iter()
            .map(|m| ("requires", m))
            .chain(self.motion.iter().map(|m| ("motion", m)))
        {
            let b = Block::new()
                .with(Entry::new("bytes").with(n(m.bytes)))
                .with(Entry::new("hash").with(s(&hex(m.hash))))
                .with(Entry::new("integrity").with(s(&m.integrity)));
            doc.body
                .push(Entry::new(key).with(s(&m.file)).with_block(b));
        }
        doc.to_text()
            .expect("manifest keys and words are identifiers")
    }

    /// Every output, by asset id.
    pub fn outputs(&self) -> impl Iterator<Item = &ManifestOutput> {
        self.assets.iter().flat_map(|a| a.outputs.iter())
    }
}

/// Normalises an asset id: `assets/x` and `x` name the same asset.
pub fn asset_id(path: &str) -> &str {
    path.strip_prefix("assets/").unwrap_or(path)
}

/// The animation blob id of a model id (`models/x.glb` → `models/x.glb#anim`).
pub fn anim_id(id: &str) -> String {
    let id = asset_id(id);
    if id.ends_with("#anim") {
        id.to_string()
    } else {
        format!("{id}#anim")
    }
}

/// The cooked bundle a game loads from.
#[derive(Clone, Debug)]
pub struct Bundle {
    manifest: Manifest,
    index: BTreeMap<String, usize>,
    assets: BTreeMap<String, usize>,
    outputs: Vec<ManifestOutput>,
    #[cfg(not(target_arch = "wasm32"))]
    dir: Option<std::path::PathBuf>,
}

impl Bundle {
    /// A bundle from the manifest text (the host fetched `cooked/.cook.ogt`).
    pub fn from_manifest(text: &str) -> Result<Bundle, LoadError> {
        let manifest = Manifest::parse(text)?;
        let outputs: Vec<ManifestOutput> = manifest.outputs().cloned().collect();
        let mut identities = std::collections::BTreeSet::new();
        for output in &outputs {
            if !identities.insert(output.id.to_lowercase()) {
                return Err(LoadError::Manifest(format!(
                    "duplicate global asset identity {}",
                    output.id
                )));
            }
        }
        let index = og_core::sort::map(outputs.iter().enumerate().map(|(i, o)| (o.id.clone(), i)));
        let assets = og_core::sort::map(
            manifest
                .assets
                .iter()
                .enumerate()
                .map(|(i, a)| (a.source.clone(), i)),
        );
        Ok(Bundle {
            manifest,
            index,
            assets,
            outputs,
            #[cfg(not(target_arch = "wasm32"))]
            dir: None,
        })
    }

    /// Opens `cooked/` on disk (native targets).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open(cooked: &std::path::Path) -> Result<Bundle, LoadError> {
        let path = cooked.join(MANIFEST);
        Self::open_manifest(cooked, &path)
    }

    /// A tooling manifest whose output files live at `cooked` (the cook's
    /// aggregate build index need not be a runtime delivery bundle).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open_manifest(
        cooked: &std::path::Path,
        path: &std::path::Path,
    ) -> Result<Bundle, LoadError> {
        let text = std::fs::read_to_string(path).map_err(|e| LoadError::Io {
            path: path.display().to_string(),
            reason: format!("{e} (run og cook)"),
        })?;
        let mut b = Bundle::from_manifest(&text)?;
        b.dir = Some(cooked.to_path_buf());
        Ok(b)
    }

    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// The streaming groups (phase 3), by name.
    pub fn streams(&self) -> &[ManifestStream] {
        &self.manifest.streams
    }

    /// The manifest entry of a source asset (`scenes/hall.ogt`; the
    /// `assets/` prefix optional): its source hash and its outputs.
    pub fn asset(&self, source: &str) -> Result<&ManifestAsset, LoadError> {
        let source = asset_id(source);
        self.assets
            .get(source)
            .map(|&i| &self.manifest.assets[i])
            .ok_or_else(|| LoadError::NotCooked {
                id: source.to_string(),
            })
    }

    /// The manifest entry of an asset id.
    pub fn output(&self, id: &str) -> Result<&ManifestOutput, LoadError> {
        let id = asset_id(id);
        self.index
            .get(id)
            .map(|&i| &self.outputs[i])
            .ok_or_else(|| LoadError::NotCooked { id: id.to_string() })
    }

    /// Reads an asset's blob from `cooked/` into 16-aligned memory (native).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn read(&self, id: &str) -> Result<og_blob::AlignedBytes, LoadError> {
        let out = self.output(id)?;
        let dir = self.dir.as_ref().ok_or_else(|| LoadError::Io {
            path: out.file.clone(),
            reason: "the bundle was not opened from a directory".into(),
        })?;
        let path = dir.join(&out.file);
        let bytes = std::fs::read(&path).map_err(|e| LoadError::Io {
            path: path.display().to_string(),
            reason: e.to_string(),
        })?;
        Ok(og_blob::AlignedBytes::from_slice(&bytes))
    }

    /// Validates `bytes` as the blob of `id`: type, schema version and the
    /// manifest's cook stamp.
    pub fn blob<'a, K: BlobKind>(&self, id: &str, bytes: &'a [u8]) -> Result<Blob<'a>, LoadError> {
        let out = self.output(id)?;
        if out.ty != K::NAME {
            return Err(LoadError::Invalid {
                path: format!("cooked/{}", out.file),
                reason: format!("is a {}, not a {}", out.ty, K::NAME),
            });
        }
        let expect = Expect {
            type_hash: K::type_hash(),
            schema_version: K::schema_version(),
            stamp: Some(out.stamp),
        };
        Ok(Blob::open(bytes, &format!("cooked/{}", out.file), &expect)?)
    }

    /// Decodes a model blob the cook has just written (no manifest yet:
    /// its type and schema version are checked, not its stamp). 16-aligned
    /// `bytes`.
    pub fn decode_model(id: &str, bytes: &[u8]) -> Result<CookedModel, LoadError> {
        let expect = Expect {
            type_hash: format::Model::type_hash(),
            schema_version: format::Model::schema_version(),
            stamp: None,
        };
        let blob = Blob::open(bytes, id, &expect)?;
        CookedModel::decode(asset_id(id), &blob)
    }

    /// Decodes a model blob.
    pub fn model(&self, id: &str, bytes: &[u8]) -> Result<CookedModel, LoadError> {
        let blob = self.blob::<format::Model>(id, bytes)?;
        CookedModel::decode(asset_id(id), &blob)
    }

    /// Decodes a texture blob (the KTX2 stays encoded until
    /// [`CookedTexture::to_texture_data`]).
    pub fn texture(&self, id: &str, bytes: &[u8]) -> Result<CookedTexture, LoadError> {
        let blob = self.blob::<format::Texture>(id, bytes)?;
        let meta = one::<TextureMeta>(&blob)?;
        let ktx2 = blob.raw("ktx2")?.to_vec();
        ktx2::parse(&ktx2).map_err(|reason| invalid(&blob, reason))?;
        let mips = match blob.raw_opt("mips") {
            Some(b) => Some(
                MipChain::from_bytes(b)
                    .ok_or_else(|| invalid(&blob, "its `mips` section is not 12 bytes"))?,
            ),
            None => None,
        };
        let orientation = match blob.raw_opt("orientation") {
            Some(b) => b
                .try_into()
                .map(u32::from_le_bytes)
                .map_err(|_| invalid(&blob, "its `orientation` section is not 4 bytes"))?,
            None => 1,
        };
        Ok(CookedTexture {
            id: asset_id(id).to_string(),
            meta,
            ktx2,
            mips,
            orientation,
        })
    }

    /// The parts of a `#sway` texture blob ([`sway_id`]); `None` for a
    /// texture without them.
    pub fn sway(&self, id: &str, bytes: &[u8]) -> Result<Option<SwayParts>, LoadError> {
        let blob = self.blob::<format::Texture>(id, bytes)?;
        match blob.raw_opt("sway") {
            Some(b) => SwayParts::from_bytes(b)
                .map(Some)
                .ok_or_else(|| invalid(&blob, "its `sway` section is not whole parts")),
            None => Ok(None),
        }
    }

    /// Decodes a colour LUT blob.
    pub fn lut(&self, id: &str, bytes: &[u8]) -> Result<og_render_data::ColorLut, LoadError> {
        let blob = self.blob::<format::Lut>(id, bytes)?;
        let meta = one::<format::LutMeta>(&blob)?;
        let rgba = blob.raw("rgba")?.to_vec();
        let n = meta.size as usize;
        if rgba.len() != n * n * n * 4 {
            return Err(invalid(
                &blob,
                format!(
                    "a {n}³ LUT holds {} bytes, not {}",
                    n * n * n * 4,
                    rgba.len()
                ),
            ));
        }
        Ok(og_render_data::ColorLut {
            size: meta.size,
            rgba,
        })
    }

    /// Native emitter fields and blob-local resource bindings.
    pub fn emitter(&self, id: &str, bytes: &[u8]) -> Result<crate::fx::cooked::Emitter, LoadError> {
        let blob = self.blob::<format::Emitter>(id, bytes)?;
        let meta = one::<format::EmitterMeta>(&blob)?;
        let data = blob.raw("data")?;
        if data.len() != meta.data_bytes as usize {
            return Err(invalid(&blob, "the emitter data length is invalid"));
        }
        crate::fx::cooked::Emitter::from_bytes(data).map_err(|e| invalid(&blob, &e))
    }

    /// Native effect material fields and blob-local resource bindings.
    pub fn effect(&self, id: &str, bytes: &[u8]) -> Result<crate::fx::cooked::Effect, LoadError> {
        let blob = self.blob::<format::Effect>(id, bytes)?;
        let meta = one::<format::EffectMeta>(&blob)?;
        let data = blob.raw("data")?;
        if data.len() != meta.data_bytes as usize {
            return Err(invalid(&blob, "the effect data length is invalid"));
        }
        crate::fx::cooked::Effect::from_bytes(data).map_err(|e| invalid(&blob, &e))
    }

    /// Native composed-effect timeline.
    pub fn vfx(&self, id: &str, bytes: &[u8]) -> Result<crate::vfx::Vfx, LoadError> {
        let blob = self.blob::<format::Vfx>(id, bytes)?;
        let meta = one::<format::VfxMeta>(&blob)?;
        let data = blob.raw("data")?;
        if data.len() != meta.data_bytes as usize {
            return Err(invalid(&blob, "the vfx data length is invalid"));
        }
        crate::vfx::Vfx::from_bytes(data).map_err(|e| invalid(&blob, &e))
    }

    /// The material shader of a shader blob.
    pub fn shader(
        &self,
        id: &str,
        bytes: &[u8],
    ) -> Result<og_render_data::MaterialShader, LoadError> {
        let blob = self.blob::<format::Shader>(id, bytes)?;
        let meta = one::<format::ShaderMeta>(&blob)?;
        let (code, params) = crate::shader::read(&blob, meta.code_bytes)?;
        let mode = match meta.mode {
            0 => og_render_data::ShaderMode::Hooks,
            1 => og_render_data::ShaderMode::Full { lit: true },
            2 => og_render_data::ShaderMode::Full { lit: false },
            3 => og_render_data::ShaderMode::Sprite,
            _ => return Err(invalid(&blob, "invalid shader mode")),
        };
        let shader = og_render_data::MaterialShader {
            name: id.into(),
            code,
            params,
            mode,
        };
        shader.check().map_err(|e| invalid(&blob, e))?;
        Ok(shader)
    }

    /// The custom post pass of a post blob.
    pub fn post(&self, id: &str, bytes: &[u8]) -> Result<og_render_data::PostShader, LoadError> {
        let blob = self.blob::<format::Post>(id, bytes)?;
        let meta = one::<format::PostMeta>(&blob)?;
        let (code, params) = crate::shader::read(&blob, meta.code_bytes)?;
        let post = og_render_data::PostShader {
            name: id.into(),
            code,
            params,
        };
        post.check().map_err(|e| invalid(&blob, e))?;
        Ok(post)
    }

    /// The custom UI material of a UI material blob.
    pub fn ui_material(
        &self,
        id: &str,
        bytes: &[u8],
    ) -> Result<og_render_data::UiMaterialShader, LoadError> {
        let blob = self.blob::<format::UiMaterial>(id, bytes)?;
        let meta = one::<format::UiMaterialMeta>(&blob)?;
        let (code, params) = crate::shader::read(&blob, meta.code_bytes)?;
        let material = og_render_data::UiMaterialShader {
            name: id.into(),
            code,
            params,
        };
        material.check().map_err(|e| invalid(&blob, e))?;
        Ok(material)
    }

    /// The triangles and paints of a vector blob, for
    /// `og_render::RenderScene::add_ui_vector`.
    pub fn vector(&self, id: &str, bytes: &[u8]) -> Result<og_draw::ui::UiVectorData, LoadError> {
        use og_core::glam::Vec2;
        let blob = self.blob::<format::Vector>(id, bytes)?;
        let meta = one::<format::VectorMeta>(&blob)?;
        let paints = table::<format::VectorPaintRow>(&blob, "paints")?;
        let stops = table::<format::VectorStopRow>(&blob, "stops")?;
        let vertices = blob.raw("vertices")?;
        let indices = blob.raw("indices")?;
        let vsize = std::mem::size_of::<og_draw::ui::UiVectorVertex>();
        if vertices.len() < meta.vertex_count as usize * vsize
            || indices.len() < meta.index_count as usize * 4
            || paints.len() != meta.paint_count as usize
        {
            return Err(invalid(&blob, "the vector's sections are cut short"));
        }
        let vertices: Vec<og_draw::ui::UiVectorVertex> =
            bytemuck::pod_collect_to_vec(&vertices[..meta.vertex_count as usize * vsize]);
        let indices: Vec<u32> =
            bytemuck::pod_collect_to_vec(&indices[..meta.index_count as usize * 4]);
        if indices.iter().any(|i| *i as usize >= vertices.len())
            || vertices.iter().any(|v| v.paint as usize >= paints.len())
        {
            return Err(invalid(&blob, "an index or a paint out of range"));
        }
        let mut out = Vec::with_capacity(paints.len());
        for p in &paints {
            out.push(if p.kind == 0 {
                og_draw::ui::UiVectorPaint::Solid([p.r, p.g, p.b, p.a])
            } else {
                let range = p.first_stop as usize..(p.first_stop + p.stop_count) as usize;
                let stops = stops
                    .get(range)
                    .ok_or_else(|| invalid(&blob, "a paint's stops out of range"))?
                    .iter()
                    .map(|s| og_draw::ui::UiStop {
                        offset: s.offset,
                        color: [s.r, s.g, s.b, s.a],
                    })
                    .collect();
                og_draw::ui::UiVectorPaint::Gradient(og_draw::ui::UiGradient {
                    kind: match p.kind {
                        1 => og_draw::ui::UiGradientKind::Linear,
                        2 => og_draw::ui::UiGradientKind::Radial,
                        3 => og_draw::ui::UiGradientKind::Conic,
                        4 => og_draw::ui::UiGradientKind::Diamond,
                        _ => return Err(invalid(&blob, "a paint of an unknown kind")),
                    },
                    from: Vec2::new(p.from_x, p.from_y),
                    to: Vec2::new(p.to_x, p.to_y),
                    stops,
                    spread: match p.spread {
                        1 => og_draw::ui::UiSpread::Repeat,
                        2 => og_draw::ui::UiSpread::Reflect,
                        3 => og_draw::ui::UiSpread::RepeatStops,
                        _ => og_draw::ui::UiSpread::Pad,
                    },
                    dither: p.dither != 0,
                })
            });
        }
        Ok(og_draw::ui::UiVectorData {
            view_box: [meta.x, meta.y, meta.width, meta.height],
            vertices,
            indices,
            paints: out,
            strokes: Vec::new(),
        })
    }

    /// The bytes of a Rive artboard blob (decode them with og-ui's
    /// `rive::decode`, which checks them).
    pub fn rive_artboard(&self, id: &str, bytes: &[u8]) -> Result<Vec<u8>, LoadError> {
        let blob = self.blob::<format::RiveArtboard>(id, bytes)?;
        let meta = one::<format::RiveArtboardMeta>(&blob)?;
        let raw = blob.raw("artboard")?;
        raw.get(..meta.bytes as usize)
            .map(<[u8]>::to_vec)
            .ok_or_else(|| invalid(&blob, "the Rive artboard is cut short"))
    }

    /// A style pack blob's `pack.json` (read it with og-ui-next's
    /// `pack::Layer::parse`; its asset paths are ids of this bundle).
    pub fn pack_text(&self, id: &str, bytes: &[u8]) -> Result<String, LoadError> {
        let blob = self.blob::<format::Pack>(id, bytes)?;
        let meta = one::<format::PackMeta>(&blob)?;
        let text = blob.raw("text")?;
        let text = text
            .get(..meta.text_bytes as usize)
            .ok_or_else(|| invalid(&blob, "the pack is cut short"))?;
        String::from_utf8(text.to_vec()).map_err(|_| invalid(&blob, "the pack is not UTF-8"))
    }

    /// Decodes a sound blob (the QOA or Opus stays encoded until
    /// [`CookedSound::decode`]).
    pub fn sound(&self, id: &str, bytes: &[u8]) -> Result<CookedSound, LoadError> {
        let blob = self.blob::<format::Sound>(id, bytes)?;
        let meta = one::<SoundMeta>(&blob)?;
        let (qoa, opus) = match meta.codec {
            format::sound_codec::QOA => (blob.raw("qoa")?.to_vec(), Vec::new()),
            format::sound_codec::OPUS => (Vec::new(), blob.raw("opus")?.to_vec()),
            c => return Err(invalid(&blob, format!("unknown sound codec {c}"))),
        };
        Ok(CookedSound {
            id: asset_id(id).to_string(),
            meta,
            qoa,
            opus,
        })
    }

    /// Reads a video blob's meta row (a clip's frames are the texture array
    /// `<id>#frames`, a stream's the `og.VideoStream` `<id>#stream`; its
    /// sound `<id>#audio`).
    pub fn video(&self, id: &str, bytes: &[u8]) -> Result<format::VideoMeta, LoadError> {
        let blob = self.blob::<format::Video>(id, bytes)?;
        let meta = one::<format::VideoMeta>(&blob)?;
        if meta.frames == 0 || meta.fps == 0 {
            return Err(invalid(&blob, "a video with no frames or no rate"));
        }
        Ok(meta)
    }

    /// Reads a video stream's index: its shape, its codec configuration
    /// and each frame's place in its chunk ([`video_stream_of`] with this
    /// bundle's stamp).
    pub fn video_stream(&self, id: &str, bytes: &[u8]) -> Result<CookedVideoStream, LoadError> {
        let o = self.output(id)?;
        video_stream_of(bytes, &o.file, o.stamp, id)
    }

    /// Reads chunk `id` of a video stream ([`video_chunk_of`] with this
    /// bundle's stamp).
    pub fn video_chunk(&self, id: &str, bytes: &[u8]) -> Result<CookedVideoChunk, LoadError> {
        let o = self.output(id)?;
        video_chunk_of(bytes, &o.file, o.stamp)
    }

    /// Decodes a texture array blob: its shape and its layers' texture ids
    /// (each an `og.Texture` of its own, loaded as any texture).
    pub fn texture_array(&self, id: &str, bytes: &[u8]) -> Result<CookedTextureArray, LoadError> {
        let blob = self.blob::<format::TextureArray>(id, bytes)?;
        let meta = one::<format::TextureArrayMeta>(&blob)?;
        let rows = table::<format::ArrayLayerRow>(&blob, "layers")?;
        let strings = blob.raw("strings")?;
        let layers = rows
            .iter()
            .map(|r| {
                pool_str(strings, r.name_offset, r.name_len)
                    .map(str::to_string)
                    .ok_or_else(|| invalid(&blob, "layer name out of range"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if layers.len() != meta.layers as usize || layers.is_empty() {
            return Err(invalid(&blob, "layer count disagrees with the meta row"));
        }
        Ok(CookedTextureArray {
            id: asset_id(id).to_string(),
            meta,
            layers,
        })
    }

    /// Reads a chunk of a sound cooked in chunks (`<id>#chunk<n>`,
    /// [`chunk_id`]): its index and its QOA frames or Opus blocks.
    pub fn sound_chunk(&self, id: &str, bytes: &[u8]) -> Result<(u32, Vec<u8>), LoadError> {
        let blob = self.blob::<format::SoundChunk>(id, bytes)?;
        let meta = one::<format::SoundChunkMeta>(&blob)?;
        let data = blob.raw("opus").or_else(|_| blob.raw("qoa"))?;
        Ok((meta.chunk, data.to_vec()))
    }

    /// Decodes a font blob, inflating its atlas pages.
    pub fn font(&self, id: &str, bytes: &[u8]) -> Result<CookedFont, LoadError> {
        let blob = self.blob::<format::Font>(id, bytes)?;
        let meta = one::<FontMeta>(&blob)?;
        let glyphs = blob
            .table::<GlyphRow>("glyphs", row_hash::<GlyphRow>())?
            .into_owned();
        let mut pages = Vec::new();
        for p in 0..meta.page_count {
            let ktx = blob.raw(&format!("atlas.{p}"))?;
            let (width, height, rgba) =
                ktx2::decode_rgba8(ktx, 0).map_err(|r| invalid(&blob, r))?;
            pages.push(TextureData {
                width,
                height,
                rgba,
                color_space: ColorSpace::Linear,
                mips: Mips::None,
            });
        }
        let kerning = blob
            .table::<KernRow>("kerning", row_hash::<KernRow>())?
            .into_owned();
        Ok(CookedFont {
            id: asset_id(id).to_string(),
            meta,
            glyphs,
            kerning,
            pages,
            file: blob.raw_opt("file").map(<[u8]>::to_vec),
            pixel: blob
                .raw_opt("pixel")
                .and_then(|b| std::str::from_utf8(b).ok()?.parse().ok())
                .unwrap_or(0),
            names: blob
                .raw("names")
                .ok()
                .map(|b| {
                    String::from_utf8_lossy(b)
                        .split('\n')
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default(),
            tones: blob
                .raw_opt("tones")
                .map(|b| String::from_utf8_lossy(b).into_owned())
                .unwrap_or_default(),
        })
    }

    /// Decodes a scene blob.
    pub fn scene(&self, id: &str, bytes: &[u8]) -> Result<SceneDesc, LoadError> {
        let blob = self.blob::<format::Scene>(id, bytes)?;
        SceneDesc::decode(asset_id(id), &blob)
    }

    /// The clip names of an animation blob (`<model>#anim`), in glTF order,
    /// without decoding its clips (what `GltfInfo.animations` lists).
    pub fn clip_names(&self, id: &str, bytes: &[u8]) -> Result<Vec<String>, LoadError> {
        let blob = self.blob::<format::Animation>(id, bytes)?;
        let clips = table::<ClipRow>(&blob, "clips")?;
        let strings = blob.raw("strings")?;
        clips
            .iter()
            .map(|c| {
                pool_str(strings, c.name_offset, c.name_len)
                    .map(str::to_string)
                    .ok_or_else(|| invalid(&blob, "clip name out of range"))
            })
            .collect()
    }

    /// Reads and decodes a model (native).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_model(&self, id: &str) -> Result<CookedModel, LoadError> {
        self.model(id, &self.read(id)?)
    }

    /// Reads a texture (native).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_texture(&self, id: &str) -> Result<CookedTexture, LoadError> {
        self.texture(id, &self.read(id)?)
    }

    /// Reads a texture array (native): its size, kind and layer ids.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_texture_array(&self, id: &str) -> Result<CookedTextureArray, LoadError> {
        self.texture_array(id, &self.read(id)?)
    }

    /// Reads a sound (native).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_sound(&self, id: &str) -> Result<CookedSound, LoadError> {
        self.sound(id, &self.read(id)?)
    }

    /// Reads a font (native).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_font(&self, id: &str) -> Result<CookedFont, LoadError> {
        self.font(id, &self.read(id)?)
    }

    /// Reads a scene (native).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_scene(&self, id: &str) -> Result<SceneDesc, LoadError> {
        self.scene(id, &self.read(id)?)
    }

    /// The hash a bake keys the static model `id` on:
    /// [`crate::lightmap::geometry_hash`] of the LOD-0 parts
    /// [`CookedModel::add_to`] adds, read from the blob (its textures are
    /// not decoded: the geometry does not depend on them).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn model_geometry_hash(&self, id: &str) -> Result<u64, LoadError> {
        let model = self.load_model(id)?;
        let mut scene = Meshes::default();
        let parts = model.add_to_with(&mut scene, &[]);
        Ok(crate::lightmap::geometry_hash(
            parts.parts.iter().filter_map(|&(m, _, _)| scene.mesh(m)),
        ))
    }

    /// Reads a model and transcodes its textures to RGBA8 (native, feature
    /// `transcode`): everything [`CookedModel::add_to`] needs.
    #[cfg(all(feature = "transcode", not(target_arch = "wasm32")))]
    pub fn load_model_with_textures(
        &self,
        id: &str,
    ) -> Result<(CookedModel, Vec<TextureData>), LoadError> {
        let model = self.load_model(id)?;
        let textures = model
            .textures
            .iter()
            .map(|t| self.load_texture(&t.id)?.to_texture_data())
            .collect::<Result<Vec<_>, _>>()?;
        Ok((model, textures))
    }
}

/// What [`Bundle::model_geometry_hash`] adds a model to: a scene that
/// keeps its meshes, as og-render's `RenderScene` holds them, and nothing
/// else (no texture, material or instance is read back).
#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
struct Meshes {
    meshes: Vec<MeshData>,
    materials: usize,
}

#[cfg(not(target_arch = "wasm32"))]
impl SceneResources for Meshes {
    fn add_texture_source(&mut self, _: TextureSource) -> TextureHandle {
        TextureHandle::from_index(0)
    }
    fn set_texture_filter(&mut self, _: TextureHandle, _: og_render_data::TextureFilter) -> bool {
        true
    }
    fn add_material(&mut self, _: Material) -> MaterialHandle {
        self.materials += 1;
        MaterialHandle::from_index(self.materials - 1)
    }
    fn add_standard_material(&mut self, _: StandardMaterial) -> MaterialHandle {
        self.materials += 1;
        MaterialHandle::from_index(self.materials - 1)
    }
    fn standard_material(&self, _: MaterialHandle) -> Option<&StandardMaterial> {
        None
    }
    fn alias_material_when_alike(&mut self, _: MaterialHandle, _: MaterialHandle) -> bool {
        false
    }
    fn add_mesh(&mut self, mesh: MeshData) -> MeshHandle {
        self.meshes.push(mesh);
        MeshHandle::from_index(self.meshes.len() - 1)
    }
    fn mesh(&self, handle: MeshHandle) -> Option<&MeshData> {
        self.meshes.get(handle.index())
    }
    fn mesh_foliage(&self, _: MeshHandle) -> Option<&og_render_data::meshlet::MeshFoliage> {
        None
    }
    fn set_mesh_clusters(
        &mut self,
        _: MeshHandle,
        _: Option<og_render_data::meshlet::MeshClusters>,
    ) -> bool {
        true
    }
    fn set_mesh_foliage(
        &mut self,
        _: MeshHandle,
        _: Option<og_render_data::meshlet::MeshFoliage>,
    ) -> bool {
        true
    }
    fn add_instance(&mut self, _: MeshHandle, _: MaterialHandle, _: Mat4) {}
}

/// A refusal of `blob`'s content, for the loaders other crates keep.
pub fn invalid(blob: &Blob, reason: impl Into<String>) -> LoadError {
    LoadError::Invalid {
        path: blob.path().to_string(),
        reason: reason.into(),
    }
}

/// The one row of `blob`'s `meta` section.
pub fn one<T: og_core::Reflect + bytemuck::Pod>(blob: &Blob) -> Result<T, LoadError> {
    let rows = blob.table::<T>("meta", row_hash::<T>())?;
    rows.first()
        .copied()
        .ok_or_else(|| invalid(blob, "empty meta section"))
}

/// The rows of `blob`'s section `name`, checked against `T`'s row hash.
pub fn table<T: og_core::Reflect + bytemuck::Pod>(
    blob: &Blob,
    name: &str,
) -> Result<Vec<T>, LoadError> {
    Ok(blob.table::<T>(name, row_hash::<T>())?.into_owned())
}

fn opt(i: u32) -> Option<usize> {
    (i != NONE).then_some(i as usize)
}

// ── tilemaps ─────────────────────────────────────────────────────────────

impl Bundle {
    /// Decodes a tileset blob (`og.Tileset`).
    pub fn tileset(&self, id: &str, bytes: &[u8]) -> Result<og_tilemap::Tileset, LoadError> {
        let blob = self.blob::<format::Tileset>(id, bytes)?;
        decode_tileset(&blob)
    }

    /// Decodes a tilemap blob (`og.Tilemap`). Its tilesets are asset ids of
    /// tileset blobs ([`Bundle::tileset`]) or embedded in it.
    pub fn tilemap(&self, id: &str, bytes: &[u8]) -> Result<og_tilemap::TilemapAsset, LoadError> {
        let blob = self.blob::<format::Tilemap>(id, bytes)?;
        decode_tilemap(&blob)
    }

    /// Reads and decodes a tileset (native).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_tileset(&self, id: &str) -> Result<og_tilemap::Tileset, LoadError> {
        self.tileset(id, &self.read(id)?)
    }

    /// Reads and decodes a tilemap (native).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_tilemap(&self, id: &str) -> Result<og_tilemap::TilemapAsset, LoadError> {
        self.tilemap(id, &self.read(id)?)
    }
}

fn decode_tileset(blob: &Blob) -> Result<og_tilemap::Tileset, LoadError> {
    let meta = one::<format::TilesetMeta>(blob)?;
    let bytes = blob.raw("data")?;
    let bytes = bytes
        .get(..meta.data_bytes as usize)
        .ok_or_else(|| invalid(blob, "the native tileset is cut short"))?;
    og_tilemap::Tileset::decode(bytes).map_err(|e| invalid(blob, e.to_string()))
}

fn decode_tilemap(blob: &Blob) -> Result<og_tilemap::TilemapAsset, LoadError> {
    use format::layer_flag as fl;
    let meta = one::<format::TilemapMeta>(blob)?;
    let layers = table::<format::MapLayerRow>(blob, "layers")?;
    let objects = table::<format::MapObjectRow>(blob, "objects")?;
    let props = table::<format::MapPropRow>(blob, "props")?;
    let tilesets = table::<format::MapTilesetRow>(blob, "tilesets")?;
    let strings = blob.raw("strings")?;
    let cells = blob.raw("cells")?;
    let s = |o: u32, l: u32| -> Result<String, LoadError> {
        pool_str(strings, o, l)
            .map(str::to_string)
            .ok_or_else(|| invalid(blob, "a string is out of range"))
    };
    let n = (meta.width as usize) * (meta.height as usize);
    // G2-6: absent in an orthogonal map's blob.
    let orientation = match blob.raw_opt("orientation") {
        None => og_tilemap::Orientation::Orthogonal,
        Some(b) => b
            .get(..4)
            .and_then(|b| {
                og_tilemap::Orientation::from_code(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            })
            .ok_or_else(|| invalid(blob, "the orientation section holds no known orientation"))?,
    };
    let mut map = og_tilemap::TilemapAsset {
        orientation,
        width: meta.width,
        height: meta.height,
        tile_width_px: meta.tile_width_px,
        tile_height_px: meta.tile_height_px,
        unit: meta.unit,
        tilesets: Vec::new(),
        layers: Vec::new(),
        objects: Vec::new(),
    };
    for t in &tilesets {
        map.tilesets.push(match t.embedded {
            0 => og_tilemap::TilesetRef::Asset(s(t.offset, t.len)?),
            1 => {
                let first = t.offset as usize;
                let end = first
                    .checked_add(t.len as usize)
                    .ok_or_else(|| invalid(blob, "embedded tileset range overflows"))?;
                let bytes = blob
                    .raw("embedded")?
                    .get(first..end)
                    .ok_or_else(|| invalid(blob, "embedded tileset range is out of bounds"))?;
                og_tilemap::TilesetRef::Embedded(
                    og_tilemap::Tileset::decode(bytes).map_err(|e| invalid(blob, e.to_string()))?,
                )
            }
            _ => return Err(invalid(blob, "unknown tileset reference tag")),
        });
    }
    for l in &layers {
        let first = l.cell_offset as usize * 4;
        let bytes = cells
            .get(first..first + n * 4)
            .ok_or_else(|| invalid(blob, "a layer's cells are out of range"))?;
        map.layers.push(og_tilemap::Layer {
            name: s(l.name_offset, l.name_len)?,
            tileset: l.tileset,
            cells: bytes
                .as_chunks::<4>()
                .0
                .iter()
                .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect(),
            terrain: if l.flags & fl::TERRAIN != 0 {
                Some(s(l.terrain_offset, l.terrain_len)?)
            } else {
                None
            },
            parallax: glam::Vec2::new(l.parallax_x, l.parallax_y),
            offset: glam::Vec2::new(l.offset_x, l.offset_y),
            opacity: l.opacity,
            visible: l.flags & fl::VISIBLE != 0,
            collision: l.flags & fl::COLLISION != 0,
            collider_layers: l.collider_layers,
            collider_mask: l.collider_mask,
            cell_colliders: l.flags & fl::CELL_COLLIDERS != 0,
            repeat_x: l.flags & fl::REPEAT_X != 0,
            repeat_y: l.flags & fl::REPEAT_Y != 0,
            z: l.z,
            lit: l.flags & fl::UNLIT == 0,
        });
    }
    for o in &objects {
        let range = o.prop_first as usize..(o.prop_first + o.prop_count) as usize;
        let rows = props
            .get(range)
            .ok_or_else(|| invalid(blob, "an object's properties are out of range"))?;
        map.objects.push(og_tilemap::MapObject {
            name: s(o.name_offset, o.name_len)?,
            kind: s(o.kind_offset, o.kind_len)?,
            x: o.x,
            y: o.y,
            width: o.width,
            height: o.height,
            properties: rows
                .iter()
                .map(|p| Ok((s(p.key_offset, p.key_len)?, s(p.value_offset, p.value_len)?)))
                .collect::<Result<_, LoadError>>()?,
        });
    }
    map.validate().map_err(|e| invalid(blob, e.to_string()))?;
    Ok(map)
}

// ── E6: behaviours and navmeshes ─────────────────────────────────────────

impl Bundle {
    /// Native behaviour fields; decode with og-ai's `Behaviour::from_bytes`.
    pub fn behaviour_data(&self, id: &str, bytes: &[u8]) -> Result<Vec<u8>, LoadError> {
        let blob = self.blob::<format::Behaviour>(id, bytes)?;
        let meta = one::<format::BehaviourMeta>(&blob)?;
        let data = blob.raw("data")?;
        if data.len() != meta.data_bytes as usize {
            return Err(invalid(
                &blob,
                "the behaviour data length differs from its metadata",
            ));
        }
        Ok(data.to_vec())
    }

    /// Native dialogue or quest payload. og-story decodes and validates its
    /// fields; this layer owns the enclosing cooked format, not the feature.
    pub fn story_data(&self, id: &str, bytes: &[u8]) -> Result<Vec<u8>, LoadError> {
        let (data, blob) = match self.output(id)?.ty.as_str() {
            "og.Dialogue" => {
                let blob = self.blob::<format::Dialogue>(id, bytes)?;
                let meta = one::<format::DialogueMeta>(&blob)?;
                (blob.raw("data")?, (meta.data_bytes, blob))
            }
            _ => {
                let blob = self.blob::<format::Quest>(id, bytes)?;
                let meta = one::<format::QuestMeta>(&blob)?;
                (blob.raw("data")?, (meta.data_bytes, blob))
            }
        };
        if data.len() != blob.0 as usize {
            return Err(invalid(
                &blob.1,
                "native story data length does not match metadata",
            ));
        }
        Ok(data.to_vec())
    }

    /// Native reflected rows and stable identities, never author source text.
    pub fn table(&self, id: &str, bytes: &[u8]) -> Result<og_core::table::Table, LoadError> {
        let blob = self.blob::<format::Table>(id, bytes)?;
        let meta = one::<format::TableMeta>(&blob)?;
        let schema = og_core::Schema::from_json(
            std::str::from_utf8(blob.raw("schema")?).map_err(|e| invalid(&blob, e.to_string()))?,
        )
        .map_err(|e| invalid(&blob, e.to_string()))?;
        #[derive(serde::Deserialize)]
        struct Info(
            String,
            Vec<String>,
            BTreeMap<String, String>,
            BTreeMap<usize, serde_json::Map<String, serde_json::Value>>,
        );
        let Info(locale, ids, strings, bindings) =
            serde_json::from_slice(blob.raw("info")?).map_err(|e| invalid(&blob, e.to_string()))?;
        let (bytes, stride) = blob.rows("rows", schema.layout_hash())?;
        let size = og_core::Layout::of(&schema).size();
        if stride != size.max(1)
            || ids.len() != meta.row_count as usize
            || bytes.len() / stride != ids.len()
            || locale.is_empty()
        {
            return Err(invalid(
                &blob,
                "table row count, stride or locale is invalid",
            ));
        }
        let mut seen = std::collections::BTreeSet::new();
        let rows = ids
            .into_iter()
            .zip(bytes.chunks_exact(stride))
            .map(|(id, bytes)| {
                if id.is_empty() || !seen.insert(id.clone()) {
                    return Err(invalid(&blob, "empty or duplicate table row ID"));
                }
                schema
                    .check_packed(&bytes[..size])
                    .map_err(|e| invalid(&blob, e.to_string()))?;
                Ok(og_core::table::Row {
                    id,
                    bytes: bytes[..size].to_vec(),
                })
            })
            .collect::<Result<Vec<_>, LoadError>>()?;
        for (row, fields) in &bindings {
            if *row >= rows.len() || fields.keys().any(|f| schema.field_index(f).is_none()) {
                return Err(invalid(&blob, "table relocation is out of range"));
            }
        }
        Ok(og_core::table::Table {
            schema,
            locale,
            rows,
            strings,
            bindings,
        })
    }

    /// The look of a look blob (`og.Look`): flattened and checked by the
    /// cook; the assets it names are the caller's to load.
    pub fn look(&self, id: &str, bytes: &[u8]) -> Result<crate::look::LookFile, LoadError> {
        let blob = self.blob::<format::Look>(id, bytes)?;
        let meta = one::<format::LookMeta>(&blob)?;
        let text = blob
            .raw("text")?
            .get(..meta.text_bytes as usize)
            .and_then(|t| std::str::from_utf8(t).ok())
            .ok_or_else(|| invalid(&blob, "the look text is cut short or not UTF-8"))?;
        crate::look::LookFile::parse(text).map_err(|e| invalid(&blob, e))
    }

    /// The colours of a palette blob (`og.Palette`): sRGB, in file order.
    pub fn palette(&self, id: &str, bytes: &[u8]) -> Result<Vec<[u8; 3]>, LoadError> {
        let blob = self.blob::<format::Palette>(id, bytes)?;
        let meta = one::<format::PaletteMeta>(&blob)?;
        let rgb = blob.raw("rgb")?;
        let n = meta.count as usize;
        if n == 0 || n > 256 || rgb.len() < n * 3 {
            return Err(invalid(
                &blob,
                format!("a palette of {n} colours in {} bytes", rgb.len()),
            ));
        }
        Ok(rgb[..n * 3].as_chunks::<3>().0.to_vec())
    }

    /// The authored reaction and its head-local hand contact.
    pub fn motion_action(
        &self,
        id: &str,
        bytes: &[u8],
    ) -> Result<crate::motion_action::MotionAction, LoadError> {
        let blob = self.blob::<format::MotionAction>(id, bytes)?;
        let meta = one::<format::MotionActionMeta>(&blob)?;
        let strings = blob.raw("strings")?;
        let text = |offset: u32, len: u32| {
            (offset as usize)
                .checked_add(len as usize)
                .and_then(|end| strings.get(offset as usize..end))
                .and_then(|t| std::str::from_utf8(t).ok())
                .map(str::to_string)
                .ok_or_else(|| invalid(&blob, "motion-action string is out of range or not UTF-8"))
        };
        let action = crate::motion_action::MotionAction {
            source: text(meta.source_offset, meta.source_len)?,
            clip: text(meta.clip_offset, meta.clip_len)?,
            rig: text(meta.rig_offset, meta.rig_len)?,
            hold_start: meta.hold_start,
            hold_end: meta.hold_end,
            hand: match meta.hand {
                0 => "",
                1 => "left",
                2 => "right",
                _ => return Err(invalid(&blob, "invalid motion-action hand")),
            }
            .into(),
            hand_target: [meta.target_x, meta.target_y, meta.target_z],
        };
        action.validate().map_err(|e| invalid(&blob, e))?;
        Ok(action)
    }

    /// Decodes a lightmap blob (`og.Lightmap`): native bindings and its
    /// pages (RGBA f16 bits per page).
    pub fn lightmap(&self, id: &str, bytes: &[u8]) -> Result<CookedLightmap, LoadError> {
        let blob = self.blob::<format::Lightmap>(id, bytes)?;
        crate::lightmap::cooked::decode(&blob)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_lightmap(&self, id: &str) -> Result<CookedLightmap, LoadError> {
        self.lightmap(id, &self.read(id)?)
    }

    /// Decodes a scene's HLOD tree (`og.Hlod`, `<scene>#hlod`).
    pub fn hlod(&self, id: &str, bytes: &[u8]) -> Result<crate::hlod::Hlod, LoadError> {
        let blob = self.blob::<format::Hlod>(id, bytes)?;
        let meta = one::<format::HlodMeta>(&blob)?;
        let nodes = table::<format::HlodNodeRow>(&blob, "nodes")?;
        let members = table::<format::HlodMemberRow>(&blob, "members")?;
        let cells = table::<format::HlodCellRow>(&blob, "cells")?;
        crate::hlod::Hlod::from_rows(&meta, &nodes, &members, &cells).map_err(|e| invalid(&blob, e))
    }

    /// The asset bytes of a fracture blob (`og.Fracture`, a model's
    /// `#fracture`), for `og_physics_data::fracture::FractureAsset::from_bytes`.
    pub fn fracture(&self, id: &str, bytes: &[u8]) -> Result<Vec<u8>, LoadError> {
        let blob = self.blob::<format::Fracture>(id, bytes)?;
        let meta = one::<format::FractureMeta>(&blob)?;
        Ok(blob
            .raw("asset")?
            .get(..meta.asset_bytes as usize)
            .ok_or_else(|| invalid(&blob, "the fracture asset is cut short"))?
            .to_vec())
    }

    /// Decodes a timeline blob (`og.TimelineAsset`): the canonical text
    /// the cook wrote, parsed by og-timeline.
    pub fn timeline(&self, id: &str, bytes: &[u8]) -> Result<og_timeline::Timeline, LoadError> {
        let blob = self.blob::<format::Timeline>(id, bytes)?;
        let meta = one::<format::TimelineMeta>(&blob)?;
        let text = blob
            .raw("text")?
            .get(..meta.text_bytes as usize)
            .and_then(|t| std::str::from_utf8(t).ok())
            .ok_or_else(|| invalid(&blob, "the timeline text is cut short or not UTF-8"))?;
        og_timeline::Timeline::parse(text).map_err(|e| invalid(&blob, e))
    }

    /// A navmesh blob's agent profiles (G2-4: the `profiles` section, the
    /// names one per line, the blob's own tables' first): empty for a
    /// navmesh of one unnamed agent.
    pub fn navmesh_profiles(&self, id: &str, bytes: &[u8]) -> Result<Vec<String>, LoadError> {
        let blob = self.blob::<format::NavMesh>(id, bytes)?;
        let Some(list) = blob.raw_opt("profiles") else {
            return Ok(Vec::new());
        };
        let text = std::str::from_utf8(list)
            .map_err(|_| invalid(&blob, "the profiles section is not UTF-8"))?;
        Ok(text.split('\n').map(str::to_string).collect())
    }

    /// The navmesh blob of agent profile `profile` inside `bytes` (the
    /// blob itself for its first profile, or `""`), which
    /// og-ai's `nav::cooked::BundleNav` loaders (`navmesh`, `navmesh_areas`,
    /// `navmesh_links`)
    /// read like any other; `None` for a profile it does not have.
    pub fn navmesh_profile<'b>(
        &self,
        id: &str,
        bytes: &'b [u8],
        profile: &str,
    ) -> Result<Option<&'b [u8]>, LoadError> {
        let names = self.navmesh_profiles(id, bytes)?;
        if profile.is_empty() || names.first().is_some_and(|n| n == profile) {
            return Ok(Some(bytes));
        }
        if !names.iter().any(|n| n == profile) {
            return Ok(None);
        }
        let blob = self.blob::<format::NavMesh>(id, bytes)?;
        Ok(Some(blob.raw(&format!("profile.{profile}"))?))
    }
}

// ── E6: string tables ────────────────────────────────────────────────────

impl Bundle {
    /// The dictionaries a font pack blob (`og.FontPack`) ships for the
    /// game's locales, by name, of one `kind`: `hyphenation` (by language,
    /// when the game declares `hyphens: auto`) or `segmentation` (ICU's
    /// word dictionaries, by marker attribute such as `cjdict`).
    pub fn font_pack_dictionaries(
        &self,
        id: &str,
        bytes: &[u8],
        kind: &str,
    ) -> Result<Vec<(String, Vec<u8>)>, LoadError> {
        let blob = self.blob::<format::FontPack>(id, bytes)?;
        let prefix = format!("{kind}:");
        Ok(blob
            .sections()
            .iter()
            .filter_map(|s| {
                let language = s.name.strip_prefix(prefix.as_str())?;
                let bytes = blob.raw(s.name).ok()?;
                Some((language.to_string(), bytes.to_vec()))
            })
            .collect())
    }
}

// ── phase 3: streaming groups ────────────────────────────────────────────

/// The files of a streaming group's pack (`cooked/stream/<group>.ogb`):
/// each path under `cooked/` with its bytes, in the pack's order.
pub fn unpack(bytes: &[u8]) -> Result<Vec<(String, Vec<u8>)>, LoadError> {
    let words = og_blob::AlignedBytes::from_slice(bytes);
    let expect = Expect {
        type_hash: format::StreamPack::type_hash(),
        schema_version: format::StreamPack::schema_version(),
        stamp: None,
    };
    let blob = Blob::open(words.as_slice(), "stream pack", &expect)?;
    let files = table::<format::StreamFileRow>(&blob, "files")?;
    let names = blob.raw("names")?;
    let data = blob.raw("data")?;
    let mut out = Vec::with_capacity(files.len());
    for f in &files {
        let name = names
            .get(f.name_at as usize..(f.name_at + f.name_len) as usize)
            .and_then(|b| std::str::from_utf8(b).ok())
            .ok_or_else(|| invalid(&blob, "a file's name is out of range"))?;
        let bytes = data
            .get(f.data_at as usize..f.data_at as usize + f.data_len as usize)
            .ok_or_else(|| invalid(&blob, "a file's bytes are out of range"))?;
        out.push((name.to_string(), bytes.to_vec()));
    }
    Ok(out)
}
