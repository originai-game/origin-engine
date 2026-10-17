//! Protocol 5's `InjectedInput` (IDL `input.InjectedInput`): one input event
//! in the game's own vocabulary, as the shell's ring, the probe's `input`
//! member and the command channel deliver it. The same shape as og-probe's
//! interim `input` module, so the probe can re-export these types.

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
                button: PointerButton::from_name(value.get("button")?.as_str()?)?,
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
