//! Ruby (CSS Ruby 1): bases and their annotations as inline boxes of a rich
//! text. [`structure`] turns a text's inline elements, each with its ruby
//! role (HTML's `<ruby>`, `<rb>`, `<rt>`, `<rtc>` and `<rp>`, or a
//! `display` from CSS), into ruby segments as §2.2 generates their boxes
//! and §2.3 pairs them: each segment one unbreakable box of its line, its
//! columns' bases (a base may hold nested ruby) under one or more levels
//! of annotations, each level paired column by column or one annotation
//! spanning them all.
//!
//! An annotation is set at half its base's size (HTML's `rt { font-size:
//! 50% }`, with the font's `ruby` forms). Interlinear levels stack outward
//! over or under the base container (`ruby-position`, `alternate` taking
//! turns, §4.1), beyond any nested ruby's on that side (§3.1.2); in a
//! vertical line over is its line-over side (the right, the left in
//! `sideways-lr`). An inter-character level (horizontal lines only) stands
//! each annotation upright in a column of its own right of its base
//! (§3.2). A column is as long as the longest of its base and its
//! annotations, a spanning annotation widening its columns evenly
//! (§3.1.1); `ruby-align` spreads each in its column (§4.3). An annotation
//! longer than its base hangs over the text beside the segment by up to
//! one of its characters on each side under `ruby-overhang: auto` (§5.1),
//! never over another ruby. Annotations take no room in their line while
//! they fit its leading; where the line is too tight, it grows on their
//! side by what they lack (§3.6).

use super::*;
use crate::layout::flow::{VerticalText, WritingMode};

/// CSS `ruby-align` (CSS Ruby 1 §4.3).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RubyAlign {
    Start,
    Center,
    SpaceBetween,
    #[default]
    SpaceAround,
}

/// CSS `ruby-overhang` (CSS Ruby 1 §5.1).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RubyOverhang {
    /// Over any text beside it but another ruby.
    #[default]
    Auto,
    /// Over nothing.
    None,
    /// Over a space beside it only.
    Spaces,
}

/// CSS `ruby-position` (CSS Ruby 1 §4.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RubyPosition {
    /// `alternate` (the initial value): over, or under with `under`, as
    /// the first level; then the other side of the level before.
    Alternate {
        under: bool,
    },
    Over,
    Under,
    /// Upright right of each base across a line; over down one.
    InterCharacter,
}

impl Default for RubyPosition {
    fn default() -> Self {
        RubyPosition::Alternate { under: false }
    }
}

impl RubyPosition {
    /// A computed `ruby-position` (`None` for a value that is not one).
    pub(crate) fn parse(v: &str) -> Option<RubyPosition> {
        let w: Vec<&str> = v.split_whitespace().collect();
        Some(match w.as_slice() {
            ["inter-character"] => RubyPosition::InterCharacter,
            ["over"] => RubyPosition::Over,
            ["under"] => RubyPosition::Under,
            ["alternate"] | ["alternate", "over"] | ["over", "alternate"] => {
                RubyPosition::Alternate { under: false }
            }
            ["alternate", "under"] | ["under", "alternate"] => {
                RubyPosition::Alternate { under: true }
            }
            _ => return None,
        })
    }
}

/// A text's ruby properties (inherited): where its annotation levels go,
/// how base and annotation share their column, and how far annotations
/// hang.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RubyStyle {
    pub position: RubyPosition,
    pub align: RubyAlign,
    pub overhang: RubyOverhang,
}

impl RubyStyle {
    /// The style computed `ruby-position`, `ruby-align` and
    /// `ruby-overhang` give (each its initial value when `None`).
    pub(crate) fn parse(
        position: Option<&str>,
        align: Option<&str>,
        overhang: Option<&str>,
    ) -> RubyStyle {
        RubyStyle {
            position: position.and_then(RubyPosition::parse).unwrap_or_default(),
            align: match align {
                Some("start") => RubyAlign::Start,
                Some("center") => RubyAlign::Center,
                Some("space-between") => RubyAlign::SpaceBetween,
                _ => RubyAlign::SpaceAround,
            },
            overhang: match overhang {
                Some("none") => RubyOverhang::None,
                Some("spaces") => RubyOverhang::Spaces,
                _ => RubyOverhang::Auto,
            },
        }
    }
}

/// Where an annotation level sits against its base container.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RubySide {
    Over,
    Under,
    /// Inter-character: upright right of each base (§3.2).
    Inter,
}

/// One annotation level of a segment (§2.3.1).
#[derive(Clone, Debug, PartialEq)]
pub struct RubyLevel {
    /// Its own `ruby-position` (an annotation container's `::part`);
    /// `None`: its text's.
    pub position: Option<RubyPosition>,
    /// Where it sits, from its position and the levels before it
    /// ([`Ruby::resolve`]).
    pub side: RubySide,
    /// One annotation for each column, or (`spanning`) one across them all.
    pub texts: Vec<String>,
    pub spanning: bool,
}

/// What a column's base holds: words, and rubies nested in it.
#[derive(Clone, Debug, PartialEq)]
pub enum RubyBase {
    Text(String),
    Ruby(Ruby),
}

/// A ruby segment (CSS Ruby 1 §2.3.1): one inline box of its line; its
/// span's text is its base text.
#[derive(Clone, Debug, PartialEq)]
pub struct Ruby {
    /// Each column's base.
    pub bases: Vec<Vec<RubyBase>>,
    pub levels: Vec<RubyLevel>,
    pub align: RubyAlign,
    /// Whether it may hang over what comes before and after it.
    pub overhang: [bool; 2],
}

impl Ruby {
    /// Its base text: its columns' words and its nested rubies' bases.
    pub fn base_text(&self) -> String {
        self.bases.iter().flatten().map(base_text).collect()
    }

    /// Sets its levels' sides and its alignment, and its nested rubies',
    /// under its text's `style` (§4.1: `alternate` takes the other side of
    /// the interlinear level before it, the first level its own).
    pub fn resolve(&mut self, style: RubyStyle) {
        self.align = style.align;
        let mut before: Option<bool> = None;
        for l in &mut self.levels {
            l.side = match l.position.unwrap_or(style.position) {
                RubyPosition::InterCharacter => RubySide::Inter,
                RubyPosition::Over => RubySide::Over,
                RubyPosition::Under => RubySide::Under,
                RubyPosition::Alternate { under } => {
                    if before.map_or(under, |b| !b) {
                        RubySide::Under
                    } else {
                        RubySide::Over
                    }
                }
            };
            if l.side != RubySide::Inter {
                before = Some(l.side == RubySide::Under);
            }
        }
        for b in self.bases.iter_mut().flatten() {
            if let RubyBase::Ruby(inner) = b {
                inner.resolve(style);
            }
        }
    }
}

fn base_text(b: &RubyBase) -> String {
    match b {
        RubyBase::Text(t) => t.clone(),
        RubyBase::Ruby(r) => r.base_text(),
    }
}

/// An element's ruby role: its `display` (CSS Ruby 1 §2.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RubyRole {
    /// Not ruby: its content flows on.
    Inline,
    /// `display: none` (HTML's `<rp>`): not shown.
    None,
    Container,
    Base,
    Annotation,
    BaseContainer,
    AnnotationContainer,
}

impl RubyRole {
    /// HTML's role for `tag` (CSS Ruby 1 Appendix A.1).
    pub fn of_tag(tag: &str) -> RubyRole {
        match tag {
            "ruby" => RubyRole::Container,
            "rb" => RubyRole::Base,
            "rt" => RubyRole::Annotation,
            "rbc" => RubyRole::BaseContainer,
            "rtc" => RubyRole::AnnotationContainer,
            "rp" => RubyRole::None,
            _ => RubyRole::Inline,
        }
    }

    /// The role a computed `display` gives (`None`: no ruby role, the
    /// element's own).
    pub fn of_display(v: &str) -> Option<RubyRole> {
        Some(match v.trim() {
            "ruby" | "inline ruby" | "ruby inline" => RubyRole::Container,
            "ruby-base" => RubyRole::Base,
            "ruby-text" => RubyRole::Annotation,
            "ruby-base-container" => RubyRole::BaseContainer,
            "ruby-text-container" => RubyRole::AnnotationContainer,
            "none" => RubyRole::None,
            "inline" => RubyRole::Inline,
            _ => return None,
        })
    }

    fn internal(self) -> bool {
        matches!(
            self,
            RubyRole::Base
                | RubyRole::Annotation
                | RubyRole::BaseContainer
                | RubyRole::AnnotationContainer
        )
    }
}

/// A run of a text for [`structure`]: the elements it sits in, outermost
/// first (each one's id and role), and its words.
pub struct RubyLeaf<'a> {
    pub path: &'a [(u32, RubyRole)],
    pub text: String,
}

/// What [`structure`] makes of a text's runs, in order: words of run
/// `run`, or a ruby segment styled as run `run`.
#[derive(Clone, Debug, PartialEq)]
pub enum RubyOut {
    Text { run: usize, text: String },
    Ruby { run: usize, ruby: Ruby },
}

/// A text's element tree, rebuilt from its runs' paths.
enum Tree {
    Leaf(usize),
    El {
        id: u32,
        role: RubyRole,
        kids: Vec<Tree>,
    },
}

/// A text's runs as ruby boxes (CSS Ruby 1 §2.2 and §2.3): each segment one
/// [`Ruby`], other words as they are. `position` is the `ruby-position`
/// an annotation container declares, given the ids of the elements it
/// sits in, innermost last (its own last, when it is an element); its
/// levels' sides are set by [`Ruby::resolve`].
pub fn structure(
    leaves: &[RubyLeaf],
    position: &dyn Fn(&[u32]) -> Option<RubyPosition>,
) -> Vec<RubyOut> {
    let mut root: Vec<Tree> = Vec::new();
    for (k, leaf) in leaves.iter().enumerate() {
        let mut level = &mut root;
        for &(id, role) in leaf.path {
            let open = matches!(level.last(), Some(Tree::El { id: last, .. }) if *last == id);
            if !open {
                level.push(Tree::El {
                    id,
                    role,
                    kids: Vec::new(),
                });
            }
            let Some(Tree::El { kids, .. }) = level.last_mut() else {
                unreachable!("an element was just pushed");
            };
            level = kids;
        }
        level.push(Tree::Leaf(k));
    }
    let cx = Cx { leaves, position };
    let mut out = Vec::new();
    cx.inline(&root, &[], &mut out);
    out
}

struct Cx<'a> {
    leaves: &'a [RubyLeaf<'a>],
    position: &'a dyn Fn(&[u32]) -> Option<RubyPosition>,
}

/// An annotation container's element (none for an anonymous one), its
/// annotations' content, and whether its one annotation spans the bases.
type Level<'t> = (Option<u32>, Vec<Vec<&'t Tree>>, bool);

/// A segment's bases' content and its annotation levels (§2.3.1).
type Segment<'t> = (Vec<Vec<&'t Tree>>, Vec<Level<'t>>);

/// A ruby container's child box (§2.2 steps 3 to 8).
enum Part<'t> {
    /// A base: an `<rb>`'s content, or inline content (anonymous: true).
    Base(Vec<&'t Tree>, bool),
    Annotation(Vec<&'t Tree>),
    Bases(Vec<Vec<&'t Tree>>),
    /// An annotation container: its element (none for an anonymous one),
    /// its annotations, and whether its one annotation spans its bases.
    Annotations(Option<u32>, Vec<Vec<&'t Tree>>, bool),
    /// White space between segments (§2.2 step 6).
    Space(usize),
}

impl<'a> Cx<'a> {
    fn white(&self, t: &Tree) -> Option<usize> {
        match t {
            Tree::Leaf(k) if self.leaves[*k].text.trim().is_empty() => Some(*k),
            _ => None,
        }
    }

    /// Inline content: words, inline elements, and ruby containers (an
    /// internal ruby box out of one in an anonymous one, §2.2 step 2).
    fn inline(&self, kids: &[Tree], chain: &[u32], out: &mut Vec<RubyOut>) {
        let mut pending: Vec<&Tree> = Vec::new();
        let flush = |pending: &mut Vec<&Tree>, out: &mut Vec<RubyOut>| {
            // White space after the last box stays outside it.
            let mut trailing = Vec::new();
            while let Some(k) = pending.last().and_then(|t| self.white(t)) {
                trailing.push(k);
                pending.pop();
            }
            if !pending.is_empty() {
                self.container(&std::mem::take(pending), chain, out);
            }
            for k in trailing.into_iter().rev() {
                out.push(self.words(k));
            }
        };
        for t in kids {
            match t {
                Tree::El { role, .. } if role.internal() => pending.push(t),
                Tree::Leaf(k) if !pending.is_empty() && self.white(t) == Some(*k) => {
                    pending.push(t)
                }
                _ => {
                    flush(&mut pending, out);
                    match t {
                        Tree::Leaf(k) => out.push(self.words(*k)),
                        Tree::El {
                            role: RubyRole::None,
                            ..
                        } => {}
                        Tree::El {
                            id,
                            role: RubyRole::Container,
                            kids,
                        } => {
                            let chain = [chain, &[*id]].concat();
                            self.container(&kids.iter().collect::<Vec<_>>(), &chain, out)
                        }
                        Tree::El { id, kids, .. } => {
                            self.inline(kids, &[chain, &[*id]].concat(), out)
                        }
                    }
                }
            }
        }
        flush(&mut pending, out);
    }

    fn words(&self, k: usize) -> RubyOut {
        RubyOut::Text {
            run: k,
            text: self.leaves[k].text.clone(),
        }
    }

    /// A ruby container's content as its segments (§2.2, §2.3.1).
    fn container(&self, kids: &[&'a Tree], chain: &[u32], out: &mut Vec<RubyOut>) {
        // Its boxes: bases, annotations and their containers, inline
        // content wrapped in anonymous bases (step 3).
        let mut parts: Vec<Part> = Vec::new();
        for &t in kids {
            match t {
                Tree::El {
                    role: RubyRole::None,
                    ..
                } => {}
                Tree::El {
                    role: RubyRole::Base,
                    kids,
                    ..
                } => parts.push(Part::Base(kids.iter().collect(), false)),
                Tree::El {
                    role: RubyRole::Annotation,
                    kids,
                    ..
                } => parts.push(Part::Annotation(kids.iter().collect())),
                Tree::El {
                    role: RubyRole::BaseContainer,
                    kids,
                    ..
                } => parts.push(Part::Bases(self.items(kids, RubyRole::Base).0)),
                Tree::El {
                    id,
                    role: RubyRole::AnnotationContainer,
                    kids,
                } => {
                    let (items, spanning) = self.items(kids, RubyRole::Annotation);
                    parts.push(Part::Annotations(Some(*id), items, spanning));
                }
                // Consecutive inline content: one anonymous base.
                _ => match parts.last_mut() {
                    Some(Part::Base(content, true)) => content.push(t),
                    _ => parts.push(Part::Base(vec![t], true)),
                },
            }
        }
        // Anonymous bases of white space only (steps 4 to 6): trimmed at
        // the ends, removed before an annotation and between bases, kept
        // as words between an annotation and a base (inter-segment).
        let space = |p: &Part| match p {
            Part::Base(c, true) if c.iter().all(|t| self.white(t).is_some()) => {
                c.iter().find_map(|t| self.white(t))
            }
            _ => None,
        };
        let annotation =
            |p: Option<&Part>| matches!(p, Some(Part::Annotation(_) | Part::Annotations(..)));
        let base = |p: Option<&Part>| matches!(p, Some(Part::Base(..) | Part::Bases(_)));
        let spaces: Vec<Option<usize>> = parts.iter().map(space).collect();
        let segment_space: Vec<bool> = (0..parts.len())
            .map(|i| {
                i > 0
                    && annotation(parts.get(i - 1))
                    && base(parts.get(i + 1))
                    && spaces[i + 1].is_none()
            })
            .collect();
        let mut kept: Vec<Part> = Vec::new();
        for (i, p) in parts.into_iter().enumerate() {
            match spaces[i] {
                None => kept.push(p),
                Some(k) if segment_space[i] => kept.push(Part::Space(k)),
                Some(_) => {}
            }
        }
        // Step 8: consecutive bases and annotations in anonymous
        // containers; then segments, each a base container and the
        // annotation containers after it (§2.3.1).
        let mut groups: Vec<Part> = Vec::new();
        for p in kept {
            match (groups.last_mut(), p) {
                (Some(Part::Bases(list)), Part::Base(c, _)) => list.push(c),
                (_, Part::Base(c, _)) => groups.push(Part::Bases(vec![c])),
                (Some(Part::Annotations(None, list, _)), Part::Annotation(c)) => list.push(c),
                (_, Part::Annotation(c)) => groups.push(Part::Annotations(None, vec![c], false)),
                (_, p) => groups.push(p),
            }
        }
        let mut segment: Option<Segment> = None;
        for g in groups {
            match g {
                Part::Bases(bases) => {
                    if let Some(s) = segment.take() {
                        self.segment(s, chain, out);
                    }
                    segment = Some((bases, Vec::new()));
                }
                Part::Annotations(id, list, spanning) => segment
                    .get_or_insert_with(|| (Vec::new(), Vec::new()))
                    .1
                    .push((id, list, spanning)),
                Part::Space(k) => {
                    if let Some(s) = segment.take() {
                        self.segment(s, chain, out);
                    }
                    out.push(self.words(k));
                }
                Part::Base(..) | Part::Annotation(_) => {}
            }
        }
        if let Some(s) = segment {
            self.segment(s, chain, out);
        }
    }

    /// A level container's items (§2.2 step 3): its `role` children, and
    /// its inline content wrapped in anonymous ones; and whether its one
    /// item is anonymous (§2.3.2: it spans the segment's bases).
    fn items<'t>(&self, kids: &'t [Tree], role: RubyRole) -> (Vec<Vec<&'t Tree>>, bool) {
        let mut items: Vec<(Vec<&Tree>, bool)> = Vec::new();
        for t in kids {
            match t {
                Tree::El {
                    role: RubyRole::None,
                    ..
                } => {}
                Tree::El { role: r, kids, .. } if *r == role => {
                    items.push((kids.iter().collect(), false))
                }
                _ => match items.last_mut() {
                    Some((c, true)) => c.push(t),
                    _ => items.push((vec![t], true)),
                },
            }
        }
        // White space between items or at the ends is not an item.
        items.retain(|(c, anon)| !*anon || c.iter().any(|t| self.white(t).is_none()));
        let spanning = items.len() == 1 && items[0].1;
        (items.into_iter().map(|(c, _)| c).collect(), spanning)
    }

    /// One segment: its columns paired with each level's annotations
    /// (§2.3.2), empty ones filling the shorter.
    fn segment(&self, (bases, levels): Segment, chain: &[u32], out: &mut Vec<RubyOut>) {
        let first = bases
            .iter()
            .chain(levels.iter().flat_map(|l| &l.1))
            .flatten()
            .find_map(|t| self.first(t));
        let Some(run) = first else { return };
        let mut columns: Vec<Vec<RubyBase>> = bases.iter().map(|c| self.base(c, chain)).collect();
        if levels.is_empty() {
            // No annotation: its bases flow as words.
            for item in columns.into_iter().flatten() {
                out.push(match item {
                    RubyBase::Text(text) => RubyOut::Text { run, text },
                    RubyBase::Ruby(ruby) => RubyOut::Ruby { run, ruby },
                });
            }
            return;
        }
        let count = levels
            .iter()
            .filter(|l| !l.2)
            .map(|l| l.1.len())
            .chain([columns.len(), 1])
            .max()
            .unwrap_or(1);
        columns.resize(count, Vec::new());
        let levels = levels
            .iter()
            .map(|(id, items, spanning)| {
                let ids: Vec<u32> = chain.iter().copied().chain(*id).collect();
                let mut texts: Vec<String> = items.iter().map(|c| self.text(c)).collect();
                if !spanning {
                    texts.resize(count, String::new());
                }
                RubyLevel {
                    position: (self.position)(&ids),
                    side: RubySide::Over,
                    texts,
                    spanning: *spanning,
                }
            })
            .collect();
        out.push(RubyOut::Ruby {
            run,
            ruby: Ruby {
                bases: columns,
                levels,
                align: RubyAlign::default(),
                overhang: [false; 2],
            },
        });
    }

    fn first(&self, t: &Tree) -> Option<usize> {
        match t {
            Tree::Leaf(k) => Some(*k),
            Tree::El {
                role: RubyRole::None,
                ..
            } => None,
            Tree::El { kids, .. } => kids.iter().find_map(|t| self.first(t)),
        }
    }

    /// A base's words and nested rubies, trimmed at both ends.
    fn base(&self, content: &[&Tree], chain: &[u32]) -> Vec<RubyBase> {
        let mut out = Vec::new();
        let mut flat: Vec<RubyOut> = Vec::new();
        for t in content {
            match t {
                Tree::Leaf(k) => flat.push(self.words(*k)),
                Tree::El {
                    role: RubyRole::None,
                    ..
                } => {}
                Tree::El {
                    id,
                    role: RubyRole::Container,
                    kids,
                } => self.container(
                    &kids.iter().collect::<Vec<_>>(),
                    &[chain, &[*id]].concat(),
                    &mut flat,
                ),
                Tree::El { id, kids, .. } => {
                    self.inline(kids, &[chain, &[*id]].concat(), &mut flat)
                }
            }
        }
        for o in flat {
            match (o, out.last_mut()) {
                (RubyOut::Text { text, .. }, Some(RubyBase::Text(t))) => t.push_str(&text),
                (RubyOut::Text { text, .. }, _) => out.push(RubyBase::Text(text)),
                (RubyOut::Ruby { ruby, .. }, _) => out.push(RubyBase::Ruby(ruby)),
            }
        }
        if let Some(RubyBase::Text(t)) = out.first_mut() {
            *t = t.trim_start().to_string();
        }
        if let Some(RubyBase::Text(t)) = out.last_mut() {
            t.truncate(t.trim_end().len());
        }
        out.retain(|b| !matches!(b, RubyBase::Text(t) if t.is_empty()));
        out
    }

    /// An annotation's words, trimmed at both ends.
    fn text(&self, content: &[&Tree]) -> String {
        fn words(cx: &Cx, t: &Tree, out: &mut String) {
            match t {
                Tree::Leaf(k) => out.push_str(&cx.leaves[*k].text),
                Tree::El {
                    role: RubyRole::None,
                    ..
                } => {}
                Tree::El { kids, .. } => kids.iter().for_each(|t| words(cx, t, out)),
            }
        }
        let mut s = String::new();
        for t in content {
            words(self, t, &mut s);
        }
        s.trim().to_string()
    }
}

/// An annotation's size, of its base's.
pub(crate) const RUBY_SIZE: f32 = 0.5;

/// `text`'s clusters and each one's length along its line (`v`: down a
/// vertical one) at `size`.
fn clusters(
    s: &RichSpan,
    text: &str,
    size: f32,
    tracking: f32,
    fonts: &dyn Metrics,
    v: Option<VerticalText>,
) -> Vec<(String, f32)> {
    let m = fonts.metrics_styled(s.font, s.query(), text);
    let chars: Vec<char> = text.chars().collect();
    og_i18n::graphemes(text)
        .into_iter()
        .map(|(a, b)| {
            let c: String = chars[a..b].iter().collect();
            let len = match v {
                Some(v) => crate::text::vertical::measure(
                    m,
                    &c,
                    size,
                    tracking,
                    crate::shape::Features::NONE,
                    v,
                ),
                None => m.measure_tracked(&c, size, tracking),
            };
            (c, len)
        })
        .collect()
}

fn length(clusters: &[(String, f32)]) -> f32 {
    clusters.iter().map(|c| c.1).sum()
}

/// Where a laid-out cluster sits across its line.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Place {
    Base,
    /// An interlinear annotation: its em box's middle `out` px beyond the
    /// base's content edge on its side.
    Level {
        under: bool,
        out: f32,
    },
    /// An inter-character annotation: upright, its em box's middle `down`
    /// px below the base's em box's.
    Inter {
        down: f32,
    },
}

/// One cluster of a ruby where it is drawn.
#[derive(Clone, Debug)]
pub(crate) struct Placed {
    pub text: String,
    /// From the ruby's start along its line, and its length there.
    pub at: f32,
    pub len: f32,
    pub size: f32,
    pub place: Place,
}

/// A ruby along its line, px: the room it takes there, its start against
/// that room (before it when the annotation hangs over the text before),
/// its whole length, and its clusters.
pub(crate) struct RubyBox {
    pub advance: f32,
    pub column: f32,
    pub length: f32,
    pub placed: Vec<Placed>,
}

/// How far a ruby's annotations reach beyond its base's content edges,
/// over and under, at base size `size` (nested rubies' on that side
/// first, §3.1.2); an inter-character level is over down a line.
pub(crate) fn stacks(r: &Ruby, size: f32, vertical: bool) -> (f32, f32) {
    let a = size * RUBY_SIZE;
    let (mut over, mut under) = r
        .bases
        .iter()
        .flatten()
        .filter_map(|b| match b {
            RubyBase::Ruby(inner) => Some(stacks(inner, size, vertical)),
            RubyBase::Text(_) => None,
        })
        .fold((0.0f32, 0.0f32), |(o, u), (a, b)| (o.max(a), u.max(b)));
    for l in &r.levels {
        match side(l.side, vertical) {
            RubySide::Under => under += a,
            RubySide::Over => over += a,
            RubySide::Inter => {}
        }
    }
    (over, under)
}

/// A level's side down a line or across it: inter-character is over down
/// one (§4.1).
fn side(s: RubySide, vertical: bool) -> RubySide {
    if vertical && s == RubySide::Inter {
        RubySide::Over
    } else {
        s
    }
}

/// Ruby `r` of span `s` at its base's `size` (px) along its line.
pub(crate) fn ruby_box(
    s: &RichSpan,
    r: &Ruby,
    size: f32,
    fonts: &dyn Metrics,
    v: Option<VerticalText>,
) -> RubyBox {
    let a = size * RUBY_SIZE;
    let vertical = v.is_some();
    let tracking = span_tracking(s, &r.base_text());
    let n = r.bases.len().max(1);
    // Each column's base: its clusters, a nested ruby one of them.
    let mut bases: Vec<Vec<(String, f32, Option<RubyBox>)>> = Vec::new();
    for base in &r.bases {
        let mut list = Vec::new();
        for item in base {
            match item {
                RubyBase::Text(t) => list.extend(
                    clusters(s, t, size, tracking, fonts, v)
                        .into_iter()
                        .map(|(c, l)| (c, l, None)),
                ),
                RubyBase::Ruby(inner) => {
                    let b = ruby_box(s, inner, size, fonts, v);
                    list.push(("\u{FFFC}".to_string(), b.length, Some(b)));
                }
            }
        }
        bases.push(list);
    }
    let interlinear: Vec<(&RubyLevel, RubySide)> = r
        .levels
        .iter()
        .map(|l| (l, side(l.side, vertical)))
        .collect();
    let annotations: Vec<Vec<Vec<(String, f32)>>> = r
        .levels
        .iter()
        .map(|l| {
            l.texts
                .iter()
                .map(|t| clusters(s, t, a, 0.0, fonts, v))
                .collect()
        })
        .collect();
    // Column lengths: the longest of base and paired annotations (§3.1.1).
    let mut widths: Vec<f32> = (0..n)
        .map(|k| bases.get(k).map_or(0.0, |b| b.iter().map(|c| c.1).sum()))
        .collect();
    for (l, (level, sd)) in interlinear.iter().enumerate() {
        if level.spanning || *sd == RubySide::Inter {
            continue;
        }
        for (k, w) in widths.iter_mut().enumerate() {
            *w = w.max(annotations[l].get(k).map_or(0.0, |c| length(c)));
        }
    }
    // Inter-character annotations: a column of their own, one annotation
    // em wide, after their base (a spanning one after the last).
    let mut inter = vec![0.0f32; n];
    for (level, sd) in &interlinear {
        if *sd != RubySide::Inter {
            continue;
        }
        for (k, text) in level.texts.iter().enumerate() {
            if !text.is_empty() {
                let k = if level.spanning { n - 1 } else { k };
                inter[k.min(n - 1)] += a;
            }
        }
    }
    // A spanning annotation longer than its columns widens them evenly.
    for (l, (level, sd)) in interlinear.iter().enumerate() {
        if !level.spanning || *sd == RubySide::Inter {
            continue;
        }
        let span: f32 = widths.iter().sum::<f32>() + inter[..n - 1].iter().sum::<f32>();
        let need = annotations[l].first().map_or(0.0, |c| length(c));
        if need > span {
            let extra = (need - span) / n as f32;
            widths.iter_mut().for_each(|w| *w += extra);
        }
    }
    let starts: Vec<f32> = (0..n)
        .scan(0.0f32, |at, k| {
            let start = *at;
            *at += widths[k] + inter[k];
            Some(start)
        })
        .collect();
    let length = starts[n - 1] + widths[n - 1] + inter[n - 1];
    let mut placed = Vec::new();
    // Bases, and the nested rubies in them.
    let (mut over_inner, mut under_inner) = (0.0f32, 0.0f32);
    for (k, list) in bases.iter().enumerate() {
        let plain: Vec<(String, f32)> = list.iter().map(|(c, l, _)| (c.clone(), *l)).collect();
        for ((c, len, inner), at) in list.iter().zip(distribute(&plain, widths[k], r.align)) {
            let at = starts[k] + at;
            match inner {
                Some(b) => {
                    placed.extend(b.placed.iter().map(|p| Placed {
                        at: p.at + at,
                        ..p.clone()
                    }));
                }
                None => placed.push(Placed {
                    text: c.clone(),
                    at,
                    len: *len,
                    size,
                    place: Place::Base,
                }),
            }
        }
        for item in &r.bases[k] {
            if let RubyBase::Ruby(inner) = item {
                let (o, u) = stacks(inner, size, vertical);
                over_inner = over_inner.max(o);
                under_inner = under_inner.max(u);
            }
        }
    }
    // Annotation levels, stacked outward on their sides past the nested
    // rubies' (§3.1.2).
    let (mut over, mut under) = (over_inner, under_inner);
    // Inter-character levels stack rightwards in level order (§3.2).
    let mut right = vec![0.0f32; n];
    for (l, (level, sd)) in interlinear.iter().enumerate() {
        match sd {
            RubySide::Inter => {
                for (k, list) in annotations[l].iter().enumerate() {
                    if list.is_empty() {
                        continue;
                    }
                    let k = if level.spanning { n - 1 } else { k };
                    let total = list.len() as f32 * a;
                    let x = starts[k] + widths[k] + right[k];
                    right[k] += a;
                    for (i, (c, len)) in list.iter().enumerate() {
                        placed.push(Placed {
                            text: c.clone(),
                            at: x,
                            len: *len,
                            size: a,
                            place: Place::Inter {
                                down: -total / 2.0 + a * (i as f32 + 0.5),
                            },
                        });
                    }
                }
            }
            RubySide::Over | RubySide::Under => {
                let under_side = *sd == RubySide::Under;
                let stack = if under_side { &mut under } else { &mut over };
                let out = *stack + a / 2.0;
                *stack += a;
                for (k, list) in annotations[l].iter().enumerate() {
                    let (x, w) = if level.spanning {
                        (0.0, length - inter[n - 1])
                    } else {
                        (starts[k], widths[k])
                    };
                    for ((c, len), at) in list.iter().zip(distribute(list, w, r.align)) {
                        placed.push(Placed {
                            text: c.clone(),
                            at: x + at,
                            len: *len,
                            size: a,
                            place: Place::Level {
                                under: under_side,
                                out,
                            },
                        });
                    }
                }
            }
        }
    }
    // Hanging over the text beside it (§5.1): by the room before its first
    // base and after its last, at most one annotation character.
    let edge = |first: bool| {
        let base: Vec<&Placed> = placed.iter().filter(|p| p.place == Place::Base).collect();
        let room = if first {
            base.iter().map(|p| p.at).fold(f32::INFINITY, f32::min)
        } else {
            length - base.iter().map(|p| p.at + p.len).fold(0.0, f32::max)
        };
        if room.is_finite() { room.max(0.0) } else { 0.0 }
    };
    let hang = |k: usize| {
        if r.overhang[k] && (k == 0 || inter[n - 1] == 0.0) {
            edge(k == 0).min(a)
        } else {
            0.0
        }
    };
    let (before, after) = (hang(0), hang(1));
    RubyBox {
        advance: length - before - after,
        column: -before,
        length,
        placed,
    }
}

/// The over and under edges of `text`'s content area in `m`, em above
/// and below its baseline (positive): its faces' ascent and descent, the
/// widest of a mixed text's (CSS Inline 3 §5.1). Ruby and emphasis marks
/// sit on these.
pub(crate) fn content_edges(m: &crate::text::FontMetrics, text: &str) -> (f32, f32) {
    let own = (m.over_edge(), m.under_edge());
    if m.runs.is_none() {
        return own;
    }
    m.font_runs(text)
        .iter()
        .filter_map(|(_, k)| m.part(*k))
        .fold(own, |(a, d), p| {
            (a.max(p.over_edge()), d.max(p.under_edge()))
        })
}

/// Ruby `r`'s extents above and below its base's baseline in a line, px:
/// the base's content and half-leading, and on each side what its
/// annotations lack of the line's leading (CSS Ruby 1 §3.6: a line as
/// tall as its line height holds its annotations when both half-leadings,
/// its own and the next line's, do).
pub(crate) fn ruby_extents(s: &RichSpan, r: &Ruby, fonts: &dyn Metrics) -> (f32, f32) {
    let size = s.step.size;
    let m = fonts.metrics_styled(s.font, s.query(), &s.text);
    let (a, d) = content_edges(m, &s.text);
    let (asc, desc) = (a * size, d * size);
    let lh = if s.text.chars().any(crate::text::is_cjk) {
        s.step.cjk_line_height
    } else {
        s.step.line_height
    };
    let lead = (lh - asc - desc) / 2.0;
    let (over, under) = stacks(r, size, false);
    (
        asc + lead + (over - 2.0 * lead).max(0.0),
        desc + lead + (under - 2.0 * lead).max(0.0),
    )
}

/// Where each of `clusters` starts in a column `length` long, by
/// `ruby-align` (CSS Ruby 1 §4.3): `space-between` and `space-around`
/// share the free room among the justification opportunities between
/// two CJK characters (`text-justify: ruby`), `space-around` with half a
/// share at each end; with none they centre, as `center` does.
pub(crate) fn distribute(clusters: &[(String, f32)], length: f32, align: RubyAlign) -> Vec<f32> {
    let free = (length - self::length(clusters)).max(0.0);
    let cjk: Vec<bool> = clusters
        .iter()
        .map(|(c, _)| c.chars().next().is_some_and(crate::text::is_cjk))
        .collect();
    let gaps: Vec<bool> = cjk.windows(2).map(|w| w[0] && w[1]).collect();
    let k = gaps.iter().filter(|g| **g).count() as f32;
    let (lead, gap) = match align {
        RubyAlign::Start => (0.0, 0.0),
        RubyAlign::SpaceBetween if k > 0.0 => (0.0, free / k),
        RubyAlign::SpaceAround if k > 0.0 => (free / (k + 1.0) / 2.0, free / (k + 1.0)),
        _ => (free / 2.0, 0.0),
    };
    let mut at = lead;
    clusters
        .iter()
        .enumerate()
        .map(|(n, (_, len))| {
            let start = at;
            at += len
                + if gaps.get(n).copied().unwrap_or(false) {
                    gap
                } else {
                    0.0
                };
            start
        })
        .collect()
}

/// Where a ruby is drawn: its stretch of its line, the base's baseline
/// (across) or the column's middle (down), its ink.
pub(super) struct RubyAt {
    pub rect: Rect,
    pub baseline: f32,
    pub color: Srgba,
}

impl Ui {
    /// Ruby `r` of span `s`'s commands: each cluster of its bases and
    /// annotations where [`ruby_box`] places it.
    pub(super) fn ruby_commands(
        &self,
        i: usize,
        s: &RichSpan,
        r: &Ruby,
        at: RubyAt,
        v: Option<VerticalText>,
        cmds: &mut Vec<crate::draw::DrawCmd>,
    ) {
        let n = &self.tree.nodes[i];
        let sk = n.placed.scale;
        let size = s.step.size * sk;
        let fonts = &self.env.fonts;
        let geometry = ruby_box(s, r, size, fonts, v);
        let ink = at.color.fade(n.placed.alpha).linear();
        let clip = self.paint_clip(n);
        let features = s.step.features;
        let ruby_forms = features.with_settings(&[(*b"ruby", 1)]);
        let weight = if s.synth.bold {
            bold_weight(size, 0.0)
        } else {
            0.0
        };
        let base_text = r.base_text();
        let base_font = fonts.for_styled(s.font, s.query(), &base_text).0;
        let rect = at.rect;
        let text_cmd = |x: f32,
                        baseline: f32,
                        size: f32,
                        text: String,
                        font: Option<og_draw::FontHandle>,
                        placements: Vec<og_draw::GlyphPlacement>,
                        bounds: Rect,
                        transform: [f32; 4],
                        tracking: f32|
         -> Option<crate::draw::DrawCmd> {
            let font = font.filter(|_| !text.is_empty())?;
            let cmd = crate::draw::DrawCmd::Text {
                x,
                baseline,
                size,
                text,
                color: ink,
                font,
                bounds,
                clip,
                tracking,
                weight,
                glow: None,
                placements,
            };
            Some(if transform == [1.0, 0.0, 0.0, 1.0] {
                cmd
            } else {
                crate::draw::DrawCmd::StyledText {
                    text: Box::new(cmd),
                    effects: Box::new(og_draw::UiTextEffects {
                        transform,
                        ..Default::default()
                    }),
                }
            })
        };
        let para = self.text_flow(i).1;
        let tracking = span_tracking(s, &base_text);
        let Some(v) = v else {
            // Across: the base on its baseline, each level's em box on
            // the base's content edge, stacked outward.
            let (ba, bd) = content_edges(&base_font.metrics, &base_text);
            let snap = |y: f32| crate::layout::snap(y, self.env.viewport.dpr());
            let x0 = rect.x + geometry.column;
            for p in &geometry.placed {
                let font = fonts.for_styled(s.font, s.query(), &p.text).0;
                let (pa, pd) = content_edges(&font.metrics, &p.text);
                // An annotation's content area centred in its em box (its
                // line height 1, HTML's `rt`).
                let centred = |middle: f32| middle + (pa - pd) / 2.0 * p.size;
                let (x, baseline, features, track) = match p.place {
                    Place::Base => (x0 + p.at, at.baseline, features, tracking),
                    Place::Level { under, out } => {
                        let middle = if under {
                            at.baseline + bd * size + out
                        } else {
                            at.baseline - ba * size - out
                        };
                        (x0 + p.at, centred(middle), ruby_forms, 0.0)
                    }
                    Place::Inter { down } => {
                        let middle = at.baseline - (ba - bd) / 2.0 * size + down;
                        let w = font.metrics.measure_tracked(&p.text, p.size, 0.0);
                        let a = size * RUBY_SIZE;
                        (x0 + p.at + (a - w) / 2.0, centred(middle), ruby_forms, 0.0)
                    }
                };
                // On the device pixel grid, as the line's other text.
                let baseline = snap(baseline);
                for pc in crate::text::pieces_with(font, &p.text, para, track, features) {
                    let x = x + pc.dx * p.size;
                    let bounds = Rect::new(x, baseline - p.size, p.len, p.size * 1.2);
                    cmds.extend(text_cmd(
                        x,
                        baseline,
                        p.size,
                        pc.text,
                        pc.font,
                        pc.placements,
                        bounds,
                        [1.0, 0.0, 0.0, 1.0],
                        track,
                    ));
                }
            }
            return;
        };
        // Down a vertical line: each level's column beside the base's, on
        // the line-over side unless under.
        let from_bottom = self.rich_from_bottom(i, v);
        let over = if v.mode == WritingMode::SidewaysLr {
            -1.0
        } else {
            1.0
        };
        for p in &geometry.placed {
            let font = fonts.for_styled(s.font, s.query(), &p.text).0;
            let (mid, track, features) = match p.place {
                Place::Base => (at.baseline, tracking, features),
                Place::Level { under, out } => {
                    let side = if under { -over } else { over };
                    (at.baseline + side * (size / 2.0 + out), 0.0, ruby_forms)
                }
                Place::Inter { .. } => continue,
            };
            let run = crate::ui::VerticalRun {
                tracking: track,
                features,
                ..self.span_run(
                    i,
                    s,
                    font,
                    &p.text,
                    p.size,
                    v,
                    (mid, mid - p.size / 2.0, p.size),
                    true,
                )
            };
            let offset = geometry.column + p.at;
            let top = |at: f32, len: f32| {
                if from_bottom {
                    rect.bottom() - offset - at - len
                } else {
                    rect.y + offset + at
                }
            };
            let mut along = 0.0;
            for (range, set) in crate::text::vertical::runs(&p.text, v) {
                along = crate::ui::vertical_run(
                    &run,
                    &p.text[range],
                    set,
                    along,
                    &top,
                    &mut || 0.0,
                    &mut |_, _, _| {},
                    &mut |x, baseline, t, handle, placements, transform, bounds| {
                        cmds.extend(text_cmd(
                            x,
                            baseline,
                            p.size,
                            t,
                            handle,
                            placements,
                            bounds,
                            transform,
                            run.tracking,
                        ));
                    },
                );
            }
        }
    }
}
