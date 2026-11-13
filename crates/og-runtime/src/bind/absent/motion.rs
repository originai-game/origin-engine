//! The IDL's `motion` module in a build without og-runtime's `motion`
//! feature: og-anim's `motion` (the motion model and its pose library) is not
//! linked, and this module stands in for `motion.rs` and `motion/`. Every
//! `og.motion` call that answers a Result answers [`ABSENT`];
//! `capabilities` answers no backend with it, `poses` no pose.
//!
//! The world is the full build's while the game never names og.MotionAgent:
//! the full module adds its system and state block only then (crate::engine::lazy),
//! so a world without one has neither in either build. og.MotionAgent stays
//! a lazy record in IDL order; naming it is refused with [`ABSENT`], so a
//! save holding agents is refused, never restored into agents that stand.
//!
//! The presentation is the full build's too: an idle Presenter (no agent,
//! no model) still marks each tick it steps with a renderer, and that tick
//! is part of the presentation hash and checkpoint
//! (`crate::present::presentation`). [`present::Presenter`] keeps exactly that, so a
//! presentation hash taken without the module equals one taken with it
//! for a game that never uses og.motion.

use crate::engine::Mount;
use og_ecs::World;
use serde_json::{Value, json};

/// Why og.motion answers nothing here.
pub(crate) const ABSENT: &str = "og.motion is not built into this engine: og-runtime was built without its `motion` feature (a default feature; og-host-web and og-host-node pass it through as `motion`)";

pub(crate) const AGENT: &str = "og.MotionAgent";

/// og.MotionAgent was named: refused.
pub(crate) fn enable(_world: &mut World, _mount: Mount<'_>) -> og_core::Result<()> {
    Err(og_core::Error::new(og_core::ErrorKind::Invalid, ABSENT))
}

/// og.motion on one engine: nothing here.
#[derive(Clone, Default)]
pub(crate) struct ModuleState;

/// The probe commands the full module registers, answering [`ABSENT`].
pub(crate) fn register_commands(probe: &mut og_probe::Probe, _engine: u32) {
    let _ = probe.register_command(
        "motion_backend",
        og_probe::CommandDesc {
            usage: "motion_backend:<cpu|gpu>".into(),
            doc: format!("og.motion.setBackend from the probe: {ABSENT}"),
        },
        |_world, _args| Err(ABSENT.to_string()),
    );
    let _ = probe.register_dump(
        "dump_motion",
        og_probe::CommandDesc {
            usage: "dump_motion[:path]".into(),
            doc: format!("og.motion: {ABSENT}"),
        },
        0,
        |_world, _args| Ok(json!({ "error": ABSENT })),
    );
}

/// og.motion's presentation without the model: what an idle full build's
/// Presenter keeps and hashes.
pub(crate) mod present {
    use crate::engine::Engine;
    use glam::Mat4;

    #[derive(Default)]
    pub(crate) struct Presenter {
        /// The last tick stepped (the full Presenter's `stepped`).
        stepped: Option<u64>,
    }

    /// og.motion's part of a presentation checkpoint: nothing a restore
    /// reads (the full Presenter without a model forgets on restore).
    #[derive(Clone, Default)]
    pub(crate) struct MotionCheckpoint;

    impl Presenter {
        pub(crate) fn forget(&mut self) {
            self.stepped = None;
        }

        /// No agent is drawn by the model here.
        pub(crate) fn drawn(&self, _entity: u64, _alpha: Option<f32>) -> Option<Vec<Mat4>> {
            None
        }

        pub(crate) fn checkpoint(&self) -> MotionCheckpoint {
            MotionCheckpoint
        }

        /// The full Presenter without a loaded model forgets on restore.
        pub(crate) fn restore(&mut self, _c: &MotionCheckpoint) {
            self.forget();
        }

        /// The full Presenter's hash with no agent and no model: FNV-1a
        /// of `{clock:?}|{stepped:?}|{held:?}|`, the clock and the held
        /// step always none while no agent moves.
        pub(crate) fn hash(&self) -> u64 {
            let text = format!("{:?}|{:?}|{:?}|", None::<f64>, self.stepped, None::<()>);
            og_core::hash::hash_bytes(text.as_bytes())
        }
    }

    /// No model is declared here (no compute slice to time).
    pub(crate) fn declares(_eng: &Engine) -> bool {
        false
    }

    /// The full module's step with no agent: with a renderer, the world's
    /// tick is the one stepped.
    pub(crate) fn step(eng: &Engine) -> bool {
        let Ok(tick) = eng.world().map(|w| w.time().tick) else {
            return true;
        };
        let mut rs = eng.render.borrow_mut();
        if rs.backend.is_some() {
            rs.motion.stepped = Some(tick);
        }
        true
    }

    pub(crate) fn settle(eng: &Engine) -> bool {
        step(eng)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// The text the full Presenter hashes when idle, stepped at tick 7
        /// (`motion/present.rs` `Presenter::hash`: clock, stepped and held,
        /// each `{:?}` and `|`-terminated, then nothing per agent and no
        /// model): the stand-in's hash is FNV-1a of exactly that.
        #[test]
        fn idle_hash_is_the_full_presenters() {
            let fnv = |s: &str| {
                s.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| {
                    (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
                })
            };
            let mut p = Presenter::default();
            assert_eq!(p.hash(), fnv("None|None|None|"));
            p.stepped = Some(7);
            assert_eq!(p.hash(), fnv("None|Some(7)|None|"));
            let c = p.checkpoint();
            p.restore(&c);
            assert_eq!(p.hash(), fnv("None|None|None|"));
        }
    }
}

pub fn og_motion_pose(_world: u32, _entity: f64, _desc: Option<String>) -> Result<(), String> {
    Err(ABSENT.into())
}

pub fn og_motion_place(_world: u32, _entity: f64, _desc: Option<String>) -> Result<(), String> {
    Err(ABSENT.into())
}

pub fn og_motion_face(_world: u32, _entity: f64, _desc: Option<String>) -> Result<(), String> {
    Err(ABSENT.into())
}

pub fn og_motion_walk_to(_world: u32, _entity: f64, _desc: Option<String>) -> Result<(), String> {
    Err(ABSENT.into())
}

pub fn og_motion_jump(_world: u32, _entity: f64, _desc: Option<String>) -> Result<(), String> {
    Err(ABSENT.into())
}

pub fn og_motion_clear(_world: u32, _entity: f64) -> Result<(), String> {
    Err(ABSENT.into())
}

pub fn og_motion_status(
    _world: u32,
    _entity: f64,
) -> Result<crate::host::wire::MotionStatus, String> {
    Err(ABSENT.into())
}

pub fn og_motion_presentation(_world: u32, _entity: f64) -> Result<serde_json::Value, String> {
    Err(ABSENT.into())
}

pub fn og_motion_look_at(_world: u32, _entity: f64, _desc: Option<String>) -> Result<(), String> {
    Err(ABSENT.into())
}

pub fn og_motion_react(_world: u32, _entity: f64, _desc: Option<String>) -> Result<(), String> {
    Err(ABSENT.into())
}

/// No backend, and why (the full module's shape).
pub fn og_motion_capabilities(_world: u32) -> Value {
    json!({ "backend": "none", "reason": ABSENT, "modelBytes": 0 })
}

pub fn og_motion_set_backend(_world: u32, _backend: String) -> Result<serde_json::Value, String> {
    Err(ABSENT.into())
}

/// No pose library is built in.
pub fn og_motion_poses() -> Value {
    Value::Array(Vec::new())
}
