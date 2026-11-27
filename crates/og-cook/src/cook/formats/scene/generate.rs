//! Typed declarations become ordinary scene text before reference planning,
//! prefab expansion and reflected component/resource packing.
use super::{SCENE, Source};
use crate::cook::{assets_root, generator};
use og_core::ogt::{Block, Document, Entry, Number, Value};
use serde::Deserialize;
use serde_json::{Map, Value as Json};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Scene {
    entities: Vec<Entity>,
    #[serde(default)]
    resources: Map<String, Json>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entity {
    name: String,
    model: Option<String>,
    scene: Option<String>,
    prefab: Option<String>,
    #[serde(rename = "static")]
    static_model: Option<bool>,
    #[serde(default)]
    components: Map<String, Json>,
    #[serde(default)]
    children: Vec<Entity>,
}

fn scalar(value: Json) -> Result<Value, String> {
    match value {
        Json::Bool(v) => Ok(Value::Word(v.to_string())),
        Json::String(v) => Ok(Value::String(v)),
        Json::Number(v) => Number::from_f64(v.as_f64().ok_or("a finite scene number")?)
            .map(Value::Number)
            .map_err(|e| e.to_string()),
        _ => Err("scene fields contain a number, boolean, string or a flat array of them".into()),
    }
}

fn fields(value: Json) -> Result<Block, String> {
    let Json::Object(fields) = value else {
        return Err("a generated component or resource is an object of fields".into());
    };
    let mut block = Block::new();
    for (name, value) in fields {
        let mut entry = Entry::new(name);
        let values = match value {
            Json::Array(values) => values,
            v => vec![v],
        };
        entry.values = values.into_iter().map(scalar).collect::<Result<_, _>>()?;
        block.push(entry);
    }
    Ok(block)
}

fn entity(e: Entity) -> Result<Entry, String> {
    let mut body = Block::new();
    for (key, value) in [("model", e.model), ("scene", e.scene), ("prefab", e.prefab)] {
        if let Some(value) = value {
            body.push(Entry::new(key).with(Value::String(value)));
        }
    }
    if let Some(value) = e.static_model {
        body.push(Entry::new("static").with(Value::Word(value.to_string())));
    }
    for (name, value) in e.components {
        if matches!(
            name.as_str(),
            "entity" | "model" | "scene" | "prefab" | "static" | "override" | "remove"
        ) {
            return Err(format!("'{name}' is scene syntax, not a component name"));
        }
        body.push(Entry::new(name).with_block(fields(value)?));
    }
    for child in e.children {
        body.push(entity(child)?);
    }
    Ok(Entry::new("entity")
        .with(Value::String(e.name))
        .with_block(body))
}

fn make(s: &Source) -> Result<Option<Vec<u8>>, String> {
    if s.kind != &SCENE || !s.rel.ends_with(".ogt") || s.error.is_some() {
        return Ok(None);
    }
    let mut doc = Document::parse(&String::from_utf8_lossy(&s.bytes)).map_err(|e| e.to_string())?;
    if doc.ty == "scatter" {
        return super::scatter::expand(s, &doc)?
            .to_text()
            .map(|text| Some(text.into_bytes()))
            .map_err(|e| e.to_string());
    }
    let entries: Vec<_> = doc.body.get_all("generate").collect();
    if entries.is_empty() {
        return Ok(None);
    }
    if entries.len() != 1 || !entries[0].values.is_empty() {
        return Err("a scene has one generate block".into());
    }
    let spec =
        generator::Spec::standalone(entries[0].block.as_ref().ok_or("generate is a block")?)?;
    let program = generator::Program::prepare(&assets_root(s), &s.rel, &spec)?;
    let scene: Scene = serde_json::from_value(program.run()?).map_err(|e| e.to_string())?;
    let mut generated = Block::new();
    for e in scene.entities {
        generated.push(entity(e)?);
    }
    for (name, value) in scene.resources {
        if doc.body.get_all("resource").any(|e| {
            e.values
                .first()
                .and_then(|v| v.as_word().or_else(|| v.as_str()))
                == Some(name.as_str())
        }) {
            return Err(format!(
                "generated resource '{name}' is also authored in the scene"
            ));
        }
        generated.push(
            Entry::new("resource")
                .with(Value::String(name))
                .with_block(fields(value)?),
        );
    }
    doc.body.entries.retain(|e| e.key != "generate");
    doc.body.entries.extend(generated.entries);
    doc.to_text()
        .map(|text| Some(text.into_bytes()))
        .map_err(|e| e.to_string())
}

pub(crate) fn expand(sources: &mut [Source]) {
    for s in sources {
        match make(s) {
            Ok(Some(bytes)) => s.bytes = bytes,
            Ok(None) => {}
            Err(e) => s.error = Some(e),
        }
    }
}
