//! Dialogues and quests: Yarn compiled to the engine's IR, quests kept as
//! data, and their keys checked against the base string table.

use super::{CookError, Source};
use crate::format::{self, BlobKind};
use og_blob::BlobWriter;

/// Yarn is compiled to engine IR; quests retain their data representation.
pub(crate) fn cook_story(
    s: &Source,
    stamp: u64,
    quest: bool,
    functions: Option<&std::collections::BTreeMap<String, og_story::yarn::types::Function>>,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let what = if quest { "quest" } else { "dialogue" };
    let text = std::str::from_utf8(&s.bytes)
        .map_err(|_| CookError(format!("a {what} is text (UTF-8)")))?;
    let compiled;
    let text = if quest {
        text
    } else {
        compiled =
            og_story::yarn::compile_with_functions(text, functions.cloned().unwrap_or_default())
                .map_err(CookError)?
                .encode()
                .map_err(CookError)?;
        &compiled
    };
    let (mut w, note) = if quest {
        let q = og_story::Quest::parse(text).map_err(CookError)?;
        let w = BlobWriter::new(
            format::Quest::type_hash(),
            format::Quest::schema_version(),
            stamp,
        );
        let objectives: usize = q.stages.iter().map(|s| s.objectives.len()).sum();
        let note = format!(
            "quest '{}': {} stages, {objectives} objectives",
            q.id,
            q.stages.len()
        );
        (w, note)
    } else {
        let d = og_story::Dialogue::decode(text).map_err(CookError)?;
        for (key, text) in &d.strings {
            og_i18n::Message::parse(text)
                .map_err(|e| CookError(format!("Yarn base string {key}: {e}")))?;
        }
        let w = BlobWriter::new(
            format::Dialogue::type_hash(),
            format::Dialogue::schema_version(),
            stamp,
        );
        let choices: usize = d.nodes.iter().map(|n| n.choices.len()).sum();
        let note = format!(
            "dialogue '{}': {} nodes, {choices} choices, {} speakers",
            d.name,
            d.nodes.len(),
            d.speakers.len()
        );
        (w, note)
    };
    if quest {
        w.table(
            "meta",
            &[format::QuestMeta {
                text_bytes: text.len() as u32,
            }],
            format::row_hash::<format::QuestMeta>(),
        );
    } else {
        w.table(
            "meta",
            &[format::DialogueMeta {
                text_bytes: text.len() as u32,
            }],
            format::row_hash::<format::DialogueMeta>(),
        );
    }
    w.raw("text", text.as_bytes().to_vec(), 16);
    Ok((w.finish(), vec![note]))
}

/// The cook's check of dialogues and quests against the base string table:
/// every key they show is in it, and every argument its message takes is
/// passed. As (source, line) warnings; nothing without a string table.
pub(crate) fn check_story_keys(
    tables: &[(&str, &[u8])],
    stories: &[(&str, &[u8])],
    functions: &std::collections::BTreeMap<String, og_story::yarn::types::Function>,
) -> Vec<(String, String)> {
    let parsed: Vec<og_i18n::StringTable> = tables
        .iter()
        .filter_map(|(_, b)| og_i18n::StringTable::parse(std::str::from_utf8(b).ok()?).ok())
        .collect();
    let Some(base) = og_i18n::base_table(&parsed) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (rel, bytes) in stories {
        let Ok(text) = std::str::from_utf8(bytes) else {
            continue;
        };
        let result = if rel.ends_with(".yarn") {
            og_story::yarn::compile_with_functions(text, functions.clone())
                .map(|d| (d.keys(), d.strings))
        } else {
            match og_core::ogt::Document::parse(text).map(|d| d.ty) {
                Ok(t) if t == og_story::QUEST_TYPE => {
                    og_story::Quest::parse(text).map(|q| (q.keys(), Default::default()))
                }
                _ => continue,
            }
        };
        let Ok((keys, authored)) = result else {
            continue;
        };
        for (key, args, line) in keys {
            let at = if line > 0 {
                format!("line {line}: ")
            } else {
                String::new()
            };
            match base.entries.get(&key) {
                None if authored.contains_key(&key) => {}
                None => out.push((
                    rel.to_string(),
                    format!(
                        "{at}the key '{key}' is not in the base string table ({}): add it there",
                        base.locale
                    ),
                )),
                Some((_, m)) => {
                    for a in m.arguments() {
                        if !args.contains(&a) {
                            out.push((
                                rel.to_string(),
                                format!(
                                    "{at}the message '{key}' takes {{{a}}}, which nothing passes: add 'arg {a} \"...\"'"
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }
    out
}
