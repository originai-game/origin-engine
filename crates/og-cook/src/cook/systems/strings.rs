//! String tables (`type strings`): validated by og-i18n and kept as text.

use super::{CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, Work, stamp};
use crate::format::{self, BlobKind};
use og_blob::BlobWriter;

/// Cooks a string table (`type strings`): validated by og-i18n, kept as text.
pub(crate) fn cook_strings(s: &Source, stamp: u64) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let text = std::str::from_utf8(&s.bytes)
        .map_err(|_| CookError("a string table is text (UTF-8)".into()))?;
    let t = og_i18n::StringTable::parse(text).map_err(CookError)?;
    let mut w = BlobWriter::new(
        format::Strings::type_hash(),
        format::Strings::schema_version(),
        stamp,
    );
    w.table(
        "meta",
        &[format::StringsMeta {
            text_bytes: text.len() as u32,
            strings: t.entries.len() as u32,
        }],
        format::row_hash::<format::StringsMeta>(),
    )
    .raw("text", text.as_bytes().to_vec(), 16);
    Ok((
        w.finish(),
        vec![format!(
            "strings '{}' ({}): {} entries{}",
            t.locale,
            t.direction.as_str(),
            t.entries.len(),
            if t.fonts.is_empty() {
                String::new()
            } else {
                format!(", fonts {}", t.fonts.join(", "))
            }
        )],
    ))
}

/// The cook's check across a project's string tables: per table, every
/// key the base table has and it lacks, takes other arguments for, or
/// has that the base lacks (og-i18n's `check_tables`), as (source, line).
pub(crate) fn check_strings(sources: &[(&str, &[u8])]) -> Vec<(String, String)> {
    let tables: Vec<(String, og_i18n::StringTable)> = sources
        .iter()
        .filter_map(|(rel, bytes)| {
            let t = og_i18n::StringTable::parse(std::str::from_utf8(bytes).ok()?).ok()?;
            Some((rel.to_string(), t))
        })
        .collect();
    let only: Vec<og_i18n::StringTable> = tables.iter().map(|(_, t)| t.clone()).collect();
    og_i18n::check_tables(&only)
        .into_iter()
        .filter_map(|g| {
            let rel = tables.iter().find(|(_, t)| t.locale == g.locale)?.0.clone();
            let what = match g.problem {
                "missing" => format!("missing key '{}' ({})", g.key, g.detail),
                "arguments" => format!("key '{}' takes other arguments ({})", g.key, g.detail),
                _ => format!("key '{}' is not in the base table ({})", g.key, g.detail),
            };
            Some((rel, what))
        })
        .collect()
}

pub(crate) static STRINGS: KindDef = KindDef {
    name: "strings",
    extensions: &[],
    types: &[og_i18n::STRINGS_TYPE],
    by_content: true,
    streams: false,
    plan: plan_strings,
};

/// The kinds this module cooks.
pub(crate) static KINDS: &[&KindDef] = &[&STRINGS];

fn plan_strings<'a>(
    _cx: &Cx<'a>,
    _state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let st = stamp(&[b"strings", &hash]);
    outs.push((
        s.rel.clone(),
        format::Strings::NAME,
        st,
        Work::Job(Box::new(StringsJob(s))),
    ));
}

/// A string table's one output.
struct StringsJob<'a>(&'a Source);

impl Job for StringsJob<'_> {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        cook_strings(self.0, p.stamp)
    }
}
