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
//!
//! Everything but `protocol`, `view` and `wire` is the `server` feature's.

#[cfg(feature = "server")]
pub mod auth;
#[cfg(all(feature = "server", not(target_arch = "wasm32")))]
pub mod bots;
#[cfg(feature = "server")]
pub mod config;
pub mod protocol;
#[cfg(all(feature = "server", not(target_arch = "wasm32")))]
pub mod server;
#[cfg(feature = "server")]
pub mod shard;
pub mod view;
pub mod wire;
