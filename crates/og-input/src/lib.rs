//! Origin Engine input (design §2.2, IDL `input` in `crates/og-api/api/input.ogt`).
//!
//! | module | what it owns |
//! |---|---|
//! | [`resource`] | [`Input`] (`og.Input`), the resource systems poll; the events [`KeyDown`], [`KeyUp`], [`PointerDown`], [`PointerUp`], [`PointerMove`], [`Wheel`], [`TextInput`] (typed and composed text), [`A11yAction`] (a screen reader's request on a UI control); [`PointerLock`] (mouse-look's lock: the game's `wanted`, the host's `locked`); [`Input::apply`] and [`Input::end_tick`], the edge rules |
//! | [`event`] | [`InjectedInput`] (protocol 5), [`KeyAction`], [`PointerAction`], [`PointerButton`], [`Modifiers`] |
//! | [`keys`] | [`KEY_CODES`] (`KeyboardEvent.code`, the key-set bit order), [`key_index`], [`key_for_char`] |
//! | [`sink`] | [`InputSink`] (the probe's seam) and [`InputLayer`], the engine's sink |
//! | [`ring`] | the web shell's SharedArrayBuffer input ring: layout, decoder, writer |
//! | [`control`] | [`Control`]: the physical inputs a binding names (`KeyW`, `mouse.left`, `pad.leftXY`, `touch.xy`), how each reads, dead zones |
//! | [`actions`] | [`ActionMap`]: gameplay actions over bindings, contexts, rebinding with conflicts, capture, saved bindings, glyphs, rumble; [`ActionsBlock`] (its state block) and [`install`] |
//! | [`glyph`] | [`Device`] (keyboard and mouse, Xbox, PlayStation, Switch, generic pads, touch) and the glyph tables og-ui's UI actions share; the keyboard layout's printed keys ([`set_key_labels`]) |
//!
//! Injected input is world state: [`InputLayer`] writes the `og.Input`
//! resource and the input event queues, so it is in snapshots and the world
//! hash, and a replay sees it. Edges last exactly one tick. Nothing here
//! reads a clock.

pub mod actions;
pub mod control;
pub mod event;
pub mod glyph;
pub mod keys;
pub mod resource;
pub mod ring;
pub mod sink;

pub use actions::{
    ACTIONS_BLOCK, ACTIONS_SYSTEM, ActionDesc, ActionFeed, ActionInfo, ActionKind, ActionMap,
    ActionState, ActionsBlock, Binding, CaptureAccept, CaptureDesc, CaptureState, Conflict,
    ConflictPolicy, RebindAnswer, RebindDesc, Rumble, Sensitivity, SharedActions, binding_glyphs,
    combined_glyph, install, overlaps,
};
pub use control::{Control, ControlZone, PadDeadzone, Surface, Zone, ZoneKind};
pub use event::{
    A11yAct, GAMEPAD_BUTTONS, GamepadState, InjectedInput, KeyAction, MAX_GAMEPADS, MAX_POINTERS,
    Modifiers, PointerAction, PointerButton, gamepad_button,
};
pub use glyph::{Device, Glyph, key_glyph, key_labels, pad_glyph, set_key_labels};
pub use keys::{KEY_CODES, KEY_SET_BITS, key_for_char, key_index};
pub use resource::{
    A11yAction, Gamepad, GamepadAxis, Input, InputEvent, KeyDown, KeySet, KeyUp, PauseIntent,
    PointerDown, PointerLock, PointerMove, PointerUp, Pointers, TextInput, Wheel, register,
};
pub use sink::{InputLayer, InputSink, RingReport, WorldInput};
