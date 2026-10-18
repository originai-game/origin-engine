//! Text measurement and line breaking over an MSDF font's glyph table: the
//! same advances og-render's `Text` lays out with (left to right, no
//! kerning, missing glyphs take no space), so measured text and drawn text
//! agree to the pixel.

use crate::shape::Shaper;
use og_render::{FontHandle, MsdfFont};
use std::sync::Arc;

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
    /// The font file, for a font cooked with a shaped script (its words are
    /// measured and drawn shaped: `crate::shape`).
    pub shaper: Option<Arc<Shaper>>,
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
            shaper: None,
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
            shaper: None,
        }
    }

    /// Shapes this font's shaped-script words with its font file.
    pub fn with_shaper(mut self, shaper: Option<Arc<Shaper>>) -> FontMetrics {
        self.shaper = shaper;
        self
    }

    /// `line` as it is drawn: shaped words, in visual order.
    pub fn visual(&self, line: &str, default: og_i18n::Direction) -> String {
        crate::shape::visual(line, default, self.shaper.as_deref(), &|c| self.has(c))
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

    /// Whether the font has a glyph for `c` (fixed metrics have every one).
    pub fn has(&self, c: char) -> bool {
        self.fixed.is_some() || self.advances.binary_search_by_key(&c, |(k, _)| *k).is_ok()
    }

    /// The width of one line of text at `size` CSS pixels per em (shaped
    /// words at their shaped width).
    pub fn measure(&self, text: &str, size: f32) -> f32 {
        if let Some(s) = &self.shaper
            && text.chars().any(og_i18n::needs_shaping)
        {
            let has = |c: char| self.has(c);
            return text
                .split_inclusive(' ')
                .map(|w| {
                    let (word, space) = w.strip_suffix(' ').map_or((w, false), |x| (x, true));
                    let drawn = s.word(word, &has).unwrap_or_else(|| word.to_string());
                    drawn.chars().map(|c| self.advance(c)).sum::<f32>()
                        + if space { self.advance(' ') } else { 0.0 }
                })
                .sum::<f32>()
                * size;
        }
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

/// Characters a line may not start with (行頭禁則, JIS X 4051 and the
/// Chinese equivalent): closing brackets and quotes, punctuation, the
/// iteration marks, the prolonged sound mark and small kana.
fn no_start(c: char) -> bool {
    matches!(
        c,
        '、' | '。'
            | '，'
            | '．'
            | '・'
            | '：'
            | '；'
            | '？'
            | '！'
            | '‼'
            | '⁇'
            | '⁈'
            | '⁉'
            | 'ー'
            | '‐'
            | '゠'
            | '–'
            | '〜'
            | '～'
            | '…'
            | '‥'
            | '%'
            | '％'
            | '‰'
            | '）'
            | '」'
            | '』'
            | '】'
            | '〕'
            | '〉'
            | '》'
            | '］'
            | '｝'
            | '〙'
            | '〗'
            | '｠'
            | '»'
            | '”'
            | '’'
            | '〟'
            | ')'
            | ']'
            | '}'
            | ','
            | '.'
            | ':'
            | ';'
            | '?'
            | '!'
            | 'ぁ'
            | 'ぃ'
            | 'ぅ'
            | 'ぇ'
            | 'ぉ'
            | 'っ'
            | 'ゃ'
            | 'ゅ'
            | 'ょ'
            | 'ゎ'
            | 'ゕ'
            | 'ゖ'
            | 'ァ'
            | 'ィ'
            | 'ゥ'
            | 'ェ'
            | 'ォ'
            | 'ッ'
            | 'ャ'
            | 'ュ'
            | 'ョ'
            | 'ヮ'
            | 'ヵ'
            | 'ヶ'
            | 'ㇰ'..='ㇿ' | '々' | '〻' | 'ゝ' | 'ゞ' | 'ヽ' | 'ヾ'
    )
}

/// Characters a line may not end with (行末禁則): opening brackets and quotes.
fn no_end(c: char) -> bool {
    matches!(
        c,
        '（' | '「'
            | '『'
            | '【'
            | '〔'
            | '〈'
            | '《'
            | '［'
            | '｛'
            | '〘'
            | '〖'
            | '｟'
            | '«'
            | '“'
            | '‘'
            | '〝'
            | '('
            | '['
            | '{'
    )
}

/// Katakana and the prolonged sound mark: a loanword's letters, kept on one
/// line as a word is (the middle dot `・` between words breaks after it).
fn katakana(c: char) -> bool {
    matches!(c, 'ァ'..='ヺ' | 'ー' | 'ヽ' | 'ヾ' | 'ㇰ'..='ㇿ' | 'ｦ'..='ﾟ')
}

/// Words, spaces and the CJK units: the units lines break between. A CJK
/// character is its own unit (a line may break before or after it), a run
/// of katakana is one, and kinsoku glues a character that may not start a
/// line to the unit before it and one that may not end a line to the unit
/// after it.
fn tokens(paragraph: &str) -> Vec<(&str, bool)> {
    let mut raw: Vec<(usize, usize, bool)> = Vec::new();
    let mut start = 0;
    let mut kind: Option<u8> = None; // 0 word, 1 space, 2 katakana
    let push = |raw: &mut Vec<(usize, usize, bool)>, a: usize, b: usize, k: u8| {
        if a < b {
            raw.push((a, b, k == 1));
        }
    };
    for (i, c) in paragraph.char_indices() {
        let k = if katakana(c) {
            2
        } else if is_cjk(c) {
            3
        } else {
            u8::from(c == ' ')
        };
        match kind {
            Some(prev) if prev == k && k != 3 => {}
            Some(prev) => {
                push(&mut raw, start, i, prev);
                start = i;
                kind = Some(k);
            }
            None => {
                start = i;
                kind = Some(k);
            }
        }
        if k == 3 {
            // Each other CJK character is a unit of its own.
            push(&mut raw, start, i + c.len_utf8(), 3);
            start = i + c.len_utf8();
            kind = None;
        }
    }
    if let Some(k) = kind {
        push(&mut raw, start, paragraph.len(), k);
    }
    // Kinsoku: glue across a boundary with no space where the next unit
    // starts with a no-start character or this one ends with a no-end one.
    let mut out: Vec<(usize, usize, bool)> = Vec::with_capacity(raw.len());
    for (a, b, space) in raw {
        if let Some(last) = out.last_mut()
            && !space
            && !last.2
            && last.1 == a
        {
            let first = paragraph[a..b].chars().next().is_some_and(no_start);
            let open = paragraph[last.0..last.1].chars().last().is_some_and(no_end);
            if first || open {
                last.1 = b;
                continue;
            }
        }
        out.push((a, b, space));
    }
    out.into_iter()
        .map(|(a, b, space)| (&paragraph[a..b], space))
        .collect()
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
