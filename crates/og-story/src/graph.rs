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

impl Dialogue {
    /// The source lines of the lines node `title` says, its choices' bodies
    /// included (og.film refuses them in a film node).
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
og-film: segments/kitchen.ogt
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
og-film: segments/hall.ogt
---
<<jump ending>>
===
title: yard
---
<<jump ending>>
===
title: ending
og-film-ending: brave
---
===
"#,
        )
        .unwrap();
        let g = d.graph();
        let titles: Vec<_> = g.iter().map(|n| n.title.as_str()).collect();
        assert_eq!(titles, ["kitchen", "hall", "yard", "ending"]);
        let k = &g[0];
        assert_eq!(k.headers["og-film"], "segments/kitchen.ogt");
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
        assert_eq!(g[3].headers["og-film-ending"], "brave");
    }
}
