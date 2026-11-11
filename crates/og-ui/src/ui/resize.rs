//! CSS `resize` (CSS UI 4 §5.2): an element whose overflow is not
//! `visible` and says `both`, `horizontal` or `vertical` (`inline`,
//! `block`) has a grab handle in its bottom end corner. A press there
//! holds the pointer, and its drag sets the element's width or height in
//! CSS px, as a browser writes the element's size: the size stays on the
//! element (UI state, presentation only) while it says `resize`, under its
//! `min-` and `max-` bounds, and the element's own build cannot undo it.
//!
//! Only an element being dragged is written again; a resized one that is
//! idle costs nothing, its size staying on its retained node.

use super::*;

/// The handle's side, CSS px: the corner square a press grabs.
pub(crate) const GRIP: f32 = 14.0;

/// [`crate::layout::Element::resize`]: across, down.
pub(crate) const RESIZE_X: u8 = 1;
pub(crate) const RESIZE_Y: u8 = 2;

/// A drag of a resize handle: the element, the axes it resizes, where the
/// press was and the element's size then.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Resizing {
    pub key: u64,
    pub axes: u8,
    pub from: Vec2,
    pub size: [f32; 2],
}

/// Its handle's key (a hit of its own, over the element).
pub(crate) fn grip_key(key: u64) -> u64 {
    child_key(key, "::resizer")
}

/// The handle's rectangle on node `n`'s box (its bottom end corner).
pub(crate) fn grip_rect(n: &Node, rtl: bool) -> Rect {
    let r = n.rect;
    let s = GRIP.min(r.width).min(r.height);
    let x = if rtl { r.x } else { r.right() - s };
    Rect::new(x, r.bottom() - s, s, s)
}

impl Ui {
    /// This tick's resizing, before interaction is decided: a press on a
    /// handle grabs it, and while it is held its drag sets the element's
    /// size. Answers whether the pointer is the handle's this tick.
    pub(crate) fn resize_input(&mut self, input: &crate::input::InputFrame) -> bool {
        let p = input.pointer;
        if input.pressed && input.pointer_inside && self.interaction.state.thumb.is_none() {
            let top = self
                .interaction
                .state
                .hits
                .iter()
                .rev()
                .find(|h| h.rect.contains(p.x, p.y) && h.flags & HIT_PASS == 0)
                .map(|h| h.key);
            self.interaction.state.resizing = top.and_then(|k| {
                self.orders
                    .with(visit::feat::RESIZE)
                    .iter()
                    .map(|&i| &self.nodes[i])
                    .find(|n| grip_key(n.key) == k)
                    .map(|n| Resizing {
                        key: n.key,
                        axes: n.el.resize,
                        from: p,
                        size: [n.w, n.h],
                    })
            });
        }
        let Some(r) = self.interaction.state.resizing else {
            return false;
        };
        if !(input.down || input.pressed) {
            self.interaction.state.resizing = None;
            return true;
        }
        let rtl = self.env.direction == og_i18n::Direction::Rtl;
        let d = p - r.from;
        let mut size = self
            .interaction
            .state
            .resized
            .get(&r.key)
            .copied()
            .unwrap_or([None; 2]);
        if r.axes & RESIZE_X != 0 {
            let dx = if rtl { -d.x } else { d.x };
            size[0] = Some((r.size[0] + dx).max(0.0));
        }
        if r.axes & RESIZE_Y != 0 {
            size[1] = Some((r.size[1] + d.y).max(0.0));
        }
        if self.interaction.state.resized.get(&r.key) != Some(&size) {
            self.interaction.state.resized.insert(r.key, size);
            self.interaction.resize_moved = Some(r.key);
        }
        true
    }

    /// Before layout: the sizes the user gave resizable elements, on the
    /// nodes written this frame and on the one a drag moved (written again
    /// for it). Nothing while no element was ever resized.
    pub(super) fn apply_resized(&mut self) {
        if self.interaction.state.resized.is_empty() {
            return;
        }
        if let Some(key) = self.interaction.resize_moved.take()
            && let Some(i) = self.tree.find(key)
            && !self.emitted.contains(&i)
        {
            let previous = self.nodes[i].clone();
            self.layout.cache.prev.entry(i).or_insert(previous);
            self.emitted.push(i);
        }
        for k in 0..self.emitted.len() {
            let i = self.emitted[k];
            let n = &mut self.nodes[i];
            if n.el.resize == 0 {
                continue;
            }
            let Some(size) = self.interaction.state.resized.get(&n.key) else {
                continue;
            };
            if let Some(w) = size[0].filter(|_| n.el.resize & RESIZE_X != 0) {
                n.el.width = crate::Size::Fixed(w);
            }
            if let Some(h) = size[1].filter(|_| n.el.resize & RESIZE_Y != 0) {
                n.el.height = crate::Size::Fixed(h);
            }
        }
    }

    /// Node `s`'s resize handle: three rows of dots in its bottom end
    /// corner, the style's strong control ink.
    pub(super) fn resizer_commands(&self, s: usize, cmds: &mut Vec<DrawCmd>) {
        let n = &self.nodes[s];
        let rtl = self.env.direction == og_i18n::Direction::Rtl;
        let g = grip_rect(n, rtl);
        let dpr = self.env.viewport.dpr();
        let color = self.env.theme.palette.control_strong.linear();
        let (dot, step) = (2.0, 4.0);
        for row in 0..3 {
            for col in 0..=row {
                // Right to left the triangle opens the other way.
                let cx = if rtl {
                    g.x + 2.0 + col as f32 * step
                } else {
                    g.right() - 2.0 - dot - col as f32 * step
                };
                let cy = g.bottom() - 2.0 - dot - (2 - row) as f32 * step;
                cmds.push(DrawCmd::Quad {
                    rect: Rect::new(cx, cy, dot, dot).snapped(dpr),
                    radius: dot / 2.0,
                    color,
                    clip: n.clip,
                });
            }
        }
    }
}
