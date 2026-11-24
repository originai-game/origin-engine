//! Drawn scroll bars (NSH base's `Scrollbar`): a node with
//! `thumb="<name>"` is the thumb of the scroller of that name, in a track
//! (its parent). The tree sizes it to the share of the content the
//! scroller shows and moves it by the scroller's offset every frame, with
//! no round trip through the game; a press on the track drags it, and the
//! scroller follows (CONVENTIONS.md §Scroll bars).

use super::ops::{Event as OpEvent, Fired};
use super::{NONE, Tree};
use crate::classes::state::PRESSED;

/// The shortest thumb, CSS px (a finger still finds it on a long list).
const MIN_THUMB: f32 = 24.0;

/// A thumb as laid out now: the scroller it follows, along which axis,
/// where its track starts on the surface and how long it is, its own
/// length and its offset in the track.
#[derive(Clone, Copy, Debug)]
pub(super) struct Thumb {
    scroller: u32,
    horizontal: bool,
    start: f32,
    track: f32,
    len: f32,
    offset: f32,
}

impl Tree {
    /// Thumb `t`'s geometry, when it names a live scroller and sits in a
    /// track.
    pub(super) fn thumb(&self, t: u32) -> Option<Thumb> {
        let p = self.input.per(t)?;
        if p.thumb.is_empty() || self.els[t as usize].dead {
            return None;
        }
        let track = self.els[t as usize].parent;
        if track == NONE {
            return None;
        }
        // `.` is the scroller its track sits in (a built-in bar).
        let s = if &*p.thumb == "." {
            Some(self.els[track as usize].parent).filter(|&s| s != NONE)
        } else {
            self.named(&p.thumb)
        }
        .filter(|&s| self.scroll_axes(s) != [false; 2])?;
        let tb = self.arena.box_of(self.els[track as usize].node);
        let horizontal = tb.w > tb.h;
        let k = usize::from(!horizontal);
        let (lead, trail) = if horizontal {
            (tb.inset[0], tb.inset[2])
        } else {
            (tb.inset[1], tb.inset[3])
        };
        let span = (if horizontal { tb.w } else { tb.h } - lead - trail).max(0.0);
        let at = self
            .abs
            .get(track as usize)
            .map_or(0.0, |a| if horizontal { a.0 } else { a.1 });
        let [vw, vh] = self.viewport(s);
        let view = if horizontal { vw } else { vh };
        let extent = self.extent(s)[k];
        let len = if extent <= 0.5 || view <= 0.0 {
            0.0
        } else {
            (span * view / (view + extent)).max(MIN_THUMB.min(span))
        };
        let scroll = self.scroll_of(s)[k];
        let offset = if extent > 0.0 {
            (span - len) * (scroll / extent).clamp(0.0, 1.0)
        } else {
            0.0
        };
        Some(Thumb {
            scroller: s,
            horizontal,
            start: at + lead,
            track: span,
            len,
            offset,
        })
    }

    /// After a layout: each thumb's length follows its scroller's share
    /// shown (none while nothing runs over); whether one changed, so the
    /// tree lays out again.
    pub(super) fn fit_thumbs(&mut self) -> bool {
        let els = &self.els;
        let per = &self.input.per;
        self.input.thumbs.retain(|&t| {
            !els[t as usize].dead && per.get(t as usize).is_some_and(|p| !p.thumb.is_empty())
        });
        let mut changed = false;
        for k in 0..self.input.thumbs.len() {
            let t = self.input.thumbs[k];
            let (len, horizontal) = self
                .thumb(t)
                .map_or((0.0, false), |g| (g.len, g.horizontal));
            let p = &mut self.input.per[t as usize];
            if (p.thumb_len - len).abs() > 0.5 || p.thumb_horizontal != horizontal {
                (p.thumb_len, p.thumb_horizontal) = (len, horizontal);
                self.inherit(t);
                changed = true;
            }
        }
        changed
    }

    /// How far thumb `e` is moved in its track at paint (x, y).
    pub(super) fn thumb_shift(&self, e: u32) -> [f32; 2] {
        if self.input.per(e).is_none_or(|p| p.thumb.is_empty()) {
            return [0.0; 2];
        }
        match self.thumb(e) {
            Some(g) if g.horizontal => [g.offset, 0.0],
            Some(g) => [0.0, g.offset],
            None => [0.0; 2],
        }
    }

    /// A press at (x, y) on a thumb's track grabs its thumb: where it was
    /// pressed on the thumb, or its middle when the press missed it (the
    /// thumb jumps there). Whether one took it.
    pub(super) fn grab_thumb(&mut self, x: f32, y: f32) -> bool {
        let mut e = self.hit_where(x, y, |_| true);
        while e != NONE {
            let found = self.input.thumbs.iter().copied().find(|&t| {
                self.els[t as usize].parent == e && self.input.per[t as usize].thumb_len > 0.0
            });
            if let Some(t) = found
                && let Some(g) = self.thumb(t)
            {
                let at = if g.horizontal { x } else { y } - g.start;
                let grip = if (g.offset..=g.offset + g.len).contains(&at) {
                    at - g.offset
                } else {
                    g.len / 2.0
                };
                self.input.grab = Some((t, grip));
                self.set_state(t, PRESSED, true);
                self.drag_thumb(x, y);
                return true;
            }
            e = self.els[e as usize].parent;
        }
        false
    }

    /// The grabbed thumb follows the pointer to (x, y), and its scroller
    /// scrolls to match. Whether a thumb is grabbed.
    pub(super) fn drag_thumb(&mut self, x: f32, y: f32) -> bool {
        let Some((t, grip)) = self.input.grab else {
            return false;
        };
        let Some(g) = self.thumb(t) else {
            self.input.grab = None;
            return false;
        };
        let room = g.track - g.len;
        if room > 0.0 {
            let at = if g.horizontal { x } else { y } - g.start - grip;
            let k = usize::from(!g.horizontal);
            let extent = self.extent(g.scroller)[k];
            self.scroll_to(g.scroller, g.horizontal, at / room * extent);
        }
        true
    }

    /// The pointer let a grabbed thumb go.
    pub(super) fn drop_thumb(&mut self) -> bool {
        match self.input.grab.take() {
            Some((t, _)) => {
                self.set_state(t, PRESSED, false);
                true
            }
            None => false,
        }
    }

    /// After a layout: a scroller that listens to `scroll` reports when
    /// its viewport or content changed size (as it first lays out too),
    /// so a virtual list knows how many rows fill it before it moves.
    pub(super) fn note_resized(&mut self) {
        for e in 0..self.input.per.len() as u32 {
            let el = &self.els[e as usize];
            if el.dead
                || el.listens & OpEvent::Scroll.bit() == 0
                || self.scroll_axes(e) == [false; 2]
            {
                continue;
            }
            let [vw, vh] = self.viewport(e);
            let [ex, ey] = self.extent(e);
            let now = [vw, vh, vw + ex, vh + ey];
            if self.input.per[e as usize].reported != now {
                self.input.per[e as usize].reported = now;
                if !self.input.scrolled.contains(&e) {
                    self.input.scrolled.push(e);
                }
            }
        }
    }

    /// What scroller `e` shows: its padding box (a browser's client
    /// size), which with its extent makes its content's size.
    fn viewport(&self, e: u32) -> [f32; 2] {
        let el = &self.els[e as usize];
        let b = self.arena.box_of(el.node);
        let [t, r, bo, l] = el.record.skin.border;
        [b.w - l - r, b.h - t - bo]
    }

    /// `scroll` on each scroller that moved since the events were last
    /// taken and listens, once: its offset in `x`, `y`, and `value`
    /// `"viewportWidth viewportHeight contentWidth contentHeight"`.
    pub(super) fn fire_scrolled(&mut self) {
        for e in std::mem::take(&mut self.input.scrolled) {
            let el = &self.els[e as usize];
            if el.dead || el.listens & OpEvent::Scroll.bit() == 0 {
                continue;
            }
            let [sx, sy] = self.scroll_of(e);
            let [vw, vh] = self.viewport(e);
            let [ex, ey] = self.extent(e);
            let mut f = Fired::at(el.js, OpEvent::Scroll, sx, sy);
            f.value = format!("{vw} {vh} {} {}", vw + ex, vh + ey).into();
            self.events.push(f);
        }
    }
}
