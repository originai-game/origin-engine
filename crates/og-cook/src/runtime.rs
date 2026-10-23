//! The runtime loaders: `cooked/` blobs into the renderer's input types and
//! plain scene data. Nothing here touches the ECS or the GPU; the host spawns
//! what [`SceneDesc`] describes and adds what [`CookedModel::add_to`] returns.
//!
//! A [`Bundle`] is the cook manifest (`cooked/.cook.ogt`) read as an index:
//! asset id to output file, type and cook stamp. Every load names an asset id
//! (the source path under `assets/`, `assets/` prefix optional; a glTF's
//! textures are `<model>#tex<n>`), finds the output in the manifest and
//! refuses a blob whose type, schema version or cook stamp disagrees with it
//! (`docs/design.md` §6). Bytes come from the host (`fetch` on the web); on
//! native targets [`Bundle::open`] and the `load_*` methods read files.

use crate::format::{self, *};
use crate::{ktx2, meshopt_codec, qoa};
use glam::{Mat4, Quat, Vec3};
use og_blob::{Blob, BlobError, Expect};
use og_core::ogt::{Block, Document};
use og_render::{
    AlphaMode, ColorSpace, Material, MaterialHandle, MeshData, MeshHandle, Mips,
    PhysicalExtensions, RenderScene, StandardMaterial, TextureData, TextureDesc, TextureFormat,
    TextureFormats, TextureHandle, TextureSource, TextureUpload, Vertex,
};
use std::collections::BTreeMap;
use std::fmt;

/// The manifest's file name inside `cooked/`.
pub const MANIFEST: &str = ".cook.ogt";

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
    pub outputs: Vec<ManifestOutput>,
}

/// `cooked/.cook.ogt`: what was cooked from what.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Manifest {
    pub cooker: String,
    /// Per asset kind, the option text its stamps cover.
    pub options: BTreeMap<String, String>,
    /// Sorted by source path.
    pub assets: Vec<ManifestAsset>,
    /// The streaming groups, by name.
    pub streams: Vec<ManifestStream>,
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
            ..Default::default()
        };
        if let Some(block) = doc.body.get("options").and_then(|e| e.block.as_ref()) {
            for e in &block.entries {
                m.options.insert(
                    e.key.clone(),
                    e.string(0).map_err(|e| bad(e.to_string()))?.to_string(),
                );
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
        let mut options = Block::new();
        for (k, v) in &self.options {
            options.push(Entry::new(k.clone()).with(s(v)));
        }
        doc.body.push(Entry::new("options").with_block(options));
        for a in &self.assets {
            let mut b = Block::new()
                .with(Entry::new("kind").with(w(&a.kind)))
                .with(Entry::new("source").with(s(&hex(a.source_hash))))
                .with(Entry::new("bytes").with(n(a.source_bytes)));
            for o in &a.outputs {
                let ob = Block::new()
                    .with(Entry::new("type").with(w(&o.ty)))
                    .with(Entry::new("file").with(s(&o.file)))
                    .with(Entry::new("stamp").with(s(&hex(o.stamp))))
                    .with(Entry::new("hash").with(s(&hex(o.hash))))
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

/// The cooked bundle a game loads from.
#[derive(Clone, Debug)]
pub struct Bundle {
    manifest: Manifest,
    index: BTreeMap<String, usize>,
    outputs: Vec<ManifestOutput>,
    #[cfg(not(target_arch = "wasm32"))]
    dir: Option<std::path::PathBuf>,
}

impl Bundle {
    /// A bundle from the manifest text (the host fetched `cooked/.cook.ogt`).
    pub fn from_manifest(text: &str) -> Result<Bundle, LoadError> {
        let manifest = Manifest::parse(text)?;
        let outputs: Vec<ManifestOutput> = manifest.outputs().cloned().collect();
        let index = outputs
            .iter()
            .enumerate()
            .map(|(i, o)| (o.id.clone(), i))
            .collect();
        Ok(Bundle {
            manifest,
            index,
            outputs,
            #[cfg(not(target_arch = "wasm32"))]
            dir: None,
        })
    }

    /// Opens `cooked/` on disk (native targets).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open(cooked: &std::path::Path) -> Result<Bundle, LoadError> {
        let path = cooked.join(MANIFEST);
        let text = std::fs::read_to_string(&path).map_err(|e| LoadError::Io {
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
        Ok(CookedTexture {
            id: asset_id(id).to_string(),
            meta,
            ktx2,
        })
    }

    /// Decodes a colour LUT blob.
    pub fn lut(&self, id: &str, bytes: &[u8]) -> Result<og_render::ColorLut, LoadError> {
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
        Ok(og_render::ColorLut {
            size: meta.size,
            rgba,
        })
    }

    /// The validated text of a particle emitter blob (parse it with
    /// [`crate::fx::parse_emitter`] and the bundle's textures).
    pub fn emitter_text(&self, id: &str, bytes: &[u8]) -> Result<String, LoadError> {
        let blob = self.blob::<format::Emitter>(id, bytes)?;
        let meta = one::<format::EmitterMeta>(&blob)?;
        let text = blob.raw("text")?;
        if text.len() < meta.text_bytes as usize {
            return Err(invalid(&blob, "the emitter text is cut short"));
        }
        String::from_utf8(text[..meta.text_bytes as usize].to_vec())
            .map_err(|_| invalid(&blob, "the emitter text is not UTF-8"))
    }

    /// The material shader of a shader blob.
    pub fn shader(&self, id: &str, bytes: &[u8]) -> Result<og_render::MaterialShader, LoadError> {
        let blob = self.blob::<format::Shader>(id, bytes)?;
        let meta = one::<format::ShaderMeta>(&blob)?;
        let text = blob.raw("text")?;
        if text.len() < meta.text_bytes as usize {
            return Err(invalid(&blob, "the shader text is cut short"));
        }
        let text = std::str::from_utf8(&text[..meta.text_bytes as usize])
            .map_err(|_| invalid(&blob, "the shader text is not UTF-8"))?;
        crate::shader::parse_shader(id, text, &mut |_| {
            Err("a cooked shader carries its code".to_string())
        })
        .map_err(|e| invalid(&blob, &e))
    }

    /// The custom post pass of a post blob.
    pub fn post(&self, id: &str, bytes: &[u8]) -> Result<og_render::PostShader, LoadError> {
        let blob = self.blob::<format::Post>(id, bytes)?;
        let meta = one::<format::PostMeta>(&blob)?;
        let text = blob.raw("text")?;
        if text.len() < meta.text_bytes as usize {
            return Err(invalid(&blob, "the post text is cut short"));
        }
        let text = std::str::from_utf8(&text[..meta.text_bytes as usize])
            .map_err(|_| invalid(&blob, "the post text is not UTF-8"))?;
        crate::shader::parse_post(id, text, &mut |_| {
            Err("a cooked post pass carries its code".to_string())
        })
        .map_err(|e| invalid(&blob, &e))
    }

    /// The validated text of a UI style blob (parse it with og-ui's
    /// `parse_style` and load the assets it names from the bundle).
    pub fn style_text(&self, id: &str, bytes: &[u8]) -> Result<String, LoadError> {
        let blob = self.blob::<format::Style>(id, bytes)?;
        let meta = one::<format::StyleMeta>(&blob)?;
        let text = blob.raw("text")?;
        if text.len() < meta.text_bytes as usize {
            return Err(invalid(&blob, "the style text is cut short"));
        }
        String::from_utf8(text[..meta.text_bytes as usize].to_vec())
            .map_err(|_| invalid(&blob, "the style text is not UTF-8"))
    }

    /// Decodes a sound blob (the QOA stays encoded until [`CookedSound::decode`]).
    pub fn sound(&self, id: &str, bytes: &[u8]) -> Result<CookedSound, LoadError> {
        let blob = self.blob::<format::Sound>(id, bytes)?;
        Ok(CookedSound {
            id: asset_id(id).to_string(),
            meta: one::<SoundMeta>(&blob)?,
            qoa: blob.raw("qoa")?.to_vec(),
        })
    }

    /// Reads a video blob's meta row (its frames are the texture array
    /// `<id>#frames`, its sound `<id>#audio`).
    pub fn video(&self, id: &str, bytes: &[u8]) -> Result<format::VideoMeta, LoadError> {
        let blob = self.blob::<format::Video>(id, bytes)?;
        let meta = one::<format::VideoMeta>(&blob)?;
        if meta.frames == 0 || meta.fps == 0 {
            return Err(invalid(&blob, "a video with no frames or no rate"));
        }
        Ok(meta)
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
    /// [`chunk_id`]): its index and its QOA frames.
    pub fn sound_chunk(&self, id: &str, bytes: &[u8]) -> Result<(u32, Vec<u8>), LoadError> {
        let blob = self.blob::<format::SoundChunk>(id, bytes)?;
        let meta = one::<format::SoundChunkMeta>(&blob)?;
        Ok((meta.chunk, blob.raw("qoa")?.to_vec()))
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
            ttf: blob.raw("ttf").ok().map(<[u8]>::to_vec),
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
        })
    }

    /// Decodes a scene blob.
    pub fn scene(&self, id: &str, bytes: &[u8]) -> Result<SceneDesc, LoadError> {
        let blob = self.blob::<format::Scene>(id, bytes)?;
        SceneDesc::decode(asset_id(id), &blob)
    }

    /// Decodes an animation blob (`<model>#anim`): the skeleton, the clips
    /// and the per-vertex skin and morph data, as og-anim's assets.
    pub fn animation(&self, id: &str, bytes: &[u8]) -> Result<CookedAnimation, LoadError> {
        let blob = self.blob::<format::Animation>(id, bytes)?;
        CookedAnimation::decode(asset_id(id), &blob)
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

    /// Reads and decodes a glTF's animation blob (native); `id` is the model's
    /// id with or without `#anim`.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_animation(&self, id: &str) -> Result<CookedAnimation, LoadError> {
        let id = anim_id(id);
        self.animation(&id, &self.read(&id)?)
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

fn invalid(blob: &Blob, reason: impl Into<String>) -> LoadError {
    LoadError::Invalid {
        path: blob.path().to_string(),
        reason: reason.into(),
    }
}

fn one<T: og_core::Reflect + bytemuck::Pod>(blob: &Blob) -> Result<T, LoadError> {
    let rows = blob.table::<T>("meta", row_hash::<T>())?;
    rows.first()
        .copied()
        .ok_or_else(|| invalid(blob, "empty meta section"))
}

fn table<T: og_core::Reflect + bytemuck::Pod>(
    blob: &Blob,
    name: &str,
) -> Result<Vec<T>, LoadError> {
    Ok(blob.table::<T>(name, row_hash::<T>())?.into_owned())
}

fn opt(i: u32) -> Option<usize> {
    (i != NONE).then_some(i as usize)
}

/// One level of detail of a [`CookedPrimitive`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lod {
    pub first: usize,
    pub count: usize,
    /// Simplification error relative to the primitive's extent (0 for LOD 0).
    pub error: f32,
}

/// A decoded mesh primitive: every LOD shares the vertices.
#[derive(Clone, Debug, PartialEq)]
pub struct CookedPrimitive {
    pub material: usize,
    pub vertices: Vec<Vertex>,
    /// Per vertex `xyz` tangent and bitangent sign `w`; empty when not cooked.
    pub tangents: Vec<[f32; 4]>,
    /// All LODs' triangles, LOD 0 first.
    pub indices: Vec<u32>,
    pub lods: Vec<Lod>,
    /// Exact local bounds of the source positions.
    pub min: Vec3,
    pub max: Vec3,
    /// The raycast BVH over LOD 0 the cook built (`None`: cooked with
    /// `raycast false`, or a blob of a cook before it): put together over
    /// [`CookedPrimitive::mesh`]`(0)` by [`CookedPrimitive::bvh`].
    pub bvh_parts: Option<(Vec<og_render::geometry::BvhNode>, Vec<u32>)>,
}

impl CookedPrimitive {
    /// The cooked raycast BVH over `mesh` (LOD 0's, as [`Self::mesh`]
    /// answers it), without building anything; `None` when none was cooked.
    pub fn bvh(&self, mesh: &MeshData) -> Option<Result<og_render::geometry::Bvh, String>> {
        let (nodes, tris) = self.bvh_parts.as_ref()?;
        Some(og_render::geometry::Bvh::from_parts(
            mesh,
            nodes.clone(),
            tris.clone(),
        ))
    }

    /// The geometry of one LOD (0 is the full mesh) for [`RenderScene::add_mesh`].
    pub fn mesh(&self, lod: usize) -> MeshData {
        let l = self.lods[lod.min(self.lods.len() - 1)];
        MeshData {
            vertices: self.vertices.clone(),
            indices: self.indices[l.first..l.first + l.count].to_vec(),
        }
    }

    pub fn triangle_count(&self, lod: usize) -> usize {
        self.lods[lod.min(self.lods.len() - 1)].count / 3
    }

    /// The geometry of one LOD with only the vertices it uses (a coarse
    /// level's own mesh: [`CookedModel::add_lod_levels`]).
    pub fn mesh_compact(&self, lod: usize) -> MeshData {
        let l = self.lods[lod.min(self.lods.len() - 1)];
        let mut at = vec![u32::MAX; self.vertices.len()];
        let mut vertices = Vec::new();
        let indices = self.indices[l.first..l.first + l.count]
            .iter()
            .map(|&i| {
                if at[i as usize] == u32::MAX {
                    at[i as usize] = vertices.len() as u32;
                    vertices.push(self.vertices[i as usize]);
                }
                at[i as usize]
            })
            .collect();
        MeshData { vertices, indices }
    }
}

/// A glTF mesh: a range of [`CookedModel::primitives`].
#[derive(Clone, Debug, PartialEq)]
pub struct CookedMesh {
    pub name: String,
    pub primitives: std::ops::Range<usize>,
}

/// A material with its textures as indices into [`CookedModel::textures`].
#[derive(Clone, Debug, PartialEq)]
pub struct CookedMaterial {
    pub name: String,
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub normal_scale: f32,
    pub occlusion_strength: f32,
    pub emissive: [f32; 3],
    pub alpha_mode: u32,
    pub alpha_cutoff: f32,
    pub double_sided: bool,
    pub base_color_texture: Option<usize>,
    pub metallic_roughness_texture: Option<usize>,
    pub normal_texture: Option<usize>,
    pub emissive_texture: Option<usize>,
    pub occlusion_texture: Option<usize>,
    /// The `KHR_materials_*` physical extensions (`present` 0 without any).
    pub physical: PhysicalExtensions<usize>,
}

/// A texture a model uses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextureRef {
    pub id: String,
    pub kind: u32,
}

/// A node of the model's scene graph.
#[derive(Clone, Debug, PartialEq)]
pub struct CookedNode {
    pub name: String,
    pub parent: Option<usize>,
    pub mesh: Option<usize>,
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl CookedNode {
    pub fn local(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

/// A glTF scene: its root nodes.
#[derive(Clone, Debug, PartialEq)]
pub struct CookedScene {
    pub name: String,
    pub roots: Vec<usize>,
}

/// A decoded model (one glTF file).
#[derive(Clone, Debug, PartialEq)]
pub struct CookedModel {
    pub id: String,
    /// In glTF mesh-primitive order (`GltfInfo.geometries`).
    pub primitives: Vec<CookedPrimitive>,
    pub meshes: Vec<CookedMesh>,
    /// In glTF order, then the default material.
    pub materials: Vec<CookedMaterial>,
    pub textures: Vec<TextureRef>,
    /// Parents precede children.
    pub nodes: Vec<CookedNode>,
    pub scenes: Vec<CookedScene>,
    pub default_scene: Option<usize>,
    /// Bounds of the default scene with node transforms applied.
    pub min: Vec3,
    pub max: Vec3,
    pub skipped_animations: u32,
    pub skipped_skins: u32,
}

/// What [`CookedModel::add_to`] added: per part the mesh, the material and
/// the node's transform within the model.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelParts {
    pub parts: Vec<(MeshHandle, MaterialHandle, Mat4)>,
}

impl ModelParts {
    /// Submits one instance of the whole model at `transform`.
    pub fn instance(&self, scene: &mut RenderScene, transform: Mat4) {
        for &(mesh, material, local) in &self.parts {
            scene.add_instance(mesh, material, transform * local);
        }
    }
}

impl CookedModel {
    fn decode(id: &str, blob: &Blob) -> Result<CookedModel, LoadError> {
        let meta = one::<ModelMeta>(blob)?;
        let prims = table::<PrimitiveRow>(blob, "primitives")?;
        let lods = table::<LodRow>(blob, "lods")?;
        let meshes = table::<MeshRow>(blob, "meshes")?;
        let materials = table::<MaterialRow>(blob, "materials")?;
        let textures = table::<TextureRefRow>(blob, "textures")?;
        let nodes = table::<NodeRow>(blob, "nodes")?;
        let scenes = table::<SceneRow>(blob, "scenes")?;
        let roots = blob.table::<u32>("roots", 0)?;
        let strings = blob.raw("strings")?;
        let vertices = blob.raw("vertices")?;
        let indices = blob.raw("indices")?;
        let bad = |r: String| invalid(blob, r);
        // The raycast BVHs: zlib-compressed rows, as many as the
        // primitives' ranges reach.
        let inflate = |name: &str, len: usize| -> Result<Vec<u8>, LoadError> {
            let z = blob.raw(name)?;
            let out = miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(z, len)
                .map_err(|e| bad(format!("{name}: {e:?}")))?;
            if out.len() != len {
                return Err(bad(format!("{name}: {} bytes, expected {len}", out.len())));
            }
            Ok(out)
        };
        let node_rows = prims
            .iter()
            .map(|p| (p.bvh_node_first + p.bvh_node_count) as usize)
            .max()
            .unwrap_or(0);
        let tri_rows = prims
            .iter()
            .map(|p| (p.bvh_tri_first + p.bvh_tri_count) as usize)
            .max()
            .unwrap_or(0);
        let node_bytes = inflate("bvh_nodes", node_rows * std::mem::size_of::<BvhNodeRow>())?;
        let tri_bytes = inflate("bvh_tris", tri_rows * 4)?;
        let bvh_nodes: Vec<BvhNodeRow> = bytemuck::pod_collect_to_vec(&node_bytes);
        let bvh_tris: Vec<u32> = bytemuck::pod_collect_to_vec(&tri_bytes);
        let s = |o: u32, l: u32| {
            pool_str(strings, o, l)
                .map(str::to_string)
                .ok_or_else(|| bad("string out of range".into()))
        };
        let range = |data: &'_ [u8],
                     o: u32,
                     l: u32,
                     what: &str|
         -> Result<std::ops::Range<usize>, LoadError> {
            let (o, l) = (o as usize, l as usize);
            (o + l <= data.len())
                .then_some(o..o + l)
                .ok_or_else(|| bad(format!("{what} out of range")))
        };
        let mut primitives = Vec::with_capacity(prims.len());
        for (i, p) in prims.iter().enumerate() {
            let vb = &vertices[range(vertices, p.vertex_offset, p.vertex_size, "vertex stream")?];
            let ib = &indices[range(indices, p.index_offset, p.index_size, "index stream")?];
            let raw = meshopt_codec::decode_vertex_buffer(
                vb,
                p.vertex_count as usize,
                p.vertex_stride as usize,
            )
            .map_err(|e| bad(format!("primitive {i}: {e}")))?;
            let idx = meshopt_codec::decode_index_buffer(ib, p.index_count as usize)
                .map_err(|e| bad(format!("primitive {i}: {e}")))?;
            if idx.iter().any(|&v| v >= p.vertex_count) {
                return Err(bad(format!("primitive {i}: index out of range")));
            }
            let (verts, tangents) = dequantize(p, &raw);
            let lod_rows = lods
                .get(p.lod_first as usize..(p.lod_first + p.lod_count) as usize)
                .ok_or_else(|| bad(format!("primitive {i}: LOD range out of range")))?;
            let mut prim_lods = Vec::new();
            for l in lod_rows {
                if (l.index_first + l.index_count) as usize > idx.len() || l.index_count % 3 != 0 {
                    return Err(bad(format!("primitive {i}: LOD indices out of range")));
                }
                prim_lods.push(Lod {
                    first: l.index_first as usize,
                    count: l.index_count as usize,
                    error: l.error,
                });
            }
            if prim_lods.is_empty() {
                return Err(bad(format!("primitive {i} has no LOD")));
            }
            if p.material as usize >= materials.len() {
                return Err(bad(format!("primitive {i}: material out of range")));
            }
            let min = Vec3::new(p.pos_offset_x, p.pos_offset_y, p.pos_offset_z);
            let bvh_parts = if p.bvh_node_count == 0 {
                None
            } else {
                let nodes = bvh_nodes
                    .get(p.bvh_node_first as usize..(p.bvh_node_first + p.bvh_node_count) as usize)
                    .ok_or_else(|| bad(format!("primitive {i}: BVH nodes out of range")))?;
                let tris = bvh_tris
                    .get(p.bvh_tri_first as usize..(p.bvh_tri_first + p.bvh_tri_count) as usize)
                    .ok_or_else(|| bad(format!("primitive {i}: BVH triangles out of range")))?;
                let off = [p.pos_offset_x, p.pos_offset_y, p.pos_offset_z];
                let scale = [p.pos_scale_x, p.pos_scale_y, p.pos_scale_z];
                // As `dequantize`: the same grid, the same arithmetic.
                let deq =
                    |a: usize, q: u32| off[a] + (q & 0xffff) as u16 as f32 / 65535.0 * scale[a];
                let nodes = nodes
                    .iter()
                    .map(|n| og_render::geometry::BvhNode {
                        min: Vec3::new(
                            deq(0, n.min_xy),
                            deq(1, n.min_xy >> 16),
                            deq(2, n.min_z_max_x),
                        ),
                        max: Vec3::new(
                            deq(0, n.min_z_max_x >> 16),
                            deq(1, n.max_yz),
                            deq(2, n.max_yz >> 16),
                        ),
                        first: n.first,
                        count: n.count,
                    })
                    .collect();
                Some((nodes, tris.to_vec()))
            };
            primitives.push(CookedPrimitive {
                bvh_parts,
                material: p.material as usize,
                vertices: verts,
                tangents,
                indices: idx,
                lods: prim_lods,
                min,
                max: min + Vec3::new(p.pos_scale_x, p.pos_scale_y, p.pos_scale_z),
            });
        }
        let tex = |i: u32| -> Result<Option<usize>, LoadError> {
            match opt(i) {
                Some(t) if t >= textures.len() => Err(bad("texture index out of range".into())),
                t => Ok(t),
            }
        };
        let mut mats = Vec::with_capacity(materials.len());
        for m in &materials {
            mats.push(CookedMaterial {
                name: s(m.name_offset, m.name_len)?,
                base_color: [m.base_r, m.base_g, m.base_b, m.base_a],
                metallic: m.metallic,
                roughness: m.roughness,
                normal_scale: m.normal_scale,
                occlusion_strength: m.occlusion_strength,
                emissive: [m.emissive_r, m.emissive_g, m.emissive_b],
                alpha_mode: m.alpha_mode,
                alpha_cutoff: m.alpha_cutoff,
                double_sided: m.double_sided != 0,
                base_color_texture: tex(m.base_color_texture)?,
                metallic_roughness_texture: tex(m.metallic_roughness_texture)?,
                normal_texture: tex(m.normal_texture)?,
                emissive_texture: tex(m.emissive_texture)?,
                occlusion_texture: tex(m.occlusion_texture)?,
                physical: PhysicalExtensions {
                    present: m.physical,
                    clearcoat: m.clearcoat,
                    clearcoat_texture: tex(m.clearcoat_texture)?,
                    clearcoat_roughness: m.clearcoat_roughness,
                    clearcoat_roughness_texture: tex(m.clearcoat_roughness_texture)?,
                    clearcoat_normal_texture: tex(m.clearcoat_normal_texture)?,
                    clearcoat_normal_scale: m.clearcoat_normal_scale,
                    sheen_color: [m.sheen_r, m.sheen_g, m.sheen_b],
                    sheen_color_texture: tex(m.sheen_color_texture)?,
                    sheen_roughness: m.sheen_roughness,
                    sheen_roughness_texture: tex(m.sheen_roughness_texture)?,
                    transmission: m.transmission,
                    transmission_texture: tex(m.transmission_texture)?,
                    thickness: m.thickness,
                    thickness_texture: tex(m.thickness_texture)?,
                    attenuation_distance: m.attenuation_distance,
                    attenuation_color: [m.attenuation_r, m.attenuation_g, m.attenuation_b],
                    ior: m.ior,
                    iridescence: m.iridescence,
                    iridescence_texture: tex(m.iridescence_texture)?,
                    iridescence_ior: m.iridescence_ior,
                    iridescence_thickness: [m.iridescence_min, m.iridescence_max],
                    iridescence_thickness_texture: tex(m.iridescence_thickness_texture)?,
                    anisotropy: m.anisotropy,
                    anisotropy_rotation: m.anisotropy_rotation,
                    anisotropy_texture: tex(m.anisotropy_texture)?,
                },
            });
        }
        let mut model_nodes = Vec::with_capacity(nodes.len());
        for (i, n) in nodes.iter().enumerate() {
            let parent = opt(n.parent);
            if parent.is_some_and(|p| p >= i) {
                return Err(bad(format!("node {i}: parent does not precede it")));
            }
            let mesh = opt(n.mesh);
            if mesh.is_some_and(|m| m >= meshes.len()) {
                return Err(bad(format!("node {i}: mesh out of range")));
            }
            model_nodes.push(CookedNode {
                name: s(n.name_offset, n.name_len)?,
                parent,
                mesh,
                translation: Vec3::new(n.tx, n.ty, n.tz),
                rotation: Quat::from_xyzw(n.rx, n.ry, n.rz, n.rw),
                scale: Vec3::new(n.sx, n.sy, n.sz),
            });
        }
        let mut model_meshes = Vec::with_capacity(meshes.len());
        for m in &meshes {
            let r = m.primitive_first as usize..(m.primitive_first + m.primitive_count) as usize;
            if r.end > primitives.len() {
                return Err(bad("mesh primitive range out of range".into()));
            }
            model_meshes.push(CookedMesh {
                name: s(m.name_offset, m.name_len)?,
                primitives: r,
            });
        }
        let mut model_scenes = Vec::with_capacity(scenes.len());
        for sc in &scenes {
            let r = sc.root_first as usize..(sc.root_first + sc.root_count) as usize;
            let rs = roots
                .get(r)
                .ok_or_else(|| bad("scene roots out of range".into()))?;
            if rs.iter().any(|&n| n as usize >= model_nodes.len()) {
                return Err(bad("scene root out of range".into()));
            }
            model_scenes.push(CookedScene {
                name: s(sc.name_offset, sc.name_len)?,
                roots: rs.iter().map(|&n| n as usize).collect(),
            });
        }
        let mut texture_refs = Vec::with_capacity(textures.len());
        for t in &textures {
            texture_refs.push(TextureRef {
                id: s(t.id_offset, t.id_len)?,
                kind: t.kind,
            });
        }
        Ok(CookedModel {
            id: id.to_string(),
            primitives,
            meshes: model_meshes,
            materials: mats,
            textures: texture_refs,
            nodes: model_nodes,
            scenes: model_scenes,
            default_scene: opt(meta.default_scene),
            min: Vec3::new(meta.min_x, meta.min_y, meta.min_z),
            max: Vec3::new(meta.max_x, meta.max_y, meta.max_z),
            skipped_animations: meta.skipped_animations,
            skipped_skins: meta.skipped_skins,
        })
    }

    /// Every node's model-space transform.
    pub fn node_world(&self) -> Vec<Mat4> {
        let mut world: Vec<Mat4> = Vec::with_capacity(self.nodes.len());
        for n in &self.nodes {
            let parent = n.parent.map_or(Mat4::IDENTITY, |p| world[p]);
            world.push(parent * n.local());
        }
        world
    }

    /// The nodes of `scene` (all descendants of its roots) in depth-first order.
    pub fn scene_nodes(&self, scene: usize) -> Vec<usize> {
        let mut children: Vec<Vec<usize>> = vec![Vec::new(); self.nodes.len()];
        for (i, n) in self.nodes.iter().enumerate() {
            if let Some(p) = n.parent {
                children[p].push(i);
            }
        }
        let mut out = Vec::new();
        let mut stack: Vec<usize> = self
            .scenes
            .get(scene)
            .map(|s| s.roots.iter().rev().copied().collect())
            .unwrap_or_default();
        while let Some(n) = stack.pop() {
            out.push(n);
            stack.extend(children[n].iter().rev());
        }
        out
    }

    /// Material `index` as the renderer draws it, sampling `textures` (the
    /// handles of [`CookedModel::textures`], in order; a missing one is
    /// drawn without its texture). A streamed model's full material over
    /// its proxy's.
    pub fn material(&self, index: usize, textures: &[TextureHandle]) -> Material {
        let m = &self.materials[index];
        let tex = |i: Option<usize>| i.and_then(|i| textures.get(i).copied());
        Material {
            base_color: m.base_color,
            base_color_texture: tex(m.base_color_texture),
            metallic: m.metallic,
            roughness: m.roughness,
            metallic_roughness_texture: tex(m.metallic_roughness_texture),
            normal_texture: tex(m.normal_texture),
            normal_scale: m.normal_scale,
            emissive: m.emissive,
            emissive_texture: tex(m.emissive_texture),
            alpha_mode: if m.alpha_mode == alpha::MASK {
                AlphaMode::Mask {
                    cutoff: m.alpha_cutoff,
                }
            } else {
                AlphaMode::Opaque
            },
        }
    }

    /// Material `index` with its physical extensions, as a standard
    /// material: what [`CookedModel::add_to`] adds for a material that has
    /// any (and the same as [`CookedModel::material`] for one without).
    pub fn standard_material(&self, index: usize, textures: &[TextureHandle]) -> StandardMaterial {
        let mut m = StandardMaterial::from(&self.material(index, textures));
        self.materials[index]
            .physical
            .map(|i| textures.get(i).copied())
            .apply(&mut m);
        m
    }

    /// Adds the model's textures, materials and LOD-0 meshes to `scene` and
    /// returns the parts of its default scene (every mesh node's primitives
    /// with the node's transform). `textures` are the decoded
    /// [`CookedModel::textures`], in order.
    pub fn add_to(&self, scene: &mut RenderScene, textures: &[TextureData]) -> ModelParts {
        let sources: Vec<_> = textures.iter().cloned().map(TextureSource::Rgba).collect();
        self.add_to_with(scene, &sources)
    }

    /// Adds every primitive's coarser levels (1 on) to the scene as meshes
    /// of their own, after whatever [`CookedModel::add_to`] added: per
    /// primitive, each level's mesh and its fraction of level 0's
    /// triangles (empty for a primitive of one level).
    pub fn add_lod_levels(&self, scene: &mut RenderScene) -> Vec<Vec<(MeshHandle, f32)>> {
        self.primitives
            .iter()
            .map(|p| {
                let full = p.triangle_count(0).max(1) as f32;
                (1..p.lods.len())
                    .map(|l| {
                        (
                            scene.add_mesh(p.mesh_compact(l)),
                            p.triangle_count(l) as f32 / full,
                        )
                    })
                    .collect()
            })
            .collect()
    }

    /// [`CookedModel::add_to`] with textures of either kind (transcoded to
    /// the device's format: [`CookedTexture::to_texture_source`]).
    pub fn add_to_with(&self, scene: &mut RenderScene, textures: &[TextureSource]) -> ModelParts {
        let handles: Vec<_> = textures
            .iter()
            .map(|t| scene.add_texture_source(t.clone()))
            .collect();
        // A material with physical extensions is added as the standard
        // material it draws as, the others in their glTF shape.
        let materials: Vec<MaterialHandle> = (0..self.materials.len())
            .map(|i| match self.materials[i].physical.present {
                0 => scene.add_material(self.material(i, &handles)),
                _ => scene.add_standard_material(self.standard_material(i, &handles)),
            })
            .collect();
        let meshes: Vec<MeshHandle> = self
            .primitives
            .iter()
            .map(|p| scene.add_mesh(p.mesh(0)))
            .collect();
        let world = self.node_world();
        let mut parts = Vec::new();
        if let Some(s) = self.default_scene {
            for n in self.scene_nodes(s) {
                if let Some(m) = self.nodes[n].mesh {
                    for p in self.meshes[m].primitives.clone() {
                        parts.push((meshes[p], materials[self.primitives[p].material], world[n]));
                    }
                }
            }
        }
        ModelParts { parts }
    }
}

fn oct_decode(x: f32, y: f32) -> Vec3 {
    let mut v = Vec3::new(x, y, 1.0 - x.abs() - y.abs());
    if v.z < 0.0 {
        let (ox, oy) = (v.x, v.y);
        v.x = (1.0 - oy.abs()) * ox.signum();
        v.y = (1.0 - ox.abs()) * oy.signum();
    }
    v.normalize_or_zero()
}

fn dequantize(p: &PrimitiveRow, raw: &[u8]) -> (Vec<Vertex>, Vec<[f32; 4]>) {
    let stride = p.vertex_stride as usize;
    let off = [p.pos_offset_x, p.pos_offset_y, p.pos_offset_z];
    let scale = [p.pos_scale_x, p.pos_scale_y, p.pos_scale_z];
    let u16_at = |v: &[u8], at: usize| u16::from_le_bytes([v[at], v[at + 1]]);
    let i16_at = |v: &[u8], at: usize| i16::from_le_bytes([v[at], v[at + 1]]);
    let snorm = |q: i16| (q as f32 / 32767.0).max(-1.0);
    let mut verts = Vec::with_capacity(p.vertex_count as usize);
    let mut tangents = Vec::new();
    for v in raw.chunks_exact(stride) {
        let mut position = [0.0f32; 3];
        for a in 0..3 {
            position[a] = off[a] + u16_at(v, a * 2) as f32 / 65535.0 * scale[a];
        }
        let normal = oct_decode(snorm(i16_at(v, 8)), snorm(i16_at(v, 10))).to_array();
        let mut at = 12;
        let mut uv = [0.0f32; 2];
        if p.attributes & attr::UV != 0 {
            uv = [
                p.uv_offset_x + u16_at(v, at) as f32 / 65535.0 * p.uv_scale_x,
                p.uv_offset_y + u16_at(v, at + 2) as f32 / 65535.0 * p.uv_scale_y,
            ];
            at += 4;
        }
        let mut color = [1.0f32; 4];
        if p.attributes & attr::COLOR != 0 {
            for (c, out) in color.iter_mut().enumerate() {
                *out = v[at + c] as f32 / 255.0;
            }
            at += 4;
        }
        // The cook's MikkTSpace tangent, or none (the renderer generates one).
        let mut tangent = [0.0f32; 4];
        if p.attributes & attr::TANGENT != 0 {
            let t = oct_decode(snorm(i16_at(v, at)), snorm(i16_at(v, at + 2)));
            let w = if i16_at(v, at + 4) < 0 { -1.0 } else { 1.0 };
            tangent = [t.x, t.y, t.z, w];
            tangents.push(tangent);
        }
        verts.push(Vertex {
            tangent,
            position,
            normal,
            uv,
            color,
        });
    }
    (verts, tangents)
}

/// A cooked texture: its metadata and KTX2 file.
#[derive(Clone, Debug, PartialEq)]
pub struct CookedTexture {
    pub id: String,
    pub meta: TextureMeta,
    pub ktx2: Vec<u8>,
}

impl CookedTexture {
    /// A `kind pixel` texture: drawn without mips and sampled nearest.
    pub fn is_pixel_art(&self) -> bool {
        self.meta.kind == crate::format::kind::PIXEL
    }

    /// Level 0 as RGBA8 for og-render (which uploads RGBA8 and builds its own
    /// mips today). RGBA8 textures decode everywhere; Basis Universal ones
    /// need the `transcode` feature (native) and are refused without it.
    pub fn to_texture_data(&self) -> Result<TextureData, LoadError> {
        let bad = |reason: String| LoadError::Invalid {
            path: self.id.clone(),
            reason,
        };
        let color_space = if self.meta.color_space == 0 {
            ColorSpace::Srgb
        } else {
            ColorSpace::Linear
        };
        let (width, height, rgba) = match self.meta.codec {
            codec::RGBA8 => ktx2::decode_rgba8(&self.ktx2, 0).map_err(bad)?,
            _ => self.transcode_basis().map_err(bad)?,
        };
        if self.is_pixel_art() {
            return Ok(TextureData {
                width,
                height,
                rgba,
                color_space,
                mips: Mips::None,
            });
        }
        Ok(TextureData {
            width,
            height,
            rgba,
            color_space,
            mips: Mips::Generate,
        })
    }

    /// The encoding this texture should reach a device that takes `formats`
    /// in: its kind's (normal maps `normal`, or `color` for a map cooked
    /// without y in alpha, linear data `data`, colour `color`), or RGBA8 for pixel art, distance fields, lossless cooks, a
    /// size not in whole 4x4 blocks, or no renderer yet (`None`).
    pub fn target_format(&self, formats: Option<&TextureFormats>) -> TextureFormat {
        let Some(f) = formats else {
            return TextureFormat::Rgba8;
        };
        let lossless = self.meta.codec == codec::RGBA8;
        let blocks = self.meta.width.is_multiple_of(4) && self.meta.height.is_multiple_of(4);
        if lossless || !blocks {
            return TextureFormat::Rgba8;
        }
        match self.meta.kind {
            // A two-channel format takes y from alpha: cooks before phase 2
            // left alpha opaque there, so their maps go as colour.
            crate::format::kind::NORMAL if f.normal.two_channel() && self.meta.has_alpha == 0 => {
                f.color
            }
            crate::format::kind::NORMAL => f.normal,
            crate::format::kind::LINEAR => f.data,
            crate::format::kind::COLOR => f.color,
            _ => TextureFormat::Rgba8,
        }
    }

    /// The texture for og-render on a device that takes `formats`
    /// (`RenderInfo::texture_formats`): every cooked mip level transcoded
    /// to [`CookedTexture::target_format`] (BC7 and BC5 on desktop GPUs,
    /// ASTC 4x4 on Apple and most Android, ETC2 and EAC elsewhere), or
    /// RGBA8 level 0 as [`CookedTexture::to_texture_data`] gives it when the
    /// target is RGBA8. Native transcoding needs the `transcode` feature.
    pub fn to_texture_source(
        &self,
        formats: Option<&TextureFormats>,
    ) -> Result<TextureSource, LoadError> {
        let target = self.target_format(formats);
        if target == TextureFormat::Rgba8 {
            return self.to_texture_data().map(TextureSource::Rgba);
        }
        let bad = |reason: String| LoadError::Invalid {
            path: self.id.clone(),
            reason,
        };
        let levels = self.transcode_levels(target).map_err(bad)?;
        let color_space = if self.meta.color_space == 0 && !target.two_channel() {
            ColorSpace::Srgb
        } else {
            ColorSpace::Linear
        };
        let upload = TextureUpload {
            desc: TextureDesc {
                width: self.meta.width,
                height: self.meta.height,
                format: target,
                mip_levels: levels.len() as u32,
                color_space,
            },
            levels,
        };
        upload.desc.validate(&upload.levels).map_err(bad)?;
        Ok(TextureSource::Encoded(upload))
    }

    /// Every mip level transcoded to `format` (a block format).
    #[cfg(feature = "transcode")]
    fn transcode_levels(&self, format: TextureFormat) -> Result<Vec<Vec<u8>>, String> {
        use basis_universal::{TranscodeParameters, Transcoder, TranscoderTextureFormat as T};
        let to = match format {
            TextureFormat::Bc7 => T::BC7_RGBA,
            TextureFormat::Bc5 => T::BC5_RG,
            TextureFormat::Astc4x4 => T::ASTC_4x4_RGBA,
            TextureFormat::Etc2Rgba8 => T::ETC2_RGBA,
            TextureFormat::EacRg11 => T::ETC2_EAC_RG11,
            TextureFormat::Rgba8 => T::RGBA32,
        };
        let basis = ktx2::to_basis(&self.ktx2)?;
        let mut t = Transcoder::new();
        t.prepare_transcoding(&basis)
            .map_err(|_| "basisu refused the texture".to_string())?;
        let count = t.image_level_count(&basis, 0);
        let mut levels = Vec::with_capacity(count as usize);
        for level_index in 0..count {
            let data = t
                .transcode_image_level(
                    &basis,
                    to,
                    TranscodeParameters {
                        image_index: 0,
                        level_index,
                        ..Default::default()
                    },
                )
                .map_err(|e| format!("basisu transcode of level {level_index} failed: {e:?}"))?;
            levels.push(data);
        }
        t.end_transcoding();
        Ok(levels)
    }

    #[cfg(not(feature = "transcode"))]
    fn transcode_levels(&self, format: TextureFormat) -> Result<Vec<Vec<u8>>, String> {
        Err(format!(
            "transcoding to {format:?} needs og-cook's `transcode` feature (native) or the web host's transcoder"
        ))
    }

    #[cfg(feature = "transcode")]
    fn transcode_basis(&self) -> Result<(u32, u32, Vec<u8>), String> {
        use basis_universal::{TranscodeParameters, Transcoder, TranscoderTextureFormat};
        let basis = ktx2::to_basis(&self.ktx2)?;
        let mut t = Transcoder::new();
        t.prepare_transcoding(&basis)
            .map_err(|_| "basisu refused the texture".to_string())?;
        let rgba = t
            .transcode_image_level(
                &basis,
                TranscoderTextureFormat::RGBA32,
                TranscodeParameters {
                    image_index: 0,
                    level_index: 0,
                    ..Default::default()
                },
            )
            .map_err(|e| format!("basisu transcode failed: {e:?}"))?;
        t.end_transcoding();
        Ok((self.meta.width, self.meta.height, rgba))
    }

    #[cfg(not(feature = "transcode"))]
    fn transcode_basis(&self) -> Result<(u32, u32, Vec<u8>), String> {
        Err("a Basis Universal texture needs a transcoder: the host's (web) or og-cook's `transcode` feature (native)".into())
    }
}

/// A cooked sound.
#[derive(Clone, Debug, PartialEq)]
pub struct CookedSound {
    pub id: String,
    pub meta: SoundMeta,
    /// The QOA file; of a sound cooked in chunks (`meta.chunk_count` > 0),
    /// its header and first chunk only.
    pub qoa: Vec<u8>,
}

/// A sound chunk blob's index and QOA frames, read from its file's bytes
/// without a bundle (a mixer thread reading a chunk the moment a voice
/// needs it), its type and cook `stamp` checked.
pub fn sound_chunk_of(bytes: &[u8], path: &str, stamp: u64) -> Result<(u32, Vec<u8>), LoadError> {
    let aligned = og_blob::AlignedBytes::from_slice(bytes);
    let expect = Expect {
        type_hash: format::SoundChunk::type_hash(),
        schema_version: format::SoundChunk::schema_version(),
        stamp: Some(stamp),
    };
    let blob = Blob::open(aligned.as_slice(), path, &expect)?;
    let meta = one::<format::SoundChunkMeta>(&blob)?;
    Ok((meta.chunk, blob.raw("qoa")?.to_vec()))
}

/// A cooked texture array: its shape and its layers' texture ids, in
/// layer order ([`Bundle::texture_array`]).
#[derive(Clone, Debug, PartialEq)]
pub struct CookedTextureArray {
    pub id: String,
    pub meta: format::TextureArrayMeta,
    pub layers: Vec<String>,
}

impl CookedTextureArray {
    /// The array og-render uploads from its layers' textures, each in the
    /// encoding `formats` takes (RGBA8 without a renderer; native only:
    /// the web host hands the layers over transcoded).
    #[cfg(feature = "transcode")]
    pub fn to_texture_array(
        &self,
        bundle: &Bundle,
        formats: Option<&og_render::TextureFormats>,
    ) -> Result<og_render::TextureArrayData, LoadError> {
        let mut layers = Vec::with_capacity(self.layers.len());
        for l in &self.layers {
            let t = bundle.load_texture(l)?;
            layers.push(t.to_texture_source(formats)?);
        }
        Ok(og_render::TextureArrayData { layers })
    }
}

/// The id of chunk `n` of a sound cooked in chunks (`audio/theme.wav#chunk3`).
pub fn chunk_id(id: &str, n: u32) -> String {
    format!("{}#chunk{n}", asset_id(id))
}

impl CookedSound {
    /// Whether it was cooked in chunks (the others are blobs of their own).
    pub fn chunked(&self) -> bool {
        self.meta.chunk_count > 0
    }

    /// Interleaved 16-bit PCM (of a sound cooked in chunks: refused, its
    /// samples are not all here).
    pub fn decode(&self) -> Result<qoa::Decoded, LoadError> {
        if self.chunked() {
            return Err(LoadError::Invalid {
                path: self.id.clone(),
                reason: "cooked in chunks: decode it chunk by chunk".into(),
            });
        }
        qoa::decode(&self.qoa).map_err(|reason| LoadError::Invalid {
            path: self.id.clone(),
            reason,
        })
    }
}

/// A cooked font: metrics, glyphs sorted by code point, atlas pages (RGBA8,
/// linear; RGB is the MSDF, A the true SDF).
#[derive(Clone, Debug, PartialEq)]
pub struct CookedFont {
    pub id: String,
    pub meta: FontMeta,
    pub glyphs: Vec<GlyphRow>,
    /// Kerning pairs between cooked characters, sorted by (left, right).
    pub kerning: Vec<KernRow>,
    pub pages: Vec<TextureData>,
    /// The font file, for a font cooked with a shaped script (og-ui shapes
    /// its words) or with `runtime true` (characters outside the charset
    /// are rendered from it when a text needs them).
    pub ttf: Option<Vec<u8>>,
    /// An icon set's names (an `.ogt` of `type icons`, cook::icons): icon
    /// `i` is the glyph at U+E000 + i. Empty for a text font.
    pub names: Vec<String>,
}

impl CookedFont {
    pub fn glyph(&self, c: char) -> Option<&GlyphRow> {
        self.glyphs
            .binary_search_by_key(&(c as u32), |g| g.codepoint)
            .ok()
            .map(|i| &self.glyphs[i])
    }
}

/// One entity of a scene.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityDesc {
    pub name: String,
    pub parent: Option<usize>,
    /// The asset id of a model this entity instantiates (its default scene
    /// becomes the entity's children).
    pub instance: Option<String>,
}

/// One component type's values in a scene.
#[derive(Clone, Debug, PartialEq)]
pub struct ComponentTable {
    pub name: String,
    /// The schema the rows are packed with (from the cook's schema set, or
    /// inferred from the scene text when `inferred`). A host migrates rows to
    /// its registered layout by field name.
    pub schema: og_core::Schema,
    pub inferred: bool,
    /// `(entity index, packed field bytes)`.
    pub rows: Vec<(usize, Vec<u8>)>,
}

impl ComponentTable {
    /// The packed value of `entity`, if it has this component.
    pub fn get(&self, entity: usize) -> Option<&[u8]> {
        self.rows
            .iter()
            .find(|r| r.0 == entity)
            .map(|r| r.1.as_slice())
    }
}

/// A scene's baked lighting as cooked (`Bundle::lightmap`).
#[derive(Clone, Debug, PartialEq)]
pub struct CookedLightmap {
    pub file: crate::lightmap::LightmapFile,
    pub width: u32,
    pub height: u32,
    /// Per page, RGBA f16 bits, rows top to bottom.
    pub pages: Vec<Vec<u16>>,
}

/// A decoded scene: plain data for the host to spawn.
#[derive(Clone, Debug, PartialEq)]
pub struct SceneDesc {
    pub id: String,
    pub name: String,
    /// Parents precede children.
    pub entities: Vec<EntityDesc>,
    /// Sorted by type name.
    pub components: Vec<ComponentTable>,
    /// Its named paths (`path "<name>" { ... }`), in file order.
    pub paths: Vec<(String, og_timeline::PathDesc)>,
    /// The resource fields it sets when instantiated (`resource <Name> {
    /// ... }`), sorted by name.
    pub resources: Vec<ResourceTable>,
}

/// One resource's fields in a scene: `schema` holds only the fields the
/// text lists (the others keep the world's values) and `bytes` their
/// packed values.
#[derive(Clone, Debug, PartialEq)]
pub struct ResourceTable {
    pub name: String,
    pub schema: og_core::Schema,
    pub inferred: bool,
    pub bytes: Vec<u8>,
}

impl SceneDesc {
    fn decode(id: &str, blob: &Blob) -> Result<SceneDesc, LoadError> {
        let meta = one::<SceneMeta>(blob)?;
        let entities = table::<EntityRow>(blob, "entities")?;
        let refs = table::<RefRow>(blob, "refs")?;
        let comps = table::<ComponentRow>(blob, "components")?;
        let strings = blob.raw("strings")?;
        let bad = |r: String| invalid(blob, r);
        let s = |o: u32, l: u32| {
            pool_str(strings, o, l)
                .map(str::to_string)
                .ok_or_else(|| bad("string out of range".into()))
        };
        let ref_ids = refs
            .iter()
            .map(|r| s(r.id_offset, r.id_len))
            .collect::<Result<Vec<_>, _>>()?;
        let mut out = Vec::with_capacity(entities.len());
        for (i, e) in entities.iter().enumerate() {
            let parent = opt(e.parent);
            if parent.is_some_and(|p| p >= i) {
                return Err(bad(format!("entity {i}: parent does not precede it")));
            }
            out.push(EntityDesc {
                name: s(e.name_offset, e.name_len)?,
                parent,
                instance: match opt(e.instance) {
                    Some(r) => Some(
                        ref_ids
                            .get(r)
                            .cloned()
                            .ok_or_else(|| bad("instance out of range".into()))?,
                    ),
                    None => None,
                },
            });
        }
        let mut components = Vec::with_capacity(comps.len());
        for c in &comps {
            let name = s(c.name_offset, c.name_len)?;
            let json = std::str::from_utf8(blob.raw(&format!("s.{name}"))?)
                .map_err(|_| bad(format!("schema of {name} is not UTF-8")))?;
            let schema = og_core::Schema::from_json(json)
                .map_err(|e| bad(format!("schema of {name}: {e}")))?;
            let hash = (c.layout_hash_hi as u64) << 32 | c.layout_hash_lo as u64;
            if schema.layout_hash() != hash {
                return Err(bad(format!(
                    "schema of {name} does not match its layout hash"
                )));
            }
            let (bytes, stride) = blob.rows(&format!("c.{name}"), hash)?;
            let size = og_core::Layout::of(&schema).size();
            if stride != 4 + size {
                return Err(bad(format!(
                    "{name}: {stride}-byte rows for a {size}-byte layout"
                )));
            }
            let mut rows = Vec::with_capacity(bytes.len() / stride);
            for r in bytes.chunks_exact(stride) {
                let entity = u32::from_le_bytes(r[..4].try_into().expect("4 bytes")) as usize;
                if entity >= out.len() {
                    return Err(bad(format!("{name}: entity index out of range")));
                }
                rows.push((entity, r[4..].to_vec()));
            }
            components.push(ComponentTable {
                name,
                schema,
                inferred: c.inferred != 0,
                rows,
            });
        }
        // Older blobs and scenes without paths have no `paths` section.
        let paths = match blob.raw_opt("paths") {
            None => Vec::new(),
            Some(text) => std::str::from_utf8(text)
                .map_err(|_| bad("the paths text is not UTF-8".into()))
                .and_then(|t| og_timeline::parse_paths(t).map_err(bad))?,
        };
        // Older blobs and scenes without resources have no `resources` table.
        let mut resources = Vec::new();
        if blob.section("resources").is_some() {
            for r in table::<ComponentRow>(blob, "resources")? {
                let name = s(r.name_offset, r.name_len)?;
                let json = std::str::from_utf8(blob.raw(&format!("rs.{name}"))?)
                    .map_err(|_| bad(format!("schema of resource {name} is not UTF-8")))?;
                let schema = og_core::Schema::from_json(json)
                    .map_err(|e| bad(format!("schema of resource {name}: {e}")))?;
                let hash = (r.layout_hash_hi as u64) << 32 | r.layout_hash_lo as u64;
                if schema.layout_hash() != hash {
                    return Err(bad(format!(
                        "schema of resource {name} does not match its layout hash"
                    )));
                }
                let (bytes, _) = blob.rows(&format!("r.{name}"), hash)?;
                let size = og_core::Layout::of(&schema).size();
                if bytes.len() < size {
                    return Err(bad(format!(
                        "resource {name}: {} bytes for a {size}-byte layout",
                        bytes.len()
                    )));
                }
                resources.push(ResourceTable {
                    name,
                    schema,
                    inferred: r.inferred != 0,
                    bytes: bytes[..size].to_vec(),
                });
            }
        }
        Ok(SceneDesc {
            id: id.to_string(),
            name: s(meta.name_offset, meta.name_len)?,
            entities: out,
            components,
            paths,
            resources,
        })
    }
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

/// The joints and weights of one skinned model primitive, per cooked vertex.
#[derive(Clone, Debug, PartialEq)]
pub struct SkinStream {
    /// The primitive's index in the model blob ([`CookedModel::primitives`]).
    pub primitive: usize,
    pub mesh: usize,
    /// Indices into the skin's joints.
    pub joints: Vec<[u16; 4]>,
    /// Summing to 1 (cooked as unorm16 summing to 65535).
    pub weights: Vec<[f32; 4]>,
}

/// The morph targets of one model primitive, per cooked vertex.
#[derive(Clone, Debug, PartialEq)]
pub struct MorphTargets {
    pub primitive: usize,
    pub mesh: usize,
    /// Per target: its name (empty when unnamed) and default weight.
    pub names: Vec<String>,
    pub default_weights: Vec<f32>,
    /// Per target, per vertex: the position delta.
    pub positions: Vec<Vec<Vec3>>,
    /// Per target, per vertex: the normal delta, when every target has one.
    pub normals: Option<Vec<Vec<Vec3>>>,
}

/// A decoded animation blob.
#[derive(Clone, Debug, PartialEq)]
pub struct CookedAnimation {
    pub id: String,
    pub skeleton: og_anim::SkeletonAsset,
    pub clips: Vec<og_anim::ClipAsset>,
    pub skin_streams: Vec<SkinStream>,
    pub morphs: Vec<MorphTargets>,
}

impl CookedAnimation {
    /// The clip called `name`.
    pub fn clip(&self, name: &str) -> Option<&og_anim::ClipAsset> {
        self.clips.iter().find(|c| c.name == name)
    }

    /// Decodes an animation blob from its bytes alone, without a manifest
    /// (the cook reads back the blobs it retargets from and onto).
    pub fn from_bytes(id: &str, bytes: &[u8]) -> Result<CookedAnimation, LoadError> {
        let aligned = og_blob::AlignedBytes::from_slice(bytes);
        let expect = Expect {
            type_hash: format::Animation::type_hash(),
            schema_version: format::Animation::schema_version(),
            stamp: None,
        };
        let blob = Blob::open(aligned.as_slice(), id, &expect)?;
        CookedAnimation::decode(asset_id(id), &blob)
    }

    fn decode(id: &str, blob: &Blob) -> Result<CookedAnimation, LoadError> {
        let bad = |r: String| invalid(blob, r);
        let meta = one::<AnimMeta>(blob)?;
        let nodes = table::<NodeRow>(blob, "nodes")?;
        let skins = table::<SkinRow>(blob, "skins")?;
        let joints = blob.table::<u32>("joints", 0)?.into_owned();
        let ibms = blob.table::<f32>("inverse_binds", 0)?.into_owned();
        let clips = table::<ClipRow>(blob, "clips")?;
        let channels = table::<ChannelRow>(blob, "channels")?;
        let samples = blob.table::<f32>("samples", 0)?.into_owned();
        let packed = blob.table::<u16>("packed", 0)?.into_owned();
        let events = table::<EventRow>(blob, "events")?;
        let streams = table::<SkinStreamRow>(blob, "skin_streams")?;
        let skin_data = blob.raw("skin_data")?;
        let morphs = table::<MorphRow>(blob, "morphs")?;
        let targets = table::<TargetRow>(blob, "targets")?;
        let morph_data = blob.raw("morph_data")?;
        let strings = blob.raw("strings")?;
        let s = |o: u32, l: u32| {
            pool_str(strings, o, l)
                .map(str::to_string)
                .ok_or_else(|| bad("string out of range".into()))
        };
        if nodes.len() != meta.node_count as usize
            || clips.len() != meta.clip_count as usize
            || ibms.len() != joints.len() * 16
        {
            return Err(bad("counts disagree with the meta row".into()));
        }
        let mut skeleton = og_anim::SkeletonAsset::default();
        for n in &nodes {
            skeleton.nodes.push(og_anim::SkeletonNode {
                name: s(n.name_offset, n.name_len)?,
                parent: n.parent,
                rest: og_anim::Trs::new(
                    Vec3::new(n.tx, n.ty, n.tz),
                    Quat::from_xyzw(n.rx, n.ry, n.rz, n.rw),
                    Vec3::new(n.sx, n.sy, n.sz),
                ),
                mesh: n.mesh,
            });
        }
        for k in &skins {
            let r = k.joint_first as usize..(k.joint_first + k.joint_count) as usize;
            let js = joints
                .get(r.clone())
                .ok_or_else(|| bad("skin joints out of range".into()))?
                .to_vec();
            let inverse_bind = r
                .map(|j| Mat4::from_cols_slice(&ibms[j * 16..j * 16 + 16]))
                .collect();
            skeleton.skins.push(og_anim::Skin {
                name: s(k.name_offset, k.name_len)?,
                joints: js,
                inverse_bind,
            });
        }
        skeleton
            .validate()
            .map_err(|e| bad(e.reason().to_string()))?;
        let mut out_clips = Vec::with_capacity(clips.len());
        for c in &clips {
            let rows = channels
                .get(c.channel_first as usize..(c.channel_first + c.channel_count) as usize)
                .ok_or_else(|| bad("clip channels out of range".into()))?;
            let mut chs = Vec::with_capacity(rows.len());
            for ch in rows {
                let path = og_anim::ChannelPath::from_code(ch.path)
                    .ok_or_else(|| bad(format!("channel path {}", ch.path)))?;
                let data = crate::anim_codec::decode(ch, c.frames, &samples, &packed)
                    .map_err(|e| bad(format!("clip channel: {e}")))?;
                let node_name = skeleton
                    .nodes
                    .get(ch.node as usize)
                    .map(|n| n.name.clone())
                    .ok_or_else(|| bad("channel node out of range".into()))?;
                chs.push(og_anim::Channel {
                    node: ch.node,
                    node_name,
                    path,
                    step: ch.interpolation == format::interpolation::STEP,
                    lanes: ch.lanes,
                    samples: data,
                });
            }
            let evs = events
                .get(c.event_first as usize..(c.event_first + c.event_count) as usize)
                .ok_or_else(|| bad("clip events out of range".into()))?
                .iter()
                .map(|e| {
                    Ok(og_anim::ClipEvent {
                        name: s(e.name_offset, e.name_len)?,
                        time: e.time,
                    })
                })
                .collect::<Result<Vec<_>, LoadError>>()?;
            let clip = og_anim::ClipAsset {
                name: s(c.name_offset, c.name_len)?,
                duration: c.duration,
                rate: c.rate,
                source_rate: c.source_rate,
                frames: c.frames,
                channels: chs,
                events: evs,
            };
            clip.validate().map_err(|e| bad(e.reason().to_string()))?;
            out_clips.push(clip);
        }
        let slice = |data: &'_ [u8], o: u32, l: u32| -> Result<std::ops::Range<usize>, LoadError> {
            let (o, l) = (o as usize, l as usize);
            (o + l <= data.len())
                .then_some(o..o + l)
                .ok_or_else(|| bad("stream out of range".into()))
        };
        let mut skin_streams = Vec::with_capacity(streams.len());
        for st in &streams {
            let n = st.vertex_count as usize;
            let raw = meshopt_codec::decode_vertex_buffer(
                &skin_data[slice(skin_data, st.data_offset, st.data_size)?],
                n,
                16,
            )
            .map_err(|e| bad(format!("skin stream: {e}")))?;
            let u16s: Vec<u16> = raw
                .as_chunks::<2>()
                .0
                .iter()
                .map(|b| u16::from_le_bytes([b[0], b[1]]))
                .collect();
            let mut joints_out = Vec::with_capacity(n);
            let mut weights_out = Vec::with_capacity(n);
            for v in u16s.as_chunks::<8>().0 {
                joints_out.push([v[0], v[1], v[2], v[3]]);
                weights_out.push([
                    v[4] as f32 / 65535.0,
                    v[5] as f32 / 65535.0,
                    v[6] as f32 / 65535.0,
                    v[7] as f32 / 65535.0,
                ]);
            }
            skin_streams.push(SkinStream {
                primitive: st.primitive as usize,
                mesh: st.mesh as usize,
                joints: joints_out,
                weights: weights_out,
            });
        }
        let mut out_morphs = Vec::with_capacity(morphs.len());
        for m in &morphs {
            let (n, k, moved) = (
                m.vertex_count as usize,
                m.target_count as usize,
                m.moved as usize,
            );
            let stride = if m.normals != 0 { 24 } else { 12 };
            let decode = |o: u32, l: u32, stride: usize| -> Result<Vec<u8>, LoadError> {
                if moved == 0 {
                    return Ok(Vec::new());
                }
                meshopt_codec::decode_vertex_buffer(
                    &morph_data[slice(morph_data, o, l)?],
                    moved,
                    stride,
                )
                .map_err(|e| bad(format!("morph stream: {e}")))
            };
            let raw = decode(m.data_offset, m.data_size, stride)?;
            let index = decode(m.index_offset, m.index_size, 4)?;
            let f: Vec<f32> = raw
                .as_chunks::<4>()
                .0
                .iter()
                .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .collect();
            let index: Vec<usize> = index
                .as_chunks::<4>()
                .0
                .iter()
                .map(|b| u32::from_le_bytes(*b) as usize)
                .collect();
            let lanes = stride / 4;
            let rows = targets
                .get(m.target_first as usize..m.target_first as usize + k)
                .ok_or_else(|| bad("morph targets out of range".into()))?;
            let mut positions = vec![vec![Vec3::ZERO; n]; k];
            let mut normals = (m.normals != 0).then(|| vec![vec![Vec3::ZERO; n]; k]);
            for (t, row) in rows.iter().enumerate() {
                let (first, count) = (row.moved_first as usize, row.moved_count as usize);
                if first + count > moved {
                    return Err(bad("morph target's moved vertices out of range".into()));
                }
                for (e, &v) in index.iter().enumerate().skip(first).take(count) {
                    if v >= n {
                        return Err(bad(format!("morph vertex {v} of {n}")));
                    }
                    let at = e * lanes;
                    positions[t][v] = Vec3::new(f[at], f[at + 1], f[at + 2]);
                    if let Some(normals) = normals.as_mut() {
                        normals[t][v] = Vec3::new(f[at + 3], f[at + 4], f[at + 5]);
                    }
                }
            }
            out_morphs.push(MorphTargets {
                primitive: m.primitive as usize,
                mesh: m.mesh as usize,
                names: rows
                    .iter()
                    .map(|t| s(t.name_offset, t.name_len))
                    .collect::<Result<_, _>>()?,
                default_weights: rows.iter().map(|t| t.default_weight).collect(),
                positions,
                normals,
            });
        }
        Ok(CookedAnimation {
            id: id.to_string(),
            skeleton,
            clips: out_clips,
            skin_streams,
            morphs: out_morphs,
        })
    }
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
    let text = blob.raw("text")?;
    let text = text
        .get(..meta.text_bytes as usize)
        .ok_or_else(|| invalid(blob, "the tileset text is cut short"))?;
    let text =
        std::str::from_utf8(text).map_err(|_| invalid(blob, "the tileset text is not UTF-8"))?;
    og_tilemap::Tileset::parse(text).map_err(|e| invalid(blob, e.to_string()))
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
    let mut map = og_tilemap::TilemapAsset {
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
        let text = s(t.offset, t.len)?;
        map.tilesets.push(if t.embedded == 1 {
            og_tilemap::TilesetRef::Embedded(
                og_tilemap::Tileset::parse(&text).map_err(|e| invalid(blob, e.to_string()))?,
            )
        } else {
            og_tilemap::TilesetRef::Asset(text)
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

impl Bundle {
    /// Decodes a flipbook sheet blob (`og.Flipbook`) into its sheet.
    pub fn flipbook(&self, id: &str, bytes: &[u8]) -> Result<og_anim::FlipbookSheet, LoadError> {
        let blob = self.blob::<format::Flipbook>(id, bytes)?;
        let meta = one::<format::FlipbookMeta>(&blob)?;
        let text = blob.raw("text")?;
        let text = text
            .get(..meta.text_bytes as usize)
            .and_then(|t| std::str::from_utf8(t).ok())
            .ok_or_else(|| invalid(&blob, "the sheet text is cut short or not UTF-8"))?;
        og_anim::FlipbookSheet::parse(text).map_err(|e| invalid(&blob, e.reason()))
    }

    /// Decodes a 2D rig blob (`og.Rig2D`) into its rig.
    pub fn rig2d(&self, id: &str, bytes: &[u8]) -> Result<og_anim::Rig2dAsset, LoadError> {
        let blob = self.blob::<format::Rig2D>(id, bytes)?;
        let meta = one::<format::Rig2DMeta>(&blob)?;
        let text = blob.raw("text")?;
        let text = text
            .get(..meta.text_bytes as usize)
            .and_then(|t| std::str::from_utf8(t).ok())
            .ok_or_else(|| invalid(&blob, "the rig text is cut short or not UTF-8"))?;
        og_anim::Rig2dAsset::parse(text).map_err(|e| invalid(&blob, e.reason()))
    }

    /// Reads and decodes a 2D rig (native).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_rig2d(&self, id: &str) -> Result<og_anim::Rig2dAsset, LoadError> {
        self.rig2d(id, &self.read(id)?)
    }

    /// Reads and decodes a flipbook sheet (native).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_flipbook(&self, id: &str) -> Result<og_anim::FlipbookSheet, LoadError> {
        self.flipbook(id, &self.read(id)?)
    }
}

// ── E6: behaviours and navmeshes ─────────────────────────────────────────

impl Bundle {
    /// The validated text of a behaviour blob (`og.Behaviour`); parse it
    /// with og-ai's `Behaviour::parse`.
    pub fn behaviour_text(&self, id: &str, bytes: &[u8]) -> Result<String, LoadError> {
        let blob = self.blob::<format::Behaviour>(id, bytes)?;
        let meta = one::<format::BehaviourMeta>(&blob)?;
        let text = blob.raw("text")?;
        text.get(..meta.text_bytes as usize)
            .and_then(|t| std::str::from_utf8(t).ok())
            .map(str::to_string)
            .ok_or_else(|| invalid(&blob, "the behaviour text is cut short or not UTF-8"))
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

    /// The humanoid map of a rig blob (`og.Rig`).
    pub fn rig(&self, id: &str, bytes: &[u8]) -> Result<og_anim::RigAsset, LoadError> {
        let blob = self.blob::<format::Rig>(id, bytes)?;
        let meta = one::<format::RigMeta>(&blob)?;
        let text = blob
            .raw("text")?
            .get(..meta.text_bytes as usize)
            .and_then(|t| std::str::from_utf8(t).ok())
            .ok_or_else(|| invalid(&blob, "the rig text is cut short or not UTF-8"))?;
        og_anim::RigAsset::parse(text).map_err(|e| invalid(&blob, e.reason().to_string()))
    }

    /// Decodes a lightmap blob (`og.Lightmap`): the lightmap text and its
    /// pages (RGBA f16 bits per page).
    pub fn lightmap(&self, id: &str, bytes: &[u8]) -> Result<CookedLightmap, LoadError> {
        let blob = self.blob::<format::Lightmap>(id, bytes)?;
        let meta = one::<format::LightmapMeta>(&blob)?;
        let text = blob
            .raw("text")?
            .get(..meta.text_bytes as usize)
            .and_then(|t| std::str::from_utf8(t).ok())
            .ok_or_else(|| invalid(&blob, "the lightmap text is cut short or not UTF-8"))?;
        let file = crate::lightmap::LightmapFile::parse(text).map_err(|e| invalid(&blob, e))?;
        let pages = blob
            .raw("pages")?
            .get(..meta.pages_bytes as usize)
            .ok_or_else(|| invalid(&blob, "the lightmap pages are cut short"))?;
        let (width, height, pages) =
            crate::lightmap::pages_from_ktx2(pages).map_err(|e| invalid(&blob, e))?;
        Ok(CookedLightmap {
            file,
            width,
            height,
            pages,
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_lightmap(&self, id: &str) -> Result<CookedLightmap, LoadError> {
        self.lightmap(id, &self.read(id)?)
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
        og_timeline::Timeline::parse(text, &mut |f| {
            Err(format!("the cooked text names caption file '{f}'"))
        })
        .map_err(|e| invalid(&blob, e))
    }

    /// Decodes a navmesh blob (`og.NavMesh`): the heightfield and the tiles
    /// the cook built (link them with `og_nav::NavMesh::from_tiles`).
    pub fn navmesh(
        &self,
        id: &str,
        bytes: &[u8],
    ) -> Result<(og_nav::Heightfield, Vec<og_nav::Tile>), LoadError> {
        let blob = self.blob::<format::NavMesh>(id, bytes)?;
        let m = one::<format::NavMeshMeta>(&blob)?;
        let columns = table::<format::NavColumnRow>(&blob, "columns")?;
        let spans = table::<format::NavSpanRow>(&blob, "spans")?;
        let tiles = table::<format::NavTileRow>(&blob, "tiles")?;
        let verts = table::<format::NavVertRow>(&blob, "verts")?;
        let polys = table::<format::NavPolyRow>(&blob, "polys")?;
        let index_bytes = blob.raw("poly_verts")?;
        let indices: Vec<u32> = index_bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|c| u32::from_le_bytes(*c))
            .collect();
        if columns.len() != (m.width as usize) * (m.depth as usize) {
            return Err(invalid(&blob, "the columns do not fit the grid"));
        }
        let params = og_nav::NavParams {
            cell_size: m.cell_size,
            cell_height: m.cell_height,
            agent_height: m.agent_height,
            agent_radius: m.agent_radius,
            agent_climb: m.agent_climb,
            max_slope: m.max_slope,
            tile_size: m.tile_size,
            max_error: m.max_error,
        };
        let field = og_nav::Heightfield {
            params,
            origin: glam::Vec3::new(m.origin_x, m.origin_y, m.origin_z),
            width: m.width,
            depth: m.depth,
            columns: columns.iter().map(|c| [c.first, c.count]).collect(),
            spans: spans
                .iter()
                .map(|s| og_nav::Span {
                    floor: s.floor,
                    clear: s.clear,
                })
                .collect(),
        };
        if field
            .columns
            .iter()
            .any(|c| (c[0] + c[1]) as usize > field.spans.len())
        {
            return Err(invalid(&blob, "a column's spans are out of range"));
        }
        let mut out = Vec::with_capacity(tiles.len());
        for t in &tiles {
            let vs = verts
                .get(t.vert_first as usize..(t.vert_first + t.vert_count) as usize)
                .ok_or_else(|| invalid(&blob, "a tile's vertices are out of range"))?;
            let ps = polys
                .get(t.poly_first as usize..(t.poly_first + t.poly_count) as usize)
                .ok_or_else(|| invalid(&blob, "a tile's polygons are out of range"))?;
            let mut tile = og_nav::Tile {
                verts: vs.iter().map(|v| glam::Vec3::new(v.x, v.y, v.z)).collect(),
                lattice: vs.iter().map(|v| (v.lx, v.lz)).collect(),
                polys: Vec::with_capacity(ps.len()),
            };
            for p in ps {
                let idx = indices
                    .get(p.index_first as usize..(p.index_first + p.index_count) as usize)
                    .ok_or_else(|| invalid(&blob, "a polygon's vertices are out of range"))?;
                if idx.len() < 3 || idx.iter().any(|&i| i as usize >= tile.verts.len()) {
                    return Err(invalid(&blob, "a polygon names a vertex its tile lacks"));
                }
                tile.polys.push(og_nav::Poly {
                    verts: idx.iter().map(|&i| i as u16).collect(),
                    links: vec![og_nav::mesh::NONE; idx.len()],
                    region: p.region,
                });
            }
            out.push(tile);
        }
        Ok((field, out))
    }
}

// ── E6: string tables ────────────────────────────────────────────────────

impl Bundle {
    /// The validated text of a string table blob (`og.Strings`); parse it
    /// with og-i18n's `StringTable::parse`.
    pub fn strings_text(&self, id: &str, bytes: &[u8]) -> Result<String, LoadError> {
        let blob = self.blob::<format::Strings>(id, bytes)?;
        let meta = one::<format::StringsMeta>(&blob)?;
        let text = blob.raw("text")?;
        text.get(..meta.text_bytes as usize)
            .and_then(|t| std::str::from_utf8(t).ok())
            .map(str::to_string)
            .ok_or_else(|| invalid(&blob, "the string table is cut short or not UTF-8"))
    }
}

// ── phase 2: terrain ─────────────────────────────────────────────────────

/// One layer of a cooked terrain: its textures (asset ids, loaded by the
/// host) and the rest of og-render's layer.
#[derive(Clone, Debug, PartialEq)]
pub struct CookedTerrainLayer {
    pub albedo: Option<String>,
    pub normal: Option<String>,
    /// The layer without its textures (`albedo` and `normal` are `None`).
    pub layer: og_render::TerrainLayer,
}

/// A cooked terrain (`og.Terrain`): og-render's tile layout, the range of
/// its heights, its layers and settings, and its tiles still compressed,
/// each decoded on its own ([`CookedTerrain::tile`], or inflated elsewhere
/// and finished by [`CookedTerrain::tile_from`]): a streamer decodes the
/// tiles a camera needs, physics the whole field ([`CookedTerrain::field`]).
/// Cloning shares the compressed tiles.
#[derive(Clone, Debug)]
pub struct CookedTerrain {
    /// The asset id.
    pub id: String,
    pub layout: og_render::TerrainLayout,
    /// The lowest and highest height (metres above the origin).
    pub range: (f32, f32),
    pub layers: Vec<CookedTerrainLayer>,
    /// The texture arrays the layers' albedos and normal maps come from
    /// (`layers`, `layerNormals`: asset ids, loaded by the host); each
    /// layer's `albedo_layer` and `normal_layer` name its layers of them.
    pub albedo_array: Option<String>,
    pub normal_array: Option<String>,
    pub height_blend: f32,
    pub lod: og_render::TerrainLod,
    pub cast_shadows: bool,
    /// Whether the game gets a height-field collider for it.
    pub collision: bool,
    tiles: std::sync::Arc<TerrainTiles>,
}

#[derive(Debug)]
struct TerrainTiles {
    rows: Vec<format::TerrainTileRow>,
    heights: Vec<u8>,
    control: Vec<u8>,
}

impl CookedTerrain {
    fn bad(&self, reason: impl Into<String>) -> LoadError {
        LoadError::Invalid {
            path: self.id.clone(),
            reason: reason.into(),
        }
    }

    /// The compressed bytes of a tile: its zlib'd 16-bit heights and, with
    /// a control map, its zlib'd texels (for an inflater elsewhere: the
    /// web host's DecompressionStream).
    pub fn tile_zlib(&self, index: u32) -> Option<(&[u8], Option<&[u8]>)> {
        let t = self.tiles.rows.get(index as usize)?;
        let h = self
            .tiles
            .heights
            .get(t.offset as usize..(t.offset + t.bytes) as usize)?;
        let c =
            if self.layout.control > 0 {
                Some(self.tiles.control.get(
                    t.control_offset as usize..(t.control_offset + t.control_bytes) as usize,
                )?)
            } else {
                None
            };
        Some((h, c))
    }

    /// A tile's heights in metres from its inflated 16-bit values.
    pub fn heights_from(&self, index: u32, raw: &[u8]) -> Result<Vec<f32>, LoadError> {
        let side = self.layout.heights_side() as usize;
        if raw.len() != side * side * 2 {
            return Err(self.bad(format!("tile {index} holds the wrong number of heights")));
        }
        let (lo, hi) = self.range;
        let span = hi - lo;
        Ok(raw
            .as_chunks::<2>()
            .0
            .iter()
            .map(|v| lo + u16::from_le_bytes(*v) as f32 / 65535.0 * span)
            .collect())
    }

    /// A tile ready to draw from its inflated bytes (normals and mips
    /// built here).
    pub fn tile_from(
        &self,
        index: u32,
        heights: &[u8],
        control: Option<&[u8]>,
    ) -> Result<og_render::TerrainTile, LoadError> {
        let h = self.heights_from(index, heights)?;
        let c = match (self.layout.control, control) {
            (0, _) => None,
            (_, Some(c)) => {
                let side = self.layout.control_side() as usize;
                if c.len() != side * side * 4 {
                    return Err(
                        self.bad(format!("tile {index}'s control texels are the wrong size"))
                    );
                }
                Some(c.to_vec())
            }
            (_, None) => return Err(self.bad(format!("tile {index} has no control texels"))),
        };
        Ok(og_render::TerrainTile::build(&self.layout, index, h, c))
    }

    fn inflate(&self, index: u32) -> Result<(Vec<u8>, Option<Vec<u8>>), LoadError> {
        let (h, c) = self
            .tile_zlib(index)
            .ok_or_else(|| self.bad(format!("tile {index} is out of range")))?;
        let side = self.layout.heights_side() as usize;
        let h = miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(h, side * side * 2)
            .map_err(|_| self.bad(format!("tile {index} does not inflate")))?;
        let c = match c {
            None => None,
            Some(c) => {
                let side = self.layout.control_side() as usize;
                Some(
                    miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(c, side * side * 4)
                        .map_err(|_| {
                            self.bad(format!("tile {index}'s control does not inflate"))
                        })?,
                )
            }
        };
        Ok((h, c))
    }

    /// Inflates and decodes a tile.
    pub fn tile(&self, index: u32) -> Result<og_render::TerrainTile, LoadError> {
        let (h, c) = self.inflate(index)?;
        self.tile_from(index, &h, c.as_deref())
    }

    /// A tile's `(tile + 3)²` heights in metres (no normals, no control).
    pub fn tile_heights(&self, index: u32) -> Result<Vec<f32>, LoadError> {
        let (h, _) = self.inflate(index)?;
        self.heights_from(index, &h)
    }

    /// The whole field from its level-0 tiles: every height, and the
    /// control map at the tiles' resolution.
    pub fn field(&self) -> Result<og_render::TerrainData, LoadError> {
        let l = &self.layout;
        let (cols, rows) = (l.columns as usize, l.rows as usize);
        let t = l.tile as usize;
        let side = l.heights_side() as usize;
        let (tx, tz) = l.tiles(0);
        let mut heights = vec![0.0f32; cols * rows];
        let c = l.control as usize;
        let cside = l.control_side() as usize;
        let (cw, ch) = (c * tx as usize, c * tz as usize);
        let mut rgba = vec![0u8; cw * ch * 4];
        for z in 0..tz as usize {
            for x in 0..tx as usize {
                let index = l.index(0, x as u32, z as u32);
                let (h, cz) = self.inflate(index)?;
                let h = self.heights_from(index, &h)?;
                for j in 0..=t {
                    let r = z * t + j;
                    if r >= rows {
                        break;
                    }
                    for i in 0..=t {
                        let col = x * t + i;
                        if col >= cols {
                            break;
                        }
                        heights[r * cols + col] = h[(j + 1) * side + i + 1];
                    }
                }
                if let Some(cz) = cz {
                    for j in 0..c {
                        for i in 0..c {
                            let src = ((j + 1) * cside + i + 1) * 4;
                            let dst = ((z * c + j) * cw + x * c + i) * 4;
                            rgba[dst..dst + 4].copy_from_slice(&cz[src..src + 4]);
                        }
                    }
                }
            }
        }
        // The tiles' control texels cover whole tiles, which may reach past
        // the terrain: the map is their texels over its own extent.
        let control = (c > 0).then(|| {
            let quads = ((l.columns - 1) as f32, (l.rows - 1) as f32);
            let per = c as f32 / t as f32;
            let w = ((quads.0 * per).round() as usize).max(1);
            let hgt = ((quads.1 * per).round() as usize).max(1);
            let texel = |x: i64, y: i64, k: usize| {
                let x = x.clamp(0, cw as i64 - 1) as usize;
                let y = y.clamp(0, ch as i64 - 1) as usize;
                rgba[(y * cw + x) * 4 + k] as f32
            };
            let mut out = vec![0u8; w * hgt * 4];
            for j in 0..hgt {
                for i in 0..w {
                    let gx = (i as f32 + 0.5) / w as f32 * quads.0 * per - 0.5;
                    let gy = (j as f32 + 0.5) / hgt as f32 * quads.1 * per - 0.5;
                    let (x0, y0) = (gx.floor(), gy.floor());
                    let (fx, fy) = (gx - x0, gy - y0);
                    let (x0, y0) = (x0 as i64, y0 as i64);
                    for k in 0..4 {
                        let a = texel(x0, y0, k) + (texel(x0 + 1, y0, k) - texel(x0, y0, k)) * fx;
                        let b = texel(x0, y0 + 1, k)
                            + (texel(x0 + 1, y0 + 1, k) - texel(x0, y0 + 1, k)) * fx;
                        out[(j * w + i) * 4 + k] =
                            (a + (b - a) * fy).round().clamp(0.0, 255.0) as u8;
                    }
                }
            }
            og_render::TerrainControl {
                width: w as u32,
                height: hgt as u32,
                rgba: out.into(),
            }
        });
        let data = og_render::TerrainData {
            size: l.size,
            columns: l.columns,
            rows: l.rows,
            heights: heights.into(),
            control,
        };
        data.check().map_err(|e| self.bad(e))?;
        Ok(data)
    }
}

impl Bundle {
    /// Reads a terrain blob: its layout, layers and settings, and its tiles
    /// still compressed (each checked to lie inside the blob).
    pub fn terrain(&self, id: &str, bytes: &[u8]) -> Result<CookedTerrain, LoadError> {
        let blob = self.blob::<format::Terrain>(id, bytes)?;
        let m = one::<format::TerrainMeta>(&blob)?;
        let tiles = table::<format::TerrainTileRow>(&blob, "tiles")?;
        let layers = table::<format::TerrainLayerRow>(&blob, "layers")?;
        let strings = blob.raw("strings")?;
        let heights = blob.raw("heights")?.to_vec();
        let control = blob.raw("control")?.to_vec();
        let control_size = (m.control_width > 0).then_some((m.control_width, m.control_height));
        let layout = og_render::TerrainLayout::new(
            glam::Vec2::new(m.size_x, m.size_z),
            m.columns,
            m.rows,
            m.tile,
            control_size,
        )
        .map_err(|e| invalid(&blob, e))?;
        if layout.control != m.control || layout.levels != m.levels {
            return Err(invalid(&blob, "the tile layout does not match its cook"));
        }
        if tiles.len() != layout.tile_count() as usize {
            return Err(invalid(&blob, "the tiles do not cover the pyramid"));
        }
        for (i, t) in tiles.iter().enumerate() {
            let inside = (t.offset as usize + t.bytes as usize) <= heights.len()
                && (t.control_offset as usize + t.control_bytes as usize) <= control.len();
            if t.index != i as u32 || !inside {
                return Err(invalid(&blob, format!("tile {i} is out of place or range")));
            }
        }
        let text = |at: u32, len: u32| -> Result<Option<String>, LoadError> {
            if at == format::NONE {
                return Ok(None);
            }
            strings
                .get(at as usize..(at + len) as usize)
                .and_then(|b| std::str::from_utf8(b).ok())
                .map(|s| Some(s.to_string()))
                .ok_or_else(|| invalid(&blob, "a layer's texture name is out of range"))
        };
        let mut out = Vec::with_capacity(layers.len());
        for l in &layers {
            out.push(CookedTerrainLayer {
                albedo: text(l.albedo_at, l.albedo_len)?,
                normal: text(l.normal_at, l.normal_len)?,
                layer: og_render::TerrainLayer {
                    albedo: None,
                    normal: None,
                    color: glam::Vec3::new(l.r, l.g, l.b),
                    roughness: l.roughness,
                    tile: l.tile,
                    triplanar: l.triplanar != 0,
                    normal_strength: l.normal_strength,
                    slope: [l.slope_min, l.slope_max],
                    slope_fade: l.slope_fade,
                    altitude: [l.altitude_min, l.altitude_max],
                    altitude_fade: l.altitude_fade,
                    albedo_layer: (l.albedo_layer != format::NONE).then_some(l.albedo_layer),
                    normal_layer: (l.normal_layer != format::NONE).then_some(l.normal_layer),
                },
            });
        }
        Ok(CookedTerrain {
            id: id.to_string(),
            layout,
            range: (m.height_min, m.height_max),
            layers: out,
            albedo_array: text(m.albedo_array_at, m.albedo_array_len)?,
            normal_array: text(m.normal_array_at, m.normal_array_len)?,
            height_blend: m.height_blend,
            lod: og_render::TerrainLod {
                spacing: m.lod_spacing,
                ring: m.lod_ring,
                budget: 0,
            },
            cast_shadows: m.cast_shadows != 0,
            collision: m.collision != 0,
            tiles: std::sync::Arc::new(TerrainTiles {
                rows: tiles,
                heights,
                control,
            }),
        })
    }

    /// Reads and decodes a terrain (native).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_terrain(&self, id: &str) -> Result<CookedTerrain, LoadError> {
        self.terrain(id, &self.read(id)?)
    }

    /// Decodes an environment map blob (`type environment`): the texels
    /// inflated, the map ready for `RenderScene::add_environment_map`.
    pub fn environment(
        &self,
        id: &str,
        bytes: &[u8],
    ) -> Result<og_render::EnvironmentMapData, LoadError> {
        let blob = self.blob::<format::EnvironmentMap>(id, bytes)?;
        let meta = one::<format::EnvironmentMeta>(&blob)?;
        if meta.levels != og_render::ENVIRONMENT_MAP_LEVELS {
            return Err(invalid(
                &blob,
                format!(
                    "{} levels; this engine's environment maps have {}",
                    meta.levels,
                    og_render::ENVIRONMENT_MAP_LEVELS
                ),
            ));
        }
        let want = og_render::EnvironmentMapData::texel_count(meta.size);
        if meta.texel_count as usize != want {
            return Err(invalid(
                &blob,
                format!(
                    "a {}² map holds {want} texels, not {}",
                    meta.size, meta.texel_count
                ),
            ));
        }
        let bytes =
            miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(blob.raw("texels")?, want * 4)
                .map_err(|e| invalid(&blob, format!("texels: {e:?}")))?;
        if bytes.len() != want * 4 {
            return Err(invalid(&blob, "texels: truncated"));
        }
        let texels = bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|b| u32::from_le_bytes(*b))
            .collect();
        let sh = table::<format::EnvironmentShRow>(&blob, "irradiance")?;
        if sh.len() != 9 {
            return Err(invalid(
                &blob,
                format!("{} irradiance rows, not 9", sh.len()),
            ));
        }
        let map = og_render::EnvironmentMapData {
            size: meta.size,
            texels,
            irradiance: std::array::from_fn(|k| [sh[k].r, sh[k].g, sh[k].b]),
        };
        map.validate().map_err(|e| invalid(&blob, e))?;
        Ok(map)
    }

    /// Reads and decodes an environment map (native).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_environment(&self, id: &str) -> Result<og_render::EnvironmentMapData, LoadError> {
        self.environment(id, &self.read(id)?)
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
