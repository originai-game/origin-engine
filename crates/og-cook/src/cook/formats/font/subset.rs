//! A face's ideographs, kana and hangul cut to the characters a game shows.
//!
//! A CJK face is mostly its ideographs (Noto Sans SC's variable build: 17 MB,
//! 99 % of it `glyf` and `gvar`), and a game shows a few hundred of them.
//! [`subset`] keeps every glyph id where it is and empties the outline and
//! variation data of each glyph no kept character reaches, so no table that
//! names glyphs (GSUB, GPOS, hmtx, HVAR, BASE) needs rewriting, and the cmap
//! maps only the kept characters: a character the cut dropped is one the
//! face lacks, which run-time fallback and the missing-glyph report see.
//! Characters outside [`cut`]'s blocks (Latin, punctuation, symbols) all
//! stay: a game's dynamic text (numbers, names) draws whatever it holds.

use std::collections::BTreeSet;
use std::path::Path;

/// Whether `c` is in a block a face ships only as the game shows it: CJK
/// ideographs and radicals, kana and hangul.
pub fn cut(c: char) -> bool {
    matches!(c as u32,
        0x1100..=0x11FF       // Hangul Jamo
        | 0x2E80..=0x2FDF     // CJK and Kangxi radicals
        | 0x3040..=0x30FF     // Hiragana, Katakana
        | 0x3100..=0x312F     // Bopomofo
        | 0x3130..=0x318F     // Hangul compatibility Jamo
        | 0x31A0..=0x31FF     // Bopomofo extended, CJK strokes, Katakana extension
        | 0x3400..=0x4DBF     // Extension A
        | 0x4E00..=0x9FFF     // Unified ideographs
        | 0xA960..=0xA97F     // Hangul Jamo extended A
        | 0xAC00..=0xD7FF     // Hangul syllables, Jamo extended B
        | 0xF900..=0xFAFF     // Compatibility ideographs
        | 0xFF66..=0xFFDC     // Halfwidth kana and hangul
        | 0x20000..=0x3FFFF) // Extensions B onwards
}

/// The characters in [`cut`]'s blocks the project under `root` shows: those
/// of every asset that is text (`assets`: string tables, stories,
/// documents) and every file under `src/` (scripts and their JSX). A face
/// ships these, its `chars` line's, and every character outside the
/// blocks.
pub fn shown<'a>(root: &Path, assets: impl Iterator<Item = &'a [u8]>) -> BTreeSet<char> {
    fn walk(dir: &Path, out: &mut BTreeSet<char>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for e in entries.flatten() {
            let p = e.path();
            let Ok(t) = e.file_type() else { continue };
            if t.is_symlink() {
                continue;
            }
            if t.is_dir() {
                let name = e.file_name();
                if !matches!(
                    name.to_str(),
                    Some(".og" | "cooked" | "og-out" | "node_modules")
                ) {
                    walk(&p, out);
                }
                continue;
            }
            // Text only: a binary asset is no UTF-8 (and a font's own
            // cmap is not text the game shows).
            if let Ok(bytes) = std::fs::read(&p)
                && let Ok(text) = std::str::from_utf8(&bytes)
            {
                out.extend(text.chars().filter(|&c| cut(c)));
            }
        }
    }
    let mut out = BTreeSet::new();
    for bytes in assets {
        if let Ok(text) = std::str::from_utf8(bytes) {
            out.extend(text.chars().filter(|&c| cut(c)));
        }
    }
    walk(&root.join("src"), &mut out);
    out
}

/// The characters of [`cut`]'s blocks the face maps.
pub fn mapped_cut(face: &ttf_parser::Face) -> BTreeSet<char> {
    let mut out = BTreeSet::new();
    if let Some(cmap) = face.tables().cmap {
        for sub in cmap.subtables {
            if sub.is_unicode() {
                sub.codepoints(|cp| {
                    if let Some(c) = char::from_u32(cp).filter(|&c| cut(c)) {
                        out.insert(c);
                    }
                });
            }
        }
    }
    out
}

/// What [`subset`] kept.
pub struct Subset {
    pub bytes: Vec<u8>,
    /// Characters of [`cut`]'s blocks kept, and how many the face mapped.
    pub kept: usize,
    pub of: usize,
}

/// `bytes` (a TrueType-outline face) with the characters of [`cut`]'s
/// blocks other than `keep` dropped: their glyphs, and the glyphs only they
/// reach through GSUB or composites, emptied in `glyf` and `gvar`, and the
/// cmap holding only what stays. `None` for a face it does not cut: one
/// with PostScript outlines, or none of the blocks' characters to drop.
pub fn subset(bytes: &[u8], keep: &BTreeSet<char>) -> Option<Subset> {
    subset_where(bytes, |c| !cut(c) || keep.contains(&c))
}

/// `bytes` keeping only the characters `stays` takes ([`subset`]).
fn subset_where(bytes: &[u8], stays: impl Fn(char) -> bool) -> Option<Subset> {
    let face = ttf_parser::Face::parse(bytes, 0).ok()?;
    let raw = face.raw_face();
    let table = |t: &[u8; 4]| raw.table(ttf_parser::Tag::from_bytes(t));
    let (glyf, loca, head) = (table(b"glyf")?, table(b"loca")?, table(b"head")?);
    let n = face.number_of_glyphs() as usize;
    let long = i16::from_be_bytes(head.get(50..52)?.try_into().ok()?) == 1;
    let offset = |g: usize| -> Option<usize> {
        if long {
            Some(u32::from_be_bytes(loca.get(g * 4..g * 4 + 4)?.try_into().ok()?) as usize)
        } else {
            Some(u16::from_be_bytes(loca.get(g * 2..g * 2 + 2)?.try_into().ok()?) as usize * 2)
        }
    };

    // The characters that stay, and their glyphs.
    let mut mapping: Vec<(u32, u16)> = Vec::new();
    let (mut of, mut kept, mut dropped) = (0, 0, false);
    let cmap = face.tables().cmap?;
    let mut seen = BTreeSet::new();
    for sub in cmap.subtables {
        if !sub.is_unicode() {
            continue;
        }
        sub.codepoints(|cp| {
            if seen.insert(cp) {
                let Some(c) = char::from_u32(cp) else { return };
                let stays = stays(c);
                if cut(c) {
                    of += 1;
                    kept += usize::from(stays);
                }
                if !stays {
                    dropped = true;
                    return;
                }
                if let Some(g) = face.glyph_index(c) {
                    mapping.push((cp, g.0));
                }
            }
        });
    }
    if !dropped {
        return None;
    }
    mapping.sort_unstable();

    // Their glyphs, what GSUB makes of them, and composites' components.
    let start: Vec<ttf_parser::GlyphId> = std::iter::once(0)
        .chain(mapping.iter().map(|&(_, g)| g))
        .map(ttf_parser::GlyphId)
        .collect();
    let mut live = vec![false; n];
    let mut todo: Vec<u16> = super::substitution_closure(&face, &start);
    todo.extend(start.iter().map(|g| g.0));
    while let Some(g) = todo.pop() {
        let g = g as usize;
        if g >= n || live[g] {
            continue;
        }
        live[g] = true;
        let data = glyf.get(offset(g)?..offset(g + 1)?)?;
        todo.extend(components(data));
    }

    // glyf and loca (long offsets): the live glyphs' data, empty otherwise.
    let mut new_glyf = Vec::new();
    let mut new_loca = Vec::with_capacity((n + 1) * 4);
    for (g, &l) in live.iter().enumerate() {
        new_loca.extend((new_glyf.len() as u32).to_be_bytes());
        if l {
            new_glyf.extend_from_slice(glyf.get(offset(g)?..offset(g + 1)?)?);
            while new_glyf.len() % 4 != 0 {
                new_glyf.push(0);
            }
        }
    }
    new_loca.extend((new_glyf.len() as u32).to_be_bytes());
    let mut new_head = head.to_vec();
    new_head[50..52].copy_from_slice(&1i16.to_be_bytes());

    let mut font = write_fonts::FontBuilder::new();
    for rec in raw.table_records {
        let tag = rec.tag.to_bytes();
        let data = match &tag {
            // A signature no longer holds for the cut file.
            b"DSIG" => continue,
            b"glyf" => new_glyf.clone(),
            b"loca" => new_loca.clone(),
            b"head" => new_head.clone(),
            b"cmap" => cmap12(&mapping),
            b"gvar" => gvar(raw.table(rec.tag)?, &live)?,
            // Glyph names of a version 2 `post` cost little and stay.
            _ => raw.table(rec.tag)?.to_vec(),
        };
        font.add_raw(write_fonts::types::Tag::new(&tag), data);
    }
    Some(Subset {
        bytes: font.build(),
        kept,
        of,
    })
}

/// The glyphs a composite glyph's components name (none for a simple one).
fn components(data: &[u8]) -> Vec<u16> {
    let mut out = Vec::new();
    if data.len() < 10 || i16::from_be_bytes([data[0], data[1]]) >= 0 {
        return out;
    }
    let mut at = 10;
    while let Some(c) = data.get(at..at + 4) {
        let flags = u16::from_be_bytes([c[0], c[1]]);
        out.push(u16::from_be_bytes([c[2], c[3]]));
        at += 4 + if flags & 0x1 != 0 { 4 } else { 2 };
        at += match flags {
            f if f & 0x8 != 0 => 2,
            f if f & 0x40 != 0 => 4,
            f if f & 0x80 != 0 => 8,
            _ => 0,
        };
        if flags & 0x20 == 0 {
            break;
        }
    }
    out
}

/// `gvar` with the variation data of glyphs not `live` dropped, its
/// offsets long.
fn gvar(t: &[u8], live: &[bool]) -> Option<Vec<u8>> {
    let u16_at = |i: usize| Some(u16::from_be_bytes(t.get(i..i + 2)?.try_into().ok()?));
    let u32_at = |i: usize| Some(u32::from_be_bytes(t.get(i..i + 4)?.try_into().ok()?));
    let axes = u16_at(4)? as usize;
    let tuples = u16_at(6)? as usize;
    let tuples_at = u32_at(8)? as usize;
    let count = u16_at(12)? as usize;
    let flags = u16_at(14)?;
    let data_at = u32_at(16)? as usize;
    let long = flags & 1 != 0;
    let off = |g: usize| -> Option<usize> {
        Some(if long {
            u32_at(20 + g * 4)? as usize
        } else {
            u16_at(20 + g * 2)? as usize * 2
        })
    };
    let shared = t.get(tuples_at..tuples_at + axes * tuples * 2)?;
    let new_tuples_at = 20 + (count + 1) * 4;
    let new_data_at = new_tuples_at + shared.len();
    let mut out = t[..20].to_vec();
    out[8..12].copy_from_slice(&(new_tuples_at as u32).to_be_bytes());
    out[14..16].copy_from_slice(&(flags | 1).to_be_bytes());
    out[16..20].copy_from_slice(&(new_data_at as u32).to_be_bytes());
    let mut data = Vec::new();
    for g in 0..count {
        out.extend((data.len() as u32).to_be_bytes());
        if live.get(g).copied().unwrap_or(false) {
            data.extend_from_slice(t.get(data_at + off(g)?..data_at + off(g + 1)?)?);
            while data.len() % 2 != 0 {
                data.push(0);
            }
        }
    }
    out.extend((data.len() as u32).to_be_bytes());
    out.extend_from_slice(shared);
    out.extend(data);
    Some(out)
}

/// A `cmap` of one format 12 subtable mapping `mapping` (sorted), as
/// Unicode full repertoire (0, 4) and Windows full repertoire (3, 10).
fn cmap12(mapping: &[(u32, u16)]) -> Vec<u8> {
    let mut groups: Vec<[u32; 3]> = Vec::new();
    for &(cp, g) in mapping {
        match groups.last_mut() {
            Some(last) if last[1] + 1 == cp && last[2] + (cp - last[0]) == g as u32 => last[1] = cp,
            _ => groups.push([cp, cp, g as u32]),
        }
    }
    let mut out = Vec::new();
    out.extend(0u16.to_be_bytes());
    out.extend(2u16.to_be_bytes());
    for (platform, encoding) in [(0u16, 4u16), (3, 10)] {
        out.extend(platform.to_be_bytes());
        out.extend(encoding.to_be_bytes());
        out.extend(20u32.to_be_bytes());
    }
    out.extend(12u16.to_be_bytes());
    out.extend(0u16.to_be_bytes());
    out.extend((16 + 12 * groups.len() as u32).to_be_bytes());
    out.extend(0u32.to_be_bytes());
    out.extend((groups.len() as u32).to_be_bytes());
    for g in groups {
        for v in g {
            out.extend(v.to_be_bytes());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use skrifa::instance::{LocationRef, Size};
    use skrifa::outline::{DrawSettings, OutlinePen};
    use skrifa::{FontRef, MetadataProvider, Tag};

    fn bytes(path: &str) -> Vec<u8> {
        std::fs::read(format!(
            "{}/../../templates/{path}",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap()
    }

    /// Every pen command, as text.
    #[derive(Default)]
    struct Record(String);
    impl OutlinePen for Record {
        fn move_to(&mut self, x: f32, y: f32) {
            self.0 += &format!("M{x},{y}");
        }
        fn line_to(&mut self, x: f32, y: f32) {
            self.0 += &format!("L{x},{y}");
        }
        fn quad_to(&mut self, a: f32, b: f32, x: f32, y: f32) {
            self.0 += &format!("Q{a},{b},{x},{y}");
        }
        fn curve_to(&mut self, a: f32, b: f32, c: f32, d: f32, x: f32, y: f32) {
            self.0 += &format!("C{a},{b},{c},{d},{x},{y}");
        }
        fn close(&mut self) {
            self.0 += "Z";
        }
    }

    fn glyph(font: &[u8], c: char) -> skrifa::GlyphId {
        FontRef::new(font).unwrap().charmap().map(c).unwrap()
    }

    /// How `c` draws in `font` at `axes` (`None`: unmapped), and its advance.
    fn drawn(font: &[u8], c: char, axes: &[(&str, f32)]) -> Option<(String, Option<f32>)> {
        let g = FontRef::new(font).unwrap().charmap().map(c)?;
        Some(drawn_id(font, g, axes))
    }

    /// How glyph `g` draws in `font` at `axes`, and its advance.
    fn drawn_id(font: &[u8], g: skrifa::GlyphId, axes: &[(&str, f32)]) -> (String, Option<f32>) {
        let f = FontRef::new(font).unwrap();
        let loc = f.axes().location(
            axes.iter()
                .map(|&(t, v)| (Tag::new(&t.as_bytes().try_into().unwrap()), v)),
        );
        let mut pen = Record::default();
        f.outline_glyphs()
            .get(g)
            .unwrap()
            .draw(DrawSettings::unhinted(Size::new(64.0), &loc), &mut pen)
            .unwrap();
        let adv = f
            .glyph_metrics(Size::new(64.0), LocationRef::from(&loc))
            .advance_width(g);
        (pen.0, adv)
    }

    /// A Chinese face keeps exactly the ideographs the game shows, each
    /// drawn as before (a composite's or a substitution's glyphs dropped
    /// with the rest would change or blank it), and every Latin character
    /// whatever the game shows; a dropped ideograph is unmapped, so
    /// fallback and the missing-glyph report see it, never a blank.
    #[test]
    fn a_cjk_face_keeps_the_ideographs_shown_and_all_else() {
        let data = bytes("styles/cjk/NotoSansSC-Regular.ttf");
        let keep: BTreeSet<char> = "开始游戏关第".chars().collect();
        let s = subset(&data, &keep).expect("it cuts");
        assert_eq!(s.kept, keep.len());
        assert!(s.of > 6000, "{}", s.of);
        assert!(
            s.bytes.len() * 10 < data.len(),
            "{} of {}",
            s.bytes.len(),
            data.len()
        );
        for c in "开始游戏关第Level 3·0123456789ABCxyz，。".chars() {
            assert_eq!(drawn(&s.bytes, c, &[]), drawn(&data, c, &[]), "{c}");
            assert!(drawn(&s.bytes, c, &[]).is_some(), "{c}");
        }
        for c in "设置退出".chars() {
            assert!(drawn(&data, c, &[]).is_some());
            assert_eq!(drawn(&s.bytes, c, &[]), None, "{c} is dropped");
        }
        assert!(
            subset(
                &data,
                &mapped_cut(&ttf_parser::Face::parse(&data, 0).unwrap())
            )
            .is_none(),
            "nothing to drop"
        );
    }

    /// A variable face's kept glyphs vary as before: their `gvar` data
    /// moved with them (a wrong offset draws another glyph's deltas, or
    /// none, away from the default), at a far corner of both axes. Glyphs
    /// no character maps stay when a kept one reaches them: per fontTools,
    /// ď is a composite of d's outline and glyph 471 (`uni030C.alt`), and
    /// f i ligates to glyph 312 (`f_i`); dropping either blanks or breaks
    /// what a game draws.
    #[test]
    fn a_variable_faces_kept_glyphs_keep_their_variations() {
        let data = bytes("styles/shared/InstrumentSans[wdth,wght].ttf");
        let s = subset_where(&data, |c| !"bpq".contains(c)).expect("it cuts");
        let corner = [("wght", 700.0), ("wdth", 75.0)];
        assert_ne!(
            drawn(&data, 'a', &[]),
            drawn(&data, 'a', &corner),
            "the face varies"
        );
        for axes in [
            &[][..],
            &[("wght", 700.0), ("wdth", 75.0)],
            &[("wght", 400.0), ("wdth", 100.0)],
        ] {
            for c in "aceghmoRSW08&ďť".chars() {
                assert_eq!(
                    drawn(&s.bytes, c, axes),
                    drawn(&data, c, axes),
                    "{c} at {axes:?}"
                );
            }
            for g in [471, 312] {
                let g = skrifa::GlyphId::new(g);
                assert_eq!(
                    drawn_id(&s.bytes, g, axes),
                    drawn_id(&data, g, axes),
                    "{g} at {axes:?}"
                );
            }
            assert_eq!(drawn(&s.bytes, 'b', axes), None);
            assert_eq!(
                drawn_id(&s.bytes, glyph(&data, 'b'), axes).0,
                "",
                "b's outline is dropped"
            );
        }
    }
}
