//! The IDL's `story` module in a build without og-runtime's `story`
//! feature: og-story is not linked, and this module stands in for
//! `story.rs`. Every `og.story` call that answers a Result answers
//! [`ABSENT`]; the others answer what the full module answers with no
//! dialogue or quest loaded (null, an empty list, false).
//!
//! The world is the full build's while the game never loads a story: the
//! og.story system is scheduled at the same place with the same
//! declarations and runs nothing (a world with change filters saves every
//! system's name and the change tick, which moves once per system run), and
//! no state block is added (the full module adds its block only once
//! og.Story is registered). og.Story and the story's events stay lazy
//! records in IDL order (crate::engine::lazy); registering them is refused with
//! [`ABSENT`], so a save that holds a story is refused, never restored into
//! no story.

use og_ecs::{SystemDesc, World};
use serde_json::{Value, json};

/// Why the story answers nothing here.
pub(crate) const ABSENT: &str = "og.story is not built into this engine: og-runtime was built without its `story` feature (a default feature; og-host-web and og-host-node pass it through as `story`)";

pub(crate) const STORY: &str = "og.Story";
pub(crate) const SHOWN: &str = "og.DialogueShown";
pub(crate) const ENDED: &str = "og.DialogueEnded";
pub(crate) const CHECK: &str = "og.DialogueCheck";
pub(crate) const QUEST_UPDATED: &str = "og.QuestUpdated";
pub(crate) const STORY_EVENT: &str = "og.StoryEvent";
pub(crate) const BARK_SAID: &str = "og.BarkSaid";
/// og.Story and the events, registered together (as the full module's).
pub(crate) const RECORDS: [&str; 7] = [
    STORY,
    SHOWN,
    ENDED,
    CHECK,
    QUEST_UPDATED,
    STORY_EVENT,
    BARK_SAID,
];
/// The og.story system's name (the full module's).
pub(crate) const SYSTEM: &str = "og.story";

/// og.story on one engine: nothing here.
#[derive(Default)]
pub(crate) struct ModuleState;

/// og.Story was registered: refused, there is no story to attach.
pub(crate) fn attach(_mount: crate::engine::Mount<'_>, _world: &mut World) -> Result<(), String> {
    Err(ABSENT.to_string())
}

/// The og.story system as the full module schedules it (its name and
/// declarations), running nothing.
pub(crate) fn mount(world: &mut World, _mount: crate::engine::Mount<'_>) -> Result<(), String> {
    world
        .add_system(
            SystemDesc::new(SYSTEM).write::<og_audio::AudioSource>(),
            |_| Ok(()),
        )
        .map_err(|e| e.reason().to_string())
}

/// `dump_story`: not loaded, and why.
pub(crate) fn dump(_engine_id: u32) -> Value {
    json!({ "loaded": false, "reason": ABSENT })
}

pub fn og_story_load(_world: u32, _path: String) -> Result<crate::host::wire::StoryAsset, String> {
    Err(ABSENT.into())
}

pub fn og_story_parse(_world: u32, _text: String) -> Result<crate::host::wire::StoryAsset, String> {
    Err(ABSENT.into())
}

pub fn og_story_start(_world: u32, _desc: Option<String>) -> Result<serde_json::Value, String> {
    Err(ABSENT.into())
}

pub fn og_story_view(_world: u32, _conversation: u32) -> Option<crate::host::wire::DialogueView> {
    None
}

pub fn og_story_history(_world: u32, _conversation: u32) -> Vec<crate::host::wire::StoryLine> {
    Vec::new()
}

pub fn og_story_conversations(_world: u32) -> Vec<u32> {
    Vec::new()
}

pub fn og_story_advance(_world: u32, _conversation: u32) -> Result<(), String> {
    Err(ABSENT.into())
}

pub fn og_story_choose(_world: u32, _conversation: u32, _index: u32) -> Result<(), String> {
    Err(ABSENT.into())
}

pub fn og_story_stop(_world: u32, _conversation: u32) -> bool {
    false
}

pub fn og_story_bark(_world: u32, _desc: Option<String>) -> Result<(), String> {
    Err(ABSENT.into())
}

pub fn og_story_ambient(_world: u32, _desc: Option<String>) -> Result<(), String> {
    Err(ABSENT.into())
}

pub fn og_story_barks(_world: u32) -> Vec<crate::host::wire::BarkView> {
    Vec::new()
}

pub fn og_story_set_saliency(_world: u32, _strategy: String) -> Result<(), String> {
    Err(ABSENT.into())
}

pub fn og_story_get(_world: u32, _name: String) -> Option<Value> {
    None
}

pub fn og_story_set(_world: u32, _name: String, _value: String) -> Result<(), String> {
    Err(ABSENT.into())
}

pub fn og_story_reset(_world: u32) {}

pub fn og_story_start_quest(_world: u32, _id: String) -> Result<serde_json::Value, String> {
    Err(ABSENT.into())
}

pub fn og_story_progress(
    _world: u32,
    _quest: String,
    _objective: String,
    _amount: Option<u32>,
) -> bool {
    false
}

pub fn og_story_set_stage(
    _world: u32,
    _quest: String,
    _stage: String,
) -> Result<serde_json::Value, String> {
    Err(ABSENT.into())
}

pub fn og_story_complete_quest(_world: u32, _quest: String) -> Result<serde_json::Value, String> {
    Err(ABSENT.into())
}

pub fn og_story_fail_quest(_world: u32, _quest: String) -> Result<serde_json::Value, String> {
    Err(ABSENT.into())
}

pub fn og_story_quest(_world: u32, _id: String) -> Option<crate::host::wire::QuestView> {
    None
}

pub fn og_story_quests(_world: u32) -> Vec<crate::host::wire::QuestView> {
    Vec::new()
}

pub fn og_story_track(_world: u32, _id: String, _on: bool) -> bool {
    false
}

pub fn og_story_mark_seen(_world: u32, _id: String) -> bool {
    false
}

pub fn og_story_mark(_world: u32, _id: String, _desc: Option<String>) -> bool {
    false
}
