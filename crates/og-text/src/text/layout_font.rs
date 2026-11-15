//! A layout-only OpenType face built from a font's own metrics, for a font
//! Parley cannot lay out from its file: fixed headless metrics, an atlas
//! cooked before faces carried their files, a pixel font (whose advances
//! snap to its grid). It holds what Parley and HarfRust read to place a
//! text — `cmap` (format 12), `hmtx`, `hhea`, `head`, `maxp`, `name` and a
//! format 0 `kern` — and no outlines; nothing is drawn from it. Each of
//! `chars` maps to its own glyph, advancing as [`FontMetrics`] measures it,
//! so a caret placed in its layout stands where the text is drawn.

use super::FontMetrics;
use std::collections::BTreeSet;

/// Font units per em: an advance of up to eight em in steps of 1/8192 em.
const UPM: f32 = 8192.0;

/// The face of `chars` (each one `m` has) in `m`'s metrics.
pub(super) fn build(m: &FontMetrics, chars: &BTreeSet<char>) -> Vec<u8> {
    let chars: Vec<char> = chars.iter().copied().filter(|&c| m.own_has(c)).collect();
    let units = |em: f32| (em * UPM).round().clamp(-32768.0, 32767.0) as i16;
    let advance = |em: f32| (em * UPM).round().clamp(0.0, 65535.0) as u16;
    // Glyph 0 (.notdef) is a missing character: no advance, as og-text measures it.
    let mut hmtx = vec![0u8; 4];
    let mut widest = 0u16;
    for &c in &chars {
        let a = advance(m.own_advance(c));
        widest = widest.max(a);
        hmtx.extend(a.to_be_bytes());
        hmtx.extend(0i16.to_be_bytes());
    }
    let glyphs = chars.len() as u16 + 1;
    let gid = |c: char| chars.binary_search(&c).ok().map(|i| i as u16 + 1);

    let mut cmap = Vec::new();
    put16(&mut cmap, &[0, 1, 3, 10]);
    cmap.extend(12u32.to_be_bytes());
    let groups = chars.len() as u32;
    put16(&mut cmap, &[12, 0]);
    cmap.extend((16 + 12 * groups).to_be_bytes());
    cmap.extend(0u32.to_be_bytes());
    cmap.extend(groups.to_be_bytes());
    for (i, &c) in chars.iter().enumerate() {
        for v in [c as u32, c as u32, i as u32 + 1] {
            cmap.extend(v.to_be_bytes());
        }
    }

    let mut head = Vec::new();
    head.extend(0x0001_0000u32.to_be_bytes());
    head.extend(0x0001_0000u32.to_be_bytes());
    head.extend(0u32.to_be_bytes());
    head.extend(0x5f0f_3cf5u32.to_be_bytes());
    put16(&mut head, &[3, UPM as u16]);
    head.extend([0u8; 16]);
    let (asc, desc) = (units(m.ascender), units(m.descender));
    for v in [0, desc, widest as i16, asc] {
        head.extend(v.to_be_bytes());
    }
    put16(&mut head, &[0, 8, 2, 0, 0]);

    let mut hhea = Vec::new();
    hhea.extend(0x0001_0000u32.to_be_bytes());
    for v in [asc, desc, 0] {
        hhea.extend(v.to_be_bytes());
    }
    put16(
        &mut hhea,
        &[widest, 0, 0, widest, 1, 0, 0, 0, 0, 0, 0, 0, glyphs],
    );

    let mut maxp = Vec::new();
    maxp.extend(0x0000_5000u32.to_be_bytes());
    put16(&mut maxp, &[glyphs]);

    let family: Vec<u16> = "og layout".encode_utf16().collect();
    let mut name = Vec::new();
    put16(
        &mut name,
        &[0, 1, 18, 3, 1, 0x0409, 1, family.len() as u16 * 2, 0],
    );
    put16(&mut name, &family);

    // Kerning between the characters it maps (format 0 holds 10 920 pairs).
    let mut pairs: Vec<(u16, u16, i16)> = m
        .kerning
        .iter()
        .filter_map(|&(key, adjust)| {
            let a = char::from_u32((key >> 32) as u32)?;
            let b = char::from_u32(key as u32)?;
            Some((gid(a)?, gid(b)?, units(adjust)))
        })
        .filter(|p| p.2 != 0)
        .collect();
    pairs.sort_unstable();
    pairs.truncate(10_920);
    let mut kern = Vec::new();
    if !pairs.is_empty() {
        let n = pairs.len() as u16;
        let select = 15 - n.leading_zeros() as u16;
        let range = 6 * (1u16 << select);
        put16(
            &mut kern,
            &[0, 1, 0, 14 + 6 * n, 1, n, range, select, 6 * n - range],
        );
        for (a, b, v) in pairs {
            put16(&mut kern, &[a, b, v as u16]);
        }
    }

    let mut tables: Vec<(&[u8; 4], Vec<u8>)> = vec![
        (b"cmap", cmap),
        (b"head", head),
        (b"hhea", hhea),
        (b"hmtx", hmtx),
        (b"maxp", maxp),
        (b"name", name),
    ];
    if !kern.is_empty() {
        tables.push((b"kern", kern));
    }
    tables.sort_by_key(|(tag, _)| **tag);
    sfnt(&tables)
}

fn put16(out: &mut Vec<u8>, values: &[u16]) {
    for v in values {
        out.extend(v.to_be_bytes());
    }
}

/// The tables in an OpenType file, each four-byte aligned and summed.
fn sfnt(tables: &[(&[u8; 4], Vec<u8>)]) -> Vec<u8> {
    let n = tables.len() as u16;
    let select = 15 - n.leading_zeros() as u16;
    let range = 16 * (1u16 << select);
    let mut out = Vec::new();
    out.extend(0x0001_0000u32.to_be_bytes());
    put16(&mut out, &[n, range, select, 16 * n - range]);
    let mut offset = 12 + 16 * tables.len();
    for (tag, data) in tables {
        let sum = data
            .chunks(4)
            .map(|c| {
                let mut w = [0u8; 4];
                w[..c.len()].copy_from_slice(c);
                u32::from_be_bytes(w)
            })
            .fold(0u32, u32::wrapping_add);
        out.extend(tag.as_slice());
        out.extend(sum.to_be_bytes());
        out.extend((offset as u32).to_be_bytes());
        out.extend((data.len() as u32).to_be_bytes());
        offset += data.len().next_multiple_of(4);
    }
    for (_, data) in tables {
        out.extend(data);
        out.resize(out.len().next_multiple_of(4), 0);
    }
    out
}
