//! A dialogue's graph: its source nodes, their choices and where each may
//! lead, read from the compiled blocks (og.story.graph). Every branch of a
//! condition counts, so it is a superset of the routes a playthrough takes.

use crate::asset::{Dialogue, Node, Step};
use crate::expr::{Expr, Val};
use std::collections::{BTreeMap, BTreeSet};

/// A choice and the source nodes it may lead to.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphChoice {
    /// Its index in its option group (what `choose` takes).
    pub index: u32,
    pub id: String,
    pub key: String,
    pub tags: BTreeMap<String, String>,
    pub to: Vec<String>,
}

/// A source node: its headers, its choices and where its own steps jump.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphNode {
    pub title: String,
    pub headers: BTreeMap<String, String>,
    pub choices: Vec<GraphChoice>,
    pub to: Vec<String>,
}

impl Dialogue {
    /// Its source nodes in order, each with where it may lead.
    pub fn graph(&self) -> Vec<GraphNode> {
        let blocks: BTreeMap<&str, &Node> = self.nodes.iter().map(|n| (n.id.as_str(), n)).collect();
        self.nodes
            .iter()
            .filter(|n| n.id == n.title)
            .map(|n| {
                let mut choices = Vec::new();
                let mut seen = BTreeSet::new();
                let to = leads(&blocks, &n.title, &n.id, &mut seen, &mut |b| {
                    for (i, c) in b.choices.iter().enumerate() {
                        choices.push((i as u32, c));
                    }
                });
                let choices = choices
                    .into_iter()
                    .map(|(index, c)| {
                        let mut to = BTreeSet::new();
                        for start in c.goto.iter().chain(c.fail.iter()) {
                            to.extend(leads(
                                &blocks,
                                &n.title,
                                start,
                                &mut BTreeSet::new(),
                                &mut |_| {},
                            ));
                        }
                        GraphChoice {
                            index,
                            id: c.id.clone(),
                            key: c.text.key.clone(),
                            tags: c.tags.clone(),
                            to: to.into_iter().collect(),
                        }
                    })
                    .collect();
                GraphNode {
                    title: n.title.clone(),
                    headers: self.headers.get(&n.title).cloned().unwrap_or_default(),
                    choices,
                    to: to.into_iter().collect(),
                }
            })
            .collect()
    }
}

/// The node header that makes a node a flow node (og.flow plays its
/// segment and holds there).
pub const FLOW_HEADER: &str = "og-flow";

/// The flow nodes (those with an `og-flow` header) reachable from `to`
/// without passing another flow node: through logic nodes, every branch
/// of their conditions and choices counting. In the order a depth-first
/// walk meets them, each once. What og.flow warms and what `og flow`
/// lists as where a choice leads.
pub fn flow_reach<'a>(graph: &'a [GraphNode], to: &'a [String]) -> Vec<&'a str> {
    let by_title: BTreeMap<&str, &GraphNode> =
        graph.iter().map(|n| (n.title.as_str(), n)).collect();
    let mut out: Vec<&str> = Vec::new();
    let mut seen = BTreeSet::new();
    let mut stack: Vec<&str> = to.iter().rev().map(String::as_str).collect();
    while let Some(t) = stack.pop() {
        if !seen.insert(t) {
            continue;
        }
        let Some(n) = by_title.get(t) else { continue };
        if n.headers.contains_key(FLOW_HEADER) {
            if !out.contains(&n.title.as_str()) {
                out.push(n.title.as_str());
            }
            continue;
        }
        for c in n.choices.iter().rev() {
            stack.extend(c.to.iter().rev().map(String::as_str));
        }
        stack.extend(n.to.iter().rev().map(String::as_str));
    }
    out
}

impl Dialogue {
    /// The source lines of the lines node `title` says, its choices' bodies
    /// included (og.flow refuses them in a flow node).
    pub fn said_in(&self, title: &str) -> Vec<usize> {
        let mut out: Vec<usize> = self
            .nodes
            .iter()
            .filter(|b| b.title == title)
            .flat_map(|b| {
                b.steps
                    .iter()
                    .zip(&b.lines)
                    .filter(|(s, _)| matches!(s, Step::Say(_)))
                    .map(|(_, &l)| l)
            })
            .collect();
        out.sort_unstable();
        out
    }

    /// Where node `title` branches on the story's variables before its
    /// choices are taken: each `<<if>>` (and each line group's condition)
    /// its body reaches without taking a choice, and each choice's own
    /// condition, as (source line, the variables it reads), in line order.
    /// og.flow runs a flow node's body as its segment starts, so in a
    /// segment held for the game these are read before the game is played.
    pub fn branches_in(&self, title: &str) -> Vec<(usize, Vec<String>)> {
        let blocks: BTreeMap<&str, &Node> = self.nodes.iter().map(|n| (n.id.as_str(), n)).collect();
        let mut body: Vec<&Node> = Vec::new();
        leads(&blocks, title, title, &mut BTreeSet::new(), &mut |b| {
            body.push(b)
        });
        let mut out = Vec::new();
        let mut read = |line: usize, e: &Expr| {
            let mut vars = Vec::new();
            e.vars(&mut vars);
            vars.dedup();
            if !vars.is_empty() {
                out.push((line, vars));
            }
        };
        for b in body {
            for (s, &line) in b.steps.iter().zip(&b.lines) {
                match s {
                    Step::If(e, _) => read(line, e),
                    Step::Select(picks) => picks
                        .iter()
                        .filter_map(|(_, c, _)| c.as_ref())
                        .for_each(|c| read(line, c)),
                    _ => {}
                }
            }
            for c in &b.choices {
                if let Some(e) = &c.cond {
                    read(c.line, e);
                }
            }
        }
        out.sort_by_key(|(line, _)| *line);
        out
    }
}

/// The source nodes reachable from block `start` of node `title` without
/// taking a choice: its jumps, and the blocks of other nodes it reaches.
/// `group` sees each block of `title` it walks (for its option groups).
fn leads<'a>(
    blocks: &BTreeMap<&str, &'a Node>,
    title: &str,
    start: &str,
    seen: &mut BTreeSet<String>,
    group: &mut dyn FnMut(&'a Node),
) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut stack = vec![start.to_string()];
    while let Some(id) = stack.pop() {
        let Some(b) = blocks.get(id.as_str()) else {
            continue;
        };
        if b.title != title {
            out.insert(b.title.clone());
            continue;
        }
        if !seen.insert(id.clone()) {
            continue;
        }
        group(b);
        let mut next = |to: &str| stack.push(to.to_string());
        for s in &b.steps {
            match s {
                Step::Goto(t) | Step::If(_, t) => next(t),
                Step::Detour(t, cont) => {
                    next(t);
                    next(cont);
                }
                Step::DynamicDetour(x, cont) => {
                    jump(x, &mut out);
                    next(cont);
                }
                Step::Jump(x) => jump(x, &mut out),
                Step::Select(picks) => picks.iter().for_each(|(t, _, _)| next(t)),
                _ => {}
            }
        }
        if let Some(f) = &b.choice_fallback {
            next(f);
        }
    }
    out
}

/// A jump's destination when it is a literal title.
fn jump(x: &Expr, out: &mut BTreeSet<String>) {
    if let Expr::Lit(Val::Str(t)) = x {
        out.insert(t.clone());
    }
}

#[cfg(test)]
mod tests {
    use crate::yarn;

    #[test]
    fn choices_lead_through_bodies_and_conditions_and_steps_jump_on_their_own() {
        let d = yarn::compile(
            r#"#og-profile:yarn-spinner-3.2.2
#og-name:film
title: kitchen
og-flow: segments/kitchen.ogt
---
<<declare $brave = false>>
-> Take the knife #line:kitchen.knife #hotspot:knife
    <<set $brave = true>>
    <<jump hall>>
-> Open the door #line:kitchen.door #default
    <<if $brave>>
        <<jump hall>>
    <<else>>
        <<jump yard>>
    <<endif>>
===
title: hall
og-flow: segments/hall.ogt
---
<<jump ending>>
===
title: yard
---
<<jump ending>>
===
title: ending
og-flow-ending: brave
---
===
"#,
        )
        .unwrap();
        let g = d.graph();
        let titles: Vec<_> = g.iter().map(|n| n.title.as_str()).collect();
        assert_eq!(titles, ["kitchen", "hall", "yard", "ending"]);
        let k = &g[0];
        assert_eq!(k.headers["og-flow"], "segments/kitchen.ogt");
        assert!(
            k.to.is_empty(),
            "the node's own steps jump nowhere: {:?}",
            k.to
        );
        let leads: Vec<_> = k.choices.iter().map(|c| (c.index, c.to.clone())).collect();
        assert_eq!(
            leads,
            [
                (0, vec!["hall".to_string()]),
                (1, vec!["hall".to_string(), "yard".to_string()])
            ]
        );
        assert_eq!(k.choices[0].tags["hotspot"], "knife");
        assert_eq!(
            k.choices[1].tags.get("default").map(String::as_str),
            Some("")
        );
        assert_eq!(g[1].to, ["ending"]);
        assert_eq!(g[3].headers["og-flow-ending"], "brave");
    }
}
