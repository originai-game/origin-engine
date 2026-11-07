//! Devices and glyphs: what a prompt draws for a control. One table for
//! gameplay actions ([`crate::ActionMap::glyph`]) and og-ui's UI actions
//! (docs/ui.md §2.1), so a key, a pad button or a stick is named the same
//! way everywhere and switches live with the device used last. A key's
//! glyph is the player's keyboard layout's printed key ([`set_key_labels`]).

use std::collections::BTreeMap;
use std::sync::RwLock;

/// A kind of input device: the set a glyph is drawn from, and what a pad
/// is (IDL `input.InputDevice`; og-ui's `UiDevice` has the same values in
/// the same order). A pad whose family the host could not tell is
/// [`Device::Gamepad`]: its glyphs name positions (South, East).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    /// Every device, in variant order.
    pub const ALL: [Device; 6] = [
        Device::KeyboardMouse,
        Device::Xbox,
        Device::PlayStation,
        Device::Switch,
        Device::Gamepad,
        Device::Touch,
    ];
    /// The variant names, by index.
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
        Self::ALL.into_iter().find(|d| d.name() == n)
    }

    /// Its variant index.
    pub fn index(self) -> u32 {
        self as u32
    }

    pub fn from_index(i: u32) -> Option<Device> {
        Self::ALL.get(i as usize).copied()
    }

    /// A pad family (Xbox, PlayStation, Switch or a generic pad).
    pub fn is_pad(self) -> bool {
        matches!(
            self,
            Device::Xbox | Device::PlayStation | Device::Switch | Device::Gamepad
        )
    }

    /// The device class a control belongs to: keyboard and mouse, a pad
    /// (any family), or touch.
    pub fn class(self) -> Device {
        if self.is_pad() { Device::Gamepad } else { self }
    }
}

/// A control's glyph on a device: the icon a style's set draws (`pad.a`,
/// `ps.cross`, `key.enter`) and the text that names it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Glyph {
    pub icon: String,
    pub text: String,
}

fn family_prefix(device: Device) -> &'static str {
    match device {
        Device::Xbox => "xbox",
        Device::PlayStation => "ps",
        Device::Switch => "switch",
        _ => "pad",
    }
}

/// A pad button's glyph on a pad family (standard-mapping names:
/// [`crate::GAMEPAD_BUTTONS`]). Switch's face buttons sit where Xbox's
/// mirror: the south button is B.
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
            format!("{}.{rest}", family_prefix(device))
        }
        _ => prefix.to_string(),
    };
    Glyph {
        icon,
        text: text.to_string(),
    }
}

/// The keyboard layout's printed keys, by `KeyboardEvent.code`: what the
/// host read from the player's layout (the web shell's
/// `navigator.keyboard.getLayoutMap()`, or the characters its keys typed
/// where the browser has no layout map). Empty is the US layout. Bindings
/// never read it: they name physical keys, so a game's WASD is ZQSD on
/// AZERTY by position, and only the glyphs say so.
static LAYOUT: RwLock<BTreeMap<String, String>> = RwLock::new(BTreeMap::new());

/// Sets the keyboard layout's printed keys (`code`, `label`), in place of
/// the last ones; an empty list is the US layout. A label is the character
/// the key types without modifiers (`a` for `KeyQ` on AZERTY); an entry
/// with an empty code or label is skipped.
pub fn set_key_labels(labels: impl IntoIterator<Item = (String, String)>) {
    let map = labels
        .into_iter()
        .filter(|(c, l)| !c.is_empty() && !l.is_empty())
        .collect();
    *LAYOUT.write().unwrap_or_else(|e| e.into_inner()) = map;
}

/// The keyboard layout's printed keys as [`set_key_labels`] last set
/// them, sorted by code.
pub fn key_labels() -> Vec<(String, String)> {
    LAYOUT
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .map(|(c, l)| (c.clone(), l.clone()))
        .collect()
}

/// The key a layout's label names, for its glyph: a letter's own key's
/// glyph (`KeyA` for `a`), another character the key that types it on a
/// US layout without Shift. `None` keeps the physical key's glyph: the
/// digit row (its digits are printed on every layout's keys) and a label
/// that is not one character.
fn printed(code: &str) -> Option<Glyph> {
    if code.starts_with("Digit") {
        return None;
    }
    let label = LAYOUT
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .get(code)
        .cloned()?;
    let mut chars = label.chars();
    let (c, None) = (chars.next()?, chars.next()) else {
        return None;
    };
    if c.is_alphabetic() {
        let text: String = c.to_uppercase().collect();
        let icon = if c.is_ascii_alphabetic() {
            format!("key.key{}", c.to_ascii_lowercase())
        } else {
            format!("key.{}", code.to_ascii_lowercase())
        };
        return Some(Glyph { icon, text });
    }
    if c.is_whitespace() || c.is_control() {
        return None;
    }
    let icon = match crate::keys::key_for_char(c) {
        Some((us, false)) => format!("key.{}", us.to_ascii_lowercase()),
        _ => format!("key.{}", code.to_ascii_lowercase()),
    };
    Some(Glyph {
        icon,
        text: c.to_string(),
    })
}

/// A key's glyph (a `KeyboardEvent.code`): `key.<code lowercased>` and a
/// short name (`W`, `Space`, `←`), as the player's keyboard layout prints
/// it ([`set_key_labels`]): `KeyQ` is `A` (`key.keya`) on AZERTY, `KeyY`
/// is `Z` on QWERTZ.
pub fn key_glyph(code: &str) -> Glyph {
    if let Some(g) = printed(code) {
        return g;
    }
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
        "ControlLeft" | "ControlRight" => "Ctrl".into(),
        "AltLeft" | "AltRight" => "Alt".into(),
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
