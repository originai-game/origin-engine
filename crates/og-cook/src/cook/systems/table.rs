//! Tables (`type table`): a game's data rows, read by og-core's one table
//! reader and checked against the project: every `asset` path exists and
//! every row a `row` field names is in its table. Stored as text, which
//! og.table reads again against the game's record.

use super::{
    CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, assets_root, stamp,
};
use crate::format::{self, BlobKind};
use og_blob::BlobWriter;
use og_core::table::{TABLE_TYPE, Table};
use std::collections::BTreeSet;

pub(crate) static TABLE: KindDef = KindDef {
    name: "table",
    extensions: &[],
    types: &[TABLE_TYPE],
    by_content: false,
    streams: false,
    plan,
};

/// The kinds this module cooks.
pub(crate) static KINDS: &[&KindDef] = &[&TABLE];

/// The tables a table's `row` fields name besides itself, by asset id.
fn other_tables(own: &str, t: &Table) -> BTreeSet<String> {
    t.row_refs()
        .filter_map(|(r, _)| {
            r.table
                .as_deref()
                .map(|p| crate::runtime::asset_id(p).to_string())
        })
        .filter(|id| id != own)
        .collect()
}

/// Checks a parsed table against the project: `known` holds the asset ids
/// there are, `read` a table's text by asset id. Each refusal names its line.
pub(crate) fn check(
    own: &str,
    t: &Table,
    known: &dyn Fn(&str) -> bool,
    read: &dyn Fn(&str) -> Result<String, String>,
) -> Result<(), String> {
    for (path, line) in t.assets() {
        if !known(crate::runtime::asset_id(path)) {
            return Err(format!(
                "line {line}: asset '{path}' is not in the project's assets"
            ));
        }
    }
    let mut others: std::collections::BTreeMap<String, Result<Table, String>> = Default::default();
    for (r, line) in t.row_refs() {
        let id = r.table.as_deref().map_or(own, crate::runtime::asset_id);
        let found = if id == own {
            t.row(&r.id).is_some()
        } else {
            let other = others
                .entry(id.to_string())
                .or_insert_with(|| read(id).and_then(|text| Table::parse(&text)));
            match other {
                Ok(o) => o.row(&r.id).is_some(),
                Err(e) => return Err(format!("line {line}: table '{id}': {e}")),
            }
        };
        if !found {
            return Err(format!("line {line}: table '{id}' has no row '{}'", r.id));
        }
    }
    Ok(())
}

/// A table's text from the assets root, by asset id.
fn read_table(s: &Source, id: &str) -> Result<String, String> {
    let bytes =
        std::fs::read(assets_root(s).join(id)).map_err(|e| format!("cannot read it: {e}"))?;
    String::from_utf8(bytes).map_err(|_| "a table is text (UTF-8)".to_string())
}

fn plan<'a>(
    cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let Cx { known, .. } = *cx;
    let own = crate::runtime::asset_id(&s.rel).to_string();
    // What its check reads beyond its own bytes: the tables it names (by
    // their bytes) and whether each asset it names is there.
    let mut deps = String::new();
    if let Ok(t) = std::str::from_utf8(&s.bytes)
        .map_err(|_| ())
        .and_then(|text| Table::parse(text).map_err(|_| ()))
    {
        let mut refs: BTreeSet<String> = other_tables(&own, &t);
        for id in &refs {
            let h = std::fs::read(assets_root(s).join(id)).map_or(0, |b| super::hash_bytes(&b));
            deps.push_str(&format!("{id}={h:016x}\n"));
        }
        for (path, _) in t.assets() {
            let id = crate::runtime::asset_id(path);
            deps.push_str(&format!("{id}={}\n", known.contains(id)));
            refs.insert(id.to_string());
        }
        state
            .parsed_refs
            .entry(s.rel.clone())
            .or_default()
            .extend(refs);
    }
    let st = stamp(&[b"table", &hash, deps.as_bytes()]);
    outs.push((
        s.rel.clone(),
        format::Table::NAME,
        st,
        Box::new(TableJob(s)),
    ));
}

/// A table's one output.
struct TableJob<'a>(&'a Source);

impl Job for TableJob<'_> {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let TableJob(s) = self;
        let Exec { known, .. } = ex;
        let text = std::str::from_utf8(&s.bytes)
            .map_err(|_| CookError("a table is text (UTF-8)".into()))?;
        let t = Table::parse(text).map_err(CookError)?;
        let own = crate::runtime::asset_id(&s.rel);
        check(own, &t, &|id| known.contains(id), &|id| read_table(s, id)).map_err(CookError)?;
        let mut w = BlobWriter::new(
            format::Table::type_hash(),
            format::Table::schema_version(),
            p.stamp,
        );
        w.table(
            "meta",
            &[format::TableMeta {
                text_bytes: text.len() as u32,
            }],
            format::row_hash::<format::TableMeta>(),
        )
        .raw("text", text.as_bytes().to_vec(), 16);
        Ok((w.finish(), vec![format!("table: {} rows", t.rows.len())]))
    }
}
