//! The element kinds and props the document runtime adds to the core
//! schema ([`super::schema`]): its `KINDS` and `PROPS` are the core's
//! followed by these, so the parser, the probe, the catalogue and the
//! README table read them as any other.
use super::schema::{KindSpec, PropSpec};

pub const KINDS: &[KindSpec] = &[];
pub const PROPS: &[PropSpec] = &[];
