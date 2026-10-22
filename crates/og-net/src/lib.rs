//! og-net: rooms and deterministic lockstep (docs/design.md §2.8, §2.10).
//!
//! - [`protocol`]: the room's wire shapes. The engine queues *ops*
//!   (`create`, `join`, `leave`, `send`) for its host and takes *events*
//!   back (`joined`, `peer_joined`, `peer_left`, `message`, `left`,
//!   `error`): the web shell maps them onto the platform's rooms
//!   (docs/platform-asks.md), the native host onto the relay below. The
//!   relay speaks exactly these shapes as WebSocket text frames.
//! - [`state`]: a world snapshot (og-ecs's format 2 to 4) read without a
//!   world: the shared hash lockstep compares (the snapshot with each
//!   peer's own session left out) and the component-level diff of two
//!   snapshots that answers "what is not deterministic".
//! - [`lockstep`]: the lockstep machine. It owns the seats, the inputs of
//!   every player by tick, the start and late-join hand-over (a peer's
//!   snapshot), the per-tick hashes and their comparison, and decides at
//!   every tick boundary whether the world may take its next tick: tick t
//!   runs only when every seated player's input for t is in. It never
//!   touches a world; og-runtime gives it snapshots and applies what it
//!   answers.
//! - `relay` and `client` (feature `ws`, native only): `og relay`, a local
//!   WebSocket relay that keeps rooms of peers and forwards their messages
//!   in order, and the client `og run --room ws://127.0.0.1:P/<room>`
//!   drives an engine's room through.

pub mod base64;
pub mod lockstep;
pub mod protocol;
pub mod state;

#[cfg(all(feature = "ws", not(target_arch = "wasm32")))]
pub mod client;
#[cfg(all(feature = "ws", not(target_arch = "wasm32")))]
pub mod relay;

pub use lockstep::{Config, Gate, Lockstep};
pub use state::Snapshot;
