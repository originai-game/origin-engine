//! Preserve the prefracture result before the output-based model stamp is made.
//! Dependencies are discovered by the fracture, then checked by content on reuse.
use super::fracture::Fractured;
use og_blob::{Blob, BlobWriter};
use og_core::hash::hash_bytes;
use og_destruct::FractureAsset;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

pub(super) fn run(
    root: &Path,
    rel: &str,
    stamp: u64,
    dir: &Path,
    fracture: impl FnOnce() -> Result<Fractured, String>,
) -> Result<(Fractured, bool), String> {
    let path = root
        .join(".og/fracture")
        .join(format!("{:016x}.ogb", hash_bytes(rel.as_bytes())));
    if let Some(result) = read(&path, stamp, dir) {
        return Ok((result, true));
    }
    let result = fracture()?;
    // This is disposable acceleration data, never a shipped manifest output.
    let _ = publish(&path, stamp, &result);
    Ok((result, false))
}

fn cache_type() -> u64 {
    hash_bytes(b"og.PrefractureCache")
}

fn read(path: &Path, stamp: u64, dir: &Path) -> Option<Fractured> {
    let bytes = std::fs::read(path).ok()?;
    let blob = Blob::parse(&bytes, "prefracture cache").ok()?;
    let h = blob.header();
    if (h.type_hash, h.schema_version, h.cook_stamp) != (cache_type(), 1, stamp) {
        return None;
    }
    let glb = blob.raw("glb").ok()?;
    let asset = blob.raw("asset").ok()?;
    let dependencies = blob.raw("dependencies").ok()?;
    if blob.raw("hash").ok()? != super::stamp(&[glb, asset, dependencies]).to_le_bytes() {
        return None;
    }
    let dependencies: Vec<(String, u64)> = serde_json::from_slice(dependencies).ok()?;
    let mut read = Vec::with_capacity(dependencies.len());
    for (file, hash) in dependencies {
        let bytes = std::fs::read(dir.join(&file)).ok()?;
        if hash_bytes(&bytes) != hash {
            return None;
        }
        read.push((file, bytes));
    }
    Some(Fractured {
        glb: glb.to_vec(),
        asset: FractureAsset::from_bytes(asset).ok()?,
        read,
    })
}

fn publish(path: &Path, stamp: u64, result: &Fractured) -> std::io::Result<()> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let dependencies: Vec<_> = result
        .read
        .iter()
        .map(|(p, b)| (p, hash_bytes(b)))
        .collect();
    let dependencies = serde_json::to_vec(&dependencies).map_err(std::io::Error::other)?;
    let asset = result.asset.to_bytes();
    let mut blob = BlobWriter::new(cache_type(), 1, stamp);
    blob.raw(
        "hash",
        super::stamp(&[&result.glb, &asset, &dependencies]).to_le_bytes(),
        1,
    )
    .raw("dependencies", dependencies, 1)
    .raw("asset", asset, 16)
    .raw("glb", result.glb.as_slice(), 16);
    std::fs::create_dir_all(path.parent().expect("cache directory"))?;
    let temp = path.with_extension(format!(
        "{}.{}.tmp",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)?;
    let written = (|| {
        file.write_all(&blob.finish())?;
        drop(file);
        std::fs::rename(&temp, path)
    })();
    if written.is_err() {
        let _ = std::fs::remove_file(temp);
    }
    written
}
