//! Named animation graphs: no runtime handles or resolved parameter indices.
use super::*;
pub use og_anim::{LayerInput as Layer, MotionInput as Motion};
use std::collections::BTreeMap;

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Animgraph {
    #[serde(default)]
    pub sprite: bool,
    #[serde(default)]
    pub bindings: Vec<Binding>,
    #[serde(default)]
    #[schemars(length(max = 8))]
    pub params: Vec<String>,
    pub motions: BTreeMap<String, Motion>,
    #[schemars(length(min = 1))]
    pub layers: Vec<Layer>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub name: String,
    #[schemars(extend("x-og-ref-kinds" = ["model"]))]
    pub source: ContentRef,
    pub clip: String,
}

schema!(Animgraph);
