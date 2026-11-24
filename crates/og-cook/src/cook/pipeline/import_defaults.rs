//! A directory's import settings: `import.ogt`, a `type import` document
//! of `match "<pattern>" { … }` blocks, each holding what a file's own
//! settings (`<file>.ogt`) hold, for every file under the directory its
//! patterns name (paths from the directory, read as project.ogt's
//! `conform` patterns are: `conform::matches`).
//!
//! A file's settings are the matching blocks of every `import.ogt` from
//! `assets/` down to its own directory, in that order and in each file's
//! order, then its own settings file; a later one replaces an earlier
//! one's entries key by key, and two blocks of one key join the same way.

use og_core::ogt::{Block, Document};
use std::path::Path;

/// The file's name in a directory.
pub(crate) const FILE: &str = "import.ogt";

/// One `import.ogt`.
pub(crate) struct Defaults {
    /// Its id (`textures/import.ogt`).
    rel: String,
    /// Its directory with a trailing `/`, empty for `assets/` itself.
    dir: String,
    rules: Vec<(Vec<String>, Block)>,
}

/// Whether `rel` is a directory's settings by its name.
pub(crate) fn is_defaults(rel: &str) -> bool {
    rel == FILE || rel.ends_with("/import.ogt")
}

impl Defaults {
    /// How far under `assets/` its directory is.
    pub(crate) fn depth(&self) -> usize {
        self.dir.matches('/').count()
    }

    pub(crate) fn parse(rel: &str, doc: &Document) -> Result<Defaults, String> {
        let mut rules = Vec::new();
        for e in &doc.body.entries {
            if e.key != "match" {
                return Err(format!(
                    "line {}: a directory's import settings hold `match \"<pattern>\" {{ … }}` blocks, and this is '{}' (a file's own settings go in <file>.ogt, or in a match block naming it)",
                    e.line, e.key
                ));
            }
            let patterns: Vec<String> = e
                .values
                .iter()
                .map(|v| v.as_str().map(str::to_string))
                .collect::<Option<_>>()
                .filter(|p: &Vec<String>| !p.is_empty())
                .ok_or_else(|| {
                    format!(
                        "line {}: match takes one or more patterns as strings, paths from this directory as project.ogt's conform patterns are (\"*.png\" its own files, \"**.png\" at any depth, \"tiles/\" a folder)",
                        e.line
                    )
                })?;
            let block = e.block.clone().ok_or_else(|| {
                format!("line {}: match takes a block of import settings", e.line)
            })?;
            rules.push((patterns, block));
        }
        Ok(Defaults {
            rel: rel.to_string(),
            dir: rel.strip_suffix(FILE).unwrap_or_default().to_string(),
            rules,
        })
    }
}

/// `from` over `into`: its entries replace `into`'s of the same key, and
/// where each side has the key once as a bare block, the blocks join.
fn merge(into: &mut Block, from: &Block) {
    let mut seen: Vec<&str> = Vec::new();
    for e in &from.entries {
        if seen.contains(&e.key.as_str()) {
            into.entries.push(e.clone());
            continue;
        }
        seen.push(&e.key);
        let once = from.entries.iter().filter(|f| f.key == e.key).count() == 1;
        let mut old = into.entries.iter_mut().filter(|o| o.key == e.key);
        match (old.next(), old.next(), &e.block) {
            (Some(o), None, Some(b)) if once && e.values.is_empty() && o.values.is_empty() => {
                match &mut o.block {
                    Some(ob) => merge(ob, b),
                    None => o.block = Some(b.clone()),
                }
            }
            _ => {
                into.entries.retain(|o| o.key != e.key);
                into.entries.push(e.clone());
            }
        }
    }
}

/// The settings of `rel`: `own` (its `<file>.ogt`) over what `defaults`
/// (sorted from `assets/` down) give it, and the files they came from. With
/// no directory settings for it, `own` as it is. The engine's built-in
/// assets (`@…`) keep their own.
pub(crate) fn effective(
    rel: &str,
    defaults: &[Defaults],
    own: Option<(String, Document)>,
) -> (Option<(String, Document)>, Vec<String>) {
    let own_file = own.as_ref().map(|_| format!("{rel}.ogt"));
    let mut body = Block::new();
    let mut files = Vec::new();
    for d in defaults {
        let Some(path) = rel.strip_prefix(&d.dir).filter(|_| !rel.starts_with('@')) else {
            continue;
        };
        for (patterns, block) in &d.rules {
            if patterns.iter().any(|p| super::conform::matches(p, path)) {
                merge(&mut body, block);
                if files.last() != Some(&d.rel) {
                    files.push(d.rel.clone());
                }
            }
        }
    }
    if files.is_empty() {
        return (own, own_file.into_iter().collect());
    }
    let mut doc = Document::new("import", 1);
    if let Some((_, own)) = &own {
        doc.version = own.version;
        merge(&mut body, &own.body);
    }
    doc.body = body;
    files.extend(own_file);
    // The text is what a source's hash takes: the settings as they apply,
    // so an edit to another pattern's block leaves this file as it was.
    let text = doc.to_text().unwrap_or_else(|_| format!("{doc:?}"));
    (Some((text, doc)), files)
}

/// The `import.ogt` ids that could apply to `rel`, from `assets/` down.
pub(crate) fn candidates(rel: &str) -> Vec<String> {
    let mut out = vec![FILE.to_string()];
    for (i, _) in rel.match_indices('/') {
        out.push(format!("{}/{FILE}", &rel[..i]));
    }
    out
}

/// The directory settings above `rel` read from `root` (`assets/`), for a
/// file the cook reads outside its scan (an animation library).
pub(crate) fn read(root: &Path, rel: &str) -> Result<Vec<Defaults>, String> {
    let mut out = Vec::new();
    for id in candidates(rel) {
        // Beside a file named `import`, it is that file's own settings.
        if root.join(id.trim_end_matches(".ogt")).is_file() {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(root.join(&id)) else {
            continue;
        };
        let doc = Document::parse(&text).map_err(|e| format!("{id}: {e}"))?;
        if doc.ty == "import" {
            out.push(Defaults::parse(&id, &doc).map_err(|e| format!("{id}: {e}"))?);
        }
    }
    Ok(out)
}
