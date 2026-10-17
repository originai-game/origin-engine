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
}

impl UiKey {
    pub const ALL: [UiKey; 8] = [
        UiKey::Backspace,
        UiKey::Delete,
        UiKey::Left,
        UiKey::Right,
        UiKey::Home,
        UiKey::End,
        UiKey::Enter,
        UiKey::Escape,
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
