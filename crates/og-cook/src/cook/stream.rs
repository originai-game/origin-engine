//! Streaming groups (phase 3): a scene names groups of assets that load
//! and unload together at runtime, instead of all at boot.
//!
//! ```text
//! type scene
//! version 1
//! ---
//! name "Valley"
//! stream "village" {
//!   asset "models/house.glb"     # every asset the group holds (a glTF brings its textures)
//!   asset "textures/roof.png"
//!   distance 120 0 40 150        # anchor x y z and radius: loads while the camera is within it
//!   unload 190                   # unloads past this distance from the anchor (default: radius × 1.25)
//!   priority 2                   # higher loads first and unloads last (default 0)
//! }
//! stream "cave" {
//!   asset "audio/drips.wav"
//!   volume "cave-mouth"          # loads while the camera is inside this entity's box
//! }
//! stream "boss" {
//!   asset "models/boss.glb"      # neither: loads on og.stream.request only
//! }
//! ```
//!
//! The cook reads every scene's groups before it cooks: an asset belongs to
//! at most one group, and every asset a group names must be cooked. Each
//! group's cooked files are packed into `cooked/stream/<group>.ogb` (one
//! download), the manifest lists the group with its trigger, pack and the
//! memory its assets take once loaded, and every output of a grouped asset
//! names its group. A grouped glTF without skins also gets a `#proxy`: its
//! coarsest LOD without textures, in the boot bundle, drawn while the group
//! is not resident. A grouped texture array, environment map, palette,
//! look or colour LUT (`.cube`) has no stand-in: the runtime loads it once
//! its group is resident and lets it go when the group unloads
//! (og-runtime's `stream`). Assets in
//! no group are the first group: what a page downloads and precaches
//! before the game starts.

use crate::format::{self, BlobKind};
use crate::runtime::{ManifestStream, StreamSound, StreamTrigger};
use og_blob::BlobWriter;
use og_core::ogt::Document;
use std::collections::BTreeMap;

/// One group a scene declares.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct GroupDecl {
    pub name: String,
    pub scene: String,
    pub assets: Vec<String>,
    pub trigger: StreamTrigger,
    pub priority: i32,
}

const KEYS: [&str; 5] = ["asset", "distance", "unload", "volume", "priority"];

fn valid_name(n: &str) -> bool {
    !n.is_empty()
        && n.len() <= 64
        && n.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// The groups a scene declares, checked: their assets cooked (`known`),
/// a trigger of distance or volume (or none: manual).
pub(super) fn groups_of(
    scene: &str,
    text: &str,
    known: &dyn Fn(&str) -> bool,
) -> Result<Vec<GroupDecl>, String> {
    let Ok(doc) = Document::parse(text) else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for e in doc.body.get_all("stream") {
        let name = e
            .string(0)
            .map_err(|_| {
                format!(
                    "line {}: a stream group is stream \"<name>\" {{ ... }}",
                    e.line
                )
            })?
            .to_string();
        if !valid_name(&name) {
            return Err(format!(
                "line {}: stream \"{name}\": a group's name is 1 to 64 letters, digits, '-' and '_' (it names its pack file)",
                e.line
            ));
        }
        let Some(b) = &e.block else {
            return Err(format!(
                "line {}: stream \"{name}\" needs a block of its assets",
                e.line
            ));
        };
        for k in &b.entries {
            if !KEYS.contains(&k.key.as_str()) {
                return Err(format!(
                    "line {}: '{}' is not a stream entry: {}",
                    k.line,
                    k.key,
                    KEYS.join(", ")
                ));
            }
        }
        let mut assets = Vec::new();
        for a in b.get_all("asset") {
            let id = crate::runtime::asset_id(
                a.string(0)
                    .map_err(|_| format!("line {}: asset \"<path under assets/>\"", a.line))?,
            )
            .to_string();
            if !known(&id) {
                return Err(format!(
                    "line {}: stream \"{name}\" names {id}, which is not an asset under assets/",
                    a.line
                ));
            }
            if !assets.contains(&id) {
                assets.push(id);
            }
        }
        if assets.is_empty() {
            return Err(format!("line {}: stream \"{name}\" holds no asset", e.line));
        }
        let number = |key: &str| -> Result<Option<Vec<f32>>, String> {
            match b.get(key) {
                None => Ok(None),
                Some(x) => x
                    .values
                    .iter()
                    .map(|v| {
                        v.as_number()
                            .map(|n| n.as_f32())
                            .filter(|f| f.is_finite())
                            .ok_or_else(|| format!("line {}: '{key}' takes numbers", x.line))
                    })
                    .collect::<Result<Vec<f32>, String>>()
                    .map(Some),
            }
        };
        let trigger = match (number("distance")?, b.get("volume")) {
            (Some(_), Some(v)) => {
                return Err(format!(
                    "line {}: stream \"{name}\" has both distance and volume; give it one trigger",
                    v.line
                ));
            }
            (Some(d), None) => {
                if d.len() != 4 || d[3] <= 0.0 {
                    return Err(format!(
                        "stream \"{name}\": distance takes the anchor's x y z and a positive radius"
                    ));
                }
                let unload = match number("unload")? {
                    None => d[3] * 1.25,
                    Some(u) if u.len() == 1 && u[0] >= d[3] => u[0],
                    Some(_) => {
                        return Err(format!(
                            "stream \"{name}\": unload takes one distance at least the radius ({})",
                            d[3]
                        ));
                    }
                };
                StreamTrigger::Distance {
                    anchor: [d[0], d[1], d[2]],
                    radius: d[3],
                    unload,
                }
            }
            (None, Some(v)) => StreamTrigger::Volume(
                v.string(0)
                    .map_err(|_| format!("line {}: volume \"<entity name>\"", v.line))?
                    .to_string(),
            ),
            (None, None) => StreamTrigger::Manual,
        };
        let priority = match number("priority")? {
            None => 0,
            Some(p) if p.len() == 1 && p[0].fract() == 0.0 => p[0] as i32,
            Some(_) => {
                return Err(format!(
                    "stream \"{name}\": priority takes one whole number"
                ));
            }
        };
        out.push(GroupDecl {
            name,
            scene: scene.to_string(),
            assets,
            trigger,
            priority,
        });
    }
    Ok(out)
}

/// A group's pack: its files (paths under `cooked/`, sorted) and their bytes.
pub(super) fn pack(files: &[(String, Vec<u8>)], stamp: u64) -> Vec<u8> {
    let mut names = String::new();
    let mut data: Vec<u8> = Vec::new();
    let mut rows = Vec::with_capacity(files.len());
    for (path, bytes) in files {
        while !data.len().is_multiple_of(16) {
            data.push(0);
        }
        rows.push(format::StreamFileRow {
            name_at: names.len() as u32,
            name_len: path.len() as u32,
            data_at: data.len() as u32,
            data_len: bytes.len() as u32,
        });
        names.push_str(path);
        data.extend_from_slice(bytes);
    }
    let mut w = BlobWriter::new(
        format::StreamPack::type_hash(),
        format::StreamPack::schema_version(),
        stamp,
    );
    w.table(
        "meta",
        &[format::StreamPackMeta {
            files: rows.len() as u32,
        }],
        format::row_hash::<format::StreamPackMeta>(),
    )
    .table("files", &rows, format::row_hash::<format::StreamFileRow>())
    .raw("names", names.into_bytes(), 1)
    .raw("data", data, 16);
    w.finish()
}

/// The memory a cooked output takes once loaded: a texture's RGBA8 with
/// mips, a model's vertices (48 bytes each) and indices, a sound's
/// decoded samples (or its QOA when it streams), else its bytes.
pub(super) fn memory_of(ty: &str, bytes: &[u8]) -> u64 {
    let words = og_blob::AlignedBytes::from_slice(bytes);
    let fallback = bytes.len() as u64;
    let Ok(blob) = og_blob::Blob::parse(words.as_slice(), ty) else {
        return fallback;
    };
    fn rows<T: og_core::Reflect + bytemuck::Pod>(blob: &og_blob::Blob, name: &str) -> Vec<T> {
        blob.table::<T>(name, format::row_hash::<T>())
            .map(|r| r.into_owned())
            .unwrap_or_default()
    }
    match ty {
        "og.Texture" => rows::<format::TextureMeta>(&blob, "meta")
            .first()
            .map_or(fallback, |m| m.width as u64 * m.height as u64 * 4 * 4 / 3),
        "og.Model" => rows::<format::PrimitiveRow>(&blob, "primitives")
            .iter()
            .map(|p| p.vertex_count as u64 * 48 + p.index_count as u64 * 4)
            .sum(),
        // The cube's RGB9E5 texels, every level (zlib'd in the blob).
        "og.EnvironmentMap" => rows::<format::EnvironmentMeta>(&blob, "meta")
            .first()
            .map_or(fallback, |m| m.texel_count as u64 * 4),
        // A 256 x 1 RGBA8 texture.
        "og.Palette" => 256 * 4,
        // A size³ RGBA8 3D texture.
        "og.Lut" => rows::<format::LutMeta>(&blob, "meta")
            .first()
            .map_or(fallback, |m| u64::from(m.size).pow(3) * 4),
        "og.Sound" => rows::<format::SoundMeta>(&blob, "meta")
            .first()
            .map_or(fallback, |m| {
                if m.frames as f64 <= 6.0 * m.sample_rate as f64 {
                    m.frames as u64 * m.channels as u64 * 2
                } else {
                    fallback
                }
            }),
        _ => fallback,
    }
}

/// A cooked sound's rate, channels and frames (`None`: not a sound blob).
pub(super) fn sound_of(id: &str, bytes: &[u8]) -> Option<StreamSound> {
    let words = og_blob::AlignedBytes::from_slice(bytes);
    let blob = og_blob::Blob::parse(words.as_slice(), "og.Sound").ok()?;
    let m = blob
        .table::<format::SoundMeta>("meta", format::row_hash::<format::SoundMeta>())
        .ok()?
        .first()
        .copied()?;
    Some(StreamSound {
        id: id.to_string(),
        sample_rate: m.sample_rate,
        channels: m.channels,
        frames: u64::from(m.frames),
    })
}

/// The manifest's entry for a group.
pub(super) fn manifest_entry(
    g: &GroupDecl,
    file: String,
    bytes: u64,
    hash: u64,
    memory: u64,
    sounds: Vec<StreamSound>,
) -> ManifestStream {
    ManifestStream {
        name: g.name.clone(),
        scene: g.scene.clone(),
        trigger: g.trigger.clone(),
        priority: g.priority,
        file,
        bytes,
        hash,
        memory,
        assets: g.assets.clone(),
        sounds,
    }
}

/// Every scene's groups, by name, and each asset's group. `Err` names the
/// scene and why (an asset in two groups, a name taken twice).
pub(super) type Groups = (BTreeMap<String, GroupDecl>, BTreeMap<String, String>);
