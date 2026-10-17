//! Input as the probe injects it: protocol 5's `InjectedInput`, the `Input`
//! resource systems poll, and the [`InputSink`] seam between them.
//!
//! The probe and the command channel deliver every event through one
//! [`InputSink`], the input layer's own path. The default sink,
//! [`WorldInput`], writes the reflected [`Input`] resource of the world, so an
//! injected event is world state: it is in snapshots and in the world hash,
//! and a replay sees it. The input module (M6) replaces it by implementing
//! [`InputSink`] over its own resource and the shell's input ring.

use og_core::glam::Vec2;
use og_core::Reflect;
use og_ecs::World;
use serde_json::{json, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyAction {
    /// Down, then up on the next tick.
    Press,
    /// Held down until an up.
    Down,
    /// Released.
    Up,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerAction {
    /// Down, then up on the next tick, at the point.
    Click,
    Move,
    Down,
    Up,
}

/// A physical pointer button; `None` is a hover move.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerButton {
    None,
    Left,
    Middle,
    Right,
}

impl PointerButton {
    /// Its bit in [`Input::buttons`]: left 1, middle 2, right 4, none 0.
    pub fn bit(self) -> u32 {
        match self {
            PointerButton::None => 0,
            PointerButton::Left => 1,
            PointerButton::Middle => 2,
            PointerButton::Right => 4,
        }
    }

    fn name(self) -> &'static str {
        match self {
            PointerButton::None => "none",
            PointerButton::Left => "left",
            PointerButton::Middle => "middle",
            PointerButton::Right => "right",
        }
    }
}

/// Modifier keys held during a key event.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub alt: bool,
    pub control: bool,
    /// Command on macOS, the Windows key on Windows.
    pub platform: bool,
    pub shift: bool,
}

/// One input event in the game's own vocabulary (protocol 5). Positions are
/// CSS pixels of the play surface, the space `pick` reads; `key` is a
/// `KeyboardEvent.code` name.
#[derive(Clone, Debug, PartialEq)]
pub enum InjectedInput {
    Key {
        key: String,
        action: KeyAction,
        modifiers: Modifiers,
    },
    Pointer {
        action: PointerAction,
        x: f32,
        y: f32,
        button: PointerButton,
    },
    Wheel {
        x: f32,
        y: f32,
        delta_x: f32,
        delta_y: f32,
    },
}

fn number(value: &Value, key: &str) -> Option<f32> {
    let n = value.get(key)?.as_f64()?;
    (n.is_finite() && n.abs() <= f64::from(f32::MAX)).then_some(n as f32)
}

impl InjectedInput {
    /// The event a JSON value describes, or `None` for a value of no known
    /// shape (which the probe answers as a refusal, `false`).
    pub fn from_json(value: &Value) -> Option<InjectedInput> {
        match value.get("kind")?.as_str()? {
            "key" => {
                let key = value.get("key")?.as_str()?;
                if key.is_empty() {
                    return None;
                }
                let action = match value.get("action")?.as_str()? {
                    "press" => KeyAction::Press,
                    "down" => KeyAction::Down,
                    "up" => KeyAction::Up,
                    _ => return None,
                };
                let modifiers = match value.get("modifiers") {
                    None | Some(Value::Null) => Modifiers::default(),
                    Some(m) => Modifiers {
                        alt: m.get("alt")?.as_bool()?,
                        control: m.get("control")?.as_bool()?,
                        platform: m.get("platform")?.as_bool()?,
                        shift: m.get("shift")?.as_bool()?,
                    },
                };
                Some(InjectedInput::Key {
                    key: key.to_owned(),
                    action,
                    modifiers,
                })
            }
            "pointer" => Some(InjectedInput::Pointer {
                action: match value.get("action")?.as_str()? {
                    "click" => PointerAction::Click,
                    "move" => PointerAction::Move,
                    "down" => PointerAction::Down,
                    "up" => PointerAction::Up,
                    _ => return None,
                },
                x: number(value, "x")?,
                y: number(value, "y")?,
                button: match value.get("button")?.as_str()? {
                    "none" => PointerButton::None,
                    "left" => PointerButton::Left,
                    "middle" => PointerButton::Middle,
                    "right" => PointerButton::Right,
                    _ => return None,
                },
            }),
            "wheel" => Some(InjectedInput::Wheel {
                x: number(value, "x")?,
                y: number(value, "y")?,
                delta_x: number(value, "deltaX")?,
                delta_y: number(value, "deltaY")?,
            }),
            _ => None,
        }
    }

    /// The event as protocol 5 JSON.
    pub fn to_json(&self) -> Value {
        match self {
            InjectedInput::Key {
                key,
                action,
                modifiers,
            } => json!({
                "kind": "key",
                "key": key,
                "action": match action { KeyAction::Press => "press", KeyAction::Down => "down", KeyAction::Up => "up" },
                "modifiers": {"alt": modifiers.alt, "control": modifiers.control, "platform": modifiers.platform, "shift": modifiers.shift},
            }),
            InjectedInput::Pointer {
                action,
                x,
                y,
                button,
            } => json!({
                "kind": "pointer",
                "action": match action { PointerAction::Click => "click", PointerAction::Move => "move", PointerAction::Down => "down", PointerAction::Up => "up" },
                "x": x, "y": y, "button": button.name(),
            }),
            InjectedInput::Wheel {
                x,
                y,
                delta_x,
                delta_y,
            } => {
                json!({"kind": "wheel", "x": x, "y": y, "deltaX": delta_x, "deltaY": delta_y})
            }
        }
    }
}

/// The `KeyboardEvent.code` names the default input layer knows, in bit
/// order of [`Input`]'s key sets. A code outside this table is refused.
pub const KEY_CODES: &[&str] = &[
    "KeyA",
    "KeyB",
    "KeyC",
    "KeyD",
    "KeyE",
    "KeyF",
    "KeyG",
    "KeyH",
    "KeyI",
    "KeyJ",
    "KeyK",
    "KeyL",
    "KeyM",
    "KeyN",
    "KeyO",
    "KeyP",
    "KeyQ",
    "KeyR",
    "KeyS",
    "KeyT",
    "KeyU",
    "KeyV",
    "KeyW",
    "KeyX",
    "KeyY",
    "KeyZ",
    "Digit0",
    "Digit1",
    "Digit2",
    "Digit3",
    "Digit4",
    "Digit5",
    "Digit6",
    "Digit7",
    "Digit8",
    "Digit9",
    "F1",
    "F2",
    "F3",
    "F4",
    "F5",
    "F6",
    "F7",
    "F8",
    "F9",
    "F10",
    "F11",
    "F12",
    "ArrowUp",
    "ArrowDown",
    "ArrowLeft",
    "ArrowRight",
    "Space",
    "Enter",
    "Escape",
    "Tab",
    "Backspace",
    "CapsLock",
    "ShiftLeft",
    "ShiftRight",
    "ControlLeft",
    "ControlRight",
    "AltLeft",
    "AltRight",
    "MetaLeft",
    "MetaRight",
    "Minus",
    "Equal",
    "BracketLeft",
    "BracketRight",
    "Backslash",
    "Semicolon",
    "Quote",
    "Backquote",
    "Comma",
    "Period",
    "Slash",
    "IntlBackslash",
    "Home",
    "End",
    "PageUp",
    "PageDown",
    "Insert",
    "Delete",
    "Numpad0",
    "Numpad1",
    "Numpad2",
    "Numpad3",
    "Numpad4",
    "Numpad5",
    "Numpad6",
    "Numpad7",
    "Numpad8",
    "Numpad9",
    "NumpadAdd",
    "NumpadSubtract",
    "NumpadMultiply",
    "NumpadDivide",
    "NumpadDecimal",
    "NumpadEnter",
    "NumpadEqual",
    "NumLock",
    "ScrollLock",
    "PrintScreen",
    "Pause",
    "ContextMenu",
];

/// The bit of a `KeyboardEvent.code` in [`Input`]'s key sets.
pub fn key_index(code: &str) -> Option<usize> {
    KEY_CODES.iter().position(|k| *k == code)
}

/// The key a character is typed with on a US layout, and whether Shift is
/// held: `a` is `KeyA`, `A` is Shift+`KeyA`, `!` is Shift+`Digit1`.
pub fn key_for_char(c: char) -> Option<(String, bool)> {
    if c.is_ascii_alphabetic() {
        return Some((
            format!("Key{}", c.to_ascii_uppercase()),
            c.is_ascii_uppercase(),
        ));
    }
    if c.is_ascii_digit() {
        return Some((format!("Digit{c}"), false));
    }
    let (code, shift) = match c {
        ' ' => ("Space", false),
        '\n' => ("Enter", false),
        '\t' => ("Tab", false),
        '-' => ("Minus", false),
        '_' => ("Minus", true),
        '=' => ("Equal", false),
        '+' => ("Equal", true),
        '[' => ("BracketLeft", false),
        '{' => ("BracketLeft", true),
        ']' => ("BracketRight", false),
        '}' => ("BracketRight", true),
        '\\' => ("Backslash", false),
        '|' => ("Backslash", true),
        ';' => ("Semicolon", false),
        ':' => ("Semicolon", true),
        '\'' => ("Quote", false),
        '"' => ("Quote", true),
        '`' => ("Backquote", false),
        '~' => ("Backquote", true),
        ',' => ("Comma", false),
        '<' => ("Comma", true),
        '.' => ("Period", false),
        '>' => ("Period", true),
        '/' => ("Slash", false),
        '?' => ("Slash", true),
        '!' => ("Digit1", true),
        '@' => ("Digit2", true),
        '#' => ("Digit3", true),
        '$' => ("Digit4", true),
        '%' => ("Digit5", true),
        '^' => ("Digit6", true),
        '&' => ("Digit7", true),
        '*' => ("Digit8", true),
        '(' => ("Digit9", true),
        ')' => ("Digit0", true),
        _ => return None,
    };
    Some((code.to_owned(), shift))
}

/// The input state at the start of a tick, as systems poll it. Edges
/// (`*_pressed`, `*_released`) last exactly one tick; key sets are 128-bit
/// masks over [`KEY_CODES`] split into `_lo` (bits 0–63) and `_hi`.
///
/// The probe's interim `og.Input` resource (design §3.3, IDL `input.Input`)
/// until the input module owns it; read it through the methods.
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.Input", version = 1)]
pub struct Input {
    /// The pointer position.
    #[reflect(unit = "cssPx")]
    pub pointer: Vec2,
    /// 1 once a pointer event has placed the pointer over the play surface.
    pub pointer_inside: u32,
    /// Scroll accumulated during this tick.
    #[reflect(unit = "cssPx")]
    pub wheel: Vec2,
    /// Held pointer buttons (left 1, middle 2, right 4).
    pub buttons: u32,
    /// Buttons that went down this tick.
    pub buttons_pressed: u32,
    /// Buttons that went up this tick.
    pub buttons_released: u32,
    /// Buttons a click releases after the next tick.
    pub buttons_release_next: u32,
    /// Held keys, bits 0–63.
    pub keys_lo: u64,
    /// Held keys, bits 64–127.
    pub keys_hi: u64,
    /// Keys that went down this tick, bits 0–63.
    pub pressed_lo: u64,
    /// Keys that went down this tick, bits 64–127.
    pub pressed_hi: u64,
    /// Keys that went up this tick, bits 0–63.
    pub released_lo: u64,
    /// Keys that went up this tick, bits 64–127.
    pub released_hi: u64,
    /// Keys a press releases after the next tick, bits 0–63.
    pub release_next_lo: u64,
    /// Keys a press releases after the next tick, bits 64–127.
    pub release_next_hi: u64,
}

fn bit(set: (u64, u64), index: usize) -> bool {
    if index < 64 {
        set.0 & (1 << index) != 0
    } else {
        set.1 & (1 << (index - 64)) != 0
    }
}

fn set_bit(lo: &mut u64, hi: &mut u64, index: usize, on: bool) {
    let (word, b) = if index < 64 {
        (lo, index)
    } else {
        (hi, index - 64)
    };
    if on {
        *word |= 1 << b;
    } else {
        *word &= !(1 << b);
    }
}

impl Input {
    /// Whether a key is held.
    pub fn down(&self, code: &str) -> bool {
        key_index(code).is_some_and(|i| bit((self.keys_lo, self.keys_hi), i))
    }

    /// Whether a key went down this tick.
    pub fn pressed(&self, code: &str) -> bool {
        key_index(code).is_some_and(|i| bit((self.pressed_lo, self.pressed_hi), i))
    }

    /// Whether a key went up this tick.
    pub fn released(&self, code: &str) -> bool {
        key_index(code).is_some_and(|i| bit((self.released_lo, self.released_hi), i))
    }

    /// -1, 0 or 1 from two keys.
    pub fn axis(&self, negative: &str, positive: &str) -> f32 {
        f32::from(u8::from(self.down(positive))) - f32::from(u8::from(self.down(negative)))
    }

    pub fn button_down(&self, button: PointerButton) -> bool {
        button.bit() != 0 && self.buttons & button.bit() != 0
    }

    pub fn button_pressed(&self, button: PointerButton) -> bool {
        button.bit() != 0 && self.buttons_pressed & button.bit() != 0
    }

    fn key(&mut self, index: usize, action: KeyAction) {
        match action {
            KeyAction::Down | KeyAction::Press => {
                if !bit((self.keys_lo, self.keys_hi), index) {
                    set_bit(&mut self.pressed_lo, &mut self.pressed_hi, index, true);
                }
                set_bit(&mut self.keys_lo, &mut self.keys_hi, index, true);
                let release = action == KeyAction::Press;
                set_bit(
                    &mut self.release_next_lo,
                    &mut self.release_next_hi,
                    index,
                    release,
                );
            }
            KeyAction::Up => {
                if bit((self.keys_lo, self.keys_hi), index) {
                    set_bit(&mut self.released_lo, &mut self.released_hi, index, true);
                }
                set_bit(&mut self.keys_lo, &mut self.keys_hi, index, false);
                set_bit(
                    &mut self.release_next_lo,
                    &mut self.release_next_hi,
                    index,
                    false,
                );
            }
        }
    }

    /// The end of a tick: edges expire, presses and clicks release.
    fn end_tick(&mut self) {
        self.pressed_lo = 0;
        self.pressed_hi = 0;
        self.released_lo = self.keys_lo & self.release_next_lo;
        self.released_hi = self.keys_hi & self.release_next_hi;
        self.keys_lo &= !self.release_next_lo;
        self.keys_hi &= !self.release_next_hi;
        self.release_next_lo = 0;
        self.release_next_hi = 0;
        self.buttons_pressed = 0;
        self.buttons_released = self.buttons & self.buttons_release_next;
        self.buttons &= !self.buttons_release_next;
        self.buttons_release_next = 0;
        self.wheel = Vec2::ZERO;
    }
}

/// The input layer's own path, which the probe's `input` member and the
/// command channel's pointer and key actions deliver through.
pub trait InputSink {
    /// Called once when the probe is created: register what the sink writes.
    fn attach(&mut self, world: &mut World) -> og_core::Result<()>;
    /// Deliver one event between ticks. `false` is a refusal: the layer
    /// cannot take this event (an unknown key code, a missing resource).
    fn inject(&mut self, world: &mut World, event: &InjectedInput) -> bool;
    /// Called by the probe after every tick it takes: expire edges.
    fn end_tick(&mut self, world: &mut World);
    /// Events the input layer accepted since load.
    fn accepted(&self) -> u32;
}

/// The default [`InputSink`]: writes the world's [`Input`] resource.
#[derive(Debug, Default)]
pub struct WorldInput {
    accepted: u32,
}

impl InputSink for WorldInput {
    fn attach(&mut self, world: &mut World) -> og_core::Result<()> {
        world.register::<Input>()?;
        if !world.has_resource::<Input>() {
            world.insert_resource(Input::default())?;
        }
        Ok(())
    }

    fn inject(&mut self, world: &mut World, event: &InjectedInput) -> bool {
        let Ok(input) = world.resource_mut::<Input>() else {
            return false;
        };
        match event {
            InjectedInput::Key { key, action, .. } => {
                let Some(index) = key_index(key) else {
                    return false;
                };
                input.key(index, *action);
            }
            InjectedInput::Pointer {
                action,
                x,
                y,
                button,
            } => {
                input.pointer = Vec2::new(*x, *y);
                input.pointer_inside = 1;
                let b = button.bit();
                match action {
                    PointerAction::Move => {}
                    PointerAction::Down | PointerAction::Click => {
                        input.buttons_pressed |= b & !input.buttons;
                        input.buttons |= b;
                        if *action == PointerAction::Click {
                            input.buttons_release_next |= b;
                        } else {
                            input.buttons_release_next &= !b;
                        }
                    }
                    PointerAction::Up => {
                        input.buttons_released |= b & input.buttons;
                        input.buttons &= !b;
                        input.buttons_release_next &= !b;
                    }
                }
            }
            InjectedInput::Wheel {
                x,
                y,
                delta_x,
                delta_y,
            } => {
                input.pointer = Vec2::new(*x, *y);
                input.pointer_inside = 1;
                input.wheel += Vec2::new(*delta_x, *delta_y);
            }
        }
        self.accepted += 1;
        true
    }

    fn end_tick(&mut self, world: &mut World) {
        if let Ok(input) = world.resource_mut::<Input>() {
            input.end_tick();
        }
    }

    fn accepted(&self) -> u32 {
        self.accepted
    }
}
