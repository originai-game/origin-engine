//! A virtual cursor for pads (docs/ui.md §2.1): a screen built for the
//! pointer (a map, a shop grid, a strategy HUD) played on a pad. The left
//! stick moves a cursor that slows and sticks over controls (the magnet),
//! the pad's A presses where it points as a mouse button would, the right
//! stick scrolls under it, and the style draws it. Declared every tick
//! with [`Ui::set_virtual_cursor`]; the real pointer moving hides it and
//! the stick shows it again, so a mouse and a pad share one screen.

use crate::draw::DrawCmd;
use crate::input::{GamepadFrame, InputFrame};
use crate::layout::Rect;
use crate::style::Srgba;
use crate::ui::{HIT_NAV, HIT_OFFSCREEN, HIT_POINTER, HIT_TEXT, Ui};
use og_core::glam::Vec2;

/// How a pad's virtual cursor moves ([`Ui::set_virtual_cursor`]). The zero
/// value is a cursor at 900 units a second, a quarter second from a fine
/// start to full speed, a half-strength magnet and a scroll of 1200 units a
/// second.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct VirtualCursorDesc {
    /// Units a second at full tilt (0: 900).
    pub speed: f32,
    /// Milliseconds a held stick takes from a third of its speed to all of
    /// it, for fine aim first (0: 250; negative: none).
    pub accel: f32,
    /// 0 to 1: how much it slows over a control and, once the stick rests,
    /// pulls to the control's centre (0: 0.5; negative: none).
    pub magnet: f32,
    /// Units a second the right stick scrolls at full tilt (0: 1200;
    /// negative: the right stick does not scroll).
    pub scroll: f32,
}

impl VirtualCursorDesc {
    fn speed(&self) -> f32 {
        if self.speed > 0.0 { self.speed } else { 900.0 }
    }

    fn accel_ms(&self) -> f32 {
        if self.accel > 0.0 {
            self.accel
        } else if self.accel < 0.0 {
            0.0
        } else {
            250.0
        }
    }

    /// Its magnet, `style` (the style's, else the timing table's) when it
    /// leaves it 0.
    fn magnet(&self, style: f32) -> f32 {
        if self.magnet > 0.0 {
            self.magnet.min(1.0)
        } else if self.magnet < 0.0 {
            0.0
        } else {
            style
        }
    }

    fn scroll(&self) -> f32 {
        if self.scroll > 0.0 {
            self.scroll
        } else if self.scroll < 0.0 {
            0.0
        } else {
            1200.0
        }
    }
}

/// The virtual cursor between ticks (in [`Ui::save_state`]).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct VCursorState {
    /// What the last frame declared (`None`: no virtual cursor).
    pub desc: Option<VirtualCursorDesc>,
    /// Where it is, in units.
    pub pos: Vec2,
    /// It is shown (the stick moved it since the real pointer last moved).
    pub shown: bool,
    /// It has had a place (the real pointer's, or the surface's centre).
    pub placed: bool,
    /// Ticks the stick has been held past its dead zone.
    pub held: u32,
    /// The pad's A holds the cursor's press.
    pub down: bool,
    /// The real pointer last tick, to see it move.
    pub last_pointer: Vec2,
    /// The control under it last tick (0: none).
    pub over: u64,
}

/// The stick's dead zone, of its travel.
const DEAD: f32 = 0.2;

/// A stick past its dead zone: its direction and how far past, 0 to 1.
fn tilt(s: Vec2) -> Option<(Vec2, f32)> {
    let len = s.length();
    (len > DEAD).then(|| (s / len, ((len - DEAD) / (1.0 - DEAD)).min(1.0)))
}

impl Ui {
    /// A pad's virtual cursor this tick (docs/ui.md §2.1), declared every
    /// tick like [`Ui::set_players`] and read from the next: pad 0's left
    /// stick moves it instead of the focus, its A presses where it points
    /// (as the mouse's button: drags, sliders and long presses work), its
    /// right stick scrolls under it. The real pointer moving hides it and
    /// the stick shows it again. `None` turns it off.
    pub fn set_virtual_cursor(&mut self, desc: Option<VirtualCursorDesc>) {
        self.interaction.vcursor = desc;
    }

    /// Where the virtual cursor is (units) and whether it is shown: the
    /// last frame's (`None`: no virtual cursor declared).
    pub fn virtual_cursor(&self) -> Option<(Vec2, bool)> {
        let v = &self.interaction.state.vcursor;
        v.desc.map(|_| (v.pos, v.shown))
    }
}

impl Ui {
    /// The topmost control of last frame under `p` (0: none).
    fn control_at(&self, p: Vec2) -> Option<&crate::ui::Hit> {
        self.interaction.state.hits.iter().rev().find(|h| {
            !h.has(HIT_OFFSCREEN) && h.has(HIT_NAV | HIT_POINTER | HIT_TEXT) && h.contains(p.x, p.y)
        })
    }

    /// Moves the virtual cursor and turns pad 0 into the pointer while it
    /// shows: its place, A as the button, the right stick as the wheel.
    /// Run on the input in units before anything reads it.
    pub(crate) fn virtual_cursor_input(&mut self, input: &mut InputFrame, w: f32, h: f32) {
        let mut v = self.interaction.state.vcursor;
        let Some(d) = v.desc else {
            self.interaction.state.vcursor = VCursorState::default();
            return;
        };
        let real = input.pointer;
        let real_moved = input.pointer_inside
            && (input.pressed || (real - v.last_pointer).length_squared() > 0.25);
        if input.pointer_inside {
            v.last_pointer = real;
        }
        let g = input.gamepad;
        let stick = tilt(g.stick);
        if real_moved && stick.is_none() {
            v.shown = false;
            v.down = false;
        }
        if !v.shown && input.pointer_inside {
            v.pos = real;
            v.placed = true;
        }
        if !v.placed {
            v.pos = Vec2::new(w / 2.0, h / 2.0);
            v.placed = true;
        }
        let tps = self.env.ticks_per_second.max(1) as f32;
        let magnet = d.magnet(
            self.env
                .theme
                .art
                .motion
                .magnet
                .unwrap_or(crate::theme::TIMING.magnet),
        );
        let over = self.control_at(v.pos).map(|h| (h.key, h.full));
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
                // Squared for fine aim near the centre of the stick's travel.
                let mut step = d.speed() * amount * amount * (1.0 / 3.0 + 2.0 / 3.0 * ramp) / tps;
                if over.is_some() {
                    step *= 1.0 - 0.6 * magnet;
                }
                v.pos += dir * step;
            }
            None => {
                v.held = 0;
                // At rest over a control it settles towards its centre.
                if let (true, Some((_, r))) = (v.shown, over) {
                    let (cx, cy) = r.center();
                    v.pos += (Vec2::new(cx, cy) - v.pos) * (0.25 * magnet);
                }
            }
        }
        v.pos = v.pos.clamp(Vec2::ZERO, Vec2::new(w.max(0.0), h.max(0.0)));
        v.over = self.control_at(v.pos).map_or(0, |h| h.key);
        // The right stick scrolls under the cursor, which it shows.
        if let Some((dir, amount)) = tilt(g.stick_r)
            && d.scroll() > 0.0
        {
            v.shown = true;
            input.wheel += dir * (d.scroll() * amount * amount / tps);
        }
        if v.shown {
            input.pointer = v.pos;
            input.pointer_inside = true;
            let pad = &mut input.gamepad;
            if pad.pressed & GamepadFrame::A != 0 {
                input.pressed = true;
                input.down = true;
                v.down = true;
            } else if v.down && pad.held & GamepadFrame::A != 0 {
                input.down = true;
            } else if v.down {
                input.released = true;
                input.down = false;
                v.down = false;
            }
            // The stick and A are the cursor's: they neither move nor press
            // the focus while it shows.
            pad.pressed &= !GamepadFrame::A;
            pad.held &= !GamepadFrame::A;
            pad.stick = Vec2::ZERO;
        } else if stick.is_some() {
            input.gamepad.stick = Vec2::ZERO;
        }
        self.interaction.state.vcursor = v;
    }

    /// The virtual cursor as the style draws it: a ring in the focus colour
    /// on a shadow, a dot of ink at its point; larger over a control and
    /// smaller while pressed.
    pub(crate) fn virtual_cursor_commands(&self, cmds: &mut Vec<DrawCmd>) {
        let v = &self.interaction.state.vcursor;
        if v.desc.is_none() || !v.shown {
            return;
        }
        let p = &self.env.theme.palette;
        let screen = Rect::new(0.0, 0.0, self.env.viewport.width, self.env.viewport.height);
        let mut r = 11.0;
        if v.over != 0 {
            r *= 1.25;
        }
        if v.down {
            r *= 0.85;
        }
        let disc = |r: f32, color: Srgba| DrawCmd::Quad {
            rect: Rect::new(v.pos.x - r, v.pos.y - r, 2.0 * r, 2.0 * r),
            radius: r,
            color: color.linear(),
            clip: screen,
        };
        cmds.push(disc(r + 3.0, p.backdrop.fade(0.45)));
        cmds.push(disc(r, p.ink));
        cmds.push(disc(r - 2.5, p.focus));
        cmds.push(disc(2.5, p.ink));
    }
}
