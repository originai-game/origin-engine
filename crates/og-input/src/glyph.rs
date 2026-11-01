//! Devices and glyphs: what a prompt draws for a control. One table for
//! gameplay actions ([`crate::ActionMap::glyph`]) and og-ui's UI actions
//! (docs/ui.md §8.2), so a key, a pad button or a stick is named the same
//! way everywhere and switches live with the device used last.

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

/// A key's glyph (a `KeyboardEvent.code`): `key.<code lowercased>` and a
/// short name (`W`, `Space`, `←`).
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
