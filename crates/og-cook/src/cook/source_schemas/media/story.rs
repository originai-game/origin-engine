//! Typed story declarations. Yarn is an explicit import, never the default
//! author representation. Expressions are data, not a second source language.
use super::*;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Value {
    Number(f64),
    Bool(bool),
    String(String),
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Expr {
    Literal {
        value: Value,
    },
    Variable {
        name: String,
    },
    Once {
        id: String,
    },
    Not {
        value: Box<Expr>,
    },
    Negate {
        value: Box<Expr>,
    },
    Binary {
        op: Op,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
    Seen {
        node: String,
    },
    Status {
        quest: String,
    },
    Stage {
        quest: String,
    },
    Count {
        quest: String,
        objective: String,
    },
    Member {
        name: String,
        member: String,
    },
    EnumValue {
        name: String,
        value: Value,
    },
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "kebab-case")]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Xor,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Text {
    /// Localization key, not the rendered sentence.
    pub key: String,
    #[serde(default)]
    pub args: BTreeMap<String, Expr>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Variable {
    pub name: String,
    pub value: Value,
    #[serde(default)]
    pub global: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Speaker {
    pub id: String,
    pub name: Option<String>,
    #[schemars(extend("x-og-ref-kinds" = ["texture"]))]
    pub portrait: Option<ContentRef>,
    pub expression: Option<String>,
    pub side: Option<Side>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Left,
    Right,
    Center,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Step {
    Line(Line),
    Set { name: String, value: Expr },
    Call { name: String, args: Vec<Expr> },
    Command { name: String, args: Vec<Expr> },
    MarkOnce { id: String },
    If { condition: Expr, node: String },
    Goto { node: String },
    Jump { node: Expr },
    End,
    Emit { name: String, value: Option<Expr> },
    Quest { operation: QuestOp },
    Detour { node: String, continuation: String },
    DynamicDetour { node: Expr, continuation: String },
    Select { candidates: Vec<Candidate> },
    Return,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Line {
    /// Stable author identity, independent of source position.
    pub id: String,
    pub speaker: Option<String>,
    pub text: Text,
    #[schemars(extend("x-og-ref-kinds" = ["sound"]))]
    pub voice: Option<ContentRef>,
    pub expression: Option<String>,
    pub shot: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    pub node: String,
    pub condition: Option<Expr>,
    #[schemars(range(min = 0))]
    pub weight: f64,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum QuestOp {
    Start {
        quest: String,
    },
    Progress {
        quest: String,
        objective: String,
        amount: Expr,
    },
    Stage {
        quest: String,
        stage: String,
    },
    Complete {
        quest: String,
    },
    Fail {
        quest: String,
    },
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub id: String,
    pub text: Text,
    pub condition: Option<Expr>,
    #[serde(default)]
    pub hide: bool,
    pub reason: Option<Text>,
    pub kind: Option<String>,
    pub check: Option<Check>,
    #[serde(default)]
    pub steps: Vec<Step>,
    pub goto: Option<String>,
    pub fail: Option<String>,
    #[serde(default)]
    pub tags: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Check {
    pub active: bool,
    pub stat: String,
    pub target: Expr,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Node {
    pub id: String,
    pub title: Option<String>,
    #[serde(default)]
    pub steps: Vec<Step>,
    #[serde(default)]
    pub choices: Vec<Choice>,
    /// Tick count and fallback choice identity, not an array index.
    pub timer: Option<(u32, String)>,
    pub choice_fallback: Option<String>,
    pub shot: Option<String>,
    #[schemars(extend("x-og-ref-kinds" = ["timeline"]))]
    pub segment: Option<ContentRef>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged, deny_unknown_fields)]
pub enum Dialogue {
    Typed(TypedDialogue),
    Yarn(YarnImport),
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct YarnImport {
    /// Explicit Yarn source text from input.text; never automatically scanned.
    pub yarn: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct TypedDialogue {
    pub name: String,
    pub start: String,
    pub locale: Option<String>,
    #[serde(default)]
    pub speakers: Vec<Speaker>,
    #[serde(default)]
    pub vars: Vec<Variable>,
    /// Lazily evaluated variables expressed in the same typed expression tree.
    #[serde(default)]
    pub smart: BTreeMap<String, Expr>,
    /// Stable line key to ICU base text. ICU remains a string language.
    #[serde(default)]
    pub strings: BTreeMap<String, String>,
    #[schemars(length(min = 1))]
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub barks: Vec<Bark>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Bark {
    pub id: String,
    pub topic: String,
    pub speaker: Option<String>,
    pub lines: Vec<Line>,
    pub when: Option<Expr>,
    pub priority: Option<i32>,
    #[schemars(range(min = 0))]
    pub weight: Option<f64>,
    #[schemars(extend("x-og-unit" = "ticks"))]
    pub cooldown: Option<u32>,
    #[schemars(extend("x-og-unit" = "ticks"))]
    pub shared: Option<u32>,
    #[schemars(extend("x-og-unit" = "ticks"))]
    pub duration: Option<u32>,
    pub random: Option<bool>,
    pub once: Option<bool>,
    #[serde(default)]
    pub steps: Vec<Step>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Quest {
    pub id: String,
    pub title: String,
    pub summary: Option<String>,
    pub category: Option<String>,
    pub giver: Option<String>,
    pub region: Option<String>,
    #[serde(default)]
    pub vars: Vec<Variable>,
    #[serde(default)]
    pub rewards: Vec<Reward>,
    #[schemars(length(min = 1))]
    pub stages: Vec<Stage>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Reward {
    pub name: String,
    pub amount: Option<f64>,
    #[schemars(extend("x-og-ref-kinds" = ["texture"]))]
    pub image: Option<ContentRef>,
    pub rarity: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Stage {
    pub id: String,
    pub summary: Option<String>,
    #[serde(default)]
    pub objectives: Vec<Objective>,
    pub next: Option<String>,
    pub fail: Option<Expr>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Objective {
    pub id: String,
    pub text: Text,
    #[schemars(range(min = 1, max = 1_000_000))]
    pub target: Option<u32>,
    pub when: Option<Expr>,
    pub optional: Option<bool>,
    pub hidden: Option<bool>,
    #[schemars(extend("x-og-unit" = "m"))]
    pub marker: Option<[f32; 3]>,
    pub on: Option<String>,
    pub filter: Option<Expr>,
}

schema!(Dialogue, Quest);
