//! Text measurement, line breaking and visual order over an MSDF font's
//! glyph table: the same advances and kerning og-render's `Text` lays out
//! with (missing glyphs take no space), so measured text and drawn text
//! agree to the pixel. A line with right-to-left or shaped characters is
//! laid out by the bidirectional algorithm (UAX #9, unicode-bidi): its runs
//! in visual order, each right-to-left run reversed or shaped in its
//! direction (`crate::shape`), and drawn with the resulting placements.
//!
//! A text that mixes scripts no one font covers is laid out per run
//! ([`FontMetrics::is_mixed`]): each grapheme cluster (a base and its
//! marks) takes the first of the UI's fonts that has it whole (the role's,
//! the locale's, the game's fallbacks, the style's), neutral characters stay
//! with the run before them, and each run is measured, kerned, shaped and
//! drawn with its own font.

use crate::shape::FontFile;
use og_i18n::Direction;
use og_render::{FontHandle, GlyphPlacement, MsdfFont};
use std::ops::Range;
use std::sync::Arc;

/// Advances, kerning and vertical extents of a font, in em.
#[derive(Clone, Debug, PartialEq)]
pub struct FontMetrics {
    /// (codepoint, advance) sorted by codepoint.
    advances: Vec<(char, f32)>,
    /// Every character's advance when the table is empty (`fixed`).
    fixed: Option<f32>,
    /// Kerning pairs (`left << 32 | right`, em), sorted.
    kerning: Vec<(u64, f32)>,
    /// Baseline to the top of the tallest glyph, in em (positive).
    pub ascender: f32,
    /// Baseline to the bottom of the lowest glyph, in em (negative).
    pub descender: f32,
    /// The font file, for a font cooked with a shaped script or `runtime
    /// true`: its shaped runs are measured and drawn shaped, and characters
    /// outside the atlas measured from it (`crate::shape`).
    pub file: Option<Arc<FontFile>>,
    /// The fonts after this one that a mixed text's runs take, first first
    /// (set on the font [`crate::Fonts::for_text`] gives a mixed text).
    pub(crate) runs: Option<Arc<[UiFont]>>,
    /// This font with the UI's other fonts after it (`runs`): what a text
    /// of this role takes when it mixes scripts ([`crate::Fonts::link`]).
    pub(crate) mixed: Option<Arc<UiFont>>,
}

/// One glyph of a line as drawn, left to right.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VisualGlyph {
    /// The character drawn (U+F0000 + glyph id for a shaped glyph).
    pub ch: char,
    /// The pen's advance after it, em (kerning and tracking included).
    pub advance: f32,
    /// The glyph's offset from the pen, em (y up).
    pub x: f32,
    pub y: f32,
    /// The logical index (in characters) of the first character it draws.
    pub index: usize,
    /// It belongs to a right-to-left run.
    pub rtl: bool,
    /// The font that draws it: 0 the font laid out with, `k` the `k`-th of
    /// a mixed text's other fonts ([`UiFont::handle_of`]).
    pub font: u8,
}

fn kern_key(a: char, b: char) -> u64 {
    (u64::from(a as u32) << 32) | u64::from(b as u32)
}

/// Brackets a right-to-left run draws mirrored (Unicode's Bidi_Mirrored
/// pairs the UI's fonts carry).
fn mirror(c: char) -> char {
    match c {
        '(' => ')',
        ')' => '(',
        '[' => ']',
        ']' => '[',
        '{' => '}',
        '}' => '{',
        '<' => '>',
        '>' => '<',
        '«' => '»',
        '»' => '«',
        '‹' => '›',
        '›' => '‹',
        '（' => '）',
        '）' => '（',
        '「' => '」',
        '」' => '「',
        _ => c,
    }
}

impl FontMetrics {
    /// The metrics of an MSDF font: its advances and kerning, and extents
    /// measured over its printable ASCII glyphs (a cooked font knows its
    /// real ascender and descender: pass them with
    /// [`FontMetrics::with_extents`]).
    pub fn from_msdf(font: &MsdfFont) -> FontMetrics {
        let mut advances: Vec<(char, f32)> = font
            .glyphs
            .iter()
            .map(|g| (g.codepoint, g.advance))
            .collect();
        advances.sort_by_key(|(c, _)| *c);
        advances.dedup_by_key(|(c, _)| *c);
        let mut kerning: Vec<(u64, f32)> = font
            .kerning
            .iter()
            .map(|k| (kern_key(k.left, k.right), k.adjust))
            .collect();
        kerning.sort_by_key(|(k, _)| *k);
        kerning.dedup_by_key(|(k, _)| *k);
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
            kerning,
            ascender: top,
            descender: bottom,
            file: None,
            runs: None,
            mixed: None,
        }
    }

    /// Every character advances `advance` em; ascender 0.8, descender -0.2.
    /// For layout without a font (a headless executor, a golden test).
    pub fn fixed(advance: f32) -> FontMetrics {
        FontMetrics {
            advances: Vec::new(),
            fixed: Some(advance),
            kerning: Vec::new(),
            ascender: 0.8,
            descender: -0.2,
            file: None,
            runs: None,
            mixed: None,
        }
    }

    /// Shapes this font's shaped runs and measures characters outside its
    /// atlas with its font file.
    pub fn with_file(mut self, file: Option<Arc<FontFile>>) -> FontMetrics {
        self.file = file;
        self
    }

    /// The font file (an older name for [`FontMetrics::with_file`]).
    pub fn with_shaper(self, file: Option<Arc<FontFile>>) -> FontMetrics {
        self.with_file(file)
    }

    pub fn with_extents(mut self, ascender: f32, descender: f32) -> FontMetrics {
        self.ascender = ascender;
        self.descender = descender;
        self
    }

    fn atlas_advance(&self, c: char) -> Option<f32> {
        self.advances
            .binary_search_by_key(&c, |(k, _)| *k)
            .ok()
            .map(|i| self.advances[i].1)
    }

    /// A character's advance in em; 0 when the font lacks it (as og-render).
    /// A character outside the atlas that the font file has is measured from
    /// the file (and rendered at run time). A mixed font measures it with
    /// the font its run takes.
    pub fn advance(&self, c: char) -> f32 {
        if self.runs.is_none() {
            return self.own_advance(c);
        }
        self.part(self.font_of(c, None))
            .map_or(0.0, |m| m.own_advance(c))
    }

    fn own_advance(&self, c: char) -> f32 {
        if let Some(a) = self.fixed {
            return a;
        }
        if let Some(a) = self.atlas_advance(c) {
            return a;
        }
        self.file.as_ref().and_then(|f| f.advance(c)).unwrap_or(0.0)
    }

    /// Whether the font has a glyph for `c` (fixed metrics have every one;
    /// a font with its file has every character the file has; a mixed font
    /// has what any of its fonts has).
    pub fn has(&self, c: char) -> bool {
        self.own_has(c)
            || self
                .runs
                .as_ref()
                .is_some_and(|r| r.iter().any(|f| f.metrics.own_has(c)))
    }

    fn own_has(&self, c: char) -> bool {
        self.fixed.is_some()
            || self.atlas_advance(c).is_some()
            || self.file.as_ref().is_some_and(|f| f.covers(c))
    }

    /// Whether the cooked atlas itself has `c`.
    pub fn in_atlas(&self, c: char) -> bool {
        self.fixed.is_some() || self.atlas_advance(c).is_some()
    }

    /// Whether a text is laid out per run, each run with the first of this
    /// font's fonts that has it (a label mixing scripts no one font covers:
    /// [`crate::Fonts::for_text`]).
    pub fn is_mixed(&self) -> bool {
        self.runs.is_some()
    }

    /// Font `k` of a mixed text (0: this one).
    fn part(&self, k: usize) -> Option<&FontMetrics> {
        match k {
            0 => Some(self),
            k => self.runs.as_ref()?.get(k - 1).map(|f| &f.metrics),
        }
    }

    /// The font (0: this one, `k`: the `k`-th of `runs`) that draws `c` in
    /// a mixed text after a character drawn with `prev`: a neutral
    /// character or a mark stays with `prev` when that font has it; any
    /// other the first font whose atlas has it, else the first whose file
    /// renders it at run time, else `prev`'s (it is missing everywhere).
    fn font_of(&self, c: char, prev: Option<usize>) -> usize {
        let Some(runs) = &self.runs else { return 0 };
        let n = runs.len() + 1;
        let m = |k: usize| self.part(k).expect("a font of the run");
        if let Some(p) = prev
            && sticks(c)
            && m(p).own_has(c)
        {
            return p;
        }
        (0..n)
            .find(|&k| m(k).in_atlas(c))
            .or_else(|| (0..n).find(|&k| m(k).own_has(c)))
            .unwrap_or(prev.unwrap_or(0))
    }

    /// The font (as [`FontMetrics::font_of`]) that draws a grapheme
    /// cluster (a base and its combining marks, a conjunct, an emoji
    /// sequence) whole: its base's font when that font has every character
    /// of it; else the first font whose atlas has them all, else the first
    /// whose atlas or file does; else its base's (a mark no font has is
    /// missing beside its base, not drawn from another face). Joiners and
    /// variation selectors, which no face need draw, are not asked for.
    fn font_of_cluster(&self, cluster: &str, prev: Option<usize>) -> usize {
        let mut chars = cluster.chars();
        let Some(base) = chars.next() else {
            return prev.unwrap_or(0);
        };
        let k = self.font_of(base, prev);
        let Some(runs) = &self.runs else { return k };
        let drawn = || cluster.chars().filter(|&c| !invisible(c));
        if chars.as_str().is_empty() || cluster.chars().skip(1).all(invisible) {
            return k;
        }
        let m = |k: usize| self.part(k).expect("a font of the run");
        if drawn().all(|c| m(k).own_has(c)) {
            return k;
        }
        let n = runs.len() + 1;
        (0..n)
            .find(|&f| drawn().all(|c| m(f).in_atlas(c)))
            .or_else(|| (0..n).find(|&f| drawn().all(|c| m(f).own_has(c))))
            .unwrap_or(k)
    }

    /// A mixed text's font runs in logical order: byte ranges and the font
    /// each takes (one run of font 0 for a font that is not mixed). The
    /// choice is per grapheme cluster ([`og_i18n::graphemes`]), so a base
    /// letter and its marks stay in one run.
    pub(crate) fn font_runs(&self, text: &str) -> Vec<(Range<usize>, usize)> {
        if self.runs.is_none() {
            return vec![(0..text.len(), 0)];
        }
        let mut out: Vec<(Range<usize>, usize)> = Vec::new();
        // Neutral characters before the first letter go with its font.
        let mut prev = text
            .chars()
            .find(|&c| !sticks(c))
            .map(|c| self.font_of(c, None));
        // Character index to byte offset.
        let bytes: Vec<usize> = text
            .char_indices()
            .map(|(i, _)| i)
            .chain([text.len()])
            .collect();
        for (a, b) in og_i18n::graphemes(text) {
            let range = bytes[a]..bytes[b];
            let k = self.font_of_cluster(&text[range.clone()], prev);
            match out.last_mut() {
                Some((r, f)) if *f == k => r.end = range.end,
                _ => out.push((range, k)),
            }
            prev = Some(k);
        }
        out
    }

    /// The ascender and descender (em) a line of `text` is placed by: this
    /// font's, or for a mixed text the highest ascender and the lowest
    /// descender of this font (the line's strut) and the fonts its runs
    /// take. Every run sits on one baseline, and the line box placed
    /// around these extents holds each run's face, as CSS's line box holds
    /// each inline box's: a taller fallback face (an Arabic or Devanagari
    /// run in a Latin label) moves the shared baseline down instead of
    /// rising out of the box.
    pub fn extents(&self, text: &str) -> (f32, f32) {
        if self.runs.is_none() {
            return (self.ascender, self.descender);
        }
        self.font_runs(text)
            .iter()
            .filter_map(|(_, k)| self.part(*k))
            .fold((self.ascender, self.descender), |(a, d), m| {
                (a.max(m.ascender), d.min(m.descender))
            })
    }

    /// The baseline of a line of `text` at `size` in a line box `line_h`
    /// tall whose top is `top`: its [`FontMetrics::extents`] centred in the
    /// box.
    pub fn baseline(&self, text: &str, top: f32, line_h: f32, size: f32) -> f32 {
        let (a, d) = self.extents(text);
        top + (line_h - (a - d) * size) / 2.0 + a * size
    }

    /// The kerning between `a` and the `b` after it, em (0: none).
    pub fn kern(&self, a: char, b: char) -> f32 {
        if self.kerning.is_empty() {
            return 0.0;
        }
        match self
            .kerning
            .binary_search_by_key(&kern_key(a, b), |(k, _)| *k)
        {
            Ok(i) => self.kerning[i].1,
            Err(_) => 0.0,
        }
    }

    /// Whether the font kerns at all.
    pub fn kerns(&self) -> bool {
        !self.kerning.is_empty()
    }

    /// The width of one line of text at `size` CSS pixels per em: the
    /// advances and the kerning between neighbours, as og-render draws it;
    /// a line with right-to-left or shaped characters as laid out visually.
    pub fn measure(&self, text: &str, size: f32) -> f32 {
        if needs_visual(text) || self.is_mixed() {
            return self
                .visual(text, Direction::Ltr, 0.0)
                .iter()
                .map(|g| g.advance)
                .sum::<f32>()
                * size;
        }
        if self.kerning.is_empty() {
            return text.chars().map(|c| self.advance(c)).sum::<f32>() * size;
        }
        let mut prev: Option<char> = None;
        let mut w = 0.0f32;
        for c in text.chars() {
            if c == '\n' || !self.has(c) {
                prev = None;
                continue;
            }
            if let Some(p) = prev {
                w += self.kern(p, c);
            }
            w += self.advance(c);
            prev = Some(c);
        }
        w * size
    }

    /// [`FontMetrics::measure`] with `tracking` em after each character the
    /// font draws (og-render's `Text::tracking`: CSS `letter-spacing`).
    pub fn measure_tracked(&self, text: &str, size: f32, tracking: f32) -> f32 {
        if tracking != 0.0 && (needs_visual(text) || self.is_mixed()) {
            return self
                .visual(text, Direction::Ltr, tracking)
                .iter()
                .map(|g| g.advance)
                .sum::<f32>()
                * size;
        }
        let w = self.measure(text, size);
        if tracking == 0.0 {
            return w;
        }
        w + tracking * size * text.chars().filter(|&c| c != '\n' && self.has(c)).count() as f32
    }

    /// One run of one direction as drawn, left to right: shaped when it has
    /// a shaped script and the font can, else its characters (reversed and
    /// mirrored when right to left), kerned when left to right. `first` is
    /// the logical index of its first character. A mixed font cuts it into
    /// its font runs, each laid out by its own font (in reverse order when
    /// right to left).
    pub fn run(&self, text: &str, rtl: bool, tracking: f32, first: usize) -> Vec<VisualGlyph> {
        if !self.is_mixed() {
            return self.run_one(text, rtl, tracking, first, 0);
        }
        let mut parts = Vec::new();
        for (range, k) in self.font_runs(text) {
            let m = self.part(k).expect("a font of the run");
            let f = first + text[..range.start].chars().count();
            parts.push(m.run_one(&text[range], rtl, tracking, f, k as u8));
        }
        if rtl {
            parts.reverse();
        }
        parts.concat()
    }

    /// [`FontMetrics::run`] with this font alone, its glyphs marked `font`.
    fn run_one(
        &self,
        text: &str,
        rtl: bool,
        tracking: f32,
        first: usize,
        font: u8,
    ) -> Vec<VisualGlyph> {
        if text.chars().any(og_i18n::needs_shaping)
            && let Some(file) = &self.file
            && let Some(shaped) = file.shape(text, rtl, &|c| self.in_atlas(c))
        {
            return shaped
                .iter()
                .map(|g| VisualGlyph {
                    ch: g.ch,
                    advance: g.advance,
                    x: g.x,
                    y: g.y,
                    index: first + text[..(g.cluster as usize).min(text.len())].chars().count(),
                    rtl,
                    font,
                })
                .collect();
        }
        let n = text.chars().count();
        let mut out: Vec<VisualGlyph> = Vec::with_capacity(n);
        let glyph = |k: usize, c: char| VisualGlyph {
            ch: c,
            advance: if self.own_has(c) {
                self.own_advance(c) + tracking
            } else {
                0.0
            },
            x: 0.0,
            y: 0.0,
            index: first + k,
            rtl,
            font,
        };
        if rtl {
            for (k, c) in text
                .chars()
                .enumerate()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
            {
                out.push(glyph(k, mirror(c)));
            }
        } else {
            let mut prev: Option<char> = None;
            for (k, c) in text.chars().enumerate() {
                let g = glyph(k, c);
                if let (Some(p), true) = (prev, self.own_has(c))
                    && let Some(last) = out.last_mut()
                {
                    last.advance += self.kern(p, c);
                }
                prev = self.own_has(c).then_some(c);
                out.push(g);
            }
        }
        out
    }

    /// One line as drawn: its bidi runs in visual order (UAX #9 at the
    /// paragraph direction of its first strong character, `default` with
    /// none), each laid out by [`FontMetrics::run`].
    pub fn visual(&self, line: &str, default: Direction, tracking: f32) -> Vec<VisualGlyph> {
        let base = og_i18n::paragraph_direction(line, default);
        self.visual_at(line, base, tracking)
    }

    /// [`FontMetrics::visual`] at a given paragraph direction.
    pub fn visual_at(&self, line: &str, base: Direction, tracking: f32) -> Vec<VisualGlyph> {
        if !needs_visual(line) && base == Direction::Ltr {
            return self.run(line, false, tracking, 0);
        }
        let mut out = Vec::new();
        for (range, rtl) in bidi_runs(line, base) {
            let first = line[..range.start].chars().count();
            out.extend(self.run(&line[range], rtl, tracking, first));
        }
        out
    }

    /// The characters and placements a visual line draws with
    /// (og-render's `Text::placements`).
    pub fn drawn(glyphs: &[VisualGlyph]) -> (String, Vec<GlyphPlacement>) {
        (
            glyphs.iter().map(|g| g.ch).collect(),
            glyphs
                .iter()
                .map(|g| GlyphPlacement {
                    advance: g.advance,
                    x: g.x,
                    y: g.y,
                })
                .collect(),
        )
    }
}

/// A line as a text command draws it: as written (og-render kerns it), or
/// for a line with right-to-left or shaped characters its visual glyphs and
/// their placements.
pub(crate) fn drawn_line(
    m: &FontMetrics,
    line: &str,
    default: Direction,
    tracking: f32,
) -> (String, Vec<GlyphPlacement>) {
    if needs_visual(line) {
        FontMetrics::drawn(&m.visual(line, default, tracking))
    } else {
        (line.to_string(), Vec::new())
    }
}

/// One text command's worth of a line: the font, what it draws and where
/// it starts, `dx` em from the line's start (its width `width` em).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Piece {
    pub font: Option<FontHandle>,
    pub dx: f32,
    pub width: f32,
    pub text: String,
    pub placements: Vec<GlyphPlacement>,
}

/// What a line of `font` draws: one piece as [`drawn_line`] gives it, or
/// for a mixed font its visual glyphs cut where the font changes.
pub(crate) fn pieces(font: &UiFont, line: &str, default: Direction, tracking: f32) -> Vec<Piece> {
    if !font.metrics.is_mixed() {
        let (text, placements) = drawn_line(&font.metrics, line, default, tracking);
        return vec![Piece {
            font: font.handle,
            dx: 0.0,
            width: 0.0,
            text,
            placements,
        }];
    }
    split(font, &font.metrics.visual(line, default, tracking))
}

/// Visual glyphs cut into one piece per run of one font, left to right.
pub(crate) fn split(font: &UiFont, glyphs: &[VisualGlyph]) -> Vec<Piece> {
    let mut out = Vec::new();
    let mut x = 0.0f32;
    for run in glyphs.chunk_by(|a, b| a.font == b.font) {
        let (text, placements) = FontMetrics::drawn(run);
        let width = run.iter().map(|g| g.advance).sum::<f32>();
        out.push(Piece {
            font: font.handle_of(run[0].font),
            dx: x,
            width,
            text,
            placements,
        });
        x += width;
    }
    out
}

/// Characters that keep the font of the run before them in a mixed text:
/// spaces, punctuation and symbols, and combining marks and joiners.
fn sticks(c: char) -> bool {
    c.is_whitespace()
        || !c.is_alphanumeric()
        || matches!(c as u32,
            0x0300..=0x036f | 0x0483..=0x0489 | 0x0591..=0x05bd | 0x05bf..=0x05c7
            | 0x0610..=0x061a | 0x064b..=0x065f | 0x0670 | 0x06d6..=0x06ed
            | 0x0900..=0x0903 | 0x093a..=0x094f | 0x0951..=0x0957 | 0x0962..=0x0963
            | 0x1ab0..=0x1aff | 0x1dc0..=0x1dff | 0x200c..=0x200d | 0x20d0..=0x20ff
            | 0xfe00..=0xfe0f)
}

/// Characters no face need draw: joiners and variation selectors.
fn invisible(c: char) -> bool {
    matches!(c as u32, 0x200c..=0x200d | 0x2060 | 0xfe00..=0xfe0f | 0xe0100..=0xe01ef)
}

/// Whether a line needs the bidi algorithm or shaping to be drawn.
pub fn needs_visual(line: &str) -> bool {
    line.chars()
        .any(|c| og_i18n::needs_shaping(c) || og_i18n::is_rtl_char(c))
}

/// A line's level runs in visual order: byte ranges and whether each runs
/// right to left (UAX #9, unicode-bidi, at paragraph direction `base`).
pub fn bidi_runs(line: &str, base: Direction) -> Vec<(std::ops::Range<usize>, bool)> {
    use unicode_bidi::{BidiInfo, Level};
    if line.is_empty() {
        return Vec::new();
    }
    let level = match base {
        Direction::Rtl => Level::rtl(),
        Direction::Ltr => Level::ltr(),
    };
    let info = BidiInfo::new(line, Some(level));
    let mut out = Vec::new();
    for para in &info.paragraphs {
        let (levels, runs) = info.visual_runs(para, para.range.clone());
        for r in runs {
            let rtl = levels[r.start].is_rtl();
            out.push((r, rtl));
        }
    }
    out
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

    /// The font that draws [`VisualGlyph::font`] `k`: this one for 0, else
    /// the `k`-th of a mixed text's other fonts.
    pub fn handle_of(&self, k: u8) -> Option<FontHandle> {
        match k {
            0 => self.handle,
            k => self.metrics.runs.as_ref()?.get(k as usize - 1)?.handle,
        }
    }

    /// This font alone: no runs and no mixed variant (what a mixed font's
    /// parts are).
    pub(crate) fn alone(&self) -> UiFont {
        let mut f = self.clone();
        f.metrics.runs = None;
        f.metrics.mixed = None;
        f
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
pub(crate) fn tokens(paragraph: &str) -> Vec<(&str, bool)> {
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
pub(crate) fn longest_word(text: &str, m: &FontMetrics, size: f32, tracking: f32) -> f32 {
    text.split('\n')
        .flat_map(tokens)
        .filter(|(_, space)| !space)
        .map(|(w, _)| m.measure_tracked(w, size, tracking))
        .fold(0.0, f32::max)
}

/// The widest `\n`-separated line, unwrapped.
pub(crate) fn widest_line(text: &str, m: &FontMetrics, size: f32, tracking: f32) -> f32 {
    text.split('\n')
        .map(|l| m.measure_tracked(l, size, tracking))
        .fold(0.0, f32::max)
}

/// Greedy line breaking at `max_width` (`None`: only at `\n`). Spaces at a
/// break are dropped; a unit wider than the line breaks between characters.
pub(crate) fn break_lines(
    text: &str,
    m: &FontMetrics,
    size: f32,
    tracking: f32,
    max_width: Option<f32>,
) -> Vec<Line> {
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let Some(max) = max_width else {
            lines.push(Line {
                text: paragraph.to_owned(),
                width: m.measure_tracked(paragraph, size, tracking),
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
                spaces_w += m.measure_tracked(unit, size, tracking);
                continue;
            }
            let w = m.measure_tracked(unit, size, tracking);
            // Kerned or shaped text measures differently whole than unit by
            // unit (pairs across the units, a run shaped together): a line
            // that fits whole stays whole.
            let fits_whole = |line: &str| {
                (m.kerns() || m.is_mixed() || needs_visual(line))
                    && m.measure_tracked(&format!("{line}{spaces}{unit}"), size, tracking) <= max
            };
            if !line.is_empty() && width + spaces_w + w > max && !fits_whole(&line) {
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
                    let cw = m.measure_tracked(c.encode_utf8(&mut [0; 4]), size, tracking);
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
    // A kerning font kerns across the units too, and a shaped run is shaped
    // whole: each line's width is its whole text measured (what is drawn),
    // not the sum of its units.
    if m.kerns() || m.is_mixed() || needs_visual(text) {
        for l in &mut lines {
            l.width = m.measure_tracked(&l.text, size, tracking);
        }
    }
    lines
}
