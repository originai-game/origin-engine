//! A project's flows as facts, read from its sources (`og flow list` and
//! `og flow routes`): each dialogue with a flow node (an `og-flow` header),
//! its nodes with their segments, footage and scenes, and where each
//! choice leads; and the routes from its start to each ending. Where a
//! choice leads is og.flow's own rule (`og_story::graph::flow_reach`),
//! through logic nodes with every branch counting, so a route through a
//! condition depends on the story's variables. The engine reports; the
//! agent judges what is missing or wrong.

use super::systems::story_functions;
use og_story::graph::{FLOW_HEADER, GraphNode, flow_reach};
use og_timeline::{Timeline, Track};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The most routes `routes` lists per dialogue.
pub const MAX_ROUTES: usize = 10_000;

/// A compiled flow dialogue: its asset id, name, start and graph.
struct Flow {
    rel: String,
    name: String,
    start: String,
    graph: Vec<GraphNode>,
}

/// The project's flow dialogues, the one project.ogt's `flow` names first.
fn flows(root: &Path) -> Result<(Option<String>, Vec<Flow>), String> {
    let declared = std::fs::read_to_string(root.join("project.ogt"))
        .ok()
        .and_then(|t| og_core::ogt::Document::parse(&t).ok())
        .map(|d| og_bundle::look::project_flow(&d.body))
        .transpose()
        .map_err(|e| format!("project.ogt:{e}"))?
        .flatten()
        .map(|f| f.dialogue);
    let functions = story_functions::read(root)?;
    let assets = root.join("assets");
    let mut files = Vec::new();
    walk(&assets, &mut files);
    files.sort();
    let mut out = Vec::new();
    for file in files {
        let rel = rel_of(&assets, &file);
        let text = std::fs::read_to_string(&file).map_err(|e| format!("assets/{rel}: {e}"))?;
        let d = og_story::yarn::compile_with_functions(&text, functions.clone())
            .map_err(|e| format!("assets/{rel}: {e}"))?;
        let graph = d.graph();
        if graph.iter().any(|n| n.headers.contains_key(FLOW_HEADER)) {
            out.push(Flow {
                rel,
                name: d.name.clone(),
                start: d.start.clone(),
                graph,
            });
        }
    }
    if let Some(id) = &declared
        && !out.iter().any(|f| &f.rel == id)
    {
        return Err(format!(
            "project.ogt's `flow` names {id}, which is no dialogue with a flow node under assets/"
        ));
    }
    out.sort_by_key(|f| Some(&f.rel) != declared.as_ref());
    Ok((declared, out))
}

/// The `.yarn` files under `dir`, never following a directory symlink or
/// entering the engine's own directories.
fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let Ok(ty) = e.file_type() else { continue };
        let name = e.file_name();
        if ty.is_dir() {
            if ![".og", "cooked", "og-out", "node_modules"]
                .contains(&name.to_string_lossy().as_ref())
            {
                walk(&e.path(), out);
            }
        } else if ty.is_file() && e.path().extension().is_some_and(|x| x == "yarn") {
            out.push(e.path());
        }
    }
}

fn rel_of(assets: &Path, file: &Path) -> String {
    file.strip_prefix(assets)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/")
}

/// A segment's timeline, read from assets/ (its caption files beside it or
/// under assets/).
fn timeline(assets: &Path, id: &str) -> Result<Timeline, String> {
    let text = std::fs::read_to_string(assets.join(id))
        .map_err(|_| format!("its segment {id} is no file under assets/"))?;
    let dir = id.rsplit_once('/').map_or("", |(d, _)| d);
    let mut read = |f: &str| -> Result<String, String> {
        std::fs::read_to_string(assets.join(dir).join(f))
            .or_else(|_| std::fs::read_to_string(assets.join(f)))
            .map_err(|e| format!("{f}: {e}"))
    };
    Timeline::parse(&text, &mut read).map_err(|e| format!("segment {id}: {e}"))
}

/// A video asset's facts: whether it exists, its source file, and whether
/// it is still placeholder footage (`placeholder true`).
fn footage(assets: &Path, id: &str, problems: &mut Vec<String>) -> Value {
    let Ok(text) = std::fs::read_to_string(assets.join(id)) else {
        problems.push(format!("its footage {id} is no file under assets/"));
        return json!({ "id": id, "exists": false });
    };
    let exists = |p: &str| assets.join(p).is_file();
    match super::video::parse(id, &text, &exists) {
        Ok(v) => {
            if v.placeholder {
                problems.push(format!(
                    "its footage {id} is placeholder footage (`placeholder true`): replace the clip"
                ));
            }
            json!({ "id": id, "exists": true, "source": v.source, "stream": v.stream, "placeholder": v.placeholder })
        }
        Err(e) => {
            problems.push(format!("its footage {id}: {e}"));
            json!({ "id": id, "exists": true, "error": e })
        }
    }
}

/// `og flow list`: every flow dialogue and its nodes, as JSON.
pub fn list(root: &Path) -> Result<Value, String> {
    let (declared, flows) = flows(root)?;
    let assets = root.join("assets");
    let mut out = Vec::new();
    for f in &flows {
        let mut nodes = Vec::new();
        for n in f
            .graph
            .iter()
            .filter(|n| n.headers.contains_key(FLOW_HEADER))
        {
            let id = &n.headers[FLOW_HEADER];
            let header = |k: &str| n.headers.get(k).cloned();
            let mut problems = Vec::new();
            let mut seg = json!({ "id": id });
            match timeline(&assets, id).and_then(|t| t.segment().map(|s| (t, s))) {
                Ok((t, s)) => {
                    let scenes: Vec<Value> = t
                        .tracks
                        .iter()
                        .filter_map(|tr| match tr {
                            Track::Scene(keys) => Some(keys.iter()),
                            _ => None,
                        })
                        .flatten()
                        .filter(|k| !k.scene.is_empty())
                        .map(|k| {
                            let exists = assets.join(&k.scene).is_file();
                            if !exists {
                                problems.push(format!(
                                    "its scene {} is no file under assets/",
                                    k.scene
                                ));
                            }
                            json!({ "id": k.scene, "at": k.time, "exists": exists })
                        })
                        .collect();
                    let kind = match (s.video.is_some(), !scenes.is_empty()) {
                        (true, true) => "both",
                        (true, false) => "footage",
                        _ => "live",
                    };
                    seg = json!({
                        "id": id,
                        "kind": kind,
                        "duration": t.duration,
                        "window": { "open": s.open, "lock": s.lock, "cut": s.cut },
                        "hold": s.hold,
                        "footage": s.video.as_ref().map(|(v, from)| {
                            let mut f = footage(&assets, v, &mut problems);
                            f["from"] = json!(from);
                            f
                        }),
                        "scenes": scenes,
                        "hotspots": s.hotspots.iter().map(|h| h.id.clone()).collect::<std::collections::BTreeSet<_>>(),
                    });
                }
                Err(e) => problems.push(e),
            }
            let choices: Vec<Value> = n
                .choices
                .iter()
                .map(|c| {
                    json!({
                        "index": c.index,
                        "line": c.key,
                        "default": c.tags.contains_key("default"),
                        "outcome": c.tags.get("outcome"),
                        "qte": c.tags.get("qte"),
                        "hotspot": c.tags.get("hotspot"),
                        "to": flow_reach(&f.graph, &c.to),
                    })
                })
                .collect();
            nodes.push(json!({
                "node": n.title,
                "segment": seg,
                "chapter": header("og-flow-chapter"),
                "ending": header("og-flow-ending"),
                "dissolve": header("og-flow-dissolve").and_then(|h| og_timeline::segment::dissolve(&h).ok()),
                "choices": choices,
                "next": if n.choices.is_empty() { flow_reach(&f.graph, &n.to) } else { Vec::new() },
                "problems": problems,
            }));
        }
        out.push(json!({
            "dialogue": f.name,
            "file": f.rel,
            "declared": declared.as_deref() == Some(f.rel.as_str()),
            "start": flow_reach(&f.graph, std::slice::from_ref(&f.start)),
            "nodes": nodes,
        }));
    }
    Ok(json!({ "flows": out }))
}

/// A flow node's ways on: the step that takes each and the flow nodes it
/// may reach (more than one: through a condition).
fn ways<'a>(graph: &'a [GraphNode], n: &'a GraphNode) -> Vec<(Value, Vec<&'a str>)> {
    if n.choices.is_empty() {
        return vec![(json!({ "cut": true }), flow_reach(graph, &n.to))];
    }
    n.choices
        .iter()
        .map(|c| {
            let take = match c.tags.get("outcome") {
                Some(o) => json!({ "advance": o }),
                None => json!({ "choose": c.index }),
            };
            (take, flow_reach(graph, &c.to))
        })
        .collect()
}

/// `og flow routes`: per flow dialogue, every route from its start to an
/// ending (a node with `og-flow-ending`, or one that leads nowhere), as
/// the steps a test takes, without passing a node twice.
pub fn routes(root: &Path) -> Result<Value, String> {
    let (declared, flows) = flows(root)?;
    let mut out = Vec::new();
    for f in &flows {
        let by: BTreeMap<&str, &GraphNode> =
            f.graph.iter().map(|n| (n.title.as_str(), n)).collect();
        let mut routes = Vec::new();
        let mut truncated = false;
        // (path of (node, step, to), node to expand)
        let mut stack: Vec<(Vec<Value>, Vec<&str>, &str)> =
            flow_reach(&f.graph, std::slice::from_ref(&f.start))
                .into_iter()
                .rev()
                .map(|s| (Vec::new(), vec![s], s))
                .collect();
        let mut reached = std::collections::BTreeSet::new();
        while let Some((steps, seen, at)) = stack.pop() {
            if routes.len() >= MAX_ROUTES {
                truncated = true;
                break;
            }
            let Some(n) = by.get(at) else { continue };
            let ending = n.headers.get("og-flow-ending");
            let ways = ways(&f.graph, n);
            if ending.is_some() || ways.iter().all(|(_, to)| to.is_empty()) {
                reached.insert(at);
                routes.push(json!({ "ending": ending, "end": at, "nodes": seen, "steps": steps }));
                continue;
            }
            for (take, to) in ways.into_iter().rev() {
                let branches = to.len() > 1;
                for next in to.into_iter().rev() {
                    if seen.contains(&next) {
                        continue;
                    }
                    let mut s = steps.clone();
                    let mut step = json!({ "node": at, "to": next });
                    for (k, v) in take.as_object().into_iter().flatten() {
                        step[k] = v.clone();
                    }
                    if branches {
                        step["condition"] = json!(true);
                    }
                    s.push(step);
                    let mut seen = seen.clone();
                    seen.push(next);
                    stack.push((s, seen, next));
                }
            }
        }
        let endings: Vec<&String> = f
            .graph
            .iter()
            .filter_map(|n| n.headers.get("og-flow-ending"))
            .collect();
        let unreached: Vec<&String> = endings
            .iter()
            .copied()
            .filter(|e| {
                !f.graph.iter().any(|n| {
                    n.headers.get("og-flow-ending") == Some(e) && reached.contains(n.title.as_str())
                })
            })
            .collect();
        out.push(json!({
            "dialogue": f.name,
            "file": f.rel,
            "declared": declared.as_deref() == Some(f.rel.as_str()),
            "routes": routes,
            "truncated": truncated,
            "unreached": unreached,
        }));
    }
    Ok(json!({ "flows": out }))
}
