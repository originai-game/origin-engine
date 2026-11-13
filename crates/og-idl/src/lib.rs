//! Origin Engine `og-idl`: the IDL (`crates/og-api/api/`), its model,
//! validator and generators. See `crates/og-api/README.md` for the grammar
//! and the rules, `docs/design.md` §3 for what the API is.
//!
//! The flow is [`read`] (text → validated [`Api`]) then [`generate`] (model →
//! files under `packages/engine/` and the hosts' generated Rust). [`check`]
//! regenerates into a temporary directory and compares, which is the drift
//! check behind `og api check`.
//!
//! No host or runtime depends on this crate: they compile what it wrote
//! (`og_api::host` and each host's `src/generated/glue.rs`), so a change here
//! rebuilds only this crate and `og`, and [`write`] leaves a file whose bytes
//! did not change untouched, so an IDL edit rebuilds only the crates whose
//! generated Rust it changes.

pub mod budget;
pub mod embed;
mod r#gen;
mod index;
mod json;
mod load;
pub mod model;
pub mod search;
mod validate;
mod values;

pub use r#gen::wiki::CORE_PREFIX;
pub use index::Index;
pub use model::{Api, Guide};

use std::fmt;
use std::path::{Path, PathBuf};

/// The IDL directory, relative to the repository root.
pub const IDL_PATH: &str = "crates/og-api/api";

/// Directories whose every file is generated from the IDL, relative to the
/// repository root. A file in them that the generator does not write is drift.
pub const OUTPUT_DIRS: &[&str] = &[
    "packages/engine/types",
    "packages/engine/schema",
    "packages/engine/src/generated",
    "crates/og-api/src/generated",
    "crates/og-records/src/generated",
    "crates/og-host-web/src/generated",
    "crates/og-host-node/src/generated",
    "crates/og-runtime/src/host/generated",
    "packages/knowledge/wiki",
];

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
    let doc = og_core::ogt::parse(text).map_err(|e| vec![syntax_diagnostic(e.reason())])?;
    if doc.ty != "api" {
        return Err(vec![Diagnostic {
            line: 1,
            message: format!("this is a `{}` document; the IDL is `type api`", doc.ty),
        }]);
    }
    let mut loader = load::Loader { diags: Vec::new() };
    let mut api = loader.api(doc.version, &doc.body.entries);
    api.source = text.lines().map(str::to_string).collect();
    let mut diags = loader.diags;
    diags.extend(validate::validate(&api));
    diags.sort_by_key(|d| d.line);
    if diags.is_empty() {
        Ok(api)
    } else {
        Err(diags)
    }
}

/// og-core's `.ogt` errors read `line N: reason`; a header error names no line.
fn syntax_diagnostic(reason: &str) -> Diagnostic {
    let parsed = reason.strip_prefix("line ").and_then(|rest| {
        let (n, message) = rest.split_once(": ")?;
        Some((n.parse().ok()?, message))
    });
    match parsed {
        Some((line, message)) => Diagnostic {
            line,
            message: message.to_string(),
        },
        None => Diagnostic {
            line: 1,
            message: reason.to_string(),
        },
    }
}

/// Why an IDL file could not be turned into an API.
#[derive(Debug)]
pub enum ReadError {
    Io(PathBuf, std::io::Error),
    Invalid(PathBuf, Vec<(PathBuf, Diagnostic)>),
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
                for (p, d) in diags {
                    writeln!(f, "  {}:{}: {}", p.display(), d.line, d.message)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ReadError {}

/// Read the parts listed in `parts.txt` and validate their concatenation.
/// Model lines remain global for generated quotes; diagnostics name the
/// part and its local line.
pub fn read(path: &Path) -> Result<Api, ReadError> {
    let manifest = path.join("parts.txt");
    let parts = std::fs::read_to_string(&manifest).map_err(|e| ReadError::Io(manifest, e))?;
    let mut text = String::new();
    let mut sources = vec![(0, path.to_path_buf())];
    let mut offset = 0;
    for name in parts.lines() {
        let part = path.join(name);
        let source = std::fs::read_to_string(&part).map_err(|e| ReadError::Io(part.clone(), e))?;
        sources.push((offset, part));
        offset += source.bytes().filter(|&b| b == b'\n').count();
        text.push_str(&source);
    }
    let invalid = |diags: Vec<Diagnostic>| {
        let diags = diags
            .into_iter()
            .map(|mut d| {
                let (offset, part) = sources.iter().rev().find(|(n, _)| *n < d.line).unwrap();
                d.line -= offset;
                (part.clone(), d)
            })
            .collect();
        ReadError::Invalid(path.to_path_buf(), diags)
    };
    let mut api = parse(&text).map_err(invalid)?;
    api.parts = sources
        .iter()
        .skip(1)
        .map(|(n, p)| {
            (
                *n,
                p.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
            )
        })
        .collect();
    api.guides = guides(&path.join("..").join("guides"))?;
    let unresolved = validate::guide_refs(&api);
    if !unresolved.is_empty() {
        return Err(invalid(unresolved));
    }
    Ok(api)
}

/// The guides beside the IDL (`crates/og-api/guides/*.md`), sorted by name;
/// none when the directory is absent.
fn guides(dir: &Path) -> Result<Vec<Guide>, ReadError> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for entry in read {
        let path = entry
            .map_err(|e| ReadError::Io(dir.to_path_buf(), e))?
            .path();
        let Some(name) = path
            .extension()
            .is_some_and(|x| x == "md")
            .then(|| path.file_stem())
            .flatten()
        else {
            continue;
        };
        let text = std::fs::read_to_string(&path).map_err(|e| ReadError::Io(path.clone(), e))?;
        out.push(Guide {
            name: name.to_string_lossy().into_owned(),
            text,
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

/// One generated file: its path relative to the repository root and its bytes.
#[derive(Debug, Clone, PartialEq)]
pub struct Generated {
    pub path: String,
    pub contents: String,
}

/// Every file the IDL generates, in a fixed order.
pub fn generate(api: &Api) -> Vec<Generated> {
    let api = &with_guide_refs(api);
    let ix = Index::new(api);
    vec![
        Generated {
            path: "packages/engine/types/index.d.ts".to_string(),
            contents: r#gen::ts::dts(&ix),
        },
        Generated {
            path: "packages/engine/types/bindings.json".to_string(),
            contents: r#gen::tables::bindings(&ix),
        },
        Generated {
            path: "packages/engine/schema/api.schema.json".to_string(),
            contents: r#gen::tables::schema(&ix),
        },
        Generated {
            path: "packages/engine/src/generated/abi.ts".to_string(),
            contents: r#gen::host::abi_ts(&ix),
        },
        Generated {
            path: "packages/engine/src/generated/exports.ts".to_string(),
            contents: r#gen::ts::exports_ts(&ix),
        },
        Generated {
            path: "crates/og-api/src/generated/host.rs".to_string(),
            contents: r#gen::host::host_rs(&ix),
        },
        Generated {
            path: "crates/og-records/src/generated/records.rs".to_string(),
            contents: r#gen::host::records_rs(&ix),
        },
        Generated {
            path: "crates/og-runtime/src/host/generated/wire.rs".to_string(),
            contents: r#gen::wire::wire_rs(&ix),
        },
        Generated {
            path: "crates/og-host-web/src/generated/glue.rs".to_string(),
            contents: r#gen::host::glue(&ix, r#gen::host::Target::Web),
        },
        Generated {
            path: "crates/og-host-node/src/generated/glue.rs".to_string(),
            contents: r#gen::host::glue(&ix, r#gen::host::Target::Node),
        },
    ]
    .into_iter()
    .chain(wiki_pages(&ix))
    .collect()
}

/// The wiki pages alone (`og api gen --wiki`): one page per module and an
/// index, under [`WIKI_DIR`].
pub fn generate_wiki(api: &Api) -> Vec<Generated> {
    wiki_pages(&Index::new(&with_guide_refs(api)))
}

/// The API as the generators print it: each declaration's `guide` lines
/// become the last paragraph of its doc, "See `saves §2`.", so the `.d.ts`,
/// the schema and the wiki all point a reader at the guide.
fn with_guide_refs(api: &Api) -> Api {
    use model::{GuideRef, Item, Member};
    fn see(doc: &mut Vec<String>, refs: &[GuideRef]) {
        if !refs.is_empty() {
            let names: Vec<String> = refs.iter().map(|r| format!("`{r}`")).collect();
            doc.push(format!("See {}.", names.join(", ")));
        }
    }
    let mut api = api.clone();
    for m in &mut api.modules {
        see(&mut m.doc, &m.see);
        for item in &mut m.items {
            match item {
                Item::Fn(f) => see(&mut f.doc, &f.see),
                Item::Struct(s) => {
                    see(&mut s.doc, &s.see);
                    for f in &mut s.fields {
                        see(&mut f.doc, &f.see);
                    }
                }
                Item::Enum(e) => see(&mut e.doc, &e.see),
                Item::Union(u) => see(&mut u.doc, &u.see),
                Item::Interface(i) => {
                    see(&mut i.doc, &i.see);
                    for mem in &mut i.members {
                        match mem {
                            Member::Fn(f) => see(&mut f.doc, &f.see),
                            Member::Prop(p) => see(&mut p.doc, &p.see),
                        }
                    }
                }
                Item::Record(r) => {
                    see(&mut r.doc, &r.see);
                    for f in &mut r.fields {
                        see(&mut f.doc, &f.see);
                    }
                    for f in &mut r.methods {
                        see(&mut f.doc, &f.see);
                    }
                }
                _ => {}
            }
        }
    }
    api
}

/// Where the wiki is generated, relative to the repository root: the
/// knowledge package (`@origingame/engine-knowledge`) ships it.
pub const WIKI_DIR: &str = "packages/knowledge/wiki";

fn wiki_pages(ix: &Index<'_>) -> Vec<Generated> {
    let pages = r#gen::wiki::pages(ix);
    // `og api find`'s index of every entry on those pages, shipped beside them.
    // In page-name order, the order `og api find` reads the pages back in.
    let mut named: Vec<(&str, &str)> = pages
        .iter()
        .map(|(name, text)| (name.trim_end_matches(".md"), text.as_str()))
        .collect();
    named.sort();
    let entries: Vec<search::Entry> = named
        .into_iter()
        .flat_map(|(page, text)| search::entries(page, text))
        .collect();
    let index = Generated {
        path: format!("{WIKI_DIR}/{}", search::INDEX_FILE),
        contents: search::SearchIndex::build(&entries).to_text(),
    };
    pages
        .into_iter()
        .map(|(name, contents)| Generated {
            path: format!("{WIKI_DIR}/{name}"),
            contents,
        })
        .chain(std::iter::once(index))
        .collect()
}

/// Write generated files under `root`, creating directories as needed. A
/// file whose bytes are already what it would get is not rewritten: cargo
/// rebuilds on a modification time, so rewriting the hosts' Rust unchanged
/// would rebuild both hosts and the wasm for nothing. Returns the paths it
/// wrote.
pub fn write<'a>(root: &Path, files: &'a [Generated]) -> std::io::Result<Vec<&'a str>> {
    let mut written = Vec::new();
    for file in files {
        let path = root.join(&file.path);
        if std::fs::read(&path).is_ok_and(|have| have == file.contents.as_bytes()) {
            continue;
        }
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(&path, &file.contents)?;
        written.push(file.path.as_str());
    }
    Ok(written)
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
    let drift = write(&scratch, &files)
        .map(|_| compare(&scratch, root, &files))
        .map_err(|e| ReadError::Io(scratch.clone(), e));
    let _ = std::fs::remove_dir_all(&scratch);
    drift
}

fn compare(fresh: &Path, root: &Path, files: &[Generated]) -> Vec<Drift> {
    let mut drift = Vec::new();
    for file in files {
        let want = std::fs::read(fresh.join(&file.path)).unwrap_or_default();
        match std::fs::read(root.join(&file.path)) {
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
