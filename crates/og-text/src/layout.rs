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
    pub align: Alignment,
    /// `text-overflow: ellipsis`.
    pub ellipsis: bool,
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
            align: Alignment::Start,
            ellipsis: false,
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
    /// The width the lines were last broken at.
    width: Option<f32>,
    /// What trim takes off the top and the bottom, CSS px.
    trimmed: [f32; 2],
    /// Whether the layout holds the ellipsized text.
    cut: bool,
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
        build(fonts, &mut self.layout, text, style);
        self.cut = false;
        self.width = None;
        self.layout.break_all_lines(None);
        self.trimmed = trim(fonts, style, &self.layout);
    }

    /// Breaks the lines at `width` (none: only at hard breaks), aligned in
    /// it. With `style.ellipsis`, a line wider than `width` ends at the last
    /// whole cluster that fits beside "…"; `text` and `style` must be the
    /// ones [`Text::shape`] took.
    pub fn break_at(
        &mut self,
        fonts: &mut Collection,
        text: &str,
        style: &Style,
        width: Option<f32>,
    ) {
        if self.cut {
            build(fonts, &mut self.layout, text, style);
            self.cut = false;
        }
        self.width = width;
        self.layout.break_all_lines(width);
        if let (true, Some(w)) = (style.ellipsis, width)
            && let Some(cut) = ellipsized(fonts, &self.layout, text, style, w)
        {
            build(fonts, &mut self.layout, &cut, style);
            self.cut = true;
            self.layout.break_all_lines(width);
        }
        self.layout.align(style.align, AlignmentOptions::default());
        self.trimmed = trim(fonts, style, &self.layout);
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
        let w = self.layout.calculate_content_widths();
        (w.min, w.max)
    }

    /// The first line's baseline, CSS px below the box's top.
    pub fn first_baseline(&self) -> Option<f32> {
        self.layout.first_baseline().map(|b| b - self.trimmed[0])
    }

    /// The layout, for Parley's cursor, selection and hit testing.
    pub fn layout(&self) -> &parley::Layout<()> {
        &self.layout
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
        for line in self.layout.lines() {
            for item in line.items() {
                let PositionedLayoutItem::GlyphRun(run) = item else {
                    continue;
                };
                let r = run.run();
                let Some(face) = fonts.face_of(r.font()) else {
                    continue;
                };
                self.glyphs.clear();
                self.glyphs.extend(run.positioned_glyphs().map(|mut g| {
                    g.y -= top;
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
                    baseline: run.baseline() - top,
                    glyphs: &self.glyphs,
                });
            }
        }
    }
}

/// Shapes `text` in `style` into `layout`.
fn build(fonts: &mut Collection, layout: &mut parley::Layout<()>, text: &str, style: &Style) {
    let (context, lcx) = fonts.contexts();
    let mut b = lcx.ranged_builder(context, text, 1.0, true);
    b.set_line_break_override(Some(parley::CHROMIUM_LINE_BREAK_OVERRIDE));
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

/// `text` with each line of `layout` wider than `width` cut at the last
/// cluster boundary that leaves room for "…", and "…" added; `None` when
/// every line fits.
fn ellipsized(
    fonts: &mut Collection,
    layout: &parley::Layout<()>,
    text: &str,
    style: &Style,
    width: f32,
) -> Option<String> {
    if layout.lines().all(|l| l.metrics().advance <= width + 0.01) {
        return None;
    }
    // "…" as this text would draw it.
    let mut mark = parley::Layout::new();
    build(fonts, &mut mark, "\u{2026}", style);
    mark.break_all_lines(None);
    let room = (width - mark.width()).max(0.0);
    let mut out = String::with_capacity(text.len() + 3);
    let mut clusters: Vec<(std::ops::Range<usize>, f32)> = Vec::new();
    for line in layout.lines() {
        let range = line.text_range();
        if line.metrics().advance <= width + 0.01 {
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
            if pen > room + 0.01 {
                break;
            }
            end = r.end;
        }
        out.push_str(text[range.start..end].trim_end());
        out.push('\u{2026}');
        // A hard break that ended the line stays.
        if text[..range.end].ends_with('\n') {
            out.push('\n');
        }
    }
    Some(out)
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
