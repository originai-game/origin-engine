//! The `og.Input` resource systems poll and the six input events, with the
//! rules that turn [`InjectedInput`] into both.

use crate::event::{
    GamepadState, InjectedInput, KeyAction, MAX_GAMEPADS, MAX_POINTERS, PointerAction,
    PointerButton,
};
use crate::keys::{KEY_CODES, key_index};
use og_core::glam::Vec2;
use og_core::{Reflect, Str64, Str256};
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.KeyDown")]
pub struct KeyDown {
    /// The physical key code (`KeyboardEvent.code`).
    pub key: Str64,
    /// Whether this is an auto-repeat (a down for a key already held).
    pub repeat: bool,
    /// When it happened, in ticks from the start of the tick that reads it, -1 to 1 (0 when the host gave no time): its world time is `time.elapsed + at * time.dt`.
    pub at: f32,
}

/// A key went up (IDL `input.KeyUp`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.KeyUp")]
pub struct KeyUp {
    /// The physical key code.
    pub key: Str64,
    /// When it happened, in ticks from the start of the tick that reads it, -1 to 1 (0 when the host gave no time): its world time is `time.elapsed + at * time.dt`.
    pub at: f32,
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
    /// Which pointer ([`crate::Pointers`]): 0 the primary, 1 to 4 further touches.
    pub id: u32,
    /// When it happened, in ticks from the start of the tick that reads it, -1 to 1 (0 when the host gave no time): its world time is `time.elapsed + at * time.dt`.
    pub at: f32,
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
    /// Which pointer ([`crate::Pointers`]): 0 the primary, 1 to 4 further touches.
    pub id: u32,
    /// When it happened, in ticks from the start of the tick that reads it, -1 to 1 (0 when the host gave no time): its world time is `time.elapsed + at * time.dt`.
    pub at: f32,
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
    /// Which pointer: 0 the primary, 1 to 4 further touches.
    pub id: u32,
}

/// Text typed or composed (IDL `input.TextInput`), from the page's
/// `beforeinput` and composition events (an engine extension of protocol
/// 5): committed text to insert, or an input method's composition so far.
/// A text longer than 255 bytes arrives as several events in order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.TextInput")]
pub struct TextInput {
    /// The text (UTF-8, whole characters).
    pub text: Str256,
    /// An input method's composition, not yet committed: it replaces the
    /// composition before it (`text` empty ends it without a commit); a
    /// commit ends it too. False: committed text, inserted at the caret.
    pub composing: bool,
    /// Continues the composition of the event before it (a composition
    /// longer than one event).
    pub more: bool,
}

impl TextInput {
    /// `text` as events of at most 255 bytes each, split between characters.
    pub fn split(text: &str, composing: bool) -> Vec<TextInput> {
        let mut out = Vec::new();
        let mut rest = text;
        loop {
            let mut cut = rest.len().min(255);
            while !rest.is_char_boundary(cut) {
                cut -= 1;
            }
            let (head, tail) = rest.split_at(cut);
            out.push(TextInput {
                text: Str256::new(head).expect("at most 255 bytes"),
                composing,
                more: composing && !out.is_empty(),
            });
            if tail.is_empty() {
                return out;
            }
            rest = tail;
        }
    }
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

/// Registers [`Input`] and the six events, then [`Pointers`],
/// [`Gamepad`] and [`TextInput`], in this order: `og.Input`, `og.KeyDown`,
/// `og.KeyUp`, `og.PointerDown`, `og.PointerUp`, `og.PointerMove`,
/// `og.Wheel`, `og.Pointers`, `og.Gamepad`, `og.TextInput`. Idempotent.
pub fn register(world: &mut World) -> og_core::Result<()> {
    world.register::<Input>()?;
    world.register::<KeyDown>()?;
    world.register::<KeyUp>()?;
    world.register::<PointerDown>()?;
    world.register::<PointerUp>()?;
    world.register::<PointerMove>()?;
    world.register::<Wheel>()?;
    world.register::<Pointers>()?;
    world.register::<Gamepad>()?;
    world.register::<TextInput>()?;
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
                        out.push(InputEvent::KeyDown(KeyDown {
                            key,
                            repeat: held,
                            at: 0.0,
                        }));
                    }
                    KeyAction::Up => {
                        if held {
                            self.put(KeySet::Released, index, true);
                            out.push(InputEvent::KeyUp(KeyUp { key, at: 0.0 }));
                        }
                        self.put(KeySet::Down, index, false);
                        self.put(KeySet::ReleaseNext, index, false);
                    }
                }
            }
            // Further touches, pads and text have their own resources and events.
            InjectedInput::Pointer { id, .. } if *id != 0 => {}
            InjectedInput::Gamepad(_) | InjectedInput::Text { .. } => {}
            InjectedInput::Pointer {
                action,
                x,
                y,
                button,
                ..
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
                                id: 0,
                                at: 0.0,
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
                                id: 0,
                                at: 0.0,
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
            out.push(InputEvent::PointerMove(PointerMove { x, y, id: 0 }));
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
                            at: 0.0,
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
                    id: 0,
                    at: 0.0,
                }));
            }
        }
        self.wheel = Vec2::ZERO;
        out
    }
}

/// Every pointer on the play surface at once (multi-touch: a stick and a
/// button held together), at the start of this tick (IDL `input.Pointers`).
/// Pointer 0 is the primary one, the same pointer as [`Input`]'s; ids 1 to
/// 4 are further touches. Masks: bit `id`. Edges last exactly one tick, as
/// [`Input`]'s do; a click holds its pointer until the end of the next tick.
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.Pointers", version = 1)]
pub struct Pointers {
    /// Pointers held down (a touch, or any button of pointer 0).
    #[reflect(name = "heldMask")]
    pub held: u32,
    /// Pointers that went down this tick.
    #[reflect(name = "pressedMask")]
    pub pressed: u32,
    /// Pointers that went up this tick.
    #[reflect(name = "releasedMask")]
    pub released: u32,
    /// Pointers a click releases when this tick ends.
    #[reflect(name = "releaseNextMask")]
    pub release_next: u32,
    /// Where each pointer is (its last event).
    #[reflect(unit = "cssPx")]
    pub position0: Vec2,
    #[reflect(unit = "cssPx")]
    pub position1: Vec2,
    #[reflect(unit = "cssPx")]
    pub position2: Vec2,
    #[reflect(unit = "cssPx")]
    pub position3: Vec2,
    #[reflect(unit = "cssPx")]
    pub position4: Vec2,
    /// Where each pointer went down.
    #[reflect(unit = "cssPx")]
    pub start0: Vec2,
    #[reflect(unit = "cssPx")]
    pub start1: Vec2,
    #[reflect(unit = "cssPx")]
    pub start2: Vec2,
    #[reflect(unit = "cssPx")]
    pub start3: Vec2,
    #[reflect(unit = "cssPx")]
    pub start4: Vec2,
}

impl Pointers {
    fn slot(&mut self, id: usize) -> (&mut Vec2, &mut Vec2) {
        match id {
            0 => (&mut self.position0, &mut self.start0),
            1 => (&mut self.position1, &mut self.start1),
            2 => (&mut self.position2, &mut self.start2),
            3 => (&mut self.position3, &mut self.start3),
            _ => (&mut self.position4, &mut self.start4),
        }
    }

    /// Where pointer `id` is (`Vec2::ZERO` for an id past [`MAX_POINTERS`]).
    pub fn position(&self, id: u32) -> Vec2 {
        [
            self.position0,
            self.position1,
            self.position2,
            self.position3,
            self.position4,
        ]
        .get(id as usize)
        .copied()
        .unwrap_or_default()
    }

    /// Where pointer `id` went down.
    pub fn start(&self, id: u32) -> Vec2 {
        [
            self.start0,
            self.start1,
            self.start2,
            self.start3,
            self.start4,
        ]
        .get(id as usize)
        .copied()
        .unwrap_or_default()
    }

    fn bit(id: u32) -> u32 {
        if (id as usize) < MAX_POINTERS {
            1 << id
        } else {
            0
        }
    }

    /// Whether pointer `id` is held.
    pub fn down(&self, id: u32) -> bool {
        self.held & Self::bit(id) != 0
    }

    /// Whether pointer `id` went down this tick.
    pub fn is_pressed(&self, id: u32) -> bool {
        self.pressed & Self::bit(id) != 0
    }

    /// Whether pointer `id` went up this tick.
    pub fn is_released(&self, id: u32) -> bool {
        self.released & Self::bit(id) != 0
    }

    /// How many pointers are held.
    pub fn count(&self) -> u32 {
        self.held.count_ones()
    }

    /// Applies a pointer event between ticks; returns the events it raises
    /// for pointers 1 to 4 (pointer 0's are [`Input::apply`]'s), `None` for
    /// an id past [`MAX_POINTERS`]. Down and click hold the pointer (button
    /// `none` only moves it), up releases it; every event moves it.
    pub fn apply(&mut self, event: &InjectedInput) -> Option<Vec<InputEvent>> {
        let InjectedInput::Pointer {
            action,
            x,
            y,
            button,
            id,
        } = event
        else {
            return Some(Vec::new());
        };
        if *id as usize >= MAX_POINTERS {
            return None;
        }
        let bit = 1 << id;
        let at = Vec2::new(*x, *y);
        let mut out = Vec::new();
        let raise = *id != 0;
        let (position, _) = self.slot(*id as usize);
        let moved = *position != at;
        *position = at;
        if raise && (moved || *action == PointerAction::Move) {
            out.push(InputEvent::PointerMove(PointerMove {
                x: *x,
                y: *y,
                id: *id,
            }));
        }
        let held = self.held & bit != 0;
        match action {
            PointerAction::Move => {}
            PointerAction::Down | PointerAction::Click if *button != PointerButton::None => {
                if !held {
                    self.pressed |= bit;
                    *self.slot(*id as usize).1 = at;
                    if raise {
                        out.push(InputEvent::PointerDown(PointerDown {
                            x: *x,
                            y: *y,
                            button: button.index(),
                            id: *id,
                            at: 0.0,
                        }));
                    }
                }
                self.held |= bit;
                if *action == PointerAction::Click {
                    self.release_next |= bit;
                } else {
                    self.release_next &= !bit;
                }
            }
            PointerAction::Up if *button != PointerButton::None => {
                if held {
                    self.released |= bit;
                    if raise {
                        out.push(InputEvent::PointerUp(PointerUp {
                            x: *x,
                            y: *y,
                            button: button.index(),
                            id: *id,
                            at: 0.0,
                        }));
                    }
                }
                self.held &= !bit;
                self.release_next &= !bit;
            }
            _ => {}
        }
        Some(out)
    }

    /// The end of a tick: edges expire, clicks release (raising `PointerUp`
    /// for pointers 1 to 4, in id order).
    pub fn end_tick(&mut self) -> Vec<InputEvent> {
        let release = self.held & self.release_next;
        self.pressed = 0;
        self.released = release;
        self.held &= !release;
        self.release_next = 0;
        (1..MAX_POINTERS as u32)
            .filter(|id| release & (1 << id) != 0)
            .map(|id| {
                let p = self.position(id);
                InputEvent::PointerUp(PointerUp {
                    x: p.x,
                    y: p.y,
                    button: PointerButton::Left.index(),
                    id,
                    at: 0.0,
                })
            })
            .collect()
    }
}

/// The first four gamepads in the W3C standard mapping, at the start of this
/// tick (IDL `input.Gamepad`). Buttons are masks over
/// [`crate::GAMEPAD_BUTTONS`] (bit 0 A ... bit 16 home), one word per pad;
/// sticks are -1..1 with y down, triggers 0..1. Edges last exactly one tick;
/// a press and a release between two ticks leave both edges set. A pad's
/// state arrives whole ([`GamepadState`]) whenever the shell sees it change,
/// through the same ring as keys, so it is world state and replays.
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.Gamepad", version = 1)]
pub struct Gamepad {
    /// Connected pads: bit `pad`.
    #[reflect(name = "connectedMask")]
    pub connected: u32,
    /// Held buttons of pads 0 to 3.
    pub buttons0: u32,
    pub buttons1: u32,
    pub buttons2: u32,
    pub buttons3: u32,
    /// Buttons that went down this tick.
    pub pressed0: u32,
    pub pressed1: u32,
    pub pressed2: u32,
    pub pressed3: u32,
    /// Buttons that went up this tick.
    pub released0: u32,
    pub released1: u32,
    pub released2: u32,
    pub released3: u32,
    /// Left sticks.
    #[reflect(name = "leftStick0")]
    pub left_stick0: Vec2,
    #[reflect(name = "leftStick1")]
    pub left_stick1: Vec2,
    #[reflect(name = "leftStick2")]
    pub left_stick2: Vec2,
    #[reflect(name = "leftStick3")]
    pub left_stick3: Vec2,
    /// Right sticks.
    #[reflect(name = "rightStick0")]
    pub right_stick0: Vec2,
    #[reflect(name = "rightStick1")]
    pub right_stick1: Vec2,
    #[reflect(name = "rightStick2")]
    pub right_stick2: Vec2,
    #[reflect(name = "rightStick3")]
    pub right_stick3: Vec2,
    /// Left (x) and right (y) analog triggers.
    pub triggers0: Vec2,
    pub triggers1: Vec2,
    pub triggers2: Vec2,
    pub triggers3: Vec2,
}

/// What a pad reports for one of its analog inputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GamepadAxis {
    LeftX,
    LeftY,
    RightX,
    RightY,
    LeftTrigger,
    RightTrigger,
}

impl GamepadAxis {
    pub const NAMES: [&'static str; 6] = [
        "leftX",
        "leftY",
        "rightX",
        "rightY",
        "leftTrigger",
        "rightTrigger",
    ];

    pub fn from_name(name: &str) -> Option<GamepadAxis> {
        use GamepadAxis::*;
        let all = [LeftX, LeftY, RightX, RightY, LeftTrigger, RightTrigger];
        Self::NAMES.iter().position(|n| *n == name).map(|i| all[i])
    }
}

impl Gamepad {
    fn words(&mut self, pad: usize) -> (&mut u32, &mut u32, &mut u32) {
        match pad {
            0 => (&mut self.buttons0, &mut self.pressed0, &mut self.released0),
            1 => (&mut self.buttons1, &mut self.pressed1, &mut self.released1),
            2 => (&mut self.buttons2, &mut self.pressed2, &mut self.released2),
            _ => (&mut self.buttons3, &mut self.pressed3, &mut self.released3),
        }
    }

    fn analog(&mut self, pad: usize) -> (&mut Vec2, &mut Vec2, &mut Vec2) {
        match pad {
            0 => (
                &mut self.left_stick0,
                &mut self.right_stick0,
                &mut self.triggers0,
            ),
            1 => (
                &mut self.left_stick1,
                &mut self.right_stick1,
                &mut self.triggers1,
            ),
            2 => (
                &mut self.left_stick2,
                &mut self.right_stick2,
                &mut self.triggers2,
            ),
            _ => (
                &mut self.left_stick3,
                &mut self.right_stick3,
                &mut self.triggers3,
            ),
        }
    }

    /// Held buttons, pressed and released this tick of `pad` (zeros past 3).
    pub fn masks(&self, pad: u32) -> (u32, u32, u32) {
        match pad {
            0 => (self.buttons0, self.pressed0, self.released0),
            1 => (self.buttons1, self.pressed1, self.released1),
            2 => (self.buttons2, self.pressed2, self.released2),
            3 => (self.buttons3, self.pressed3, self.released3),
            _ => (0, 0, 0),
        }
    }

    /// Whether `pad` is connected.
    pub fn is_connected(&self, pad: u32) -> bool {
        (pad as usize) < MAX_GAMEPADS && self.connected & (1 << pad) != 0
    }

    /// Whether standard button `button` ([`crate::GAMEPAD_BUTTONS`] index) is held on `pad`.
    pub fn down(&self, button: usize, pad: u32) -> bool {
        button < 32 && self.masks(pad).0 & (1 << button) != 0
    }

    /// Whether it went down this tick.
    pub fn pressed(&self, button: usize, pad: u32) -> bool {
        button < 32 && self.masks(pad).1 & (1 << button) != 0
    }

    /// Whether it went up this tick.
    pub fn released(&self, button: usize, pad: u32) -> bool {
        button < 32 && self.masks(pad).2 & (1 << button) != 0
    }

    /// One analog input of `pad` (0 past pad 3).
    pub fn axis(&self, axis: GamepadAxis, pad: u32) -> f32 {
        let (l, r, t) = match pad {
            0 => (self.left_stick0, self.right_stick0, self.triggers0),
            1 => (self.left_stick1, self.right_stick1, self.triggers1),
            2 => (self.left_stick2, self.right_stick2, self.triggers2),
            3 => (self.left_stick3, self.right_stick3, self.triggers3),
            _ => return 0.0,
        };
        match axis {
            GamepadAxis::LeftX => l.x,
            GamepadAxis::LeftY => l.y,
            GamepadAxis::RightX => r.x,
            GamepadAxis::RightY => r.y,
            GamepadAxis::LeftTrigger => t.x,
            GamepadAxis::RightTrigger => t.y,
        }
    }

    /// Takes a pad's whole state between ticks: edges from the change in its
    /// buttons (a disconnected pad holds nothing); `false` (no change) for a
    /// pad past [`MAX_GAMEPADS`] or a non-finite value.
    pub fn apply(&mut self, state: &GamepadState) -> bool {
        let pad = state.pad as usize;
        if pad >= MAX_GAMEPADS
            || !state
                .axes
                .iter()
                .chain(&state.triggers)
                .all(|v| v.is_finite())
        {
            return false;
        }
        let next = if state.connected { state.buttons } else { 0 };
        let (held, pressed, released) = self.words(pad);
        *pressed |= next & !*held;
        *released |= *held & !next;
        *held = next;
        let clamp = |v: f32, lo: f32| v.clamp(lo, 1.0);
        let (l, r, t) = self.analog(pad);
        if state.connected {
            *l = Vec2::new(clamp(state.axes[0], -1.0), clamp(state.axes[1], -1.0));
            *r = Vec2::new(clamp(state.axes[2], -1.0), clamp(state.axes[3], -1.0));
            *t = Vec2::new(clamp(state.triggers[0], 0.0), clamp(state.triggers[1], 0.0));
            self.connected |= 1 << pad;
        } else {
            *l = Vec2::ZERO;
            *r = Vec2::ZERO;
            *t = Vec2::ZERO;
            self.connected &= !(1 << pad);
        }
        true
    }

    /// The end of a tick: this tick's edges expire.
    pub fn end_tick(&mut self) {
        for pad in 0..MAX_GAMEPADS {
            let (_, pressed, released) = self.words(pad);
            *pressed = 0;
            *released = 0;
        }
    }
}
