//! Origin Engine AI as data (design §2.7): behaviour trees and state
//! machines declared in `.ogt` (`type behaviour`), perception, per-agent
//! blackboards, and the `og.ai` system that thinks for every `og.AiAgent`
//! and moves it through [`nav`]. Deterministic: agents think in query order,
//! wander points and sight rays depend only on the world, and memories,
//! blackboards and pending noises are a state block.
//!
//! | Module | What it holds |
//! |---|---|
//! | [`sense`] | what an agent sees and hears: `og.Perception`, `og.Perceivable`, and the host's seams ([`SightRay`], [`Hearing`]) |
//! | [`decide::asset`] | [`Behaviour`]: the parsed and validated tree or machine, keys, routes; [`Value`], [`Check`], [`Action`], [`Kind`] |
//! | [`decide::tree`] | ticking: [`tree::think`], [`AgentMem`], [`Status`] |
//! | [`decide::agent`] | the agent's records: `og.AiAgent`, `og.AiChanged` and the action events |
//! | [`nav`], [`steer`] | where it can walk and how it avoids the others |
//! | [`system`] | [`Behaviours`], [`install`], [`AiSystem`] (noises, commands, blackboard access, descriptions) and the `og.ai` state block |

pub mod decide;
pub mod nav;
pub mod sense;
pub mod steer;
pub mod system;

pub use decide::agent::{
    AiActionStarted, AiActionStopped, AiAgent, AiChanged, enable_action_started,
    enable_action_stopped, enable_actions, has_actions,
};
pub use decide::asset::{
    Action, BEHAVIOUR_TYPE, BUILTIN_KEYS, Behaviour, Check, Consideration, Curve, Kind, Op,
    Resolve, Root, Speed, Value,
};
pub use decide::tree::{AI_INTERRUPTED, AgentMem, Status};
pub use decide::{asset, tree};
pub use sense::{HeardSounds, Hearing, NEVER, Noise, Parent, Perceivable, Perception, SightRay};
pub use system::{AiDesc, AiSystem, Behaviours, CommandSink, Description, SYSTEM_NAME, install};

/// `AiAgent.status` after a think that is still running.
pub const AI_RUNNING: u32 = 1;
/// After a think that succeeded.
pub const AI_SUCCESS: u32 = 2;
/// After a think that failed.
pub const AI_FAILURE: u32 = 3;
