//! Text measurement and line breaking over an MSDF font's glyph table: the
//! same advances og-render's `Text` lays out with (left to right, no
//! kerning, missing glyphs take no space), so measured text and drawn text
//! agree to the pixel.

use og_render::{FontHandle, MsdfFont};

/// Advances and vertical extents of a font, in em.
#[derive(Clone, Debug, PartialEq)]
pub struct FontMetrics {
    /// (codepoint, advance) sorted by codepoint.
    advances: Vec<(char, f32)>,
    /// Every character's advance when the table is empty (`fixed`).
    fixed: Option<f32>,
    /// Baseline to the top of the tallest glyph, in em (positive).
    pub ascender: f32,
    /// Baseline to the bottom of the lowest glyph, in em (negative).
    pub descender: f32,
}

impl FontMetrics {
    /// The metrics of an MSDF font: its advances, and extents measured over
    /// its printable ASCII glyphs (a cooked font knows its real ascender and
    /// descender: pass them with [`FontMetrics::with_extents`]).
    pub fn from_msdf(font: &MsdfFont) -> FontMetrics {
        let mut advances: Vec<(char, f32)> = font
            .glyphs
            .iter()
            .map(|g| (g.codepoint, g.advance))
            .collect();
        advances.sort_by_key(|(c, _)| *c);
        advances.dedup_by_key(|(c, _)| *c);
        let (mut top, mut bottom) = (0.0f32, 0.0f32);
        for g in font
            .glyphs
            .iter()
            .filter(|g| g.codepoint.is_ascii_graphic())
        {
            top = top.max(g.plane_bounds[3]);
            bottom = bottom.min(g.plane_bounds[1]);
        }
        if top <= 0.0 {
            top = 0.8;
            bottom = -0.2;
        }
        FontMetrics {
            advances,
            fixed: None,
            ascender: top,
            descender: bottom,
        }
    }

    /// Every character advances `advance` em; ascender 0.8, descender -0.2.
    /// For layout without a font (a headless executor, a golden test).
    pub fn fixed(advance: f32) -> FontMetrics {
        FontMetrics {
            advances: Vec::new(),
            fixed: Some(advance),
            ascender: 0.8,
            descender: -0.2,
        }
    }

    pub fn with_extents(mut self, ascender: f32, descender: f32) -> FontMetrics {
        self.ascender = ascender;
        self.descender = descender;
        self
    }

    /// A character's advance in em; 0 when the font lacks it (as og-render).
    pub fn advance(&self, c: char) -> f32 {
        if let Some(a) = self.fixed {
            return a;
        }
        match self.advances.binary_search_by_key(&c, |(k, _)| *k) {
            Ok(i) => self.advances[i].1,
            Err(_) => 0.0,
        }
    }

    /// The width of one line of text at `size` CSS pixels per em.
    pub fn measure(&self, text: &str, size: f32) -> f32 {
        text.chars().map(|c| self.advance(c)).sum::<f32>() * size
    }
}

/// A font the UI lays out with and draws with.
#[derive(Clone, Debug, PartialEq)]
pub struct UiFont {
    /// The font in the host's `RenderScene`; `None` measures but draws no text.
    pub handle: Option<FontHandle>,
    pub metrics: FontMetrics,
}

impl UiFont {
    pub fn new(handle: FontHandle, font: &MsdfFont) -> UiFont {
        UiFont {
            handle: Some(handle),
            metrics: FontMetrics::from_msdf(font),
        }
    }

    /// Fixed-advance metrics and nothing drawn: [`FontMetrics::fixed`].
    pub fn unrendered(advance: f32) -> UiFont {
        UiFont {
            handle: None,
            metrics: FontMetrics::fixed(advance),
        }
    }
}

impl Default for UiFont {
    fn default() -> UiFont {
        UiFont::unrendered(0.5)
    }
}

/// One laid-out line.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Line {
    pub text: String,
    pub width: f32,
}

/// Characters that break anywhere (CJK ideographs, kana, Hangul, fullwidth forms).
pub(crate) fn is_cjk(c: char) -> bool {
    matches!(c as u32,
        0x3000..=0x30ff | 0x3400..=0x4dbf | 0x4e00..=0x9fff | 0xac00..=0xd7af
        | 0xf900..=0xfaff | 0xff00..=0xffef | 0x20000..=0x2fa1f)
}

/// Words, spaces and single CJK characters: the units lines break between.
fn tokens(paragraph: &str) -> Vec<(&str, bool)> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut kind: Option<u8> = None; // 0 word, 1 space
    for (i, c) in paragraph.char_indices() {
        if is_cjk(c) {
            if let Some(k) = kind.take() {
                out.push((&paragraph[start..i], k == 1));
            }
            out.push((&paragraph[i..i + c.len_utf8()], false));
            start = i + c.len_utf8();
            continue;
        }
        let k = u8::from(c == ' ');
        match kind {
            Some(prev) if prev == k => {}
            Some(prev) => {
                out.push((&paragraph[start..i], prev == 1));
                start = i;
                kind = Some(k);
            }
            None => {
                start = i;
                kind = Some(k);
            }
        }
    }
    if let Some(k) = kind {
        out.push((&paragraph[start..], k == 1));
    }
    out
}

/// The widest unbreakable unit: a wrapping text's minimum width.
pub(crate) fn longest_word(text: &str, m: &FontMetrics, size: f32) -> f32 {
    text.split('\n')
        .flat_map(tokens)
        .filter(|(_, space)| !space)
        .map(|(w, _)| m.measure(w, size))
        .fold(0.0, f32::max)
}

/// The widest `\n`-separated line, unwrapped.
pub(crate) fn widest_line(text: &str, m: &FontMetrics, size: f32) -> f32 {
    text.split('\n')
        .map(|l| m.measure(l, size))
        .fold(0.0, f32::max)
}

/// Greedy line breaking at `max_width` (`None`: only at `\n`). Spaces at a
/// break are dropped; a unit wider than the line breaks between characters.
pub(crate) fn break_lines(
    text: &str,
    m: &FontMetrics,
    size: f32,
    max_width: Option<f32>,
) -> Vec<Line> {
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let Some(max) = max_width else {
            lines.push(Line {
                text: paragraph.to_owned(),
                width: m.measure(paragraph, size),
            });
            continue;
        };
        let mut line = String::new();
        let mut width = 0.0f32;
        let mut spaces = String::new();
        let mut spaces_w = 0.0f32;
        let mut pushed_any = false;
        for (unit, is_space) in tokens(paragraph) {
            if is_space {
                spaces.push_str(unit);
                spaces_w += m.measure(unit, size);
                continue;
            }
            let w = m.measure(unit, size);
            if !line.is_empty() && width + spaces_w + w > max {
                lines.push(Line {
                    text: std::mem::take(&mut line),
                    width,
                });
                pushed_any = true;
                width = 0.0;
            } else if !line.is_empty() {
                line.push_str(&spaces);
                width += spaces_w;
            }
            spaces.clear();
            spaces_w = 0.0;
            if line.is_empty() && w > max {
                // Break the unit between characters.
                for c in unit.chars() {
                    let cw = m.advance(c) * size;
                    if !line.is_empty() && width + cw > max {
                        lines.push(Line {
                            text: std::mem::take(&mut line),
                            width,
                        });
                        pushed_any = true;
                        width = 0.0;
                    }
                    line.push(c);
                    width += cw;
                }
                continue;
            }
            line.push_str(unit);
            width += w;
        }
        if !line.is_empty() || !pushed_any {
            lines.push(Line { text: line, width });
        }
    }
    lines
}
