//! Scrolling both ways: offsets on either axis, the wheel and drags handed
//! from the innermost scroll container outwards when it reaches its end,
//! momentum after a drag (a fling that decays over `motion.spring.momentum`'s
//! response), and scrollbars a press can grab. Everything is by the tick and
//! in the UI state; the layout clamps the offsets to the content.

use crate::ui::{HIT_SCROLL, Ui};
use og_core::glam::Vec2;

/// How wide a scrollbar's grab strip is along a scroll container's edge, CSS px.
pub(crate) const BAR_GRAB: f32 = 12.0;
/// A fling stops below this speed, CSS px per tick.
const FLING_STOP: f32 = 0.1;

/// What the last frame knew about a scroll container: its range on each
/// axis and its scrollbars' lengths (0: no bar on that axis).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct ScrollInfo {
    pub range: Vec2,
    pub bar: Vec2,
}

/// A grabbed scrollbar: its container, the axis (0 x, 1 y) and where on
/// the bar the press landed.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Thumb {
    pub key: u64,
    pub axis: u8,
    pub grab: f32,
}

impl Ui {
    /// The offset of scroll container `key` (x, y).
    pub(crate) fn offset(&self, key: u64) -> Vec2 {
        Vec2::new(
            self.state.scroll_x.get(&key).copied().unwrap_or(0.0),
            self.state.scroll.get(&key).copied().unwrap_or(0.0),
        )
    }

    /// Moves `key` by `d` as far as its range allows; answers what it could
    /// not take (for the container around it).
    pub(crate) fn scroll_by(&mut self, key: u64, d: Vec2) -> Vec2 {
        let info = self
            .state
            .scroll_info
            .get(&key)
            .copied()
            .unwrap_or_default();
        let at = self.offset(key);
        let to = (at + d).clamp(Vec2::ZERO, info.range.max(Vec2::ZERO));
        if to.x != at.x {
            self.state.scroll_x.insert(key, to.x);
        }
        if to.y != at.y {
            self.state.scroll.insert(key, to.y);
        }
        d - (to - at)
    }

    /// `key` and the scroll containers around it, innermost first.
    fn scroll_chain(&self, key: u64) -> Vec<u64> {
        let mut out = Vec::new();
        let mut k = key;
        while k != 0 && out.len() < 32 {
            out.push(k);
            k = self.hit(k).map_or(0, |h| h.scroller);
        }
        out
    }

    /// Moves `d` through `key`'s chain: what one container cannot take
    /// goes to the one around it (nested scroll handoff).
    fn scroll_chain_by(&mut self, key: u64, d: Vec2) -> Vec2 {
        let mut rest = d;
        for k in self.scroll_chain(key) {
            if rest == Vec2::ZERO {
                break;
            }
            rest = self.scroll_by(k, rest);
        }
        rest
    }

    /// The scrollbar under `p`, if a press there grabs one.
    fn thumb_at(&self, p: Vec2) -> Option<Thumb> {
        self.state.hits.iter().rev().find_map(|h| {
            if !h.has(HIT_SCROLL) || !h.rect.contains(p.x, p.y) {
                return None;
            }
            let info = self.state.scroll_info.get(&h.key)?;
            let r = h.full;
            let off = self.offset(h.key);
            if info.range.y > 0.0 && p.x >= h.rect.right() - BAR_GRAB {
                let start = bar_start(r.y, r.height, info.bar.y, off.y, info.range.y);
                let grab = (p.y - start).clamp(0.0, info.bar.y);
                return Some(Thumb {
                    key: h.key,
                    axis: 1,
                    grab,
                });
            }
            if info.range.x > 0.0 && p.y >= h.rect.bottom() - BAR_GRAB {
                let start = bar_start(r.x, r.width, info.bar.x, off.x, info.range.x);
                let grab = (p.x - start).clamp(0.0, info.bar.x);
                return Some(Thumb {
                    key: h.key,
                    axis: 0,
                    grab,
                });
            }
            None
        })
    }

    /// This tick's scrolling: a grabbed bar, the wheel, a drag (with its
    /// velocity) and a fling, before interaction is decided. Answers the
    /// scroll container a press on a scrollbar grabbed (it holds the press).
    pub(crate) fn scroll_input(&mut self, input: &crate::input::InputFrame) -> Option<u64> {
        let p = input.pointer;
        let mut grabbed = None;
        if input.pressed {
            // A press stops a fling; one on a scrollbar grabs it.
            self.state.fling = None;
            self.state.thumb = input.pointer_inside.then(|| self.thumb_at(p)).flatten();
            grabbed = self.state.thumb.map(|t| t.key);
            self.state.drag_v = Vec2::ZERO;
        }
        if let Some(t) = self.state.thumb {
            if input.down || input.pressed {
                if let (Some(h), Some(info)) = (
                    self.hit(t.key).copied(),
                    self.state.scroll_info.get(&t.key).copied(),
                ) {
                    let (start, len, bar, range, at) = if t.axis == 1 {
                        (h.full.y, h.full.height, info.bar.y, info.range.y, p.y)
                    } else {
                        (h.full.x, h.full.width, info.bar.x, info.range.x, p.x)
                    };
                    let travel = (len - bar).max(1.0);
                    let v = ((at - t.grab - start) / travel).clamp(0.0, 1.0) * range;
                    if t.axis == 1 {
                        self.state.scroll.insert(t.key, v);
                    } else {
                        self.state.scroll_x.insert(t.key, v);
                    }
                }
            } else {
                self.state.thumb = None;
            }
            return grabbed;
        }
        // The wheel: the innermost scroll container under the pointer first.
        if input.wheel != Vec2::ZERO && input.pointer_inside {
            self.state.fling = None;
            if let Some(h) = self
                .state
                .hits
                .iter()
                .rev()
                .find(|h| h.has(HIT_SCROLL) && h.rect.contains(p.x, p.y))
            {
                let key = h.key;
                self.scroll_chain_by(key, input.wheel);
            }
        }
        // A drag on a container that holds the press, with its velocity kept.
        let active = self.state.active;
        if self.hit(active).is_some_and(|h| h.has(HIT_SCROLL)) {
            if input.down && !input.pressed {
                let d = self.state.pointer - p;
                self.scroll_chain_by(active, d);
                self.state.drag_v = self.state.drag_v * 0.5 + d * 0.5;
            } else if input.released {
                let v = self.state.drag_v;
                if !self.theme().reduced_motion && v.length() > 1.0 {
                    self.state.fling = Some((active, v));
                }
                self.state.drag_v = Vec2::ZERO;
            }
        }
        // A fling: the released drag's velocity, decaying to rest; it stops
        // at the content's end on each axis.
        if let Some((key, v)) = self.state.fling {
            if self.hit(key).is_none() {
                self.state.fling = None;
            } else {
                let left = self.scroll_by(key, v);
                let response = (crate::theme::SPRING_MOMENTUM.response_ms / 1000.0).max(1e-3);
                let decay = (1.0 - 1.0 / (self.ticks_per_second() as f32 * response)).max(0.0);
                let mut next = v * decay;
                if left.x != 0.0 {
                    next.x = 0.0;
                }
                if left.y != 0.0 {
                    next.y = 0.0;
                }
                self.state.fling = (next.length() >= FLING_STOP).then_some((key, next));
            }
        }
        grabbed
    }
}

/// Where a scrollbar of length `bar` starts on a track at `start`, `len`
/// long, for `offset` of `range`.
pub(crate) fn bar_start(start: f32, len: f32, bar: f32, offset: f32, range: f32) -> f32 {
    if range <= 0.0 {
        return start;
    }
    start + (len - bar) * (offset / range).clamp(0.0, 1.0)
}

/// A scrollbar's length on a track `len` long showing `inner` of `content`.
pub(crate) fn bar_len(len: f32, inner: f32, content: f32, min: f32) -> f32 {
    if content <= 0.0 {
        return len;
    }
    (len * inner / content).max(min).min(len)
}
