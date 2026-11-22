//! Paint order is separate from parent-first geometry. An explicit z
//! establishes an atomic context, including z-0; auto descendants belong
//! to their nearest context. Each context paints its own box first, then
//! negative, auto/zero and positive members in stable document order.
//! A filter (a blur, a drop shadow or a backdrop's) establishes one too,
//! as in CSS, so a filter group's subtree stays one run of quads.

use super::{El, NONE, PaintGroup, PaintQuad};
use std::ops::Range;

#[derive(Default)]
pub(super) struct Stacking {
    pub order: Vec<u32>,
    // (owning context, level, document position, element).
    members: Vec<(u32, i32, usize, u32)>,
    context: Vec<u32>,
    walk: Vec<u32>,
    ranges: Vec<Range<usize>>,
    scratch: Vec<PaintQuad>,
    // Each quad's place after the reorder, for the groups.
    moved: Vec<u32>,
    changed: bool,
}

impl Stacking {
    pub fn begin(&mut self, els: &[El], structural: &[u32]) {
        self.order.clear();
        if !structural
            .iter()
            .any(|&e| els[e as usize].record.z_index.is_some())
        {
            self.order.extend_from_slice(structural);
            self.changed = false;
            return;
        }
        self.context.clear();
        self.context.resize(els.len(), NONE);
        self.members.clear();
        for (position, &e) in structural.iter().enumerate() {
            let el = &els[e as usize];
            let owner = if el.arena_parent == NONE {
                NONE
            } else {
                self.context[el.arena_parent as usize]
            };
            self.members
                .push((owner, el.record.z_index.unwrap_or(0), position, e));
            self.context[e as usize] = if el.record.z_index.is_some() || filters(el) {
                e
            } else {
                owner
            };
        }
        // The document position breaks ties, so the nonallocating sort
        // is stable in the only sense the painter needs.
        self.members.sort_unstable();
        self.walk.clear();
        self.push_context(NONE);
        while let Some(e) = self.walk.pop() {
            self.order.push(e);
            self.push_context(e);
        }
        self.changed = self.order != structural;
        if self.changed {
            self.ranges.clear();
            self.ranges.resize(els.len(), 0..0);
        }
    }

    fn push_context(&mut self, owner: u32) {
        let start = self.members.partition_point(|m| m.0 < owner);
        let end = self.members.partition_point(|m| m.0 <= owner);
        self.walk
            .extend(self.members[start..end].iter().rev().map(|m| m.3));
    }

    /// Whether the paint order moves quads ([`Self::reorder`] runs).
    pub fn reorders(&self) -> bool {
        self.changed
    }

    pub fn painted(&mut self, e: usize, range: Range<usize>) {
        if self.changed {
            self.ranges[e] = range;
        }
    }

    /// Puts the quads in paint order, and each group's quads where they
    /// moved (a group's subtree is a context, so they stay one run).
    pub fn reorder(&mut self, quads: &mut Vec<PaintQuad>, groups: &mut [PaintGroup]) {
        if !self.changed {
            return;
        }
        self.scratch.clear();
        self.moved.clear();
        self.moved.resize(quads.len(), u32::MAX);
        for &e in &self.order {
            let range = self.ranges[e as usize].clone();
            for (k, old) in range.clone().enumerate() {
                self.moved[old] = (self.scratch.len() + k) as u32;
            }
            self.scratch.extend_from_slice(&quads[range]);
        }
        std::mem::swap(quads, &mut self.scratch);
        for g in groups {
            let at = self.moved[g.quad as usize];
            let len = g.end - g.quad;
            debug_assert!(
                self.moved[g.quad as usize..g.end as usize]
                    .iter()
                    .all(|&k| (at..at + len).contains(&k)),
                "a group's quads stay one run"
            );
            g.end = at + len;
            g.quad = at;
        }
    }
}

/// Whether `el` filters what it holds or what is under it, which makes it
/// a stacking context.
fn filters(el: &El) -> bool {
    let r = &el.record;
    r.effect_count > 0 || r.skin.backdrop_blur > 0.0 || r.backdrop_filter.is_some()
}
