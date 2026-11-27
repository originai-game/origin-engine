//! What the game's code reaches, read from the bundler's import graph.
//!
//! The engine's bundler (`packages/engine/src/node/build.ts`) resolves every
//! import from the game's entry and writes the files it reached beside the
//! bundle (`<bundle>.graph.json`: the engine package's root and every file,
//! by absolute path). The cook takes that graph as an input
//! ([`super::CookOptions::code`]): the built-in UI a game draws, the string
//! tables of the engine components it imports, whether it raycasts and
//! whether it hyphenates are read from the modules the bundle holds, never from the files that happen
//! to sit in the project.
use super::ts_refs;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// The cook's reading of the bundler's graph (see the module doc).
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CodeGraph {
    /// The code imports the engine's TSX UI (a module under the engine's
    /// `src/ui/`), which draws with the default face and the default pack's
    /// images.
    pub tsx: bool,
    /// The engine declarations the bundle holds, by the id the bundle names
    /// them by (`@ui/components/hud/bar.en.strings.ts`), with the file each is
    /// read from: its components' string tables, and the emitters they
    /// draw (`@ui/components/hud/fx/sparks.ogt`, a `type particles`).
    pub tables: BTreeMap<String, PathBuf>,
    /// The built-in `@ui/` documents and `@styles/` files the code names by
    /// id.
    pub ids: BTreeSet<String>,
    /// The code reads `raycast` (og.render.raycast): its models carry a
    /// cooked raycast BVH.
    pub raycast: bool,
    /// A string the code writes holds a `hyphens-auto` class (under any
    /// variant): its text hyphenates, so its locales' hyphenation
    /// dictionaries ship ([`hyphens_auto`]).
    pub hyphens: bool,
    /// The images the code's `nine-<name>` classes name, by name: the
    /// default pack's images that join the bundle ([`nine_names`]).
    #[serde(default)]
    pub nine: BTreeSet<String>,
    /// The game's own code the bundle holds (its modules outside the
    /// engine), by absolute path: what the asset index reads for the
    /// assets the code loads ([`super::assets::add_code`]).
    pub files: Vec<PathBuf>,
}

/// The id a bundled engine string table registers under: its path under
/// the engine's `src/ui/`, as build.ts names it.
const TABLE_PREFIX: &str = "@ui/";

impl CodeGraph {
    /// The graph the bundler wrote at `path` for the project at `project`.
    ///
    /// A file is the engine's when it sits under the engine package, unless
    /// the project sits there too and the file is the project's: the deeper
    /// root owns it. A game in the engine's checkout (its test fixtures)
    /// keeps its code, and an installed engine
    /// (`<project>/node_modules/@origingame/engine`) stays the engine's.
    pub fn read(path: &Path, project: &Path) -> Result<CodeGraph, String> {
        #[derive(serde::Deserialize)]
        struct Reached {
            engine: PathBuf,
            files: Vec<PathBuf>,
        }
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let reached: Reached =
            serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        let canonical = |p: &Path| {
            p.canonicalize()
                .map_err(|e| format!("{}: {e}", p.display()))
        };
        let (root, engine) = (canonical(project)?, canonical(&reached.engine)?);
        let in_project = root.starts_with(&engine);
        let engine_code = |file: &Path| {
            file.starts_with(&reached.engine)
                && !(in_project && (file.starts_with(&root) || file.starts_with(project)))
        };
        let ui = reached.engine.join("src").join("ui");
        let mut graph = CodeGraph::default();
        for file in reached.files {
            if let Ok(rel) = file.strip_prefix(&ui) {
                graph.tsx = true;
                if rel.extension().is_some_and(|e| e == "ogt")
                    || rel.to_string_lossy().ends_with(".strings.ts")
                {
                    let id = format!("{TABLE_PREFIX}{}", rel.to_string_lossy().replace('\\', "/"));
                    graph.tables.insert(id, file);
                } else if rel.extension().is_some_and(|e| e == "tsx" || e == "ts")
                    && let Ok(text) = std::fs::read_to_string(&file)
                    && let Ok(refs) = ts_refs::parse(&file, &text)
                {
                    // The engine's components write classes too.
                    graph.hyphens |= hyphens_auto(&refs);
                    graph.nine.extend(nine_names(strings(&refs)));
                }
                continue;
            }
            if engine_code(&file)
                || !matches!(
                    file.extension().and_then(|e| e.to_str()),
                    Some("ts" | "tsx" | "js" | "jsx" | "mts" | "mjs" | "cts" | "cjs")
                )
            {
                continue;
            }
            let text =
                std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
            // Code that does not parse does not bundle either.
            let refs = ts_refs::parse(&file, &text)?;
            graph.raycast |= refs.names.contains("raycast")
                || refs.literals.iter().any(|l| l.value == "raycast");
            graph.hyphens |= hyphens_auto(&refs);
            graph.nine.extend(nine_names(strings(&refs)));
            graph.ids.extend(
                refs.literals
                    .into_iter()
                    .map(|l| l.value)
                    .filter(|v| v.starts_with(TABLE_PREFIX) || v.starts_with("@styles/")),
            );
            graph.files.push(file);
        }
        Ok(graph)
    }
}

/// Whether a string `refs` holds writes a `hyphens-auto` class: a
/// whitespace-separated token whose utility, after its variants
/// (`md:hyphens-auto`) and an important mark, is `hyphens-auto`.
fn hyphens_auto(refs: &ts_refs::References) -> bool {
    strings(refs)
        .flat_map(str::split_ascii_whitespace)
        .any(|class| {
            let utility = class.rsplit(':').next().unwrap_or(class);
            utility.trim_matches('!') == "hyphens-auto"
        })
}

/// The strings the code writes: its literals and its templates' text.
fn strings(refs: &ts_refs::References) -> impl Iterator<Item = &str> {
    refs.literals
        .iter()
        .map(|l| l.value.as_str())
        .chain(refs.fragments.iter().map(String::as_str))
}

/// The image names of the `nine-<name>` classes in `texts` (code's strings
/// or a pack.json), after a class's variants (`hover:nine-plate`) and
/// important mark. A name is matched as a prefix ([`names_image`]), so a
/// class cut short by a substitution (`nine-${art}` writes `nine-`) keeps
/// every image it may name.
pub fn nine_names<'a>(texts: impl Iterator<Item = &'a str>) -> BTreeSet<String> {
    texts
        .flat_map(|t| t.split(|c: char| c.is_ascii_whitespace() || c == '"'))
        .filter_map(|class| {
            let utility = class.rsplit(':').next().unwrap_or(class);
            utility
                .trim_matches('!')
                .strip_prefix("nine-")
                .map(str::to_owned)
        })
        .collect()
}

/// Whether the image `name` may be what one of `nine` names.
pub fn names_image(nine: &BTreeSet<String>, name: &str) -> bool {
    nine.iter().any(|n| name.starts_with(n.as_str()))
}
