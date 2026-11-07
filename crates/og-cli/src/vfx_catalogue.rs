//! The effect-pack catalogue: every vfx pack the engine carries
//! (`packages/engine/extensions/<pack>/`, bundled) or releases
//! (`packages/vfx-packs/<pack>/`, downloadable), with its description,
//! tags, keywords and effects, as one guide
//! (`crates/og-api/guides/effect-packs.md`) that `og api gen` writes from
//! the packs' extension.json and assets.json and `og api check` holds equal,
//! so `og api find fire explosion` answers with the packs that have them.

use std::collections::BTreeMap;
use std::fmt::Write;
use std::path::Path;

use serde_json::Value;

/// Where the catalogue lives, from the repository root.
pub const PATH: &str = "crates/og-api/guides/effect-packs.md";

/// The pack roots, from the repository root, and how a game gets a pack there.
const ROOTS: [(&str, &str); 2] = [
    ("packages/engine/extensions", "bundled with the engine"),
    (
        "packages/vfx-packs",
        "downloadable: a release's `vfx/<pack>.tar.gz`",
    ),
];

fn json(path: &Path) -> Option<Value> {
    serde_json::from_slice(&std::fs::read(path).ok()?).ok()
}

fn words(v: &Value) -> String {
    v.as_array()
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default()
}

fn cell(s: &str) -> String {
    s.replace('|', "\\|")
}

/// The catalogue's text for the packs under `root`.
pub fn markdown(root: &Path) -> String {
    let mut out = String::from(
        "<!-- Generated from the vfx packs' extension.json and assets.json (packages/engine/extensions/, packages/vfx-packs/) by `og api gen`. Do not edit: `og api check` fails on any difference. -->\n\
# The effect packs\n\n\
Ready-made effects a game installs with `og vfx add <pack>`: converted Effekseer samples (`type vfx`, 3D) and pixel-art recipes (`type vfx2d`, 2D), each listed with its tags, keywords and effects. A bundled pack ships with the engine; a downloadable one comes with each release and `og vfx add` fetches it. Once installed, an effect spawns as `vfx/<pack>/<effect>` (`og api show vfx`; the `pixel-vfx` guide for `type vfx2d`); the pack format is docs/vfx-packs.md.\n",
    );
    let mut n = 0;
    // Category -> pack -> (effect path, title): a section per category after
    // the packs', short enough for its words to rank (a pack's section is a
    // long table where one category is a few cells of many).
    let mut by_category: BTreeMap<String, BTreeMap<String, Vec<(String, String)>>> =
        BTreeMap::new();
    for (dir, how) in ROOTS {
        let mut packs: Vec<_> = std::fs::read_dir(root.join(dir))
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .collect();
        packs.sort();
        for pack in packs {
            let Some(ext) = json(&pack.join("extension.json")) else {
                continue;
            };
            if ext["kind"] != "vfx" {
                continue;
            }
            let name = ext["name"].as_str().unwrap_or_default();
            let effects = json(&pack.join("assets.json"))
                .and_then(|a| a["effects"].as_array().cloned())
                .unwrap_or_default();
            n += 1;
            let tags = &ext["tags"];
            let _ = writeln!(
                out,
                "\n## {n}. {name}\n\n{}. {} effects, {how}: `og vfx add {name}`.\n",
                ext["description"]
                    .as_str()
                    .unwrap_or_default()
                    .trim_end_matches('.'),
                effects.len()
            );
            let _ = writeln!(
                out,
                "- Style: {}, {}\n- Categories: {}\n- Elements: {}\n- Keywords: {}\n",
                tags["style"].as_str().unwrap_or_default(),
                tags["dim"].as_str().unwrap_or_default(),
                words(&tags["category"]),
                words(&tags["element"]),
                words(&ext["keywords"])
            );
            out.push_str(
                "| Effect | Title | Category | Element | Seconds |\n|---|---|---|---|---|\n",
            );
            for e in &effects {
                if let Some(c) = e["category"].as_str().filter(|c| !c.is_empty()) {
                    by_category
                        .entry(c.to_string())
                        .or_default()
                        .entry(name.to_string())
                        .or_default()
                        .push((
                            e["path"].as_str().unwrap_or_default().to_string(),
                            e["title"].as_str().unwrap_or_default().to_string(),
                        ));
                }
                let secs = if e["loop"] == true {
                    "loops".to_string()
                } else {
                    format!("{}", e["durationSeconds"].as_f64().unwrap_or(0.0))
                };
                let _ = writeln!(
                    out,
                    "| `vfx/{name}/{}` | {} | {} | {} | {secs} |",
                    cell(e["path"].as_str().unwrap_or_default()),
                    cell(e["title"].as_str().unwrap_or_default()),
                    e["category"].as_str().unwrap_or_default(),
                    e["element"].as_str().unwrap_or_default(),
                );
            }
        }
    }
    for (category, packs) in &by_category {
        n += 1;
        // `level-up` reads as the words a game asks for: level up.
        let words = category.replace('-', " ");
        let count: usize = packs.values().map(Vec::len).sum();
        let mut head = words.clone();
        head[..1].make_ascii_uppercase();
        let _ = writeln!(
            out,
            "\n## {n}. {head} effects\n\n{head} effects ({category}): {count} in {}.\n",
            packs
                .keys()
                .map(|p| format!("`{p}`"))
                .collect::<Vec<_>>()
                .join(", ")
        );
        for (pack, effects) in packs {
            let list = effects
                .iter()
                .map(|(path, title)| format!("`vfx/{pack}/{path}` ({title})"))
                .collect::<Vec<_>>()
                .join(", ");
            let _ = writeln!(out, "- {pack}: {list}");
        }
    }
    out
}
