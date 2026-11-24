//! Pointer hit testing, touch target expansion, and screen-space transform
//! helpers.

use super::{El, NONE, Tree};

/// The side, CSS px, a control's box is grown to for a finger on a touch
/// screen ([`Tree::near_target`]): WCAG 2.5.8's enhanced target, Apple's
/// 44 pt and the design's `--size-touch`.
pub(crate) const TOUCH_TARGET: f32 = 44.0;

impl Tree {
    /// The element a press at (x, y) reaches: of the shown, enabled
    /// elements that answer a press and whose box, inside the clip their
    /// parent paints them in, holds the point, the last painted in its
    /// stacking context; NONE for none.
    pub(super) fn hit(&self, x: f32, y: f32) -> u32 {
        let e = self.hit_where(x, y, |el| el.clickable);
        if e != NONE || !self.input.touch {
            return e;
        }
        self.near_target(x, y)
    }

    /// Inverse of element `e`'s last painted 2D affine transform, mapping
    /// surface coordinates `(x, y)` back into `e`'s pre-transform space.
    /// Returns `None` if the transform is singular (zero scale / collapsed).
    pub(super) fn untransform(&self, e: u32, x: f32, y: f32) -> Option<(f32, f32)> {
        let xf = self.xfs.get(e as usize).copied().unwrap_or(0);
        if xf == 0 {
            return Some((x, y));
        }
        let Some(&[a, b, c, d, tx, ty, _, _]) = self.painted_transforms.get(xf as usize) else {
            return Some((x, y));
        };
        let det = a * d - b * c;
        if det.abs() < 1e-6 {
            return None;
        }
        let (dx, dy) = (x - tx, y - ty);
        Some(((d * dx - c * dy) / det, (a * dy - b * dx) / det))
    }

    /// Axis-aligned screen bounds `[x0, y0, x1, y1]` of element `e`'s box
    /// `(x, y, w, h)` after its painted 2D affine transform.
    pub(super) fn screen_bounds(&self, e: u32, x: f32, y: f32, w: f32, h: f32) -> [f32; 4] {
        let xf = self.xfs.get(e as usize).copied().unwrap_or(0);
        if xf == 0 {
            return [x, y, x + w, y + h];
        }
        let Some(&[a, b, c, d, tx, ty, _, _]) = self.painted_transforms.get(xf as usize) else {
            return [x, y, x + w, y + h];
        };
        let mut out = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
        for [cx, cy] in [[x, y], [x + w, y], [x, y + h], [x + w, y + h]] {
            let (sx, sy) = (a * cx + c * cy + tx, b * cx + d * cy + ty);
            out = [
                out[0].min(sx),
                out[1].min(sy),
                out[2].max(sx),
                out[3].max(sy),
            ];
        }
        out
    }

    /// On a touch screen, a press that lands on no control goes to the
    /// nearest enabled one whose box, grown to [`TOUCH_TARGET`] square
    /// round its centre, holds the point (Material's minimum interactive
    /// size, Apple's 44 pt): a small control keeps its look and still
    /// takes a finger. The nearest box wins; at a tie, the one on top.
    fn near_target(&self, x: f32, y: f32) -> u32 {
        let (mut best, mut far) = (NONE, f32::MAX);
        for &e in self.stacking.order.iter().rev() {
            let (e, el) = (e as usize, &self.els[e as usize]);
            if !el.clickable || el.disabled || el.dead {
                continue;
            }
            if self.shown.get(e).is_some_and(|s| s.out || !s.touch) {
                continue;
            }
            let Some(&(bx, by, shown, _)) = self.abs.get(e) else {
                continue;
            };
            // The clip its parent paints it in, as `topmost` reads it.
            let all = [f32::MIN, f32::MIN, f32::MAX, f32::MAX];
            let clip = match el.arena_parent {
                _ if el.anchored().is_some() => all,
                NONE => all,
                p => self.abs.get(p as usize).map_or(all, |a| a.3),
            };
            if !shown || self.in_hole(e as u32, x, y).is_some() || !self.reachable(e as u32) {
                continue;
            }
            let b = self
                .motion
                .box_of(el.node)
                .unwrap_or_else(|| self.arena.box_of(el.node));
            let [sx0, sy0, sx1, sy1] = self.screen_bounds(e as u32, bx, by, b.w, b.h);
            // What of it is drawn: a control scrolled out of view takes nothing.
            let [x0, y0, x1, y1] = [
                sx0.max(clip[0]),
                sy0.max(clip[1]),
                sx1.min(clip[2]),
                sy1.min(clip[3]),
            ];
            if x1 <= x0 || y1 <= y0 {
                continue;
            }
            let gx = ((TOUCH_TARGET - (x1 - x0)) / 2.0).max(0.0);
            let gy = ((TOUCH_TARGET - (y1 - y0)) / 2.0).max(0.0);
            if x < x0 - gx || x >= x1 + gx || y < y0 - gy || y >= y1 + gy {
                continue;
            }
            let (dx, dy) = ((x0 - x).max(x - x1).max(0.0), (y0 - y).max(y - y1).max(0.0));
            let d = dx * dx + dy * dy;
            if d < far {
                (best, far) = (e as u32, d);
            }
        }
        best
    }

    /// [`Tree::hit`] over the elements `want` accepts; a press needs them
    /// enabled too. Behind an open modal, nothing answers.
    pub(super) fn hit_where(&self, x: f32, y: f32, want: impl Fn(&El) -> bool) -> u32 {
        self.topmost(x, y, want, true)
    }

    /// The topmost shown element at (x, y) that `want` accepts; a disabled
    /// control counts only when `enabled` is false (its cursor shows).
    pub(super) fn topmost(&self, x: f32, y: f32, want: impl Fn(&El) -> bool, enabled: bool) -> u32 {
        for &e in self.stacking.order.iter().rev() {
            let (e, el) = (e as usize, &self.els[e as usize]);
            // An element on its way out, or in one, answers nothing.
            if el.dead || self.shown.get(e).is_some_and(|s| s.out || !s.touch) {
                continue;
            }
            let Some(&(bx, by, shown, _)) = self.abs.get(e) else {
                continue;
            };
            // A spotlight takes the surface outside its hole, whatever its
            // box: the lit node under it takes its own presses, and what
            // else is under it none.
            if let Some(in_hole) = self
                .in_hole(e as u32, x, y)
                .filter(|_| shown && self.reachable(e as u32))
            {
                if in_hole {
                    continue;
                }
                let takes = want(el) && !(enabled && el.clickable && el.disabled);
                return if takes { e as u32 } else { NONE };
            }
            if !want(el) {
                continue;
            }
            let all = [f32::MIN, f32::MIN, f32::MAX, f32::MAX];
            let [cx0, cy0, cx1, cy1] = match el.arena_parent {
                _ if el.anchored().is_some() => all,
                NONE => all,
                p => self.abs.get(p as usize).map_or(all, |a| a.3),
            };
            if x < cx0 || x >= cx1 || y < cy0 || y >= cy1 {
                continue;
            }
            let b = self
                .motion
                .box_of(el.node)
                .unwrap_or_else(|| self.arena.box_of(el.node));
            // A span in a paragraph is where its glyphs are.
            let inside = match self.on_span(e as u32, x, y) {
                Some(on) => on,
                None => match self.untransform(e as u32, x, y) {
                    Some((lx, ly)) => lx >= bx && lx < bx + b.w && ly >= by && ly < by + b.h,
                    None => false,
                },
            };
            if !shown || !inside || !self.reachable(e as u32) {
                continue;
            }
            if enabled && el.clickable && el.disabled {
                continue;
            }
            return e as u32;
        }
        NONE
    }

    /// The pointer this frame, at (x, y) on the tree's surface. The op
    /// nodes take it when it moved, a frame painted since (a box may have
    /// come under it), or a button changed.
    pub fn pointer(&mut self, x: f32, y: f32, pressed: bool, released: bool) {
        let was = std::mem::replace(&mut self.last_pointer, (x, y, self.paints));
        self.select_due(x, y);
        if self.has_ops() && (was != self.last_pointer || pressed || released) {
            let moved = (was.0, was.1) != (x, y);
            self.pointer_ops(x, y, moved, pressed, released);
            self.select_pointer(x, y, moved, pressed, released);
        }
    }
}
