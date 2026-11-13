//! Parsing a behaviour's text: its checks, actions and nodes, each error
//! naming its line and the fix.

mod compound;

use super::*;

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

struct Parser<'r, 'a> {
    nodes: Vec<Node>,
    routes: Vec<(String, Vec<Vec3>)>,
    /// Keys the subtrees declare that the behaviour does not.
    keys: Vec<(String, Value)>,
    subtrees: Vec<String>,
    resolve: Option<&'r mut Resolve<'a>>,
    /// The files whose `use` led here, the behaviour's own first.
    chain: Vec<String>,
}

fn parse_consideration(e: &Entry) -> Result<Consideration, String> {
    let bad = || {
        err(
            e,
            "score takes a constant (`score 0.3`) or an input, a curve and its range: `score targetDistance inverse 0 10` (curves: linear, inverse, quadratic, sqrt, smooth, step)",
        )
    };
    match e.values.as_slice() {
        [v] => {
            let c = num(v).ok_or_else(bad)? as f32;
            Ok(Consideration {
                operand: None,
                constant: c,
                curve: Curve::Linear,
                lo: 0.0,
                hi: 1.0,
            })
        }
        [o, c, lo, hi] => {
            let operand = word_or_string(o).ok_or_else(bad)?.to_string();
            let curve_word = c.as_word().ok_or_else(bad)?;
            let curve = Curve::NAMES
                .iter()
                .find(|(n, _)| *n == curve_word)
                .map(|(_, c)| *c)
                .ok_or_else(|| {
                    err(
                        e,
                        format!(
                            "'{curve_word}' is not a curve: linear, inverse, quadratic, sqrt, smooth or step"
                        ),
                    )
                })?;
            Ok(Consideration {
                operand: Some(operand),
                constant: 0.0,
                curve,
                lo: num(lo).ok_or_else(bad)? as f32,
                hi: num(hi).ok_or_else(bad)? as f32,
            })
        }
        _ => Err(bad()),
    }
}

impl Parser<'_, '_> {
    fn action(&self, e: &Entry) -> Result<Option<Action>, String> {
        let b = e.block.as_ref();
        let arg = e.values.first().and_then(word_or_string);
        Ok(Some(match e.key.as_str() {
            "action" => {
                check_options(e, &[])?;
                let name = e
                    .values
                    .first()
                    .and_then(word_or_string)
                    .filter(|n| !n.is_empty() && n.len() < 64)
                    .ok_or_else(|| {
                        err(
                            e,
                            "action takes a name (under 64 bytes) and the game's arguments: action reload 2",
                        )
                    })?
                    .to_string();
                let args = e.values[1..]
                    .iter()
                    .map(|v| {
                        v.as_number().map_or_else(
                            || word_or_string(v).unwrap_or("").to_string(),
                            |n| n.text().to_string(),
                        )
                    })
                    .collect::<Vec<String>>();
                if args.join(" ").len() > 255 {
                    return Err(err(
                        e,
                        "an action's arguments are at most 255 bytes, written with a space between them (og.AiActionStarted carries them)",
                    ));
                }
                Action::Game { name, args }
            }
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
            "use" => self.subtree(e),
            "utility" => {
                let b = e
                    .block
                    .as_ref()
                    .ok_or_else(|| err(e, "utility needs a block of `option <name> { ... }`"))?;
                let mut inertia = 0.0;
                let mut children = Vec::new();
                for c in &b.entries {
                    match c.key.as_str() {
                        "inertia" => {
                            inertia = c.f32(0).map_err(|_| {
                                err(c, "inertia takes the score the running option gains")
                            })?
                        }
                        "option" => children.push(self.option(c)?),
                        other => {
                            return Err(err(
                                c,
                                format!("'{other}' is not a utility entry: inertia or option"),
                            ));
                        }
                    }
                }
                if children.is_empty() {
                    return Err(err(e, "utility needs at least one option"));
                }
                self.nodes.push(Node {
                    kind: Kind::Utility { inertia },
                    name: match label {
                        Some(l) => format!("utility {l}"),
                        None => "utility".into(),
                    },
                    children,
                    line: e.line,
                });
                Ok(self.nodes.len() - 1)
            }
            "ReactiveFallback" => composite(self, Kind::Selector, e),
            "ReactiveSequence" => composite(self, Kind::Sequence, e),
            "Sequence" => composite(self, Kind::Steps, e),
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
            "Inverter" => decorator(self, Kind::Invert, e),
            "ForceSuccess" => decorator(self, Kind::Succeed, e),
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
                        "'{}' is not a node: ReactiveFallback, ReactiveSequence, Sequence, parallel, utility, Inverter, ForceSuccess, repeat, cooldown, timeout, check, use, move_to, chase, flee, patrol, wander, wait, stop, set, command or action",
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
    /// Parses and validates a behaviour's text; `use` is refused (it
    /// needs [`Behaviour::parse_with`]).
    pub fn parse(text: &str) -> Result<Behaviour, String> {
        Self::parse_inner(text, None, vec![String::new()])
    }

    /// Parses a behaviour whose `use` nodes read other behaviours through
    /// `resolve` (by asset path). `own` is its own path (empty for text),
    /// so a cycle back to it is refused naming the chain.
    pub fn parse_with(text: &str, own: &str, resolve: Resolve<'_>) -> Result<Behaviour, String> {
        let mut r = resolve;
        Self::parse_inner(text, Some(&mut r), vec![own.to_string()])
    }

    fn parse_inner(
        text: &str,
        resolve: Option<&mut Resolve<'_>>,
        chain: Vec<String>,
    ) -> Result<Behaviour, String> {
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
            keys: keys.clone(),
            subtrees: Vec::new(),
            resolve,
            chain,
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
            keys: p.keys,
            routes: p.routes,
            nodes: p.nodes,
            root,
            subtrees: p.subtrees,
        })
    }
}
