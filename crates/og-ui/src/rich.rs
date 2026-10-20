//! Rich text: one paragraph of inline spans (colour, weight, size, font,
//! inline icons, links and per-glyph motion) from markup ([`crate::markup`]),
//! laid out as one text: the lines break between the spans' words (a word
//! split across two spans stays whole), each line as tall as its tallest
//! span, every span on one shared baseline.
//!
//! Links are children of the paragraph for the probe and for input: each
//! `[link=<id>]` is an element `<paragraph>/<id>` (kind `link`) with its
//! bounds (the union of its pieces), reachable by keyboard and gamepad
//! focus, pressed like a button; [`crate::Ui::rich_text`] answers which
//! was clicked this tick.

use crate::id::child_key;

use crate::layout::{Content, Element, FontRole, Metrics, Rect, Size, TextAlign};
use crate::markup::{SpanFx, parse_markup, plain_text};
use crate::style::{Srgba, TypeStep};
use crate::ui::{Response, Ui};

/// One resolved span: its text (or icon) in its own style.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RichSpan {
    pub text: String,
    pub step: TypeStep,
    pub color: Srgba,
    pub font: FontRole,
    pub bold: bool,
    /// An inline icon: its cell in the icon atlas.
    pub icon: Option<u32>,
    pub link: Option<String>,
    pub fx: SpanFx,
}

/// A paragraph of spans.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RichContent {
    pub spans: Vec<RichSpan>,
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
}

/// One laid-out line of a paragraph.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct RichLine {
    pub pieces: Vec<Piece>,
    pub width: f32,
    pub height: f32,
    /// From the line's top to the shared baseline.
    pub baseline: f32,
}

/// How rich text looks by default (`RichTextDesc` in the IDL).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RichTextDesc {
    /// `None`: `text-body`.
    pub step: Option<TypeStep>,
    /// `None`: `fg`.
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
}

/// A unit lines break between: a span's word, space or CJK character.
struct Unit {
    span: usize,
    text: String,
    space: bool,
    width: f32,
    /// Glued to the unit before it (the same word continued in another span).
    glued: bool,
}

pub(crate) fn span_width(s: &RichSpan, text: &str, fonts: &dyn Metrics) -> f32 {
    if s.icon.is_some() {
        return s.step.size * 1.1;
    }
    let m = fonts.metrics_for(s.font, text);
    m.measure_tracked(
        text,
        s.step.size,
        crate::layout::step_tracking(&s.step, text),
    )
}

fn span_metrics(s: &RichSpan, fonts: &dyn Metrics) -> (f32, f32, f32) {
    // (line height, ascent, descent) in CSS px.
    if s.icon.is_some() {
        return (s.step.line_height, s.step.size * 0.85, -s.step.size * 0.25);
    }
    let m = fonts.metrics_for(s.font, &s.text);
    let lh = if s.text.chars().any(crate::text::is_cjk) {
        s.step.cjk_line_height
    } else {
        s.step.line_height
    };
    (lh, m.ascender * s.step.size, m.descender * s.step.size)
}

/// The paragraphs' units (paragraphs split at `\n`).
fn paragraphs(rc: &RichContent, fonts: &dyn Metrics) -> Vec<Vec<Unit>> {
    let mut out: Vec<Vec<Unit>> = vec![Vec::new()];
    for (k, s) in rc.spans.iter().enumerate() {
        if s.icon.is_some() {
            let glued = out.last().and_then(|p| p.last()).is_some_and(|u| !u.space);
            out.last_mut().expect("a paragraph").push(Unit {
                span: k,
                text: String::new(),
                space: false,
                width: span_width(s, "", fonts),
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
                    width: span_width(s, t, fonts),
                    glued,
                });
            }
        }
    }
    out
}

/// The widest paragraph unbroken, and the widest unbreakable group.
pub(crate) fn rich_widths(rc: &RichContent, fonts: &dyn Metrics) -> (f32, f32) {
    let mut pref = 0.0f32;
    let mut min = 0.0f32;
    for para in paragraphs(rc, fonts) {
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

/// Greedy lines at `max` (`None`: only at `\n`).
pub(crate) fn rich_lines(rc: &RichContent, fonts: &dyn Metrics, max: Option<f32>) -> Vec<RichLine> {
    let mut lines = Vec::new();
    for para in paragraphs(rc, fonts) {
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
            Some(p) if p.span == u.span && rc.spans[u.span].icon.is_none() => {
                p.text.push_str(&u.text);
                p.width += u.width;
            }
            _ => pieces.push(Piece {
                span: u.span,
                text: u.text.clone(),
                x,
                width: u.width,
            }),
        }
        x += u.width;
    }
    let (mut h, mut a, mut d) = (0.0f32, 0.0f32, 0.0f32);
    for p in &pieces {
        let (lh, asc, desc) = span_metrics(&rc.spans[p.span], fonts);
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
    RichLine {
        width: x,
        height: h,
        baseline,
        pieces,
    }
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
struct Segment {
    span: usize,
    text: String,
    glyphs: Option<Vec<crate::text::VisualGlyph>>,
    width: f32,
}

/// A rich line's segments in visual order. A line of left-to-right text in
/// a left-to-right UI is its pieces as laid out; any other line runs the
/// bidi algorithm over its whole text (icons as U+FFFC) at the UI's
/// direction and cuts the runs at the pieces' edges.
fn visual_segments(
    rc: &RichContent,
    fonts: &dyn Metrics,
    line: &RichLine,
    direction: og_i18n::Direction,
) -> Vec<Segment> {
    let plain = direction == og_i18n::Direction::Ltr
        && line
            .pieces
            .iter()
            .all(|p| !crate::text::needs_visual(&p.text));
    if plain {
        return line
            .pieces
            .iter()
            .map(|p| Segment {
                span: p.span,
                text: p.text.clone(),
                glyphs: None,
                width: p.width,
            })
            .collect();
    }
    // The line's text and where each piece's text lies in it.
    let mut text = String::new();
    let mut ranges = Vec::with_capacity(line.pieces.len());
    for p in &line.pieces {
        let start = text.len();
        if rc.spans[p.span].icon.is_some() {
            text.push('\u{fffc}');
        } else {
            text.push_str(&p.text);
        }
        ranges.push(start..text.len());
    }
    let mut out = Vec::new();
    for (run, rtl) in crate::text::bidi_runs(&text, direction) {
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
            if s.icon.is_some() {
                out.push(Segment {
                    span: p.span,
                    text: String::new(),
                    glyphs: None,
                    width: p.width,
                });
                continue;
            }
            let piece_text = &text[r.clone()];
            let m = fonts.metrics_for(s.font, piece_text);
            let tracking = crate::layout::step_tracking(&s.step, piece_text);
            let needs = rtl || crate::text::needs_visual(piece_text);
            if !needs {
                out.push(Segment {
                    span: p.span,
                    text: piece_text.to_string(),
                    glyphs: None,
                    width: span_width(s, piece_text, fonts),
                });
                continue;
            }
            let first = text[ranges[k].start..r.start].chars().count();
            let glyphs = m.run(piece_text, rtl, tracking, first);
            let width = glyphs.iter().map(|g| g.advance).sum::<f32>() * s.step.size;
            out.push(Segment {
                span: p.span,
                text: piece_text.to_string(),
                glyphs: Some(glyphs),
                width,
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
    /// colour (`[color=accent]`, `[color=#ff8800]`), weight (`[b]`), size
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
        let t = self.theme;
        let (parsed, warnings) = parse_markup(markup);
        self.warnings.extend(warnings);
        let base_step = desc.step.unwrap_or(t.type_ramp.body);
        let base_color = desc.color.unwrap_or(t.palette.fg);
        let mut spans = Vec::with_capacity(parsed.len());
        for s in &parsed {
            let mut step = base_step;
            if let Some(name) = &s.size {
                let mut ramp = t.type_ramp;
                match crate::skin::ramp_step(&mut ramp, name) {
                    Some(st) => step = *st,
                    None => self
                        .warnings
                        .push(format!("rich text: [size={name}] is not a TypeStepName")),
                }
            }
            let mut color = if s.link.is_some() {
                t.palette.accent_ink
            } else {
                base_color
            };
            if let Some(name) = &s.color {
                let mut palette = t.palette;
                match (
                    crate::skin::parse_color(name),
                    crate::skin::palette_color(&mut palette, name),
                ) {
                    (Some(c), _) => color = c,
                    (None, Some(c)) => color = *c,
                    _ => self.warnings.push(format!(
                        "rich text: [color={name}] is neither a ThemeColor nor #rrggbb"
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
            let icon = match &s.icon {
                Some(name) => match self.icons.index_of(name).filter(|_| self.icons.is_drawn()) {
                    Some(k) => Some(k),
                    None => {
                        self.warnings
                            .push(format!("rich text: [icon={name}] is not in the icon atlas"));
                        None
                    }
                },
                None => None,
            };
            if s.icon.is_some() && icon.is_none() {
                continue;
            }
            spans.push(RichSpan {
                text: s.text.clone(),
                step,
                color,
                font,
                bold: s.bold,
                icon,
                link: s.link.clone(),
                fx: s.fx,
            });
        }
        let i = self.leaf(
            "richtext",
            id.into(),
            Element {
                width: desc.width,
                ..Element::default()
            },
        );
        let key = self.nodes[i].key;
        let mut out = RichTextResponse::default();
        let mut links: Vec<String> = Vec::new();
        for s in &spans {
            if let Some(l) = &s.link
                && !links.contains(l)
            {
                links.push(l.clone());
            }
        }
        for l in &links {
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
        let n = &mut self.nodes[i];
        n.probe_text = plain_text(&parsed);
        n.content = Content::Rich(RichContent {
            spans,
            align: desc.align,
            wrap: !desc.nowrap,
        });
        n.links = links;
        out
    }

    /// Where each piece of rich node `i` sits on screen, after layout and
    /// motion, in visual order: each line is laid out by the bidirectional
    /// algorithm over its whole text at the UI's direction (UAX #9), so
    /// pieces are reordered and so are runs of either direction within one
    /// piece (a piece split by a direction change is several segments).
    pub(crate) fn rich_pieces(&self, i: usize) -> Vec<DrawnPiece> {
        let n = &self.nodes[i];
        let Content::Rich(rc) = &n.content else {
            return Vec::new();
        };
        let sk = n.scale;
        let pad = n.el.padding;
        let inner = Rect::new(
            n.rect.x + pad.left * sk,
            n.rect.y + pad.top * sk,
            n.rect.width - pad.horizontal() * sk,
            n.rect.height - pad.vertical() * sk,
        );
        let rtl = self.direction == og_i18n::Direction::Rtl;
        let mut out = Vec::new();
        let mut y = inner.y;
        for line in &n.rich_lines {
            let segments = visual_segments(rc, &self.fonts, line, self.direction);
            let width: f32 = segments.iter().map(|s| s.width).sum();
            let free = inner.width - width * sk;
            let x0 = inner.x
                + match (rc.align, rtl) {
                    (TextAlign::Start, false) | (TextAlign::End, true) => 0.0,
                    (TextAlign::Center, _) => free / 2.0,
                    (TextAlign::End, false) | (TextAlign::Start, true) => free,
                };
            let mut x = x0;
            for seg in segments {
                out.push(DrawnPiece {
                    span: seg.span,
                    text: seg.text,
                    glyphs: seg.glyphs,
                    rect: Rect::new(x, y, seg.width * sk, line.height * sk),
                    baseline: y + line.baseline * sk,
                });
                x += seg.width * sk;
            }
            y += line.height * sk;
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
        let t = &self.theme;
        let p = &t.palette;
        let alpha = n.alpha;
        let dpr = self.viewport.dpr();
        let snap = |v: f32| (v * dpr).round() / dpr;
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
                        color: self.behind(i).fade(alpha).linear(),
                        clip,
                    });
                }
            }
        }
        for piece in self.rich_pieces(i) {
            let (k, text, r, baseline) = (piece.span, piece.text, piece.rect, piece.baseline);
            let s = &rc.spans[k];
            let size = s.step.size * n.scale;
            let link_key = s.link.as_deref().map(|l| child_key(n.key, l));
            let hot = link_key.is_some_and(|lk| self.state.hot == lk);
            let color = if hot { p.accent_hover } else { s.color };
            if let Some(cell) = s.icon {
                let side = s.step.size * n.scale;
                if let Some((font, ch)) = self.icons.glyph(cell) {
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
                } else if let Some(tex) = self.icons.texture {
                    cmds.push(DrawCmd::Image {
                        rect: Rect::new(r.x + side * 0.05, baseline - side * 0.85, side, side)
                            .snapped(dpr),
                        texture: Some(tex),
                        uv: self.icons.uv(cell),
                        color: color.fade(alpha).linear(),
                        clip,
                    });
                }
                continue;
            }
            let font = self.fonts.for_text(s.font, &text);
            let Some(handle) = font.handle else { continue };
            let m = &font.metrics;
            let tracking = crate::layout::step_tracking(&s.step, &text);
            // Bold: thickened in the shader (as far as the field reaches)
            // and drawn twice, the second copy one device pixel to the
            // right, so small text whose field is thin is bold too.
            let weight = if s.bold { (size * 0.045).max(0.4) } else { 0.0 };
            let smear = if s.bold { 1.0 / dpr } else { 0.0 };
            let glyph = |x: f32,
                         dy: f32,
                         a: f32,
                         text: String,
                         placements: Vec<og_render::GlyphPlacement>,
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
            if let Some(glyphs) = piece.glyphs {
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
                    let (dy, a) = match s.fx {
                        SpanFx::Wave => (-0.14 * size * sine(tick * 0.16 + g as f32 * 0.7), 1.0),
                        SpanFx::Shake => {
                            // A fixed hash of the glyph and the tick's half: jitter, no clock.
                            let h = (g as u32).wrapping_mul(2_654_435_761)
                                ^ ((tick as u32) / 2).wrapping_mul(40_503);
                            (((h % 100) as f32 / 100.0 - 0.5) * 0.12 * size, 1.0)
                        }
                        _ => (0.0, 0.7 + 0.3 * sine(tick * 0.1)),
                    };
                    if smear > 0.0 {
                        cmds.push(glyph(x + smear, dy, a, cs.clone(), Vec::new(), w));
                    }
                    cmds.push(glyph(x, dy, a, cs, Vec::new(), w));
                    x += w;
                }
            }
            if link_key.is_some() {
                // The underline: a hairline under the piece, in its ink.
                let th = (size * 0.07).max(1.0 / dpr);
                cmds.push(DrawCmd::Quad {
                    rect: Rect::new(r.x, baseline + size * 0.12, r.width, th).snapped(dpr),
                    radius: 0.0,
                    color: color.fade(alpha).linear(),
                    clip,
                });
            }
        }
    }
}
