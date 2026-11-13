//! Quests: started, progressed, staged, ended, viewed and marked.

use super::*;

pub fn og_story_start_quest(world: u32, id: String) -> Result<serde_json::Value, String> {
    story(world, |lib, s| s.start_quest(lib, &id))
        .and_then(|r| r)
        .map(|b| json!(b))
}

pub fn og_story_progress(
    world: u32,
    quest: String,
    objective: String,
    amount: Option<u32>,
) -> bool {
    story(world, |lib, s| {
        s.progress(lib, &quest, &objective, amount.unwrap_or(1))
    })
    .unwrap_or(false)
}

pub(super) fn quest_op(world: u32, op: og_story::QuestOp) -> Result<serde_json::Value, String> {
    story(world, |lib, s| {
        let r = s.quest_op(lib, &op);
        s.settle(lib);
        r
    })
    .and_then(|r| r)
    .map(|b| json!(b))
}

pub fn og_story_set_stage(
    world: u32,
    quest: String,
    stage: String,
) -> Result<serde_json::Value, String> {
    quest_op(world, og_story::QuestOp::Stage(quest, stage))
}

pub fn og_story_complete_quest(world: u32, quest: String) -> Result<serde_json::Value, String> {
    quest_op(world, og_story::QuestOp::Complete(quest))
}

pub fn og_story_fail_quest(world: u32, quest: String) -> Result<serde_json::Value, String> {
    quest_op(world, og_story::QuestOp::Fail(quest))
}

pub(super) fn quest_view(
    engine: u32,
    lib: &Library,
    s: &Story,
    q: &og_story::QuestState,
) -> QuestView {
    let def = lib.quests.get(&q.id);
    let none = json!({});
    let t = |k: &str| tr(engine, k, &none);
    let stage = def.and_then(|d| d.stage(&q.stage));
    let mut objectives = Vec::new();
    let mut marker = q.marker.map(|(e, p)| MarkerView {
        entity: e,
        position: glam::Vec3::from_array(p),
    });
    if let Some(st) = stage {
        for o in &st.objectives {
            let count = q.counts.get(&o.id).copied().unwrap_or(0);
            let done = count >= o.target;
            let mut args = args_json(&s.args(lib, None, 0, &o.text.args));
            args["count"] = json!(count);
            args["target"] = json!(o.target);
            if marker.is_none()
                && !done
                && let Some(p) = o.marker
            {
                marker = Some(MarkerView {
                    entity: 0,
                    position: glam::Vec3::from_array(p),
                });
            }
            objectives.push(ObjectiveView {
                id: o.id.clone(),
                text: tr(engine, &o.text.key, &args),
                count,
                target: o.target,
                done,
                failed: q.failed.contains(&o.id),
                optional: o.optional,
                hidden: o.hidden && count == 0 && !done,
            });
        }
    }
    let summary = stage
        .map(|st| st.summary.as_str())
        .filter(|k| !k.is_empty())
        .or(def.map(|d| d.summary.as_str()))
        .unwrap_or("");
    QuestView {
        id: q.id.clone(),
        title: def.map_or(String::new(), |d| t(&d.title)),
        summary: t(summary),
        category: def.map_or("", |d| d.category.as_str()).to_string(),
        stage: q.stage.clone(),
        status: q.status.to_string(),
        tracked: q.tracked,
        is_new: q.is_new,
        updated: q.updated,
        updated_tick: q.updated_tick,
        giver: def.map_or(String::new(), |d| t(&d.giver)),
        region: def.map_or(String::new(), |d| t(&d.region)),
        marker,
        rewards: def.map_or(Vec::new(), |d| {
            d.rewards
                .iter()
                .map(|r| RewardView {
                    name: t(&r.name),
                    amount: r.amount,
                    image: r.image.clone(),
                    rarity: r.rarity.clone(),
                })
                .collect()
        }),
        objectives,
    }
}

pub fn og_story_quest(world: u32, id: String) -> Option<crate::host::wire::QuestView> {
    story(world, |lib, s| {
        s.quest(&id).map(|q| quest_view(world, lib, s, q))
    })
    .ok()
    .flatten()
}

pub fn og_story_quests(world: u32) -> Vec<crate::host::wire::QuestView> {
    story(world, |lib, s| {
        s.quests
            .iter()
            .map(|q| quest_view(world, lib, s, q))
            .collect::<Vec<_>>()
    })
    .unwrap_or_default()
}

pub(super) fn with_quest(world: u32, id: &str, f: impl FnOnce(&mut og_story::QuestState)) -> bool {
    story(world, |_, s| {
        match s.quests.iter_mut().find(|q| q.id == id) {
            Some(q) => {
                f(q);
                true
            }
            None => false,
        }
    })
    .unwrap_or(false)
}

pub fn og_story_track(world: u32, id: String, on: bool) -> bool {
    with_quest(world, &id, |q| q.tracked = on)
}

pub fn og_story_mark_seen(world: u32, id: String) -> bool {
    with_quest(world, &id, |q| {
        q.is_new = false;
        q.updated = false;
    })
}

pub fn og_story_mark(world: u32, id: String, desc: Option<String>) -> bool {
    let Ok(d) = crate::host::wire::arg::<crate::host::wire::MarkerDesc>(desc.as_deref(), "desc")
    else {
        return false;
    };
    let (entity, pos) = (d.entity, d.position.to_array());
    with_quest(world, &id, |q| {
        q.marker = (entity != 0 || pos != [0.0; 3]).then_some((entity, pos));
    })
}
