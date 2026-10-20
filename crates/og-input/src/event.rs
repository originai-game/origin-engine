//! Protocol 5's `InjectedInput` (IDL `input.InjectedInput`): one input event
//! in the game's own vocabulary, as the shell's ring, the probe's `input`
//! member and the command channel deliver it. The same shape as og-probe's
//! interim `input` module, so the probe can re-export these types.

use serde_json::{Value, json};

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
    /// Every button, in variant order (the IDL's `PointerButton`).
    pub const ALL: [PointerButton; 4] = [
        PointerButton::None,
        PointerButton::Left,
        PointerButton::Middle,
        PointerButton::Right,
    ];

    /// The variant names, by index: the `variants` of the events'
    /// `button` enum field.
    pub const NAMES: [&'static str; 4] = ["none", "left", "middle", "right"];

    /// Its bit in [`crate::Input::buttons`]: left 1, middle 2, right 4, none 0.
    pub fn bit(self) -> u32 {
        match self {
            PointerButton::None => 0,
            PointerButton::Left => 1,
            PointerButton::Middle => 2,
            PointerButton::Right => 4,
        }
    }

    /// Its variant index (none 0, left 1, middle 2, right 3), the value of
    /// an event's `button` field.
    pub fn index(self) -> u32 {
        match self {
            PointerButton::None => 0,
            PointerButton::Left => 1,
            PointerButton::Middle => 2,
            PointerButton::Right => 3,
        }
    }

    /// The button of a variant index.
    pub fn from_index(index: u32) -> Option<PointerButton> {
        PointerButton::ALL.get(index as usize).copied()
    }

    /// Its IDL name: `none`, `left`, `middle`, `right`.
    pub fn name(self) -> &'static str {
        PointerButton::NAMES[self.index() as usize]
    }

    /// The button an IDL name names.
    pub fn from_name(name: &str) -> Option<PointerButton> {
        PointerButton::NAMES
            .iter()
            .position(|n| *n == name)
            .map(|i| PointerButton::ALL[i])
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

/// The most pointers held at once ([`crate::Pointers`]): ids 0 to 4.
pub const MAX_POINTERS: usize = 5;
/// The most gamepads read at once ([`crate::Gamepad`]): pads 0 to 3.
pub const MAX_GAMEPADS: usize = 4;
/// Standard-mapping buttons (W3C Gamepad `standard` layout), by index.
pub const GAMEPAD_BUTTONS: [&str; 17] = [
    "a",
    "b",
    "x",
    "y",
    "leftBumper",
    "rightBumper",
    "leftTrigger",
    "rightTrigger",
    "back",
    "start",
    "leftStick",
    "rightStick",
    "up",
    "down",
    "left",
    "right",
    "home",
];

/// The index of a standard-mapping button name ([`GAMEPAD_BUTTONS`]).
pub fn gamepad_button(name: &str) -> Option<usize> {
    GAMEPAD_BUTTONS.iter().position(|n| *n == name)
}

/// One pad's whole state, as the shell polls it on the display frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GamepadState {
    /// Which pad (0 to [`MAX_GAMEPADS`] - 1).
    pub pad: u32,
    pub connected: bool,
    /// Held buttons: bit `i` is [`GAMEPAD_BUTTONS`]`[i]`.
    pub buttons: u32,
    /// Left stick x, y, right stick x, y; each -1..1, y down (the Gamepad API's).
    pub axes: [f32; 4],
    /// The analog left and right triggers, 0..1.
    pub triggers: [f32; 2],
}

/// What an assistive technology asks of one UI control (the `a11y` kind):
/// the web shell's mirror of the accessibility tree sends it when a screen
/// reader focuses, activates or adjusts a mirrored node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum A11yAct {
    /// Move the UI's focus to the control (with the focus ring).
    Focus,
    /// Press it, as Enter on it would.
    Activate,
    /// Step an adjustable control (a slider, a stepper) up.
    Increment,
    /// Step it down.
    Decrement,
}

impl A11yAct {
    /// Every action, in variant order (the IDL's `A11yActionKind`).
    pub const ALL: [A11yAct; 4] = [
        A11yAct::Focus,
        A11yAct::Activate,
        A11yAct::Increment,
        A11yAct::Decrement,
    ];
    /// The variant names, by index.
    pub const NAMES: [&'static str; 4] = ["focus", "activate", "increment", "decrement"];

    /// Its variant index.
    pub fn index(self) -> u32 {
        A11yAct::ALL.iter().position(|a| *a == self).unwrap_or(0) as u32
    }

    /// The action of a variant index.
    pub fn from_index(index: u32) -> Option<A11yAct> {
        A11yAct::ALL.get(index as usize).copied()
    }

    /// Its IDL name.
    pub fn name(self) -> &'static str {
        A11yAct::NAMES[self.index() as usize]
    }

    /// The action an IDL name names.
    pub fn from_name(name: &str) -> Option<A11yAct> {
        A11yAct::NAMES
            .iter()
            .position(|n| *n == name)
            .map(|i| A11yAct::ALL[i])
    }
}

/// One input event in the game's own vocabulary (protocol 5, plus four
/// engine extensions: a pointer `id` for multi-touch, the `gamepad` kind,
/// the `text` kind and the `a11y` kind).
/// Positions are CSS pixels of the play surface, the space `pick` reads;
/// `key` is a `KeyboardEvent.code` name.
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
        /// Which pointer: 0 the primary (the mouse, the first touch, what
        /// protocol 5 means), 1 to [`MAX_POINTERS`] - 1 further touches.
        id: u32,
    },
    Wheel {
        x: f32,
        y: f32,
        delta_x: f32,
        delta_y: f32,
    },
    /// A pad's state, whole (the shell sends one when it changes).
    Gamepad(GamepadState),
    /// Typed text (an engine extension): what the page's `beforeinput`
    /// inserted (a key's character, a paste, an IME's committed text), or,
    /// with `composing`, an input method's composition so far (the
    /// underlined text not yet committed; empty ends it). Raises
    /// [`crate::TextInput`] events; the UI's focused field takes them.
    Text { text: String, composing: bool },
    /// An assistive technology's request on the UI control at `path` (an
    /// engine extension): focus, activate, increment or decrement it.
    /// Raises an [`crate::A11yAction`] event; the UI acts on it at the
    /// start of the next frame, against the last frame's controls.
    A11y { action: A11yAct, path: String },
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
                button: PointerButton::from_name(value.get("button")?.as_str()?)?,
                id: match value.get("id") {
                    None | Some(Value::Null) => 0,
                    Some(v) => u32::try_from(v.as_u64()?).ok()?,
                },
            }),
            "gamepad" => {
                let mut state = GamepadState {
                    pad: match value.get("pad") {
                        None | Some(Value::Null) => 0,
                        Some(v) => u32::try_from(v.as_u64()?).ok()?,
                    },
                    connected: match value.get("connected") {
                        None | Some(Value::Null) => true,
                        Some(v) => v.as_bool()?,
                    },
                    ..GamepadState::default()
                };
                for b in value
                    .get("buttons")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                {
                    state.buttons |= 1 << gamepad_button(b.as_str()?)?;
                }
                let floats = |key: &str, out: &mut [f32]| -> Option<()> {
                    if let Some(list) = value.get(key).and_then(Value::as_array) {
                        if list.len() > out.len() {
                            return None;
                        }
                        for (o, v) in out.iter_mut().zip(list) {
                            let f = v.as_f64()?;
                            if !f.is_finite() {
                                return None;
                            }
                            *o = f as f32;
                        }
                    }
                    Some(())
                };
                floats("axes", &mut state.axes)?;
                floats("triggers", &mut state.triggers)?;
                Some(InjectedInput::Gamepad(state))
            }
            "text" => Some(InjectedInput::Text {
                text: value.get("text")?.as_str()?.to_owned(),
                composing: match value.get("composing") {
                    None | Some(Value::Null) => false,
                    Some(v) => v.as_bool()?,
                },
            }),
            "a11y" => {
                let path = value.get("path")?.as_str()?;
                if path.is_empty() {
                    return None;
                }
                Some(InjectedInput::A11y {
                    action: A11yAct::from_name(value.get("action")?.as_str()?)?,
                    path: path.to_owned(),
                })
            }
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
                id,
            } => {
                let mut v = json!({
                    "kind": "pointer",
                    "action": match action { PointerAction::Click => "click", PointerAction::Move => "move", PointerAction::Down => "down", PointerAction::Up => "up" },
                    "x": x, "y": y, "button": button.name(),
                });
                if *id != 0 {
                    v["id"] = json!(id);
                }
                v
            }
            InjectedInput::Gamepad(g) => json!({
                "kind": "gamepad",
                "pad": g.pad,
                "connected": g.connected,
                "buttons": GAMEPAD_BUTTONS.iter().enumerate().filter(|(i, _)| g.buttons & (1 << i) != 0).map(|(_, n)| *n).collect::<Vec<_>>(),
                "axes": g.axes,
                "triggers": g.triggers,
            }),
            InjectedInput::Wheel {
                x,
                y,
                delta_x,
                delta_y,
            } => {
                json!({"kind": "wheel", "x": x, "y": y, "deltaX": delta_x, "deltaY": delta_y})
            }
            InjectedInput::Text { text, composing } => {
                let mut v = json!({"kind": "text", "text": text});
                if *composing {
                    v["composing"] = json!(true);
                }
                v
            }
            InjectedInput::A11y { action, path } => {
                json!({"kind": "a11y", "action": action.name(), "path": path})
            }
        }
    }
}
