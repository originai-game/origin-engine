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
    /// With Control or Command: select all.
    KeyA,
    /// With Control or Command: copy.
    KeyC,
    /// With Control or Command: cut.
    KeyX,
}

impl UiKey {
    pub const ALL: [UiKey; 16] = [
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
        UiKey::KeyA,
        UiKey::KeyC,
        UiKey::KeyX,
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
            UiKey::KeyA => "KeyA",
            UiKey::KeyC => "KeyC",
            UiKey::KeyX => "KeyX",
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
    /// A Control or Command key is held (Ctrl/Cmd+A, C, X in a text field).
    fn ctrl(&self) -> bool {
        false
    }
    /// The first gamepad (standard mapping); `og.Input` alone has none, and
    /// [`InputFrame::from_world`] reads `og.Gamepad` for it.
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
    /// Pad 0 of the world's `og.Gamepad` (og-input's resource).
    pub fn from_gamepad(pad: &og_input::Gamepad) -> GamepadFrame {
        GamepadFrame::from_pad(pad, 0)
    }

    /// Pad `n` (0 to 3) of the world's `og.Gamepad`.
    pub fn from_pad(pad: &og_input::Gamepad, n: u32) -> GamepadFrame {
        GamepadFrame {
            pressed: pad.masks(n).1,
            stick: Vec2::new(
                pad.axis(og_input::GamepadAxis::LeftX, n),
                pad.axis(og_input::GamepadAxis::LeftY, n),
            ),
        }
    }

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
    /// key is held. This is the fallback for a world that has never had
    /// typed text (og-input's `og.TextInput` events, which the shell sends
    /// from `beforeinput` and the probe's `type:` sends too): once one has
    /// arrived, the UI takes text from those alone ([`InputFrame::with_text`]).
    fn typed(&self, out: &mut String) {
        if self.ctrl() {
            return;
        }
        let shift = self.down("ShiftLeft") || self.down("ShiftRight");
        for c in (0x20u8..=0x7e).map(char::from) {
            if let Some((code, shifted)) = og_input::key_for_char(c)
                && shifted == shift
                && self.pressed(&code)
            {
                out.push(c);
            }
        }
    }

    fn shift(&self) -> bool {
        self.down("ShiftLeft") || self.down("ShiftRight")
    }

    fn ctrl(&self) -> bool {
        self.down("ControlLeft")
            || self.down("ControlRight")
            || self.down("MetaLeft")
            || self.down("MetaRight")
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
    /// Text typed this tick: committed `og.TextInput` text when any arrived
    /// this tick (or ever: [`InputFrame::typed_stream`]), else the
    /// characters of the printable keys pressed.
    pub text: String,
    /// An input method's composition changed this tick: what it shows now
    /// (empty: it ended). `None`: no change.
    pub composition: Option<String>,
    /// This tick's text came from `og.TextInput` events (the world has a
    /// typed-text stream; key presses then type nothing).
    pub typed_stream: bool,
    /// A Shift key is held.
    pub shift: bool,
    /// A Control or Command key is held.
    pub ctrl: bool,
    /// The first gamepad.
    pub gamepad: GamepadFrame,
    /// Pads 1 to 3 (`og.Gamepad`): each moves a focus of its own while the
    /// screen allows that many players ([`crate::Ui::set_players`]).
    pub extra_pads: [GamepadFrame; 3],
    /// Every pointer at once (og-input's `og.Pointers`): touch controls
    /// (joystick, d-pad, action buttons) also answer pointers 1 to 4, so a
    /// stick and a button can be held together.
    pub pointers: og_input::Pointers,
    /// An assistive technology's requests this tick (og-input's
    /// `og.A11yAction`, from the shell's mirror of the accessibility tree):
    /// the action and the control's path.
    pub a11y: Vec<(og_input::A11yAct, String)>,
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
            composition: None,
            typed_stream: false,
            shift: source.shift(),
            ctrl: source.ctrl(),
            gamepad: source.gamepad(),
            extra_pads: [GamepadFrame::default(); 3],
            pointers: og_input::Pointers::default(),
            a11y: Vec::new(),
        }
    }

    /// Adds the world's `og.Gamepad` (pad 0, and pads 1 to 3 as
    /// `extra_pads`) and `og.Pointers` to a frame.
    pub fn with(
        mut self,
        gamepad: Option<&og_input::Gamepad>,
        pointers: Option<&og_input::Pointers>,
    ) -> InputFrame {
        if let Some(g) = gamepad {
            self.gamepad = GamepadFrame::from_gamepad(g);
            for (k, pad) in self.extra_pads.iter_mut().enumerate() {
                *pad = GamepadFrame::from_pad(g, k as u32 + 1);
            }
        }
        if let Some(p) = pointers {
            self.pointers = *p;
        }
        self
    }

    /// Adds this tick's `og.TextInput` events (og-input's typed-text
    /// stream): committed text replaces the key-derived text, and the last
    /// composition is what an input method shows. With no event the frame
    /// keeps the key-derived text (the UI drops it once a world has had a
    /// stream: `crate::Ui` remembers).
    pub fn with_text(mut self, events: &[og_input::TextInput]) -> InputFrame {
        if events.is_empty() {
            return self;
        }
        self.typed_stream = true;
        self.text.clear();
        for e in events {
            let t = e.text.as_str();
            if e.composing {
                match (&mut self.composition, e.more) {
                    (Some(c), true) => c.push_str(t),
                    _ => self.composition = Some(t.to_string()),
                }
            } else {
                self.text.push_str(t);
                // A commit ends the composition.
                self.composition = Some(String::new());
            }
        }
        self
    }

    /// Adds this tick's `og.A11yAction` events (a screen reader focusing,
    /// activating or adjusting a mirrored control).
    pub fn with_a11y(mut self, events: &[og_input::A11yAction]) -> InputFrame {
        for e in events {
            if let Some(act) = e.act() {
                self.a11y.push((act, e.path.as_str().to_owned()));
            }
        }
        self
    }

    /// The world's [`OgInput`] with its `og.Gamepad` and `og.Pointers`, or
    /// `None` when it has no input.
    pub fn from_world(world: &World) -> Option<InputFrame> {
        let input = world.resource::<OgInput>().ok()?;
        let frame = InputFrame::read(&*input);
        let pad = world.resource::<og_input::Gamepad>().ok().map(|g| *g);
        let pointers = world.resource::<og_input::Pointers>().ok().map(|p| *p);
        Some(frame.with(pad.as_ref(), pointers.as_ref()))
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
