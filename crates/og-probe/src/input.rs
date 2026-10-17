//! Input as the probe injects it: og-input's protocol 5 `InjectedInput`, the
//! `og.Input` resource systems poll, and the [`InputSink`] the probe and the
//! command channel deliver every event through. The default sink is
//! og-input's `InputLayer` ([`WorldInput`]), which writes the resource and
//! sends the input events, so injected input is world state: in snapshots,
//! in the world hash, replayed.

pub use og_input::{
    key_for_char, key_index, InjectedInput, Input, InputLayer, InputSink, KeyAction, Modifiers,
    PointerAction, PointerButton, WorldInput, KEY_CODES,
};
