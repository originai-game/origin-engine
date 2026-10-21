//! Origin Engine AI as data (design §2.7): behaviour trees and state
//! machines declared in `.ogt` (`type behaviour`), perception, per-agent
//! blackboards, and the `og.ai` system that thinks for every `og.AiAgent`
//! and moves it through og-nav. Deterministic: agents think in query order,
//! wander points and sight rays depend only on the world, and memories,
//! blackboards and pending noises are a state block.
//!
//! | Module | What it holds |
//! |---|---|
//! | [`asset`] | [`Behaviour`]: the parsed and validated tree or machine, keys, routes; [`Value`], [`Check`], [`Action`], [`Kind`] |
//! | [`tree`] | ticking: [`tree::think`], [`AgentMem`], [`Status`] |
//! | [`system`] | the records (`og.AiAgent`, `og.Perception`, `og.Perceivable`, `og.AiChanged`), [`Behaviours`], [`install`], [`AiSystem`] (noises, commands, blackboard access, descriptions) |

pub mod asset;
pub mod system;
pub mod tree;

pub use asset::{
    Action, BEHAVIOUR_TYPE, BUILTIN_KEYS, Behaviour, Check, Kind, Op, Root, Speed, Value,
};
pub use system::{
    AiAgent, AiChanged, AiDesc, AiSystem, Behaviours, CommandSink, Description, NEVER, Noise,
    Parent, Perceivable, Perception, SYSTEM_NAME, SightRay, install,
};
pub use tree::{AgentMem, Status};

/// `AiAgent.status` after a think that is still running.
pub const AI_RUNNING: u32 = 1;
/// After a think that succeeded.
pub const AI_SUCCESS: u32 = 2;
/// After a think that failed.
pub const AI_FAILURE: u32 = 3;
