//! og-text's measurement, line breaking and visual order, with the parts
//! that read og-ui's rich content and layout: styled paragraphs
//! ([`paragraph`]) and vertical text ([`vertical`]).

pub use og_text::text::*;

pub mod colr;
pub(crate) mod edit;
#[doc(hidden)]
pub mod input_type;
pub mod markup;
pub mod paragraph;
#[cfg(feature = "shaping")]
#[doc(hidden)]
pub mod rich;
pub mod text_path;
pub mod vertical;
