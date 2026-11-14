//! What is read at a place in the world. The wind ([`wind`]) and the
//! ground ([`ground`]: the height field the terrain draws and what grows
//! on it reads), and what every camera-centred map shares: where its
//! window sits in the world ([`window`]: the wind field, the grass
//! trample, the sea's foam, wakes and caustics, the weather's shelter)
//! and which ticks a frame steps it through ([`catch_up`]: the wind
//! field, the trample, the visual bodies and fluids). Two maps keep a
//! schedule of their own because their steps are not the world's ticks:
//! the foam steps the tick's time at 30 Hz and holds on a step back
//! (`ocean_fft::foam::schedule`), and the wakes step the ticks the world
//! recorded cuts for (`water_ripples::plan`).
//!
//! A map of this kind is presentation state: stepped once per tick on the
//! GPU, never read by the tick and never read back. Its window is snapped
//! to whole cells of the absolute world, so its texels stay on fixed
//! world points while the camera and the scene's origin move, and a step
//! that follows a move shifts the map by whole cells.

pub(crate) mod catch_up;
pub(crate) mod ground;
pub(crate) mod wind;
pub mod wind_field;
pub(crate) mod window;

pub use catch_up::MAX_CATCH_UP;
