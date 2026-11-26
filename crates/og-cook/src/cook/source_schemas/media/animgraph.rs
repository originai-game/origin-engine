//! Named animation graphs: no runtime handles or resolved parameter indices.
use super::*;
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

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Motion {
    Clip {
        clip: String,
        #[schemars(extend("exclusiveMinimum" = 0))]
        speed: Option<f32>,
        looping: Option<bool>,
        mirror: Option<bool>,
    },
    Blend1d {
        param: String,
        #[schemars(length(min = 1))]
        children: Vec<(f32, String)>,
    },
    Blend2d {
        x: String,
        y: String,
        #[schemars(length(min = 1))]
        children: Vec<([f32; 2], String)>,
    },
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Layer {
    pub name: String,
    pub mode: Option<LayerMode>,
    #[schemars(range(min = 0, max = 1))]
    pub weight: Option<f32>,
    pub weight_param: Option<String>,
    #[schemars(length(min = 1))]
    pub states: Vec<State>,
    pub entry: String,
    #[serde(default)]
    pub transitions: Vec<Transition>,
    /// Skeleton node names to weights; the domain resolves subtrees.
    #[serde(default)]
    pub mask: BTreeMap<String, f32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum LayerMode {
    Override,
    Additive,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct State {
    pub name: String,
    pub motion: String,
    pub speed: Option<f32>,
    pub sync: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Transition {
    /// Omitted means any state except the target.
    pub from: Option<String>,
    pub to: String,
    #[serde(default)]
    pub conditions: Vec<Condition>,
    /// Source phase, in cycles since entry.
    #[schemars(range(min = 0))]
    pub exit: Option<f32>,
    #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
    pub fade: Option<f32>,
    pub interrupt: Option<Interrupt>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Condition {
    pub param: String,
    pub op: Op,
    pub value: f32,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum Op {
    Gt,
    Ge,
    Lt,
    Le,
    Eq,
    Ne,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum Interrupt {
    Any,
    None,
    Source,
    Next,
}

schema!(Animgraph);
