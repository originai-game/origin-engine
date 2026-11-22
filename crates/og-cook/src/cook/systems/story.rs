//! Dialogues and quests: Yarn compiled to the engine's IR, quests kept as
//! data, and their keys checked against the base string table.

use super::{CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, stamp};
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

/// The cook's check of the dialogues og.flow plays (those with a node
/// headed `og-flow`): each flow node's segment is a timeline og.flow can
/// play (`segment` answers it by asset id: `None` when there is none), at
/// most one of its choices is `#default`, it says no Yarn lines, and every
/// node with choices is a flow node. As (source, error).
pub(crate) fn check_flows(
    stories: &[(&str, &[u8])],
    segment: &dyn Fn(&str) -> Option<Result<og_timeline::segment::Segment, String>>,
    functions: &std::collections::BTreeMap<String, og_story::yarn::types::Function>,
) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (rel, bytes) in stories.iter().filter(|(r, _)| r.ends_with(".yarn")) {
        let Some(d) = std::str::from_utf8(bytes)
            .ok()
            .and_then(|t| og_story::yarn::compile_with_functions(t, functions.clone()).ok())
        else {
            continue;
        };
        let graph = d.graph();
        if !graph.iter().any(|n| n.headers.contains_key("og-flow")) {
            continue;
        }
        let mut errors = Vec::new();
        for n in &graph {
            let Some(seg) = n.headers.get("og-flow") else {
                if !n.choices.is_empty() {
                    errors.push(format!(
                        "node '{}' offers choices but has no og-flow segment: in a flow every choice is a flow node's, timed by its segment",
                        n.title
                    ));
                }
                continue;
            };
            let hotspots: Option<Vec<String>> = match segment(seg) {
                None => {
                    errors.push(format!(
                        "node '{}': its segment '{seg}' is no timeline under assets/",
                        n.title
                    ));
                    None
                }
                Some(Err(e)) => {
                    errors.push(format!("node '{}': segment {seg}: {e}", n.title));
                    None
                }
                Some(Ok(s)) => {
                    if let Err(e) = og_timeline::segment::outcomes(
                        s.hold.is_some(),
                        n.choices.iter().map(|c| (c.index, &c.tags)),
                    ) {
                        errors.push(format!("node '{}': {e}", n.title));
                    }
                    Some(s.hotspots.into_iter().map(|k| k.id).collect())
                }
            };
            if let Some(h) = n.headers.get("og-flow-dissolve")
                && let Err(e) = og_timeline::segment::dissolve(h)
            {
                errors.push(format!("node '{}': {e}", n.title));
            }
            for c in &n.choices {
                if let (Some(id), Some(known)) = (c.tags.get("hotspot"), &hotspots)
                    && !known.contains(id)
                {
                    errors.push(format!(
                        "node '{}': choice {} is #hotspot:{id}, which its segment {seg} has no `track hotspot` key for",
                        n.title, c.index
                    ));
                }
                if let Some(q) = c.tags.get("qte") {
                    if let Err(e) = og_timeline::segment::Qte::parse(q) {
                        errors.push(format!("node '{}': choice {}: {e}", n.title, c.index));
                    } else if c.index as usize >= og_timeline::segment::QTE_CHOICES {
                        errors.push(format!(
                            "node '{}': choice {} is a #qte; only a node's first {} choices may be",
                            n.title,
                            c.index,
                            og_timeline::segment::QTE_CHOICES
                        ));
                    }
                }
            }
            let defaults = n
                .choices
                .iter()
                .filter(|c| c.tags.contains_key("default"))
                .count();
            if defaults > 1 {
                errors.push(format!(
                    "node '{}': {defaults} choices are #default; one at most",
                    n.title
                ));
            }
            let said = d.said_in(&n.title);
            if !said.is_empty() {
                let lines: Vec<String> = said.iter().map(usize::to_string).collect();
                errors.push(format!(
                    "node '{}' says Yarn lines (line {}): a flow node shows none; put what is said in its segment's caption track",
                    n.title,
                    lines.join(", ")
                ));
            }
        }
        out.extend(errors.into_iter().map(|e| (rel.to_string(), e)));
    }
    out
}

pub(crate) static DIALOGUE: KindDef = KindDef {
    name: "dialogue",
    extensions: &["yarn"],
    types: &[og_story::DIALOGUE_TYPE],
    by_content: true,
    streams: false,
    plan: plan_dialogue,
};

pub(crate) static QUEST: KindDef = KindDef {
    name: "quest",
    extensions: &[],
    types: &[og_story::QUEST_TYPE],
    by_content: true,
    streams: false,
    plan: plan_quest,
};

/// The kinds this module cooks.
pub(crate) static KINDS: &[&KindDef] = &[&DIALOGUE, &QUEST];

fn plan_dialogue<'a>(
    cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let Cx {
        story_functions,
        story_functions_stamp,
        ..
    } = *cx;
    // A flow node's `og-flow: <segment>` header: the dialogue reaches the
    // segment, and through it the segment's footage and scenes (needs.rs).
    let segments: std::collections::BTreeSet<String> = String::from_utf8_lossy(&s.bytes)
        .lines()
        .filter_map(|l| l.trim().strip_prefix("og-flow:"))
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect();
    if !segments.is_empty() {
        state
            .parsed_refs
            .entry(s.rel.clone())
            .or_default()
            .extend(segments);
    }
    let st = stamp(&[b"dialogue", &hash, story_functions_stamp]);
    outs.push((
        s.rel.clone(),
        format::Dialogue::NAME,
        st,
        Box::new(DialogueJob(s, story_functions)),
    ));
}

fn plan_quest<'a>(
    _cx: &Cx<'a>,
    _state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let st = stamp(&[b"quest", &hash]);
    outs.push((
        s.rel.clone(),
        format::Quest::NAME,
        st,
        Box::new(QuestJob(s)),
    ));
}

/// A dialogue's one output, compiled with the project's story functions.
struct DialogueJob<'a>(
    &'a Source,
    &'a std::collections::BTreeMap<String, og_story::yarn::types::Function>,
);

impl Job for DialogueJob<'_> {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        cook_story(self.0, p.stamp, false, Some(self.1))
    }
}

/// A quest's one output.
struct QuestJob<'a>(&'a Source);

impl Job for QuestJob<'_> {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        cook_story(self.0, p.stamp, true, None)
    }
}
