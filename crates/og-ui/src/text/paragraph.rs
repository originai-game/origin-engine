//! A rich paragraph laid out by Parley as one inline formatting context
//! (CSS 2 §9.4.2, §10.8): one bidi analysis and UAX #14 line breaker over
//! all its spans, atomic content as inline boxes, each line box holding
//! the paragraph's strut, its spans' line heights and their
//! `vertical-align`. Lines beside floats break one at a time at the room
//! each leaves ([`lines_beside`]). The discretionary hyphen, which Parley
//! breaks at but does not draw or reserve, is og-ui's: a line that ends
//! at a SHY breaks again with room for its hyphen and is shaped once more
//! with the hyphen shown.

use super::VisualGlyph;
use crate::layout::float::Broken;
use crate::rich::{Edge, LineBreak, Piece, RichContent, RichLine, Segment, TextIndent};
use og_i18n::Direction;
use og_text::text::paragraph::{Brush, CONTEXT, Context};
use parley::{BreakReason, Layout, PositionedLayoutItem, StyleProperty, YieldData};

/// The room beside a line at `y` as tall as `h`: its left, its width, and
/// the floats' next bottom (`None`: no float narrows it).
pub(crate) type Room<'a> = &'a dyn Fn(f32, f32) -> (f32, f32, Option<f32>);

/// The paragraph's lines at `width` (`None`: only at forced breaks) and
/// its widest and narrowest content widths. `None` when a face it chose
/// cannot be shaped.
pub(crate) fn rich(
    content: &RichContent,
    fonts: &dyn crate::layout::Metrics,
    width: Option<f32>,
) -> Option<(Vec<RichLine>, (f32, f32))> {
    let w = width.unwrap_or(f32::MAX);
    let (broken, widths) = laid_out(content, fonts, None, &|_, _| (0.0, w, None))?;
    Some((broken.0, widths))
}

/// The paragraph's lines from its top, each broken at the room `room`
/// leaves beside it; a line no word fits in moves down to the floats'
/// next bottom. `None` when a face it chose cannot be shaped.
pub(crate) fn lines_beside(
    content: &RichContent,
    fonts: &dyn crate::layout::Metrics,
    room: Room<'_>,
) -> Option<Broken> {
    Some(laid_out(content, fonts, None, room)?.0)
}

/// Where a line broke: its byte range, its left, drop and width beside
/// the floats, and whether it ends at a soft wrap opportunity.
struct Broke {
    range: std::ops::Range<usize>,
    shift: [f32; 3],
    wrapped: bool,
}

fn laid_out(
    content: &RichContent,
    fonts: &dyn crate::layout::Metrics,
    direction: Option<Direction>,
    room: Room<'_>,
) -> Option<(Broken, (f32, f32))> {
    let shy = content.spans.iter().any(|s| s.text.contains('\u{ad}'));
    let hyphens = if shy {
        hyphens(content, fonts)
    } else {
        Vec::new()
    };
    // Each line's room held back for its hyphen, found by the passes
    // before (a line ending at a SHY whose hyphen does not fit).
    let mut reserve: Vec<f32> = Vec::new();
    // What each line holds back or breaks within for its inline boxes'
    // edges, found by the breaks before.
    let mut fit: Vec<LineFit> = Vec::new();
    let boxed = content.spans.iter().any(|s| s.inline_box.is_some());
    loop {
        let mut planned = Vec::new();
        let pass = shaped(content, fonts, direction, |layout, text, indices| {
            let widths = layout.calculate_content_widths();
            // Broken again over the same shaping until each inline box's
            // edges sit with its text (Parley may break beside any inline
            // box); a bound stops a fit that will not settle.
            let mut tries = 0;
            let (broke, narrowed, drop) = loop {
                let broken = break_beside(layout, content, &mut reserve, &mut fit, room);
                tries += 1;
                if !boxed || tries > 64 || !refit(&mut fit, content, layout, text, direction) {
                    break broken;
                }
            };
            // A line ending at a SHY: its hyphen, which a pass must make
            // room for when it does not fit.
            for (k, b) in broke.iter().enumerate() {
                let Some(span) = b
                    .wrapped
                    .then(|| ends_at_shy(content, text, &b.range))
                    .flatten()
                else {
                    continue;
                };
                if reserve.get(k).is_some_and(|&r| r < 0.0) {
                    continue;
                }
                let line = layout.lines().nth(k).expect("broken line");
                let m = line.metrics();
                let width = m.advance - m.hanging_advance + m.indent + hyphens[span];
                if width > b.shift[2] + 0.01 && reserve.get(k).is_none_or(|&r| r == 0.0) {
                    return Err((k, hyphens[span]));
                }
            }
            let mut lines = Vec::new();
            for (k, (line, b)) in layout.lines().zip(&broke).enumerate() {
                if b.wrapped
                    && reserve.get(k).copied().unwrap_or(0.0) >= 0.0
                    && let Some(span) = ends_at_shy(content, text, &b.range)
                {
                    planned.push((k, planned_line(content, text, &line, span, &b.range)));
                    lines.push(RichLine::default());
                    continue;
                }
                let edges = if boxed {
                    cloned(content, text, &b.range, direction)
                } else {
                    [None; 2]
                };
                let rtl = boxed && rtl_at(text, b.range.start, direction);
                lines.push(rich_line(content, fonts, text, indices, &line, edges, rtl));
            }
            let shifts = broke.iter().map(|b| b.shift).collect();
            Ok(((lines, shifts, narrowed, drop), (widths.max, widths.min)))
        })?;
        match pass {
            Err((k, hyphen)) => {
                reserve.resize(reserve.len().max(k + 1), 0.0);
                reserve[k] = hyphen;
            }
            Ok((mut broken, widths)) => {
                // Shape each hyphenated line again outside the font
                // context's borrow: contextual glyphs and bidi see the
                // visible hyphen; the spans keep their ink, links and
                // atomic content.
                for (k, line) in planned {
                    broken.0[k] = reshape(line, fonts)?;
                }
                return Some((broken, widths));
            }
        }
    }
}

/// Breaks the layout's lines one at a time at the room beside each
/// (`reserve`: what each line holds back for its hyphen; negative where
/// holding it back left only an emergency break, so the line breaks at
/// its SHY as if it were not there, its hyphen not shown): a line taller
/// than the room asked for asks again as tall as it is, and a line wider
/// than its room moves down to the floats' next bottom. Each line's range
/// and shift, whether a float narrowed one, and the drop.
fn break_beside(
    layout: &mut Layout<Brush>,
    content: &RichContent,
    reserve: &mut [f32],
    fit: &mut [LineFit],
    room: Room<'_>,
) -> (Vec<Broke>, bool, f32) {
    let guess = content.base.0.line_height;
    let mut out: Vec<[f32; 3]> = Vec::new();
    let (mut y, mut drop, mut narrowed) = (0.0f32, 0.0f32, false);
    {
        let mut lines = layout.break_lines();
        lines.state_mut().set_layout_max_advance(f32::INFINITY);
        'lines: loop {
            let start = lines.state().clone();
            let mut h = guess;
            loop {
                let (l, avail, next) = room(y, h);
                let held = reserve.get(out.len()).copied().unwrap_or(0.0).max(0.0);
                let f = fit.get(out.len()).copied().unwrap_or_default();
                let state = lines.state_mut();
                state.set_line_max_advance((avail - held - f.room).max(0.0).min(f.cap));
                state.set_line_y(f64::from(y));
                let Some(YieldData::LineBreak(d)) = lines.break_next() else {
                    break 'lines;
                };
                if held > 0.0 && d.reason == BreakReason::Emergency {
                    // No break leaves room for the hyphen.
                    lines.revert_to(start.clone());
                    reserve[out.len()] = -1.0;
                    continue;
                }
                if f.cap.is_finite() && d.reason == BreakReason::Emergency {
                    // No break keeps the edge with its text: it stays.
                    lines.revert_to(start.clone());
                    fit[out.len()].cap = f32::INFINITY;
                    fit[out.len()].stuck = true;
                    continue;
                }
                let height = d.metrics.line_height;
                let width = d.metrics.advance - d.metrics.hanging_advance + d.metrics.indent;
                if next.is_some() && height > h + 0.01 {
                    lines.revert_to(start.clone());
                    h = height;
                    continue;
                }
                if let Some(b) = next
                    && width > avail + 0.5
                    && b > y
                {
                    lines.revert_to(start.clone());
                    drop += b - y;
                    y = b;
                    h = guess;
                    continue;
                }
                narrowed |= next.is_some();
                out.push([l, drop, avail]);
                y += height;
                if lines.is_done() {
                    break 'lines;
                }
                break;
            }
        }
        lines.finish();
    }
    let broke = layout
        .lines()
        .zip(out)
        .map(|(line, shift)| Broke {
            range: line.text_range(),
            shift,
            wrapped: line.break_reason() == BreakReason::Regular,
        })
        .collect();
    (broke, narrowed, drop)
}

/// The span whose SHY a line's text ends at, past its hanging spaces.
fn ends_at_shy(content: &RichContent, text: &str, range: &std::ops::Range<usize>) -> Option<usize> {
    let line = text[range.clone()].trim_end_matches([' ', '\t']);
    if !line.ends_with('\u{ad}') {
        return None;
    }
    let at = range.start + line.len() - 1;
    let mut start = 0;
    content.spans.iter().position(|s| {
        let len = if s.atom.is_some() { 0 } else { s.text.len() };
        let hit = (start..start + len).contains(&at);
        start += len;
        hit
    })
}

/// A Parley line as og-ui draws it: its runs and inline boxes in visual
/// order, each raised from the line's baseline as Parley set it.
/// `edges`: a cloned box's start and end edges where the line breaks
/// inside it (their spans and widths), set at its start and end (`rtl`:
/// from the right).
fn rich_line(
    content: &RichContent,
    fonts: &dyn crate::layout::Metrics,
    text: &str,
    indices: &[usize],
    line: &parley::Line<'_, Brush>,
    edges: [Option<(usize, f32)>; 2],
    rtl: bool,
) -> RichLine {
    let m = line.metrics();
    let mut segments: Vec<Segment> = Vec::new();
    // Each segment's first character, counted over the segments' text.
    let mut at = 0;
    for item in line.items() {
        match item {
            PositionedLayoutItem::InlineBox(b) => {
                let (span, edge) = match edge_of(b.id) {
                    Some((span, edge)) => (span, Some(edge)),
                    None => (b.id as usize, None),
                };
                segments.push(Segment {
                    span,
                    text: String::new(),
                    at,
                    glyphs: None,
                    width: b.width,
                    rise: m.baseline - (b.y + b.baseline.unwrap_or(b.height)),
                    edge,
                });
            }
            PositionedLayoutItem::GlyphRun(glyph_run) => {
                let brush = glyph_run.style().brush;
                let s = &content.spans[brush.span];
                let metrics = fonts.metrics_styled(s.font, s.query(), &s.text);
                let font = metrics.part(brush.font as usize).expect("selected face");
                let file = font.file.as_ref().expect("cooked font");
                let run = glyph_run.run();
                let range = run.text_range();
                let mut glyphs = Vec::new();
                for cluster in run.visual_clusters().filter(|c| c.style().brush == brush) {
                    let cr = cluster.text_range();
                    let source = text[cr.clone()].chars().next().unwrap_or(' ');
                    let index = indices[cr.start] - indices[range.start];
                    for g in cluster.glyphs() {
                        let (ch, glyph) =
                            file.character_for_glyph(g.id, source, &|k| font.in_atlas_key(k));
                        glyphs.push(VisualGlyph {
                            ch,
                            glyph,
                            advance: g.advance / s.step.size,
                            x: g.x / s.step.size,
                            y: -g.y / s.step.size,
                            index,
                            rtl: run.is_rtl(),
                            font: brush.font,
                        });
                    }
                }
                segments.push(Segment {
                    span: brush.span,
                    text: text[range.clone()].to_owned(),
                    at,
                    glyphs: Some(glyphs),
                    width: glyph_run.advance(),
                    rise: m.baseline - glyph_run.baseline(),
                    edge: None,
                });
                at += text[range].chars().count();
            }
        }
    }
    // A cloned box's edges at the break: its start before the line's
    // first segment, its end after its last (the other way round from
    // the right).
    let edge = |(span, width): (usize, f32), edge: Edge, at: usize| Segment {
        span,
        text: String::new(),
        at,
        glyphs: None,
        width,
        rise: segments
            .iter()
            .find(|s| s.span == span)
            .map_or(0.0, |s| s.rise),
        edge: Some(edge),
    };
    let [start, end] = edges;
    let (front, back) = if rtl {
        ((end, Edge::End), (start, Edge::Start))
    } else {
        ((start, Edge::Start), (end, Edge::End))
    };
    let back = back.0.map(|e| edge(e, back.1, at));
    if let Some(e) = front.0 {
        segments.insert(0, edge(e, front.1, 0));
    }
    segments.extend(back);
    let mut pieces = Vec::new();
    let mut x = 0.0;
    for segment in &segments {
        pieces.push(Piece {
            span: segment.span,
            text: segment.text.clone(),
            x,
            width: segment.width,
            rise: segment.rise,
            edge: segment.edge,
        });
        x += segment.width;
    }
    let mut line = RichLine {
        pieces,
        // Its indent is room at its start (CSS Text 3 §8.1), part of
        // the width its line box takes.
        width: x + m.indent,
        indent: m.indent,
        height: m.line_height,
        baseline: m.baseline - m.block_min_coord,
        shaped: Some(segments),
    };
    crate::rich::emphasis::make_room(content, fonts, &mut line);
    line
}

/// Parley's `vertical-align` for a span of font `size` and line height
/// `line`.
fn valign(v: crate::VerticalAlign, size: f32, line: f32) -> parley::VerticalAlign {
    use crate::VerticalAlign as V;
    match v {
        V::Baseline => parley::VerticalAlign::BASELINE,
        V::Sub => parley::VerticalAlign::SUB,
        V::Super => parley::VerticalAlign::SUPER,
        V::TextTop => parley::VerticalAlign::TEXT_TOP,
        V::TextBottom => parley::VerticalAlign::TEXT_BOTTOM,
        V::Middle => parley::VerticalAlign::MIDDLE,
        V::Top => parley::VerticalAlign::TOP,
        V::Bottom => parley::VerticalAlign::BOTTOM,
        V::Px(px) => parley::VerticalAlign::length(px),
        V::Em(em) => parley::VerticalAlign::length(em * size),
        V::Percent(p) => parley::VerticalAlign::length(p * line),
    }
}

/// A span's line height: its step's, or its CJK one for CJK text.
fn line_height(s: &crate::rich::RichSpan) -> f32 {
    if s.text.chars().any(crate::text::is_cjk) {
        s.step.cjk_line_height
    } else {
        s.step.line_height
    }
}

/// The spans' paragraph shaped as one layout, handed to `f` with its text
/// and each byte's character index. The layout's root style is the
/// paragraph's own text (its strut); each span sets its face, size, line
/// height, tracking, features and `vertical-align`. `None` when a chosen
/// face cannot be shaped.
fn shaped<R>(
    content: &RichContent,
    fonts: &dyn crate::layout::Metrics,
    direction: Option<Direction>,
    f: impl FnOnce(&mut Layout<Brush>, &str, &[usize]) -> R,
) -> Option<R> {
    use crate::rich::{span_tracking, span_width};
    let mut text = String::new();
    let mut ranges = Vec::new();
    let mut boxes = Vec::new();
    let mut selected = Vec::new();
    // The inline boxes' edges among `boxes`, their widths set once the
    // text is whole (they take their paragraph's direction).
    let mut edges = Vec::new();
    let same = |k: Option<usize>, b: &crate::rich::InlineBox| {
        k.and_then(|k| content.spans.get(k)?.inline_box.as_ref())
            .is_some_and(|o| o.id == b.id)
    };
    let mut edge = |boxes: &mut Vec<parley::InlineBox>, span: usize, at: usize, e: Edge| {
        let s = &content.spans[span];
        edges.push((boxes.len(), span, e));
        boxes.push(parley::InlineBox {
            id: edge_id(span, e),
            index: at,
            kind: parley::InlineBoxKind::InFlow,
            width: 0.0,
            height: 0.0,
            baseline: Some(0.0),
            vertical_align: valign(s.valign, s.step.size, s.step.line_height),
        });
    };
    for (span, s) in content.spans.iter().enumerate() {
        let start = text.len();
        // An inline box's start edge before its first span, unless (on a
        // hyphenated line alone) it began on a line before.
        if let Some(b) = &s.inline_box
            && !same(span.checked_sub(1), b)
            && (!b.open[0] || b.clone)
        {
            edge(&mut boxes, span, start, Edge::Start);
        }
        if s.atom.is_some() {
            // A ruby's box holds its base and, beyond the leading, its
            // annotation (CSS Ruby 1 §3.6).
            let (height, baseline) = match &s.atom {
                Some(crate::rich::Atom::Ruby(r)) => {
                    let (above, below) = crate::rich::ruby_extents(s, r, fonts);
                    (above + below, above)
                }
                _ => (s.step.size * 1.1, s.step.size * 0.85),
            };
            boxes.push(parley::InlineBox {
                id: span as u64,
                index: start,
                kind: parley::InlineBoxKind::InFlow,
                width: span_width(s, "", fonts),
                height,
                baseline: Some(baseline),
                vertical_align: valign(s.valign, s.step.size, s.step.line_height),
            });
        } else {
            text.push_str(&s.text);
            let metrics = fonts.metrics_styled(s.font, s.query(), &s.text);
            for (range, k) in metrics.font_runs(&s.text) {
                let m = metrics.part(k)?;
                if m.fixed.is_some() || m.pixel != 0 {
                    return None;
                }
                selected.push((
                    start + range.start..start + range.end,
                    Brush {
                        font: k as u8,
                        span,
                    },
                    m.file.clone()?,
                ));
            }
        }
        if let Some(b) = &s.inline_box
            && !same(Some(span + 1), b)
            && (!b.open[1] || b.clone)
        {
            edge(&mut boxes, span, text.len(), Edge::End);
        }
        ranges.push(start..text.len());
    }
    for (k, span, e) in edges {
        let b = content.spans[span]
            .inline_box
            .as_ref()
            .expect("a boxed span");
        let [start, end] = b.edges(rtl_at(&text, boxes[k].index, direction));
        boxes[k].width = if e == Edge::Start { start } else { end };
    }
    // The strut: the paragraph's own face at its step.
    let (base, role) = content.base;
    let strut = {
        let m = fonts.metrics_styled(role, crate::FontQuery::new(base.weight, false), "");
        let part = m.part(0)?;
        if part.fixed.is_some() || part.pixel != 0 {
            return None;
        }
        part.file.clone()?
    };
    let mut indices = vec![0; text.len() + 1];
    for (index, (byte, _)) in text.char_indices().enumerate() {
        indices[byte] = index;
    }
    CONTEXT.with_borrow_mut(|cx| {
        if cx.files.len() > 128 {
            *cx = Context::default();
        }
        let families: Vec<_> = selected
            .iter()
            .map(|(_, _, f)| cx.family(f.clone()))
            .collect();
        let strut = cx.family(strut);
        let mut builder = cx.layout.ranged_builder(&mut cx.fonts, &text, 1.0, false);
        builder.set_line_break_override(Some(&super::keep_katakana));
        if let Some(direction) = direction {
            builder.set_base_direction(match direction {
                Direction::Ltr => parley::BaseDirection::Ltr,
                Direction::Rtl => parley::BaseDirection::Rtl,
            });
        }
        builder.push_default(StyleProperty::OverflowWrap(parley::OverflowWrap::Anywhere));
        builder.push_default(StyleProperty::LineBreak(match content.line_break {
            LineBreak::Auto | LineBreak::Strict => parley::LineBreak::Strict,
            LineBreak::Loose => parley::LineBreak::Loose,
            LineBreak::Normal => parley::LineBreak::Normal,
            LineBreak::Anywhere => parley::LineBreak::Anywhere,
        }));
        builder.push_default(StyleProperty::FontFamily(strut.as_str().into()));
        builder.push_default(StyleProperty::FontSize(base.size));
        builder.push_default(StyleProperty::LineHeight(parley::LineHeight::Absolute(
            base.line_height,
        )));
        for ((range, brush, _), family) in selected.iter().zip(&families) {
            builder.push(
                StyleProperty::FontFamily(family.as_str().into()),
                range.clone(),
            );
            builder.push(StyleProperty::Brush(*brush), range.clone());
        }
        for (s, range) in content.spans.iter().zip(&ranges) {
            builder.push(StyleProperty::FontSize(s.step.size), range.clone());
            builder.push(
                StyleProperty::LineHeight(parley::LineHeight::Absolute(line_height(s))),
                range.clone(),
            );
            builder.push(
                StyleProperty::VerticalAlign(valign(s.valign, s.step.size, line_height(s))),
                range.clone(),
            );
            builder.push(
                StyleProperty::LetterSpacing(span_tracking(s, &s.text) * s.step.size),
                range.clone(),
            );
            builder.push(StyleProperty::WordSpacing(s.word_spacing), range.clone());
            let features = s.step.features.css();
            builder.push(
                StyleProperty::FontFeatures(features.as_str().into()),
                range.clone(),
            );
        }
        for atom in boxes {
            builder.push_inline_box(atom);
        }
        let mut layout = builder.build(&text);
        let TextIndent {
            px,
            hanging,
            each_line,
        } = content.indent;
        layout.set_text_indent(px, parley::IndentOptions { each_line, hanging });
        Some(f(&mut layout, &text, &indices))
    })
}

/// A hyphenated line's spans (renumbered, its SHY shown as a hyphen), their
/// indices in the paragraph, and its paragraph's base direction.
type Planned = (RichContent, Vec<usize>, Direction);

/// The line `range` holds as spans of its own, the SHY of span `shy` at
/// its end shown as a hyphen and its hanging spaces gone.
fn planned_line(
    content: &RichContent,
    text: &str,
    line: &parley::Line<'_, Brush>,
    shy: usize,
    range: &std::ops::Range<usize>,
) -> Planned {
    let atoms: Vec<usize> = line
        .items()
        .filter_map(|item| match item {
            PositionedLayoutItem::InlineBox(b) if edge_of(b.id).is_none() => Some(b.id as usize),
            _ => None,
        })
        .collect();
    let end = range.start + text[range.clone()].trim_end_matches([' ', '\t']).len();
    // One line laid out alone: its own indent as a first line's.
    let mut out = RichContent {
        spans: Vec::new(),
        indent: TextIndent {
            px: line.metrics().indent,
            ..TextIndent::default()
        },
        ..content.clone()
    };
    let mut spans = Vec::new();
    let mut start = 0;
    for (k, s) in content.spans.iter().enumerate() {
        if s.atom.is_some() {
            if atoms.contains(&k) {
                out.spans.push(s.clone());
                spans.push(k);
            }
            continue;
        }
        let (a, b) = (start.max(range.start), (start + s.text.len()).min(end));
        start += s.text.len();
        if a >= b {
            continue;
        }
        let mut span = s.clone();
        span.text = text[a..b].to_owned();
        // An inline box the line's ends cut: no edge there under slice.
        if let Some(ib) = &mut span.inline_box {
            let open = [
                a == range.start && inside(content, text, a).is_some(),
                b == end && inside(content, text, range.end).is_some(),
            ];
            if open != [false; 2] {
                std::sync::Arc::make_mut(ib).open = open;
            }
        }
        if k == shy && b == end {
            span.text.pop();
            span.text.push('-');
        }
        out.spans.push(span);
        spans.push(k);
    }
    // Its paragraph's direction: the text between the forced breaks
    // around it.
    let from = text[..range.start].rfind('\n').map_or(0, |p| p + 1);
    let to = text[range.start..]
        .find('\n')
        .map_or(text.len(), |p| range.start + p);
    let base = og_i18n::paragraph_direction(&text[from..to], Direction::Ltr);
    (out, spans, base)
}

/// A planned line shaped on its own (with its paragraph's direction), its
/// spans numbered as the paragraph's.
fn reshape((line, spans, base): Planned, fonts: &dyn crate::layout::Metrics) -> Option<RichLine> {
    let w = f32::MAX;
    let ((mut shaped, _, _, _), _) = laid_out(&line, fonts, Some(base), &|_, _| (0.0, w, None))?;
    let mut line = shaped.swap_remove(0);
    for piece in &mut line.pieces {
        piece.span = spans[piece.span];
    }
    for segment in line.shaped.iter_mut().flatten() {
        segment.span = spans[segment.span];
    }
    Some(line)
}

/// Each span's hyphen width, which a taken SHY reserves.
fn hyphens(content: &RichContent, fonts: &dyn crate::layout::Metrics) -> Vec<f32> {
    content
        .spans
        .iter()
        .map(|s| crate::rich::span_width(s, "-", fonts))
        .collect()
}

/// Parley ids of inline boxes' edges: this bit, the span shifted up one
/// and the end edge's bit (atoms' ids are their spans).
const EDGE: u64 = 1 << 62;

fn edge_id(span: usize, edge: Edge) -> u64 {
    EDGE | (span as u64) << 1 | u64::from(edge == Edge::End)
}

/// An edge's span and side from its Parley id.
fn edge_of(id: u64) -> Option<(usize, Edge)> {
    (id & EDGE != 0).then(|| {
        let edge = if id & 1 == 1 { Edge::End } else { Edge::Start };
        (((id & !EDGE) >> 1) as usize, edge)
    })
}

/// The span holding byte `at` of the paragraph's text (an atom holds
/// none), and its byte range.
fn span_at(content: &RichContent, at: usize) -> Option<(usize, std::ops::Range<usize>)> {
    let mut start = 0;
    for (k, s) in content.spans.iter().enumerate() {
        let len = if s.atom.is_some() { 0 } else { s.text.len() };
        if (start..start + len).contains(&at) {
            return Some((k, start..start + len));
        }
        start += len;
    }
    None
}

/// Span `k`'s bytes in the paragraph's text (an atom's empty, where it
/// sits).
fn span_range(content: &RichContent, k: usize) -> std::ops::Range<usize> {
    let len = |s: &crate::rich::RichSpan| if s.atom.is_some() { 0 } else { s.text.len() };
    let start: usize = content.spans[..k].iter().map(len).sum();
    start..start + len(&content.spans[k])
}

/// The spans before and after byte `at` when one inline box holds both: a
/// break there is inside it.
fn inside(content: &RichContent, text: &str, at: usize) -> Option<(usize, usize)> {
    if at == 0 || at >= text.len() {
        return None;
    }
    let (a, _) = span_at(content, at - 1)?;
    let (b, _) = span_at(content, at)?;
    let id = |k: usize| content.spans[k].inline_box.as_ref().map(|b| b.id);
    (id(a).is_some() && id(a) == id(b)).then_some((a, b))
}

/// Whether the paragraph holding byte `at` runs right to left: the text
/// between the forced breaks round it, or `direction` where it is set.
fn rtl_at(text: &str, at: usize, direction: Option<Direction>) -> bool {
    let d = direction.unwrap_or_else(|| {
        let from = text[..at].rfind('\n').map_or(0, |p| p + 1);
        let to = text[at..].find('\n').map_or(text.len(), |p| at + p);
        og_i18n::paragraph_direction(&text[from..to], Direction::Ltr)
    });
    d == Direction::Rtl
}

/// A cloned inline box a line's range breaks inside (CSS Fragmentation 3
/// §5.4): its start edge's span and width at the line's start, its end
/// edge's at its end.
fn cloned(
    content: &RichContent,
    text: &str,
    range: &std::ops::Range<usize>,
    direction: Option<Direction>,
) -> [Option<(usize, f32)>; 2] {
    let at = |byte: usize, end: bool| {
        let (before, after) = inside(content, text, byte)?;
        let b = content.spans[before].inline_box.as_ref()?;
        let [s, e] = b.edges(rtl_at(text, byte, direction));
        b.clone
            .then_some(if end { (before, e) } else { (after, s) })
    };
    [at(range.start, false), at(range.end, true)]
}

/// What a line holds back for a cloned box's edges at its breaks, and the
/// width it breaks within so an edge stays with its text (`stuck`: only
/// an emergency break did, so it no longer tries).
#[derive(Clone, Copy, Debug)]
struct LineFit {
    room: f32,
    cap: f32,
    stuck: bool,
}

impl Default for LineFit {
    fn default() -> LineFit {
        LineFit {
            room: 0.0,
            cap: f32::INFINITY,
            stuck: false,
        }
    }
}

/// Fits the first line whose inline box edges do not sit with their text
/// (CSS 2 §10.8: an inline box's start padding and border stay with its
/// first character, its end with its last), for the next break; the
/// lines after it fit again. Whether it changed one.
fn refit(
    fit: &mut Vec<LineFit>,
    content: &RichContent,
    layout: &Layout<Brush>,
    text: &str,
    direction: Option<Direction>,
) -> bool {
    let lines: Vec<_> = layout.lines().collect();
    // Line `k`'s fit, the lines after it to fit again.
    let at = |fit: &mut Vec<LineFit>, k: usize| -> LineFit {
        fit.resize(fit.len().max(k + 1), LineFit::default());
        fit.truncate(k + 1);
        fit[k]
    };
    for (k, line) in lines.iter().enumerate() {
        let range = line.text_range();
        let m = line.metrics();
        let width = |m: &parley::LineMetrics| m.advance - m.hanging_advance + m.indent;
        let here = fit.get(k).copied().unwrap_or_default();
        // A cloned box broken at its ends: its edges' room.
        let need: f32 = cloned(content, text, &range, direction)
            .iter()
            .flatten()
            .map(|(_, w)| w)
            .sum();
        if need > here.room + 0.01 {
            at(fit, k);
            fit[k].room = need;
            return true;
        }
        // Only a break at a soft wrap opportunity moves an edge.
        let last = k + 1 == lines.len() || line.break_reason() != BreakReason::Regular;
        for item in line.items() {
            let PositionedLayoutItem::InlineBox(b) = item else {
                continue;
            };
            let Some((span, edge)) = edge_of(b.id) else {
                continue;
            };
            let r = span_range(content, span);
            let byte = if edge == Edge::Start { r.start } else { r.end };
            match edge {
                // A start edge the line ends at: it moves down with its text.
                Edge::Start if !last && byte == range.end && !here.stuck => {
                    at(fit, k);
                    fit[k].cap = here.cap.min(width(m) - 0.5);
                    return true;
                }
                // An end edge after a word it cannot break beside: the word
                // moves down with it.
                Edge::End
                    if !last && byte == range.end && !here.stuck && !opportunity(text, byte) =>
                {
                    at(fit, k);
                    fit[k].cap = here.cap.min(width(m) - 0.5);
                    return true;
                }
                // An end edge alone at a line's start: its box's last word
                // moves down to it.
                Edge::End
                    if k > 0
                        && byte == range.start
                        && lines[k - 1].break_reason() == BreakReason::Regular =>
                {
                    let before = fit.get(k - 1).copied().unwrap_or_default();
                    if before.stuck {
                        continue;
                    }
                    at(fit, k - 1);
                    fit[k - 1].cap = before.cap.min(width(&lines[k - 1].metrics()) - 0.5);
                    return true;
                }
                _ => {}
            }
        }
    }
    false
}

/// Whether a line may break at byte `at` beside an inline box's edge: a
/// space or hyphen before it, a space after, or CJK on either side.
fn opportunity(text: &str, at: usize) -> bool {
    let before = text[..at].chars().next_back();
    let after = text[at..].chars().next();
    before.is_some_and(|c| c.is_whitespace() || c == '-' || crate::text::is_cjk(c))
        || after.is_some_and(|c| c.is_whitespace() || crate::text::is_cjk(c))
}
