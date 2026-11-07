//! `og i18n extract`: every og.i18n key a project's dialogues and quests
//! show (`.yarn`, `type quest`), with the arguments they pass and
//! where, checked against each string table (`type strings`): per locale,
//! the keys it lacks and the messages that take an argument nothing
//! passes. [`write_base`] adds the keys the base table lacks to it, each
//! as its own key for a writer to replace.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// A key the stories show.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Key {
    /// Base-language text extracted from Yarn, when present.
    pub text: Option<String>,
    /// The argument names passed, across every place it is shown.
    pub args: Vec<String>,
    /// Where it is shown: `assets/story/smith.yarn:12` (`:0` for a header line).
    pub sources: Vec<String>,
}

/// One string table against the stories' keys.
#[derive(Clone, Debug, PartialEq)]
pub struct Table {
    pub locale: String,
    pub base: bool,
    /// Its path under the project, `assets/strings/ja.ogt`.
    pub path: String,
    /// Keys the stories show that it lacks (a non-base table falls back to
    /// the base for them).
    pub missing: Vec<String>,
    /// `(key, argument)`: its message takes the argument, nothing passes it.
    pub unpassed: Vec<(String, String)>,
}

/// What [`extract`] found.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Extract {
    pub keys: BTreeMap<String, Key>,
    /// The dialogues and quests read, by path.
    pub stories: Vec<String>,
    /// Stories that do not parse: (path, error).
    pub errors: Vec<(String, String)>,
    pub tables: Vec<Table>,
}

impl Extract {
    /// Whether every table has every key, with its arguments passed.
    pub fn complete(&self) -> bool {
        self.errors.is_empty()
            && !self.tables.is_empty()
            && self
                .tables
                .iter()
                .all(|t| t.missing.is_empty() && t.unpassed.is_empty())
    }
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<PathBuf> = rd.filter_map(|e| e.ok().map(|e| e.path())).collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            walk(&p, out);
        } else if p.extension().is_some_and(|x| x == "ogt" || x == "yarn") {
            out.push(p);
        }
    }
}

/// Reads every `.ogt` under `<project>/assets`: the keys of its dialogues
/// and quests, and each string table checked against them.
pub fn extract(project: &Path) -> Result<Extract, String> {
    let assets = project.join("assets");
    if !assets.is_dir() {
        return Err(format!("{}: no assets/ directory", project.display()));
    }
    let mut files = Vec::new();
    walk(&assets, &mut files);
    let mut x = Extract::default();
    let mut tables = Vec::new();
    for f in files {
        let rel = f
            .strip_prefix(project)
            .unwrap_or(&f)
            .to_string_lossy()
            .replace('\\', "/");
        let Ok(text) = std::fs::read_to_string(&f) else {
            continue;
        };
        let ty = if f.extension().is_some_and(|x| x == "yarn") {
            og_story::DIALOGUE_TYPE.into()
        } else if let Ok(doc) = og_core::ogt::Document::parse(&text) {
            doc.ty
        } else {
            continue;
        };
        let keys = match ty.as_str() {
            t if t == og_story::DIALOGUE_TYPE => og_story::Dialogue::parse(&text).map(|d| {
                for (key, text) in &d.strings {
                    x.keys.entry(key.clone()).or_default().text = Some(text.clone());
                }
                d.keys()
            }),
            t if t == og_story::QUEST_TYPE => og_story::Quest::parse(&text).map(|q| q.keys()),
            t if t == og_i18n::STRINGS_TYPE => {
                match og_i18n::StringTable::parse(&text) {
                    Ok(t) => tables.push((rel, t)),
                    Err(e) => x.errors.push((rel, e.to_string())),
                }
                continue;
            }
            _ => continue,
        };
        match keys {
            Ok(keys) => {
                for (key, args, line) in keys {
                    let k = x.keys.entry(key).or_default();
                    for a in args {
                        if !k.args.contains(&a) {
                            k.args.push(a);
                        }
                    }
                    k.sources.push(format!("{rel}:{line}"));
                }
                x.stories.push(rel);
            }
            Err(e) => x.errors.push((rel, e)),
        }
    }
    let parsed: Vec<og_i18n::StringTable> = tables.iter().map(|(_, t)| t.clone()).collect();
    let base = og_i18n::base_table(&parsed).map(|t| t.locale.clone());
    for (path, t) in tables {
        let missing = x
            .keys
            .keys()
            .filter(|k| !t.entries.contains_key(*k))
            .cloned()
            .collect();
        let mut unpassed = Vec::new();
        for (key, k) in &x.keys {
            if let Some((_, m)) = t.entries.get(key) {
                for a in m.arguments() {
                    if !k.args.contains(&a) {
                        unpassed.push((key.clone(), a));
                    }
                }
            }
        }
        x.tables.push(Table {
            base: base.as_deref() == Some(t.locale.as_str()),
            locale: t.locale,
            path,
            missing,
            unpassed,
        });
    }
    Ok(x)
}

/// Adds the keys the base table lacks to it (each written as its own key,
/// with its arguments, for a writer to replace), creating
/// `assets/strings/en.ogt` as the base when the project has no table.
/// Answers the base table's path and how many keys it added.
pub fn write_base(project: &Path, x: &Extract) -> Result<(String, usize), String> {
    let (path, missing): (String, Vec<String>) = match x.tables.iter().find(|t| t.base) {
        Some(t) => (t.path.clone(), t.missing.clone()),
        None => (
            "assets/strings/en.ogt".to_string(),
            x.keys.keys().cloned().collect(),
        ),
    };
    if missing.is_empty() {
        return Ok((path, 0));
    }
    let file = project.join(&path);
    let mut text = match std::fs::read_to_string(&file) {
        Ok(t) => t,
        Err(_) => "type strings\nversion 1\n---\nlocale \"en\"\nbase true\n".to_string(),
    };
    if !text.ends_with('\n') {
        text.push('\n');
    }
    text.push_str("# og i18n extract: shown by the dialogues and quests; write each line\n");
    for k in &missing {
        let args: String = x.keys[k].args.iter().map(|a| format!(" {{{a}}}")).collect();
        let msg = x.keys[k]
            .text
            .clone()
            .unwrap_or_else(|| format!("{k}{args}"))
            .replace('\\', "\\\\")
            .replace('"', "\\\"");
        text.push_str(&format!("string \"{k}\" \"{msg}\"\n"));
    }
    og_i18n::StringTable::parse(&text).map_err(|e| format!("{path}: {e}"))?;
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::write(&file, text).map_err(|e| format!("{}: {e}", file.display()))?;
    Ok((path, missing.len()))
}
