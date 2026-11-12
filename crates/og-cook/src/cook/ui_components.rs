//! The engine's built-in component documents: every `.svelte` under the
//! engine package's `src/ui/components/`, its group directories (`rpg/`,
//! `shooter/`) included, by the name its `<meta name="component">`
//! declares. A game uses one by that name (`<RpgBackpack/>`) without a
//! copy; the cook then bundles it, its string tables and the files it
//! names under the reserved id `@ui/<path>` (`@ui/rpg/backpack.svelte`),
//! as it bundles a look's `@styles/`. A game document declaring the same
//! component name wins (`og ui eject` makes one).
//!
//! The directory is `src/ui/components` of the engine package
//! `OG_ENGINE_PACKAGE` names (bin/og.mjs sets it), else of an installed
//! `@origingame/engine` or a checkout's `packages/engine` above the running
//! executable, else of the checkout this crate was built in. It is read
//! when a game first uses a name it does not define.

use og_ui::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// The reserved id prefix of the built-in components' files.
pub const PREFIX: &str = "@ui/";

/// One built-in component document.
pub struct Component {
    /// Its bundle id, `@ui/<path under src/ui/components>`.
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

/// The directory the built-in components are read from; `None` when none
/// was found (an engine package without its `src/`).
pub fn components_dir() -> Option<&'static Path> {
    static DIR: OnceLock<Option<PathBuf>> = OnceLock::new();
    DIR.get_or_init(|| {
        let components = |p: PathBuf| p.join("README.md").is_file().then_some(p);
        if let Some(pkg) = std::env::var_os("OG_ENGINE_PACKAGE").map(PathBuf::from) {
            return components(pkg.join("src/ui/components"));
        }
        // A checkout's og (target/, packages/engine/native/) reads the
        // checkout's documents; the og of an npm install sits in
        // @origingame/engine-<platform>/, a sibling of @origingame/engine.
        if let Ok(exe) = std::env::current_exe() {
            for sub in [
                "packages/engine/src/ui/components",
                "engine/src/ui/components",
            ] {
                if let Some(d) = exe
                    .ancestors()
                    .skip(1)
                    .find_map(|a| components(a.join(sub)))
                {
                    return Some(d);
                }
            }
        }
        components(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/engine/src/ui/components"),
        )
    })
    .as_deref()
}

/// The bytes of the built-in file `id` (`@ui/rpg/rpg.en.ogt`).
pub fn read(id: &str) -> Result<Vec<u8>, String> {
    let rel = id
        .strip_prefix(PREFIX)
        .filter(|r| !r.split('/').any(|c| c == ".." || c.is_empty()))
        .ok_or_else(|| format!("'{id}' is not a built-in component file"))?;
    let dir = components_dir().ok_or("the engine package has no src/ui/components")?;
    std::fs::read(dir.join(rel)).map_err(|e| format!("{id}: {e}"))
}

/// Every built-in component by its name. An error names a document that
/// does not parse, or two that declare one name.
pub fn index() -> Result<&'static BTreeMap<String, Component>, String> {
    static INDEX: OnceLock<Result<BTreeMap<String, Component>, String>> = OnceLock::new();
    INDEX
        .get_or_init(|| {
            let dir = components_dir().ok_or("the engine package has no src/ui/components")?;
            build(dir)
        })
        .as_ref()
        .map_err(Clone::clone)
}

fn build(dir: &Path) -> Result<BTreeMap<String, Component>, String> {
    let mut files = Vec::new();
    let mut pending = vec![String::new()];
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
        if doc.kind != og_ui::doc::DocKind::Component {
            return Err(format!(
                "{id}: a built-in component declares <meta name=\"component\">"
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
                "{id} and {} both declare the built-in component '{name}'; rename one",
                other.id
            ));
        }
    }
    Ok(out)
}

/// The built-in components a game's documents reach: every name they use
/// that no game component declares, and the names those use, resolved
/// against the index. Answers each used name's document id (the game's
/// or the built-in one) and the built-in files to bundle, by id.
pub struct Resolved {
    pub components: BTreeMap<String, String>,
    pub files: BTreeMap<String, &'static [u8]>,
}

/// Resolves `used` (names a game's documents use) with the game's own
/// components `game` (name → id) first.
pub fn resolve(
    game: BTreeMap<String, String>,
    used: impl IntoIterator<Item = String>,
) -> Result<Resolved, String> {
    let mut components = game;
    let mut files: BTreeMap<String, &'static [u8]> = BTreeMap::new();
    let mut pending: Vec<String> = used.into_iter().collect();
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
