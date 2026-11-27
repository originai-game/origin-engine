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

/// The project's flow dialogues, the one project.ts's `flow` names first.
fn flows(root: &Path) -> Result<(Option<String>, Vec<Flow>), String> {
    let project = super::source_project::Project::open(root).map_err(|e| e.to_string())?;
    let bindings = super::project_assets(root, &project).map_err(|e| e.to_string())?;
    let declared = project
        .input
        .flow
        .as_ref()
        .map(|flow| {
            bindings
                .get(&flow.source.id)
                .filter(|output| output.kind == "dialogue")
                .map(|output| output.asset.clone())
                .ok_or_else(|| format!("project.ts: {} is not a scanned dialogue", flow.source.id))
        })
        .transpose()?;
    let functions = story_functions::read(&project.input.story_functions)?;
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
            "project.ts's flow names {id}, which is no dialogue with a flow node under assets/"
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
fn footage(root: &Path, id: &str, problems: &mut Vec<String>) -> Value {
    match super::video::read(root, id) {
        Ok((_, v)) => {
            if v.placeholder {
                problems.push(format!(
                    "its footage {id} is placeholder footage (`placeholder true`): replace the clip"
                ));
            }
            json!({
                "id": id,
                "exists": true,
                "source": v.source,
                "stream": v.stream,
                "placeholder": v.placeholder,
                "audioLocales": v.audio_locales.iter().map(|(t, _)| t.clone()).collect::<Vec<_>>(),
            })
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
            for &line in &n.dynamic_jumps {
                problems.push(format!(
                    "line {line}: uses dynamic jump (`<<jump {{...}}>>`), breaking static pre-warming"
                ));
            }
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
                        "loopRegion": s.loop_region.map(|(a, b)| json!({ "start": a, "end": b })),
                        "footage": s.video.as_ref().map(|(v, from)| {
                            let mut f = footage(root, v, &mut problems);
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
                        "group": c.group,
                        "line": c.key,
                        "default": c.tags.contains_key("default"),
                        "immediate": c.tags.contains_key("immediate"),
                        "outcome": c.tags.get("outcome"),
                        "qte": c.tags.get("qte"),
                        "hotspot": c.tags.get("hotspot"),
                        "to": flow_reach(&f.graph, &c.to),
                    })
                })
                .collect();
            nodes.push(json!({
                "node": n.title,
                "title": header("og-flow-title"),
                "segment": seg,
                "chapter": header("og-flow-chapter"),
                "ending": header("og-flow-ending"),
                "music": header("og-flow-music"),
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
/// the steps a test takes, without passing a node twice, plus detected
/// cycles and unreachable flow nodes.
pub fn routes(root: &Path) -> Result<Value, String> {
    let (declared, flows) = flows(root)?;
    let mut out = Vec::new();
    for f in &flows {
        let by: BTreeMap<&str, &GraphNode> =
            f.graph.iter().map(|n| (n.title.as_str(), n)).collect();
        let mut routes = Vec::new();
        let mut truncated = false;
        let mut stack: Vec<(Vec<Value>, Vec<&str>, &str)> =
            flow_reach(&f.graph, std::slice::from_ref(&f.start))
                .into_iter()
                .rev()
                .map(|s| (Vec::new(), vec![s], s))
                .collect();
        let mut reached = std::collections::BTreeSet::new();
        let mut visited_nodes = std::collections::BTreeSet::new();
        let mut cycles = std::collections::BTreeSet::new();
        while let Some((steps, seen, at)) = stack.pop() {
            visited_nodes.insert(at);
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
                        cycles.insert((at.to_string(), next.to_string()));
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
        let unreachable_nodes: Vec<&str> = f
            .graph
            .iter()
            .filter(|n| {
                n.headers.contains_key(FLOW_HEADER) && !visited_nodes.contains(n.title.as_str())
            })
            .map(|n| n.title.as_str())
            .collect();
        let cycle_json: Vec<Value> = cycles
            .into_iter()
            .map(|(from, to)| json!({ "from": from, "to": to }))
            .collect();
        out.push(json!({
            "dialogue": f.name,
            "file": f.rel,
            "declared": declared.as_deref() == Some(f.rel.as_str()),
            "routes": routes,
            "truncated": truncated,
            "unreached": unreached,
            "unreachableNodes": unreachable_nodes,
            "cycles": cycle_json,
        }));
    }
    Ok(json!({ "flows": out }))
}

/// Extracts a 64×36 RGB24 frame from `media` at `at_seconds` (if `Some`) or
/// the clip's last frame (if `None`).
fn sample_frame_rgb(media: &Path, at_seconds: Option<f64>) -> Result<Vec<u8>, String> {
    let mut cmd = std::process::Command::new(super::video::ffmpeg_bin());
    cmd.args(["-v", "error", "-nostdin"]);
    match at_seconds {
        Some(t) if t > 0.0 => {
            cmd.args(["-ss", &format!("{t:.3}")]);
        }
        Some(_) => {}
        None => {
            cmd.args(["-sseof", "-1.0"]);
        }
    }
    cmd.arg("-i").arg(media);
    cmd.args(["-map", "0:v:0", "-an", "-vf", "scale=64:36:flags=area"]);
    if at_seconds.is_some() {
        cmd.args(["-frames:v", "1"]);
    }
    cmd.args(["-f", "rawvideo", "-pix_fmt", "rgb24", "-"]);
    let out = cmd
        .output()
        .map_err(|e| format!("ffmpeg cannot sample {}: {e}", media.display()))?;
    if !out.status.success() {
        return Err(format!(
            "ffmpeg failed on {}: {}",
            media.display(),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let frame_bytes = 64 * 36 * 3;
    if out.stdout.len() < frame_bytes {
        if at_seconds.is_some() {
            return sample_frame_rgb(media, None);
        }
        return Err(format!("no frame decoded from {}", media.display()));
    }
    let start = out.stdout.len() - frame_bytes;
    Ok(out.stdout[start..].to_vec())
}

/// `og flow check-cuts`: measures visual continuity (mean luminance delta and
/// mean RGB pixel distance in `[0, 1]`) across every footage-to-footage cut
/// edge in the project's flows. Reports facts only; the caller decides.
pub fn check_cuts(root: &Path) -> Result<Value, String> {
    let (_, flows) = flows(root)?;
    let assets = root.join("assets");
    let mut out = Vec::new();
    for f in &flows {
        // Resolve each flow node's (media_path, from, cut) when it has footage.
        let mut media_of: BTreeMap<&str, (PathBuf, f64, f64)> = BTreeMap::new();
        for n in f
            .graph
            .iter()
            .filter(|n| n.headers.contains_key(FLOW_HEADER))
        {
            let seg_id = &n.headers[FLOW_HEADER];
            let Ok(t) = timeline(&assets, seg_id) else {
                continue;
            };
            let Ok(s) = t.segment() else { continue };
            let Some((vid_id, from)) = s.video else {
                continue;
            };
            let Ok((module, vdecl)) = super::video::read(root, &vid_id) else {
                continue;
            };
            media_of.insert(
                n.title.as_str(),
                (
                    module.join(&vdecl.source),
                    f64::from(from),
                    f64::from(from) + f64::from(s.cut),
                ),
            );
        }
        let mut frame_cache: BTreeMap<(PathBuf, i64), Result<Vec<u8>, String>> = BTreeMap::new();
        let mut sample = |path: &Path, t: f64| -> Result<Vec<u8>, String> {
            let key = (path.to_path_buf(), (t * 100.0).round() as i64);
            frame_cache
                .entry(key)
                .or_insert_with(|| sample_frame_rgb(path, Some(t.max(0.0))))
                .clone()
        };
        let mut cuts = Vec::new();
        let mut seen_edges = std::collections::BTreeSet::new();
        for n in f
            .graph
            .iter()
            .filter(|n| n.headers.contains_key(FLOW_HEADER))
        {
            let Some((from_path, _, from_end)) = media_of.get(n.title.as_str()) else {
                continue;
            };
            let targets: Vec<&str> = if n.choices.is_empty() {
                flow_reach(&f.graph, &n.to)
            } else {
                n.choices
                    .iter()
                    .flat_map(|c| flow_reach(&f.graph, &c.to))
                    .collect()
            };
            for to in targets {
                if !seen_edges.insert((n.title.as_str(), to)) {
                    continue;
                }
                let Some((to_path, to_start, _)) = media_of.get(to) else {
                    continue;
                };
                match (
                    sample(from_path, (from_end - 0.04).max(0.0)),
                    sample(to_path, *to_start),
                ) {
                    (Ok(a), Ok(b)) => {
                        let pixels = (a.len() / 3).max(1) as f64;
                        let mut rgb_diff = 0.0f64;
                        let mut luma_a = 0.0f64;
                        let mut luma_b = 0.0f64;
                        for (pa, pb) in a.as_chunks::<3>().0.iter().zip(b.as_chunks::<3>().0) {
                            let (ra, ga, ba) = (
                                f64::from(pa[0]) / 255.0,
                                f64::from(pa[1]) / 255.0,
                                f64::from(pa[2]) / 255.0,
                            );
                            let (rb, gb, bb) = (
                                f64::from(pb[0]) / 255.0,
                                f64::from(pb[1]) / 255.0,
                                f64::from(pb[2]) / 255.0,
                            );
                            rgb_diff += ((ra - rb).abs() + (ga - gb).abs() + (ba - bb).abs()) / 3.0;
                            luma_a += 0.2126 * ra + 0.7152 * ga + 0.0722 * ba;
                            luma_b += 0.2126 * rb + 0.7152 * gb + 0.0722 * bb;
                        }
                        cuts.push(json!({
                            "from": n.title,
                            "to": to,
                            "rgbDelta": (rgb_diff / pixels * 1000.0).round() / 1000.0,
                            "lumaDelta": ((luma_a - luma_b).abs() / pixels * 1000.0).round() / 1000.0,
                        }));
                    }
                    (Err(e), _) | (_, Err(e)) => {
                        cuts.push(json!({
                            "from": n.title,
                            "to": to,
                            "error": e,
                        }));
                    }
                }
            }
        }
        out.push(json!({
            "dialogue": f.name,
            "file": f.rel,
            "cuts": cuts,
        }));
    }
    Ok(json!({ "flows": out }))
}
