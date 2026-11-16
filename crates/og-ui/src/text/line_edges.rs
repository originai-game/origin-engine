//! What a paragraph sets that depends on where its lines break: CJK
//! spacing ([`super::spacing`]) at a line's start and end and between
//! neighbours on one line, and hanging punctuation ([`super::hanging`]). Each pass plans it on the lines the pass
//! before broke ([`Sides`]) and the paragraph is shaped again until the
//! plan holds; a unit the next line starts with is pulled back onto a
//! line where, set as a line's last, it fits.

use super::{hanging, spacing};
use crate::rich::RichContent;
use std::ops::Range;
use std::sync::Arc;

/// A line a pass broke, as its edges see it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Line {
    /// Its text's byte range in the paragraph.
    pub range: Range<usize>,
    /// Whether it is the paragraph's first line or follows a forced break
    /// (`space-first` spaces its start).
    pub first: bool,
    /// Whether it ends at a soft wrap opportunity, so the next line's
    /// start may be pulled back onto it.
    pub wrapped: bool,
    /// Its room and the width its content takes, CSS px.
    pub room: f32,
    pub width: f32,
}

/// Where a paragraph's line edges are planned: the lines a pass broke
/// (before the first, a line between each forced break), the units
/// pulled back to a line's end (their closing punctuation trimmed, no
/// autospace after them), and those tried and left inside a line, not
/// tried again.
#[derive(Clone, Debug, Default)]
pub(crate) struct Sides {
    lines: Vec<Line>,
    pulled: Vec<usize>,
    rejected: Vec<usize>,
}

/// What a paragraph sets on its lines: the fullwidth punctuation set
/// half-width, room after units (CSS px, from `text-autospace`), and
/// what hangs: the units at a line's end and the width each takes back,
/// and the width hanging before the first line.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Plan {
    pub halt: Vec<Range<usize>>,
    pub space: Vec<(Range<usize>, f32)>,
    pub hang: Vec<(Range<usize>, f32)>,
    pub hang_start: f32,
}

impl Sides {
    pub fn new(content: &RichContent) -> Self {
        let (text, _, _) = units(content);
        let mut lines = Vec::new();
        let mut start = 0;
        for end in text
            .match_indices('\n')
            .map(|(at, _)| at + 1)
            .chain([text.len()])
        {
            lines.push(Line {
                range: start..end,
                first: true,
                wrapped: false,
                room: f32::INFINITY,
                width: 0.0,
            });
            start = end;
        }
        Self {
            lines,
            ..Self::default()
        }
    }

    /// What the paragraph sets on these lines.
    pub fn plan(&self, content: &RichContent, fonts: &dyn crate::layout::Metrics) -> Plan {
        let mut out = Plan::default();
        if !applies(content) {
            return out;
        }
        let p = Paragraph::new(content, &self.lines);
        let metrics = Metrics(content, fonts);
        if spacing::applies(content) {
            spacing::plan(&p, &metrics, &self.pulled, &mut out);
        }
        if hanging::applies(content) {
            hanging::plan(&p, &metrics, &self.pulled, &mut out);
        }
        out
    }

    /// The sides a pass that broke `lines` (shaped with `used`) wants when
    /// they plan otherwise: a closing bracket `allow-end` trims at a
    /// line's end is kept where its full glyph does not fit (`trim-end`
    /// trims it there anyway), and the unit a wrapped line's next one
    /// starts with (its characters to the next soft wrap opportunity) is
    /// pulled back when, without the room it would lose as a line's last
    /// ([`spacing::lost_at_end`]), it fits. `advance`: what a byte range
    /// takes as shaped.
    pub fn next(
        &self,
        content: &RichContent,
        fonts: &dyn crate::layout::Metrics,
        lines: Vec<Line>,
        used: &Plan,
        advance: &dyn Fn(Range<usize>) -> f32,
    ) -> Option<Self> {
        if !applies(content) {
            return None;
        }
        let p = Paragraph::new(content, &lines);
        let ends: Vec<usize> = lines.iter().map(|l| p.body_end(l)).collect();
        let mut rejected = self.rejected.clone();
        for at in &self.pulled {
            let end = p
                .units
                .iter()
                .find(|u| u.range.start == *at)
                .map(|u| u.range.end);
            if !end.is_some_and(|e| ends.contains(&e)) {
                rejected.push(*at);
            }
        }
        let breaks = opportunities(content, &p.text);
        let mut pulled = Vec::new();
        for (k, line) in lines.iter().enumerate() {
            // Its last unit, trimmed or hung: kept where it does not fit
            // otherwise.
            if let Some(u) = p.units.iter().find(|u| u.range.end == ends[k])
                && self.pulled.contains(&u.range.start)
            {
                let restored = match hanging::allowed(&p, u, used) {
                    Some(w) => Some(w),
                    None => spacing::trims_at_end(&p, u).then(|| advance(u.range.clone())),
                };
                if restored.is_some_and(|w| line.width + w > line.room + 0.01) {
                    pulled.push(u.range.start);
                    continue;
                }
            }
            let Some(next) = lines.get(k + 1).filter(|_| line.wrapped) else {
                continue;
            };
            // The next line's first unit, to its next soft wrap opportunity.
            let Some(first) = p
                .units
                .iter()
                .position(|u| u.range.start == next.range.start)
            else {
                continue;
            };
            let Some(last) = p.units[first..]
                .iter()
                .position(|u| breaks.contains(&u.range.end) || u.range.end >= ends[k + 1])
                .map(|n| first + n)
            else {
                continue;
            };
            let u = &p.units[last];
            let lost = if hanging::may_hang_at_end(&p, u) {
                advance(u.range.clone())
            } else {
                spacing::lost_at_end(&p, u, used, advance)
            };
            if lost > 0.0
                && !rejected.contains(&u.range.start)
                && line.width + advance(p.units[first].range.start..u.range.end) - lost
                    <= line.room + 0.01
            {
                pulled.push(u.range.start);
            }
        }
        let out = Self {
            lines,
            pulled,
            rejected,
        };
        (out.plan(content, fonts) != *used).then_some(out)
    }
}

/// Whether a paragraph sets anything at its line edges.
fn applies(content: &RichContent) -> bool {
    spacing::applies(content) || hanging::applies(content)
}

/// A paragraph on its lines, as its edges are planned: its text as its
/// shaper sees it (its spans' text but atomic inlines) and its units.
pub(super) struct Paragraph<'a> {
    pub content: &'a RichContent,
    pub text: String,
    pub units: Vec<Unit>,
    /// The byte offsets between two units nothing reaches across (an
    /// atomic inline or an inline box's margin, border or padding).
    barriers: Vec<usize>,
    lines: &'a [Line],
}

impl<'a> Paragraph<'a> {
    fn new(content: &'a RichContent, lines: &'a [Line]) -> Self {
        let (text, units, barriers) = units(content);
        Self {
            content,
            text,
            units,
            barriers,
            lines,
        }
    }

    pub fn line_of(&self, at: usize) -> Option<&Line> {
        self.lines.iter().find(|l| l.range.contains(&at))
    }

    /// Whether `a` and `b` are neighbours on one line with nothing between
    /// them.
    pub fn joined(&self, a: &Unit, b: &Unit) -> bool {
        a.range.end == b.range.start
            && !self.barriers.contains(&b.range.start)
            && self.line_of(a.range.start) == self.line_of(b.range.start)
    }

    /// Where a line's content ends, before the spaces and break that hang.
    fn body_end(&self, l: &Line) -> usize {
        l.range.start
            + self.text[l.range.clone()]
                .trim_end_matches([' ', '\t', '\n'])
                .len()
    }

    /// Whether `u` is its line's first unit past leading spaces.
    pub fn starts_line(&self, u: &Unit) -> bool {
        self.line_of(u.range.start).is_some_and(|l| {
            let line = &self.text[l.range.clone()];
            l.range.start + line.len() - line.trim_start().len() == u.range.start
        })
    }

    /// Whether `u` is its line's last unit before the spaces that hang.
    pub fn ends_line(&self, u: &Unit) -> bool {
        self.line_of(u.range.start)
            .is_some_and(|l| self.body_end(l) == u.range.end)
    }

    /// Whether inline padding, a border or an atomic inline is at byte
    /// `at`.
    pub fn barrier(&self, at: usize) -> bool {
        self.barriers.contains(&at)
    }

    /// Whether `u` is the paragraph's last unit before the spaces that
    /// hang.
    pub fn ends_paragraph(&self, u: &Unit) -> bool {
        self.text.trim_end_matches([' ', '\t', '\n']).len() == u.range.end
    }

    /// Its content language.
    pub fn lang(&self, u: &Unit) -> Option<Arc<str>> {
        self.content.spans[u.span].step.features.language()
    }
}

/// A typographic character unit of the paragraph: its bytes, its base
/// character and its span.
pub(super) struct Unit {
    pub range: Range<usize>,
    pub c: char,
    pub span: usize,
}

/// The paragraph's text as its shaper sees it, its units and its
/// barriers ([`Paragraph`]).
fn units(content: &RichContent) -> (String, Vec<Unit>, Vec<usize>) {
    let mut text = String::new();
    let mut owner = Vec::new();
    let mut barriers = Vec::new();
    for (k, s) in content.spans.iter().enumerate() {
        if s.atom.is_some() {
            barriers.push(text.len());
            continue;
        }
        let [before, after] = s.room();
        if before != 0.0 {
            barriers.push(text.len());
        }
        text.push_str(&s.text);
        owner.extend(std::iter::repeat_n(k, s.text.len()));
        if after != 0.0 {
            barriers.push(text.len());
        }
    }
    let bytes: Vec<usize> = text
        .char_indices()
        .map(|(b, _)| b)
        .chain([text.len()])
        .collect();
    let units = og_i18n::graphemes(&text)
        .into_iter()
        .map(|(a, b)| {
            let range = bytes[a]..bytes[b];
            Unit {
                c: text[range.clone()].chars().next().unwrap_or(' '),
                span: owner[range.start],
                range,
            }
        })
        .collect();
    (text, units, barriers)
}

/// The byte offsets of the text's soft wrap opportunities, as the
/// paragraph's line breaker finds them (UAX #14 at its `line-break`).
fn opportunities(content: &RichContent, text: &str) -> Vec<usize> {
    use crate::rich::LineBreak;
    let config = parley_engine::LineBreakConfig {
        line_break: match content.line_break {
            LineBreak::Auto | LineBreak::Strict => parley::LineBreak::Strict,
            LineBreak::Loose => parley::LineBreak::Loose,
            LineBreak::Normal => parley::LineBreak::Normal,
            LineBreak::Anywhere => parley::LineBreak::Anywhere,
        },
        ..Default::default()
    };
    let mut analysis = parley_engine::Analysis::new();
    parley_engine::Analyzer::new().analyze(
        text,
        &parley_engine::AnalysisOptions {
            line_break: &[(0..text.len(), config)],
            line_break_override: Some(&super::keep_katakana),
            ..Default::default()
        },
        &mut analysis,
    );
    text.char_indices()
        .zip(analysis.char_info())
        .filter_map(|((byte, _), info)| {
            (byte > 0 && info.is_soft_wrap_opportunity()).then_some(byte)
        })
        .collect()
}

/// A span's ideographic advance and its advance of a character, CSS px.
pub(super) struct Metrics<'a>(&'a RichContent, &'a dyn crate::layout::Metrics);

impl Metrics<'_> {
    fn of(&self, k: usize) -> (&og_text::text::FontMetrics, f32) {
        let s = &self.0.spans[k];
        (
            self.1.metrics_styled(s.font, s.query(), &s.text),
            s.step.size,
        )
    }

    pub fn ic(&self, k: usize) -> f32 {
        let (m, size) = self.of(k);
        m.aspect(og_text::text::AspectMetric::IcWidth) * size
    }

    pub fn advance(&self, k: usize, c: char) -> f32 {
        let (m, size) = self.of(k);
        m.advance(c) * size
    }

    /// U+202F NARROW NO-BREAK SPACE's advance: else a thin space's, else
    /// a fifth of an em (a thin space's traditional width).
    /// What `text` of span `k` takes, with its tracking.
    pub fn width(&self, k: usize, text: &str) -> f32 {
        crate::rich::span_width(&self.0.spans[k], text, self.1)
    }

    pub fn narrow(&self, k: usize) -> f32 {
        [self.advance(k, '\u{202f}'), self.advance(k, '\u{2009}')]
            .into_iter()
            .find(|a| *a > 0.0)
            .unwrap_or(self.of(k).1 / 5.0)
    }
}
