//! Resolve named definitions once into the native tree and machine indices.
use super::{asset as a, definition as d};
use og_core::glam::Vec3;

impl a::Behaviour {
    /// Compile named input, resolving subtree references to validated native trees.
    /// The caller owns reference identity and cycle detection at its asset boundary.
    pub fn from_definition<R>(
        input: d::Behaviour<R>,
        resolve: &mut dyn FnMut(R) -> Result<(String, Self), String>,
    ) -> Result<Self, String> {
        let mut c = Compiler {
            output: Self {
                name: input.name,
                keys: input.keys.into_iter().map(|(k, v)| (k, value(v))).collect(),
                routes: input
                    .routes
                    .into_iter()
                    .map(|(k, p)| (k, p.into_iter().map(Vec3::from_array).collect()))
                    .collect(),
                nodes: Vec::new(),
                root: a::Root::Tree(0),
                subtrees: Vec::new(),
            },
            resolve,
        };
        c.output.root = match input.root {
            d::Root::Tree { node } => a::Root::Tree(c.node(node)?),
            d::Root::Machine { initial, states } => {
                let names = states.iter().map(|s| s.name.clone()).collect::<Vec<_>>();
                let index = |name: &str| {
                    names
                        .iter()
                        .position(|n| n == name)
                        .ok_or_else(|| format!("unknown state '{name}'"))
                };
                let initial = index(&initial)?;
                let states = states
                    .into_iter()
                    .map(|s| {
                        Ok(a::State {
                            name: s.name,
                            enter: s
                                .enter
                                .into_iter()
                                .map(|a| c.action(a))
                                .collect::<Result<_, _>>()?,
                            exit: s
                                .exit
                                .into_iter()
                                .map(|a| c.action(a))
                                .collect::<Result<_, _>>()?,
                            body: s.body.map(|n| c.node(n)).transpose()?,
                            transitions: s
                                .transitions
                                .into_iter()
                                .map(|t| {
                                    Ok(a::Transition {
                                        to: index(&t.to)?,
                                        after: t.after.unwrap_or(0.0),
                                        checks: t
                                            .checks
                                            .into_iter()
                                            .map(check)
                                            .collect::<Result<_, _>>()?,
                                    })
                                })
                                .collect::<Result<_, String>>()?,
                        })
                    })
                    .collect::<Result<_, String>>()?;
                a::Root::Machine { initial, states }
            }
        };
        c.output.validate()?;
        Ok(c.output)
    }
}

struct Compiler<'a, R> {
    output: a::Behaviour,
    resolve: &'a mut dyn FnMut(R) -> Result<(String, a::Behaviour), String>,
}

fn value(v: d::Value) -> a::Value {
    match v {
        d::Value::Number(n) => a::Value::Num(n),
        d::Value::Bool(b) => a::Value::Bool(b),
        d::Value::Vector(v) => a::Value::Vec(Vec3::from_array(v)),
    }
}
fn speed(s: Option<d::Speed>) -> Option<a::Speed> {
    s.map(|s| match s {
        d::Speed::Fixed(n) => a::Speed::Fixed(n),
        d::Speed::Read(k) => a::Speed::Read(k),
    })
}
fn check(c: d::Check) -> Result<a::Check, String> {
    let op = match c.op {
        d::Op::Is => a::Op::Is,
        d::Op::Not => a::Op::Not,
        d::Op::Below => a::Op::Below,
        d::Op::Above => a::Op::Above,
        d::Op::AtMost => a::Op::AtMost,
        d::Op::AtLeast => a::Op::AtLeast,
        d::Op::Truthy => a::Op::Truthy,
    };
    if if op == a::Op::Truthy {
        c.value.is_some() || c.against.is_some()
    } else {
        c.value.is_some() == c.against.is_some()
    } {
        return Err(format!(
            "check '{}': truthy takes no comparison; other operators take exactly one of value or against",
            c.operand
        ));
    }
    Ok(a::Check {
        operand: c.operand,
        op,
        value: c.value.map(value).unwrap_or(a::Value::Bool(true)),
        against: c.against,
    })
}

impl<R> Compiler<'_, R> {
    fn action(&self, action: d::Action) -> Result<a::Action, String> {
        Ok(match action {
            d::Action::MoveTo {
                key,
                point,
                speed: s,
            } => {
                if key.is_some() == point.is_some() {
                    return Err("move-to takes exactly one of key or point".into());
                }
                a::Action::MoveTo {
                    key,
                    point: Vec3::from_array(point.unwrap_or([0.0; 3])),
                    speed: speed(s),
                }
            }
            d::Action::Chase {
                key,
                speed: s,
                reach,
            } => a::Action::Chase {
                key,
                speed: speed(s),
                reach,
            },
            d::Action::Flee {
                key,
                speed: s,
                distance,
            } => a::Action::Flee {
                key,
                speed: speed(s),
                distance,
            },
            d::Action::Patrol {
                route,
                wait,
                speed: s,
                loops,
            } => a::Action::Patrol {
                route: self
                    .output
                    .routes
                    .iter()
                    .position(|r| r.0 == route)
                    .ok_or_else(|| format!("unknown route '{route}'"))?,
                wait,
                speed: speed(s),
                loops,
            },
            d::Action::Wander {
                radius,
                speed: s,
                wait,
            } => a::Action::Wander {
                radius,
                speed: speed(s),
                wait,
            },
            d::Action::Wait { seconds } => a::Action::Wait { seconds },
            d::Action::Stop => a::Action::Stop,
            d::Action::Set { key, value: v } => a::Action::Set {
                key,
                value: value(v),
            },
            d::Action::Command {
                name,
                args,
                cooldown,
            } => a::Action::Command {
                name,
                args,
                cooldown,
            },
            d::Action::Game { name, args } => a::Action::Game { name, args },
        })
    }

    fn push(&mut self, kind: a::Kind, name: String, children: Vec<usize>) -> usize {
        let id = self.output.nodes.len();
        self.output.nodes.push(a::Node {
            kind,
            name,
            children,
        });
        id
    }

    fn node(&mut self, node: d::Node<R>) -> Result<usize, String> {
        use d::Node::*;
        let (kind, name, default, children) = match node {
            Selector { name, children } => (a::Kind::Selector, name, "selector", children),
            Sequence { name, children } => (a::Kind::Sequence, name, "sequence", children),
            Steps { name, children } => (a::Kind::Steps, name, "steps", children),
            Parallel {
                name,
                all,
                children,
            } => (a::Kind::Parallel { all }, name, "parallel", children),
            Invert { name, child } => (a::Kind::Invert, name, "invert", vec![*child]),
            Succeed { name, child } => (a::Kind::Succeed, name, "succeed", vec![*child]),
            Repeat { name, times, child } => {
                (a::Kind::Repeat { times }, name, "repeat", vec![*child])
            }
            Cooldown {
                name,
                seconds,
                child,
            } => (
                a::Kind::Cooldown { seconds },
                name,
                "cooldown",
                vec![*child],
            ),
            Timeout {
                name,
                seconds,
                child,
            } => (a::Kind::Timeout { seconds }, name, "timeout", vec![*child]),
            Check { name, check: c } => (a::Kind::Check(check(c)?), name, "check", Vec::new()),
            Action { name, action } => (
                a::Kind::Action(self.action(action)?),
                name,
                "action",
                Vec::new(),
            ),
            Subtree { name, source } => {
                let (path, sub) = (self.resolve)(source)?;
                sub.validate()?;
                let a::Root::Tree(root) = sub.root else {
                    return Err(format!("subtree '{path}' is a machine; a tree is required"));
                };
                let mut routes = Vec::new();
                for (name, points) in sub.routes {
                    let index = match self.output.routes.iter().position(|r| r.0 == name) {
                        Some(i) if self.output.routes[i].1 == points => i,
                        Some(_) => {
                            return Err(format!(
                                "subtree '{path}': route '{name}' has different points"
                            ));
                        }
                        None => {
                            self.output.routes.push((name, points));
                            self.output.routes.len() - 1
                        }
                    };
                    routes.push(index);
                }
                for (name, v) in sub.keys {
                    if !self.output.keys.iter().any(|k| k.0 == name) {
                        self.output.keys.push((name, v));
                    }
                }
                let offset = self.output.nodes.len();
                for mut node in sub.nodes {
                    for child in &mut node.children {
                        *child += offset;
                    }
                    if let a::Kind::Action(a::Action::Patrol { route, .. }) = &mut node.kind {
                        *route = routes[*route];
                    }
                    self.output.nodes.push(node);
                }
                for id in std::iter::once(path.clone()).chain(sub.subtrees) {
                    if !self.output.subtrees.contains(&id) {
                        self.output.subtrees.push(id);
                    }
                }
                return Ok(self.push(
                    a::Kind::Subtree { path: path.clone() },
                    name.unwrap_or_else(|| format!("subtree {path}")),
                    vec![offset + root],
                ));
            }
            Utility {
                name,
                inertia,
                options,
            } => {
                let mut children = Vec::new();
                for option in options {
                    let scores = option
                        .scores
                        .into_iter()
                        .map(|s| match s {
                            d::Consideration::Constant { value } => a::Consideration {
                                operand: None,
                                constant: value,
                                curve: a::Curve::Linear,
                                lo: 0.0,
                                hi: 1.0,
                            },
                            d::Consideration::Read {
                                operand,
                                curve,
                                lo,
                                hi,
                            } => a::Consideration {
                                operand: Some(operand),
                                constant: 0.0,
                                lo,
                                hi,
                                curve: match curve {
                                    d::Curve::Linear => a::Curve::Linear,
                                    d::Curve::Inverse => a::Curve::Inverse,
                                    d::Curve::Quadratic => a::Curve::Quadratic,
                                    d::Curve::Sqrt => a::Curve::Sqrt,
                                    d::Curve::Smooth => a::Curve::Smooth,
                                    d::Curve::Step => a::Curve::Step,
                                },
                            },
                        })
                        .collect();
                    let nodes = option
                        .children
                        .into_iter()
                        .map(|n| self.node(n))
                        .collect::<Result<_, _>>()?;
                    children.push(self.push(a::Kind::Option { scores }, option.name, nodes));
                }
                return Ok(self.push(
                    a::Kind::Utility { inertia },
                    name.unwrap_or_else(|| "utility".into()),
                    children,
                ));
            }
        };
        let children = children
            .into_iter()
            .map(|n| self.node(n))
            .collect::<Result<_, _>>()?;
        Ok(self.push(kind, name.unwrap_or_else(|| default.into()), children))
    }
}
