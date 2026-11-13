//! The batches of a resident model: started, run a step a tick, landed and
//! decoded into their characters' frames at their due ticks.

use super::{Compute, Ready};
use crate::motion::contract::{FRAMES, MOTION_FEATURES};
use crate::motion::cpu::{Job, gather};
use crate::motion::glue::{StudentOutput, decoder_inputs, finish_character, student_inputs};
use crate::motion::gpu::{Gpu, GpuError};
use crate::motion::intent::Batch;
use std::collections::{BTreeSet, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

pub type Running =
    Pin<Box<dyn Future<Output = (Box<Gpu>, Result<(StudentOutput, Vec<f32>), GpuError>)>>>;

/// A batch's student output and decoder features.
pub type Generated = (StudentOutput, Vec<f32>);

/// How a batch is computed.
pub enum Work {
    /// On the GPU (the default): the future gives the GPU back with the result.
    Gpu(Running),
    /// On the CPU: each character's job, a share of its units a tick
    /// across the lanes, the rest when its tick comes ([`run_flights`]).
    Cpu(Vec<Job>),
}

/// A batch being computed.
pub struct Flight {
    pub work: Work,
    pub batch: Batch,
    pub started: f64,
    /// The tick its frames are taken at ([`BATCH_SECONDS`] after its start).
    pub due: u64,
    /// Started before a restore: its result is dropped when it lands.
    pub stale: bool,
}

/// A batch read back, held until its tick, then decoded into its agents'
/// frames.
pub struct Landing {
    pub batch: Batch,
    pub student: StudentOutput,
    pub local: Vec<f32>,
    /// The next character to decode.
    pub next: usize,
    /// One character's motion, reused.
    pub motion: Vec<f32>,
    pub started: f64,
    pub due: u64,
}

/// The batches a checkpoint keeps: in flight, landed and held for their
/// tick, or queued after a restore; by due tick, then by character.
pub fn outstanding(r: &Ready) -> Vec<(Batch, u64)> {
    let mut all: Vec<(Batch, u64)> = r
        .flights
        .iter()
        .filter(|f| !f.stale)
        .map(|f| (f.batch.clone(), f.due))
        .chain(r.landings.iter().map(|l| (l.batch.clone(), l.due)))
        .chain(r.queued.iter().cloned())
        .collect();
    all.sort_by(|a, b| (a.1, &a.0.characters).cmp(&(b.1, &b.0.characters)));
    all
}

/// Whether a poll's future asked to be polled again (a slice's turn ended).
#[derive(Default)]
pub struct Woken(std::sync::atomic::AtomicBool);

impl std::task::Wake for Woken {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.0.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

/// Runs the batches in flight a step at tick `tick`; those `wait` picks
/// are run to their end. On the GPU each poll submits its next slice or
/// polls its readback, and a wait runs it to its readback and waits for
/// that (native only: the web answers false until it lands). On the CPU
/// each job runs its share of the units left (spread evenly over the ticks
/// to its due tick), every flight's across the lanes at once, and a wait
/// runs all its units: a CPU batch is never waited for past its tick, on
/// any host. Answers whether every flight `wait` picks has landed.
pub fn run_flights(
    r: &mut Ready,
    tick: u64,
    wait: impl Fn(&Flight) -> bool,
    may_wait: bool,
    errors: &mut Vec<String>,
) -> bool {
    if r.flights.is_empty() {
        return true;
    }
    let now_ms = r.clock;
    let mut done: Vec<(usize, Result<Generated, String>)> = Vec::new();
    match &mut r.compute {
        Compute::Cpu(cpu) => {
            let start = now_ms();
            let cpu = &**cpu;
            let statics = &*r.statics;
            let mut work: Vec<(&mut Job, usize)> = Vec::new();
            for f in &mut r.flights {
                let ticks = f.due.saturating_sub(tick).max(1) as usize;
                let all = wait(f);
                let Work::Cpu(jobs) = &mut f.work else {
                    unreachable!("a CPU model's flights are CPU jobs")
                };
                for job in jobs {
                    let share = if all {
                        usize::MAX
                    } else {
                        job.left(cpu).div_ceil(ticks)
                    };
                    work.push((job, share));
                }
            }
            og_lanes::for_each_mut(og_lanes::Site::Motion, &mut work, 1, |(job, share)| {
                cpu.run(statics, job, *share);
            });
            let ms = now_ms() - start;
            if r.startup.first_inference_wall_ms.is_none() {
                r.startup.first_inference_poll_ms += ms;
            }
            r.cpu_ticks.add(tick, ms);
            for (k, f) in r.flights.iter().enumerate() {
                let Work::Cpu(jobs) = &f.work else {
                    unreachable!("a CPU model's flights are CPU jobs")
                };
                if jobs.iter().all(|j| j.left(cpu) == 0) {
                    done.push((k, Ok(gather(jobs))));
                }
            }
        }
        Compute::Gpu { gpu: slot, device } => {
            let f = &mut r.flights[0];
            let block = may_wait && wait(f);
            let mut maintained = false;
            // A test's late GPU (`OG_MOTION_READBACK_DELAY_MS`, as
            // tests/web/motion-ticks.ts delays the web's readbacks): the
            // result is not asked for until that long after the batch
            // started, unless a capture settles.
            #[cfg(not(target_arch = "wasm32"))]
            if !block && now_ms() - f.started < readback_delay_ms() {
                return r.flights.iter().all(|f| !wait(f));
            }
            let Work::Gpu(future) = &mut f.work else {
                unreachable!("a GPU model's flights are GPU batches")
            };
            let woken = Arc::new(Woken::default());
            let waker = Waker::from(woken.clone());
            let mut cx = Context::from_waker(&waker);
            // Blocking runs every slice left at once (a slice's turn wakes
            // it) and, natively, waits for the GPU.
            let polled = loop {
                let start = now_ms();
                woken.0.store(false, std::sync::atomic::Ordering::Relaxed);
                let polled = future.as_mut().poll(&mut cx);
                if r.startup.first_inference_wall_ms.is_none() {
                    r.startup.first_inference_poll_ms += now_ms() - start;
                }
                match polled {
                    Poll::Ready(answer) => break Some(answer),
                    Poll::Pending
                        if block && woken.0.load(std::sync::atomic::Ordering::Relaxed) => {}
                    Poll::Pending if block && cfg!(not(target_arch = "wasm32")) => {
                        #[cfg(not(target_arch = "wasm32"))]
                        let _ = device.poll(wgpu::PollType::wait_indefinitely());
                    }
                    // Natively a readback lands only when the device is
                    // polled: once, without waiting, then asked again.
                    Poll::Pending if !maintained && cfg!(not(target_arch = "wasm32")) => {
                        maintained = true;
                        #[cfg(not(target_arch = "wasm32"))]
                        let _ = device.poll(wgpu::PollType::Poll);
                    }
                    Poll::Pending => break None,
                }
            };
            #[cfg(target_arch = "wasm32")]
            let _ = device;
            if let Some((gpu, result)) = polled {
                r.rate = gpu.rate();
                *slot = Some(gpu);
                done.push((0, result.map_err(|e| e.to_string())));
            }
        }
    }
    for (k, result) in done.into_iter().rev() {
        let flight = r.flights.remove(k);
        if r.startup.first_inference_wall_ms.is_none() {
            let finished = now_ms();
            r.startup.first_inference_wall_ms = Some(finished - flight.started);
            r.startup.inference_finished_at = Some(finished);
        }
        if flight.stale {
            continue;
        }
        match result {
            Ok((student, local)) => {
                r.failure = None;
                r.landings.push(Landing {
                    batch: flight.batch,
                    student,
                    local,
                    next: 0,
                    motion: vec![0.0; FRAMES * MOTION_FEATURES],
                    started: flight.started,
                    due: flight.due,
                });
            }
            Err(e) => {
                if errors.len() < 8 {
                    errors.push(e.clone());
                }
                r.failure = Some(e);
            }
        }
    }
    r.flights.iter().all(|f| !wait(f))
}

/// `OG_MOTION_READBACK_DELAY_MS`, read once (0 without it).
#[cfg(not(target_arch = "wasm32"))]
pub fn readback_delay_ms() -> f64 {
    static DELAY: std::sync::OnceLock<f64> = std::sync::OnceLock::new();
    *DELAY.get_or_init(|| {
        std::env::var("OG_MOTION_READBACK_DELAY_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.0)
    })
}

/// How many batches `dump_motion`'s `taken` lists.
pub const TAKEN: usize = 256;

/// The batches decoded, the last one's characters and wall time, and the
/// last [`TAKEN`] batches' ticks taken and due ([`Presenter`]'s).
pub type Stats = (u64, (usize, f64), VecDeque<(u64, u64)>);

/// Runs the batches due by `tick` until they have landed, and with
/// `starved`, the one with that character: on the GPU, the one outstanding
/// (a restore's queued one started once a stale flight has given the GPU
/// back). Natively it waits; on the web, false while the GPU's readback has
/// not landed.
pub fn settle_batches(
    r: &mut Ready,
    tick: u64,
    starved: Option<usize>,
    may_wait: bool,
    errors: &mut Vec<String>,
) -> bool {
    let gpu = matches!(r.compute, Compute::Gpu { .. });
    let wanted = |b: &Batch, due: u64| {
        due <= tick || starved.is_some_and(|c| gpu || b.characters.contains(&c))
    };
    loop {
        if r.flights.iter().any(|f| wanted(&f.batch, f.due)) {
            if !run_flights(r, tick, |f| wanted(&f.batch, f.due), may_wait, errors) {
                return false;
            }
        } else if r.queued.iter().any(|(b, due)| wanted(b, *due))
            && (!gpu || (r.landings.is_empty() && r.free()))
        {
            start_flights(r, 0);
        } else {
            return true;
        }
    }
}

/// Decodes the landed batches due by `tick` (and with `starved`, the one
/// with that character; on the GPU, the one landed) into their frames, all
/// their characters: how many are decoded never depends on the time it
/// takes. In the order they landed.
pub fn decode(r: &mut Ready, stats: &mut Stats, tick: u64, starved: Option<usize>) {
    let now_ms = r.clock;
    let gpu = matches!(r.compute, Compute::Gpu { .. });
    let mut k = 0;
    while k < r.landings.len() {
        let l = &r.landings[k];
        let forced =
            starved.is_some() && (gpu || l.batch.characters.contains(&starved.unwrap_or(0)));
        if l.due > tick && !forced {
            k += 1;
            continue;
        }
        let mut l = r.landings.remove(k);
        let n = l.batch.characters.len();
        while l.next < n {
            let i = l.next;
            finish_character(
                &r.statics,
                &l.batch.inputs,
                &l.student,
                &l.local,
                i,
                &mut l.motion,
            );
            r.motion
                .accept_character(&l.batch, i, l.student.counts[i], &l.motion);
            l.next += 1;
        }
        // Wall time, a diagnostic.
        r.generations += n as u64;
        stats.0 += 1;
        stats.1 = (n, now_ms() - l.started);
        if stats.2.len() == TAKEN {
            stats.2.pop_front();
        }
        stats.2.push_back((tick, l.due));
    }
}

/// Starts the next batches, their frames due at tick `due`: a restored
/// checkpoint's first, each with its own tick. On the GPU one batch of
/// every character asking, once the last has been decoded. On the CPU one
/// a character asking that has none outstanding; a character's batch whose
/// request changed since ([`Motion::is_current`]) is dropped first, so it
/// asks again at once.
pub fn start_flights(r: &mut Ready, due: u64) {
    let now_ms = r.clock;
    if let Compute::Cpu(_) = r.compute {
        let motion = &r.motion;
        r.flights
            .retain(|f| (0..f.batch.characters.len()).all(|i| motion.is_current(&f.batch, i)));
        r.landings
            .retain(|l| (0..l.batch.characters.len()).all(|i| motion.is_current(&l.batch, i)));
    }
    loop {
        if !r.free() || (matches!(r.compute, Compute::Gpu { .. }) && !r.landings.is_empty()) {
            return;
        }
        let next = if r.queued.is_empty() {
            match &r.compute {
                Compute::Gpu { .. } => r.motion.batch(usize::MAX).map(|b| (b, due)),
                Compute::Cpu(_) => {
                    let busy: BTreeSet<usize> = r
                        .flights
                        .iter()
                        .map(|f| &f.batch)
                        .chain(r.landings.iter().map(|l| &l.batch))
                        .flat_map(|b| b.characters.iter().copied())
                        .collect();
                    (0..r.motion.len())
                        .find(|c| !busy.contains(c) && r.motion.goals(*c) > 0)
                        .and_then(|c| r.motion.batch_where(usize::MAX, |x| x == c))
                        .map(|b| (b, due))
                }
            }
        } else {
            Some(r.queued.remove(0))
        };
        let Some((batch, due)) = next else {
            return;
        };
        let work = match &mut r.compute {
            Compute::Cpu(cpu) => Work::Cpu(
                (0..batch.inputs.n)
                    .map(|i| cpu.job(&r.statics, batch.inputs.character(i)))
                    .collect(),
            ),
            Compute::Gpu { gpu, .. } => {
                let Some(mut gpu) = gpu.take() else {
                    return;
                };
                let inputs = batch.inputs.clone();
                let statics = r.statics.clone();
                Work::Gpu(Box::pin(async move {
                    let result = async {
                        let student = gpu.student(&student_inputs(&statics, &inputs)).await?;
                        let local = gpu
                            .decoder(&decoder_inputs(&statics, &inputs, &student))
                            .await?;
                        Ok((student, local))
                    }
                    .await;
                    (gpu, result)
                }))
            }
        };
        let started = now_ms();
        r.startup
            .first_dispatch_delay_ms
            .get_or_insert(started - r.startup.resident_at);
        r.flights.push(Flight {
            work,
            batch,
            started,
            due,
            stale: false,
        });
    }
}
