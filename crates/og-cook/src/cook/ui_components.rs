//! The engine's built-in UI documents: every `.svelte` under the engine
//! package's `src/ui/components/` (in its folders: `base/`, `hud/`,
//! `world/`, `panels/`, `meta/` and the genre kits) and `src/ui/screens/`, by the name its
//! `<meta name="component">` or `<meta name="screen">` declares, under
//! the reserved id `@ui/<path under src/ui>` (`@ui/components/panels/backpack.svelte`,
//! `@ui/screens/pause.svelte`). A game uses a component by its name
//! (`<RpgBackpack/>`) and loads a screen by its id
//! (`og.load.document(world, "@ui/screens/pause.svelte")`) without a copy;
//! the cook then bundles it, its string tables, the files it names and the
//! components it uses, as it bundles a look's `@styles/`. A game document
//! declaring the same name wins (`og ui eject` makes one).
//!
//! `src/ui/controls/` holds the control documents (docs/ui.md §2.4), each
//! the inside of the element its `<meta name="control">` names. A document
//! uses a control document by having such an element, so a game bundles
//! only the controls its documents show.
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

/// The default style as a development cook from a checkout carries it
/// ([`base_css`]): og-ui's `set_base_css` puts it in the compiled one's
/// place in a development build.
pub const BASE: &str = "@ui/base/default.css";

// The generator gen-theme.sh runs, so the cook reads the tokens as it does.
#[allow(dead_code, unused_imports)]
#[path = "../../../og-ui/scripts/theme_codegen.rs"]
mod theme_codegen;

/// The directories under `src/ui` that hold built-in documents.
const ROOTS: [&str; 3] = ["components/", "controls/", "screens/"];

type FileBytes = Vec<(String, Vec<u8>)>;

/// One built-in document, a component or a screen.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Component {
    /// Its bundle id, `@ui/<path under src/ui>`.
    pub id: String,
    pub screen: bool,
    /// A control's small host selector, not its document's stylesheet.
    pub control: Option<String>,
    /// The component names it uses.
    pub uses: BTreeSet<String>,
    /// Its own text and every file it names that sits beside it (string
    /// tables, `@import`ed sheets, `url()` art, materials, Rive files and
    /// their import settings), by bundle id.
    pub files: Vec<String>,
    pub imports: Vec<String>,
    #[serde(skip)]
    doc: OnceLock<Result<og_ui::UiDocument, String>>,
    #[serde(skip)]
    bytes: OnceLock<Result<FileBytes, String>>,
}

impl Component {
    /// Parse only a document whose schema the caller actually needs.
    pub fn document(&self) -> Result<&og_ui::UiDocument, String> {
        self.doc
            .get_or_init(|| {
                let text = String::from_utf8(read(&self.id)?).map_err(|e| e.to_string())?;
                let imports = self
                    .imports
                    .iter()
                    .map(|id| {
                        Ok((
                            id.clone(),
                            String::from_utf8(read(id)?).map_err(|e| e.to_string())?,
                        ))
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                og_ui::UiDocument::parse_with_imports(&self.id, &text, imports)
                    .map_err(|e| e.join("\n"))
            })
            .as_ref()
            .map_err(Clone::clone)
    }

    fn bytes(&self) -> Result<&FileBytes, String> {
        self.bytes
            .get_or_init(|| {
                self.files
                    .iter()
                    .map(|id| Ok((id.clone(), read(id)?)))
                    .collect()
            })
            .as_ref()
            .map_err(Clone::clone)
    }
}

/// The documents a document uses, by name: the components its
/// capitalised or `ui:` tags name and the built-in control documents its elements
/// are hosts of.
pub fn uses(doc: &og_ui::UiDocument) -> BTreeSet<String> {
    let controls: Vec<(String, og_ui::doc::ControlHost)> = index().map_or_else(
        |_| Vec::new(),
        |index| {
            index
                .iter()
                .filter_map(|(name, c)| {
                    Some((
                        name.clone(),
                        og_ui::doc::ControlHost::parse(c.control.as_deref()?).ok()?,
                    ))
                })
                .collect()
        },
    );
    uses_among(doc, &controls)
}

/// [`uses`] with the control documents there are.
fn uses_among(
    doc: &og_ui::UiDocument,
    controls: &[(String, og_ui::doc::ControlHost)],
) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for n in &doc.nodes {
        match &n.kind {
            og_ui::doc::NodeKind::Use(name) => {
                out.insert(name.clone());
            }
            og_ui::doc::NodeKind::Element(_) => out.extend(
                controls
                    .iter()
                    .filter(|(name, host)| *name != doc.name && host.hosts(n))
                    .map(|(name, _)| name.clone()),
            ),
            _ => {}
        }
    }
    out
}

/// The directory the built-in documents are read from (`src/ui`); `None`
/// when none was found (an engine package without its `src/`).
pub fn ui_dir() -> Option<&'static Path> {
    static DIR: OnceLock<Option<PathBuf>> = OnceLock::new();
    DIR.get_or_init(|| {
        let ui = |p: PathBuf| p.join("manifest.json").is_file().then_some(p);
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

/// The default style as the checkout's source tree has it, when that is
/// not the style this cook was compiled with, and the files it is read
/// from (inputs of the cook either way; none outside a checkout). It is
/// `crates/og-ui/default.css`, or, when a token file
/// (`packages/design/tokens/*.css`) is newer than it, what gen-theme.sh
/// would write there from the tokens. A published game never carries it:
/// its default style is the compiled one.
pub fn base_css() -> (Result<Option<&'static [u8]>, String>, Vec<PathBuf>) {
    let Some(root) = ui_dir().and_then(|d| d.ancestors().nth(4)) else {
        return (Ok(None), Vec::new());
    };
    let (tokens, file) = (
        root.join("packages/design/tokens"),
        root.join("crates/og-ui/default.css"),
    );
    if !tokens.is_dir() || !file.is_file() {
        return (Ok(None), Vec::new());
    }
    let mut inputs: Vec<PathBuf> = std::fs::read_dir(&tokens)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "css"))
        .collect();
    inputs.sort();
    let modified = |p: &Path| std::fs::metadata(p).and_then(|m| m.modified()).ok();
    let generate = inputs.iter().any(|t| modified(t) > modified(&file));
    inputs.push(file.clone());
    let css = if generate {
        theme_codegen::load(&tokens)
            .and_then(|t| theme_codegen::generate_css(&t))
            .map_err(|e| format!("{}: {e}", tokens.display()))
    } else {
        std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))
    };
    let css = css.map(|css| {
        (css != og_ui::DEFAULT_CSS).then(|| {
            // Kept for the process (`og dev` cooks in it again and again),
            // each distinct text once.
            static KEPT: std::sync::Mutex<Vec<&'static [u8]>> = std::sync::Mutex::new(Vec::new());
            let mut kept = KEPT.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            match kept.iter().find(|k| **k == css.as_bytes()) {
                Some(k) => *k,
                None => {
                    let k: &'static [u8] = Box::leak(css.into_bytes().into_boxed_slice());
                    kept.push(k);
                    k
                }
            }
        })
    });
    (css, inputs)
}

/// Where the built-in file `id` (`@ui/components/rpg/rpg.en.ogt`) is read
/// from.
pub fn path(id: &str) -> Result<PathBuf, String> {
    let dir = ui_dir().ok_or("the engine package has no src/ui")?;
    Ok(dir.join(relative(id)?))
}

fn relative(id: &str) -> Result<&str, String> {
    id.strip_prefix(PREFIX)
        .filter(|r| ROOTS.iter().any(|root| r.starts_with(root)))
        .filter(|r| !r.split('/').any(|c| c == ".." || c.is_empty()))
        .ok_or_else(|| format!("'{id}' is not a built-in UI file"))
}

/// The bytes of the built-in file `id` (`@ui/components/rpg/rpg.en.ogt`).
pub fn read(id: &str) -> Result<Vec<u8>, String> {
    std::fs::read(path(id)?).map_err(|e| format!("{id}: {e}"))
}

/// Every built-in document by its name, read from the shipped manifest.
pub fn index() -> Result<&'static BTreeMap<String, Component>, String> {
    static INDEX: OnceLock<Result<BTreeMap<String, Component>, String>> = OnceLock::new();
    INDEX
        .get_or_init(|| {
            let dir = ui_dir().ok_or("the engine package has no src/ui")?;
            let file = dir.join("manifest.json");
            let text = std::fs::read_to_string(&file)
                .map_err(|e| format!("{}: {e}; regenerate with og api gen", file.display()))?;
            serde_json::from_str(&text).map_err(|e| format!("{}: {e}", file.display()))
        })
        .as_ref()
        .map_err(Clone::clone)
}

/// The shipped index, regenerated and drift-checked alongside the API.
pub const MANIFEST: &str = "packages/engine/src/ui/manifest.json";

/// Generate from source, deliberately without reading the installed index.
pub fn manifest(dir: &Path) -> Result<String, String> {
    struct Parsed {
        id: String,
        doc: og_ui::UiDocument,
        files: Vec<String>,
        imports: Vec<String>,
    }
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
    let mut out: BTreeMap<String, Parsed> = BTreeMap::new();
    for rel in files {
        let id = format!("{PREFIX}{rel}");
        let read =
            |id: &str| std::fs::read(dir.join(relative(id)?)).map_err(|e| format!("{id}: {e}"));
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
        if doc.control.is_some() != rel.starts_with("controls/") {
            return Err(format!(
                "{id}: src/ui/controls holds the control documents (<meta name=\"control\">), and only them"
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
        let imported = imports.iter().map(|(id, _)| id.clone()).collect();
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
        let mut component = Parsed {
            id: id.clone(),
            files: vec![id.clone()],
            imports: imported,
            doc,
        };
        component.files.extend(beside.into_iter().map(|(id, _)| id));
        component.files.sort();
        component.files.dedup();
        let name = component.doc.name.clone();
        if let Some(other) = out.insert(name.clone(), component) {
            return Err(format!(
                "{id} and {} both declare the built-in document '{name}'; rename one",
                other.id
            ));
        }
    }
    let controls: Vec<(String, og_ui::doc::ControlHost)> = out
        .values()
        .filter_map(|c| Some((c.doc.name.clone(), c.doc.control.clone()?)))
        .collect();
    let entries: BTreeMap<_, _> = out
        .into_iter()
        .map(|(name, c)| {
            let uses = uses_among(&c.doc, &controls);
            (
                name,
                Component {
                    id: c.id,
                    screen: c.doc.kind == og_ui::doc::DocKind::Screen,
                    control: c.doc.control.map(|host| {
                        host.document_name()
                            .trim_start_matches("control:")
                            .to_owned()
                    }),
                    uses,
                    files: c.files,
                    imports: c.imports,
                    doc: OnceLock::new(),
                    bytes: OnceLock::new(),
                },
            )
        })
        .collect();
    serde_json::to_string_pretty(&entries)
        .map(|text| text + "\n")
        .map_err(|e| e.to_string())
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
            if let Some((name, _)) = index()?.iter().find(|(_, c)| c.id == name) {
                pending.push(name.clone());
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
        for (id, bytes) in c.bytes()? {
            files.insert(id.clone(), bytes.as_slice());
        }
        pending.extend(c.uses.iter().cloned());
    }
    Ok(Resolved { components, files })
}
