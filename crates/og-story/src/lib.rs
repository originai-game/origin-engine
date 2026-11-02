//! Origin Engine dialogue and quests as data (G2-8): dialogue graphs
//! (`type dialogue`) and quests (`type quest`) declared in `.ogt`, their
//! expressions, and the story a world keeps. Pure logic over og-core: the
//! runtime's og.story module mounts it on a world, saves it as a state
//! block and sends its events. Deterministic: a story draws only from its
//! own og.Rng stream (og.Rng's fork "story"), counts timers in ticks and
//! does arithmetic in IEEE `f64` alone.
//!
//! | Module | What it holds |
//! |---|---|
//! | [`expr`] | [`Expr`], [`Val`]: conditions and values |
//! | [`asset`] | [`Dialogue`], [`Quest`]: parsing and validation |
//! | [`story`] | [`Story`], [`Library`]: conversations, quests, variables |
//! | [`save`] | [`Story::save`], [`Story::load`] |

pub mod asset;
pub mod expr;
pub mod save;
pub mod story;

pub use asset::{
    Check, Choice, DIALOGUE_TYPE, Dialogue, Line, NARRATOR, Node, Objective, QUEST_TYPE, Quest,
    QuestOp, Reward, Speaker, Stage, Step, Text, VarDecl,
};
pub use expr::{Expr, Val};
pub use story::{Conv, Event, Library, QuestState, Roll, Shown, Story, Wait};
