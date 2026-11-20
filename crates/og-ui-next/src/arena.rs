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
    /// A text leaf's text and how it runs.
    text: Option<(Box<str>, Run)>,
    /// A text leaf's last two measures, by the width given (`u32::MAX`:
    /// none, `EMPTY`: unused): layout asks the same sizes many times, and
    /// a measure shapes. Cleared when its text or run changes.
    memo: std::cell::Cell<[(u32, f32, f32); 2]>,
    slot: SlotId,
}

const EMPTY: u32 = u32::MAX - 1;
const NO_MEMO: [(u32, f32, f32); 2] = [(EMPTY, 0.0, 0.0); 2];

/// How a text leaf's text runs: what its measure reads, resolved from its
/// element's record ([`crate::style::Text`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Run {
    /// Font size, CSS px.
    pub size: f32,
    /// Letter spacing, CSS px.
    pub tracking: f32,
    /// Line height, CSS px.
    pub line: f32,
    pub weight: u16,
    /// Stays on one line.
    pub nowrap: bool,
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
        }
    }
}

/// Measures a text leaf: its size given the width it may take.
pub type Measure<'a> = dyn FnMut(&str, &Run, Option<f32>) -> (f32, f32) + 'a;

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
            text: None,
            memo: std::cell::Cell::new(NO_MEMO),
            slot: SlotId(0),
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

    /// Sets an element's style; an equal style marks nothing.
    pub fn set_style(&mut self, id: Id, style: CompactStyle) {
        let n = &mut self.nodes[id as usize];
        if n.style != style {
            n.style = style;
            self.touch(id, true);
        }
    }

    /// Sets a text leaf's text; the same text marks nothing.
    pub fn set_text(&mut self, id: Id, text: &str) {
        let n = &mut self.nodes[id as usize];
        if let Some((t, _)) = &mut n.text
            && &**t != text
        {
            *t = text.into();
            self.dirty = true;
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
                    memo.iter().filter(|m| m.0 != EMPTY).all(|&(key, w, h)| {
                        let avail = (key != u32::MAX).then(|| f32::from_bits(key));
                        measure(t, run, avail) == (w, h)
                    })
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

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Lays out what changed, in a `width` by `height` viewport. Answers
    /// whether anything was laid out; a clean arena returns at once.
    pub fn layout(&mut self, width: f32, height: f32, measure: &mut Measure<'_>) -> bool {
        if !self.dirty {
            return false;
        }
        self.settle_texts(measure);
        if !self.structure && self.touched.is_empty() {
            // Only texts that kept their size changed: nothing moves.
            self.dirty = false;
            return false;
        }
        let size = Size {
            width: taffy::Dimension::length(width),
            height: taffy::Dimension::length(height),
        };
        if taffy::CoreStyle::size(&self.nodes[0].style) != size {
            self.nodes[0].style.set_size(size);
            self.touch(0, true);
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
                let parent = |i: Id| {
                    let p = nodes[i as usize].parent;
                    (p != NONE).then_some(p)
                };
                let up = std::iter::successors(parent(id), |&p| parent(p))
                    .map(|p| nodes[p as usize].slot);
                if style {
                    self.store.set_style_and_mark_ancestors(
                        slot,
                        nodes[id as usize].style.clone(),
                        up,
                    );
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
                        let avail = known.width.or(match avail.width {
                            AvailableSpace::Definite(w) => Some(w),
                            _ => None,
                        });
                        let n = &nodes[i];
                        let key = avail.map_or(u32::MAX, |w| w.to_bits().min(EMPTY - 1));
                        let mut memo = n.memo.get();
                        let (w, h) = match memo.iter().find(|m| m.0 == key) {
                            Some(&(_, w, h)) => (w, h),
                            None => {
                                let (w, h) = measure(text, run, avail);
                                memo = [(key, w, h), memo[0]];
                                n.memo.set(memo);
                                (w, h)
                            }
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
        true
    }

    /// Reconciles every element into the store (an element came or went).
    fn reconcile(&mut self) {
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
    pub fn box_of(&self, id: Id) -> BoxLayout {
        self.store.box_layout(self.nodes[id as usize].slot)
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
        [run.size, run.tracking, run.line]
            .map(f32::to_bits)
            .hash(&mut h);
        (run.weight, run.nowrap).hash(&mut h);
        h.finish()
    });
    let (slot, dirty) = store.sync_node(
        n.key,
        i,
        n.style.clone(),
        measure,
        false,
        mark,
        children_dirty,
    );
    nodes[i as usize].slot = slot;
    (slot, dirty)
}
