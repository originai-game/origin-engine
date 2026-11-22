//! TrueType/OpenType → font blobs: the face's file as its source wrote it,
//! with its vertical metrics. UI, world and Rive text are laid out and
//! rastered from the file at the size they are drawn (og-text), so nothing
//! of the face is cut or cooked into an atlas.

use super::{
    Action, CookError, Cx, Exec, FontSettings, Job, KindDef, Out, PlanState, Planned, Source,
    font_settings, stamp,
};
use crate::format::{self, *};
use og_blob::BlobWriter;

/// A font's cook notes.
pub struct FontNotes {
    pub note: String,
}

/// Cooks a font: its metrics, its file, and a pixel font's grid.
pub fn cook(bytes: &[u8], s: &FontSettings, stamp: u64) -> Result<(Vec<u8>, FontNotes), CookError> {
    let face = ttf_parser::Face::parse(bytes, 0).map_err(|e| CookError(format!("font: {e}")))?;
    let upem = f32::from(face.units_per_em());
    let meta = FontMeta {
        units_per_em: u32::from(face.units_per_em()),
        em_px: 0.0,
        px_range: 0.0,
        ascender: f32::from(face.ascender()) / upem,
        descender: f32::from(face.descender()) / upem,
        line_gap: f32::from(face.line_gap()) / upem,
        glyph_count: 0,
        page_count: 0,
        page_width: 0,
        page_height: 0,
        uncooked_codepoints: 0,
    };
    let mut w = BlobWriter::new(
        format::Font::type_hash(),
        format::Font::schema_version(),
        stamp,
    );
    w.table("meta", &[meta], row_hash::<FontMeta>())
        .table("glyphs", &Vec::<GlyphRow>::new(), row_hash::<GlyphRow>())
        .table("kerning", &Vec::<KernRow>::new(), row_hash::<KernRow>());
    w.raw("file", bytes.to_vec(), 16);
    let mut note = format!("the font file ({} bytes), whole", bytes.len());
    if s.pixel > 0 {
        // The grid, as text: the raster snaps the face to its multiples.
        w.raw("pixel", s.pixel.to_string().into_bytes(), 1);
        note.push_str(&format!("; pixel font on a {} px grid", s.pixel));
        let ascii: Vec<u32> = (0x20..0x7f).collect();
        let (off, all) = off_grid(&face, &ascii, s.pixel);
        if off * 10 > all {
            note.push_str(&format!(
                "; warning: {off} of {all} outline points are off the {} px grid \
                 (a face drawn on another pitch): its pixels are taken at the cells' centres",
                s.pixel
            ));
        }
    }
    Ok((w.finish(), FontNotes { note }))
}

/// The share of a face's outline points, over its ASCII glyphs, that
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

pub(crate) static FONT: KindDef = KindDef {
    name: "font",
    extensions: &["ttf", "otf"],
    types: &[],
    by_content: true,
    streams: false,
    plan: plan_font,
};

/// The kinds this module cooks.
pub(crate) static KINDS: &[&KindDef] = &[&FONT];

fn plan_font<'a>(
    cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let Cx { options, .. } = *cx;
    let PlanState { early_errors, .. } = state;
    match font_settings(s, &options.font) {
        Ok(f) => {
            // "font 11": the file and its metrics, no atlas.
            let st = stamp(&[b"font 11", f.describe().as_bytes(), &hash]);
            outs.push((
                s.rel.clone(),
                format::Font::NAME,
                st,
                Box::new(FontJob(s, f)),
            ));
        }
        Err(e) => early_errors.push((s.rel.clone(), "font", e)),
    }
}

/// A font's one output: its file and metrics.
struct FontJob<'a>(&'a Source, FontSettings);

impl Job for FontJob<'_> {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let FontJob(s, f) = self;
        cook(&s.bytes, f, p.stamp).map(|(b, n)| (b, vec![n.note]))
    }

    /// The blob is the font file's and its settings' alone: one cooked
    /// face serves every project that imports it the same way (a built-in
    /// style pack's fonts, first cooked when a look switches to the pack).
    fn action(&self) -> Action<'_> {
        let FontJob(s, f) = self;
        Action::Input(s.bytes.as_slice(), format!("font {f:?}"), false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A font ships its file whole with its metrics; a pixel font also its
    /// grid, and a face drawn on another pitch is named in the note.
    #[test]
    fn a_font_ships_its_file_and_a_pixel_font_its_grid() {
        let read = |p: &str| {
            std::fs::read(format!("{}/../../{p}", env!("CARGO_MANIFEST_DIR"))).expect("the font")
        };
        let cook_px = |bytes: &[u8], pixel: u32| {
            let s = FontSettings { pixel };
            let (blob, notes) = cook(bytes, &s, 0).expect("cooks");
            (blob, notes.note)
        };
        let fusion = read("templates/styles/cjk/FusionPixel12-zhHans.ttf");
        let (blob, note) = cook_px(&fusion, 12);
        let b = og_blob::Blob::parse(&blob, "test").unwrap();
        assert_eq!(b.raw("pixel").unwrap(), b"12");
        assert_eq!(b.raw("file").unwrap(), fusion.as_slice());
        assert!(
            note.contains("pixel font") && !note.contains("off the"),
            "{note}"
        );
        let (_, note) = cook_px(
            &read("tests/fixtures/pixel-font/DotGothic16-Regular.ttf"),
            16,
        );
        assert!(note.contains("off the 16 px grid"), "{note}");
        let (plain, _) = cook_px(&fusion, 0);
        let plain = og_blob::Blob::parse(&plain, "test").unwrap();
        assert!(plain.raw("pixel").is_err() && plain.raw_opt("atlas.0").is_none());
    }
}
