//! The agent's records: `og.AiAgent`, `og.AiChanged` and the game
//! actions' events.

use crate::decide::asset::{Action, Behaviour, Kind};
use crate::system::SYSTEM_NAME;
use og_core::reflect::Str256;
use og_core::{Reflect, Result};
use og_ecs::World;

/// An agent that thinks with a behaviour (`og.ai.load`).
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.AiAgent")]
pub struct AiAgent {
    /// The behaviour it runs (og.ai.load); 0 runs nothing.
    pub behaviour: u64,
    /// Thinks while true.
    #[reflect(default = 1)]
    pub active: bool,
    /// Thinks every this many ticks (1: every tick).
    #[reflect(name = "thinkEvery", min = 1, max = 600, default = 1, unit = "ticks")]
    pub think_every: u32,
    /// AI_RUNNING, AI_SUCCESS or AI_FAILURE after the last think (0 before the first; written by og.ai).
    pub status: u32,
    /// The state machine's state, an index into its states (written by og.ai).
    pub state: u32,
    /// The node it ran last, an index into the behaviour's nodes (written by og.ai).
    pub node: u32,
    /// Times the state or the running node changed (written by og.ai).
    pub changes: u32,
}

impl Default for AiAgent {
    fn default() -> Self {
        AiAgent {
            behaviour: 0,
            active: true,
            think_every: 1,
            status: 0,
            state: 0,
            node: 0,
            changes: 0,
        }
    }
}

/// Sent when an agent's state or running node changes (read the tick after).
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.AiChanged")]
pub struct AiChanged {
    /// The agent.
    pub entity: u64,
    /// Its state now.
    pub state: u32,
    /// The node it runs now.
    pub node: u32,
    /// AI_RUNNING, AI_SUCCESS or AI_FAILURE.
    pub status: u32,
}

/// A game action (`action <name> args` in a behaviour) started: the game
/// plays it (a reload, a door, a dialogue) and calls og.ai.finish when it
/// is over. Registered only once a behaviour with one is loaded
/// ([`enable_actions`]).
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.AiActionStarted")]
pub struct AiActionStarted {
    /// The agent.
    pub entity: u64,
    /// The action's node (BehaviourInfo.nodes).
    pub node: u32,
    /// The action's name.
    pub name: Str256,
    /// Its arguments, as written, separated by spaces.
    pub args: Str256,
}

/// A game action stopped: the game finished it, or something else won
/// (another branch, a state change, a restart).
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.AiActionStopped")]
pub struct AiActionStopped {
    /// The agent.
    pub entity: u64,
    /// The action's node.
    pub node: u32,
    /// The action's name.
    pub name: Str256,
    /// AI_SUCCESS or AI_FAILURE (what og.ai.finish said), or AI_INTERRUPTED.
    pub result: u32,
}

/// Registers og.AiActionStarted and og.AiActionStopped and grants og.ai
/// their writes: what a host calls when it first loads a behaviour with a
/// game action.
pub fn enable_actions(world: &mut World) -> Result<()> {
    enable_action_started(world)?;
    enable_action_stopped(world)
}

/// [`enable_actions`] for og.AiActionStarted alone (a host's lazy record).
pub fn enable_action_started(world: &mut World) -> Result<()> {
    world.register::<AiActionStarted>()?;
    world.grant(SYSTEM_NAME, &[AiActionStarted::NAME])
}

/// [`enable_actions`] for og.AiActionStopped alone.
pub fn enable_action_stopped(world: &mut World) -> Result<()> {
    world.register::<AiActionStopped>()?;
    world.grant(SYSTEM_NAME, &[AiActionStopped::NAME])
}

/// Whether a behaviour has a game action (needs [`enable_actions`]).
pub fn has_actions(b: &Behaviour) -> bool {
    b.nodes
        .iter()
        .any(|n| matches!(n.kind, Kind::Action(Action::Game { .. })))
}
