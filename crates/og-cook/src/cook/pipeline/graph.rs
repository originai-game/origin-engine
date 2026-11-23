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
    /// The engine `.ogt` files the bundle holds, by the id the bundle names
    /// them by (`@ui/components/hud/bar.en.ogt`), with the file each is
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
                if rel.extension().is_some_and(|e| e == "ogt") {
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
            CodeGraph::read(&graph, &game).unwrap()
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
        // The images `nine-` classes name, under a variant, in a template
        // whose substitution cuts a name short (it keeps every image the
        // name starts), from the game's code and the engine's.
        let skin = engine.join("src/ui/components/hud/skin.tsx");
        std::fs::write(
            &skin,
            "export const S = (a: string) => <p class={`hover:nine-plate nine-button-${a}`} />;\n",
        )
        .unwrap();
        std::fs::write(
            &page,
            "export const P = () => <p class=\"!nine-slot\" />;\n",
        )
        .unwrap();
        let nine = write(&[&main, &skin, &page]).nine;
        let named = |n: &str| names_image(&nine, n);
        assert!(named("plate") && named("button-primary") && named("slot"));
        assert!(!named("button") && !named("panel") && !named("glow"));
        assert!(write(&[&main, &aim]).nine.is_empty());
        assert_eq!(
            all.tables.into_iter().collect::<Vec<_>>(),
            vec![("@ui/components/hud/bar.en.ogt".to_string(), table)]
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The deeper root owns a file: a game inside the engine package (a
    /// test fixture) keeps its code, whose loads the publish plan keeps,
    /// and an engine installed inside the game stays the engine's.
    #[test]
    fn a_game_under_the_engine_keeps_its_code_and_an_installed_engine_stays_the_engines() {
        let dir = std::env::temp_dir().join(format!("og-graph-nest-{}", std::process::id()));
        let engine = dir.join("engine");
        let fixture = engine.join("tests/fixtures/crowd");
        let game = dir.join("game");
        let installed = game.join("node_modules/@origingame/engine");
        for d in [
            engine.join("src"),
            fixture.join("src"),
            installed.join("src"),
        ] {
            std::fs::create_dir_all(d).unwrap();
        }
        let load =
            "export const m = (og: any, w: any) => og.load.gltf(w, \"models/walker.glb\");\n";
        let main = fixture.join("src/main.ts");
        std::fs::write(&main, load).unwrap();
        let game_main = game.join("src/main.ts");
        std::fs::create_dir_all(game.join("src")).unwrap();
        std::fs::write(&game_main, load).unwrap();
        let engine_index = engine.join("src/index.ts");
        let installed_index = installed.join("src/index.ts");
        std::fs::write(&engine_index, "export const raycast = 1;\n").unwrap();
        std::fs::write(&installed_index, "export const raycast = 1;\n").unwrap();
        let graph = dir.join("graph.json");
        let read = |engine: &Path, files: &[&Path], project: &Path| {
            let v = serde_json::json!({ "engine": engine, "files": files });
            std::fs::write(&graph, v.to_string()).unwrap();
            CodeGraph::read(&graph, project).unwrap()
        };
        let nested = read(&engine, &[&engine_index, &main], &fixture);
        assert_eq!(nested.files, std::slice::from_ref(&main));
        assert!(
            !nested.raycast,
            "the engine's own modules are not the game's"
        );
        let shipped = read(&installed, &[&installed_index, &game_main], &game);
        assert_eq!(shipped.files, std::slice::from_ref(&game_main));
        assert!(
            !shipped.raycast,
            "an installed engine's modules are not the game's"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
