//! Source/world schemas. Entries arrive with their domain input contracts.
use super::Kind;

pub mod geometry;
pub mod pack;
pub mod source_data;

// Geometry is a model/terrain input, not a top-level file kind. Register model
// only when the model's remaining parts/materials/processing inputs are covered.
pub(super) const KINDS: &[Kind] = &[
    Kind {
        name: "pack",
        schema: || pack::PackInput::schema(),
    },
    Kind {
        name: "field",
        schema: || source_data::FieldData::schema(),
    },
    Kind {
        name: "sound",
        schema: || source_data::PcmInput::schema(),
    },
    Kind {
        name: "icons",
        schema: || source_data::IconSet::schema(),
    },
    Kind {
        name: "vector",
        schema: || source_data::VectorInput::schema(),
    },
    Kind {
        name: "texture-array",
        schema: || source_data::TextureArrayInput::schema(),
    },
    Kind {
        name: "environment",
        schema: || source_data::EnvironmentInput::schema(),
    },
];
