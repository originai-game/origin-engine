//! Cooked-font paragraph layout with Parley. The collection never
//! discovers host fonts: it holds the cooked fonts' files and the
//! layout-only faces built for fonts without a usable one
//! ([`super::layout_font`]).
#[cfg(feature = "shaping")]
use super::VisualGlyph;
use super::{Features, FontMetrics};
use crate::shape::FontFile;
use og_i18n::Direction;
use parley::{FontContext, Layout, LayoutContext, StyleProperty};
use std::collections::{BTreeSet, HashMap};
use std::ops::Range;
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
    /// The layout-only faces registered, by a hash of their bytes: their
    /// family's number.
    built: HashMap<u64, usize>,
    /// Each of `files`' family name and `font-variation-settings`, made
    /// once when it is registered.
    names: Vec<(String, String)>,
    /// Each feature set's CSS, made once.
    settings: HashMap<Features, String>,
    /// What [`measure`] builds into, its buffers kept between calls: the
    /// runs of a text, the face of each, and the layout.
    runs: Vec<(Range<usize>, usize)>,
    faces: Vec<usize>,
    scratch: Layout<Brush>,
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
            built: HashMap::new(),
            names: Vec::new(),
            settings: HashMap::new(),
            runs: Vec::new(),
            faces: Vec::new(),
            scratch: Layout::new(),
        }
    }
}

thread_local! {
    pub static CONTEXT: RefCell<Context> = RefCell::new(Context::default());
}

impl Context {
    pub fn family(&mut self, file: Arc<FontFile>) -> String {
        let index = self.face(&file);
        self.names[index].0.clone()
    }

    /// `file`'s place in `files`, registered the first time it is seen.
    fn face(&mut self, file: &Arc<FontFile>) -> usize {
        if let Some(index) = self.files.iter().position(|f| Arc::ptr_eq(f, file)) {
            return index;
        }
        let index = self.files.len();
        let name = format!("og-{index}");
        self.fonts.collection.register_fonts(
            parley::fontique::Blob::new(file.bytes().clone()),
            Some(parley::fontique::FontInfoOverride {
                family_name: Some(&name),
                ..Default::default()
            }),
        );
        self.files.push(file.clone());
        let variations = self.variations(&name);
        self.names.push((name, variations));
        index
    }

    /// The CSS `font-variation-settings` of `family`'s file: an instance's
    /// location, empty for any other face.
    pub fn variations(&self, family: &str) -> String {
        let file = family
            .strip_prefix("og-")
            .and_then(|i| i.parse::<usize>().ok())
            .and_then(|i| self.files.get(i));
        file.map_or_else(String::new, |f| {
            f.variations()
                .iter()
                .map(|(t, v)| format!("\"{}\" {v}", String::from_utf8_lossy(t)))
                .collect::<Vec<_>>()
                .join(", ")
        })
    }

    /// The family of a layout-only face for `chars` in `metrics`, built
    /// and registered once per distinct face.
    fn built(&mut self, metrics: &FontMetrics, chars: &BTreeSet<char>) -> String {
        let bytes = super::layout_font::build(metrics, chars);
        let key = og_core::hash::hash_bytes(&bytes);
        let next = self.built.len();
        let index = *self.built.entry(key).or_insert_with(|| {
            let name = format!("og-layout-{next}");
            self.fonts.collection.register_fonts(
                parley::fontique::Blob::new(Arc::new(bytes)),
                Some(parley::fontique::FontInfoOverride {
                    family_name: Some(&name),
                    ..Default::default()
                }),
            );
            next
        });
        format!("og-layout-{index}")
    }

    /// Starts over when it holds too many faces (game reloads, a headless
    /// text growing its characters one at a time).
    fn bound(&mut self) {
        if self.files.len() + self.built.len() > 128 {
            *self = Context::default();
        }
    }
}

/// A paragraph's styles: its size, tracking (none on a shaped script's
/// run, whose letters join), features, and each font run's family and
/// brush, as both layouts below set them.
/// `names` holds the family and variations of each run's face, run `i`'s
/// at `faces[i]`; `settings` is `features`' CSS.
#[allow(clippy::too_many_arguments)]
fn styled<'a>(
    layout: &'a mut LayoutContext<Brush>,
    fonts: &'a mut FontContext,
    text: &'a str,
    size: f32,
    tracking: f32,
    features: Features,
    settings: &str,
    direction: Direction,
    runs: &[(Range<usize>, usize)],
    names: &[(String, String)],
    faces: &[usize],
    metrics: &FontMetrics,
) -> parley::RangedBuilder<'a, Brush> {
    let mut builder = layout.ranged_builder(fonts, text, 1.0, false);
    builder.set_line_break_override(Some(&super::keep_katakana));
    builder.set_base_direction(match direction {
        Direction::Ltr => parley::BaseDirection::Ltr,
        Direction::Rtl => parley::BaseDirection::Rtl,
    });
    builder.push_default(StyleProperty::FontSize(size));
    builder.push_default(StyleProperty::OverflowWrap(parley::OverflowWrap::Anywhere));
    builder.push_default(StyleProperty::FontFeatures(settings.into()));
    // Shaped in its language, so a face's `locl` forms for it apply.
    if let Some(l) = features.language().and_then(|l| l.parse().ok()) {
        builder.push_default(StyleProperty::Locale(Some(l)));
    }
    for ((range, k), &face) in runs.iter().zip(faces) {
        // A variable font's instance is laid out at its location.
        let (family, variations) = &names[face];
        builder.push(
            StyleProperty::FontFamily(family.as_str().into()),
            range.clone(),
        );
        if !variations.is_empty() {
            builder.push(
                StyleProperty::FontVariations(variations.as_str().into()),
                range.clone(),
            );
        }
        // Each face at its own size (`font-size-adjust`).
        let scale = metrics.part(*k).map_or(1.0, |m| m.scale);
        if scale != 1.0 {
            builder.push(StyleProperty::FontSize(size * scale), range.clone());
        }
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
    builder
}

/// A text as it is edited: laid out by Parley in every font, at `size`
/// CSS px, its bidi base `direction`, broken at `width` (none: one line
/// per paragraph), each line `line_height` tall, its lines aligned to
/// `start`'s edge (the UI's start, whichever way the text runs). A font Parley
/// shapes from its file is laid out as [`layout`] lays it out for drawing;
/// any other (fixed metrics, an atlas without its file, a pixel font)
/// through a layout-only face of its own advances and kerning, so Parley's
/// cursor and selection place a caret where og-text draws the text.
#[allow(clippy::too_many_arguments)]
pub fn editable(
    metrics: &FontMetrics,
    text: &str,
    size: f32,
    tracking: f32,
    direction: Direction,
    start: Direction,
    width: Option<f32>,
    line_height: f32,
) -> Layout<Brush> {
    let runs = metrics.font_runs(text);
    CONTEXT.with_borrow_mut(|cx| {
        cx.bound();
        let mut families = Vec::with_capacity(runs.len());
        for (_, k) in &runs {
            let m = metrics.part(*k).expect("a font of the run");
            let family = match &m.file {
                Some(file) if cfg!(feature = "shaping") && m.fixed.is_none() && m.pixel == 0 => {
                    cx.family(file.clone())
                }
                _ => {
                    let chars = runs
                        .iter()
                        .filter(|(_, j)| j == k)
                        .flat_map(|(r, _)| text[r.clone()].chars())
                        .collect();
                    cx.built(m, &chars)
                }
            };
            families.push(family);
        }
        let names: Vec<(String, String)> = families
            .into_iter()
            .map(|f| {
                let v = cx.variations(&f);
                (f, v)
            })
            .collect();
        let faces: Vec<usize> = (0..names.len()).collect();
        let mut builder = styled(
            &mut cx.layout,
            &mut cx.fonts,
            text,
            size,
            tracking,
            Features::NONE,
            "",
            direction,
            &runs,
            &names,
            &faces,
            metrics,
        );
        builder.push_default(StyleProperty::LineHeight(parley::LineHeight::Absolute(
            line_height,
        )));
        let mut layout = builder.build(text);
        layout.break_all_lines(width);
        let align = match start {
            Direction::Ltr => parley::Alignment::Left,
            Direction::Rtl => parley::Alignment::Right,
        };
        layout.align(align, parley::AlignmentOptions::default());
        layout
    })
}

/// Parley 0.12 treats SHY as default-ignorable, but does not reserve or
/// draw the discretionary glyph. Select its breaks from Parley's shaped
/// graphemes and UAX #14 opportunities, reserving the hyphen only at a
/// taken break. Prefix advances keep a long discretionary word linear.
#[cfg(feature = "shaping")]
pub fn discretionary(
    metrics: &FontMetrics,
    text: &str,
    size: f32,
    tracking: f32,
    features: Features,
    max: Option<f32>,
    emergency: bool,
) -> Option<Vec<super::Line>> {
    if !text.contains('\u{ad}') {
        return None;
    }
    if text.contains('\n') {
        return Some(
            text.split('\n')
                .flat_map(|paragraph| {
                    super::break_lines_with_overflow(
                        paragraph, metrics, size, tracking, features, max, emergency,
                    )
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
            if prefix[at] - prefix[start] + extra <= max || (!emergency && best == start) {
                best = at;
            }
        }
        if prefix[at + 1] - prefix[start] > max && at > start && (emergency || best > start) {
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
    layout_with_overflow(
        metrics, text, size, tracking, features, direction, width, true,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn layout_with_overflow(
    metrics: &FontMetrics,
    text: &str,
    size: f32,
    tracking: f32,
    features: Features,
    direction: Direction,
    width: Option<f32>,
    emergency: bool,
) -> Option<Layout<Brush>> {
    let runs = metrics.font_runs(text);
    // `tnum` on a face whose figures stay proportional under it (Meridian's
    // Chakra Petch has no `tnum`): the run path sets its digits on the
    // widest digit's advance (FontMetrics::tabulate), which a shaped layout
    // would not, and a counting clock would change width as it runs.
    if runs.iter().any(|(_, k)| {
        metrics
            .part(*k)
            .is_some_and(|m| m.tabulates(text, features))
    }) {
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
        cx.bound();
        let faces: Vec<usize> = files.iter().map(|f| cx.face(f)).collect();
        let settings = features.css();
        let mut builder = styled(
            &mut cx.layout,
            &mut cx.fonts,
            text,
            size,
            tracking,
            features,
            &settings,
            direction,
            &runs,
            &cx.names,
            &faces,
            metrics,
        );
        if !emergency {
            builder.push_default(StyleProperty::OverflowWrap(parley::OverflowWrap::Normal));
        }
        let mut layout = builder.build(text);
        layout.break_all_lines(width);
        Some(layout)
    })
}

#[cfg(feature = "shaping")]
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

/// The widest line of `text` (em, at size 1) and its number of lines,
/// broken at `width` (em; none: only at `\n`), as [`layout`] would lay it
/// out, without making a layout: the contexts, face names, runs and the
/// layout's buffers are kept between calls, so measuring a text of a face
/// already seen allocates nothing. `None` where [`layout`] declines (no
/// font file, a tabulated face) or a soft hyphen needs its own breaking.
pub fn measure(
    metrics: &FontMetrics,
    text: &str,
    tracking: f32,
    features: Features,
    width: Option<f32>,
) -> Option<(f32, usize)> {
    if text.contains('\u{ad}') {
        return None;
    }
    CONTEXT.with_borrow_mut(|cx| {
        cx.bound();
        let mut runs = std::mem::take(&mut cx.runs);
        metrics.font_runs_into(text, &mut runs);
        let usable = runs.iter().all(|(_, k)| {
            metrics.part(*k).is_some_and(|m| {
                !m.tabulates(text, features)
                    && m.fixed.is_none()
                    && m.pixel == 0
                    && m.file.is_some()
            })
        });
        let out = usable.then(|| {
            cx.faces.clear();
            for (_, k) in &runs {
                let file = metrics
                    .part(*k)
                    .and_then(|m| m.file.as_ref())
                    .expect("checked above");
                let face = cx.face(file);
                cx.faces.push(face);
            }
            cx.settings
                .entry(features)
                .or_insert_with(|| features.css());
            let Context {
                layout,
                fonts,
                names,
                settings,
                faces,
                scratch,
                ..
            } = cx;
            let direction = match width {
                Some(_) => og_i18n::paragraph_direction(text, Direction::Ltr),
                None => Direction::Ltr,
            };
            let builder = styled(
                layout,
                fonts,
                text,
                1.0,
                tracking,
                features,
                &settings[&features],
                direction,
                &runs,
                names,
                faces,
                metrics,
            );
            builder.build_into(scratch, text);
            scratch.break_all_lines(width);
            let wrapped = width.is_some();
            let widest = scratch
                .lines()
                .map(|l| {
                    let m = l.metrics();
                    if wrapped {
                        m.advance - m.hanging_advance
                    } else {
                        m.advance
                    }
                })
                .fold(0.0, f32::max);
            (widest, scratch.len())
        });
        cx.runs = runs;
        out
    })
}
