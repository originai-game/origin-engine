//! Origin Engine shared worlds (docs/design.md §2.12): visitors of one
//! world see each other's avatars in real time, near them only.
//!
//! - [`protocol`]: the binary messages, version 1, and the quantized avatar state.
//! - [`shard`]: one server's state with no IO: joins, movement checks, the
//!   interest grid, rate tiers under the byte cap, delta baselines.
//! - [`view`]: a client's side: remote avatars, acknowledgements, proposals.
//! - [`auth`]: token verifiers (`none`, `hs256`, `jwks`), offline.
//! - [`config`]: the `world-server` `.ogt` file.
//! - `server` and `bots` (native): `og world-server` and the load test.

pub mod auth;
#[cfg(not(target_arch = "wasm32"))]
pub mod bots;
pub mod config;
pub mod protocol;
#[cfg(not(target_arch = "wasm32"))]
pub mod server;
pub mod shard;
pub mod view;
pub mod wire;
