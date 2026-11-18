//! Rich text: one paragraph of inline spans (colour, weight, slant, lines
//! under and through, outline, size, font, inline icons, pictures and action
//! glyphs, links and per-glyph motion) from markup ([`crate::markup`]),
//! laid out as one text: the lines break between the spans' words (a word
//! split across two spans stays whole), each line as tall as its tallest
//! span, the spans on one shared baseline unless a span's CSS
//! `vertical-align` moves it (CSS 2.1 §10.8), the line then grown to hold it.
//!
//! Links are children of the paragraph for the probe and for input: each
//! `[link=<id>]` is an element `<paragraph>/<id>` (kind `link`) with its
//! bounds (the union of its pieces), reachable by keyboard and gamepad
//! focus, pressed like a button; [`crate::Ui::rich_text`] answers which
//! was clicked this tick.

use crate::css::text_css::SkipInk;
use crate::id::child_key;

use crate::layout::{
    Content, Element, FontRole, InlinePlace, Metrics, Rect, Size, Strut, TextAlign, VerticalAlign,
};
use crate::markup::SpanFx;
use crate::style::{Srgba, TypeStep};
use crate::ui::{Response, Ui};

/// An inline thing a span is instead of text, laid out as one unbreakable
/// unit on the baseline.
#[derive(Clone, Debug, PartialEq)]
pub enum Atom {
    /// An icon of the atlas, one em square, in the span's ink.
    Icon(crate::ui::IconCell),
    /// A registered picture, one em tall at its aspect, in its own colours.
    Image(crate::ui::InlineImage),
    /// A key's name on a cap (an action's glyph with no icon).
    Cap(String),
    /// The span's text with its annotation over or under it (HTML's
    /// `<ruby>` and `<rt>`, CSS Ruby 1): one box, never broken.
    Ruby(Ruby),
}

/// One resolved span: its text (or atom) in its own style.
#[derive(Clone, Debug, PartialEq)]
pub struct RichSpan {
    pub text: String,
    pub step: TypeStep,
    pub color: Srgba,
    pub font: FontRole,
    pub bold: bool,
    pub italic: bool,
    /// The weight its face is picked at: 700 for bold, else its step's.
    pub weight: u16,
    /// The width its face is picked at (CSS `font-stretch`), a percentage
    /// of the normal; `None`: normal.
    pub stretch: Option<f32>,
    /// A variable face's settings (CSS `font-variation-settings`), and
    /// whether its `opsz` follows the size (`font-optical-sizing: auto`).
    pub variations: crate::shape::Variations,
    pub optical_sizing: bool,
    /// A colour face's palette (CSS `font-palette`).
    pub palette: crate::shape::Palette,
    /// How its emoji are presented (CSS `font-variant-emoji`).
    pub emoji: crate::text::EmojiStyle,
    /// The size its faces are drawn at (CSS `font-size-adjust`).
    pub size_adjust: Option<crate::text::SizeAdjust>,
    /// The bold and slant no real face of the style draws, synthesised.
    pub synth: crate::ui::Synth,
    /// A synthesised oblique's angle, degrees (CSS `font-style: oblique
    /// <angle>`); `None`: the engine's slant.
    pub oblique: Option<f32>,
    pub underline: bool,
    pub strike: bool,
    /// An outline's width (CSS px), colour and where it runs against the
    /// glyphs' edge.
    pub outline: Option<(f32, Srgba, og_draw::UiAlign)>,
    pub atom: Option<Atom>,
    pub link: Option<String>,
    pub fx: SpanFx,
    /// Its CSS decoration's style, colour, thickness, offset and overline.
    pub decoration: Option<crate::css::text_css::Decoration>,
    /// What a line broken at its soft hyphen ends in (CSS
    /// `hyphenate-character`); `None`: a hyphen-minus.
    pub hyphen: Option<std::sync::Arc<str>>,
    /// Where it sits in its line against the paragraph's text.
    pub valign: VerticalAlign,
    /// CSS `word-spacing`, px: added after each word separator.
    pub word_spacing: f32,
    /// Its own `text-combine-upright` (CSS Writing Modes 4 §9.1); `None`:
    /// its text's.
    pub combine: Option<crate::layout::flow::Combine>,
    /// The keys of the inline elements it sits in, outermost first.
    pub owners: Vec<u64>,
    /// Each owner's edges at this span, beside [`RichSpan::owners`].
    pub edges: Vec<InlineEdges>,
}

/// An inline element's margin, border and padding at one of its spans (CSS
/// 2 §10.3.1, §10.6.1). Its inline start's and end's (`[margin, border
/// and padding]`, px) take room on the line before its first character and
/// after its last: on the span that `opens` it and the one that `closes`
/// it, and under `box-decoration-break: clone` at each line break inside
/// it too (CSS Fragmentation 3 §5.4, [`cut_room`]). Its border and padding
/// across its lines (above and below a horizontal line, left and right of
/// a vertical one) only grow its fragments.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct InlineEdges {
    pub start: [f32; 2],
    pub end: [f32; 2],
    pub across: [f32; 2],
    pub opens: bool,
    pub closes: bool,
    pub clone: bool,
}

impl InlineEdges {
    /// The room it takes before a span's text and after it.
    fn room(&self) -> [f32; 2] {
        [
            if self.opens {
                self.start[0] + self.start[1]
            } else {
                0.0
            },
            if self.closes {
                self.end[0] + self.end[1]
            } else {
                0.0
            },
        ]
    }

    /// It takes room or grows its fragments.
    pub(crate) fn any(&self) -> bool {
        [self.start, self.end, self.across] != [[0.0; 2]; 3]
    }
}

/// The room a line break between spans `last` and `next` takes: the end's
/// of each element it splits under `box-decoration-break: clone`, before
/// it, and their start's after it (CSS Fragmentation 3 §5.4).
pub(crate) fn cut_room(rc: &RichContent, last: usize, next: usize) -> [f32; 2] {
    let (a, b) = (&rc.spans[last], &rc.spans[next]);
    a.owners
        .iter()
        .zip(&a.edges)
        .filter(|(o, e)| e.clone && b.owners.contains(o))
        .fold([0.0; 2], |[x, y], (_, e)| {
            [x + e.end[0] + e.end[1], y + e.start[0] + e.start[1]]
        })
}

/// Line `line` with room for the edges a break splits at its logical start
/// (`start`: its first span and the width) and end, on its left and right
/// (the other way round when `rtl`): pieces with no text, as an edge's room
/// at an element's start and end is.
pub(crate) fn with_room(line: &mut RichLine, start: (usize, f32), end: (usize, f32), rtl: bool) {
    let (left, right) = if rtl { (end, start) } else { (start, end) };
    let piece = |(span, width): (usize, f32), x: f32| Piece {
        span,
        text: String::new(),
        x,
        width,
        rise: 0.0,
    };
    let segment = |(span, width): (usize, f32)| Segment {
        span,
        text: String::new(),
        at: 0,
        glyphs: None,
        width,
        rise: 0.0,
    };
    if left.1 > 0.0 {
        for p in &mut line.pieces {
            p.x += left.1;
        }
        line.pieces.insert(0, piece(left, 0.0));
        if let Some(segments) = &mut line.shaped {
            segments.insert(0, segment(left));
        }
        line.width += left.1;
    }
    if right.1 > 0.0 {
        let x = line.pieces.last().map_or(0.0, |p| p.x + p.width);
        line.pieces.push(piece(right, x));
        if let Some(segments) = &mut line.shaped {
            let at = segments.last().map_or(0, |s| s.at + s.text.chars().count());
            segments.push(Segment {
                at,
                ..segment(right)
            });
        }
        line.width += right.1;
    }
}

impl RichSpan {
    /// What it asks of its font family: its weight, slant, width and a
    /// variable face's axes.
    pub fn query(&self) -> crate::FontQuery {
        crate::FontQuery::new(self.weight, self.italic)
            .at_width(self.stretch)
            .with_axes(self.variations, self.optical_sizing, self.step.size)
            .in_palette(self.palette)
            .with_emoji(self.emoji)
            .with_size_adjust(self.size_adjust)
    }

    /// Its text's setting down a vertical line `v`: under its own
    /// `text-combine-upright`, `all` composing its whole text in one em.
    pub(crate) fn vertical(
        &self,
        v: crate::layout::flow::VerticalText,
    ) -> crate::layout::flow::VerticalText {
        self.combine.map_or(v, |c| v.combining(c, &self.text))
    }

    /// The room its owners' edges take before its text and after it, px.
    pub(crate) fn room(&self) -> [f32; 2] {
        self.edges.iter().fold([0.0; 2], |[a, b], e| {
            let [s, t] = e.room();
            [a + s, b + t]
        })
    }

    /// Whether it is composed in one upright em down `v`, one unit of
    /// its line (CSS Writing Modes 4 §9.1: as U+FFFC for line breaking).
    fn combined(&self, v: Option<crate::layout::flow::VerticalText>) -> bool {
        self.atom.is_none() && v.is_some_and(|v| self.vertical(v).combine)
    }
}

/// An italic's slant: x moves by this much of the height above the baseline.
const SLANT: f32 = 0.2;

/// A cap's text size, margin and padding, of the span's size.
const CAP_TEXT: f32 = 0.78;
const CAP_MARGIN: f32 = 0.08;
const CAP_PAD: f32 = 0.24;

/// A paragraph of spans.
#[derive(Clone, Debug, PartialEq)]
pub struct RichContent {
    pub spans: Vec<RichSpan>,
    /// The paragraph's own type step and font: the text a span's
    /// `vertical-align` is measured against (its strut).
    pub base: (TypeStep, FontRole),
    pub align: TextAlign,
    pub wrap: bool,
    pub ellipsis: bool,
    pub(crate) overflow: Option<crate::layout::TextOverflow>,
    pub max_lines: u32,
    pub cut: Option<String>,
    pub indent: TextIndent,
    pub line_break: LineBreak,
    /// Its text's emphasis marks (CSS `text-emphasis`).
    pub emphasis: Option<crate::css::text_css::Emphasis>,
    /// CSS `dominant-baseline`: the baseline of its root text a span
    /// aligned at `baseline` meets its own at (CSS Inline 3 §4.1).
    pub dominant: crate::layout::Baseline,
    /// CSS `text-autospace` and `text-spacing-trim` (CSS Text 4 §8.4,
    /// §8.5).
    pub spacing: crate::text::spacing::CjkSpacing,
    /// CSS `hanging-punctuation` (CSS Text 4 §9.2.1).
    pub hanging: crate::text::hanging::HangingPunctuation,
    /// CSS `text-box-trim` and `text-box-edge` (CSS Inline 3 §6).
    pub text_box: crate::text::text_box::TextBox,
}

/// CSS `text-indent` (CSS Text 3 §8.1): the room its lines leave at
/// their start, in CSS px; `hanging` indents every line but the first,
/// `each-line` the first after each forced break too.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextIndent {
    pub px: f32,
    pub hanging: bool,
    pub each_line: bool,
}

/// CSS `line-break` (CSS Text 3 §5.3): how strictly CJK punctuation and
/// small kana may start or end a line. `auto` is Parley's default,
/// `strict`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum LineBreak {
    #[default]
    Auto,
    Loose,
    Normal,
    Strict,
    Anywhere,
}

/// A piece of one span on one line, `x` from the line's start.
#[derive(Clone, Debug, PartialEq)]
pub struct Piece {
    pub span: usize,
    pub text: String,
    pub x: f32,
    pub width: f32,
    /// How far its span's baseline is raised above the line's.
    pub rise: f32,
}

/// One laid-out line of a paragraph.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RichLine {
    pub pieces: Vec<Piece>,
    /// Its line box's width: its pieces and its indent.
    pub width: f32,
    /// The room its `text-indent` leaves at its inline start, before its
    /// pieces (CSS Text 3 §8.1).
    pub indent: f32,
    pub height: f32,
    /// From the line's top to the shared baseline.
    pub baseline: f32,
    pub shaped: Option<Vec<Segment>>,
}

/// How rich text looks by default.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RichTextDesc {
    /// `None`: `text-body`.
    pub step: Option<TypeStep>,
    /// `None`: `ink`.
    pub color: Option<Srgba>,
    pub align: TextAlign,
    pub nowrap: bool,
    pub ellipsis: bool,
    pub max_lines: u32,
    pub cut: Option<String>,
    pub width: Size,
    pub font: FontRole,
    pub indent: TextIndent,
    pub line_break: LineBreak,
    /// CSS `text-emphasis`.
    pub emphasis: Option<crate::css::text_css::Emphasis>,
    /// CSS `dominant-baseline`.
    pub dominant: crate::layout::Baseline,
    /// CSS `ruby-position`, `ruby-align` and `ruby-overhang`: how its
    /// inline `<ruby>` is set.
    pub ruby: RubyStyle,
    /// CSS `text-autospace` and `text-spacing-trim`.
    pub spacing: crate::text::spacing::CjkSpacing,
    /// CSS `hanging-punctuation`.
    pub hanging: crate::text::hanging::HangingPunctuation,
    /// CSS `text-box-trim` and `text-box-edge`.
    pub text_box: crate::text::text_box::TextBox,
}

/// What a paragraph's links did this tick.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RichTextResponse {
    /// The links together: hovered while one is, clicked when one was.
    pub response: Response,
    /// The link clicked this tick (its `[link=…]` id).
    pub link: Option<String>,
    /// The link under the pointer.
    pub hovered: Option<String>,
    /// Structured attributes, including unknown names and zero-length markers.
    pub spans: Vec<TextSpan>,
}

pub(crate) mod emphasis;
mod ruby;
mod spans;
pub use ruby::{
    Ruby, RubyAlign, RubyBase, RubyLeaf, RubyLevel, RubyOut, RubyOverhang, RubyPosition, RubyRole,
    RubySide, RubyStyle, structure as ruby_structure,
};
pub(crate) use ruby::{ruby_box, ruby_extents};
pub(crate) use spans::synthetic;
pub use spans::{InlineAtom, TextSpan, TextSpanStyle, TextSpanValue};

/// A unit lines break between: a span's word, space or CJK character.
struct Unit {
    span: usize,
    text: String,
    space: bool,
    width: f32,
    /// Glued to the unit before it (the same word continued in another span).
    glued: bool,
}

/// What a span's `text` takes along its line: its width across, or its
/// length down a vertical line `v` (CSS Writing Modes 4), as
/// [`Ui::vertical_rich_pieces`] sets it: text at its vertical advances
/// (upright clusters) and horizontal ones (sideways runs), a picture or
/// icon standing upright, a key cap's label down its plate.
pub(crate) fn span_advance(
    s: &RichSpan,
    text: &str,
    fonts: &dyn Metrics,
    v: Option<crate::layout::flow::VerticalText>,
) -> f32 {
    let Some(v) = v else {
        return span_width(s, text, fonts);
    };
    let v = s.vertical(v);
    let size = s.step.size;
    let down = |t: &str, size: f32| {
        let m = fonts.metrics_styled(s.font, s.query(), t);
        crate::text::vertical::measure(
            m,
            t,
            size,
            span_tracking(s, t),
            crate::shape::Features::NONE,
            v,
        )
    };
    match &s.atom {
        Some(Atom::Icon(_) | Atom::Image(_)) => size * 1.1,
        Some(Atom::Cap(t)) => down(t, size * CAP_TEXT) + 2.0 * (CAP_MARGIN + CAP_PAD) * size,
        Some(Atom::Ruby(r)) => ruby_box(s, r, size, fonts, Some(v)).advance,
        None => down(text, size),
    }
}

pub(crate) fn span_width(s: &RichSpan, text: &str, fonts: &dyn Metrics) -> f32 {
    let size = s.step.size;
    match &s.atom {
        Some(Atom::Icon(_)) => return size * 1.1,
        Some(Atom::Image(im)) => return size * (im.aspect.max(0.05) + 0.1),
        Some(Atom::Cap(t)) => {
            let m = fonts.metrics_for(s.font, t);
            return m.measure_tracked(t, size * CAP_TEXT, 0.0)
                + 2.0 * (CAP_MARGIN + CAP_PAD) * size;
        }
        Some(Atom::Ruby(r)) => return ruby_box(s, r, size, fonts, None).advance,
        None => {}
    }
    let m = fonts.metrics_styled(s.font, s.query(), text);
    m.measure_tracked(text, s.step.size, span_tracking(s, text))
}

/// Authored tracking (em). Synthetic bold grows ink only, as Skia does:
/// it must not add advances, tracking or trailing room to the run.
pub(crate) fn span_tracking(s: &RichSpan, text: &str) -> f32 {
    crate::layout::step_tracking(&s.step, text)
}

/// Synthetic outline growth in em, as in FreeType's
/// FT_GlyphSlot_Embolden and Skia's SkScalerContext_FreeType::emboldenIfNeeded
/// (src/ports/SkFontHost_FreeType.cpp, BSD-3-Clause). Using the difference
/// between a real bold and regular face instead overfills small counters.
const BOLD_GROWTH: f32 = 1.0 / 24.0;

/// A bold span's thickening at `size`, each side, when its second copy is
/// `smear` to the right (both in the text's units): together they grow its
/// stems by [`BOLD_GROWTH`], and small text by at least a quarter of the
/// copy's offset more.
fn bold_weight(size: f32, smear: f32) -> f32 {
    ((size * BOLD_GROWTH - smear) * 0.5).max(smear * 0.25)
}

/// A synthesised bold at `size` (the text's units), `device` device
/// pixels to one of them: the thickening in the shader and how far right
/// its second copy is drawn (one device pixel).
pub(crate) fn synthetic_bold(size: f32, device: f32) -> (f32, f32) {
    let smear = 1.0 / device.max(1e-3);
    (bold_weight(size, smear), smear)
}

pub(crate) fn span_metrics(s: &RichSpan, fonts: &dyn Metrics) -> (f32, f32, f32) {
    // (line height, ascent, descent) in CSS px.
    if let Some(Atom::Ruby(r)) = &s.atom {
        let (above, below) = ruby_extents(s, r, fonts);
        return (above + below, above, -below);
    }
    if s.atom.is_some() {
        return (s.step.line_height, s.step.size * 0.85, -s.step.size * 0.25);
    }
    let m = fonts.metrics_styled(s.font, s.query(), &s.text);
    let lh = if s.text.chars().any(crate::text::is_cjk) {
        s.step.cjk_line_height
    } else {
        s.step.line_height
    };
    // A span mixing faces: the extents of every face it takes.
    let (a, d) = m.extents(&s.text);
    (lh, a * s.step.size, d * s.step.size)
}

/// The byte ranges of `text`'s first and last grapheme, where an inline
/// element's start and end take room ([`RichSpan::room`]): the same
/// range for a text of one. `None` for an empty text.
pub(crate) fn edge_graphemes(
    text: &str,
) -> Option<(std::ops::Range<usize>, std::ops::Range<usize>)> {
    let g = og_i18n::graphemes(text);
    let (first, last) = (g.first()?, g.last()?);
    let byte = |c: usize| text.char_indices().nth(c).map_or(text.len(), |(b, _)| b);
    Some((byte(first.0)..byte(first.1), byte(last.0)..byte(last.1)))
}

/// The paragraphs' units (paragraphs split at `\n`), measured along a
/// line ([`span_advance`]).
fn paragraphs(
    rc: &RichContent,
    fonts: &dyn Metrics,
    v: Option<crate::layout::flow::VerticalText>,
) -> Vec<Vec<Unit>> {
    let mut out: Vec<Vec<Unit>> = vec![Vec::new()];
    for (k, s) in rc.spans.iter().enumerate() {
        let [before, after] = s.room();
        // An atom, or a text composed in one em: one unit of the line. An
        // atom's box holds its owners' edges.
        if s.atom.is_some() || s.combined(v) {
            let text = if s.atom.is_some() {
                String::new()
            } else {
                s.text.replace('\n', "")
            };
            let glued = out.last().and_then(|p| p.last()).is_some_and(|u| !u.space);
            let room = if s.atom.is_some() {
                before + after
            } else {
                0.0
            };
            if s.atom.is_none() {
                room_unit(&mut out, k, before, glued);
            }
            out.last_mut().expect("a paragraph").push(Unit {
                span: k,
                width: span_advance(s, &text, fonts, v) + room,
                text,
                space: false,
                glued: glued || (s.atom.is_none() && before != 0.0),
            });
            if s.atom.is_none() {
                room_unit(&mut out, k, after, true);
            }
            continue;
        }
        for (n, para) in s.text.split('\n').enumerate() {
            if n > 0 {
                out.push(Vec::new());
            }
            for (i, (t, space)) in crate::text::tokens(para).into_iter().enumerate() {
                let glued = i == 0
                    && !space
                    && out
                        .last()
                        .and_then(|p| p.last())
                        .is_some_and(|u| !u.space && u.span != k);
                // Its owners' start: room before its first word, glued to it
                // and broken before as the word is.
                if n == 0 && i == 0 && before != 0.0 {
                    room_unit(&mut out, k, before, glued);
                }
                let glued = glued || (n == 0 && i == 0 && before != 0.0 && !space);
                out.last_mut().expect("a paragraph").push(Unit {
                    span: k,
                    text: t.to_string(),
                    space,
                    width: span_advance(s, t, fonts, v),
                    glued,
                });
            }
        }
        room_unit(&mut out, k, after, true);
    }
    out
}

/// Room of `width` for span `k`'s owners' edges ([`RichSpan::room`]): a
/// unit with no text, its own piece of the line.
fn room_unit(out: &mut [Vec<Unit>], k: usize, width: f32, glued: bool) {
    if width != 0.0 {
        out.last_mut().expect("a paragraph").push(Unit {
            span: k,
            text: String::new(),
            space: false,
            width,
            glued,
        });
    }
}

/// The widest paragraph unbroken, and the widest unbreakable group: along
/// the line, the longest down a vertical one `v`.
pub(crate) fn rich_widths(
    rc: &RichContent,
    fonts: &dyn Metrics,
    v: Option<crate::layout::flow::VerticalText>,
) -> (f32, f32) {
    let least = |pref: f32, min: f32| {
        if rc.ellipsis
            || matches!(
                rc.overflow,
                Some(crate::layout::TextOverflow::Scroll { .. })
            )
        {
            rc.spans
                .iter()
                .find(|s| s.atom.is_none())
                .map_or(pref, |s| {
                    let m = fonts.metrics_styled(s.font, s.query(), &s.text);
                    let dots = rc
                        .cut
                        .as_deref()
                        .unwrap_or_else(|| crate::layout::ellipsis_of(m));
                    span_advance(s, dots, fonts, v).min(pref)
                })
        } else if rc.wrap {
            min
        } else {
            pref
        }
    };
    #[cfg(feature = "shaping")]
    if v.is_none()
        && let Some((_, widths)) = crate::text::paragraph::rich(rc, fonts, None)
    {
        return (widths.0, least(widths.0, widths.1));
    }
    let mut pref = 0.0f32;
    let mut min = 0.0f32;
    for para in paragraphs(rc, fonts, v) {
        pref = pref.max(para.iter().map(|u| u.width).sum());
        let mut group = 0.0f32;
        for u in &para {
            if u.space {
                group = 0.0;
                continue;
            }
            group = if u.glued { group + u.width } else { u.width };
            min = min.max(group);
        }
    }
    (pref, least(pref, min))
}

/// Cut the final kept line at the content width, retaining its span styles.
/// Cached shaped segments describe the uncut line, so a cut reshapes its
/// surviving pieces rather than drawing those stale glyphs.
pub(crate) fn truncate_lines(
    lines: &mut Vec<RichLine>,
    rc: &RichContent,
    fonts: &dyn Metrics,
    width: f32,
    v: Option<crate::layout::flow::VerticalText>,
) {
    let cut = rc.max_lines > 0 && lines.len() > rc.max_lines as usize;
    if cut {
        lines.truncate(rc.max_lines as usize);
    }
    let Some(line) = lines.last_mut() else { return };
    if !(cut || rc.ellipsis && line.width > width) {
        return;
    }
    // An all-atomic line has no text style in which to draw a cut marker;
    // keep its existing overflow rather than deleting the inline objects.
    if !line.pieces.iter().any(|p| rc.spans[p.span].atom.is_none()) {
        return;
    }
    line.shaped = None;
    while let Some(last) = line.pieces.last() {
        let s = &rc.spans[last.span];
        if s.atom.is_some() {
            line.pieces.pop();
            continue;
        }
        let m = fonts.metrics_styled(s.font, s.query(), &last.text);
        let dots = rc
            .cut
            .as_deref()
            .unwrap_or_else(|| crate::layout::ellipsis_of(m));
        let text = format!("{}{dots}", last.text.trim_end());
        let advance = span_advance(s, &text, fonts, v);
        let end = last.x + advance;
        if end <= width || line.pieces.len() == 1 && last.text.is_empty() {
            let last = line.pieces.last_mut().unwrap();
            last.text = text;
            last.width = advance;
            line.width = end;
            return;
        }
        if last.text.is_empty() {
            line.pieces.pop();
        } else {
            // Do not cut a combining sequence or an emoji in half.
            let keep = og_i18n::graphemes(&last.text).last().map_or(0, |g| g.0);
            // The segmenter returns character indices; String needs bytes.
            let byte = last.text.char_indices().nth(keep).map_or(0, |(b, _)| b);
            line.pieces.last_mut().unwrap().text.truncate(byte);
        }
    }
    line.width = 0.0;
}

/// Greedy lines at `max` (`None`: only at `\n`). Down a vertical line `v`
/// each unit takes its vertical length ([`span_advance`]): the lines break
/// where they are drawn, not where a horizontal line would, so an upright
/// run or a mix of upright and sideways text fills its column. The shaper
/// lays out horizontal lines only; a vertical one is shaped as it is drawn.
pub(crate) fn rich_lines(
    rc: &RichContent,
    fonts: &dyn Metrics,
    max: Option<f32>,
    v: Option<crate::layout::flow::VerticalText>,
) -> Vec<RichLine> {
    #[cfg(feature = "shaping")]
    if v.is_none()
        && let Some((lines, _)) = crate::text::paragraph::rich(rc, fonts, max)
    {
        return lines;
    }
    let mut lines = Vec::new();
    for para in paragraphs(rc, fonts, v) {
        // Groups: glued units together, breakable between.
        let mut groups: Vec<Vec<&Unit>> = Vec::new();
        for u in &para {
            match groups.last_mut() {
                Some(g) if u.glued => g.push(u),
                _ => groups.push(vec![u]),
            }
        }
        let mut line: Vec<&Unit> = Vec::new();
        let mut width = 0.0f32;
        let mut pending: Vec<&Unit> = Vec::new();
        let mut pending_w = 0.0f32;
        // The room the edges a break cuts take at the line's start.
        let mut start = (0, 0.0);
        let finish = |line: &mut Vec<&Unit>, lines: &mut Vec<RichLine>, start, end| {
            let mut l = build_line(rc, fonts, std::mem::take(line));
            with_room(&mut l, start, end, false);
            lines.push(l);
        };
        let mut any = false;
        for (gi, g) in groups.iter().enumerate() {
            if g[0].space {
                pending.extend(g.iter().copied());
                pending_w += g.iter().map(|u| u.width).sum::<f32>();
                continue;
            }
            let gw: f32 = g.iter().map(|u| u.width).sum();
            // The room a break after it would hold back for the edges it
            // cuts.
            let last = g[g.len() - 1].span;
            let after = groups[gi + 1..]
                .iter()
                .find(|n| !n[0].space)
                .map_or(0.0, |n| cut_room(rc, last, n[0].span)[0]);
            if let Some(max) = max
                && !line.is_empty()
                && width + pending_w + gw + after > max
            {
                let last = line[line.len() - 1].span;
                let [end, next] = cut_room(rc, last, g[0].span);
                finish(&mut line, &mut lines, start, (last, end));
                start = (g[0].span, next);
                any = true;
                width = next;
            } else if !line.is_empty() {
                line.extend(pending.iter().copied());
                width += pending_w;
            }
            pending.clear();
            pending_w = 0.0;
            line.extend(g.iter().copied());
            width += gw;
        }
        if !line.is_empty() || !any {
            finish(&mut line, &mut lines, start, (0, 0.0));
        }
    }
    if v.is_none() {
        crate::text::text_box::apply(rc, fonts, &mut lines);
    }
    lines
}

fn build_line(rc: &RichContent, fonts: &dyn Metrics, units: Vec<&Unit>) -> RichLine {
    let mut pieces: Vec<Piece> = Vec::new();
    let mut x = 0.0;
    for u in units {
        match pieces.last_mut() {
            // Room is a piece of its own: no text is drawn in it.
            Some(p)
                if p.span == u.span
                    && rc.spans[u.span].atom.is_none()
                    && !p.text.is_empty()
                    && !u.text.is_empty() =>
            {
                p.text.push_str(&u.text);
                p.width += u.width;
            }
            _ => pieces.push(Piece {
                span: u.span,
                text: u.text.clone(),
                x,
                width: u.width,
                rise: 0.0,
            }),
        }
        x += u.width;
    }
    let (h, baseline) = place_line(rc, fonts, &mut pieces);
    let mut line = RichLine {
        width: x,
        indent: 0.0,
        height: h,
        baseline,
        pieces,
        shaped: None,
    };
    emphasis::make_room(rc, fonts, &mut line);
    line
}

/// A line's height and baseline from its pieces' spans, and each piece's
/// rise, where Parley cannot shape them. The baseline-aligned spans share
/// one baseline, their extents centred in the tallest line height. A span
/// with another `vertical-align` sits against the paragraph's text (CSS
/// 2.1 §10.8.1, half its leading above and below it). Every line holds
/// the paragraph's own line (its strut) and grows to hold its spans.
pub(crate) fn place_line(
    rc: &RichContent,
    fonts: &dyn Metrics,
    pieces: &mut [Piece],
) -> (f32, f32) {
    let (mut h, mut a, mut d) = (0.0f32, 0.0f32, 0.0f32);
    for p in pieces.iter() {
        let s = &rc.spans[p.span];
        if !s.valign.is_baseline(rc.dominant) {
            continue;
        }
        let (lh, asc, desc) = span_metrics(s, fonts);
        h = h.max(lh);
        a = a.max(asc);
        d = d.min(desc);
    }
    if pieces.is_empty()
        && let Some(s) = rc.spans.first()
    {
        h = s.step.line_height;
    }
    let baseline = (h - (a - d)) / 2.0 + a;
    let (step, font) = rc.base;
    let m = fonts.metrics_styled(font, crate::FontQuery::new(step.weight, false), "");
    let strut = Strut::of(m, step.size, rc.dominant);
    let lead = (step.line_height - strut.ascent - strut.descent) / 2.0;
    let base = (
        baseline.max(strut.ascent + lead),
        (h - baseline).max(strut.descent + lead),
    );
    let moving: Vec<usize> = (0..pieces.len())
        .filter(|&k| !rc.spans[pieces[k].span].valign.is_baseline(rc.dominant))
        .collect();
    let items: Vec<(InlinePlace, f32, f32)> = moving
        .iter()
        .map(|&k| {
            let s = &rc.spans[pieces[k].span];
            let (lh, asc, desc) = span_metrics(s, fonts);
            let lead = (lh - (asc - desc)) / 2.0;
            let (above, below) = (asc + lead, lead - desc);
            (
                s.valign.place(
                    &strut,
                    &crate::layout::baselines_of(fonts.metrics_styled(s.font, s.query(), &s.text))
                        .scaled(s.step.size),
                    s.step.size,
                    lh,
                    above,
                    below,
                ),
                above,
                below,
            )
        })
        .collect();
    let (height, top, at) = crate::layout::line_box(base, &items);
    for (&k, at) in moving.iter().zip(at) {
        pieces[k].rise = top - at;
    }
    (height, top)
}

/// A piece as drawn: its span, its text (logical), the visual glyphs of a
/// right-to-left or shaped segment (`None`: drawn as written, kerned by the
/// renderer) and where it sits.
pub(crate) struct DrawnPiece {
    pub span: usize,
    pub text: String,
    /// Its first character among the node's marks: its lines' pieces'
    /// text, each line ended by one break ([`crate::ui::Tree::text_source`]).
    /// A glyph's `index` counts from it.
    pub at: usize,
    pub glyphs: Option<Vec<crate::text::VisualGlyph>>,
    pub rect: Rect,
    pub baseline: f32,
    /// Its line, in logical order.
    pub line: usize,
}

/// The smallest rectangle holding `a` and `b`.
fn union(a: Rect, b: Rect) -> Rect {
    let (x0, y0) = (a.x.min(b.x), a.y.min(b.y));
    Rect::new(
        x0,
        y0,
        a.right().max(b.right()) - x0,
        a.bottom().max(b.bottom()) - y0,
    )
}

/// One segment of a line in visual order: a piece, or the part of one in a
/// single bidi run; its width in CSS px at scale 1.
#[derive(Clone, Debug, PartialEq)]
pub struct Segment {
    pub span: usize,
    pub text: String,
    /// Its first character, counted over its line's pieces' text; its
    /// glyphs' `index` counts from it.
    pub at: usize,
    pub glyphs: Option<Vec<crate::text::VisualGlyph>>,
    pub width: f32,
    /// Its piece's rise above the line's baseline.
    pub rise: f32,
}

/// A rich line's segments in visual order. A line of left-to-right text in
/// a left-to-right UI is its pieces as laid out; any other line runs the
/// bidi algorithm over its whole text (atoms as U+FFFC) at the UI's
/// direction and cuts the runs at the pieces' edges.
fn visual_segments(
    rc: &RichContent,
    fonts: &dyn Metrics,
    line: &RichLine,
    direction: og_i18n::Direction,
) -> Vec<Segment> {
    if let Some(shaped) = &line.shaped {
        return shaped.clone();
    }
    let mixed = |p: &Piece| {
        let s = &rc.spans[p.span];
        s.atom.is_none() && fonts.metrics_styled(s.font, s.query(), &p.text).is_mixed()
    };
    // The line's paragraph direction is its first strong character's (the
    // UI's with none, and skipping isolates; left to right for a line with
    // no letter), as a plain text's is: an English line in a right-to-left
    // UI keeps its full stop at its end.
    let base = crate::text::line_direction(
        &line
            .pieces
            .iter()
            .filter(|p| rc.spans[p.span].atom.is_none())
            .map(|p| p.text.as_str())
            .collect::<String>(),
        direction,
    );
    let plain = base == og_i18n::Direction::Ltr
        && line
            .pieces
            .iter()
            .all(|p| !crate::text::needs_visual(&p.text) && !mixed(p));
    if plain {
        let mut at = 0;
        return line
            .pieces
            .iter()
            .map(|p| {
                let s = Segment {
                    span: p.span,
                    text: p.text.clone(),
                    at,
                    glyphs: None,
                    width: p.width,
                    rise: p.rise,
                };
                at += p.text.chars().count();
                s
            })
            .collect();
    }
    // The line's text and where each piece's text lies in it.
    let mut text = String::new();
    let mut ranges = Vec::with_capacity(line.pieces.len());
    // Each piece's first character, counted over the pieces' text.
    let mut starts = Vec::with_capacity(line.pieces.len());
    let mut chars = 0;
    for p in &line.pieces {
        starts.push(chars);
        chars += p.text.chars().count();
        let start = text.len();
        // An atom, or an inline element's room, is one object of the line.
        if rc.spans[p.span].atom.is_some() || p.text.is_empty() {
            text.push('\u{fffc}');
        } else {
            text.push_str(&p.text);
        }
        ranges.push(start..text.len());
    }
    let mut out = Vec::new();
    for (run, rtl) in crate::text::bidi_runs(&text, base) {
        // The pieces this run crosses, in visual order.
        let mut parts: Vec<(usize, std::ops::Range<usize>)> = ranges
            .iter()
            .enumerate()
            .filter_map(|(k, r)| {
                let a = r.start.max(run.start);
                let b = r.end.min(run.end);
                (a < b).then_some((k, a..b))
            })
            .collect();
        if rtl {
            parts.reverse();
        }
        for (k, r) in parts {
            let p = &line.pieces[k];
            let s = &rc.spans[p.span];
            let at = starts[k] + text[ranges[k].start..r.start].chars().count();
            if s.atom.is_some() || p.text.is_empty() {
                out.push(Segment {
                    span: p.span,
                    text: String::new(),
                    at,
                    glyphs: None,
                    width: p.width,
                    rise: p.rise,
                });
                continue;
            }
            let piece_text = &text[r.clone()];
            let m = fonts.metrics_styled(s.font, s.query(), piece_text);
            let tracking = span_tracking(s, piece_text);
            let needs = rtl || crate::text::needs_visual(piece_text) || m.is_mixed();
            if !needs {
                out.push(Segment {
                    span: p.span,
                    text: piece_text.to_string(),
                    at,
                    glyphs: None,
                    width: span_width(s, piece_text, fonts),
                    rise: p.rise,
                });
                continue;
            }
            let mut glyphs = m.run(piece_text, rtl, tracking, 0);
            glyphs.retain(|g| !og_i18n::is_bidi_control(g.ch));
            let width = glyphs.iter().map(|g| g.advance).sum::<f32>() * s.step.size;
            out.push(Segment {
                span: p.span,
                text: piece_text.to_string(),
                at,
                glyphs: Some(glyphs),
                width,
                rise: p.rise,
            });
        }
    }
    out
}

/// A sine with `+ − × ÷` only (Bhaskara I), for per-glyph motion: the UI
/// reads no transcendental functions.
pub(crate) fn sine(x: f32) -> f32 {
    let pi = std::f32::consts::PI;
    let t = x.rem_euclid(2.0 * pi);
    let (t, sign) = if t > pi { (t - pi, -1.0) } else { (t, 1.0) };
    sign * 16.0 * t * (pi - t) / (5.0 * pi * pi - 4.0 * t * (pi - t))
}

impl Ui {
    /// What the links of the rich node keyed `key` did this tick.
    pub fn respond_links(&self, key: u64, links: &[String]) -> RichTextResponse {
        let mut out = RichTextResponse::default();
        for l in links {
            let r = self.respond(child_key(key, l));
            if r.clicked && out.link.is_none() {
                out.link = Some(l.clone());
            }
            if r.hovered && out.hovered.is_none() {
                out.hovered = Some(l.clone());
            }
            out.response.hovered |= r.hovered;
            out.response.pressed |= r.pressed;
            out.response.clicked |= r.clicked;
            out.response.focused |= r.focused;
        }
        out
    }

    /// Where each piece of rich node `i` sits on screen, after layout and
    /// motion, in visual order: each line is laid out by the bidirectional
    /// algorithm over its whole text at its paragraph direction (its first
    /// strong character's, the UI's with none: UAX #9), so
    /// pieces are reordered and so are runs of either direction within one
    /// piece (a piece split by a direction change is several segments).
    pub(crate) fn rich_pieces(&self, i: usize) -> Vec<DrawnPiece> {
        let n = &self.tree.nodes[i];
        let Content::Rich(rc) = &*n.content else {
            return Vec::new();
        };
        let sk = n.placed.scale;
        let inner = self.tree.rich_inner(i);
        if let Some(v) = self.tree.rich_vertical(i) {
            return self
                .vertical_rich_pieces(i, rc, v)
                .0
                .into_iter()
                .map(|(_, p)| p)
                .collect();
        }
        let rtl = self.env.direction == og_i18n::Direction::Rtl;
        let mut out = Vec::new();
        let mut line_y = inner.y;
        let mut first = 0;
        for (k, line) in n.layout.rich_lines.iter().enumerate() {
            let segments = visual_segments(rc, &self.env.fonts, line, self.env.direction);
            let width: f32 = segments.iter().map(|s| s.width).sum::<f32>() + line.indent;
            // Its line box: the room floats leave it (`Node::wrap_lines`,
            // its drop below the lines above), or its box.
            let (bx, bw, dy) = match n.layout.wrap_lines.get(k) {
                Some(&[dx, dy, w]) if rtl => {
                    (inner.x + inner.width - (dx + w) * sk, w * sk, dy * sk)
                }
                Some(&[dx, dy, w]) => (inner.x + dx * sk, w * sk, dy * sk),
                None => (inner.x, inner.width, 0.0),
            };
            let y = line_y + dy;
            let free = bw - width * sk;
            let x0 = bx
                + match (rc.align, rtl) {
                    (TextAlign::Start | TextAlign::Justify, false) | (TextAlign::End, true) => 0.0,
                    (TextAlign::Center, _) => free / 2.0,
                    (TextAlign::End, false) | (TextAlign::Start | TextAlign::Justify, true) => free,
                };
            // The indent at its inline start: before the pieces, or after
            // them in a right-to-left line set from the right.
            let mut x = x0 + if rtl { 0.0 } else { line.indent * sk };
            for seg in segments {
                out.push(DrawnPiece {
                    span: seg.span,
                    text: seg.text,
                    at: first + seg.at,
                    glyphs: seg.glyphs,
                    rect: Rect::new(x, y, seg.width * sk, line.height * sk),
                    baseline: y + (line.baseline - seg.rise) * sk,
                    line: k,
                });
                x += seg.width * sk;
            }
            first += line
                .pieces
                .iter()
                .map(|p| p.text.chars().count())
                .sum::<usize>()
                + 1;
            line_y += line.height * sk;
        }
        out
    }

    /// The fragments of rich node `i` whose `pieces` (its
    /// [`Ui::rich_pieces`]) are of the spans `owns` picks: on each line
    /// with one, in logical order, the union of their content areas (their
    /// fonts' ascent and descent about their baseline; down a vertical
    /// line, their pieces).
    pub(crate) fn rich_frags(
        &self,
        i: usize,
        pieces: &[DrawnPiece],
        owns: impl Fn(&RichSpan) -> bool,
    ) -> Vec<Rect> {
        let n = &self.tree.nodes[i];
        let Content::Rich(rc) = &*n.content else {
            return Vec::new();
        };
        let vertical = self.tree.rich_vertical(i).is_some();
        let sk = n.placed.scale;
        let mut out: Vec<(usize, Rect)> = Vec::new();
        for p in pieces {
            let s = &rc.spans[p.span];
            if !owns(s) {
                continue;
            }
            // Down a vertical line, its em box about the line's centre.
            let r = if vertical {
                let em = s.step.size * sk;
                Rect::new(p.baseline - em / 2.0, p.rect.y, em, p.rect.height)
            } else {
                let (_, a, d) = span_metrics(s, &self.env.fonts);
                Rect::new(p.rect.x, p.baseline - a * sk, p.rect.width, (a - d) * sk)
            };
            match out.iter_mut().find(|(line, _)| *line == p.line) {
                Some((_, u)) => *u = union(*u, r),
                None => out.push((p.line, r)),
            }
        }
        out.sort_by_key(|(line, _)| *line);
        out.into_iter().map(|(_, r)| r).collect()
    }

    /// The inline element nodes of each document text among `order` on
    /// their fragments: each one's `inline_frags` (border boxes: its words'
    /// content area grown by its border and padding across the line, and
    /// at its start and end on the fragments that hold them), and its box
    /// their union (an empty box at its text's corner when it has no words
    /// on a line). Returns the nodes it placed, to map.
    pub(crate) fn place_inline_fragments(&mut self, order: &[usize]) -> Vec<usize> {
        let mut placed = Vec::new();
        for &text in order {
            let n = &self.tree.nodes[text];
            if !matches!(&*n.content, Content::Rich(_))
                || !n
                    .ordered_children
                    .iter()
                    .any(|&c| self.tree.kind_of(c) == "inline")
            {
                continue;
            }
            let pieces = self.rich_pieces(text);
            let corner = n.placed.rect;
            let mut stack: Vec<usize> = n.ordered_children.clone();
            while let Some(i) = stack.pop() {
                if self.tree.kind_of(i) != "inline" {
                    continue;
                }
                let key = self.tree.nodes[i].key;
                let mut rects = self.rich_frags(text, &pieces, |s| s.owners.contains(&key));
                let last = rects.len().saturating_sub(1);
                self.border_boxes(text, &pieces, key, &mut rects);
                let n = &mut self.tree.nodes[i];
                n.placed.rect = rects
                    .iter()
                    .copied()
                    .reduce(union)
                    .unwrap_or(Rect::new(corner.x, corner.y, 0.0, 0.0));
                n.layout.inline_frags = rects
                    .into_iter()
                    .enumerate()
                    .map(|(k, rect)| crate::layout::InlineFrag {
                        rect,
                        start: k == 0,
                        end: k == last,
                    })
                    .collect();
                stack.extend_from_slice(&n.ordered_children);
                placed.push(i);
            }
        }
        placed
    }

    /// Inline element `key`'s fragments `rects` of rich node `i` (its
    /// words' content area on each line, about its edges' room) as its
    /// border boxes ([`InlineEdges`]): out by its border and padding across
    /// the line, in from its first and last piece by the room before its
    /// edge (its margin, an outer element's edge there).
    fn border_boxes(&self, i: usize, pieces: &[DrawnPiece], key: u64, rects: &mut [Rect]) {
        let n = &self.tree.nodes[i];
        let Content::Rich(rc) = &*n.content else {
            return;
        };
        let sk = n.placed.scale;
        let at = |k: usize| {
            let s = &rc.spans[k];
            let j = s.owners.iter().position(|&o| o == key)?;
            Some((s.edges.get(j).copied()?, &s.edges[..j]))
        };
        let first = (0..rc.spans.len()).find_map(|k| Some((k, at(k)?)));
        let last = (0..rc.spans.len()).rev().find_map(|k| Some((k, at(k)?)));
        let (Some((fk, (e, outer))), Some((lk, (end, outer_end)))) = (first, last) else {
            return;
        };
        let into = [
            outer.iter().map(|o| o.room()[0]).sum::<f32>() + e.start[0],
            outer_end.iter().map(|o| o.room()[1]).sum::<f32>() + end.end[0],
        ]
        .map(|v| v * sk);
        // Under `clone` each fragment holds both edges: at a break, inside
        // the room of the cloned elements around it (CSS Fragmentation 3
        // §5.4).
        let cut = |side: fn(&InlineEdges) -> [f32; 2], own: f32| {
            let outer: f32 = outer
                .iter()
                .filter(|o| o.clone)
                .map(|o| side(o)[0] + side(o)[1])
                .sum();
            (outer + own) * sk
        };
        let cut = [cut(|o| o.start, e.start[0]), cut(|o| o.end, end.end[0])];
        let across = e.across.map(|v| v * sk);
        // The side its start is on: the right of a right-to-left run (an
        // atom's: the UI's direction), the bottom of a line set from it.
        let vertical = self.tree.rich_vertical(i);
        let reversed = |k: usize| match vertical {
            Some(v) => self.rich_from_bottom(i, v),
            None => pieces
                .iter()
                .find(|p| p.span == k)
                .and_then(|p| p.glyphs.as_ref()?.first().map(|g| g.rtl))
                .unwrap_or(self.env.direction == og_i18n::Direction::Rtl),
        };
        let count = rects.len();
        for (f, r) in rects.iter_mut().enumerate() {
            // Its inline axis: [from, to] along it.
            let (mut from, mut to) = match vertical {
                Some(_) => (r.y, r.bottom()),
                None => (r.x, r.right()),
            };
            let (first, last) = (f == 0, f + 1 == count);
            for (holds, k, inset, end) in [
                (
                    first || e.clone,
                    fk,
                    if first { into[0] } else { cut[0] },
                    false,
                ),
                (
                    last || e.clone,
                    lk,
                    if last { into[1] } else { cut[1] },
                    true,
                ),
            ] {
                if !holds {
                    continue;
                }
                if reversed(k) != end {
                    to -= inset;
                } else {
                    from += inset;
                }
            }
            *r = match vertical {
                Some(_) => Rect::new(
                    r.x - across[0],
                    from,
                    r.width + across[0] + across[1],
                    to - from,
                ),
                None => Rect::new(
                    from,
                    r.y - across[0],
                    to - from,
                    r.height + across[0] + across[1],
                ),
            };
        }
    }

    /// The fragments of each link of rich node `i`, in logical order.
    pub(crate) fn rich_link_frags(&self, i: usize) -> Vec<(String, Vec<Rect>)> {
        let pieces = self.rich_pieces(i);
        self.tree.nodes[i]
            .layout
            .links
            .iter()
            .map(|l| {
                let frags = self.rich_frags(i, &pieces, |s| s.link.as_deref() == Some(l.as_str()));
                (l.clone(), frags)
            })
            .collect()
    }

    /// The rectangle of each link of rich node `i` (the union of its pieces).
    pub(crate) fn rich_links(&self, i: usize) -> Vec<(String, Rect)> {
        let n = &self.tree.nodes[i];
        let Content::Rich(rc) = &*n.content else {
            return Vec::new();
        };
        let pieces = self.rich_pieces(i);
        n.layout
            .links
            .iter()
            .filter_map(|l| {
                let rects: Vec<Rect> = pieces
                    .iter()
                    .filter(|p| rc.spans[p.span].link.as_deref() == Some(l.as_str()))
                    .map(|p| p.rect)
                    .collect();
                let first = rects.first()?;
                let (mut x0, mut y0, mut x1, mut y1) =
                    (first.x, first.y, first.right(), first.bottom());
                for r in &rects {
                    x0 = x0.min(r.x);
                    y0 = y0.min(r.y);
                    x1 = x1.max(r.right());
                    y1 = y1.max(r.bottom());
                }
                Some((l.clone(), Rect::new(x0, y0, x1 - x0, y1 - y0)))
            })
            .collect()
    }

    /// The boxes of text `i`'s inline elements (`Node::inline_decor`),
    /// under its words, an outer element's under the elements in it (CSS
    /// 2 Appendix E): each drawn on its fragments ([`Ui::decor_pieces`]).
    pub(crate) fn inline_box_commands(&self, i: usize, cmds: &mut Vec<crate::draw::DrawCmd>) {
        let n = &self.tree.nodes[i];
        let t = self.paint_theme();
        let clip = self.paint_clip(n);
        let alpha = n.placed.alpha;
        // Its inline elements in document order.
        let mut stack: Vec<usize> = n.ordered_children.iter().rev().copied().collect();
        while let Some(j) = stack.pop() {
            if self.tree.kind_of(j) != "inline" {
                continue;
            }
            let e = &self.tree.nodes[j];
            stack.extend(e.ordered_children.iter().rev());
            let Some(decor) = e.inline_decor.as_deref() else {
                continue;
            };
            for p in self.decor_pieces(j, e.placed.rect) {
                let at = crate::paint::Placed {
                    rect: p.at,
                    radius: 0.0,
                    clip,
                    alpha,
                };
                let d = decor.over(
                    None,
                    false,
                    crate::css::paint_css::LayerFrame {
                        size: [p.at.width, p.at.height],
                        viewport: [self.env.viewport.width, self.env.viewport.height],
                        ..Default::default()
                    },
                );
                cmds.extend(p.cut.map(crate::ui::cut_to));
                cmds.extend(d.enter(t, &at));
                cmds.extend(d.leave(t, &at));
                cmds.extend(decor.lines(&at));
                cmds.extend(p.cut.map(|_| crate::draw::DrawCmd::PopClip));
            }
        }
    }

    /// The draw commands of rich node `i`: each piece in its span's font,
    /// ink, weight and tracking (a moving span glyph by glyph), icons from
    /// the atlas, links underlined (and ringed while focused by keys).
    pub(crate) fn rich_commands(
        &self,
        i: usize,
        rc: &RichContent,
        cmds: &mut Vec<crate::draw::DrawCmd>,
    ) {
        use crate::draw::DrawCmd;
        let n = &self.tree.nodes[i];
        let t = self.paint_theme();
        let p = &t.palette;
        let alpha = n.placed.alpha;
        let dpr = self.env.viewport.dpr();
        let snap = |v: f32| crate::layout::snap(v, dpr);
        let clip = if n.decor.as_ref().is_some_and(|d| d.clip) {
            self.paint_clip(n).intersect(&n.placed.rect)
        } else {
            self.paint_clip(n)
        };
        let tick = self.interaction.state.tick as f32;
        let still = t.reduced_motion;
        self.inline_box_commands(i, cmds);
        // A link focused by keys or a pad: the focus ring around it, under the text.
        if self.interaction.state.focus_visible {
            for (l, r) in self.rich_links(i) {
                if self.interaction.state.focus == child_key(n.key, &l) {
                    let w = t.space.xxs;
                    cmds.push(DrawCmd::Quad {
                        rect: r.outset(w * 2.0),
                        radius: w * 2.0,
                        color: p.focus.fade(alpha).linear(),
                        clip,
                    });
                    cmds.push(DrawCmd::Quad {
                        rect: r.outset(w),
                        radius: w,
                        color: self.sampled_behind(i).fade(alpha).linear(),
                        clip,
                    });
                }
            }
        }
        if let Some(v) = self.tree.rich_vertical(i) {
            self.vertical_rich_commands(i, rc, v, cmds);
            return;
        }
        let pieces = self.rich_pieces(i);
        let edges = line_edges(&pieces);
        for (piece, (line_start, line_end)) in pieces.into_iter().zip(edges) {
            let (k, text, r, baseline) = (piece.span, piece.text, piece.rect, piece.baseline);
            let s = &rc.spans[k];
            let size = s.step.size * n.placed.scale;
            // An atom is drawn inside its owners' edges.
            let r = if s.atom.is_some() {
                let [before, after] = s.room().map(|e| e * n.placed.scale);
                let left = if self.env.direction == og_i18n::Direction::Rtl {
                    after
                } else {
                    before
                };
                Rect::new(r.x + left, r.y, r.width - before - after, r.height)
            } else {
                r
            };
            let link_key = s.link.as_deref().map(|l| child_key(n.key, l));
            // A document's inline `<a>` takes its hover from its CSS (`a:hover`).
            let hot =
                s.owners.is_empty() && link_key.is_some_and(|lk| self.interaction.state.hot == lk);
            let color = if hot { p.primary_hover } else { s.color };
            if let Some(Atom::Ruby(ruby)) = &s.atom {
                let at = ruby::RubyAt {
                    rect: r,
                    baseline,
                    color,
                };
                self.ruby_commands(i, s, ruby, at, None, cmds);
                continue;
            }
            if let Some(Atom::Image(im)) = &s.atom {
                // A picture in its own colours, one em tall on the baseline.
                let h = s.step.size * n.placed.scale;
                cmds.push(DrawCmd::Image {
                    rect: Rect::new(
                        r.x + h * 0.05,
                        baseline - h * 0.85,
                        h * im.aspect.max(0.05),
                        h,
                    )
                    .snapped(dpr),
                    texture: Some(im.texture),
                    uv: im.uv,
                    color: Srgba::WHITE.fade(alpha).linear(),
                    clip,
                });
                continue;
            }
            if let Some(Atom::Cap(label)) = &s.atom {
                // A key cap: a soft rounded plate in the ink, its name on it.
                let em = s.step.size * n.placed.scale;
                let (m, pad) = (CAP_MARGIN * em, CAP_PAD * em);
                let plate = Rect::new(
                    r.x + m,
                    baseline - em * 0.82,
                    (r.width - 2.0 * m).max(0.0),
                    em * 1.06,
                );
                cmds.push(DrawCmd::Quad {
                    rect: plate.snapped(dpr),
                    radius: em * 0.22,
                    color: color.fade(alpha * 0.16).linear(),
                    clip,
                });
                let font = self.env.fonts.for_text(s.font, label);
                if let Some(handle) = font.handle {
                    let size = em * CAP_TEXT;
                    let w = font.metrics.measure_tracked(label, size, 0.0);
                    cmds.push(DrawCmd::Text {
                        source: 0,
                        x: snap(r.x + m + pad),
                        baseline: snap(baseline - em * 0.02),
                        size,
                        text: label.clone(),
                        color: color.fade(alpha).linear(),
                        font: handle,
                        bounds: Rect::new(r.x + m + pad, plate.y, w, plate.height),
                        clip,
                        tracking: 0.0,
                        weight: 0.0,
                        glow: None,
                        placements: Vec::new(),
                    });
                }
                continue;
            }
            if let Some(Atom::Icon(cell)) = &s.atom {
                let side = s.step.size * n.placed.scale;
                match *cell {
                    crate::ui::IconCell::Builtin(name) => {
                        let rect = Rect::new(r.x + side * 0.05, baseline - side * 0.85, side, side);
                        cmds.extend(crate::default_icons::commands(
                            name,
                            rect,
                            clip,
                            color.fade(alpha),
                            self.paint_theme(),
                        ));
                    }
                    crate::ui::IconCell::Glyph(font, ch) => {
                        // An icon set's glyph: its em square where a cell would be.
                        let (x, top) = (r.x + side * 0.05, baseline - side * 0.85);
                        cmds.push(DrawCmd::Text {
                            source: 0,
                            x,
                            baseline: top + side,
                            size: side,
                            text: ch.to_string(),
                            color: color.fade(alpha).linear(),
                            font,
                            bounds: Rect::new(x, top, side, side),
                            clip,
                            tracking: 0.0,
                            weight: 0.0,
                            glow: None,
                            placements: Vec::new(),
                        });
                    }
                    crate::ui::IconCell::Image(tex, uv) => {
                        cmds.push(DrawCmd::Image {
                            rect: Rect::new(r.x + side * 0.05, baseline - side * 0.85, side, side)
                                .snapped(dpr),
                            texture: Some(tex),
                            uv,
                            color: color.fade(alpha).linear(),
                            clip,
                        });
                    }
                }
                continue;
            }
            // An inline element's room: nothing to draw.
            if text.is_empty() {
                continue;
            }
            let font = self.env.fonts.for_styled(s.font, s.query(), &text).0;
            let Some(handle) = font.handle else { continue };
            // Emphasis marks over (or under) each cluster's middle.
            if let Some(e) = &rc.emphasis {
                let tracking = span_tracking(s, &text);
                let xs = match &piece.glyphs {
                    Some(glyphs) => {
                        let mut pen = 0.0;
                        emphasis::spots(
                            &text,
                            glyphs
                                .iter()
                                .map(|g| {
                                    let x0 = pen;
                                    pen += g.advance * size;
                                    (g.index, x0, pen)
                                })
                                .collect::<Vec<_>>(),
                        )
                    }
                    None => emphasis::spots(
                        &text,
                        crate::edit::boxes(
                            &font.metrics,
                            &text,
                            og_i18n::Direction::Ltr,
                            size,
                            tracking,
                        )
                        .into_iter()
                        .map(|g| (g.index, g.x0, g.x1)),
                    ),
                };
                let (a, d) = ruby::content_edges(&font.metrics, &text);
                let y = if e.under {
                    baseline + d * size + size / 4.0
                } else {
                    baseline - a * size - size / 4.0
                };
                let centres: Vec<(f32, f32)> = xs.into_iter().map(|x| (r.x + x, y)).collect();
                let ink = e.color.unwrap_or(color).fade(alpha).linear();
                emphasis::marks(e, false, &centres, size, font, ink, clip, cmds);
            }
            let first = cmds.len();
            let mixed = font.metrics.is_mixed();
            let m = &font.metrics;
            let tracking = span_tracking(s, &text);
            // The glyphs its decoration lines break around (CSS Text
            // Decoration 4 §2.10.3, §2.10.4), as the piece draws them.
            let ruled = link_key.is_some()
                || s.underline
                || s.strike
                || s.decoration.as_ref().is_some_and(|d| d.overline);
            let line_glyphs = ruled.then(|| {
                piece
                    .glyphs
                    .clone()
                    .unwrap_or_else(|| m.visual(&text, og_i18n::Direction::Ltr, tracking))
            });
            // Bold: thickened in the shader (as far as the field reaches)
            // and drawn twice, the second copy one device pixel to the
            // right, so small text whose field is thin is bold too: a
            // device pixel in the text's units (an SVG's text is laid out
            // in its view box's and scaled).
            let w = &n.placed.world;
            let device = dpr * (w.a * w.d - w.b * w.c).abs().sqrt().max(1e-3);
            let (weight, smear) = if s.synth.bold {
                synthetic_bold(size, device)
            } else {
                (0.0, 0.0)
            };
            // A face drawn at its own size (`font-size-adjust`).
            let scale = m.scale;
            let glyph = |x: f32,
                         dy: f32,
                         a: f32,
                         text: String,
                         mut placements: Vec<og_draw::GlyphPlacement>,
                         w: f32| {
                crate::text::unscale(&mut placements, scale);
                DrawCmd::Text {
                    source: 0,
                    x: snap(x),
                    baseline: snap(baseline + dy),
                    size: size * scale,
                    text,
                    color: color.fade(alpha * a).linear(),
                    font: handle,
                    bounds: Rect::new(x, r.y + dy, w, r.height),
                    clip,
                    tracking: tracking / scale,
                    weight,
                    glow: None,
                    placements,
                }
            };
            // A span's effect at this tick for its `g`-th glyph: (rise, alpha).
            let effect = |g: usize| match s.fx {
                SpanFx::Wave => (-0.14 * size * sine(tick * 0.16 + g as f32 * 0.7), 1.0),
                SpanFx::Shake => {
                    // A fixed hash of the glyph and the tick's half: jitter, no clock.
                    let h = (g as u32).wrapping_mul(2_654_435_761)
                        ^ ((tick as u32) / 2).wrapping_mul(40_503);
                    (((h % 100) as f32 / 100.0 - 0.5) * 0.12 * size, 1.0)
                }
                _ => (0.0, 0.7 + 0.3 * sine(tick * 0.1)),
            };
            let moving = s.fx != SpanFx::None && !still;
            if let Some(glyphs) = piece.glyphs.as_ref().filter(|_| moving) {
                // A moving span laid out glyph by glyph (it mixes faces, runs
                // right to left or is shaped): each glyph in its own face,
                // moved by the effect; a shaped run's joined glyphs move as
                // one (their joins would break apart).
                let shaped = |g: &crate::text::VisualGlyph| g.glyph.is_some();
                let mut x = r.x;
                for (g, run) in glyphs
                    .chunk_by(|a, b| a.font == b.font && shaped(a) && shaped(b))
                    .enumerate()
                {
                    let w = run.iter().map(|v| v.advance).sum::<f32>() * size;
                    let Some(f) = font.handle_of(run[0].font) else {
                        x += w;
                        continue;
                    };
                    let scale = font.scale_of(run[0].font);
                    let (text, mut placements) = crate::text::FontMetrics::drawn(run);
                    crate::text::unscale(&mut placements, scale);
                    let (dy, a) = effect(g);
                    let copies = if smear > 0.0 {
                        &[smear, 0.0][..]
                    } else {
                        &[0.0][..]
                    };
                    for &dx in copies {
                        cmds.push(DrawCmd::Text {
                            source: 0,
                            x: snap(x + dx),
                            baseline: snap(baseline + dy),
                            size: size * scale,
                            text: text.clone(),
                            color: color.fade(alpha * a).linear(),
                            font: f,
                            bounds: Rect::new(x + dx, r.y + dy, w, r.height),
                            clip,
                            tracking: tracking / scale,
                            weight,
                            glow: None,
                            placements: placements.clone(),
                        });
                    }
                    x += w;
                }
            } else if let Some(glyphs) = piece.glyphs.as_ref().filter(|_| mixed) {
                // A segment mixing fonts: a command per font, each where its
                // glyphs start.
                for pc in crate::text::split(font, glyphs) {
                    let Some(f) = pc.font else { continue };
                    // Bold: a copy one device pixel to the right first.
                    let copies = if smear > 0.0 {
                        &[smear, 0.0][..]
                    } else {
                        &[0.0][..]
                    };
                    for &dx in copies {
                        cmds.push(DrawCmd::Text {
                            source: 0,
                            x: snap(r.x + dx) + pc.dx * size,
                            baseline: snap(baseline),
                            size: size * pc.scale,
                            text: pc.text.clone(),
                            color: color.fade(alpha).linear(),
                            font: f,
                            bounds: Rect::new(
                                r.x + dx + pc.dx * size,
                                r.y,
                                pc.width * size,
                                r.height,
                            ),
                            clip,
                            tracking: tracking / pc.scale,
                            weight,
                            glow: None,
                            placements: pc.placements.clone(),
                        });
                    }
                }
            } else if let Some(glyphs) = piece.glyphs {
                // A right-to-left or shaped segment: drawn whole in visual
                // order with its placements (its joins would break glyph by glyph).
                let (drawn, placements) = crate::text::FontMetrics::drawn(&glyphs);
                if smear > 0.0 {
                    cmds.push(glyph(
                        r.x + smear,
                        0.0,
                        1.0,
                        drawn.clone(),
                        placements.clone(),
                        r.width,
                    ));
                }
                cmds.push(glyph(r.x, 0.0, 1.0, drawn, placements, r.width));
            } else if s.fx == SpanFx::None || still {
                if smear > 0.0 {
                    cmds.push(glyph(
                        r.x + smear,
                        0.0,
                        1.0,
                        text.clone(),
                        Vec::new(),
                        r.width,
                    ));
                }
                cmds.push(glyph(r.x, 0.0, 1.0, text.clone(), Vec::new(), r.width));
            } else {
                // Glyph by glyph: each moved by the span's effect at this tick.
                let mut x = r.x;
                let chars: Vec<char> = text.chars().collect();
                for (g, &c) in chars.iter().enumerate() {
                    let cs = c.to_string();
                    let w = m.measure_tracked(&cs, size, tracking)
                        + chars.get(g + 1).map_or(0.0, |&nx| m.kern(c, nx) * size);
                    let (dy, a) = effect(g);
                    if smear > 0.0 {
                        cmds.push(glyph(x + smear, dy, a, cs.clone(), Vec::new(), w));
                    }
                    cmds.push(glyph(x, dy, a, cs, Vec::new(), w));
                    x += w;
                }
            }
            // Italic and outline: the piece's glyphs slanted about their
            // baseline and outlined from the distance field.
            if s.synth.italic || s.outline.is_some() {
                for c in &mut cmds[first..] {
                    let DrawCmd::Text { bounds, .. } = c else {
                        continue;
                    };
                    let fx = og_draw::UiTextEffects {
                        outline: s.outline.map(|(w, oc, align)| og_draw::UiTextOutline {
                            width: w * n.placed.scale,
                            color: oc.fade(alpha).linear(),
                            align,
                        }),
                        fill_box: [bounds.x, bounds.y, bounds.width, bounds.height],
                        transform: if s.synth.italic {
                            let shear = s.oblique.map_or(SLANT, |a| a.to_radians().tan());
                            [1.0, 0.0, -shear, 1.0]
                        } else {
                            [1.0, 0.0, 0.0, 1.0]
                        },
                        ..Default::default()
                    };
                    let text = std::mem::replace(c, DrawCmd::PopClip);
                    *c = DrawCmd::StyledText {
                        text: Box::new(text),
                        effects: Box::new(fx),
                    };
                }
            }
            // A link's or [u]'s underline and [s]'s strike, and CSS's
            // decoration lines (CSS Text Decoration 3 §2): across the piece
            // in its ink or the decoration's colour, in its style.
            let d = s.decoration.clone().unwrap_or_default();
            let (th, off) = d.placed(m, size, n.placed.scale, dpr);
            let ink = d.color.unwrap_or(color).fade(alpha).linear();
            // Every line skips the spacers `text-decoration-skip-spaces`
            // names; an underline and an overline also break where they
            // would cross a glyph's ink and a thickness to either side.
            let spaces = line_glyphs.as_ref().map_or_else(Vec::new, |g| {
                space_gaps(g, size, tracking, d.skip_spaces, line_start, line_end)
            });
            let mut rule = |y: f32, skips_ink: bool| {
                let from = cmds.len();
                decoration_line(r.x, r.width, y, th, d.style, ink, clip, dpr, cmds);
                let mut gaps = spaces.clone();
                if let Some(glyphs) = line_glyphs
                    .as_ref()
                    .filter(|_| skips_ink && d.skip_ink != SkipInk::None)
                {
                    gaps.extend(og_text::ink::ink_gaps(
                        font,
                        glyphs,
                        size,
                        y - baseline,
                        y - baseline + d.extent(th),
                        th.max(1.0 / dpr),
                        d.skip_ink == SkipInk::All,
                    ));
                }
                gaps.sort_by(|a, b| a.0.total_cmp(&b.0));
                let gaps: Vec<_> = gaps.iter().map(|(a, b)| (r.x + a, r.x + b)).collect();
                cut_lines(cmds, from, &gaps);
            };
            if link_key.is_some() || s.underline {
                rule(baseline + off, true);
            }
            if d.overline {
                rule(baseline - size * 0.95, true);
            }
            if s.strike {
                rule(baseline - size * 0.3, false);
            }
        }
    }
}

impl Ui {
    /// What one rich span's text is drawn with down a vertical line whose
    /// column is `col` (`lh` wide, its middle `mid`).
    #[allow(clippy::too_many_arguments)]
    fn span_run<'a>(
        &self,
        i: usize,
        s: &RichSpan,
        font: &'a crate::UiFont,
        text: &str,
        size: f32,
        v: crate::layout::flow::VerticalText,
        (mid, col, lh): (f32, f32, f32),
        draw: bool,
    ) -> crate::ui::VerticalRun<'a> {
        let (asc, desc) = font.metrics.extents(text);
        let (rtl, para) = self.text_flow(i);
        crate::ui::VerticalRun {
            font,
            size,
            tracking: span_tracking(s, text),
            features: crate::shape::Features::NONE,
            para,
            dir: if rtl {
                og_i18n::Direction::Rtl
            } else {
                og_i18n::Direction::Ltr
            },
            up_line: v.mode == crate::layout::flow::WritingMode::SidewaysLr,
            mid,
            centre: (asc + desc) / 2.0 * size,
            col,
            lh,
            space_extra: 0.0,
            draw,
        }
    }

    /// Whether a vertical rich line starts at its column's bottom (an
    /// `rtl` line, or `sideways-lr`'s, which runs up).
    fn rich_from_bottom(&self, i: usize, v: crate::layout::flow::VerticalText) -> bool {
        self.text_flow(i).0 != (v.mode == crate::layout::flow::WritingMode::SidewaysLr)
    }

    /// A vertical rich node's pieces (CSS Writing Modes 4), each with its
    /// line, and each line's column (its left edge and width): its lines,
    /// laid out in its frame, stack from its block-start side as columns,
    /// each running down (up in `sideways-lr`, from the bottom right to
    /// left) in logical order, a run lying sideways shaped in its own
    /// direction as it is drawn; each piece's rect is its stretch of its
    /// column, its `baseline` the column's middle (the central baseline)
    /// moved by its rise.
    #[allow(clippy::type_complexity)]
    fn vertical_rich_pieces(
        &self,
        i: usize,
        rc: &RichContent,
        v: crate::layout::flow::VerticalText,
    ) -> (Vec<(usize, DrawnPiece)>, Vec<(f32, f32)>) {
        let n = &self.tree.nodes[i];
        let sk = n.placed.scale;
        let inner = self.tree.rich_inner(i);
        let from_bottom = self.rich_from_bottom(i, v);
        let mut out = Vec::new();
        let mut cols = Vec::with_capacity(n.layout.rich_lines.len());
        let mut line_at = 0.0;
        let mut first = 0;
        for (k, line) in n.layout.rich_lines.iter().enumerate() {
            let line_first = first;
            first += line
                .pieces
                .iter()
                .map(|p| p.text.chars().count())
                .sum::<usize>()
                + 1;
            let segments: Vec<Segment> = line
                .pieces
                .iter()
                .scan(0, |at, p| {
                    *at += p.text.chars().count();
                    Some(Segment {
                        span: p.span,
                        text: p.text.clone(),
                        at: *at - p.text.chars().count(),
                        glyphs: None,
                        width: p.width,
                        rise: p.rise,
                    })
                })
                .collect();
            let [dx, dy, room] = n.layout.wrap_lines.get(k).copied().unwrap_or([
                0.0,
                0.0,
                inner.height / sk.max(1e-6),
            ]);
            let across = line.height * sk;
            let col = if v.mode.block_rl() {
                inner.right() - (line_at + dy * sk) - across
            } else {
                inner.x + line_at + dy * sk
            };
            let mid = col + across / 2.0;
            // Each segment's length down the line.
            let lengths: Vec<f32> = segments
                .iter()
                .map(|seg| {
                    let sp = &rc.spans[seg.span];
                    if sp.atom.is_some() || seg.text.is_empty() {
                        return seg.width * sk;
                    }
                    let font = self.env.fonts.for_styled(sp.font, sp.query(), &seg.text).0;
                    let size = sp.step.size * sk;
                    let r =
                        self.span_run(i, sp, font, &seg.text, size, v, (mid, col, across), false);
                    crate::text::vertical::runs(&seg.text, sp.vertical(v))
                        .into_iter()
                        .fold(0.0, |at, (range, up)| {
                            crate::ui::vertical_run(
                                &r,
                                &seg.text[range],
                                up,
                                at,
                                &|a, _| a,
                                &mut || 0.0,
                                &mut |_, _, _| {},
                                &mut |_, _, _, _, _, _, _| {},
                            )
                        })
                })
                .collect();
            let total: f32 = lengths.iter().sum();
            let free = room * sk - total;
            let lead = dx * sk
                + match rc.align {
                    TextAlign::Start | TextAlign::Justify => 0.0,
                    TextAlign::Center => free / 2.0,
                    TextAlign::End => free,
                };
            let mut at = 0.0;
            for (seg, len) in segments.into_iter().zip(lengths) {
                let y = if from_bottom {
                    inner.bottom() - lead - at - len
                } else {
                    inner.y + lead + at
                };
                out.push((
                    k,
                    DrawnPiece {
                        span: seg.span,
                        text: seg.text,
                        at: line_first + seg.at,
                        glyphs: seg.glyphs,
                        rect: Rect::new(col, y, across, len),
                        baseline: mid + seg.rise * sk,
                        line: k,
                    },
                ));
                at += len;
            }
            cols.push((col, across));
            line_at += across;
        }
        (out, cols)
    }

    /// A vertical rich node's lines where they are drawn, for the player's
    /// selection: each line's column and its clusters' stretches, a
    /// cluster's characters counted over its line's pieces (a picture, an
    /// icon or a key cap none).
    pub(crate) fn vertical_rich_lines(&self, i: usize) -> Vec<crate::ui::VerticalLine> {
        let n = &self.tree.nodes[i];
        let (Content::Rich(rc), Some(v)) = (&*n.content, self.tree.rich_vertical(i)) else {
            return Vec::new();
        };
        let (pieces, cols) = self.vertical_rich_pieces(i, rc, v);
        let from_bottom = self.rich_from_bottom(i, v);
        let mut out: Vec<crate::ui::VerticalLine> = cols
            .into_iter()
            .map(|(col, width)| crate::ui::VerticalLine {
                col,
                width,
                glyphs: Vec::new(),
            })
            .collect();
        let mut before = vec![0usize; out.len()];
        for (k, piece) in pieces {
            let first = before[k];
            before[k] += piece.text.chars().count();
            let s = &rc.spans[piece.span];
            if s.atom.is_some() {
                continue;
            }
            let font = self.env.fonts.for_styled(s.font, s.query(), &piece.text).0;
            let r = piece.rect;
            let size = s.step.size * n.placed.scale;
            let run = self.span_run(
                i,
                s,
                font,
                &piece.text,
                size,
                v,
                (piece.baseline, r.x, r.width),
                false,
            );
            let top = |at: f32, len: f32| {
                if from_bottom {
                    r.bottom() - at - len
                } else {
                    r.y + at
                }
            };
            let glyphs = &mut out[k].glyphs;
            let mut at = 0.0;
            for (range, up) in crate::text::vertical::runs(&piece.text, s.vertical(v)) {
                let off = first + piece.text[..range.start].chars().count();
                at = crate::ui::vertical_run(
                    &run,
                    &piece.text[range],
                    up,
                    at,
                    &top,
                    &mut || 0.0,
                    &mut |c, y0, y1| glyphs.push((off + c.start..off + c.end, y0, y1)),
                    &mut |_, _, _, _, _, _, _| {},
                );
            }
        }
        out
    }

    /// A vertical rich node's commands: each piece's runs down its column
    /// in its span's font, ink, weight and outline
    /// ([`crate::ui::vertical_run`]), a picture or icon upright and
    /// centred in its stretch, a key cap's plate along it, and a link's or
    /// decoration's lines down it: under on the column's left (line-under),
    /// over on its right, through its middle (CSS Text Decoration 4 §2.3).
    fn vertical_rich_commands(
        &self,
        i: usize,
        rc: &RichContent,
        v: crate::layout::flow::VerticalText,
        cmds: &mut Vec<crate::draw::DrawCmd>,
    ) {
        use crate::draw::DrawCmd;
        let n = &self.tree.nodes[i];
        let p = &self.paint_theme().palette;
        let (alpha, dpr, clip) = (n.placed.alpha, self.env.viewport.dpr(), self.paint_clip(n));
        let from_bottom = self.rich_from_bottom(i, v);
        // The player's selection: each line's selected stretch, merged,
        // filled under the text, its column a line wide.
        let mut selected: Vec<Rect> = Vec::new();
        if let Some((a, b)) = n.marks.selected {
            let mut first = 0usize;
            let lines = self.vertical_rich_lines(i);
            for (line, l) in lines.iter().zip(&n.layout.rich_lines) {
                let count: usize = l.pieces.iter().map(|p| p.text.chars().count()).sum();
                let (a, b) = (
                    (a as usize).saturating_sub(first),
                    (b as usize).saturating_sub(first),
                );
                let mut picked: Vec<(f32, f32)> = line
                    .glyphs
                    .iter()
                    .filter(|(c, ..)| c.start >= a && c.start < b)
                    .map(|&(_, y0, y1)| (y0, y1))
                    .collect();
                picked.sort_by(|p, q| p.0.total_cmp(&q.0));
                let mut runs: Vec<(f32, f32)> = Vec::new();
                for (y0, y1) in picked {
                    match runs.last_mut() {
                        Some(r) if y0 <= r.1 + 0.01 => r.1 = r.1.max(y1),
                        _ => runs.push((y0, y1)),
                    }
                }
                selected.extend(
                    runs.into_iter()
                        .map(|(y0, y1)| Rect::new(line.col, y0, line.width, y1 - y0)),
                );
                first += count + 1;
            }
            self.selection_fills(i, &selected, clip, cmds);
        }
        let selection = (!selected.is_empty())
            .then(|| self.selection_paint(i))
            .flatten()
            .filter(|s| s.color.is_some() || s.decoration.is_some() || s.shadows.is_some());
        for piece in self.rich_pieces(i) {
            let s = &rc.spans[piece.span];
            // An atom is drawn inside its owners' edges.
            let r = if s.atom.is_some() {
                let [before, after] = s.room().map(|e| e * n.placed.scale);
                let r = piece.rect;
                let top = if self.rich_from_bottom(i, v) {
                    after
                } else {
                    before
                };
                Rect::new(r.x, r.y + top, r.width, r.height - before - after)
            } else {
                piece.rect
            };
            let mid = piece.baseline;
            let em = s.step.size * n.placed.scale;
            let link_key = s.link.as_deref().map(|l| child_key(n.key, l));
            // A document's inline `<a>` takes its hover from its CSS (`a:hover`).
            let hot =
                s.owners.is_empty() && link_key.is_some_and(|lk| self.interaction.state.hot == lk);
            let color = if hot { p.primary_hover } else { s.color };
            let upright_box = |w: f32, h: f32| {
                let h = h.min(r.height.max(1.0));
                Rect::new(mid - w / 2.0, r.y + (r.height - h) / 2.0, w, h)
            };
            match &s.atom {
                Some(Atom::Image(im)) => {
                    cmds.push(DrawCmd::Image {
                        rect: upright_box(em * im.aspect.max(0.05), em).snapped(dpr),
                        texture: Some(im.texture),
                        uv: im.uv,
                        color: Srgba::WHITE.fade(alpha).linear(),
                        clip,
                    });
                    continue;
                }
                Some(Atom::Icon(cell)) => {
                    let b = upright_box(em, em);
                    match *cell {
                        crate::ui::IconCell::Builtin(name) => {
                            cmds.extend(crate::default_icons::commands(
                                name,
                                b,
                                clip,
                                color.fade(alpha),
                                self.paint_theme(),
                            ))
                        }
                        crate::ui::IconCell::Glyph(font, ch) => cmds.push(DrawCmd::Text {
                            source: 0,
                            x: b.x,
                            baseline: b.y + em * 0.85,
                            size: em,
                            text: ch.to_string(),
                            color: color.fade(alpha).linear(),
                            font,
                            bounds: b,
                            clip,
                            tracking: 0.0,
                            weight: 0.0,
                            glow: None,
                            placements: Vec::new(),
                        }),
                        crate::ui::IconCell::Image(tex, uv) => cmds.push(DrawCmd::Image {
                            rect: b.snapped(dpr),
                            texture: Some(tex),
                            uv,
                            color: color.fade(alpha).linear(),
                            clip,
                        }),
                    }
                    continue;
                }
                Some(Atom::Ruby(ruby)) => {
                    let at = ruby::RubyAt {
                        rect: r,
                        baseline: mid,
                        color,
                    };
                    self.ruby_commands(i, s, ruby, at, Some(v), cmds);
                    continue;
                }
                Some(Atom::Cap(_)) => {
                    let m = CAP_MARGIN * em;
                    let plate = Rect::new(
                        mid - em * 0.53,
                        r.y + m,
                        em * 1.06,
                        (r.height - 2.0 * m).max(0.0),
                    );
                    cmds.push(DrawCmd::Quad {
                        rect: plate.snapped(dpr),
                        radius: em * 0.22,
                        color: color.fade(alpha * 0.16).linear(),
                        clip,
                    });
                }
                None => {}
            }
            let (text, size, start) = match &s.atom {
                Some(Atom::Cap(label)) => {
                    (label.clone(), em * CAP_TEXT, (CAP_MARGIN + CAP_PAD) * em)
                }
                _ => (piece.text.clone(), em, 0.0),
            };
            let font = self.env.fonts.for_styled(s.font, s.query(), &text).0;
            if font.handle.is_none() || text.is_empty() {
                continue;
            }
            let weight = if s.synth.bold {
                bold_weight(size, 0.0)
            } else {
                0.0
            };
            let run = self.span_run(i, s, font, &text, size, v, (mid, r.x, r.width), true);
            let top = |at: f32, len: f32| {
                if from_bottom {
                    r.bottom() - at - len
                } else {
                    r.y + at
                }
            };
            let first = cmds.len();
            let mut at = start;
            // Each cluster's characters and its stretch down the column,
            // for its emphasis mark.
            let mut stretches: Vec<(usize, f32, f32)> = Vec::new();
            for (range, set) in crate::text::vertical::runs(&text, s.vertical(v)) {
                let off = text[..range.start].chars().count();
                at = crate::ui::vertical_run(
                    &run,
                    &text[range],
                    set,
                    at,
                    &top,
                    &mut || 0.0,
                    &mut |c, y0, y1| stretches.push((off + c.start, y0, y1)),
                    &mut |x, baseline, t, handle, placements, transform, bounds| {
                        let Some(font) = handle.filter(|_| !t.is_empty()) else {
                            return;
                        };
                        let cmd = DrawCmd::Text {
                            source: 0,
                            x,
                            baseline,
                            size,
                            text: t,
                            color: color.fade(alpha).linear(),
                            font,
                            bounds,
                            clip,
                            tracking: run.tracking,
                            weight,
                            glow: None,
                            placements,
                        };
                        cmds.push(if transform == [1.0, 0.0, 0.0, 1.0] {
                            cmd
                        } else {
                            DrawCmd::StyledText {
                                text: Box::new(cmd),
                                effects: Box::new(og_draw::UiTextEffects {
                                    transform,
                                    ..Default::default()
                                }),
                            }
                        });
                    },
                );
            }
            // Emphasis marks beside each cluster: right or left in an
            // upright line, over or under in a sideways one (its over the
            // right, but the left in `sideways-lr`).
            if let Some(e) = s.atom.is_none().then_some(rc.emphasis).flatten() {
                use crate::layout::flow::WritingMode;
                let right = if v.sideways() {
                    e.under == (v.mode == WritingMode::SidewaysLr)
                } else {
                    !e.left
                };
                let x = mid + if right { 1.0 } else { -1.0 } * (size / 2.0 + size / 4.0);
                let centres: Vec<(f32, f32)> = emphasis::spots(&text, stretches)
                    .into_iter()
                    .map(|y| (x, y))
                    .collect();
                let ink = e.color.unwrap_or(color).fade(alpha).linear();
                emphasis::marks(&e, !v.sideways(), &centres, size, font, ink, clip, cmds);
            }
            // An outline from the distance field, as across.
            if let Some((w, oc, align)) = s.outline {
                let outline = Some(og_draw::UiTextOutline {
                    width: w * n.placed.scale,
                    color: oc.fade(alpha).linear(),
                    align,
                });
                for c in &mut cmds[first..] {
                    match c {
                        DrawCmd::StyledText { effects, .. } => effects.outline = outline,
                        DrawCmd::Text { .. } => {
                            let text = std::mem::replace(c, DrawCmd::PopClip);
                            *c = DrawCmd::StyledText {
                                text: Box::new(text),
                                effects: Box::new(og_draw::UiTextEffects {
                                    outline,
                                    transform: [1.0, 0.0, 0.0, 1.0],
                                    ..Default::default()
                                }),
                            };
                        }
                        _ => {}
                    }
                }
            }
            // `::selection`'s ink, lines and shadows: the glyphs cut at
            // the selected stretches down the column, and drawn again
            // there in its style, as across.
            let mut bands: Vec<Rect> = selected
                .iter()
                .filter(|b| b.overlaps(&r))
                .copied()
                .collect();
            if let Some(sel) = selection.as_ref().filter(|_| !bands.is_empty()) {
                bands.sort_by(|a, b| a.y.total_cmp(&b.y));
                let drawn: Vec<DrawCmd> = cmds.drain(first..).collect();
                // The unselected stretches: each glyph clipped between the bands.
                for c in &drawn {
                    let mut from = f32::NEG_INFINITY;
                    for gap in bands
                        .iter()
                        .map(|b| (b.y, b.bottom()))
                        .chain([(f32::INFINITY, f32::INFINITY)])
                    {
                        let mut piece = c.clone();
                        let keep = match text_clip(&mut piece) {
                            Some(cl) => {
                                let (top, bottom) = (from.max(cl.y), gap.0.min(cl.bottom()));
                                *cl = Rect::new(cl.x, top, cl.width, bottom - top);
                                bottom > top
                            }
                            None => from == f32::NEG_INFINITY,
                        };
                        if keep {
                            cmds.push(piece);
                        }
                        from = gap.1;
                    }
                }
                let ink = sel.color.unwrap_or(color);
                let shadows: Option<Vec<og_draw::UiTextShadow>> =
                    sel.shadows.as_ref().map(|list| {
                        list.iter()
                            .map(|(o, blur, c)| og_draw::UiTextShadow {
                                offset: og_core::glam::Vec2::new(
                                    o[0] * n.placed.scale,
                                    o[1] * n.placed.scale,
                                ),
                                blur: blur * n.placed.scale,
                                color: c.fade(alpha * s.color.alpha).linear(),
                            })
                            .collect()
                    });
                for band in &bands {
                    for c in &drawn {
                        let mut c = c.clone();
                        let Some(cl) = text_clip(&mut c) else {
                            continue;
                        };
                        *cl = cl.intersect(band);
                        let c = match c {
                            DrawCmd::StyledText {
                                mut text,
                                mut effects,
                            } => {
                                if let DrawCmd::Text { color, .. } = text.as_mut() {
                                    *color = ink.fade(alpha * s.color.alpha).linear();
                                }
                                if let Some(sh) = &shadows {
                                    effects.shadows = sh.clone();
                                }
                                DrawCmd::StyledText { text, effects }
                            }
                            DrawCmd::Text { .. } => {
                                let mut c = c;
                                if let DrawCmd::Text { color, .. } = &mut c {
                                    *color = ink.fade(alpha * s.color.alpha).linear();
                                }
                                match &shadows {
                                    Some(sh) => DrawCmd::StyledText {
                                        text: Box::new(c),
                                        effects: Box::new(og_draw::UiTextEffects {
                                            shadows: sh.clone(),
                                            transform: [1.0, 0.0, 0.0, 1.0],
                                            ..Default::default()
                                        }),
                                    },
                                    None => c,
                                }
                            }
                            c => c,
                        };
                        cmds.push(c);
                    }
                    // Its lines down the stretch, on the sides the span's take.
                    if let Some(d) = &sel.decoration {
                        // Down a column the underline is on its line-left
                        // side already, half an em from its middle.
                        let (th, off) = crate::css::text_css::Decoration {
                            under: false,
                            ..d.clone()
                        }
                        .placed(&font.metrics, size, n.placed.scale, dpr);
                        let lc = d.color.unwrap_or(ink).fade(alpha).linear();
                        let (y0, y1) = (band.y.max(r.y), band.bottom().min(r.bottom()));
                        let mut rule = |x: f32| {
                            vertical_decoration_line(
                                x - th / 2.0,
                                y0,
                                (y1 - y0).max(0.0),
                                th,
                                d.style,
                                lc,
                                clip,
                                dpr,
                                cmds,
                            )
                        };
                        if d.underline {
                            rule(mid - size * 0.5 - off);
                        }
                        if d.overline {
                            rule(mid + size * 0.6);
                        }
                        if d.through {
                            rule(mid);
                        }
                    }
                }
            }
            let d = crate::css::text_css::Decoration {
                under: false,
                ..s.decoration.clone().unwrap_or_default()
            };
            let (th, off) = d.placed(&font.metrics, size, n.placed.scale, dpr);
            let ink = d.color.unwrap_or(color).fade(alpha).linear();
            let mut rule = |x: f32| {
                cmds.push(DrawCmd::Quad {
                    rect: Rect::new(x - th / 2.0, r.y, th, r.height).snapped(dpr),
                    radius: 0.0,
                    color: ink,
                    clip,
                })
            };
            if link_key.is_some() || s.underline {
                rule(mid - size * 0.5 - off);
            }
            if d.overline {
                rule(mid + size * 0.6);
            }
            if s.strike {
                rule(mid);
            }
        }
    }
}

impl crate::ui::Tree {
    /// A rich node's vertical setting, `None` when its lines run across.
    pub(crate) fn rich_vertical(&self, i: usize) -> Option<crate::layout::flow::VerticalText> {
        self.nodes[i].rich_vertical()
    }

    /// Rich node `i`'s content box.
    fn rich_inner(&self, i: usize) -> Rect {
        let n = &self.nodes[i];
        let (sk, pad) = (n.placed.scale, n.layout.used.padding);
        Rect::new(
            n.placed.rect.x + pad.left * sk,
            n.placed.rect.y + pad.top * sk,
            n.placed.rect.width - pad.horizontal() * sk,
            n.placed.rect.height - pad.vertical() * sk,
        )
    }
}

/// A text command's clip, through its effects.
pub(crate) fn text_clip(c: &mut crate::draw::DrawCmd) -> Option<&mut Rect> {
    use crate::draw::DrawCmd;
    match c {
        DrawCmd::StyledText { text, .. } => match text.as_mut() {
            DrawCmd::Text { clip, .. } => Some(clip),
            _ => None,
        },
        DrawCmd::Text { clip, .. } => Some(clip),
        _ => None,
    }
}

/// One decoration line of `style` down a vertical line: from `y` for
/// `length`, its left at `x` ([`decoration_line`] turned a quarter).
#[allow(clippy::too_many_arguments)]
pub(crate) fn vertical_decoration_line(
    x: f32,
    y: f32,
    length: f32,
    th: f32,
    style: crate::css::text_css::DecorationStyle,
    color: [f32; 4],
    clip: Rect,
    dpr: f32,
    cmds: &mut Vec<crate::draw::DrawCmd>,
) {
    let from = cmds.len();
    decoration_line(y, length, x, th, style, color, clip, dpr, cmds);
    for c in &mut cmds[from..] {
        if let crate::draw::DrawCmd::Quad { rect, .. } = c {
            *rect = Rect::new(rect.y, rect.x, rect.height, rect.width);
        }
    }
}

/// Whether each of a horizontal line's `pieces` (in line order) starts
/// its line and ends it: only spacers before it on its line, and after it.
fn line_edges(pieces: &[DrawnPiece]) -> Vec<(bool, bool)> {
    let blank = |p: &DrawnPiece| p.text.chars().all(og_i18n::text::spacer);
    let mut out = vec![(false, false); pieces.len()];
    let mut from = 0;
    for line in pieces.chunk_by(|a, b| a.rect.y == b.rect.y) {
        let mut start = true;
        for (k, p) in line.iter().enumerate() {
            out[from + k].0 = start;
            start &= blank(p);
        }
        let mut end = true;
        for (k, p) in line.iter().enumerate().rev() {
            out[from + k].1 = end;
            end &= blank(p);
        }
        from += line.len();
    }
    out
}

/// The x ranges (px from a piece's start) of the spacers its decoration
/// lines skip (CSS Text Decoration 4 §2.10.3): those `skip` names at
/// the line's start or end (when the piece is there) or all of them and
/// every word separator, each with the letter spacing (`tracking` em
/// after each glyph) before it.
fn space_gaps(
    glyphs: &[crate::text::VisualGlyph],
    size: f32,
    tracking: f32,
    skip: crate::css::text_css::SkipSpaces,
    line_start: bool,
    line_end: bool,
) -> Vec<(f32, f32)> {
    use og_i18n::text::{spacer, word_separator};
    let first = glyphs.iter().position(|g| !spacer(g.ch));
    let last = glyphs.iter().rposition(|g| !spacer(g.ch));
    let mut out: Vec<(f32, f32)> = Vec::new();
    let mut pen = 0.0f32;
    for (k, g) in glyphs.iter().enumerate() {
        let x0 = pen;
        pen += g.advance * size;
        let leading = line_start && skip.start && first.is_none_or(|f| k < f);
        let trailing = line_end && skip.end && last.is_none_or(|l| k > l);
        let skipped = (spacer(g.ch) && (skip.all || leading || trailing))
            || (skip.all && word_separator(g.ch));
        if !skipped {
            continue;
        }
        match out.last_mut() {
            Some(gap) if gap.1 == x0 => gap.1 = pen,
            _ => {
                // The letter spacing after the glyph before it.
                let spacing = if k > 0 { tracking * size } else { 0.0 };
                out.push((x0 - spacing, pen));
            }
        }
    }
    out
}

/// Cuts the horizontal gaps `gaps` (sorted x ranges) out of the quads
/// pushed since `from`: a quad across a gap becomes its parts outside it.
fn cut_lines(cmds: &mut Vec<crate::draw::DrawCmd>, from: usize, gaps: &[(f32, f32)]) {
    use crate::draw::DrawCmd;
    if gaps.is_empty() {
        return;
    }
    let drawn = cmds.split_off(from);
    for c in drawn {
        let DrawCmd::Quad {
            rect,
            radius,
            color,
            clip,
        } = c
        else {
            cmds.push(c);
            continue;
        };
        let mut x = rect.x;
        for &(a, b) in gaps
            .iter()
            .filter(|(a, b)| *b > rect.x && *a < rect.right())
        {
            if a > x {
                cmds.push(DrawCmd::Quad {
                    rect: Rect::new(x, rect.y, a - x, rect.height),
                    radius,
                    color,
                    clip,
                });
            }
            x = x.max(b);
        }
        if x < rect.right() {
            cmds.push(DrawCmd::Quad {
                rect: Rect::new(x, rect.y, rect.right() - x, rect.height),
                radius,
                color,
                clip,
            });
        }
    }
}

/// One decoration line of `style` from `x` across `width`, its top at `y`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn decoration_line(
    x: f32,
    width: f32,
    y: f32,
    th: f32,
    style: crate::css::text_css::DecorationStyle,
    color: [f32; 4],
    clip: Rect,
    dpr: f32,
    cmds: &mut Vec<crate::draw::DrawCmd>,
) {
    use crate::css::text_css::DecorationStyle as S;
    use crate::draw::DrawCmd;
    let mut quad = |r: Rect, radius: f32| {
        cmds.push(DrawCmd::Quad {
            rect: r.snapped(dpr),
            radius,
            color,
            clip,
        })
    };
    match style {
        S::Solid => quad(Rect::new(x, y, width, th), 0.0),
        S::Double => {
            quad(Rect::new(x, y, width, th), 0.0);
            quad(Rect::new(x, y + th * 2.0, width, th), 0.0);
        }
        S::Dotted | S::Dashed => {
            let (on, gap) = if style == S::Dotted {
                (th, th)
            } else {
                (th * 3.0, th * 2.0)
            };
            let mut at = x;
            while at < x + width {
                let w = on.min(x + width - at);
                quad(
                    Rect::new(at, y, w, th),
                    if style == S::Dotted { th / 2.0 } else { 0.0 },
                );
                at += on + gap;
            }
        }
        S::Wavy => {
            // A sine of a period of six thicknesses, as dots along it.
            let (amp, step) = (th * 1.2, th * 0.5);
            let mut at = 0.0;
            while at < width {
                let dy = amp * (at / (th * 6.0) * std::f32::consts::TAU).sin();
                quad(Rect::new(x + at, y + amp + dy, step.max(th), th), th / 2.0);
                at += step;
            }
        }
    }
}
