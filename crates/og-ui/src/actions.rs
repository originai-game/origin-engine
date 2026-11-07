//! Named UI actions and their glyphs (docs/ui.md §2.1). An action
//! (`confirm`, `back`, `tabNext`, `tabPrev`, `details`, or one the game
//! defines) is bound to og-input's controls, key codes and standard-mapping
//! pad buttons, or follows an og-input gameplay action
//! ([`ActionDesc::action`]: og-input's binding model, so the player's
//! rebinding moves both), and fires on press, release or a hold; every hold
//! has a toggle alternative (XAG 107: a press starts it, the next ends it).
//! A component asks for an action, not a key: [`Ui::action`].
//!
//! Glyph sets name each control for keyboard and mouse, Xbox, PlayStation,
//! Switch, generic pads and touch (og-input's [`og_input::glyph`] tables,
//! the gameplay actions' own); the set in use is the device the player
//! used last ([`Ui::device`]), so prompts switch live, and a pad shows
//! its own family when the host told it (`og.Gamepad`'s devices). The
//! action bar ([`Ui::action_bar`]) is the data an action bar component
//! reads: the actions the top layer accepts, each with its hint and glyph.

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
    /// An og-input action (og.input.defineAction) it follows: down and
    /// pressed are that action's, and so are its glyphs, so the player's
    /// rebinding moves both. Empty: `keys` and `buttons`.
    pub action: String,
}

pub use og_input::{Device, Glyph, key_glyph, pad_glyph};

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
        // A held piece turns a quarter (an inventory's item, a placed
        // building): R, as in the games that carry items in grids, and the
        // pad's free face button.
        a("rotate", &["KeyR"], &["x"], "Rotate"),
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
    /// Fired between ticks (the probe's `ui:action`): they fire next tick.
    pub(crate) pending: Vec<String>,
    pub(crate) device: Device,
    /// The pad family a pad's input shows (the host's guess from the pad's id).
    pub(crate) pad: Device,
    pub(crate) defined: bool,
    /// og-input's actions this tick, as the input frame carried them (what
    /// [`ActionDesc::action`] follows).
    pub(crate) feed: Vec<og_input::ActionFeed>,
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

    /// Every action defined (the defaults until one is).
    pub fn action_defs(&self) -> Vec<ActionDesc> {
        if self.actions.defined {
            self.actions.defs.clone()
        } else {
            default_actions()
        }
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

    /// Fires an action this tick (a document's `action` trigger, a test),
    /// or between ticks the next tick.
    pub fn fire_action(&mut self, name: &str) {
        if !self.building {
            self.actions.pending.push(name.to_string());
        } else if !self.action_fired(name) {
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
        if device.is_pad() {
            self.actions.pad = device;
        }
    }

    /// The pad family the host saw (Xbox by default): pad input shows its glyphs.
    pub fn set_pad_family(&mut self, device: Device) {
        if device.is_pad() {
            self.actions.pad = device;
            if self.actions.device.is_pad() {
                self.actions.device = device;
            }
        }
    }

    /// An action's glyph on the current device: its first binding there
    /// (a followed og-input action's: [`ActionDesc::action`]).
    pub fn glyph(&self, name: &str) -> Option<Glyph> {
        let d = self.action_defs().into_iter().find(|d| d.name == name)?;
        let device = self.actions.device;
        if !d.action.is_empty() {
            let feed = self.actions.feed.iter().find(|f| f.name == d.action)?;
            let parts = og_input::binding_glyphs(&feed.bindings, device);
            return og_input::combined_glyph(&parts, device).or_else(|| {
                (device == Device::Touch).then(|| Glyph {
                    icon: format!("touch.{name}"),
                    text: d.label.clone(),
                })
            });
        }
        Some(if device.is_pad() {
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
    pub fn action_bar(&self) -> Vec<ActionHint> {
        let defs = self.action_defs();
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
            let def = defs.iter().find(|d| d.name == a).cloned();
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
        self.actions.fired = std::mem::take(&mut self.actions.pending);
        self.actions.feed.clone_from(&input.actions);
        // The last device: a key or the mouse, a pad, a touch.
        let pad_used = input.gamepad.pressed != 0 || input.gamepad.stick.length() > 0.5;
        if pad_used && let Some(family) = input.gamepad.device {
            self.actions.pad = family;
        }
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
            if !d.action.is_empty() {
                // A followed action: its state is og-input's this tick.
                let s = self
                    .actions
                    .feed
                    .iter()
                    .find(|f| f.name == d.action)
                    .map(|f| f.state)
                    .unwrap_or_default();
                let st = &mut self.actions.states[k];
                let was = st.down;
                st.down = s.down;
                let fire = match d.trigger {
                    ActionTrigger::Press => s.pressed,
                    ActionTrigger::Release => s.released || (was && !s.down),
                    ActionTrigger::Hold(ms) => {
                        if s.pressed {
                            st.since = now;
                            st.held_fired = false;
                        }
                        let need = ((ms / 1000.0) * tps as f32).ceil() as u64;
                        let mut fire = false;
                        if s.down && !st.held_fired && now.saturating_sub(st.since) >= need {
                            st.held_fired = true;
                            fire = true;
                        }
                        if !s.down && was {
                            if d.toggle && !st.held_fired {
                                st.toggled = !st.toggled;
                                fire = st.toggled;
                            }
                            st.held_fired = false;
                        }
                        fire
                    }
                };
                if fire {
                    self.actions.fired.push(d.name.clone());
                }
                continue;
            }
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
    pub fn actions_json(&self) -> Value {
        let fired = self.actions.fired.clone();
        json!({
            "device": self.actions.device.name(),
            "fired": fired,
            "defined": self.action_defs().iter().map(|d| json!({
                "name": d.name, "keys": d.keys, "buttons": d.buttons, "label": d.label, "toggle": d.toggle, "action": d.action,
                "trigger": match d.trigger { ActionTrigger::Press => json!("press"), ActionTrigger::Release => json!("release"), ActionTrigger::Hold(ms) => json!({ "hold": ms }) },
            })).collect::<Vec<_>>(),
            "bar": self.action_bar().iter().map(ActionHint::to_json).collect::<Vec<_>>(),
        })
    }
}
