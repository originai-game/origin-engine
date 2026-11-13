//! The engine's built-in UI documents: every `.svelte` under the engine
//! package's `src/ui/components/` (its group directories, `rpg/` and
//! `shooter/`, included) and `src/ui/screens/`, by the name its
//! `<meta name="component">` or `<meta name="screen">` declares, under
//! the reserved id `@ui/<path under src/ui>` (`@ui/components/rpg/backpack.svelte`,
//! `@ui/screens/pause.svelte`). A game uses a component by its name
//! (`<RpgBackpack/>`) and loads a screen by its id
//! (`og.load.document(world, "@ui/screens/pause.svelte")`) without a copy;
//! the cook then bundles it, its string tables, the files it names and the
//! components it uses, as it bundles a look's `@styles/`. A game document
//! declaring the same name wins (`og ui eject` makes one).
//!
//! The directory is `src/ui` of the engine package `OG_ENGINE_PACKAGE`
//! names (bin/og.mjs sets it), else of an installed `@origingame/engine`
//! or a checkout's `packages/engine` above the running executable, else of
//! the checkout this crate was built in. It is read when a game first uses
//! a name it does not define.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// The reserved id prefix of the built-in documents' files.
pub const PREFIX: &str = "@ui/";

/// The directories under `src/ui` that hold built-in documents.
const ROOTS: [&str; 2] = ["components/", "screens/"];

/// One built-in document, a component or a screen.
pub struct Component {
    /// Its bundle id, `@ui/<path under src/ui>`.
    pub id: String,
    pub doc: og_ui::UiDocument,
    /// The component names it uses.
    pub uses: BTreeSet<String>,
    /// Its own text and every file it names that sits beside it (string
    /// tables, `@import`ed sheets, `url()` art, materials, Rive files and
    /// their import settings), by bundle id.
    pub files: Vec<(String, Vec<u8>)>,
}

/// The component names a document uses (its capitalised tags).
pub fn uses(doc: &og_ui::UiDocument) -> BTreeSet<String> {
    doc.nodes
        .iter()
        .filter_map(|n| match &n.kind {
            og_ui::doc::NodeKind::Use(name) => Some(name.clone()),
            _ => None,
        })
        .collect()
}

/// The directory the built-in documents are read from (`src/ui`); `None`
/// when none was found (an engine package without its `src/`).
pub fn ui_dir() -> Option<&'static Path> {
    static DIR: OnceLock<Option<PathBuf>> = OnceLock::new();
    DIR.get_or_init(|| {
        let ui = |p: PathBuf| p.join("components/README.md").is_file().then_some(p);
        if let Some(pkg) = std::env::var_os("OG_ENGINE_PACKAGE").map(PathBuf::from) {
            return ui(pkg.join("src/ui"));
        }
        // A checkout's og (target/, packages/engine/native/) reads the
        // checkout's documents; the og of an npm install sits in
        // @origingame/engine-<platform>/, a sibling of @origingame/engine.
        if let Ok(exe) = std::env::current_exe() {
            for sub in ["packages/engine/src/ui", "engine/src/ui"] {
                if let Some(d) = exe.ancestors().skip(1).find_map(|a| ui(a.join(sub))) {
                    return Some(d);
                }
            }
        }
        ui(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/engine/src/ui"))
    })
    .as_deref()
}

/// The bytes of the built-in file `id` (`@ui/components/rpg/rpg.en.ogt`).
pub fn read(id: &str) -> Result<Vec<u8>, String> {
    let rel = id
        .strip_prefix(PREFIX)
        .filter(|r| ROOTS.iter().any(|root| r.starts_with(root)))
        .filter(|r| !r.split('/').any(|c| c == ".." || c.is_empty()))
        .ok_or_else(|| format!("'{id}' is not a built-in UI file"))?;
    let dir = ui_dir().ok_or("the engine package has no src/ui")?;
    std::fs::read(dir.join(rel)).map_err(|e| format!("{id}: {e}"))
}

/// Every built-in document by its name. An error names a document that
/// does not parse, a screen among the components, or two that declare one
/// name.
pub fn index() -> Result<&'static BTreeMap<String, Component>, String> {
    static INDEX: OnceLock<Result<BTreeMap<String, Component>, String>> = OnceLock::new();
    INDEX
        .get_or_init(|| {
            let dir = ui_dir().ok_or("the engine package has no src/ui")?;
            build(dir)
        })
        .as_ref()
        .map_err(Clone::clone)
}

fn build(dir: &Path) -> Result<BTreeMap<String, Component>, String> {
    let mut files = Vec::new();
    let mut pending: Vec<String> = ROOTS.iter().map(|r| r.to_string()).collect();
    while let Some(prefix) = pending.pop() {
        let parent = dir.join(&prefix);
        let entries =
            std::fs::read_dir(&parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                continue;
            }
            let rel = format!("{prefix}{name}");
            if e.file_type().is_ok_and(|t| t.is_dir()) {
                pending.push(format!("{rel}/"));
            } else if name.ends_with(".svelte") {
                files.push(rel);
            }
        }
    }
    files.sort();
    let mut out: BTreeMap<String, Component> = BTreeMap::new();
    for rel in files {
        let id = format!("{PREFIX}{rel}");
        let text = String::from_utf8(read(&id)?).map_err(|_| format!("{id}: not UTF-8"))?;
        let mut imports: Vec<(String, String)> = Vec::new();
        let mut wanted: Vec<(String, String)> = og_ui::UiDocument::imports(&text)
            .into_iter()
            .map(|p| (id.clone(), p))
            .collect();
        while let Some((from, path)) = wanted.pop() {
            let at = og_ui::doc::at_rules::resolve(&from, &path);
            if imports.iter().any(|(p, _)| *p == at) {
                continue;
            }
            let css = String::from_utf8(read(&at)?).map_err(|_| format!("{at}: not UTF-8"))?;
            wanted.extend(
                og_ui::UiDocument::imports(&css)
                    .into_iter()
                    .map(|p| (at.clone(), p)),
            );
            imports.push((at, css));
        }
        let doc = og_ui::UiDocument::parse_with_imports(&id, &text, imports.clone())
            .map_err(|e| e.join("\n"))?;
        if doc.kind != og_ui::doc::DocKind::Component && rel.starts_with("components/") {
            return Err(format!(
                "{id}: a built-in component declares <meta name=\"component\">; screens go in src/ui/screens"
            ));
        }
        let mut named: BTreeSet<String> = doc.strings.iter().cloned().collect();
        named.extend(doc.sheet.assets.iter().cloned());
        named.extend(doc.material_assets());
        named.extend(doc.rive_ids());
        named.extend(
            doc.sheet
                .font_faces
                .iter()
                .flat_map(|f| f.src.iter().cloned()),
        );
        let mut beside = Vec::new();
        for (path, css) in imports {
            beside.push((path, css.into_bytes()));
        }
        for path in named.into_iter().filter(|p| p.starts_with(PREFIX)) {
            beside.push((path.clone(), read(&path)?));
            let settings = format!("{path}.ogt");
            if let Ok(bytes) = read(&settings) {
                beside.push((settings, bytes));
            }
        }
        let mut component = Component {
            id: id.clone(),
            uses: uses(&doc),
            files: vec![(id.clone(), text.into_bytes())],
            doc,
        };
        component.files.extend(beside);
        let name = component.doc.name.clone();
        if let Some(other) = out.insert(name.clone(), component) {
            return Err(format!(
                "{id} and {} both declare the built-in document '{name}'; rename one",
                other.id
            ));
        }
    }
    Ok(out)
}

/// The built-in documents a game reaches: every name its documents use and
/// every built-in screen its code names by id, that no game document
/// declares, and the names those use, resolved against the index. Answers
/// each reached name's document id (the game's or the built-in one) and
/// the built-in files to bundle, by id.
pub struct Resolved {
    pub components: BTreeMap<String, String>,
    pub files: BTreeMap<String, &'static [u8]>,
}

/// Resolves `used` (names a game's documents use, or the built-in ids its
/// code names) with the game's own documents `game` (name → id) first.
pub fn resolve(
    game: BTreeMap<String, String>,
    used: impl IntoIterator<Item = String>,
) -> Result<Resolved, String> {
    let mut components = game;
    let mut files: BTreeMap<String, &'static [u8]> = BTreeMap::new();
    let mut pending: Vec<String> = Vec::new();
    for name in used {
        if name.starts_with(PREFIX) {
            // An id the code names: the built-in document it is, if any.
            if let Some(c) = index()?.values().find(|c| c.id == name) {
                pending.push(c.doc.name.clone());
            }
        } else {
            pending.push(name);
        }
    }
    while let Some(name) = pending.pop() {
        if components.contains_key(&name) {
            continue;
        }
        // A name neither the game nor the engine declares is the build's
        // warning ("no component …"), not the cook's.
        let Some(c) = index()?.get(&name) else {
            continue;
        };
        components.insert(name, c.id.clone());
        for (id, bytes) in &c.files {
            files.insert(id.clone(), bytes.as_slice());
        }
        pending.extend(c.uses.iter().cloned());
    }
    Ok(Resolved { components, files })
}
