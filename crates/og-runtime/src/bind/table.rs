//! Native reflected data tables. Load and structured replacement both install
//! rows atomically; stable handles are independent of row order and values.

use crate::engine::{Engine, engine, running};
use crate::host::codec::{Layout, RecordKind};
use crate::host::wire::{MissingRow, TableInfo, TableRowName};
use og_core::table::{Table, row_handle};
use serde_json::{Map, Value};
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

/// Install a complete native snapshot, or leave the previously loaded table.
fn build(eng: &Engine, path: &str, t: &Table, schema: &str) -> Result<TableInfo, String> {
    let id = asset_id(path);
    let layout = component(eng, schema)?;
    if t.schema.name != layout.schema {
        return Err(format!(
            "{id}: table declares {}, not {}",
            t.schema.name, layout.schema
        ));
    }
    let mut rows = Vec::with_capacity(t.rows.len());
    for (index, row) in t.rows.iter().enumerate() {
        let mut value =
            crate::world::assets::migrate(&t.schema, &row.bytes, &layout, &mut Vec::new());
        if let Some(fields) = t.bindings.get(&index) {
            crate::world::assets::bind_fields(eng, eng.world()?, &layout, fields, &mut value)?;
        }
        layout
            .check_ranges(&value)
            .map_err(|e| format!("{id}: row '{}': {e}", row.id))?;
        let bytes = layout
            .encode(None, Some(&value))
            .map_err(|e| format!("{id}: row '{}': {e}", row.id))?;
        rows.push((
            row.id.clone(),
            row_handle(&id, &row.id),
            Arc::new(layout.decode(&bytes)),
        ));
    }
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
    crate::bind::i18n::base_strings(eng, &t.locale, &t.strings)?;
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
    let table = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&id)?;
        bundle
            .table(&id, bytes.bytes())
            .map_err(|e| e.to_string())?
    };
    build(&eng, &id, &table, &schema)
}

pub fn og_table_replace(
    world: u32,
    path: String,
    data: String,
    schema: String,
) -> Result<TableInfo, String> {
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Input {
        locale: Option<String>,
        #[serde(default)]
        strings: BTreeMap<String, String>,
        rows: Vec<InputRow>,
    }
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct InputRow {
        id: String,
        values: Map<String, Value>,
    }
    let input: Input = serde_json::from_str(&data).map_err(|e| e.to_string())?;
    let eng = engine(world)?;
    let layout = component(&eng, &schema)?;
    let locale = input.locale.unwrap_or_else(|| "en".into());
    if locale.is_empty() {
        return Err("a table locale must be nonempty".into());
    }
    let mut seen = std::collections::BTreeSet::new();
    let mut rows = Vec::new();
    for row in input.rows {
        if row.id.is_empty() || !seen.insert(row.id.clone()) {
            return Err("empty or duplicate row ID".into());
        }
        for name in row.values.keys() {
            if layout
                .fields
                .iter()
                .find(|f| &f.name == name)
                .is_none_or(|f| f.readonly)
            {
                return Err(format!(
                    "{path}: row '{}': field '{name}' is unknown or read-only",
                    row.id
                ));
            }
        }
        let value = Value::Object(row.values);
        layout
            .check_ranges(&value)
            .map_err(|e| format!("{path}: row '{}': {e}", row.id))?;
        let bytes = layout
            .encode(None, Some(&value))
            .map_err(|e| format!("{path}: row '{}': {e}", row.id))?;
        rows.push(og_core::table::Row { id: row.id, bytes });
    }
    let table = Table {
        schema: layout.schema(),
        locale,
        rows,
        strings: input.strings,
        bindings: Default::default(),
    };
    build(&eng, &path, &table, &schema)
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
    let tb = eng.modules.table.0.borrow();
    let t = &tb.by_path[&path];
    Ok(TableRowName {
        id: t.rows[i].0.clone(),
        schema: t.schema.clone(),
        table: path.clone(),
    })
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
