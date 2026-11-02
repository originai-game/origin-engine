//! The story assets: dialogues (`type dialogue`) and quests (`type quest`),
//! parsed and validated so a cooked one always runs. Every error names its
//! line and the fix. Text is never inline: lines, choices, reasons, names
//! and titles are og.i18n keys, resolved in the active locale when shown.
//!
//! ```text
//! type dialogue
//! version 1
//! ---
//! name smith
//! start greet
//! speaker smith {
//!   name "npc.smith"
//!   portrait "art/smith.png"
//!   side left
//! }
//! var mood 0                     # this conversation's own
//! global gold 10                 # the story's, shared with every dialogue and quest
//! node greet {
//!   say smith "smith.hello" {
//!     arg gold "gold"
//!     voice "vo/smith_hello"
//!     shot smith_close           # a virtual camera's name or a cast role
//!   }
//!   choice "smith.buy" {
//!     if "gold >= 10"
//!     reason "smith.need_gold" {
//!       arg need "10 - gold"
//!     }
//!     kind trade
//!     set gold "gold - 10"
//!     goto bought
//!   }
//!   choice "smith.haggle" {
//!     check active charm 12      # d20 + charm against 12
//!     goto bought
//!     fail angry
//!   }
//!   choice "smith.leave"
//!   timer 600 smith.leave        # ticks; the choice taken when it runs out
//! }
//! node bought {
//!   shot counter                 # every line of the node, unless one names its own
//!   say smith "smith.thanks"
//!   quest progress rescue sword
//!   emit give_sword
//! }
//! node angry {
//!   pick {
//!     greet 1
//!     bought 3
//!   }
//! }
//! ```
//!
//! A node runs its steps in order: `say` shows a line and waits for
//! `advance`; `set`, `emit` and `quest` act at once; `if "cond" node`,
//! `goto`, `pick` (weighted, on the story's og.Rng stream) and `end` jump.
//! A node's choices are shown with its last line: when that line is
//! reached, the steps after it run at once and the line waits for a
//! choice. A node that runs out of steps without choices ends the
//! conversation. A choice without `goto` ends it too.
//!
//! `shot` names a virtual camera (its entity's name, or a cast role that
//! the conversation's cast maps to one): a node's covers its lines, a
//! line's replaces it for that line, and `shot none` shows a line without
//! one. The runtime makes it og.CameraBrain's solo shot while the line
//! shows and hands the solo back when the line, the node or the
//! conversation ends.
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
use og_core::ogt::{Block, Document, Entry, Value as OgtValue};
use std::collections::BTreeSet;

/// The `.ogt` type of a dialogue.
pub const DIALOGUE_TYPE: &str = "dialogue";
/// The `.ogt` type of a quest.
pub const QUEST_TYPE: &str = "quest";
/// The speaker every dialogue has without declaring it: no name, no portrait.
pub const NARRATOR: &str = "narrator";

/// A variable a dialogue or quest declares.
#[derive(Clone, Debug, PartialEq)]
pub struct VarDecl {
    pub name: String,
    pub value: Val,
    /// `global`: the story's, shared by every dialogue and quest; else the
    /// conversation's own.
    pub global: bool,
}

/// A speaker.
#[derive(Clone, Debug, Default, PartialEq)]
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
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Text {
    pub key: String,
    pub args: Vec<(String, Expr)>,
}

/// A line.
#[derive(Clone, Debug, PartialEq)]
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
#[derive(Clone, Debug, PartialEq)]
pub enum QuestOp {
    Start(String),
    Progress(String, String, Expr),
    Stage(String, String),
    Complete(String),
    Fail(String),
}

/// A step of a node.
#[derive(Clone, Debug, PartialEq)]
pub enum Step {
    Say(Line),
    Set(String, Expr),
    /// `if "cond" node`.
    If(Expr, String),
    Goto(String),
    End,
    /// `pick { node weight ... }`.
    Pick(Vec<(String, f64)>),
    /// `emit name ["arg"]`.
    Emit(String, Option<Expr>),
    Quest(QuestOp),
}

impl Step {
    fn jumps(&self) -> bool {
        matches!(
            self,
            Step::If(..) | Step::Goto(_) | Step::End | Step::Pick(_)
        )
    }
}

/// A check on a choice: `passive` enables it only when the stat reaches
/// the target; `active` rolls d20 + the stat against the target when it
/// is chosen, going to the choice's `goto` on a pass and `fail` on a miss.
#[derive(Clone, Debug, PartialEq)]
pub struct Check {
    pub active: bool,
    /// The variable read as the stat.
    pub stat: String,
    pub target: Expr,
}

/// A choice.
#[derive(Clone, Debug, PartialEq)]
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
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub id: String,
    pub steps: Vec<Step>,
    /// Each step's source line.
    pub lines: Vec<usize>,
    pub choices: Vec<Choice>,
    /// `timer ticks choice`: the choice taken when the time runs out.
    pub timer: Option<(u32, String)>,
    /// The step of the line its choices are shown with (its last `say`);
    /// `None` without one (they show with the line before).
    pub choice_line: Option<usize>,
    /// `shot <camera>`: the shot its lines are shown in unless a line
    /// names its own (empty: none).
    pub shot: String,
    pub line: usize,
}

/// A parsed dialogue.
#[derive(Clone, Debug, PartialEq)]
pub struct Dialogue {
    pub name: String,
    pub start: String,
    pub speakers: Vec<Speaker>,
    pub vars: Vec<VarDecl>,
    pub nodes: Vec<Node>,
    pub barks: Vec<Bark>,
}

/// A bark: an ambient line an entity says on its own (a guard's yawn, a
/// shopkeeper's call), not a conversation. og.story.bark asks for one of a
/// topic; og.story.ambient asks every so often. The highest `priority` of
/// the topic whose `when` holds and whose cooldowns are over is said; ties
/// are drawn by `weight` on the story's stream.
#[derive(Clone, Debug, PartialEq)]
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

fn no_block(e: &Entry) -> Result<(), String> {
    if e.block.is_some() {
        return Err(err(e, format!("'{}' takes no block", e.key)));
    }
    Ok(())
}

fn quest_op(e: &Entry) -> Result<QuestOp, String> {
    let op = name_at(
        e,
        0,
        "an operation: start, progress, stage, complete or fail",
    )?;
    let id = name_at(e, 1, "a quest id")?;
    Ok(match op.as_str() {
        "start" => QuestOp::Start(id),
        "complete" => QuestOp::Complete(id),
        "fail" => QuestOp::Fail(id),
        "stage" => QuestOp::Stage(id, name_at(e, 2, "a stage id")?),
        "progress" => QuestOp::Progress(
            id,
            name_at(e, 2, "an objective id")?,
            if e.values.len() > 3 {
                expr_at(e, 3)?
            } else {
                Expr::Lit(Val::Num(1.0))
            },
        ),
        other => {
            return Err(err(
                e,
                format!(
                    "'quest {other}' is not a quest operation: start, progress, stage, complete or fail"
                ),
            ));
        }
    })
}

/// A step every block may hold (`set`, `emit`, `quest`), or `None`.
fn action(e: &Entry) -> Result<Option<Step>, String> {
    Ok(Some(match e.key.as_str() {
        "set" => Step::Set(name_at(e, 0, "a variable")?, expr_at(e, 1)?),
        "emit" => Step::Emit(
            name_at(e, 0, "an event name")?,
            if e.values.len() > 1 {
                Some(expr_at(e, 1)?)
            } else {
                None
            },
        ),
        "quest" => Step::Quest(quest_op(e)?),
        _ => return Ok(None),
    }))
}

fn choice(e: &Entry) -> Result<Choice, String> {
    let text = text_of(e, 0)?;
    let mut c = Choice {
        id: text.key.clone(),
        text,
        cond: None,
        hide: false,
        reason: None,
        kind: String::new(),
        check: None,
        steps: Vec::new(),
        goto: None,
        fail: None,
        line: e.line,
    };
    let Some(b) = &e.block else { return Ok(c) };
    for x in &b.entries {
        if let Some(step) = action(x)? {
            c.steps.push(step);
            continue;
        }
        match x.key.as_str() {
            "arg" => {}
            "id" => c.id = name_at(x, 0, "a word")?,
            "if" => c.cond = Some(expr_at(x, 0)?),
            "hide" => c.hide = true,
            "reason" => c.reason = Some(text_of(x, 0)?),
            "kind" => c.kind = name_at(x, 0, "a word, such as trade or leave")?,
            "goto" => c.goto = Some(name_at(x, 0, "a node")?),
            "fail" => c.fail = Some(name_at(x, 0, "a node")?),
            "check" => {
                let active = match name_at(x, 0, "passive or active")?.as_str() {
                    "active" => true,
                    "passive" => false,
                    other => {
                        return Err(err(
                            x,
                            format!(
                                "a check is passive (a threshold) or active (a d20 roll), not '{other}'"
                            ),
                        ));
                    }
                };
                c.check = Some(Check {
                    active,
                    stat: name_at(x, 1, "a variable to read as the stat")?,
                    target: expr_at(x, 2)?,
                });
            }
            other => {
                return Err(err(
                    x,
                    format!(
                        "'{other}' is not part of a choice: goto, if, hide, reason, kind, id, arg, check, fail, set, emit or quest"
                    ),
                ));
            }
        }
    }
    match (&c.check, &c.fail) {
        (Some(k), None) if k.active => {
            return Err(err(
                e,
                "an active check needs 'fail <node>', where a missed roll goes",
            ));
        }
        (Some(k), Some(_)) if !k.active => {
            return Err(err(
                e,
                "'fail' is for an active check: a passive check only disables the choice",
            ));
        }
        (None, Some(_)) => {
            return Err(err(e, "'fail' needs 'check active <stat> <target>'"));
        }
        _ => {}
    }
    Ok(c)
}

/// A line: the i18n key at value `i`, with its block's `arg`, `voice`,
/// `expression` and `shot`.
fn line_at(x: &Entry, speaker: String, i: usize) -> Result<Line, String> {
    let text = text_of(x, i)?;
    let opt = |k: &str| -> Result<String, String> {
        match x.block.as_ref().and_then(|b| b.get(k)) {
            Some(v) => name_at(v, 0, "a value"),
            None => Ok(String::new()),
        }
    };
    if let Some(b) = &x.block
        && let Some(bad) = b
            .entries
            .iter()
            .find(|v| !matches!(v.key.as_str(), "arg" | "voice" | "expression" | "shot"))
    {
        return Err(err(
            bad,
            format!(
                "'{}' is not part of a line: arg, voice, expression or shot",
                bad.key
            ),
        ));
    }
    Ok(Line {
        speaker,
        text,
        voice: opt("voice")?,
        expression: opt("expression")?,
        shot: opt("shot")?,
        line: x.line,
    })
}

fn ticks_of(x: &Entry, max: u64) -> Result<u32, String> {
    x.values
        .first()
        .and_then(|v| v.as_number())
        .and_then(|v| v.as_u64().ok())
        .filter(|t| *t <= max)
        .map(|t| t as u32)
        .ok_or_else(|| err(x, format!("'{}' takes ticks, 0 to {max}", x.key)))
}

fn bark(e: &Entry) -> Result<Bark, String> {
    let mut b = Bark {
        id: name_at(e, 0, "an id")?,
        topic: String::new(),
        speaker: String::new(),
        lines: Vec::new(),
        when: None,
        priority: 0,
        weight: 1.0,
        cooldown: 600,
        shared: 0,
        duration: 180,
        random: false,
        once: false,
        steps: Vec::new(),
        line: e.line,
    };
    for x in e
        .block
        .as_ref()
        .map(|b| b.entries.as_slice())
        .unwrap_or(&[])
    {
        if let Some(step) = action(x)? {
            no_block(x)?;
            b.steps.push(step);
            continue;
        }
        match x.key.as_str() {
            "topic" => b.topic = name_at(x, 0, "a word, such as idle or alarm")?,
            "speaker" => b.speaker = name_at(x, 0, "a speaker")?,
            "say" => b.lines.push(line_at(x, String::new(), 0)?),
            "when" => b.when = Some(expr_at(x, 0)?),
            "priority" => {
                b.priority = x
                    .values
                    .first()
                    .and_then(|v| v.as_number())
                    .and_then(|n| n.as_i64().ok())
                    .filter(|p| (-1000..=1000).contains(p))
                    .ok_or_else(|| err(x, "'priority' is a whole number, -1000 to 1000"))?
                    as i32
            }
            "weight" => {
                b.weight = x
                    .values
                    .first()
                    .and_then(|v| v.as_number())
                    .map(|n| n.as_f64())
                    .filter(|w| *w > 0.0 && w.is_finite())
                    .ok_or_else(|| err(x, "'weight' is a number above 0"))?
            }
            "cooldown" => b.cooldown = ticks_of(x, 1 << 30)?,
            "shared" => b.shared = ticks_of(x, 1 << 30)?,
            "duration" => {
                b.duration = ticks_of(x, 1 << 30)?;
                if b.duration == 0 {
                    return Err(err(x, "'duration' is 1 tick or more"));
                }
            }
            "order" => {
                b.random = match name_at(x, 0, "cycle or random")?.as_str() {
                    "cycle" => false,
                    "random" => true,
                    other => {
                        return Err(err(
                            x,
                            format!("a bark's order is cycle or random, not '{other}'"),
                        ));
                    }
                }
            }
            "once" => b.once = true,
            other => {
                return Err(err(
                    x,
                    format!(
                        "'{other}' is not part of a bark: topic, speaker, say, when, priority, weight, cooldown, shared, duration, order, once, set, emit or quest"
                    ),
                ));
            }
        }
    }
    if b.topic.is_empty() {
        return Err(err(e, format!("the bark '{}' needs a 'topic'", b.id)));
    }
    if b.lines.is_empty() {
        return Err(err(
            e,
            format!("the bark '{}' needs a line: say \"<key>\"", b.id),
        ));
    }
    Ok(b)
}

fn node(e: &Entry) -> Result<Node, String> {
    let id = name_at(e, 0, "an id")?;
    let mut n = Node {
        id,
        steps: Vec::new(),
        lines: Vec::new(),
        choices: Vec::new(),
        timer: None,
        choice_line: None,
        shot: String::new(),
        line: e.line,
    };
    let empty = Block::new();
    for x in &e.block.as_ref().unwrap_or(&empty).entries {
        if let Some(step) = action(x)? {
            no_block(x)?;
            n.steps.push(step);
            n.lines.push(x.line);
            continue;
        }
        let step = match x.key.as_str() {
            "say" => Step::Say(line_at(x, name_at(x, 0, "a speaker")?, 1)?),
            "if" => Step::If(expr_at(x, 0)?, name_at(x, 1, "a node to go to")?),
            "goto" => Step::Goto(name_at(x, 0, "a node")?),
            "end" => Step::End,
            "pick" => {
                let mut opts = Vec::new();
                for o in x
                    .block
                    .as_ref()
                    .map(|b| b.entries.as_slice())
                    .unwrap_or(&[])
                {
                    let w = o
                        .values
                        .first()
                        .and_then(|v| v.as_number())
                        .map(|n| n.as_f64());
                    match w {
                        Some(w) if w > 0.0 && w.is_finite() => opts.push((o.key.clone(), w)),
                        _ => {
                            return Err(err(
                                o,
                                format!(
                                    "'{}' needs a weight above 0, such as '{} 2'",
                                    o.key, o.key
                                ),
                            ));
                        }
                    }
                }
                if opts.is_empty() {
                    return Err(err(
                        x,
                        "'pick' needs nodes with weights: pick { win 1  lose 3 }",
                    ));
                }
                Step::Pick(opts)
            }
            "choice" => {
                n.choices.push(choice(x)?);
                continue;
            }
            "shot" => {
                no_block(x)?;
                n.shot = name_at(
                    x,
                    0,
                    "a camera: a virtual camera's entity name or a cast role",
                )?;
                continue;
            }
            "timer" => {
                let ticks = x
                    .values
                    .first()
                    .and_then(|v| v.as_number())
                    .and_then(|v| v.as_u64().ok())
                    .filter(|t| (1..=u64::from(u32::MAX)).contains(t))
                    .ok_or_else(|| err(x, "'timer' needs ticks (1 or more) and a choice id"))?;
                n.timer = Some((
                    ticks as u32,
                    name_at(x, 1, "the choice taken when it runs out")?,
                ));
                continue;
            }
            other => {
                return Err(err(
                    x,
                    format!(
                        "'{other}' is not a step: say, set, if, goto, end, pick, emit, quest, choice, timer or shot"
                    ),
                ));
            }
        };
        if !matches!(step, Step::Say(_) | Step::Pick(_)) {
            no_block(x)?;
        }
        n.steps.push(step);
        n.lines.push(x.line);
    }
    n.choice_line = n.steps.iter().rposition(|s| matches!(s, Step::Say(_)));
    if !n.choices.is_empty() {
        let after = n.choice_line.map_or(0, |i| i + 1);
        if let Some(j) = n.steps[after..].iter().find(|s| s.jumps()) {
            let _ = j;
            return Err(format!(
                "line {}: node '{}' has choices and a jump after its last line: choices end a node, so move the jump into a choice or another node",
                e.line, n.id
            ));
        }
    }
    let mut ids = BTreeSet::new();
    for c in &n.choices {
        if !ids.insert(c.id.as_str()) {
            return Err(format!(
                "line {}: node '{}' has two choices with the id '{}': give one an 'id'",
                c.line, n.id, c.id
            ));
        }
    }
    if let Some((_, t)) = &n.timer
        && !ids.contains(t.as_str())
    {
        return Err(format!(
            "line {}: node '{}' has a timer for the choice '{t}', which it does not have (choices: {})",
            e.line,
            n.id,
            ids.into_iter().collect::<Vec<_>>().join(", ")
        ));
    }
    Ok(n)
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
            "line {line}: '{v}' is not declared: add 'var {v} <value>' (this conversation's) or 'global {v} <value>' (the story's)"
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
        let doc = header(text, DIALOGUE_TYPE)?;
        let mut d = Dialogue {
            name: String::new(),
            start: String::new(),
            speakers: Vec::new(),
            vars: Vec::new(),
            nodes: Vec::new(),
            barks: Vec::new(),
        };
        let mut start_line = 0;
        for e in &doc.body.entries {
            match e.key.as_str() {
                "name" => d.name = name_at(e, 0, "a name")?,
                "start" => {
                    d.start = name_at(e, 0, "the first node")?;
                    start_line = e.line;
                }
                "speaker" => {
                    let id = name_at(e, 0, "an id")?;
                    if id == NARRATOR || d.speaker(&id).is_some() {
                        return Err(err(e, format!("the speaker '{id}' is already declared")));
                    }
                    let mut s = Speaker {
                        id,
                        side: "left".into(),
                        ..Speaker::default()
                    };
                    for x in e
                        .block
                        .as_ref()
                        .map(|b| b.entries.as_slice())
                        .unwrap_or(&[])
                    {
                        match x.key.as_str() {
                            "name" => s.name = key_at(x, 0)?,
                            "portrait" => s.portrait = name_at(x, 0, "an image path")?,
                            "expression" => s.expression = name_at(x, 0, "a word")?,
                            "side" => {
                                s.side = name_at(x, 0, "left, right or center")?;
                                if !matches!(s.side.as_str(), "left" | "right" | "center") {
                                    return Err(err(x, "a side is left, right or center"));
                                }
                            }
                            other => {
                                return Err(err(
                                    x,
                                    format!(
                                        "'{other}' is not part of a speaker: name, portrait, expression or side"
                                    ),
                                ));
                            }
                        }
                    }
                    d.speakers.push(s);
                }
                "var" | "global" => {
                    let v = var_decl(e, e.key == "global")?;
                    if d.vars.iter().any(|o| o.name == v.name) {
                        return Err(err(e, format!("'{}' is declared twice", v.name)));
                    }
                    d.vars.push(v);
                }
                "bark" => {
                    let b = bark(e)?;
                    if d.bark(&b.id).is_some() {
                        return Err(err(e, format!("the bark '{}' is declared twice", b.id)));
                    }
                    d.barks.push(b);
                }
                "node" => {
                    let n = node(e)?;
                    if d.node(&n.id).is_some() {
                        return Err(err(e, format!("the node '{}' is declared twice", n.id)));
                    }
                    d.nodes.push(n);
                }
                other => {
                    return Err(err(
                        e,
                        format!(
                            "'{other}' is not part of a dialogue: name, start, speaker, var, global, node or bark"
                        ),
                    ));
                }
            }
        }
        if d.name.is_empty() {
            return Err("a dialogue needs a 'name'".into());
        }
        if d.nodes.is_empty() && d.barks.is_empty() {
            return Err("a dialogue needs a 'node' or a 'bark'".into());
        }
        if d.nodes.is_empty() {
            if !d.start.is_empty() {
                return Err(format!(
                    "line {start_line}: 'start {}' names no node",
                    d.start
                ));
            }
        } else if d.start.is_empty() {
            d.start = d.nodes[0].id.clone();
        } else if d.node(&d.start).is_none() {
            return Err(format!(
                "line {start_line}: 'start {}' names no node",
                d.start
            ));
        }
        d.validate()?;
        Ok(d)
    }

    fn validate(&self) -> Result<(), String> {
        let nodes: BTreeSet<&str> = self.nodes.iter().map(|n| n.id.as_str()).collect();
        let vars: BTreeSet<&str> = self.vars.iter().map(|v| v.name.as_str()).collect();
        let target = |t: &str, line: usize| -> Result<(), String> {
            if t == "end" || nodes.contains(t) {
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
                                "line {line}: 'set {v}': '{v}' is not declared (var or global)"
                            ));
                        }
                        check_expr(x, line, &vars, Some(&nodes))?;
                    }
                    Step::If(x, t) => {
                        check_expr(x, line, &vars, Some(&nodes))?;
                        target(t, line)?;
                    }
                    Step::Goto(t) => target(t, line)?,
                    Step::Pick(o) => {
                        for (t, _) in o {
                            target(t, line)?;
                        }
                    }
                    Step::Emit(_, Some(x)) => check_expr(x, line, &vars, Some(&nodes))?,
                    Step::Quest(QuestOp::Progress(_, _, x)) => {
                        check_expr(x, line, &vars, Some(&nodes))?
                    }
                    Step::Emit(_, None) | Step::End | Step::Quest(_) => {}
                }
            }
            Ok(())
        };
        for n in &self.nodes {
            steps(&n.steps, &n.lines, n.line)?;
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
