//! Names are resolved by the behaviour validator, not encoded as node indices.
use super::*;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Vector([f32; 3]),
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Speed {
    Fixed(f32),
    Read(String),
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Behaviour {
    pub name: String,
    #[serde(default)]
    pub keys: BTreeMap<String, Value>,
    /// Named patrol points in world units.
    #[serde(default)]
    pub routes: BTreeMap<String, Vec<[f32; 3]>>,
    pub root: Root,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Root {
    Tree { node: Node },
    Machine { initial: String, states: Vec<State> },
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct State {
    pub name: String,
    #[serde(default)]
    pub enter: Vec<Action>,
    #[serde(default)]
    pub exit: Vec<Action>,
    pub body: Option<Node>,
    #[serde(default)]
    pub transitions: Vec<Transition>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Transition {
    pub to: String,
    #[serde(default)]
    pub checks: Vec<Check>,
    #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
    pub after: Option<f32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Check {
    /// Blackboard key or reflected Component.field / Resource.field path.
    pub operand: String,
    pub op: Op,
    pub value: Option<Value>,
    /// Dynamic comparison operand; exclusive with value (domain check).
    pub against: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase")]
pub enum Op {
    Is,
    Not,
    Below,
    Above,
    AtMost,
    AtLeast,
    Truthy,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Node {
    Selector {
        children: Vec<Node>,
    },
    Sequence {
        children: Vec<Node>,
    },
    Steps {
        children: Vec<Node>,
    },
    Parallel {
        all: bool,
        children: Vec<Node>,
    },
    Invert {
        child: Box<Node>,
    },
    Succeed {
        child: Box<Node>,
    },
    Repeat {
        times: u32,
        child: Box<Node>,
    },
    Cooldown {
        #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
        seconds: f32,
        child: Box<Node>,
    },
    Timeout {
        #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
        seconds: f32,
        child: Box<Node>,
    },
    Check {
        check: Check,
    },
    Action {
        action: Action,
    },
    Subtree {
        #[schemars(extend("x-og-ref-kinds" = ["behaviour"]))]
        source: ContentRef,
    },
    Utility {
        inertia: f32,
        options: Vec<UtilityOption>,
    },
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct UtilityOption {
    pub name: String,
    #[schemars(length(min = 1))]
    pub scores: Vec<Consideration>,
    #[schemars(length(min = 1))]
    pub children: Vec<Node>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Consideration {
    Constant {
        value: f32,
    },
    Read {
        operand: String,
        curve: Curve,
        lo: f32,
        hi: f32,
    },
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum Curve {
    Linear,
    Inverse,
    Quadratic,
    Sqrt,
    Smooth,
    Step,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Action {
    MoveTo {
        key: Option<String>,
        point: Option<[f32; 3]>,
        speed: Option<Speed>,
    },
    Chase {
        key: String,
        speed: Option<Speed>,
        reach: f32,
    },
    Flee {
        key: String,
        speed: Option<Speed>,
        distance: f32,
    },
    Patrol {
        route: String,
        wait: f32,
        speed: Option<Speed>,
        loops: bool,
    },
    Wander {
        radius: f32,
        speed: Option<Speed>,
        wait: f32,
    },
    Wait {
        #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
        seconds: f32,
    },
    Stop,
    Set {
        key: String,
        value: Value,
    },
    Command {
        name: String,
        args: Vec<String>,
        cooldown: f32,
    },
    Game {
        name: String,
        args: Vec<String>,
    },
}

schema!(Behaviour);
