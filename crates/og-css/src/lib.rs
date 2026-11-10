//! Origin Engine CSS values: the value lexer ([`tokens`]) and `calc()`,
//! the math functions and units ([`calc`]), shared by og-ui's documents
//! and style packs.

pub mod calc;
mod tokens;

pub use tokens::{Tok, describe, quoted, tok_text, tokens};
