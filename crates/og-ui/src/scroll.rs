//! Scrolling both ways: offsets on either axis, the wheel and drags handed
//! from the innermost scroll container outwards when it reaches its end,
//! momentum after a drag (a fling that decays over `motion.spring.momentum`'s
//! response), and scrollbars a press can grab. Everything is by the tick and
//! in the UI state; the layout clamps the offsets to the content.
//!
//! Two touches (og-input's `og.Pointers`): while two fingers are held on
//! one scroll container or zoom view (the innermost under each), it
//! follows them: a container scrolls by the centre's movement (two-finger
//! scroll, which a list of buttons needs: one finger presses a row), a zoom
//! view ([`Ui::zoom_view`]) pans by it and zooms by the change of their
//! spread about the centre (pinch). The press the first finger began
//! presses nothing once the second lands.

use crate::id::child_key;
use crate::ui::{HIT_OFFSCREEN, HIT_SCROLL, Ui};
use og_core::glam::Vec2;

/// Two touches moving something: the scroll container or zoom view under
/// their centre when the second landed (0: none), and their centre and
/// spread (the distance between them) last tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Pinch {
    pub key: u64,
    pub center: Vec2,
    pub spread: f32,
}

/// A zoom view's state: its zoom and the centre of what it shows, as a
/// fraction of its picture (0.5, 0.5: the middle), and its zoom limits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ZoomState {
    pub zoom: f32,
    pub center: Vec2,
    pub min: f32,
    pub max: f32,
}

fn zoom_key(key: u64) -> u64 {
    child_key(key, "\u{1}zoom")
}

fn pan_key(key: u64) -> u64 {
    child_key(key, "\u{1}pan")
}

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
    /// The zoom view `key`'s state, when `key` is one.
    pub(crate) fn zoom_state(&self, key: u64) -> Option<ZoomState> {
        let z = self.state.anims.get(&zoom_key(key))?;
        let p = self
            .state
            .anims
            .get(&pan_key(key))
            .copied()
            .unwrap_or_default();
        Some(ZoomState {
            zoom: z.a,
            center: Vec2::new(p.a, p.b),
            min: f32::from_bits(z.t as u32),
            max: f32::from_bits(z.h as u32),
        })
    }

    /// Stores a zoom view's state (marking it used this frame), clamped:
    /// the zoom to its limits, the centre so the view stays on the picture.
    pub(crate) fn set_zoom_state(&mut self, key: u64, s: ZoomState) {
        let zoom = s.zoom.clamp(s.min, s.max);
        let half = 0.5 / zoom;
        let c = Vec2::new(
            s.center.x.clamp(half.min(0.5), (1.0 - half).max(0.5)),
            s.center.y.clamp(half.min(0.5), (1.0 - half).max(0.5)),
        );
        self.set_anim(
            zoom_key(key),
            crate::ui::Anim {
                a: zoom,
                b: 0.0,
                t: u64::from(s.min.to_bits()),
                h: u64::from(s.max.to_bits()),
            },
        );
        self.set_anim(
            pan_key(key),
            crate::ui::Anim {
                a: c.x,
                b: c.y,
                t: 0,
                h: 0,
            },
        );
    }

    /// Zooms view `key` by `k` about the point `at` (CSS px) and pans it by
    /// `moved` (CSS px, the picture following the pointer).
    fn zoom_by(&mut self, key: u64, k: f32, at: Vec2, moved: Vec2) {
        let (Some(mut s), Some(h)) = (self.zoom_state(key), self.hit(key).copied()) else {
            return;
        };
        let r = h.full;
        if r.width <= 0.0 || r.height <= 0.0 {
            return;
        }
        let size = Vec2::new(r.width, r.height);
        // The picture point under `at` stays under it.
        let f = (at - Vec2::new(r.x, r.y)) / size - Vec2::splat(0.5);
        let to = (s.zoom * if k.is_finite() && k > 0.0 { k } else { 1.0 }).clamp(s.min, s.max);
        s.center += f / s.zoom - f / to;
        s.zoom = to;
        s.center -= moved / (size * s.zoom);
        self.set_zoom_state(key, s);
    }

    /// Two touches: track them, and move what is under them. Answers
    /// whether a two-finger gesture holds the pointer this tick.
    fn two_touches(&mut self, input: &crate::input::InputFrame) -> bool {
        let p = &input.pointers;
        let held: Vec<u32> = (0..og_input::MAX_POINTERS as u32)
            .filter(|&id| p.down(id))
            .collect();
        if held.len() < 2 {
            self.state.pinch = None;
            return false;
        }
        let (a, b) = (p.position(held[0]), p.position(held[1]));
        let center = (a + b) / 2.0;
        let spread = a.distance(b);
        match self.state.pinch {
            Some(last) => {
                let moved = center - last.center;
                if last.key != 0 {
                    if self.zoom_state(last.key).is_some() {
                        let k = if last.spread > 1.0 {
                            spread / last.spread
                        } else {
                            1.0
                        };
                        self.zoom_by(last.key, k, center, moved);
                    } else {
                        self.scroll_chain_by(last.key, -moved);
                    }
                }
                self.state.pinch = Some(Pinch {
                    key: last.key,
                    center,
                    spread,
                });
            }
            None => {
                // Both touches on one scroller or zoom view (the innermost
                // under each): anything else (a stick and a button, two
                // controls) is not a gesture.
                let under = |p: Vec2| {
                    self.state
                        .hits
                        .iter()
                        .rev()
                        .find(|h| {
                            h.has(HIT_SCROLL) && !h.has(HIT_OFFSCREEN) && h.rect.contains(p.x, p.y)
                        })
                        .map_or(0, |h| h.key)
                };
                let key = under(a);
                if key == 0 || under(b) != key {
                    return false;
                }
                self.state.pinch = Some(Pinch {
                    key,
                    center,
                    spread,
                });
                // What the first finger pressed is not pressed.
                self.state.active = 0;
                self.state.fling = None;
                self.state.thumb = None;
                self.state.drag_v = Vec2::ZERO;
            }
        }
        true
    }

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
        self.gesture = self.two_touches(input);
        if self.gesture {
            return None;
        }
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
                if self.zoom_state(key).is_some() {
                    // A zoom view zooms about the pointer: a notch (100 px
                    // of wheel) up is about a seventh in.
                    let k = 0.9985f32.powf(input.wheel.y);
                    self.zoom_by(key, k, p, Vec2::ZERO);
                } else {
                    self.scroll_chain_by(key, input.wheel);
                }
            }
        }
        // A drag on a container that holds the press, with its velocity kept.
        let active = self.state.active;
        if self.zoom_state(active).is_some() {
            // A drag on a zoom view pans it.
            if input.down && !input.pressed {
                let moved = p - self.state.pointer;
                self.zoom_by(active, 1.0, p, moved);
            }
        } else if self.hit(active).is_some_and(|h| h.has(HIT_SCROLL)) {
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
