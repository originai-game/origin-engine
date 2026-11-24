//! The element arena: every element a slot indexed by `u32`, laid out on
//! rune-taffy's dense store. An element's layout style is a typed
//! `CompactStyle` set when its record is applied; a write that changes
//! nothing marks nothing. A frame with nothing dirty returns at once: no
//! traversal, no layout and no allocation.

use rune_taffy::{BoxLayout, CompactStyle, LayoutStore, SlotId};
use taffy::{AvailableSpace, LayoutInput, Size};

/// An element's index in its arena.
pub type Id = u32;

const NONE: Id = u32::MAX;

#[derive(Clone, Debug)]
struct Node {
    parent: Id,
    children: Vec<Id>,
    key: u64,
    style: CompactStyle,
    /// What its `space-*` gives its children ([`spaced`]).
    space: crate::record::Spacing,
    /// A text leaf's text and how it runs.
    text: Option<(String, Run)>,
    /// A text leaf's last two measures, by the width given
    /// ([`Avail::key`]; `EMPTY`: unused): layout asks the same sizes many
    /// times, and a measure breaks lines. Cleared when its text or run
    /// changes.
    memo: std::cell::Cell<[(u32, f32, f32); 2]>,
    slot: SlotId,
}

const EMPTY: u32 = u32::MAX - 1;
const NO_MEMO: [(u32, f32, f32); 2] = [(EMPTY, 0.0, 0.0); 2];

/// How a text leaf's text runs: what its layout is made from, resolved
/// from its element's record ([`crate::record::Text`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Run {
    /// Font size, CSS px.
    pub size: f32,
    /// Letter spacing, CSS px.
    pub tracking: f32,
    /// Line height, CSS px; negative is CSS `normal`: the ascent and
    /// descent of the fonts drawn on the line.
    pub line: f32,
    pub weight: u16,
    /// Stays on one line.
    pub nowrap: bool,
    /// The most lines it takes (`line-clamp-*`); 0 all.
    pub clamp: u16,
    /// Its `font-family` list (a CSS list, interned); `None`: the theme's
    /// sans.
    pub family: Option<nsh_tailwind::style::FontFamily>,
    pub italic: bool,
    /// `font-variant-*` (NSH's `FontVariant` bits).
    pub variant: u8,
    /// `overflow-wrap`: 0 normal, 1 break-word, 2 anywhere.
    pub wrap: u8,
    pub hyphens: og_text::layout::Hyphens,
    /// `text-overflow: ellipsis`.
    pub ellipsis: bool,
    /// `text-align`: 0 start, 1 center, 2 right, 3 left, 4 justify.
    pub align: u8,
    /// A one-line field's text (an `Input` without `multiline`): it stays
    /// on one line and takes no more width than its box has, as an HTML
    /// input's text scrolls inside it instead of widening it.
    pub field: bool,
    /// `writing-mode: vertical-rl`: its lines are columns, right to left,
    /// its glyphs upright.
    pub vertical: bool,
}

impl Run {
    /// `size` px, untracked, lines 1.2 apart, wrapping.
    pub fn sized(size: f32) -> Run {
        Run {
            size,
            tracking: 0.0,
            line: size * 1.2,
            weight: 400,
            nowrap: false,
            clamp: 0,
            family: None,
            italic: false,
            variant: 0,
            wrap: 0,
            hyphens: og_text::layout::Hyphens::Manual,
            ellipsis: false,
            align: 0,
            field: false,
            vertical: false,
        }
    }

    /// Hashes everything a layout is made from.
    fn hash(&self, h: &mut impl std::hash::Hasher) {
        use std::hash::Hash;
        [self.size, self.tracking, self.line]
            .map(f32::to_bits)
            .hash(h);
        (self.weight, self.nowrap, self.family, self.italic).hash(h);
        (
            self.variant,
            self.wrap,
            self.ellipsis,
            self.align,
            self.field,
            self.vertical,
        )
            .hash(h);
        self.clamp.hash(h);
        self.hyphens.hash(h);
    }
}

/// The width a text leaf is measured at (CSS Sizing 3 §4).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Avail {
    /// Its min-content size: broken at every opportunity.
    Min,
    /// Its max-content size: broken only where its text says.
    Max,
    /// Broken at this width, CSS px.
    Width(f32),
}

impl Avail {
    pub(crate) fn key(self) -> u32 {
        match self {
            Avail::Max => u32::MAX,
            Avail::Min => EMPTY - 1,
            Avail::Width(w) => w.to_bits().min(EMPTY - 2),
        }
    }

    fn of(key: u32) -> Avail {
        match key {
            u32::MAX => Avail::Max,
            k if k == EMPTY - 1 => Avail::Min,
            k => Avail::Width(f32::from_bits(k)),
        }
    }
}

/// Measures a text leaf (its id, its text and run): its size at the width
/// it may take.
pub type Measure<'a> = dyn FnMut(Id, &str, &Run, Avail) -> (f32, f32) + 'a;

#[derive(Clone, Debug)]
pub struct Arena {
    nodes: Vec<Node>,
    store: LayoutStore,
    root_slot: SlotId,
    /// Something changed since the last layout.
    dirty: bool,
    /// An element was added or taken out: the store is reconciled whole.
    /// Else only `touched` is patched into it.
    structure: bool,
    /// Elements whose style (true) or text (false) changed since the
    /// last layout, each once.
    touched: Vec<(Id, bool)>,
    /// Text leaves whose text changed since the last layout, each once,
    /// their memo kept: one whose new text measures the same at the widths
    /// it was laid out at moves nothing (a clock's next minute).
    retexted: Vec<Id>,
    next_key: u64,
    /// Counts what moves what the arena shows: a layout that laid out, a
    /// text that changed.
    rev: u64,
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

impl Arena {
    /// An arena holding its root (id 0), a flex column.
    pub fn new() -> Arena {
        let mut a = Arena {
            nodes: Vec::new(),
            store: LayoutStore::new(),
            root_slot: SlotId(0),
            dirty: true,
            structure: true,
            touched: Vec::new(),
            retexted: Vec::new(),
            next_key: 0,
            rev: 0,
        };
        a.push(NONE);
        a
    }

    fn push(&mut self, parent: Id) -> Id {
        let id = self.nodes.len() as Id;
        self.next_key += 1;
        self.nodes.push(Node {
            parent,
            children: Vec::new(),
            key: self.next_key,
            style: CompactStyle::default(),
            space: crate::record::Spacing::default(),
            text: None,
            memo: std::cell::Cell::new(NO_MEMO),
            // Not laid out yet: an empty box until a reconcile reaches it.
            slot: DETACHED,
        });
        if parent != NONE {
            self.nodes[parent as usize].children.push(id);
        }
        self.dirty = true;
        self.structure = true;
        id
    }

    /// A new element, the last child of `parent`.
    pub fn element(&mut self, parent: Id, style: CompactStyle) -> Id {
        let id = self.push(parent);
        self.nodes[id as usize].style = style;
        id
    }

    /// A new text leaf, the last child of `parent`.
    pub fn text(&mut self, parent: Id, text: &str, size: f32) -> Id {
        let id = self.push(parent);
        self.nodes[id as usize].text = Some((text.into(), Run::sized(size)));
        id
    }

    /// Sets how a text leaf runs; the same run marks nothing.
    pub fn set_run(&mut self, id: Id, run: Run) {
        if let Some((_, r)) = &mut self.nodes[id as usize].text
            && *r != run
        {
            *r = run;
            self.nodes[id as usize].memo.set(NO_MEMO);
            self.touch(id, false);
        }
    }

    /// Every text leaf measures again at the next layout: the font it
    /// measures in changed (a locale linked a face for what it lacked).
    pub fn remeasure(&mut self) {
        for id in 0..self.nodes.len() as Id {
            if self.nodes[id as usize].text.is_some() {
                self.nodes[id as usize].memo.set(NO_MEMO);
                self.touch(id, false);
            }
        }
    }

    /// Text leaf `id` measures again at the next layout: what it is laid
    /// out from beside its text and run changed (a paragraph's spans).
    pub fn remeasure_one(&mut self, id: Id) {
        self.nodes[id as usize].memo.set(NO_MEMO);
        self.touch(id, false);
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.len() <= 1
    }

    pub fn parent(&self, id: Id) -> Option<Id> {
        let p = self.nodes[id as usize].parent;
        (p != NONE).then_some(p)
    }

    /// An element's children, in order.
    pub fn children(&self, id: Id) -> &[Id] {
        &self.nodes[id as usize].children
    }

    /// A text leaf's text and how it runs.
    pub fn text_run(&self, id: Id) -> &Option<(String, Run)> {
        &self.nodes[id as usize].text
    }

    /// A text leaf's text.
    pub fn text_of(&self, id: Id) -> Option<&str> {
        self.nodes[id as usize].text.as_ref().map(|(t, _)| &**t)
    }

    /// Whether it makes a box (its style is not `display: none`).
    pub fn shown(&self, id: Id) -> bool {
        self.nodes[id as usize].style.display() != taffy::Display::None
    }

    /// Counts what moved what the arena shows: each layout that laid
    /// something out and each text that changed. Equal: nothing did.
    pub fn rev(&self) -> u64 {
        self.rev
    }

    /// Sets an element's style; an equal style marks nothing.
    pub fn set_style(&mut self, id: Id, style: CompactStyle) {
        let n = &mut self.nodes[id as usize];
        if n.style != style {
            n.style = style;
            self.touch(id, true);
        }
    }

    /// Sets the margins an element gives its children but the last
    /// (`space-x-*`); its children lay out again when they change.
    pub fn set_space(&mut self, id: Id, space: crate::record::Spacing) {
        if self.nodes[id as usize].space != space {
            self.nodes[id as usize].space = space;
            for k in 0..self.nodes[id as usize].children.len() {
                self.touch(self.nodes[id as usize].children[k], true);
            }
        }
    }

    /// Sets a text leaf's text; the same text marks nothing.
    pub fn set_text(&mut self, id: Id, text: &str) {
        let n = &mut self.nodes[id as usize];
        if let Some((t, _)) = &mut n.text
            && &**t != text
        {
            t.clear();
            t.push_str(text);
            self.dirty = true;
            self.rev += 1;
            if !self.retexted.contains(&id) {
                self.retexted.push(id);
            }
        }
    }

    /// Settles the leaves whose text changed: one that measures the same
    /// at every width its memo holds keeps its box (the memo stays); any
    /// other is touched, its memo cleared. With the store reconciled whole,
    /// every one is cleared.
    fn settle_texts(&mut self, measure: &mut Measure<'_>) {
        for k in 0..self.retexted.len() {
            let id = self.retexted[k];
            let n = &self.nodes[id as usize];
            let memo = n.memo.get();
            let same = !self.structure
                && memo[0].0 != EMPTY
                && n.text.as_ref().is_some_and(|(t, run)| {
                    memo.iter()
                        .filter(|m| m.0 != EMPTY)
                        .all(|&(key, w, h)| measure(id, t, run, Avail::of(key)) == (w, h))
                });
            if !same {
                self.nodes[id as usize].memo.set(NO_MEMO);
                self.touch(id, false);
            }
        }
        self.retexted.clear();
    }

    fn touch(&mut self, id: Id, style: bool) {
        self.dirty = true;
        if self.structure {
            return;
        }
        match self.touched.iter_mut().find(|(t, _)| *t == id) {
            Some((_, s)) => *s |= style,
            None => self.touched.push((id, style)),
        }
    }

    /// Takes `id` out of its parent's children: it and its subtree are no
    /// longer laid out (their slots stay, unused, until the arena is made
    /// again).
    pub fn detach(&mut self, id: Id) {
        let p = self.nodes[id as usize].parent;
        if p != NONE {
            self.nodes[p as usize].children.retain(|&c| c != id);
            self.nodes[id as usize].parent = NONE;
            self.dirty = true;
            self.structure = true;
        }
    }

    /// Puts `id` under `parent`, before its child `before` (or last when
    /// `before` is none or not its child); a node elsewhere moves.
    pub fn attach(&mut self, id: Id, parent: Id, before: Option<Id>) {
        self.detach(id);
        let kids = &mut self.nodes[parent as usize].children;
        let at = before
            .and_then(|b| kids.iter().position(|&c| c == b))
            .unwrap_or(kids.len());
        kids.insert(at, id);
        self.nodes[id as usize].parent = parent;
        self.dirty = true;
        self.structure = true;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Whether every element has its box: none came or went since the
    /// last layout, so each one's slot in the store is its own.
    pub fn settled(&self) -> bool {
        !self.structure
    }

    /// Lays out what changed, in a `width` by `height` viewport. Answers
    /// whether anything was laid out; a clean arena returns at once.
    pub fn layout(&mut self, width: f32, height: f32, measure: &mut Measure<'_>) -> bool {
        // A new viewport is a change of the root's style, whatever else
        // changed or did not.
        let size = Size {
            width: taffy::Dimension::length(width),
            height: taffy::Dimension::length(height),
        };
        if taffy::CoreStyle::size(&self.nodes[0].style) != size {
            self.nodes[0].style.set_size(size);
            self.touch(0, true);
        }
        if !self.dirty {
            return false;
        }
        self.settle_texts(measure);
        if !self.structure && self.touched.is_empty() {
            // Only texts that kept their size changed: nothing moves.
            self.dirty = false;
            return false;
        }
        if self.structure {
            self.reconcile();
        } else {
            // The same elements: patch the ones that changed, clearing
            // their ancestors' caches, with no walk and no clone of the
            // others' styles.
            for k in 0..self.touched.len() {
                let (id, style) = self.touched[k];
                let nodes = &self.nodes;
                let slot = nodes[id as usize].slot;
                // A node out of the tree has nothing in the store to patch.
                if slot == DETACHED {
                    continue;
                }
                let parent = |i: Id| {
                    let p = nodes[i as usize].parent;
                    (p != NONE).then_some(p)
                };
                let up = std::iter::successors(parent(id), |&p| parent(p))
                    .map(|p| nodes[p as usize].slot);
                if style {
                    self.store
                        .set_style_and_mark_ancestors(slot, spaced(nodes, id), up);
                } else {
                    self.store.mark_dirty_and_ancestors(slot, up);
                }
            }
            self.touched.clear();
        }
        let nodes = &self.nodes;
        self.store.compute_layout(
            self.root_slot,
            Size {
                width: AvailableSpace::Definite(width),
                height: AvailableSpace::Definite(height),
            },
            |input: LayoutInput, ctx: Option<usize>, style: &CompactStyle| {
                // Every leaf gets its own size, padding and limits from its
                // style; a text leaf's content is its measured run.
                let text = ctx.and_then(|i| nodes[i].text.as_ref().map(|t| (i, t)));
                taffy::compute_leaf_layout(
                    input,
                    style,
                    |_, _| 0.0,
                    |known, avail| {
                        let Some((i, (text, run))) = text else {
                            return Size::ZERO;
                        };
                        // CSS intrinsic sizes: min-content is the widest
                        // piece that cannot break, max-content the text
                        // broken only where it says.
                        // Columns (`writing-vertical-rl`) break at the height
                        // they may take: their intrinsic heights are the
                        // shortest and the longest column.
                        let avail = match (known.width, avail.width) {
                            _ if run.vertical => match (known.height, avail.height) {
                                (Some(h), _) | (None, AvailableSpace::Definite(h)) => {
                                    Avail::Width(h)
                                }
                                (None, AvailableSpace::MinContent) => Avail::Min,
                                (None, AvailableSpace::MaxContent) => Avail::Max,
                            },
                            (Some(w), _) | (None, AvailableSpace::Definite(w)) => Avail::Width(w),
                            (None, AvailableSpace::MinContent) => Avail::Min,
                            (None, AvailableSpace::MaxContent) => Avail::Max,
                        };
                        let n = &nodes[i];
                        let key = avail.key();
                        let mut memo = n.memo.get();
                        let (w, h) = match memo.iter().find(|m| m.0 == key) {
                            Some(&(_, w, h)) => (w, h),
                            None => {
                                let (w, h) = measure(i as Id, text, run, avail);
                                memo = [(key, w, h), memo[0]];
                                n.memo.set(memo);
                                (w, h)
                            }
                        };
                        // A one-line field's text is no wider than it may be.
                        let w = match (run.field, avail) {
                            (false, _) | (true, Avail::Max) => w,
                            (true, Avail::Min) => 0.0,
                            (true, Avail::Width(a)) => w.min(a),
                        };
                        Size {
                            width: known.width.unwrap_or(w),
                            height: known.height.unwrap_or(h),
                        }
                    },
                )
            },
        );
        self.dirty = false;
        self.rev += 1;
        true
    }

    /// Reconciles every element into the store (an element came or went).
    fn reconcile(&mut self) {
        // A node the walk below does not reach (taken out, released) has no
        // slot once the store compacts: it is marked detached, not left
        // pointing at a slot that may be another node's or past the end.
        for n in &mut self.nodes {
            n.slot = DETACHED;
        }
        self.store.begin_commit();
        let (slot, _) = sync(&mut self.store, &mut self.nodes, 0);
        self.root_slot = slot;
        // Slots move only when the store compacts.
        let nodes = &mut self.nodes;
        self.store
            .end_commit(&mut self.root_slot, |idx, s| nodes[idx as usize].slot = s);
        self.structure = false;
        self.touched.clear();
    }

    /// An element's box, relative to its parent's.
    /// A node out of the tree (taken out or released) has an empty box.
    pub fn box_of(&self, id: Id) -> BoxLayout {
        match self.nodes[id as usize].slot {
            DETACHED => BoxLayout::default(),
            slot => self.store.box_layout(slot),
        }
    }

    /// Whether `id`'s children run past the end of its content box, as a
    /// scroller at offset 0 has more below.
    pub fn runs_past_end(&self, id: Id) -> bool {
        let b = self.box_of(id);
        let end = b.h - b.inset[3];
        self.nodes[id as usize].children.iter().any(|&c| {
            let k = self.box_of(c);
            k.h > 0.0 && k.y + k.h > end + 0.5
        })
    }
}

/// The slot of a node the last reconcile did not reach.
const DETACHED: SlotId = SlotId(u32::MAX);

/// `i`'s style with the margin its parent's `space-*` gives it: every
/// child but the last, on a side its own margin leaves at 0.
fn spaced(nodes: &[Node], i: Id) -> CompactStyle {
    let n = &nodes[i as usize];
    let Some(p) = nodes.get(n.parent as usize) else {
        return n.style.clone();
    };
    let space = p.space;
    if space == crate::record::Spacing::default() || p.children.last() == Some(&i) {
        return n.style.clone();
    }
    let mut s = taffy::Style::from(&n.style);
    let zero = taffy::LengthPercentageAuto::length(0.0);
    let m = &mut s.margin;
    if let Some((v, left)) = space.x {
        let side = if left { &mut m.left } else { &mut m.right };
        if *side == zero {
            *side = v;
        }
    }
    if let Some((v, top)) = space.y {
        let side = if top { &mut m.top } else { &mut m.bottom };
        if *side == zero {
            *side = v;
        }
    }
    CompactStyle::from(&s)
}

/// Reconciles `i`'s subtree into the store, post-order; its slot and
/// whether anything in it changed.
fn sync(store: &mut LayoutStore, nodes: &mut [Node], i: Id) -> (SlotId, bool) {
    let mark = store.begin_children();
    let mut children_dirty = false;
    for k in 0..nodes[i as usize].children.len() {
        let c = nodes[i as usize].children[k];
        let (s, d) = sync(store, nodes, c);
        store.push_child(s);
        children_dirty |= d;
    }
    let n = &nodes[i as usize];
    let measure = n.text.as_ref().map(|(t, run)| {
        use std::hash::{Hash, Hasher};
        let mut h = rustc_hash::FxHasher::default();
        t.hash(&mut h);
        run.hash(&mut h);
        h.finish()
    });
    let (slot, dirty) = store.sync_node(
        n.key,
        i,
        spaced(nodes, i),
        measure,
        false,
        mark,
        children_dirty,
    );
    nodes[i as usize].slot = slot;
    (slot, dirty)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A node not laid out yet (made, or classed while its screen is
    /// built) reads an empty box, not another node's or past the store.
    #[test]
    fn a_new_node_reads_an_empty_box_until_it_is_laid_out() {
        let mut a = Arena::new();
        let n = a.element(0, CompactStyle::default());
        assert_eq!(a.box_of(n), BoxLayout::default());
    }

    /// A subtree taken out keeps its arena nodes, but the store compacts
    /// their slots away: its boxes read empty, and a style set on it
    /// touches nothing, rather than indexing a slot past the end or one
    /// another node now holds (a screen closed and the next one opened).
    #[test]
    fn a_node_taken_out_reads_an_empty_box_after_the_store_compacts() {
        let mut a = Arena::new();
        let mut measure = |_: Id, _: &str, _: &Run, _: Avail| (0.0, 0.0);
        let sized = |w: f32| {
            CompactStyle::from(&taffy::Style {
                size: taffy::Size::from_lengths(w, 10.0),
                ..Default::default()
            })
        };
        let screen = a.element(0, CompactStyle::default());
        let rows: Vec<Id> = (0..100)
            .map(|i| a.element(screen, sized(10.0 + i as f32)))
            .collect();
        a.layout(400.0, 800.0, &mut measure);
        assert_eq!(a.box_of(rows[19]).w, 29.0);
        // Over 64 slots, three quarters of them gone: the store compacts.

        a.detach(screen);
        let next = a.element(0, sized(50.0));
        a.layout(400.0, 800.0, &mut measure);
        assert_eq!(a.box_of(next).w, 50.0);
        assert_eq!(a.box_of(rows[99]), BoxLayout::default(), "taken out");
        a.set_style(rows[3], sized(99.0));
        a.layout(400.0, 800.0, &mut measure);
        assert_eq!(a.box_of(next).w, 50.0, "the shown node is untouched");
        assert_eq!(a.box_of(rows[3]), BoxLayout::default());
    }
}
