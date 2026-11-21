//! One binding per domain: the IDL module of each domain crate over the
//! runtime's engines. The 2D profile's terrain is the one stand-in
//! (bind/absent/terrain.rs).
//! Domain crates do not depend on each other; where two meet (AI hearing
//! audio, cloth posed by a skin, the sea's floor from a terrain) the wiring
//! is here.

pub(crate) mod agents;
pub(crate) mod ai;
pub(crate) mod anim;
pub(crate) mod destruction;
pub(crate) mod destruction2d;
pub(crate) mod flow;
pub(crate) mod forces;
pub(crate) mod garment;
pub(crate) mod i18n;
pub(crate) mod modules;
pub(crate) mod motion;
pub(crate) mod nav;
pub(crate) mod net;
pub(crate) mod particles;
pub(crate) mod physics_ext;
pub(crate) mod procgen;
pub(crate) mod seabed;
pub(crate) mod season;
pub(crate) mod shared;
pub(crate) mod sight;
pub(crate) mod skeleton2d;
pub(crate) mod story;
#[cfg(not(feature = "profile2d"))]
pub(crate) mod terrain;
// The 2D profile leaves the 3D world out: its terrain answers the refusal.
#[cfg(feature = "profile2d")]
#[path = "bind/absent/terrain.rs"]
pub(crate) mod terrain;
#[cfg(not(feature = "profile2d"))]
pub(crate) mod terrain_stream;
pub(crate) mod timeline;
pub(crate) mod vfx;
pub(crate) mod video;
pub(crate) mod weather;
pub(crate) mod wind;
