//! Streaming groups (phase 3): a scene names groups of assets that load
//! and unload together at runtime, instead of all at boot.
//!
//! A scene's TS `streams` map supplies canonical content refs and a tagged
//! trigger (`manual`, `distance`, `volume` or `cell`). Distance triggers carry
//! an anchor/radius and optional unload distance (default radius × 1.25).
//! Volume triggers name an entity. Higher priority loads first/unloads last.
//!
//! A sub-scene (placed with `scene: ref(id)`) can declare a group with
//! `trigger: {kind: 'cell'}`: the group
//! holds the sub-scene's own blob beside the assets it names, so the blob
//! leaves the boot bundle, and the runtime loads the group ahead of the
//! anchors nearing the cells that place the sub-scene (og-runtime's
//! `stream::cells`). A scene declares at most one, and it has no other
//! trigger.
//!
//! The cook reads every scene's groups before it cooks: an asset belongs to
//! at most one group, and every asset a group names must be cooked. Each
//! group's cooked files are packed into `cooked/stream/<group>.ogb` (one
//! download), the manifest lists the group with its trigger, pack and the
//! memory its assets take once loaded, and every output of a grouped asset
//! names its group. A grouped glTF without skins also gets a `#proxy`: its
//! coarsest LOD without textures, in the boot bundle, drawn while the group
//! is not resident. A grouped texture array, flipbook (an `array true`
//! sheet's array and layers come with it), environment map, palette, look
//! or colour LUT (`.cube`) has no stand-in: the runtime loads it once its
//! group is resident and lets it go when the group unloads (og-runtime's
//! `stream`). Assets in
//! no group are the first group: what a page downloads and precaches
//! before the game starts.

use crate::cook::formats::scene::Document;
use crate::cook::source_schemas::world::scene::StreamTriggerInput;
use crate::format::{self, BlobKind};
use crate::runtime::{ManifestStream, StreamSound, StreamTrigger};
use og_blob::BlobWriter;
use std::collections::BTreeMap;

/// One group a scene declares.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GroupDecl {
    pub name: String,
    pub scene: String,
    pub assets: Vec<String>,
    pub trigger: StreamTrigger,
    pub priority: i32,
}

fn valid_name(n: &str) -> bool {
    !n.is_empty()
        && n.len() <= 64
        && n.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// The groups a scene declares, checked: their assets cooked (`known`),
/// a trigger of distance or volume (or none: manual).
pub(crate) fn groups_of(
    scene: &str,
    doc: &Document,
    known: &dyn Fn(&str) -> bool,
) -> Result<Vec<GroupDecl>, String> {
    let mut out = Vec::new();
    for (name, input) in &doc.scene.streams {
        if !valid_name(name) {
            return Err(format!(
                "stream {name:?}: a group's name is 1 to 64 letters, digits, '-' and '_' (it names its pack file)"
            ));
        }
        let mut assets = Vec::new();
        for reference in &input.assets {
            let id = &reference.id;
            if !known(id) {
                return Err(format!(
                    "stream {name:?} names {id}, which is not a cooked asset of this project"
                ));
            }
            if !assets.contains(id) {
                assets.push(id.clone());
            }
        }
        let trigger = match input
            .trigger
            .as_ref()
            .unwrap_or(&StreamTriggerInput::Manual)
        {
            StreamTriggerInput::Manual => StreamTrigger::Manual,
            StreamTriggerInput::Cell => StreamTrigger::Cell,
            StreamTriggerInput::Volume { entity } => StreamTrigger::Volume(entity.clone()),
            StreamTriggerInput::Distance {
                anchor,
                radius,
                unload,
            } => {
                if !anchor.iter().all(|n| n.is_finite()) || !radius.is_finite() || *radius <= 0.0 {
                    return Err(format!(
                        "stream {name:?}: distance needs a finite anchor and positive radius"
                    ));
                }
                let unload = unload.unwrap_or(radius * 1.25);
                if !unload.is_finite() || unload < *radius {
                    return Err(format!(
                        "stream {name:?}: unload must be finite and at least radius ({radius})"
                    ));
                }
                StreamTrigger::Distance {
                    anchor: *anchor,
                    radius: *radius,
                    unload,
                }
            }
        };
        // A cell group holds its own scene blob, even with no named assets.
        if assets.is_empty() && !matches!(trigger, StreamTrigger::Cell) {
            return Err(format!("stream {name:?} holds no asset"));
        }
        out.push(GroupDecl {
            name: name.clone(),
            scene: scene.to_string(),
            assets,
            trigger,
            priority: input.priority.unwrap_or(0),
        });
    }
    Ok(out)
}

/// A group's pack: its files (paths under `cooked/`, sorted) and their bytes.
pub(crate) fn pack(files: &[(String, Vec<u8>)], stamp: u64) -> Vec<u8> {
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
pub(crate) fn memory_of(ty: &str, bytes: &[u8]) -> u64 {
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
pub(crate) fn sound_of(id: &str, bytes: &[u8]) -> Option<StreamSound> {
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
pub(crate) fn manifest_entry(
    g: &GroupDecl,
    file: String,
    bytes: u64,
    hash: u64,
    integrity: String,
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
        integrity,
        memory,
        assets: g.assets.clone(),
        sounds,
    }
}

/// Every scene's groups, by name, and each asset's group. `Err` names the
/// scene and why (an asset in two groups, a name taken twice).
pub(crate) type Groups = (BTreeMap<String, GroupDecl>, BTreeMap<String, String>);
