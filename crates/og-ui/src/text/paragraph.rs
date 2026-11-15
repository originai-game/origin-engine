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
use crate::rich::{Piece, RichContent, RichLine, Segment};
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
    loop {
        let mut planned = Vec::new();
        let pass = shaped(content, fonts, direction, |layout, text, indices| {
            let widths = layout.calculate_content_widths();
            let (broke, narrowed, drop) = break_beside(layout, content, &mut reserve, room);
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
                let width = m.advance - m.hanging_advance + hyphens[span];
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
                lines.push(rich_line(content, fonts, text, indices, &line));
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
                let state = lines.state_mut();
                state.set_line_max_advance((avail - held).max(0.0));
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
                let height = d.metrics.line_height;
                let width = d.metrics.advance - d.metrics.hanging_advance;
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
fn rich_line(
    content: &RichContent,
    fonts: &dyn crate::layout::Metrics,
    text: &str,
    indices: &[usize],
    line: &parley::Line<'_, Brush>,
) -> RichLine {
    let m = line.metrics();
    let mut segments = Vec::new();
    for item in line.items() {
        match item {
            PositionedLayoutItem::InlineBox(b) => segments.push(Segment {
                span: b.id as usize,
                text: String::new(),
                glyphs: None,
                width: b.width,
                rise: m.baseline - (b.y + b.baseline.unwrap_or(b.height)),
            }),
            PositionedLayoutItem::GlyphRun(glyph_run) => {
                let brush = glyph_run.style().brush;
                let s = &content.spans[brush.span];
                let metrics = fonts.metrics_styled(s.font, s.weight, s.italic, &s.text);
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
                    text: text[range].to_owned(),
                    glyphs: Some(glyphs),
                    width: glyph_run.advance(),
                    rise: m.baseline - glyph_run.baseline(),
                });
            }
        }
    }
    let mut pieces = Vec::new();
    let mut x = 0.0;
    for segment in &segments {
        pieces.push(Piece {
            span: segment.span,
            text: segment.text.clone(),
            x,
            width: segment.width,
            rise: segment.rise,
        });
        x += segment.width;
    }
    RichLine {
        pieces,
        width: x,
        height: m.line_height,
        baseline: m.baseline - m.block_min_coord,
        shaped: Some(segments),
    }
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
    for (span, s) in content.spans.iter().enumerate() {
        let start = text.len();
        if s.atom.is_some() {
            boxes.push(parley::InlineBox {
                id: span as u64,
                index: start,
                kind: parley::InlineBoxKind::InFlow,
                width: span_width(s, "", fonts),
                height: s.step.size * 1.1,
                baseline: Some(s.step.size * 0.85),
                vertical_align: valign(s.valign, s.step.size, s.step.line_height),
            });
        } else {
            text.push_str(&s.text);
            let metrics = fonts.metrics_styled(s.font, s.weight, s.italic, &s.text);
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
        ranges.push(start..text.len());
    }
    // The strut: the paragraph's own face at its step.
    let (base, role) = content.base;
    let strut = {
        let m = fonts.metrics_styled(role, base.weight, false, "");
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
            let features = s
                .step
                .features
                .tags()
                .map(|tag| format!("'{tag}' 1"))
                .collect::<Vec<_>>()
                .join(",");
            builder.push(
                StyleProperty::FontFeatures(features.as_str().into()),
                range.clone(),
            );
        }
        for atom in boxes {
            builder.push_inline_box(atom);
        }
        let mut layout = builder.build(&text);
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
            PositionedLayoutItem::InlineBox(b) => Some(b.id as usize),
            PositionedLayoutItem::GlyphRun(_) => None,
        })
        .collect();
    let end = range.start + text[range.clone()].trim_end_matches([' ', '\t']).len();
    let mut out = RichContent {
        spans: Vec::new(),
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
