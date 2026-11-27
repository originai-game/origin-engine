//! The UI domain owns the fields; the project graph owns reference identity.
use super::super::ContentRef;
use og_core::reflect::source::InputSchema;
use std::sync::OnceLock;

pub type PackInput = og_ui_next::pack::input::PackInput<ContentRef>;

pub fn schema() -> &'static InputSchema<PackInput> {
    static SCHEMA: OnceLock<InputSchema<PackInput>> = OnceLock::new();
    SCHEMA.get_or_init(|| InputSchema::new().expect("style pack input schema"))
}
