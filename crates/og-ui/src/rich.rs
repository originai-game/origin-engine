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

use crate::id::child_key;

use crate::layout::{
    Content, Element, FontRole, InlinePlace, Metrics, Rect, Size, Strut, TextAlign, VerticalAlign,
};
use crate::markup::{SpanFx, parse_markup};
use crate::style::{Srgba, TypeStep};
use crate::ui::{Response, Ui};

/// An inline thing a span is instead of text, laid out as one unbreakable
/// unit on the baseline.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Atom {
    /// An icon of the atlas, one em square, in the span's ink.
    Icon(crate::ui::IconCell),
    /// A registered picture, one em tall at its aspect, in its own colours.
    Image(crate::ui::InlineImage),
    /// A key's name on a cap (an action's glyph with no icon).
    Cap(String),
}

/// One resolved span: its text (or atom) in its own style.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RichSpan {
    pub text: String,
    pub step: TypeStep,
    pub color: Srgba,
    pub font: FontRole,
    pub bold: bool,
    pub italic: bool,
    /// The weight its face is picked at: 700 for bold, else its step's.
    pub weight: u16,
    /// The bold and slant no real face of the style draws, synthesised.
    pub synth: crate::ui::Synth,
    pub underline: bool,
    pub strike: bool,
    /// An outline's width (CSS px) and colour.
    pub outline: Option<(f32, Srgba)>,
    pub atom: Option<Atom>,
    pub link: Option<String>,
    pub fx: SpanFx,
    /// Its CSS decoration's style, colour, thickness, offset and overline.
    pub decoration: Option<crate::doc::text_css::Decoration>,
    /// Where it sits in its line against the paragraph's text.
    pub valign: VerticalAlign,
}

/// An italic's slant: x moves by this much of the height above the baseline.
const SLANT: f32 = 0.2;

/// A cap's text size, margin and padding, of the span's size.
const CAP_TEXT: f32 = 0.78;
const CAP_MARGIN: f32 = 0.08;
const CAP_PAD: f32 = 0.24;

/// A paragraph of spans.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RichContent {
    pub spans: Vec<RichSpan>,
    /// The paragraph's own type step and font: the text a span's
    /// `vertical-align` is measured against (its strut).
    pub base: (TypeStep, FontRole),
    pub align: TextAlign,
    pub wrap: bool,
}

/// A piece of one span on one line, `x` from the line's start.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Piece {
    pub span: usize,
    pub text: String,
    pub x: f32,
    pub width: f32,
    /// How far its span's baseline is raised above the line's.
    pub rise: f32,
}

/// One laid-out line of a paragraph.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct RichLine {
    pub pieces: Vec<Piece>,
    pub width: f32,
    pub height: f32,
    /// From the line's top to the shared baseline.
    pub baseline: f32,
    pub shaped: Option<Vec<Segment>>,
}

/// How rich text looks by default (`RichTextDesc` in the IDL).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RichTextDesc {
    /// `None`: `text-body`.
    pub step: Option<TypeStep>,
    /// `None`: `ink`.
    pub color: Option<Srgba>,
    pub align: TextAlign,
    pub nowrap: bool,
    pub width: Size,
    pub font: FontRole,
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

mod spans;
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
    let size = s.step.size;
    let down = |t: &str, size: f32| {
        let m = fonts.metrics_styled(s.font, s.weight, s.italic, t);
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
        None => {}
    }
    let m = fonts.metrics_styled(s.font, s.weight, s.italic, text);
    m.measure_tracked(text, s.step.size, span_tracking(s, text)) + bold_overhang(s, text)
}

/// A span's tracking (em): its step's, and for a bold run [`BOLD_GROWTH`]
/// more per glyph, as a bold face's advance grows by about its added stem: its letters stay apart without reading letter-spaced. A
/// shaped run takes none (its joins would break). Either keeps
/// [`bold_overhang`] after it.
pub(crate) fn span_tracking(s: &RichSpan, text: &str) -> f32 {
    let step = crate::layout::step_tracking(&s.step, text);
    if s.synth.bold && !text.chars().any(og_i18n::needs_shaping) {
        step + BOLD_GROWTH
    } else {
        step
    }
}

/// How far a bold span's ink reaches past its advance (CSS px): the
/// shader's thickening ([`bold_weight`]) and the second copy, one device
/// pixel to the right (at most 1 px). Reserved after each bold run, so the
/// space after it does not close up.
fn bold_overhang(s: &RichSpan, text: &str) -> f32 {
    if s.synth.bold && !text.trim().is_empty() {
        bold_weight(s.step.size, 1.0) + 1.0
    } else {
        0.0
    }
}

/// How much wider a bold face's stems are than its regular's, em (Noto
/// Sans: 0.092 em to 0.148 em). A faux bold heavier than that closes the
/// counters of an `a` or an `A`.
const BOLD_GROWTH: f32 = 0.056;

/// A bold span's thickening at `size`, each side, when its second copy is
/// `smear` to the right (both in the text's units): together they grow its
/// stems by [`BOLD_GROWTH`], and small text by at least a quarter of the
/// copy's offset more.
fn bold_weight(size: f32, smear: f32) -> f32 {
    ((size * BOLD_GROWTH - smear) * 0.5).max(smear * 0.25)
}

pub(crate) fn span_metrics(s: &RichSpan, fonts: &dyn Metrics) -> (f32, f32, f32) {
    // (line height, ascent, descent) in CSS px.
    if s.atom.is_some() {
        return (s.step.line_height, s.step.size * 0.85, -s.step.size * 0.25);
    }
    let m = fonts.metrics_styled(s.font, s.weight, s.italic, &s.text);
    let lh = if s.text.chars().any(crate::text::is_cjk) {
        s.step.cjk_line_height
    } else {
        s.step.line_height
    };
    // A span mixing faces: the extents of every face it takes.
    let (a, d) = m.extents(&s.text);
    (lh, a * s.step.size, d * s.step.size)
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
        if s.atom.is_some() {
            let glued = out.last().and_then(|p| p.last()).is_some_and(|u| !u.space);
            out.last_mut().expect("a paragraph").push(Unit {
                span: k,
                text: String::new(),
                space: false,
                width: span_advance(s, "", fonts, v),
                glued,
            });
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
                out.last_mut().expect("a paragraph").push(Unit {
                    span: k,
                    text: t.to_string(),
                    space,
                    width: span_advance(s, t, fonts, v),
                    glued,
                });
            }
        }
    }
    out
}

/// The widest paragraph unbroken, and the widest unbreakable group: along
/// the line, the longest down a vertical one `v`.
pub(crate) fn rich_widths(
    rc: &RichContent,
    fonts: &dyn Metrics,
    v: Option<crate::layout::flow::VerticalText>,
) -> (f32, f32) {
    #[cfg(feature = "shaping")]
    if v.is_none()
        && let Some((_, widths)) = crate::text::paragraph::rich(rc, fonts, None)
    {
        return (widths.0, if rc.wrap { widths.1 } else { widths.0 });
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
    (pref, if rc.wrap { min } else { pref })
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
        let finish = |line: &mut Vec<&Unit>, lines: &mut Vec<RichLine>| {
            lines.push(build_line(rc, fonts, std::mem::take(line)));
        };
        let mut any = false;
        for g in groups {
            if g[0].space {
                pending.extend(g.iter().copied());
                pending_w += g.iter().map(|u| u.width).sum::<f32>();
                continue;
            }
            let gw: f32 = g.iter().map(|u| u.width).sum();
            if let Some(max) = max
                && !line.is_empty()
                && width + pending_w + gw > max
            {
                finish(&mut line, &mut lines);
                any = true;
                width = 0.0;
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
            finish(&mut line, &mut lines);
        }
    }
    lines
}

fn build_line(rc: &RichContent, fonts: &dyn Metrics, units: Vec<&Unit>) -> RichLine {
    let mut pieces: Vec<Piece> = Vec::new();
    let mut x = 0.0;
    for u in units {
        match pieces.last_mut() {
            Some(p) if p.span == u.span && rc.spans[u.span].atom.is_none() => {
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
    RichLine {
        width: x,
        height: h,
        baseline,
        pieces,
        shaped: None,
    }
}

/// A line's height and baseline from its pieces' spans, and each piece's
/// rise. The baseline-aligned spans share one baseline, their extents
/// centred in the tallest line height. A span with another
/// `vertical-align` sits against the paragraph's text (CSS 2.1 §10.8.1,
/// half its leading above and below it) and the line grows to hold it and
/// the paragraph's own line (its strut).
pub(crate) fn place_line(
    rc: &RichContent,
    fonts: &dyn Metrics,
    pieces: &mut [Piece],
) -> (f32, f32) {
    let (mut h, mut a, mut d) = (0.0f32, 0.0f32, 0.0f32);
    let mut moved = false;
    for p in pieces.iter() {
        let s = &rc.spans[p.span];
        if s.valign != VerticalAlign::Baseline {
            moved = true;
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
    if !moved {
        return (h, baseline);
    }
    let (step, font) = rc.base;
    let m = fonts.metrics_styled(font, step.weight, false, "");
    let strut = Strut {
        size: step.size,
        ascent: m.ascender * step.size,
        descent: -m.descender * step.size,
        x_height: m.x_height() * step.size,
    };
    let lead = (step.line_height - strut.ascent - strut.descent) / 2.0;
    let base = (
        baseline.max(strut.ascent + lead),
        (h - baseline).max(strut.descent + lead),
    );
    let moving: Vec<usize> = (0..pieces.len())
        .filter(|&k| rc.spans[pieces[k].span].valign != VerticalAlign::Baseline)
        .collect();
    let items: Vec<(InlinePlace, f32, f32)> = moving
        .iter()
        .map(|&k| {
            let s = &rc.spans[pieces[k].span];
            let (lh, asc, desc) = span_metrics(s, fonts);
            let lead = (lh - (asc - desc)) / 2.0;
            let (above, below) = (asc + lead, lead - desc);
            (
                s.valign.place(&strut, s.step.size, lh, above, below),
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
    pub glyphs: Option<Vec<crate::text::VisualGlyph>>,
    pub rect: Rect,
    pub baseline: f32,
}

/// One segment of a line in visual order: a piece, or the part of one in a
/// single bidi run; its width in CSS px at scale 1.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Segment {
    pub span: usize,
    pub text: String,
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
        s.atom.is_none()
            && fonts
                .metrics_styled(s.font, s.weight, s.italic, &p.text)
                .is_mixed()
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
        return line
            .pieces
            .iter()
            .map(|p| Segment {
                span: p.span,
                text: p.text.clone(),
                glyphs: None,
                width: p.width,
                rise: p.rise,
            })
            .collect();
    }
    // The line's text and where each piece's text lies in it.
    let mut text = String::new();
    let mut ranges = Vec::with_capacity(line.pieces.len());
    for p in &line.pieces {
        let start = text.len();
        if rc.spans[p.span].atom.is_some() {
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
            if s.atom.is_some() {
                out.push(Segment {
                    span: p.span,
                    text: String::new(),
                    glyphs: None,
                    width: p.width,
                    rise: p.rise,
                });
                continue;
            }
            let piece_text = &text[r.clone()];
            let m = fonts.metrics_styled(s.font, s.weight, s.italic, piece_text);
            let tracking = span_tracking(s, piece_text);
            let needs = rtl || crate::text::needs_visual(piece_text) || m.is_mixed();
            if !needs {
                out.push(Segment {
                    span: p.span,
                    text: piece_text.to_string(),
                    glyphs: None,
                    width: span_width(s, piece_text, fonts),
                    rise: p.rise,
                });
                continue;
            }
            let first = text[ranges[k].start..r.start].chars().count();
            let mut glyphs = m.run(piece_text, rtl, tracking, first);
            glyphs.retain(|g| !og_i18n::is_bidi_control(g.ch));
            let width = glyphs.iter().map(|g| g.advance).sum::<f32>() * s.step.size
                + bold_overhang(s, piece_text);
            out.push(Segment {
                span: p.span,
                text: piece_text.to_string(),
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
    /// A paragraph of rich text from markup ([`crate::markup`]): spans of
    /// colour (`[color=primary]`, `[color=#ff8800]`), weight (`[b]`), size
    /// (`[size=lg]`), font (`[font=mono]`), inline icons (`[icon=coin]`),
    /// links (`[link=shop]Shop[/link]`) and per-glyph motion (`[wave]`,
    /// `[shake]`, `[pulse]`). Pass a string-table message (og.i18n) so a
    /// translation carries its own emphasis. Markup it cannot read is shown
    /// as text and named in a frame warning.
    pub fn rich_text(
        &mut self,
        id: impl Into<crate::Id>,
        markup: &str,
        desc: RichTextDesc,
    ) -> RichTextResponse {
        let t = self.env.theme;
        let (parsed, warnings) = parse_markup(markup);
        self.warnings.extend(warnings);
        let base_step = desc.step.unwrap_or(t.type_ramp.body);
        let base_color = desc.color.unwrap_or(t.palette.ink);
        let mut spans = Vec::with_capacity(parsed.len());
        // What the probe and a screen reader read: an action's glyph by name.
        let mut probe = String::new();
        for s in &parsed {
            let mut step = base_step;
            if let Some(name) = &s.size {
                let mut ramp = t.type_ramp;
                match crate::style_file::ramp_step(&mut ramp, name) {
                    Some(st) => step = *st,
                    None => self
                        .warnings
                        .push(format!("rich text: [size={name}] is not a TypeStepName")),
                }
            }
            let mut color = if s.link.is_some() {
                t.palette.highlight
            } else {
                base_color
            };
            if let Some(name) = &s.color {
                match crate::style_file::parse_color(name).or_else(|| t.color(name)) {
                    Some(c) => color = c,
                    None => self.warnings.push(format!(
                        "rich text: [color={name}] is neither a colour role (a ThemeColor or one the style declares) nor #rrggbb"
                    )),
                }
            }
            let font = match s.font.as_deref() {
                Some("display") => FontRole::Display,
                Some("mono") => FontRole::Mono,
                Some("rounded") => FontRole::Rounded,
                Some("body") => FontRole::Body,
                _ => desc.font,
            };
            let outline = s.outline.as_deref().and_then(|v| {
                let (w, c) = v.split_once(',').unwrap_or((v, "shadow"));
                let w = w.trim().parse::<f32>().ok().filter(|w| *w > 0.0 && w.is_finite());
                let c = crate::style_file::parse_color(c.trim()).or_else(|| t.color(c.trim()));
                match (w, c) {
                    (Some(w), Some(c)) => Some((w, c)),
                    _ => {
                        self.warnings.push(format!(
                            "rich text: [outline={v}] is not a width above 0 and a colour role or #rrggbb"
                        ));
                        None
                    }
                }
            });
            let mut text = s.text.clone();
            let atom = if let Some(name) = &s.icon {
                match self.env.icons.find(name) {
                    Some(k) => Some(Atom::Icon(k)),
                    None => {
                        self.warnings
                            .push(format!("rich text: [icon={name}] is not in the icon atlas"));
                        continue;
                    }
                }
            } else if let Some(name) = &s.image {
                match self.env.images.get(name) {
                    Some(im) => Some(Atom::Image(*im)),
                    None => {
                        self.warnings.push(format!(
                            "rich text: [img={name}] is no picture the game registered (UiDesc.images)"
                        ));
                        continue;
                    }
                }
            } else if let Some(name) = &s.action {
                // The glyph on the device used last, this frame: its icon
                // where the atlas has one, else its name on a cap.
                match self.glyph(name) {
                    Some(g) => {
                        text = g.text.clone();
                        match self.env.icons.find(&g.icon) {
                            Some(k) => Some(Atom::Icon(k)),
                            None if !g.text.is_empty() => Some(Atom::Cap(g.text)),
                            None => continue,
                        }
                    }
                    None => {
                        self.warnings.push(format!(
                            "rich text: [action={name}] is no action with a binding on this device"
                        ));
                        continue;
                    }
                }
            } else {
                None
            };
            probe.push_str(&text);
            // A real bold or italic face when the style has one; else the
            // thickening and the oblique stand in for it.
            let weight = if s.bold { 700 } else { step.weight };
            let synth = self.env.fonts.for_styled(font, weight, s.italic, "").1;
            let synth = crate::ui::Synth {
                bold: s.bold && synth.bold,
                italic: synth.italic,
            };
            spans.push(RichSpan {
                text: if atom.is_some() { String::new() } else { text },
                step,
                color,
                font,
                bold: s.bold,
                italic: s.italic,
                weight,
                synth,
                underline: s.underline,
                strike: s.strike,
                outline,
                atom,
                link: s.link.clone(),
                fx: s.fx,
                decoration: None,
                valign: VerticalAlign::Baseline,
            });
        }
        self.rich_runs(id.into(), probe, spans, desc)
    }

    fn rich_runs(
        &mut self,
        id: crate::Id,
        probe: String,
        spans: Vec<RichSpan>,
        desc: RichTextDesc,
    ) -> RichTextResponse {
        let i = self.leaf(
            "richtext",
            id,
            Element {
                width: desc.width,
                ..Element::default()
            },
        );
        let mut links: Vec<String> = Vec::new();
        for s in &spans {
            if let Some(l) = &s.link
                && !links.contains(l)
            {
                links.push(l.clone());
            }
        }
        let out = self.respond_links(self.nodes[i].key, &links);
        let n = &mut self.nodes[i];
        n.probe_text = probe;
        n.content = Content::Rich(RichContent {
            spans,
            base: (
                desc.step.unwrap_or(self.env.theme.type_ramp.body),
                desc.font,
            ),
            align: desc.align,
            wrap: !desc.nowrap,
        });
        n.links = links;
        out
    }

    /// What the links of the rich node keyed `key` did this tick.
    pub(crate) fn respond_links(&self, key: u64, links: &[String]) -> RichTextResponse {
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
        let n = &self.nodes[i];
        let Content::Rich(rc) = &n.content else {
            return Vec::new();
        };
        let sk = n.scale;
        let inner = self.rich_inner(i);
        if let Some(v) = self.rich_vertical(i) {
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
        for (k, line) in n.rich_lines.iter().enumerate() {
            let segments = visual_segments(rc, &self.env.fonts, line, self.env.direction);
            let width: f32 = segments.iter().map(|s| s.width).sum();
            // Its line box: the room floats leave it (`Node::wrap_lines`,
            // its drop below the lines above), or its box.
            let (bx, bw, dy) = match n.wrap_lines.get(k) {
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
            let mut x = x0;
            for seg in segments {
                out.push(DrawnPiece {
                    span: seg.span,
                    text: seg.text,
                    glyphs: seg.glyphs,
                    rect: Rect::new(x, y, seg.width * sk, line.height * sk),
                    baseline: y + (line.baseline - seg.rise) * sk,
                });
                x += seg.width * sk;
            }
            line_y += line.height * sk;
        }
        out
    }

    /// The rectangle of each link of rich node `i` (the union of its pieces).
    pub(crate) fn rich_links(&self, i: usize) -> Vec<(String, Rect)> {
        let n = &self.nodes[i];
        let Content::Rich(rc) = &n.content else {
            return Vec::new();
        };
        let pieces = self.rich_pieces(i);
        n.links
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
        let n = &self.nodes[i];
        let t = &self.env.theme;
        let p = &t.palette;
        let alpha = n.alpha;
        let dpr = self.env.viewport.dpr();
        let snap = |v: f32| crate::layout::snap(v, dpr);
        let clip = n.clip;
        let tick = self.state.tick as f32;
        let still = t.reduced_motion;
        // A link focused by keys or a pad: the focus ring around it, under the text.
        if self.state.focus_visible {
            for (l, r) in self.rich_links(i) {
                if self.state.focus == child_key(n.key, &l) {
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
        if let Some(v) = self.rich_vertical(i) {
            self.vertical_rich_commands(i, rc, v, cmds);
            return;
        }
        for piece in self.rich_pieces(i) {
            let (k, text, r, baseline) = (piece.span, piece.text, piece.rect, piece.baseline);
            let s = &rc.spans[k];
            let size = s.step.size * n.scale;
            let link_key = s.link.as_deref().map(|l| child_key(n.key, l));
            let hot = link_key.is_some_and(|lk| self.state.hot == lk);
            let color = if hot { p.primary_hover } else { s.color };
            if let Some(Atom::Image(im)) = &s.atom {
                // A picture in its own colours, one em tall on the baseline.
                let h = s.step.size * n.scale;
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
                let em = s.step.size * n.scale;
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
                let side = s.step.size * n.scale;
                match *cell {
                    crate::ui::IconCell::Glyph(font, ch) => {
                        // An icon set's glyph: its em square where a cell would be.
                        let (x, top) = (r.x + side * 0.05, baseline - side * 0.85);
                        cmds.push(DrawCmd::Text {
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
            let font = self
                .env
                .fonts
                .for_styled(s.font, s.weight, s.italic, &text)
                .0;
            let Some(handle) = font.handle else { continue };
            let first = cmds.len();
            let mixed = font.metrics.is_mixed();
            let m = &font.metrics;
            let tracking = span_tracking(s, &text);
            // Bold: thickened in the shader (as far as the field reaches)
            // and drawn twice, the second copy one device pixel to the
            // right, so small text whose field is thin is bold too: a
            // device pixel in the text's units (an SVG's text is laid out
            // in its view box's and scaled).
            let w = &n.world;
            let device = dpr * (w.a * w.d - w.b * w.c).abs().sqrt().max(1e-3);
            let smear = if s.synth.bold { 1.0 / device } else { 0.0 };
            let weight = if s.synth.bold {
                bold_weight(size, smear)
            } else {
                0.0
            };
            let glyph = |x: f32,
                         dy: f32,
                         a: f32,
                         text: String,
                         placements: Vec<og_draw::GlyphPlacement>,
                         w: f32| DrawCmd::Text {
                x: snap(x),
                baseline: snap(baseline + dy),
                size,
                text,
                color: color.fade(alpha * a).linear(),
                font: handle,
                bounds: Rect::new(x, r.y + dy, w, r.height),
                clip,
                tracking,
                weight,
                glow: None,
                placements,
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
                    let (text, placements) = crate::text::FontMetrics::drawn(run);
                    let (dy, a) = effect(g);
                    let copies = if smear > 0.0 {
                        &[smear, 0.0][..]
                    } else {
                        &[0.0][..]
                    };
                    for &dx in copies {
                        cmds.push(DrawCmd::Text {
                            x: snap(x + dx),
                            baseline: snap(baseline + dy),
                            size,
                            text: text.clone(),
                            color: color.fade(alpha * a).linear(),
                            font: f,
                            bounds: Rect::new(x + dx, r.y + dy, w, r.height),
                            clip,
                            tracking,
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
                            x: snap(r.x + dx) + pc.dx * size,
                            baseline: snap(baseline),
                            size,
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
                            tracking,
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
                        outline: s.outline.map(|(w, oc)| og_draw::UiTextOutline {
                            width: w * n.scale,
                            color: oc.fade(alpha).linear(),
                            align: og_draw::UiAlign::Outside,
                        }),
                        fill_box: [bounds.x, bounds.y, bounds.width, bounds.height],
                        transform: if s.synth.italic {
                            [1.0, 0.0, -SLANT, 1.0]
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
            let th = d
                .thickness
                .map_or((size * 0.07).max(1.0 / dpr), |t| t * n.scale);
            let ink = d.color.unwrap_or(color).fade(alpha).linear();
            let mut rule =
                |y: f32| decoration_line(r.x, r.width, y, th, d.style, ink, clip, dpr, cmds);
            if link_key.is_some() || s.underline {
                let off = d.offset.map_or(size * 0.12, |o| o * n.scale);
                let under = if d.under { size * 0.1 } else { 0.0 };
                rule(baseline + off + under);
            }
            if d.overline {
                rule(baseline - size * 0.95);
            }
            if s.strike {
                rule(baseline - size * 0.3);
            }
        }
    }
}

impl Ui {
    /// A rich node's vertical setting, `None` when its lines run across.
    pub(crate) fn rich_vertical(&self, i: usize) -> Option<crate::layout::flow::VerticalText> {
        self.nodes[i].rich_vertical()
    }

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

    /// Rich node `i`'s content box.
    fn rich_inner(&self, i: usize) -> Rect {
        let n = &self.nodes[i];
        let (sk, pad) = (n.scale, n.el.padding);
        Rect::new(
            n.rect.x + pad.left * sk,
            n.rect.y + pad.top * sk,
            n.rect.width - pad.horizontal() * sk,
            n.rect.height - pad.vertical() * sk,
        )
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
        let n = &self.nodes[i];
        let sk = n.scale;
        let inner = self.rich_inner(i);
        let from_bottom = self.rich_from_bottom(i, v);
        let mut out = Vec::new();
        let mut cols = Vec::with_capacity(n.rich_lines.len());
        let mut line_at = 0.0;
        for (k, line) in n.rich_lines.iter().enumerate() {
            let segments: Vec<Segment> = line
                .pieces
                .iter()
                .map(|p| Segment {
                    span: p.span,
                    text: p.text.clone(),
                    glyphs: None,
                    width: p.width,
                    rise: p.rise,
                })
                .collect();
            let [dx, dy, room] =
                n.wrap_lines
                    .get(k)
                    .copied()
                    .unwrap_or([0.0, 0.0, inner.height / sk.max(1e-6)]);
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
                    if sp.atom.is_some() {
                        return seg.width * sk;
                    }
                    let font = self
                        .env
                        .fonts
                        .for_styled(sp.font, sp.weight, sp.italic, &seg.text)
                        .0;
                    let size = sp.step.size * sk;
                    let r =
                        self.span_run(i, sp, font, &seg.text, size, v, (mid, col, across), false);
                    crate::text::vertical::runs(&seg.text, v).into_iter().fold(
                        0.0,
                        |at, (range, up)| {
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
                        },
                    )
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
                        glyphs: seg.glyphs,
                        rect: Rect::new(col, y, across, len),
                        baseline: mid + seg.rise * sk,
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
        let n = &self.nodes[i];
        let (Content::Rich(rc), Some(v)) = (&n.content, self.rich_vertical(i)) else {
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
            let font = self
                .env
                .fonts
                .for_styled(s.font, s.weight, s.italic, &piece.text)
                .0;
            let r = piece.rect;
            let size = s.step.size * n.scale;
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
            for (range, up) in crate::text::vertical::runs(&piece.text, v) {
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
        let n = &self.nodes[i];
        let p = &self.env.theme.palette;
        let (alpha, dpr, clip) = (n.alpha, self.env.viewport.dpr(), n.clip);
        let from_bottom = self.rich_from_bottom(i, v);
        // The player's selection: each line's selected stretch, merged,
        // filled under the text, its column a line wide.
        let mut selected: Vec<Rect> = Vec::new();
        if let Some((a, b)) = n.selected {
            let mut first = 0usize;
            let lines = self.vertical_rich_lines(i);
            for (line, l) in lines.iter().zip(&n.rich_lines) {
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
        let selection = self
            .docs
            .selection
            .get(&n.key)
            .filter(|s| {
                !selected.is_empty()
                    && (s.color.is_some() || s.decoration.is_some() || s.shadows.is_some())
            })
            .cloned();
        for piece in self.rich_pieces(i) {
            let s = &rc.spans[piece.span];
            let r = piece.rect;
            let mid = piece.baseline;
            let em = s.step.size * n.scale;
            let link_key = s.link.as_deref().map(|l| child_key(n.key, l));
            let hot = link_key.is_some_and(|lk| self.state.hot == lk);
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
                        crate::ui::IconCell::Glyph(font, ch) => cmds.push(DrawCmd::Text {
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
            let font = self
                .env
                .fonts
                .for_styled(s.font, s.weight, s.italic, &text)
                .0;
            if font.handle.is_none() {
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
            for (range, up) in crate::text::vertical::runs(&text, v) {
                at = crate::ui::vertical_run(
                    &run,
                    &text[range],
                    up,
                    at,
                    &top,
                    &mut || 0.0,
                    &mut |_, _, _| {},
                    &mut |x, baseline, t, handle, placements, transform, bounds| {
                        let Some(font) = handle.filter(|_| !t.is_empty()) else {
                            return;
                        };
                        let cmd = DrawCmd::Text {
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
            // An outline from the distance field, as across.
            if let Some((w, oc)) = s.outline {
                let outline = Some(og_draw::UiTextOutline {
                    width: w * n.scale,
                    color: oc.fade(alpha).linear(),
                    align: og_draw::UiAlign::Outside,
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
            if let Some(sel) = selection.as_deref().filter(|_| !bands.is_empty()) {
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
                                offset: og_core::glam::Vec2::new(o[0] * n.scale, o[1] * n.scale),
                                blur: blur * n.scale,
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
                        let th = d
                            .thickness
                            .map_or((size * 0.07).max(1.0 / dpr), |t| t * n.scale);
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
                            rule(mid - size * 0.5 - d.offset.map_or(size * 0.12, |o| o * n.scale));
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
            let d = s.decoration.clone().unwrap_or_default();
            let th = d
                .thickness
                .map_or((size * 0.07).max(1.0 / dpr), |t| t * n.scale);
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
                let off = d.offset.map_or(size * 0.12, |o| o * n.scale);
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

/// A text command's clip, through its effects.
fn text_clip(c: &mut crate::draw::DrawCmd) -> Option<&mut Rect> {
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
fn vertical_decoration_line(
    x: f32,
    y: f32,
    length: f32,
    th: f32,
    style: crate::doc::text_css::DecorationStyle,
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

/// One decoration line of `style` from `x` across `width`, its top at `y`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn decoration_line(
    x: f32,
    width: f32,
    y: f32,
    th: f32,
    style: crate::doc::text_css::DecorationStyle,
    color: [f32; 4],
    clip: Rect,
    dpr: f32,
    cmds: &mut Vec<crate::draw::DrawCmd>,
) {
    use crate::doc::text_css::DecorationStyle as S;
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
