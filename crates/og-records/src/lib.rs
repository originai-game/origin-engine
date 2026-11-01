//! Origin Engine `og-records`: the engine-owned record and column tables
//! generated from `api.ogt` (`og api gen` writes `src/generated/records.rs`),
//! which `og_api::host` re-exports. See `crates/og-api/README.md`.
//!
//! They are their own crate because most IDL edits change the host traits
//! and not the records: what reads only the records (`og`'s cook) then does
//! not rebuild.

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

#[rustfmt::skip]
#[path = "generated/records.rs"]
mod records;

pub use records::*;
