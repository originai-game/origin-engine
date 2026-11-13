//! The direction of the workspace's dependencies (`docs/design.md` §2.1,
//! Crate layers). [`LAYERS`] names every crate under `crates/`, lowest
//! layer first. A crate depends on a lower layer, or on its own layer;
//! in the domain layer it depends on no other domain, since what two
//! domains need of each other is wired in `og-runtime`. A crate missing
//! from the table fails the test, so a new crate is given its layer.
//!
//! It reads each `crates/*/Cargo.toml`: every `og-*` key in a
//! `dependencies` or `build-dependencies` table, target-specific tables
//! included. Dev-dependencies are not edges: a test may build a scene with
//! the cook.
//!
//! [`KNOWN`] is the edges that broke the rule when the table was written.
//! It only shrinks: a line that no longer applies fails the test as well
//! as a new one.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

const DOMAIN: &str = "domain";

/// `(layer, its crates)`, lowest first.
const LAYERS: &[(&str, &[&str])] = &[
    (
        // Math, hashes, noise, easing, reflection, `.ogt`; the ECS; lanes.
        "foundation",
        &["og-core", "og-core-derive", "og-ecs", "og-lanes"],
    ),
    (
        // No ECS and no systems: what the cook and the runtime both link.
        "data",
        &[
            "og-api",
            "og-blob",
            "og-bundle",
            "og-css",
            "og-draw",
            "og-env",
            "og-geom",
            "og-i18n",
            "og-idl",
            "og-physics-data",
            "og-records",
            "og-render-data",
            "og-tilemap",
            "og-timeline",
            "og-ui-model",
        ],
    ),
    (
        DOMAIN,
        &[
            "og-ai",
            "og-anim",
            "og-audio",
            "og-bake",
            "og-input",
            "og-net",
            "og-physics",
            "og-probe",
            "og-reactive",
            "og-render",
            "og-rive",
            "og-story",
            "og-text",
            "og-tween",
            "og-ui",
            "og-video",
            "og-video-wasm",
        ],
    ),
    ("assembly", &["og-runtime"]),
    ("tools", &["og-cli", "og-cook", "og-sccache-wrap"]),
    (
        "hosts",
        &[
            "og-host-node",
            "og-host-web",
            "og-tests",
            "og-tests-cooked",
            "og-tests-ui",
            "og-tests-web",
        ],
    ),
];

/// The domains made of more than one crate; every other domain crate is
/// a domain of its own.
const DOMAINS: &[&[&str]] = &[
    &["og-reactive", "og-rive", "og-text", "og-ui"],
    &["og-video", "og-video-wasm"],
];

/// `(crate, the crate it depends on)`.
const KNOWN: &[(&str, &str)] = &[
    ("og-probe", "og-input"),
    ("og-ui", "og-input"),
    ("og-ui", "og-probe"),
    ("og-ui", "og-tween"),
];

/// The `og-*` crates a manifest depends on outside its tests.
fn dependencies(manifest: &str) -> BTreeSet<String> {
    let name = |s: &str| {
        let end = s
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
            .unwrap_or(s.len());
        s[..end].to_string()
    };
    let (mut out, mut reads) = (BTreeSet::new(), false);
    for line in manifest.lines().map(str::trim) {
        if let Some(table) = line.strip_prefix('[') {
            let table = table.trim_end_matches(']');
            // `[dependencies.og-x]` names the crate in the header.
            let (table, key) = match table.rsplit_once(".og-") {
                Some((t, k)) if t.ends_with("dependencies") => (t, Some(format!("og-{k}"))),
                _ => (table, None),
            };
            reads = table.ends_with("dependencies") && !table.ends_with("dev-dependencies");
            if reads {
                out.extend(key);
            }
        } else if reads && line.starts_with("og-") {
            out.insert(name(line));
        } else if reads {
            // A renamed dependency: `x = { package = "og-y" }`.
            if let Some((_, rest)) = line.split_once("package = \"og-") {
                out.insert(format!("og-{}", name(rest)));
            }
        }
    }
    out
}

fn domain(name: &str) -> &[&str] {
    DOMAINS
        .iter()
        .find(|d| d.contains(&name))
        .copied()
        .unwrap_or(&[])
}

#[test]
fn a_crate_depends_downwards_and_a_domain_on_no_other() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let layer: BTreeMap<&str, usize> = LAYERS
        .iter()
        .enumerate()
        .flat_map(|(i, (_, names))| names.iter().map(move |n| (*n, i)))
        .collect();

    let mut manifests = BTreeMap::new();
    for dir in fs::read_dir(crates).unwrap().map(|e| e.unwrap().path()) {
        if let Ok(text) = fs::read_to_string(dir.join("Cargo.toml")) {
            let name = dir.file_name().unwrap().to_str().unwrap().to_string();
            manifests.insert(name, text);
        }
    }
    let unplaced: Vec<_> = manifests
        .keys()
        .filter(|n| !layer.contains_key(n.as_str()))
        .collect();
    let absent: Vec<_> = layer
        .keys()
        .filter(|n| !manifests.contains_key(**n))
        .collect();
    assert!(
        unplaced.is_empty() && absent.is_empty(),
        "the workspace's layers (crates/og-core/tests/layers.rs).\n\
         Not in LAYERS; give each its layer: {unplaced:?}\n\
         In LAYERS and not under crates/; delete: {absent:?}",
    );

    let mut found = BTreeSet::new();
    for (from, text) in &manifests {
        let at = layer[from.as_str()];
        for to in dependencies(text) {
            let Some(&below) = layer.get(to.as_str()) else {
                panic!("{from} depends on {to}, which is not under crates/");
            };
            let sideways = at == below
                && LAYERS[at].0 == DOMAIN
                && *from != to
                && !domain(from).contains(&to.as_str());
            if below > at || sideways {
                found.insert((from.clone(), to));
            }
        }
    }

    let known: BTreeSet<(String, String)> = KNOWN
        .iter()
        .map(|(f, t)| ((*f).to_string(), (*t).to_string()))
        .collect();
    let new: Vec<_> = found.difference(&known).collect();
    let gone: Vec<_> = known.difference(&found).collect();
    let line = |(f, t): &(String, String)| format!("    (\"{f}\", \"{t}\"),");
    assert!(
        new.is_empty() && gone.is_empty(),
        "the workspace's dependency direction (crates/og-core/tests/layers.rs).\n\
         New: a crate depends on a higher layer, or a domain on another domain. \
         Move what both need into a lower layer, or wire the two in og-runtime:\n{}\n\
         No longer there: delete these lines from KNOWN:\n{}",
        new.iter().map(|e| line(e)).collect::<Vec<_>>().join("\n"),
        gone.iter().map(|e| line(e)).collect::<Vec<_>>().join("\n"),
    );
}
