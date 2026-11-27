//! A checked TS font-pack declaration (docs/ui.md §3.4) cooked into
//! og-i18n's native `FontPack`: its
//! languages, its faces' asset ids and the code points their cmaps map.
//! The faces themselves cook as fonts by their own import settings; the
//! index adds nothing to them, so a pack never changes another font's
//! cooked output. The index carries the pack's hyphenation dictionaries
//! for the game's locales (`hyphenation:<language>`), only when its code
//! writes a `hyphens-auto` class or its style sheets declare `hyphens:
//! auto`, and ICU's word
//! dictionary for a shipped locale of its languages that needs one
//! (`segmentation:<dictionary>`, og-i18n's `segmenter`): Chinese and
//! Japanese, Thai, Lao, Khmer and Burmese, once per game.

use super::*;
use super::{CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, stamp};
use crate::format::BlobKind;
use og_blob::BlobWriter;

/// One snapshot shared by diagnostics, planning and execution. Ref IDs are
/// canonical until `bind` resolves actual scanned font outputs and cmaps.
pub(crate) struct Prepared {
    pack: og_i18n::FontPack,
    hyphenation: BTreeMap<String, Vec<u8>>,
    inputs: BTreeMap<String, u64>,
    face_hashes: String,
}

pub(crate) fn declaration(
    input: crate::cook::source_schemas::world::localization::FontExtensionInput,
    root: &Path,
) -> Result<Prepared, String> {
    if input.name.trim().is_empty()
        || input.faces.is_empty()
        || input.languages.is_empty() && input.scripts.is_empty()
    {
        return Err("a font pack needs a name, at least one face and languages or scripts".into());
    }
    for script in &input.scripts {
        if script.len() != 4
            || !script.as_bytes()[0].is_ascii_uppercase()
            || !script.as_bytes()[1..].iter().all(u8::is_ascii_lowercase)
        {
            return Err(format!("'{script}' is not an ISO 15924 script code"));
        }
    }
    let mut hyphenation = BTreeMap::new();
    let mut inputs = BTreeMap::new();
    for file in input.hyphenation {
        let language = Path::new(&file)
            .file_name()
            .and_then(|f| f.to_str())
            .and_then(|f| f.strip_suffix(".hyph"))
            .filter(|f| !f.is_empty())
            .ok_or_else(|| format!("'{file}' must name a <language>.hyph dictionary"))?;
        let path = crate::cook::source_path::checked(root, &file)
            .map_err(|e| format!("dictionary '{file}': {e}"))?;
        let bytes = std::fs::read(path).map_err(|e| format!("dictionary '{file}': {e}"))?;
        inputs.insert(file.clone(), super::hash_bytes(&bytes));
        if hyphenation.insert(language.into(), bytes).is_some() {
            return Err(format!("duplicate hyphenation dictionary for '{language}'"));
        }
    }
    Ok(Prepared {
        pack: og_i18n::FontPack {
            name: input.name,
            languages: input
                .languages
                .iter()
                .map(|l| og_i18n::locale::canonical(l))
                .collect(),
            faces: input.faces.into_iter().map(|f| f.id).collect(),
            coverage: Vec::new(),
        },
        hyphenation,
        inputs,
        face_hashes: String::new(),
    })
}

pub(crate) fn bind(
    sources: &mut [Source],
    graph: &crate::cook::source_content::Graph,
) -> Result<(), CookError> {
    let fonts: BTreeMap<_, _> = sources
        .iter()
        .filter(|s| s.kind == &font::FONT)
        .map(|s| (s.rel.as_str(), s))
        .collect();
    let mut bound = Vec::new();
    for (index, source) in sources.iter().enumerate() {
        let Some(decl) = &source.font_pack else {
            continue;
        };
        let mut faces = Vec::new();
        let mut points = Vec::new();
        let mut hashes = String::new();
        for id in &decl.pack.faces {
            let asset = if let Some(output) = graph.outputs.get(id) {
                output.asset.clone()
            } else if let Some((pack, _)) = source
                .rel
                .strip_prefix("@fonts/")
                .and_then(|s| s.split_once('/'))
            {
                fonts
                    .keys()
                    .find(|asset| {
                        asset
                            .strip_prefix(&format!("@fonts/{pack}/"))
                            .is_some_and(|stem| id == &format!("base/fonts/{pack}/{stem}"))
                    })
                    .map(|asset| (*asset).to_owned())
                    .ok_or_else(|| {
                        CookError(format!("{}: unknown shipped font ref '{id}'", source.rel))
                    })?
            } else {
                return Err(CookError(format!(
                    "{}: unknown font ref '{id}'",
                    source.rel
                )));
            };
            let font = fonts.get(asset.as_str()).ok_or_else(|| {
                CookError(format!("{}: '{id}' is not a scanned font", source.rel))
            })?;
            let face = ttf_parser::Face::parse(&font.bytes, 0)
                .map_err(|e| CookError(format!("{}: font '{id}': {e}", source.rel)))?;
            if let Some(cmap) = face.tables().cmap {
                for sub in cmap.subtables {
                    if sub.is_unicode() {
                        sub.codepoints(|c| points.push(c));
                    }
                }
            }
            hashes.push_str(&format!("{asset}={:016x}\n", font.hash));
            faces.push(asset);
        }
        bound.push((index, faces, og_i18n::font_pack::ranges(points), hashes));
    }
    for (index, faces, coverage, hashes) in bound {
        let decl = sources[index].font_pack.as_mut().expect("font declaration");
        decl.pack.faces = faces;
        decl.pack.coverage = coverage;
        decl.face_hashes = hashes;
    }
    Ok(())
}

/// Whether a game hyphenates (CSS Text 4 §6.1): its code writes a
/// `hyphens-auto` class (`code`, the bundler graph's reading). Only then
/// does it ship hyphenation dictionaries.
pub(crate) fn uses_auto_hyphens(_sources: &[Source], code: &CodeGraph) -> bool {
    code.hyphens
}

/// The dictionary languages of the locales a game ships: its string
/// tables', or the default base table's `en` when it has none.
pub(crate) fn shipped_languages(sources: &[Source]) -> std::collections::BTreeSet<String> {
    let mut out: std::collections::BTreeSet<String> = sources
        .iter()
        .filter(|s| s.kind == &crate::cook::systems::strings::STRINGS)
        .filter_map(|s| s.strings.as_ref())
        .map(|t| og_i18n::hyphenation::dictionary_language(&t.locale))
        .collect();
    if out.is_empty() {
        out.insert("en".into());
    }
    out
}

/// The dictionary languages of pack `m` a game of `sources` ships.
fn shipped(m: &Prepared, sources: &[Source], code: &CodeGraph) -> Vec<String> {
    if m.hyphenation.is_empty() || !uses_auto_hyphens(sources, code) {
        return Vec::new();
    }
    let languages = shipped_languages(sources);
    m.hyphenation
        .keys()
        .filter(|language| languages.contains(*language))
        .cloned()
        .collect()
}

/// The cook's warnings about hyphenation: a game that hyphenates
/// ([`uses_auto_hyphens`]) and ships a locale whose own pack (one naming its language, with
/// dictionaries of its own) has none for it, which then does not hyphenate.
pub(crate) fn check_hyphenation(sources: &[Source], code: &CodeGraph) -> Vec<(String, String)> {
    if !uses_auto_hyphens(sources, code) {
        return Vec::new();
    }
    let languages = shipped_languages(sources);
    let mut out = Vec::new();
    for s in sources.iter().filter(|s| s.kind == &FONT_PACK) {
        let Some(m) = &s.font_pack else { continue };
        if m.hyphenation.is_empty() {
            continue;
        }
        for l in &languages {
            let named = m
                .pack
                .languages
                .iter()
                .any(|p| &og_i18n::hyphenation::dictionary_language(p) == l);
            if named && !m.hyphenation.contains_key(l) {
                out.push((
                    s.rel.clone(),
                    format!("warning: hyphens: auto: font pack '{}' has no hyphenation dictionary for '{l}', so its text is hyphenated only at soft hyphens", m.pack.name),
                ));
            }
        }
    }
    out
}

/// Whether checked pack `m` names a language of dictionary `attr`.
fn names_dictionary(m: &Prepared, attr: &str) -> bool {
    m.pack
        .languages
        .iter()
        .any(|l| og_i18n::segmenter::dictionary_for(l) == Some(attr))
}

/// The ICU word dictionaries pack `s` carries for the game of `sources`:
/// each one a shipped locale needs whose language the pack names, unless
/// a pack before it (by path) carries it already.
fn segmentation(s: &Source, sources: &[Source]) -> Vec<&'static str> {
    let Some(m) = &s.font_pack else {
        return Vec::new();
    };
    let wanted: std::collections::BTreeSet<&'static str> = shipped_languages(sources)
        .iter()
        .filter_map(|l| og_i18n::segmenter::dictionary_for(l))
        .collect();
    wanted
        .into_iter()
        .filter(|attr| names_dictionary(m, attr))
        .filter(|attr| {
            !sources.iter().any(|o| {
                o.kind == &FONT_PACK
                    && o.rel < s.rel
                    && o.font_pack
                        .as_ref()
                        .is_some_and(|om| names_dictionary(om, attr))
            })
        })
        .collect()
}

/// The cook's warnings about word segmentation: a shipped locale whose
/// script is written without spaces (Chinese, Japanese, Thai, Lao, Khmer,
/// Burmese) and that no font pack names, so no pack carries ICU's
/// dictionary for it and its words and lines break by the rules alone.
pub(crate) fn check_segmentation(sources: &[Source]) -> Vec<(String, String)> {
    let packs: Vec<&Prepared> = sources
        .iter()
        .filter(|s| s.kind == &FONT_PACK)
        .filter_map(|s| s.font_pack.as_ref())
        .collect();
    sources
        .iter()
        .filter(|s| s.kind == &crate::cook::systems::strings::STRINGS)
        .filter_map(|s| {
            let t = s.strings.as_ref()?;
            let attr = og_i18n::segmenter::dictionary_for(&t.locale)?;
            (!packs.iter().any(|m| names_dictionary(m, attr))).then(|| {
                (
                    s.rel.clone(),
                    format!(
                        "warning: locale '{}': no font pack names its language, so none carries ICU's '{attr}' dictionary and its words and lines break by the rules alone",
                        t.locale
                    ),
                )
            })
        })
        .collect()
}

/// The asset ids of a pack's faces (`fonts/arabic/NotoSansArabic-Regular.ttf`).
pub(crate) fn faces(s: &Source) -> Vec<String> {
    s.font_pack
        .as_ref()
        .map(|m| m.pack.faces.clone())
        .unwrap_or_default()
}

pub(crate) fn cook(
    s: &Source,
    dictionaries: &[String],
    segmentation: &[&str],
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let m = s.font_pack.as_ref().expect("prepared font pack");
    let pack = &m.pack;
    let chars: u32 = pack.coverage.iter().map(|r| r.end() - r.start() + 1).sum();
    let mut strings = super::model::Strings(Vec::new());
    let (name_offset, name_len) = strings.add(&pack.name);
    let mut rows = |values: &[String]| {
        values
            .iter()
            .map(|s| {
                let (offset, len) = strings.add(s);
                format::FontPackStringRow { offset, len }
            })
            .collect::<Vec<_>>()
    };
    let faces = rows(&pack.faces);
    let languages = rows(&pack.languages);
    let coverage: Vec<_> = pack
        .coverage
        .iter()
        .map(|r| format::FontPackRangeRow {
            start: *r.start(),
            end: *r.end(),
        })
        .collect();
    let mut w = BlobWriter::new(
        format::FontPack::type_hash(),
        format::FontPack::schema_version(),
        stamp,
    );
    w.table(
        "meta",
        &[format::FontPackMeta {
            name_offset,
            name_len,
        }],
        format::row_hash::<format::FontPackMeta>(),
    )
    .table(
        "faces",
        &faces,
        format::row_hash::<format::FontPackStringRow>(),
    )
    .table(
        "languages",
        &languages,
        format::row_hash::<format::FontPackStringRow>(),
    )
    .table(
        "coverage",
        &coverage,
        format::row_hash::<format::FontPackRangeRow>(),
    )
    .raw("strings", strings.0, 16);
    for language in dictionaries {
        w.raw(
            &format!("hyphenation:{language}"),
            m.hyphenation[language].clone(),
            16,
        );
    }
    for attr in segmentation {
        let bytes = og_i18n::segmenter::compiled(attr).ok_or_else(|| {
            CookError(format!(
                "font pack '{}': no ICU dictionary '{attr}'",
                pack.name
            ))
        })?;
        w.raw(&format!("segmentation:{attr}"), bytes, 16);
    }
    let mut extra = String::new();
    if !dictionaries.is_empty() {
        extra += &format!(", hyphenation {}", dictionaries.join(" "));
    }
    if !segmentation.is_empty() {
        extra += &format!(", segmentation {}", segmentation.join(" "));
    }
    Ok((
        w.finish(),
        vec![format!(
            "font pack '{}': {} face(s), {chars} characters, for {}{extra}",
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
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    // The stamp covers its faces' bytes (their cmaps are the coverage) and
    // the dictionaries it ships, by their bytes (ICU's, as this engine
    // links them).
    let m = s.font_pack.as_ref().expect("prepared font pack");
    let deps = m
        .inputs
        .iter()
        .map(|(f, h)| format!("{f}={h:016x}\n"))
        .collect::<String>();
    assets::record_dependencies(&mut state.parsed_refs, &s.rel, &deps);
    assets::record_dependencies(&mut state.parsed_refs, &s.rel, &m.face_hashes);
    let dictionaries = shipped(m, cx.sources, &cx.options.code);
    let shipped: String = dictionaries
        .iter()
        .map(|l| {
            let h = super::hash_bytes(&m.hyphenation[l]);
            format!("{l}={h:016x}\n")
        })
        .collect();
    let segmentation = segmentation(s, cx.sources);
    let words: String = segmentation
        .iter()
        .map(|a| {
            let h = og_i18n::segmenter::compiled(a)
                .map(|b| super::hash_bytes(&b))
                .unwrap_or(0);
            format!("{a}={h:016x}\n")
        })
        .collect();
    let st = stamp(
        &[Area::Font],
        &[
            b"font-pack 3",
            &hash,
            m.face_hashes.as_bytes(),
            shipped.as_bytes(),
            words.as_bytes(),
        ],
    );
    outs.push((
        s.rel.clone(),
        format::FontPack::NAME,
        st,
        Box::new(FontPackJob(s, dictionaries, segmentation)),
    ));
}

/// A font pack's one output, with the hyphenation and word dictionaries
/// it ships.
struct FontPackJob<'a>(&'a Source, Vec<String>, Vec<&'static str>);

impl Job for FontPackJob<'_> {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        cook(self.0, &self.1, &self.2, p.stamp)
    }
}
