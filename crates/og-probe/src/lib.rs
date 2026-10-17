//! Origin Engine `og-probe`: the command channel and the probe, over a world
//! the host owns (design §3.3, §3.4; IDL `commands`, `probe`).
//!
//! | item | what it is |
//! |---|---|
//! | [`Probe`] | RuntimeProbe protocol 5 + `hello`, `capture`, `sweep`, `reload`, `snapshot`, `restore`, `dump`, `commands`; the command channel's scheduler and actions |
//! | [`Probe::call`] | the transport-free JSON-RPC 2.0 entry (`call(json) -> json`) |
//! | [`server::ProbeServer`] | the loopback WebSocket at `127.0.0.1:<port>/probe` (feature `ws`, native only) |
//! | [`command`] | the script grammar ([`command::parse`]), registry entries, reports, outputs |
//! | [`hooks`] | what hosts and modules provide: [`Capture`], [`UiTree`] (+ [`StaticUi`]), [`Reloader`] |
//! | [`input`] | protocol 5's [`InjectedInput`], the `Input` resource, the [`InputSink`] seam |
//! | [`Names`] | probe entity ids: names, or `#<index>.<generation>` |
//!
//! Rules: a refusal is `false`/`null` exactly as protocol 5; a JSON-RPC
//! error is only for a malformed call, an unknown method (`-32601`) or a
//! fault. The probe never writes files and never reads a clock; every call
//! is answered on the world's thread between ticks. With the `publish`
//! feature only the player-facing members answer. README.md has the whole
//! contract.

mod base64;
pub mod command;
pub mod hooks;
pub mod input;
mod names;
mod probe;
pub mod rpc;
#[cfg(all(feature = "ws", not(target_arch = "wasm32")))]
pub mod server;
mod value;

pub use command::{
    CommandDesc, CommandInfo, CommandStep, CommandsReport, LogLine, Output, ParseError, Payload,
};
pub use hooks::{Capture, Frame, Pick, Rebuilt, Reloader, StaticUi, UiElement, UiTree};
pub use input::{InjectedInput, Input, InputSink, WorldInput};
pub use names::Names;
pub use probe::{
    served_members, DumpKind, Probe, SweepDesc, BUILD, ENGINE_VERSION, MAX_CAPTURE_EDGE,
    MAX_SNAPSHOTS, MAX_SWEEP_VALUES, MEMBERS, PLAYTEST_MAX_TICKS, PROBE_PROGRAM_MAX_OPERATIONS,
    PROBE_VERSION, PROBE_WIRE_VERSION, PUBLISH_MEMBERS,
};
