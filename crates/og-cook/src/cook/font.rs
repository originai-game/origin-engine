//! TrueType/OpenType → font blobs: an MTSDF atlas (RGB multi-channel and A
//! true signed distance, fdsm) and a glyph table for the declared charset.
//!
//! Shaped scripts (Arabic and the others og-i18n's `needs_shaping` names):
//! when the charset holds one, the blob also carries the font file (raw
//! section `ttf`) for og-ui's run-time shaping, and the atlas every glyph the
//! font's substitutions reach from the charset's (contextual forms,
//! ligatures), each under the private-use code point U+F0000 + glyph id,
//! which is what og-ui draws a shaped run as.

use super::{CookError, FontSettings};
use crate::format::{self, *};
use crate::ktx2;
use og_blob::BlobWriter;

/// Where a glyph id is drawn from: U+F0000 + id (Supplementary Private Use Area-A).
pub const GLYPH_PUA: u32 = 0xF_0000;

/// Every glyph the font's GSUB substitutions reach from `start`, start included.
fn substitution_closure(face: &ttf_parser::Face, start: &[ttf_parser::GlyphId]) -> Vec<u16> {
    use ttf_parser::gsub::SubstitutionSubtable as S;
    let mut set: std::collections::BTreeSet<u16> = start.iter().map(|g| g.0).collect();
    let Some(gsub) = face.tables().gsub else {
        return set.into_iter().collect();
    };
    loop {
        let before = set.len();
        let now: Vec<ttf_parser::GlyphId> = set.iter().map(|&g| ttf_parser::GlyphId(g)).collect();
        for lookup in gsub.lookups {
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

/// The code points a charset word names.
// A charset is a list of ranges and some charsets are one range: clippy's
// single_range_in_vec_init reads `vec![a..=b]` as a mistaken `(a..=b).collect()`.
#[allow(clippy::single_range_in_vec_init)]
fn charset(word: &str) -> Option<Vec<std::ops::RangeInclusive<u32>>> {
    Some(match word {
        "ascii" => vec![0x20..=0x7e],
        "latin" => vec![0x20..=0x7e, 0xa0..=0xff],
        "punct" => vec![0x2000..=0x206f],
        "cjk-punct" => vec![0x3000..=0x303f, 0xff00..=0xffef],
        "kana" => vec![0x3040..=0x30ff],
        "hangul" => vec![0x1100..=0x11ff, 0x3130..=0x318f, 0xac00..=0xd7af],
        "arabic" => vec![
            0x0600..=0x06ff,
            0x0750..=0x077f,
            0xfb50..=0xfdff,
            0xfe70..=0xfeff,
        ],
        "all" => vec![0..=0x10ffff],
        _ => return None,
    })
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
    // A private-use code point past GLYPH_PUA is a glyph id (a shaped form).
    let gid = if (GLYPH_PUA..GLYPH_PUA + 0x1_0000).contains(&cp) {
        ttf_parser::GlyphId((cp - GLYPH_PUA) as u16)
    } else {
        face.glyph_index(char::from_u32(cp)?)?
    };
    let upem = face.units_per_em() as f64;
    let advance = face.glyph_hor_advance(gid).unwrap_or(0) as f32 / upem as f32;
    let empty = Glyph {
        codepoint: cp,
        advance,
        plane: None,
        width: 0,
        height: 0,
        pixels: Vec::new(),
    };
    let Some(bbox) = face.glyph_bounding_box(gid) else {
        return Some(empty);
    };
    let Some(mut shape) = fdsm_ttf_parser::load_shape_from_face(face, gid) else {
        return Some(empty);
    };
    if shape.contours.is_empty() || bbox.width() <= 0 || bbox.height() <= 0 {
        return Some(empty);
    }
    let scale = s.em_px as f64 / upem;
    let pad = (s.px_range as f64 / 2.0).ceil() + 1.0;
    let width = ((bbox.width() as f64) * scale + 2.0 * pad).ceil() as u32;
    let height = ((bbox.height() as f64) * scale + 2.0 * pad).ceil() as u32;
    let tx = pad - bbox.x_min as f64 * scale;
    let ty = pad - bbox.y_min as f64 * scale;
    let transform = nalgebra::convert::<_, nalgebra::Affine2<f64>>(nalgebra::Similarity2::new(
        nalgebra::Vector2::new(tx, ty),
        0.0,
        scale,
    ));
    shape.transform(&transform);
    let colored = Shape::edge_coloring_simple(shape, 0.03, 0x6f67_636f_6f6b);
    let prepared = colored.clone().prepare();
    // In f32, so msdfgen's error correction can run before the sign
    // correction (as msdfgen orders them), then quantised to bytes.
    let mut field: image::ImageBuffer<image::Rgba<f32>, Vec<f32>> =
        image::ImageBuffer::new(width, height);
    fdsm::generate::generate_mtsdf(&prepared, s.px_range as f64, &mut field);
    // msdfgen's error correction (edge priority, distance checks at edges):
    // texels whose channels would interpolate into a false edge, the
    // artefacts at sharp corners and close contours, become single-channel.
    fdsm::correct_error::correct_error_mtsdf(
        &mut field,
        &colored,
        &prepared,
        s.px_range as f64,
        &fdsm::correct_error::ErrorCorrectionConfig::default(),
    );
    fdsm::render::correct_sign_mtsdf(&mut field, &prepared, FillRule::Nonzero);
    let mut img = image::RgbaImage::new(width, height);
    for (o, i) in img.pixels_mut().zip(field.pixels()) {
        *o = image::Rgba(i.0.map(|v| (f64::from(v.clamp(0.0, 1.0)) * 255.0) as u8));
    }
    // Font units are y-up; atlas rows go top to bottom.
    image::imageops::flip_vertical_in_place(&mut img);
    let left = (bbox.x_min as f64 * scale - pad) / s.em_px as f64;
    let bottom = (bbox.y_min as f64 * scale - pad) / s.em_px as f64;
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
        let ranges = charset(word).ok_or_else(|| {
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
        .filter(|cp| in_font.binary_search(cp).is_err())
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
    // Shaped scripts: every form the substitutions reach, as glyph ids.
    let shaped: Vec<ttf_parser::GlyphId> = wanted
        .iter()
        .filter_map(|&cp| char::from_u32(cp))
        .filter(|&c| og_i18n::needs_shaping(c))
        .filter_map(|c| face.glyph_index(c))
        .collect();
    let shaping = !shaped.is_empty();
    if shaping {
        // The marks and joiners shaping inserts or keeps.
        let mut start = shaped;
        for c in ['\u{200c}', '\u{200d}', '\u{0640}', ' '] {
            start.extend(face.glyph_index(c));
        }
        let forms = substitution_closure(&face, &start);
        wanted.extend(forms.into_iter().map(|g| GLYPH_PUA + u32::from(g)));
        wanted.sort_unstable();
        wanted.dedup();
    }

    // Kerning between the cooked characters (shaped forms kern in the shaper).
    let kern_glyphs: Vec<(u32, ttf_parser::GlyphId)> = wanted
        .iter()
        .filter(|&&cp| cp < GLYPH_PUA)
        .filter_map(|&cp| Some((cp, face.glyph_index(char::from_u32(cp)?)?)))
        .collect();
    let upem_f = f32::from(face.units_per_em());
    let kern_rows: Vec<KernRow> = kerning(&face, &kern_glyphs)
        .into_iter()
        .map(|(left, right, v)| KernRow {
            left,
            right,
            adjust: v as f32 / upem_f,
        })
        .collect();
    // The font file ships with the atlas for run-time shaping and for
    // characters outside the charset (`runtime true`: og-ui renders them
    // from these outlines when a text needs them).
    let ship_ttf = shaping || s.runtime;

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
    let glyphs: Vec<Glyph> = slots
        .into_iter()
        .filter_map(|m| m.into_inner().expect("no poisoning"))
        .collect();

    // Shelf packing: tallest first, 1-pixel gutters, pages of page_size².
    let page = s.page_size.max(64);
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
        }
        place[i] = (pg, x, y);
        x += g.width + 1;
        shelf = shelf.max(g.height);
        used_height = used_height.max(y + g.height + 1);
    }
    let page_count = pg + 1;
    let height = if page_count == 1 {
        used_height.div_ceil(4) * 4
    } else {
        page
    };
    let mut pages = vec![vec![0u8; (width * height * 4) as usize]; page_count as usize];
    let mut rows = Vec::with_capacity(glyphs.len());
    for (i, g) in glyphs.iter().enumerate() {
        let (p, gx, gy) = place[i];
        if g.plane.is_some() {
            let dst = &mut pages[p as usize];
            for row in 0..g.height {
                let s0 = (row * g.width * 4) as usize;
                let d0 = (((gy + row) * width + gx) * 4) as usize;
                dst[d0..d0 + (g.width * 4) as usize]
                    .copy_from_slice(&g.pixels[s0..s0 + (g.width * 4) as usize]);
            }
        }
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
        uncooked_codepoints: in_font
            .len()
            .saturating_sub(rows.iter().filter(|r| r.codepoint < GLYPH_PUA).count())
            as u32,
    };
    let mut w = BlobWriter::new(
        format::Font::type_hash(),
        format::Font::schema_version(),
        stamp,
    );
    w.table("meta", &[meta], row_hash::<FontMeta>())
        .table("glyphs", &rows, row_hash::<GlyphRow>())
        .table("kerning", &kern_rows, row_hash::<KernRow>());
    for (i, px) in pages.iter().enumerate() {
        let ktx = ktx2::write_rgba8_zlib(width, height, px, false, false, format::COOKER_VERSION);
        w.raw(&format!("atlas.{i}"), ktx, 16);
    }
    if ship_ttf {
        w.raw("ttf", bytes.to_vec(), 16);
    }
    let mut note = format!(
        "{} glyphs of {} in the font, {page_count} page(s) of {width}x{height}, {} px/em, range {} px",
        rows.len(),
        in_font.len(),
        s.em_px,
        s.px_range
    );
    if !kern_rows.is_empty() {
        note.push_str(&format!("; {} kerning pair(s)", kern_rows.len()));
    }
    if s.runtime && !shaping {
        note.push_str(&format!(
            "; the font file ({} bytes) for characters outside the charset, rendered at run time",
            bytes.len()
        ));
    }
    if shaping {
        let forms = rows.iter().filter(|r| r.codepoint >= GLYPH_PUA).count();
        note.push_str(&format!(
            "; shaped script: {forms} contextual form(s) and the font file ({} bytes) for run-time shaping",
            bytes.len()
        ));
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
