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
        let source = &text[c.text_range()];
        if source.contains('\n') {
            ranges.push((start, at + 1, false));
            at += 1;
            start = at;
            best = at;
            continue;
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
                    let ch = file.character_for_glyph(g.id, source, &|c| font.in_atlas(c));
                    out.push(VisualGlyph {
                        ch,
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
    use crate::rich::{Piece, RichLine, Segment, span_metrics, span_tracking, span_width};
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
        for atom in boxes {
            builder.push_inline_box(atom);
        }
        let mut layout = builder.build(&text);
        let widths = layout.calculate_content_widths();
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
                                glyphs.push(VisualGlyph {
                                    ch: file
                                        .character_for_glyph(g.id, source, &|c| font.in_atlas(c)),
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
                        });
                    }
                }
            }
            let (mut height, mut ascent, mut descent) = (0.0f32, 0.0f32, 0.0f32);
            let mut pieces = Vec::new();
            let mut x = 0.0;
            for segment in &segments {
                let (h, a, d) = span_metrics(&content.spans[segment.span], fonts);
                height = height.max(h);
                ascent = ascent.max(a);
                descent = descent.min(d);
                pieces.push(Piece {
                    span: segment.span,
                    text: segment.text.clone(),
                    x,
                    width: segment.width,
                });
                x += segment.width;
            }
            if segments.is_empty() {
                height = content.spans.first().map_or(0.0, |s| s.step.line_height);
            }
            lines.push(RichLine {
                pieces,
                width: x,
                height,
                baseline: (height - ascent + descent) / 2.0 + ascent,
                shaped: Some(segments),
            });
        }
        Some((lines, (widths.max, widths.min)))
    })
}
