//! A pad's virtual cursor (og.ui.setVirtualCursor): a screen built for the
//! pointer played on a pad. Pad 0's left stick moves a cursor that slows
//! and settles over controls (the magnet), its A presses where it points
//! as the mouse's button, its right stick scrolls under it. It only turns
//! the pad into the pointer the tree already takes: the press, the hit
//! test and the wheel are the real pointer's path. The real pointer moving
//! hides it and the stick shows it again.

use super::{Rgba, Tree};
use og_draw::paint::{PaintKind, PaintQuad};

/// How it moves. The zero value moves 900 CSS px a second, takes a quarter
/// second from a fine start to full speed, has a half-strength magnet and
/// scrolls 1200 CSS px a second.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct VirtualCursorDesc {
    /// CSS px a second at full tilt (0: 900).
    pub speed: f32,
    /// Ms a held stick takes from a third of its speed to all of it (0:
    /// 250; negative: none).
    pub accel: f32,
    /// 0 to 1: how much it slows over a control and, the stick at rest,
    /// pulls to its centre (0: 0.5; negative: none).
    pub magnet: f32,
    /// CSS px a second the right stick scrolls (0: 1200; negative: the
    /// right stick is the game's).
    pub scroll: f32,
}

impl VirtualCursorDesc {
    fn speed(&self) -> f32 {
        if self.speed > 0.0 { self.speed } else { 900.0 }
    }
    fn accel_ms(&self) -> f32 {
        match self.accel {
            a if a > 0.0 => a,
            a if a < 0.0 => 0.0,
            _ => 250.0,
        }
    }
    fn magnet(&self) -> f32 {
        match self.magnet {
            m if m > 0.0 => m.min(1.0),
            m if m < 0.0 => 0.0,
            _ => 0.5,
        }
    }
    fn scroll(&self) -> f32 {
        match self.scroll {
            s if s > 0.0 => s,
            s if s < 0.0 => 0.0,
            _ => 1200.0,
        }
    }
}

/// Pad 0 this tick: what the cursor reads, and takes while it shows.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pad {
    /// The left and right sticks, x right and y down, -1 to 1.
    pub stick: [f32; 2],
    pub stick_r: [f32; 2],
    /// A went down this tick, and is held.
    pub a_pressed: bool,
    pub a_held: bool,
}

/// The pointer this tick, CSS px on the tree's surface: what the cursor
/// replaces while it shows.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pointer {
    pub x: f32,
    pub y: f32,
    pub inside: bool,
    pub pressed: bool,
    pub released: bool,
    pub wheel: [f32; 2],
}

/// The cursor between ticks.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct State {
    desc: Option<VirtualCursorDesc>,
    pos: [f32; 2],
    shown: bool,
    placed: bool,
    held: u32,
    down: bool,
    last: [f32; 2],
    over: bool,
}

/// The stick's dead zone, of its travel.
const DEAD: f32 = 0.2;

/// A stick past its dead zone: its direction and how far past, 0 to 1.
fn tilt([x, y]: [f32; 2]) -> Option<([f32; 2], f32)> {
    let len = (x * x + y * y).sqrt();
    (len > DEAD).then(|| ([x / len, y / len], ((len - DEAD) / (1.0 - DEAD)).min(1.0)))
}

impl Tree {
    /// This tick's virtual cursor (`None`: none), declared each tick at
    /// `tps` ticks a second: moves it from `pad` and, while it shows, makes
    /// it `pointer` (its place, A as the button, the right stick as the
    /// wheel) and takes the stick and A from `pad`. Call it before the
    /// pointer, the wheel and the pad's focus moves.
    pub fn virtual_cursor(
        &mut self,
        desc: Option<VirtualCursorDesc>,
        pad: &mut Pad,
        pointer: &mut Pointer,
        tps: u32,
    ) {
        let before = self.vcursor;
        let mut v = self.vcursor;
        let Some(d) = desc else {
            self.vcursor = State::default();
            self.paint |= before.shown;
            return;
        };
        v.desc = Some(d);
        let (w, h) = self.size;
        let real = [pointer.x, pointer.y];
        let (dx, dy) = (real[0] - v.last[0], real[1] - v.last[1]);
        let moved = dx * dx + dy * dy > 0.25;
        let real_moved = pointer.inside && (pointer.pressed || moved);
        if pointer.inside {
            v.last = real;
        }
        let stick = tilt(pad.stick);
        if real_moved && stick.is_none() {
            v.shown = false;
            v.down = false;
        }
        if !v.shown && pointer.inside {
            (v.pos, v.placed) = (real, true);
        }
        if !v.placed {
            (v.pos, v.placed) = ([w / 2.0, h / 2.0], true);
        }
        let tps = tps.max(1) as f32;
        let magnet = d.magnet();
        let over = self.control_box(v.pos);
        match stick {
            Some((dir, amount)) => {
                v.shown = true;
                v.held = v.held.saturating_add(1);
                let ramp_ticks = d.accel_ms() * tps / 1000.0;
                let ramp = if ramp_ticks <= 0.0 {
                    1.0
                } else {
                    ((v.held - 1) as f32 / ramp_ticks).min(1.0)
                };
                // Squared for fine aim near the centre of the travel.
                let mut step = d.speed() * amount * amount * (1.0 / 3.0 + 2.0 / 3.0 * ramp) / tps;
                if over.is_some() {
                    step *= 1.0 - 0.6 * magnet;
                }
                v.pos = [v.pos[0] + dir[0] * step, v.pos[1] + dir[1] * step];
            }
            None => {
                v.held = 0;
                // At rest over a control it settles towards its centre.
                if let (true, Some([x, y, bw, bh])) = (v.shown, over) {
                    let k = 0.25 * magnet;
                    v.pos[0] += (x + bw / 2.0 - v.pos[0]) * k;
                    v.pos[1] += (y + bh / 2.0 - v.pos[1]) * k;
                }
            }
        }
        v.pos = [
            v.pos[0].clamp(0.0, w.max(0.0)),
            v.pos[1].clamp(0.0, h.max(0.0)),
        ];
        v.over = self.control_box(v.pos).is_some();
        if let Some((dir, amount)) = tilt(pad.stick_r)
            && d.scroll() > 0.0
        {
            v.shown = true;
            let s = d.scroll() * amount * amount / tps;
            pointer.wheel = [pointer.wheel[0] + dir[0] * s, pointer.wheel[1] + dir[1] * s];
        }
        if v.shown {
            (pointer.x, pointer.y, pointer.inside) = (v.pos[0], v.pos[1], true);
            if pad.a_pressed {
                pointer.pressed = true;
                v.down = true;
            } else if v.down && !pad.a_held {
                pointer.released = true;
                v.down = false;
            }
            // The stick and A are the cursor's while it shows.
            (pad.a_pressed, pad.a_held, pad.stick) = (false, false, [0.0; 2]);
        } else if stick.is_some() {
            pad.stick = [0.0; 2];
        }
        self.vcursor = v;
        self.paint |= v != before && (v.shown || before.shown);
    }

    /// Where the virtual cursor is and whether it shows; `None` when none
    /// is declared.
    pub fn virtual_cursor_at(&self) -> Option<([f32; 2], bool)> {
        let v = &self.vcursor;
        v.desc.map(|_| (v.pos, v.shown))
    }

    /// The box of the control under `p` (what a press there takes).
    fn control_box(&self, [x, y]: [f32; 2]) -> Option<[f32; 4]> {
        let e = self.hit(x, y);
        (e != super::NONE).then(|| {
            let (bx, by, _, _) = self.abs[e as usize];
            let b = self.arena.box_of(self.els[e as usize].node);
            [bx, by, b.w, b.h]
        })
    }

    /// The cursor as drawn: a ring in the pack's focus colour on a shadow,
    /// a dot of ink at its point; larger over a control, smaller pressed.
    pub(super) fn paint_virtual_cursor(&self, quads: &mut Vec<PaintQuad>) {
        let v = &self.vcursor;
        if v.desc.is_none() || !v.shown {
            return;
        }
        let (w, h) = self.size;
        let [shadow, ink, focus] = self.cursor_colors;
        let mut r: f32 = 11.0;
        if v.over {
            r *= 1.25;
        }
        if v.down {
            r *= 0.85;
        }
        let disc = |r: f32, color: Rgba| PaintQuad {
            rect: [v.pos[0] - r, v.pos[1] - r, 2.0 * r, 2.0 * r],
            clip: [0.0, 0.0, w, h],
            color,
            radius: [r; 4],
            p: [0.0; 4],
            kind: PaintKind::Solid as u32,
            a: 0,
            b: 0,
            xf: 0,
            uv: [0.0; 4],
            wipe: [0.0; 4],
            mask: 0,
            filter: 0,
        };
        quads.extend([
            disc(r + 3.0, shadow),
            disc(r, ink),
            disc(r - 2.5, focus),
            disc(2.5, ink),
        ]);
    }
}
