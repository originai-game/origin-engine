//! Named UI actions and their glyphs (docs/ui.md §8.2). An action
//! (`confirm`, `back`, `tabNext`, `tabPrev`, `details`, or one the game
//! defines) is bound to og-input's controls, key codes and standard-mapping
//! pad buttons, and fires on press, release or a hold; every hold has a
//! toggle alternative (XAG 107: a press starts it, the next ends it). A
//! component asks for an action, not a key: [`Ui::action`].
//!
//! Glyph sets name each control for keyboard and mouse, Xbox, PlayStation,
//! Switch, generic pads and touch; the set in use is the device the player
//! used last ([`Ui::device`]), so prompts switch live. The action bar
//! ([`Ui::action_bar`]) is the data an action bar component reads: the
//! actions the top layer accepts, each with its hint and glyph.

use crate::input::InputFrame;
use crate::ui::Ui;
use serde_json::{Value, json};

/// When an action fires.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ActionTrigger {
    #[default]
    Press,
    Release,
    /// Held this long (ms).
    Hold(f32),
}

/// A named UI action and its bindings.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ActionDesc {
    pub name: String,
    /// Key codes (`KeyboardEvent.code`: `Enter`, `KeyE`).
    pub keys: Vec<String>,
    /// Standard-mapping pad buttons (`a`, `b`, `rightBumper`).
    pub buttons: Vec<String>,
    pub trigger: ActionTrigger,
    /// A hold also fires as a toggle: press to start, press again to stop
    /// ([`Ui::action_held`] answers true between).
    pub toggle: bool,
    /// What it says when no document hint names it.
    pub label: String,
}

/// The device a glyph set is for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Device {
    #[default]
    KeyboardMouse,
    Xbox,
    PlayStation,
    Switch,
    Gamepad,
    Touch,
}

impl Device {
    pub const NAMES: [&'static str; 6] = [
        "keyboardMouse",
        "xbox",
        "playstation",
        "switch",
        "gamepad",
        "touch",
    ];
    pub fn name(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    pub fn from_name(n: &str) -> Option<Device> {
        use Device::*;
        [KeyboardMouse, Xbox, PlayStation, Switch, Gamepad, Touch]
            .into_iter()
            .find(|d| d.name() == n)
    }
    fn pad(self) -> bool {
        matches!(
            self,
            Device::Xbox | Device::PlayStation | Device::Switch | Device::Gamepad
        )
    }
}

/// A control's glyph on a device: the icon a style's set draws (`pad.a`,
/// `ps.cross`, `key.enter`) and the text that names it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Glyph {
    pub icon: String,
    pub text: String,
}

/// A pad button's glyph on a pad family.
pub fn pad_glyph(device: Device, button: &str) -> Glyph {
    let (prefix, text) = match (device, button) {
        (Device::PlayStation, "a") => ("ps.cross", "×"),
        (Device::PlayStation, "b") => ("ps.circle", "○"),
        (Device::PlayStation, "x") => ("ps.square", "□"),
        (Device::PlayStation, "y") => ("ps.triangle", "△"),
        (Device::PlayStation, "leftBumper") => ("ps.l1", "L1"),
        (Device::PlayStation, "rightBumper") => ("ps.r1", "R1"),
        (Device::PlayStation, "leftTrigger") => ("ps.l2", "L2"),
        (Device::PlayStation, "rightTrigger") => ("ps.r2", "R2"),
        (Device::PlayStation, "back") => ("ps.create", "Create"),
        (Device::PlayStation, "start") => ("ps.options", "Options"),
        // Switch's face buttons sit where Xbox's mirror: south is B.
        (Device::Switch, "a") => ("switch.b", "B"),
        (Device::Switch, "b") => ("switch.a", "A"),
        (Device::Switch, "x") => ("switch.y", "Y"),
        (Device::Switch, "y") => ("switch.x", "X"),
        (Device::Switch, "leftBumper") => ("switch.l", "L"),
        (Device::Switch, "rightBumper") => ("switch.r", "R"),
        (Device::Switch, "leftTrigger") => ("switch.zl", "ZL"),
        (Device::Switch, "rightTrigger") => ("switch.zr", "ZR"),
        (Device::Switch, "back") => ("switch.minus", "−"),
        (Device::Switch, "start") => ("switch.plus", "+"),
        (Device::Xbox, "a") => ("xbox.a", "A"),
        (Device::Xbox, "b") => ("xbox.b", "B"),
        (Device::Xbox, "x") => ("xbox.x", "X"),
        (Device::Xbox, "y") => ("xbox.y", "Y"),
        (Device::Xbox, "leftBumper") => ("xbox.lb", "LB"),
        (Device::Xbox, "rightBumper") => ("xbox.rb", "RB"),
        (Device::Xbox, "leftTrigger") => ("xbox.lt", "LT"),
        (Device::Xbox, "rightTrigger") => ("xbox.rt", "RT"),
        (Device::Xbox, "back") => ("xbox.view", "View"),
        (Device::Xbox, "start") => ("xbox.menu", "Menu"),
        (_, "a") => ("pad.south", "South"),
        (_, "b") => ("pad.east", "East"),
        (_, "x") => ("pad.west", "West"),
        (_, "y") => ("pad.north", "North"),
        (_, "leftBumper") => ("pad.l1", "L1"),
        (_, "rightBumper") => ("pad.r1", "R1"),
        (_, "leftTrigger") => ("pad.l2", "L2"),
        (_, "rightTrigger") => ("pad.r2", "R2"),
        (_, "back") => ("pad.select", "Select"),
        (_, "start") => ("pad.start", "Start"),
        (_, "up") => ("pad.dpadUp", "↑"),
        (_, "down") => ("pad.dpadDown", "↓"),
        (_, "left") => ("pad.dpadLeft", "←"),
        (_, "right") => ("pad.dpadRight", "→"),
        (_, "leftStick") => ("pad.ls", "L3"),
        (_, "rightStick") => ("pad.rs", "R3"),
        (_, "home") => ("pad.home", "Home"),
        (_, other) => {
            return Glyph {
                icon: format!("pad.{other}"),
                text: other.to_string(),
            };
        }
    };
    let icon = match (device, prefix.split_once('.')) {
        // A generic pad's d-pad and sticks are every family's.
        (Device::Xbox | Device::PlayStation | Device::Switch, Some(("pad", rest))) => {
            format!(
                "{}.{rest}",
                match device {
                    Device::Xbox => "xbox",
                    Device::PlayStation => "ps",
                    _ => "switch",
                }
            )
        }
        _ => prefix.to_string(),
    };
    Glyph {
        icon,
        text: text.to_string(),
    }
}

/// A key's glyph.
pub fn key_glyph(code: &str) -> Glyph {
    let text = match code {
        "Enter" | "NumpadEnter" => "Enter".to_string(),
        "Escape" => "Esc".into(),
        "Space" => "Space".into(),
        "Tab" => "Tab".into(),
        "Backspace" => "⌫".into(),
        "ArrowUp" => "↑".into(),
        "ArrowDown" => "↓".into(),
        "ArrowLeft" => "←".into(),
        "ArrowRight" => "→".into(),
        "ShiftLeft" | "ShiftRight" => "Shift".into(),
        "BracketLeft" => "[".into(),
        "BracketRight" => "]".into(),
        c if c.starts_with("Key") => c[3..].to_string(),
        c if c.starts_with("Digit") => c[5..].to_string(),
        c => c.to_string(),
    };
    Glyph {
        icon: format!("key.{}", code.to_ascii_lowercase()),
        text,
    }
}

/// The actions every UI has, with their default bindings.
pub fn default_actions() -> Vec<ActionDesc> {
    let a = |name: &str, keys: &[&str], buttons: &[&str], label: &str| ActionDesc {
        name: name.into(),
        keys: keys.iter().map(|s| s.to_string()).collect(),
        buttons: buttons.iter().map(|s| s.to_string()).collect(),
        label: label.into(),
        ..ActionDesc::default()
    };
    vec![
        a("confirm", &["Enter", "Space"], &["a"], "Confirm"),
        a("back", &["Escape"], &["b"], "Back"),
        a(
            "tabNext",
            &["KeyE", "BracketRight"],
            &["rightBumper"],
            "Next tab",
        ),
        a(
            "tabPrev",
            &["KeyQ", "BracketLeft"],
            &["leftBumper"],
            "Previous tab",
        ),
        a("details", &["KeyX"], &["y"], "Details"),
    ]
}

/// One action's state between ticks.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ActionState {
    pub down: bool,
    /// The tick its press started (0: not held).
    pub since: u64,
    /// A hold already fired for this press.
    pub held_fired: bool,
    /// Its toggle is on.
    pub toggled: bool,
}

/// The actions and the last device, kept in the UI.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Actions {
    pub(crate) defs: Vec<ActionDesc>,
    pub(crate) states: Vec<ActionState>,
    /// Fired this tick.
    pub(crate) fired: Vec<String>,
    pub(crate) device: Device,
    /// The pad family a pad's input shows (the host's guess from the pad's id).
    pub(crate) pad: Device,
    pub(crate) defined: bool,
}

/// One entry of the action bar.
#[derive(Clone, Debug, PartialEq)]
pub struct ActionHint {
    pub action: String,
    pub hint: String,
    pub glyph: Glyph,
    pub hold: bool,
}

impl ActionHint {
    pub fn to_json(&self) -> Value {
        json!({ "action": self.action, "hint": self.hint, "icon": self.glyph.icon, "text": self.glyph.text, "hold": self.hold })
    }
}

impl Ui {
    fn ensure_actions(&mut self) {
        if !self.actions.defined {
            self.actions.defined = true;
            self.actions.defs = default_actions();
            self.actions.pad = Device::Xbox;
        }
        self.actions
            .states
            .resize(self.actions.defs.len(), ActionState::default());
    }

    /// Defines (or redefines) a UI action.
    pub fn define_action(&mut self, desc: ActionDesc) {
        self.ensure_actions();
        match self.actions.defs.iter_mut().find(|d| d.name == desc.name) {
            Some(d) => *d = desc,
            None => self.actions.defs.push(desc),
        }
        self.ensure_actions();
    }

    /// Every action defined.
    pub fn action_defs(&mut self) -> Vec<ActionDesc> {
        self.ensure_actions();
        self.actions.defs.clone()
    }

    /// The action fired this tick.
    pub fn action(&self, name: &str) -> bool {
        self.action_fired(name)
    }

    pub(crate) fn action_fired(&self, name: &str) -> bool {
        self.actions.fired.iter().any(|a| a == name)
    }

    /// A hold action is held (its press past the hold, or its toggle on).
    pub fn action_held(&self, name: &str) -> bool {
        let Some(k) = self.actions.defs.iter().position(|d| d.name == name) else {
            return false;
        };
        let s = self.actions.states.get(k).cloned().unwrap_or_default();
        s.toggled || (s.down && s.held_fired)
    }

    /// Fires an action this tick (a document's `action` trigger, a test).
    pub fn fire_action(&mut self, name: &str) {
        if !self.action_fired(name) {
            self.actions.fired.push(name.to_string());
        }
    }

    /// The device the player used last (its glyphs are shown).
    pub fn device(&self) -> Device {
        self.actions.device
    }

    /// Sets the device (the host's when it knows; a test's).
    pub fn set_device(&mut self, device: Device) {
        self.actions.device = device;
        if device.pad() {
            self.actions.pad = device;
        }
    }

    /// The pad family the host saw (Xbox by default): pad input shows its glyphs.
    pub fn set_pad_family(&mut self, device: Device) {
        if device.pad() {
            self.actions.pad = device;
            if self.actions.device.pad() {
                self.actions.device = device;
            }
        }
    }

    /// An action's glyph on the current device: its first binding there.
    pub fn glyph(&mut self, name: &str) -> Option<Glyph> {
        self.ensure_actions();
        let d = self.actions.defs.iter().find(|d| d.name == name)?.clone();
        let device = self.actions.device;
        Some(if device.pad() {
            pad_glyph(device, d.buttons.first()?)
        } else if device == Device::Touch {
            Glyph {
                icon: format!("touch.{name}"),
                text: d.label.clone(),
            }
        } else {
            key_glyph(d.keys.first()?)
        })
    }

    /// The action bar: the actions the top layer's document lists, each
    /// with its hint and glyph (none: the default actions it accepts, Back
    /// when Back does something).
    pub fn action_bar(&mut self) -> Vec<ActionHint> {
        self.ensure_actions();
        let top = self
            .layer_stack()
            .into_iter()
            .rev()
            .find(|l| !l.hidden && l.spec.input != crate::doc::InputMode::Game);
        let listed: Vec<(String, String)> = match &top {
            Some(l) => self
                .document(&l.screen)
                .map(|d| d.actions.clone())
                .unwrap_or_default(),
            None => Vec::new(),
        };
        let mut out = Vec::new();
        for (a, hint) in listed {
            let def = self.actions.defs.iter().find(|d| d.name == a).cloned();
            let Some(glyph) = self.glyph(&a) else {
                continue;
            };
            out.push(ActionHint {
                hint: if hint.is_empty() {
                    def.as_ref().map(|d| d.label.clone()).unwrap_or_default()
                } else {
                    hint
                },
                action: a,
                glyph,
                hold: def.is_some_and(|d| matches!(d.trigger, ActionTrigger::Hold(_))),
            });
        }
        out
    }

    /// Reads this tick's input into the actions and the last device (from `begin`).
    pub(crate) fn actions_begin(&mut self, input: &InputFrame) {
        self.ensure_actions();
        self.actions.fired.clear();
        // The last device: a key or the mouse, a pad, a touch.
        let pad_used = input.gamepad.pressed != 0 || input.gamepad.stick.length() > 0.5;
        let touch = self.viewport.coarse_pointer && (input.pressed || input.pointers.pressed != 0);
        let keys = !input.keys_held.is_empty()
            && input
                .keys_held
                .iter()
                .any(|k| input.keys_pressed.contains(k));
        let mouse = !self.viewport.coarse_pointer && (input.pressed || input.wheel.length() > 0.0);
        if pad_used {
            self.actions.device = self.actions.pad;
        } else if touch {
            self.actions.device = Device::Touch;
        } else if keys || mouse {
            self.actions.device = Device::KeyboardMouse;
        }
        let now = self.state.tick;
        let tps = self.ticks_per_second.max(1);
        for (k, d) in self.actions.defs.iter().enumerate() {
            let button_bits = d
                .buttons
                .iter()
                .filter_map(|b| og_input::gamepad_button(b))
                .fold(0u32, |m, i| m | 1 << i);
            let down = d
                .keys
                .iter()
                .any(|c| input.keys_held.iter().any(|h| h == c))
                || input.gamepad.held & button_bits != 0;
            let pressed = d
                .keys
                .iter()
                .any(|c| input.keys_pressed.iter().any(|h| h == c))
                || input.gamepad.pressed & button_bits != 0;
            let s = &mut self.actions.states[k];
            let was = s.down;
            let pressed = pressed || (down && !was);
            let released = was && !down;
            s.down = down;
            let mut fire = false;
            match d.trigger {
                ActionTrigger::Press => fire = pressed,
                ActionTrigger::Release => fire = released,
                ActionTrigger::Hold(ms) => {
                    if pressed {
                        s.since = now;
                        s.held_fired = false;
                    }
                    let need = ((ms / 1000.0) * tps as f32).ceil() as u64;
                    if down && !s.held_fired && now.saturating_sub(s.since) >= need {
                        s.held_fired = true;
                        fire = true;
                    }
                    // The toggle: a press shorter than the hold turns it on.
                    if released && d.toggle && !s.held_fired {
                        s.toggled = !s.toggled;
                        fire = s.toggled;
                    }
                    if released {
                        s.held_fired = false;
                    }
                }
            }
            if fire {
                self.actions.fired.push(d.name.clone());
            }
        }
    }

    /// The actions' state as JSON (dump_ui's `actions`).
    pub fn actions_json(&mut self) -> Value {
        self.ensure_actions();
        let fired = self.actions.fired.clone();
        json!({
            "device": self.actions.device.name(),
            "fired": fired,
            "defined": self.actions.defs.iter().map(|d| json!({
                "name": d.name, "keys": d.keys, "buttons": d.buttons, "label": d.label, "toggle": d.toggle,
                "trigger": match d.trigger { ActionTrigger::Press => json!("press"), ActionTrigger::Release => json!("release"), ActionTrigger::Hold(ms) => json!({ "hold": ms }) },
            })).collect::<Vec<_>>(),
            "bar": self.action_bar().iter().map(ActionHint::to_json).collect::<Vec<_>>(),
        })
    }
}
