//! Source/world schemas. Entries arrive with their domain input contracts.
use super::Kind;

pub mod bake;
pub mod geometry;
pub mod localization;
pub mod pack;
mod records;
pub mod scene;
pub mod source_data;
pub mod table;

// Geometry is a model/terrain input, not a top-level file kind. Register model
// only when the model's remaining parts/materials/processing inputs are covered.
pub(super) const KINDS: &[Kind] = &[
    Kind {
        name: "navmesh",
        schema: || bake::NavmeshInput::schema(),
    },
    Kind {
        name: "lightmap",
        schema: || bake::LightmapInput::schema(),
    },
    Kind {
        name: "strings",
        schema: || localization::StringsInput::schema(),
    },
    Kind {
        name: "font-extension",
        schema: || localization::FontExtensionInput::schema(),
    },
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
