//! The element arena: every element a slot indexed by `u32`, laid out on
//! rune-taffy's dense store. An element's layout style is a typed
//! `CompactStyle` set when its record is applied; a write that changes
//! nothing marks nothing. A frame with nothing dirty returns at once: no
//! traversal, no layout and no allocation.

use rune_taffy::{BoxLayout, CompactStyle, LayoutStore, SlotId};
use taffy::{AvailableSpace, LayoutInput, LayoutOutput, Size};

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
    slot: SlotId,
}

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
            slot: SlotId(0),
        });
        if parent != NONE {
            self.nodes[parent as usize].children.push(id);
        }
        self.dirty = true;
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
            self.dirty = true;
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
            self.dirty = true;
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
        let mut root = self.nodes[0].style.clone();
        root.set_size(Size {
            width: taffy::Dimension::length(width),
            height: taffy::Dimension::length(height),
        });
        self.nodes[0].style = root;
        self.store.begin_commit();
        let (slot, _) = sync(&mut self.store, &mut self.nodes, 0);
        self.root_slot = slot;
        // Slots move only when the store compacts.
        let nodes = &mut self.nodes;
        self.store
            .end_commit(&mut self.root_slot, |idx, s| nodes[idx as usize].slot = s);
        let nodes = &self.nodes;
        self.store.compute_layout(
            self.root_slot,
            Size {
                width: AvailableSpace::Definite(width),
                height: AvailableSpace::Definite(height),
            },
            |input: LayoutInput, ctx: Option<usize>, _style: &CompactStyle| {
                let text = ctx.and_then(|i| nodes[i].text.as_ref());
                let Some((text, run)) = text else {
                    return LayoutOutput::HIDDEN;
                };
                let avail = input
                    .known_dimensions
                    .width
                    .or(match input.available_space.width {
                        AvailableSpace::Definite(w) => Some(w),
                        _ => None,
                    });
                let (w, h) = measure(text, run, avail);
                LayoutOutput::from_outer_size(Size {
                    width: input.known_dimensions.width.unwrap_or(w),
                    height: input.known_dimensions.height.unwrap_or(h),
                })
            },
        );
        self.dirty = false;
        true
    }

    /// An element's box, relative to its parent's.
    pub fn box_of(&self, id: Id) -> BoxLayout {
        self.store.box_layout(self.nodes[id as usize].slot)
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
