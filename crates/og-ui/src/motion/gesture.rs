//! Gestures on any element (docs/ui.md §3.1): a drag with an axis lock and
//! bounds, rubber-banding past them, a flick's momentum onto snap points,
//! swipe to dismiss, drop targets, and pinch and rotate with two touches.
//! Run in [`crate::Ui::end`] with the tick's input (the press was taken
//! against last tick's rectangles, as every press is); on release the
//! drag's velocity goes into the spring that takes it over.

use super::spring::{Channel, Transition, pack};
use super::state::*;
use super::timeline::UiEvent;
use crate::element::DragAxis;
use crate::ui::{HIT_DROP, Ui};
use og_core::glam::Vec2;

/// iOS's rubber band: past the edge by `x`, the element shows `c·x·d /
/// (d + c·x)` past it (`d` the element's size along the axis).
pub fn rubber_band(x: f32, d: f32, c: f32) -> f32 {
    if c <= 0.0 || d <= 0.0 {
        return 0.0;
    }
    (1.0 - 1.0 / (x * c / d + 1.0)) * d
}

/// What a drag or pinch on an element is doing (`Ui::gesture`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GestureInfo {
    /// A press is held on it.
    pub held: bool,
    /// It moves with the pointer (the press travelled past the threshold).
    pub dragging: bool,
    /// Its offset from where it was laid out, CSS px.
    pub offset_x: f32,
    pub offset_y: f32,
    /// CSS px per second.
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub scale: f32,
    /// Radians.
    pub rotation: f32,
    /// The snap point the last release chose (-1: none).
    pub snapped: i32,
    /// A swipe threw it off.
    pub dismissed: bool,
    /// The drop target under the drag (empty: none).
    pub over: String,
    /// The drop target it was dropped on, the tick it was (empty: none).
    pub dropped_on: String,
    /// For a drop target: a drag is over it.
    pub hovering: bool,
    /// For a drop target: the element dropped on it this tick (empty: none).
    pub received: String,
    /// Its tap, double tap and long press this tick ([`crate::GestureSpec`]).
    pub tapped: bool,
    pub double_tapped: bool,
    pub long_pressed: bool,
    /// How far into its long press the held press is, 0 to 1.
    pub long_press: f32,
}

impl Ui {
    /// One element's drag and pinch for this tick: answers its offset,
    /// scale and turn.
    pub(crate) fn run_gesture(&mut self, i: usize, m: &mut MotionState) -> [f32; 4] {
        let n = &self.nodes[i];
        let key = n.key;
        let now = self.state.tick;
        let tps = self.ticks_per_second;
        let tokens = self.theme.motion;
        let drag = n.props.drag.clone();
        let pinch = n.props.pinch;
        let size = Vec2::new(n.rect.width, n.rect.height);
        let input = self.input.clone();
        let p = input.pointer;
        let held_here = self.state.active == key;
        let mut g = m.gestures.get(&key).copied().unwrap_or(Gesture {
            snapped: -1,
            scale: 1.0,
            ..Gesture::default()
        });
        g.tick = now;
        g.dropped_on = 0;
        let (kx, ky) = (channel_key(key, DRAG_X), channel_key(key, DRAG_Y));
        let mut cx = m
            .channels
            .get(&kx)
            .copied()
            .unwrap_or(Channel::at(g.offset.x, now));
        let mut cy = m
            .channels
            .get(&ky)
            .copied()
            .unwrap_or(Channel::at(g.offset.y, now));
        if let Some(spec) = &drag {
            let threshold = if spec.threshold > 0.0 {
                spec.threshold
            } else {
                4.0
            };
            if input.pressed && held_here {
                g.held = true;
                g.dragging = false;
                g.dismissed = false;
                g.start = p;
                g.last = p;
                g.start_offset = Vec2::new(cx.value, cy.value);
                g.velocity = Vec2::ZERO;
                g.lock = match spec.axis {
                    DragAxis::X => 1,
                    DragAxis::Y => 2,
                    _ => 0,
                };
            }
            if g.held && input.down && held_here {
                let d = p - g.start;
                if !g.dragging && d.length() >= threshold {
                    g.dragging = true;
                    if spec.axis == DragAxis::Lock {
                        g.lock = if d.x.abs() >= d.y.abs() { 1 } else { 2 };
                    }
                }
                if g.dragging {
                    let mut raw = g.start_offset + d;
                    if g.lock == 1 {
                        raw.y = g.start_offset.y;
                    }
                    if g.lock == 2 {
                        raw.x = g.start_offset.x;
                    }
                    if let Some(b) = spec.bounds {
                        let band = |v: f32, lo: f32, hi: f32, d: f32| {
                            if v < lo {
                                lo - rubber_band(lo - v, d, spec.rubber)
                            } else if v > hi {
                                hi + rubber_band(v - hi, d, spec.rubber)
                            } else {
                                v
                            }
                        };
                        raw.x = band(raw.x, b.x, b.right(), size.x.max(1.0));
                        raw.y = band(raw.y, b.y, b.bottom(), size.y.max(1.0));
                    }
                    let v = (p - g.last) * tps as f32;
                    g.velocity = g.velocity * 0.4 + v * 0.6;
                    g.offset = raw;
                    cx = Channel::at(raw.x, now);
                    cy = Channel::at(raw.y, now);
                    cx.velocity = g.velocity.x;
                    cy.velocity = g.velocity.y;
                    // The drop target under the pointer that takes it.
                    g.over = self
                        .state
                        .hits
                        .iter()
                        .rev()
                        .find(|h| h.key != key && h.has(HIT_DROP) && h.contains(p.x, p.y))
                        .filter(|h| {
                            let accept = self.drop_accepts.get(&h.key);
                            accept.is_none_or(|a| a.is_empty() || a.contains(&spec.tag))
                        })
                        .map_or(0, |h| h.key);
                }
                g.last = p;
            } else if g.held {
                // Released (or the press went elsewhere): the spring takes it.
                g.held = false;
                if g.dragging {
                    g.dragging = false;
                    let v = if self.theme.reduced_motion {
                        Vec2::ZERO
                    } else {
                        g.velocity
                    };
                    let at = g.offset;
                    // A flick carries it on: where friction would stop it.
                    let projected = if spec.momentum { at + v * 0.3 } else { at };
                    let mut to = at;
                    let along = |w: Vec2| match g.lock {
                        1 => w.x,
                        2 => w.y,
                        _ => {
                            if w.x.abs() >= w.y.abs() {
                                w.x
                            } else {
                                w.y
                            }
                        }
                    };
                    let mut dismissed = false;
                    if let Some(dm) = spec.dismiss {
                        let axis_size = match g.lock {
                            2 => size.y,
                            1 => size.x,
                            _ => size.x.max(size.y),
                        };
                        let dist = if dm.distance > 0.0 {
                            dm.distance
                        } else {
                            0.4 * axis_size
                        };
                        let vel = if dm.velocity > 0.0 {
                            dm.velocity
                        } else {
                            800.0
                        };
                        if along(projected).abs() >= dist || along(v).abs() >= vel {
                            dismissed = true;
                            let sign = if along(projected) < 0.0 { -1.0 } else { 1.0 };
                            let far = self.viewport.width.max(self.viewport.height) + axis_size;
                            to = match g.lock {
                                2 => Vec2::new(at.x, sign * far),
                                1 => Vec2::new(sign * far, at.y),
                                _ if projected.x.abs() >= projected.y.abs() => {
                                    Vec2::new(sign * far, at.y)
                                }
                                _ => Vec2::new(at.x, sign * far),
                            };
                        }
                    }
                    if !dismissed {
                        if !spec.snap.is_empty() {
                            let (k, &(sx, sy)) = spec
                                .snap
                                .iter()
                                .enumerate()
                                .min_by(|a, b| {
                                    let da = (projected - Vec2::new(a.1.0, a.1.1)).length_squared();
                                    let db = (projected - Vec2::new(b.1.0, b.1.1)).length_squared();
                                    da.total_cmp(&db)
                                })
                                .unwrap_or((0, &(0.0, 0.0)));
                            to = Vec2::new(sx, sy);
                            g.snapped = k as i32;
                            self.events.push(UiEvent {
                                kind: "snap".into(),
                                name: k.to_string(),
                                target: self.path_of(i),
                                pitch: 1.0,
                                ..UiEvent::default()
                            });
                        } else if spec.return_home {
                            to = Vec2::ZERO;
                        } else {
                            to = projected;
                        }
                        if let Some(b) = spec.bounds {
                            to.x = to.x.clamp(b.x, b.right().max(b.x));
                            to.y = to.y.clamp(b.y, b.bottom().max(b.y));
                        }
                    } else {
                        g.dismissed = true;
                        self.events.push(UiEvent {
                            kind: "dismiss".into(),
                            target: self.path_of(i),
                            pitch: 1.0,
                            ..UiEvent::default()
                        });
                    }
                    if g.over != 0 {
                        g.dropped_on = g.over;
                        let target = self.drop_paths.get(&g.over).cloned().unwrap_or_default();
                        self.events.push(UiEvent {
                            kind: "drop".into(),
                            name: target,
                            target: self.path_of(i),
                            pitch: 1.0,
                            ..UiEvent::default()
                        });
                    }
                    g.over = 0;
                    let t = spec
                        .spring
                        .unwrap_or(Transition::Token(super::MotionToken::Ui));
                    let t = if self.theme.reduced_motion {
                        Transition::Instant
                    } else {
                        t
                    };
                    let (kind, mode) = pack(&t, &tokens);
                    cx.retarget(to.x, kind, mode, now);
                    cy.retarget(to.y, kind, mode, now);
                }
            }
            if !g.held || !g.dragging {
                g.offset = Vec2::new(cx.step(now, tps, false), cy.step(now, tps, false));
            }
            cx.stepped = now;
            cy.stepped = now;
            m.channels.insert(kx, cx);
            m.channels.insert(ky, cy);
        }
        if let Some(ps) = pinch {
            let pts = &input.pointers;
            let rect = self.hit(key).copied();
            let held: Vec<u32> = (0..og_input::MAX_POINTERS as u32)
                .filter(|&id| pts.down(id))
                .filter(|&id| {
                    let s = pts.start(id);
                    rect.is_some_and(|h| h.contains(s.x, s.y))
                })
                .collect();
            if held.len() >= 2 {
                let (a, b) = (pts.position(held[0]), pts.position(held[1]));
                let spread = a.distance(b);
                let angle = og_core::math::atan2(b.y - a.y, b.x - a.x);
                if !g.pinch {
                    g.pinch = true;
                    g.pinch_spread = spread.max(1.0);
                    g.pinch_angle = angle;
                    g.pinch_scale = g.scale;
                    g.pinch_rotation = g.rotation;
                }
                let lo = if ps.min_scale > 0.0 {
                    ps.min_scale
                } else {
                    0.5
                };
                let hi = if ps.max_scale > 0.0 {
                    ps.max_scale
                } else {
                    4.0
                };
                g.scale = (g.pinch_scale * spread / g.pinch_spread).clamp(lo, hi.max(lo));
                if ps.rotate {
                    g.rotation = g.pinch_rotation + (angle - g.pinch_angle);
                }
            } else {
                g.pinch = false;
            }
        }
        let out = [g.offset.x, g.offset.y, g.scale, g.rotation];
        m.gestures.insert(key, g);
        out
    }

    /// Places element `key`'s drag at rest at `offset` (a sheet opening at
    /// its first detent), as if a release had settled it there.
    pub(crate) fn seed_drag(&mut self, key: u64, offset: Vec2) {
        let now = self.state.tick;
        let m = &mut self.state.motion;
        let g = m.gestures.entry(key).or_insert(Gesture {
            snapped: -1,
            scale: 1.0,
            ..Gesture::default()
        });
        g.offset = offset;
        g.dismissed = false;
        g.held = false;
        g.dragging = false;
        g.snapped = 0;
        g.tick = now;
        m.channels
            .insert(channel_key(key, DRAG_X), Channel::at(offset.x, now));
        m.channels
            .insert(channel_key(key, DRAG_Y), Channel::at(offset.y, now));
    }

    /// What the drag or pinch on `id` (under the open element) is doing:
    /// last tick's answer, as responses are.
    pub fn gesture(&self, id: &str) -> GestureInfo {
        let key = self.key_for(id);
        self.gesture_of(key)
    }

    /// [`Ui::gesture`] of the element at a full path.
    pub fn gesture_at(&self, path: &str) -> GestureInfo {
        self.gesture_of(crate::id::key_of_path(path))
    }

    pub(crate) fn gesture_of(&self, key: u64) -> GestureInfo {
        let m = &self.state.motion;
        let g = m.gestures.get(&key).copied().unwrap_or(Gesture {
            snapped: -1,
            scale: 1.0,
            ..Gesture::default()
        });
        let path = |k: u64| self.drop_paths.get(&k).cloned().unwrap_or_default();
        let source = m
            .gestures
            .iter()
            .find(|(_, s)| s.dropped_on == key && key != 0)
            .map(|(k, _)| *k);
        GestureInfo {
            held: g.held,
            dragging: g.dragging,
            offset_x: g.offset.x,
            offset_y: g.offset.y,
            velocity_x: g.velocity.x,
            velocity_y: g.velocity.y,
            scale: g.scale,
            rotation: g.rotation,
            snapped: g.snapped,
            dismissed: g.dismissed,
            over: path(g.over),
            dropped_on: path(g.dropped_on),
            hovering: m.gestures.values().any(|s| s.over == key && s.dragging),
            received: source
                .map(|k| self.drag_paths.get(&k).cloned().unwrap_or_default())
                .unwrap_or_default(),
            tapped: key != 0 && (self.nav.tapped == key || self.tapped_by_keys(key)),
            double_tapped: key != 0 && self.nav.double_tapped == key,
            long_pressed: key != 0 && self.nav.long_pressed == key,
            long_press: self.long_press_progress(key),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::rubber_band;

    #[test]
    fn the_rubber_band_resists_more_the_further_it_goes() {
        let a = rubber_band(50.0, 200.0, 0.55);
        let b = rubber_band(100.0, 200.0, 0.55);
        assert!(a > 0.0 && a < 50.0 && b < 100.0 && b - a < a);
        assert!(rubber_band(1e6, 200.0, 0.55) < 200.0, "never past its size");
        assert_eq!(rubber_band(50.0, 200.0, 0.0), 0.0);
    }
}
