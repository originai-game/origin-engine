//! Scene `.ogt` → scene blobs: entities, model instances and one packed
//! component table per reflected type.
//!
//! ```text
//! type scene
//! version 1
//! ---
//! name "Main"
//! entity "ship" {
//!   model "models/dropship.glb"
//!   Transform {
//!     position 0 1 0
//!     scale 2 2 2
//!   }
//!   entity "light" {
//!     Transform {
//!       position 0 3 0
//!     }
//!   }
//! }
//! ```
//!
//! Inside an `entity` block: `entity` nests a child, `model "<asset id>"`
//! instantiates a cooked glTF under the entity, and any other key with a
//! block is a component whose entries are its fields (numbers, the words
//! `true`/`false`, or one quoted string for a `strN` field: `name "Mei"`,
//! packed as its UTF-8 then zeros, refused over the capacity). With the
//! component's schema (`CookOptions::schemas`) the fields are checked,
//! ordered and defaulted from it; without one the cook infers a layout from
//! the text (f32 lanes; a string needs the schema for its capacity) and
//! every entity must set the same fields.

use super::CookError;
use crate::format::{self, *};
use og_blob::BlobWriter;
use og_core::ogt::{Block, Document, Value};
use og_core::{FieldSchema, FieldType, Layout, Schema};
use std::collections::{BTreeMap, BTreeSet};

struct Entity {
    name: String,
    parent: u32,
    instance: u32,
}

struct Pending {
    entity: u32,
    line: usize,
    fields: Vec<(String, Given, usize)>,
}

/// A field's value as the scene text gives it.
enum Given {
    /// Numbers (and `true`/`false` as 1/0), one per lane.
    Lanes(Vec<f64>),
    /// One quoted string: a `strN` field's text.
    Text(String),
}

impl Given {
    fn len(&self) -> usize {
        match self {
            Given::Lanes(v) => v.len(),
            Given::Text(_) => 1,
        }
    }
}

fn lanes(values: &[Value], line: usize, comp: &str, field: &str) -> Result<Given, CookError> {
    if let [Value::String(text)] = values {
        return Ok(Given::Text(text.clone()));
    }
    values
        .iter()
        .map(|v| match v {
            Value::Number(n) => Ok(n.as_f64()),
            Value::Word(w) if w == "true" => Ok(1.0),
            Value::Word(w) if w == "false" => Ok(0.0),
            _ => Err(CookError(format!(
                "line {line}: {comp}.{field}: values are numbers, true/false, or one quoted string for a string field"
            ))),
        })
        .collect::<Result<Vec<f64>, CookError>>()
        .map(Given::Lanes)
}

fn infer(name: &str, rows: &[Pending]) -> Result<Schema, CookError> {
    let mut fields: Vec<FieldSchema> = Vec::new();
    for r in rows {
        for (f, v, line) in &r.fields {
            if let Given::Text(_) = v {
                return Err(CookError(format!(
                    "line {line}: {name}.{f} is a string; without {name}'s schema the cook cannot know its capacity (pass the schema set to the cook)"
                )));
            }
            let ty = match v.len() {
                1 => FieldType::F32,
                2 => FieldType::Vec2,
                3 => FieldType::Vec3,
                4 => FieldType::Vec4,
                n => {
                    return Err(CookError(format!(
                        "line {line}: {name}.{f}: {n} values; without a schema a field has 1 to 4"
                    )));
                }
            };
            match fields.iter().find(|x| &x.name == f) {
                Some(x) if x.ty != ty => {
                    return Err(CookError(format!(
                        "line {line}: {name}.{f} has {} value(s) here and {} elsewhere",
                        v.len(),
                        x.ty.lanes()
                    )));
                }
                Some(_) => {}
                None => fields.push(FieldSchema::new(f.clone(), ty)),
            }
        }
    }
    for r in rows {
        for f in &fields {
            if !r.fields.iter().any(|(n, ..)| n == &f.name) {
                return Err(CookError(format!(
                    "line {}: {name} omits '{}' that another entity sets; without {name}'s schema the cook cannot know its default (pass the schema set to the cook)",
                    r.line, f.name
                )));
            }
        }
    }
    let schema = Schema {
        name: name.to_string(),
        version: 1,
        doc: String::new(),
        fields,
    };
    schema
        .validate()
        .map_err(|e| CookError(format!("{name}: {e}")))?;
    Ok(schema)
}

fn pack(schema: &Schema, row: &Pending, out: &mut Vec<u8>) -> Result<(), CookError> {
    let comp = &schema.name;
    for (f, _, line) in &row.fields {
        if schema.field_index(f).is_none() {
            return Err(CookError(format!("line {line}: {comp} has no field '{f}'")));
        }
    }
    for fs in &schema.fields {
        let given = row.fields.iter().find(|(n, ..)| n == &fs.name);
        if fs.ty.is_string() {
            match given {
                Some((_, Given::Text(text), line)) => out.extend_from_slice(
                    &og_core::pack_str(text, fs.ty.size())
                        .map_err(|e| CookError(format!("line {line}: {comp}.{}: {e}", fs.name)))?,
                ),
                Some((_, Given::Lanes(_), line)) => {
                    return Err(CookError(format!(
                        "line {line}: {comp}.{} is a {}: give it one quoted string",
                        fs.name,
                        fs.ty.name()
                    )));
                }
                None => out.resize(out.len() + fs.ty.size(), 0),
            }
            continue;
        }
        let values: Vec<f64> = match given {
            Some((_, Given::Text(_), line)) => {
                return Err(CookError(format!(
                    "line {line}: {comp}.{} is a {}, not a string: give it {} number(s)",
                    fs.name,
                    fs.ty.name(),
                    fs.ty.lanes()
                )));
            }
            Some((_, Given::Lanes(v), line)) => {
                if v.len() != fs.ty.lanes() {
                    return Err(CookError(format!(
                        "line {line}: {comp}.{} is a {} ({} value(s)), got {}",
                        fs.name,
                        fs.ty.name(),
                        fs.ty.lanes(),
                        v.len()
                    )));
                }
                v.clone()
            }
            None => fs
                .default
                .clone()
                .unwrap_or_else(|| vec![0.0; fs.ty.lanes()]),
        };
        let line = given.map_or(row.line, |g| g.2);
        for v in values {
            let int = |lo: f64, hi: f64| -> Result<f64, CookError> {
                if v.fract() != 0.0 || v < lo || v > hi {
                    Err(CookError(format!(
                        "line {line}: {comp}.{}: {v} is not a {}",
                        fs.name,
                        fs.ty.name()
                    )))
                } else {
                    Ok(v)
                }
            };
            match fs.ty {
                FieldType::F64 => out.extend_from_slice(&v.to_le_bytes()),
                FieldType::I32 => out.extend_from_slice(
                    &(int(i32::MIN as f64, i32::MAX as f64)? as i32).to_le_bytes(),
                ),
                FieldType::U32 => {
                    out.extend_from_slice(&(int(0.0, u32::MAX as f64)? as u32).to_le_bytes())
                }
                FieldType::I64 => {
                    out.extend_from_slice(&(int(-9.007e15, 9.007e15)? as i64).to_le_bytes())
                }
                FieldType::U64 => {
                    out.extend_from_slice(&(int(0.0, 9.007e15)? as u64).to_le_bytes())
                }
                // og-core packs a bool in one byte and an enum as its u32
                // variant index; four f32 bytes would break the row stride.
                FieldType::Bool => out.push(int(0.0, 1.0)? as u8),
                FieldType::Enum => {
                    out.extend_from_slice(&(int(0.0, u32::MAX as f64)? as u32).to_le_bytes())
                }
                _ => out.extend_from_slice(&(v as f32).to_le_bytes()),
            }
        }
    }
    Ok(())
}

/// The asset ids a scene references (for its stamp and the planner).
pub fn references(text: &str) -> Vec<String> {
    let mut out = BTreeSet::new();
    fn walk(b: &Block, out: &mut BTreeSet<String>) {
        for e in &b.entries {
            if e.key == "model"
                && let Ok(s) = e.string(0)
            {
                out.insert(crate::runtime::asset_id(s).to_string());
            }
            if e.key == "entity"
                && let Some(b) = &e.block
            {
                walk(b, out);
            }
        }
    }
    if let Ok(doc) = Document::parse(text) {
        walk(&doc.body, &mut out);
    }
    out.into_iter().collect()
}

/// Cooks a scene document. `known` says whether an asset id is cooked.
pub fn cook(
    doc: &Document,
    schemas: &BTreeMap<String, Schema>,
    known: &dyn Fn(&str) -> bool,
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    if doc.ty != "scene" {
        return Err(CookError(format!(
            "expected a 'scene' document, got '{}'",
            doc.ty
        )));
    }
    if doc.version != 1 {
        return Err(CookError(format!(
            "scene version {} is not supported (1)",
            doc.version
        )));
    }
    let mut notes = Vec::new();
    let mut strings: Vec<u8> = Vec::new();
    let mut add = |s: &str| {
        let at = strings.len() as u32;
        strings.extend_from_slice(s.as_bytes());
        (at, s.len() as u32)
    };
    let mut entities: Vec<Entity> = Vec::new();
    let mut refs: Vec<String> = Vec::new();
    let mut tables: BTreeMap<String, Vec<Pending>> = BTreeMap::new();
    let mut name = String::new();
    for e in &doc.body.entries {
        match e.key.as_str() {
            "name" => {
                name = e
                    .string(0)
                    .map_err(|x| CookError(format!("line {}: {x}", e.line)))?
                    .to_string()
            }
            "entity" => {}
            // Streaming groups: read by the cook's planner (cook/stream.rs).
            "stream" => {}
            k => {
                return Err(CookError(format!(
                    "line {}: unknown scene entry '{k}' (expected name, entity or stream)",
                    e.line
                )));
            }
        }
    }
    // Depth-first, parents before children.
    let empty = Block::new();
    let mut stack: Vec<(&og_core::ogt::Entry, u32)> = doc
        .body
        .get_all("entity")
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|e| (e, NONE))
        .collect();
    while let Some((e, parent)) = stack.pop() {
        let line = e.line;
        let ename = e
            .string(0)
            .map_err(|x| CookError(format!("line {line}: entity: {x}")))?
            .to_string();
        let index = entities.len() as u32;
        let mut entity = Entity {
            name: ename,
            parent,
            instance: NONE,
        };
        let block = e.block.as_ref().unwrap_or(&empty);
        let mut children = Vec::new();
        for c in &block.entries {
            match (c.key.as_str(), &c.block) {
                ("entity", _) => children.push(c),
                ("model", None) => {
                    let id = crate::runtime::asset_id(
                        c.string(0)
                            .map_err(|x| CookError(format!("line {}: model: {x}", c.line)))?,
                    )
                    .to_string();
                    if !known(&id) {
                        return Err(CookError(format!(
                            "line {}: model '{id}' is not an asset of this project",
                            c.line
                        )));
                    }
                    let r = match refs.iter().position(|r| r == &id) {
                        Some(r) => r,
                        None => {
                            refs.push(id);
                            refs.len() - 1
                        }
                    };
                    entity.instance = r as u32;
                }
                (comp, Some(fields)) => {
                    let mut pending = Pending {
                        entity: index,
                        line: c.line,
                        fields: Vec::new(),
                    };
                    for f in &fields.entries {
                        if f.block.is_some() {
                            return Err(CookError(format!(
                                "line {}: {comp}.{}: fields do not nest",
                                f.line, f.key
                            )));
                        }
                        if pending.fields.iter().any(|(n, ..)| n == &f.key) {
                            return Err(CookError(format!(
                                "line {}: {comp}.{} is set twice",
                                f.line, f.key
                            )));
                        }
                        pending.fields.push((
                            f.key.clone(),
                            lanes(&f.values, f.line, comp, &f.key)?,
                            f.line,
                        ));
                    }
                    let rows = tables.entry(comp.to_string()).or_default();
                    if rows.iter().any(|r| r.entity == index) {
                        return Err(CookError(format!(
                            "line {}: the entity has two {comp} components",
                            c.line
                        )));
                    }
                    rows.push(pending);
                }
                (k, None) => {
                    return Err(CookError(format!(
                        "line {}: '{k}' in an entity is neither a component block, a child entity nor a model",
                        c.line
                    )));
                }
            }
        }
        entities.push(entity);
        for c in children.into_iter().rev() {
            stack.push((c, index));
        }
    }
    let (name_offset, name_len) = add(&name);
    let mut entity_rows = Vec::with_capacity(entities.len());
    for e in &entities {
        let (name_offset, name_len) = add(&e.name);
        entity_rows.push(EntityRow {
            name_offset,
            name_len,
            parent: e.parent,
            instance: e.instance,
        });
    }
    let mut ref_rows = Vec::new();
    for r in &refs {
        let (id_offset, id_len) = add(r);
        ref_rows.push(RefRow { id_offset, id_len });
    }
    let mut comp_rows = Vec::new();
    let mut payloads = Vec::new();
    for (comp, rows) in &tables {
        let (schema, inferred) = match schemas.get(comp) {
            Some(s) => (s.clone(), false),
            None => {
                notes.push(format!(
                    "{comp}: no schema given; layout inferred from the scene text"
                ));
                (infer(comp, rows)?, true)
            }
        };
        let size = Layout::of(&schema).size();
        let mut bytes = Vec::with_capacity(rows.len() * (4 + size));
        for r in rows {
            bytes.extend_from_slice(&r.entity.to_le_bytes());
            pack(&schema, r, &mut bytes)?;
        }
        let hash = schema.layout_hash();
        let (name_offset, name_len) = add(comp);
        comp_rows.push(ComponentRow {
            name_offset,
            name_len,
            layout_hash_lo: hash as u32,
            layout_hash_hi: (hash >> 32) as u32,
            rows: rows.len() as u32,
            inferred: inferred as u32,
        });
        payloads.push((
            comp.clone(),
            bytes,
            (4 + size) as u32,
            hash,
            schema.to_json(),
        ));
    }
    let meta = SceneMeta {
        entity_count: entity_rows.len() as u32,
        component_count: comp_rows.len() as u32,
        ref_count: ref_rows.len() as u32,
        name_offset,
        name_len,
    };
    let mut w = BlobWriter::new(
        format::Scene::type_hash(),
        format::Scene::schema_version(),
        stamp,
    );
    w.table("meta", &[meta], row_hash::<SceneMeta>())
        .table("entities", &entity_rows, row_hash::<EntityRow>())
        .table("refs", &ref_rows, row_hash::<RefRow>())
        .table("components", &comp_rows, row_hash::<ComponentRow>())
        .raw("strings", strings, 1);
    for (comp, bytes, stride, hash, json) in payloads {
        w.rows(&format!("c.{comp}"), bytes, stride, hash);
        w.raw(&format!("s.{comp}"), json.into_bytes(), 1);
    }
    notes.push(format!(
        "{} entities, {} component table(s), {} model reference(s)",
        entity_rows.len(),
        comp_rows.len(),
        ref_rows.len()
    ));
    Ok((w.finish(), notes))
}
