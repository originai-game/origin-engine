//! Barks and ambient lines.

use super::*;

/// A bark's entity and topic, which it needs.
pub(super) fn bark_target(entity: u64, topic: &str) -> Result<(), String> {
    if entity == 0 {
        return Err("the desc's entity is the entity that barks".into());
    }
    if topic.is_empty() {
        return Err("the desc's topic names the barks to choose from, such as idle".into());
    }
    Ok(())
}

pub fn og_story_bark(world: u32, desc: Option<String>) -> Result<(), String> {
    (|| {
        let d: crate::host::wire::BarkDesc = crate::host::wire::arg(desc.as_deref(), "desc")?;
        bark_target(d.entity, &d.topic)?;
        story(world, |lib, s| s.bark(lib, d.entity, &d.topic, &d.dialogue))?
    })()
}

pub fn og_story_ambient(world: u32, desc: Option<String>) -> Result<(), String> {
    (|| {
        let d: crate::host::wire::AmbientDesc = crate::host::wire::arg(desc.as_deref(), "desc")?;
        bark_target(d.entity, &d.topic)?;
        let (entity, topic, dialogue) = (d.entity, d.topic, d.dialogue);
        let every = d.every.min(1_000_000_000);
        let jitter = d.jitter.min(1_000_000_000);
        story(world, |lib, s| {
            if every > 0 {
                let known = lib
                    .dialogues
                    .values()
                    .filter(|x| dialogue.is_empty() || x.name == dialogue)
                    .any(|x| x.barks.iter().any(|b| b.topic == topic));
                if !known {
                    return Err(format!("no loaded dialogue has a bark of topic '{topic}'"));
                }
            }
            s.ambient(entity, &topic, &dialogue, every, jitter);
            Ok(())
        })?
    })()
}

pub(super) fn bark_view(
    engine: u32,
    lib: &Library,
    s: &Story,
    e: u64,
    b: &og_story::Said,
) -> BarkView {
    let args = args_json(&b.args);
    let d = lib.dialogues.get(&b.dialogue);
    let sp = d.and_then(|d| d.speaker(&b.speaker));
    let (text, spans) = rich_line(engine, &b.key, &args);
    BarkView {
        entity: e,
        dialogue: b.dialogue.clone(),
        bark: b.bark.clone(),
        seq: b.seq,
        key: b.key.clone(),
        args,
        text,
        spans,
        speaker: SpeakerView {
            id: b.speaker.clone(),
            name: sp.map_or(String::new(), |x| tr(engine, &x.name, &json!({}))),
            entity: e,
            portrait: sp.map_or("", |x| x.portrait.as_str()).to_string(),
            expression: sp.map_or("", |x| x.expression.as_str()).to_string(),
            side: sp.map_or("left", |x| x.side.as_str()).to_string(),
        },
        voice: b.voice.clone(),
        voice_left: b.voice_left(s.tick),
        ticks_left: b.until.saturating_sub(s.tick),
        priority: b.priority,
    }
}

pub fn og_story_barks(world: u32) -> Vec<crate::host::wire::BarkView> {
    story(world, |lib, s| {
        s.barks
            .iter()
            .map(|(e, b)| bark_view(world, lib, s, *e, b))
            .collect::<Vec<_>>()
    })
    .unwrap_or_default()
}
