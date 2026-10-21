//! String tables (`type strings` .ogt, one per locale) and the catalog a
//! game's tables make: the active locale, its fallback chain, `t`, and the
//! cook's check that every locale has every key with the same arguments.

use crate::locale::{Direction, canonical, direction_of_locale, language, negotiate};
use crate::message::{Arg, Message};
use og_core::ogt::{Document, Value};
use std::collections::BTreeMap;

/// The `.ogt` type of a string table.
pub const STRINGS_TYPE: &str = "strings";

/// One locale's strings.
#[derive(Clone, Debug, PartialEq)]
pub struct StringTable {
    /// The locale, canonical (`zh-CN`, `ja`, `ar`).
    pub locale: String,
    /// The table to read a missing key from (default: the base table).
    pub fallback: Option<String>,
    /// The table every other must match (the `base true` one, else `en`, else the first).
    pub base: bool,
    pub direction: Direction,
    /// This locale's font chain (asset paths), first first.
    pub fonts: Vec<String>,
    /// Key → (source text, parsed message), sorted by key.
    pub entries: BTreeMap<String, (String, Message)>,
}

fn string_of(v: &Value) -> Option<&str> {
    v.as_str().or(v.as_word())
}

impl StringTable {
    /// Parses a table, refusing a malformed entry or message with its line.
    pub fn parse(text: &str) -> Result<StringTable, String> {
        let doc = Document::parse(text).map_err(|e| e.to_string())?;
        if doc.ty != STRINGS_TYPE {
            return Err(format!(
                "a string table is `type {STRINGS_TYPE}`, not `{}`",
                doc.ty
            ));
        }
        let mut locale = None;
        let mut t = StringTable {
            locale: String::new(),
            fallback: None,
            base: false,
            direction: Direction::Ltr,
            fonts: Vec::new(),
            entries: BTreeMap::new(),
        };
        let mut direction = None;
        for e in &doc.body.entries {
            let first = e.values.first().and_then(string_of);
            match e.key.as_str() {
                "locale" => locale = first.map(canonical),
                "fallback" => t.fallback = first.map(canonical),
                "base" => t.base = first != Some("false"),
                "direction" => {
                    direction = Some(match first {
                        Some("rtl") => Direction::Rtl,
                        Some("ltr") => Direction::Ltr,
                        _ => return Err(format!("line {}: direction is ltr or rtl", e.line)),
                    })
                }
                "fonts" => {
                    t.fonts = e
                        .values
                        .iter()
                        .filter_map(string_of)
                        .map(str::to_string)
                        .collect()
                }
                "string" => {
                    let (Some(key), Some(msg)) = (
                        e.values.first().and_then(string_of),
                        e.values.get(1).and_then(Value::as_str),
                    ) else {
                        return Err(format!("line {}: write string \"key\" \"message\"", e.line));
                    };
                    let m = Message::parse(msg)
                        .map_err(|err| format!("line {}: '{key}': {err}", e.line))?;
                    if t.entries
                        .insert(key.to_string(), (msg.to_string(), m))
                        .is_some()
                    {
                        return Err(format!("line {}: '{key}' is defined twice", e.line));
                    }
                }
                k => {
                    return Err(format!(
                        "line {}: '{k}' is not a string table entry: locale, fallback, base, direction, fonts or string",
                        e.line
                    ));
                }
            }
        }
        t.locale = locale.ok_or("a string table needs `locale \"<tag>\"` (en, zh-CN, ja, ar)")?;
        t.direction = direction.unwrap_or_else(|| direction_of_locale(&t.locale));
        Ok(t)
    }
}

/// A key a locale lacks, or whose message takes other arguments than the base's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MissingKey {
    pub locale: String,
    pub key: String,
    /// `missing`, `arguments` (they differ from the base's) or `extra` (not in the base).
    pub problem: &'static str,
    pub detail: String,
}

fn base_of(tables: &[StringTable]) -> Option<&StringTable> {
    base_in(tables.iter())
}

/// [`base_of`] over borrowed tables (the catalog's, without copying them).
fn base_in<'a, I>(tables: I) -> Option<&'a StringTable>
where
    I: Iterator<Item = &'a StringTable> + Clone,
{
    let mut all = tables.clone();
    tables
        .clone()
        .find(|t| t.base)
        .or_else(|| tables.clone().find(|t| t.locale == "en"))
        .or_else(|| all.next())
}

/// Every key the base table has that another lacks (or takes other
/// arguments for), and every key another has that the base lacks.
pub fn check_tables(tables: &[StringTable]) -> Vec<MissingKey> {
    let Some(base) = base_of(tables) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for t in tables.iter().filter(|t| t.locale != base.locale) {
        for (key, (_, m)) in &base.entries {
            match t.entries.get(key) {
                None => out.push(MissingKey {
                    locale: t.locale.clone(),
                    key: key.clone(),
                    problem: "missing",
                    detail: format!("{} has it", base.locale),
                }),
                Some((_, tm)) if tm.arguments() != m.arguments() => out.push(MissingKey {
                    locale: t.locale.clone(),
                    key: key.clone(),
                    problem: "arguments",
                    detail: format!(
                        "{} takes {{{}}}, {} {{{}}}",
                        t.locale,
                        tm.arguments().join("}, {"),
                        base.locale,
                        m.arguments().join("}, {")
                    ),
                }),
                _ => {}
            }
        }
        for key in t.entries.keys().filter(|k| !base.entries.contains_key(*k)) {
            out.push(MissingKey {
                locale: t.locale.clone(),
                key: key.clone(),
                problem: "extra",
                detail: format!("{} does not have it", base.locale),
            });
        }
    }
    out
}

/// A game's tables and the active locale.
#[derive(Clone, Debug, Default)]
pub struct Catalog {
    tables: BTreeMap<String, StringTable>,
    active: Option<String>,
    requested: Vec<String>,
}

impl Catalog {
    /// Adds (or replaces) a locale's table; the active locale is
    /// renegotiated against the request.
    pub fn insert(&mut self, table: StringTable) {
        self.tables.insert(table.locale.clone(), table);
        self.renegotiate();
    }

    pub fn locales(&self) -> Vec<String> {
        self.tables.keys().cloned().collect()
    }

    /// Asks for a locale (or a list, best first: the player's languages);
    /// the active one is the best table for it, or the base table.
    pub fn request(&mut self, locales: &[&str]) {
        self.requested = locales.iter().map(|l| canonical(l)).collect();
        self.renegotiate();
    }

    fn renegotiate(&mut self) {
        let available = self.locales();
        let req: Vec<&str> = self.requested.iter().map(String::as_str).collect();
        self.active = negotiate(&req, &available)
            .map(str::to_string)
            .or_else(|| base_in(self.tables.values()).map(|b| b.locale.clone()));
    }

    /// The active locale (`None` without tables).
    pub fn active(&self) -> Option<&StringTable> {
        self.tables.get(self.active.as_ref()?)
    }

    /// The locale requested first (what the host said), canonical.
    pub fn requested(&self) -> Option<&str> {
        self.requested.first().map(String::as_str)
    }

    /// The tables a key is looked up in: the active, its fallback, the base.
    fn chain(&self) -> Vec<&StringTable> {
        let mut out: Vec<&StringTable> = Vec::new();
        let mut next = self.active.as_deref();
        while let Some(l) = next {
            if out.iter().any(|t| t.locale == l) {
                break;
            }
            let Some(t) = self.tables.get(l) else { break };
            out.push(t);
            next = t.fallback.as_deref();
        }
        // The base table, borrowed: copying every table on each lookup was
        // most of a tick's og.i18n.t (perf-games, kestrel-stair).
        if let Some(b) = base_in(self.tables.values())
            && !out.iter().any(|t| t.locale == b.locale)
        {
            out.push(b);
        }
        out
    }

    /// The message for `key` in the active locale (through the fallback
    /// chain), formatted; `None` when no table has it.
    pub fn t(&self, key: &str, args: &BTreeMap<String, Arg>) -> Option<String> {
        for t in self.chain() {
            if let Some((_, m)) = t.entries.get(key) {
                return Some(m.format(&language(&t.locale), args));
            }
        }
        None
    }

    /// Whether the active locale itself (not a fallback) has `key`.
    pub fn has(&self, key: &str) -> bool {
        self.active().is_some_and(|t| t.entries.contains_key(key))
    }

    /// The active direction (left to right without tables).
    pub fn direction(&self) -> Direction {
        self.active().map_or(Direction::Ltr, |t| t.direction)
    }
}
