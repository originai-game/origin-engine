//! Origin Engine motion generation (design §2.7, planned): a pose-conditioned
//! humanoid motion model, a student distilled from NVIDIA MotionBricks.
//!
//! Given each character's four most recent poses and one to four target
//! poses, [`gpu::Gpu::generate`] returns the motion between them: 6 to 16
//! tokens of four frames at 30 Hz, per-frame foot contacts included. Its
//! poses are presentation (§2.4): nothing the simulation reads.
//!
//! | Module | What it holds |
//! |---|---|
//! | [`bundle`] | [`Bundle`]: the weights (one safetensors file) and their [`Config`] |
//! | [`contract`] | world poses to the graph inputs ([`encode`]) and motion frames back ([`decode_frame`]) |
//! | [`glue`] | everything around the two networks, shared by every path |
//! | [`cpu`] | the student and decoder on the CPU, the same bits on every host: the engine's path and the reference |
//! | [`flight`] | a generated jump drawn on the simulation's flight: its local height and its retiming |
//! | [`gpu`] | the same on wgpu compute, the shipping path ([`gpu::Gpu`]) |
//! | [`intent`] | the intent API: [`intent::Goal`]s queued per character, [`intent::Motion`] chaining generations in batches |
//! | [`library`] | the poses goals name ([`library::POSES`]), carried by the bundle |
//! | [`soma`] | the generated skeleton for og-anim: its asset, poses as local transforms, feet planted from contacts, frames as a clip |
//!
//! The specification is og-motion-train's `ogm/reference.py`; the tests
//! check this crate against fixtures it wrote. Redistributing a bundle
//! carries its attribution (NVIDIA Open Model Agreement).

// Index loops mirror the reference's array arithmetic, which they are checked against.
#![allow(clippy::needless_range_loop)]

pub mod bundle;
pub mod contract;
pub mod cpu;
pub mod flight;
pub mod glue;
pub mod gpu;
pub mod intent;
pub mod library;
pub mod soma;

pub use bundle::{Bundle, BundleError, Config, DecoderConfig, StudentConfig, read_safetensors};
pub use contract::{Alignment, Inputs, Pose, Request, decode_frame, encode, valid_frames};
pub use glue::StudentOutput;

/// One batch of generated motion.
#[derive(Clone, Debug, PartialEq)]
pub struct Generation {
    /// Token counts chosen per character, 6..=16.
    pub counts: Vec<u32>,
    /// `[n,64,366]` in each character's aligned frame; the frames of
    /// [`valid_frames`]`(count)` are motion.
    pub motion: Vec<f32>,
    /// The student's root path and tokens, for inspection.
    pub student: StudentOutput,
}

impl Generation {
    /// Character `i`'s frame `f` (366 features).
    pub fn frame(&self, i: usize, f: usize) -> &[f32] {
        let at = (i * contract::FRAMES + f) * contract::MOTION_FEATURES;
        &self.motion[at..at + contract::MOTION_FEATURES]
    }
}

/// The model on the CPU, end to end, every character's generation at once.
pub struct Model {
    statics: glue::Statics,
    cpu: cpu::Cpu,
}

impl Model {
    pub fn new(bundle: &Bundle) -> Result<Model, BundleError> {
        Ok(Model {
            statics: glue::Statics::new(bundle)?,
            cpu: cpu::Cpu::new(bundle)?,
        })
    }

    /// Generates motion for already encoded inputs.
    pub fn generate(&self, inputs: &Inputs) -> Generation {
        let (student, local) = self.cpu.generate(&self.statics, inputs);
        let motion = glue::finish(&self.statics, inputs, &student, &local);
        Generation {
            counts: student.counts.clone(),
            motion,
            student,
        }
    }

    /// Encodes the requests, generates, and returns each character's alignment
    /// for [`decode_frame`].
    pub fn generate_requests(&self, requests: &[Request]) -> (Generation, Vec<Alignment>) {
        let (inputs, alignments) = encode(requests);
        (self.generate(&inputs), alignments)
    }
}
