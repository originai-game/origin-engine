//! Content-keyed prefracture actions, with a discovered dependency list.
use super::{cache, fracture::Fractured};
use og_blob::{Blob, BlobWriter};
use og_core::hash::hash_bytes;
use og_physics_data::fracture::FractureAsset;
use std::path::Path;

pub(crate) fn run(
    directory: Option<&Path>,
    input: &str,
    stamp: u64,
    dir: &Path,
    fracture: impl FnOnce() -> Result<Fractured, String>,
) -> Result<(Fractured, bool), String> {
    let store = directory.map(|p| cache::Store::new(p.to_owned()));
    // Discovery records contain only names. The action key also covers their
    // current contents, so equal models in different directories are safe.
    let discovery = cache::key("fracture-dependencies", &[input.as_bytes()]);
    if let Some(store) = &store
        && let Some((names, _)) = store.action(&discovery)
        && let Ok(names) = serde_json::from_slice::<Vec<String>>(&names)
        && let Ok(read) = names
            .into_iter()
            .map(|name| std::fs::read(dir.join(&name)).map(|bytes| (name, bytes)))
            .collect::<Result<Vec<_>, _>>()
        && let Some((bytes, _)) = store.action(&action_key(input, &read))
        && let Some(result) = decode(&bytes, stamp, read)
    {
        return Ok((result, true));
    }
    let result = fracture()?;
    if let Some(store) = &store {
        let mut blob = BlobWriter::new(hash_bytes(b"og.PrefractureCache"), 1, stamp);
        blob.raw("asset", result.asset.to_bytes(), 16)
            .raw("glb", result.glb.as_slice(), 16);
        let _ = store.put_action(&action_key(input, &result.read), &blob.finish(), &[]);
        let names: Vec<_> = result.read.iter().map(|(name, _)| name).collect();
        if let Ok(names) = serde_json::to_vec(&names) {
            let _ = store.put_action(&discovery, &names, &[]);
        }
    }
    Ok((result, false))
}

fn action_key(input: &str, read: &[(String, Vec<u8>)]) -> String {
    let mut dependencies: Vec<_> = read
        .iter()
        .map(|(name, bytes)| (name, blake3::hash(bytes).to_hex().to_string()))
        .collect();
    dependencies.sort();
    cache::key(
        "fracture-result",
        &[
            input.as_bytes(),
            &serde_json::to_vec(&dependencies).expect("dependency strings"),
        ],
    )
}

fn decode(bytes: &[u8], stamp: u64, read: Vec<(String, Vec<u8>)>) -> Option<Fractured> {
    let blob = Blob::parse(bytes, "prefracture cache").ok()?;
    let h = blob.header();
    if (h.type_hash, h.schema_version, h.cook_stamp)
        != (hash_bytes(b"og.PrefractureCache"), 1, stamp)
    {
        return None;
    }
    Some(Fractured {
        glb: blob.raw("glb").ok()?.to_vec(),
        asset: FractureAsset::from_bytes(blob.raw("asset").ok()?).ok()?,
        read,
    })
}
