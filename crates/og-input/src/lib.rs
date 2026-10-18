//! Origin Engine input (design §2.2, IDL `input` in `crates/og-api/api.ogt`).
//!
//! | module | what it owns |
//! |---|---|
//! | [`resource`] | [`Input`] (`og.Input`), the resource systems poll; the events [`KeyDown`], [`KeyUp`], [`PointerDown`], [`PointerUp`], [`PointerMove`], [`Wheel`]; [`Input::apply`] and [`Input::end_tick`], the edge rules |
//! | [`event`] | [`InjectedInput`] (protocol 5), [`KeyAction`], [`PointerAction`], [`PointerButton`], [`Modifiers`] |
//! | [`keys`] | [`KEY_CODES`] (`KeyboardEvent.code`, the key-set bit order), [`key_index`], [`key_for_char`] |
//! | [`sink`] | [`InputSink`] (the probe's seam) and [`InputLayer`], the engine's sink |
//! | [`ring`] | the web shell's SharedArrayBuffer input ring: layout, decoder, writer |
//!
//! Injected input is world state: [`InputLayer`] writes the `og.Input`
//! resource and the input event queues, so it is in snapshots and the world
//! hash, and a replay sees it. Edges last exactly one tick. Nothing here
//! reads a clock.

pub mod event;
pub mod keys;
pub mod resource;
pub mod ring;
pub mod sink;

pub use event::{
    gamepad_button, GamepadState, InjectedInput, KeyAction, Modifiers, PointerAction,
    PointerButton, GAMEPAD_BUTTONS, MAX_GAMEPADS, MAX_POINTERS,
};
pub use keys::{key_for_char, key_index, KEY_CODES, KEY_SET_BITS};
pub use resource::{
    register, Gamepad, GamepadAxis, Input, InputEvent, KeyDown, KeySet, KeyUp, PointerDown,
    PointerMove, PointerUp, Pointers, Wheel,
};
pub use sink::{InputLayer, InputSink, RingReport, WorldInput};
