//! Cooked-font paragraph layout. The collection never discovers host fonts.
use super::{Features, FontMetrics, VisualGlyph};
use crate::shape::FontFile;
use og_i18n::Direction;
use parley::{FontContext, Layout, LayoutContext, StyleProperty};
use std::{cell::RefCell, sync::Arc};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Brush {
    font: u8,
    span: usize,
}

struct Context {
    fonts: FontContext,
    layout: LayoutContext<Brush>,
    files: Vec<Arc<FontFile>>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            fonts: FontContext {
                collection: parley::fontique::Collection::new(
                    parley::fontique::CollectionOptions {
                        shared: false,
                        system_fonts: false,
                    },
                ),
                source_cache: Default::default(),
            },
            layout: LayoutContext::new(),
            files: Vec::new(),
        }
    }
}

thread_local! {
    static CONTEXT: RefCell<Context> = RefCell::new(Context::default());
}

impl Context {
    fn family(&mut self, file: Arc<FontFile>) -> String {
        let index = match self.files.iter().position(|f| Arc::ptr_eq(f, &file)) {
            Some(index) => index,
            None => {
                let index = self.files.len();
                let name = format!("og-{index}");
                self.fonts.collection.register_fonts(
                    parley::fontique::Blob::new(file.bytes().clone()),
                    Some(parley::fontique::FontInfoOverride {
                        family_name: Some(&name),
                        ..Default::default()
                    }),
                );
                self.files.push(file);
                index
            }
        };
        format!("og-{index}")
    }
}

/// Parley 0.12 treats SHY as default-ignorable, but does not reserve or
/// draw the discretionary glyph. Select its breaks from Parley's shaped
/// graphemes and UAX #14 opportunities, reserving the hyphen only at a
/// taken break. Prefix advances keep a long discretionary word linear.
pub(super) fn discretionary(
    metrics: &FontMetrics,
    text: &str,
    size: f32,
    tracking: f32,
    features: Features,
    max: Option<f32>,
) -> Option<Vec<super::Line>> {
    if !text.contains('\u{ad}') {
        return None;
    }
    if text.contains('\n') {
        return Some(
            text.split('\n')
                .flat_map(|paragraph| {
                    super::break_lines(paragraph, metrics, size, tracking, features, max)
                })
                .collect(),
        );
    }
    let max = max? / size;
    let shaped = layout(
        metrics,
        text,
        1.0,
        tracking,
        features,
        og_i18n::paragraph_direction(text, Direction::Ltr),
        None,
    )?;
    let mut clusters = shaped
        .lines()
        .flat_map(|l| l.runs())
        .flat_map(|r| r.clusters())
        .collect::<Vec<_>>();
    clusters.sort_by_key(|c| c.text_range().start);
    let mut prefix = vec![0.0];
    for c in &clusters {
        prefix.push(prefix.last().unwrap() + c.advance());
    }
    let mut analysis = parley_engine::Analysis::new();
    parley_engine::Analyzer::new().analyze(
        text,
        &parley_engine::AnalysisOptions {
            line_break_override: Some(&super::keep_katakana),
            ..Default::default()
        },
        &mut analysis,
    );
    let opportunities: std::collections::BTreeSet<_> = text
        .char_indices()
        .zip(analysis.char_info())
        .filter_map(|((byte, _), info)| info.is_soft_wrap_opportunity().then_some(byte))
        .collect();
    let mut ranges = Vec::new();
    let (mut start, mut at, mut best) = (0, 0, 0);
    while at < clusters.len() {
        let c = clusters[at];
        if at > start && opportunities.contains(&c.text_range().start) {
            let previous = clusters[at - 1];
            let extra = if text[previous.text_range()].ends_with('\u{ad}') {
                metrics
                    .part(previous.style().brush.font as usize)?
                    .measure_with("-", 1.0, tracking, features)
            } else {
                0.0
            };
            if prefix[at] - prefix[start] + extra <= max {
                best = at;
            }
        }
        if prefix[at + 1] - prefix[start] > max && at > start {
            let mut end = if best > start { best } else { at };
            if best == start
                && end > start + 1
                && text[clusters[end - 1].text_range()].ends_with('\u{ad}')
            {
                // A discretionary glyph that does not fit cannot turn an
                // emergency grapheme break into an overflowing hyphen.
                end -= 1;
            }
            ranges.push((start, end, true));
            start = end;
            at = end;
            best = end;
        } else {
            at += 1;
        }
    }
    if start < clusters.len() {
        ranges.push((start, clusters.len(), false));
    }
    let lines = ranges
        .into_iter()
        .map(|(a, b, wrapped)| {
            let range = clusters[a].text_range().start..clusters[b - 1].text_range().end;
            let mut line = text[range]
                .trim_end_matches([' ', '\t', '\r', '\n'])
                .to_owned();
            if wrapped && line.ends_with('\u{ad}') {
                line.pop();
                line.push('-');
            }
            let width = metrics.measure_with(&line, size, tracking, features);
            super::Line { text: line, width }
        })
        .collect();
    Some(lines)
}

/// Old atlas-only blobs and headless fixed metrics have no OpenType data.
/// They keep their table metrics until recooked; every newly cooked font
/// carries its file. A mixed paragraph takes this path only when all of
/// its chosen fonts can be shaped.
pub(super) fn layout(
    metrics: &FontMetrics,
    text: &str,
    size: f32,
    tracking: f32,
    features: Features,
    direction: Direction,
    width: Option<f32>,
) -> Option<Layout<Brush>> {
    let runs = metrics.font_runs(text);
    // `tnum` on a face whose figures stay proportional under it (Meridian's
    // Chakra Petch has no `tnum`): the run path sets its digits on the
    // widest digit's advance (FontMetrics::tabulate), which a shaped layout
    // would not, and a counting clock would change width as it runs.
    if super::tabulates(text, features)
        && runs.iter().any(|(_, k)| {
            metrics
                .part(*k)
                .is_some_and(|m| m.tabular_advance(features).is_some())
        })
    {
        return None;
    }
    let files = runs
        .iter()
        .map(|(_, k)| {
            let m = metrics.part(*k)?;
            (m.fixed.is_none() && m.pixel == 0).then_some(())?;
            m.file.clone()
        })
        .collect::<Option<Vec<_>>>()?;
    CONTEXT.with_borrow_mut(|cx| {
        // Bound ownership of font bytes across game reloads.
        if cx.files.len() > 128 {
            *cx = Context::default();
        }
        let mut families = Vec::with_capacity(files.len());
        for file in files {
            families.push(cx.family(file));
        }
        let mut builder = cx.layout.ranged_builder(&mut cx.fonts, text, 1.0, false);
        builder.set_line_break_override(Some(&super::keep_katakana));
        builder.set_base_direction(match direction {
            Direction::Ltr => parley::BaseDirection::Ltr,
            Direction::Rtl => parley::BaseDirection::Rtl,
        });
        builder.push_default(StyleProperty::FontSize(size));
        builder.push_default(StyleProperty::OverflowWrap(parley::OverflowWrap::Anywhere));
        let settings = features
            .tags()
            .map(|tag| format!("'{tag}' 1"))
            .collect::<Vec<_>>()
            .join(",");
        builder.push_default(StyleProperty::FontFeatures(settings.as_str().into()));
        for ((range, k), family) in runs.iter().zip(&families) {
            builder.push(
                StyleProperty::FontFamily(family.as_str().into()),
                range.clone(),
            );
            builder.push(
                StyleProperty::Brush(Brush {
                    font: *k as u8,
                    span: 0,
                }),
                range.clone(),
            );
            let track = if text[range.clone()].chars().any(og_i18n::needs_shaping) {
                0.0
            } else {
                tracking * size
            };
            builder.push(StyleProperty::LetterSpacing(track), range.clone());
        }
        let mut layout = builder.build(text);
        layout.break_all_lines(width);
        Some(layout)
    })
}

pub(super) fn visual(
    metrics: &FontMetrics,
    text: &str,
    layout: &Layout<Brush>,
) -> Vec<VisualGlyph> {
    let mut out = Vec::new();
    let mut indices = vec![0; text.len() + 1];
    for (index, (byte, _)) in text.char_indices().enumerate() {
        indices[byte] = index;
    }
    for line in layout.lines() {
        for run in line.runs() {
            for cluster in run.visual_clusters() {
                let range = cluster.text_range();
                let k = cluster.style().brush.font;
                let font = metrics.part(k as usize).expect("the selected font");
                let file = font.file.as_ref().expect("a cooked font file");
                let source = text[range.clone()].chars().next().unwrap_or(' ');
                for g in cluster.glyphs() {
                    let (ch, glyph) =
                        file.character_for_glyph(g.id, source, &|k| font.in_atlas_key(k));
                    out.push(VisualGlyph {
                        ch,
                        glyph,
                        advance: g.advance,
                        x: g.x,
                        y: -g.y,
                        index: indices[range.start],
                        rtl: run.is_rtl(),
                        font: k,
                    });
                }
            }
        }
    }
    out
}

/// Styled spans share one bidi analysis and line breaker. Atomic content
/// participates as an inline box, not a placeholder measured as a letter.
pub(crate) fn rich(
    content: &crate::rich::RichContent,
    fonts: &dyn crate::layout::Metrics,
    width: Option<f32>,
) -> Option<(Vec<crate::rich::RichLine>, (f32, f32))> {
    rich_at(content, fonts, width, None)
}

fn rich_at(
    content: &crate::rich::RichContent,
    fonts: &dyn crate::layout::Metrics,
    width: Option<f32>,
    direction: Option<Direction>,
) -> Option<(Vec<crate::rich::RichLine>, (f32, f32))> {
    use crate::rich::{Piece, RichLine, Segment, place_line};
    let discretionary = width.is_some() && content.spans.iter().any(|s| s.text.contains('\u{ad}'));
    let hyphens = if discretionary {
        hyphens(content, fonts)
    } else {
        Vec::new()
    };
    let mut planned = None;
    let result = shaped(content, fonts, direction, |layout, text, indices, boxes| {
        let widths = layout.calculate_content_widths();
        if discretionary {
            layout.break_all_lines(None);
            let mut breaker = Breaker::of(content, hyphens, text, layout, boxes);
            let mut lines = Vec::new();
            loop {
                let step = breaker.next(width.unwrap());
                lines.push(breaker.content(&step));
                breaker.at = step.resume;
                if step.last {
                    break;
                }
            }
            planned = Some(lines);
            return (Vec::new(), (widths.max, widths.min));
        }
        layout.break_all_lines(width);
        let mut lines = Vec::new();
        for line in layout.lines() {
            let mut segments = Vec::new();
            for item in line.items() {
                match item {
                    parley::PositionedLayoutItem::InlineBox(b) => segments.push(Segment {
                        span: b.id as usize,
                        text: String::new(),
                        glyphs: None,
                        width: b.width,
                        rise: 0.0,
                    }),
                    parley::PositionedLayoutItem::GlyphRun(glyph_run) => {
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
                                let (ch, glyph) = file
                                    .character_for_glyph(g.id, source, &|k| font.in_atlas_key(k));
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
                            rise: 0.0,
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
                    rise: 0.0,
                });
                x += segment.width;
            }
            let (height, baseline) = place_line(content, fonts, &mut pieces);
            for (segment, piece) in segments.iter_mut().zip(&pieces) {
                segment.rise = piece.rise;
            }
            lines.push(RichLine {
                pieces,
                width: x,
                height,
                baseline,
                shaped: Some(segments),
            });
        }
        (lines, (widths.max, widths.min))
    });
    let (mut lines, widths) = result?;
    if let Some(planned) = planned {
        // Release the font context before reshaping each chosen line. This
        // gives contextual glyphs and bidi the actual visible hyphen, and
        // keeps the original spans' ink, links, sizes and atomic content.
        for line in planned {
            lines.extend(reshape(line, fonts)?);
        }
    }
    Some((lines, widths))
}

/// The spans' paragraph shaped as one layout (one bidi analysis and line
/// breaker, atomic content as inline boxes), handed to `f` with its text,
/// each byte's character index and its inline boxes. `None` when a chosen
/// face cannot be shaped.
fn shaped<R>(
    content: &crate::rich::RichContent,
    fonts: &dyn crate::layout::Metrics,
    direction: Option<Direction>,
    f: impl FnOnce(&mut Layout<Brush>, &str, &[usize], &[parley::InlineBox]) -> R,
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
                vertical_align: parley::VerticalAlign::BASELINE,
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
        let mut builder = cx.layout.ranged_builder(&mut cx.fonts, &text, 1.0, false);
        builder.set_line_break_override(Some(&super::keep_katakana));
        if let Some(direction) = direction {
            builder.set_base_direction(match direction {
                Direction::Ltr => parley::BaseDirection::Ltr,
                Direction::Rtl => parley::BaseDirection::Rtl,
            });
        }
        builder.push_default(StyleProperty::OverflowWrap(parley::OverflowWrap::Anywhere));
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
        for atom in &boxes {
            builder.push_inline_box(atom.clone());
        }
        let mut layout = builder.build(&text);
        Some(f(&mut layout, &text, &indices, &boxes))
    })
}

/// A chosen line's spans (renumbered), its spans' indices in the
/// paragraph, and its paragraph's base direction.
type Planned = (crate::rich::RichContent, Vec<usize>, Direction);

/// A planned line shaped on its own (with its paragraph's direction), its
/// spans numbered as the paragraph's.
fn reshape(
    (line, spans, base): Planned,
    fonts: &dyn crate::layout::Metrics,
) -> Option<Vec<crate::rich::RichLine>> {
    let (mut shaped, _) = rich_at(&line, fonts, None, Some(base))?;
    for line in &mut shaped {
        for piece in &mut line.pieces {
            piece.span = spans[piece.span];
        }
        for segment in line.shaped.iter_mut().flatten() {
            segment.span = spans[segment.span];
        }
    }
    Some(shaped)
}

/// Each span's hyphen width, which a taken SHY reserves.
fn hyphens(content: &crate::rich::RichContent, fonts: &dyn crate::layout::Metrics) -> Vec<f32> {
    content
        .spans
        .iter()
        .map(|s| crate::rich::span_width(s, "-", fonts))
        .collect()
}

/// A unit of a shaped paragraph: a cluster, or an inline box.
struct Unit {
    range: std::ops::Range<usize>,
    span: usize,
    width: f32,
    atom: bool,
}

/// Where a line breaks: it ends before unit `end`, the next starts at
/// `resume` (past a forced break); `wrapped` at a soft break, `last` at the
/// paragraph's end.
pub(crate) struct Step {
    end: usize,
    resume: usize,
    wrapped: bool,
    last: bool,
}

/// A rich paragraph shaped once and broken one line at a time, each at a
/// width of its own (lines beside floats), from its shaped clusters and
/// UAX #14 opportunities, never from characters: bidi and the
/// discretionary hyphen (SHY, reserved only at a taken break) see the
/// whole paragraph. Inline boxes keep their own advances and cannot be
/// split. Each chosen line is reshaped on its own ([`Breaker::lines`]).
pub(crate) struct Breaker {
    content: crate::rich::RichContent,
    text: String,
    units: Vec<Unit>,
    prefix: Vec<f32>,
    opportunities: std::collections::BTreeSet<usize>,
    hyphens: Vec<f32>,
    paragraphs: Vec<(usize, Direction)>,
    /// The first unit of the next line.
    pub at: usize,
}

impl Breaker {
    /// The paragraph of `content` shaped once (`None`: a face it chose
    /// cannot be shaped).
    pub(crate) fn new(
        content: &crate::rich::RichContent,
        fonts: &dyn crate::layout::Metrics,
    ) -> Option<Breaker> {
        let hyphens = hyphens(content, fonts);
        shaped(content, fonts, None, |layout, text, _, boxes| {
            layout.break_all_lines(None);
            Breaker::of(content, hyphens, text, layout, boxes)
        })
    }

    /// From the paragraph's layout (`hyphens`: each span's hyphen width,
    /// measured outside the font context's borrow).
    fn of(
        content: &crate::rich::RichContent,
        hyphens: Vec<f32>,
        text: &str,
        layout: &Layout<Brush>,
        boxes: &[parley::InlineBox],
    ) -> Breaker {
        let mut units: Vec<_> = layout
            .lines()
            .flat_map(|l| l.runs())
            .flat_map(|r| r.clusters())
            .map(|c| Unit {
                range: c.text_range(),
                span: c.style().brush.span,
                width: c.advance(),
                atom: false,
            })
            .collect();
        units.extend(boxes.iter().map(|b| Unit {
            range: b.index..b.index,
            span: b.id as usize,
            width: b.width,
            atom: true,
        }));
        units.sort_by_key(|u| (u.range.start, !u.atom));
        let mut analysis = parley_engine::Analysis::new();
        parley_engine::Analyzer::new().analyze(
            text,
            &parley_engine::AnalysisOptions {
                line_break_override: Some(&super::keep_katakana),
                ..Default::default()
            },
            &mut analysis,
        );
        let opportunities = text
            .char_indices()
            .zip(analysis.char_info())
            .filter_map(|((byte, _), info)| info.is_soft_wrap_opportunity().then_some(byte))
            .collect();
        let mut prefix = vec![0.0];
        for u in &units {
            prefix.push(prefix.last().unwrap() + u.width);
        }
        let mut paragraph_start = 0;
        let paragraphs = text
            .split_inclusive('\n')
            .map(|paragraph| {
                let entry = (
                    paragraph_start,
                    og_i18n::paragraph_direction(paragraph, Direction::Ltr),
                );
                paragraph_start += paragraph.len();
                entry
            })
            .collect();
        Breaker {
            content: content.clone(),
            text: text.to_owned(),
            units,
            prefix,
            opportunities,
            hyphens,
            paragraphs,
            at: 0,
        }
    }

    fn shy(&self, i: usize) -> bool {
        !self.units[i].atom && self.text[self.units[i].range.clone()].ends_with('\u{ad}')
    }

    /// White space, which hangs at the end of a line.
    fn space(&self, i: usize) -> bool {
        let u = &self.units[i];
        !u.atom
            && self.text[u.range.clone()]
                .chars()
                .all(|c| c == ' ' || c == '\t')
    }

    /// The line from [`Breaker::at`] at most `max` wide: to the last break
    /// opportunity that fits (a SHY's hyphen counted), else to the last
    /// cluster that does (overflow-wrap: anywhere), its first unit always.
    pub(crate) fn next(&self, max: f32) -> Step {
        let (units, prefix, start) = (&self.units, &self.prefix, self.at);
        let (mut at, mut best) = (start, start);
        while at < units.len() {
            let u = &units[at];
            if !u.atom && self.text[u.range.clone()].contains('\n') {
                return Step {
                    end: at,
                    resume: at + 1,
                    wrapped: false,
                    last: false,
                };
            }
            if at > start
                && (u.atom || units[at - 1].atom || self.opportunities.contains(&u.range.start))
            {
                let extra = if self.shy(at - 1) {
                    self.hyphens[units[at - 1].span]
                } else {
                    0.0
                };
                // Its trailing spaces hang.
                let mut end = at;
                while end > start + 1 && self.space(end - 1) {
                    end -= 1;
                }
                if prefix[end] - prefix[start] + extra <= max {
                    best = at;
                }
            }
            if !self.space(at) && prefix[at + 1] - prefix[start] > max && at > start {
                let mut end = if best > start { best } else { at };
                if best == start && end > start + 1 && self.shy(end - 1) {
                    // A discretionary glyph that does not fit cannot turn
                    // an emergency grapheme break into an overflowing
                    // hyphen.
                    end -= 1;
                }
                return Step {
                    end,
                    resume: end,
                    wrapped: true,
                    last: false,
                };
            }
            at += 1;
        }
        Step {
            end: units.len(),
            resume: units.len(),
            wrapped: false,
            last: true,
        }
    }

    /// The line `step` ends, from [`Breaker::at`]: its spans (a taken SHY
    /// as a hyphen, the spaces at a soft break gone), their indices in the
    /// paragraph, and its paragraph's direction.
    fn content(&self, step: &Step) -> Planned {
        let (a, b) = (self.at, step.end);
        let mut styles = self.content.spans.clone();
        for style in &mut styles {
            style.text.clear();
        }
        let mut line = crate::rich::RichContent {
            spans: Vec::new(),
            base: self.content.base,
            align: self.content.align,
            wrap: self.content.wrap,
        };
        let mut b_end = b;
        while step.wrapped && b_end > a + 1 && self.space(b_end - 1) {
            b_end -= 1;
        }
        let mut spans = Vec::new();
        for (i, u) in self.units.iter().enumerate().take(b_end).skip(a) {
            let mut value = self.text[u.range.clone()].to_owned();
            if step.wrapped && i + 1 == b_end && self.shy(i) {
                value.pop();
                value.push('-');
            }
            if !u.atom && spans.last() == Some(&u.span) {
                line.spans.last_mut().unwrap().text.push_str(&value);
            } else {
                let mut span = styles[u.span].clone();
                span.text = value;
                line.spans.push(span);
                spans.push(u.span);
            }
        }
        if line.spans.is_empty()
            && let Some(first) = styles.first()
        {
            let mut span = first.clone();
            span.text.clear();
            span.atom = None;
            line.spans.push(span);
            spans.push(0);
        }
        let byte = self.units.get(a).map_or(self.text.len(), |u| u.range.start);
        let paragraph = self.paragraphs.partition_point(|(start, _)| *start <= byte);
        let base = self
            .paragraphs
            .get(paragraph.wrapping_sub(1))
            .map_or(Direction::Ltr, |p| p.1);
        (line, spans, base)
    }

    /// The line `step` ends, shaped (`None`: it cannot be).
    pub(crate) fn lines(
        &self,
        step: &Step,
        fonts: &dyn crate::layout::Metrics,
    ) -> Option<Vec<crate::rich::RichLine>> {
        reshape(self.content(step), fonts)
    }

    /// Moves past the line `step` ends; true when it was the last.
    pub(crate) fn take(&mut self, step: &Step) -> bool {
        self.at = step.resume;
        step.last
    }
}
