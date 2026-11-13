//! The reflected types physics reads and writes, one file per mechanism
//! beside the code that steps them. Field order, names and storage types
//! are the layout the IDL declares, enumerations are `u32` codes with
//! `pub const`s, and the doc comments are the user-facing documentation.
//! Fields a step writes are marked "written".

#[path = "character/schema.rs"]
mod character;
#[path = "fluid/schema.rs"]
mod fluid;
#[path = "rigid/schema_joints.rs"]
mod joints;
#[path = "rigid/schema.rs"]
mod rigid;
#[path = "soft/schema.rs"]
mod soft;
#[path = "vehicle/schema.rs"]
mod vehicle;
#[path = "water/schema.rs"]
mod water;

pub use character::*;
pub use fluid::*;
pub use joints::*;
pub use rigid::*;
pub use soft::*;
pub use vehicle::*;
pub use water::*;

/// `Cloth.simulation`, `SoftBody.simulation`, `Garment.simulation`: in the
/// physics step on the CPU, world state (the default).
pub const SIMULATION_GAMEPLAY: u32 = 0;
/// `Cloth.simulation`, `SoftBody.simulation`, `Garment.simulation`: by the
/// renderer on the GPU, for looks only; the tick carries its rest shape
/// rigidly with the entity (a garment's: the skinned pose).
pub const SIMULATION_VISUAL: u32 = 1;
