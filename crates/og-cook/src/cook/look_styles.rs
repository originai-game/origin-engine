//! Built-in CSS packs and shared assets under `templates/styles/`, read
//! from disk rather than compiled into `og`. Published engines carry them
//! in `styles/`. A look names a pack directory by slug; the game's own
//! `assets/styles/<slug>/` wins over `@styles/<slug>/`.
//!
//! The directory is the first of: `OG_STYLES_DIR`; `styles/` of the engine
//! package `OG_ENGINE_PACKAGE` names (bin/og.mjs sets it); an installed
//! `@origingame/engine/styles` or a checkout's `templates/styles` above the
//! running executable; the checkout this crate was built in. A file is read
//! when the cook first asks for it.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Whether a file is a licence (`LICENSES.md`, a font's `OFL.txt` or
/// `OFL-<family>.txt`) or a generated style's `art-requests.json` (the art
/// it asks for): never cooked into a game's bundle, only written beside the
/// rest by `og style eject`, since a copied font travels with its licence.
fn licence(file: &str) -> bool {
    file.ends_with(".md")
        || file == "extension.json"
        || file == "choices.json"
        || file == "art-requests.json"
        || (file.starts_with("OFL") && file.ends_with(".txt"))
}

/// One file of a built-in style, by the bundle id it would have.
pub struct StyleFile {
    id: &'static str,
    path: PathBuf,
    bytes: OnceLock<Vec<u8>>,
}

impl StyleFile {
    /// Its bundle id: `@styles/<name>/<file>`.
    pub fn id(&self) -> &'static str {
        self.id
    }

    /// Its bytes, read on first use; empty when the file cannot be read (the
    /// cook then reports the asset it fails to parse).
    pub fn bytes(&'static self) -> &'static [u8] {
        self.bytes
            .get_or_init(|| std::fs::read(&self.path).unwrap_or_default())
    }

    /// Its id and bytes.
    pub fn pair(&'static self) -> (&'static str, &'static [u8]) {
        (self.id, self.bytes())
    }
}

struct Table {
    dir: Option<PathBuf>,
    styles: Vec<StyleFile>,
    licences: Vec<StyleFile>,
}

fn table() -> &'static Table {
    static TABLE: OnceLock<Table> = OnceLock::new();
    TABLE.get_or_init(|| {
        let dir = find_dir();
        let (mut styles, mut licences) = (Vec::new(), Vec::new());
        if let Some(dir) = &dir {
            for (pack, file, path) in files(dir) {
                let f = StyleFile {
                    id: Box::leak(format!("@styles/{pack}/{file}").into_boxed_str()),
                    path,
                    bytes: OnceLock::new(),
                };
                if licence(&file) {
                    licences.push(f);
                } else {
                    styles.push(f);
                }
            }
        }
        Table {
            dir,
            styles,
            licences,
        }
    })
}

/// Every shipped file in a pack's directory, sorted: `(pack, file, path)`.
fn files(dir: &Path) -> Vec<(String, String, PathBuf)> {
    let names = |d: &Path, want_dir: bool| -> Vec<String> {
        let mut out: Vec<String> = std::fs::read_dir(d)
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.file_type().is_ok_and(|t| t.is_dir() == want_dir))
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| !n.starts_with('.'))
            .collect();
        out.sort();
        out
    };
    let mut out = Vec::new();
    for pack in names(dir, true) {
        let mut pending = vec![String::new()];
        while let Some(prefix) = pending.pop() {
            let parent = dir.join(&pack).join(&prefix);
            for file in names(&parent, false) {
                let relative = format!("{prefix}{file}");
                out.push((pack.clone(), relative, parent.join(file)));
            }
            for child in names(&parent, true) {
                pending.push(format!("{prefix}{child}/"));
            }
        }
    }
    out.sort();
    out
}

fn find_dir() -> Option<PathBuf> {
    // Named, it is the one place looked at: a wrong one is reported, not
    // passed over for another.
    if let Some(d) = std::env::var_os("OG_STYLES_DIR") {
        let d = PathBuf::from(d);
        return d.is_dir().then_some(d);
    }
    let styles = |p: PathBuf| {
        std::fs::read_dir(&p)
            .ok()?
            .flatten()
            .any(|e| {
                e.file_name() == "shared"
                    || std::fs::read_dir(e.path())
                        .into_iter()
                        .flatten()
                        .flatten()
                        .any(|f| f.path().extension().is_some_and(|x| x == "css"))
            })
            .then_some(p)
    };
    // A checkout's packages/engine has the checkout's packs two levels up.
    if let Some(pkg) = std::env::var_os("OG_ENGINE_PACKAGE").map(PathBuf::from)
        && let Some(d) =
            styles(pkg.join("../../templates/styles")).or_else(|| styles(pkg.join("styles")))
    {
        return Some(d);
    }
    // A checkout's og (target/, packages/engine/native/) reads the
    // checkout's packs, not the copy pack.sh leaves in packages/engine;
    // the og of an npm install sits in @origingame/engine-<platform>/, a
    // sibling of @origingame/engine.
    if let Ok(exe) = std::env::current_exe() {
        for sub in ["templates/styles", "engine/styles"] {
            if let Some(d) = exe.ancestors().skip(1).find_map(|a| styles(a.join(sub))) {
                return Some(d);
            }
        }
    }
    styles(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates/styles"))
}

/// The directory the bundled font packs are read from (`templates/fonts`
/// in a checkout, `fonts/` beside `styles/` in the engine package, or
/// `OG_FONTS_DIR`); `None` when none was found.
pub fn fonts_dir() -> Option<&'static Path> {
    static DIR: std::sync::OnceLock<Option<PathBuf>> = std::sync::OnceLock::new();
    DIR.get_or_init(|| {
        if let Some(d) = std::env::var_os("OG_FONTS_DIR") {
            let d = PathBuf::from(d);
            return d.is_dir().then_some(d);
        }
        let packs = |p: PathBuf| {
            std::fs::read_dir(&p)
                .ok()?
                .flatten()
                .any(|e| e.path().join("extension.json").is_file())
                .then_some(p)
        };
        if let Some(pkg) = std::env::var_os("OG_ENGINE_PACKAGE").map(PathBuf::from)
            && let Some(d) =
                packs(pkg.join("../../templates/fonts")).or_else(|| packs(pkg.join("fonts")))
        {
            return Some(d);
        }
        if let Ok(exe) = std::env::current_exe() {
            for sub in ["templates/fonts", "engine/fonts"] {
                if let Some(d) = exe.ancestors().skip(1).find_map(|a| packs(a.join(sub))) {
                    return Some(d);
                }
            }
        }
        packs(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates/fonts"))
    })
    .as_deref()
}

/// The directory the built-in styles are read from; `None` when none was
/// found (an engine package without its `styles/`).
pub fn styles_dir() -> Option<&'static Path> {
    table().dir.as_deref()
}

/// Every cooked file of the built-in styles, sorted by id.
pub fn styles() -> &'static [StyleFile] {
    &table().styles
}

/// The built-in styles' licence files and art requests, sorted by id.
pub fn style_licences() -> &'static [StyleFile] {
    &table().licences
}
