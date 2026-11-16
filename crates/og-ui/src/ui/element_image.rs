//! CSS `element()` images (CSS Images 4 §2.7): a box layer that shows
//! another element's rendering. Its layer draws a group cut to where it
//! paints ([`crate::draw::LayerRole::Element`]); once the command pass has
//! drawn every element, each group is filled with its element's drawing,
//! one copy in each tile the layer places at the element's border box
//! size.

use super::*;
use crate::draw::LayerRole;

/// The most tiles one `element()` layer draws: each is a copy of the
/// element's whole drawing.
const MAX_TILES: usize = 256;

/// One `element()` group in the frame's list.
struct Hole {
    /// Its [`DrawCmd::BeginLayer`].
    at: usize,
    node: usize,
    slot: u32,
    /// Its element and that element's commands in the list.
    source: Option<(usize, std::ops::Range<usize>)>,
}

impl Ui {
    /// Marks the `element()` groups node `j` drew in `cmds` (its subtree's
    /// commands; its children's groups are marked already) as its own,
    /// and the frame as drawing them.
    pub(super) fn own_element_images(&mut self, cmds: &mut [(DrawCmd, Affine)], j: usize) {
        for (c, _) in cmds {
            if let DrawCmd::BeginLayer {
                role: LayerRole::Element { node, .. },
                ..
            } = c
                && *node == u32::MAX
            {
                *node = j as u32;
                self.paint.element_images = true;
            }
        }
    }

    /// Fills the frame's `element()` groups with their elements' drawing
    /// (the module's doc). `order` and `ends` are the drawing order the
    /// pass drew `cmds` in. An element that is not drawn, or whose drawing
    /// holds the group (itself, an ancestor, or a cycle of `element()`s),
    /// is an invalid image: its group stays empty. A group inside the
    /// element drawn into another stays empty in that copy.
    pub(super) fn fill_element_images(
        &mut self,
        cmds: &mut Vec<(DrawCmd, Affine)>,
        order: &[usize],
        ends: &[usize],
    ) {
        if !std::mem::take(&mut self.paint.element_images) {
            return;
        }
        let mut holes: Vec<Hole> = cmds
            .iter()
            .enumerate()
            .filter_map(|(at, (c, _))| match c {
                DrawCmd::BeginLayer {
                    role: LayerRole::Element { node, slot },
                    ..
                } => Some(Hole {
                    at,
                    node: *node as usize,
                    slot: *slot,
                    source: None,
                }),
                _ => None,
            })
            .collect();
        let pos = &self.tree.orders.pos;
        let drawn = |i: usize| pos.get(i).is_some_and(|&p| p != u32::MAX);
        let press_scale = self.env.theme.press_scale;
        // Each group's layer, and the element it names: the document's
        // first of that id that is drawn.
        let mut layers = Vec::with_capacity(holes.len());
        for h in &mut holes {
            let layer = (h.node < self.tree.nodes.len() && drawn(h.node))
                .then(|| self.paint.draw_of(h.node, &self.tree))
                .flatten()
                .zip(clip_rect(&cmds[h.at + 1].0))
                .and_then(|(d, r)| {
                    let b = super::node_draw::part_rect(&self.tree.nodes[h.node], press_scale);
                    let (id, bg) = d.element_layer(h.slot, b, r)?;
                    let s = (0..self.tree.nodes.len())
                        .find(|&s| self.tree.nodes[s].id == id && drawn(s))?;
                    Some((r, bg, s))
                });
            if let Some((_, _, s)) = layer {
                h.source = self.drawn_range(s, order, ends).map(|r| (s, r));
            }
            layers.push(layer);
        }
        // A group depends on the groups in its element's drawing: one
        // that reaches itself is in a cycle.
        let n = holes.len();
        let inside = |h: &Hole, g: &Hole| h.source.as_ref().is_some_and(|(_, r)| r.contains(&g.at));
        let cyclic: Vec<bool> = (0..n)
            .map(|start| {
                let (mut seen, mut stack) = (vec![false; n], vec![start]);
                while let Some(h) = stack.pop() {
                    for g in 0..n {
                        if inside(&holes[h], &holes[g]) {
                            if g == start {
                                return true;
                            }
                            if !seen[g] {
                                seen[g] = true;
                                stack.push(g);
                            }
                        }
                    }
                }
                false
            })
            .collect();
        // Every copy from the list as the pass drew it.
        let copies: Vec<Vec<(DrawCmd, Affine)>> = holes
            .iter()
            .zip(&layers)
            .zip(&cyclic)
            .map(|((h, layer), &cyclic)| match (layer, &h.source, cyclic) {
                (Some((r, bg, _)), Some((s, range)), false) => {
                    self.element_copy(&cmds[range.clone()], *s, *r, bg, cmds[h.at].1)
                }
                _ => Vec::new(),
            })
            .collect();
        for (h, copy) in holes.iter().zip(copies).rev() {
            if let DrawCmd::BeginLayer { role, .. } = &mut cmds[h.at].0 {
                *role = LayerRole::Draw;
            }
            if copy.is_empty() {
                continue;
            }
            // After its cut: every later command moves.
            let at = h.at + 2;
            let by = copy.len();
            self.paint.kept_spans.retain(|r| r.end <= at);
            self.paint.cache.aligned.retain(|r| r.end <= at);
            for c in self.paint.cache.copied.iter_mut().filter(|c| c.0 >= at) {
                c.0 += by;
            }
            super::commands::shift_named(&mut self.paint.cache.named, at, by);
            self.grow_ranges(h.node, by, order, ends);
            cmds.splice(at..at, copy);
        }
        // The copies hold groups of the elements drawn into them: empty.
        for (c, _) in cmds.iter_mut() {
            if let DrawCmd::BeginLayer { role, .. } = c
                && matches!(role, LayerRole::Element { .. })
            {
                *role = LayerRole::Draw;
            }
        }
    }

    /// Element `s`'s drawing, `range` its commands as the pass drew them,
    /// as an image in each tile `bg` places in `r` (its group's cut,
    /// under the group's map `world`): the element at its border box's
    /// size, without the transforms, the opacity and the clips of its
    /// ancestors and its own transform (CSS Images 4 §2.7), each tile cut
    /// to its rectangle.
    fn element_copy(
        &self,
        range: &[(DrawCmd, Affine)],
        s: usize,
        r: Rect,
        bg: &og_draw::UiBackground,
        world: Affine,
    ) -> Vec<(DrawCmd, Affine)> {
        let n = &self.tree.nodes[s];
        let from = n.placed.rect;
        let Some(unmap) = n.placed.world.invert() else {
            return Vec::new();
        };
        if from.width <= 0.0 || from.height <= 0.0 {
            return Vec::new();
        }
        let inherited = if n.parent == NONE {
            1.0
        } else {
            self.tree.nodes[n.parent].placed.alpha
        };
        let own: Vec<(DrawCmd, Affine)> = range
            .iter()
            .filter(|(c, _)| !matches!(c, DrawCmd::World(_)))
            .map(|(c, m)| {
                let mut c = c.clone();
                if inherited > 0.0 && inherited < 1.0 && !matches!(c, DrawCmd::EndLayer { .. }) {
                    fade_cmd(&mut c, 1.0 / inherited);
                }
                if let Some(clip) = c.clip_mut() {
                    *clip = super::commands::INFINITE;
                }
                (c, unmap.mul(m))
            })
            .collect();
        let mut out = Vec::new();
        for [x, y, w, h] in bg.tiles([r.width, r.height], [from.width, from.height], MAX_TILES) {
            let tile = Rect::new(r.x + x, r.y + y, w, h);
            let map = world.mul(&Affine::rect_to_rect(&from, &tile));
            out.push((
                DrawCmd::PushClip {
                    geometry: og_draw::UiGeometry::Rect {
                        radii: [0.0; 4],
                        corner: og_draw::UiCorner::Round,
                    },
                    center: (tile.x + w / 2.0, tile.y + h / 2.0),
                    half: (w / 2.0, h / 2.0),
                },
                world,
            ));
            out.extend(own.iter().map(|(c, m)| (c.clone(), map.mul(m))));
            out.push((DrawCmd::PopClip, world));
        }
        out
    }

    /// Where node `s`'s commands lie in the frame's list: its range from
    /// its parent's start (in the drawing order) as the pass recorded it,
    /// down from a tree's root. `None` when it was not drawn.
    fn drawn_range(
        &self,
        s: usize,
        order: &[usize],
        ends: &[usize],
    ) -> Option<std::ops::Range<usize>> {
        let p = *self.tree.orders.pos.get(s)? as usize;
        let (mut k, mut end, mut base) = (0, order.len(), 0);
        while k < end {
            if p >= ends[k] {
                k = ends[k];
                continue;
            }
            let &(rel, len, _) = self
                .paint
                .cache
                .ranges
                .get(&self.tree.nodes[order[k]].key)?;
            let start = base + rel;
            if k == p {
                return Some(start..start + len);
            }
            (base, end, k) = (start, ends[k], k + 1);
        }
        None
    }

    /// `by` commands went into node `o`'s drawing: it and the subtrees
    /// around it in the drawing order grow by them, and the subtrees after
    /// it in each start that much later, so next frame copies each from
    /// where it stands.
    fn grow_ranges(&mut self, o: usize, by: usize, order: &[usize], ends: &[usize]) {
        let Some(&p) = self.tree.orders.pos.get(o) else {
            return;
        };
        let p = p as usize;
        let ranges = &mut self.paint.cache.ranges;
        let key = |k: usize| self.tree.nodes[order[k]].key;
        let (mut k, mut end, mut past) = (0, order.len(), false);
        while k < end {
            if past || p >= ends[k] {
                if past && let Some(r) = ranges.get_mut(&key(k)) {
                    r.0 += by;
                }
                k = ends[k];
                continue;
            }
            if let Some(r) = ranges.get_mut(&key(k)) {
                r.1 += by;
            }
            // Its siblings after the subtree holding it.
            let mut j = ends[k];
            while j < end {
                if let Some(r) = ranges.get_mut(&key(j)) {
                    r.0 += by;
                }
                j = ends[j];
            }
            // Below it: its children all come after its own drawing.
            (end, k, past) = (ends[k], k + 1, k == p);
        }
    }
}

/// The rectangle a [`DrawCmd::PushClip`] cuts to, in its map's space.
fn clip_rect(c: &DrawCmd) -> Option<Rect> {
    match c {
        DrawCmd::PushClip { center, half, .. } => Some(Rect::new(
            center.0 - half.0,
            center.1 - half.1,
            2.0 * half.0,
            2.0 * half.1,
        )),
        _ => None,
    }
}
