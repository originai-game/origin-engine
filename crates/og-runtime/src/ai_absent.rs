//! The IDL's `ai` module in a build without og-runtime's `ai` feature:
//! og-ai is not linked, and this module stands in for `ai.rs`. Every
//! `og.ai` call that answers a Result answers [`ABSENT`]; the others answer
//! what the full module answers with no behaviour loaded (null, false).
//!
//! The world is the full build's while the game never uses og.ai: the
//! og.ai system and its state block are mounted idle (super::e6_idle), and
//! og.AiActionStarted and og.AiActionStopped stay lazy records whose
//! registration is refused with [`ABSENT`].

use og_ecs::{Entity, World};
use serde_json::Value;

/// Why AI answers nothing here.
pub(crate) const ABSENT: &str = "og.ai is not built into this engine: og-runtime was built without its `ai` feature (a default feature; og-host-web and og-host-node pass it through as `ai`)";

/// `dump_ai`: no agent thinks here.
pub(crate) fn describe_json(_engine: u32, _world: &World, _e: Entity) -> Option<Value> {
    None
}

pub fn og_ai_load(_world: u32, _path: String) -> Result<crate::wire::BehaviourInfo, String> {
    Err(ABSENT.into())
}

pub fn og_ai_parse(_world: u32, _text: String) -> Result<crate::wire::BehaviourInfo, String> {
    Err(ABSENT.into())
}

pub fn og_ai_get(_world: u32, _entity: f64, _key: String) -> String {
    Value::Null.to_string()
}

pub fn og_ai_set(_world: u32, _entity: f64, _key: String, _value: String) -> Result<(), String> {
    Err(ABSENT.into())
}

pub fn og_ai_restart(_world: u32, _entity: f64) -> Result<(), String> {
    Err(ABSENT.into())
}

pub fn og_ai_finish(_world: u32, _entity: f64, _success: bool, _action: Option<String>) -> bool {
    false
}

pub fn og_ai_noise(_world: u32, _desc: Option<String>) -> Result<(), String> {
    Err(ABSENT.into())
}

pub fn og_ai_describe(_world: u32, _entity: f64) -> Option<crate::wire::AiDescription> {
    None
}

fn refused() -> og_core::Error {
    og_core::Error::new(og_core::ErrorKind::Invalid, ABSENT)
}

/// og.AiActionStarted and og.AiActionStopped: refused, there is no og.ai.
pub(crate) fn enable_action_started(_w: &mut World) -> og_core::Result<()> {
    Err(refused())
}

pub(crate) fn enable_action_stopped(_w: &mut World) -> og_core::Result<()> {
    Err(refused())
}
