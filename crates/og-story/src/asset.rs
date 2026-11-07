//! Engine-owned story IR compiled from `.yarn` by [`crate::yarn`], and
//! quests parsed from `type quest` data. Yarn lines carry base text and
//! explicit stable `#line:` IDs; the runtime resolves them through i18n.
//! Continuation blocks use semantic IDs, not source positions. The
//! dialogue box still reads the same lines, speakers, choices and shots.
//!
//! ```text
//! type quest
//! version 1
//! ---
//! id rescue
//! title "quest.rescue"
//! summary "quest.rescue.summary"
//! category main
//! giver "npc.smith"
//! region "region.vale"
//! global wolvesFled false
//! reward "item.gold" 50 {
//!   image "art/gold.png"
//!   rarity common
//! }
//! stage hunt {
//!   objective wolves "quest.rescue.wolves" {
//!     target 5
//!     marker 12 0 4
//!   }
//!   objective sword "quest.rescue.sword" {
//!     optional
//!   }
//!   fail "wolvesFled"
//!   next report
//! }
//! stage report {
//!   objective talk "quest.rescue.talk" {
//!     when "seen(smith.bought)"
//!   }
//! }
//! ```
//!
//! A stage is done when every objective that is not `optional` is; the
//! quest moves to `next`, or completes after its last stage. `fail`
//! fails the quest when its condition holds; `when` completes an
//! objective when its condition holds; `hidden` keeps an objective out of
//! the log until it first counts.

use crate::expr::{Expr, Func, Val};
use og_core::ogt::{Document, Entry, Value as OgtValue};
use std::collections::BTreeSet;

/// The cooked asset kind (legacy `.ogt` sources receive a migration error).
pub const DIALOGUE_TYPE: &str = "dialogue";
/// The `.ogt` type of a quest.
pub const QUEST_TYPE: &str = "quest";
/// The speaker every dialogue has without declaring it: no name, no portrait.
pub const NARRATOR: &str = "narrator";

/// A variable a dialogue or quest declares.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct VarDecl {
    pub name: String,
    pub value: Val,
    /// `global`: the story's, shared by every dialogue and quest; else the
    /// conversation's own.
    pub global: bool,
}

/// A speaker.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Speaker {
    pub id: String,
    /// The i18n key of its name (empty: none).
    pub name: String,
    /// A portrait image's asset path (empty: none).
    pub portrait: String,
    /// The portrait's default variant.
    pub expression: String,
    /// `left`, `right` or `center`.
    pub side: String,
}

/// An i18n key with its arguments (expressions, evaluated when shown).
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Text {
    pub key: String,
    pub args: Vec<(String, Expr)>,
}

/// A line.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Line {
    pub speaker: String,
    pub text: Text,
    /// A voice clip's asset path (empty: none).
    pub voice: String,
    /// The portrait variant (empty: the speaker's).
    pub expression: String,
    /// The camera shot it is shown in: a virtual camera's entity name or
    /// a cast role (empty: the node's; `none`: no shot even if the node
    /// has one).
    pub shot: String,
    pub line: usize,
}

/// A quest operation a dialogue step or choice performs.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum QuestOp {
    Start(String),
    Progress(String, String, Expr),
    Stage(String, String),
    Complete(String),
    Fail(String),
}

/// A step of a node.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Step {
    Say(Line),
    Set(String, Expr),
    Call(Expr),
    Command(Expr),
    MarkOnce(String),
    /// `if "cond" node`.
    If(Expr, String),
    Goto(String),
    /// Yarn jump exits the complete detour stack; Goto only links IR blocks.
    Jump(Expr),
    End,
    /// `emit name ["arg"]`.
    Emit(String, Option<Expr>),
    Quest(QuestOp),
    /// Calls a Yarn node, returning to the named continuation block.
    Detour(String, String),
    DynamicDetour(Expr, String),
    Return,
    /// Eligible line-group candidates: target, condition, weight.
    Select(Vec<(String, Option<Expr>, f64)>),
}

/// A check on a choice: `passive` enables it only when the stat reaches
/// the target; `active` rolls d20 + the stat against the target when it
/// is chosen, going to the choice's `goto` on a pass and `fail` on a miss.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Check {
    pub active: bool,
    /// The variable read as the stat.
    pub stat: String,
    pub target: Expr,
}

/// A choice.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Choice {
    /// Its id within the node (`id`, else its key): what `visited` and a
    /// node's `timer` name.
    pub id: String,
    pub text: Text,
    /// Shown enabled only when this holds.
    pub cond: Option<Expr>,
    /// Hidden, not disabled, while it does not hold.
    pub hide: bool,
    /// Why it is disabled.
    pub reason: Option<Text>,
    /// The game's word for its icon: leave, trade, attack, lie...
    pub kind: String,
    pub check: Option<Check>,
    /// Steps run when it is chosen (`set`, `emit`, `quest`).
    pub steps: Vec<Step>,
    pub goto: Option<String>,
    /// Where an active check's miss goes.
    pub fail: Option<String>,
    pub line: usize,
}

/// A node.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Node {
    pub id: String,
    pub title: String,
    pub steps: Vec<Step>,
    /// Each step's source line.
    pub lines: Vec<usize>,
    pub choices: Vec<Choice>,
    /// `timer ticks choice`: the choice taken when the time runs out.
    pub timer: Option<(u32, String)>,
    /// The step of the line its choices are shown with (its last `say`);
    /// `None` without one (they show with the line before).
    pub choice_line: Option<usize>,
    /// Continuation when the host selects Yarn's NoOptionSelected (-1).
    #[serde(default)]
    pub choice_fallback: Option<String>,
    /// `shot <camera>`: the shot its lines are shown in unless a line
    /// names its own (empty: none).
    pub shot: String,
    pub line: usize,
}

/// A parsed dialogue.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Dialogue {
    pub name: String,
    pub start: String,
    #[serde(default)]
    pub file_tags: Vec<String>,
    #[serde(default)]
    pub headers: std::collections::BTreeMap<String, std::collections::BTreeMap<String, String>>,
    pub speakers: Vec<Speaker>,
    pub vars: Vec<VarDecl>,
    /// Read-only expressions evaluated when a Yarn variable is read.
    #[serde(default)]
    pub smart: std::collections::BTreeMap<String, Expr>,
    #[serde(default)]
    pub functions: std::collections::BTreeMap<String, crate::yarn::types::Function>,
    #[serde(default)]
    pub group_members: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    pub saliency_scores: std::collections::BTreeMap<String, u32>,
    #[serde(default)]
    pub untracked: BTreeSet<String>,
    pub nodes: Vec<Node>,
    pub barks: Vec<Bark>,
    pub strings: std::collections::BTreeMap<String, String>,
    pub locale: String,
}

/// A bark: an ambient line an entity says on its own (a guard's yawn, a
/// shopkeeper's call), not a conversation. og.story.bark asks for one of a
/// topic; og.story.ambient asks every so often. The highest `priority` of
/// the topic whose `when` holds and whose cooldowns are over is said; ties
/// are drawn by `weight` on the story's stream.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Bark {
    pub id: String,
    pub topic: String,
    /// The speaker it is for (empty: any): its name and portrait.
    pub speaker: String,
    pub lines: Vec<Line>,
    /// Said only while this holds; it may read the world.
    pub when: Option<Expr>,
    pub priority: i32,
    pub weight: f64,
    /// Ticks before the same entity says it again.
    pub cooldown: u32,
    /// Ticks before anyone says it again.
    pub shared: u32,
    /// Ticks it shows (and holds its speaker against lower priorities).
    pub duration: u32,
    /// Its lines in turn (`cycle`, the default) or drawn (`random`).
    pub random: bool,
    /// Said once per playthrough.
    pub once: bool,
    /// Steps run when it is said (`set`, `emit`, `quest`).
    pub steps: Vec<Step>,
    pub line: usize,
}

impl Dialogue {
    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }
    pub fn speaker(&self, id: &str) -> Option<&Speaker> {
        self.speakers.iter().find(|s| s.id == id)
    }
    pub fn bark(&self, id: &str) -> Option<&Bark> {
        self.barks.iter().find(|b| b.id == id)
    }
}

/// An objective of a stage.
#[derive(Clone, Debug, PartialEq)]
pub struct Objective {
    pub id: String,
    pub text: Text,
    /// Counts needed (default 1).
    pub target: u32,
    /// Done when this holds.
    pub when: Option<Expr>,
    pub optional: bool,
    pub hidden: bool,
    pub marker: Option<[f32; 3]>,
    /// `on <Event>`: each such event counts one, with no game code (the
    /// og.story system reads it the tick after it is sent).
    pub on: Option<String>,
    /// `where "<cond>"`: an `on` event counts only when this holds; it reads
    /// the event's fields as `event.<field>` and the world.
    pub filter: Option<Expr>,
    pub line: usize,
}

/// A stage of a quest.
#[derive(Clone, Debug, PartialEq)]
pub struct Stage {
    pub id: String,
    pub summary: String,
    pub objectives: Vec<Objective>,
    pub next: Option<String>,
    pub fail: Option<Expr>,
    pub line: usize,
}

/// A reward shown in the quest log.
#[derive(Clone, Debug, PartialEq)]
pub struct Reward {
    /// Its name's i18n key.
    pub name: String,
    pub amount: f64,
    pub image: String,
    pub rarity: String,
}

/// A parsed quest.
#[derive(Clone, Debug, PartialEq)]
pub struct Quest {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub category: String,
    pub giver: String,
    pub region: String,
    pub vars: Vec<VarDecl>,
    pub rewards: Vec<Reward>,
    pub stages: Vec<Stage>,
}

impl Quest {
    pub fn stage(&self, id: &str) -> Option<&Stage> {
        self.stages.iter().find(|s| s.id == id)
    }
}

fn err(e: &Entry, msg: impl Into<String>) -> String {
    format!("line {}: {}", e.line, msg.into())
}

fn word_or_str(v: &OgtValue) -> Option<&str> {
    v.as_word().or_else(|| v.as_str())
}

/// The entry's value `i` as a word (or string).
fn name_at(e: &Entry, i: usize, what: &str) -> Result<String, String> {
    e.values
        .get(i)
        .and_then(word_or_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .ok_or_else(|| err(e, format!("'{}' needs {what}", e.key)))
}

fn key_at(e: &Entry, i: usize) -> Result<String, String> {
    match e.values.get(i) {
        Some(OgtValue::String(s)) if !s.is_empty() => Ok(s.clone()),
        _ => Err(err(
            e,
            format!(
                "'{}' needs an og.i18n key in quotes, such as \"npc.hello\" (text is never written inline: put it in your string tables)",
                e.key
            ),
        )),
    }
}

/// A value written as a literal (number, true/false) or a quoted expression.
fn expr_at(e: &Entry, i: usize) -> Result<Expr, String> {
    match e.values.get(i) {
        Some(OgtValue::Number(n)) => Ok(Expr::Lit(Val::Num(n.as_f64()))),
        Some(v) if v.as_bool().is_some() => Ok(Expr::Lit(Val::Bool(v.as_bool().unwrap()))),
        Some(OgtValue::String(s)) => Expr::parse(s).map_err(|m| err(e, m)),
        _ => Err(err(
            e,
            format!(
                "'{}' needs a value: a number, true/false or an expression in quotes",
                e.key
            ),
        )),
    }
}

fn var_decl(e: &Entry, global: bool) -> Result<VarDecl, String> {
    let name = name_at(e, 0, "a name")?;
    if name.contains('.') || matches!(name.as_str(), "and" | "or" | "not" | "true" | "false") {
        return Err(err(e, format!("'{name}' cannot name a variable")));
    }
    let value = match e.values.get(1) {
        Some(OgtValue::Number(n)) => Val::Num(n.as_f64()),
        Some(OgtValue::String(s)) => Val::Str(s.clone()),
        Some(v) if v.as_bool().is_some() => Val::Bool(v.as_bool().unwrap()),
        _ => {
            return Err(err(
                e,
                format!(
                    "'{} {name}' needs a starting value: a number, true/false or a string",
                    e.key
                ),
            ));
        }
    };
    Ok(VarDecl {
        name,
        value,
        global,
    })
}

fn text_of(e: &Entry, i: usize) -> Result<Text, String> {
    let key = key_at(e, i)?;
    let mut args = Vec::new();
    if let Some(b) = &e.block {
        for a in b.get_all("arg") {
            args.push((name_at(a, 0, "a name")?, expr_at(a, 1)?));
        }
    }
    Ok(Text { key, args })
}

fn header(text: &str, ty: &str) -> Result<Document, String> {
    let doc = Document::parse(text).map_err(|e| e.reason().to_string())?;
    if doc.ty != ty {
        return Err(format!("expected 'type {ty}', found 'type {}'", doc.ty));
    }
    if doc.version != 1 {
        return Err(format!(
            "{ty} version {} is not supported (1 is)",
            doc.version
        ));
    }
    Ok(doc)
}

/// Checks every variable an expression reads (`vars` declares them) and
/// every `seen(node)` without a dialogue (`nodes`, when given).
fn check_expr(
    x: &Expr,
    line: usize,
    vars: &BTreeSet<&str>,
    nodes: Option<&BTreeSet<&str>>,
) -> Result<(), String> {
    check_reads(x, line, vars, nodes, false)
}

/// [`check_expr`], where `world` allows world reads (`Game.alarm`,
/// `self.Health.hp`, `event.kind`): a bark's `when` and arguments, an
/// objective's `where`, evaluated where the world is at hand.
fn check_reads(
    x: &Expr,
    line: usize,
    vars: &BTreeSet<&str>,
    nodes: Option<&BTreeSet<&str>>,
    world: bool,
) -> Result<(), String> {
    let mut vs = Vec::new();
    x.vars(&mut vs);
    if let Some(v) = vs.iter().find(|v| v.contains('.')) {
        if !world {
            return Err(format!(
                "line {line}: '{v}' reads the world, which only a bark's when and arguments and an objective's where can: copy it into a story variable with og.story.set"
            ));
        }
        if v.split('.').count() < 2 || v.ends_with('.') || v.starts_with('.') {
            return Err(format!(
                "line {line}: '{v}' is not a world read: write Record.field, self.Component.field or event.field"
            ));
        }
    }
    if let Some(v) = vs
        .iter()
        .find(|v| !v.contains('.') && !vars.contains(v.as_str()))
    {
        return Err(format!(
            "line {line}: '{v}' is not declared: in Yarn add <<declare ${v} = value>>; in a quest add global {v} value"
        ));
    }
    let mut calls = Vec::new();
    x.calls(&mut calls);
    for c in calls {
        if let Func::Seen(n) = c {
            match nodes {
                Some(ns) if !n.contains('.') && !ns.contains(n.as_str()) => {
                    return Err(format!(
                        "line {line}: seen({n}) names no node of this dialogue (write dialogue.node for another's)"
                    ));
                }
                None if !n.contains('.') => {
                    return Err(format!(
                        "line {line}: seen({n}) needs the dialogue: seen(dialogue.node)"
                    ));
                }
                _ => {}
            }
        }
    }
    Ok(())
}

impl Dialogue {
    /// Parses and validates a dialogue.
    pub fn parse(text: &str) -> Result<Dialogue, String> {
        crate::yarn::compile(text)
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        let nodes: BTreeSet<&str> = self.nodes.iter().map(|n| n.id.as_str()).collect();
        let vars: BTreeSet<&str> = self
            .vars
            .iter()
            .map(|v| v.name.as_str())
            .chain(self.smart.keys().map(String::as_str))
            .collect();
        let target = |t: &str, line: usize| -> Result<(), String> {
            if nodes.contains(t) {
                Ok(())
            } else {
                Err(format!(
                    "line {line}: '{t}' names no node of '{}'",
                    self.name
                ))
            }
        };
        let text = |t: &Text, line: usize| -> Result<(), String> {
            for (_, x) in &t.args {
                check_expr(x, line, &vars, Some(&nodes))?;
            }
            Ok(())
        };
        let steps = |ss: &[Step], lines: &[usize], at: usize| -> Result<(), String> {
            for (i, s) in ss.iter().enumerate() {
                let line = lines.get(i).copied().unwrap_or(at);
                match s {
                    Step::Say(l) => {
                        if l.speaker != NARRATOR && self.speaker(&l.speaker).is_none() {
                            return Err(format!(
                                "line {}: '{}' is not a speaker: declare 'speaker {} {{ name \"...\" }}' or use narrator",
                                l.line, l.speaker, l.speaker
                            ));
                        }
                        text(&l.text, l.line)?;
                    }
                    Step::Set(v, x) => {
                        if !vars.contains(v.as_str()) {
                            return Err(format!(
                                "line {line}: 'set {v}': add <<declare ${v} = value>>"
                            ));
                        }
                        check_expr(x, line, &vars, Some(&nodes))?;
                    }
                    Step::If(x, t) => {
                        check_expr(x, line, &vars, Some(&nodes))?;
                        target(t, line)?;
                    }
                    Step::Goto(t) => target(t, line)?,
                    Step::Detour(t, next) => {
                        target(t, line)?;
                        target(next, line)?;
                    }
                    Step::Jump(x) => {
                        check_expr(x, line, &vars, Some(&nodes))?;
                        if let Expr::Lit(Val::Str(name)) = x {
                            target(name, line)?;
                        }
                    }
                    Step::DynamicDetour(x, next) => {
                        check_expr(x, line, &vars, Some(&nodes))?;
                        target(next, line)?;
                    }
                    Step::Return => {}
                    Step::Select(options) => {
                        for (t, cond, _) in options {
                            target(t, line)?;
                            if let Some(x) = cond {
                                check_expr(x, line, &vars, Some(&nodes))?;
                            }
                        }
                    }
                    Step::Command(x) | Step::Call(x) | Step::Emit(_, Some(x)) => {
                        check_expr(x, line, &vars, Some(&nodes))?
                    }
                    Step::Quest(QuestOp::Progress(_, _, x)) => {
                        check_expr(x, line, &vars, Some(&nodes))?
                    }
                    Step::MarkOnce(_) | Step::Emit(_, None) | Step::End | Step::Quest(_) => {}
                }
            }
            Ok(())
        };
        for n in &self.nodes {
            steps(&n.steps, &n.lines, n.line)?;
            if let Some(fallback) = &n.choice_fallback {
                target(fallback, n.line)?;
            }
            for c in &n.choices {
                text(&c.text, c.line)?;
                if let Some(r) = &c.reason {
                    text(r, c.line)?;
                }
                if let Some(x) = &c.cond {
                    check_expr(x, c.line, &vars, Some(&nodes))?;
                }
                if let Some(k) = &c.check {
                    if !vars.contains(k.stat.as_str()) {
                        return Err(format!(
                            "line {}: the check's stat '{}' is not declared (var or global)",
                            c.line, k.stat
                        ));
                    }
                    check_expr(&k.target, c.line, &vars, Some(&nodes))?;
                }
                steps(&c.steps, &[], c.line)?;
                for t in c.goto.iter().chain(&c.fail) {
                    target(t, c.line)?;
                }
            }
        }
        for b in &self.barks {
            if !b.speaker.is_empty() && b.speaker != NARRATOR && self.speaker(&b.speaker).is_none()
            {
                return Err(format!(
                    "line {}: the bark '{}' names the speaker '{}', which is not declared",
                    b.line, b.id, b.speaker
                ));
            }
            if let Some(x) = &b.when {
                check_reads(x, b.line, &vars, Some(&nodes), true)?;
            }
            for l in &b.lines {
                for (_, x) in &l.text.args {
                    check_reads(x, l.line, &vars, Some(&nodes), true)?;
                }
            }
            steps(&b.steps, &[], b.line)?;
        }
        Ok(())
    }

    /// Every i18n key it shows, with the argument names it passes and the
    /// line: what the cook checks against the base string table.
    pub fn keys(&self) -> Vec<(String, Vec<String>, usize)> {
        let mut out = Vec::new();
        let mut put = |t: &Text, line: usize| {
            out.push((
                t.key.clone(),
                t.args.iter().map(|a| a.0.clone()).collect(),
                line,
            ))
        };
        for s in &self.speakers {
            if !s.name.is_empty() {
                put(
                    &Text {
                        key: s.name.clone(),
                        args: Vec::new(),
                    },
                    0,
                );
            }
        }
        for b in &self.barks {
            for l in &b.lines {
                put(&l.text, l.line);
            }
        }
        for n in &self.nodes {
            for s in &n.steps {
                if let Step::Say(l) = s {
                    put(&l.text, l.line);
                }
            }
            for c in &n.choices {
                put(&c.text, c.line);
                if let Some(r) = &c.reason {
                    put(r, c.line);
                }
            }
        }
        out
    }
}

impl Quest {
    /// Parses and validates a quest.
    pub fn parse(text: &str) -> Result<Quest, String> {
        let doc = header(text, QUEST_TYPE)?;
        let mut q = Quest {
            id: String::new(),
            title: String::new(),
            summary: String::new(),
            category: "main".into(),
            giver: String::new(),
            region: String::new(),
            vars: Vec::new(),
            rewards: Vec::new(),
            stages: Vec::new(),
        };
        for e in &doc.body.entries {
            match e.key.as_str() {
                "id" => q.id = name_at(e, 0, "an id")?,
                "title" => q.title = key_at(e, 0)?,
                "summary" => q.summary = key_at(e, 0)?,
                "category" => q.category = name_at(e, 0, "a word, such as main or side")?,
                "giver" => q.giver = key_at(e, 0)?,
                "region" => q.region = key_at(e, 0)?,
                "global" => q.vars.push(var_decl(e, true)?),
                "var" => {
                    return Err(err(
                        e,
                        "a quest's variables are the story's: write 'global <name> <value>'",
                    ));
                }
                "reward" => {
                    let name = key_at(e, 0)?;
                    let amount = e
                        .values
                        .get(1)
                        .and_then(|v| v.as_number())
                        .map_or(1.0, |n| n.as_f64());
                    let mut r = Reward {
                        name,
                        amount,
                        image: String::new(),
                        rarity: String::new(),
                    };
                    for x in e
                        .block
                        .as_ref()
                        .map(|b| b.entries.as_slice())
                        .unwrap_or(&[])
                    {
                        match x.key.as_str() {
                            "image" => r.image = name_at(x, 0, "an image path")?,
                            "rarity" => r.rarity = name_at(x, 0, "a word")?,
                            other => {
                                return Err(err(
                                    x,
                                    format!("'{other}' is not part of a reward: image or rarity"),
                                ));
                            }
                        }
                    }
                    q.rewards.push(r);
                }
                "stage" => {
                    let mut s = Stage {
                        id: name_at(e, 0, "an id")?,
                        summary: String::new(),
                        objectives: Vec::new(),
                        next: None,
                        fail: None,
                        line: e.line,
                    };
                    if q.stage(&s.id).is_some() {
                        return Err(err(e, format!("the stage '{}' is declared twice", s.id)));
                    }
                    for x in e
                        .block
                        .as_ref()
                        .map(|b| b.entries.as_slice())
                        .unwrap_or(&[])
                    {
                        match x.key.as_str() {
                            "summary" => s.summary = key_at(x, 0)?,
                            "next" => s.next = Some(name_at(x, 0, "a stage")?),
                            "fail" => s.fail = Some(expr_at(x, 0)?),
                            "objective" => {
                                let id = name_at(x, 0, "an id")?;
                                let mut o = Objective {
                                    id,
                                    text: text_of(x, 1)?,
                                    target: 1,
                                    when: None,
                                    optional: false,
                                    hidden: false,
                                    marker: None,
                                    on: None,
                                    filter: None,
                                    line: x.line,
                                };
                                for y in x
                                    .block
                                    .as_ref()
                                    .map(|b| b.entries.as_slice())
                                    .unwrap_or(&[])
                                {
                                    match y.key.as_str() {
                                        "arg" => {}
                                        "target" => {
                                            o.target = y
                                                .values
                                                .first()
                                                .and_then(|v| v.as_number())
                                                .and_then(|n| n.as_u64().ok())
                                                .filter(|t| (1..=1_000_000).contains(t))
                                                .ok_or_else(|| {
                                                    err(y, "'target' is a count, 1 to 1000000")
                                                })?
                                                as u32
                                        }
                                        "when" => o.when = Some(expr_at(y, 0)?),
                                        "on" => {
                                            o.on = Some(name_at(
                                                y,
                                                0,
                                                "an event's name, such as og.CollisionStarted or EnemyDied",
                                            )?)
                                        }
                                        "where" => o.filter = Some(expr_at(y, 0)?),
                                        "optional" => o.optional = true,
                                        "hidden" => o.hidden = true,
                                        "marker" => {
                                            let n = |i: usize| {
                                                y.values
                                                    .get(i)
                                                    .and_then(|v| v.as_number())
                                                    .map(|n| n.as_f32())
                                            };
                                            o.marker = Some([
                                                n(0).ok_or_else(|| {
                                                    err(y, "'marker' is a point: x y z")
                                                })?,
                                                n(1).unwrap_or(0.0),
                                                n(2).unwrap_or(0.0),
                                            ]);
                                        }
                                        other => {
                                            return Err(err(
                                                y,
                                                format!(
                                                    "'{other}' is not part of an objective: target, when, on, where, optional, hidden, marker or arg"
                                                ),
                                            ));
                                        }
                                    }
                                }
                                if s.objectives.iter().any(|p| p.id == o.id) {
                                    return Err(err(
                                        x,
                                        format!("the objective '{}' is declared twice", o.id),
                                    ));
                                }
                                s.objectives.push(o);
                            }
                            other => {
                                return Err(err(
                                    x,
                                    format!(
                                        "'{other}' is not part of a stage: summary, objective, next or fail"
                                    ),
                                ));
                            }
                        }
                    }
                    q.stages.push(s);
                }
                other => {
                    return Err(err(
                        e,
                        format!(
                            "'{other}' is not part of a quest: id, title, summary, category, giver, region, global, reward or stage"
                        ),
                    ));
                }
            }
        }
        if q.id.is_empty() {
            return Err("a quest needs an 'id'".into());
        }
        if q.title.is_empty() {
            return Err("a quest needs a 'title' (an og.i18n key)".into());
        }
        if q.stages.is_empty() {
            return Err("a quest needs a 'stage'".into());
        }
        let vars: BTreeSet<&str> = q.vars.iter().map(|v| v.name.as_str()).collect();
        for s in &q.stages {
            if let Some(n) = &s.next
                && q.stage(n).is_none()
            {
                return Err(format!(
                    "line {}: 'next {n}' names no stage of '{}'",
                    s.line, q.id
                ));
            }
            if let Some(x) = &s.fail {
                check_expr(x, s.line, &vars, None)?;
            }
            for o in &s.objectives {
                if let Some(x) = &o.when {
                    check_expr(x, o.line, &vars, None)?;
                }
                if let Some(x) = &o.filter {
                    if o.on.is_none() {
                        return Err(format!(
                            "line {}: 'where' filters an 'on' event: add 'on <Event>'",
                            o.line
                        ));
                    }
                    check_reads(x, o.line, &vars, None, true)?;
                }
                for (_, x) in &o.text.args {
                    check_expr(x, o.line, &vars, None)?;
                }
            }
        }
        Ok(q)
    }

    /// Every i18n key it shows, with argument names and lines.
    pub fn keys(&self) -> Vec<(String, Vec<String>, usize)> {
        let mut out = Vec::new();
        for k in [&self.title, &self.summary, &self.giver, &self.region] {
            if !k.is_empty() {
                out.push((k.clone(), Vec::new(), 0));
            }
        }
        for r in &self.rewards {
            out.push((r.name.clone(), Vec::new(), 0));
        }
        for s in &self.stages {
            if !s.summary.is_empty() {
                out.push((s.summary.clone(), Vec::new(), s.line));
            }
            for o in &s.objectives {
                let mut args: Vec<String> = o.text.args.iter().map(|a| a.0.clone()).collect();
                // The log passes count and target to every objective's text.
                args.extend(["count".to_string(), "target".to_string()]);
                out.push((o.text.key.clone(), args, o.line));
            }
        }
        out
    }
}
