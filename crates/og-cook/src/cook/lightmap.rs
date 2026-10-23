//! `type lightmap` → `og.Lightmap` blobs (depth S7): the text `og bake
//! lightmaps` wrote beside a scene, checked, with its pages file (the
//! `.ktx2` it names, beside it) packed into the same blob. The pages file
//! is consumed: it is part of the lightmap and in its stamp, never a
//! texture of its own.

use super::{CookError, Source, hash_bytes};
use crate::format::{self, BlobKind};
use crate::lightmap::{LightmapFile, pages_from_ktx2};
use og_blob::BlobWriter;

pub(super) const LIGHTMAP_TYPE: &str = crate::lightmap::LIGHTMAP_TYPE;

/// The pages file a lightmap names, asset-relative.
pub(super) fn pages_file(rel: &str, text: &str) -> Option<String> {
    let file = LightmapFile::parse(text).ok()?;
    let dir = rel.rsplit_once('/').map_or("", |(d, _)| d);
    Some(if dir.is_empty() {
        file.pages
    } else {
        format!("{dir}/{}", file.pages)
    })
}

/// The stamp's dependency: the pages file's hash.
pub(super) fn dependencies(s: &Source) -> String {
    let text = String::from_utf8_lossy(&s.bytes);
    match pages_file(&s.rel, &text) {
        Some(p) => {
            let root = super::assets_root(s);
            let hash = std::fs::read(root.join(&p)).map_or(0, |b| hash_bytes(&b));
            format!("{p} {hash:016x}")
        }
        None => String::new(),
    }
}

pub(super) fn cook_lightmap(s: &Source, stamp: u64) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let text = std::str::from_utf8(&s.bytes)
        .map_err(|_| CookError("a lightmap is text (UTF-8)".into()))?;
    let file = LightmapFile::parse(text).map_err(CookError)?;
    let rel = pages_file(&s.rel, text).expect("parsed above");
    let root = super::assets_root(s);
    let pages = std::fs::read(root.join(&rel)).map_err(|e| {
        CookError(format!(
            "{rel}: {e} (the lightmap's pages; run og bake lightmaps again)"
        ))
    })?;
    let (width, height, layers) =
        pages_from_ktx2(&pages).map_err(|e| CookError(format!("{rel}: {e}")))?;
    let canonical = file.to_text();
    let probes: usize = file.volumes.iter().map(|v| v.probes.len()).sum();
    let mut w = BlobWriter::new(
        format::Lightmap::type_hash(),
        format::Lightmap::schema_version(),
        stamp,
    );
    w.table(
        "meta",
        &[format::LightmapMeta {
            text_bytes: canonical.len() as u32,
            pages_bytes: pages.len() as u32,
            models: file.models.len() as u32,
            pages: layers.len() as u32,
            texels: width.max(height),
            volumes: file.volumes.len() as u32,
            probes: probes as u32,
        }],
        format::row_hash::<format::LightmapMeta>(),
    )
    .raw("text", canonical.into_bytes(), 16)
    .raw("pages", pages, 16);
    Ok((
        w.finish(),
        vec![format!(
            "lightmap of '{}': {} static model(s), {} page(s) of {width}x{height}, {} volume(s), {probes} probe(s)",
            file.scene,
            file.models.len(),
            layers.len(),
            file.volumes.len()
        )],
    ))
}
