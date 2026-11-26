//! `type table`: a game's data as rows keyed by id (items, abilities,
//! recipes, loot, waves). The one reader of a table's text, which og cook
//! and og.table's `load` and `parse` all run, so a table one accepts the
//! others accept: a row's id and its fields' forms are checked here; what
//! a field may hold is checked against the game's record by og.table.
//!
//! ```text
//! type table
//! version 1
//! ---
//! locale "en"                         # the base text's language (default en)
//! row "sword" {
//!   damage 12
//!   size 0.2 1.1 0.05                 # a Vector3
//!   rarity rare                       # an enum's variant
//!   icon asset "icons/sword.png"      # a path og cook checks exists
//!   name text "items.sword" "Sword"   # an og.i18n key and its base text
//!   ammo row "arrow"                  # a row of this table
//!   salvage row "materials.ogt" "iron" row "materials.ogt" "wood"   # rows of another
//! }
//! ```
//!
//! A row's handle ([`row_handle`]) is a hash of its table's id and its own,
//! so it names the same row in every load, reload and save.

use crate::ogt::{Document, Entry, Value};
use std::collections::BTreeMap;

/// The document type of a table.
pub const TABLE_TYPE: &str = "table";

/// A table's locale when it names none.
pub const DEFAULT_LOCALE: &str = "en";

/// The shared table reader's structural fields. Row contents belong to
/// the game's reflected record, not to an engine-defined data template.
pub const LINES: &[crate::ogt::line::Line] = &[
    crate::ogt::line::Line::new(
        "locale",
        "\"<language tag>\"",
        DEFAULT_LOCALE,
        "One non-empty tag for text fields' base language.",
        "",
    ),
    crate::ogt::line::Line::new(
        "row",
        "\"<id>\" { ... }",
        "",
        "Repeat unique, non-empty row ids. Each row field occurs once, has values and has no block.",
        "",
    ),
];
/// Value forms accepted by the row parser, in addition to plain values.
pub const ROW_LINES: &[crate::ogt::line::Line] = &[
    crate::ogt::line::Line::new(
        "<field>",
        "<number | string | word> ...",
        "",
        "Plain values are checked against the game's record when loaded.",
        "",
    ),
    crate::ogt::line::Line::new(
        "<field>",
        "asset \"<path>\"",
        "",
        "One asset-root path checked by cook.",
        "",
    ),
    crate::ogt::line::Line::new(
        "<field>",
        "text \"<key>\" \"<base message>\"",
        "",
        "Localized text; a key cannot have conflicting base messages.",
        "",
    ),
    crate::ogt::line::Line::new(
        "<field>",
        "row [\"<table asset>\"] \"<id>\" ...",
        "",
        "One or more row references; omitted table refers to this table. Cook checks referenced rows.",
        "",
    ),
];

/// A parsed table: its rows in file order.
#[derive(Clone, Debug, PartialEq)]
pub struct Table {
    /// The language its `text` fields' base text is in.
    pub locale: String,
    pub rows: Vec<Row>,
}

/// One row: its id, its line and its fields in file order.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub id: String,
    pub line: usize,
    pub fields: Vec<Field>,
}

/// One field of a row.
#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    pub name: String,
    pub line: usize,
    pub value: FieldValue,
}

/// What a field's line holds.
#[derive(Clone, Debug, PartialEq)]
pub enum FieldValue {
    /// Numbers, a string or a word, as written.
    Plain(Vec<Value>),
    /// `asset "path"`: a path under the project's assets.
    Asset(String),
    /// `text "key" "Base text"`: an og.i18n key and its base text.
    Text { key: String, base: String },
    /// One or more `row ["table"] "id"`: rows, of this table when the table is left out.
    Rows(Vec<RowRef>),
}

/// A row a field names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RowRef {
    /// The table's path as written, or `None` for the table the field is in.
    pub table: Option<String>,
    pub id: String,
}

fn at(line: usize, what: impl std::fmt::Display) -> String {
    format!("line {line}: {what}")
}

impl Table {
    /// Reads a table's text, refusing what no record could hold: another
    /// document type or version, an entry other than `locale` and `row`, a
    /// row without an id or with one seen before, a field named twice, a
    /// field with a block or without a value, and a malformed `asset`,
    /// `text` or `row` form. Each refusal names its line.
    pub fn parse(text: &str) -> Result<Table, String> {
        let doc = Document::parse(text).map_err(|e| e.reason().to_string())?;
        if doc.ty != TABLE_TYPE {
            return Err(format!(
                "expected 'type {TABLE_TYPE}', found 'type {}'",
                doc.ty
            ));
        }
        if doc.version != 1 {
            return Err(format!(
                "table version {} is not supported (1 is)",
                doc.version
            ));
        }
        let mut locale: Option<String> = None;
        let mut rows: Vec<Row> = Vec::new();
        let mut seen: BTreeMap<String, usize> = BTreeMap::new();
        for e in &doc.body.entries {
            if !LINES.iter().any(|f| f.key == e.key) {
                return Err(at(
                    e.line,
                    format!("'{}' is not a table entry (locale, row)", e.key),
                ));
            }
            match e.key.as_str() {
                "locale" => {
                    if locale.is_some() {
                        return Err(at(e.line, "a second locale line"));
                    }
                    match (e.values.as_slice(), &e.block) {
                        ([Value::String(l)], None) if !l.is_empty() => locale = Some(l.clone()),
                        _ => {
                            return Err(at(e.line, "locale takes one language tag: locale \"en\""));
                        }
                    }
                }
                "row" => {
                    let id = match e.values.as_slice() {
                        [Value::String(id)] if !id.is_empty() => id.clone(),
                        _ => {
                            return Err(at(
                                e.line,
                                "a row takes one non-empty id: row \"sword\" {",
                            ));
                        }
                    };
                    if let Some(first) = seen.insert(id.clone(), e.line) {
                        return Err(at(
                            e.line,
                            format!("row '{id}' again (first at line {first})"),
                        ));
                    }
                    let mut fields: Vec<Field> = Vec::new();
                    for f in e.block.iter().flat_map(|b| &b.entries) {
                        if let Some(was) = fields.iter().find(|x| x.name == f.key) {
                            return Err(at(
                                f.line,
                                format!(
                                    "row '{id}': field '{}' again (first at line {})",
                                    f.key, was.line
                                ),
                            ));
                        }
                        let value = field_value(f).map_err(|why| {
                            at(f.line, format!("row '{id}': field '{}': {why}", f.key))
                        })?;
                        fields.push(Field {
                            name: f.key.clone(),
                            line: f.line,
                            value,
                        });
                    }
                    rows.push(Row {
                        id,
                        line: e.line,
                        fields,
                    });
                }
                other => {
                    return Err(at(
                        e.line,
                        format!("'{other}' is not a table entry (locale, row)"),
                    ));
                }
            }
        }
        let table = Table {
            locale: locale.unwrap_or_else(|| DEFAULT_LOCALE.to_string()),
            rows,
        };
        table.strings()?;
        Ok(table)
    }

    /// The base text of every `text` field, by key; refused when one key
    /// is given two texts.
    pub fn strings(&self) -> Result<BTreeMap<String, String>, String> {
        let mut out: BTreeMap<String, (String, usize)> = BTreeMap::new();
        for (key, base, line) in self.texts() {
            match out.get(key) {
                Some((was, first)) if was != base => {
                    return Err(at(
                        line,
                        format!("text '{key}' is \"{was}\" at line {first}, here \"{base}\""),
                    ));
                }
                Some(_) => {}
                None => {
                    out.insert(key.to_string(), (base.to_string(), line));
                }
            }
        }
        Ok(out.into_iter().map(|(k, (v, _))| (k, v)).collect())
    }

    /// Each `text` field: its key, base text and line.
    pub fn texts(&self) -> impl Iterator<Item = (&str, &str, usize)> {
        self.fields().filter_map(|f| match &f.value {
            FieldValue::Text { key, base } => Some((key.as_str(), base.as_str(), f.line)),
            _ => None,
        })
    }

    /// Each `asset` field's path and line.
    pub fn assets(&self) -> impl Iterator<Item = (&str, usize)> {
        self.fields().filter_map(|f| match &f.value {
            FieldValue::Asset(path) => Some((path.as_str(), f.line)),
            _ => None,
        })
    }

    /// Each row a `row` field names, with its line.
    pub fn row_refs(&self) -> impl Iterator<Item = (&RowRef, usize)> {
        self.fields().flat_map(|f| match &f.value {
            FieldValue::Rows(refs) => refs.iter().map(|r| (r, f.line)).collect::<Vec<_>>(),
            _ => Vec::new(),
        })
    }

    /// The row with `id`.
    pub fn row(&self, id: &str) -> Option<&Row> {
        self.rows.iter().find(|r| r.id == id)
    }

    fn fields(&self) -> impl Iterator<Item = &Field> {
        self.rows.iter().flat_map(|r| &r.fields)
    }
}

/// A field line's value: a reference form when it starts with the word
/// `asset`, `text` or `row` and a string follows, else its values as
/// written (a lone word is an enum's variant or a bool).
fn field_value(f: &Entry) -> Result<FieldValue, String> {
    if f.block.is_some() {
        return Err("a field takes values, not a block".into());
    }
    let v = f.values.as_slice();
    let string = |i: usize| v.get(i).and_then(Value::as_str);
    match (v.first().and_then(Value::as_word), string(1)) {
        (Some("asset"), Some(path)) => match v.len() {
            2 if !path.is_empty() => Ok(FieldValue::Asset(path.to_string())),
            _ => Err("write asset \"path\"".into()),
        },
        (Some("text"), Some(key)) => match (v.len(), string(2)) {
            (3, Some(base)) if !key.is_empty() => Ok(FieldValue::Text {
                key: key.to_string(),
                base: base.to_string(),
            }),
            _ => Err("write text \"key\" \"Base text\"".into()),
        },
        (Some("row"), Some(_)) => {
            let mut refs = Vec::new();
            let mut i = 0;
            while i < v.len() {
                if v[i].as_word() != Some("row") {
                    return Err("write row \"id\" or row \"table\" \"id\", once per row".into());
                }
                let (table, id, next) = match (string(i + 1), string(i + 2)) {
                    (Some(t), Some(id)) => (Some(t.to_string()), id, i + 3),
                    (Some(id), None) => (None, id, i + 2),
                    _ => return Err("write row \"id\" or row \"table\" \"id\"".into()),
                };
                if id.is_empty() || table.as_deref() == Some("") {
                    return Err("a row reference names a non-empty table and id".into());
                }
                refs.push(RowRef {
                    table,
                    id: id.to_string(),
                });
                i = next;
            }
            Ok(FieldValue::Rows(refs))
        }
        _ if v.is_empty() => Err("a field takes a value".into()),
        _ => Ok(FieldValue::Plain(v.to_vec())),
    }
}

/// The handle of row `id` of the table `table` (its asset id): FNV-1a 64
/// over the table's id, a zero byte and the row's id, kept to its low 52
/// bits so a handle is exact as a JavaScript number, and never 0 (no row).
pub fn row_handle(table: &str, id: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in table.bytes().chain(std::iter::once(0)).chain(id.bytes()) {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    match h & ((1 << 52) - 1) {
        0 => 1,
        h => h,
    }
}
