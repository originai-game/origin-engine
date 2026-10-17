//! The `og.Input` resource systems poll and the six input events, with the
//! rules that turn [`InjectedInput`] into both.

use crate::event::{InjectedInput, KeyAction, PointerAction, PointerButton};
use crate::keys::{key_index, KEY_CODES};
use og_core::glam::Vec2;
use og_core::{Reflect, Str64};
use og_ecs::World;

/// The input state at the start of a tick, as systems poll it (IDL
/// `input.Input`). Edges (`pressed`, `released`, the button edges, `wheel`)
/// last exactly one tick: an event delivered between ticks N-1 and N is seen
/// by every system of tick N, and the edges expire when tick N ends.
///
/// The IDL members are `pointer`, `pointerInside`, `wheel` and the methods;
/// the other fields are the state the methods read, laid out for typed
/// access from TypeScript: button masks (left 1, middle 2, right 4) and
/// 128-bit key sets as four `u32` words each over [`KEY_CODES`] (bit `i` is
/// word `i / 32`, bit `i % 32`; `keys0` holds codes 0–31).
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.Input", version = 2)]
pub struct Input {
    /// The pointer position.
    #[reflect(unit = "cssPx")]
    pub pointer: Vec2,
    /// Whether the pointer is over the play surface (set by the first
    /// pointer or wheel event).
    #[reflect(name = "pointerInside")]
    pub pointer_inside: bool,
    /// Scroll accumulated during this tick.
    #[reflect(unit = "cssPx")]
    pub wheel: Vec2,
    /// Held pointer buttons (left 1, middle 2, right 4).
    pub buttons: u32,
    /// Buttons that went down this tick.
    #[reflect(name = "buttonsPressed")]
    pub buttons_pressed: u32,
    /// Buttons that went up this tick.
    #[reflect(name = "buttonsReleased")]
    pub buttons_released: u32,
    /// Buttons a click releases when this tick ends.
    #[reflect(name = "buttonsReleaseNext")]
    pub buttons_release_next: u32,
    /// Held keys, codes 0–31.
    pub keys0: u32,
    /// Held keys, codes 32–63.
    pub keys1: u32,
    /// Held keys, codes 64–95.
    pub keys2: u32,
    /// Held keys, codes 96–127.
    pub keys3: u32,
    /// Keys that went down this tick, codes 0–31.
    pub pressed0: u32,
    pub pressed1: u32,
    pub pressed2: u32,
    pub pressed3: u32,
    /// Keys that went up this tick, codes 0–31.
    pub released0: u32,
    pub released1: u32,
    pub released2: u32,
    pub released3: u32,
    /// Keys a press releases when this tick ends, codes 0–31.
    #[reflect(name = "releaseNext0")]
    pub release_next0: u32,
    #[reflect(name = "releaseNext1")]
    pub release_next1: u32,
    #[reflect(name = "releaseNext2")]
    pub release_next2: u32,
    #[reflect(name = "releaseNext3")]
    pub release_next3: u32,
}

/// One of [`Input`]'s four 128-bit key sets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeySet {
    /// Held keys.
    Down,
    /// Keys that went down this tick.
    Pressed,
    /// Keys that went up this tick.
    Released,
    /// Keys a press releases when this tick ends.
    ReleaseNext,
}

/// A key went down (IDL `input.KeyDown`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(name = "og.KeyDown")]
pub struct KeyDown {
    /// The physical key code (`KeyboardEvent.code`).
    pub key: Str64,
    /// Whether this is an auto-repeat (a down for a key already held).
    pub repeat: bool,
}

/// A key went up (IDL `input.KeyUp`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(name = "og.KeyUp")]
pub struct KeyUp {
    /// The physical key code.
    pub key: Str64,
}

/// A pointer button went down (IDL `input.PointerDown`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.PointerDown")]
pub struct PointerDown {
    /// Horizontal position in CSS pixels.
    #[reflect(unit = "cssPx")]
    pub x: f32,
    /// Vertical position in CSS pixels.
    #[reflect(unit = "cssPx")]
    pub y: f32,
    /// The button ([`PointerButton::index`]).
    #[reflect(variants = ["none", "left", "middle", "right"])]
    pub button: u32,
}

/// A pointer button went up (IDL `input.PointerUp`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.PointerUp")]
pub struct PointerUp {
    /// Horizontal position in CSS pixels.
    #[reflect(unit = "cssPx")]
    pub x: f32,
    /// Vertical position in CSS pixels.
    #[reflect(unit = "cssPx")]
    pub y: f32,
    /// The button ([`PointerButton::index`]).
    #[reflect(variants = ["none", "left", "middle", "right"])]
    pub button: u32,
}

/// The pointer moved (IDL `input.PointerMove`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.PointerMove")]
pub struct PointerMove {
    /// Horizontal position in CSS pixels.
    #[reflect(unit = "cssPx")]
    pub x: f32,
    /// Vertical position in CSS pixels.
    #[reflect(unit = "cssPx")]
    pub y: f32,
}

/// A wheel or trackpad scroll (IDL `input.Wheel`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.Wheel")]
pub struct Wheel {
    /// Horizontal scroll in CSS pixels.
    #[reflect(name = "deltaX", unit = "cssPx")]
    pub delta_x: f32,
    /// Vertical scroll in CSS pixels.
    #[reflect(name = "deltaY", unit = "cssPx")]
    pub delta_y: f32,
}

impl PointerDown {
    pub fn button(&self) -> Option<PointerButton> {
        PointerButton::from_index(self.button)
    }
}

impl PointerUp {
    pub fn button(&self) -> Option<PointerButton> {
        PointerButton::from_index(self.button)
    }
}

/// One input event as [`Input::apply`] and [`Input::end_tick`] produce it,
/// in the order it is sent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InputEvent {
    KeyDown(KeyDown),
    KeyUp(KeyUp),
    PointerDown(PointerDown),
    PointerUp(PointerUp),
    PointerMove(PointerMove),
    Wheel(Wheel),
}

impl InputEvent {
    /// Sends the event into `world` (delivered at the next step).
    pub fn send(self, world: &mut World) -> og_core::Result<()> {
        match self {
            InputEvent::KeyDown(e) => world.send(e),
            InputEvent::KeyUp(e) => world.send(e),
            InputEvent::PointerDown(e) => world.send(e),
            InputEvent::PointerUp(e) => world.send(e),
            InputEvent::PointerMove(e) => world.send(e),
            InputEvent::Wheel(e) => world.send(e),
        }
    }
}

/// Registers [`Input`] and the six events, in this order: `og.Input`,
/// `og.KeyDown`, `og.KeyUp`, `og.PointerDown`, `og.PointerUp`,
/// `og.PointerMove`, `og.Wheel`. Idempotent.
pub fn register(world: &mut World) -> og_core::Result<()> {
    world.register::<Input>()?;
    world.register::<KeyDown>()?;
    world.register::<KeyUp>()?;
    world.register::<PointerDown>()?;
    world.register::<PointerUp>()?;
    world.register::<PointerMove>()?;
    world.register::<Wheel>()?;
    Ok(())
}

fn code_str(index: usize) -> Str64 {
    Str64::new(KEY_CODES[index]).expect("key codes are short")
}

impl Input {
    /// One word (`0..4`) of a key set.
    pub fn word(&self, set: KeySet, word: usize) -> u32 {
        let w = match set {
            KeySet::Down => [self.keys0, self.keys1, self.keys2, self.keys3],
            KeySet::Pressed => [self.pressed0, self.pressed1, self.pressed2, self.pressed3],
            KeySet::Released => [
                self.released0,
                self.released1,
                self.released2,
                self.released3,
            ],
            KeySet::ReleaseNext => [
                self.release_next0,
                self.release_next1,
                self.release_next2,
                self.release_next3,
            ],
        };
        w[word]
    }

    fn word_mut(&mut self, set: KeySet, word: usize) -> &mut u32 {
        match (set, word) {
            (KeySet::Down, 0) => &mut self.keys0,
            (KeySet::Down, 1) => &mut self.keys1,
            (KeySet::Down, 2) => &mut self.keys2,
            (KeySet::Down, _) => &mut self.keys3,
            (KeySet::Pressed, 0) => &mut self.pressed0,
            (KeySet::Pressed, 1) => &mut self.pressed1,
            (KeySet::Pressed, 2) => &mut self.pressed2,
            (KeySet::Pressed, _) => &mut self.pressed3,
            (KeySet::Released, 0) => &mut self.released0,
            (KeySet::Released, 1) => &mut self.released1,
            (KeySet::Released, 2) => &mut self.released2,
            (KeySet::Released, _) => &mut self.released3,
            (KeySet::ReleaseNext, 0) => &mut self.release_next0,
            (KeySet::ReleaseNext, 1) => &mut self.release_next1,
            (KeySet::ReleaseNext, 2) => &mut self.release_next2,
            (KeySet::ReleaseNext, _) => &mut self.release_next3,
        }
    }

    /// Whether key code `index` ([`KEY_CODES`]) is in `set`.
    pub fn contains(&self, set: KeySet, index: usize) -> bool {
        index < 128 && self.word(set, index / 32) & (1 << (index % 32)) != 0
    }

    fn put(&mut self, set: KeySet, index: usize, on: bool) {
        let word = self.word_mut(set, index / 32);
        if on {
            *word |= 1 << (index % 32);
        } else {
            *word &= !(1 << (index % 32));
        }
    }

    /// Whether a key is held. An unknown code is never held.
    pub fn down(&self, code: &str) -> bool {
        key_index(code).is_some_and(|i| self.contains(KeySet::Down, i))
    }

    /// Whether a key went down this tick.
    pub fn pressed(&self, code: &str) -> bool {
        key_index(code).is_some_and(|i| self.contains(KeySet::Pressed, i))
    }

    /// Whether a key went up this tick.
    pub fn released(&self, code: &str) -> bool {
        key_index(code).is_some_and(|i| self.contains(KeySet::Released, i))
    }

    /// -1, 0 or 1 from two keys: `axis("KeyA", "KeyD")`. Both held is 0.
    pub fn axis(&self, negative: &str, positive: &str) -> f32 {
        f32::from(u8::from(self.down(positive))) - f32::from(u8::from(self.down(negative)))
    }

    /// Two axes at once: `(axis(left, right), axis(down, up))`, as for WASD
    /// with `("KeyA", "KeyD", "KeyS", "KeyW")`. Not normalised.
    pub fn axis2(&self, left: &str, right: &str, down: &str, up: &str) -> Vec2 {
        Vec2::new(self.axis(left, right), self.axis(down, up))
    }

    /// Whether a pointer button is held (`None` never is).
    pub fn button_down(&self, button: PointerButton) -> bool {
        button.bit() != 0 && self.buttons & button.bit() != 0
    }

    /// Whether a pointer button went down this tick.
    pub fn button_pressed(&self, button: PointerButton) -> bool {
        button.bit() != 0 && self.buttons_pressed & button.bit() != 0
    }

    /// Whether a pointer button went up this tick.
    pub fn button_released(&self, button: PointerButton) -> bool {
        button.bit() != 0 && self.buttons_released & button.bit() != 0
    }

    /// Applies one event between ticks and returns the events it raises, in
    /// send order; `None` (and no change) for a key code outside
    /// [`KEY_CODES`].
    ///
    /// - key `down`/`press`: held; a pressed edge and `KeyDown` unless it
    ///   was already held (then `KeyDown { repeat: true }`, no edge). `press`
    ///   also releases it when the tick ends.
    /// - key `up`: a released edge and `KeyUp` if it was held; otherwise
    ///   nothing.
    /// - pointer: the pointer moves there (`PointerMove` for a `move`, or
    ///   for any action that changes the position, before the button event);
    ///   `down`/`click` hold the button with a pressed edge and
    ///   `PointerDown` unless it was held; `click` releases it when the tick
    ///   ends; `up` releases it with a released edge and `PointerUp` if it
    ///   was held. Button `none` only moves.
    /// - wheel: the pointer moves there (as above), `wheel` accumulates, and
    ///   `Wheel` is raised.
    ///
    /// A down and an up of the same key or button between two ticks leave
    /// both edges set, so a tap shorter than a tick is still seen.
    pub fn apply(&mut self, event: &InjectedInput) -> Option<Vec<InputEvent>> {
        let mut out = Vec::new();
        match event {
            InjectedInput::Key { key, action, .. } => {
                let index = key_index(key)?;
                let key = code_str(index);
                let held = self.contains(KeySet::Down, index);
                match action {
                    KeyAction::Down | KeyAction::Press => {
                        if !held {
                            self.put(KeySet::Pressed, index, true);
                        }
                        self.put(KeySet::Down, index, true);
                        self.put(KeySet::ReleaseNext, index, *action == KeyAction::Press);
                        out.push(InputEvent::KeyDown(KeyDown { key, repeat: held }));
                    }
                    KeyAction::Up => {
                        if held {
                            self.put(KeySet::Released, index, true);
                            out.push(InputEvent::KeyUp(KeyUp { key }));
                        }
                        self.put(KeySet::Down, index, false);
                        self.put(KeySet::ReleaseNext, index, false);
                    }
                }
            }
            InjectedInput::Pointer {
                action,
                x,
                y,
                button,
            } => {
                self.move_to(*x, *y, *action == PointerAction::Move, &mut out);
                let b = button.bit();
                let held = self.buttons & b != 0;
                match action {
                    PointerAction::Move => {}
                    PointerAction::Down | PointerAction::Click => {
                        if b != 0 && !held {
                            self.buttons_pressed |= b;
                            out.push(InputEvent::PointerDown(PointerDown {
                                x: *x,
                                y: *y,
                                button: button.index(),
                            }));
                        }
                        self.buttons |= b;
                        if *action == PointerAction::Click {
                            self.buttons_release_next |= b;
                        } else {
                            self.buttons_release_next &= !b;
                        }
                    }
                    PointerAction::Up => {
                        if held {
                            self.buttons_released |= b;
                            out.push(InputEvent::PointerUp(PointerUp {
                                x: *x,
                                y: *y,
                                button: button.index(),
                            }));
                        }
                        self.buttons &= !b;
                        self.buttons_release_next &= !b;
                    }
                }
            }
            InjectedInput::Wheel {
                x,
                y,
                delta_x,
                delta_y,
            } => {
                self.move_to(*x, *y, false, &mut out);
                self.wheel += Vec2::new(*delta_x, *delta_y);
                out.push(InputEvent::Wheel(Wheel {
                    delta_x: *delta_x,
                    delta_y: *delta_y,
                }));
            }
        }
        Some(out)
    }

    fn move_to(&mut self, x: f32, y: f32, always: bool, out: &mut Vec<InputEvent>) {
        let to = Vec2::new(x, y);
        if always || !self.pointer_inside || to != self.pointer {
            out.push(InputEvent::PointerMove(PointerMove { x, y }));
        }
        self.pointer = to;
        self.pointer_inside = true;
    }

    /// The end of a tick: this tick's edges and wheel expire; keys a press
    /// holds and buttons a click holds are released, which sets next tick's
    /// released edges and returns their `KeyUp`s (in key-code order) then
    /// `PointerUp`s (left, middle, right) at the pointer.
    pub fn end_tick(&mut self) -> Vec<InputEvent> {
        let mut out = Vec::new();
        for w in 0..4 {
            let release = self.word(KeySet::Down, w) & self.word(KeySet::ReleaseNext, w);
            *self.word_mut(KeySet::Pressed, w) = 0;
            *self.word_mut(KeySet::Released, w) = release;
            *self.word_mut(KeySet::Down, w) &= !release;
            *self.word_mut(KeySet::ReleaseNext, w) = 0;
            for bit in 0..32 {
                if release & (1 << bit) != 0 {
                    let index = w * 32 + bit;
                    if index < KEY_CODES.len() {
                        out.push(InputEvent::KeyUp(KeyUp {
                            key: code_str(index),
                        }));
                    }
                }
            }
        }
        let release = self.buttons & self.buttons_release_next;
        self.buttons_pressed = 0;
        self.buttons_released = release;
        self.buttons &= !release;
        self.buttons_release_next = 0;
        for button in [
            PointerButton::Left,
            PointerButton::Middle,
            PointerButton::Right,
        ] {
            if release & button.bit() != 0 {
                out.push(InputEvent::PointerUp(PointerUp {
                    x: self.pointer.x,
                    y: self.pointer.y,
                    button: button.index(),
                }));
            }
        }
        self.wheel = Vec2::ZERO;
        out
    }
}
