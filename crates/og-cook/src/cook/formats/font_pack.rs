//! A font pack's index (docs/ui.md §3.4): an installed pack's
//! `extension.json` of kind `font` cooked into og-i18n's `FontPack` — its
//! languages, its faces' asset ids and the code points their cmaps map.
//! The faces themselves cook as fonts by their own import settings; the
//! index adds nothing to them, so a pack never changes another font's
//! cooked output.

use super::*;
use crate::format::BlobKind;
use og_blob::BlobWriter;

#[derive(serde::Deserialize)]
struct Manifest {
    kind: String,
    name: String,
    #[serde(default)]
    languages: Vec<String>,
    /// ISO 15924 codes; a pack of no language (`emoji`: `Zsye`) names
    /// only its scripts and is never a locale's own.
    #[serde(default)]
    scripts: Vec<String>,
    #[serde(default)]
    faces: Vec<String>,
}

/// Whether a file is a font pack's manifest: an `extension.json` of kind `font`.
pub(crate) fn is_pack(rel: &str, bytes: &[u8]) -> bool {
    rel.rsplit('/').next() == Some("extension.json")
        && serde_json::from_slice::<Manifest>(bytes).is_ok_and(|m| m.kind == "font")
}

fn manifest(s: &Source) -> Result<Manifest, CookError> {
    serde_json::from_slice(&s.bytes).map_err(|e| CookError(format!("font pack manifest: {e}")))
}

/// The directory of the pack, as an asset id prefix (`fonts/arabic/`).
fn prefix(rel: &str) -> &str {
    rel.strip_suffix("extension.json").unwrap_or(rel)
}

/// The asset ids of a pack's faces (`fonts/arabic/NotoSansArabic-Regular.ttf`).
pub(crate) fn faces(s: &Source) -> Vec<String> {
    manifest(s)
        .map(|m| {
            m.faces
                .iter()
                .map(|f| format!("{}{f}", prefix(&s.rel)))
                .collect()
        })
        .unwrap_or_default()
}

/// What the stamp covers besides the manifest: each face's bytes.
pub(crate) fn dependencies(s: &Source) -> String {
    let Ok(m) = manifest(s) else {
        return String::new();
    };
    let dir = s.path.parent().map(Path::to_path_buf).unwrap_or_default();
    m.faces
        .iter()
        .map(|f| {
            let h = std::fs::read(dir.join(f))
                .map(|b| super::hash_bytes(&b))
                .unwrap_or(0);
            format!("{f}={h:016x}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn cook(s: &Source, stamp: u64) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let m = manifest(s)?;
    if m.faces.is_empty() || (m.languages.is_empty() && m.scripts.is_empty()) {
        return Err(CookError(format!(
            "font pack '{}': a pack names its languages (or, of no language, its scripts) and at least one face",
            m.name
        )));
    }
    let dir = s.path.parent().map(Path::to_path_buf).unwrap_or_default();
    let mut points = Vec::new();
    for f in &m.faces {
        let bytes = std::fs::read(dir.join(f))
            .map_err(|e| CookError(format!("font pack '{}': face {f}: {e}", m.name)))?;
        let face = ttf_parser::Face::parse(&bytes, 0)
            .map_err(|e| CookError(format!("font pack '{}': face {f}: {e}", m.name)))?;
        if let Some(cmap) = face.tables().cmap {
            for sub in cmap.subtables {
                if sub.is_unicode() {
                    sub.codepoints(|c| points.push(c));
                }
            }
        }
    }
    let pack = og_i18n::FontPack {
        name: m.name.clone(),
        languages: m
            .languages
            .iter()
            .map(|l| og_i18n::locale::canonical(l))
            .collect(),
        faces: faces(s),
        coverage: og_i18n::font_pack::ranges(points),
    };
    let text = pack.to_text();
    let chars: u32 = pack.coverage.iter().map(|r| r.end() - r.start() + 1).sum();
    let mut w = BlobWriter::new(
        format::FontPack::type_hash(),
        format::FontPack::schema_version(),
        stamp,
    );
    w.table(
        "meta",
        &[format::FontPackMeta {
            text_bytes: text.len() as u32,
            faces: pack.faces.len() as u32,
        }],
        format::row_hash::<format::FontPackMeta>(),
    )
    .raw("text", text.into_bytes(), 16);
    Ok((
        w.finish(),
        vec![format!(
            "font pack '{}': {} face(s), {chars} characters, for {}",
            pack.name,
            pack.faces.len(),
            pack.languages.join(" ")
        )],
    ))
}
