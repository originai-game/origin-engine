//! One binding per domain: the IDL module of each domain crate over the
//! runtime's engines, and the stub that answers the refusal in a build
//! that leaves the crate out ([`absent`](self) holds one per feature).
//! Domain crates do not depend on each other; where two meet (AI hearing
//! audio, cloth posed by a skin, the sea's floor from a terrain) the wiring
//! is here.

#[cfg(feature = "ai")]
pub(crate) mod ai;
// Without the `ai` feature og-ai is not linked: og.ai answers the refusal
// (bind/absent/ai.rs).
#[cfg(not(feature = "ai"))]
#[path = "bind/absent/ai.rs"]
pub(crate) mod ai;
pub(crate) mod anim;
#[cfg(feature = "destruction")]
pub(crate) mod destruction;
#[cfg(feature = "destruction")]
pub(crate) mod destruction2d;
// Without the `destruction` feature og-physics' destruction is not built: both halves
// of og.destruction answer the refusal (bind/absent/destruction*.rs).
pub(crate) mod agents;
pub(crate) mod agents_idle;
#[cfg(not(feature = "destruction"))]
#[path = "bind/absent/destruction.rs"]
pub(crate) mod destruction;
#[cfg(not(feature = "destruction"))]
#[path = "bind/absent/destruction2d.rs"]
pub(crate) mod destruction2d;
pub(crate) mod forces;
pub(crate) mod garment;
pub(crate) mod i18n;
pub(crate) mod modules;
#[cfg(feature = "motion")]
pub(crate) mod motion;
// Without the `motion` feature og-anim's motion model is not linked: og.motion answers
// the refusal (bind/absent/motion.rs).
#[cfg(not(feature = "motion"))]
#[path = "bind/absent/motion.rs"]
pub(crate) mod motion;
#[cfg(feature = "nav")]
pub(crate) mod nav;
// Without the `nav` feature og-ai is not linked: og.nav answers the
// refusal (bind/absent/nav.rs).
#[cfg(not(feature = "nav"))]
#[path = "bind/absent/nav.rs"]
pub(crate) mod nav;
pub(crate) mod net;
pub(crate) mod particles;
pub(crate) mod physics_ext;
pub(crate) mod procgen;
pub(crate) mod seabed;
pub(crate) mod sight;
pub(crate) mod skeleton2d;
#[cfg(feature = "story")]
pub(crate) mod story;
// Without the `story` feature og-story is not linked: og.story answers the
// refusal (bind/absent/story.rs).
#[cfg(not(feature = "story"))]
#[path = "bind/absent/story.rs"]
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
#[cfg(feature = "video")]
pub(crate) mod video;
// Without the `video` feature og-video is not linked: og.video answers the
// refusal (bind/absent/video.rs).
pub(crate) mod vfx;
#[cfg(not(feature = "video"))]
#[path = "bind/absent/video.rs"]
pub(crate) mod video;
pub(crate) mod weather;
pub(crate) mod wind;
