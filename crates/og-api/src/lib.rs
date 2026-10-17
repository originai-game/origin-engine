//! Origin Engine `og-api`: the IDL (`api.ogt`), its model, validator and
//! generators. See `crates/og-api/README.md` for the grammar and the rules,
//! `docs/design.md` §3 for what the API is.
//!
//! The flow is [`read`] (text → validated [`Api`]) then [`generate`] (model →
//! files under `packages/engine/`). [`check`] regenerates into a temporary
//! directory and compares, which is the drift check behind `og api check`.

mod gen;
mod index;
mod json;
mod load;
pub mod model;
mod ogt;
mod validate;

pub use index::Index;
pub use model::Api;

use std::fmt;
use std::path::{Path, PathBuf};

/// The IDL, relative to the repository root.
pub const IDL_PATH: &str = "crates/og-api/api.ogt";

/// Directories whose every file is generated from the IDL, relative to the
/// repository root. A file in them that the generator does not write is drift.
pub const OUTPUT_DIRS: &[&str] = &["packages/engine/types", "packages/engine/schema"];

/// One problem in an IDL, with the 1-based line it was found on.
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

/// Parse, load and validate IDL text. Every problem found is returned, in
/// line order; an API is returned only when there are none.
pub fn parse(text: &str) -> Result<Api, Vec<Diagnostic>> {
    let doc = ogt::parse(text).map_err(|e| {
        vec![Diagnostic {
            line: e.line,
            message: e.message,
        }]
    })?;
    if doc.kind != "api" {
        return Err(vec![Diagnostic {
            line: 1,
            message: format!("this is a `{}` document; the IDL is `type api`", doc.kind),
        }]);
    }
    let mut loader = load::Loader { diags: Vec::new() };
    let api = loader.api(doc.version, &doc.nodes);
    let mut diags = loader.diags;
    diags.extend(validate::validate(&api));
    diags.sort_by_key(|d| d.line);
    if diags.is_empty() {
        Ok(api)
    } else {
        Err(diags)
    }
}

/// Why an IDL file could not be turned into an API.
#[derive(Debug)]
pub enum ReadError {
    Io(PathBuf, std::io::Error),
    Invalid(PathBuf, Vec<Diagnostic>),
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadError::Io(p, e) => write!(f, "cannot read {}: {e}", p.display()),
            ReadError::Invalid(p, diags) => {
                writeln!(
                    f,
                    "{} is not a valid API ({} problem(s)):",
                    p.display(),
                    diags.len()
                )?;
                for d in diags {
                    writeln!(f, "  {}:{}: {}", p.display(), d.line, d.message)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ReadError {}

/// Read and validate an IDL file.
pub fn read(path: &Path) -> Result<Api, ReadError> {
    let text = std::fs::read_to_string(path).map_err(|e| ReadError::Io(path.to_path_buf(), e))?;
    parse(&text).map_err(|d| ReadError::Invalid(path.to_path_buf(), d))
}

/// One generated file: its path relative to the repository root and its bytes.
#[derive(Debug, Clone, PartialEq)]
pub struct Generated {
    pub path: &'static str,
    pub contents: String,
}

/// Every file the IDL generates, in a fixed order.
pub fn generate(api: &Api) -> Vec<Generated> {
    let ix = Index::new(api);
    vec![
        Generated {
            path: "packages/engine/types/index.d.ts",
            contents: gen::ts::dts(&ix),
        },
        Generated {
            path: "packages/engine/types/bindings.json",
            contents: gen::tables::bindings(&ix),
        },
        Generated {
            path: "packages/engine/schema/api.schema.json",
            contents: gen::tables::schema(&ix),
        },
    ]
}

/// Write generated files under `root`, creating directories as needed.
pub fn write(root: &Path, files: &[Generated]) -> std::io::Result<()> {
    for file in files {
        let path = root.join(file.path);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(&path, &file.contents)?;
    }
    Ok(())
}

/// How one file differs from what the IDL generates.
#[derive(Debug, Clone, PartialEq)]
pub enum Drift {
    /// The generator writes this file and it is absent.
    Missing(String),
    /// The file differs; `line` is the first differing 1-based line.
    Differs { path: String, line: usize },
    /// A file in a generated directory that the generator does not write.
    Unexpected(String),
}

impl Drift {
    pub fn path(&self) -> &str {
        match self {
            Drift::Missing(p) | Drift::Unexpected(p) => p,
            Drift::Differs { path, .. } => path,
        }
    }
}

impl fmt::Display for Drift {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Drift::Missing(p) => write!(f, "{p}: missing; run `og api gen`"),
            Drift::Differs { path, line } => write!(
                f,
                "{path}: differs from what api.ogt generates, first at line {line}; generated files are not edited by hand: change {IDL_PATH} and run `og api gen`"
            ),
            Drift::Unexpected(p) => write!(
                f,
                "{p}: not generated from api.ogt; delete it, or declare what it holds in {IDL_PATH}"
            ),
        }
    }
}

/// The drift check: read `root`'s IDL, regenerate into a temporary directory
/// and compare with the files under `root`. An empty list means no drift.
pub fn check(root: &Path) -> Result<Vec<Drift>, ReadError> {
    let api = read(&root.join(IDL_PATH))?;
    let files = generate(&api);
    let scratch = scratch_dir();
    write(&scratch, &files).map_err(|e| ReadError::Io(scratch.clone(), e))?;
    let drift = compare(&scratch, root, &files);
    let _ = std::fs::remove_dir_all(&scratch);
    Ok(drift)
}

fn compare(fresh: &Path, root: &Path, files: &[Generated]) -> Vec<Drift> {
    let mut drift = Vec::new();
    for file in files {
        let want = std::fs::read(fresh.join(file.path)).unwrap_or_default();
        match std::fs::read(root.join(file.path)) {
            Err(_) => drift.push(Drift::Missing(file.path.to_string())),
            Ok(have) if have != want => drift.push(Drift::Differs {
                path: file.path.to_string(),
                line: first_difference(&have, &want),
            }),
            Ok(_) => {}
        }
    }
    for dir in OUTPUT_DIRS {
        let mut found = Vec::new();
        list_files(&root.join(dir), &mut found);
        for path in found {
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            if !files.iter().any(|f| f.path == rel) {
                drift.push(Drift::Unexpected(rel));
            }
        }
    }
    drift
}

fn list_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.filter_map(Result::ok).map(|e| e.path()).collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            list_files(&path, out);
        } else {
            out.push(path);
        }
    }
}

fn first_difference(a: &[u8], b: &[u8]) -> usize {
    let a = String::from_utf8_lossy(a);
    let b = String::from_utf8_lossy(b);
    let mut la = a.split('\n');
    let mut lb = b.split('\n');
    let mut n = 1;
    loop {
        match (la.next(), lb.next()) {
            (None, None) => return n,
            (x, y) if x != y => return n,
            _ => n += 1,
        }
    }
}

fn scratch_dir() -> PathBuf {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static N: AtomicUsize = AtomicUsize::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!(
        "og-api-check-{}-{nanos}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    ))
}
