//! The weather a frame draws: its look in the air (`look`: particles, fog
//! and dimmed light), what it leaves on the lit surfaces (`surface`), its
//! lightning (`lightning`) and where a roof keeps it off (`shelter`). The
//! runtime places the particles over the camera each frame; they are
//! simulated by the renderer from the tick like every particle system.

mod lightning;
mod look;
mod shelter;
mod surface;

pub use lightning::*;
pub use look::*;
pub(crate) use shelter::*;
pub use surface::*;
