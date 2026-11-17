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
use crate::rich::{LineBreak, Piece, RichContent, RichLine, Segment, TextIndent};
use crate::text::line_edges;
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
    let (mut broken, widths) = laid_out(content, fonts, None, &|_, _| (0.0, w, None))?;
    crate::text::text_box::apply(content, fonts, &mut broken.0);
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
    let mut broken = laid_out(content, fonts, None, room)?.0;
    crate::text::text_box::apply(content, fonts, &mut broken.0);
    Some(broken)
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
    // Each line's room held back for the edges of the cloned inline
    // elements its breaks split (`box-decoration-break: clone`), found by
    // the passes before; a few passes settle it.
    let mut hold: Vec<f32> = Vec::new();
    // Where CJK spacing is planned (CSS Text 4 §8.4, §8.5): the lines the
    // pass before broke.
    let mut sides = line_edges::Sides::new(content);
    let mut passes = 0;
    loop {
        let mut planned = Vec::new();
        passes += 1;
        let plan = sides.plan(content, fonts);
        let pass = shaped(content, fonts, direction, &plan, |layout, text, indices| {
            let widths = layout.calculate_content_widths();
            let (broke, narrowed, drop) = break_beside(layout, content, &mut reserve, &hold, room);
            // Each line's cut edges: at its start (its first span and their
            // width) and at its end.
            let mut cuts = vec![((0, 0.0), (0, 0.0)); broke.len()];
            for k in 1..broke.len() {
                let (Some(last), Some(next)) = (
                    line_span(content, text, &broke[k - 1].range, true),
                    line_span(content, text, &broke[k].range, false),
                ) else {
                    continue;
                };
                let [end, start] = crate::rich::cut_room(content, last, next);
                cuts[k - 1].1 = (last, end);
                cuts[k].0 = (next, start);
            }
            let wanted: Vec<f32> = cuts.iter().map(|(a, b)| a.1 + b.1).collect();
            let held = |k: usize| hold.get(k).copied().unwrap_or(0.0);
            if passes < 8
                && wanted
                    .iter()
                    .enumerate()
                    .any(|(k, w)| (w - held(k)).abs() > 0.01)
            {
                return Err(Retry::Hold(wanted));
            }
            // CJK spacing on the lines as they broke.
            let lines = layout
                .lines()
                .zip(&broke)
                .enumerate()
                .map(|(k, (line, b))| {
                    let m = line.metrics();
                    // A line Parley left empty may have no range in the text.
                    let start = b.range.start.min(text.len());
                    let range = start..b.range.end.clamp(start, text.len());
                    line_edges::Line {
                        first: k == 0 || text[..range.start].ends_with('\n'),
                        range,
                        wrapped: b.wrapped,
                        room: b.shift[2]
                            - reserve.get(k).copied().unwrap_or(0.0).max(0.0)
                            - held(k),
                        width: m.advance - m.hanging_advance + m.indent,
                    }
                })
                .collect();
            let advance = |r: std::ops::Range<usize>| {
                let mut at = r.start;
                let mut w = 0.0;
                while at < r.end {
                    let Some(c) = parley::Cluster::from_byte_index(layout, at) else {
                        break;
                    };
                    w += c.advance();
                    at = c.text_range().end.max(at + 1);
                }
                w
            };
            if passes < 8
                && let Some(next) = sides.next(content, fonts, lines, &plan, &advance)
            {
                return Err(Retry::Spacing(next));
            }
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
                    return Err(Retry::Hyphen(k, hyphens[span]));
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
                let mut drawn = rich_line(content, fonts, text, indices, &line);
                super::hanging::finish(content, fonts, &mut drawn, b.range.clone(), &plan);
                lines.push(drawn);
            }
            let shifts = broke.iter().map(|b| b.shift).collect();
            let rtl = layout.is_rtl();
            Ok((
                (lines, shifts, narrowed, drop),
                (widths.max, widths.min),
                cuts,
                rtl,
            ))
        })?;
        match pass {
            Err(Retry::Hyphen(k, hyphen)) => {
                reserve.resize(reserve.len().max(k + 1), 0.0);
                reserve[k] = hyphen;
            }
            Err(Retry::Hold(wanted)) => hold = wanted,
            Err(Retry::Spacing(next)) => sides = next,
            Ok((mut broken, widths, cuts, rtl)) => {
                // Shape each hyphenated line again outside the font
                // context's borrow: contextual glyphs and bidi see the
                // visible hyphen; the spans keep their ink, links and
                // atomic content.
                for (k, line) in planned {
                    broken.0[k] = reshape(line, fonts)?;
                }
                for (line, (start, end)) in broken.0.iter_mut().zip(cuts) {
                    crate::rich::with_room(line, start, end, rtl);
                }
                return Some((broken, widths));
            }
        }
    }
}

/// Why a pass breaks the lines again: a line's hyphen does not fit (its
/// index and the hyphen's width), the room cut edges take differs from
/// what the lines held back (what each line holds now), or the lines'
/// CJK spacing differs from what they were shaped with (where it is
/// planned now).
enum Retry {
    Hyphen(usize, f32),
    Hold(Vec<f32>),
    Spacing(line_edges::Sides),
}

/// The span of a line's first character, or of its last before its
/// hanging spaces (`last`).
fn line_span(
    content: &RichContent,
    text: &str,
    range: &std::ops::Range<usize>,
    last: bool,
) -> Option<usize> {
    // A line Parley left empty has no span.
    if range.is_empty() {
        return None;
    }
    let at = if last {
        range.start
            + text[range.clone()]
                .trim_end_matches([' ', '\t', '\n'])
                .len()
    } else {
        range.start + 1
    }
    .checked_sub(1)?;
    let mut start = 0;
    content.spans.iter().position(|s| {
        let len = if s.atom.is_some() { 0 } else { s.text.len() };
        let hit = (start..start + len).contains(&at);
        start += len;
        hit
    })
}

/// Breaks the layout's lines one at a time at the room beside each
/// (`hold`: what each line holds back for the edges its breaks cut;
/// `reserve`: what each line holds back for its hyphen; negative where
/// holding it back left only an emergency break, so the line breaks at
/// its SHY as if it were not there, its hyphen not shown): a line taller
/// than the room asked for asks again as tall as it is, and a line wider
/// than its room moves down to the floats' next bottom. Each line's range
/// and shift, whether a float narrowed one, and the drop.
fn break_beside(
    layout: &mut Layout<Brush>,
    content: &RichContent,
    reserve: &mut [f32],
    hold: &[f32],
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
                let cut = hold.get(out.len()).copied().unwrap_or(0.0);
                let state = lines.state_mut();
                state.set_line_max_advance((avail - held - cut).max(0.0));
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
fn rich_line(
    content: &RichContent,
    fonts: &dyn crate::layout::Metrics,
    text: &str,
    indices: &[usize],
    line: &parley::Line<'_, Brush>,
) -> RichLine {
    let m = line.metrics();
    let mut segments: Vec<Segment> = Vec::new();
    // Each segment's first character, counted over the segments' text.
    let mut at = 0;
    // Each span's first byte in the text.
    let starts: Vec<usize> = content
        .spans
        .iter()
        .scan(0, |at, s| {
            let start = *at;
            *at += if s.atom.is_some() { 0 } else { s.text.len() };
            Some(start)
        })
        .collect();
    for item in line.items() {
        match item {
            PositionedLayoutItem::InlineBox(b) => segments.push(Segment {
                span: b.id as usize,
                text: String::new(),
                at,
                glyphs: None,
                width: b.width,
                rise: m.baseline - (b.y + b.baseline.unwrap_or(b.height)),
            }),
            PositionedLayoutItem::GlyphRun(glyph_run) => {
                let brush = glyph_run.style().brush;
                let s = &content.spans[brush.span];
                let metrics = fonts.metrics_styled(s.font, s.query(), &s.text);
                let font = metrics.part(brush.font as usize).expect("selected face");
                let file = font.file.as_ref().expect("cooked font");
                let run = glyph_run.run();
                let range = run.text_range();
                // The room its owners' start takes after its first grapheme
                // (its end's after its last) is drawn before it: the glyphs
                // move along by it, on its right in a right-to-left run.
                let [before, after] = s.room();
                let edges = crate::rich::edge_graphemes(&s.text);
                let shift = |cr: &std::ops::Range<usize>| {
                    let Some((first, last)) = &edges else {
                        return 0.0;
                    };
                    let edge = if run.is_rtl() {
                        (last, after)
                    } else {
                        (first, before)
                    };
                    if cr.start == starts[brush.span] + edge.0.start {
                        edge.1
                    } else {
                        0.0
                    }
                };
                let mut glyphs = Vec::new();
                for cluster in run.visual_clusters().filter(|c| c.style().brush == brush) {
                    let cr = cluster.text_range();
                    let source = text[cr.clone()].chars().next().unwrap_or(' ');
                    let index = indices[cr.start] - indices[range.start];
                    let dx = shift(&cr);
                    for g in cluster.glyphs() {
                        let (ch, glyph) =
                            file.character_for_glyph(g.id, source, &|k| font.in_atlas_key(k));
                        glyphs.push(VisualGlyph {
                            ch,
                            glyph,
                            advance: g.advance / s.step.size,
                            x: (g.x + dx) / s.step.size,
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
                });
                at += text[range].chars().count();
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

/// A span's or atom's `vertical-align` as Parley places it against the
/// paragraph's root text `parent`, with its own baselines `own` (CSS px
/// above its alphabetic one), font `size` and line height `line`: the
/// baselines Parley knows itself it aligns, the others the two tables'
/// difference raises (CSS Inline 3 §4.2.2); a shift adds to that. Parley
/// has no line-relative `center`: such a span keeps its baseline place.
fn valign(
    v: crate::VerticalAlign,
    parent: &crate::layout::Strut,
    own: &og_text::baselines::Baselines,
    size: f32,
    line: f32,
) -> parley::VerticalAlign {
    use crate::layout::{Baseline as B, BaselineShift as S};
    use parley::AlignmentBaseline as A;
    let shift = match v.shift {
        S::Top => return parley::VerticalAlign::TOP,
        S::Bottom => return parley::VerticalAlign::BOTTOM,
        S::Center => 0.0,
        S::Px(px) => px,
        S::Em(em) => em * size,
        S::Percent(p) => p * line,
        S::Sub => -parent.size / 5.0,
        S::Super => parent.size / 3.0,
    };
    let (alignment, raise) = match v.baseline.unwrap_or(parent.dominant) {
        B::Alphabetic => (A::Baseline, 0.0),
        B::TextTop => (A::TextTop, 0.0),
        B::TextBottom => (A::TextBottom, 0.0),
        B::Middle => (A::Middle, 0.0),
        b => (A::Baseline, parent.baselines.at(b) - own.at(b)),
    };
    parley::VerticalAlign::new(alignment, parley::BaselineShift::Length(raise + shift))
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
    plan: &line_edges::Plan,
    f: impl FnOnce(&mut Layout<Brush>, &str, &[usize]) -> R,
) -> Option<R> {
    use crate::rich::{span_tracking, span_width};
    let mut text = String::new();
    let mut ranges = Vec::new();
    let mut boxes = Vec::new();
    let mut selected = Vec::new();
    let mut scaled = Vec::new();
    // The root text every span and atom aligns against.
    let (base, role) = content.base;
    let root = fonts.metrics_styled(role, crate::FontQuery::new(base.weight, false), "");
    let parent = crate::layout::Strut::of(root, base.size, content.dominant);
    for (span, s) in content.spans.iter().enumerate() {
        let start = text.len();
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
            // Its owners' edges take room in its box, drawn inside it.
            let [before, after] = s.room();
            boxes.push(parley::InlineBox {
                id: span as u64,
                index: start,
                kind: parley::InlineBoxKind::InFlow,
                width: span_width(s, "", fonts) + before + after,
                height,
                baseline: Some(baseline),
                vertical_align: valign(
                    s.valign,
                    &parent,
                    &og_text::baselines::Baselines::of_box(baseline, baseline - height),
                    s.step.size,
                    s.step.line_height,
                ),
            });
        } else {
            text.push_str(&s.text);
            let metrics = fonts.metrics_styled(s.font, s.query(), &s.text);
            for (range, k) in metrics.font_runs(&s.text) {
                let m = metrics.part(k)?;
                // A face that sets its digits on the tabular backstop is
                // laid out run by run, which holds them as drawn.
                if m.fixed.is_some() || m.pixel != 0 || m.tabulates(&s.text, s.step.features) {
                    return None;
                }
                // Each face at its own size (`font-size-adjust`).
                if m.scale != 1.0 {
                    scaled.push((
                        start + range.start..start + range.end,
                        s.step.size * m.scale,
                    ));
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
            let own = crate::layout::baselines_of(fonts.metrics_styled(s.font, s.query(), &s.text))
                .scaled(s.step.size);
            builder.push(
                StyleProperty::VerticalAlign(valign(
                    s.valign,
                    &parent,
                    &own,
                    s.step.size,
                    line_height(s),
                )),
                range.clone(),
            );
            let tracking = span_tracking(s, &s.text) * s.step.size;
            builder.push(StyleProperty::LetterSpacing(tracking), range.clone());
            // Its owners' start and end take room after its first and last
            // grapheme, which [`rich_line`] turns into room before the
            // first. Each is shaped apart from its neighbours (with them as
            // context: joins hold), as CSS Text 3 §7.3 breaks shaping at a
            // box's padding.
            let [before, after] = s.room();
            if s.atom.is_none()
                && (before, after) != (0.0, 0.0)
                && let Some((first, last)) = crate::rich::edge_graphemes(&s.text)
            {
                let at = |r: std::ops::Range<usize>| range.start + r.start..range.start + r.end;
                if first == last {
                    builder.push(
                        StyleProperty::LetterSpacing(tracking + before + after),
                        at(first),
                    );
                } else {
                    builder.push(StyleProperty::LetterSpacing(tracking + before), at(first));
                    builder.push(StyleProperty::LetterSpacing(tracking + after), at(last));
                }
            }
            builder.push(StyleProperty::WordSpacing(s.word_spacing), range.clone());
            let features = s.step.features.css();
            builder.push(
                StyleProperty::FontFeatures(features.as_str().into()),
                range.clone(),
            );
            // Parley takes a BCP 47 language only: an OpenType language
            // system (`font-language-override`) does not parse.
            let language = s.step.features.language();
            builder.push(
                StyleProperty::Locale(language.and_then(|l| l.parse().ok())),
                range.clone(),
            );
        }
        // Line edges ([`line_edges`]): a trimmed punctuation's `halt`
        // (CSS Text 4 §8.5), and room after a unit beside its span's
        // tracking and its owners' edges: its autospace (§8.4), less its
        // width where it hangs (§9.2.1), so it takes none.
        let span_at = |at: usize| ranges.iter().position(|r| r.contains(&at));
        for r in &plan.halt {
            if let Some(k) = span_at(r.start) {
                let mut features = content.spans[k].step.features.css();
                if !features.is_empty() {
                    features.push(',');
                }
                features.push_str("'halt' 1");
                builder.push(
                    StyleProperty::FontFeatures(features.as_str().into()),
                    r.clone(),
                );
            }
        }
        let mut room: Vec<(std::ops::Range<usize>, f32)> = plan.space.clone();
        for (r, w) in &plan.hang {
            match room.iter_mut().find(|(at, _)| at == r) {
                Some((_, extra)) => *extra -= w,
                None => room.push((r.clone(), -w)),
            }
        }
        for (r, extra) in &room {
            let Some(k) = span_at(r.start) else {
                continue;
            };
            let s = &content.spans[k];
            let mut spacing = span_tracking(s, &s.text) * s.step.size + extra;
            if let Some((first, last)) = crate::rich::edge_graphemes(&s.text) {
                let [before, after] = s.room();
                let local = r.start - ranges[k].start;
                if first.contains(&local) {
                    spacing += before;
                }
                if last.contains(&local) {
                    spacing += after;
                }
            }
            builder.push(StyleProperty::LetterSpacing(spacing), r.clone());
        }
        for (range, size) in scaled {
            builder.push(StyleProperty::FontSize(size), range);
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
        // A first line's hanging punctuation sits in its indent (§9.2.1).
        layout.set_text_indent(
            px - plan.hang_start,
            parley::IndentOptions { each_line, hanging },
        );
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
        // Its owners' start takes room only before its first character,
        // their end only after its last.
        for e in &mut span.edges {
            e.opens &= a == start - s.text.len();
            e.closes &= b == start;
        }
        if k == shy && b == end {
            span.text.pop();
            span.text.push_str(s.hyphen.as_deref().unwrap_or("-"));
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

/// Each span's hyphen width (its `hyphenate-character`), which a taken
/// SHY reserves.
fn hyphens(content: &RichContent, fonts: &dyn crate::layout::Metrics) -> Vec<f32> {
    content
        .spans
        .iter()
        .map(|s| crate::rich::span_width(s, s.hyphen.as_deref().unwrap_or("-"), fonts))
        .collect()
}
