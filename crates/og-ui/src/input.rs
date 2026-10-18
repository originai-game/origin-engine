//! What the UI reads from input, behind one small trait.
//!
//! The UI never reads the shell or the probe directly: it takes an
//! [`InputFrame`] read from any [`InputSource`] at the start of the tick.
//! [`OgInput`] names the `og.Input` resource it reads from the world
//! (`og-input`'s, version 2); pointing the UI at another resource is the one
//! `pub type OgInput` line below plus an `impl InputSource` for it.

use og_core::glam::Vec2;
use og_ecs::World;
use og_input::PointerButton;

/// The world's `og.Input` resource as the UI reads it.
pub type OgInput = og_input::Input;

/// Keys the UI acts on (text editing and dismissal).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiKey {
    Backspace,
    Delete,
    Left,
    Right,
    Home,
    End,
    Enter,
    Escape,
    Up,
    Down,
    Tab,
    Space,
    NumpadEnter,
}

impl UiKey {
    pub const ALL: [UiKey; 13] = [
        UiKey::Backspace,
        UiKey::Delete,
        UiKey::Left,
        UiKey::Right,
        UiKey::Home,
        UiKey::End,
        UiKey::Enter,
        UiKey::Escape,
        UiKey::Up,
        UiKey::Down,
        UiKey::Tab,
        UiKey::Space,
        UiKey::NumpadEnter,
    ];

    /// Its `KeyboardEvent.code`.
    pub fn code(self) -> &'static str {
        match self {
            UiKey::Backspace => "Backspace",
            UiKey::Delete => "Delete",
            UiKey::Left => "ArrowLeft",
            UiKey::Right => "ArrowRight",
            UiKey::Home => "Home",
            UiKey::End => "End",
            UiKey::Enter => "Enter",
            UiKey::Escape => "Escape",
            UiKey::Up => "ArrowUp",
            UiKey::Down => "ArrowDown",
            UiKey::Tab => "Tab",
            UiKey::Space => "Space",
            UiKey::NumpadEnter => "NumpadEnter",
        }
    }
}

/// Where input comes from: the `og.Input` resource today, `og-input`'s
/// resource when it lands. Positions are CSS pixels of the play surface.
pub trait InputSource {
    fn pointer(&self) -> Vec2;
    fn pointer_inside(&self) -> bool;
    /// Scroll accumulated this tick (positive y scrolls content up, as DOM `deltaY`).
    fn wheel(&self) -> Vec2;
    /// The primary button (left, or a touch) is held.
    fn primary_down(&self) -> bool;
    /// The primary button went down this tick.
    fn primary_pressed(&self) -> bool;
    /// The primary button went up this tick.
    fn primary_released(&self) -> bool;
    /// A key (`KeyboardEvent.code`) went down this tick.
    fn key_pressed(&self, code: &str) -> bool;
    /// Text typed this tick, appended to `out` in order.
    fn typed(&self, out: &mut String);
    /// A Shift key is held (Shift+Tab moves focus backwards).
    fn shift(&self) -> bool {
        false
    }
    /// The first gamepad (standard mapping). og.Input has no gamepad yet, so
    /// the default is none; a source that has one reports it here.
    fn gamepad(&self) -> GamepadFrame {
        GamepadFrame::default()
    }
}

/// One tick of the first gamepad in the W3C standard mapping, as the UI
/// navigates with it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GamepadFrame {
    /// Buttons that went down this tick: bit `i` is standard button `i`
    /// (0 A/south, 1 B/east, 12–15 d-pad up, down, left, right).
    pub pressed: u32,
    /// The left stick, x right and y down, each -1..1.
    pub stick: Vec2,
}

impl GamepadFrame {
    pub const A: u32 = 1 << 0;
    pub const B: u32 = 1 << 1;
    pub const UP: u32 = 1 << 12;
    pub const DOWN: u32 = 1 << 13;
    pub const LEFT: u32 = 1 << 14;
    pub const RIGHT: u32 = 1 << 15;
}

/// A focus-navigation intent: from the arrow keys, Tab, Enter, Space and
/// Escape, a gamepad's d-pad, stick, A and B, or [`crate::Ui::navigate`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nav {
    Up,
    Down,
    Left,
    Right,
    /// The next focusable control in reading order (Tab).
    Next,
    /// The previous one (Shift+Tab).
    Prev,
    /// Press the focused control (Enter, Space, gamepad A).
    Activate,
    /// Leave: close the topmost dialog or menu, or answer `Ui::back` (Escape, gamepad B).
    Back,
}

impl Nav {
    pub fn index(self) -> u8 {
        self as u8
    }

    pub fn from_index(i: u8) -> Option<Nav> {
        [
            Nav::Up,
            Nav::Down,
            Nav::Left,
            Nav::Right,
            Nav::Next,
            Nav::Prev,
            Nav::Activate,
            Nav::Back,
        ]
        .get(i as usize)
        .copied()
    }
}

impl InputSource for og_input::Input {
    fn pointer(&self) -> Vec2 {
        self.pointer
    }

    fn pointer_inside(&self) -> bool {
        self.pointer_inside
    }

    fn wheel(&self) -> Vec2 {
        self.wheel
    }

    fn primary_down(&self) -> bool {
        self.button_down(PointerButton::Left)
    }

    fn primary_pressed(&self) -> bool {
        self.button_pressed(PointerButton::Left)
    }

    fn primary_released(&self) -> bool {
        self.button_released(PointerButton::Left)
    }

    fn key_pressed(&self, code: &str) -> bool {
        self.pressed(code)
    }

    /// `og.Input` has key codes but no characters: a printable key pressed
    /// this tick types the US-layout character for it, shifted while a Shift
    /// key is held. (Injected `modifiers` do not reach the resource, so
    /// `type:Hello` types `hello`; a typed-text stream is owed by og-input.)
    fn typed(&self, out: &mut String) {
        let shift = self.down("ShiftLeft") || self.down("ShiftRight");
        for c in (0x20u8..=0x7e).map(char::from) {
            if let Some((code, shifted)) = og_input::key_for_char(c) {
                if shifted == shift && self.pressed(&code) {
                    out.push(c);
                }
            }
        }
    }

    fn shift(&self) -> bool {
        self.down("ShiftLeft") || self.down("ShiftRight")
    }
}

/// One tick of input, as the UI consumes it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InputFrame {
    pub pointer: Vec2,
    pub pointer_inside: bool,
    pub wheel: Vec2,
    pub down: bool,
    pub pressed: bool,
    pub released: bool,
    /// Editing keys that went down this tick.
    pub keys: Vec<UiKey>,
    /// Text typed this tick.
    pub text: String,
    /// A Shift key is held.
    pub shift: bool,
    /// The first gamepad.
    pub gamepad: GamepadFrame,
}

impl InputFrame {
    pub fn read(source: &impl InputSource) -> InputFrame {
        let mut text = String::new();
        source.typed(&mut text);
        InputFrame {
            pointer: source.pointer(),
            pointer_inside: source.pointer_inside(),
            wheel: source.wheel(),
            down: source.primary_down(),
            pressed: source.primary_pressed(),
            released: source.primary_released(),
            keys: UiKey::ALL
                .into_iter()
                .filter(|k| source.key_pressed(k.code()))
                .collect(),
            text,
            shift: source.shift(),
            gamepad: source.gamepad(),
        }
    }

    /// The world's [`OgInput`], or `None` when it has none.
    pub fn from_world(world: &World) -> Option<InputFrame> {
        world
            .resource::<OgInput>()
            .ok()
            .map(|input| InputFrame::read(&*input))
    }

    /// A pointer at a point with no buttons (for tools and tests).
    pub fn hover(x: f32, y: f32) -> InputFrame {
        InputFrame {
            pointer: Vec2::new(x, y),
            pointer_inside: true,
            ..InputFrame::default()
        }
    }
}
