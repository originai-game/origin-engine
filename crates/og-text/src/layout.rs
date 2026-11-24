//! One text's layout, read by both measure and paint: Parley shapes it
//! (HarfRust), orders it (UAX #9), breaks it (UAX #14, `overflow-wrap`)
//! and places its lines by CSS's half-leading with ascent, descent and the
//! leading above the baseline rounded as browsers do (Parley's `quantize`).
//! This module maps a CSS text style onto Parley and adds what Parley 0.12
//! lacks: `text-box-trim` (CSS Inline 3 §5) and `text-overflow: ellipsis`
//! (CSS Overflow 3 §4).

use crate::fonts::{Collection, FaceId};
use og_i18n::Direction;
use parley::TextWrapMode;
use parley::layout::{AlignmentOptions, PositionedLayoutItem};
use parley::style::{FontFamily, FontFeatures, FontStyle, FontWeight, StyleProperty};
use skrifa::instance::LocationRef;
use skrifa::{FontRef, MetadataProvider};
use std::borrow::Cow;

pub use parley::layout::Alignment;
pub use parley::style::FontFeature;
pub use parley::{LineHeight, OverflowWrap, WhiteSpaceCollapse};

/// CSS `hyphens`: author soft hyphens, dictionary breaks, or neither.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Hyphens {
    #[default]
    Manual,
    Auto,
    None,
}

/// OpenType feature `tag` (`b"tnum"`) set to `value` (1 on, 0 off), for
/// [`Style::features`].
pub fn feature(tag: &[u8; 4], value: u16) -> FontFeature {
    FontFeature::new(parley::setting::Tag::new(tag), value)
}

/// CSS `text-box-edge`'s over edge: where a trimmed first line starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Over {
    /// The primary font's ascent.
    Text,
    /// Its cap height.
    Cap,
    /// Its x-height.
    Ex,
    /// Its ideographic-over baseline.
    Ideographic,
}

/// CSS `text-box-edge`'s under edge: where a trimmed last line ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Under {
    /// The primary font's descent.
    Text,
    /// The alphabetic baseline.
    Alphabetic,
    /// The ideographic-under baseline.
    Ideographic,
}

/// CSS `text-box`: the edges trimmed off the first line's top and the last
/// line's bottom; `None` keeps that side's half-leading.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Trim {
    pub start: Option<Over>,
    pub end: Option<Under>,
}

/// A text's computed CSS style, as its node resolved it (the player's
/// `textScale` already in `size`, `line_height` and the spacings).
#[derive(Clone, Debug)]
pub struct Style<'a> {
    /// CSS `font-family`, a list: named families as their font files name
    /// them, and generic ones ([`Collection::set_generic`]).
    pub family: &'a str,
    /// CSS px.
    pub size: f32,
    pub weight: f32,
    pub italic: bool,
    /// CSS px.
    pub letter_spacing: f32,
    /// CSS px.
    pub word_spacing: f32,
    pub line_height: LineHeight,
    /// `font-feature-settings` and the `font-variant-*` features, in order.
    pub features: &'a [FontFeature],
    /// BCP 47 (`lang`): `locl` forms, line breaking, Han forms.
    pub locale: Option<&'a str>,
    /// The paragraph's base direction (CSS `direction`).
    pub direction: Direction,
    pub white_space: WhiteSpaceCollapse,
    /// `text-wrap-mode: wrap`; `false` is `nowrap`.
    pub wrap: bool,
    pub overflow_wrap: OverflowWrap,
    pub hyphens: Hyphens,
    pub align: Alignment,
    /// `text-overflow: ellipsis`.
    pub ellipsis: bool,
    /// `line-clamp`: the most lines shown, the last ending in "…" when
    /// more follow; 0 all.
    pub max_lines: u16,
    pub trim: Trim,
    /// Ranges drawn in a style of their own (a paragraph's styled runs),
    /// sorted and apart; the rest is drawn in this one, ink 0.
    pub spans: &'a [Span<'a>],
    /// Boxes laid out in the line (an inline icon or image), by the byte
    /// they stand before, in order.
    pub boxes: &'a [Inline],
    /// CSS `writing-mode: vertical-rl` with `text-orientation: upright`:
    /// each line a column, the first on the right, its glyphs upright and
    /// in their vertical forms (`vert`). The width it breaks at is the
    /// columns' height.
    pub vertical: bool,
}

/// A range of a paragraph's text in its own font, size, weight, slant and
/// spacing; its glyph runs carry its `ink` ([`GlyphRun::ink`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Span<'a> {
    /// Bytes of the text.
    pub start: usize,
    pub end: usize,
    pub family: &'a str,
    /// CSS px.
    pub size: f32,
    pub weight: f32,
    pub italic: bool,
    /// CSS px.
    pub letter_spacing: f32,
    /// Its line height: a line it is on is at least this tall.
    pub line_height: LineHeight,
    /// What paints it, the caller's; 0 is the paragraph's own.
    pub ink: u16,
}

/// A box laid out in a paragraph's line like a character (CSS
/// `display: inline-block`), centred on the text's middle (`vertical-align:
/// middle`): an icon or an image in a sentence.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Inline {
    /// The byte of the text it stands before.
    pub at: usize,
    /// CSS px.
    pub width: f32,
    pub height: f32,
    /// The caller's, given back with its place ([`Text::boxes`]).
    pub id: u64,
}

impl Default for Style<'_> {
    fn default() -> Self {
        Self {
            family: "",
            size: 16.0,
            weight: 400.0,
            italic: false,
            letter_spacing: 0.0,
            word_spacing: 0.0,
            line_height: LineHeight::NORMAL,
            features: &[],
            locale: None,
            direction: Direction::Ltr,
            white_space: WhiteSpaceCollapse::Preserve,
            wrap: true,
            overflow_wrap: OverflowWrap::Normal,
            hyphens: Hyphens::Manual,
            align: Alignment::Start,
            ellipsis: false,
            max_lines: 0,
            trim: Trim::default(),
            spans: &[],
            boxes: &[],
            vertical: false,
        }
    }
}

/// A run of glyphs of one face, as paint draws it.
pub struct GlyphRun<'a> {
    pub face: FaceId,
    /// The face's index in its file (a collection).
    pub index: u32,
    /// CSS px per em.
    pub size: f32,
    /// The face's variation location: each axis's normalized coordinate,
    /// 2.14 fixed point.
    pub coords: &'a [i16],
    /// Bold the face lacks, drawn by emboldening (fontique's synthesis).
    pub embolden: bool,
    /// Italic the face lacks: a skew, degrees.
    pub skew: Option<f32>,
    /// The baseline, CSS px below the box's top; paint rounds it to a
    /// device pixel.
    pub baseline: f32,
    /// Glyph ids and pen positions, CSS px from the box's top left (y the
    /// glyph's own baseline).
    pub glyphs: &'a [parley::layout::Glyph],
    /// Its span's ink ([`Span::ink`]); 0 the paragraph's.
    pub ink: u16,
}

/// One text: shaped once per text or style, broken at each width asked.
#[derive(Default)]
pub struct Text {
    layout: parley::Layout<u16>,
    /// A shaped line ending, shared by all taken discretionary breaks.
    hyphen: Option<parley::Layout<u16>>,
    /// The width the lines were last broken at.
    width: Option<f32>,
    /// What trim takes off the top and the bottom, CSS px.
    trimmed: [f32; 2],
    /// The ellipsized text, when the layout holds it and not the text
    /// it was shaped from.
    cut: Option<String>,
    /// Intrinsic widths when min-content needs separate ligature shaping,
    /// or while the layout holds an ellipsized version of the source.
    content: Option<(f32, f32)>,
    /// A narrow line had to reshape across a ligature. Restore natural
    /// shaping before reflow, so an untaken break never loses its ligature.
    resegmented: bool,
    /// Laid out as columns ([`Style::vertical`]): every place it gives is
    /// its line layout's turned a quarter clockwise.
    vertical: bool,
    /// Scratch for paint's glyphs and a run's location.
    glyphs: Vec<parley::layout::Glyph>,
    coords: Vec<i16>,
}

impl Text {
    pub fn new() -> Self {
        Self::default()
    }

    /// Shapes `text` in `style`. Its lines are broken at no width until
    /// [`Text::break_at`].
    pub fn shape(&mut self, fonts: &mut Collection, text: &str, style: &Style) {
        (self.hyphen, self.content) = build(fonts, &mut self.layout, text, style);
        self.vertical = style.vertical;
        self.cut = None;
        self.resegmented = false;
        self.width = None;
        self.layout.break_all_lines(None);
        self.trimmed = trim(fonts, style, &self.layout);
    }

    /// Breaks the lines at `width` (none: only at hard breaks), aligned in
    /// it. With `style.ellipsis`, a line wider than `width` ends at the last
    /// whole cluster that fits beside "…"; with `style.max_lines`, the last
    /// line shown does when more follow it, and those are not laid out.
    /// `text` and `style` must be the ones [`Text::shape`] took.
    pub fn break_at(
        &mut self,
        fonts: &mut Collection,
        text: &str,
        style: &Style,
        width: Option<f32>,
    ) {
        if self.cut.take().is_some() || self.resegmented {
            (self.hyphen, self.content) = build(fonts, &mut self.layout, text, style);
            self.resegmented = false;
        }
        self.width = width;
        self.layout.break_all_lines(width);
        if let Some(width) = width {
            let boundaries = ligature_breaks(&self.layout, Some(width));
            if !boundaries.is_empty() {
                let breaks = self.layout.discretionary_breaks().to_vec();
                shape_layout(fonts, &mut self.layout, text, style, &boundaries);
                self.layout.set_discretionary_breaks(&breaks);
                self.layout.break_all_lines(Some(width));
                self.resegmented = true;
            }
        }
        if let Some((cut, pieces)) = ellipsized(fonts, &self.layout, text, style, width) {
            self.content = Some(self.content_widths());
            let (spans, boxes) = moved(style, &pieces);
            let style = Style {
                spans: &spans,
                boxes: &boxes,
                ..style.clone()
            };
            self.hyphen = build(fonts, &mut self.layout, &cut, &style).0;
            self.cut = Some(cut);
            self.layout.break_all_lines(width);
        }
        self.layout.align(style.align, AlignmentOptions::default());
        self.trimmed = trim(fonts, style, &self.layout);
    }

    /// The text the layout holds, which [`Text::lines`], [`Text::caret`]
    /// and [`Text::missing`] index: `text` (what it was shaped from), or
    /// its ellipsized form when a line was cut.
    pub fn laid<'a>(&'a self, text: &'a str) -> &'a str {
        self.cut.as_deref().unwrap_or(text)
    }

    /// Its widest line, CSS px; in columns, their line boxes' sum.
    pub fn width(&self) -> f32 {
        if self.vertical {
            self.across()
        } else {
            self.layout.width()
        }
    }

    /// Its height: its line boxes', less what `text-box-trim` takes; in
    /// columns, the longest's.
    pub fn height(&self) -> f32 {
        if self.vertical {
            self.layout.width()
        } else {
            self.across()
        }
    }

    /// Its line boxes' sum, less what `text-box-trim` takes.
    fn across(&self) -> f32 {
        (self.layout.height() - self.trimmed[0] - self.trimmed[1]).max(0.0)
    }

    /// A rectangle of the line layout, `[x, y, w, h]` with y below its
    /// trimmed top, as the box places it: itself, or turned into columns
    /// (a line's top edge at the box's right).
    fn placed(&self, [x, y, w, h]: [f32; 4]) -> [f32; 4] {
        if self.vertical {
            [self.across() - y - h, x, h, w]
        } else {
            [x, y, w, h]
        }
    }

    /// A point of the box as the line layout sees it, y from its untrimmed
    /// top.
    fn unplaced(&self, x: f32, y: f32) -> (f32, f32) {
        let (x, y) = if self.vertical {
            (y, self.across() - x)
        } else {
            (x, y)
        };
        (x, y + self.trimmed[0])
    }

    /// CSS's min-content and max-content widths (Parley's content widths:
    /// the longest unbreakable segment under `overflow-wrap: normal` and
    /// `break-word`, a grapheme under `anywhere`, the whole line under
    /// `nowrap`).
    pub fn content_widths(&self) -> (f32, f32) {
        self.content.unwrap_or_else(|| {
            let w = self.layout.calculate_content_widths();
            (w.min, w.max)
        })
    }

    /// The first line's baseline, CSS px below the box's top.
    pub fn first_baseline(&self) -> Option<f32> {
        self.layout.first_baseline().map(|b| b - self.trimmed[0])
    }

    /// The layout, for Parley's cursor, selection and hit testing.
    pub fn layout(&self) -> &parley::Layout<u16> {
        &self.layout
    }

    /// Whether the laid-out line ending at source byte `at` took a discretionary break.
    pub fn is_discretionary_break(&self, at: usize) -> bool {
        self.layout
            .lines()
            .any(|l| l.text_range().end == at && l.discretionary_offset().is_some())
    }

    /// Calls `f` with each line: where its text starts (CSS px from the
    /// box's left), its baseline (below the box's top), its text's width,
    /// and the bytes of the laid-out text it holds.
    pub fn lines(&self, mut f: impl FnMut([f32; 3], std::ops::Range<usize>)) {
        let top = self.trimmed[0];
        for line in self.layout.lines() {
            let m = line.metrics();
            let width = m.advance - m.hanging_advance;
            f([m.offset, m.baseline - top, width], line.text_range());
        }
    }

    /// Where the caret before byte `at` of the laid-out text stands: its
    /// x from the box's left, its top below the box's top and its height,
    /// CSS px (Parley's cursor: after the cluster before it, on the line
    /// that holds it).
    pub fn caret(&self, at: usize) -> [f32; 3] {
        let cursor =
            parley::Cursor::from_byte_index(&self.layout, at, parley::Affinity::Downstream);
        let b = cursor.geometry(&self.layout, 0.0);
        let [x, y, _, h] = self.placed([
            b.x0 as f32,
            b.y0 as f32 - self.trimmed[0],
            0.0,
            (b.y1 - b.y0) as f32,
        ]);
        [x, y, h]
    }

    /// The caret position nearest the point (x, y), CSS px from the box's
    /// top left, as a byte offset of the laid-out text: where a press puts
    /// the caret (Parley's cursor: the nearer edge of the cluster under it,
    /// on the line under it or the nearest one).
    pub fn hit(&self, x: f32, y: f32) -> usize {
        let (x, y) = self.unplaced(x, y);
        parley::Cursor::from_point(&self.layout, x, y).index()
    }

    /// The word at the point (x, y), as a byte range of the laid-out text
    /// (UAX #29 words, with dictionaries for scripts written without
    /// spaces): what a double press selects.
    pub fn word_at(&self, x: f32, y: f32) -> std::ops::Range<usize> {
        let (x, y) = self.unplaced(x, y);
        parley::Selection::word_from_point(&self.layout, x, y).text_range()
    }

    /// Where the caret at byte `at` goes `lines` lines down (up when
    /// negative), keeping its x: to the text's start or end past its
    /// first or last line.
    pub fn line_step(&self, at: usize, lines: isize) -> usize {
        parley::Selection::from_byte_index(&self.layout, at, parley::Affinity::Downstream)
            .move_lines(&self.layout, lines, false)
            .focus()
            .index()
    }

    /// Selection highlight rectangles in layout CSS pixels. Parley owns
    /// bidi reordering and grapheme geometry, as it does for the caret.
    pub fn selection(&self, range: std::ops::Range<usize>, mut f: impl FnMut([f32; 4])) {
        let cursor =
            |at| parley::Cursor::from_byte_index(&self.layout, at, parley::Affinity::Downstream);
        parley::Selection::new(cursor(range.start), cursor(range.end)).geometry_with(
            &self.layout,
            |b, _| {
                f(self.placed([
                    b.x0 as f32,
                    b.y0 as f32 - self.trimmed[0],
                    (b.x1 - b.x0) as f32,
                    (b.y1 - b.y0) as f32,
                ]))
            },
        );
    }

    /// Calls `f` with each character of `text` (the laid-out text) no
    /// face draws: one whose cluster took the missing glyph, or all of
    /// them when no face was found at all. White space, controls and the
    /// characters that draw nothing (joiners, variation selectors) are
    /// never missing.
    pub fn missing(&self, text: &str, mut f: impl FnMut(char)) {
        let mut lacks = |s: &str| {
            s.chars()
                .filter(|c| !c.is_whitespace() && !c.is_control() && !crate::text::invisible(*c))
                .for_each(&mut f);
        };
        let mut shaped = false;
        for line in self.layout.lines() {
            for run in line.runs() {
                shaped = true;
                for c in run.clusters() {
                    if c.glyphs().any(|g| g.id == 0)
                        && let Some(s) = text.get(c.text_range())
                    {
                        lacks(s);
                    }
                }
            }
        }
        if !shaped {
            lacks(text);
        }
    }

    /// Calls `f` with each inline box ([`Style::boxes`]) laid out: its id
    /// and its rectangle, CSS px from the box's top left. A box a cut
    /// line dropped is not called.
    pub fn boxes(&self, mut f: impl FnMut(u64, [f32; 4])) {
        let top = self.trimmed[0];
        for line in self.layout.lines() {
            for item in line.items() {
                if let PositionedLayoutItem::InlineBox(b) = item {
                    // A box stands upright in a column, centred where
                    // its place in the line turns to.
                    let [x, y, w, h] = self.placed([b.x, b.y - top, b.width, b.height]);
                    let (cx, cy) = (x + 0.5 * w, y + 0.5 * h);
                    let (w, h) = (b.width, b.height);
                    f(b.id, [cx - 0.5 * w, cy - 0.5 * h, w, h]);
                }
            }
        }
    }

    /// Calls `f` with the stretches of each line drawn in one ink, in
    /// visual order: the ink ([`Span::ink`]) and its start x, baseline and
    /// width, CSS px. A span's decoration lines lie along them.
    pub fn inks(&self, mut f: impl FnMut(u16, [f32; 3])) {
        let top = self.trimmed[0];
        for line in self.layout.lines() {
            let mut open: Option<(u16, [f32; 3])> = None;
            for item in line.items() {
                let PositionedLayoutItem::GlyphRun(run) = item else {
                    continue;
                };
                let ink = run.style().brush;
                let (x, w) = (run.offset(), run.advance());
                let base = run.baseline() - top;
                match &mut open {
                    Some((i, a)) if *i == ink && (a[0] + a[2] - x).abs() < 0.01 && a[1] == base => {
                        a[2] += w
                    }
                    _ => {
                        if let Some((i, a)) = open.take() {
                            f(i, a);
                        }
                        open = Some((ink, [x, base, w]));
                    }
                }
            }
            if let Some((i, a)) = open {
                f(i, a);
            }
        }
    }

    /// Calls `f` with each glyph run, line by line in visual order.
    pub fn runs(&mut self, fonts: &Collection, mut f: impl FnMut(GlyphRun)) {
        let top = self.trimmed[0];
        let (vertical, across) = (self.vertical, self.across());
        // `col`: in columns, the middle of the line's column from the box's
        // left.
        let mut emit =
            |run: parley::layout::GlyphRun<'_, u16>, x: f32, y: f32, col: f32, ink: Option<u16>| {
                let r = run.run();
                let Some(face) = fonts.face_of(r.font()) else {
                    return;
                };
                // An upright glyph in a column: its advance centred across
                // the column, and its em box (ascent over descent) centred
                // in the stretch of the column its advance takes.
                let m = r.font_metrics();
                let rise = 0.5 * (m.ascent - m.descent);
                self.glyphs.clear();
                self.glyphs.extend(run.positioned_glyphs().map(|mut g| {
                    g.x += x;
                    if vertical {
                        let half = 0.5 * g.advance;
                        (g.x, g.y) = (col - half, g.x + half + rise);
                    } else {
                        g.y += y - top;
                    }
                    g
                }));
                self.coords.clear();
                self.coords
                    .extend(r.normalized_coords().iter().map(|c| c.to_bits()));
                let synthesis = r.synthesis();
                f(GlyphRun {
                    face,
                    index: r.font().index,
                    size: r.font_size(),
                    coords: &self.coords,
                    embolden: synthesis.embolden(),
                    skew: synthesis.skew(),
                    baseline: if vertical {
                        col
                    } else {
                        run.baseline() + y - top
                    },
                    glyphs: &self.glyphs,
                    ink: ink.unwrap_or(run.style().brush),
                });
            };
        for line in self.layout.lines() {
            let m = line.metrics();
            let col = across - (0.5 * (m.block_min_coord + m.block_max_coord) - top);
            // A hyphen a break adds is drawn in the ink of the text it ends.
            let mut last = 0;
            for item in line.items() {
                if let PositionedLayoutItem::GlyphRun(run) = item {
                    last = run.style().brush;
                    emit(run, 0., 0., col, None);
                }
            }
            if let (Some(x), Some(hyphen)) = (line.discretionary_offset(), &self.hyphen) {
                for ending in hyphen.lines() {
                    let y = line.metrics().baseline - ending.metrics().baseline;
                    for item in ending.items() {
                        if let PositionedLayoutItem::GlyphRun(run) = item {
                            emit(run, x, y, col, Some(last));
                        }
                    }
                }
            }
        }
    }
}

/// Shapes `text` in `style` into `layout`.
fn build(
    fonts: &mut Collection,
    layout: &mut parley::Layout<u16>,
    text: &str,
    style: &Style,
) -> (Option<parley::Layout<u16>>, Option<(f32, f32)>) {
    shape_layout(fonts, layout, text, style, &[]);
    if style.hyphens == Hyphens::None || !style.wrap {
        return (None, None);
    }
    let marked = match (style.hyphens, style.locale) {
        (Hyphens::Auto, Some(lang)) => {
            Cow::Owned(og_i18n::hyphenation::hyphenate(text, lang, 0, None, None))
        }
        _ => Cow::Borrowed(text),
    };
    // Dictionary markers never enter the shaped source. Cursor, selection
    // and probe ranges remain byte offsets into the author's text.
    let mut original = text.chars().peekable();
    let mut byte = 0;
    let mut breaks = Vec::new();
    for c in marked.chars() {
        if c == '\u{ad}' && original.peek() != Some(&c) {
            breaks.push((byte, 0.));
        } else {
            original.next();
            byte += c.len_utf8();
            if c == '\u{ad}' && byte < text.len() {
                breaks.push((byte, 0.));
            }
        }
    }
    if breaks.is_empty() {
        return (None, None);
    }
    let mut hyphen = parley::Layout::new();
    shape_layout(fonts, &mut hyphen, "-", &plain(style), &[]);
    hyphen.break_all_lines(None);
    for point in &mut breaks {
        point.1 = hyphen.width();
    }
    layout.set_discretionary_breaks(&breaks);
    layout.break_all_lines(None);
    let boundaries = ligature_breaks(layout, None);
    let intrinsic = if boundaries.is_empty() {
        None
    } else {
        // Min-content takes every opportunity; its shape must be allowed
        // to separate the ligatures an unbroken max-content line keeps.
        let mut minimum = parley::Layout::new();
        shape_layout(fonts, &mut minimum, text, style, &boundaries);
        minimum.set_discretionary_breaks(&breaks);
        Some((
            minimum.calculate_content_widths().min,
            layout.calculate_content_widths().max,
        ))
    };
    (Some(hyphen), intrinsic)
}

/// Discretionary opportunities that shaping joined into a ligature. For
/// wrapping, only overflowing lines need reshaping; a line that fits keeps
/// its original glyphs. With no width, find them all for min-content.
fn ligature_breaks(layout: &parley::Layout<u16>, width: Option<f32>) -> Vec<usize> {
    let mut boundaries = Vec::new();
    if layout.discretionary_breaks().is_empty() {
        return boundaries;
    }
    for line in layout.lines() {
        if width.is_some_and(|w| line.metrics().advance - line.metrics().hanging_advance <= w) {
            continue;
        }
        for run in line.runs() {
            for cluster in run.clusters() {
                let at = cluster.text_range().start;
                if cluster.is_ligature_continuation()
                    && layout
                        .discretionary_breaks()
                        .binary_search_by_key(&at, |p| p.0)
                        .is_ok()
                {
                    boundaries.push(at);
                }
            }
        }
    }
    boundaries.sort_unstable();
    boundaries
}

fn shape_layout(
    fonts: &mut Collection,
    layout: &mut parley::Layout<u16>,
    text: &str,
    style: &Style,
    boundaries: &[usize],
) {
    let (context, lcx) = fonts.contexts();
    let no_hyphens = style.hyphens == Hyphens::None;
    let overrides = move |cx: parley_engine::break_overrides::LineBreakContext| {
        if no_hyphens && cx.before == '\u{ad}' {
            Some(false)
        } else {
            parley::CHROMIUM_LINE_BREAK_OVERRIDE(cx)
        }
    };
    let mut b = lcx.ranged_builder(context, text, 1.0, true);
    b.set_line_break_override(Some(&overrides));
    b.set_shaping_boundaries(boundaries);
    b.set_base_direction(match style.direction {
        Direction::Ltr => parley::BaseDirection::Ltr,
        Direction::Rtl => parley::BaseDirection::Rtl,
    });
    b.push_default(StyleProperty::FontFamily(FontFamily::Source(
        Cow::Borrowed(style.family),
    )));
    b.push_default(StyleProperty::FontSize(style.size));
    b.push_default(StyleProperty::FontWeight(FontWeight::new(style.weight)));
    if style.italic {
        b.push_default(StyleProperty::FontStyle(FontStyle::Italic));
    }
    b.push_default(StyleProperty::LetterSpacing(style.letter_spacing));
    b.push_default(StyleProperty::WordSpacing(style.word_spacing));
    b.push_default(StyleProperty::LineHeight(style.line_height));
    // CSS `font-optical-sizing: auto`, as browsers apply it: a face with an
    // `opsz` axis is drawn at its font size in CSS px. One without the
    // axis ignores the setting.
    let optical = [parley::style::FontVariation::new(
        parley::setting::Tag::new(b"opsz"),
        style.size,
    )];
    b.push_default(StyleProperty::FontVariations(
        parley::style::FontVariations::List(Cow::Borrowed(&optical)),
    ));
    // In columns, the glyphs' vertical forms (`vert`, `vrt2`): CJK
    // punctuation set in its corner, brackets and dashes turned.
    let features: Cow<[FontFeature]> = if style.vertical {
        let mut f = style.features.to_vec();
        f.extend([feature(b"vert", 1), feature(b"vrt2", 1)]);
        Cow::Owned(f)
    } else {
        Cow::Borrowed(style.features)
    };
    if !features.is_empty() {
        b.push_default(StyleProperty::FontFeatures(FontFeatures::List(features)));
    }
    if let Some(l) = style.locale.and_then(|l| l.parse().ok()) {
        b.push_default(StyleProperty::Locale(Some(l)));
    }
    b.push_default(StyleProperty::WhiteSpaceCollapse(style.white_space));
    b.push_default(StyleProperty::TextWrapMode(if style.wrap {
        TextWrapMode::Wrap
    } else {
        TextWrapMode::NoWrap
    }));
    b.push_default(StyleProperty::OverflowWrap(style.overflow_wrap));
    // Each span over the paragraph's style; its `opsz` follows its size.
    let mut sized = Vec::with_capacity(style.spans.len());
    for s in style.spans {
        sized.push([parley::style::FontVariation::new(
            parley::setting::Tag::new(b"opsz"),
            s.size,
        )]);
    }
    for (s, optical) in style.spans.iter().zip(&sized) {
        let range = s.start.min(text.len())..s.end.min(text.len());
        if range.is_empty() {
            continue;
        }
        let r = || range.clone();
        b.push(
            StyleProperty::FontFamily(FontFamily::Source(Cow::Borrowed(s.family))),
            r(),
        );
        b.push(StyleProperty::FontSize(s.size), r());
        b.push(StyleProperty::FontWeight(FontWeight::new(s.weight)), r());
        b.push(
            StyleProperty::FontStyle(if s.italic {
                FontStyle::Italic
            } else {
                FontStyle::Normal
            }),
            r(),
        );
        b.push(StyleProperty::LetterSpacing(s.letter_spacing), r());
        b.push(StyleProperty::LineHeight(s.line_height), r());
        b.push(
            StyleProperty::FontVariations(parley::style::FontVariations::List(Cow::Borrowed(
                optical,
            ))),
            r(),
        );
        b.push(StyleProperty::Brush(s.ink), r());
    }
    for i in style.boxes {
        b.push_inline_box(parley::InlineBox {
            id: i.id,
            kind: parley::InlineBoxKind::InFlow,
            index: i.at.min(text.len()),
            width: i.width,
            height: i.height,
            baseline: None,
            vertical_align: parley::VerticalAlign::MIDDLE,
        });
    }
    b.build_into(layout, text);
}

/// `style` without its spans and boxes: what a hyphen or an ellipsis
/// added to a line is shaped in.
fn plain<'a>(style: &Style<'a>) -> Style<'a> {
    Style {
        spans: &[],
        boxes: &[],
        ..style.clone()
    }
}

/// `style`'s spans and boxes moved onto a cut text made of `pieces`:
/// (where a piece starts in the cut text, where in the source, its
/// length, and where the cut text's "…" after it ends, if one follows).
fn moved<'a>(style: &Style<'a>, pieces: &[[usize; 4]]) -> (Vec<Span<'a>>, Vec<Inline>) {
    let mut spans = Vec::new();
    for s in style.spans {
        for &[out, src, len, mark] in pieces {
            let (lo, hi) = (s.start.max(src), s.end.min(src + len));
            if lo < hi {
                // A span the cut ends in draws its "…" too.
                let end = if hi == src + len {
                    mark
                } else {
                    out + hi - src
                };
                spans.push(Span {
                    start: out + lo - src,
                    end,
                    ..*s
                });
            }
        }
    }
    let boxes = style
        .boxes
        .iter()
        .filter_map(|i| {
            let &[out, src, ..] = pieces
                .iter()
                .find(|&&[_, src, len, _]| i.at >= src && i.at <= src + len)?;
            Some(Inline {
                at: out + i.at - src,
                ..*i
            })
        })
        .collect();
    (spans, boxes)
}

/// `text` as `style` cuts it: under `text-overflow: ellipsis`, each line
/// of `layout` wider than `width` cut at the last cluster boundary that
/// leaves room for "…", and "…" added; under `line-clamp`, the lines
/// after its last dropped and that line cut the same way. `None` when
/// nothing is cut.
fn ellipsized(
    fonts: &mut Collection,
    layout: &parley::Layout<u16>,
    text: &str,
    style: &Style,
    width: Option<f32>,
) -> Option<(String, Vec<[usize; 4]>)> {
    let shown = match style.max_lines {
        0 => usize::MAX,
        n => usize::from(n),
    };
    let clamped = layout.len() > shown;
    let over = |advance: f32| match width {
        Some(w) => style.ellipsis && advance > w + 0.01,
        None => false,
    };
    if !clamped && layout.lines().all(|l| !over(l.metrics().advance)) {
        return None;
    }
    // "…" as this text would draw it.
    let mut mark = parley::Layout::new();
    build(fonts, &mut mark, "\u{2026}", &plain(style));
    mark.break_all_lines(None);
    let room = width.map(|w| (w - mark.width()).max(0.0));
    let mut out = String::with_capacity(text.len() + 3);
    // Where each piece of the source went, for the spans and boxes.
    let mut pieces: Vec<[usize; 4]> = Vec::new();
    let mut clusters: Vec<(std::ops::Range<usize>, f32)> = Vec::new();
    for (k, line) in layout.lines().take(shown).enumerate() {
        let range = line.text_range();
        let last = clamped && k + 1 == shown;
        if !last && !over(line.metrics().advance) {
            pieces.push([out.len(), range.start, range.len(), out.len() + range.len()]);
            out.push_str(&text[range]);
            continue;
        }
        clusters.clear();
        for run in line.runs() {
            clusters.extend(run.clusters().map(|c| (c.text_range(), c.advance())));
        }
        clusters.sort_by_key(|(r, _)| r.start);
        let (mut end, mut pen) = (range.start, 0.0);
        for (r, advance) in &clusters {
            pen += advance;
            if room.is_some_and(|room| pen > room + 0.01) {
                break;
            }
            end = r.end;
        }
        let kept = text[range.start..end].trim_end();
        let at = out.len();
        out.push_str(kept);
        out.push('\u{2026}');
        pieces.push([at, range.start, kept.len(), out.len()]);
        // A hard break that ended the line stays, unless the text ends here.
        if !last && text[..range.end].ends_with('\n') {
            out.push('\n');
        }
    }
    Some((out, pieces))
}

/// Where a text's decoration lines lie, from its primary font (CSS Text
/// Decoration 4 §2): CSS px, offsets above the baseline.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Decorations {
    /// The underline's centre (negative: below the baseline) and its
    /// thickness.
    pub underline: [f32; 2],
    /// The line-through's.
    pub through: [f32; 2],
    /// The primary font's ascent, where an overline lies.
    pub ascent: f32,
}

/// The decoration lines of a text in `style`: its primary font's `post`
/// underline and OS/2 strikeout, or a fifteenth of the size a ninth below
/// the baseline and a third of the size above it for a font without them.
pub fn decorations(fonts: &mut Collection, style: &Style) -> Decorations {
    let size = style.size;
    let fallback = Decorations {
        underline: [-size / 9.0, size / 15.0],
        through: [size / 3.0, size / 15.0],
        ascent: size * 0.8,
    };
    let Some((face, index)) = fonts.primary(style.family, style.weight, style.italic) else {
        return fallback;
    };
    let Ok(font) = FontRef::from_index(fonts.file(face), index) else {
        return fallback;
    };
    let location = font.axes().location([("wght", style.weight)]);
    let m = font.metrics(
        skrifa::instance::Size::new(size),
        LocationRef::from(&location),
    );
    Decorations {
        // skrifa gives a line's top; CSS centres its thickness on it.
        underline: m.underline.map_or(fallback.underline, |d| {
            [d.offset - d.thickness / 2.0, d.thickness]
        }),
        through: m.strikeout.map_or(fallback.through, |d| {
            [d.offset - d.thickness / 2.0, d.thickness]
        }),
        ascent: m.ascent,
    }
}

/// What `style.trim` takes off the top and bottom of `layout`, CSS px: the
/// first line's box above the primary font's over edge, the last line's
/// below its under edge (CSS Inline 3 §5.1; the primary font is
/// [`Collection::primary`]).
fn trim(fonts: &mut Collection, style: &Style, layout: &parley::Layout<u16>) -> [f32; 2] {
    if style.trim == Trim::default() {
        return [0.0; 2];
    }
    let (Some(first), Some(last)) = (layout.lines().next(), layout.lines().last()) else {
        return [0.0; 2];
    };
    let Some((face, index)) = fonts.primary(style.family, style.weight, style.italic) else {
        return [0.0; 2];
    };
    let Ok(font) = FontRef::from_index(fonts.file(face), index) else {
        return [0.0; 2];
    };
    let location = font.axes().location([("wght", style.weight)]);
    let location = LocationRef::from(&location);
    let m = font.metrics(skrifa::instance::Size::new(style.size), location);
    let b = crate::baselines::face_baselines(&font, location).scaled(style.size);
    let ideographic_over = 2.0 * b.central - b.ideographic;
    let over = style.trim.start.map(|o| match o {
        Over::Text => m.ascent.round(),
        Over::Cap => m.cap_height.unwrap_or(m.ascent * 0.7),
        Over::Ex => m.x_height.unwrap_or(m.ascent * 0.56),
        Over::Ideographic => ideographic_over,
    });
    let under = style.trim.end.map(|u| match u {
        Under::Text => -m.descent.round(),
        Under::Alphabetic => 0.0,
        Under::Ideographic => -b.ideographic,
    });
    let top = over.map_or(0.0, |o| first.metrics().baseline - o);
    let bottom = under.map_or(0.0, |u| layout.height() - (last.metrics().baseline + u));
    [top.max(0.0), bottom.max(0.0)]
}
