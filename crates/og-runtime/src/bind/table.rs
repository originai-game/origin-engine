//! og.table (IDL `table`): a game's data tables read as one of its
//! components. og-core's table reader parses the text (the same reader
//! og cook runs); [`build`] reads each row's fields against the
//! component's layout, the one row builder og.table.load and
//! og.table.parse share. The loaded tables are the engine's assets (the
//! module's state), never world state: the world holds rows by handle
//! ([`og_core::table::row_handle`]), which names the same row in every
//! load, reload and save.

use crate::engine::{Engine, engine, running};
use crate::host::codec::{FieldLayout, Kind, Layout, RecordKind};
use crate::host::wire::{MissingRow, TableInfo, TableRowName};
use og_core::ogt::Value as Ogt;
use og_core::table::{FieldValue, Table, row_handle};
use serde_json::{Map, Value, json};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::Arc;

/// The column a TableRow field is declared with.
const COLUMN: &str = "TableRow";

/// One loaded table.
struct Loaded {
    schema: String,
    /// Its rows in file order: id, handle and values as the schema reads them.
    rows: Vec<(String, u64, Arc<Value>)>,
}

#[derive(Default)]
struct Tables {
    by_path: BTreeMap<String, Loaded>,
    /// Every loaded row by handle: its table and index, across all tables.
    by_handle: BTreeMap<u64, (String, usize)>,
}

/// og.table on one engine: the tables loaded into it.
#[derive(Default)]
pub(crate) struct ModuleState(RefCell<Tables>);

fn asset_id(path: &str) -> String {
    og_bundle::runtime::asset_id(path).to_string()
}

/// The game component `schema` names.
fn component(eng: &Engine, schema: &str) -> Result<Layout, String> {
    let name = schema;
    match eng.state.borrow().layouts.get(name) {
        Some(l) if l.kind == RecordKind::Component => Ok(l.clone()),
        Some(l) => Err(format!(
            "'{name}' is a {}; a table's rows are read as a component",
            l.kind.as_str()
        )),
        None => Err(format!("'{name}' is not a component declared in this game")),
    }
}

/// The handle a `row` reference names, from the table `own`.
fn handle_of(own: &str, r: &og_core::table::RowRef) -> u64 {
    row_handle(
        &r.table.as_deref().map_or_else(|| own.to_string(), asset_id),
        &r.id,
    )
}

/// A field's JSON value, as the layout's encoder takes it, from its line.
fn field_json(f: &FieldLayout, v: &FieldValue, own: &str) -> Result<Value, String> {
    let is_row = f.column == COLUMN;
    match v {
        FieldValue::Rows(refs) if is_row => {
            let handles: Vec<Value> = refs.iter().map(|r| json!(handle_of(own, r))).collect();
            if f.ty.is_list() {
                Ok(Value::Array(handles))
            } else if let [h] = handles.as_slice() {
                Ok(h.clone())
            } else {
                Err(format!(
                    "a TableRow field holds one row, not {}: make it a list",
                    handles.len()
                ))
            }
        }
        FieldValue::Rows(_) => Err(format!(
            "a row reference goes in a TableRow field, not a {}",
            kind_name(f)
        )),
        _ if is_row => Err("write row \"id\" or row \"table\" \"id\"".into()),
        FieldValue::Asset(text) | FieldValue::Text { key: text, .. }
            if f.kind == Kind::Str && !f.ty.is_list() =>
        {
            Ok(json!(text))
        }
        FieldValue::Asset(_) | FieldValue::Text { .. } => Err(format!(
            "asset and text forms go in a str field, not a {}",
            kind_name(f)
        )),
        FieldValue::Plain(vs) if f.ty.is_list() => vs
            .iter()
            .map(|v| match v {
                Ogt::Number(n) if f.kind != Kind::Asset && f.kind != Kind::Entity => {
                    Ok(json!(n.as_f64()))
                }
                _ => Err(format!("a list of {} takes numbers", kind_name(f))),
            })
            .collect::<Result<Vec<_>, String>>()
            .map(Value::Array),
        FieldValue::Plain(vs) => match f.kind {
            Kind::F32 | Kind::F64 => Ok(json!(one_number(f, vs)?.as_f64())),
            Kind::U32 => {
                let n = one_number(f, vs)?
                    .as_u64()
                    .ok()
                    .filter(|n| *n <= u64::from(u32::MAX));
                n.map(|n| json!(n))
                    .ok_or_else(|| "a u32 field takes a whole number from 0 to 4294967295".into())
            }
            Kind::I32 => {
                let n = one_number(f, vs)?
                    .as_i64()
                    .ok()
                    .filter(|n| i32::try_from(*n).is_ok());
                n.map(|n| json!(n))
                    .ok_or_else(|| "an i32 field takes a whole number that fits 32 bits".into())
            }
            Kind::Bool => match vs.as_slice() {
                [v] if v.as_bool().is_some() => Ok(json!(v.as_bool())),
                _ => Err("a bool field takes true or false".into()),
            },
            Kind::Str => match vs.as_slice() {
                [Ogt::String(s)] => Ok(json!(s)),
                _ => Err("a str field takes one string".into()),
            },
            Kind::Enum => match vs.as_slice() {
                [Ogt::Word(w)] if f.variants.iter().any(|v| v == w) => Ok(json!(w)),
                _ => Err(format!(
                    "an enum field takes one of {}",
                    f.variants.join(", ")
                )),
            },
            Kind::Vector2 | Kind::Vector3 | Kind::Quaternion | Kind::Color => {
                let keys = f.kind.keys();
                let numbers: Option<Vec<f64>> = vs
                    .iter()
                    .map(|v| v.as_number().map(|n| n.as_f64()))
                    .collect();
                match numbers {
                    Some(n) if n.len() == keys.len() => Ok(Value::Object(
                        keys.iter()
                            .zip(n)
                            .map(|(k, n)| (k.to_string(), json!(n)))
                            .collect(),
                    )),
                    _ => Err(format!(
                        "a {} field takes {} numbers",
                        kind_name(f),
                        keys.len()
                    )),
                }
            }
            Kind::Matrix4 => {
                let numbers: Option<Vec<f64>> = vs
                    .iter()
                    .map(|v| v.as_number().map(|n| n.as_f64()))
                    .collect();
                match numbers {
                    Some(n) if n.len() == 16 => Ok(json!({ "elements": n })),
                    _ => Err("a Matrix4 field takes 16 numbers, column by column".into()),
                }
            }
            Kind::Entity
            | Kind::Geometry
            | Kind::Material
            | Kind::Asset
            | Kind::Sound
            | Kind::Ease => Err(format!(
                "a table cannot hold a {}: give the field a str and write asset \"path\" to name a file",
                kind_name(f)
            )),
        },
    }
}

/// A field line's one number.
fn one_number<'a>(f: &FieldLayout, vs: &'a [Ogt]) -> Result<&'a og_core::ogt::Number, String> {
    match vs {
        [Ogt::Number(n)] => Ok(n),
        _ => Err(format!("a {} field takes one number", kind_name(f))),
    }
}

fn kind_name(f: &FieldLayout) -> &str {
    if f.column.is_empty() {
        "field of this kind"
    } else {
        &f.column
    }
}

/// Reads `text` as `schema`'s rows under the asset id `path`, and puts it
/// in place of what was loaded there, or refuses and changes nothing: the
/// one builder og.table.load and og.table.parse share.
fn build(eng: &Engine, path: &str, text: &str, schema: &str) -> Result<TableInfo, String> {
    let id = asset_id(path);
    let layout = component(eng, schema)?;
    let t = Table::parse(text).map_err(|e| format!("{id}: {e}"))?;
    let at = |line: usize, row: &str, field: &str, why: String| {
        format!("{id}: line {line}: row '{row}': field '{field}': {why}")
    };
    let mut rows = Vec::with_capacity(t.rows.len());
    for row in &t.rows {
        let mut m = Map::new();
        for f in &row.fields {
            let Some(fl) = layout.fields.iter().find(|x| x.name == f.name) else {
                return Err(at(
                    f.line,
                    &row.id,
                    &f.name,
                    format!("'{}' has no such field", layout.schema),
                ));
            };
            if fl.readonly {
                return Err(at(f.line, &row.id, &f.name, "it is read-only".into()));
            }
            let v =
                field_json(fl, &f.value, &id).map_err(|why| at(f.line, &row.id, &f.name, why))?;
            // The encoder's own checks (a str's capacity, a list's length), by field.
            let mut one = Map::new();
            one.insert(f.name.clone(), v.clone());
            layout
                .encode(None, Some(&Value::Object(one)))
                .map_err(|why| at(f.line, &row.id, &f.name, why))?;
            m.insert(f.name.clone(), v);
        }
        let bytes = layout
            .encode(None, Some(&Value::Object(m)))
            .map_err(|why| format!("{id}: line {}: row '{}': {why}", row.line, row.id))?;
        rows.push((
            row.id.clone(),
            row_handle(&id, &row.id),
            Arc::new(layout.decode(&bytes)),
        ));
    }
    let strings = t.strings().map_err(|e| format!("{id}: {e}"))?;
    let tables = &eng.modules.table.0;
    {
        // One handle names one row across every loaded table.
        let tb = tables.borrow();
        let mut mine: BTreeMap<u64, &str> = BTreeMap::new();
        for (row, h, _) in &rows {
            if let Some(other) = mine.insert(*h, row) {
                return Err(format!(
                    "{id}: rows '{other}' and '{row}' hash to one handle: rename one"
                ));
            }
            if let Some((p, i)) = tb.by_handle.get(h).filter(|(p, _)| *p != id) {
                let other = &tb.by_path[p].rows[*i].0;
                return Err(format!(
                    "{id}: row '{row}' hashes to the handle of row '{other}' of {p}: rename one"
                ));
            }
        }
    }
    crate::bind::i18n::base_strings(eng, &t.locale, &strings)?;
    let mut tb = tables.borrow_mut();
    if let Some(old) = tb.by_path.remove(&id) {
        for (_, h, _) in &old.rows {
            tb.by_handle.remove(h);
        }
    }
    for (i, (_, h, _)) in rows.iter().enumerate() {
        tb.by_handle.insert(*h, (id.clone(), i));
    }
    let info = TableInfo {
        path: id.clone(),
        schema: layout.schema.clone(),
        locale: t.locale.clone(),
        rows: rows.iter().map(|(r, _, _)| r.clone()).collect(),
    };
    tb.by_path.insert(
        id,
        Loaded {
            schema: layout.schema,
            rows,
        },
    );
    Ok(info)
}

pub fn og_table_load(world: u32, path: String, schema: String) -> Result<TableInfo, String> {
    let eng = engine(world)?;
    let id = asset_id(&path);
    let text = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&id)?;
        bundle
            .table_text(&id, bytes.bytes())
            .map_err(|e| e.to_string())?
    };
    build(&eng, &id, &text, &schema)
}

pub fn og_table_parse(
    world: u32,
    path: String,
    text: String,
    schema: String,
) -> Result<TableInfo, String> {
    build(&*engine(world)?, &path, &text, &schema)
}

/// The loaded row `handle`: its table's path and index.
fn find(eng: &Engine, handle: f64) -> Result<(String, usize), String> {
    eng.modules
        .table
        .0
        .borrow()
        .by_handle
        .get(&(handle as u64))
        .cloned()
        .ok_or_else(|| format!("no loaded table has the row {handle}"))
}

pub fn og_table_row(world: u32, table: String, id: String) -> Result<f64, String> {
    let eng = engine(world)?;
    let tb = eng.modules.table.0.borrow();
    let path = asset_id(&table);
    let t = tb
        .by_path
        .get(&path)
        .ok_or_else(|| format!("the table {path} is not loaded"))?;
    t.rows
        .iter()
        .find(|(r, _, _)| *r == id)
        .map(|(_, h, _)| *h as f64)
        .ok_or_else(|| format!("the table {path} has no row '{id}'"))
}

pub fn og_table_rows(world: u32, table: String) -> Result<Vec<f64>, String> {
    let eng = engine(world)?;
    let tb = eng.modules.table.0.borrow();
    let path = asset_id(&table);
    let t = tb
        .by_path
        .get(&path)
        .ok_or_else(|| format!("the table {path} is not loaded"))?;
    Ok(t.rows.iter().map(|(_, h, _)| *h as f64).collect())
}

pub fn og_table_get(world: u32, row: f64, schema: String) -> Result<Value, String> {
    let eng = engine(world)?;
    let (path, i) = find(&eng, row)?;
    let name = component(&eng, &schema)?.schema;
    let tb = eng.modules.table.0.borrow();
    let t = &tb.by_path[&path];
    if t.schema != name {
        return Err(format!(
            "the table {path} was loaded as {}, not {name}",
            t.schema
        ));
    }
    Ok(t.rows[i].2.as_ref().clone())
}

pub fn og_table_name(world: u32, row: f64) -> Result<TableRowName, String> {
    let eng = engine(world)?;
    let (path, i) = find(&eng, row)?;
    let id = eng.modules.table.0.borrow().by_path[&path].rows[i]
        .0
        .clone();
    Ok(TableRowName { table: path, id })
}

/// Every TableRow a component or resource field holds that no loaded
/// table has: outside a system every record's, inside one the records it
/// reads (the others are not its to see).
pub fn og_table_missing(world: u32) -> Vec<MissingRow> {
    (|| -> Result<Vec<MissingRow>, String> {
        let eng = engine(world)?;
        let layouts: Vec<Layout> = eng
            .state
            .borrow()
            .layouts
            .values()
            .filter(|l| l.kind != RecordKind::Event && l.fields.iter().any(|f| f.column == COLUMN))
            .cloned()
            .collect();
        let known = |h: u64| eng.modules.table.0.borrow().by_handle.contains_key(&h);
        let mut out = Vec::new();
        let mut look = |entity: u64, l: &Layout, bytes: &[u8]| {
            for (i, f) in l
                .fields
                .iter()
                .enumerate()
                .filter(|(_, f)| f.column == COLUMN)
            {
                let at = l.offset(i);
                let b = &bytes[at..at + f.ty.size()];
                let handles: Vec<u64> = if f.ty.is_list() {
                    og_core::list_elements(f.ty, b)
                        .map(|e| u64::from_le_bytes(e[..8].try_into().expect("8 bytes")))
                        .collect()
                } else {
                    vec![u64::from_le_bytes(b[..8].try_into().expect("8 bytes"))]
                };
                for h in handles.into_iter().filter(|h| *h != 0 && !known(*h)) {
                    out.push(MissingRow {
                        entity,
                        record: l.schema.clone(),
                        field: f.name.clone(),
                        row: h,
                    });
                }
            }
        };
        let q = |name: &str| og_ecs::QueryDesc {
            read: vec![name.to_string()],
            ..Default::default()
        };
        // SAFETY: the frame is published while its system runs.
        match running(world).map(|f| unsafe { f.ctx() }) {
            Some(ctx) => {
                for l in &layouts {
                    let Ok(id) = ctx.id_by_name(&l.schema) else {
                        continue;
                    };
                    if l.kind == RecordKind::Resource {
                        if let Ok(bytes) = ctx.resource_bytes(id) {
                            look(0, l, &bytes);
                        }
                        continue;
                    }
                    let Ok(it) = ctx.query_with(&q(&l.schema)) else {
                        continue;
                    };
                    for e in it.flat_map(|a| a.entities().to_vec()) {
                        if let Ok(bytes) = ctx.get_bytes(e, id) {
                            look(e.to_bits(), l, &bytes);
                        }
                    }
                }
            }
            None => {
                let w = eng.world()?;
                for l in &layouts {
                    let Ok(id) = w.id_by_name(&l.schema) else {
                        continue;
                    };
                    if l.kind == RecordKind::Resource {
                        if let Ok(bytes) = w.resource_bytes(id) {
                            look(0, l, &bytes);
                        }
                        continue;
                    }
                    let Ok(it) = w.query(&q(&l.schema)) else {
                        continue;
                    };
                    for e in it.flat_map(|a| a.entities().to_vec()) {
                        if let Ok(bytes) = w.get_bytes(e, id) {
                            look(e.to_bits(), l, &bytes);
                        }
                    }
                }
            }
        }
        out.sort_by(|a, b| {
            (a.entity, &a.record, &a.field, a.row).cmp(&(b.entity, &b.record, &b.field, b.row))
        });
        Ok(out)
    })()
    .unwrap_or_default()
}
