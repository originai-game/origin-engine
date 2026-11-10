//! Origin Engine CSS values: the value lexer ([`tokens`]) and `calc()`,
//! the math functions and units ([`calc`]), shared by og-ui's documents
//! and style packs.

pub mod calc;
pub mod color;
mod srgba;
mod tokens;

pub use srgba::{Srgba, parse_color};
pub use tokens::{Tok, camel, describe, quoted, tok_text, tokens};
