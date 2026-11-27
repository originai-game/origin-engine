//! Checked TS dialogues and quests share one native snapshot through planning,
//! diagnostics and execution. Yarn is an explicit input branch of file.dialogue.

use super::{CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, stamp};
use crate::format::{self, Area, BlobKind};
use og_blob::BlobWriter;

fn cook_story(s: &Source, stamp: u64) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let asset = s
        .story
        .as_ref()
        .ok_or_else(|| CookError("story was not prepared".into()))?;
    let data = asset.to_bytes().map_err(CookError)?;
    let (mut w, note) = if let og_story::asset::Asset::Quest(q) = asset {
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
        let og_story::asset::Asset::Dialogue(d) = asset else {
            unreachable!()
        };
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
    if matches!(asset, og_story::asset::Asset::Quest(_)) {
        w.table(
            "meta",
            &[format::QuestMeta {
                data_bytes: data.len() as u32,
            }],
            format::row_hash::<format::QuestMeta>(),
        );
    } else {
        w.table(
            "meta",
            &[format::DialogueMeta {
                data_bytes: data.len() as u32,
            }],
            format::row_hash::<format::DialogueMeta>(),
        );
    }
    w.raw("data", data, 16);
    Ok((w.finish(), vec![note]))
}

/// The cook's check of dialogues and quests against the base string table:
/// every key they show is in it, and every argument its message takes is
/// passed. As (source, line) warnings; nothing without a string table.
pub(crate) fn check_story_keys(
    tables: &[(&str, &og_i18n::StringTable)],
    stories: &[(&str, &og_story::asset::Asset)],
) -> Vec<(String, String)> {
    let parsed: Vec<og_i18n::StringTable> =
        tables.iter().map(|(_, table)| (*table).clone()).collect();
    let Some(base) = og_i18n::base_table(&parsed) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (rel, asset) in stories {
        let keys = match asset {
            og_story::asset::Asset::Dialogue(d) => d.keys(),
            og_story::asset::Asset::Quest(q) => q.keys(),
        };
        for (key, args, line) in keys {
            let at = if line > 0 {
                format!("line {line}: ")
            } else {
                String::new()
            };
            match base.entries.get(&key) {
                None if matches!(asset, og_story::asset::Asset::Dialogue(d) if d.strings.contains_key(&key)) =>
                    {}
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
                                    "{at}the message '{key}' takes {{{a}}}, which nothing passes: add '{a}' to this text's args"
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

/// (source, message) findings of a check.
type Findings = Vec<(String, String)>;

/// The cook's check of the dialogues og.flow plays (those with a node
/// headed `og-flow`): each flow node's segment is a timeline og.flow can
/// play (`segment` answers it by asset id: `None` when there is none), at
/// most one of its choices is `#default`, it says no Yarn lines, and every
/// node with choices is a flow node. As (source, error) errors, then
/// (source, warning) warnings: a node whose segment holds for the game
/// (`flow.hold`) and which branches on the story's variables before its
/// choices are taken (they are read as the segment starts, before the
/// game is played; its results belong in `#outcome` choices).
pub(crate) fn check_flows(
    stories: &[(&str, &og_story::asset::Asset)],
    segment: &dyn Fn(&str) -> Option<Result<og_timeline::segment::Segment, String>>,
    video_info: &dyn Fn(&str) -> Option<(f64, f64)>,
) -> (Findings, Findings) {
    let mut out = Vec::new();
    let mut warnings = Vec::new();
    for (rel, asset) in stories {
        let og_story::asset::Asset::Dialogue(d) = asset else {
            continue;
        };
        let graph = d.graph();
        if !graph.iter().any(|n| n.headers.contains_key("og-flow")) {
            continue;
        }
        let mut errors = Vec::new();
        for n in &graph {
            for &line in &n.dynamic_jumps {
                warnings.push((
                    rel.to_string(),
                    format!(
                        "line {line}: node '{}' uses a dynamic jump (`<<jump {{...}}>>`), which prevents static deck pre-warming and route analysis (`og flow routes`)",
                        n.title
                    ),
                ));
            }
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
                        "node '{}': its segment '{seg}' is not a timeline declaration",
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
                    if s.hold.is_some() {
                        for (line, vars) in d.branches_in(&n.title) {
                            let vars: Vec<String> = vars.iter().map(|v| format!("${v}")).collect();
                            warnings.push((
                                rel.to_string(),
                                format!(
                                    "line {line}: node '{}' plays {seg}, which holds for the game (flow.hold), but it branches on {} as the segment starts, before the game is played: give each result a choice tagged #outcome:<name> (`-> won #outcome:won` then its jump) and have the game call og.flow.advance(world, flow, \"<name>\") when it is decided",
                                    n.title,
                                    vars.join(", ")
                                ),
                            ));
                        }
                    }
                    if let Some((vid, from)) = &s.video
                        && let Some((vid_secs, keyframe)) = video_info(vid)
                    {
                        if f64::from(*from) + f64::from(s.cut) > vid_secs + 0.05 {
                            warnings.push((
                                rel.to_string(),
                                format!(
                                    "node '{}': segment {seg} plays {vid} from {from:.2}s to cut {:.2}s, past the video's {:.2}s length",
                                    n.title, s.cut, vid_secs
                                ),
                            ));
                        }
                        if keyframe > 0.0 && *from > 0.0 {
                            let rem = (f64::from(*from) / keyframe).fract();
                            if rem > 0.02 && rem < 0.98 {
                                warnings.push((
                                    rel.to_string(),
                                    format!(
                                        "node '{}': segment {seg} starts {vid} at `from {from:.2}`, which is not aligned to its {keyframe:.2}s keyframe interval (adds seek decode latency)",
                                        n.title
                                    ),
                                ));
                            }
                        }
                    }
                    Some(s.hotspots.into_iter().map(|k| k.id).collect())
                }
            };
            if let Some(h) = n.headers.get("og-flow-dissolve")
                && let Err(e) = og_timeline::segment::dissolve(h)
            {
                errors.push(format!("node '{}': {e}", n.title));
            }
            if let Some(h) = n.headers.get("og-flow-music")
                && let Err(e) = og_timeline::segment::music_header(h)
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
            // Group-aware #default validation: at most one #default per option group.
            let mut by_group: std::collections::BTreeMap<u32, usize> =
                std::collections::BTreeMap::new();
            for c in n.choices.iter().filter(|c| c.tags.contains_key("default")) {
                *by_group.entry(c.group).or_default() += 1;
            }
            for (group, defaults) in by_group {
                if defaults > 1 {
                    let which = if group == 0 {
                        String::new()
                    } else {
                        format!(" in option group {group}")
                    };
                    errors.push(format!(
                        "node '{}': {defaults} choices are #default{which}; one at most",
                        n.title
                    ));
                }
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
    (out, warnings)
}

pub(crate) static DIALOGUE: KindDef = KindDef {
    name: "dialogue",
    extensions: &[],
    types: &[],
    by_content: true,
    streams: false,
    plan: plan_dialogue,
};

pub(crate) static QUEST: KindDef = KindDef {
    name: "quest",
    extensions: &[],
    types: &[],
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
    let story_functions_stamp = cx.story_functions_stamp;
    let mut segments = std::collections::BTreeSet::new();
    if let Some(mut asset) = s.story.clone() {
        asset
            .map_resources(|_, id| {
                segments.insert(id.clone());
                Ok(())
            })
            .expect("collect resource IDs");
    }
    if !segments.is_empty() {
        state
            .parsed_refs
            .entry(s.rel.clone())
            .or_default()
            .extend(segments);
    }
    let st = stamp(&[Area::Data], &[b"dialogue", &hash, story_functions_stamp]);
    outs.push((
        s.rel.clone(),
        format::Dialogue::NAME,
        st,
        Box::new(StoryJob(s)),
    ));
}

fn plan_quest<'a>(
    _cx: &Cx<'a>,
    _state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let st = stamp(&[Area::Data], &[b"quest", &hash]);
    outs.push((
        s.rel.clone(),
        format::Quest::NAME,
        st,
        Box::new(StoryJob(s)),
    ));
}

/// A story's one native output.
struct StoryJob<'a>(&'a Source);

impl Job for StoryJob<'_> {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        cook_story(self.0, p.stamp)
    }
}
