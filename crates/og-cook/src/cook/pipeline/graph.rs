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
    /// The engine string tables the bundle holds, by the id the bundle
    /// registers them under (`@ui/components/hud/bar.en.ogt`), with the
    /// file each is read from.
    pub tables: BTreeMap<String, PathBuf>,
    /// The built-in `@ui/` documents the code names by id.
    pub ids: BTreeSet<String>,
    /// The code reads `raycast` (og.render.raycast): its models carry a
    /// cooked raycast BVH.
    pub raycast: bool,
    /// A string the code writes holds a `hyphens-auto` class (under any
    /// variant): its text hyphenates, so its locales' hyphenation
    /// dictionaries ship ([`hyphens_auto`]).
    pub hyphens: bool,
    /// The game's own code the bundle holds (its modules outside the
    /// engine), by absolute path: what the asset index reads for the
    /// assets the code loads ([`super::assets::add_code`]).
    pub files: Vec<PathBuf>,
}

/// The id a bundled engine string table registers under: its path under
/// the engine's `src/ui/`, as build.ts names it.
const TABLE_PREFIX: &str = "@ui/";

impl CodeGraph {
    /// The graph the bundler wrote at `path`.
    pub fn read(path: &Path) -> Result<CodeGraph, String> {
        #[derive(serde::Deserialize)]
        struct Reached {
            engine: PathBuf,
            files: Vec<PathBuf>,
        }
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let reached: Reached =
            serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        let ui = reached.engine.join("src").join("ui");
        let mut graph = CodeGraph::default();
        for file in reached.files {
            if let Ok(rel) = file.strip_prefix(&ui) {
                graph.tsx = true;
                if rel.extension().is_some_and(|e| e == "ogt") {
                    let id = format!("{TABLE_PREFIX}{}", rel.to_string_lossy().replace('\\', "/"));
                    graph.tables.insert(id, file);
                } else if !graph.hyphens
                    && rel.extension().is_some_and(|e| e == "tsx" || e == "ts")
                    && let Ok(text) = std::fs::read_to_string(&file)
                    && let Ok(refs) = ts_refs::parse(&file, &text)
                {
                    // The engine's components write classes too.
                    graph.hyphens |= hyphens_auto(&refs);
                }
                continue;
            }
            if file.starts_with(&reached.engine)
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
            graph.ids.extend(
                refs.literals
                    .into_iter()
                    .map(|l| l.value)
                    .filter(|v| v.starts_with(TABLE_PREFIX)),
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
    refs.literals
        .iter()
        .map(|l| l.value.as_str())
        .chain(refs.fragments.iter().map(String::as_str))
        .flat_map(str::split_ascii_whitespace)
        .any(|class| {
            let utility = class.rsplit(':').next().unwrap_or(class);
            utility.trim_matches('!') == "hyphens-auto"
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Only what the entry reaches counts: a component's table and the
    /// engine UI it imports, and `raycast` read in a reached module, while
    /// a word in a comment or a string elsewhere is not a raycast.
    #[test]
    fn the_graph_reads_tables_ui_and_raycasts_from_reached_files() {
        let dir = std::env::temp_dir().join(format!("og-graph-{}", std::process::id()));
        let engine = dir.join("engine");
        let game = dir.join("game");
        std::fs::create_dir_all(engine.join("src/ui/components/hud")).unwrap();
        std::fs::create_dir_all(game.join("src")).unwrap();
        let table = engine.join("src/ui/components/hud/bar.en.ogt");
        std::fs::write(&table, "").unwrap();
        let main = game.join("src/main.ts");
        std::fs::write(
            &main,
            "// raycast and hyphens-auto in a comment\nconst s = \"no raycast\";\nexport default s;\n",
        )
        .unwrap();
        // A class list built in a template, under a variant.
        let page = game.join("src/page.tsx");
        std::fs::write(
            &page,
            "export const P = (x: string) => <p class={`p-2 ${x} md:hyphens-auto`} />;\n",
        )
        .unwrap();
        let aim = game.join("src/aim.ts");
        std::fs::write(
            &aim,
            "export const hit = (og: any, w: any) => og.render.raycast(w, {});\n",
        )
        .unwrap();
        let graph = dir.join("graph.json");
        let write = |files: &[&Path]| {
            let v = serde_json::json!({ "engine": engine, "files": files });
            std::fs::write(&graph, v.to_string()).unwrap();
            CodeGraph::read(&graph).unwrap()
        };
        let none = write(&[&engine.join("src/index.ts"), &main]);
        assert_eq!(
            none,
            CodeGraph {
                files: vec![main.clone()],
                ..CodeGraph::default()
            }
        );
        let all = write(&[
            &engine.join("src/index.ts"),
            &engine.join("src/ui/index.ts"),
            &table,
            &main,
            &aim,
        ]);
        assert!(all.tsx && all.raycast && !all.hyphens);
        assert_eq!(all.files, [main.clone(), aim.clone()]);
        assert!(write(&[&main, &page]).hyphens);
        // An engine component the game imports writes classes too.
        let wrap = engine.join("src/ui/components/hud/wrap.tsx");
        std::fs::write(
            &wrap,
            "export const W = () => <p class=\"hyphens-auto\" />;\n",
        )
        .unwrap();
        assert!(write(&[&main, &wrap]).hyphens);
        assert_eq!(
            all.tables.into_iter().collect::<Vec<_>>(),
            vec![("@ui/components/hud/bar.en.ogt".to_string(), table)]
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
