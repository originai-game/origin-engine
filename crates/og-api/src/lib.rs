//! Origin Engine `og-api`: the host contract generated from the API declared
//! in `api.ogt`, the traits `og-host-web` and `og-host-node` implement and
//! the engine-owned record tables (`og api gen` writes
//! `src/generated/host.rs`). The IDL, its validator and generators are
//! `og-idl`; the host-neutral runtime the hosts wrap is `og-runtime`. See
//! `crates/og-api/README.md`.
//!
//! This crate has no dependencies and changes only when the generated Rust
//! does, so neither the generators nor the runtime rebuild what depends on
//! the contract alone (`og`'s cook).

/// The host contract generated from the IDL: the traits `og-host-web` and
/// `og-host-node` implement (`og api gen` writes `src/generated/host.rs`).
#[rustfmt::skip]
#[path = "generated/host.rs"]
pub mod host;
