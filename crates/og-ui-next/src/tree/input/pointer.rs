//! Pointer hover, press/click/hold, touch gestures (pinch/rotate/swipe/pan),
//! and context menu dispatch.

use super::*;

impl Tree {
    /// Every pointer this frame (og-input's `og.Pointers`, in tree units):
    /// two fingers down on an element that hears `onPinch` or `onRotate`
    /// pinch and turn it (NSH's `Pinch`), and a pointer let go fast on one
    /// that hears `onSwipe` swipes it (NSH's `VelocityTracker`). Each
    /// pointer keeps the element it went down on.
    pub fn touches(&mut self, pointers: &og_input::Pointers) {
        let hears: u32 = OpEvent::Pinch.bit() | OpEvent::Rotate.bit() | OpEvent::Swipe.bit();
        let ms = self.time * 1000.0;
        let at = |id: u32| {
            let p = pointers.position(id);
            nsh_input::Point {
                x: nsh_input::Points(p.x),
                y: nsh_input::Points(p.y),
            }
        };
        for id in 0..5u32 {
            let bit = 1 << id;
            let pos = pointers.position(id);
            if pointers.pressed & bit != 0 {
                let hit = self.hit_where(pos.x, pos.y, |el| el.js != NONE);
                let e = self.up(hit, |el| el.listens & hears != 0);
                self.input.gestures[id as usize] = (e != NONE).then(|| {
                    let mut tracker = nsh::VelocityTracker::default();
                    tracker.add(ms, at(id));
                    Gesturing {
                        e,
                        from: (pos.x, pos.y),
                        tracker,
                        pinched: false,
                    }
                });
            } else if pointers.held & bit != 0
                && let Some(g) = &mut self.input.gestures[id as usize]
            {
                g.tracker.add(ms, at(id));
            }
        }
        // A second finger on the element the first is on starts a pinch.
        let pinches = OpEvent::Pinch.bit() | OpEvent::Rotate.bit();
        if self.input.pinch.is_none() {
            let down: Vec<(u32, u32)> = (0..5u32)
                .filter(|id| pointers.held & (1 << id) != 0)
                .filter_map(|id| self.input.gestures[id as usize].map(|g| (id, g.e)))
                .collect();
            if let Some(&(a, e)) = down.first()
                && let Some(&(b, _)) = down.iter().skip(1).find(|d| d.1 == e)
                && self.els[e as usize].listens & pinches != 0
            {
                for id in [a, b] {
                    if let Some(g) = &mut self.input.gestures[id as usize] {
                        g.pinched = true;
                    }
                }
                let p = Pinching {
                    e,
                    a,
                    b,
                    p: nsh::Pinch::new(at(a), at(b)),
                    last: (1.0, 0.0),
                };
                self.input.pinch = Some(p);
                self.fire_pinch(e, "start", (1.0, 0.0), &p, pointers);
            }
        }
        if let Some(mut p) = self.input.pinch {
            let both = pointers.held & (1 << p.a) != 0 && pointers.held & (1 << p.b) != 0;
            let dead = self.els[p.e as usize].dead;
            if both && !dead {
                let (scale, turn) = p.p.at(at(p.a), at(p.b));
                if (scale, turn.0) != p.last {
                    p.last = (scale, turn.0);
                    self.input.pinch = Some(p);
                    self.fire_pinch(p.e, "move", p.last, &p, pointers);
                }
            } else {
                self.input.pinch = None;
                if !dead {
                    self.fire_pinch(p.e, "end", p.last, &p, pointers);
                }
            }
        }
        for id in 0..5u32 {
            if pointers.released & (1 << id) == 0 {
                continue;
            }
            let Some(g) = self.input.gestures[id as usize].take() else {
                continue;
            };
            let el = &self.els[g.e as usize];
            if g.pinched || el.dead || el.listens & OpEvent::Swipe.bit() == 0 {
                continue;
            }
            let pos = pointers.position(id);
            let (dx, dy) = (pos.x - g.from.0, pos.y - g.from.1);
            let (vx, vy) = g.tracker.velocity();
            let horizontal = vx.abs() >= vy.abs();
            let (d, v) = if horizontal { (dx, vx) } else { (dy, vy) };
            if d.abs() < SWIPE_DISTANCE || v.abs() < SWIPE_VELOCITY || d.signum() != v.signum() {
                continue;
            }
            let dir = match (horizontal, v > 0.0) {
                (true, true) => "right",
                (true, false) => "left",
                (false, true) => "down",
                (false, false) => "up",
            };
            let mut f = Fired::at(self.els[g.e as usize].js, OpEvent::Swipe, pos.x, pos.y);
            f.key = dir.into();
            f.value = format!("{vx} {vy}").into();
            self.events.push(f);
        }
    }

    /// `onPinch` (the scale) and `onRotate` (the turn) on `e`, at the
    /// pinching fingers' midpoint, for those it hears.
    fn fire_pinch(
        &mut self,
        e: u32,
        phase: &str,
        (scale, turn): (f32, f32),
        p: &Pinching,
        pointers: &og_input::Pointers,
    ) {
        let (a, b) = (pointers.position(p.a), pointers.position(p.b));
        self.fire_pinch_at(
            e,
            phase,
            scale,
            turn,
            ((a.x + b.x) / 2.0, (a.y + b.y) / 2.0),
        );
    }

    fn fire_pinch_at(&mut self, e: u32, phase: &str, scale: f32, turn: f32, (x, y): (f32, f32)) {
        let el = &self.els[e as usize];
        let (js, listens) = (el.js, el.listens);
        for (ev, v) in [(OpEvent::Pinch, scale), (OpEvent::Rotate, turn)] {
            if listens & ev.bit() != 0 {
                let mut f = Fired::at(js, ev, x, y);
                f.key = phase.into();
                f.value = v.to_string().into();
                self.events.push(f);
            }
        }
    }

    /// The pointer moved to (x, y): the op elements it left lose `hover:`
    /// from the inside out, those it came over gain it from the outside
    /// in (NSH's `hover_changes`), and the innermost that listens hears
    /// the move. A held press is inside its element while the pointer is
    /// within NSH's press retention around it.
    fn hover_to(&mut self, x: f32, y: f32) {
        // CSS's `cursor`: the innermost element under the pointer that
        // names one, listening or not.
        let mut e = self.topmost(x, y, |_| true, false);
        self.input.cursor = nsh_tailwind::effects::Cursor::Auto;
        while e != NONE {
            let c = self.els[e as usize].record.cursor;
            if c != nsh_tailwind::effects::Cursor::Auto {
                self.input.cursor = c;
                break;
            }
            e = self.els[e as usize].parent;
        }
        let top = self.hit_where(x, y, |el| el.js != NONE);
        let mut next = std::mem::take(&mut self.input.next);
        next.clear();
        // A finger hovers nothing: `hover:` is the mouse's, so a tapped
        // control is not left lit after the finger lifts (sticky hover).
        let mut e = if self.input.touch { NONE } else { top };
        while e != NONE {
            if self.els[e as usize].js != NONE {
                next.push(e);
            }
            e = self.els[e as usize].parent;
        }
        let hover = std::mem::take(&mut self.input.hover);
        // Both lists are out of `self`, so the states can restyle.
        let (left, came) = nsh::hover_changes(&hover, &next);
        for &e in left {
            self.set_state(e, HOVERED, false);
        }
        // The innermost control the pointer came over (they come from the
        // outside in) sounds `hover`; a finger has no hover.
        let mut control = NONE;
        for &e in came {
            self.set_state(e, HOVERED, true);
            if self.els[e as usize].clickable && !self.disabled(e) {
                control = e;
            }
        }
        if control != NONE && !self.input.touch {
            self.sound(control, "hover");
        }
        self.input.hover = next;
        self.input.next = hover;
        if let Some(h) = &mut self.input.held {
            h.moved |= !near(h.at, (x, y));
            let e = h.e;
            let inside = self.inside(e, x, y);
            self.set_state(e, PRESSED, inside);
        }
    }

    /// Whether (x, y) is on op element `e`'s box or within NSH's press
    /// retention around it.
    pub(super) fn inside(&self, e: u32, x: f32, y: f32) -> bool {
        let Some(&(bx, by, shown, _)) = self.abs.get(e as usize) else {
            return false;
        };
        // A span in a paragraph is its glyphs.
        if let Some(on) = self.on_span(e, x, y) {
            return shown && on;
        }
        // A spotlight's box is the surface outside its hole.
        if let Some(in_hole) = self.in_hole(e, x, y) {
            let (w, h) = self.size;
            let on = x >= 0.0 && y >= 0.0 && x < w && y < h;
            return shown && on && !in_hole;
        }
        let Some((ux, uy)) = self.untransform(e, x, y) else {
            return false;
        };
        let node = self.els[e as usize].node;
        let b = self
            .motion
            .box_of(node)
            .unwrap_or_else(|| self.arena.box_of(node));
        let local = nsh_input::Point {
            x: nsh_input::Points(ux - bx),
            y: nsh_input::Points(uy - by),
        };
        let size = nsh_input::Size {
            width: nsh_input::Points(b.w),
            height: nsh_input::Points(b.h),
        };
        // On a touch screen the box a press took by its grown target
        // (`Tree::near_target`) keeps it while the finger stays in that.
        let (gx, gy) = if self.input.touch {
            (
                ((TOUCH_TARGET - b.w) / 2.0).max(0.0),
                ((TOUCH_TARGET - b.h) / 2.0).max(0.0),
            )
        } else {
            (0.0, 0.0)
        };
        let slop = nsh_input::EdgeInsets {
            top: nsh_input::Points(gy),
            right: nsh_input::Points(gx),
            bottom: nsh_input::Points(gy),
            left: nsh_input::Points(gx),
        };
        shown && nsh::released_inside(local, size, slop)
    }

    /// The pointer on op nodes this frame, at (x, y): it hovers, a press
    /// takes the innermost element that answers one (and focuses the
    /// nearest focusable around it), and a release within the press's
    /// retention clicks it: `press` and `tap`, with `value` "2" for the
    /// second of a double press. Whether a press went to an op node.
    pub(in crate::tree) fn pointer_ops(
        &mut self,
        x: f32,
        y: f32,
        moved: bool,
        pressed: bool,
        released: bool,
    ) -> bool {
        self.sync_modals();
        self.hover_to(x, y);
        let under = self.input.hover.first().copied().unwrap_or(NONE);
        // While a pointerdown holds it, the moves go to its listener.
        let to = match self.input.capture {
            c if c != NONE && !self.els[c as usize].dead => c,
            _ => under,
        };
        if moved && to != NONE {
            let l = self.up(to, |el| el.listens & OpEvent::PointerMove.bit() != 0);
            if l != NONE {
                self.fire(l, OpEvent::PointerMove, x, y);
            }
        }
        // A grabbed scroll bar thumb takes the pointer until it lets go.
        if self.input.grab.is_some() {
            if moved {
                self.drag_thumb(x, y);
            }
            if released {
                self.drop_thumb();
            }
            return true;
        }
        if pressed && self.grab_thumb(x, y) {
            return true;
        }
        if moved && !pressed {
            self.drag_moved(x, y);
            self.finger_moved(x, y);
            self.field_drag(x, y);
        }
        let mut took = false;
        if pressed {
            if under != NONE {
                let l = self.up(under, |el| el.listens & OpEvent::PointerDown.bit() != 0);
                if l != NONE {
                    self.fire(l, OpEvent::PointerDown, x, y);
                }
                self.input.capture = l;
            }
            let e = self.hit(x, y);
            if e != NONE && self.els[e as usize].js != NONE {
                took = true;
                let second = self.input.first.is_some_and(|(f, at, p)| {
                    f == e && self.time - at <= DOUBLE_PRESS && near(p, (x, y))
                });
                self.input.held = Some(Held {
                    e,
                    at: (x, y),
                    moved: false,
                    second,
                });
                self.set_state(e, PRESSED, true);
                if self.input.per(e).is_some_and(|p| p.hold > 0.0) {
                    self.input.holding = Some(Holding {
                        e,
                        start: self.time,
                        keys: false,
                        done: false,
                    });
                }
            }
            // A `draggable` takes the press whether or not it answers one:
            // from the pressed element, else from what is under the pointer.
            let src = self.drag_up(if took { e } else { under });
            if src != NONE && self.input.drag.is_none() {
                took = true;
                self.input.drag = Some(Drag {
                    src,
                    from: (x, y),
                    live: false,
                    keys: false,
                });
            }
            // A finger stops a fling under it, and may pan or long-press.
            if self.input.touch {
                self.input.fling = None;
                self.input.finger = Some(Finger {
                    origin: (x, y),
                    last: 0.0,
                    at: self.time,
                    velocity: 0.0,
                    mode: Touch::Pending,
                    due: self.time + LONG_PRESS,
                });
            }
            // A press focuses what it lands in, or blurs (Rune: a press
            // on content that takes no focus).
            if under != NONE || e != NONE {
                let from = if e != NONE { e } else { under };
                let f = self.focusable_up(from);
                self.input.modality = Modality::Pointer;
                self.focus(f);
            }
            self.field_press(x, y);
        }
        if released {
            self.input.selecting = None;
            match self.input.drag {
                Some(d) if d.live && !d.keys => {
                    let t = self.drop_at(x, y);
                    self.put_down(t, x, y);
                }
                Some(d) if !d.live => self.input.drag = None,
                _ => {}
            }
            let c = std::mem::replace(&mut self.input.capture, NONE);
            let to = if c != NONE && !self.els[c as usize].dead {
                c
            } else {
                under
            };
            if to != NONE {
                let l = self.up(to, |el| el.listens & OpEvent::PointerUp.bit() != 0);
                if l != NONE {
                    self.fire(l, OpEvent::PointerUp, x, y);
                }
            }
            if let Some(f) = self.input.finger.take()
                && let Touch::Pan { e, horizontal, .. } = f.mode
                && self.time - f.at < FLING_HOLD
                && f.velocity.abs() > 1.0
            {
                self.input.fling = Some((e, horizontal, f.velocity, self.time));
            }
            if let Some(h) = self.input.held.take() {
                took = true;
                self.set_state(h.e, PRESSED, false);
                let el = &self.els[h.e as usize];
                let holds = self.input.per(h.e).is_some_and(|p| p.hold > 0.0);
                if holds {
                    // A hold presses when it completes, not on release.
                    self.input.first = None;
                } else if !el.dead && el.js != NONE && self.inside(h.e, x, y) && !self.disabled(h.e)
                {
                    self.click(h.e, x, y, h.second);
                    self.input.first = (!h.moved && !h.second).then_some((h.e, self.time, h.at));
                } else {
                    self.input.first = None;
                }
            }
        }
        took
    }

    pub(super) fn click(&mut self, e: u32, x: f32, y: f32, second: bool) {
        let event = if std::mem::take(&mut self.input.confirming) {
            "confirm"
        } else {
            "press"
        };
        self.sound(e, event);
        let js = self.els[e as usize].js;
        let listens = self.els[e as usize].listens;
        for ev in [OpEvent::Press, OpEvent::Tap] {
            if listens & ev.bit() != 0 {
                let mut f = Fired::at(js, ev, x, y);
                if second {
                    f.value = "2".into();
                }
                self.events.push(f);
            }
        }
    }

    /// The op elements at (x, y), innermost first, into `chain`.
    pub(super) fn chain_at(&mut self, x: f32, y: f32) {
        let mut chain = std::mem::take(&mut self.input.chain);
        chain.clear();
        let mut e = self.hit_where(x, y, |el| el.js != NONE);
        while e != NONE {
            if self.els[e as usize].js != NONE {
                chain.push(e);
            }
            e = self.els[e as usize].parent;
        }
        self.input.chain = chain;
    }

    /// A press on a node with a `hold` starts it; elsewhere it presses.
    pub(super) fn press_or_hold(&mut self, e: u32, keys: bool) {
        if self.input.per(e).is_some_and(|p| p.hold > 0.0) {
            if self.input.holding.as_ref().is_none_or(|h| h.e != e) {
                self.input.holding = Some(Holding {
                    e,
                    start: self.time,
                    keys,
                    done: false,
                });
            }
        } else {
            self.click(e, 0.0, 0.0, false);
        }
    }

    pub(super) fn fire_value(&mut self, e: u32, event: OpEvent, value: &str) {
        let el = &self.els[e as usize];
        if el.listens & event.bit() != 0 {
            let mut f = Fired::at(el.js, event, 0.0, 0.0);
            f.value = value.into();
            self.events.push(f);
        }
    }

    /// The secondary button went down at (x, y): `contextmenu` on the
    /// nearest node there that listens (a page's `contextmenu`, NSH's
    /// `onContextMenu`). Whether one heard it.
    pub fn context_menu(&mut self, x: f32, y: f32) -> bool {
        if !self.has_ops() {
            return false;
        }
        self.sync_modals();
        self.chain_at(x, y);
        let l = self
            .input
            .chain
            .iter()
            .copied()
            .find(|&e| self.els[e as usize].listens & OpEvent::ContextMenu.bit() != 0);
        l.inspect(|&l| self.fire(l, OpEvent::ContextMenu, x, y))
            .is_some()
    }

    /// Shift+F10 or the Menu key: `contextmenu` on the focused node or the
    /// nearest around it that listens, `value` `"key"` and at its box's
    /// bottom start, where a menu opened by the keys stands (WAI-ARIA).
    pub(super) fn context_menu_key(&mut self) -> bool {
        let mut e = self.input.focus;
        while e != NONE {
            let el = &self.els[e as usize];
            if !el.dead && el.listens & OpEvent::ContextMenu.bit() != 0 {
                let r = self.rect(e);
                let mut f = Fired::at(
                    self.els[e as usize].js,
                    OpEvent::ContextMenu,
                    r.x,
                    r.y + r.height,
                );
                f.value = "key".into();
                self.events.push(f);
                return true;
            }
            e = el.parent;
        }
        false
    }

    /// The press the pointer holds lets go without a press.
    pub(super) fn cancel_press(&mut self) {
        if let Some(h) = self.input.held.take() {
            self.set_state(h.e, PRESSED, false);
        }
        self.input.first = None;
    }

    /// A finger moved to (x, y): past the slop it pans the scroller under
    /// where it went down along the way it went, unless what heard its
    /// `pointerdown` drags; a pan lets the press go.
    fn finger_moved(&mut self, x: f32, y: f32) {
        let Some(f) = &self.input.finger else { return };
        let (ox, oy) = f.origin;
        let (dx, dy) = (x - ox, y - oy);
        if f.mode == Touch::Pending && dx * dx + dy * dy > TOUCH_SLOP * TOUCH_SLOP {
            let horizontal = dx.abs() > dy.abs();
            // A finger selecting text (after its long press) drags.
            let selecting = self.select_press.is_some_and(|p| p.held);
            let mode = if selecting
                || self.input.capture != NONE
                || self.input.drag.is_some_and(|d| d.live)
            {
                Touch::Drag
            } else {
                self.chain_at(ox, oy);
                let e =
                    self.input.chain.iter().copied().find(|&e| {
                        self.scroller(e) && self.extent(e)[usize::from(!horizontal)] > 0.0
                    });
                match e {
                    Some(e) => Touch::Pan {
                        e,
                        horizontal,
                        from: self.scroll_at(e, horizontal),
                    },
                    None => Touch::Drag,
                }
            };
            if matches!(mode, Touch::Pan { .. }) {
                self.cancel_press();
            }
            if let Some(f) = &mut self.input.finger {
                f.mode = mode;
            }
        }
        let now = self.time;
        let Some(f) = &mut self.input.finger else {
            return;
        };
        if let Touch::Pan {
            e,
            horizontal,
            from,
        } = f.mode
        {
            let along = if horizontal { dx } else { dy };
            let dt = (now - f.at) as f32;
            if dt > 0.0 {
                f.velocity = -(along - f.last) / dt;
            }
            (f.last, f.at) = (along, now);
            self.scroll_to(e, horizontal, from - along);
        }
    }
}
