//! Checked TS rows become native reflected bytes. References are checked
//! against one project snapshot, so legal cross-table cycles need no recursion.
use super::{CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, stamp};
use crate::cook::{
    generator,
    source_content::{self, Graph},
    source_records::PackedRecord,
    source_schemas::world::table::TableInput,
};
use crate::format::{self, Area, BlobKind};
use og_blob::BlobWriter;
use og_core::table::{Row, Table, row_handle};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) static TABLE: KindDef = KindDef {
    name: "table",
    extensions: &[],
    types: &[],
    by_content: true,
    streams: false,
    plan,
};
pub(crate) static KINDS: &[&KindDef] = &[&TABLE];

pub(crate) fn prepare(
    sources: &mut [Source],
    graph: &Graph,
    records: &BTreeMap<String, PackedRecord>,
    captured: &BTreeMap<String, generator::source::Source>,
) -> Result<(), CookError> {
    let mut inputs = BTreeMap::new();
    for source in sources.iter().filter(|s| s.kind == &TABLE) {
        let input: TableInput = source.input().map_err(CookError)?;
        input.validate().map_err(|e| {
            CookError(
                source_content::diagnostic(
                    &captured[&source.rel],
                    &source.path.to_string_lossy(),
                    "/rows",
                    e,
                )
                .to_string(),
            )
        })?;
        inputs.insert(source.rel.clone(), input);
    }
    let ids: BTreeMap<_, BTreeSet<_>> = inputs
        .iter()
        .map(|(id, input)| {
            (
                id.as_str(),
                input.rows.iter().map(|r| r.id.as_str()).collect(),
            )
        })
        .collect();
    for source in sources.iter_mut().filter(|s| s.kind == &TABLE) {
        let input = &inputs[&source.rel];
        let record = &records[&input.record];
        let mut table = Table {
            schema: record.schema.clone(),
            locale: input.locale.clone().unwrap_or_else(|| "en".into()),
            rows: Vec::new(),
            strings: BTreeMap::new(),
            bindings: BTreeMap::new(),
        };
        for (index, row) in input.rows.iter().enumerate() {
            let mut values = row.values.clone();
            let at = |name: &str, e: String| {
                let name = name.replace('~', "~0").replace('/', "~1");
                CookError(
                    source_content::diagnostic(
                        &captured[&source.rel],
                        &source.path.to_string_lossy(),
                        &format!("/rows/{index}/values/{name}"),
                        e,
                    )
                    .to_string(),
                )
            };
            for (name, value) in &mut values {
                let resolve = |value: &mut Value,
                               strings: &mut BTreeMap<String, String>|
                 -> Result<(), String> {
                    if record.column(name) == Some("TableRow") {
                        let id = value["row"].as_str().ok_or("expected a row reference")?;
                        let target = value
                            .get("table")
                            .map(|t| t["$ogRef"].as_str().expect("checked table ref"))
                            .unwrap_or(&source.rel);
                        if !ids.get(target).is_some_and(|rows| rows.contains(id)) {
                            return Err(format!("table '{target}' has no row '{id}'"));
                        }
                        *value = json!(row_handle(&graph.outputs[target].asset, id));
                    } else if let Some(token) = value.get("asset") {
                        *value = json!(
                            graph.outputs[token["$ogRef"].as_str().expect("checked asset ref")]
                                .asset
                        );
                    } else if let Some(text) = value.get("text") {
                        let key = text["key"].as_str().expect("checked text key");
                        let base = text["base"].as_str().expect("checked base text");
                        if strings
                            .insert(key.into(), base.into())
                            .is_some_and(|was| was != base)
                        {
                            return Err(format!("conflicting base text for '{key}'"));
                        }
                        *value = json!(key);
                    }
                    Ok(())
                };
                let result = if record.schema.fields
                    [record.schema.field_index(name).expect("checked field")]
                .ty
                .is_list()
                {
                    value
                        .as_array_mut()
                        .expect("checked list")
                        .iter_mut()
                        .try_for_each(|v| resolve(v, &mut table.strings))
                } else {
                    resolve(value, &mut table.strings)
                };
                result.map_err(|e| at(name, format!("row '{}', field '{name}': {e}", row.id)))?;
            }
            let (bytes, bindings) = record
                .pack(&values)
                .map_err(|e| at("", format!("row '{}': {e}", row.id)))?;
            if !bindings.is_empty() {
                table.bindings.insert(index, bindings);
            }
            table.rows.push(Row {
                id: row.id.clone(),
                bytes,
            });
        }
        // Resolve typed asset relocations through actual outputs, not filenames.
        fn bind(value: &mut Value, graph: &Graph) {
            match value {
                Value::Object(m) if m.contains_key("$ogRef") => {
                    let id = m["$ogRef"].as_str().unwrap();
                    m.insert("$ogRef".into(), json!(graph.outputs[id].asset));
                }
                Value::Object(m) => m.values_mut().for_each(|v| bind(v, graph)),
                Value::Array(a) => a.iter_mut().for_each(|v| bind(v, graph)),
                _ => {}
            }
        }
        for fields in table.bindings.values_mut() {
            for value in fields.values_mut() {
                bind(value, graph);
            }
        }
        source.table = Some(table);
    }
    Ok(())
}

fn plan<'a>(
    _cx: &Cx<'a>,
    _state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let table = s.table.as_ref().expect("prepared table");
    let mut native = blake3::Hasher::new();
    native.update(table.schema.to_json().as_bytes());
    native.update(
        &serde_json::to_vec(&(&table.locale, &table.strings, &table.bindings))
            .expect("table metadata JSON"),
    );
    for row in &table.rows {
        native.update(&row.bytes);
    }
    let st = stamp(
        &[Area::Data],
        &[b"native-table-v2", &hash, native.finalize().as_bytes()],
    );
    outs.push((
        s.rel.clone(),
        format::Table::NAME,
        st,
        Box::new(TableJob(s)),
    ));
}

struct TableJob<'a>(&'a Source);
impl Job for TableJob<'_> {
    fn cook(&self, p: &Planned, _ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let t = self.0.table.as_ref().expect("prepared table");
        let mut w = BlobWriter::new(
            format::Table::type_hash(),
            format::Table::schema_version(),
            p.stamp,
        );
        let info = (
            &t.locale,
            t.rows.iter().map(|r| &r.id).collect::<Vec<_>>(),
            &t.strings,
            &t.bindings,
        );
        let size = og_core::Layout::of(&t.schema).size();
        let rows: Vec<u8> = t
            .rows
            .iter()
            .flat_map(|r| if size == 0 { vec![0] } else { r.bytes.clone() })
            .collect();
        w.table(
            "meta",
            &[format::TableMeta {
                row_count: t.rows.len() as u32,
            }],
            format::row_hash::<format::TableMeta>(),
        )
        .raw("schema", t.schema.to_json().into_bytes(), 1)
        .raw(
            "info",
            serde_json::to_vec(&info).map_err(|e| CookError(e.to_string()))?,
            1,
        )
        .rows("rows", rows, size.max(1) as u32, t.schema.layout_hash());
        Ok((w.finish(), vec![format!("table: {} rows", t.rows.len())]))
    }
}
