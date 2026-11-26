//! Ordered shape-grammar operations, not source text embedded in a TS field.
//! Terminal ordering, rule/parameter existence and shrinking recursion are
//! grammar-domain checks; every operation's structural shape is closed here.
use super::super::ContentRef;
use og_core::reflect::source::{InputSchema, schemars::JsonSchema};
use serde::Deserialize;
use std::{collections::BTreeMap, sync::OnceLock};

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct GrammarInput {
    #[serde(default)]
    pub params: BTreeMap<String, f64>,
    #[schemars(length(min = 1))]
    pub start: String,
    pub rules: BTreeMap<String, Vec<Operation>>,
}

/// A literal or a named grammar parameter. This is not an expression evaluator.
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Number {
    Literal(f64),
    Parameter(#[schemars(length(min = 1))] String),
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "op", rename_all = "camelCase", deny_unknown_fields)]
pub enum Operation {
    Recess {
        #[schemars(extend("x-og-unit" = "m"))]
        depth: Number,
        jamb: Option<String>,
    },
    Detail {
        #[schemars(range(max = 16))]
        level: u32,
    },
    Label {
        #[schemars(length(min = 1))]
        label: String,
    },
    Extrude {
        #[schemars(extend("x-og-unit" = "m"))]
        depth: Number,
        faces: Faces,
    },
    Split {
        axis: Axis,
        #[schemars(length(min = 1))]
        parts: Vec<SplitPart>,
    },
    Inset {
        #[schemars(extend("x-og-unit" = "m"))]
        width: Number,
        border: String,
        inner: String,
    },
    Roof {
        roof: Roof,
        slope: String,
        gable: Option<String>,
        eave: Option<String>,
        #[schemars(extend("x-og-unit" = "m"))]
        overhang: Option<Number>,
    },
    Insert {
        #[schemars(extend("x-og-ref-kinds" = ["model"]))]
        model: ContentRef,
    },
    Pick {
        #[schemars(length(min = 1))]
        options: Vec<WeightedRule>,
    },
    Case {
        arms: Vec<CaseArm>,
        otherwise: Option<String>,
    },
    Drop,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Faces {
    pub top: Option<String>,
    pub bottom: Option<String>,
    pub front: Option<String>,
    pub back: Option<String>,
    pub side: Option<String>,
    pub sides: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum Axis {
    X,
    Y,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum Size {
    Abs,
    Fill,
    Repeat,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct SplitPart {
    pub size: Size,
    pub length: Number,
    pub rule: String,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Roof {
    Flat,
    Gable {
        #[schemars(extend("x-og-unit" = "deg"))]
        angle: Number,
    },
    Hip {
        #[schemars(extend("x-og-unit" = "deg"))]
        angle: Number,
    },
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct WeightedRule {
    #[schemars(extend("exclusiveMinimum" = 0))]
    pub weight: f64,
    pub rule: String,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct CaseArm {
    pub condition: Condition,
    pub rule: String,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Condition {
    Wider {
        #[schemars(extend("x-og-unit" = "m"))]
        value: Number,
    },
    Taller {
        #[schemars(extend("x-og-unit" = "m"))]
        value: Number,
    },
    Param {
        name: String,
        value: Number,
    },
}

impl GrammarInput {
    pub fn schema() -> &'static InputSchema<Self> {
        static SCHEMA: OnceLock<InputSchema<GrammarInput>> = OnceLock::new();
        SCHEMA.get_or_init(|| InputSchema::new().expect("grammar input schema"))
    }
}
