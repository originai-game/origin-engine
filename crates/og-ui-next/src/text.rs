//! Text leaves measured with og-text: its cooked font metrics, UAX #14
//! line breaks and Parley/HarfRust shaping, the engine's one text stack.
//! A leaf is measured only when its text or [`Run`] changes: the arena
//! keys rune-taffy's measure cache on both.

use crate::arena::Run;
use og_text::shape::Features;
use og_text::text::{FontMetrics, break_lines, widest_line};

/// The size of `text` run as `run`, given the width it may take.
pub fn measure(m: &FontMetrics, text: &str, run: &Run, width: Option<f32>) -> (f32, f32) {
    // The tree draws a text by its characters: each one its atlas lacks is
    // asked of the font's file (FontMetrics::advance records it), which a
    // run-time font renders before the frame draws. The shaped layout
    // below records nothing of its own.
    for c in text
        .chars()
        .filter(|c| !c.is_whitespace() && !c.is_control())
    {
        m.advance(c);
    }
    if run.nowrap || width.is_none() {
        let lines = text.split('\n').count().max(1);
        let w = widest_line(text, m, run.size, run.tracking, Features::NONE);
        return (w, lines as f32 * run.line);
    }
    let lines = break_lines(text, m, run.size, run.tracking, Features::NONE, width);
    let w = lines.iter().map(|l| l.width).fold(0.0, f32::max);
    (w, lines.len().max(1) as f32 * run.line)
}

#[cfg(test)]
mod tests {
    use super::*;
    use og_text::shape::FontFile;
    use std::sync::Arc;

    /// A run-time font draws only the glyphs its file was asked for: a
    /// measured text asks for each character its atlas lacks.
    #[test]
    fn a_measured_text_asks_its_font_for_the_glyphs_its_atlas_lacks() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../templates/fonts/latin/NotoSans-Latin.ttf"
        );
        let file = FontFile::new(Arc::new(std::fs::read(path).unwrap())).unwrap();
        let atlas = og_draw::MsdfFont {
            atlas: og_draw::TextureHandle::from_slot(1).unwrap(),
            distance_range: 4.0,
            foreground: false,
            silhouette: false,
            line_height: 1.2,
            glyphs: Vec::new(),
            kerning: Vec::new(),
        };
        let m = FontMetrics::from_msdf(&atlas).with_file(Some(file.clone()));
        file.take_wanted();
        measure(&m, "06:55 Dawn", &Run::sized(14.0), None);
        let wanted: Vec<char> = file.take_wanted().iter().filter_map(|k| k.char()).collect();
        for c in "06:5Dawn".chars() {
            assert!(wanted.contains(&c), "{c} asked for: {wanted:?}");
        }
    }
}
