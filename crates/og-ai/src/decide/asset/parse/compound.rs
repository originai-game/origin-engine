//! The nodes made of others: utility options, subtrees (`use`) and state
//! machines.

use super::*;

impl Parser<'_, '_> {
    /// `option <name> { score ...; nodes }`.
    pub(super) fn option(&mut self, e: &Entry) -> Result<usize, String> {
        let label = e
            .values
            .first()
            .and_then(word_or_string)
            .ok_or_else(|| err(e, "option takes a name: option attack { ... }"))?
            .to_string();
        let b = e
            .block
            .as_ref()
            .ok_or_else(|| err(e, "option needs a block: its scores, then its nodes"))?;
        let mut scores = Vec::new();
        let mut children = Vec::new();
        for c in &b.entries {
            if c.key == "score" {
                scores.push(parse_consideration(c)?);
            } else {
                children.push(self.node(c)?);
            }
        }
        if scores.is_empty() {
            return Err(err(e, format!("option {label} needs a score line")));
        }
        if children.is_empty() {
            return Err(err(e, format!("option {label} needs a node to run")));
        }
        self.nodes.push(Node {
            kind: Kind::Option { scores },
            name: format!("option {label}"),
            children,
            line: e.line,
        });
        Ok(self.nodes.len() - 1)
    }

    /// `use "<path>"`: the behaviour at `path` (a tree) inlined, its keys
    /// and routes joined to this one's.
    pub(super) fn subtree(&mut self, e: &Entry) -> Result<usize, String> {
        check_options(e, &[])?;
        let path = e
            .values
            .first()
            .and_then(word_or_string)
            .ok_or_else(|| err(e, "use takes a behaviour's path: use \"ai/patrol.ogt\""))?
            .to_string();
        let chain = |c: &[String]| {
            c.iter()
                .chain(std::iter::once(&path))
                .map(|p| {
                    if p.is_empty() {
                        "(this behaviour)"
                    } else {
                        p.as_str()
                    }
                })
                .collect::<Vec<_>>()
                .join(" → ")
        };
        if self.chain.contains(&path) {
            return Err(err(e, format!("use makes a cycle: {}", chain(&self.chain))));
        }
        let Some(resolve) = self.resolve.as_deref_mut() else {
            return Err(err(
                e,
                "use reads another behaviour asset: cook it (og cook) and load it with og.ai.load",
            ));
        };
        let text = resolve(&path).map_err(|m| err(e, format!("use \"{path}\": {m}")))?;
        let mut sub_chain = self.chain.clone();
        sub_chain.push(path.clone());
        let sub = Behaviour::parse_inner(&text, Some(resolve), sub_chain).map_err(|m| {
            if m.contains("use makes a cycle") {
                m
            } else {
                format!("{path}: {m}")
            }
        })?;
        let Root::Tree(root) = sub.root else {
            return Err(err(e, format!("{path} is a machine; use takes a tree")));
        };
        // Its routes, by name: the same name must be the same points.
        let mut remap = Vec::new();
        for (name, pts) in sub.routes {
            match self.routes.iter().position(|(n, _)| *n == name) {
                Some(k) if self.routes[k].1 == pts => remap.push(k),
                Some(_) => {
                    return Err(err(
                        e,
                        format!(
                            "{path}'s route '{name}' differs from this behaviour's route of that name"
                        ),
                    ));
                }
                None => {
                    self.routes.push((name, pts));
                    remap.push(self.routes.len() - 1);
                }
            }
        }
        for (k, v) in sub.keys {
            if !self.keys.iter().any(|(n, _)| *n == k) {
                self.keys.push((k, v));
            }
        }
        let offset = self.nodes.len();
        for mut n in sub.nodes {
            for c in &mut n.children {
                *c += offset;
            }
            if let Kind::Action(Action::Patrol { route, .. }) = &mut n.kind {
                *route = remap[*route];
            }
            self.nodes.push(n);
        }
        for p in std::iter::once(path.clone()).chain(sub.subtrees) {
            if !self.subtrees.contains(&p) {
                self.subtrees.push(p);
            }
        }
        self.nodes.push(Node {
            kind: Kind::Subtree { path: path.clone() },
            name: format!("use {path}"),
            children: vec![root + offset],
            line: e.line,
        });
        Ok(self.nodes.len() - 1)
    }

    pub(super) fn machine(&mut self, m: &Entry) -> Result<Root, String> {
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
                    "entry" | "exit" => {
                        let list = if c.key == "entry" {
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
