//! Source/world schemas. Entries arrive with their domain input contracts.
use super::Kind;

pub mod geometry;
pub mod pack;

// Geometry is a model/terrain input, not a top-level file kind. Register model
// only when the model's remaining parts/materials/processing inputs are covered.
pub(super) const KINDS: &[Kind] = &[Kind {
    name: "pack",
    schema: || pack::PackInput::schema(),
}];
