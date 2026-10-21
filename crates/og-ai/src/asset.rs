//! The behaviour asset: an `.ogt` of `type behaviour` holding a behaviour
//! tree (`tree { ... }`) or a state machine (`machine { ... }`), its
//! blackboard keys and its patrol routes. Parsing validates everything a
//! tick would trip over (unknown node kinds, operators, routes, states,
//! missing children), each error naming its line and the fix, so a cooked
//! behaviour always runs.
//!
//! ```text
//! type behaviour
//! version 1
//! ---
//! name "slime"
//! key alert 0
//! route beat {
//!   point 11 6.5 0
//!   point 16 6.5 0
//! }
//! tree {
//!   selector {
//!     sequence flee {
//!       check Slime.hp below 20
//!       flee
//!     }
//!     sequence chase {
//!       check sees
//!       chase
//!     }
//!     patrol beat
//!   }
//! }
//! ```

use og_core::glam::Vec3;
use og_core::ogt::{Block, Document, Entry, Value as OgtValue};

/// The `.ogt` type of a behaviour.
pub const BEHAVIOUR_TYPE: &str = "behaviour";

/// A blackboard value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Num(f64),
    Bool(bool),
    Vec(Vec3),
    Entity(u64),
}

impl Value {
    /// The value as a number: bools are 0 or 1, entities their handle,
    /// vectors their length.
    pub fn num(self) -> f64 {
        match self {
            Value::Num(n) => n,
            Value::Bool(b) => f64::from(u8::from(b)),
            Value::Vec(v) => v.length() as f64,
            Value::Entity(e) => e as f64,
        }
    }
    pub fn truthy(self) -> bool {
        match self {
            Value::Num(n) => n != 0.0,
            Value::Bool(b) => b,
            Value::Vec(v) => v != Vec3::ZERO,
            Value::Entity(e) => e != 0,
        }
    }
}

/// A comparison of a condition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Is,
    Not,
    Below,
    Above,
    AtMost,
    AtLeast,
    /// No operator: the operand is truthy.
    Truthy,
}

impl Op {
    fn parse(w: &str) -> Option<Op> {
        Some(match w {
            "is" => Op::Is,
            "not" => Op::Not,
            "below" => Op::Below,
            "above" => Op::Above,
            "atMost" => Op::AtMost,
            "atLeast" => Op::AtLeast,
            _ => return None,
        })
    }
    pub fn name(self) -> &'static str {
        match self {
            Op::Is => "is",
            Op::Not => "not",
            Op::Below => "below",
            Op::Above => "above",
            Op::AtMost => "atMost",
            Op::AtLeast => "atLeast",
            Op::Truthy => "",
        }
    }
    pub fn test(self, a: Value, b: Value) -> bool {
        match self {
            Op::Truthy => a.truthy(),
            Op::Is => a == b || (a.num() - b.num()).abs() < 1e-9,
            Op::Not => !(a == b || (a.num() - b.num()).abs() < 1e-9),
            Op::Below => a.num() < b.num(),
            Op::Above => a.num() > b.num(),
            Op::AtMost => a.num() <= b.num(),
            Op::AtLeast => a.num() >= b.num(),
        }
    }
}

/// A condition: an operand (a blackboard key, `Component.field` of the
/// agent, `Resource.field`, with an optional `.x`/`.y`/`.z` lane), an
/// operator and a value: a literal, or (`against`) a key or field read
/// each time the check runs, so a threshold can live in a tuning resource.
#[derive(Clone, Debug, PartialEq)]
pub struct Check {
    pub operand: String,
    pub op: Op,
    pub value: Value,
    /// The key, `Component.field` or `Resource.field` the operand is
    /// compared with instead of `value` (`check Foe.hp below Tuning.fleeHp`).
    pub against: Option<String>,
}

/// A speed: fixed, or read from a key or field each think (a tunable).
#[derive(Clone, Debug, PartialEq)]
pub enum Speed {
    Fixed(f32),
    Read(String),
}

/// A movement or game action.
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    /// To a blackboard key's point (or a fixed point); succeeds on arrival, fails when unreachable.
    MoveTo {
        key: Option<String>,
        point: Vec3,
        speed: Option<Speed>,
    },
    /// After a blackboard entity key's position (default `target`); succeeds within reach.
    Chase {
        key: String,
        speed: Option<Speed>,
        reach: f32,
    },
    /// Away from a key's position (default `target`) until `distance` away.
    Flee {
        key: String,
        speed: Option<Speed>,
        distance: f32,
    },
    /// Round a route's points, waiting at each; `loops` false succeeds at its end.
    Patrol {
        route: usize,
        wait: f32,
        speed: Option<Speed>,
        loops: bool,
    },
    /// To random points within `radius` of where the agent started.
    Wander {
        radius: f32,
        speed: Option<Speed>,
        wait: f32,
    },
    /// Running for `seconds`, then success.
    Wait { seconds: f32 },
    /// Stops moving; success.
    Stop,
    /// Sets a blackboard key; success.
    Set { key: String, value: Value },
    /// Runs the game's command `name` with the agent's handle and `args`
    /// at the start of the next tick; success. At most once per `cooldown`.
    Command {
        name: String,
        args: Vec<String>,
        cooldown: f32,
    },
}

/// One node of a tree.
#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    /// Children in order; the first that does not fail decides (re-checked every tick).
    Selector,
    /// Children in order while they succeed (re-checked every tick from the first).
    Sequence,
    /// A sequence that resumes at its running child (earlier children are not re-checked).
    Steps,
    /// Every child every tick: `all` succeeds when all do and fails when one does; `one` the reverse.
    Parallel {
        all: bool,
    },
    Invert,
    /// Success whatever the child answers (running stays running).
    Succeed,
    /// The child `times` times (0: forever) while it succeeds.
    Repeat {
        times: u32,
    },
    /// After the child succeeds, fails for `seconds`.
    Cooldown {
        seconds: f32,
    },
    /// Fails when the child runs longer than `seconds`.
    Timeout {
        seconds: f32,
    },
    Check(Check),
    Action(Action),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub kind: Kind,
    /// A label (`sequence chase`) or the node's kind and argument (`patrol beat`).
    pub name: String,
    pub children: Vec<usize>,
    /// The source line.
    pub line: usize,
}

/// A transition of a state machine: to `to` when every check holds and the
/// state has lasted `after` seconds.
#[derive(Clone, Debug, PartialEq)]
pub struct Transition {
    pub to: usize,
    pub checks: Vec<Check>,
    pub after: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct State {
    pub name: String,
    /// Instant actions on entering and leaving (`set`, `stop`, `command`).
    pub enter: Vec<Action>,
    pub exit: Vec<Action>,
    /// The node ticked while in the state (a sequence of the state's nodes), if any.
    pub body: Option<usize>,
    pub transitions: Vec<Transition>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Root {
    Tree(usize),
    Machine { states: Vec<State>, initial: usize },
}

/// A parsed behaviour.
#[derive(Clone, Debug, PartialEq)]
pub struct Behaviour {
    pub name: String,
    /// Declared keys and their starting values, in order.
    pub keys: Vec<(String, Value)>,
    pub routes: Vec<(String, Vec<Vec3>)>,
    pub nodes: Vec<Node>,
    pub root: Root,
}

/// The keys og.ai writes every tick from the agent's perception and body.
pub const BUILTIN_KEYS: &[&str] = &[
    "self",
    "position",
    "sees",
    "heard",
    "target",
    "targetPosition",
    "targetDistance",
    "lastSeen",
    "sinceSeen",
    "heardAt",
    "sinceHeard",
    "elapsed",
];

fn err(e: &Entry, msg: impl Into<String>) -> String {
    format!("line {}: {}", e.line, msg.into())
}

fn num(v: &OgtValue) -> Option<f64> {
    v.as_number().map(|n| n.as_f64())
}

/// A value from an entry's values starting at `i`: a number, three numbers, `true`/`false`.
fn value_at(e: &Entry, i: usize) -> Result<Value, String> {
    let vs = &e.values[i.min(e.values.len())..];
    match vs {
        [a, b, c] if num(a).is_some() && num(b).is_some() && num(c).is_some() => {
            Ok(Value::Vec(Vec3::new(
                num(a).unwrap() as f32,
                num(b).unwrap() as f32,
                num(c).unwrap() as f32,
            )))
        }
        [a] if num(a).is_some() => Ok(Value::Num(num(a).unwrap())),
        [a] if a.as_bool().is_some() => Ok(Value::Bool(a.as_bool().unwrap())),
        _ => Err(err(
            e,
            format!(
                "'{}' needs a value: a number, three numbers (a point) or true/false",
                e.key
            ),
        )),
    }
}

fn opt_f32(b: Option<&Block>, key: &str, default: f32) -> Result<f32, String> {
    match b.and_then(|b| b.get(key)) {
        None => Ok(default),
        Some(e) => e
            .f32(0)
            .map_err(|_| err(e, format!("'{key}' takes a number"))),
    }
}

fn opt_speed(b: Option<&Block>) -> Result<Option<Speed>, String> {
    match b.and_then(|b| b.get("speed")) {
        None => Ok(None),
        Some(e) => match e.values.first() {
            Some(OgtValue::Number(n)) => Ok(Some(Speed::Fixed(n.as_f32()))),
            Some(v) if word_or_string(v).is_some() => {
                Ok(Some(Speed::Read(word_or_string(v).unwrap().to_string())))
            }
            _ => Err(err(
                e,
                "'speed' takes metres per second, or a key or field holding them (Tuning.slimeSpeed)",
            )),
        },
    }
}

fn word_or_string(v: &OgtValue) -> Option<&str> {
    v.as_word().or_else(|| v.as_str())
}

/// Every option key a node's block may hold.
fn check_options(e: &Entry, allowed: &[&str]) -> Result<(), String> {
    if let Some(b) = &e.block {
        for o in &b.entries {
            if !allowed.contains(&o.key.as_str()) {
                return Err(err(
                    o,
                    format!(
                        "'{}' is not an option of {} (it takes {})",
                        o.key,
                        e.key,
                        if allowed.is_empty() {
                            "none".to_string()
                        } else {
                            allowed.join(", ")
                        }
                    ),
                ));
            }
        }
    }
    Ok(())
}

pub(crate) fn parse_check(e: &Entry) -> Result<Check, String> {
    let operand = e
        .values
        .first()
        .and_then(word_or_string)
        .ok_or_else(|| {
            err(
                e,
                "check needs an operand: a key, Component.field or Resource.field",
            )
        })?
        .to_string();
    if e.values.len() == 1 {
        return Ok(Check {
            operand,
            op: Op::Truthy,
            value: Value::Bool(true),
            against: None,
        });
    }
    let op_word = e.values[1].as_word().ok_or_else(|| {
        err(
            e,
            "check's second word is an operator: is, not, below, above, atMost or atLeast",
        )
    })?;
    let op = Op::parse(op_word).ok_or_else(|| {
        err(
            e,
            format!("'{op_word}' is not an operator: is, not, below, above, atMost or atLeast"),
        )
    })?;
    // A key or a field to compare with, read each time (Tuning.fleeHp).
    if let [v] = &e.values[2..]
        && v.as_bool().is_none()
        && let Some(r) = word_or_string(v)
    {
        return Ok(Check {
            operand,
            op,
            value: Value::Num(0.0),
            against: Some(r.to_string()),
        });
    }
    let value = value_at(e, 2).map_err(|_| {
        err(
            e,
            "'check' compares with a value: a number, three numbers (a point), true/false, or a key or field holding it (Tuning.fleeHp)",
        )
    })?;
    Ok(Check {
        operand,
        op,
        value,
        against: None,
    })
}

struct Parser {
    nodes: Vec<Node>,
    routes: Vec<(String, Vec<Vec3>)>,
}

impl Parser {
    fn action(&self, e: &Entry) -> Result<Option<Action>, String> {
        let b = e.block.as_ref();
        let arg = e.values.first().and_then(word_or_string);
        Ok(Some(match e.key.as_str() {
            "move_to" => {
                check_options(e, &["speed"])?;
                if e.values.len() == 3 {
                    let Value::Vec(p) = value_at(e, 0)? else {
                        unreachable!()
                    };
                    Action::MoveTo {
                        key: None,
                        point: p,
                        speed: opt_speed(b)?,
                    }
                } else {
                    let k =
                        arg.ok_or_else(|| err(e, "move_to takes a key holding a point, or x y z"))?;
                    Action::MoveTo {
                        key: Some(k.to_string()),
                        point: Vec3::ZERO,
                        speed: opt_speed(b)?,
                    }
                }
            }
            "chase" => {
                check_options(e, &["speed", "reach"])?;
                Action::Chase {
                    key: arg.unwrap_or("target").to_string(),
                    speed: opt_speed(b)?,
                    reach: opt_f32(b, "reach", 0.5)?,
                }
            }
            "flee" => {
                check_options(e, &["speed", "distance"])?;
                Action::Flee {
                    key: arg.unwrap_or("target").to_string(),
                    speed: opt_speed(b)?,
                    distance: opt_f32(b, "distance", 6.0)?,
                }
            }
            "patrol" => {
                check_options(e, &["speed", "wait", "loop"])?;
                let name = arg.ok_or_else(|| err(e, "patrol takes a route's name (declare it with route <name> { point x y z ... })"))?;
                let route = self.routes.iter().position(|r| r.0 == name).ok_or_else(|| err(e, format!("there is no route '{name}': declare it with route {name} {{ point x y z ... }} before the tree")))?;
                let loops = match b.and_then(|b| b.get("loop")) {
                    None => true,
                    Some(l) => l
                        .values
                        .first()
                        .and_then(OgtValue::as_bool)
                        .ok_or_else(|| err(l, "loop takes true or false"))?,
                };
                Action::Patrol {
                    route,
                    wait: opt_f32(b, "wait", 0.0)?,
                    speed: opt_speed(b)?,
                    loops,
                }
            }
            "wander" => {
                check_options(e, &["speed", "radius", "wait"])?;
                Action::Wander {
                    radius: opt_f32(b, "radius", 4.0)?,
                    speed: opt_speed(b)?,
                    wait: opt_f32(b, "wait", 0.0)?,
                }
            }
            "wait" => {
                check_options(e, &[])?;
                Action::Wait {
                    seconds: e.f32(0).map_err(|_| err(e, "wait takes seconds"))?,
                }
            }
            "stop" => {
                check_options(e, &[])?;
                Action::Stop
            }
            "set" => {
                check_options(e, &[])?;
                let key = arg
                    .ok_or_else(|| err(e, "set takes a key and a value"))?
                    .to_string();
                if BUILTIN_KEYS.contains(&key.as_str()) {
                    return Err(err(
                        e,
                        format!("'{key}' is written by og.ai every tick; set a key of your own"),
                    ));
                }
                Action::Set {
                    key,
                    value: value_at(e, 1)?,
                }
            }
            "command" => {
                check_options(e, &["cooldown"])?;
                let name = arg.ok_or_else(|| {
                    err(
                        e,
                        "command takes the game command's name (GameDesc.commands)",
                    )
                })?;
                if !name
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
                    || name.is_empty()
                {
                    return Err(err(
                        e,
                        format!("'{name}' is not a command name: commands are snake_case"),
                    ));
                }
                let args = e.values[1..]
                    .iter()
                    .map(|v| match v {
                        OgtValue::Number(n) => n.text().to_string(),
                        OgtValue::String(s) | OgtValue::Word(s) => s.clone(),
                    })
                    .collect();
                Action::Command {
                    name: name.to_string(),
                    args,
                    cooldown: opt_f32(b, "cooldown", 0.0)?,
                }
            }
            _ => return Ok(None),
        }))
    }

    fn node(&mut self, e: &Entry) -> Result<usize, String> {
        let label = e.values.first().and_then(word_or_string);
        let composite = |p: &mut Parser, kind: Kind, e: &Entry| -> Result<usize, String> {
            let b = e
                .block
                .as_ref()
                .filter(|b| !b.entries.is_empty())
                .ok_or_else(|| err(e, format!("{} needs children in a block", e.key)))?;
            let mut children = Vec::new();
            for c in &b.entries {
                if e.key == "parallel" && c.key == "policy" {
                    continue;
                }
                children.push(p.node(c)?);
            }
            if children.is_empty() {
                return Err(err(e, format!("{} needs children", e.key)));
            }
            let name = match e.values.first().and_then(word_or_string) {
                Some(l) => format!("{} {l}", e.key),
                None => e.key.clone(),
            };
            p.nodes.push(Node {
                kind,
                name,
                children,
                line: e.line,
            });
            Ok(p.nodes.len() - 1)
        };
        let decorator = |p: &mut Parser, kind: Kind, e: &Entry| -> Result<usize, String> {
            let b = e
                .block
                .as_ref()
                .ok_or_else(|| err(e, format!("{} needs one child in a block", e.key)))?;
            if b.entries.len() != 1 {
                return Err(err(
                    e,
                    format!(
                        "{} takes exactly one child, it has {}",
                        e.key,
                        b.entries.len()
                    ),
                ));
            }
            let child = p.node(&b.entries[0])?;
            let name = if e.values.is_empty() {
                e.key.clone()
            } else {
                format!(
                    "{} {}",
                    e.key,
                    e.values
                        .iter()
                        .map(|v| v.as_number().map_or_else(
                            || word_or_string(v).unwrap_or("").to_string(),
                            |n| n.text().to_string()
                        ))
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            };
            p.nodes.push(Node {
                kind,
                name,
                children: vec![child],
                line: e.line,
            });
            Ok(p.nodes.len() - 1)
        };
        match e.key.as_str() {
            "selector" => composite(self, Kind::Selector, e),
            "sequence" => composite(self, Kind::Sequence, e),
            "steps" => composite(self, Kind::Steps, e),
            "parallel" => {
                let all = match e.block.as_ref().and_then(|b| b.get("policy")) {
                    None => true,
                    Some(p) => match p.values.first().and_then(OgtValue::as_word) {
                        Some("all") => true,
                        Some("one") => false,
                        _ => return Err(err(p, "policy is all or one")),
                    },
                };
                composite(self, Kind::Parallel { all }, e)
            }
            "invert" => decorator(self, Kind::Invert, e),
            "succeed" => decorator(self, Kind::Succeed, e),
            "repeat" => {
                let times = match e.values.first() {
                    None => 0,
                    Some(v) => v.as_number().and_then(|n| n.as_u64().ok()).ok_or_else(|| {
                        err(e, "repeat takes a whole number of times (0: forever)")
                    })? as u32,
                };
                decorator(self, Kind::Repeat { times }, e)
            }
            "cooldown" => decorator(
                self,
                Kind::Cooldown {
                    seconds: e.f32(0).map_err(|_| err(e, "cooldown takes seconds"))?,
                },
                e,
            ),
            "timeout" => decorator(
                self,
                Kind::Timeout {
                    seconds: e.f32(0).map_err(|_| err(e, "timeout takes seconds"))?,
                },
                e,
            ),
            "check" => {
                let c = parse_check(e)?;
                let name = format!(
                    "check {}{}",
                    c.operand,
                    if c.op == Op::Truthy {
                        String::new()
                    } else {
                        format!(
                            " {} {}",
                            c.op.name(),
                            c.against.clone().unwrap_or_else(|| describe(c.value))
                        )
                    }
                );
                self.nodes.push(Node {
                    kind: Kind::Check(c),
                    name,
                    children: Vec::new(),
                    line: e.line,
                });
                Ok(self.nodes.len() - 1)
            }
            _ => match self.action(e)? {
                Some(a) => {
                    let name = match label {
                        Some(l) => format!("{} {l}", e.key),
                        None => e.key.clone(),
                    };
                    self.nodes.push(Node {
                        kind: Kind::Action(a),
                        name,
                        children: Vec::new(),
                        line: e.line,
                    });
                    Ok(self.nodes.len() - 1)
                }
                None => Err(err(
                    e,
                    format!(
                        "'{}' is not a node: selector, sequence, steps, parallel, invert, succeed, repeat, cooldown, timeout, check, move_to, chase, flee, patrol, wander, wait, stop, set or command",
                        e.key
                    ),
                )),
            },
        }
    }
}

fn describe(v: Value) -> String {
    match v {
        Value::Num(n) => format!("{n}"),
        Value::Bool(b) => b.to_string(),
        Value::Vec(v) => format!("{} {} {}", v.x, v.y, v.z),
        Value::Entity(e) => format!("#{e}"),
    }
}

impl Behaviour {
    /// Parses and validates a behaviour's text.
    pub fn parse(text: &str) -> Result<Behaviour, String> {
        let doc = Document::parse(text).map_err(|e| e.to_string())?;
        if doc.ty != BEHAVIOUR_TYPE {
            return Err(format!(
                "the file is of type '{}', not '{BEHAVIOUR_TYPE}'",
                doc.ty
            ));
        }
        let body = &doc.body;
        let name = body
            .get("name")
            .and_then(|e| e.values.first())
            .and_then(word_or_string)
            .unwrap_or("behaviour")
            .to_string();
        let mut keys = Vec::new();
        let mut routes = Vec::new();
        for e in &body.entries {
            match e.key.as_str() {
                "name" | "tree" | "machine" => {}
                "key" => {
                    let k = e
                        .values
                        .first()
                        .and_then(OgtValue::as_word)
                        .ok_or_else(|| err(e, "key takes a name and a starting value"))?
                        .to_string();
                    if BUILTIN_KEYS.contains(&k.as_str()) {
                        return Err(err(
                            e,
                            format!("'{k}' is a key og.ai writes itself; name yours differently"),
                        ));
                    }
                    if keys.iter().any(|(n, _): &(String, Value)| *n == k) {
                        return Err(err(e, format!("the key '{k}' is declared twice")));
                    }
                    keys.push((k, value_at(e, 1)?));
                }
                "route" => {
                    let r = e
                        .values
                        .first()
                        .and_then(word_or_string)
                        .ok_or_else(|| err(e, "route takes a name and a block of points"))?
                        .to_string();
                    let b = e
                        .block
                        .as_ref()
                        .ok_or_else(|| err(e, "route needs a block of `point x y z` lines"))?;
                    let mut pts = Vec::new();
                    for p in &b.entries {
                        if p.key != "point" {
                            return Err(err(p, "a route holds `point x y z` lines"));
                        }
                        match value_at(p, 0)? {
                            Value::Vec(v) => pts.push(v),
                            _ => return Err(err(p, "a point is three numbers: x y z")),
                        }
                    }
                    if pts.is_empty() {
                        return Err(err(e, format!("the route '{r}' has no points")));
                    }
                    routes.push((r, pts));
                }
                other => {
                    return Err(err(
                        e,
                        format!(
                            "'{other}' is not a behaviour entry: name, key, route, tree or machine"
                        ),
                    ));
                }
            }
        }
        let mut p = Parser {
            nodes: Vec::new(),
            routes,
        };
        let root = match (body.get("tree"), body.get("machine")) {
            (Some(_), Some(m)) => {
                return Err(err(m, "a behaviour is a tree or a machine, not both"));
            }
            (Some(t), None) => {
                let b = t
                    .block
                    .as_ref()
                    .filter(|b| b.entries.len() == 1)
                    .ok_or_else(|| err(t, "tree takes one root node in its block"))?;
                Root::Tree(p.node(&b.entries[0])?)
            }
            (None, Some(m)) => p.machine(m)?,
            (None, None) => {
                return Err("a behaviour needs a `tree { ... }` or a `machine { ... }`".into());
            }
        };
        Ok(Behaviour {
            name,
            keys,
            routes: p.routes,
            nodes: p.nodes,
            root,
        })
    }

    /// The canonical display name of a node path (`selector > sequence chase > chase`).
    pub fn node_name(&self, i: usize) -> &str {
        self.nodes.get(i).map_or("", |n| n.name.as_str())
    }

    /// Every operand a check names (either side) that is not a key: `Component.field` or `Resource.field`.
    pub fn field_operands(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut push = |c: &Check| {
            for o in std::iter::once(&c.operand).chain(c.against.as_ref()) {
                if !self.keys.iter().any(|k| &k.0 == o)
                    && !BUILTIN_KEYS.contains(&o.as_str())
                    && o.contains('.')
                    && !out.contains(o)
                {
                    out.push(o.clone());
                }
            }
        };
        for n in &self.nodes {
            if let Kind::Check(c) = &n.kind {
                push(c);
            }
        }
        if let Root::Machine { states, .. } = &self.root {
            for s in states {
                for t in &s.transitions {
                    for c in &t.checks {
                        push(c);
                    }
                }
            }
        }
        out
    }
}

impl Parser {
    fn machine(&mut self, m: &Entry) -> Result<Root, String> {
        let b = m
            .block
            .as_ref()
            .ok_or_else(|| err(m, "machine needs a block of states"))?;
        let names: Vec<String> = b
            .get_all("state")
            .map(|s| {
                s.values
                    .first()
                    .and_then(word_or_string)
                    .unwrap_or("")
                    .to_string()
            })
            .collect();
        if names.is_empty() {
            return Err(err(
                m,
                "a machine needs at least one `state <name> { ... }`",
            ));
        }
        for (i, n) in names.iter().enumerate() {
            if n.is_empty() {
                return Err(err(m, "every state needs a name"));
            }
            if names[..i].contains(n) {
                return Err(err(m, format!("the state '{n}' is declared twice")));
            }
        }
        let state_index = |e: &Entry, n: &str| -> Result<usize, String> {
            names.iter().position(|s| s == n).ok_or_else(|| {
                err(
                    e,
                    format!(
                        "there is no state '{n}' (the machine has {})",
                        names.join(", ")
                    ),
                )
            })
        };
        let initial = match b.get("initial") {
            Some(i) => state_index(i, i.values.first().and_then(word_or_string).unwrap_or(""))?,
            None => 0,
        };
        let mut states = Vec::new();
        for e in &b.entries {
            match e.key.as_str() {
                "initial" => continue,
                "state" => {}
                other => {
                    return Err(err(
                        e,
                        format!("'{other}' is not a machine entry: initial or state"),
                    ));
                }
            }
            let name = e
                .values
                .first()
                .and_then(word_or_string)
                .unwrap_or("")
                .to_string();
            let mut state = State {
                name: name.clone(),
                enter: Vec::new(),
                exit: Vec::new(),
                body: None,
                transitions: Vec::new(),
            };
            let mut body = Vec::new();
            for c in e
                .block
                .as_ref()
                .map(|b| b.entries.as_slice())
                .unwrap_or(&[])
            {
                match c.key.as_str() {
                    "enter" | "exit" => {
                        let list = if c.key == "enter" {
                            &mut state.enter
                        } else {
                            &mut state.exit
                        };
                        for a in c
                            .block
                            .as_ref()
                            .map(|b| b.entries.as_slice())
                            .unwrap_or(&[])
                        {
                            match self.action(a)? {
                                Some(
                                    act @ (Action::Set { .. }
                                    | Action::Stop
                                    | Action::Command { .. }),
                                ) => list.push(act),
                                _ => {
                                    return Err(err(
                                        a,
                                        format!(
                                            "{} runs instant actions only: set, stop or command",
                                            c.key
                                        ),
                                    ));
                                }
                            }
                        }
                    }
                    "to" => {
                        let to = state_index(
                            c,
                            c.values.first().and_then(word_or_string).unwrap_or(""),
                        )?;
                        let mut t = Transition {
                            to,
                            checks: Vec::new(),
                            after: 0.0,
                        };
                        for k in c
                            .block
                            .as_ref()
                            .map(|b| b.entries.as_slice())
                            .unwrap_or(&[])
                        {
                            match k.key.as_str() {
                                "check" => t.checks.push(parse_check(k)?),
                                "after" => {
                                    t.after = k
                                        .f32(0)
                                        .map_err(|_| err(k, "after takes seconds in the state"))?
                                }
                                other => {
                                    return Err(err(
                                        k,
                                        format!(
                                            "'{other}' is not a transition entry: check or after"
                                        ),
                                    ));
                                }
                            }
                        }
                        state.transitions.push(t);
                    }
                    _ => body.push(self.node(c)?),
                }
            }
            state.body = match body.len() {
                0 => None,
                1 => Some(body[0]),
                _ => {
                    self.nodes.push(Node {
                        kind: Kind::Sequence,
                        name: format!("state {name}"),
                        children: body,
                        line: e.line,
                    });
                    Some(self.nodes.len() - 1)
                }
            };
            states.push(state);
        }
        Ok(Root::Machine { states, initial })
    }
}
