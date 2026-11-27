//! The engine wasm modules each asset and the bundle need: the manifest's
//! `modules` lines (docs/web-export.md §Modules). A scene's or prefab's own
//! are its records' (the IDL's `split`: `RigidBody` is in `physics`); a
//! dialogue's, a video's and a navmesh's are their kind's; an asset needs
//! its own and those of every asset it references (parsed_refs: a scene's
//! sub-scenes, prefabs and models, a timeline's clips and scenes, a flow
//! dialogue's segments). Physics
//! is two modules: `physics`, which every physics record's split names, and
//! the simulation of the world's profile, `physics3d` or `physics2d`: the
//! scene's own `resource og.Physics { profile }`, else the project's. The
//! bundle needs its boot scene's and its project.ogt `flow` dialogue's, or
//! every asset's when the cook cannot name that scene.
//!
//! Readers fetch them before the first call that would load one where it
//! stands: og build beside the primary, og.load.bundle as the bundle
//! opens, og.scene.preload, og.load.scene and a timeline's pre-roll as the
//! content is prepared (og-runtime's `world::needs`).

use super::*;
use crate::look::Profile;

/// The modules an asset of `kind` needs whatever it holds.
fn of_kind(kind: &str) -> &'static [&'static str] {
    match kind {
        "dialogue" | "quest" => &["story"],
        "video" => &["video"],
        "navmesh" | "behaviour" => &["nav"],
        "tilemap" | "tileset" => &["tilemap"],
        "timeline" => &["timeline"],
        "terrain" => &["world3d"],
        "motion-action" => &["motion"],
        _ => &[],
    }
}

/// The simulation module of a physics profile (og.Physics.profile: 2, the
/// IDL's PROFILE_2D, is rapier2d; anything else rapier3d, whose step answers
/// a bad value).
fn simulation(profile: u64) -> &'static str {
    if profile == 2 {
        "physics2d"
    } else {
        "physics3d"
    }
}

/// The modules a checked scene/prefab needs by its own records, including
/// component overrides and resources, by the IDL's `split`; with
/// `physics`, the simulation of its `og.Physics` profile, else `project`'s.
fn of_records(doc: &scene::Document, project: Profile) -> BTreeSet<&'static str> {
    fn split(name: &str) -> &'static [&'static str] {
        let name = name.strip_prefix("og.").unwrap_or(name);
        og_records::RECORDS
            .iter()
            .find(|r| r.name == name)
            .map_or(&[], |r| r.split)
    }
    fn walk(e: &source_schemas::world::scene::EntityInput, out: &mut BTreeSet<&'static str>) {
        for name in e.components.keys() {
            out.extend(split(name));
        }
        for edit in &e.edits {
            if let source_schemas::world::scene::PrefabEdit::Override { components, .. } = edit {
                for name in components.keys() {
                    out.extend(split(name));
                }
            }
        }
        for child in &e.children {
            walk(child, out);
        }
    }
    let mut out = BTreeSet::new();
    let mut profile = match project {
        Profile::TwoD => 2,
        Profile::ThreeD => 3,
    };
    for e in &doc.scene.entities {
        walk(e, &mut out);
    }
    for (name, fields) in &doc.scene.resources {
        out.extend(split(name));
        if name == "og.Physics"
            && let Some(p) = fields.get("profile").and_then(serde_json::Value::as_u64)
        {
            profile = p;
        }
    }
    if out.contains("physics") {
        out.insert(simulation(profile));
    }
    if out.remove("destruction") && !out.contains("destruction2d") && !out.contains("destruction3d")
    {
        out.insert(if profile == 2 {
            "destruction2d"
        } else {
            "destruction3d"
        });
    }
    out
}

/// Writes each asset's `modules` and the bundle's into `manifest`.
pub(crate) fn record(
    manifest: &mut Manifest,
    sources: &[Source],
    parsed_refs: &BTreeMap<String, BTreeSet<String>>,
    boot: Option<&str>,
    flow: Option<&str>,
    profile: Profile,
) {
    let by_rel: BTreeMap<&str, &Source> = sources.iter().map(|s| (s.rel.as_str(), s)).collect();
    // Every output id (`models/ship.glb#tex0`) to the asset that made it.
    let owner: BTreeMap<&str, usize> = manifest
        .assets
        .iter()
        .enumerate()
        .flat_map(|(i, a)| {
            std::iter::once((a.source.as_str(), i))
                .chain(a.outputs.iter().map(move |o| (o.id.as_str(), i)))
        })
        .collect();
    let own: Vec<BTreeSet<&'static str>> = manifest
        .assets
        .iter()
        .map(|a| {
            let mut m: BTreeSet<&'static str> = of_kind(&a.kind).iter().copied().collect();
            if let Some(doc) = by_rel.get(a.source.as_str()).and_then(|s| s.scene.as_ref()) {
                m.extend(of_records(doc, profile));
            }
            m
        })
        .collect();
    let edges: Vec<Vec<usize>> = manifest
        .assets
        .iter()
        .enumerate()
        .map(|(i, a)| {
            parsed_refs
                .get(&a.source)
                .into_iter()
                .flatten()
                .filter_map(|r| owner.get(crate::runtime::asset_id(r)).copied())
                .filter(|&j| j != i)
                .collect()
        })
        .collect();
    // Each asset's needs: its own and every asset it reaches.
    let needs: Vec<BTreeSet<&'static str>> = (0..manifest.assets.len())
        .map(|start| {
            if edges[start].is_empty() {
                return own[start].clone();
            }
            let mut seen = BTreeSet::from([start]);
            let mut stack = vec![start];
            let mut out = BTreeSet::new();
            while let Some(i) = stack.pop() {
                out.extend(&own[i]);
                stack.extend(edges[i].iter().filter(|&&j| seen.insert(j)));
            }
            out
        })
        .collect();
    let flow_boot_modules = |flow_id: &str| -> BTreeSet<&'static str> {
        let mut m = BTreeSet::from(["story", "video"]);
        let Some(s) = by_rel.get(flow_id) else {
            return m;
        };
        let Ok(text) = std::str::from_utf8(&s.bytes) else {
            return m;
        };
        if let Ok(d) = og_story::yarn::compile(text) {
            let graph = d.graph();
            let roots = og_story::graph::flow_reach(&graph, std::slice::from_ref(&d.start));
            for r in roots {
                if let Some(seg) = graph
                    .iter()
                    .find(|n| n.title == r)
                    .and_then(|n| n.headers.get(og_story::graph::FLOW_HEADER))
                    && let Some(&idx) = owner.get(crate::runtime::asset_id(seg))
                {
                    m.extend(&needs[idx]);
                }
            }
        } else if let Some(&f) = owner.get(flow_id) {
            m.extend(&needs[f]);
        }
        m
    };
    let bundle: BTreeSet<&'static str> = match (boot.and_then(|b| owner.get(b)), flow) {
        (Some(&i), Some(f)) => &needs[i] | &flow_boot_modules(f),
        (Some(&i), None) => needs[i].clone(),
        (None, Some(f)) => flow_boot_modules(f),
        (None, None) => needs.iter().flatten().copied().collect(),
    };
    manifest.modules = bundle.into_iter().map(str::to_string).collect();
    for (a, n) in manifest.assets.iter_mut().zip(needs) {
        a.modules = n.into_iter().map(str::to_string).collect();
        a.dependencies = parsed_refs
            .get(&a.source)
            .into_iter()
            .flatten()
            .cloned()
            .collect();
    }
}
