//! The direction of the crate's dependencies (`README.md`, Layout): a
//! mechanism (`gpu/`, `shader/`, `frame/`, `visibility/`, `mesh/`,
//! `shading/`, `media/`, `post/`, `particles/`, `debug/`) names no world
//! system and not the renderer that composes them, and a world system
//! (`systems/<name>/`) names no other system and not the renderer. What
//! two systems share is a mechanism.
//!
//! It reads the sources: a file names a module with a `crate::` path, and
//! `lib.rs` says which directory each of those names lives in. An item
//! re-exported at the root by a glob (`crate::Renderer`) is not seen.
//!
//! [`KNOWN`] is what the tree named before the mechanisms were separated,
//! one line per file and the directory it reaches into. It only shrinks:
//! a line that no longer applies fails the test as well as a new one.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

const MECHANISMS: &[&str] = &[
    "gpu",
    "shader",
    "frame",
    "visibility",
    "mesh",
    "shading",
    "media",
    "post",
    "particles",
    "debug",
];

/// `(file under src/, the directory it names)`.
const KNOWN: &[(&str, &str)] = &[
    ("debug/id_pass.rs", "systems/sprites"),
    ("debug/id_pass.rs", "systems/terrain"),
    ("debug/id_pass.rs", "systems/vegetation"),
    ("frame/mod.rs", "systems/environment"),
    ("frame/mod.rs", "systems/vegetation"),
    ("frame/views.rs", "systems/sprites"),
    ("media/volumetric.rs", "renderer"),
    ("particles/particles_gpu.rs", "systems/sprites"),
    ("particles/particles_gpu/prepare.rs", "systems/sprites"),
    ("post/fsr.rs", "renderer"),
    ("post/glass.rs", "renderer"),
    ("post/graph.rs", "renderer"),
    ("post/passes.rs", "systems/stylise"),
    ("post/plan.rs", "systems/stylise"),
    ("shader/pipeline_features.rs", "systems/terrain"),
    ("shader/pipeline_features.rs", "systems/vegetation"),
    ("shader/pipeline_features.rs", "systems/water"),
    ("shader/shaders.rs", "systems/crowd"),
    ("shader/shaders.rs", "systems/destruction"),
    ("shader/shaders.rs", "systems/environment"),
    ("shader/shaders.rs", "systems/fluids"),
    ("shader/shaders.rs", "systems/softbody"),
    ("shader/shaders.rs", "systems/sprites"),
    ("shader/shaders.rs", "systems/stylise"),
    ("shader/shaders.rs", "systems/terrain"),
    ("shader/shaders.rs", "systems/ui"),
    ("shader/shaders.rs", "systems/vegetation"),
    ("shader/shaders.rs", "systems/water"),
    ("shading/ibl.rs", "renderer"),
    ("shading/ibl.rs", "systems/environment"),
    ("shading/probe_content.rs", "systems/terrain"),
    ("shading/probes.rs", "renderer"),
    ("shading/probes.rs", "systems/fluids"),
    ("shading/shadow_keep.rs", "renderer"),
    ("shading/ssr.rs", "renderer"),
    ("systems/environment/sky_capture.rs", "renderer"),
    ("systems/environment/wind_field.rs", "renderer"),
    ("systems/fluids/visual_fluid.rs", "systems/softbody"),
    ("systems/sprites/overlay.rs", "systems/ui"),
    ("systems/sprites/sprite_skin.rs", "renderer"),
    ("systems/sprites/tiles_gpu.rs", "renderer"),
    ("systems/terrain/mod.rs", "systems/vegetation"),
    ("systems/terrain/renderer.rs", "systems/vegetation"),
    ("systems/terrain/shading.rs", "systems/environment"),
    ("systems/ui/mass.rs", "systems/sprites"),
    ("systems/ui/pass.rs", "systems/sprites"),
    ("systems/vegetation/foliage.rs", "systems/environment"),
    ("systems/vegetation/grass.rs", "systems/environment"),
    ("systems/vegetation/grass.rs", "systems/terrain"),
    ("systems/vegetation/scatter.rs", "systems/crowd"),
    ("systems/vegetation/scatter.rs", "systems/terrain"),
    ("systems/vegetation/scatter/draw.rs", "systems/terrain"),
    ("systems/vegetation/scatter/placement.rs", "systems/terrain"),
    (
        "systems/vegetation/scatter/prepare.rs",
        "systems/environment",
    ),
    ("systems/vegetation/scatter/prepare.rs", "systems/terrain"),
    (
        "systems/vegetation/scatter/renderer.rs",
        "systems/environment",
    ),
    ("systems/vegetation/scatter_place.rs", "systems/environment"),
    ("systems/vegetation/scatter_place.rs", "systems/terrain"),
    ("systems/vegetation/trample.rs", "renderer"),
    ("systems/vegetation/trample.rs", "systems/environment"),
    ("visibility/meshlet_gpu.rs", "systems/environment"),
    ("visibility/meshlet_gpu.rs", "systems/vegetation"),
    ("visibility/meshlet_gpu/vis.rs", "systems/vegetation"),
];

/// The directory that owns a module path below the root: its first
/// segment, or `systems/<name>`.
fn owner(path: &[&str]) -> Option<String> {
    match path {
        ["systems", name, ..] => Some(format!("systems/{name}")),
        ["systems"] => None,
        [first, ..] => Some((*first).to_string()),
        [] => None,
    }
}

/// The source without its `//` comments, so a path in a doc line is not a
/// dependency.
fn code(text: &str) -> String {
    text.lines()
        .map(|l| l.find("//").map_or(l, |i| &l[..i]))
        .collect::<Vec<_>>()
        .join("\n")
}

fn ident(s: &str) -> &str {
    let end = s
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .unwrap_or(s.len());
    &s[..end]
}

/// The first segments a `use` group opens with: `{a::b, c, d::{e}}` gives
/// a, c, d. `s` starts after the `{`.
fn group(s: &str) -> Vec<&str> {
    let (mut depth, mut start, mut out) = (1, 0, Vec::new());
    for (i, c) in s.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => depth -= 1,
            _ => {}
        }
        if depth == 0 || (depth == 1 && c == ',') {
            out.push(ident(s[start..i].trim_start()));
            start = i + 1;
        }
        if depth == 0 {
            break;
        }
    }
    out.retain(|n| !n.is_empty());
    out
}

/// The names that follow `prefix` in `text`, through a `{...}` group too.
fn after<'t>(text: &'t str, prefix: &str) -> Vec<&'t str> {
    let mut out = Vec::new();
    for (i, _) in text.match_indices(prefix) {
        let before = text[..i].chars().next_back();
        if before.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_' || c == ':') {
            continue;
        }
        let rest = &text[i + prefix.len()..];
        match rest.strip_prefix('{') {
            Some(inner) => out.extend(group(inner)),
            None => out.push(ident(rest)),
        }
    }
    out
}

/// Where each name at the crate root lives: `mod frame;` is `frame`,
/// `use shading::ibl;` makes `ibl` a name for `shading`.
fn root_names(lib: &str) -> BTreeMap<String, String> {
    let mut names = BTreeMap::new();
    for line in code(lib).lines() {
        let line = line.trim();
        let line = line
            .strip_prefix("pub(crate) ")
            .or_else(|| line.strip_prefix("pub "))
            .unwrap_or(line);
        if let Some(name) = line.strip_prefix("mod ").and_then(|m| m.strip_suffix(';')) {
            if let Some(dir) = owner(&[name]) {
                names.insert(name.to_string(), dir);
            }
        } else if let Some(path) = line.strip_prefix("use ").and_then(|u| u.strip_suffix(';')) {
            let path: Vec<&str> = path.split("::").collect();
            let Some(name) = path.last().filter(|n| ident(n) == **n) else {
                continue;
            };
            if let Some(dir) = owner(&path) {
                names.insert((*name).to_string(), dir);
            }
        }
    }
    names
}

fn files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()) {
        if entry.is_dir() {
            files(&entry, out);
        } else if entry.extension().is_some_and(|e| e == "rs") {
            out.push(entry);
        }
    }
}

#[test]
fn a_mechanism_names_no_system_and_a_system_no_other() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let names = root_names(&fs::read_to_string(src.join("lib.rs")).unwrap());
    let mut all = Vec::new();
    files(&src, &mut all);

    let mut found = BTreeSet::new();
    for path in all {
        let file = path
            .strip_prefix(&src)
            .unwrap()
            .to_str()
            .unwrap()
            .replace('\\', "/");
        let parts: Vec<&str> = file.split('/').collect();
        let Some(from) = owner(&parts[..parts.len() - 1]) else {
            continue;
        };
        let system = from.starts_with("systems/");
        if !system && !MECHANISMS.contains(&from.as_str()) {
            continue;
        }
        let text = code(&fs::read_to_string(&path).unwrap());
        let mut reached = BTreeSet::new();
        for name in after(&text, "crate::") {
            if name == "systems" {
                continue;
            }
            reached.extend(names.get(name).cloned());
        }
        for name in after(&text, "crate::systems::") {
            reached.insert(format!("systems/{name}"));
        }
        for to in reached {
            let other_system = to.starts_with("systems/") && to != from;
            if other_system || to == "renderer" {
                found.insert((file.clone(), to));
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
        "og-render's dependency direction (tests/structure.rs).\n\
         New: a mechanism reaches into a system or the renderer, or a system into another. \
         Move what both need into a mechanism:\n{}\n\
         No longer there: delete these lines from KNOWN:\n{}",
        new.iter().map(|e| line(e)).collect::<Vec<_>>().join("\n"),
        gone.iter().map(|e| line(e)).collect::<Vec<_>>().join("\n"),
    );
}
