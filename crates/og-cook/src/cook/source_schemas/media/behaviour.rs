//! The native behaviour definition, with author-only canonical references.
use super::*;
pub type Behaviour = og_ai::definition::Behaviour<ContentRef>;
pub fn schema() -> &'static og_core::reflect::source::InputSchema<Behaviour> {
    static SCHEMA: std::sync::OnceLock<og_core::reflect::source::InputSchema<Behaviour>> =
        std::sync::OnceLock::new();
    SCHEMA.get_or_init(|| og_core::reflect::source::InputSchema::new().expect("behaviour schema"))
}
