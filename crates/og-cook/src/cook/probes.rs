//! Baked reflection probes (`og bake probes`, `docs/design.md` §2.6): a
//! probe's capture, read back from the native renderer, written as a
//! `type environment` blob under the cook cache (`cooked/probes/<key>.ogb`)
//! and recorded in the manifest as an asset of kind `probe` with no source,
//! which the cook keeps while its file is there. The runtime loads it as
//! the environment map of the og.ReflectionProbe whose key it is
//! (`og_runtime`'s `probes.rs`: the probe's position, box and face size),
//! and draws the capture itself when there is none.
//!
//! The key includes the probe's room (what its capture draws, by value),
//! so a changed room is never served an old bake. Beside each blob,
//! `cooked/probes/<key>.ogt` records what the bake was taken of: the probe,
//! the commands that reached it, the model assets drawn in its room with
//! their cooked hashes, and the hash of the game's source when its code
//! built part of the room. [`status`] reads them back for `og doctor
//! --project`: a bake whose assets or code changed since is stale (the
//! runtime no longer samples it and draws the capture on first use).

use super::{COOKER_VERSION, CookError, environment::environment_blob, hash_bytes, stamp};
use crate::format::BlobKind;
use crate::runtime::{MANIFEST, Manifest, ManifestAsset, ManifestOutput};
use og_core::ogt::{Document, Entry, Value};
use std::path::{Path, PathBuf};

/// What a bake was taken of (`cooked/probes/<key>.ogt`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProbeRecord {
    /// The probe's position and box (world).
    pub position: [f32; 3],
    pub size: [f32; 3],
    /// Its room's content hash (`og_render::ProbeRoom::hash`, hex).
    pub room: String,
    /// The `og bake probes --commands` that reached it.
    pub commands: String,
    /// The model assets drawn in the room, with their cooked output hash.
    pub assets: Vec<(String, u64)>,
    /// The game source's hash, when the game's code built part of the room.
    pub code: Option<u64>,
}

impl ProbeRecord {
    pub fn to_text(&self) -> String {
        let s = |v: String| Value::String(v);
        let three = |k: &str, v: [f32; 3]| {
            v.iter()
                .fold(Entry::new(k), |e, x| e.with(s(format!("{x}"))))
        };
        let mut doc = Document::new("probe", 1);
        doc.body.push(three("position", self.position));
        doc.body.push(three("box", self.size));
        doc.body.push(Entry::new("room").with(s(self.room.clone())));
        doc.body
            .push(Entry::new("commands").with(s(self.commands.clone())));
        for (id, hash) in &self.assets {
            doc.body.push(
                Entry::new("asset")
                    .with(s(id.clone()))
                    .with(s(format!("{hash:016x}"))),
            );
        }
        if let Some(c) = self.code {
            doc.body
                .push(Entry::new("code").with(s(format!("{c:016x}"))));
        }
        doc.to_text().expect("keys are identifiers")
    }

    pub fn parse(text: &str) -> Result<ProbeRecord, String> {
        let doc = Document::parse(text).map_err(|e| e.to_string())?;
        if doc.ty != "probe" || doc.version != 1 {
            return Err(format!("not a probe record ('{}' {})", doc.ty, doc.version));
        }
        let text_of = |k: &str, i: usize| -> Option<String> {
            doc.body.get(k)?.string(i).ok().map(str::to_string)
        };
        let three = |k: &str| -> [f32; 3] {
            std::array::from_fn(|i| text_of(k, i).and_then(|t| t.parse().ok()).unwrap_or(0.0))
        };
        let hex = |t: &str| u64::from_str_radix(t, 16).map_err(|_| format!("'{t}' is not hex"));
        let mut assets = Vec::new();
        for e in doc.body.get_all("asset") {
            let id = e.string(0).map_err(|e| e.to_string())?.to_string();
            let hash = hex(e.string(1).map_err(|e| e.to_string())?)?;
            assets.push((id, hash));
        }
        Ok(ProbeRecord {
            position: three("position"),
            size: three("box"),
            room: text_of("room", 0).unwrap_or_default(),
            commands: text_of("commands", 0).unwrap_or_default(),
            assets,
            code: text_of("code", 0).map(|t| hex(&t)).transpose()?,
        })
    }
}

/// The record beside the bake `id` (`probes/<key>`).
pub fn record_path(root: &Path, id: &str) -> PathBuf {
    root.join("cooked").join(format!("{id}.ogt"))
}

/// The project's stale probe bakes, as (asset id, why): a bake without a
/// record (from before rooms were part of the key: never sampled), one of
/// whose assets changed or is gone, or whose room the game's code built
/// part of and `code` (the source's hash now) differs.
pub fn status(root: &Path, code: u64) -> Vec<(String, String)> {
    let cooked = root.join("cooked");
    let Ok(text) = std::fs::read_to_string(cooked.join(MANIFEST)) else {
        return Vec::new();
    };
    let Ok(manifest) = Manifest::parse(&text) else {
        return Vec::new();
    };
    let hash_of = |id: &str| manifest.outputs().find(|o| o.id == id).map(|o| o.hash);
    let mut out = Vec::new();
    for a in manifest.assets.iter().filter(|a| a.kind == KIND) {
        let id = a.source.clone();
        let record = std::fs::read_to_string(record_path(root, &id))
            .map_err(|e| e.to_string())
            .and_then(|t| ProbeRecord::parse(&t));
        let reason = match record {
            Err(_) => Some(format!(
                "{id} has no record of its room (baked before rooms keyed bakes): no probe samples it"
            )),
            Ok(r) => {
                let at = format!(
                    "the probe at ({}, {}, {})",
                    r.position[0], r.position[1], r.position[2]
                );
                r.assets
                    .iter()
                    .find_map(|(asset, h)| match hash_of(asset) {
                        None => Some(format!("{id} ({at}): {asset} is no longer cooked")),
                        Some(now) if now != *h => {
                            Some(format!("{id} ({at}): {asset} changed since its bake"))
                        }
                        _ => None,
                    })
                    .or_else(|| {
                        r.code.filter(|&c| c != code).map(|_| {
                            format!("{id} ({at}): the game's code, which builds part of its room, changed since its bake")
                        })
                    })
            }
        };
        out.extend(reason.map(|r| (id, r)));
    }
    out
}

/// The manifest kind of a baked probe.
pub const KIND: &str = "probe";

/// Writes the capture of the probe whose asset id is `id`
/// (`probes/<16 hex digits>`): `levels` are its prefiltered levels of
/// face size `size` (`og_render::bake::PrefilteredCube`'s layout), made an
/// environment map as given (irradiance projected from level 0). Returns
/// the file written.
pub fn write_probe(
    root: &Path,
    id: &str,
    size: u32,
    levels: &[Vec<f32>],
    record: &ProbeRecord,
) -> Result<PathBuf, CookError> {
    let key = id
        .strip_prefix("probes/")
        .filter(|k| k.len() == 16 && k.bytes().all(|b| b.is_ascii_hexdigit()))
        .ok_or_else(|| CookError(format!("'{id}' is not a probe id (probes/<16 hex digits>)")))?;
    let map = og_render::bake::from_prefiltered(
        &og_render::bake::PrefilteredCube { size, levels },
        &og_render::bake::BakeDesc {
            size,
            ..Default::default()
        },
    )
    .map_err(|e| CookError(format!("{id}: {e}")))?;
    let st = stamp(&[b"probe", key.as_bytes()]);
    let (blob, _, _) = environment_blob(&map, (size, size), 1.0, 0.0, 0, st);
    let cooked = root.join("cooked");
    let file = format!("probes/{key}.ogb");
    let path = cooked.join(&file);
    std::fs::create_dir_all(path.parent().expect("under cooked/"))
        .map_err(|e| CookError(format!("{}: {e}", path.display())))?;
    std::fs::write(&path, &blob).map_err(|e| CookError(format!("{}: {e}", path.display())))?;
    let record_file = record_path(root, id);
    std::fs::write(&record_file, record.to_text())
        .map_err(|e| CookError(format!("{}: {e}", record_file.display())))?;
    let manifest_path = cooked.join(MANIFEST);
    let mut manifest = match std::fs::read_to_string(&manifest_path) {
        Ok(t) => Manifest::parse(&t)
            .map_err(|e| CookError(format!("{}: {e}", manifest_path.display())))?,
        Err(_) => Manifest {
            cooker: COOKER_VERSION.to_string(),
            ..Default::default()
        },
    };
    manifest.assets.retain(|a| a.source != id);
    manifest.assets.push(ManifestAsset {
        source: id.to_string(),
        kind: KIND.to_string(),
        source_hash: 0,
        source_bytes: 0,
        outputs: vec![ManifestOutput {
            id: id.to_string(),
            ty: crate::format::EnvironmentMap::NAME.to_string(),
            file,
            stamp: st,
            hash: hash_bytes(&blob),
            bytes: blob.len() as u64,
            group: None,
        }],
    });
    manifest.assets.sort_by(|a, b| a.source.cmp(&b.source));
    std::fs::write(&manifest_path, manifest.to_text())
        .map_err(|e| CookError(format!("{}: {e}", manifest_path.display())))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A written probe is in the manifest as an og.EnvironmentMap output
    /// whose bytes load back as a map of its size, and a cook of the
    /// project keeps it.
    #[test]
    fn a_baked_probe_loads_back_and_survives_a_cook() {
        let dir = std::env::temp_dir().join(format!("og-probe-bake-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("assets")).unwrap();
        let size = 16;
        let levels: Vec<Vec<f32>> = (0..og_render::ENVIRONMENT_MAP_LEVELS)
            .map(|m| {
                let n = (size >> m).max(1) as usize;
                vec![0.25; 6 * n * n * 3]
            })
            .collect();
        let id = "probes/00112233445566aa";
        let record = ProbeRecord {
            position: [1.0, 1.5, -2.0],
            size: [10.0, 4.0, 10.0],
            room: "00000000000000ff".into(),
            commands: "0:station:reflections".into(),
            assets: vec![("models/tree.glb".into(), 0xabc)],
            code: Some(7),
        };
        let path = write_probe(&dir, id, size, &levels, &record).unwrap();
        let read = std::fs::read_to_string(record_path(&dir, id)).unwrap();
        assert_eq!(ProbeRecord::parse(&read).unwrap(), record);
        assert!(path.is_file());
        let text = std::fs::read_to_string(dir.join("cooked").join(MANIFEST)).unwrap();
        let m = Manifest::parse(&text).unwrap();
        let out = m.outputs().find(|o| o.id == id).expect("in the manifest");
        assert_eq!(out.ty, crate::format::EnvironmentMap::NAME);
        crate::cook::cook_project(&dir, &Default::default()).unwrap();
        let bundle = crate::runtime::Bundle::open(&dir.join("cooked")).unwrap();
        let bytes = bundle.read(id).expect("kept by the cook");
        let map = bundle.environment(id, bytes.as_slice()).unwrap();
        assert_eq!(map.size, size);
        // Its tree is not cooked here: stale; and with the code changed too.
        let st = status(&dir, 7);
        assert_eq!(st.len(), 1);
        assert!(
            st[0].1.contains("models/tree.glb is no longer cooked"),
            "{st:?}"
        );
        // Without assets, only the code counts.
        let code_only = ProbeRecord {
            assets: Vec::new(),
            ..record
        };
        std::fs::write(record_path(&dir, id), code_only.to_text()).unwrap();
        assert!(status(&dir, 7).is_empty());
        assert!(status(&dir, 8)[0].1.contains("the game's code"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
