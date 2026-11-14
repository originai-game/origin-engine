//! Conversations: started, viewed, advanced, chosen and stopped.

use super::*;

pub fn og_story_start(world: u32, desc: Option<String>) -> Result<serde_json::Value, String> {
    (|| {
        let d: crate::host::wire::StartDesc = crate::host::wire::arg(desc.as_deref(), "desc")?;
        if d.dialogue.is_empty() {
            return Err("StartDesc.dialogue names the dialogue to start".into());
        }
        let dialogue = d.dialogue;
        let node = Some(d.node.as_str()).filter(|s| !s.is_empty());
        let mut cast = BTreeMap::new();
        if let Some(o) = d.cast.as_object() {
            for (k, v) in o {
                let e = v
                    .as_f64()
                    .filter(|e| e.is_finite() && *e >= 0.0)
                    .ok_or_else(|| format!("StartDesc.cast.{k} is an entity"))?;
                cast.insert(k.clone(), e as u64);
            }
        }
        let mut vars = BTreeMap::new();
        if let Some(o) = d.vars.as_object() {
            for (k, v) in o {
                let v = val_of(v).ok_or_else(|| {
                    format!("StartDesc.vars.{k} is a number, a boolean or a string")
                })?;
                vars.insert(k.clone(), v);
            }
        }
        story(world, |lib, s| s.start(lib, &dialogue, cast, vars, node))?.map(|c| json!(c))
    })()
}

pub(super) fn speaker_view(
    engine: u32,
    d: &Dialogue,
    c: &og_story::Conv,
    id: &str,
    expression: &str,
) -> SpeakerView {
    let sp = d.speaker(id);
    let name = sp.map_or(String::new(), |s| tr(engine, &s.name, &json!({})));
    let expression = if expression.is_empty() {
        sp.map_or("", |s| s.expression.as_str())
    } else {
        expression
    };
    SpeakerView {
        id: id.to_string(),
        name,
        entity: c.cast.get(id).copied().unwrap_or(0),
        portrait: sp.map_or("", |s| s.portrait.as_str()).to_string(),
        expression: expression.to_string(),
        side: sp.map_or("left", |s| s.side.as_str()).to_string(),
    }
}

pub(super) fn story_line(
    engine: u32,
    d: &Dialogue,
    c: &og_story::Conv,
    l: &og_story::Shown,
) -> StoryLine {
    let args = args_json(&l.args);
    let (text, spans) = rich_line(engine, &l.key, &args);
    let chosen = l.chosen.as_ref().and_then(|(i, id)| {
        let choice = d
            .nodes
            .iter()
            .flat_map(|n| &n.choices)
            .find(|c| &c.id == id)?;
        let (text, spans) = rich_line(engine, &choice.text.key, &args_json(&l.chosen_args));
        Some(ChosenView {
            index: *i,
            text,
            spans,
        })
    });
    StoryLine {
        seq: l.seq,
        node: d
            .node(&l.node)
            .map_or(l.node.as_str(), |n| n.title.as_str())
            .to_string(),
        speaker: speaker_view(engine, d, c, &l.speaker, &l.expression),
        key: l.key.clone(),
        text,
        spans,
        args,
        voice: l.voice.clone(),
        seen: l.seen,
        chosen,
    }
}

pub fn og_story_view(world: u32, conversation: u32) -> Option<crate::host::wire::DialogueView> {
    story(world, |lib, s| -> Option<DialogueView> {
        let d = s.resolve(lib, conversation).ok()?;
        let c = s.convs.get(&conversation)?;
        let last = c.history.last();
        let mut choices = Vec::new();
        let mut timer = None;
        if c.wait == Wait::Choices
            && let Some(node) = d.node(&c.node)
        {
            for (i, ch) in node.choices.iter().enumerate() {
                let (visible, enabled, chance) = s.choice_state(lib, &d, conversation, ch);
                if !visible {
                    continue;
                }
                let args = args_json(&s.choice_args(conversation, ch));
                let values = c.choice_values.get(&ch.id);
                let (reason_key, reason_args) = match &ch.reason {
                    Some(r) => (
                        r.key.clone(),
                        values.map_or_else(
                            || args_json(&s.args(lib, Some(&d), conversation, &r.args)),
                            |v| args_json(&v.reason_args),
                        ),
                    ),
                    None => (String::new(), json!({})),
                };
                let check = ch.check.as_ref().map(|k| {
                    let target = values.and_then(|v| v.target).unwrap_or_else(|| {
                        match s.eval(lib, Some(&d), conversation, &k.target) {
                            Some(Val::Num(n)) => n,
                            _ => 0.0,
                        }
                    });
                    CheckView {
                        stat: k.stat.clone(),
                        target: target as f32,
                        chance: chance as f32,
                        kind: if k.active { "active" } else { "passive" }.to_string(),
                    }
                });
                let (text, spans) = rich_line(world, &ch.text.key, &args);
                choices.push(ChoiceView {
                    index: i as u32,
                    id: ch.id.clone(),
                    text,
                    spans,
                    key: ch.text.key.clone(),
                    args,
                    enabled,
                    reason: if enabled {
                        String::new()
                    } else {
                        tr(world, &reason_key, &reason_args)
                    },
                    reason_key,
                    reason_args,
                    visited: c.visited.contains(&format!("{}/{}", node.id, ch.id)),
                    kind: ch.kind.clone(),
                    check,
                });
            }
            if let (Some((left, total)), Some((_, t))) = (c.timer, &node.timer) {
                let default = node.choices.iter().position(|ch| &ch.id == t).unwrap_or(0);
                timer = Some(TimerView {
                    ticks_left: left,
                    ticks_total: total,
                    default_choice: default as u32,
                });
            }
        }
        let empty = og_story::Shown {
            node: c.node.clone(),
            step: 0,
            speaker: og_story::NARRATOR.into(),
            key: String::new(),
            args: Vec::new(),
            voice: String::new(),
            expression: String::new(),
            shot: String::new(),
            seq: 0,
            seen: false,
            chosen: None,
            chosen_args: Vec::new(),
            voice_ticks: 0,
            voice_start: None,
        };
        let l = last.unwrap_or(&empty);
        let args = args_json(&l.args);
        let (text, spans) = rich_line(world, &l.key, &args);
        Some(DialogueView {
            conversation,
            node: d
                .node(&c.node)
                .map_or(c.node.as_str(), |n| n.title.as_str())
                .to_string(),
            seq: l.seq,
            seen: l.seen,
            key: l.key.clone(),
            args,
            text,
            spans,
            voice: l.voice.clone(),
            voice_ticks: if l.voice.is_empty() { 0 } else { l.voice_ticks },
            voice_ticks_left: if c.wait == Wait::Ended {
                0
            } else {
                l.voice_left(s.tick)
            },
            speaker: speaker_view(world, &d, c, &l.speaker, &l.expression),
            shot: l.shot.clone(),
            choices,
            timer,
            last_check: c.last_roll.as_ref().map(|r| RollView {
                stat: r.stat.clone(),
                target: r.target as f32,
                roll: r.roll,
                passed: r.passed,
            }),
            ended: c.wait == Wait::Ended,
        })
    })
    .ok()
    .flatten()
}

pub fn og_story_history(world: u32, conversation: u32) -> Vec<crate::host::wire::StoryLine> {
    story(world, |lib, s| -> Option<Vec<StoryLine>> {
        let c = s.convs.get(&conversation)?;
        let d = lib.dialogues.get(&c.dialogue)?;
        Some(
            c.history
                .iter()
                .map(|l| story_line(world, d, c, l))
                .collect(),
        )
    })
    .ok()
    .flatten()
    .unwrap_or_default()
}

pub fn og_story_conversations(world: u32) -> Vec<u32> {
    story(world, |_, s| s.convs.keys().copied().collect()).unwrap_or_default()
}

pub fn og_story_advance(world: u32, conversation: u32) -> Result<(), String> {
    story(world, |lib, s| s.advance(lib, conversation)).and_then(|r| r)
}

pub fn og_story_choose(world: u32, conversation: u32, index: u32) -> Result<(), String> {
    story(world, |lib, s| s.choose(lib, conversation, index)).and_then(|r| r)
}

pub fn og_story_stop(world: u32, conversation: u32) -> bool {
    story(world, |_, s| s.stop(conversation)).unwrap_or(false)
}
