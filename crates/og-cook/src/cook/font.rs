//! TrueType/OpenType → font blobs: an MTSDF atlas (RGB multi-channel and A
//! true signed distance, fdsm) and a glyph table for the declared charset.

use super::{CookError, FontSettings};
use crate::format::{self, *};
use crate::ktx2;
use og_blob::BlobWriter;

/// The code points a charset word names.
fn charset(word: &str) -> Option<Vec<std::ops::RangeInclusive<u32>>> {
    Some(match word {
        "ascii" => vec![0x20..=0x7e],
        "latin" => vec![0x20..=0x7e, 0xa0..=0xff],
        "punct" => vec![0x2000..=0x206f],
        "cjk-punct" => vec![0x3000..=0x303f, 0xff00..=0xffef],
        "kana" => vec![0x3040..=0x30ff],
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
    let gid = face.glyph_index(char::from_u32(cp)?)?;
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
    let prepared = colored.prepare();
    let mut img = image::RgbaImage::new(width, height);
    fdsm::generate::generate_mtsdf(&prepared, s.px_range as f64, &mut img);
    fdsm::render::correct_sign_mtsdf(&mut img, &prepared, FillRule::Nonzero);
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
                "unknown charset '{word}' (known: ascii, latin, punct, cjk-punct, kana, all)"
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
        uncooked_codepoints: (in_font.len() - rows.len()) as u32,
    };
    let mut w = BlobWriter::new(
        format::Font::type_hash(),
        format::Font::schema_version(),
        stamp,
    );
    w.table("meta", &[meta], row_hash::<FontMeta>())
        .table("glyphs", &rows, row_hash::<GlyphRow>());
    for (i, px) in pages.iter().enumerate() {
        let ktx = ktx2::write_rgba8_zlib(width, height, px, false, false, format::COOKER_VERSION);
        w.raw(&format!("atlas.{i}"), ktx, 16);
    }
    let mut note = format!(
        "{} glyphs of {} in the font, {page_count} page(s) of {width}x{height}, {} px/em, range {} px",
        rows.len(),
        in_font.len(),
        s.em_px,
        s.px_range
    );
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
