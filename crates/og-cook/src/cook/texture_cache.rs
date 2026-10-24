//! Best-effort, shared results for ordinary texture work, not project manifests.
use super::{CookError, Planned, Work, format};
use og_blob::{Blob, BlobWriter};
use og_core::hash::hash_bytes;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

pub(super) fn default_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("OG_TEXTURE_CACHE") {
        return (!path.is_empty() && path != "off").then(|| PathBuf::from(path));
    }
    std::env::var_os("XDG_CACHE_HOME")
        .filter(|p| !p.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("LOCALAPPDATA")
                .filter(|p| !p.is_empty())
                .map(PathBuf::from)
        })
        .or_else(|| {
            std::env::var_os("HOME")
                .filter(|p| !p.is_empty())
                .map(|p| PathBuf::from(p).join(".cache"))
        })
        .map(|p| p.join("origingame/texture-results-v1"))
}

pub(super) fn run(
    directory: Option<&Path>,
    p: &Planned,
    cook: impl FnOnce() -> Result<(Vec<u8>, Vec<String>), CookError>,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let eligible = matches!(p.work, Some(Work::Texture(..) | Work::GltfTexture(..)));
    let path = directory.filter(|_| eligible).and_then(|dir| {
        let (ty, schema) = format::expected_header(p.ty)?;
        Some(dir.join(format!("{:016x}-{ty:016x}-{schema}.ogb", p.stamp)))
    });
    if let Some(bytes) = path.as_deref().and_then(|path| read(path, p)) {
        return Ok((bytes, vec![format!("{}: shared texture cache hit", p.id)]));
    }
    let result = cook()?;
    if let Some(path) = path {
        // Cache I/O must never turn a successful cook into a failure.
        let _ = publish(&path, p.stamp, &result.0);
    }
    Ok(result)
}

fn cache_type() -> u64 {
    hash_bytes(b"og.TextureCache")
}

fn read(path: &Path, p: &Planned) -> Option<Vec<u8>> {
    let bytes = std::fs::read(path).ok()?;
    let entry = Blob::parse(&bytes, "texture cache").ok()?;
    let h = entry.header();
    if (h.type_hash, h.schema_version, h.cook_stamp) != (cache_type(), 1, p.stamp) {
        return None;
    }
    let result = entry.raw("result").ok()?;
    let hash = format!("{:016x}", hash_bytes(result));
    if entry.raw("hash").ok()? != hash.as_bytes() {
        return None;
    }
    let blob = Blob::parse(result, "cached texture").ok()?;
    let h = blob.header();
    if format::expected_header(p.ty) != Some((h.type_hash, h.schema_version))
        || h.cook_stamp != p.stamp
    {
        return None;
    }
    Some(result.to_vec())
}

fn publish(path: &Path, stamp: u64, result: &[u8]) -> std::io::Result<()> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let mut entry = BlobWriter::new(cache_type(), 1, stamp);
    entry
        .raw(
            "hash",
            format!("{:016x}", hash_bytes(result)).into_bytes(),
            1,
        )
        .raw("result", result, 16);
    std::fs::create_dir_all(path.parent().expect("cache directory"))?;
    let temp = path.with_extension(format!(
        "{}.{}.tmp",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    // A sibling created exclusively, then renamed: readers see a complete entry
    // even when identical cooks finish together. A stale temp is only a miss.
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)?;
    let written = (|| {
        file.write_all(&entry.finish())?;
        drop(file);
        std::fs::rename(&temp, path)
    })();
    if written.is_err() {
        let _ = std::fs::remove_file(temp);
    }
    written
}
