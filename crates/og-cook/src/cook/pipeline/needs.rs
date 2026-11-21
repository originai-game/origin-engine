//! The engine wasm modules each asset and the bundle need: the manifest's
//! `modules` lines (docs/web-export.md §Modules). A scene's or prefab's own
//! are its records' (the IDL's `split`: `RigidBody` is in `physics`); a
//! dialogue's, a video's and a navmesh's are their kind's; an asset needs
//! its own and those of every asset it references (parsed_refs: a scene's
//! sub-scenes, prefabs and models, a timeline's clips and scenes). The
//! bundle needs its boot scene's, or every asset's when the cook cannot
//! name that scene.
//!
//! Readers fetch them before the first call that would load one where it
//! stands: og build beside the primary, og.load.bundle as the bundle
//! opens, og.scene.preload, og.load.scene and a timeline's pre-roll as the
//! content is prepared (og-runtime's `world::needs`).

use super::*;
use og_core::ogt::Block;

/// The modules an asset of `kind` needs whatever it holds.
fn of_kind(kind: &str) -> &'static [&'static str] {
    match kind {
        "dialogue" | "quest" => &["story"],
        "video" => &["video"],
        "navmesh" => &["nav"],
        _ => &[],
    }
}

/// The modules a scene's or prefab's text needs by its own records: each
/// component block in its entities and their overrides, and each
/// `resource`, by the IDL's `split` (a game's own records need none).
fn of_records(text: &str) -> BTreeSet<&'static str> {
    fn split(name: &str) -> &'static [&'static str] {
        let name = name.strip_prefix("og.").unwrap_or(name);
        og_records::RECORDS
            .iter()
            .find(|r| r.name == name)
            .map_or(&[], |r| r.split)
    }
    fn walk(b: &Block, out: &mut BTreeSet<&'static str>) {
        for e in &b.entries {
            let Some(block) = &e.block else { continue };
            match e.key.as_str() {
                "entity" | "override" => walk(block, out),
                component => out.extend(split(component)),
            }
        }
    }
    let mut out = BTreeSet::new();
    let Ok(doc) = Document::parse(text) else {
        return out;
    };
    for e in &doc.body.entries {
        match (e.key.as_str(), &e.block) {
            ("entity", Some(b)) => walk(b, &mut out),
            ("resource", _) => {
                if let Some(r) = e
                    .values
                    .first()
                    .and_then(|v| v.as_word().or_else(|| v.as_str()))
                {
                    out.extend(split(r));
                }
            }
            _ => {}
        }
    }
    out
}

/// Writes each asset's `modules` and the bundle's into `manifest`.
pub(crate) fn record(
    manifest: &mut Manifest,
    sources: &[Source],
    parsed_refs: &BTreeMap<String, BTreeSet<String>>,
    boot: Option<&str>,
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
            if let Some(s) = by_rel.get(a.source.as_str())
                && [&scene::SCENE, &scene::PREFAB].contains(&s.kind)
            {
                m.extend(of_records(&String::from_utf8_lossy(&s.bytes)));
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
    let bundle: BTreeSet<&'static str> = match boot.and_then(|b| owner.get(b)) {
        Some(&i) => needs[i].clone(),
        None => needs.iter().flatten().copied().collect(),
    };
    manifest.modules = bundle.into_iter().map(str::to_string).collect();
    for (a, n) in manifest.assets.iter_mut().zip(needs) {
        a.modules = n.into_iter().map(str::to_string).collect();
    }
}
