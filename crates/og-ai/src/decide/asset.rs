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
//!   ReactiveFallback {
//!     ReactiveSequence flee {
//!       check Slime.hp below 20
//!       flee
//!     }
//!     ReactiveSequence chase {
//!       check sees
//!       chase
//!     }
//!     patrol beat
//!   }
//! }
//! ```

mod parse;

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
    /// A game action (`action <name> args`): running from when it starts
    /// (og.AiActionStarted) until the game ends it with og.ai.finish, then
    /// success or failure as the game said (og.AiActionStopped).
    Game { name: String, args: Vec<String> },
}

/// How a consideration's input maps onto its score: `t` is the input
/// placed between `lo` (0) and `hi` (1) and clamped to that range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Curve {
    /// `t`.
    Linear,
    /// `1 - t`.
    Inverse,
    /// `t * t`: low inputs matter little.
    Quadratic,
    /// `sqrt t`: low inputs already matter.
    Sqrt,
    /// `3t² - 2t³`: an S between the two ends.
    Smooth,
    /// 1 from `hi` on, else 0.
    Step,
}

impl Curve {
    const NAMES: &[(&str, Curve)] = &[
        ("linear", Curve::Linear),
        ("inverse", Curve::Inverse),
        ("quadratic", Curve::Quadratic),
        ("sqrt", Curve::Sqrt),
        ("smooth", Curve::Smooth),
        ("step", Curve::Step),
    ];

    /// The score for input `v` between `lo` and `hi` (plain f32 arithmetic and sqrt).
    pub fn score(self, v: f32, lo: f32, hi: f32) -> f32 {
        let t = if hi != lo {
            (v - lo) / (hi - lo)
        } else if v >= hi {
            1.0
        } else {
            0.0
        };
        let t = if t.is_nan() { 0.0 } else { t.clamp(0.0, 1.0) };
        match self {
            Curve::Linear => t,
            Curve::Inverse => 1.0 - t,
            Curve::Quadratic => t * t,
            Curve::Sqrt => t.sqrt(),
            Curve::Smooth => t * t * (3.0 - 2.0 * t),
            Curve::Step => {
                if t >= 1.0 {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }
}

/// One factor of a utility option's score: a constant, or a key or field
/// read each think through a curve.
#[derive(Clone, Debug, PartialEq)]
pub struct Consideration {
    /// None: `constant`.
    pub operand: Option<String>,
    pub constant: f32,
    pub curve: Curve,
    pub lo: f32,
    pub hi: f32,
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
    /// Another behaviour's tree, inlined (`use "ai/patrol.ogt"`): its one child.
    Subtree {
        path: String,
    },
    /// Its children are options; each think the option with the highest
    /// score runs (the running one gets `inertia` more; ties to the first;
    /// none above 0 fails).
    Utility {
        inertia: f32,
    },
    /// A utility's option: its score is the product of its considerations;
    /// its children run as a sequence.
    Option {
        scores: Vec<Consideration>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub kind: Kind,
    /// A label (`ReactiveSequence chase`) or the node's kind and argument (`patrol beat`).
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
    /// The behaviours its `use` nodes inline, every one reached, in the
    /// order first met.
    pub subtrees: Vec<String>,
}

/// Reads a subtree's text by its asset path (`use "ai/patrol.ogt"`).
pub type Resolve<'a> = &'a mut dyn FnMut(&str) -> Result<String, String>;

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

impl Behaviour {
    /// The canonical display name of a node path (`ReactiveFallback > ReactiveSequence chase > chase`).
    pub fn node_name(&self, i: usize) -> &str {
        self.nodes.get(i).map_or("", |n| n.name.as_str())
    }

    /// Every operand a check names (either side) that is not a key: `Component.field` or `Resource.field`.
    pub fn field_operands(&self) -> Vec<String> {
        let mut named: Vec<&String> = Vec::new();
        for n in &self.nodes {
            match &n.kind {
                Kind::Check(c) => named.extend(std::iter::once(&c.operand).chain(&c.against)),
                Kind::Option { scores } => {
                    named.extend(scores.iter().filter_map(|f| f.operand.as_ref()))
                }
                _ => {}
            }
        }
        if let Root::Machine { states, .. } = &self.root {
            for c in states
                .iter()
                .flat_map(|s| &s.transitions)
                .flat_map(|t| &t.checks)
            {
                named.extend(std::iter::once(&c.operand).chain(&c.against));
            }
        }
        let mut out: Vec<String> = Vec::new();
        for o in named {
            if !self.keys.iter().any(|k| &k.0 == o)
                && !BUILTIN_KEYS.contains(&o.as_str())
                && o.contains('.')
                && !out.contains(o)
            {
                out.push(o.clone());
            }
        }
        out
    }
}
