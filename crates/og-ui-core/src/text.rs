//! og-text's measurement, line breaking and visual order, with the parts
//! that read og-ui's rich content and layout: styled paragraphs
//! ([`paragraph`]) and vertical text ([`vertical`]).

pub use og_text::text::*;

#[cfg(feature = "shaping")]
pub mod paragraph;
pub mod vertical;
