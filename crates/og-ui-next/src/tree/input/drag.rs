//! Pointer and keyboard drag-and-drop (`draggable`, `dropTarget`, `dropAccept`).

use super::*;

/// A node being dragged, or a press on one not yet past the slop.
#[derive(Clone, Copy)]
pub(super) struct Drag {
    pub(super) src: u32,
    /// Where the press went down.
    pub(super) from: (f32, f32),
    /// Picked up: past the slop, or by a key.
    pub(super) live: bool,
    /// Carried by the keys (or an assistive technology): focus moves it.
    pub(super) keys: bool,
}

/// How far a mouse press moves before it drags (a finger's is the pan's).
const DRAG_SLOP: f32 = 4.0;

impl Tree {
    /// The `draggable` node `e` is in, itself first (NONE: none).
    pub(super) fn drag_up(&self, mut e: u32) -> u32 {
        while e != NONE {
            if self.input.per(e).is_some_and(|p| !p.drag.is_empty()) && !self.disabled(e) {
                return e;
            }
            e = self.els[e as usize].parent;
        }
        NONE
    }

    /// The `dropTarget` node `e` is in, itself first (NONE: none).
    pub(super) fn drop_up(&self, mut e: u32) -> u32 {
        while e != NONE {
            if self.input.per(e).is_some_and(|p| !p.drop.is_empty()) && !self.disabled(e) {
                return e;
            }
            e = self.els[e as usize].parent;
        }
        NONE
    }

    pub(super) fn drop_at(&self, x: f32, y: f32) -> u32 {
        self.drop_up(self.hit_where(x, y, |el| el.js != NONE))
    }

    pub(super) fn centre(&self, e: u32) -> (f32, f32) {
        let Some(&(x, y, ..)) = self.abs.get(e as usize) else {
            return (0.0, 0.0);
        };
        let b = self.arena.box_of(self.els[e as usize].node);
        (x + b.w / 2.0, y + b.h / 2.0)
    }

    fn box_text(&self, e: u32) -> Box<str> {
        let Some(&(x, y, ..)) = self.abs.get(e as usize) else {
            return Box::default();
        };
        let b = self.arena.box_of(self.els[e as usize].node);
        format!("{x} {y} {} {}", b.w, b.h).into()
    }

    /// Fires a drag event on `e` when it listens.
    fn fire_drag(&mut self, e: u32, event: OpEvent, x: f32, y: f32, key: &str, value: Box<str>) {
        let el = &self.els[e as usize];
        if el.listens & event.bit() != 0 {
            let mut f = Fired::at(el.js, event, x, y);
            f.key = key.into();
            f.value = value;
            self.events.push(f);
        }
    }

    fn data_of(&self, e: u32) -> Box<str> {
        self.input
            .per(e)
            .map_or_else(Box::default, |p| p.drag.clone())
    }

    fn target_id(&self, t: u32) -> Box<str> {
        match t {
            NONE => Box::default(),
            t => self
                .input
                .per(t)
                .map_or_else(Box::default, |p| p.drop.clone()),
        }
    }

    /// A carried node is over target `t` (NONE: none): its source hears
    /// `dragmove` with the target's id.
    pub(super) fn drag_over(&mut self, t: u32, x: f32, y: f32) {
        let Some(d) = self.input.drag else { return };
        let src = d.src;
        let (data, id) = (self.data_of(src), self.target_id(t));
        self.fire_drag(src, OpEvent::DragMove, x, y, &data, id);
    }

    fn pick_up(&mut self, src: u32, x: f32, y: f32) {
        self.set_state(src, PRESSED, true);
        let (data, b) = (self.data_of(src), self.box_text(src));
        self.fire_drag(src, OpEvent::DragStart, x, y, &data, b);
        self.sound(src, "select");
    }

    /// The pointer moved while a press holds a `draggable`: past the slop
    /// it picks it up (the press then clicks nothing); once carried, each
    /// move reports the target under it.
    pub(super) fn drag_moved(&mut self, x: f32, y: f32) {
        let Some(d) = self.input.drag else { return };
        if d.keys {
            return;
        }
        if !d.live {
            let slop = if self.input.touch {
                TOUCH_SLOP
            } else {
                DRAG_SLOP
            };
            let (dx, dy) = (x - d.from.0, y - d.from.1);
            if dx * dx + dy * dy <= slop * slop {
                return;
            }
            self.cancel_press();
            if let Some(d) = &mut self.input.drag {
                d.live = true;
            }
            self.pick_up(d.src, x, y);
        }
        let t = self.drop_at(x, y);
        self.drag_over(t, x, y);
    }

    /// Puts the carried node down on target `t` (NONE: back where it
    /// was): the target hears `drop` with the node's data, the node
    /// `dragend` with the target's id ("" when put back).
    pub(super) fn put_down(&mut self, t: u32, x: f32, y: f32) {
        let Some(d) = self.input.drag.take() else {
            return;
        };
        let src = d.src;
        self.set_state(src, PRESSED, false);
        let (data, id) = (self.data_of(src), self.target_id(t));
        if t != NONE {
            let b = self.box_text(t);
            self.fire_drag(t, OpEvent::Drop, x, y, &data, b);
            self.sound(t, "confirm");
        } else {
            self.sound(src, "cancel");
        }
        self.fire_drag(src, OpEvent::DragEnd, x, y, &data, id);
        // Drop targets are focus stops only while a node is carried.
        if d.keys && self.input.focus != NONE && !self.focusable(self.input.focus) {
            self.focus(src);
        }
    }

    /// Confirm (a key, the action or an assistive technology) on `e`:
    /// with a node carried by the keys, puts it down on the target `e` is
    /// in (else back); else picks `e` up when it is `draggable`. Whether
    /// it did either.
    pub(super) fn carry(&mut self, e: u32) -> bool {
        if e == NONE {
            return false;
        }
        match self.input.drag {
            Some(d) if d.live && d.keys => {
                let t = self.drop_up(e);
                let (x, y) = self.centre(e);
                self.put_down(t, x, y);
                true
            }
            Some(_) => false,
            None if self.input.per(e).is_some_and(|p| !p.drag.is_empty()) && !self.disabled(e) => {
                self.input.drag = Some(Drag {
                    src: e,
                    from: self.centre(e),
                    live: true,
                    keys: true,
                });
                let (x, y) = self.centre(e);
                self.pick_up(e, x, y);
                true
            }
            None => false,
        }
    }

    /// `e` is freed: a drag of it ends without events.
    pub(in crate::tree) fn forget_drag(&mut self, e: u32) {
        if self.input.drag.is_some_and(|d| d.src == e) {
            self.input.drag = None;
        }
    }
}
