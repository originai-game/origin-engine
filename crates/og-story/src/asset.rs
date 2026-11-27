//! Engine-owned native story data from checked TS declarations or explicit
//! Yarn imports. Lines have stable identities independent of localization
//! keys; continuations use semantic IDs, never source positions.
//!
//! A stage is done when every objective that is not `optional` is; the
//! quest moves to `next`, or completes after its last stage. `fail`
//! fails the quest when its condition holds; `when` completes an
//! objective when its condition holds; `hidden` keeps an objective out of
//! the log until it first counts.

use crate::expr::{Expr, Func, Val};
use std::collections::BTreeSet;

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
    /// Stable identity, separate from the localization key.
    pub id: String,
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
    /// Its hashtags (`#hotspot:knife` is `hotspot` → `knife`, `#default`
    /// `default` → empty), for the game or a module that reads them.
    #[serde(default)]
    pub tags: std::collections::BTreeMap<String, String>,
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
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Stage {
    pub id: String,
    pub summary: String,
    pub objectives: Vec<Objective>,
    pub next: Option<String>,
    pub fail: Option<Expr>,
    pub line: usize,
}

/// A reward shown in the quest log.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Reward {
    /// Its name's i18n key.
    pub name: String,
    pub amount: f64,
    pub image: String,
    pub rarity: String,
}

/// A parsed quest.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
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

/// Checked engine story data, shared by typed declarations and the Yarn importer.
/// The enclosing cooked blob versions this native representation.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Asset {
    Dialogue(Dialogue),
    Quest(Quest),
}

impl Asset {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Dialogue(d) => {
                crate::yarn::functions::validate(&mut d.functions.clone())?;
                d.validate()
            }
            Self::Quest(q) => q.validate(),
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        use bincode::Options;
        self.validate()?;
        bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .serialize(self)
            .map_err(|e| format!("native story: {e}"))
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        use bincode::Options;
        let asset: Self = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .with_limit(bytes.len() as u64)
            .reject_trailing_bytes()
            .deserialize(bytes)
            .map_err(|e| format!("native story: {e}"))?;
        asset.validate()?;
        Ok(asset)
    }

    /// Resolve only resource-bearing fields, never story names or text keys.
    pub fn map_resources(
        &mut self,
        mut map: impl FnMut(&str, &mut String) -> Result<(), String>,
    ) -> Result<(), String> {
        fn voice(
            line: &mut Line,
            map: &mut dyn FnMut(&str, &mut String) -> Result<(), String>,
        ) -> Result<(), String> {
            if !line.voice.is_empty() {
                map("sound", &mut line.voice)?;
            }
            Ok(())
        }
        match self {
            Self::Dialogue(d) => {
                for speaker in &mut d.speakers {
                    if !speaker.portrait.is_empty() {
                        map("texture", &mut speaker.portrait)?;
                    }
                }
                for header in d.headers.values_mut() {
                    if let Some(segment) = header.get_mut(crate::graph::FLOW_HEADER) {
                        map("timeline", segment)?;
                    }
                    if let Some(music) = header.get_mut("og-flow-music") {
                        let mut parts = music.split_whitespace();
                        if let Some(sound) = parts.next().filter(|s| *s != "none") {
                            let mut sound = sound.to_string();
                            map("sound", &mut sound)?;
                            *music = std::iter::once(sound)
                                .chain(parts.map(str::to_string))
                                .collect::<Vec<_>>()
                                .join(" ");
                        }
                    }
                }
                for node in &mut d.nodes {
                    for step in node
                        .steps
                        .iter_mut()
                        .chain(node.choices.iter_mut().flat_map(|c| &mut c.steps))
                    {
                        if let Step::Say(line) = step {
                            voice(line, &mut map)?;
                        }
                    }
                }
                for bark in &mut d.barks {
                    for line in &mut bark.lines {
                        voice(line, &mut map)?;
                    }
                    for step in &mut bark.steps {
                        if let Step::Say(line) = step {
                            voice(line, &mut map)?;
                        }
                    }
                }
            }
            Self::Quest(q) => {
                for reward in &mut q.rewards {
                    if !reward.image.is_empty() {
                        map("texture", &mut reward.image)?;
                    }
                }
            }
        }
        Ok(())
    }
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
            "line {line}: '{v}' is not declared: add it to dialogue vars or quest globals; in imported Yarn use <<declare ${v} = value>>"
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
    /// Resolve and type-check a structured author declaration before packing.
    pub fn check_native(&mut self) -> Result<(), String> {
        self.validate()?;
        crate::yarn::typing::check_native(self)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() || self.nodes.is_empty() || self.node(&self.start).is_none() {
            return Err(
                "a dialogue needs a name, nodes and a start naming one of its nodes".into(),
            );
        }
        identities("node", self.nodes.iter().map(|n| n.id.as_str()))?;
        identities("speaker", self.speakers.iter().map(|s| s.id.as_str()))?;
        identities("bark", self.barks.iter().map(|b| b.id.as_str()))?;
        identities(
            "variable",
            self.vars
                .iter()
                .map(|v| v.name.as_str())
                .chain(self.smart.keys().map(String::as_str)),
        )?;
        crate::yarn::typing::validate_smart(self)?;
        let nodes: BTreeSet<&str> = self.nodes.iter().map(|n| n.id.as_str()).collect();
        let vars: BTreeSet<&str> = self
            .vars
            .iter()
            .map(|v| v.name.as_str())
            .chain(self.smart.keys().map(String::as_str))
            .collect();
        for expression in self.smart.values() {
            check_expr(expression, 0, &vars, Some(&nodes))?;
        }
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
                                "line {}: '{}' is not a speaker: add it to speakers or use narrator",
                                l.line, l.speaker
                            ));
                        }
                        text(&l.text, l.line)?;
                    }
                    Step::Set(v, x) => {
                        if !vars.contains(v.as_str()) {
                            return Err(format!(
                                "line {line}: 'set {v}': add it to vars; in imported Yarn use <<declare ${v} = value>>"
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
            identities("choice", n.choices.iter().map(|c| c.id.as_str()))?;
            identities(
                "line",
                n.steps.iter().filter_map(|s| match s {
                    Step::Say(l) => Some(l.id.as_str()),
                    _ => None,
                }),
            )?;
            if let Some((ticks, id)) = &n.timer
                && (*ticks == 0 || !n.choices.iter().any(|c| &c.id == id))
            {
                return Err(format!(
                    "node '{}': a timer needs ticks and an existing choice id",
                    n.id
                ));
            }
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
            if b.lines.is_empty() || !b.weight.is_finite() || b.weight < 0.0 {
                return Err(format!(
                    "bark '{}': needs lines and a finite nonnegative weight",
                    b.id
                ));
            }
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
    /// Checks the native quest independently of its author representation.
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("a quest needs an 'id'".into());
        }
        if self.title.is_empty() {
            return Err("a quest needs a 'title' (an og.i18n key)".into());
        }
        if self.stages.is_empty() {
            return Err("a quest needs a 'stage'".into());
        }
        identities("stage", self.stages.iter().map(|s| s.id.as_str()))?;
        identities("variable", self.vars.iter().map(|v| v.name.as_str()))?;
        let vars: BTreeSet<&str> = self.vars.iter().map(|v| v.name.as_str()).collect();
        for s in &self.stages {
            identities("objective", s.objectives.iter().map(|o| o.id.as_str()))?;
            if let Some(n) = &s.next
                && self.stage(n).is_none()
            {
                return Err(format!(
                    "line {}: 'next {n}' names no stage of '{}'",
                    s.line, self.id
                ));
            }
            if let Some(x) = &s.fail {
                check_expr(x, s.line, &vars, None)?;
            }
            for o in &s.objectives {
                if !(1..=1_000_000).contains(&o.target) || o.text.key.is_empty() {
                    return Err(format!(
                        "objective '{}': needs a text key and target 1 to 1000000",
                        o.id
                    ));
                }
                if let Some(x) = &o.when {
                    check_expr(x, o.line, &vars, None)?;
                }
                if let Some(x) = &o.filter {
                    if o.on.is_none() {
                        return Err(format!(
                            "line {}: an objective's filter needs its on event name",
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
        crate::yarn::typing::check_quest(self)
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

fn identities<'a>(kind: &str, ids: impl Iterator<Item = &'a str>) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for id in ids {
        if id.is_empty() || !seen.insert(id) {
            return Err(format!(
                "{kind} identities must be nonempty and unique: '{id}'"
            ));
        }
    }
    Ok(())
}
