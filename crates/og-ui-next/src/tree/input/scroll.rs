//! Scroll containers (`Scroll`), wheel scrolling, scroll clamping, and
//! focus scroll-into-view.

use super::*;

impl Tree {
    /// How far a scroller's children may move it: what runs past its box,
    /// right and down.
    pub(in crate::tree) fn extent(&self, e: u32) -> [f32; 2] {
        let node = self.els[e as usize].node;
        let b = self.arena.box_of(node);
        let (mut right, mut bottom) = (0.0f32, 0.0f32);
        for &c in self.arena.children(node) {
            // What stays put (a built-in scroll bar) runs over nothing.
            if self.pinned_node(c) {
                continue;
            }
            let k = self.arena.box_of(c);
            right = right.max(k.x + k.w);
            bottom = bottom.max(k.y + k.h);
        }
        let [x, y] = self.scroll_axes(e);
        [
            if x {
                (right + b.inset[2] - b.w).max(0.0)
            } else {
                0.0
            },
            if y {
                (bottom + b.inset[3] - b.h).max(0.0)
            } else {
                0.0
            },
        ]
    }

    /// The axes op element `e` scrolls (x, y): both for a `Scroll`, else
    /// those its `overflow` is `auto` or `scroll` along, as in CSS.
    pub(in crate::tree) fn scroll_axes(&self, e: u32) -> [bool; 2] {
        let Some(p) = self.input.per(e) else {
            return [false; 2];
        };
        if self.els[e as usize].dead {
            return [false; 2];
        }
        if p.scroller {
            return [true; 2];
        }
        let o = taffy::CoreStyle::overflow(&self.els[e as usize].record.layout);
        [o.x, o.y].map(|o| o == taffy::Overflow::Scroll)
    }

    pub(super) fn scroller(&self, e: u32) -> bool {
        self.scroll_axes(e) != [false; 2]
    }

    /// A scroller's axis: horizontal when only its width runs over.
    pub(super) fn axis(&self, e: u32) -> Option<bool> {
        if !self.scroller(e) {
            return None;
        }
        let [w, h] = self.extent(e);
        Some(h <= 0.0 && w > 0.0)
    }

    /// How far op element `e`'s children are moved: its scroll.
    pub(in crate::tree) fn scroll_of(&self, e: u32) -> [f32; 2] {
        match e {
            NONE => [0.0; 2],
            e => self.input.per(e).map_or([0.0; 2], |p| p.scroll),
        }
    }

    /// Whether arena node `n` is a `pinned` element's.
    fn pinned_node(&self, n: Id) -> bool {
        self.input.pinned.contains(&n)
    }

    /// How far op element `e` is moved by the scroller it is in: its
    /// parent's scroll, none when it is `pinned`.
    pub(in crate::tree) fn moved_by(&self, e: u32) -> [f32; 2] {
        if self.input.per(e).is_some_and(|p| p.pinned) {
            return [0.0; 2];
        }
        self.scroll_of(self.els[e as usize].arena_parent)
    }

    /// Op element `e`'s scroll offset along `horizontal` (else down).
    pub(in crate::tree) fn scroll_at(&self, e: u32, horizontal: bool) -> f32 {
        self.scroll_of(e)[usize::from(!horizontal)]
    }

    /// Sets scroller `e`'s offset along an axis, within its extent;
    /// whether it moved.
    pub(in crate::tree) fn scroll_to(&mut self, e: u32, horizontal: bool, to: f32) -> bool {
        let k = usize::from(!horizontal);
        let to = to.clamp(0.0, self.extent(e)[k]);
        let p = &mut self.input.per[e as usize];
        if p.scroll[k] == to {
            return false;
        }
        p.scroll[k] = to;
        self.paint = true;
        if !self.input.scrolled.contains(&e) {
            self.input.scrolled.push(e);
        }
        true
    }

    /// After a layout: each scroller's offset stays within its extent.
    pub(in crate::tree) fn clamp_scrolls(&mut self) {
        for e in 0..self.input.per.len() as u32 {
            if self.scroller(e) {
                let [x, y] = self.scroll_of(e);
                self.scroll_to(e, true, x);
                self.scroll_to(e, false, y);
            }
        }
        self.note_resized();
    }

    /// A wheel by (dx, dy) at (x, y). The nearest op element under it
    /// that listens hears it, with the deltas in `x` and `y`, and takes it
    /// unless a scroller is nearer (NSH's `wheel_listener`); else the
    /// innermost scroller along the wheel's axis that can move scrolls
    /// (NSH's `scroll_target`), and notches that keep coming stay on it
    /// (NSH's `latch_holds`).
    pub fn wheel(&mut self, x: f32, y: f32, dx: f32, dy: f32) {
        if dx == 0.0 && dy == 0.0 {
            return;
        }
        self.chain_at(x, y);
        let chain = std::mem::take(&mut self.input.chain);
        let heard = nsh::wheel_listener(
            chain.iter().copied(),
            |&e| self.els[e as usize].listens & OpEvent::Wheel.bit() != 0,
            |&e| self.scroller(e),
        );
        if let Some((l, _)) = heard {
            self.fire(l, OpEvent::Wheel, dx, dy);
        }
        let taken = heard.is_some_and(|(_, takes)| takes);
        let horizontal = dx.abs() > dy.abs();
        let delta = |axis_h: bool| if axis_h && dx != 0.0 { dx } else { dy };
        let now = self.time * 1000.0;
        let latched = self
            .input
            .latch
            .filter(|&(e, h, last)| self.scroller(e) && nsh::latch_holds(h, last, horizontal, now));
        let target = match latched {
            Some((e, ..)) => Some(e),
            None if taken => None,
            None => nsh::scroll_target(
                chain
                    .iter()
                    .copied()
                    .filter(|&e| self.scroller(e))
                    .collect(),
                nsh_input::Points(dx),
                nsh_input::Points(dy),
                |&e| self.axis(e),
                |&e| {
                    let h = self.axis(e) == Some(true);
                    let (at, max) = (self.scroll_at(e, h), self.extent(e)[usize::from(!h)]);
                    let d = delta(h);
                    (d > 0.0 && at < max) || (d < 0.0 && at > 0.0)
                },
            ),
        };
        self.input.chain = chain;
        if let Some(e) = target {
            let h = self.axis(e) == Some(true);
            self.scroll_to(e, h, self.scroll_at(e, h) + delta(h));
            self.input.fling = None;
            self.input.latch = Some((e, horizontal, now));
        }
    }

    /// Scrolls the scrollers around op element `e` so its box shows, as
    /// focus moved by the keys does (CSS's `scrollIntoView` nearest).
    pub(super) fn reveal(&mut self, e: u32) {
        let Some(&(mut ex, mut ey, ..)) = self.abs.get(e as usize) else {
            return;
        };
        let b = self.arena.box_of(self.els[e as usize].node);
        let mut s = self.els[e as usize].parent;
        while s != NONE {
            if self.scroller(s)
                && let Some(&(sx, sy, ..)) = self.abs.get(s as usize)
            {
                let sb = self.arena.box_of(self.els[s as usize].node);
                for (h, at, size, from, span) in
                    [(true, ex, b.w, sx, sb.w), (false, ey, b.h, sy, sb.h)]
                {
                    let by = if at < from {
                        at - from
                    } else if at + size > from + span {
                        (at + size - from - span).min(at - from)
                    } else {
                        0.0
                    };
                    let old = self.scroll_at(s, h);
                    if by != 0.0 && self.scroll_to(s, h, old + by) {
                        let moved = self.scroll_at(s, h) - old;
                        if h {
                            ex -= moved;
                        } else {
                            ey -= moved;
                        }
                    }
                }
            }
            s = self.els[s as usize].parent;
        }
    }
}
