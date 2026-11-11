//! Scroll-driven animations (CSS Scroll-driven Animations 1, docs/ui.md
//! §3.2): `animation-timeline` with `scroll()`, `view()` and the named
//! timelines `scroll-timeline` and `view-timeline` declare, seen through
//! `timeline-scope`, and `animation-range`. A progress-based animation's
//! time is its timeline's progress, read from the laid-out geometry in the
//! motion step: its scroller's offset and the subject's layout box
//! (transforms left out, so an animation never moves its own timeline).
//! Progress is derived, never state: a restore lands on the same value.

use super::css::CssAnimation;
use crate::layout::NONE;

/// What drives an animation's time (`animation-timeline`).
#[derive(Clone, Debug, Default, PartialEq)]
pub enum AnimationTimeline {
    /// `auto`: presentation ticks.
    #[default]
    Document,
    /// `none`: it has no effect.
    None,
    /// `scroll(<scroller> <axis>)`.
    Scroll { source: ScrollSource, axis: Axis },
    /// `view(<axis> <inset>)`: the element's own visibility in its nearest
    /// scroll container.
    View { axis: Axis, inset: [Inset; 2] },
    /// A `scroll-timeline-name` or `view-timeline-name` in scope.
    Named(String),
}

impl AnimationTimeline {
    /// Whether presentation ticks drive it (`auto`).
    pub fn ticks(&self) -> bool {
        matches!(self, AnimationTimeline::Document)
    }

    /// One `animation-timeline` item.
    pub fn parse(word: &str) -> Option<AnimationTimeline> {
        if word == "auto" {
            return Some(AnimationTimeline::Document);
        }
        if word == "none" {
            return Some(AnimationTimeline::None);
        }
        if dashed(word) {
            return Some(AnimationTimeline::Named(word.to_string()));
        }
        let args = |f: &str| {
            word.strip_prefix(f)
                .and_then(|r| r.strip_prefix('('))
                .and_then(|r| r.strip_suffix(')'))
                .map(|r| r.split_whitespace().collect::<Vec<_>>())
        };
        if let Some(words) = args("scroll") {
            let (mut source, mut axis) = (None, None);
            for w in words {
                let s = match w {
                    "nearest" => ScrollSource::Nearest,
                    "root" => ScrollSource::Root,
                    "self" => ScrollSource::Own,
                    _ => {
                        if axis.replace(Axis::from_css(w)?).is_some() {
                            return None;
                        }
                        continue;
                    }
                };
                if source.replace(s).is_some() {
                    return None;
                }
            }
            return Some(AnimationTimeline::Scroll {
                source: source.unwrap_or_default(),
                axis: axis.unwrap_or_default(),
            });
        }
        let words = args("view")?;
        let mut axis = None;
        let mut inset = Vec::new();
        for w in words {
            match Axis::from_css(w) {
                Some(a) if axis.is_none() => axis = Some(a),
                Some(_) => return None,
                None => inset.push(w),
            }
        }
        Some(AnimationTimeline::View {
            axis: axis.unwrap_or_default(),
            inset: if inset.is_empty() {
                [Inset::default(); 2]
            } else {
                Inset::pair(&inset.join(" "))?
            },
        })
    }
}

/// Whether a cooked `animation-timeline` item is valid (a `view()` inset
/// may be in any length unit).
pub(crate) fn timeline_valid(word: &str) -> bool {
    if AnimationTimeline::parse(word).is_some() {
        return true;
    }
    let Some(words) = word.strip_prefix("view(").and_then(|r| r.strip_suffix(')')) else {
        return false;
    };
    let (axes, insets): (Vec<&str>, Vec<&str>) = words
        .split_whitespace()
        .partition(|w| Axis::from_css(w).is_some());
    axes.len() <= 1 && (insets.is_empty() || Inset::pair_valid(&insets.join(" ")))
}

/// A `<dashed-ident>`: a timeline name.
pub(crate) fn dashed(word: &str) -> bool {
    word.len() > 2
        && word.starts_with("--")
        && word[2..]
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

/// `scroll()`'s scroller.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollSource {
    /// The nearest ancestor scroll container.
    #[default]
    Nearest,
    /// The outermost ancestor scroll container (a screen's own scroller).
    Root,
    /// `self`.
    Own,
}

/// A timeline's axis; block is y and inline x (horizontal writing).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Axis {
    #[default]
    Block,
    Inline,
    X,
    Y,
}

impl Axis {
    pub fn from_css(word: &str) -> Option<Axis> {
        Some(match word {
            "block" => Axis::Block,
            "inline" => Axis::Inline,
            "x" => Axis::X,
            "y" => Axis::Y,
            _ => return None,
        })
    }

    /// Whether it runs horizontally.
    pub fn horizontal(self) -> bool {
        matches!(self, Axis::Inline | Axis::X)
    }
}

/// A length-percentage: `px` plus `frac` of a reference length.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Inset {
    pub px: f32,
    pub frac: f32,
}

impl Inset {
    /// `0`, `<n>px` or `<n>%` (other lengths are made px when the element
    /// resolves).
    pub fn parse(word: &str) -> Option<Inset> {
        let num = |s: &str| s.parse::<f32>().ok().filter(|v| v.is_finite());
        if let Some(p) = word.strip_suffix('%') {
            return Some(Inset {
                px: 0.0,
                frac: num(p)? / 100.0,
            });
        }
        if let Some(p) = word.strip_suffix("px") {
            return Some(Inset {
                px: num(p)?,
                frac: 0.0,
            });
        }
        (num(word)? == 0.0).then_some(Inset::default())
    }

    /// Whether a cooked value is a length-percentage, in any unit.
    pub(crate) fn valid(word: &str) -> bool {
        let unit =
            word.trim_start_matches(|c: char| c.is_ascii_digit() || matches!(c, '.' | '-' | '+'));
        let number = &word[..word.len() - unit.len()];
        Inset::parse(word).is_some()
            || (number.parse::<f32>().is_ok()
                && matches!(
                    unit,
                    "em" | "rem"
                        | "ex"
                        | "ch"
                        | "cap"
                        | "ic"
                        | "lh"
                        | "rlh"
                        | "vw"
                        | "vh"
                        | "vi"
                        | "vb"
                        | "vmin"
                        | "vmax"
                        | "svh"
                        | "lvh"
                        | "dvh"
                        | "svw"
                        | "lvw"
                        | "dvw"
                        | "cqw"
                        | "cqh"
                        | "cqi"
                        | "cqb"
                        | "cqmin"
                        | "cqmax"
                        | "pt"
                        | "pc"
                        | "cm"
                        | "mm"
                        | "in"
                        | "q"
                ))
    }

    /// `view-timeline-inset`: one or two of `auto` (0) and
    /// length-percentages, start then end.
    pub fn pair(text: &str) -> Option<[Inset; 2]> {
        let one = |w: &str| {
            if w == "auto" {
                Some(Inset::default())
            } else {
                Inset::parse(w)
            }
        };
        let words: Vec<&str> = text.split_whitespace().collect();
        match words[..] {
            [a] => Some([one(a)?; 2]),
            [a, b] => Some([one(a)?, one(b)?]),
            _ => None,
        }
    }

    /// Whether a cooked `view-timeline-inset` item is valid.
    pub(crate) fn pair_valid(text: &str) -> bool {
        let words: Vec<&str> = text.split_whitespace().collect();
        (1..=2).contains(&words.len()) && words.iter().all(|w| *w == "auto" || Inset::valid(w))
    }

    fn of(self, length: f32) -> f32 {
        self.px + self.frac * length
    }
}

/// A named view progress range (CSS Scroll-driven Animations 1 §3.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RangeName {
    Cover,
    Contain,
    Entry,
    Exit,
    EntryCrossing,
    ExitCrossing,
}

impl RangeName {
    pub fn from_css(word: &str) -> Option<RangeName> {
        Some(match word {
            "cover" => RangeName::Cover,
            "contain" => RangeName::Contain,
            "entry" => RangeName::Entry,
            "exit" => RangeName::Exit,
            "entry-crossing" => RangeName::EntryCrossing,
            "exit-crossing" => RangeName::ExitCrossing,
            _ => return None,
        })
    }
}

/// One end of `animation-range`: the point `at` of the way through a named
/// range (the whole timeline when `name` is `None`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RangeEdge {
    pub name: Option<RangeName>,
    pub at: Inset,
}

impl RangeEdge {
    /// An `animation-range-start` (`end` false) or `-end` value: `normal`
    /// (`Some(None)`), a length-percentage, or a range name with an
    /// optional one (0% for a start, 100% for an end). `None`: invalid.
    pub fn parse(text: &str, end: bool) -> Option<Option<RangeEdge>> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let fallback = Inset {
            px: 0.0,
            frac: if end { 1.0 } else { 0.0 },
        };
        Some(Some(match words[..] {
            ["normal"] => return Some(None),
            [w] => match RangeName::from_css(w) {
                Some(name) => RangeEdge {
                    name: Some(name),
                    at: fallback,
                },
                None => RangeEdge {
                    name: None,
                    at: Inset::parse(w)?,
                },
            },
            [n, w] => RangeEdge {
                name: Some(RangeName::from_css(n)?),
                at: Inset::parse(w)?,
            },
            _ => return None,
        }))
    }

    /// Whether a cooked range edge is valid.
    pub(crate) fn valid(text: &str) -> bool {
        let words: Vec<&str> = text.split_whitespace().collect();
        match words[..] {
            ["normal"] => true,
            [w] => RangeName::from_css(w).is_some() || Inset::valid(w),
            [n, w] => RangeName::from_css(n).is_some() && Inset::valid(w),
            _ => false,
        }
    }
}

/// A timeline an element declares (`scroll-timeline`, `view-timeline`).
#[derive(Clone, Debug, PartialEq)]
pub struct NamedTimeline {
    pub name: String,
    /// A view timeline (the element is the subject), else a scroll one
    /// (the element is the scroller).
    pub view: bool,
    pub axis: Axis,
    pub inset: [Inset; 2],
}

/// A scroll container and a subject in it, on one axis: the scroll offset
/// and its range, the scrollport's length, and the subject's start (in the
/// content, at offset 0) and length.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ScrollGeometry {
    pub offset: f32,
    pub max: f32,
    pub port: f32,
    pub subject: Option<(f32, f32)>,
}

impl ScrollGeometry {
    /// The offsets a named range of a view timeline spans (§3.2).
    fn range(&self, name: RangeName, inset: [Inset; 2]) -> Option<(f32, f32)> {
        let (top, h) = self.subject?;
        let start_inset = inset[0].of(self.port);
        let port = (self.port - start_inset - inset[1].of(self.port)).max(0.0);
        // Cover: from the subject's start edge reaching the scrollport's
        // end edge to its end edge passing the start edge.
        let s0 = top - start_inset - port;
        let e0 = top + h - start_inset;
        let a = h.min(port);
        Some(match name {
            RangeName::Cover => (s0, e0),
            RangeName::Contain => (s0 + a, e0 - a),
            RangeName::Entry => (s0, s0 + a),
            RangeName::Exit => (e0 - a, e0),
            RangeName::EntryCrossing => (s0, s0 + h),
            RangeName::ExitCrossing => (e0 - h, e0),
        })
    }

    /// Progress through the animation's range `[start, end]` of the
    /// timeline (`None` while it is inactive: a scroller that cannot
    /// scroll, an empty range).
    pub(crate) fn progress(
        &self,
        view: Option<[Inset; 2]>,
        start: Option<RangeEdge>,
        end: Option<RangeEdge>,
    ) -> Option<f32> {
        let whole = match view {
            Some(inset) => self.range(RangeName::Cover, inset)?,
            None if self.max > 0.0 => (0.0, self.max),
            None => return None,
        };
        let at = |edge: Option<RangeEdge>, normal: f32| -> Option<f32> {
            let Some(e) = edge else {
                return Some(whole.0 + normal * (whole.1 - whole.0));
            };
            let (a, b) = match (e.name, view) {
                (Some(name), Some(inset)) => self.range(name, inset)?,
                _ => whole,
            };
            Some(a + e.at.of(b - a))
        };
        let (a, b) = (at(start, 0.0)?, at(end, 1.0)?);
        if b - a <= 0.0 {
            return None;
        }
        Some((self.offset - a) / (b - a))
    }
}

impl CssAnimation {
    /// As driven by a progress-based timeline at `progress`: its active
    /// interval spans the range, time-based durations and a positive delay
    /// as proportions of it and `auto` (0s) as all of it (Web Animations 2
    /// §4.4.3), and its active time there. The end of the range is inside
    /// it.
    pub(crate) fn on_progress(&self, progress: f32) -> (CssAnimation, f32) {
        let n = if self.iterations.is_finite() && self.iterations > 0.0 {
            self.iterations
        } else {
            1.0
        };
        let (duration_ms, delay) = if self.duration_ms > 0.0 {
            let k = 1000.0 / (self.delay_ms.max(0.0) + self.duration_ms * n);
            (self.duration_ms * k, self.delay_ms.max(0.0) * k)
        } else {
            (1000.0 / n, 0.0)
        };
        let a = CssAnimation {
            duration_ms,
            delay_ms: 0.0,
            iterations: n,
            paused: false,
            ..self.clone()
        };
        let active = if progress == 1.0 {
            duration_ms * n - 1e-3
        } else {
            progress * 1000.0 - delay
        };
        (a, active)
    }
}

impl crate::ui::Style {
    /// Animation `a` of node `i`'s progress through its range (`None`:
    /// its timeline is inactive or `none`).
    pub(crate) fn timeline_progress(
        &self,
        i: usize,
        a: &CssAnimation,
        interaction: &crate::ui::Interaction,
        tree: &crate::ui::Tree,
    ) -> Option<f32> {
        let none = [Inset::default(); 2];
        let (scroller, subject, axis, inset) = match &a.timeline {
            AnimationTimeline::Document | AnimationTimeline::None => return None,
            AnimationTimeline::Scroll { source, axis } => {
                let s = match source {
                    ScrollSource::Own => Some(i).filter(|&i| tree.nodes[i].scrolls()),
                    ScrollSource::Nearest => tree.scroll_ancestor(i),
                    ScrollSource::Root => {
                        let mut out = None;
                        let mut at = i;
                        while let Some(s) = tree.scroll_ancestor(at) {
                            out = Some(s);
                            at = s;
                        }
                        out
                    }
                };
                (s?, None, *axis, none)
            }
            AnimationTimeline::View { axis, inset } => {
                (tree.scroll_ancestor(i)?, Some(i), *axis, *inset)
            }
            AnimationTimeline::Named(name) => {
                let (d, t) = tree.named_timeline(i, name)?;
                if t.view {
                    (tree.scroll_ancestor(d)?, Some(d), t.axis, t.inset)
                } else {
                    (
                        Some(d).filter(|&d| tree.nodes[d].scrolls())?,
                        None,
                        t.axis,
                        none,
                    )
                }
            }
        };
        let s = &tree.nodes[scroller];
        let pad = s.el.padding;
        let (offset, max, start, port) = if axis.horizontal() {
            let max = s.max_scroll_x();
            let at = interaction
                .state
                .scroll_x
                .get(&s.key)
                .copied()
                .unwrap_or(0.0);
            (
                at.clamp(0.0, max),
                max,
                s.placed.rect.x + pad.left,
                s.placed.rect.width - pad.horizontal(),
            )
        } else {
            let max = s.max_scroll();
            let at = interaction.state.scroll.get(&s.key).copied().unwrap_or(0.0);
            (
                at.clamp(0.0, max),
                max,
                s.placed.rect.y + pad.top,
                s.placed.rect.height - pad.vertical(),
            )
        };
        // Laid out at this offset: its place in the content is where it is
        // plus the offset.
        let subject = subject.map(|j| {
            let r = tree.nodes[j].placed.rect;
            if axis.horizontal() {
                (r.x - start + offset, r.width)
            } else {
                (r.y - start + offset, r.height)
            }
        });
        let g = ScrollGeometry {
            offset,
            max,
            port: port.max(0.0),
            subject,
        };
        g.progress(subject.map(|_| inset), a.range_start, a.range_end)
    }
}

impl crate::ui::Tree {
    fn scroll_ancestor(&self, i: usize) -> Option<usize> {
        let mut p = self.nodes[i].parent;
        while p != NONE {
            if self.nodes[p].scrolls() {
                return Some(p);
            }
            p = self.nodes[p].parent;
        }
        None
    }

    /// The timeline `name` node `i` sees: declared by it or an ancestor,
    /// or, past an ancestor whose `timeline-scope` names it, the one
    /// declaration in that ancestor's subtree (none, or several: inactive).
    fn named_timeline(&self, i: usize, name: &str) -> Option<(usize, &NamedTimeline)> {
        let declares = |j: usize| {
            self.nodes[j]
                .props
                .timelines
                .iter()
                .rev()
                .find(|t| t.name == name)
        };
        let mut at = i;
        loop {
            if let Some(t) = declares(at) {
                return Some((at, t));
            }
            let n = &self.nodes[at];
            if n.props
                .timeline_scope
                .iter()
                .any(|s| s == name || s == "all")
            {
                let mut found = None;
                let mut stack: Vec<usize> = n.children.iter().rev().copied().collect();
                while let Some(j) = stack.pop() {
                    if let Some(t) = declares(j) {
                        if found.is_some() {
                            return None;
                        }
                        found = Some((j, t));
                    }
                    stack.extend(self.nodes[j].children.iter().rev().copied());
                }
                return found;
            }
            if n.parent == NONE {
                return None;
            }
            at = n.parent;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_ranges_follow_the_spec() {
        // A 100 tall subject at 1000 in the content, a 400 scrollport.
        let g = |offset| ScrollGeometry {
            offset,
            max: 2000.0,
            port: 400.0,
            subject: Some((1000.0, 100.0)),
        };
        let none = [Inset::default(); 2];
        let edge = |n| {
            Some(RangeEdge {
                name: Some(n),
                at: Inset::default(),
            })
        };
        let end = |n| {
            Some(RangeEdge {
                name: Some(n),
                at: Inset { px: 0.0, frac: 1.0 },
            })
        };
        // Cover spans 600 (its top at the port's bottom) to 1100.
        assert_eq!(g(600.0).progress(Some(none), None, None), Some(0.0));
        assert_eq!(g(850.0).progress(Some(none), None, None), Some(0.5));
        // Entry: 600 to 700 (wholly inside); exit: 1000 to 1100.
        let c = RangeName::Entry;
        assert_eq!(g(650.0).progress(Some(none), edge(c), end(c)), Some(0.5));
        let c = RangeName::Exit;
        assert_eq!(g(1075.0).progress(Some(none), edge(c), end(c)), Some(0.75));
        // A 100px start inset shortens the port from its start.
        let inset = [
            Inset {
                px: 100.0,
                frac: 0.0,
            },
            Inset::default(),
        ];
        assert_eq!(g(600.0).progress(Some(inset), None, None), Some(0.0));
        assert_eq!(g(1000.0).progress(Some(inset), None, None), Some(1.0));
        // A scroll timeline spans the scroll range; one that cannot scroll
        // is inactive.
        let s = ScrollGeometry {
            offset: 500.0,
            max: 2000.0,
            port: 400.0,
            subject: None,
        };
        assert_eq!(s.progress(None, None, None), Some(0.25));
        assert_eq!(
            ScrollGeometry { max: 0.0, ..s }.progress(None, None, None),
            None
        );
    }

    #[test]
    fn timelines_and_ranges_parse() {
        assert_eq!(
            AnimationTimeline::parse("scroll(x root)"),
            Some(AnimationTimeline::Scroll {
                source: ScrollSource::Root,
                axis: Axis::X
            })
        );
        assert_eq!(
            AnimationTimeline::parse("view(inline 10% 20px)"),
            Some(AnimationTimeline::View {
                axis: Axis::Inline,
                inset: [
                    Inset { px: 0.0, frac: 0.1 },
                    Inset {
                        px: 20.0,
                        frac: 0.0
                    }
                ]
            })
        );
        assert_eq!(AnimationTimeline::parse("scroll(x y)"), None);
        assert_eq!(AnimationTimeline::parse("-x"), None);
        assert_eq!(
            RangeEdge::parse("exit", true),
            Some(Some(RangeEdge {
                name: Some(RangeName::Exit),
                at: Inset { px: 0.0, frac: 1.0 }
            }))
        );
    }
}
