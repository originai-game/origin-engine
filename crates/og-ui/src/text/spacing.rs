//! CJK spacing (CSS Text 4 §8.4, §8.5): `text-autospace` puts an eighth
//! of an ideograph's advance between ideographs and Latin letters or
//! digits on a line, and `text-spacing-trim` sets fullwidth punctuation
//! half-width at a line's start or end and between punctuation pairs.
//! Each depends on where the lines break: [`super::line_edges`] plans
//! them on the lines a pass broke.
//!
//! A trimmed glyph is set by its font's `halt` feature (§8.5.1: never
//! `hwid`, whose glyphs differ); a face without it keeps its full width,
//! as §8.5.1 leaves to the UA where trimming may clip.

use super::line_edges::{Metrics, Paragraph, Plan, Unit};
use crate::rich::RichContent;
use og_i18n::spacing::{Fullwidth, InterScript};
use std::ops::Range;

/// CSS `text-autospace` (CSS Text 4 §8.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Autospace {
    /// `ideograph-alpha`.
    pub alpha: bool,
    /// `ideograph-numeric`.
    pub numeric: bool,
    /// `punctuation`: French's no-break spaces before `:;!?` and inside
    /// guillemets.
    pub punctuation: bool,
    /// `replace`: a U+0020 where the space goes is replaced by it.
    pub replace: bool,
}

impl Default for Autospace {
    /// `normal`: `ideograph-alpha ideograph-numeric`.
    fn default() -> Self {
        Self {
            alpha: true,
            numeric: true,
            punctuation: false,
            replace: false,
        }
    }
}

impl Autospace {
    /// `normal | <autospace> | auto`, where `<autospace>` is
    /// `no-autospace | [ ideograph-alpha || ideograph-numeric ||
    /// punctuation ] || [ insert | replace ]`; `auto` is the UA's choice,
    /// `normal`'s.
    pub fn parse(v: &str) -> Option<Self> {
        match v {
            "normal" | "auto" => return Some(Self::default()),
            "no-autospace" => {
                return Some(Self {
                    alpha: false,
                    numeric: false,
                    ..Self::default()
                });
            }
            _ => {}
        }
        let mut out = Self {
            alpha: false,
            numeric: false,
            ..Self::default()
        };
        let (mut classes, mut mode) = (false, false);
        for w in v.split_whitespace() {
            let seen = match w {
                "ideograph-alpha" => std::mem::replace(&mut out.alpha, true),
                "ideograph-numeric" => std::mem::replace(&mut out.numeric, true),
                "punctuation" => std::mem::replace(&mut out.punctuation, true),
                "insert" | "replace" => {
                    out.replace = w == "replace";
                    std::mem::replace(&mut mode, true)
                }
                _ => return None,
            };
            if seen {
                return None;
            }
            classes |= !matches!(w, "insert" | "replace");
        }
        if !classes {
            // `insert` or `replace` alone: `normal`'s classes.
            (out.alpha, out.numeric) = (true, true);
        }
        Some(out)
    }
}

/// Where `text-spacing-trim` trims at a line's start.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TrimStart {
    Trim,
    /// `space-start`.
    #[default]
    Space,
    /// `space-first`: spaced on the first line and after a forced break,
    /// trimmed on the others.
    SpaceFirst,
}

/// Where `text-spacing-trim` trims at a line's end.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TrimEnd {
    Trim,
    Space,
    /// `allow-end`: trimmed only where it does not otherwise fit.
    #[default]
    Allow,
}

/// CSS `text-spacing-trim` (CSS Text 4 §8.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SpacingTrim {
    pub start: TrimStart,
    /// `trim-adjacent` (else `space-adjacent`).
    pub adjacent: bool,
    pub end: TrimEnd,
    /// `trim-all`: every fullwidth punctuation half-width, wherever it is.
    pub all: bool,
}

impl Default for SpacingTrim {
    /// `normal`: `space-start trim-adjacent allow-end`.
    fn default() -> Self {
        Self {
            start: TrimStart::Space,
            adjacent: true,
            end: TrimEnd::Allow,
            all: false,
        }
    }
}

impl SpacingTrim {
    /// `normal | auto | space-all | trim-both | trim-all | [ trim-start |
    /// space-start | space-first ] || [ trim-adjacent | space-adjacent ]
    /// || [ trim-end | space-end | allow-end ]`, each omitted part
    /// `trim-start`, `trim-adjacent` or `allow-end`; `auto` is the UA's
    /// choice, `normal`'s.
    pub fn parse(v: &str) -> Option<Self> {
        let both = Self {
            start: TrimStart::Trim,
            adjacent: true,
            end: TrimEnd::Trim,
            all: false,
        };
        match v {
            "normal" | "auto" => return Some(Self::default()),
            "space-all" => {
                return Some(Self {
                    start: TrimStart::Space,
                    adjacent: false,
                    end: TrimEnd::Space,
                    all: false,
                });
            }
            "trim-both" => return Some(both),
            "trim-all" => return Some(Self { all: true, ..both }),
            _ => {}
        }
        let (mut start, mut adjacent, mut end) = (None, None, None);
        for w in v.split_whitespace() {
            let seen = match w {
                "trim-start" => start.replace(TrimStart::Trim).is_some(),
                "space-start" => start.replace(TrimStart::Space).is_some(),
                "space-first" => start.replace(TrimStart::SpaceFirst).is_some(),
                "trim-adjacent" => adjacent.replace(true).is_some(),
                "space-adjacent" => adjacent.replace(false).is_some(),
                "trim-end" => end.replace(TrimEnd::Trim).is_some(),
                "space-end" => end.replace(TrimEnd::Space).is_some(),
                "allow-end" => end.replace(TrimEnd::Allow).is_some(),
                _ => return None,
            };
            if seen {
                return None;
            }
        }
        Some(Self {
            start: start.unwrap_or(TrimStart::Trim),
            adjacent: adjacent.unwrap_or(true),
            end: end.unwrap_or(TrimEnd::Allow),
            all: false,
        })
    }
}

/// A paragraph's CJK spacing: its `text-autospace` and
/// `text-spacing-trim`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CjkSpacing {
    pub autospace: Autospace,
    pub trim: SpacingTrim,
}

impl CjkSpacing {
    /// Whether it may set any of `text` apart from the plain text path:
    /// `text` has fullwidth punctuation to trim, an ideograph to space
    /// from its neighbours, or (French `punctuation`) a mark to space.
    pub fn applies(&self, text: &str, lang: Option<&str>) -> bool {
        let trims = self.trim != Self::NONE.trim;
        let spaces = self.autospace.alpha || self.autospace.numeric;
        let french = self.autospace.punctuation && is_french(lang);
        text.chars().any(|c| {
            (trims && og_i18n::spacing::fullwidth(c, lang).is_some())
                || (spaces && og_i18n::spacing::inter_script(c) == Some(InterScript::Ideograph))
                || (french && matches!(c, ':' | ';' | '!' | '?' | '«' | '»'))
        })
    }

    /// Spacing that sets nothing: `no-autospace` and `space-all`.
    const NONE: Self = Self {
        autospace: Autospace {
            alpha: false,
            numeric: false,
            punctuation: false,
            replace: false,
        },
        trim: SpacingTrim {
            start: TrimStart::Space,
            adjacent: false,
            end: TrimEnd::Space,
            all: false,
        },
    };
}

/// Whether a paragraph's spacing may set any of its text.
pub(super) fn applies(content: &RichContent) -> bool {
    content.spans.iter().any(|s| {
        s.atom.is_none()
            && content
                .spacing
                .applies(&s.text, s.step.features.language().as_deref())
    })
}

fn is_french(lang: Option<&str>) -> bool {
    lang.is_some_and(|l| l == "fr" || l.starts_with("fr-") || l.starts_with("fr_"))
}

/// The room unit `u` loses as the last on a line, of what `used` set it
/// mid-line: a closing bracket's blank half (`allow-end`, `trim-end`), and
/// its autospace. `advance`: what a byte range takes as shaped.
pub(super) fn lost_at_end(
    p: &Paragraph,
    u: &Unit,
    used: &Plan,
    advance: &dyn Fn(Range<usize>) -> f32,
) -> f32 {
    let half = if trims_at_end(p, u) && !used.halt.contains(&u.range) {
        advance(u.range.clone()) / 2.0
    } else {
        0.0
    };
    half + used
        .space
        .iter()
        .find(|(r, _)| *r == u.range)
        .map_or(0.0, |s| s.1)
}

/// Whether `u` is closing punctuation a line's end trims (`allow-end`,
/// `trim-end`).
pub(super) fn trims_at_end(p: &Paragraph, u: &Unit) -> bool {
    let trim = p.content.spacing.trim;
    trim.end != TrimEnd::Space
        && !trim.all
        && og_i18n::spacing::fullwidth(u.c, p.lang(u).as_deref()) == Some(Fullwidth::Closing)
}

/// The paragraph's spacing on its lines, into `out`; `pulled`: the units
/// pulled back to a line's end.
pub(super) fn plan(p: &Paragraph, metrics: &Metrics, pulled: &[usize], out: &mut Plan) {
    let spacing = p.content.spacing;
    let units = &p.units;
    let size = |u: &Unit| p.content.spans[u.span].step.size;
    let joined = |a: &Unit, b: &Unit| p.joined(a, b);
    let lang = |u: &Unit| p.lang(u);
    let class = |u: &Unit| og_i18n::spacing::fullwidth(u.c, lang(u).as_deref());
    let trim = spacing.trim;
    for (k, u) in units.iter().enumerate() {
        let Some(own) = class(u) else {
            continue;
        };
        let prev = k.checked_sub(1).map(|p| &units[p]).filter(|p| joined(p, u));
        let next = units.get(k + 1).filter(|n| joined(u, n));
        let line = p.line_of(u.range.start);
        let (first, last) = (p.starts_line(u), p.ends_line(u));
        let halve = match own {
            Fullwidth::Space => false,
            _ if trim.all => true,
            Fullwidth::Opening if first => match trim.start {
                TrimStart::Trim => true,
                TrimStart::Space => false,
                TrimStart::SpaceFirst => !line.is_some_and(|l| l.first),
            },
            // Pulled back to a line's end, trimmed there.
            Fullwidth::Closing if trim.end != TrimEnd::Space && pulled.contains(&u.range.start) => {
                true
            }
            Fullwidth::Closing if last => trim.end == TrimEnd::Trim,
            // §8.5.2: an opening one after an opening, a middle dot, an
            // ideographic space, a closing one as large or any Ps.
            Fullwidth::Opening => {
                trim.adjacent
                    && prev.is_some_and(|p| match class(p) {
                        Some(Fullwidth::Opening | Fullwidth::MiddleDot | Fullwidth::Space) => true,
                        Some(Fullwidth::Closing) => size(p) >= size(u),
                        None => og_i18n::spacing::bracket(p.c) == Some(Fullwidth::Opening),
                    })
            }
            // A closing one before a closing, a middle dot, an
            // ideographic space, a larger opening one or any Pe.
            Fullwidth::Closing => {
                trim.adjacent
                    && next.is_some_and(|n| match class(n) {
                        Some(Fullwidth::Closing | Fullwidth::MiddleDot | Fullwidth::Space) => true,
                        Some(Fullwidth::Opening) => size(n) > size(u),
                        None => og_i18n::spacing::bracket(n.c) == Some(Fullwidth::Closing),
                    })
            }
            Fullwidth::MiddleDot => false,
        };
        if halve {
            out.halt.push(u.range.clone());
        }
    }
    // §8.4.1: an eighth of an ideograph's advance between an ideograph
    // and a non-ideographic letter or numeral on one line, inside the
    // unit before (none after a unit pulled back to a line's end).
    let auto = spacing.autospace;
    let spaced = |a: InterScript, b: InterScript| {
        use InterScript::*;
        match (a, b) {
            (Ideograph, Letter) | (Letter, Ideograph) => auto.alpha,
            (Ideograph, Numeral) | (Numeral, Ideograph) => auto.numeric,
            _ => false,
        }
    };
    let class = |u: &Unit| og_i18n::spacing::inter_script(u.c);
    for (k, pair) in units.windows(2).enumerate() {
        let [a, b] = pair else { continue };
        if pulled.contains(&a.range.start) {
            continue;
        }
        if let (Some(x), Some(y)) = (class(a), class(b))
            && joined(a, b)
            && spaced(x, y)
        {
            out.space.push((a.range.clone(), metrics.ic(a.span) / 8.0));
        }
        // `replace`: a U+0020 between them is the space, at its width.
        if auto.replace
            && b.c == ' '
            && let Some(c) = units.get(k + 2)
            && let (Some(x), Some(y)) = (class(a), class(c))
            && joined(a, b)
            && joined(b, c)
            && spaced(x, y)
        {
            let width = metrics.ic(b.span) / 8.0 - metrics.advance(b.span, ' ');
            out.space.push((b.range.clone(), width));
        }
    }
    // `punctuation`: French's no-break spaces, where no space is.
    if auto.punctuation {
        for (k, u) in units.iter().enumerate() {
            if !is_french(lang(u).as_deref()) {
                continue;
            }
            let prev = k
                .checked_sub(1)
                .map(|p| &units[p])
                .filter(|p| joined(p, u) && !og_i18n::spacing::separator(p.c));
            let next = units
                .get(k + 1)
                .filter(|n| joined(u, n) && !og_i18n::spacing::separator(n.c));
            let (room, at) = match u.c {
                ':' => (metrics.advance(u.span, '\u{a0}'), prev),
                ';' | '!' | '?' | '»' => (metrics.narrow(u.span), prev),
                '«' => (metrics.narrow(u.span), next.map(|_| u)),
                _ => continue,
            };
            if let Some(at) = at {
                out.space.push((at.range.clone(), room));
            }
        }
    }
    out.space.sort_by_key(|(r, _)| r.start);
    // One unit spaced twice takes both.
    out.space.dedup_by(|b, a| {
        let same = a.0 == b.0;
        if same {
            a.1 += b.1;
        }
        same
    });
}
