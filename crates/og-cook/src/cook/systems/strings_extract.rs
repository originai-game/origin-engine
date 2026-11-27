//! `og i18n extract`: every og.i18n key a project's dialogues, quests and
//! tables show (`file.dialogue`, `file.quest`, `file.table`), with the arguments they pass and
//! where, checked against each typed `file.strings` table: per locale,
//! the keys it lacks and the messages that take an argument nothing
//! passes. [`write_base`] adds the keys the base table lacks to it, each
//! as its own key for a writer to replace.

use crate::cook::{
    generator, source_content, source_index, source_project, source_schemas, source_transaction,
};
use std::collections::BTreeMap;
use std::path::Path;

/// Read a shipped declaration in the same closed evaluator as cook. Project
/// entries must instead use the module-aware reader below.
pub fn read_builtin(root: &Path, file: &str) -> Result<og_i18n::StringTable, String> {
    let program = generator::Program::prepare_value(root, file)?;
    let value = program.run()?;
    let declaration: source_schemas::File =
        serde_json::from_value(value).map_err(|e| e.to_string())?;
    declaration.check(None).map_err(|e| e.to_string())?;
    if declaration.kind != "strings" {
        return Err(format!("{file}: expected file.strings"));
    }
    super::strings::declaration(
        serde_json::from_value(declaration.value).map_err(|e| e.to_string())?,
    )
}

/// Discover every locale across project modules without executing game code.
/// Paths and font refs retain their author identities for diagnostics/editing.
pub fn read_tables(
    root: &Path,
    project: &source_project::Project,
    index: &source_index::Index,
) -> Result<Vec<(String, og_i18n::StringTable)>, String> {
    let mut evaluator = source_content::Evaluator::default();
    index
        .entries
        .values()
        .filter(|e| e.factory == "strings")
        .map(|entry| {
            let content = evaluator
                .evaluate(root, project, entry, &BTreeMap::new(), None)
                .map_err(|e| e.to_string())?;
            let file: source_schemas::File =
                serde_json::from_value(content.data().root().into_json())
                    .map_err(|e| e.to_string())?;
            let input = serde_json::from_value(file.value).map_err(|e| e.to_string())?;
            let table = super::strings::declaration(input).map_err(|e| {
                source_content::diagnostic(content.source(), &entry.file, "", e).to_string()
            })?;
            Ok((entry.file.clone(), table))
        })
        .collect()
}

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
    /// Its path under the project, `content/strings/ja.ts`.
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

/// Read only explicit story declarations and module-aware typed locale tables.
pub fn extract(project: &Path) -> Result<Extract, String> {
    let source = crate::cook::source_project::Project::open(project).map_err(|e| e.to_string())?;
    let functions = super::story_functions::read(&source.input.story_functions)?;
    let mut x = Extract::default();
    let index = source_index::Index::open(project, &source).map_err(|e| e.to_string())?;
    let tables = read_tables(project, &source, &index)?;
    let mut evaluator = source_content::Evaluator::default();
    for entry in index
        .entries
        .values()
        .filter(|e| matches!(e.factory.as_str(), "dialogue" | "quest"))
    {
        let rel = entry.file.clone();
        let keys = (|| {
            let content = evaluator
                .evaluate(project, &source, entry, &BTreeMap::new(), None)
                .map_err(|e| e.to_string())?;
            let file: source_schemas::File =
                serde_json::from_value(content.data().root().into_json())
                    .map_err(|e| e.to_string())?;
            let asset = if file.kind == "dialogue" {
                let input: source_schemas::media::story::Dialogue =
                    serde_json::from_value(file.value).map_err(|e| e.to_string())?;
                input.build(functions.clone())?
            } else {
                let input: source_schemas::media::story::Quest =
                    serde_json::from_value(file.value).map_err(|e| e.to_string())?;
                og_story::asset::Asset::Quest(input.into())
            };
            asset.validate()?;
            Ok::<_, String>(match asset {
                og_story::asset::Asset::Dialogue(d) => {
                    for (key, text) in &d.strings {
                        x.keys.entry(key.clone()).or_default().text = Some(text.clone());
                    }
                    d.keys()
                }
                og_story::asset::Asset::Quest(q) => q.keys(),
            })
        })();
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
    let records = crate::cook::source_records::Records::open(project, &source)?;
    for entry in index.entries.values().filter(|e| e.factory == "table") {
        let result = evaluator.evaluate(
            project,
            &source,
            entry,
            &BTreeMap::new(),
            records.contexts[&entry.module].schema("table"),
        );
        match result {
            Ok(content) => {
                let file: source_schemas::File =
                    serde_json::from_value(content.data().root().into_json())
                        .map_err(|e| e.to_string())?;
                let input: source_schemas::world::table::TableInput =
                    serde_json::from_value(file.value).map_err(|e| e.to_string())?;
                for (i, row) in input.rows.iter().enumerate() {
                    for (name, value) in &row.values {
                        if let Some(text) = value.get("text") {
                            let key = text["key"].as_str().expect("checked text");
                            let base = text["base"].as_str().expect("checked text");
                            let k = x.keys.entry(key.into()).or_default();
                            k.text = Some(base.into());
                            let pointer = format!(
                                "/rows/{i}/values/{}",
                                name.replace('~', "~0").replace('/', "~1")
                            );
                            let at = source_content::diagnostic(
                                content.source(),
                                &entry.file,
                                &pointer,
                                "",
                            );
                            k.sources
                                .push(format!("{}:{}", entry.file, at.location.line));
                        }
                    }
                }
                x.stories.push(entry.file.clone());
            }
            Err(e) => x.errors.push((entry.file.clone(), e.to_string())),
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
/// `content/strings/en.ts` as the base when the project has no table.
/// Answers the base table's path and how many keys it added.
pub fn write_base(project: &Path, x: &Extract) -> Result<(String, usize), String> {
    let (path, missing): (String, Vec<String>) = match x.tables.iter().find(|t| t.base) {
        Some(t) => (t.path.clone(), t.missing.clone()),
        None => (
            "content/strings/en.ts".to_string(),
            x.keys.keys().cloned().collect(),
        ),
    };
    if missing.is_empty() {
        return Ok((path, 0));
    }
    let mut entries = BTreeMap::new();
    for k in &missing {
        let args: String = x.keys[k].args.iter().map(|a| format!(" {{{a}}}")).collect();
        let msg = x.keys[k]
            .text
            .clone()
            .unwrap_or_else(|| format!("{k}{args}"));
        og_i18n::Message::parse(&msg).map_err(|e| format!("{path}: {k}: {e}"))?;
        entries.insert(k.clone(), msg);
    }
    if x.tables.iter().any(|t| t.path == path) {
        let configuration = source_project::Project::open(project).map_err(|e| e.to_string())?;
        let index =
            source_index::Index::open(project, &configuration).map_err(|e| e.to_string())?;
        let entry = index
            .entries
            .values()
            .find(|e| e.file == path)
            .ok_or_else(|| format!("{path}: table no longer exists; extract again"))?;
        let content = source_content::Evaluator::default()
            .evaluate(project, &configuration, entry, &BTreeMap::new(), None)
            .map_err(|e| e.to_string())?;
        let edits: Vec<_> = entries
            .into_iter()
            .map(|(key, message)| generator::source::Edit {
                path: format!("/entries/{}", key.replace('~', "~0").replace('/', "~1")),
                expected: None,
                value: message.into(),
            })
            .collect();
        let source = content.source();
        let current = std::fs::read_to_string(project.join(&path)).map_err(|e| e.to_string())?;
        let text = source.patch(&current, &edits).map_err(|e| e.to_string())?;
        source_transaction::commit(
            project,
            &[source_transaction::FileEdit {
                file: path.clone(),
                digest: source.digest(),
                text,
            }],
            |files| {
                let overlay = files
                    .iter()
                    .map(|f| (f.file.clone(), f.text.clone()))
                    .collect();
                let candidate =
                    source_index::Index::open_with_overlay(project, &configuration, &overlay)
                        .map_err(|e| vec![e])?;
                source_content::Evaluator::default()
                    .evaluate(
                        project,
                        &configuration,
                        &candidate.entries[&entry.id],
                        &overlay,
                        None,
                    )
                    .map(|_| ())
                    .map_err(|e| vec![e])
            },
        )
        .map_err(|e| {
            e.iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        })?;
    } else {
        use std::io::Write;
        let value = serde_json::json!({"locale":"en", "base":true, "entries":entries});
        let text = format!(
            "import {{ file }} from '@origingame/engine/author';\nexport default file.strings({});\n",
            serde_json::to_string_pretty(&value).expect("string table JSON")
        );
        let file = project.join(&path);
        if let Some(dir) = file.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&file)
            .and_then(|mut f| f.write_all(text.as_bytes()))
            .map_err(|e| format!("{}: {e}", file.display()))?;
    }
    Ok((path, missing.len()))
}
