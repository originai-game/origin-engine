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

/// A record field's doc as `RECORDS` holds it: the text, or empty with the
/// `publish` feature (a published game's build: nothing there reads field
/// docs, so the text is not linked).
#[cfg(not(feature = "publish"))]
macro_rules! doc {
    ($doc:literal) => {
        $doc
    };
}

#[cfg(feature = "publish")]
macro_rules! doc {
    ($doc:literal) => {
        ""
    };
}

/// The host contract generated from the IDL: the traits `og-host-web` and
/// `og-host-node` implement (`og api gen` writes `src/generated/host.rs`).
#[rustfmt::skip]
#[path = "generated/host.rs"]
pub mod host;
