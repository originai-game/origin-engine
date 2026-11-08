//! Controls: the physical inputs an action binding names, by one string
//! each, and how each reads from the tick's input resources.
//!
//! | control | reads |
//! |---|---|
//! | `KeyW`, `Space`, … (any [`crate::KEY_CODES`] name) | a key: 0 or 1 |
//! | `mouse.left`, `mouse.middle`, `mouse.right` | a mouse button: 0 or 1 |
//! | `mouse.x`, `mouse.y`, `mouse.xy` | the mouse's movement this tick, CSS px (og.Pointers.movement) |
//! | `wheel.x`, `wheel.y` | the wheel this tick, in notches (100 CSS px) |
//! | `pad.a` … `pad.home` (GamepadButton names but the triggers) | a pad button: 0 or 1, any pad |
//! | `pad.leftX`, `pad.leftY`, `pad.rightX`, `pad.rightY` | a stick axis, -1..1 past its dead zone |
//! | `pad.leftTrigger`, `pad.rightTrigger` | a trigger, 0..1 past its dead zone |
//! | `pad.leftXY`, `pad.rightXY` | a stick, two axes with a radial dead zone |
//! | `pad.dpad` | the d-pad's four buttons as two axes |
//! | `touch` | a touch held that started in the binding's zone: 0 or 1 |
//! | `touch.xy` | a touch stick: the drag of the touch that started in the zone, over the binding's radius |
//!
//! Every y is up-positive (W, the stick pushed up, the mouse moved up,
//! the wheel scrolled up, a touch dragged up: +1), so 2D actions read as
//! a plane; og.Gamepad's raw sticks stay y down. A one-axis control
//! takes a `+` or `-` suffix for its half (`pad.leftX+`, `wheel.y-`):
//! 0..1, a direction as a button.

use crate::event::{GAMEPAD_BUTTONS, MAX_GAMEPADS, MAX_POINTERS, PointerButton};
use crate::glyph::{Device, Glyph, key_glyph, pad_glyph};
use crate::keys::{KEY_CODES, key_index};
use crate::resource::{Gamepad, GamepadAxis, Input, KeySet, Pointers};
use og_core::glam::Vec2;

/// A one-axis analog source.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Axis {
    MouseX,
    MouseY,
    WheelX,
    WheelY,
    Pad(GamepadAxis),
}

/// Which part of a one-axis source a control reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Half {
    /// The whole axis, signed.
    Full,
    /// Its positive half, 0..1.
    Positive,
    /// Its negative half, 0..1.
    Negative,
}

/// A two-axis source.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Axis2 {
    Mouse,
    PadLeft,
    PadRight,
    Dpad,
    Touch,
}

/// One physical input, as a binding names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Control {
    /// A key: its index in [`KEY_CODES`].
    Key(u16),
    Mouse(PointerButton),
    /// A pad button: its [`GAMEPAD_BUTTONS`] index (never the triggers,
    /// which are [`Axis::Pad`]).
    PadButton(u8),
    /// A touch held on the surface.
    Touch,
    Axis(Axis, Half),
    Axis2(Axis2),
}

/// How many dimensions a control gives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// 0..1: a key, a button, a touch, a half axis, a trigger.
    Digital,
    /// -1..1 (or unbounded: the mouse, the wheel).
    Signed,
    /// Two axes.
    Plane,
}

const AXIS_NAMES: [(&str, Axis); 10] = [
    ("mouse.x", Axis::MouseX),
    ("mouse.y", Axis::MouseY),
    ("wheel.x", Axis::WheelX),
    ("wheel.y", Axis::WheelY),
    ("pad.leftX", Axis::Pad(GamepadAxis::LeftX)),
    ("pad.leftY", Axis::Pad(GamepadAxis::LeftY)),
    ("pad.rightX", Axis::Pad(GamepadAxis::RightX)),
    ("pad.rightY", Axis::Pad(GamepadAxis::RightY)),
    ("pad.leftTrigger", Axis::Pad(GamepadAxis::LeftTrigger)),
    ("pad.rightTrigger", Axis::Pad(GamepadAxis::RightTrigger)),
];

const AXIS2_NAMES: [(&str, Axis2); 5] = [
    ("mouse.xy", Axis2::Mouse),
    ("pad.leftXY", Axis2::PadLeft),
    ("pad.rightXY", Axis2::PadRight),
    ("pad.dpad", Axis2::Dpad),
    ("touch.xy", Axis2::Touch),
];

/// The pad buttons that are read as axes instead (the analog triggers).
const TRIGGER_BUTTONS: [usize; 2] = [6, 7];

impl Control {
    /// The control a name names; `None` for a name that names none.
    pub fn parse(name: &str) -> Option<Control> {
        if let Some(i) = key_index(name) {
            return Some(Control::Key(i as u16));
        }
        if name == "touch" {
            return Some(Control::Touch);
        }
        if let Some((_, a)) = AXIS2_NAMES.iter().find(|(n, _)| *n == name) {
            return Some(Control::Axis2(*a));
        }
        let (base, half) = match name.as_bytes().last() {
            Some(b'+') => (&name[..name.len() - 1], Half::Positive),
            Some(b'-') => (&name[..name.len() - 1], Half::Negative),
            _ => (name, Half::Full),
        };
        if let Some((_, a)) = AXIS_NAMES.iter().find(|(n, _)| *n == base) {
            // A trigger is 0..1 already: it has no negative half.
            if matches!(a, Axis::Pad(t) if t.is_trigger()) && half == Half::Negative {
                return None;
            }
            return Some(Control::Axis(*a, half));
        }
        if half != Half::Full {
            return None;
        }
        if let Some(b) = name.strip_prefix("mouse.") {
            return match PointerButton::from_name(b) {
                Some(PointerButton::None) | None => None,
                Some(b) => Some(Control::Mouse(b)),
            };
        }
        if let Some(b) = name.strip_prefix("pad.") {
            let i = GAMEPAD_BUTTONS.iter().position(|n| *n == b)?;
            return (!TRIGGER_BUTTONS.contains(&i)).then_some(Control::PadButton(i as u8));
        }
        None
    }

    /// Its name, as [`Control::parse`] reads it.
    pub fn name(&self) -> String {
        match *self {
            Control::Key(i) => KEY_CODES[i as usize].to_string(),
            Control::Mouse(b) => format!("mouse.{}", b.name()),
            Control::PadButton(i) => format!("pad.{}", GAMEPAD_BUTTONS[i as usize]),
            Control::Touch => "touch".into(),
            Control::Axis(a, half) => {
                let base = AXIS_NAMES
                    .iter()
                    .find(|(_, x)| *x == a)
                    .map_or("", |(n, _)| *n);
                match half {
                    Half::Full => base.to_string(),
                    Half::Positive => format!("{base}+"),
                    Half::Negative => format!("{base}-"),
                }
            }
            Control::Axis2(a) => AXIS2_NAMES
                .iter()
                .find(|(_, x)| *x == a)
                .map_or(String::new(), |(n, _)| (*n).to_string()),
        }
    }

    /// What it reads: 0..1, signed, or a plane.
    pub fn shape(&self) -> Shape {
        match self {
            Control::Axis2(_) => Shape::Plane,
            Control::Axis(Axis::Pad(a), Half::Full) if a.is_trigger() => Shape::Digital,
            Control::Axis(_, Half::Full) => Shape::Signed,
            _ => Shape::Digital,
        }
    }

    /// The device class it belongs to: keyboard and mouse, a pad
    /// ([`Device::Gamepad`]) or touch.
    pub fn class(&self) -> Device {
        match self {
            Control::Key(_)
            | Control::Mouse(_)
            | Control::Axis(Axis::MouseX | Axis::MouseY | Axis::WheelX | Axis::WheelY, _)
            | Control::Axis2(Axis2::Mouse) => Device::KeyboardMouse,
            Control::Touch | Control::Axis2(Axis2::Touch) => Device::Touch,
            _ => Device::Gamepad,
        }
    }

    /// Its glyph on `device` (a pad family for a pad control; any other
    /// device draws a pad control as an Xbox pad's).
    pub fn glyph(&self, device: Device) -> Glyph {
        let family = if device.is_pad() {
            device
        } else {
            Device::Xbox
        };
        let prefix = match family {
            Device::Xbox => "xbox",
            Device::PlayStation => "ps",
            Device::Switch => "switch",
            _ => "pad",
        };
        let g = |icon: String, text: &str| Glyph {
            icon,
            text: text.to_string(),
        };
        let arrow = |half: Half, vertical: bool| match (half, vertical) {
            (Half::Full, _) => "",
            (Half::Positive, false) => " →",
            (Half::Negative, false) => " ←",
            (Half::Positive, true) => " ↑",
            (Half::Negative, true) => " ↓",
        };
        match *self {
            Control::Key(i) => key_glyph(KEY_CODES[i as usize]),
            Control::Mouse(b) => g(
                format!("mouse.{}", b.name()),
                match b {
                    PointerButton::Left => "LMB",
                    PointerButton::Middle => "MMB",
                    _ => "RMB",
                },
            ),
            Control::PadButton(i) => pad_glyph(family, GAMEPAD_BUTTONS[i as usize]),
            Control::Touch => g("touch.tap".into(), "Tap"),
            Control::Axis(Axis::MouseX, h) => {
                g("mouse.move".into(), &format!("Mouse{}", arrow(h, false)))
            }
            Control::Axis(Axis::MouseY, h) => {
                g("mouse.move".into(), &format!("Mouse{}", arrow(h, true)))
            }
            Control::Axis(Axis::WheelX, h) => {
                g("mouse.wheel".into(), &format!("Wheel{}", arrow(h, false)))
            }
            Control::Axis(Axis::WheelY, h) => {
                g("mouse.wheel".into(), &format!("Wheel{}", arrow(h, true)))
            }
            Control::Axis(Axis::Pad(a), h) => match a {
                GamepadAxis::LeftTrigger => pad_glyph(family, "leftTrigger"),
                GamepadAxis::RightTrigger => pad_glyph(family, "rightTrigger"),
                GamepadAxis::LeftX | GamepadAxis::LeftY => g(
                    format!("{prefix}.stickL"),
                    &format!("L Stick{}", arrow(h, a == GamepadAxis::LeftY)),
                ),
                GamepadAxis::RightX | GamepadAxis::RightY => g(
                    format!("{prefix}.stickR"),
                    &format!("R Stick{}", arrow(h, a == GamepadAxis::RightY)),
                ),
            },
            Control::Axis2(Axis2::Mouse) => g("mouse.move".into(), "Mouse"),
            Control::Axis2(Axis2::PadLeft) => g(format!("{prefix}.stickL"), "L Stick"),
            Control::Axis2(Axis2::PadRight) => g(format!("{prefix}.stickR"), "R Stick"),
            Control::Axis2(Axis2::Dpad) => g(format!("{prefix}.dpad"), "D-pad"),
            Control::Axis2(Axis2::Touch) => g("touch.stick".into(), "Stick"),
        }
    }
}

/// A pad's dead zones.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PadDeadzone {
    /// Sticks: radial for a stick's two axes, per axis for one.
    pub stick: f32,
    /// Triggers.
    pub trigger: f32,
}

impl PadDeadzone {
    pub const DEFAULT: PadDeadzone = PadDeadzone {
        stick: 0.15,
        trigger: 0.05,
    };
}

impl Default for PadDeadzone {
    fn default() -> Self {
        PadDeadzone::DEFAULT
    }
}

/// `v` past a dead zone `d`: 0 inside it, the rest rescaled to 0..1.
pub fn deadzone(v: f32, d: f32) -> f32 {
    let a = v.abs();
    if a <= d || d >= 1.0 {
        0.0
    } else {
        v.signum() * ((a - d) / (1.0 - d)).min(1.0)
    }
}

/// `v` past a radial dead zone `d`.
pub fn deadzone2(v: Vec2, d: f32) -> Vec2 {
    let len = v.length();
    if len <= d || d >= 1.0 || len == 0.0 {
        Vec2::ZERO
    } else {
        v / len * ((len - d) / (1.0 - d)).min(1.0)
    }
}

/// The play surface touch zones are fractions of.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Surface {
    /// CSS pixels; 0 when the host has not said (a zone is then all of it).
    pub width: f32,
    pub height: f32,
    /// The surface's pointer is coarse (a touch screen): pointer 0 is a
    /// touch, not the mouse.
    pub coarse: bool,
}

/// The tick's input as controls read it, with the session's settings.
pub struct Reader<'a> {
    pub input: &'a Input,
    pub pointers: &'a Pointers,
    pub pads: &'a Gamepad,
    pub deadzones: &'a [PadDeadzone; MAX_GAMEPADS],
    pub surface: Surface,
}

/// What an on-screen control reads as ([`ControlZone::kind`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ZoneKind {
    /// A stick: the finger's offset from the zone's centre over its
    /// radius (half its smaller side), y up, past the dead zone.
    #[default]
    Stick,
    /// A d-pad: the nearest of eight directions from the centre, each
    /// part -1, 0 or 1 (a diagonal holds both), nothing in its middle.
    Pad,
    /// A button: 1 while a press that started on it is held.
    Button,
}

/// An on-screen control the UI lays out (a document's `<joystick>`,
/// `<dpad>` or touch `<button>`): an input source of one action, read like
/// a binding from the presses that started on it. Its rect is the UI's,
/// the play surface's CSS px; the UI hands the map its zones each frame
/// ([`crate::ActionMap::set_zones`]), as the host hands it the surface.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ControlZone {
    pub action: String,
    pub kind: ZoneKind,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    /// A stick's dead zone as a ratio of its radius; 0: 0.15. Past it the
    /// value is rescaled to rise from 0 (as `og.ui.joystick` answers).
    pub dead_zone: f32,
}

impl ControlZone {
    fn contains(&self, p: Vec2) -> bool {
        p.x >= self.x && p.y >= self.y && p.x < self.x + self.width && p.y < self.y + self.height
    }

    fn centre(&self) -> Vec2 {
        Vec2::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}

/// One control's reading this tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Reading {
    /// x for a one-axis control.
    pub value: Vec2,
    /// A digital control went down this tick (the resource's edge).
    pub pressed: bool,
    /// A digital control went up this tick.
    pub released: bool,
    /// What gave it (`None`: it reads nothing).
    pub device: Option<Device>,
}

/// Where a touch binding's touches must start: fractions of the surface.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Zone {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Zone {
    /// Whether a point (CSS px) lies in it; a zone of no size is the whole surface.
    pub fn contains(&self, p: Vec2, surface: Surface) -> bool {
        if self.width <= 0.0 || self.height <= 0.0 || surface.width <= 0.0 || surface.height <= 0.0
        {
            return true;
        }
        let (u, v) = (p.x / surface.width, p.y / surface.height);
        u >= self.x && u < self.x + self.width && v >= self.y && v < self.y + self.height
    }
}

/// Binding parameters a control's reading depends on.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ReadOptions {
    /// Overrides the pad's dead zone when above 0.
    pub deadzone: f32,
    pub zone: Zone,
    /// A touch stick's full deflection, CSS px (0: 64).
    pub radius: f32,
}

impl Reader<'_> {
    fn pad_family(&self, pad: u32) -> Device {
        self.pads.device(pad).unwrap_or(Device::Xbox)
    }

    fn touch_ids(&self) -> impl Iterator<Item = u32> + '_ {
        let first = if self.surface.coarse { 0 } else { 1 };
        first..MAX_POINTERS as u32
    }

    /// An on-screen control's reading: the first held press that started
    /// on it (the mouse's too on a fine pointer, so it works at a desk).
    pub fn read_zone(&self, z: &ControlZone) -> Reading {
        let mut r = Reading::default();
        let Some(id) = (0..MAX_POINTERS as u32)
            .find(|&id| self.pointers.down(id) && z.contains(self.pointers.start(id)))
        else {
            return r;
        };
        let radius = (z.width.min(z.height) / 2.0).max(1.0);
        let d = self.pointers.position(id) - z.centre();
        let d = Vec2::new(d.x, -d.y) / radius;
        r.value = match z.kind {
            ZoneKind::Button => Vec2::new(1.0, 0.0),
            ZoneKind::Stick => {
                let len = d.length();
                let dz = if z.dead_zone > 0.0 {
                    z.dead_zone.min(0.95)
                } else {
                    0.15
                };
                if len <= dz {
                    Vec2::ZERO
                } else {
                    d / len * ((len.min(1.0) - dz) / (1.0 - dz))
                }
            }
            ZoneKind::Pad => {
                // The middle is no direction (the UI's pad: 0.35 of an arm,
                // a third of the pad); outside it, the nearest of eight.
                if d.length() <= 0.35 / 1.5 {
                    Vec2::ZERO
                } else {
                    // An axis counts past tan 22.5° of the other: the
                    // nearest of eight 45° sectors, without trigonometry.
                    const TAN_22_5: f32 = 0.414_213_57;
                    let part = |a: f32, b: f32| {
                        if a.abs() > TAN_22_5 * b.abs() {
                            a.signum()
                        } else {
                            0.0
                        }
                    };
                    Vec2::new(part(d.x, d.y), part(d.y, d.x))
                }
            }
        };
        r.pressed = self.pointers.is_pressed(id);
        r.device = Some(if id == 0 && !self.surface.coarse {
            Device::KeyboardMouse
        } else {
            Device::Touch
        });
        r
    }

    /// The analog value of a pad axis on `pad`, past its dead zone, y up.
    fn pad_axis(&self, a: GamepadAxis, pad: u32, o: &ReadOptions) -> f32 {
        let raw = self.pads.axis(a, pad);
        let dz = self.deadzones[pad as usize];
        let d = if o.deadzone > 0.0 {
            o.deadzone
        } else if a.is_trigger() {
            dz.trigger
        } else {
            dz.stick
        };
        let v = deadzone(raw, d);
        if matches!(a, GamepadAxis::LeftY | GamepadAxis::RightY) {
            -v
        } else {
            v
        }
    }

    /// One control's reading. Over several pads, the one that reads most.
    pub fn read(&self, control: Control, o: &ReadOptions) -> Reading {
        let mut r = Reading::default();
        match control {
            Control::Key(i) => {
                let i = i as usize;
                let on = self.input.contains(KeySet::Down, i);
                r.value.x = f32::from(u8::from(on));
                r.pressed = self.input.contains(KeySet::Pressed, i);
                r.released = self.input.contains(KeySet::Released, i);
                if on || r.pressed {
                    r.device = Some(Device::KeyboardMouse);
                }
            }
            Control::Mouse(b) => {
                let bit = b.bit();
                let on = self.input.buttons & bit != 0;
                r.value.x = f32::from(u8::from(on));
                r.pressed = self.input.buttons_pressed & bit != 0;
                r.released = self.input.buttons_released & bit != 0;
                if on || r.pressed {
                    r.device = Some(Device::KeyboardMouse);
                }
            }
            Control::PadButton(i) => {
                for pad in 0..MAX_GAMEPADS as u32 {
                    let (held, pressed, released) = self.pads.masks(pad);
                    let bit = 1u32 << i;
                    if held & bit != 0 || pressed & bit != 0 {
                        r.device.get_or_insert(self.pad_family(pad));
                    }
                    if held & bit != 0 {
                        r.value.x = 1.0;
                    }
                    r.pressed |= pressed & bit != 0;
                    r.released |= released & bit != 0;
                }
            }
            Control::Touch => {
                for id in self.touch_ids() {
                    if !o.zone.contains(self.pointers.start(id), self.surface) {
                        continue;
                    }
                    if self.pointers.down(id) {
                        r.value.x = 1.0;
                    }
                    r.pressed |= self.pointers.is_pressed(id);
                    r.released |= self.pointers.is_released(id);
                }
                if r.value.x > 0.0 || r.pressed {
                    r.device = Some(Device::Touch);
                }
            }
            Control::Axis(axis, half) => {
                let (v, device) = match axis {
                    Axis::MouseX => (self.pointers.movement.x, Device::KeyboardMouse),
                    Axis::MouseY => (-self.pointers.movement.y, Device::KeyboardMouse),
                    Axis::WheelX => (self.input.wheel.x / 100.0, Device::KeyboardMouse),
                    Axis::WheelY => (-self.input.wheel.y / 100.0, Device::KeyboardMouse),
                    Axis::Pad(a) => {
                        let mut best = (0.0f32, Device::Xbox);
                        for pad in 0..MAX_GAMEPADS as u32 {
                            if !self.pads.is_connected(pad) {
                                continue;
                            }
                            let v = match half {
                                Half::Full => self.pad_axis(a, pad, o),
                                Half::Positive => self.pad_axis(a, pad, o).max(0.0),
                                Half::Negative => (-self.pad_axis(a, pad, o)).max(0.0),
                            };
                            if v.abs() > best.0.abs() {
                                best = (v, self.pad_family(pad));
                            }
                        }
                        r.value.x = best.0;
                        if best.0 != 0.0 {
                            r.device = Some(best.1);
                        }
                        return r;
                    }
                };
                r.value.x = match half {
                    Half::Full => v,
                    Half::Positive => v.max(0.0),
                    Half::Negative => (-v).max(0.0),
                };
                if r.value.x != 0.0 {
                    r.device = Some(device);
                }
            }
            Control::Axis2(a) => match a {
                Axis2::Mouse => {
                    let m = self.pointers.movement;
                    r.value = Vec2::new(m.x, -m.y);
                    if r.value != Vec2::ZERO {
                        r.device = Some(Device::KeyboardMouse);
                    }
                }
                Axis2::PadLeft | Axis2::PadRight => {
                    let (ax, ay) = if a == Axis2::PadLeft {
                        (GamepadAxis::LeftX, GamepadAxis::LeftY)
                    } else {
                        (GamepadAxis::RightX, GamepadAxis::RightY)
                    };
                    for pad in 0..MAX_GAMEPADS as u32 {
                        if !self.pads.is_connected(pad) {
                            continue;
                        }
                        let raw = Vec2::new(self.pads.axis(ax, pad), -self.pads.axis(ay, pad));
                        let d = if o.deadzone > 0.0 {
                            o.deadzone
                        } else {
                            self.deadzones[pad as usize].stick
                        };
                        let v = deadzone2(raw, d);
                        if v.length() > r.value.length() {
                            r.value = v;
                            r.device = Some(self.pad_family(pad));
                        }
                    }
                }
                Axis2::Dpad => {
                    let part = |b: u8| self.read(Control::PadButton(b), o);
                    let (up, down, left, right) = (part(12), part(13), part(14), part(15));
                    r.value = Vec2::new(right.value.x - left.value.x, up.value.x - down.value.x);
                    r.device = [up, down, left, right].iter().find_map(|p| p.device);
                }
                Axis2::Touch => {
                    let radius = if o.radius > 0.0 { o.radius } else { 64.0 };
                    for id in self.touch_ids() {
                        let start = self.pointers.start(id);
                        if !self.pointers.down(id) || !o.zone.contains(start, self.surface) {
                            continue;
                        }
                        let d = self.pointers.position(id) - start;
                        let v = Vec2::new(d.x, -d.y) / radius;
                        r.value = if v.length() > 1.0 { v.normalize() } else { v };
                        r.device = Some(Device::Touch);
                        break;
                    }
                }
            },
        }
        r
    }
}
