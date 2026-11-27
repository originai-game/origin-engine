//! Checked TS declarations enter look composition before binary asset discovery.
use super::*;
use crate::cook::{generator, source_content, source_index, source_project, source_schemas};
use serde_json::Value as Json;
use std::collections::BTreeMap;

/// Engine presets are embedded TS, evaluated by the ordinary closed realm.
pub(super) fn builtin(id: &str, text: &str) -> Result<LookFile, String> {
    let file = format!("{id}.ts");
    let sources = BTreeMap::from([(file.clone(), text.into())]);
    let program = generator::Program::prepare_embedded(&sources, &file)?;
    let value: source_schemas::File =
        serde_json::from_value(program.run()?).map_err(|e| format!("{file}: {e}"))?;
    value.check(None).map_err(|e| format!("{file}: {e}"))?;
    if value.kind != "look" {
        return Err(format!(
            "{file}: an engine look layer must export file.look"
        ));
    }
    declaration(&value.value, &|_| 0)
}

/// A canonical checked look patch, not a text serialization or parser.
pub(super) fn declaration(value: &Json, line: &dyn Fn(&str) -> usize) -> Result<LookFile, String> {
    fn identity(value: &Json) -> Option<String> {
        value
            .as_str()
            .or_else(|| value.get("$ogRef")?.as_str())
            .map(str::to_owned)
    }
    fn scalar(value: &Json, word: bool) -> Result<Value, String> {
        match value {
            Json::Number(n) => Number::from_f64(n.as_f64().ok_or("look number is not finite")?)
                .map(Value::Number)
                .map_err(|e| e.to_string()),
            Json::Bool(b) => Ok(Value::Word(b.to_string())),
            Json::String(s) if word => Ok(Value::Word(s.clone())),
            _ => identity(value)
                .map(Value::String)
                .ok_or_else(|| "invalid checked look value".into()),
        }
    }
    let text = |key| value[key].as_str().map(str::to_owned);
    let mut look = LookFile {
        name: text("name"),
        mood: text("mood"),
        variant: text("variant"),
        ui: identity(&value["ui"]),
        antialias: text("antialias"),
        extends: value["extends"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|v| identity(v).ok_or_else(|| "invalid checked look base".into()))
            .collect::<Result<_, String>>()?,
        ui_seeds: value["uiSeeds"]
            .as_object()
            .into_iter()
            .flatten()
            .filter_map(|(key, value)| value.as_str().map(|v| (key.clone(), v.into())))
            .collect(),
        blocks: Vec::new(),
    };
    for resource in crate::look::LOOK_RESOURCES {
        let Some(fields) = value[resource].as_object() else {
            continue;
        };
        let record = og_records::RECORDS
            .iter()
            .find(|r| r.name == resource)
            .ok_or_else(|| format!("unknown look resource {resource}"))?;
        let mut block = crate::look::LookBlock {
            resource: resource.into(),
            fields: Vec::new(),
        };
        for (name, value) in fields {
            let field = record
                .fields
                .iter()
                .find(|f| f.name == name)
                .ok_or_else(|| format!("unknown look field {resource}.{name}"))?;
            let words = !field.variants.is_empty();
            let values = match value.as_array() {
                Some(values) => values
                    .iter()
                    .map(|v| scalar(v, words))
                    .collect::<Result<_, _>>()?,
                None => vec![scalar(value, words)?],
            };
            block
                .fields
                .push((name.clone(), values, line(&format!("/{resource}/{name}"))));
        }
        look.blocks.push(block);
    }
    Ok(look)
}

/// Retain each evaluated entry for the complete content graph. Composition
/// sees canonical references; binding to produced asset IDs happens later.
pub(crate) struct Prepared {
    pub content: BTreeMap<String, source_content::Content>,
    pub looks: BTreeMap<String, LookFile>,
}

pub(crate) fn project(
    root: &Path,
    project: &source_project::Project,
) -> Result<Prepared, CookError> {
    let index = source_index::Index::open(root, project).map_err(|e| CookError(e.to_string()))?;
    let mut evaluator = source_content::Evaluator::default();
    let mut content = BTreeMap::new();
    let mut patches = BTreeMap::new();
    for entry in index.entries.values().filter(|e| e.factory == "look") {
        let checked = evaluator
            .evaluate(root, project, entry, &BTreeMap::new(), None)
            .map_err(|e| CookError(e.to_string()))?;
        let file: source_schemas::File = serde_json::from_value(checked.data().root().into_json())
            .map_err(|e| CookError(e.to_string()))?;
        let at = |path: &str| source_content::diagnostic(checked.source(), &entry.file, path, "");
        let patch = declaration(&file.value, &|path| at(path).location.line)
            .map_err(|e| CookError(at("").to_string() + &e))?;
        patches.insert(entry.id.clone(), patch);
        content.insert(entry.id.clone(), checked);
    }
    let mut looks = BTreeMap::new();
    for id in patches.keys() {
        let look = flatten(id, &patches).map_err(|e| {
            CookError(
                source_content::diagnostic(
                    content[id].source(),
                    &index.entries[id].file,
                    "/extends",
                    e,
                )
                .to_string(),
            )
        })?;
        looks.insert(id.clone(), look);
    }
    if let Some(source_schemas::procedural::LookBase::Preset(slug)) = &project.metadata.look {
        let id = format!("@looks/{slug}");
        let patch = BTreeMap::from([(
            id.clone(),
            LookFile {
                extends: vec![slug.clone()],
                ..Default::default()
            },
        )]);
        looks.insert(id.clone(), flatten(&id, &patch).map_err(CookError)?);
    }
    Ok(Prepared { content, looks })
}
