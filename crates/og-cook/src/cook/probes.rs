//! Baked reflection probes (`og bake probes`, `docs/design.md` §2.6): a
//! probe's capture, read back from the native renderer, written as a
//! `type environment` blob under the cook cache (`cooked/probes/<key>.ogb`)
//! and recorded in the manifest as an asset of kind `probe` with no source,
//! which the cook keeps while its file is there. The runtime loads it as
//! the environment map of the og.ReflectionProbe whose key it is
//! (`og_runtime`'s `probes.rs`: the probe's position, box and face size),
//! and draws the capture itself when there is none.

use super::{COOKER_VERSION, CookError, environment::environment_blob, hash_bytes, stamp};
use crate::format::BlobKind;
use crate::runtime::{MANIFEST, Manifest, ManifestAsset, ManifestOutput};
use std::path::{Path, PathBuf};

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
        let path = write_probe(&dir, id, size, &levels).unwrap();
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
        let _ = std::fs::remove_dir_all(&dir);
    }
}
