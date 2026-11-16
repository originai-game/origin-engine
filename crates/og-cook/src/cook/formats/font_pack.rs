//! A font pack's index (docs/ui.md §3.4): an installed pack's
//! `extension.json` of kind `font` cooked into og-i18n's `FontPack` — its
//! languages, its faces' asset ids and the code points their cmaps map.
//! The faces themselves cook as fonts by their own import settings; the
//! index adds nothing to them, so a pack never changes another font's
//! cooked output. The index carries the pack's hyphenation dictionaries
//! for the game's locales (`hyphenation:<language>`), only when its
//! documents or style sheets declare `hyphens: auto`.

use super::*;
use super::{CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, stamp};
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
    /// Its hyphenation dictionaries, `<language>.hyph`.
    #[serde(default)]
    hyphenation: Vec<String>,
}

/// Whether a game's documents or style sheets declare `hyphens: auto`
/// (CSS Text 4 §6.1): only then does it ship hyphenation dictionaries.
pub(crate) fn uses_auto_hyphens(sources: &[Source]) -> bool {
    sources
        .iter()
        .filter(|s| s.rel.ends_with(".svelte") || s.rel.ends_with(".css"))
        .any(|s| {
            let text = String::from_utf8_lossy(&s.bytes);
            text.match_indices("hyphens").any(|(at, _)| {
                text[at + "hyphens".len()..]
                    .trim_start()
                    .strip_prefix(':')
                    .is_some_and(|v| v.trim_start().starts_with("auto"))
            })
        })
}

/// The dictionary languages of the locales a game ships: its string
/// tables', or the default base table's `en` when it has none.
pub(crate) fn shipped_languages(sources: &[Source]) -> std::collections::BTreeSet<String> {
    let mut out: std::collections::BTreeSet<String> = sources
        .iter()
        .filter(|s| s.kind == &crate::cook::systems::strings::STRINGS)
        .filter_map(|s| og_i18n::StringTable::parse(std::str::from_utf8(&s.bytes).ok()?).ok())
        .map(|t| og_i18n::hyphenation::dictionary_language(&t.locale))
        .collect();
    if out.is_empty() {
        out.insert("en".into());
    }
    out
}

/// The dictionaries of pack `m` a game of `sources` ships: (language, file).
fn shipped(m: &Manifest, sources: &[Source]) -> Vec<(String, String)> {
    if m.hyphenation.is_empty() || !uses_auto_hyphens(sources) {
        return Vec::new();
    }
    let languages = shipped_languages(sources);
    m.hyphenation
        .iter()
        .filter_map(|f| {
            let language = f.strip_suffix(".hyph")?;
            languages
                .contains(language)
                .then(|| (language.to_string(), f.clone()))
        })
        .collect()
}

/// The cook's warnings about hyphenation: a game that declares `hyphens:
/// auto` and ships a locale whose own pack (one naming its language, with
/// dictionaries of its own) has none for it, which then does not hyphenate.
pub(crate) fn check_hyphenation(sources: &[Source]) -> Vec<(String, String)> {
    if !uses_auto_hyphens(sources) {
        return Vec::new();
    }
    let languages = shipped_languages(sources);
    let mut out = Vec::new();
    for s in sources.iter().filter(|s| s.kind == &FONT_PACK) {
        let Ok(m) = manifest(s) else { continue };
        if m.hyphenation.is_empty() {
            continue;
        }
        for l in &languages {
            let named = m
                .languages
                .iter()
                .any(|p| &og_i18n::hyphenation::dictionary_language(p) == l);
            if named && !m.hyphenation.iter().any(|f| f == &format!("{l}.hyph")) {
                out.push((
                    s.rel.clone(),
                    format!("warning: hyphens: auto: font pack '{}' has no hyphenation dictionary for '{l}', so its text is hyphenated only at soft hyphens", m.name),
                ));
            }
        }
    }
    out
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

pub(crate) fn cook(
    s: &Source,
    dictionaries: &[(String, String)],
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
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
    for (language, file) in dictionaries {
        let bytes = std::fs::read(dir.join(file))
            .map_err(|e| CookError(format!("font pack '{}': {file}: {e}", m.name)))?;
        w.raw(&format!("hyphenation:{language}"), bytes, 16);
    }
    let hyphenation = if dictionaries.is_empty() {
        String::new()
    } else {
        let names: Vec<&str> = dictionaries.iter().map(|(l, _)| l.as_str()).collect();
        format!(", hyphenation {}", names.join(" "))
    };
    Ok((
        w.finish(),
        vec![format!(
            "font pack '{}': {} face(s), {chars} characters, for {}{hyphenation}",
            pack.name,
            pack.faces.len(),
            pack.languages.join(" ")
        )],
    ))
}

pub(crate) static FONT_PACK: KindDef = KindDef {
    name: "font-pack",
    extensions: &[],
    types: &[],
    by_content: false,
    streams: false,
    plan: plan_font_pack,
};

/// The kinds this module cooks.
pub(crate) static KINDS: &[&KindDef] = &[&FONT_PACK];

fn plan_font_pack<'a>(
    cx: &Cx<'a>,
    _state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    // The stamp covers its faces' bytes (their cmaps are the coverage) and
    // the dictionaries it ships, by their bytes.
    let deps = dependencies(s);
    let dictionaries = manifest(s)
        .map(|m| shipped(&m, cx.sources))
        .unwrap_or_default();
    let dir = s.path.parent().map(Path::to_path_buf).unwrap_or_default();
    let shipped: String = dictionaries
        .iter()
        .map(|(l, f)| {
            let h = std::fs::read(dir.join(f))
                .map(|b| super::hash_bytes(&b))
                .unwrap_or(0);
            format!("{l}={h:016x}\n")
        })
        .collect();
    let st = stamp(&[b"font-pack 2", &hash, deps.as_bytes(), shipped.as_bytes()]);
    outs.push((
        s.rel.clone(),
        format::FontPack::NAME,
        st,
        Box::new(FontPackJob(s, dictionaries)),
    ));
}

/// A font pack's one output, with the dictionaries it ships.
struct FontPackJob<'a>(&'a Source, Vec<(String, String)>);

impl Job for FontPackJob<'_> {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        cook(self.0, &self.1, p.stamp)
    }
}
