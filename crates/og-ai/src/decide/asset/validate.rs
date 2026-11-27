//! Shared validation of compiled behaviour indices and numeric domains.
use super::*;
fn nonnegative(n: f32) -> bool {
    n.is_finite() && n >= 0.0
}
fn value(v: Value) -> bool {
    match v {
        Value::Num(n) => n.is_finite(),
        Value::Vec(v) => v.is_finite(),
        _ => true,
    }
}
fn check(c: &Check) -> bool {
    !c.operand.is_empty() && c.against.as_ref().is_none_or(|s| !s.is_empty()) && value(c.value)
}
fn speed(s: &Option<Speed>) -> bool {
    match s {
        None => true,
        Some(Speed::Fixed(n)) => nonnegative(*n),
        Some(Speed::Read(s)) => !s.is_empty(),
    }
}
fn action(a: &Action, routes: usize) -> bool {
    match a {
        Action::MoveTo {
            key,
            point,
            speed: s,
        } => key.as_ref().is_none_or(|s| !s.is_empty()) && point.is_finite() && speed(s),
        Action::Chase {
            key,
            speed: s,
            reach,
        } => !key.is_empty() && speed(s) && nonnegative(*reach),
        Action::Flee {
            key,
            speed: s,
            distance,
        } => !key.is_empty() && speed(s) && nonnegative(*distance),
        Action::Patrol {
            route,
            wait,
            speed: s,
            ..
        } => *route < routes && nonnegative(*wait) && speed(s),
        Action::Wander {
            radius,
            speed: s,
            wait,
        } => nonnegative(*radius) && speed(s) && nonnegative(*wait),
        Action::Wait { seconds } => nonnegative(*seconds),
        Action::Stop => true,
        Action::Set { key, value: v } => {
            !key.is_empty() && !BUILTIN_KEYS.contains(&key.as_str()) && value(*v)
        }
        Action::Command { name, cooldown, .. } => {
            !name.is_empty()
                && name
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
                && nonnegative(*cooldown)
        }
        Action::Game { name, args } => {
            !name.is_empty() && name.len() < 64 && args.join(" ").len() <= 255
        }
    }
}
impl Behaviour {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("a behaviour needs a name".into());
        }
        for (i, (key, v)) in self.keys.iter().enumerate() {
            if key.is_empty()
                || BUILTIN_KEYS.contains(&key.as_str())
                || !value(*v)
                || self.keys[..i].iter().any(|(k, _)| k == key)
            {
                return Err(format!("invalid or duplicate blackboard key '{key}'"));
            }
        }
        for (i, (name, points)) in self.routes.iter().enumerate() {
            if name.is_empty()
                || points.is_empty()
                || points.iter().any(|p| !p.is_finite())
                || self.routes[..i].iter().any(|(n, _)| n == name)
            {
                return Err(format!("invalid or duplicate route '{name}'"));
            }
        }
        for (i, n) in self.nodes.iter().enumerate() {
            if n.children.iter().any(|&c| c >= i) {
                return Err(format!("node {i}: children must precede their parent"));
            }
            let count = n.children.len();
            let valid = match &n.kind {
                Kind::Selector | Kind::Sequence | Kind::Steps | Kind::Parallel { .. } => count > 0,
                Kind::Invert | Kind::Succeed | Kind::Repeat { .. } | Kind::Subtree { .. } => {
                    count == 1
                }
                Kind::Cooldown { seconds } | Kind::Timeout { seconds } => {
                    count == 1 && nonnegative(*seconds)
                }
                Kind::Check(c) => count == 0 && check(c),
                Kind::Action(a) => count == 0 && action(a, self.routes.len()),
                Kind::Utility { inertia } => {
                    count > 0
                        && inertia.is_finite()
                        && n.children
                            .iter()
                            .all(|&c| matches!(self.nodes[c].kind, Kind::Option { .. }))
                }
                Kind::Option { scores } => {
                    count > 0
                        && !scores.is_empty()
                        && scores.iter().all(|s| {
                            s.constant.is_finite()
                                && s.lo.is_finite()
                                && s.hi.is_finite()
                                && s.operand.as_ref().is_none_or(|s| !s.is_empty())
                        })
                }
            };
            if !valid {
                return Err(format!(
                    "node {i} '{}': invalid children, indices or values",
                    n.name
                ));
            }
        }
        match &self.root {
            Root::Tree(root) if *root < self.nodes.len() => {}
            Root::Machine { states, initial } if *initial < states.len() => {
                for (i, s) in states.iter().enumerate() {
                    if s.name.is_empty()
                        || states[..i].iter().any(|p| p.name == s.name)
                        || s.body.is_some_and(|b| b >= self.nodes.len())
                        || s.enter.iter().chain(&s.exit).any(|a| {
                            !matches!(
                                a,
                                Action::Set { .. } | Action::Stop | Action::Command { .. }
                            ) || !action(a, self.routes.len())
                        })
                        || s.transitions.iter().any(|t| {
                            t.to >= states.len()
                                || !nonnegative(t.after)
                                || t.checks.iter().any(|c| !check(c))
                        })
                    {
                        return Err(format!(
                            "state '{}': invalid name, actions, body or transitions",
                            s.name
                        ));
                    }
                }
            }
            _ => return Err("invalid behaviour root".into()),
        }
        Ok(())
    }
}
