//! A prefractured prop in a world, without the world: what `og.destruction`
//! keeps of it between ticks ([`state`]), where its chunks stand
//! ([`pose`]), how it breaks in one tick ([`breaking`]), what its loose
//! islands rest on ([`support`]), a blast's push on its pieces ([`blast`]),
//! the fine pieces and grit it sheds ([`debris`]), the budgets' order
//! ([`budget`]) and the material presets ([`materials`]).
//!
//! og-runtime's `og.destruction` system reads the world's rows, calls
//! these and writes the rows back; nothing here reads an entity.

pub mod blast;
pub mod breaking;
pub mod budget;
pub mod debris;
pub mod materials;
pub mod pose;
pub mod state;
pub mod support;
