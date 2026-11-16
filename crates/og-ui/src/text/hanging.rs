//! Hanging punctuation (CSS Text 4 §9.2.1): a bracket or quote at the
//! paragraph's first line's start or last line's end, or a stop or comma
//! at a line's end, is set outside the line box. It takes no room on its
//! line, so neither the line's fit nor its alignment measures it. Where a
//! line ends depends on it, so [`super::line_edges`] plans it on the
//! lines a pass broke.

use super::line_edges::{Metrics, Paragraph, Plan, Unit};
use crate::rich::RichContent;
use og_i18n::spacing::Hangs;

/// CSS `hanging-punctuation`: `none | [ first || [ force-end | allow-end
/// ] || last ]`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct HangingPunctuation {
    pub first: bool,
    pub end: Option<HangEnd>,
    pub last: bool,
}

/// Whether a stop or comma at a line's end hangs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HangEnd {
    /// `force-end`: always.
    Force,
    /// `allow-end`: only where the line does not otherwise fit.
    Allow,
}

impl HangingPunctuation {
    pub fn parse(v: &str) -> Option<Self> {
        let mut out = Self::default();
        if v == "none" {
            return Some(out);
        }
        for w in v.split_whitespace() {
            let seen = match w {
                "first" => std::mem::replace(&mut out.first, true),
                "last" => std::mem::replace(&mut out.last, true),
                "force-end" => out.end.replace(HangEnd::Force).is_some(),
                "allow-end" => out.end.replace(HangEnd::Allow).is_some(),
                _ => return None,
            };
            if seen {
                return None;
            }
        }
        (out != Self::default()).then_some(out)
    }
}

/// Whether a paragraph hangs any punctuation.
pub(super) fn applies(content: &RichContent) -> bool {
    content.hanging != HangingPunctuation::default()
        && content.spans.iter().any(|s| {
            s.atom.is_none() && s.text.chars().any(|c| og_i18n::spacing::hangs(c).is_some())
        })
}

/// The units the paragraph hangs on its lines, into `out` (after its CJK
/// spacing, whose trim they hang at): its first line's first unit (an
/// opening bracket or quote, or U+3000, under `first`) as room taken back
/// before the line, and each line's last (a stop or comma under
/// `force-end`, or under `allow-end` where it was pulled back onto the
/// line; the paragraph's last closing bracket or quote under `last`) as
/// its width taken back. A unit inline padding or a border separates
/// from the line's edge does not hang (§9.2.1).
pub(super) fn plan(p: &Paragraph, metrics: &Metrics, pulled: &[usize], out: &mut Plan) {
    let h = p.content.hanging;
    let indent = p.content.indent;
    if h.first
        && !indent.hanging
        && !indent.each_line
        && let Some(u) = p.units.iter().find(|u| p.starts_line(u))
        && p.line_of(u.range.start).is_some_and(|l| l.range.start == 0)
        && matches!(
            og_i18n::spacing::hangs(u.c),
            Some(Hangs::Opening | Hangs::Quote)
        )
        && !p.barrier(u.range.start)
    {
        out.hang_start = width(p, metrics, u, out);
    }
    for u in &p.units {
        if pulled.contains(&u.range.start) && hangs_at_end(p, u, pulled) && !p.barrier(u.range.end)
        {
            let w = width(p, metrics, u, out);
            out.hang.push((u.range.clone(), w));
        }
    }
}

/// Hanging an already fitting line end is a placement operation, not a
/// new wrap opportunity: removing its advance before breaking would let
/// the next word occupy the punctuation's place and oscillate the plan.
pub(super) fn finish(
    content: &RichContent,
    fonts: &dyn crate::layout::Metrics,
    line: &mut crate::rich::RichLine,
    range: std::ops::Range<usize>,
    used: &Plan,
) {
    if !applies(content) {
        return;
    }
    let lines = [super::line_edges::Line {
        range,
        first: false,
        wrapped: false,
        room: 0.0,
        width: line.width,
    }];
    let p = Paragraph::new(content, &lines);
    let Some(u) = p.units.iter().find(|u| p.ends_line(u)) else {
        return;
    };
    if !hangs_at_end(&p, u, &[])
        || p.barrier(u.range.end)
        || used.hang.iter().any(|(r, _)| *r == u.range)
    {
        return;
    }
    let w = width(&p, &Metrics(content, fonts), u, used);
    line.width -= w;
    if let Some(piece) = line.pieces.last_mut() {
        piece.width -= w;
    }
    if let Some(segment) = line.shaped.as_mut().and_then(|s| s.last_mut()) {
        segment.width -= w;
        if let Some(g) = segment.glyphs.as_mut().and_then(|g| g.last_mut()) {
            g.advance -= w / content.spans[segment.span].step.size;
        }
    }
}

/// Unit `u`'s width as set: its span's measure, half of it trimmed.
fn width(p: &Paragraph, metrics: &Metrics, u: &Unit, plan: &Plan) -> f32 {
    let w = metrics.width(u.span, &p.text[u.range.clone()]);
    if plan.halt.contains(&u.range) {
        w / 2.0
    } else {
        w
    }
}

/// Whether `u`, as the last on its line, hangs there.
fn hangs_at_end(p: &Paragraph, u: &Unit, pulled: &[usize]) -> bool {
    let h = p.content.hanging;
    match og_i18n::spacing::hangs(u.c) {
        Some(Hangs::Stop) => match h.end {
            Some(HangEnd::Force) => true,
            Some(HangEnd::Allow) => pulled.contains(&u.range.start),
            None => false,
        },
        Some(Hangs::Closing | Hangs::Quote) => h.last && p.ends_paragraph(u),
        _ => false,
    }
}

/// Whether `u` would hang as a line's last unit: a stop under `force-end`
/// or `allow-end`, the paragraph's last closing bracket or quote under
/// `last`.
pub(super) fn may_hang_at_end(p: &Paragraph, u: &Unit) -> bool {
    let h = p.content.hanging;
    !p.barrier(u.range.end)
        && match og_i18n::spacing::hangs(u.c) {
            Some(Hangs::Stop) => h.end.is_some(),
            Some(Hangs::Closing | Hangs::Quote) => h.last && p.ends_paragraph(u),
            _ => false,
        }
}

/// The width a stop `allow-end` hangs at a line's end takes back on the
/// line unhung (`None`: `u` is no such stop).
pub(super) fn allowed(p: &Paragraph, u: &Unit, used: &Plan) -> Option<f32> {
    (p.content.hanging.end == Some(HangEnd::Allow)
        && og_i18n::spacing::hangs(u.c) == Some(Hangs::Stop))
    .then(|| used.hang.iter().find(|(r, _)| *r == u.range).map(|h| h.1))
    .flatten()
}
