//! Typed story declarations. Yarn is an explicit import, never the default
//! author representation. Expressions are data, not a second source language.
use super::*;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Value {
    Number(f64),
    Bool(bool),
    String(String),
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
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

#[derive(Clone, Debug, Deserialize, JsonSchema)]
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
    /// Owning source node; omit unless this is an explicit continuation block.
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
    pub flow: Option<Flow>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Flow {
    pub title: Option<String>,
    pub chapter: Option<String>,
    pub ending: Option<String>,
    #[schemars(range(min = 0.001, max = 5))]
    pub dissolve: Option<f32>,
    pub music: Option<Music>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Music {
    /// Omit to stop the current music.
    #[schemars(extend("x-og-ref-kinds" = ["sound"]))]
    pub sound: Option<ContentRef>,
    #[schemars(range(min = 0, max = 2))]
    pub volume: Option<f32>,
    #[schemars(range(min = 0))]
    pub fade: Option<f32>,
    #[schemars(range(min = 0, max = 1))]
    pub duck: Option<f32>,
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
    /// Yarn resource names mapped explicitly to checked content references.
    #[serde(default)]
    pub assets: BTreeMap<String, ContentRef>,
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
    /// Quest variables always belong to the shared story, not a conversation.
    #[serde(default)]
    pub globals: BTreeMap<String, Value>,
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

impl From<Value> for og_story::Val {
    fn from(value: Value) -> Self {
        match value {
            Value::Number(v) => Self::Num(v),
            Value::Bool(v) => Self::Bool(v),
            Value::String(v) => Self::Str(v),
        }
    }
}

impl From<Op> for og_story::expr::Op {
    fn from(value: Op) -> Self {
        match value {
            Op::Add => Self::Add,
            Op::Sub => Self::Sub,
            Op::Mul => Self::Mul,
            Op::Div => Self::Div,
            Op::Rem => Self::Rem,
            Op::Eq => Self::Eq,
            Op::Ne => Self::Ne,
            Op::Lt => Self::Lt,
            Op::Le => Self::Le,
            Op::Gt => Self::Gt,
            Op::Ge => Self::Ge,
            Op::And => Self::And,
            Op::Or => Self::Or,
            Op::Xor => Self::Xor,
        }
    }
}

impl From<Expr> for og_story::Expr {
    fn from(value: Expr) -> Self {
        use og_story::expr::Func;
        match value {
            Expr::Literal { value } => Self::Lit(value.into()),
            Expr::Variable { name } => Self::Var(name),
            Expr::Once { id } => Self::Once(id),
            Expr::Not { value } => Self::Not(Box::new((*value).into())),
            Expr::Negate { value } => Self::Neg(Box::new((*value).into())),
            Expr::Binary { op, left, right } => Self::Bin(
                op.into(),
                Box::new((*left).into()),
                Box::new((*right).into()),
            ),
            Expr::Call { name, args } => {
                Self::Invoke(name, args.into_iter().map(Into::into).collect())
            }
            Expr::Seen { node } => Self::Call(Func::Seen(node)),
            Expr::Status { quest } => Self::Call(Func::Status(quest)),
            Expr::Stage { quest } => Self::Call(Func::Stage(quest)),
            Expr::Count { quest, objective } => Self::Call(Func::Count(quest, objective)),
            Expr::Member { name, member } => Self::Member(name, member),
            Expr::EnumValue { name, value } => Self::EnumValue(name, value.into()),
        }
    }
}

impl From<Text> for og_story::Text {
    fn from(value: Text) -> Self {
        Self {
            key: value.key,
            args: value.args.into_iter().map(|(k, v)| (k, v.into())).collect(),
        }
    }
}

impl From<Variable> for og_story::VarDecl {
    fn from(value: Variable) -> Self {
        Self {
            name: value.name,
            value: value.value.into(),
            global: value.global,
        }
    }
}

impl From<Line> for og_story::Line {
    fn from(value: Line) -> Self {
        Self {
            id: value.id,
            speaker: value.speaker.unwrap_or_else(|| og_story::NARRATOR.into()),
            text: value.text.into(),
            voice: value.voice.map_or_else(String::new, |r| r.id),
            expression: value.expression.unwrap_or_default(),
            shot: value.shot.unwrap_or_default(),
            line: 0,
        }
    }
}

impl From<QuestOp> for og_story::QuestOp {
    fn from(value: QuestOp) -> Self {
        match value {
            QuestOp::Start { quest } => Self::Start(quest),
            QuestOp::Progress {
                quest,
                objective,
                amount,
            } => Self::Progress(quest, objective, amount.into()),
            QuestOp::Stage { quest, stage } => Self::Stage(quest, stage),
            QuestOp::Complete { quest } => Self::Complete(quest),
            QuestOp::Fail { quest } => Self::Fail(quest),
        }
    }
}

impl From<Step> for og_story::Step {
    fn from(value: Step) -> Self {
        match value {
            Step::Line(line) => Self::Say(line.into()),
            Step::Set { name, value } => Self::Set(name, value.into()),
            Step::Call { name, args } => Self::Call(og_story::Expr::Invoke(
                name,
                args.into_iter().map(Into::into).collect(),
            )),
            Step::Command { name, args } => Self::Command(args.into_iter().fold(
                og_story::Expr::Lit(og_story::Val::Str(name)),
                |text, arg| {
                    og_story::Expr::Bin(
                        og_story::expr::Op::Add,
                        Box::new(og_story::Expr::Bin(
                            og_story::expr::Op::Add,
                            Box::new(text),
                            Box::new(og_story::Expr::Lit(og_story::Val::Str(" ".into()))),
                        )),
                        Box::new(og_story::Expr::Invoke("string".into(), vec![arg.into()])),
                    )
                },
            )),
            Step::MarkOnce { id } => Self::MarkOnce(id),
            Step::If { condition, node } => Self::If(condition.into(), node),
            Step::Goto { node } => Self::Goto(node),
            Step::Jump { node } => Self::Jump(node.into()),
            Step::End => Self::End,
            Step::Emit { name, value } => Self::Emit(name, value.map(Into::into)),
            Step::Quest { operation } => Self::Quest(operation.into()),
            Step::Detour { node, continuation } => Self::Detour(node, continuation),
            Step::DynamicDetour { node, continuation } => {
                Self::DynamicDetour(node.into(), continuation)
            }
            Step::Select { candidates } => Self::Select(
                candidates
                    .into_iter()
                    .map(|c| (c.node, c.condition.map(Into::into), c.weight))
                    .collect(),
            ),
            Step::Return => Self::Return,
        }
    }
}

impl From<Choice> for og_story::Choice {
    fn from(value: Choice) -> Self {
        Self {
            id: value.id,
            text: value.text.into(),
            cond: value.condition.map(Into::into),
            hide: value.hide,
            reason: value.reason.map(Into::into),
            kind: value.kind.unwrap_or_default(),
            check: value.check.map(|c| og_story::Check {
                active: c.active,
                stat: c.stat,
                target: c.target.into(),
            }),
            steps: value.steps.into_iter().map(Into::into).collect(),
            goto: value.goto,
            fail: value.fail,
            tags: value.tags,
            line: 0,
        }
    }
}

impl From<Node> for og_story::Node {
    fn from(value: Node) -> Self {
        let steps: Vec<og_story::Step> = value.steps.into_iter().map(Into::into).collect();
        Self {
            title: value.title.unwrap_or_else(|| value.id.clone()),
            id: value.id,
            choice_line: steps
                .iter()
                .rposition(|s| matches!(s, og_story::Step::Say(_))),
            lines: vec![0; steps.len()],
            steps,
            choices: value.choices.into_iter().map(Into::into).collect(),
            timer: value.timer,
            choice_fallback: value.choice_fallback,
            shot: value.shot.unwrap_or_default(),
            line: 0,
        }
    }
}

impl From<Bark> for og_story::Bark {
    fn from(value: Bark) -> Self {
        Self {
            id: value.id,
            topic: value.topic,
            speaker: value.speaker.unwrap_or_default(),
            lines: value.lines.into_iter().map(Into::into).collect(),
            when: value.when.map(Into::into),
            priority: value.priority.unwrap_or(0),
            weight: value.weight.unwrap_or(1.0),
            cooldown: value.cooldown.unwrap_or(0),
            shared: value.shared.unwrap_or(0),
            duration: value.duration.unwrap_or(180),
            random: value.random.unwrap_or(false),
            once: value.once.unwrap_or(false),
            steps: value.steps.into_iter().map(Into::into).collect(),
            line: 0,
        }
    }
}

impl TypedDialogue {
    pub fn build(
        self,
        functions: BTreeMap<String, og_story::yarn::types::Function>,
    ) -> Result<og_story::Dialogue, String> {
        let headers = self
            .nodes
            .iter()
            .filter_map(|n| {
                let mut fields = BTreeMap::new();
                if let Some(segment) = &n.segment {
                    fields.insert("og-flow".into(), segment.id.clone());
                }
                if let Some(flow) = &n.flow {
                    for (key, value) in [
                        ("og-flow-title", &flow.title),
                        ("og-flow-chapter", &flow.chapter),
                        ("og-flow-ending", &flow.ending),
                    ] {
                        if let Some(value) = value {
                            fields.insert(key.into(), value.clone());
                        }
                    }
                    if let Some(seconds) = flow.dissolve {
                        fields.insert("og-flow-dissolve".into(), seconds.to_string());
                    }
                    if let Some(music) = &flow.music {
                        let mut value = music.sound.as_ref().map_or("none", |r| &r.id).to_string();
                        for (key, number) in [
                            ("volume", music.volume),
                            ("fade", music.fade),
                            ("duck", music.duck),
                        ] {
                            if let Some(number) = number {
                                value.push_str(&format!(" {key}={number}"));
                            }
                        }
                        fields.insert("og-flow-music".into(), value);
                    }
                }
                (!fields.is_empty()).then(|| (n.title.as_ref().unwrap_or(&n.id).clone(), fields))
            })
            .collect();
        let mut dialogue = og_story::Dialogue {
            name: self.name,
            start: self.start,
            file_tags: Vec::new(),
            headers,
            speakers: self
                .speakers
                .into_iter()
                .map(|s| og_story::Speaker {
                    id: s.id,
                    name: s.name.unwrap_or_default(),
                    portrait: s.portrait.map_or_else(String::new, |r| r.id),
                    expression: s.expression.unwrap_or_default(),
                    side: match s.side.unwrap_or(Side::Left) {
                        Side::Left => "left",
                        Side::Right => "right",
                        Side::Center => "center",
                    }
                    .into(),
                })
                .collect(),
            vars: self.vars.into_iter().map(Into::into).collect(),
            smart: self.smart.into_iter().map(|(k, v)| (k, v.into())).collect(),
            functions,
            group_members: BTreeMap::new(),
            saliency_scores: BTreeMap::new(),
            untracked: Default::default(),
            nodes: self.nodes.into_iter().map(Into::into).collect(),
            barks: self.barks.into_iter().map(Into::into).collect(),
            strings: self.strings,
            locale: self.locale.unwrap_or_else(|| "en".into()),
        };
        dialogue.check_native()?;
        Ok(dialogue)
    }
}

impl Dialogue {
    pub fn build(
        self,
        functions: BTreeMap<String, og_story::yarn::types::Function>,
    ) -> Result<og_story::asset::Asset, String> {
        let mut asset =
            match self {
                Self::Typed(input) => og_story::asset::Asset::Dialogue(input.build(functions)?),
                Self::Yarn(input) => {
                    let dialogue = og_story::yarn::compile_with_functions(&input.yarn, functions)?;
                    let mut asset = og_story::asset::Asset::Dialogue(dialogue);
                    asset.map_resources(|_, id| {
                        *id = input.assets.get(id).ok_or_else(|| format!(
                        "Yarn resource '{id}' needs an explicit assets mapping to ref(id)"
                    ))?.id.clone();
                        Ok(())
                    })?;
                    asset
                }
            };
        if let og_story::asset::Asset::Dialogue(d) = &mut asset {
            for (key, text) in &d.strings {
                og_i18n::Message::parse(text).map_err(|e| format!("base string {key}: {e}"))?;
            }
        }
        asset.validate()?;
        Ok(asset)
    }
}

impl From<Quest> for og_story::Quest {
    fn from(value: Quest) -> Self {
        Self {
            id: value.id,
            title: value.title,
            summary: value.summary.unwrap_or_default(),
            category: value.category.unwrap_or_else(|| "main".into()),
            giver: value.giver.unwrap_or_default(),
            region: value.region.unwrap_or_default(),
            vars: value
                .globals
                .into_iter()
                .map(|(name, value)| og_story::VarDecl {
                    name,
                    value: value.into(),
                    global: true,
                })
                .collect(),
            rewards: value
                .rewards
                .into_iter()
                .map(|r| og_story::Reward {
                    name: r.name,
                    amount: r.amount.unwrap_or(1.0),
                    image: r.image.map_or_else(String::new, |r| r.id),
                    rarity: r.rarity.unwrap_or_default(),
                })
                .collect(),
            stages: value
                .stages
                .into_iter()
                .map(|s| og_story::Stage {
                    id: s.id,
                    summary: s.summary.unwrap_or_default(),
                    objectives: s
                        .objectives
                        .into_iter()
                        .map(|o| og_story::Objective {
                            id: o.id,
                            text: o.text.into(),
                            target: o.target.unwrap_or(1),
                            when: o.when.map(Into::into),
                            optional: o.optional.unwrap_or(false),
                            hidden: o.hidden.unwrap_or(false),
                            marker: o.marker,
                            on: o.on,
                            filter: o.filter.map(Into::into),
                            line: 0,
                        })
                        .collect(),
                    next: s.next,
                    fail: s.fail.map(Into::into),
                    line: 0,
                })
                .collect(),
        }
    }
}
