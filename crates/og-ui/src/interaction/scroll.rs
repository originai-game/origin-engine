//! Scrolling both ways: offsets on either axis, the wheel and drags handed
//! from the innermost scroll container outwards when it reaches its end,
//! momentum after a drag (a fling that decays over `motion.spring.momentum`'s
//! response), and scrollbars a press can grab. Everything is by the tick and
//! in the UI state; the layout clamps the offsets to the content.
//!
//! Two touches (og-input's `og.Pointers`): while two fingers are held on
//! one scroll container (the innermost under each), it scrolls by their
//! centre's movement (two-finger scroll, which a list of buttons needs:
//! one finger presses a row). The press the first finger began presses
//! nothing once the second lands. Two fingers lifting off a container
//! fling it with their last velocity, as one finger does. A picture that
//! zooms is a document's: its `onpinch`, `onwheel` and `onpointermove`
//! with CSS `scale` and `translate`.

use crate::ui::{HIT_OFFSCREEN, HIT_SCROLL, Ui};
use og_core::glam::Vec2;

/// Two touches scrolling something: the scroll container under their
/// centre when the second landed (0: none), and their centre last tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pinch {
    pub key: u64,
    pub center: Vec2,
}

/// How wide a scrollbar's grab strip is along a scroll container's edge, CSS px.
pub(crate) const BAR_GRAB: f32 = 12.0;
/// A fling stops below this speed, CSS px per tick.
const FLING_STOP: f32 = 0.1;

/// What the last frame knew about a scroll container: its range on each
/// axis and its scrollbars' lengths (0: no bar on that axis).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScrollInfo {
    pub range: Vec2,
    pub bar: Vec2,
}

/// A scroll container as the last frame laid it out ([`Ui::scroll_state`]).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScrollState {
    /// How far its content is scrolled right.
    pub offset_x: f32,
    /// How far its content is scrolled down.
    pub offset_y: f32,
    /// Its view's width, inside its padding.
    pub view_width: f32,
    /// Its view's height, inside its padding.
    pub view_height: f32,
    /// Its whole content's width (the view's on an axis it does not scroll).
    pub content_width: f32,
    /// Its whole content's height.
    pub content_height: f32,
}

/// CSS `overscroll-behavior` on one axis (CSS Overscroll 1 §2).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Overscroll {
    /// What it cannot take goes on to the scroll container around it.
    #[default]
    Auto,
    /// Scroll chaining stops at it (og-ui draws no overscroll affordance,
    /// so `none` is the same).
    Contain,
    None,
}

/// An element's CSS scroll and scrollbar properties beyond `overflow` and
/// snap's type and alignment ([`crate::css::scroll_css`]).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScrollCss {
    /// `overscroll-behavior-x` and `-y` of a scroll container.
    pub overscroll: [Overscroll; 2],
    /// `scroll-behavior: smooth`: a scroll into view (a focus move, a pad's
    /// step, `og.ui.scrollIntoView`) glides there instead of jumping.
    pub smooth: bool,
    /// `scroll-snap-stop: always` on a snap area: a fling does not pass it.
    pub snap_stop: bool,
    /// `scroll-margin` (top, right, bottom, left, CSS px): the area a scroll
    /// into view or a snap position lines up is its box grown by these.
    pub margin: [f32; 4],
    /// `scroll-padding` of a scroll container (top, right, bottom, left):
    /// each side's px and fraction of its view, inset from the view where
    /// a scroll into view or snapping lines things up.
    pub padding: [(f32, f32); 4],
    /// `scrollbar-width: thin`.
    pub thin: bool,
    /// `scrollbar-color`: the thumb's and the track's (`None`: the style's
    /// thumb and no track).
    pub colors: Option<[crate::Srgba; 2]>,
    /// `scrollbar-gutter`: 0 auto, 1 stable, 2 stable both-edges.
    pub gutter: u8,
    /// `scroll-axis-lock: none` (CSS Overflow 5 §4): a drag on it is never
    /// locked to one axis.
    pub free_axes: bool,
    /// `overflow-anchor: none` (CSS Scroll Anchoring 1 §3): a scroll
    /// container on it anchors nothing, and it and its subtree are never
    /// an anchor.
    pub no_anchor: bool,
}

impl ScrollCss {
    /// Its scroll padding in px for a view `w` × `h` (top, right, bottom,
    /// left).
    pub(crate) fn padding_px(&self, w: f32, h: f32) -> [f32; 4] {
        let p = self.padding;
        [
            p[0].0 + p[0].1 * h,
            p[1].0 + p[1].1 * w,
            p[2].0 + p[2].1 * h,
            p[3].0 + p[3].1 * w,
        ]
    }
}

/// A grabbed scrollbar: its container, the axis (0 x, 1 y) and where on
/// the bar the press landed.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Thumb {
    pub key: u64,
    pub axis: u8,
    pub grab: f32,
}

impl Ui {
    /// Two touches: track them, and move what is under them. Answers
    /// whether a two-finger gesture holds the pointer this tick.
    fn two_touches(&mut self, input: &crate::input::InputFrame) -> bool {
        let p = &input.pointers;
        let held: Vec<u32> = (0..og_input::MAX_POINTERS as u32)
            .filter(|&id| p.down(id))
            .collect();
        if held.len() < 2 {
            // Two fingers that scrolled a container fling it as they lift,
            // as one finger's drag does.
            if let Some(last) = self.interaction.state.pinch.take()
                && last.key != 0
            {
                let v = self.interaction.state.drag_v;
                if !self.theme().reduced_motion && v.length() > 1.0 {
                    self.interaction.state.fling = Some((last.key, v));
                }
                self.interaction.state.drag_v = Vec2::ZERO;
            }
            return false;
        }
        let (a, b) = (p.position(held[0]), p.position(held[1]));
        let center = (a + b) / 2.0;
        match self.interaction.state.pinch {
            Some(last) => {
                let moved = center - last.center;
                if last.key != 0 {
                    self.tree
                        .scroll_chain_by(last.key, -moved, &mut self.interaction);
                    // Its velocity, kept for the fling as they lift.
                    self.interaction.state.drag_v =
                        self.interaction.state.drag_v * 0.5 - moved * 0.5;
                }
                self.interaction.state.pinch = Some(Pinch {
                    key: last.key,
                    center,
                });
            }
            None => {
                // Both touches on one scroller (the innermost under each):
                // anything else (a stick and a button, two controls) is
                // not a gesture.
                let under = |p: Vec2| {
                    self.interaction
                        .state
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
                self.interaction.state.pinch = Some(Pinch { key, center });
                // What the first finger pressed is not pressed.
                self.interaction.state.active = 0;
                self.interaction.state.fling = None;
                self.interaction.state.thumb = None;
                self.interaction.state.drag_v = Vec2::ZERO;
            }
        }
        true
    }

    /// Scroll container `path` (a full path, as the probe and
    /// [`Ui::last_path`] name it) in the last frame: where its content stood
    /// (this tick's offset, clamped to last frame's range), its view and its
    /// whole content, unclipped. What a component virtualises its rows by
    /// (a row scrolled out of view has no bounds to place them from).
    /// `None` for a path that was no scroll container.
    pub fn scroll_state(&self, path: &str) -> Option<ScrollState> {
        let key = crate::id::key_of_path(path);
        let n = if self.tree.building {
            self.tree
                .previous_nodes()
                .find(|n| n.key == key && n.scrolls())
        } else {
            self.tree
                .index
                .find(key)
                .map(|i| &self.tree.nodes[i])
                .filter(|n| n.key == key && n.scrolls())
        }?;
        let view_width = (n.layout.w - n.layout.used.padding.horizontal()).max(0.0);
        let view_height = (n.layout.h - n.layout.used.padding.vertical()).max(0.0);
        let at = self.interaction.offset(key);
        Some(ScrollState {
            offset_x: at.x.clamp(0.0, n.max_scroll_x()),
            offset_y: at.y.clamp(0.0, n.max_scroll()),
            view_width,
            view_height,
            content_width: if n.layout.used.scroll_x {
                n.layout.content_w
            } else {
                view_width
            },
            content_height: if n.layout.used.scroll {
                n.layout.content_h
            } else {
                view_height
            },
        })
    }

    /// CSS Overflow 5 §4 `scroll-axis-lock`: a drag on container `key`,
    /// when it can scroll both ways, keeps to the axis its first move runs
    /// along (within a 2:1 slope of it) until the press ends, the other
    /// axis's delta dropped; a diagonal first move, `scroll-axis-lock:
    /// none` or a container with room on one axis leaves it free.
    fn axis_lock(&mut self, key: u64, d: Vec2) -> Vec2 {
        if self.interaction.state.drag_axis == 0 && d != Vec2::ZERO {
            let both = self
                .interaction
                .state
                .scroll_info
                .get(&key)
                .is_some_and(|i| i.range.x > 0.0 && i.range.y > 0.0);
            let free = !both || self.tree.scroll_css(key).free_axes;
            self.interaction.state.drag_axis = if free {
                3
            } else if d.x.abs() >= 2.0 * d.y.abs() {
                1
            } else if d.y.abs() >= 2.0 * d.x.abs() {
                2
            } else {
                3
            };
        }
        match self.interaction.state.drag_axis {
            1 => Vec2::new(d.x, 0.0),
            2 => Vec2::new(0.0, d.y),
            _ => d,
        }
    }

    /// The fraction of a fling's velocity a tick keeps off (1 − its decay).
    pub(crate) fn fling_keep(&self) -> f32 {
        let response = (crate::theme::SPRING_MOMENTUM.response_ms / 1000.0).max(1e-3);
        let decay = (1.0 - 1.0 / (self.ticks_per_second() as f32 * response)).max(0.0);
        1.0 - decay
    }

    /// Scrolls container `key` to offset `to` (clamped to its range) as
    /// a program does (CSSOM View): with `scroll-behavior: smooth` a glide
    /// there along the momentum curve (a fling aimed at it, which lands on
    /// it exactly), else, or with reduced motion, at once. Stops a fling.
    pub(crate) fn scroll_to(&mut self, key: u64, to: Vec2) {
        let range = self
            .interaction
            .state
            .scroll_info
            .get(&key)
            .map_or(Vec2::ZERO, |i| i.range)
            .max(Vec2::ZERO);
        let at = self.interaction.offset(key);
        let to = Vec2::new(
            if range.x > 0.0 { to.x } else { at.x },
            if range.y > 0.0 { to.y } else { at.y },
        )
        .clamp(Vec2::ZERO, range.max(at));
        if self.interaction.state.fling.is_some_and(|(k, _)| k == key) {
            self.interaction.state.fling = None;
        }
        if self.tree.scroll_css(key).smooth && !self.theme().reduced_motion {
            if (to - at).length() > 0.25 {
                self.interaction.state.fling = Some((key, (to - at) * self.fling_keep()));
            }
            return;
        }
        if to.x != at.x {
            self.interaction.state.scroll_x.insert(key, to.x);
        }
        if to.y != at.y {
            self.interaction.state.scroll.insert(key, to.y);
        }
    }

    /// CSS Scroll Snap 2 §3, `scroll-initial-target: nearest` on element
    /// `key`, about to be built: its nearest scroll container, if that is
    /// new this frame (it was not laid out last frame and has no offset),
    /// starts scrolled to it; the first such target in tree order wins.
    pub(crate) fn note_initial_target(&mut self, key: u64) {
        let Some(&c) = self.tree.stack.iter().rev().find(|&&i| {
            let n = &self.tree.nodes[i];
            // This runs during the build, before used geometry is derived.
            n.el.scroll || n.el.scroll_x || n.layout.modal_room.is_some()
        }) else {
            return;
        };
        let container = self.tree.nodes[c].key;
        let s = &self.interaction.state;
        if s.scroll_info.contains_key(&container)
            || s.scroll.contains_key(&container)
            || s.scroll_x.contains_key(&container)
            || s.initial_targets.iter().any(|(k, _)| *k == container)
        {
            return;
        }
        self.interaction
            .state
            .initial_targets
            .push((container, key));
    }

    pub(crate) fn scroll_input(&mut self, input: &crate::input::InputFrame) -> Option<u64> {
        let p = input.pointer;
        self.interaction.gesture = self.two_touches(input);
        if self.interaction.gesture {
            return None;
        }
        let mut grabbed = None;
        if input.pressed {
            // A press stops a fling; one on a scrollbar grabs it.
            self.interaction.state.fling = None;
            self.interaction.state.wheel_from = None;
            self.interaction.state.thumb = input
                .pointer_inside
                .then(|| self.interaction.thumb_at(p, &self.tree))
                .flatten();
            grabbed = self.interaction.state.thumb.map(|t| t.key);
            self.interaction.state.drag_v = Vec2::ZERO;
            self.interaction.state.drag_axis = 0;
            // The element the touch landed on, whose touch-action the
            // drag keeps to wherever it moves.
            self.interaction.state.touch_key = if self.env.viewport.coarse_pointer {
                self.tree
                    .cursor_node(&self.tree.orders.draw, &self.paint, p)
                    .map_or(0, |i| self.tree.nodes[i].key)
            } else {
                0
            };
        }
        if let Some(t) = self.interaction.state.thumb {
            if input.down || input.pressed {
                if let (Some(h), Some(info)) = (
                    self.interaction.hit(t.key).copied(),
                    self.interaction.state.scroll_info.get(&t.key).copied(),
                ) {
                    let (start, len, bar, range, at) = if t.axis == 1 {
                        (h.full.y, h.full.height, info.bar.y, info.range.y, p.y)
                    } else {
                        (h.full.x, h.full.width, info.bar.x, info.range.x, p.x)
                    };
                    let travel = (len - bar).max(1.0);
                    let v = ((at - t.grab - start) / travel).clamp(0.0, 1.0) * range;
                    if t.axis == 1 {
                        self.interaction.state.scroll.insert(t.key, v);
                    } else {
                        self.interaction.state.scroll_x.insert(t.key, v);
                    }
                }
            } else {
                self.interaction.state.thumb = None;
            }
            return grabbed;
        }
        // The wheel: the innermost scroll container under the pointer first.
        // A snapping one settles once the wheel stops (the next tick, when
        // its boxes are laid out at the offset the wheel left).
        let mut wheel_snap = None;
        if input.wheel != Vec2::ZERO && input.pointer_inside {
            self.interaction.state.fling = None;
            if let Some(h) = self
                .interaction
                .state
                .hits
                .iter()
                .rev()
                .find(|h| h.has(HIT_SCROLL) && h.rect.contains(p.x, p.y))
            {
                let key = h.key;
                let from = self.interaction.offset(key);
                self.tree
                    .scroll_chain_by(key, input.wheel, &mut self.interaction);
                wheel_snap = self.tree.snaps(key).then_some(key);
                if wheel_snap.is_some()
                    && self.interaction.state.wheel_from.is_none_or(|w| w.0 != key)
                {
                    self.interaction.state.wheel_from = Some((key, from));
                }
            }
        }
        // A drag on a container that holds the press, with its velocity kept.
        let active = self.interaction.state.active;
        if self
            .interaction
            .hit(active)
            .is_some_and(|h| h.has(HIT_SCROLL))
        {
            if input.down && !input.pressed {
                let d = self.interaction.touch_allows(
                    self.interaction.state.pointer - p,
                    &self.env,
                    &self.tree,
                );
                let d = self.axis_lock(active, d);
                self.tree.scroll_chain_by(active, d, &mut self.interaction);
                self.interaction.state.drag_v = self.interaction.state.drag_v * 0.5 + d * 0.5;
            } else if input.released {
                self.interaction.state.wheel_from = None;
                let v = self.interaction.state.drag_v;
                let reduced = self.theme().reduced_motion;
                if self.tree.snaps(active) {
                    // It settles on a snap position (a still release too).
                    self.interaction.state.fling =
                        Some((active, if reduced { Vec2::ZERO } else { v }));
                } else if !reduced && v.length() > 1.0 {
                    self.interaction.state.fling = Some((active, v));
                }
                self.interaction.state.drag_v = Vec2::ZERO;
            }
        }
        // A fling: the released drag's velocity, decaying to rest; it stops
        // at the content's end on each axis.
        if let Some((key, v)) = self.interaction.state.fling {
            if self.interaction.hit(key).is_none() {
                self.interaction.state.fling = None;
            } else {
                let keep = self.fling_keep();
                let decay = 1.0 - keep;
                let at = self.interaction.offset(key);
                // A snapping container steers its fling onto the snap
                // position nearest where it would stop: a velocity of
                // (target − offset)·(1 − decay) decays onto the target
                // exactly, so the next tick projects the same target and the
                // fling needs no state beyond its velocity.
                let from = self
                    .interaction
                    .state
                    .wheel_from
                    .filter(|w| w.0 == key)
                    .map_or(at, |w| w.1);
                if let Some(target) =
                    self.interaction
                        .snap_target(key, at + v / keep, from, &self.tree)
                {
                    if self.theme().reduced_motion {
                        self.interaction.scroll_by(key, target - at);
                        self.interaction.state.fling = None;
                        self.interaction.state.wheel_from = None;
                    } else {
                        self.interaction.scroll_by(key, (target - at) * keep);
                        let rest = target - self.interaction.offset(key);
                        if rest.length() < 0.25 {
                            self.interaction.scroll_by(key, rest);
                            self.interaction.state.fling = None;
                            self.interaction.state.wheel_from = None;
                        } else {
                            self.interaction.state.fling = Some((key, rest * keep));
                        }
                    }
                    if let Some(k) = wheel_snap {
                        self.interaction.state.fling = Some((k, Vec2::ZERO));
                    }
                    return grabbed;
                }
                let left = self.interaction.scroll_by(key, v);
                let mut next = v * decay;
                // At the content's end on an axis (what the offset could not
                // take, past a float's rounding of the move), it stops there.
                if left.x.abs() > 1e-3 {
                    next.x = 0.0;
                }
                if left.y.abs() > 1e-3 {
                    next.y = 0.0;
                }
                if next.length() >= FLING_STOP {
                    self.interaction.state.fling = Some((key, next));
                } else {
                    self.interaction.state.fling = None;
                    // A smooth scroll's glide lands exactly where it was
                    // aimed: the rest a decaying fling still had to go.
                    if self.tree.scroll_css(key).smooth && keep > 0.0 {
                        self.interaction.scroll_by(key, next / keep);
                    }
                }
            }
        }
        if let Some(k) = wheel_snap {
            self.interaction.state.fling = Some((k, Vec2::ZERO));
        }
        grabbed
    }
}

impl crate::ui::Interaction {
    /// The room at the end of vertical scroller `key` its bar takes, the bar
    /// being drawn over its right edge: the bar and a hairline either side
    /// while its content ran past it last frame, so that content (a results
    /// stat's figure, a grade) never lies under the bar; else 0. Narrower,
    /// the content is no shorter, so it keeps overflowing and the gutter
    /// holds. Layout gives it to every vertical-only scroller with bars.
    /// CSS `scrollbar-gutter: stable` keeps it while nothing overflows,
    /// and `scrollbar-width: thin` halves the bar's part of it.
    pub(crate) fn bar_gutter(&self, key: u64, css: ScrollCss, env: &crate::ui::Env) -> f32 {
        if css.gutter != 0
            || self
                .state
                .scroll_info
                .get(&key)
                .is_some_and(|s| s.range.y > 0.0)
        {
            let bar = if css.thin {
                env.theme.space.xs / 2.0
            } else {
                env.theme.space.xs
            };
            bar + 2.0 * env.theme.space.xxs
        } else {
            0.0
        }
    }

    /// Element `key`'s `scroll-state()` flags ([`crate::css::media::scroll`])
    /// as the last frame laid it out: the edges a scroll container can still
    /// scroll toward, and the axes on which a snap area sits at its
    /// scroller's snap position, and the edges a sticky box is held to.
    /// `scrolled` is the document layer's.
    pub(crate) fn scroll_flags_now(&self, key: u64, tree: &crate::ui::Tree) -> u16 {
        use crate::css::media::scroll::*;
        let get = |i: usize| {
            if tree.building {
                tree.previous_node(i)
            } else {
                tree.nodes.get(i)
            }
        };
        // Kept (retained) nodes stay in the tree while a tick builds;
        // rebuilt ones are among the retiring nodes.
        let found = tree
            .index
            .find(key)
            .and_then(get)
            .filter(|n| n.key == key)
            .or_else(|| {
                tree.building
                    .then(|| tree.previous_nodes().find(|n| n.key == key))
                    .flatten()
            });
        let Some(n) = found else {
            return 0;
        };
        let mut flags = 0;
        let at = self.offset(key);
        if n.layout.used.scroll {
            flags |= if at.y > 0.5 { TOP } else { 0 };
            flags |= if at.y < n.max_scroll() - 0.5 {
                BOTTOM
            } else {
                0
            };
        }
        if n.layout.used.scroll_x {
            flags |= if at.x > 0.5 { LEFT } else { 0 };
            flags |= if at.x < n.max_scroll_x() - 0.5 {
                RIGHT
            } else {
                0
            };
        }
        flags |= u16::from(n.layout.stuck) << STUCK.trailing_zeros();
        let align = n.el.snap_align;
        let mut p = n.parent;
        while align != 0 && p != crate::layout::NONE {
            let Some(s) = get(p) else { break };
            if !s.scrolls() {
                p = s.parent;
                continue;
            }
            let at = self.offset(s.key);
            let max = [s.max_scroll_x(), s.max_scroll()];
            let view = snapport(s);
            for axis in 0..2 {
                let a = (align >> (2 * axis)) & 3;
                if s.el.snap & (1 << axis) == 0 || a == 0 {
                    continue;
                }
                let d = snap_delta(axis, a, n, view);
                let now = at[axis];
                if ((now + d).clamp(0.0, max[axis].max(0.0)) - now).abs() < 0.5 {
                    flags |= 1 << (4 + axis);
                }
            }
            break;
        }
        flags
    }

    /// Moves each new container noted by `Ui::note_initial_target` so its
    /// target's scroll-margin box starts at its view's start (inside its
    /// padding and scroll-padding), clamped to its range, from the boxes
    /// just laid out (CSSOM View's scroll into view, `start` alignment).
    /// Whether one moved: the layout runs again at the new offsets.
    pub(crate) fn apply_initial_targets(&mut self, tree: &crate::ui::Tree) -> bool {
        let mut moved = false;
        for (container, target) in std::mem::take(&mut self.state.initial_targets) {
            let (Some(c), Some(t)) = (tree.index.find(container), tree.index.find(target)) else {
                continue;
            };
            let cn = &tree.nodes[c];
            let at = self.offset(container);
            let range = Vec2::new(cn.max_scroll_x(), cn.max_scroll());
            let to = (at + into_view_start(tree, c, t)).clamp(Vec2::ZERO, range.max(Vec2::ZERO));
            let s = &mut self.state;
            if cn.layout.used.scroll_x && to.x != at.x {
                s.scroll_x.insert(container, to.x);
                moved = true;
            }
            if cn.layout.used.scroll && to.y != at.y {
                s.scroll.insert(container, to.y);
                moved = true;
            }
        }
        moved
    }

    /// CSS Scroll Anchoring 1 §2.2: each scroll container that chose an
    /// anchor box last frame ([`Self::select_scroll_anchors`]) moves its
    /// offset by how far the anchor moved in its content since, so what it
    /// shows stays put when content above it grows or shrinks. Suppressed
    /// at offset 0. Run once a frame, after the trees are laid out (the
    /// anchors are taken); answers whether an offset moved, so they are
    /// laid out again.
    pub(crate) fn apply_scroll_anchors(&mut self, tree: &crate::ui::Tree) -> bool {
        let mut moved = false;
        for (container, anchor, was) in std::mem::take(&mut self.state.scroll_anchors) {
            let (Some(c), Some(a)) = (tree.index.find(container), tree.index.find(anchor)) else {
                continue;
            };
            let at = self.offset(container).y;
            if at <= 0.0 || !tree.nodes[c].scrolls() {
                continue;
            }
            // Unsnapped: a snapped rect moves by its rounding as it scrolls.
            let now = tree.nodes[a].layout.y - tree.nodes[c].layout.y + at;
            let to = (at + now - was).clamp(0.0, tree.nodes[c].max_scroll().max(0.0));
            if (to - at).abs() > 1.0 / 256.0 {
                self.state.scroll.insert(container, to);
                moved = true;
            }
        }
        moved
    }

    /// CSS Scroll Anchoring 1 §2.1: for each scroll container of the main
    /// tree scrolled down from its start (and not snapping, not
    /// `overflow-anchor: none`), the anchor box: the first box in tree
    /// order wholly inside its view (inset by `scroll-padding`), going
    /// into the first one only partly inside to find a deeper one. Hidden
    /// boxes, sticky and screen-fixed ones and `overflow-anchor: none`
    /// subtrees are skipped; a nested scroll container is a box whose
    /// content is not searched. Kept with its top in the container's
    /// content for the next frame.
    pub(crate) fn select_scroll_anchors(&mut self, tree: &crate::ui::Tree) {
        use crate::element::Position;
        self.state.scroll_anchors.clear();
        let mut scrolled: Vec<_> = self
            .state
            .scroll
            .iter()
            .filter(|(_, at)| **at > 0.0)
            .filter_map(|(&key, &at)| tree.index.find(key).map(|c| (c, at)))
            .collect();
        // Keep the slot order used by the anchors' application pass.
        scrolled.sort_unstable_by_key(|&(c, _)| c);
        for (c, at) in scrolled {
            let cn = &tree.nodes[c];
            if !cn.layout.used.scroll
                || cn.el.scroll_css.no_anchor
                || cn.el.snap & 3 != 0
                || cn.hidden
            {
                continue;
            }
            // The main tree's (floating layers lay out after the anchors
            // are applied).
            let mut root = c;
            while tree.nodes[root].parent != crate::layout::NONE {
                root = tree.nodes[root].parent;
            }
            if root != 0 {
                continue;
            }
            let view = snapport(cn);
            let (top, bottom) = (view.y, view.y + view.height);
            let excluded = |n: &crate::layout::Node| {
                n.hidden
                    || n.el.scroll_css.no_anchor
                    || matches!(n.props.position, Position::Sticky | Position::Screen)
            };
            // Candidate examination: a box wholly in view is the anchor; one
            // partly in view is unless a box inside it is.
            fn examine(
                tree: &crate::ui::Tree,
                i: usize,
                (top, bottom): (f32, f32),
                excluded: &dyn Fn(&crate::layout::Node) -> bool,
            ) -> Option<usize> {
                let n = &tree.nodes[i];
                let r = n.placed.rect;
                if excluded(n) || r.y + r.height <= top || r.y >= bottom {
                    return None;
                }
                if r.y >= top && r.y + r.height <= bottom {
                    return Some(i);
                }
                if !n.scrolls() {
                    for k in crate::layout::all_children(n) {
                        if let Some(a) = examine(tree, k, (top, bottom), excluded) {
                            return Some(a);
                        }
                    }
                }
                Some(i)
            }
            let anchor = crate::layout::all_children(cn)
                .into_iter()
                .find_map(|k| examine(tree, k, (top, bottom), &excluded));
            if let Some(a) = anchor {
                let pos = tree.nodes[a].layout.y - cn.layout.y + at;
                self.state
                    .scroll_anchors
                    .push((cn.key, tree.nodes[a].key, pos));
            }
        }
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

    /// Where snapping container `key` settles from offset `projected` (CSS
    /// Scroll Snap 1): on each snapping axis, the nearest snap position of
    /// the areas inside it (not those inside a nested scroller), clamped to
    /// its range; `mandatory` always, `proximity` only within a third of
    /// its view, else `projected`. Read from last frame's boxes, which were
    /// laid out at its offset now. Walks only down to the snap areas, and
    /// only while the container settles.
    /// `from` is where the scroll started (the offset before the wheel
    /// moved it, else now): it does not pass a `scroll-snap-stop: always`
    /// area on its way from there.
    fn snap_target(
        &self,
        key: u64,
        projected: Vec2,
        from: Vec2,
        tree: &crate::ui::Tree,
    ) -> Option<Vec2> {
        let i = tree.index.find(key)?;
        let n = &tree.nodes[i];
        let snap = n.el.snap;
        if snap & 3 == 0 {
            return None;
        }
        let view = snapport(n);
        let at = self.offset(key);
        // Each axis's snap positions, and whether the area says
        // `scroll-snap-stop: always`.
        let mut found: [Vec<(f32, bool)>; 2] = Default::default();
        // A multi-column box's `::column`s are snap areas (CSS Multicol 2
        // §4.8), the container's own among them.
        let columns = |cn: &crate::layout::Node, found: &mut [Vec<(f32, bool)>; 2]| {
            let Some(m) = cn.layout.used.multicol.filter(|m| m.snap.align != 0) else {
                return;
            };
            for &r in &cn.layout.columns {
                for (axis, found) in found.iter_mut().enumerate() {
                    let align = (m.snap.align >> (2 * axis)) & 3;
                    if snap & (1 << axis) == 0 || align == 0 {
                        continue;
                    }
                    let d = snap_delta_of(axis, align, r, m.snap.margin, view);
                    found.push((at[axis] + d, m.snap.stop));
                }
            }
        };
        columns(n, &mut found);
        let mut stack: Vec<usize> = n.children.clone();
        while let Some(c) = stack.pop() {
            let cn = &tree.nodes[c];
            if cn.is_hidden() {
                continue;
            }
            if !cn.scrolls() {
                columns(cn, &mut found);
            }
            let a = cn.el.snap_align;
            if a == 0 {
                if !cn.scrolls() {
                    stack.extend(cn.children.iter().copied());
                }
                continue;
            }
            for (axis, found) in found.iter_mut().enumerate() {
                let align = (a >> (2 * axis)) & 3;
                if snap & (1 << axis) == 0 || align == 0 {
                    continue;
                }
                let d = snap_delta(axis, align, cn, view);
                found.push((at[axis] + d, cn.el.scroll_css.snap_stop));
            }
        }
        let range = self
            .state
            .scroll_info
            .get(&key)
            .map_or(Vec2::ZERO, |s| s.range)
            .max(Vec2::ZERO);
        let mut out = projected.clamp(Vec2::ZERO, range);
        for axis in 0..2 {
            let p = out[axis];
            let Some(mut best) = found[axis]
                .iter()
                .map(|c| c.0.clamp(0.0, range[axis]))
                .min_by(|a, b| (a - p).abs().total_cmp(&(b - p).abs()))
            else {
                continue;
            };
            // CSS Scroll Snap 1 §6.2: a fling or a wheel does not pass an
            // area that says always; it stops at the first one on its way.
            let now = from[axis];
            if let Some(stop) = found[axis]
                .iter()
                .filter(|c| c.1)
                .map(|c| c.0.clamp(0.0, range[axis]))
                .filter(|&c| (c - now).abs() > 0.5 && (c - now) * (p - now) > 0.0)
                .filter(|&c| (c - now).abs() < (best - now).abs())
                .min_by(|a, b| (a - now).abs().total_cmp(&(b - now).abs()))
            {
                best = stop;
            }
            let size = if axis == 0 { view.width } else { view.height };
            if snap & 4 == 0 || (best - p).abs() <= size / 3.0 {
                out[axis] = best;
            }
        }
        Some(out)
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

    /// The scrollbar under `p`, if a press there grabs one.
    fn thumb_at(&self, p: Vec2, tree: &crate::ui::Tree) -> Option<Thumb> {
        self.state.hits.iter().rev().find_map(|h| {
            if !h.has(HIT_SCROLL) || !h.rect.contains(p.x, p.y) {
                return None;
            }
            let info = self.state.scroll_info.get(&h.key)?;
            let grab_w = if tree.scroll_css(h.key).thin {
                BAR_GRAB / 2.0
            } else {
                BAR_GRAB
            };
            let r = h.full;
            let off = self.offset(h.key);
            if info.range.y > 0.0 && p.x >= h.rect.right() - grab_w {
                let start = bar_start(r.y, r.height, info.bar.y, off.y, info.range.y);
                let grab = (p.y - start).clamp(0.0, info.bar.y);
                return Some(Thumb {
                    key: h.key,
                    axis: 1,
                    grab,
                });
            }
            if info.range.x > 0.0 && p.y >= h.rect.bottom() - grab_w {
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
    /// A touch drag's move `d` (content offset) as CSS `touch-action`
    /// lets the element it started on pan (its used value, intersected
    /// with its ancestors'): an axis or direction it does not allow is
    /// dropped. A fine pointer pans freely.
    fn touch_allows(&self, d: Vec2, env: &crate::ui::Env, tree: &crate::ui::Tree) -> Vec2 {
        use crate::element::Interact as I;
        if !env.viewport.coarse_pointer {
            return d;
        }
        let Some(t) = tree
            .index
            .find(self.state.touch_key)
            .and_then(|i| tree.orders.touch_of.get(i))
        else {
            return d;
        };
        // A finger moving left moves the content's offset right.
        let ok = |v: f32, fwd: u8, back: u8| {
            if (v > 0.0 && t & fwd != 0) || (v < 0.0 && t & back != 0) {
                v
            } else {
                0.0
            }
        };
        Vec2::new(
            ok(d.x, I::PAN_LEFT, I::PAN_RIGHT),
            ok(d.y, I::PAN_UP, I::PAN_DOWN),
        )
    }
}

impl crate::ui::Tree {
    /// Element `key`'s CSS scroll properties, as last laid out.
    pub(crate) fn scroll_css(&self, key: u64) -> crate::scroll::ScrollCss {
        self.index
            .find(key)
            .and_then(|i| self.nodes.get(i))
            .map(|n| n.el.scroll_css)
            .unwrap_or_default()
    }

    /// Whether scroll container `key` snaps (CSS `scroll-snap-type`).
    fn snaps(&self, key: u64) -> bool {
        self.index
            .find(key)
            .is_some_and(|i| self.nodes[i].el.snap & 3 != 0)
    }

    /// Moves `d` through `key`'s chain: what one container cannot take
    /// goes to the one around it (nested scroll handoff).
    fn scroll_chain_by(
        &mut self,
        key: u64,
        d: Vec2,
        interaction: &mut crate::ui::Interaction,
    ) -> Vec2 {
        let mut rest = d;
        for k in interaction.scroll_chain(key) {
            if rest == Vec2::ZERO {
                break;
            }
            rest = interaction.scroll_by(k, rest);
            // CSS `overscroll-behavior` contain or none: no chaining past
            // it on an axis it scrolls.
            if let Some(n) = self.index.find(k).and_then(|i| self.nodes.get(i)) {
                let o = n.el.scroll_css.overscroll;
                if n.layout.used.scroll_x && o[0] != Overscroll::Auto {
                    rest.x = 0.0;
                }
                if n.layout.used.scroll && o[1] != Overscroll::Auto {
                    rest.y = 0.0;
                }
            }
        }
        rest
    }
}

/// A scroll container's snapport (CSS Scroll Snap 1 §5.1): its view inside
/// its padding, inset by its `scroll-padding`.
fn snapport(n: &crate::layout::Node) -> crate::layout::Rect {
    let pad = n.layout.used.padding;
    let (w, h) = (
        (n.placed.rect.width - pad.horizontal()).max(0.0),
        (n.placed.rect.height - pad.vertical()).max(0.0),
    );
    let [t, r, b, l] = n.el.scroll_css.padding_px(w, h);
    crate::layout::Rect::new(
        n.placed.rect.x + pad.left + l,
        n.placed.rect.y + pad.top + t,
        (w - l - r).max(0.0),
        (h - t - b).max(0.0),
    )
}

/// How far snap area `n`'s box, grown by its `scroll-margin`, lies from
/// its aligned place in snapport `view` on `axis` (`align` 1 start, 2
/// centre, 3 end).
fn snap_delta(axis: usize, align: u8, n: &crate::layout::Node, view: crate::layout::Rect) -> f32 {
    snap_delta_of(axis, align, n.placed.rect, n.el.scroll_css.margin, view)
}

/// [`snap_delta`] of a snap area `r` with `scroll-margin` `margin` (top,
/// right, bottom, left): an element's box or a `::column`.
fn snap_delta_of(
    axis: usize,
    align: u8,
    r: crate::layout::Rect,
    margin: [f32; 4],
    view: crate::layout::Rect,
) -> f32 {
    let [mt, mr, mb, ml] = margin;
    let (lo, len, vlo, vlen) = if axis == 0 {
        (r.x - ml, r.width + ml + mr, view.x, view.width)
    } else {
        (r.y - mt, r.height + mt + mb, view.y, view.height)
    };
    match align {
        1 => lo - vlo,
        2 => (lo + len / 2.0) - (vlo + vlen / 2.0),
        _ => (lo + len) - (vlo + vlen),
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

/// How far scroll container `c` scrolls (unclamped) to bring node `t`'s
/// scroll-margin box to its view's start, inside its padding and
/// scroll-padding (CSSOM View's scroll-into-view position, `start`
/// alignment), from the boxes as laid out.
pub(crate) fn into_view_start(tree: &crate::ui::Tree, c: usize, t: usize) -> Vec2 {
    let tn = &tree.nodes[t];
    into_view_start_of(tree, c, tn.placed.rect, tn.el.scroll_css.margin)
}

/// [`into_view_start`] of a box `r` with `scroll-margin` `m` (top, right,
/// bottom, left): an element's or a `::column`'s.
pub(crate) fn into_view_start_of(
    tree: &crate::ui::Tree,
    c: usize,
    r: crate::layout::Rect,
    m: [f32; 4],
) -> Vec2 {
    let cn = &tree.nodes[c];
    let pad = cn.layout.used.padding;
    let view = cn.placed.rect;
    let (w, h) = (
        (view.width - pad.horizontal()).max(0.0),
        (view.height - pad.vertical()).max(0.0),
    );
    let sp = cn.el.scroll_css.padding_px(w, h);
    Vec2::new(
        r.x - m[3] - (view.x + pad.left + sp[3]),
        r.y - m[0] - (view.y + pad.top + sp[0]),
    )
}
