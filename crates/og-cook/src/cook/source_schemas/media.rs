//! Animation, media and story author inputs, before reference resolution.
//! Cross-field identity, topology and shader compilation stay in the domain
//! validators. The OGT frontends are untouched until their kind cutover.
use super::{ContentRef, Kind};
use og_core::reflect::source::{InputSchema, schemars::JsonSchema};
use serde::Deserialize;
use std::sync::OnceLock;

pub mod animgraph;
pub mod behaviour;
pub mod clips;
pub mod rig2d;
pub mod shader;
pub mod story;
pub mod tiles;
pub mod timeline;
pub mod vfx2d;

pub(super) const KINDS: &[Kind] = &[
    Kind {
        name: "animgraph",
        schema: || animgraph::Animgraph::schema(),
    },
    Kind {
        name: "rig",
        schema: || clips::Rig::schema(),
    },
    Kind {
        name: "motion-action",
        schema: || clips::MotionAction::schema(),
    },
    Kind {
        name: "timeline",
        schema: || timeline::Timeline::schema(),
    },
    Kind {
        name: "flipbook",
        schema: || clips::Flipbook::schema(),
    },
    Kind {
        name: "rig2d",
        schema: || rig2d::Rig2d::schema(),
    },
    Kind {
        name: "tileset",
        schema: || tiles::Tileset::schema(),
    },
    Kind {
        name: "tilemap",
        schema: || tiles::Tilemap::schema(),
    },
    Kind {
        name: "video",
        schema: || clips::Video::schema(),
    },
    Kind {
        name: "vfx2d",
        schema: || vfx2d::Vfx2d::schema(),
    },
    Kind {
        name: "behaviour",
        schema: || behaviour::Behaviour::schema(),
    },
    Kind {
        name: "dialogue",
        schema: || story::Dialogue::schema(),
    },
    Kind {
        name: "quest",
        schema: || story::Quest::schema(),
    },
    Kind {
        name: "shader",
        schema: || shader::Shader::schema(),
    },
    Kind {
        name: "post",
        schema: || shader::Post::schema(),
    },
    Kind {
        name: "ui-material",
        schema: || shader::UiMaterial::schema(),
    },
];

macro_rules! schema {
    ($($ty:ty),+ $(,)?) => {$(
        impl $ty {
            pub fn schema() -> &'static InputSchema<Self> {
                static SCHEMA: OnceLock<InputSchema<$ty>> = OnceLock::new();
                SCHEMA.get_or_init(|| InputSchema::new().expect(concat!(stringify!($ty), " input schema")))
            }
        }
    )+};
}
pub(crate) use schema;

#[cfg(test)]
mod tests;
