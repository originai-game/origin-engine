//! Source/world schemas. Entries arrive with their domain input contracts.
use super::Kind;

pub mod bake;
pub mod district;
pub mod geometry;
pub mod grammar;
pub mod localization;
pub mod materials;
pub mod model;
pub mod pack;
mod records;
pub mod scene;
pub mod source_data;
pub mod table;

// Geometry is a nested model/terrain input, not a top-level file kind.
pub(super) const KINDS: &[Kind] = &[
    Kind {
        name: "model",
        schema: || model::ModelInput::schema(),
    },
    Kind {
        name: "materials",
        schema: || materials::MaterialsInput::schema(),
    },
    Kind {
        name: "district",
        schema: || district::DistrictInput::schema(),
    },
    Kind {
        name: "grammar",
        schema: || grammar::GrammarInput::schema(),
    },
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
