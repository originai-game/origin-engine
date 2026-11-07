//! TrueType/OpenType → font blobs: an MTSDF atlas (RGB multi-channel and A
//! true signed distance, fdsm) and a glyph table for the declared charset.
//!
//! Shaped scripts (Arabic and the others og-i18n's `needs_shaping` names):
//! when the charset holds one, the blob also carries the font file (raw
//! section `ttf`) for og-ui's run-time shaping, and the atlas every glyph the
//! font's substitutions reach from the charset's (contextual forms,
//! ligatures), each keyed by its glyph id (`GLYPH_ID_KEYS` + id, past
//! Unicode, so no character of the font can alias one), which is what og-ui
//! draws a shaped run as.

use super::{CookError, FontSettings, Palette};
use crate::format::{self, *};
use crate::ktx2;
use og_blob::BlobWriter;

mod colr;
mod overlap;

/// Where a glyph by id (a shaped form) is keyed in the blob: 0x110000 + id,
/// past every Unicode code point (og_bundle::format::GLYPH_ID_KEYS).
use og_bundle::format::GLYPH_ID_KEYS;

/// Shaping needs metrics and layout tables, not raster outlines or hints.
/// FontBuilder writes the SFNT directory and checksums; glyph IDs stay intact.
fn shaping_tables(face: &ttf_parser::Face) -> Vec<u8> {
    let mut font = write_fonts::FontBuilder::new();
    for tag in [
        b"head", b"hhea", b"hmtx", b"maxp", b"OS/2", b"name", b"post", b"cmap", b"GDEF", b"GPOS",
        b"GSUB", b"kern", b"fvar", b"avar", b"HVAR", b"VVAR", b"MVAR", b"STAT", b"vhea", b"vmtx",
        b"BASE", b"morx", b"mort", b"kerx", b"ankr", b"trak",
    ] {
        if let Some(data) = face.raw_face().table(ttf_parser::Tag::from_bytes(tag)) {
            font.add_raw(write_fonts::types::Tag::new(tag), data);
        }
    }
    font.build()
}

/// Every glyph the font's GSUB substitutions reach from `start`, start included.
fn substitution_closure(face: &ttf_parser::Face, start: &[ttf_parser::GlyphId]) -> Vec<u16> {
    closure_of(face, start, None)
}

/// The GSUB lookups of the features `tags`, in any script and language.
fn feature_lookups(face: &ttf_parser::Face, tags: &[String]) -> std::collections::BTreeSet<u16> {
    let mut out = std::collections::BTreeSet::new();
    if let Some(gsub) = face.tables().gsub {
        for f in gsub.features {
            if tags
                .iter()
                .any(|t| ttf_parser::Tag::from_bytes_lossy(t.as_bytes()) == f.tag)
            {
                out.extend(f.lookup_indices);
            }
        }
        // Contextual lookups reference other lookups which need not be
        // listed by a feature. Follow that graph before closing glyphs.
        use ttf_parser::gsub::SubstitutionSubtable as S;
        use ttf_parser::opentype_layout::{
            ChainedContextLookup as Chain, ContextLookup as Context,
        };
        let mut pending: Vec<_> = out.iter().copied().collect();
        while let Some(index) = pending.pop() {
            let Some(lookup) = gsub.lookups.get(index) else {
                continue;
            };
            let mut visit = |records: ttf_parser::LazyArray16<
                '_,
                ttf_parser::opentype_layout::SequenceLookupRecord,
            >| {
                for record in records {
                    if out.insert(record.lookup_list_index) {
                        pending.push(record.lookup_list_index);
                    }
                }
            };
            for k in 0..lookup.subtables.len() {
                match lookup.subtables.get::<S>(k) {
                    Some(S::Context(
                        Context::Format1 { sets, .. } | Context::Format2 { sets, .. },
                    )) => {
                        for set in sets {
                            for rule in set {
                                visit(rule.lookups);
                            }
                        }
                    }
                    Some(S::ChainContext(
                        Chain::Format1 { sets, .. } | Chain::Format2 { sets, .. },
                    )) => {
                        for set in sets {
                            for rule in set {
                                visit(rule.lookups);
                            }
                        }
                    }
                    Some(
                        S::Context(Context::Format3 { lookups, .. })
                        | S::ChainContext(Chain::Format3 { lookups, .. }),
                    ) => visit(lookups),
                    _ => {}
                }
            }
        }
    }
    out
}

/// [`substitution_closure`] through the lookups `only` (all when `None`).
fn closure_of(
    face: &ttf_parser::Face,
    start: &[ttf_parser::GlyphId],
    only: Option<&std::collections::BTreeSet<u16>>,
) -> Vec<u16> {
    use ttf_parser::gsub::SubstitutionSubtable as S;
    let mut set: std::collections::BTreeSet<u16> = start.iter().map(|g| g.0).collect();
    let Some(gsub) = face.tables().gsub else {
        return set.into_iter().collect();
    };
    loop {
        let before = set.len();
        let now: Vec<ttf_parser::GlyphId> = set.iter().map(|&g| ttf_parser::GlyphId(g)).collect();
        for (li, lookup) in gsub.lookups.into_iter().enumerate() {
            if only.is_some_and(|o| !o.contains(&(li as u16))) {
                continue;
            }
            for k in 0..lookup.subtables.len() {
                let Some(sub) = lookup.subtables.get::<S>(k) else {
                    continue;
                };
                for &g in &now {
                    match &sub {
                        S::Single(ttf_parser::gsub::SingleSubstitution::Format1 {
                            coverage,
                            delta,
                        }) => {
                            if coverage.contains(g) {
                                set.insert((i32::from(g.0) + i32::from(*delta)) as u16);
                            }
                        }
                        S::Single(ttf_parser::gsub::SingleSubstitution::Format2 {
                            coverage,
                            substitutes,
                        }) => {
                            if let Some(o) = coverage.get(g).and_then(|i| substitutes.get(i)) {
                                set.insert(o.0);
                            }
                        }
                        S::Multiple(m) => {
                            if let Some(seq) = m.coverage.get(g).and_then(|i| m.sequences.get(i)) {
                                set.extend(seq.substitutes.into_iter().map(|o| o.0));
                            }
                        }
                        S::Alternate(a) => {
                            if let Some(alt) =
                                a.coverage.get(g).and_then(|i| a.alternate_sets.get(i))
                            {
                                set.extend(alt.alternates.into_iter().map(|o| o.0));
                            }
                        }
                        S::Ligature(l) => {
                            if let Some(ligs) =
                                l.coverage.get(g).and_then(|i| l.ligature_sets.get(i))
                            {
                                for lig in ligs {
                                    if lig.components.into_iter().all(|c| now.contains(&c)) {
                                        set.insert(lig.glyph.0);
                                    }
                                }
                            }
                        }
                        S::ReverseChainSingle(r) => {
                            if let Some(o) = r.coverage.get(g).and_then(|i| r.substitutes.get(i)) {
                                set.insert(o.0);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        if set.len() == before {
            break;
        }
    }
    set.into_iter().collect()
}

/// Horizontal pair kerning among `glyphs` (code point, glyph id), in font
/// units: the pair adjustments (GPOS lookup type 2, formats 1 and 2) of
/// every `kern` feature of every script, applied as a shaper does (the
/// first subtable of a lookup that takes a pair decides it; lookups add),
/// else the legacy `kern` table's horizontal subtables. Pairs that adjust
/// by nothing are left out; the list is sorted by (left, right).
fn kerning(face: &ttf_parser::Face, glyphs: &[(u32, ttf_parser::GlyphId)]) -> Vec<(u32, u32, i32)> {
    use std::collections::{BTreeMap, BTreeSet, HashSet};
    use ttf_parser::gpos::{PairAdjustment, PositioningSubtable};
    let by_glyph: BTreeMap<u16, Vec<u32>> =
        glyphs.iter().fold(BTreeMap::new(), |mut m, (cp, g)| {
            m.entry(g.0).or_insert_with(Vec::new).push(*cp);
            m
        });
    let ids: Vec<ttf_parser::GlyphId> = by_glyph.keys().map(|&g| ttf_parser::GlyphId(g)).collect();
    let mut pairs: BTreeMap<(u16, u16), i32> = BTreeMap::new();
    let mut lookups: BTreeSet<u16> = BTreeSet::new();
    if let Some(gpos) = face.tables().gpos {
        for f in gpos.features {
            if f.tag == ttf_parser::Tag::from_bytes(b"kern") {
                lookups.extend(f.lookup_indices);
            }
        }
        for &l in &lookups {
            let Some(lookup) = gpos.lookups.get(l) else {
                continue;
            };
            // What an earlier subtable of this lookup already took.
            let mut taken_first: HashSet<u16> = HashSet::new();
            let mut taken: HashSet<(u16, u16)> = HashSet::new();
            for k in 0..lookup.subtables.len() {
                let Some(PositioningSubtable::Pair(pair)) =
                    lookup.subtables.get::<PositioningSubtable>(k)
                else {
                    continue;
                };
                match pair {
                    PairAdjustment::Format1 { coverage, sets } => {
                        for &a in &ids {
                            if taken_first.contains(&a.0) {
                                continue;
                            }
                            let Some(set) = coverage.get(a).and_then(|i| sets.get(i)) else {
                                continue;
                            };
                            for &b in &ids {
                                if taken.contains(&(a.0, b.0)) {
                                    continue;
                                }
                                if let Some((v1, v2)) = set.get(b) {
                                    taken.insert((a.0, b.0));
                                    *pairs.entry((a.0, b.0)).or_default() +=
                                        i32::from(v1.x_advance) + i32::from(v2.x_placement);
                                }
                            }
                        }
                    }
                    PairAdjustment::Format2 {
                        coverage,
                        classes,
                        matrix,
                    } => {
                        // The cooked glyphs by their second class.
                        let mut seconds: BTreeMap<u16, Vec<u16>> = BTreeMap::new();
                        for &b in &ids {
                            seconds.entry(classes.1.get(b)).or_default().push(b.0);
                        }
                        for &a in &ids {
                            if taken_first.contains(&a.0) || coverage.get(a).is_none() {
                                continue;
                            }
                            taken_first.insert(a.0);
                            let c1 = classes.0.get(a);
                            for (&c2, bs) in &seconds {
                                let Some((v1, v2)) = matrix.get((c1, c2)) else {
                                    continue;
                                };
                                let v = i32::from(v1.x_advance) + i32::from(v2.x_placement);
                                if v == 0 {
                                    continue;
                                }
                                for &b in bs {
                                    if !taken.contains(&(a.0, b)) {
                                        *pairs.entry((a.0, b)).or_default() += v;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    // The legacy table is read pair by pair: only for a charset of a size a
    // Latin font has (a CJK font kerns in GPOS).
    if lookups.is_empty()
        && ids.len() <= 4096
        && let Some(kern) = face.tables().kern
    {
        for sub in kern.subtables {
            if !sub.horizontal || sub.variable || sub.has_cross_stream {
                continue;
            }
            for &a in &ids {
                for &b in &ids {
                    if let Some(v) = sub.glyphs_kerning(a, b) {
                        *pairs.entry((a.0, b.0)).or_default() += i32::from(v);
                    }
                }
            }
        }
    }
    let mut out = Vec::new();
    for ((a, b), v) in pairs {
        if v == 0 {
            continue;
        }
        for &l in &by_glyph[&a] {
            for &r in &by_glyph[&b] {
                out.push((l, r, v));
            }
        }
    }
    out.sort_unstable();
    out
}

struct Glyph {
    codepoint: u32,
    advance: f32,
    /// Plane bounds in em (left, bottom, right, top); None for an empty glyph.
    plane: Option<[f32; 4]>,
    width: u32,
    height: u32,
    /// RGBA8, rows top to bottom.
    pixels: Vec<u8>,
}

fn render(face: &ttf_parser::Face, cp: u32, s: &FontSettings) -> Option<Glyph> {
    use fdsm::bezier::scanline::FillRule;
    use fdsm::shape::Shape;
    use fdsm::transform::Transform;
    // A key past GLYPH_ID_KEYS is a glyph id (a shaped form).
    let gid = if (GLYPH_ID_KEYS..GLYPH_ID_KEYS + 0x1_0000).contains(&cp) {
        ttf_parser::GlyphId((cp - GLYPH_ID_KEYS) as u16)
    } else {
        face.glyph_index(char::from_u32(cp)?)?
    };
    let upem = face.units_per_em() as f64;
    let mut advance = face.glyph_hor_advance(gid).unwrap_or(0) as f32 / upem as f32;
    if s.pixel > 0 {
        advance = og_ui::glyphs::snap_to_grid(advance, s.pixel);
    }
    let empty = Glyph {
        codepoint: cp,
        advance,
        plane: None,
        width: 0,
        height: 0,
        pixels: Vec::new(),
    };
    let (bbox, mut shape) = if s.pixel > 0 {
        // A pixel font: the field is of the glyph's cells on its grid
        // (og-ui's `pixel_cells`, which run-time glyphs share), square
        // corners and straight runs, never overlapping.
        let cells = og_ui::glyphs::pixel_cells(face, gid, s.pixel);
        let cell = upem / f64::from(s.pixel);
        let Some(b) = cells.bounds() else {
            return Some(empty);
        };
        (b.map(|v| f64::from(v) * cell), pixel_shape(&cells, cell))
    } else {
        let Some(bbox) = face.glyph_bounding_box(gid) else {
            return Some(empty);
        };
        let Some(shape) = fdsm_ttf_parser::load_shape_from_face(face, gid) else {
            return Some(empty);
        };
        // Overlapping contours (the OVERLAP flag) would leave their inner
        // edges as seams in the field: the field is of the outline's union.
        let b = [bbox.x_min, bbox.y_min, bbox.x_max, bbox.y_max].map(f64::from);
        (b, overlap::remove_overlaps(shape))
    };
    // The glyph's box in font units: left, bottom, right, top.
    if shape.contours.is_empty() || bbox[2] <= bbox[0] || bbox[3] <= bbox[1] {
        return Some(empty);
    }
    let scale = s.em_px as f64 / upem;
    let pad = (s.px_range as f64 / 2.0).ceil() + 1.0;
    let width = ((bbox[2] - bbox[0]) * scale + 2.0 * pad).ceil() as u32;
    let height = ((bbox[3] - bbox[1]) * scale + 2.0 * pad).ceil() as u32;
    let tx = pad - bbox[0] * scale;
    let ty = pad - bbox[1] * scale;
    let transform = nalgebra::convert::<_, nalgebra::Affine2<f64>>(nalgebra::Similarity2::new(
        nalgebra::Vector2::new(tx, ty),
        0.0,
        scale,
    ));
    shape.transform(&transform);
    let colored = Shape::edge_coloring_simple(shape, 0.03, 0x6f67_636f_6f6b);
    let prepared = colored.clone().prepare();
    // Keep f32 through both corrections before quantising to bytes.
    let mut field: image::ImageBuffer<image::Rgba<f32>, Vec<f32>> =
        image::ImageBuffer::new(width, height);
    fdsm::generate::generate_mtsdf(&prepared, s.px_range as f64, &mut field);
    // Match msdf-atlas-gen's scanline pipeline: sign first, then error
    // correction without distance checks against the uncorrected shape.
    // Correcting signs afterwards can reintroduce false interpolated edges.
    fdsm::render::correct_sign_mtsdf(&mut field, &prepared, FillRule::Nonzero);
    let mut correction = fdsm::correct_error::ErrorCorrectionConfig::default();
    correction.distance_check = fdsm::correct_error::DistanceCheckMode::Never;
    fdsm::correct_error::correct_error_mtsdf(
        &mut field,
        &colored,
        &prepared,
        s.px_range as f64,
        &correction,
    );
    let mut img = image::RgbaImage::new(width, height);
    for (o, i) in img.pixels_mut().zip(field.pixels()) {
        *o = image::Rgba(i.0.map(|v| (f64::from(v.clamp(0.0, 1.0)) * 255.0) as u8));
    }
    // Font units are y-up; atlas rows go top to bottom.
    image::imageops::flip_vertical_in_place(&mut img);
    let left = (bbox[0] * scale - pad) / s.em_px as f64;
    let bottom = (bbox[1] * scale - pad) / s.em_px as f64;
    Some(Glyph {
        codepoint: cp,
        advance,
        plane: Some([
            left as f32,
            bottom as f32,
            (left + width as f64 / s.em_px as f64) as f32,
            (bottom + height as f64 / s.em_px as f64) as f32,
        ]),
        width,
        height,
        pixels: img.into_raw(),
    })
}

/// The outline of a pixel glyph's cells, `cell` font units each.
fn pixel_shape(
    cells: &og_ui::glyphs::PixelCells,
    cell: f64,
) -> fdsm::shape::Shape<fdsm::shape::Contour> {
    use fdsm::bezier::{Point, Segment};
    let mut shape = fdsm::shape::Shape::default();
    for c in cells.contours() {
        let p = |(x, y): (i32, i32)| Point::new(f64::from(x) * cell, f64::from(y) * cell);
        let mut contour = fdsm::shape::Contour::default();
        for k in 0..c.len() {
            contour
                .segments
                .push(Segment::line(p(c[k]), p(c[(k + 1) % c.len()])));
        }
        shape.contours.push(contour);
    }
    shape
}

/// The share of a face's outline points, over the charset's glyphs, that
/// lie off a `grid`-cell em, and how many points there are: a pixel font
/// drawn on its grid has none off it.
fn off_grid(face: &ttf_parser::Face, wanted: &[u32], grid: u32) -> (usize, usize) {
    struct Points(Vec<(f32, f32)>);
    impl ttf_parser::OutlineBuilder for Points {
        fn move_to(&mut self, x: f32, y: f32) {
            self.0.push((x, y));
        }
        fn line_to(&mut self, x: f32, y: f32) {
            self.0.push((x, y));
        }
        fn quad_to(&mut self, _: f32, _: f32, x: f32, y: f32) {
            self.0.push((x, y));
        }
        fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, x: f32, y: f32) {
            self.0.push((x, y));
        }
        fn close(&mut self) {}
    }
    let cell = f32::from(face.units_per_em()) / grid as f32;
    let mut pts = Points(Vec::new());
    for &cp in wanted.iter().filter(|&&cp| cp < 0x80) {
        if let Some(g) = char::from_u32(cp).and_then(|c| face.glyph_index(c)) {
            face.outline_glyph(g, &mut pts);
        }
    }
    let off = |v: f32| ((v / cell) - (v / cell).round()).abs() > 0.02;
    let n = pts.0.iter().filter(|&&(x, y)| off(x) || off(y)).count();
    (n, pts.0.len())
}

/// The Unicode spaces a cooked font always carries.
const SPACES: [u32; 14] = [
    0x00a0, 0x2000, 0x2001, 0x2002, 0x2003, 0x2004, 0x2005, 0x2006, 0x2007, 0x2008, 0x2009, 0x200a,
    0x202f, 0x205f,
];

/// An empty glyph for each of [`SPACES`] that `cooked` lacks: the font's own
/// advance when its file has the character outside the charset, else the
/// width Unicode gives it, from the font's space (U+00A0 as the space; the
/// quads and en/em spaces in em; figure and punctuation spaces as `0` and
/// `.`; thin spaces and U+202F about a third of an em, never wider than the
/// space; the hair space a tenth). None without a space glyph to measure by.
fn unicode_spaces(face: &ttf_parser::Face, cooked: &[Glyph]) -> Vec<Glyph> {
    let upem = f32::from(face.units_per_em());
    let advance_of = |c: char| {
        face.glyph_index(c)
            .and_then(|g| face.glyph_hor_advance(g))
            .map(|a| f32::from(a) / upem)
    };
    let Some(space) = advance_of(' ') else {
        return Vec::new();
    };
    SPACES
        .iter()
        .filter(|&&cp| !cooked.iter().any(|g| g.codepoint == cp))
        .map(|&cp| {
            let c = char::from_u32(cp).expect("a space code point");
            let advance = advance_of(c).unwrap_or(match cp {
                0x2000 | 0x2002 => 0.5,
                0x2001 | 0x2003 => 1.0,
                0x2004 => 1.0 / 3.0,
                0x2005 => 0.25,
                0x2006 => 1.0 / 6.0,
                0x2007 => advance_of('0').unwrap_or(space),
                0x2008 => advance_of('.').unwrap_or(space),
                0x2009 | 0x202f => space.min(1.0 / 3.0),
                0x200a => 0.1,
                0x205f => 4.0 / 18.0,
                _ => space,
            });
            Glyph {
                codepoint: cp,
                advance,
                plane: None,
                width: 0,
                height: 0,
                pixels: Vec::new(),
            }
        })
        .collect()
}

/// What a font cook reports besides the blob.
pub struct FontNotes {
    pub note: String,
}

/// Cooks a font for the settings' charset.
pub fn cook(
    bytes: &[u8],
    s: &FontSettings,
    threads: usize,
    stamp: u64,
) -> Result<(Vec<u8>, FontNotes), CookError> {
    let face = ttf_parser::Face::parse(bytes, 0).map_err(|e| CookError(format!("font: {e}")))?;
    if s.color && s.runtime {
        return Err(CookError(
            "color true and runtime true: run-time glyphs are drawn from outlines, without their colour; \
             list the colour characters in chars or charset and drop runtime"
                .into(),
        ));
    }
    let mut in_font: Vec<u32> = Vec::new();
    if let Some(cmap) = face.tables().cmap {
        for sub in cmap.subtables {
            if sub.is_unicode() {
                sub.codepoints(|cp| in_font.push(cp));
            }
        }
    }
    in_font.sort_unstable();
    in_font.dedup();
    let mut wanted: Vec<u32> = Vec::new();
    for word in &s.charset {
        let ranges = crate::format::charset(word).ok_or_else(|| {
            CookError(format!(
                "unknown charset '{word}' (known: ascii, latin, punct, cjk-punct, kana, hangul, arabic, all)"
            ))
        })?;
        for r in ranges {
            wanted.extend(in_font.iter().copied().filter(|cp| r.contains(cp)));
        }
    }
    let extra: Vec<u32> = s.chars.chars().map(|c| c as u32).collect();
    let missing: Vec<u32> = extra
        .iter()
        .copied()
        .filter(|cp| in_font.binary_search(cp).is_err() && !SPACES.contains(cp))
        .collect();
    wanted.extend(
        extra
            .iter()
            .copied()
            .filter(|cp| in_font.binary_search(cp).is_ok()),
    );
    wanted.sort_unstable();
    wanted.dedup();
    if wanted.is_empty() {
        return Err(CookError(
            "the charset selects no glyph of this font".into(),
        ));
    }
    // Every cooked face is shaped. Seed complex-script closure with the
    // whole charset, including its digits and punctuation.
    let shaped: Vec<ttf_parser::GlyphId> = wanted
        .iter()
        .filter_map(|&cp| char::from_u32(cp))
        .filter_map(|c| face.glyph_index(c))
        .collect();
    let shaping = wanted
        .iter()
        .filter_map(|&cp| char::from_u32(cp))
        .any(og_i18n::needs_shaping);
    if shaping {
        // The marks and joiners shaping inserts or keeps.
        let mut start = shaped;
        for c in ['\u{200c}', '\u{200d}', '\u{0640}', ' '] {
            start.extend(face.glyph_index(c));
        }
        let forms = substitution_closure(&face, &start);
        wanted.extend(forms.into_iter().map(|g| GLYPH_ID_KEYS + u32::from(g)));
        wanted.sort_unstable();
        wanted.dedup();
    } else if s.pixel == 0 {
        // HarfRust's common/horizontal features, directional mirroring,
        // and automatic fractions (ot_shape.rs collect_features).
        let defaults = [
            "abvm", "blwm", "ccmp", "locl", "mark", "mkmk", "rlig", "calt", "clig", "curs", "dist",
            "kern", "liga", "rclt", "rvrn", "ltra", "ltrm", "rtla", "rtlm", "frac", "numr", "dnom",
        ]
        .map(String::from);
        let lookups = feature_lookups(&face, &defaults);
        wanted.extend(
            closure_of(&face, &shaped, Some(&lookups))
                .into_iter()
                .map(|g| GLYPH_ID_KEYS + u32::from(g)),
        );
        wanted.sort_unstable();
        wanted.dedup();
    }

    // OpenType features: every glyph their lookups substitute for the
    // charset's (small caps, tabular figures), drawn as glyph ids.
    let featured = if s.features.is_empty() {
        0
    } else {
        let lookups = feature_lookups(&face, &s.features);
        let start: Vec<ttf_parser::GlyphId> = wanted
            .iter()
            .filter(|&&cp| cp < GLYPH_ID_KEYS)
            .filter_map(|&cp| face.glyph_index(char::from_u32(cp)?))
            .collect();
        let before = wanted.len();
        // The start glyphs too: a shaped run draws every glyph by its id.
        wanted.extend(
            closure_of(&face, &start, Some(&lookups))
                .into_iter()
                .map(|g| GLYPH_ID_KEYS + u32::from(g)),
        );
        wanted.sort_unstable();
        wanted.dedup();
        wanted.len() - before
    };

    // Kerning between the cooked characters (shaped forms kern in the shaper).
    let kern_glyphs: Vec<(u32, ttf_parser::GlyphId)> = wanted
        .iter()
        .filter(|&&cp| cp < GLYPH_ID_KEYS)
        .filter_map(|&cp| Some((cp, face.glyph_index(char::from_u32(cp)?)?)))
        .collect();
    let upem_f = f32::from(face.units_per_em());
    let kern_rows: Vec<KernRow> = kerning(&face, &kern_glyphs)
        .into_iter()
        .map(|(left, right, v)| KernRow {
            left,
            right,
            adjust: match s.pixel {
                0 => v as f32 / upem_f,
                g => og_ui::glyphs::snap_to_grid(v as f32 / upem_f, g),
            },
        })
        .filter(|k| k.adjust != 0.0)
        .collect();
    // Render in parallel; results land by index, so the order is fixed.
    let slots: Vec<std::sync::Mutex<Option<Glyph>>> =
        wanted.iter().map(|_| std::sync::Mutex::new(None)).collect();
    let next = std::sync::atomic::AtomicUsize::new(0);
    std::thread::scope(|scope| {
        for _ in 0..threads.max(1).min(wanted.len()) {
            scope.spawn(|| {
                let face = ttf_parser::Face::parse(bytes, 0).expect("parsed above");
                loop {
                    let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    if i >= wanted.len() {
                        break;
                    }
                    *slots[i].lock().expect("no poisoning") = render(&face, wanted[i], s);
                }
            });
        }
    });
    let mut glyphs: Vec<Glyph> = slots
        .into_iter()
        .filter_map(|m| m.into_inner().expect("no poisoning"))
        .collect();
    // Unicode spaces the cooked glyphs lack (CLDR writes en's "3:05 PM" and
    // fr's "1 234" with U+202F, and fonts often leave it out): empty glyphs
    // that advance as the space would, so no space vanishes from a text.
    let mut spaces = unicode_spaces(&face, &glyphs);
    if s.pixel > 0 {
        for g in &mut spaces {
            g.advance = og_ui::glyphs::snap_to_grid(g.advance, s.pixel);
        }
    }
    if !spaces.is_empty() {
        glyphs.extend(spaces);
        glyphs.sort_by_key(|g| g.codepoint);
    }
    // `color true`: the glyphs with a COLR paint come off the distance
    // field and are drawn in colour on pages of their own.
    let Colour {
        plain: colour,
        black,
        white,
        bent,
    } = if s.color {
        colour_glyphs(bytes, &face, &mut glyphs, s)?
    } else {
        Colour::default()
    };

    let page = s.page_size.max(64);
    let field = shelf(&glyphs, page)?;
    let (width, height, page_count, last_height) =
        (field.width, field.height, field.count, field.last);
    let mut rows = Vec::with_capacity(glyphs.len());
    for (i, g) in glyphs.iter().enumerate() {
        let (p, gx, gy) = field.place[i];
        let [l, b, r, t] = g.plane.unwrap_or([0.0; 4]);
        rows.push(GlyphRow {
            codepoint: g.codepoint,
            page: p,
            advance: g.advance,
            plane_left: l,
            plane_bottom: b,
            plane_right: r,
            plane_top: t,
            atlas_x: if g.plane.is_some() { gx } else { 0 },
            atlas_y: if g.plane.is_some() { gy } else { 0 },
            atlas_w: g.width,
            atlas_h: g.height,
        });
    }
    // A colour glyph's row points past the field's pages, at its image:
    // the colour pages', then the foreground pages' (its black image).
    let mut point = |glyphs: &[Glyph], first: u32| -> Result<Option<Shelf>, CookError> {
        if glyphs.is_empty() {
            return Ok(None);
        }
        let pages = shelf(glyphs, page)?;
        for (i, g) in glyphs.iter().enumerate() {
            let (p, gx, gy) = pages.place[i];
            let [l, b, r, t] = g.plane.expect("a colour glyph has ink");
            let row = rows
                .iter_mut()
                .find(|r| r.codepoint == g.codepoint)
                .expect("a colour glyph is a cooked glyph");
            *row = GlyphRow {
                page: first + p,
                plane_left: l,
                plane_bottom: b,
                plane_right: r,
                plane_top: t,
                atlas_x: gx,
                atlas_y: gy,
                atlas_w: g.width,
                atlas_h: g.height,
                ..*row
            };
        }
        Ok(Some(pages))
    };
    let colour_pages = point(&colour, page_count)?;
    let first_foreground = page_count + colour_pages.as_ref().map_or(0, |c| c.count);
    // A foreground page holds each glyph twice, side by side: painted with
    // a black foreground on the left half, white on the right. The white
    // images are the same sizes, so they pack to the same places.
    let foreground_pages = point(&black, first_foreground)?.map(|b| {
        let w = shelf(&white, page).expect("packs as the black images did");
        debug_assert_eq!(w.place, b.place);
        let pages = b
            .pages
            .iter()
            .zip(&w.pages)
            .map(|(l, r)| {
                let row = (b.width * 4) as usize;
                l.chunks_exact(row)
                    .zip(r.chunks_exact(row))
                    .flat_map(|(a, b)| a.iter().chain(b).copied())
                    .collect()
            })
            .collect();
        Shelf {
            width: b.width * 2,
            pages,
            ..b
        }
    });
    let upem = face.units_per_em() as f32;
    let meta = FontMeta {
        units_per_em: face.units_per_em() as u32,
        em_px: s.em_px,
        px_range: s.px_range,
        ascender: face.ascender() as f32 / upem,
        descender: face.descender() as f32 / upem,
        line_gap: face.line_gap() as f32 / upem,
        glyph_count: rows.len() as u32,
        page_count,
        page_width: width,
        page_height: height,
        // Shaped forms are glyphs, not code points of the font's cmap.
        uncooked_codepoints: in_font.len().saturating_sub(
            rows.iter()
                .filter(|r| in_font.binary_search(&r.codepoint).is_ok())
                .count(),
        ) as u32,
    };
    let mut w = BlobWriter::new(
        format::Font::type_hash(),
        format::Font::schema_version(),
        stamp,
    );
    w.table("meta", &[meta], row_hash::<FontMeta>())
        .table("glyphs", &rows, row_hash::<GlyphRow>())
        .table("kerning", &kern_rows, row_hash::<KernRow>());
    for (i, px) in field.pages.iter().enumerate() {
        let h = (px.len() / (width as usize * 4)) as u32;
        let ktx = ktx2::write_rgba8_zlib(width, h, px, false, false, format::COOKER_VERSION);
        w.raw(&format!("atlas.{i}"), ktx, 16);
    }
    let mut colour_bytes = 0;
    if let Some(c) = &colour_pages {
        // sRGB colour, straight alpha; the runtime makes its mips.
        for (i, px) in c.pages.iter().enumerate() {
            let h = (px.len() / (c.width as usize * 4)) as u32;
            let ktx = ktx2::write_rgba8_zlib(c.width, h, px, true, false, format::COOKER_VERSION);
            colour_bytes += ktx.len();
            w.raw(&format!("color.{i}"), ktx, 16);
        }
    }
    let mut foreground_bytes = 0;
    if let Some(c) = &foreground_pages {
        for (i, px) in c.pages.iter().enumerate() {
            let h = (px.len() / (c.width as usize * 4)) as u32;
            let ktx = ktx2::write_rgba8_zlib(c.width, h, px, true, false, format::COOKER_VERSION);
            foreground_bytes += ktx.len();
            w.raw(&format!("foreground.{i}"), ktx, 16);
        }
    }
    if s.runtime || shaping {
        w.raw("ttf", bytes.to_vec(), 16);
    } else {
        w.raw("shaping", shaping_tables(&face), 16);
    }
    if s.pixel > 0 {
        // The grid, as text: og-ui snaps the font's sizes to its multiples.
        w.raw("pixel", s.pixel.to_string().into_bytes(), 1);
    }
    let pages_note = if page_count == 1 {
        format!("1 page of {width}x{height}")
    } else {
        format!("{page_count} pages of {width}x{height}, the last {width}x{last_height}")
    };
    let mut note = format!(
        "{} glyphs of {} in the font, {pages_note}, {} px/em, range {} px",
        rows.len(),
        in_font.len(),
        s.em_px,
        s.px_range
    );
    if !kern_rows.is_empty() {
        note.push_str(&format!("; {} kerning pair(s)", kern_rows.len()));
    }
    if let Some(c) = &colour_pages {
        note.push_str(&format!(
            "; {} colour glyph(s) at {} px/em on {} page(s) of {}x{} ({colour_bytes} bytes)",
            colour.len(),
            s.color_px,
            c.count,
            c.width,
            c.height
        ));
    }
    if let Some(c) = &foreground_pages {
        note.push_str(&format!(
            "; {} colour glyph(s) in the text's colour on {} page(s) of {}x{} ({foreground_bytes} bytes)",
            black.len(),
            c.count,
            c.width,
            c.height
        ));
    }
    if !bent.is_empty() {
        let name = |k: u32| {
            if k >= GLYPH_ID_KEYS {
                format!("glyph {}", k - GLYPH_ID_KEYS)
            } else {
                format!("U+{k:04X}")
            }
        };
        let (names, worst): (Vec<String>, Vec<u8>) =
            bent.iter().map(|&(k, e)| (name(k), e)).unzip();
        note.push_str(&format!(
            "; warning: {} blend the text's colour non-linearly (a separable blend that \
             branches on it, it multiplied or screened by itself, a clamping plus, or a \
             hue, saturation, colour or luminosity blend): drawn as the mix of their black \
             and white paints, they are off by up to {} of 255 levels in a text colour \
             other than black or white",
            names.join(", "),
            worst.iter().max().expect("one")
        ));
    }
    if s.runtime && !shaping {
        note.push_str(&format!(
            "; the font file ({} bytes) for characters outside the charset, rendered at run time",
            bytes.len()
        ));
    }
    if shaping {
        let forms = rows.iter().filter(|r| r.codepoint >= GLYPH_ID_KEYS).count();
        note.push_str(&format!(
            "; shaped script: {forms} contextual form(s) and the font file ({} bytes) for run-time shaping",
            bytes.len()
        ));
    }
    if !s.features.is_empty() {
        note.push_str(&format!(
            "; OpenType features {}: {featured} glyph(s) by id and the font file for run-time shaping",
            s.features.join(" ")
        ));
    }
    note.push_str(&figures_note(&face, &wanted, &s.features));
    if s.pixel > 0 {
        note.push_str(&format!("; pixel font on a {} px grid", s.pixel));
        let (off, all) = off_grid(&face, &wanted, s.pixel);
        if off * 10 > all {
            note.push_str(&format!(
                "; warning: {off} of {all} outline points are off the {} px grid \\
                 (a face drawn on another pitch): its pixels are taken at the cells' centres",
                s.pixel
            ));
        }
    }
    if !missing.is_empty() {
        let list: String = missing
            .iter()
            .take(8)
            .filter_map(|&c| char::from_u32(c))
            .collect();
        note.push_str(&format!(
            "; {} requested character(s) not in the font: {list}",
            missing.len()
        ));
    }
    Ok((w.finish(), FontNotes { note }))
}

/// Glyphs placed on atlas pages: each glyph's (page, x, y) and the pages'
/// pixels.
struct Shelf {
    place: Vec<(u32, u32, u32)>,
    count: u32,
    width: u32,
    /// The first page's height (every page's but the last when several).
    height: u32,
    /// The last page's height.
    last: u32,
    pages: Vec<Vec<u8>>,
}

/// Shelf packing: tallest first, 1-pixel gutters, pages of `page`² (the
/// last, often the only one, cut to the rows it uses).
fn shelf(glyphs: &[Glyph], page: u32) -> Result<Shelf, CookError> {
    let area: u64 = glyphs
        .iter()
        .map(|g| (g.width as u64 + 1) * (g.height as u64 + 1))
        .sum();
    let mut width = 64u32;
    while (width as u64) * (width as u64) < area * 5 / 4 && width < page {
        width *= 2;
    }
    let mut order: Vec<usize> = (0..glyphs.len())
        .filter(|&i| glyphs[i].plane.is_some())
        .collect();
    order.sort_by_key(|&i| {
        (
            std::cmp::Reverse(glyphs[i].height),
            std::cmp::Reverse(glyphs[i].width),
            glyphs[i].codepoint,
        )
    });
    let mut place = vec![(0u32, 0u32, 0u32); glyphs.len()];
    let (mut pg, mut x, mut y, mut shelf) = (0u32, 1u32, 1u32, 0u32);
    let mut used_height = 0;
    for &i in &order {
        let g = &glyphs[i];
        if g.width + 2 > width || g.height + 2 > page {
            return Err(CookError(format!(
                "glyph U+{:04X} ({}x{} px) does not fit a {page}-pixel page; lower em_px",
                g.codepoint, g.width, g.height
            )));
        }
        if x + g.width + 1 > width {
            x = 1;
            y += shelf + 1;
            shelf = 0;
        }
        if y + g.height + 1 > page {
            pg += 1;
            x = 1;
            y = 1;
            shelf = 0;
            used_height = 0;
        }
        place[i] = (pg, x, y);
        x += g.width + 1;
        shelf = shelf.max(g.height);
        used_height = used_height.max(y + g.height + 1);
    }
    let count = pg + 1;
    // An empty page (every glyph in colour) keeps one row of 4.
    let last_height = used_height.div_ceil(4).max(1) * 4;
    let mut pages: Vec<Vec<u8>> = (0..count)
        .map(|p| {
            let h = if p == pg { last_height } else { page };
            vec![0u8; (width * h * 4) as usize]
        })
        .collect();
    for (i, g) in glyphs.iter().enumerate() {
        if g.plane.is_some() {
            let (p, gx, gy) = place[i];
            let dst = &mut pages[p as usize];
            for row in 0..g.height {
                let s0 = (row * g.width * 4) as usize;
                let d0 = (((gy + row) * width + gx) * 4) as usize;
                dst[d0..d0 + (g.width * 4) as usize]
                    .copy_from_slice(&g.pixels[s0..s0 + (g.width * 4) as usize]);
            }
        }
    }
    Ok(Shelf {
        place,
        count,
        width,
        height: if count == 1 { last_height } else { page },
        last: last_height,
        pages,
    })
}

/// A font's colour glyphs (`font { color true }`).
#[derive(Default)]
struct Colour {
    /// The glyphs drawn in their own colours.
    plain: Vec<Glyph>,
    /// The glyphs that use the text's colour (COLR palette entry 0xFFFF),
    /// painted with a black foreground and, in the same order and sizes,
    /// a white one: the renderer mixes the two by the text's colour.
    black: Vec<Glyph>,
    white: Vec<Glyph>,
    /// The glyphs in the text's colour the renderer's mix of the two is
    /// off for (`colr::mix_error` over `colr::MIX_PROBES`), by how many
    /// 8-bit levels at worst.
    bent: Vec<(u32, u8)>,
}

/// `font { color true }`: each cooked glyph with a COLR paint drawn in
/// colour at `color_px` (colr.rs) and taken off the distance field, which
/// keeps its advance. A font none of whose cooked glyphs is coloured is
/// an error: the setting would ship nothing.
fn colour_glyphs(
    bytes: &[u8],
    face: &ttf_parser::Face,
    glyphs: &mut [Glyph],
    s: &FontSettings,
) -> Result<Colour, CookError> {
    let font = skrifa::FontRef::new(bytes).map_err(|e| CookError(format!("font: {e}")))?;
    let palette = colr::palette(&font, s.palette).map_err(CookError)?;
    let mut out = Colour::default();
    for g in glyphs.iter_mut() {
        let gid = if (GLYPH_ID_KEYS..GLYPH_ID_KEYS + 0x1_0000).contains(&g.codepoint) {
            g.codepoint - GLYPH_ID_KEYS
        } else {
            match char::from_u32(g.codepoint).and_then(|c| face.glyph_index(c)) {
                Some(id) => u32::from(id.0),
                None => continue,
            }
        };
        let paint = |foreground| {
            colr::render(
                &font,
                skrifa::GlyphId::new(gid),
                s.color_px,
                &palette,
                foreground,
            )
            .map_err(|e| CookError(format!("U+{:04X}: {e}", g.codepoint)))
        };
        let Some(image) = paint([0.0; 3])? else {
            continue;
        };
        let glyph = |image: colr::ColorImage| Glyph {
            codepoint: g.codepoint,
            advance: g.advance,
            plane: Some(image.plane),
            width: image.width,
            height: image.height,
            pixels: image.rgba,
        };
        if image.foreground {
            let white = paint([1.0; 3])?.expect("painted in black");
            // The renderer mixes the two by the text's colour: exact where
            // the paint is affine in it; painted at a few colours to find
            // where a blend bends it.
            let mut worst = 0;
            for fg in colr::MIX_PROBES {
                let probe = paint(fg)?.expect("painted in black");
                worst = worst.max(colr::mix_error(&image.rgba, &white.rgba, &probe.rgba, fg));
            }
            if worst > 2 {
                out.bent.push((g.codepoint, worst));
            }
            out.white.push(glyph(white));
            out.black.push(glyph(image));
        } else {
            out.plain.push(glyph(image));
        }
        *g = Glyph {
            plane: None,
            width: 0,
            height: 0,
            pixels: Vec::new(),
            ..*g
        };
    }
    if out.plain.is_empty() && out.black.is_empty() {
        return Err(CookError(
            "color true, but no character of the charset has a COLR colour glyph".into(),
        ));
    }
    Ok(out)
}

/// The warning for a face whose figures are proportional (the typography
/// audit, fix 4): a score or a timer in it changes width as it ticks unless
/// its figures are tabular. With the face's own `tnum` imported, nothing;
/// otherwise og-ui sets the digits of the numeric role and of a `tnum`
/// step on the widest figure's advance, each centred, which the note says.
fn figures_note(face: &ttf_parser::Face, wanted: &[u32], features: &[String]) -> String {
    let em = f32::from(face.units_per_em());
    let advances: Vec<f32> = ('0'..='9')
        .filter(|&c| wanted.binary_search(&(c as u32)).is_ok())
        .filter_map(|c| face.glyph_index(c))
        .filter_map(|g| face.glyph_hor_advance(g))
        .map(|a| f32::from(a) / em)
        .collect();
    let (lo, hi) = advances
        .iter()
        .fold((f32::MAX, 0.0f32), |(lo, hi), &a| (lo.min(a), hi.max(a)));
    if advances.len() < 2 || hi - lo < 1e-3 {
        return String::new();
    }
    let tnum = ["tnum".to_string()];
    let asked = features.iter().any(|t| t == "tnum");
    if asked && !feature_lookups(face, &tnum).is_empty() {
        return String::new();
    }
    let has = if feature_lookups(face, &tnum).is_empty() {
        "the face has no `tnum`"
    } else {
        "`features tnum` is not imported"
    };
    format!(
        "; warning: proportional figures ({lo:.3} to {hi:.3} em) and {has}: a numeric \
         text in it is set on the widest figure's advance, each digit centred"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A font imported with `features smcp` carries the small-cap glyph of
    /// each lower-case letter by id (what og-ui draws a featured run with)
    /// and its file for shaping; without the line only the file ships.
    #[test]
    fn features_put_their_glyphs_in_the_atlas() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/ui-text/assets/fonts/EBGaramond-Medium.ttf"
        );
        let bytes = std::fs::read(path).expect("the kit's EB Garamond");
        let face = ttf_parser::Face::parse(&bytes, 0).unwrap();
        let a = face.glyph_index('a').unwrap();
        let lookups = feature_lookups(&face, &["smcp".to_string()]);
        let small_a: Vec<u16> = closure_of(&face, &[a], Some(&lookups))
            .into_iter()
            .filter(|&g| g != a.0)
            .collect();
        assert_eq!(small_a.len(), 1, "smcp maps a to one small cap");
        let cooked = |features: &[&str]| {
            let s = FontSettings {
                charset: vec!["ascii".into()],
                em_px: 16.0,
                features: features.iter().map(|t| t.to_string()).collect(),
                ..FontSettings::default()
            };
            let (blob, _) = cook(&bytes, &s, 2, 0).expect("cooks");
            let b = og_blob::Blob::parse(&blob, "test").unwrap();
            let rows: Vec<u32> = b
                .table::<GlyphRow>("glyphs", row_hash::<GlyphRow>())
                .unwrap()
                .iter()
                .map(|r| r.codepoint)
                .collect();
            (rows, b.raw("ttf").is_ok() || b.raw("shaping").is_ok())
        };
        let (with, ships) = cooked(&["smcp"]);
        assert!(ships, "the file ships for run-time shaping");
        assert!(
            with.contains(&(GLYPH_ID_KEYS + u32::from(small_a[0]))),
            "a.sc by id"
        );
        assert!(
            with.contains(&(GLYPH_ID_KEYS + u32::from(a.0))),
            "a by id too"
        );
        let (without, ships) = cooked(&[]);
        assert!(ships);
        assert!(!without.contains(&(GLYPH_ID_KEYS + u32::from(small_a[0]))));
        assert!(without.contains(&(GLYPH_ID_KEYS + u32::from(a.0))));
    }

    #[test]
    fn latin_default_contextual_glyphs_are_cooked() {
        let bytes = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/ui-text/assets/fonts/Inter-Display.ttf"
        ))
        .unwrap();
        let (blob, _) = cook(
            &bytes,
            &FontSettings {
                charset: vec!["ascii".into()],
                em_px: 16.0,
                ..Default::default()
            },
            2,
            0,
        )
        .unwrap();
        let blob = og_blob::Blob::parse(&blob, "Inter").unwrap();
        let rows = blob
            .table::<GlyphRow>("glyphs", row_hash::<GlyphRow>())
            .unwrap();
        // This fixture's calt numeric hyphen in "Level 1-2": a nested
        // lookup, not a lookup directly listed by the feature record.
        assert!(rows.iter().any(|g| g.codepoint == GLYPH_ID_KEYS + 213));
    }

    /// Subtexel samples of `c`'s field at `em_px` whose position lies in the
    /// outline's fill (nonzero winding) a pixel or more from its boundary
    /// (the centre and a ring of radius 1 px all filled, so an edge of one
    /// contour inside another's fill is no boundary) and whose
    /// median(r, g, b) is not above 0.55.
    fn seam_texels(face: &ttf_parser::Face, c: char, em_px: f32) -> usize {
        use fdsm::transform::Transform;
        let gid = face.glyph_index(c).expect("in the font");
        let mut outline = fdsm_ttf_parser::load_shape_from_face(face, gid).expect("an outline");
        // fdsm sorts scanline x by bits, so negative bearings must be
        // shifted for the independent fill oracle too.
        let shift = f64::from(face.units_per_em()) * 4.0;
        outline.transform(&nalgebra::convert::<_, nalgebra::Affine2<f64>>(
            nalgebra::Translation2::new(shift, 0.0),
        ));
        let outline = outline.prepare();
        let filled =
            |x: f64, y: f64| outline.scanline(y).cursor().sum_intersections(x + shift) != 0;
        let s = FontSettings {
            em_px,
            ..FontSettings::default()
        };
        let g = render(face, c as u32, &s).expect("a glyph");
        let [left, bottom, ..] = g.plane.expect("not empty");
        let em = f64::from(em_px);
        let px = f64::from(face.units_per_em()) / em;
        let mut seam = 0;
        for row in 0..(g.height - 1) * 4 {
            // Rows run top to bottom; font units are y-up.
            let sy = f64::from(row) / 4.0;
            let y = (f64::from(bottom) * em + f64::from(g.height) - sy - 0.5) * px;
            for col in 0..(g.width - 1) * 4 {
                let sx = f64::from(col) / 4.0;
                let x = (f64::from(left) * em + sx + 0.5) * px;
                let deep = filled(x, y)
                    && (0..16).all(|k| {
                        let a = f64::from(k) * std::f64::consts::TAU / 16.0;
                        filled(x + a.cos() * px, y + a.sin() * px)
                    });
                let sample = |dx, dy, c| {
                    f64::from(
                        g.pixels[(((row / 4 + dy) * g.width + col / 4 + dx) * 4 + c) as usize],
                    ) / 255.0
                };
                let mut m = [0, 1, 2].map(|c| {
                    let mix = |a, b, t| a * (1.0 - t) + b * t;
                    mix(
                        mix(sample(0, 0, c), sample(1, 0, c), sx.fract()),
                        mix(sample(0, 1, c), sample(1, 1, c), sx.fract()),
                        sy.fract(),
                    )
                });
                m.sort_by(f64::total_cmp);
                if deep && m[1] <= 0.55 {
                    seam += 1;
                }
            }
        }
        seam
    }

    /// Glyphs drawn as overlapping contours (the TrueType OVERLAP flag: the
    /// self-crossing contour of Inter's "e", the join of its "t", the
    /// strokes of 商) read as filled everywhere inside: no seam where one
    /// contour's edge runs through another's fill.
    #[test]
    fn overlapping_contours_leave_no_seam_inside() {
        let fonts = concat!(env!("CARGO_MANIFEST_DIR"), "/../../templates");
        let mut bad = Vec::new();
        for (file, c) in [
            (
                "../tests/fixtures/ui-text/assets/fonts/Inter-Display.ttf",
                'e',
            ),
            (
                "../tests/fixtures/ui-text/assets/fonts/Inter-Display.ttf",
                't',
            ),
            ("styles/cjk/NotoSansSC-Regular.ttf", '商'),
            ("styles/cjk/ZCOOLKuaiLe-Regular.ttf", 'Y'),
            ("styles/cjk/ZCOOLKuaiLe-Regular.ttf", '?'),
            ("styles/shared/ChakraPetch-BoldItalic.ttf", 'A'),
            ("styles/fantasy/Cinzel-Bold.ttf", 'A'),
            ("styles/fantasy/Newsreader-Medium.ttf", 'V'),
        ] {
            let bytes = std::fs::read(format!("{fonts}/{file}")).expect("the font");
            let face = ttf_parser::Face::parse(&bytes, 0).expect("parses");
            for em_px in [32.0, 64.0] {
                let n = seam_texels(&face, c, em_px);
                if n > 0 {
                    bad.push(format!("{file}: {c} at {em_px} px/em: {n} sample(s)"));
                }
            }
        }
        assert!(bad.is_empty(), "seams inside the fill: {bad:?}");
    }

    /// Inter's default figures are proportional: cooked without `tnum` it
    /// is warned about, with the face's own `tnum` it is not.
    #[test]
    fn proportional_figures_are_warned_without_tnum() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/ui-text/assets/fonts/Inter-Display.ttf"
        );
        let bytes = std::fs::read(path).expect("the kit's Inter");
        let note = |features: &[&str]| {
            let s = FontSettings {
                charset: vec!["ascii".into()],
                em_px: 16.0,
                features: features.iter().map(|t| t.to_string()).collect(),
                ..FontSettings::default()
            };
            cook(&bytes, &s, 2, 0).expect("cooks").1.note
        };
        let plain = note(&[]);
        assert!(
            plain.contains("proportional figures") && plain.contains("not imported"),
            "{plain}"
        );
        let tnum = note(&["tnum"]);
        assert!(!tnum.contains("proportional figures"), "{tnum}");
    }

    /// `pixel 12` cooks Fusion Pixel as its 12-pixel cells: every advance
    /// and plane edge a whole number of cells, the grid in the blob for
    /// og-ui, and a face drawn off the grid (DotGothic16's dots, on a pitch
    /// finer than its 16-pixel em) named in the note.
    #[test]
    fn a_pixel_font_cooks_on_its_grid() {
        let read = |p: &str| {
            std::fs::read(format!("{}/../../{p}", env!("CARGO_MANIFEST_DIR"))).expect("the font")
        };
        let cook_px = |bytes: &[u8], pixel: u32| {
            let s = FontSettings {
                charset: vec!["ascii".into()],
                em_px: 48.0,
                px_range: 16.0,
                pixel,
                ..FontSettings::default()
            };
            let (blob, notes) = cook(bytes, &s, 2, 0).expect("cooks");
            (blob, notes.note)
        };
        let (blob, note) = cook_px(&read("templates/styles/cjk/FusionPixel12-zhHans.ttf"), 12);
        let b = og_blob::Blob::parse(&blob, "test").unwrap();
        assert_eq!(b.raw("pixel").unwrap(), b"12");
        let pad = ((16.0f32 / 2.0).ceil() + 1.0) / 48.0;
        let whole = |v: f32| (v * 12.0 - (v * 12.0).round()).abs() < 1e-4;
        let rows = b
            .table::<GlyphRow>("glyphs", row_hash::<GlyphRow>())
            .unwrap();
        assert!(rows.len() > 90);
        for r in rows.iter() {
            assert!(
                whole(r.advance),
                "{:?} advance {}",
                char::from_u32(r.codepoint),
                r.advance
            );
            if r.atlas_w > 0 {
                assert!(
                    whole(r.plane_left + pad) && whole(r.plane_bottom + pad),
                    "{:?}: {} {}",
                    char::from_u32(r.codepoint),
                    r.plane_left,
                    r.plane_bottom
                );
            }
        }
        assert!(
            note.contains("pixel font") && !note.contains("off the"),
            "{note}"
        );
        let (_, note) = cook_px(
            &read("tests/fixtures/pixel-font/DotGothic16-Regular.ttf"),
            16,
        );
        assert!(note.contains("off the 16 px grid"), "{note}");
        let (plain, _) = cook_px(&read("templates/styles/cjk/FusionPixel12-zhHans.ttf"), 0);
        assert!(
            og_blob::Blob::parse(&plain, "test")
                .unwrap()
                .raw("pixel")
                .is_err()
        );
    }

    /// `color true`: a COLR v1 glyph cooks as an sRGB image on a colour
    /// page past the field's, its gradient's colours intact; the same font
    /// cooked without it ships no colour page; a font without colour
    /// glyphs refuses it.
    #[test]
    fn colour_glyphs_cook_onto_their_own_pages_only_when_asked() {
        let bytes = std::fs::read(format!(
            "{}/../../tests/fixtures/ui-text/assets/fonts/test_glyphs-glyf_colr_1.ttf",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        // A linear gradient (U+F0100), a sweep (U+F0200), a radial
        // (U+F0503) and composites (U+F0600).
        let chars = "\u{F0100}\u{F0200}\u{F0503}\u{F0600}";
        let settings = |color| FontSettings {
            charset: Vec::new(),
            chars: chars.into(),
            color,
            ..FontSettings::default()
        };
        let (blob, notes) = cook(&bytes, &settings(true), 2, 0).expect("cooks");
        // The four, and the same glyphs by id for shaped runs.
        assert!(
            notes.note.contains("8 colour glyph(s) at 64 px/em"),
            "{}",
            notes.note
        );
        let b = og_blob::Blob::parse(&blob, "test").unwrap();
        let meta = b.table::<FontMeta>("meta", row_hash::<FontMeta>()).unwrap()[0];
        let rows = b
            .table::<GlyphRow>("glyphs", row_hash::<GlyphRow>())
            .unwrap();
        let (w, h, rgba) = ktx2::decode_rgba8(b.raw("color.0").unwrap(), 0).unwrap();
        assert!(b.raw_opt("color.1").is_none());
        // Its characters sit in Supplementary Private Use Area-A, as an
        // icon font's do, and each has a by-id twin past Unicode for shaped
        // runs: the same glyph under its own key, never the character's.
        let face = ttf_parser::Face::parse(&bytes, 0).unwrap();
        for c in chars.chars() {
            let id = face.glyph_index(c).unwrap().0;
            let by_id = rows
                .iter()
                .find(|r| r.codepoint == GLYPH_ID_KEYS + u32::from(id))
                .unwrap_or_else(|| panic!("{c:?}'s glyph {id} by id"));
            let r = rows.iter().find(|r| r.codepoint == c as u32).unwrap();
            assert_eq!((by_id.atlas_w, by_id.atlas_h), (r.atlas_w, r.atlas_h));
            assert_eq!(by_id.key(), Some(og_render::GlyphKey::Id(id)));
            assert_eq!(r.key(), Some(og_render::GlyphKey::Char(c)));
        }
        assert!(
            rows.iter().all(|r| r.codepoint >= GLYPH_ID_KEYS
                || chars.contains(char::from_u32(r.codepoint).unwrap())),
            "no row but the characters and the glyphs by id"
        );
        for c in chars.chars() {
            let r = rows.iter().find(|r| r.codepoint == c as u32).unwrap();
            assert_eq!(r.page, meta.page_count, "{c:?} is on the colour page");
            assert!(r.atlas_x + r.atlas_w <= w && r.atlas_y + r.atlas_h <= h);
            // The image is its plane at 64 px/em; every clip box here is
            // at most an em wide, with 4 px of padding each side.
            let plane_w = (r.plane_right - r.plane_left) * 64.0;
            assert!((r.atlas_w as f32 - plane_w).abs() < 0.5, "{c:?}");
            assert!((40..=72).contains(&r.atlas_w), "{c:?} {}", r.atlas_w);
        }
        // The linear gradient of U+F0100 runs red to blue across its box.
        let r = rows.iter().find(|r| r.codepoint == 0xF0100).unwrap();
        let px = |fx: f32| {
            let x = r.atlas_x + (fx * r.atlas_w as f32) as u32;
            let y = r.atlas_y + r.atlas_h / 2;
            let i = ((y * w + x) * 4) as usize;
            [rgba[i], rgba[i + 1], rgba[i + 2], rgba[i + 3]]
        };
        let (left, right) = (px(0.2), px(0.8));
        assert!(left[3] == 255 && right[3] == 255, "{left:?} {right:?}");
        assert!(
            left[0] > left[2] + 100 && right[2] > right[0] + 100,
            "{left:?} {right:?}"
        );
        // Without the setting: the distance field alone, as before.
        let (plain, notes) = cook(&bytes, &settings(false), 2, 0).expect("cooks");
        let b = og_blob::Blob::parse(&plain, "test").unwrap();
        assert!(b.raw_opt("color.0").is_none() && !notes.note.contains("colour"));
        // A font without colour glyphs refuses it.
        let inter = std::fs::read(format!(
            "{}/../../tests/fixtures/ui-text/assets/fonts/Inter-Display.ttf",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let s = FontSettings {
            color: true,
            ..FontSettings::default()
        };
        let e = cook(&inter, &s, 2, 0).err().expect("refused");
        assert!(
            e.0.contains("no character of the charset has a COLR"),
            "{}",
            e.0
        );
    }

    /// `palette` picks the CPAL palette the colour glyphs are painted
    /// with: by index, or the first the font marks for a dark or a light
    /// background (the test font's 1 and 2); a palette the font lacks is
    /// refused, naming how many it has.
    #[test]
    fn a_colour_font_cooks_in_the_palette_it_names() {
        let bytes = std::fs::read(format!(
            "{}/../../tests/fixtures/ui-text/assets/fonts/test_glyphs-glyf_colr_1.ttf",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let page = |palette| {
            let s = FontSettings {
                charset: Vec::new(),
                chars: "\u{F0100}".into(),
                color: true,
                palette,
                ..FontSettings::default()
            };
            let (blob, _) = cook(&bytes, &s, 2, 0).map_err(|e| e.0)?;
            let b = og_blob::Blob::parse(&blob, "test").unwrap();
            let rows = b
                .table::<GlyphRow>("glyphs", row_hash::<GlyphRow>())
                .unwrap();
            let r = *rows.iter().find(|r| r.codepoint == 0xF0100).unwrap();
            let (w, _, rgba) = ktx2::decode_rgba8(b.raw("color.0").unwrap(), 0).unwrap();
            // The gradient's first fifth, mid-height.
            let (x, y) = (r.atlas_x + r.atlas_w / 5, r.atlas_y + r.atlas_h / 2);
            let i = ((y * w + x) * 4) as usize;
            Ok::<_, String>((rgba.clone(), [rgba[i], rgba[i + 1], rgba[i + 2]]))
        };
        let (first, red) = page(Palette::First).unwrap();
        let (dark, navy) = page(Palette::Dark).unwrap();
        let (light, orange) = page(Palette::Light).unwrap();
        assert_eq!(page(Palette::Index(1)).unwrap().0, dark);
        assert_eq!(page(Palette::Index(2)).unwrap().0, light);
        assert!(first != dark && dark != light);
        // Palette 0 starts red, 1 a dark navy, 2 orange.
        assert!(red[0] > 200 && red[1] < 60, "{red:?}");
        assert!(
            navy[2] > navy[0] && navy.iter().all(|&v| v < 140),
            "{navy:?}"
        );
        assert!(
            orange[0] > 230 && (90..170).contains(&orange[1]) && orange[2] < 60,
            "{orange:?}"
        );
        let e = page(Palette::Index(3)).expect_err("refused");
        assert!(e.contains("3 CPAL palette(s)"), "{e}");
    }

    /// A colour glyph whose paint uses the text's colour (COLR palette
    /// entry 0xFFFF) cooks twice onto a foreground page, black on the left
    /// half and white at the same place on the right, the same alpha in
    /// both; the font's other colour glyphs keep their page byte for byte.
    #[test]
    fn foreground_glyphs_cook_black_and_white_side_by_side() {
        let bytes = std::fs::read(format!(
            "{}/../../tests/fixtures/ui-text/assets/fonts/test_glyphs-glyf_colr_1.ttf",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let settings = |chars: &str| FontSettings {
            charset: Vec::new(),
            chars: chars.into(),
            color: true,
            ..FontSettings::default()
        };
        // U+F0B00: a linear gradient across 0.1..0.9 em, orange (palette
        // entry 1) at both ends and the foreground at the middle.
        let (without, _) = cook(&bytes, &settings("\u{F0100}\u{F0600}"), 2, 0).expect("cooks");
        let (with, notes) =
            cook(&bytes, &settings("\u{F0100}\u{F0600}\u{F0B00}"), 2, 0).expect("cooks");
        // Its gradient is affine in the foreground: the mix is exact.
        assert!(!notes.note.contains("non-linearly"), "{}", notes.note);
        assert!(
            notes.note.contains("4 colour glyph(s) at 64 px/em")
                && notes
                    .note
                    .contains("2 colour glyph(s) in the text's colour on 1 page(s)"),
            "{}",
            notes.note
        );
        let (a, b) = (
            og_blob::Blob::parse(&without, "test").unwrap(),
            og_blob::Blob::parse(&with, "test").unwrap(),
        );
        assert_eq!(a.raw("color.0").unwrap(), b.raw("color.0").unwrap());
        assert!(a.raw_opt("foreground.0").is_none() && b.raw_opt("foreground.1").is_none());
        let meta = b.table::<FontMeta>("meta", row_hash::<FontMeta>()).unwrap()[0];
        let rows = b
            .table::<GlyphRow>("glyphs", row_hash::<GlyphRow>())
            .unwrap();
        let r = rows.iter().find(|r| r.codepoint == 0xF0B00).unwrap();
        assert_eq!(r.page, meta.page_count + 1, "past the colour page");
        let (w, h, rgba) = ktx2::decode_rgba8(b.raw("foreground.0").unwrap(), 0).unwrap();
        assert!(r.atlas_x + r.atlas_w <= w / 2 && r.atlas_y + r.atlas_h <= h);
        let px = |x: u32, y: u32| {
            let i = ((y * w + x) * 4) as usize;
            [rgba[i], rgba[i + 1], rgba[i + 2], rgba[i + 3]]
        };
        for y in r.atlas_y..r.atlas_y + r.atlas_h {
            for x in r.atlas_x..r.atlas_x + r.atlas_w {
                assert_eq!(px(x, y)[3], px(x + w / 2, y)[3], "alpha at {x},{y}");
            }
        }
        // The image is 4 px of padding then 0.1..0.9 em at 64 px/em.
        let y = r.atlas_y + r.atlas_h / 2;
        let mid = r.atlas_x + r.atlas_w / 2;
        let near = |p: [u8; 4], q: [u8; 4]| p.iter().zip(q).all(|(&a, b)| a.abs_diff(b) <= 6);
        assert!(near(px(mid, y), [0, 0, 0, 255]), "{:?}", px(mid, y));
        assert!(
            near(px(mid + w / 2, y), [255; 4]),
            "{:?}",
            px(mid + w / 2, y)
        );
        // A pixel into the ink: 6% of the way to the foreground.
        let edge = r.atlas_x + 5;
        for x in [edge, edge + w / 2] {
            let [r, g, b, a] = px(x, y);
            assert!(
                r >= 238 && (150..=170).contains(&g) && b <= 16 && a == 255,
                "{:?}",
                px(x, y)
            );
        }
    }
}
