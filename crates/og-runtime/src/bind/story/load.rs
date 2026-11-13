//! Loading a story asset and its voices.

use super::*;

pub(super) fn insert(engine_id: u32, text: &str) -> Result<StoryAsset, String> {
    let ty = if text
        .lines()
        .map(str::trim)
        .find(|s| !s.is_empty() && !s.starts_with("//") && !s.starts_with('#'))
        .is_some_and(|s| s.starts_with("type "))
    {
        og_core::ogt::Document::parse(text)
            .map_err(|e| e.reason().to_string())?
            .ty
    } else {
        og_story::DIALOGUE_TYPE.to_string()
    };
    // The first load registers og.Story and the events (outside a system).
    let eng = engine(engine_id)?;
    match eng.world() {
        Ok(world) => crate::engine::lazy::ensure(&eng, world, STORY)?,
        Err(e) => {
            if !with(engine_id, |rt| rt.story.is_some()) {
                return Err(format!(
                    "the first og.story.load registers og.Story, which a system cannot do: load dialogues and quests in setup ({e})"
                ));
            }
        }
    }
    if ty == og_story::QUEST_TYPE {
        let q = Quest::parse(text)?;
        let info = StoryAsset {
            kind: "quest".into(),
            name: q.id.clone(),
            nodes: q.stages.iter().map(|s| s.id.clone()).collect(),
            speakers: Vec::new(),
            variables: q.vars.iter().map(|v| v.name.clone()).collect(),
        };
        let id = q.id.clone();
        let old = with(engine_id, |rt| {
            rt.lib.quests.insert(id.clone(), Arc::new(q))
        });
        if let Err(e) = grant(engine_id, eng.world().ok()) {
            with(engine_id, |rt| match old {
                Some(o) => rt.lib.quests.insert(id, o),
                None => rt.lib.quests.remove(&id),
            });
            return Err(e);
        }
        Ok(info)
    } else if ty == og_story::DIALOGUE_TYPE {
        let d = if text.trim_start().starts_with('{') {
            Dialogue::decode(text)?
        } else {
            Dialogue::parse(text)?
        };
        crate::bind::i18n::base_strings(&eng, &d.locale, &d.strings)?;
        load_voices(&eng, engine_id, &d);
        let info = StoryAsset {
            kind: "dialogue".into(),
            name: d.name.clone(),
            nodes: d
                .nodes
                .iter()
                .filter(|n| n.id == n.title)
                .map(|n| n.id.clone())
                .collect(),
            speakers: d.speakers.iter().map(|s| s.id.clone()).collect(),
            variables: d.vars.iter().map(|v| v.name.clone()).collect(),
        };
        let name = d.name.clone();
        let old = with(engine_id, |rt| {
            rt.lib.dialogues.insert(name.clone(), Arc::new(d))
        });
        if let Err(e) = grant(engine_id, eng.world().ok()) {
            with(engine_id, |rt| match old {
                Some(o) => rt.lib.dialogues.insert(name, o),
                None => rt.lib.dialogues.remove(&name),
            });
            return Err(e);
        }
        Ok(info)
    } else {
        Err(format!(
            "expected Yarn dialogue or 'type quest', found 'type {ty}'"
        ))
    }
}

/// Loads each voice clip a dialogue names into the engine's sounds, and
/// records its length in ticks (a one-shot's, as og.audio ends it). A clip
/// that does not load is warned about; its line shows without a voice.
pub(super) fn load_voices(eng: &crate::engine::Engine, engine_id: u32, d: &Dialogue) {
    let clips: std::collections::BTreeSet<String> = d
        .nodes
        .iter()
        .flat_map(|n| n.steps.iter())
        .filter_map(|s| match s {
            og_story::Step::Say(l) => Some(l.voice.clone()),
            _ => None,
        })
        .chain(
            d.barks
                .iter()
                .flat_map(|b| b.lines.iter().map(|l| l.voice.clone())),
        )
        .filter(|v| !v.is_empty())
        .collect();
    let rate = f64::from(eng.state.borrow().rate.max(1));
    let mut warnings = Vec::new();
    for clip in clips {
        if with(engine_id, |rt| rt.voices.contains_key(&clip)) {
            continue;
        }
        match crate::world::assets::load_sound(eng, &clip) {
            Ok(i) => {
                let (handle, ticks) = {
                    let a = eng.assets.borrow();
                    let m = &a.sounds[i].meta;
                    let ticks =
                        (f64::from(m.frames) * rate / f64::from(m.sample_rate.max(1))).ceil();
                    (a.sound_handles[i], ticks as u32)
                };
                with(engine_id, |rt| {
                    rt.voices.insert(clip.clone(), handle);
                    rt.lib.voice_ticks.insert(clip, ticks);
                });
            }
            Err(e) => warnings.push(format!(
                "og.story: dialogue '{}': its voice '{clip}' does not load ({e}); its lines show without it",
                d.name
            )),
        }
    }
    let mut st = eng.state.borrow_mut();
    for w in warnings {
        st.log
            .push(json!({ "tick": og_ecs::last_tick(), "warning": true, "text": w }));
    }
}

pub fn og_story_load(world: u32, path: String) -> Result<crate::host::wire::StoryAsset, String> {
    (|| {
        let eng = engine(world)?;
        let id = og_bundle::runtime::asset_id(&path).to_string();
        let text = {
            let assets = eng.assets.borrow();
            let (bytes, bundle) = assets.blob(&id)?;
            bundle
                .story_text(&id, bytes.bytes())
                .map_err(|e| e.to_string())?
        };
        insert(world, &text).map_err(|e| format!("{id}: {e}"))
    })()
}

pub fn og_story_parse(world: u32, text: String) -> Result<crate::host::wire::StoryAsset, String> {
    insert(world, &text)
}
