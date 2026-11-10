//! Cooked-font paragraph layout. The collection never discovers host fonts.
use super::{Features, FontMetrics, VisualGlyph};
use crate::shape::FontFile;
use og_i18n::Direction;
use parley::{FontContext, Layout, LayoutContext, StyleProperty};
use std::{cell::RefCell, sync::Arc};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Brush {
    pub font: u8,
    pub span: usize,
}

pub struct Context {
    pub fonts: FontContext,
    pub layout: LayoutContext<Brush>,
    pub files: Vec<Arc<FontFile>>,
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
    pub static CONTEXT: RefCell<Context> = RefCell::new(Context::default());
}

impl Context {
    pub fn family(&mut self, file: Arc<FontFile>) -> String {
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
pub fn discretionary(
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
pub fn layout(
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

pub fn visual(metrics: &FontMetrics, text: &str, layout: &Layout<Brush>) -> Vec<VisualGlyph> {
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
