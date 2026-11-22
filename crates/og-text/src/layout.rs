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
}

/// One text: shaped once per text or style, broken at each width asked.
#[derive(Default)]
pub struct Text {
    layout: parley::Layout<()>,
    /// A shaped line ending, shared by all taken discretionary breaks.
    hyphen: Option<parley::Layout<()>>,
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
        if let Some(cut) = ellipsized(fonts, &self.layout, text, style, width) {
            self.content = Some(self.content_widths());
            self.hyphen = build(fonts, &mut self.layout, &cut, style).0;
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

    /// Its widest line, CSS px.
    pub fn width(&self) -> f32 {
        self.layout.width()
    }

    /// Its height: its line boxes', less what `text-box-trim` takes.
    pub fn height(&self) -> f32 {
        (self.layout.height() - self.trimmed[0] - self.trimmed[1]).max(0.0)
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
    pub fn layout(&self) -> &parley::Layout<()> {
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
        [
            b.x0 as f32,
            b.y0 as f32 - self.trimmed[0],
            (b.y1 - b.y0) as f32,
        ]
    }

    /// Selection highlight rectangles in layout CSS pixels. Parley owns
    /// bidi reordering and grapheme geometry, as it does for the caret.
    pub fn selection(&self, range: std::ops::Range<usize>, mut f: impl FnMut([f32; 4])) {
        let cursor =
            |at| parley::Cursor::from_byte_index(&self.layout, at, parley::Affinity::Downstream);
        parley::Selection::new(cursor(range.start), cursor(range.end)).geometry_with(
            &self.layout,
            |b, _| {
                f([
                    b.x0 as f32,
                    b.y0 as f32 - self.trimmed[0],
                    (b.x1 - b.x0) as f32,
                    (b.y1 - b.y0) as f32,
                ])
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

    /// Calls `f` with each glyph run, line by line in visual order.
    pub fn runs(&mut self, fonts: &Collection, mut f: impl FnMut(GlyphRun)) {
        let top = self.trimmed[0];
        let mut emit = |run: parley::layout::GlyphRun<'_, ()>, x: f32, y: f32| {
            let r = run.run();
            let Some(face) = fonts.face_of(r.font()) else {
                return;
            };
            self.glyphs.clear();
            self.glyphs.extend(run.positioned_glyphs().map(|mut g| {
                g.x += x;
                g.y += y - top;
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
                baseline: run.baseline() + y - top,
                glyphs: &self.glyphs,
            });
        };
        for line in self.layout.lines() {
            for item in line.items() {
                if let PositionedLayoutItem::GlyphRun(run) = item {
                    emit(run, 0., 0.);
                }
            }
            if let (Some(x), Some(hyphen)) = (line.discretionary_offset(), &self.hyphen) {
                for ending in hyphen.lines() {
                    let y = line.metrics().baseline - ending.metrics().baseline;
                    for item in ending.items() {
                        if let PositionedLayoutItem::GlyphRun(run) = item {
                            emit(run, x, y);
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
    layout: &mut parley::Layout<()>,
    text: &str,
    style: &Style,
) -> (Option<parley::Layout<()>>, Option<(f32, f32)>) {
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
    shape_layout(fonts, &mut hyphen, "-", style, &[]);
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
fn ligature_breaks(layout: &parley::Layout<()>, width: Option<f32>) -> Vec<usize> {
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
    layout: &mut parley::Layout<()>,
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
    if !style.features.is_empty() {
        b.push_default(StyleProperty::FontFeatures(FontFeatures::List(
            Cow::Borrowed(style.features),
        )));
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
    b.build_into(layout, text);
}

/// `text` as `style` cuts it: under `text-overflow: ellipsis`, each line
/// of `layout` wider than `width` cut at the last cluster boundary that
/// leaves room for "…", and "…" added; under `line-clamp`, the lines
/// after its last dropped and that line cut the same way. `None` when
/// nothing is cut.
fn ellipsized(
    fonts: &mut Collection,
    layout: &parley::Layout<()>,
    text: &str,
    style: &Style,
    width: Option<f32>,
) -> Option<String> {
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
    build(fonts, &mut mark, "\u{2026}", style);
    mark.break_all_lines(None);
    let room = width.map(|w| (w - mark.width()).max(0.0));
    let mut out = String::with_capacity(text.len() + 3);
    let mut clusters: Vec<(std::ops::Range<usize>, f32)> = Vec::new();
    for (k, line) in layout.lines().take(shown).enumerate() {
        let range = line.text_range();
        let last = clamped && k + 1 == shown;
        if !last && !over(line.metrics().advance) {
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
        out.push_str(text[range.start..end].trim_end());
        out.push('\u{2026}');
        // A hard break that ended the line stays, unless the text ends here.
        if !last && text[..range.end].ends_with('\n') {
            out.push('\n');
        }
    }
    Some(out)
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
fn trim(fonts: &mut Collection, style: &Style, layout: &parley::Layout<()>) -> [f32; 2] {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn file(path: &str) -> Arc<Vec<u8>> {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../templates/");
        Arc::new(std::fs::read(format!("{root}{path}")).unwrap())
    }

    /// Scifi's text: Barlow Semi Condensed, Noto Sans SC for ideographs.
    const FAMILY: &str = "\"Barlow Semi Condensed\", \"Noto Sans SC\"";

    fn fonts() -> Collection {
        let mut fonts = Collection::new();
        for src in [
            "styles/shared/BarlowSemiCondensed-SemiBold.ttf",
            "styles/cjk/NotoSansSC[wght].ttf",
        ] {
            fonts.add(file(src)).unwrap();
        }
        fonts
    }

    fn laid(fonts: &mut Collection, text: &str, style: &Style, width: Option<f32>) -> Text {
        let mut t = Text::new();
        t.shape(fonts, text, style);
        t.break_at(fonts, text, style, width);
        t
    }

    /// The text a layout draws.
    fn drawn(t: &Text, text_len: usize) -> usize {
        t.layout()
            .lines()
            .map(|l| l.text_range().len())
            .sum::<usize>()
            .min(text_len)
    }

    #[test]
    fn discretionary_hyphens_take_space_and_ink_only_at_a_taken_break() {
        let mut fonts = fonts();
        let style = Style {
            family: FAMILY,
            ..Default::default()
        };
        let mut dash = laid(&mut fonts, "-", &style, None);
        let dash_width = dash.width();
        let mut dash_id = 0;
        dash.runs(&fonts, |r| dash_id = r.glyphs[0].id);
        let prefix = laid(&mut fonts, "hy", &style, None).width();
        let text = "hy\u{ad}phenation";
        let mut t = laid(&mut fonts, text, &style, Some(prefix + dash_width + 0.01));
        let first = t.layout().lines().next().unwrap();
        assert_eq!(
            first.text_range(),
            0..4,
            "source offsets include only the author's SHY"
        );
        assert!((first.metrics().advance - prefix - dash_width).abs() < 0.01);
        let mut ink = Vec::new();
        t.runs(&fonts, |r| {
            ink.extend(
                r.glyphs
                    .iter()
                    .filter(|g| g.id == dash_id)
                    .map(|g| (g.x, g.y)),
            )
        });
        assert_eq!(ink.len(), 1);
        assert!((ink[0].0 - prefix).abs() < 0.01);
        assert!((ink[0].1 - t.first_baseline().unwrap()).abs() < 0.01);

        // The letters fit but the hyphen does not: do not take this break.
        t.break_at(&mut fonts, text, &style, Some(prefix + dash_width / 2.));
        assert_eq!(t.layout().len(), 1);
        t.runs(&fonts, |r| {
            assert!(r.glyphs.iter().all(|g| g.id != dash_id))
        });
        t.break_at(&mut fonts, text, &style, None);
        assert_eq!(t.layout().len(), 1);
        t.runs(&fonts, |r| {
            assert!(r.glyphs.iter().all(|g| g.id != dash_id))
        });

        let none = Style {
            hyphens: Hyphens::None,
            ..style
        };
        let mut t = laid(&mut fonts, text, &none, Some(prefix + dash_width + 0.01));
        assert_eq!(t.layout().len(), 1);
        t.runs(&fonts, |r| {
            assert!(r.glyphs.iter().all(|g| g.id != dash_id))
        });
        assert!(t.content_widths().0 > prefix + dash_width);
    }

    #[test]
    fn automatic_hyphens_follow_the_installed_language_without_rewriting_source() {
        let mut fonts = fonts();
        for (lang, word, prefix) in [("en", "hyphenation", "hy"), ("de", "Silbentrennung", "Sil")] {
            og_i18n::hyphenation::install(
                lang,
                file(&format!("fonts/latin/{lang}.hyph")).as_slice().into(),
            );
            let style = Style {
                family: FAMILY,
                hyphens: Hyphens::Auto,
                locale: Some(lang),
                ..Default::default()
            };
            let width = laid(&mut fonts, prefix, &style, None).width()
                + laid(&mut fonts, "-", &style, None).width()
                + 1.;
            let mut t = laid(&mut fonts, word, &style, Some(width + 0.01));
            assert_eq!(
                t.layout().lines().next().unwrap().text_range(),
                0..prefix.len(),
                "{lang}: {width}"
            );
            assert!(
                t.layout()
                    .lines()
                    .next()
                    .unwrap()
                    .discretionary_offset()
                    .is_some()
            );
            assert_eq!(t.laid(word), word);
            assert_eq!(
                t.layout().lines().last().unwrap().text_range().end,
                word.len()
            );
            assert!(t.content_widths().0 < t.content_widths().1);
            t.break_at(&mut fonts, word, &style, None);
            assert_eq!(t.layout().len(), 1);
            assert!(
                t.layout()
                    .lines()
                    .next()
                    .unwrap()
                    .discretionary_offset()
                    .is_none()
            );
            // Missing dictionaries retain manual opportunities, never guess a language.
            let unknown = Style {
                locale: Some("und"),
                ..style
            };
            assert_eq!(
                laid(&mut fonts, word, &unknown, Some(width)).layout().len(),
                1
            );
        }
    }

    #[test]
    fn a_discretionary_ending_follows_its_word_in_bidi_text() {
        let mut fonts = fonts();
        fonts
            .add(file("fonts/hebrew/NotoSansHebrew-Regular.ttf"))
            .unwrap();
        let style = Style {
            family: "\"Noto Sans Hebrew\", \"Barlow Semi Condensed\"",
            direction: Direction::Rtl,
            ..Default::default()
        };
        for (text, prefix) in [("אב\u{ad}גדהוז", "אב"), ("אב hy\u{ad}phenation", "אב hy")]
        {
            let mut dash = laid(&mut fonts, "-", &style, None);
            let mut dash_id = 0;
            dash.runs(&fonts, |r| dash_id = r.glyphs[0].id);
            let width = laid(&mut fonts, prefix, &style, None).width() + dash.width() + 1.;
            let mut t = laid(&mut fonts, text, &style, Some(width));
            let first = t.first_baseline().unwrap();
            let mut glyphs = Vec::new();
            t.runs(&fonts, |r| {
                glyphs.extend(
                    r.glyphs
                        .iter()
                        .filter(|g| (g.y - first).abs() < 0.01)
                        .copied(),
                )
            });
            let ending = glyphs
                .iter()
                .find(|g| g.id == dash_id)
                .expect("visible ending");
            assert!(
                ending.x >= -0.01 && ending.x + ending.advance <= width + 0.01,
                "{text}: {glyphs:?}"
            );
            assert!(
                glyphs
                    .iter()
                    .filter(|g| g.id != dash_id)
                    .all(|g| g.x + g.advance <= ending.x + 0.01
                        || g.x >= ending.x + ending.advance - 0.01),
                "ending has its own gap: {glyphs:?}"
            );
        }
    }

    #[test]
    fn a_hyphenation_point_inside_a_ligature_can_break_without_losing_wide_ligatures() {
        let mut fonts = fonts();
        fonts
            .add(file("styles/shared/Newsreader[opsz,wght].ttf"))
            .unwrap();
        let style = Style {
            family: "Newsreader",
            ..Default::default()
        };
        let text = "of\u{ad}fice";
        let width = laid(&mut fonts, "of-", &style, None).width() + 1.;
        let mut t = laid(&mut fonts, text, &style, Some(width));
        assert_eq!(t.layout().lines().next().unwrap().text_range(), 0..4);
        let min = laid(&mut fonts, "fice", &style, None)
            .width()
            .max(width - 1.);
        assert!((t.content_widths().0 - min).abs() < 0.01);
        let mut whole = laid(&mut fonts, "office", &style, None);
        let mut expected = Vec::new();
        whole.runs(&fonts, |r| {
            expected.extend(
                r.glyphs
                    .iter()
                    .filter(|g| g.advance > 0.)
                    .map(|g| (g.id, g.x)),
            )
        });
        for width in [Some(200.), None, Some(200.)] {
            t.break_at(&mut fonts, text, &style, width);
            let mut actual = Vec::new();
            t.runs(&fonts, |r| {
                actual.extend(
                    r.glyphs
                        .iter()
                        .filter(|g| g.advance > 0.)
                        .map(|g| (g.id, g.x)),
                )
            });
            assert_eq!(
                actual, expected,
                "wide reflow restores the unbroken ligature"
            );
        }
    }

    /// CSS 2 §10.8.1 with Blink's rounding. At line-height 28 px a 16 px
    /// line's baseline lies at round(ascent) plus the floored half of the
    /// leading, both rounded first, from the primary font alone: a CJK
    /// fallback on the line does not move it. At `normal` the line spans
    /// the largest rounded ascent and descent of the faces drawn on it, as
    /// Blink counts its used fonts.
    #[test]
    fn a_line_sits_by_half_leading() {
        let mut fonts = fonts();
        let fixed = Style {
            family: FAMILY,
            line_height: LineHeight::Absolute(28.0),
            ..Default::default()
        };
        let metrics = |src: &str| {
            let data = file(src);
            let f = FontRef::new(&data).unwrap();
            let m = f.metrics(skrifa::instance::Size::new(16.0), LocationRef::default());
            (m.ascent.round(), (-m.descent).round())
        };
        let (a, d) = metrics("styles/shared/BarlowSemiCondensed-SemiBold.ttf");
        let (ca, cd) = metrics("styles/cjk/NotoSansSC[wght].ttf");
        let baseline = Some(a + ((28.0 - a - d) / 2.0).floor());
        for text in ["Level 3", "第 3 关 · Level 3"] {
            let t = laid(&mut fonts, text, &fixed, None);
            assert_eq!((t.height(), t.first_baseline()), (28.0, baseline), "{text}");
        }
        let normal = Style {
            line_height: LineHeight::NORMAL,
            ..fixed
        };
        let t = laid(&mut fonts, "第 3 关 · Level 3", &normal, None);
        let (ma, md) = (a.max(ca), d.max(cd));
        assert_eq!((t.height(), t.first_baseline()), (ma + md, Some(ma)));
    }

    /// A face with an optical size axis is drawn at its font size (CSS
    /// `font-optical-sizing: auto`): Newsreader's axis runs 6 to 72 around
    /// 18, so 12 px lies below its default, 18 px on it and 48 px above. A
    /// face without the axis has no location.
    #[test]
    fn a_face_with_an_optical_size_axis_is_drawn_at_its_size() {
        let mut fonts = fonts();
        let data = file("styles/shared/Newsreader[opsz,wght].ttf");
        let axis = FontRef::new(&data)
            .unwrap()
            .axes()
            .iter()
            .position(|a| a.tag() == skrifa::Tag::new(b"opsz"))
            .unwrap();
        fonts.add(data).unwrap();
        let opsz = |fonts: &mut Collection, family: &'static str, size: f32| {
            let style = Style {
                family,
                size,
                ..Default::default()
            };
            let mut t = laid(fonts, "Hamburg", &style, None);
            let mut at = None;
            t.runs(fonts, |run| at = Some(run.coords.get(axis).copied()));
            at.unwrap()
        };
        let small = opsz(&mut fonts, "Newsreader", 12.0).unwrap();
        let large = opsz(&mut fonts, "Newsreader", 48.0).unwrap();
        assert!(small < 0 && large > 0, "{small} {large}");
        // At the axis's default the face has no location to set.
        assert_eq!(opsz(&mut fonts, "Newsreader", 18.0).unwrap_or(0), 0);
        assert_eq!(opsz(&mut fonts, "\"Barlow Semi Condensed\"", 12.0), None);
    }

    /// `text-box: trim-both cap alphabetic` leaves the cap height: the box
    /// starts at the caps' top and ends on the baseline.
    #[test]
    fn trim_leaves_cap_height_to_baseline() {
        let mut fonts = fonts();
        let style = Style {
            family: FAMILY,
            size: 20.0,
            line_height: LineHeight::Absolute(44.0),
            trim: Trim {
                start: Some(Over::Cap),
                end: Some(Under::Alphabetic),
            },
            ..Default::default()
        };
        let data = file("styles/shared/BarlowSemiCondensed-SemiBold.ttf");
        let f = FontRef::new(&data).unwrap();
        let cap = f
            .metrics(skrifa::instance::Size::new(20.0), LocationRef::default())
            .cap_height
            .unwrap();
        let t = laid(&mut fonts, "PLAY", &style, None);
        assert!(
            (t.height() - cap).abs() < 1e-3,
            "{} vs cap {cap}",
            t.height()
        );
        assert!((t.first_baseline().unwrap() - cap).abs() < 1e-3);
    }

    /// A no-wrap line wider than its box ends in "…" after the longest
    /// prefix that fits beside it; one that fits is left whole.
    #[test]
    fn an_overflowing_line_ends_in_an_ellipsis() {
        let mut fonts = fonts();
        let style = Style {
            family: FAMILY,
            wrap: false,
            ellipsis: true,
            ..Default::default()
        };
        let name = "The Warden of the Far Northern Observatory";
        let whole = laid(&mut fonts, name, &style, None).width();
        let width = (whole / 2.0).round();
        let t = laid(&mut fonts, name, &style, Some(width));
        assert!(t.width() <= width, "{} > {width}", t.width());
        // The longest prefix that fits beside "…", found by trying each.
        let mut cut = None;
        for end in (1..name.len()).rev().filter(|&e| name.is_char_boundary(e)) {
            let prefix = format!("{}\u{2026}", name[..end].trim_end());
            if laid(
                &mut fonts,
                &prefix,
                &Style {
                    ellipsis: false,
                    ..style.clone()
                },
                None,
            )
            .width()
                <= width
            {
                cut = Some(prefix);
                break;
            }
        }
        let cut = cut.unwrap();
        let mut plain = Style {
            ellipsis: false,
            ..style.clone()
        };
        plain.wrap = false;
        let expect = laid(&mut fonts, &cut, &plain, None).width();
        assert!(
            (t.width() - expect).abs() < 1e-3,
            "{} drawn, {expect} for {cut:?}",
            t.width()
        );
        let fits = laid(&mut fonts, name, &style, Some(whole + 1.0));
        assert_eq!(drawn(&fits, name.len()), name.len());
        assert!((fits.width() - whole).abs() < 1e-3);
    }

    /// `line-clamp: 2` lays out a long text's first two lines, the second
    /// ending in "…" inside the box, and sizes the text by them; a text of
    /// fewer lines is left whole, and its min-content stays the uncut
    /// text's. Ways this could fail: the lines after the clamp still laid
    /// out, the last line left whole, or "…" pushed past the box.
    #[test]
    fn a_clamped_text_shows_its_first_lines_and_an_ellipsis() {
        let mut fonts = fonts();
        let style = Style {
            family: FAMILY,
            max_lines: 2,
            ..Default::default()
        };
        let words = "alpha bravo charlie delta echo foxtrot golf hotel";
        let free = Style {
            max_lines: 0,
            ..style.clone()
        };
        let width = laid(&mut fonts, "alpha bravo char", &free, None).width();
        let all = laid(&mut fonts, words, &free, Some(width));
        assert!(all.layout().len() > 2, "{} lines", all.layout().len());
        let min = all.content_widths().0;
        let t = laid(&mut fonts, words, &style, Some(width));
        assert_eq!(t.layout().len(), 2);
        assert!(t.width() <= width + 0.01, "{} > {width}", t.width());
        let shown = t.laid(words);
        assert!(shown.ends_with('\u{2026}'), "{shown:?}");
        assert!(
            words.starts_with(shown.trim_end_matches('\u{2026}')),
            "{shown:?}"
        );
        // As tall as the uncut text's first two lines.
        let two: f32 = all
            .layout()
            .lines()
            .take(2)
            .map(|l| l.metrics().line_height)
            .sum();
        assert!((t.height() - two).abs() < 1e-3, "{} vs {two}", t.height());
        assert_eq!(t.content_widths().0, min);
        // Two lines of it fit: nothing is cut.
        let short = laid(&mut fonts, "alpha bravo charlie", &style, Some(width));
        assert_eq!(short.laid("alpha bravo charlie"), "alpha bravo charlie");
        // No width: only hard breaks, the second line marked.
        let hard = laid(&mut fonts, "one\ntwo\nthree", &style, None);
        assert_eq!(hard.laid("one\ntwo\nthree"), "one\ntwo\u{2026}");
    }

    /// `overflow-wrap`: under `normal` a word wider than the box overflows
    /// it; under `break-word` it breaks at the box, the other words wrap
    /// whole, and min-content stays the longest word; under `anywhere`
    /// min-content is the widest letter.
    #[test]
    fn overflow_wrap_follows_css() {
        let mut fonts = fonts();
        let text = "go Supercalifragilistic go";
        let style = |overflow_wrap| Style {
            family: FAMILY,
            overflow_wrap,
            ..Default::default()
        };
        let word = laid(
            &mut fonts,
            "Supercalifragilistic",
            &style(OverflowWrap::Normal),
            None,
        )
        .width();
        let width = (word / 2.0).round();
        let normal = laid(&mut fonts, text, &style(OverflowWrap::Normal), Some(width));
        assert!((normal.width() - word).abs() < 1e-3);
        let broken = laid(
            &mut fonts,
            text,
            &style(OverflowWrap::BreakWord),
            Some(width),
        );
        assert!(broken.width() <= width + 1e-3, "{}", broken.width());
        let first = broken.layout().lines().next().unwrap().text_range();
        assert_eq!(text[first].trim_end(), "go");
        assert!((broken.content_widths().0 - word).abs() < 1e-3);
        let anywhere = laid(
            &mut fonts,
            text,
            &style(OverflowWrap::Anywhere),
            Some(width),
        );
        let widest = "Supercalifragilistic"
            .chars()
            .map(|c| {
                laid(
                    &mut fonts,
                    &c.to_string(),
                    &style(OverflowWrap::Normal),
                    None,
                )
                .width()
            })
            .fold(0.0, f32::max);
        assert!((anywhere.content_widths().0 - widest).abs() < 1e-3);
    }

    /// The caret stands at the width of the text before it, on the line
    /// that holds it; a line reports where its text lies; and a character
    /// no face draws is missing, a joiner or a space never.
    #[test]
    fn carets_lines_and_missing_characters() {
        let mut fonts = fonts();
        let style = Style {
            family: FAMILY,
            line_height: LineHeight::Absolute(20.0),
            ..Default::default()
        };
        let text = "Level 3\nGo";
        let t = laid(&mut fonts, text, &style, None);
        let level = laid(&mut fonts, "Level", &style, None).width();
        let [x, top, height] = t.caret(5);
        assert!((x - level).abs() < 1e-3, "{x} vs {level}");
        assert_eq!((top, height), (0.0, 20.0));
        assert_eq!(t.caret(0)[0], 0.0);
        let [x, top, _] = t.caret(text.len());
        let go = laid(&mut fonts, "Go", &style, None).width();
        assert!((x - go).abs() < 1e-3 && top == 20.0, "{x} {top}");
        let mut lines = Vec::new();
        t.lines(|at, range| lines.push((at, text[range].trim_end().to_string())));
        assert_eq!(lines.len(), 2);
        assert_eq!((lines[0].0[0], &*lines[0].1), (0.0, "Level 3"));
        assert_eq!(lines[1].0[1] - lines[0].0[1], 20.0, "a line apart");
        assert!((lines[1].0[2] - go).abs() < 1e-3);
        // Barlow and Noto Sans SC lack the Arabic letter; ZWJ draws nothing.
        let text = "a \u{200d}\u{0628}关";
        let t = laid(&mut fonts, text, &style, None);
        let mut missing = Vec::new();
        t.missing(text, |c| missing.push(c));
        assert_eq!(missing, ['\u{0628}']);
        let mut none = Collection::new();
        let t = laid(&mut none, "a b", &style, None);
        let mut missing = Vec::new();
        t.missing("a b", |c| missing.push(c));
        assert_eq!(missing, ['a', 'b'], "no face at all");
    }
}
