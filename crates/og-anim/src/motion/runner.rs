//! The motion model resident ([`Ready`]): where it runs ([`Compute`]), the
//! batches it has in flight and landed ([`flights`]), and the generated
//! frames queued per character. Nothing here reads a world: a caller starts
//! batches at its ticks and takes their frames at their due ticks, so what a
//! tick shows never depends on the wall clock, which only the diagnostics
//! read ([`Ready`]'s `clock`).

mod cpu_ticks;
mod flights;

pub use cpu_ticks::*;
pub use flights::*;

use crate::motion::Bundle;
use crate::motion::contract::{BOUNDARY_FRAMES, MAX_TOKENS, MIN_TOKENS, POSES_PER_TOKEN};
use crate::motion::cpu::Cpu;
use crate::motion::glue::Statics;
use crate::motion::gpu::Gpu;
use crate::motion::intent::{Batch, Motion};
use crate::motion::library::Library;
use crate::motion::soma::Skeleton;
use crate::{HumanoidMap, SkeletonAsset};
use std::sync::Arc;

/// The GPU time a batch may add to one frame, ms: a batch runs in slices of
/// about this much, one a frame (`Gpu::set_frame_budget`), so a frame of
/// 16.7 ms keeps room for the scene.
pub const FRAME_BUDGET_MS: f64 = 3.0;

/// How long after it starts a batch's frames are taken, s: at that tick,
/// waiting for it natively if it is still running, and never sooner unless
/// an agent has nothing left to show. A fixed number of ticks, not the
/// GPU's speed, so a machine's load never changes what is drawn (the
/// sliced batch of 16 agents takes 0.5 to 0.6 s).
pub const BATCH_SECONDS: f32 = 0.5;

/// The model's frame rate.
pub const RATE: f32 = 30.0;

/// Tokens either side of a model goal's own duration it may take
/// ([`token_counts`]).
pub const TOKEN_SLACK: usize = 2;

/// SOMA's ankles, the joints a jump's local height measures to the ground.
pub const LEFT_ANKLE: usize = 24;

pub const RIGHT_ANKLE: usize = 28;

/// Where the model runs.
pub enum Compute {
    /// The opt-in: the same bits on every host.
    Cpu(Arc<Cpu>),
    /// The default: the same bits on one host and GPU.
    Gpu {
        /// None while a batch has it.
        gpu: Option<Box<Gpu>>,
        /// Polled to wait for a batch (native only).
        #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
        device: wgpu::Device,
    },
}

pub struct Ready {
    pub compute: Compute,
    /// The batches being computed, in the order they started: at most one
    /// on the GPU, all of every character asking at once; on the CPU one a
    /// character, started the tick it asks, each with its own due tick.
    pub flights: Vec<Flight>,
    /// Batches computed and held until their tick, in the order they landed.
    pub landings: Vec<Landing>,
    /// A restored checkpoint's batches with their ticks: started again (on
    /// the GPU once it is free) in place of new ones.
    pub queued: Vec<(Batch, u64)>,
    pub statics: Arc<Statics>,
    pub motion: Motion,
    pub library: Library,
    pub soma: SkeletonAsset,
    pub soma_skeleton: Skeleton,
    pub soma_map: HumanoidMap,
    /// SOMA's ankle height standing on the ground (the library idle's
    /// lower ankle), the clearance of a jump's local height.
    pub clearance: f32,
    /// The GPU's rate as the last batch measured it, GFLOP/s (0 on the CPU).
    pub rate: f64,
    /// The CPU model's wall time on each tick it ran (a diagnostic).
    pub cpu_ticks: CpuTicks,
    /// The characters' generations decoded, in all.
    pub generations: u64,
    pub failure: Option<String>,
    pub startup: Startup,
    /// The host's wall clock, ms: the diagnostics' only, never the world's.
    pub clock: fn() -> f64,
}

/// Wall-time diagnostics only; none of these clocks enter the world.
#[derive(Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Startup {
    pub compile_ms: Option<f64>,
    pub upload_submit_ms: Option<f64>,
    pub first_dispatch_delay_ms: Option<f64>,
    pub first_inference_wall_ms: Option<f64>,
    pub first_inference_poll_ms: f64,
    pub first_palette_delay_ms: Option<f64>,
    #[serde(skip)]
    pub resident_at: f64,
    #[serde(skip)]
    pub inference_finished_at: Option<f64>,
}

/// What [`build`] makes off the engine thread: everything of [`Ready`] but
/// the batch state.
pub struct Parts {
    pub compute: Compute,
    pub statics: Statics,
    pub library: Library,
    pub soma: SkeletonAsset,
    pub soma_skeleton: Skeleton,
    pub soma_map: HumanoidMap,
}

/// The model from its weights (one safetensors file): on the CPU, or with
/// `gpu` on that device.
pub fn parts(weights: &[u8], gpu: Option<(wgpu::Device, wgpu::Queue)>) -> Result<Parts, String> {
    let bundle = Bundle::from_safetensors(weights).map_err(|e| e.to_string())?;
    let library = Library::from_bundle(&bundle).map_err(|e| e.to_string())?;
    let soma_skeleton = Skeleton::from_bundle(&bundle).map_err(|e| e.to_string())?;
    let soma = soma_skeleton.asset();
    let soma_map = HumanoidMap::detect(&soma);
    let statics = Statics::new(&bundle).map_err(|e| e.to_string())?;
    let compute = match gpu {
        Some((device, queue)) => {
            let mut gpu = Gpu::new(device.clone(), queue, &bundle).map_err(|e| e.to_string())?;
            gpu.set_blocking(false);
            gpu.set_frame_budget(Some(FRAME_BUDGET_MS));
            Compute::Gpu {
                gpu: Some(Box::new(gpu)),
                device,
            }
        }
        None => Compute::Cpu(Arc::new(Cpu::new(&bundle).map_err(|e| e.to_string())?)),
    };
    Ok(Parts {
        compute,
        statics,
        library,
        soma,
        soma_skeleton,
        soma_map,
    })
}

/// The model resident, no batch started: `now_ms` is the host's wall clock
/// (the diagnostics' only).
pub fn resident(parts: Parts, now_ms: fn() -> f64) -> Ready {
    let gpu = match &parts.compute {
        Compute::Gpu { gpu: Some(g), .. } => Some(g),
        _ => None,
    };
    let startup = Startup {
        #[cfg(not(target_arch = "wasm32"))]
        compile_ms: gpu.map(|g| g.startup_ms.0),
        #[cfg(not(target_arch = "wasm32"))]
        upload_submit_ms: gpu.map(|g| g.startup_ms.1),
        resident_at: now_ms(),
        ..Default::default()
    };
    Ready {
        clock: now_ms,
        rate: gpu.map_or(0.0, |g| g.rate()),
        cpu_ticks: CpuTicks::default(),
        generations: 0,
        compute: parts.compute,
        flights: Vec::new(),
        landings: Vec::new(),
        queued: Vec::new(),
        failure: None,
        startup,
        statics: Arc::new(parts.statics),
        motion: Motion::default(),
        clearance: parts.library.frames(0).first().map_or(0.0, |p| {
            p.positions[LEFT_ANKLE][1].min(p.positions[RIGHT_ANKLE][1])
        }),
        library: parts.library,
        soma: parts.soma,
        soma_skeleton: parts.soma_skeleton,
        soma_map: parts.soma_map,
    }
}

impl Ready {
    /// Whether a starved character's frames may come from a batch
    /// outstanding: on the GPU any (its batch may be the next), on the CPU
    /// its own.
    pub fn awaits(&self, character: usize) -> bool {
        let mut batches = self
            .flights
            .iter()
            .map(|f| &f.batch)
            .chain(self.landings.iter().map(|l| &l.batch))
            .chain(self.queued.iter().map(|q| &q.0));
        match self.compute {
            Compute::Gpu { .. } => batches.next().is_some(),
            Compute::Cpu(_) => batches.any(|b| b.characters.contains(&character)),
        }
    }

    /// Where it runs, as og.motion.capabilities names it.
    pub fn backend(&self) -> &'static str {
        match self.compute {
            Compute::Cpu(_) => "cpu",
            Compute::Gpu { .. } => "gpu",
        }
    }

    /// Whether a batch can start: the CPU always can, the GPU once the last
    /// batch gave it back.
    pub fn free(&self) -> bool {
        match &self.compute {
            Compute::Cpu(_) => true,
            Compute::Gpu { gpu, .. } => gpu.is_some(),
        }
    }
}

/// The token counts a model goal `seconds` long may take: about as many
/// as make its motion frames last that long (the model reaches the goal's
/// pose at its last frame, so the drawn motion keeps the simulation's
/// time), with a little room either side for the model's own timing.
pub fn token_counts(seconds: f32) -> Vec<usize> {
    let frames = seconds * RATE + (2 * BOUNDARY_FRAMES + 1) as f32;
    let n = (frames / POSES_PER_TOKEN as f32).round() as usize;
    let lo = n.saturating_sub(TOKEN_SLACK).clamp(MIN_TOKENS, MAX_TOKENS);
    (lo..=(n + TOKEN_SLACK).clamp(MIN_TOKENS, MAX_TOKENS)).collect()
}
