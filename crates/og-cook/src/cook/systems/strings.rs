//! String tables: checked native metadata and ICU sources, never author text.

use super::{CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, stamp};
use crate::format::{self, Area, BlobKind};
use og_blob::BlobWriter;

pub(crate) fn declaration(
    input: crate::cook::source_schemas::world::localization::StringsInput,
) -> Result<og_i18n::StringTable, String> {
    use crate::cook::source_schemas::world::localization::Direction;
    og_i18n::StringTable::new(
        input.locale,
        input.fallback,
        input.base.unwrap_or(false),
        input.direction.map(|d| match d {
            Direction::Ltr => og_i18n::Direction::Ltr,
            Direction::Rtl => og_i18n::Direction::Rtl,
        }),
        input.fonts.into_iter().map(|f| f.id).collect(),
        input.entries,
    )
}

/// Cooks native locale fields. ICU source text is retained for formatting.
pub(crate) fn cook_strings(s: &Source, stamp: u64) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let t = s.strings.as_ref().expect("prepared string table");
    let mut text = super::super::formats::model::Strings(Vec::new());
    let (locale_offset, locale_len) = text.add(&t.locale);
    let (fallback_offset, fallback_len) = text.add(t.fallback.as_deref().unwrap_or(""));
    let entries: Vec<_> = t
        .entries
        .iter()
        .map(|(key, (source, _))| {
            let (key_offset, key_len) = text.add(key);
            let (message_offset, message_len) = text.add(source);
            format::StringEntryRow {
                key_offset,
                key_len,
                message_offset,
                message_len,
            }
        })
        .collect();
    let fonts: Vec<_> = t
        .fonts
        .iter()
        .map(|font| {
            let (offset, len) = text.add(font);
            format::StringFontRow { offset, len }
        })
        .collect();
    let mut w = BlobWriter::new(
        format::Strings::type_hash(),
        format::Strings::schema_version(),
        stamp,
    );
    w.table(
        "meta",
        &[format::StringsMeta {
            locale_offset,
            locale_len,
            fallback_offset,
            fallback_len,
            base: u32::from(t.base),
            direction: u32::from(t.direction == og_i18n::Direction::Rtl),
        }],
        format::row_hash::<format::StringsMeta>(),
    )
    .table(
        "entries",
        &entries,
        format::row_hash::<format::StringEntryRow>(),
    )
    .table("fonts", &fonts, format::row_hash::<format::StringFontRow>())
    .raw("text", text.0, 1);
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
/// Only the project's own tables: an engine table the bundle holds
/// (`@ui/…`) belongs to its component's catalogue, not the project's, and
/// the engine's own test keeps its locales in step.
pub(crate) fn check_strings(sources: &[(&str, &og_i18n::StringTable)]) -> Vec<(String, String)> {
    let tables: Vec<(String, og_i18n::StringTable)> = sources
        .iter()
        .filter(|(rel, _)| !rel.starts_with('@'))
        .map(|(rel, table)| (rel.to_string(), (*table).clone()))
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
    types: &[],
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
    let fonts = s
        .strings
        .as_ref()
        .expect("prepared string table")
        .fonts
        .join("\0");
    let st = stamp(&[Area::Data], &[b"strings", &hash, fonts.as_bytes()]);
    outs.push((
        s.rel.clone(),
        format::Strings::NAME,
        st,
        Box::new(StringsJob(s)),
    ));
}

/// A string table's one output.
struct StringsJob<'a>(&'a Source);

impl Job for StringsJob<'_> {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        cook_strings(self.0, p.stamp)
    }
}

#[cfg(test)]
mod tests {
    /// Every string table the engine's UI ships in English has a zh-CN
    /// translation beside it with exactly its keys, each taking the same
    /// arguments: the two cannot drift apart.
    #[test]
    fn engine_ui_tables_have_a_matching_zh_cn_table() {
        let ui =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/engine/src/ui");
        let mut stack = vec![ui];
        let mut pairs = 0;
        let mut gaps = Vec::new();
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                let Some(stem) = name.strip_suffix(".en.strings.ts") else {
                    continue;
                };
                let zh = path.with_file_name(format!("{stem}.zh-CN.strings.ts"));
                let read = |p: &std::path::Path| {
                    super::super::strings_extract::read_builtin(
                        p.parent().unwrap(),
                        p.file_name().unwrap().to_str().unwrap(),
                    )
                    .map_err(|e| format!("{}: {e}", p.display()))
                };
                let (en, zh_table) = match (read(&path), read(&zh)) {
                    (Ok(en), Ok(zh)) => (en, zh),
                    (Err(e), _) | (_, Err(e)) => {
                        gaps.push(e);
                        continue;
                    }
                };
                if zh_table.locale != "zh-CN" {
                    gaps.push(format!(
                        "{}: locale '{}', not zh-CN",
                        zh.display(),
                        zh_table.locale
                    ));
                }
                pairs += 1;
                for g in og_i18n::check_tables(&[en, zh_table]) {
                    gaps.push(format!(
                        "{}: {} key '{}' ({})",
                        zh.display(),
                        g.problem,
                        g.key,
                        g.detail
                    ));
                }
            }
        }
        assert!(pairs > 0, "no engine UI tables found");
        assert!(gaps.is_empty(), "{}", gaps.join("\n"));
    }
}
