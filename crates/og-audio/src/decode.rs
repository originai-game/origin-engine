//! Decoding beside the tick: the QOA frames voices will read are decoded
//! before they read them, by a helper the host runs off the engine thread
//! (natively a thread of this crate, [`SoundBank::start_decode_thread`];
//! on the web a decoder Worker the engine Worker feeds through
//! [`SoundBank::take_decode_jobs`] and [`SoundBank::put_decoded`]).
//!
//! What is decoded is planned, never timed:
//!
//! - **Pins**, queued when a sound enters the bank: every frame of a short
//!   sound (up to the bank's resident seconds; it stays decoded, as one
//!   decoded at insert did) and the first frame of every other, so a start
//!   from the top finds it.
//! - **The decode-ahead**, planned by the `og.audio` system at the start of
//!   every tick from the world's playheads (`og.AudioPlayback`): per heard
//!   voice, the QOA frame its cursor is in and the next ones up to
//!   [`DECODE_AHEAD_SECONDS`] of the device at its pitch, at most
//!   [`DECODE_AHEAD_FRAMES`] past the cursor's, across a loop's wrap;
//!   soonest first. Frames decoded ahead and no longer planned are dropped
//!   (a long sound is never decoded whole).
//!
//! The plan is a function of the world, but whether the helper has run is
//! not, so nothing reads it except the mixer, and the mixer reads it only
//! as a cache: a frame not decoded yet (a miss: the helper behind, a start
//! part-way into a long sound, a host without a helper) is decoded where
//! the voice reads it, to the same samples. The PCM, the command stream
//! and the world are the same whatever the helper did; only where the
//! decoding ran differs, and [`DecodeStats`] counts it.

use crate::qoa;
use crate::sound::{QOA_FRAME_LEN, SoundBank, SoundData};
use og_core::IdMap;
use std::collections::{BTreeSet, VecDeque};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::atomic::AtomicBool;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, Weak};

/// Seconds of the device a voice's decode-ahead covers past its cursor.
pub const DECODE_AHEAD_SECONDS: f64 = 0.25;
/// The most QOA frames a voice's decode-ahead reaches past the one its
/// cursor is in (a voice pitched up covers fewer seconds).
pub const DECODE_AHEAD_FRAMES: u64 = 4;
/// Plans after which a job handed to a helper that never answered is
/// handed out again (a decoder Worker that died).
const IN_FLIGHT_PLANS: u64 = 120;

/// What the decoding has done and holds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DecodeStats {
    /// Jobs waiting for a helper (the decode-ahead and the pins).
    pub queued: u64,
    /// Jobs a helper took and has not answered.
    pub in_flight: u64,
    /// Frames a helper decoded.
    pub ahead: u64,
    /// Frames a voice decoded where it read them (none decoded them first).
    pub misses: u64,
    /// Plans made (ticks of the `og.audio` system with a sink attached).
    pub plans: u64,
}

/// One QOA frame to decode: `qoa` is a QOA file of that frame alone
/// (`qoaf`, its length, the frame), so any QOA decoder reads it.
pub struct DecodeJob {
    /// The answer's key ([`SoundBank::put_decoded`]).
    pub key: u32,
    /// The QOA frame's index in its sound.
    pub frame: u32,
    /// The one-frame QOA file.
    pub qoa: Vec<u8>,
}

impl DecodeJob {
    /// Decodes it with this crate's codec: interleaved samples.
    pub fn decode(&self) -> Option<Vec<i16>> {
        qoa::decode(&self.qoa).ok().map(|d| d.samples)
    }
}

/// A frame of a sound: the sound (held weakly: a sound the bank dropped is
/// not decoded) and its QOA frame index.
#[derive(Clone)]
struct Entry {
    sound: Weak<SoundData>,
    frame: usize,
}

impl Entry {
    fn id(&self) -> (usize, usize) {
        (self.sound.as_ptr() as usize, self.frame)
    }
}

#[derive(Default)]
struct State {
    /// This tick's decode-ahead, soonest first.
    ahead: VecDeque<Entry>,
    /// Pinned frames, in the order their sounds arrived.
    pins: VecDeque<Entry>,
    /// Jobs a helper holds, by key, with the plan they were taken in.
    in_flight: IdMap<u32, (Entry, u64)>,
    flying: BTreeSet<(usize, usize)>,
    next_key: u32,
    /// Frames decoded ahead that are not pinned: dropped once unplanned.
    kept: Vec<Entry>,
    plans: u64,
    ahead_decoded: u64,
    closed: bool,
}

/// A bank's decoding: its queue, the jobs helpers hold, and its counters.
#[derive(Default)]
pub(crate) struct Decoding {
    state: Mutex<State>,
    wake: Condvar,
    misses: AtomicU64,
    /// Whether the decode thread runs.
    #[cfg(not(target_arch = "wasm32"))]
    thread: AtomicBool,
}

/// A playing voice's playhead, as the decode-ahead reads it.
pub(crate) struct Head {
    pub sound: Arc<SoundData>,
    pub cursor: f64,
    pub pitch: f32,
    pub looping: bool,
    pub loop_start: u64,
    /// Exclusive; 0 is the sound's end.
    pub loop_end: u64,
}

impl Decoding {
    fn state(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().expect("audio decoding")
    }

    pub(crate) fn missed(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn close(&self) {
        self.state().closed = true;
        self.wake.notify_all();
    }

    /// Queues a sound's pinned frames (every frame of a short sound, else
    /// its first).
    pub(crate) fn pin(&self, data: &Arc<SoundData>) {
        let Some(cache) = &data.cache else { return };
        let n = if cache.pinned {
            data.qoa_frames()
        } else {
            data.qoa_frames().min(1)
        };
        let mut s = self.state();
        for frame in 0..n {
            s.pins.push_back(Entry {
                sound: Arc::downgrade(data),
                frame,
            });
        }
        drop(s);
        self.wake.notify_all();
    }

    /// The decode-ahead of this tick's playheads: replaces the last one,
    /// and drops frames decoded ahead that it no longer wants.
    pub(crate) fn plan(&self, heads: &[Head]) {
        // (seconds until needed, entry), in head order.
        let mut wanted: Vec<(f64, Entry)> = Vec::new();
        for h in heads {
            if h.sound.cache.is_none() {
                continue;
            }
            let total = h.sound.qoa_frames() as u64;
            if total == 0 {
                continue;
            }
            let speed = f64::from(h.sound.sample_rate.max(1))
                * f64::from(crate::system::clamp_pitch(h.pitch));
            let cursor = h.cursor.max(0.0);
            let reach = DECODE_AHEAD_SECONDS * speed;
            let mut want = |from: f64, to: f64, offset: f64| {
                let a = from as u64 / QOA_FRAME_LEN;
                let b = (to as u64 / QOA_FRAME_LEN)
                    .min(a + DECODE_AHEAD_FRAMES)
                    .min(total - 1);
                for k in a..=b.max(a).min(total - 1) {
                    let at = ((k * QOA_FRAME_LEN) as f64 - from).max(0.0);
                    wanted.push((
                        offset + at / speed,
                        Entry {
                            sound: Arc::downgrade(&h.sound),
                            frame: k as usize,
                        },
                    ));
                }
            };
            let end = if h.looping {
                if h.loop_end == 0 {
                    h.sound.frames
                } else {
                    h.loop_end.min(h.sound.frames)
                }
            } else {
                h.sound.frames
            } as f64;
            if h.looping && cursor + reach >= end && end > h.loop_start as f64 {
                // Up to the wrap, then on from the loop's start.
                want(cursor, (end - 1.0).max(cursor), 0.0);
                let wrap = (end - cursor).max(0.0) / speed;
                let rest = (cursor + reach - end).min(end - h.loop_start as f64);
                want(h.loop_start as f64, h.loop_start as f64 + rest, wrap);
            } else {
                want(cursor, (cursor + reach).min(end), 0.0);
            }
        }
        og_core::sort::sort_by_key(&mut wanted, |w| w.0.to_bits());
        let keep: BTreeSet<(usize, usize)> = wanted.iter().map(|w| w.1.id()).collect();
        let mut evict = Vec::new();
        {
            let mut s = self.state();
            s.plans += 1;
            let plans = s.plans;
            let expired: Vec<u32> = s
                .in_flight
                .iter()
                .filter(|(_, (_, at))| at + IN_FLIGHT_PLANS < plans)
                .map(|(k, _)| *k)
                .collect();
            for k in expired {
                if let Some((e, _)) = s.in_flight.remove(&k) {
                    s.flying.remove(&e.id());
                }
            }
            let mut seen = BTreeSet::new();
            let ahead: VecDeque<Entry> = wanted
                .into_iter()
                .map(|w| w.1)
                .filter(|e| seen.insert(e.id()) && !s.flying.contains(&e.id()) && !decoded(e))
                .collect();
            s.ahead = ahead;
            let kept = std::mem::take(&mut s.kept);
            for e in kept {
                if keep.contains(&e.id()) {
                    s.kept.push(e);
                } else {
                    evict.push(e);
                }
            }
        }
        for e in evict {
            if let Some(d) = e.sound.upgrade()
                && let Some(c) = &d.cache
            {
                c.evict(e.frame);
            }
        }
        self.wake.notify_all();
    }

    /// The next job, marked in flight; None when nothing is queued.
    fn take(&self) -> Option<(u32, Arc<SoundData>, usize)> {
        let mut s = self.state();
        loop {
            let e = match s.ahead.pop_front() {
                Some(e) => e,
                None => s.pins.pop_front()?,
            };
            let Some(data) = e.sound.upgrade() else {
                continue;
            };
            let decoded = data.cache.as_ref().is_none_or(|c| c.get(e.frame).is_some());
            if decoded || s.flying.contains(&e.id()) {
                continue;
            }
            let key = s.next_key;
            s.next_key = s.next_key.wrapping_add(1);
            let plans = s.plans;
            s.flying.insert(e.id());
            s.in_flight.insert(key, (e.clone(), plans));
            return Some((key, data, e.frame));
        }
    }

    /// The answer to job `key`: its samples (None: it could not be
    /// decoded). False for a key not in flight or samples of another length.
    fn finish(&self, key: u32, samples: Option<Arc<[i16]>>) -> bool {
        let entry = {
            let mut s = self.state();
            let Some((e, _)) = s.in_flight.remove(&key) else {
                return false;
            };
            s.flying.remove(&e.id());
            e
        };
        let (Some(samples), Some(data)) = (samples, entry.sound.upgrade()) else {
            return false;
        };
        let Some(cache) = &data.cache else {
            return false;
        };
        if samples.len() as u64 != data.frame_len(entry.frame) * data.channels as u64 {
            return false;
        }
        cache.put(entry.frame, samples);
        let mut s = self.state();
        s.ahead_decoded += 1;
        if !cache.pins(entry.frame) {
            s.kept.push(entry);
        }
        true
    }

    /// Takes the next job, waiting for one; None once the bank is gone.
    #[cfg(not(target_arch = "wasm32"))]
    fn take_waiting(&self) -> Option<(u32, Arc<SoundData>, usize)> {
        loop {
            {
                let s = self.state();
                if s.closed {
                    return None;
                }
                if s.ahead.is_empty() && s.pins.is_empty() {
                    drop(self.wake.wait(s).expect("audio decoding"));
                    continue;
                }
            }
            if let Some(job) = self.take() {
                return Some(job);
            }
        }
    }

    fn stats(&self) -> DecodeStats {
        let s = self.state();
        DecodeStats {
            queued: (s.ahead.len() + s.pins.len()) as u64,
            in_flight: s.in_flight.len() as u64,
            ahead: s.ahead_decoded,
            misses: self.misses.load(Ordering::Relaxed),
            plans: s.plans,
        }
    }
}

/// Whether an entry's frame is in its sound's cache (or its sound is gone).
fn decoded(e: &Entry) -> bool {
    e.sound
        .upgrade()
        .is_none_or(|d| d.cache.as_ref().is_none_or(|c| c.get(e.frame).is_some()))
}

/// Decodes job `(key, data, frame)` here and answers it.
fn run(d: &Decoding, key: u32, data: &SoundData, frame: usize) {
    let samples = data
        .frame_file(frame)
        .and_then(|file| qoa::decode(&file).ok())
        .map(|x| Arc::<[i16]>::from(x.samples));
    d.finish(key, samples);
}

impl SoundBank {
    /// Up to `max` queued frames as jobs for a helper off the engine
    /// thread (the web host's decoder Worker), soonest needed first; each
    /// answered with [`SoundBank::put_decoded`]. A frame whose chunk is not
    /// resident is skipped (planned again next tick).
    pub fn take_decode_jobs(&self, max: usize) -> Vec<DecodeJob> {
        let d = self.decoding();
        let mut jobs = Vec::new();
        while jobs.len() < max {
            let Some((key, data, frame)) = d.take() else {
                break;
            };
            match data.frame_file(frame) {
                Some(qoa) => jobs.push(DecodeJob {
                    key,
                    frame: frame as u32,
                    qoa,
                }),
                None => {
                    d.finish(key, None);
                }
            }
        }
        jobs
    }

    /// A helper's answer to job `key`: the frame's interleaved samples. True
    /// when they were put in the sound's frame cache; false for a key not
    /// in flight (answered, or handed out again) or samples not of the
    /// frame's length. The samples must be the QOA decode of the job's
    /// file: voices read them as their own decode.
    pub fn put_decoded(&self, key: u32, samples: &[i16]) -> bool {
        self.decoding().finish(key, Some(Arc::from(samples)))
    }

    /// Runs up to `max` queued jobs on the calling thread; returns how many
    /// it ran. A host's helper that is not a thread of this crate, and tests.
    pub fn decode_queued(&self, max: usize) -> usize {
        let d = self.decoding();
        let mut n = 0;
        while n < max {
            let Some((key, data, frame)) = d.take() else {
                break;
            };
            run(&d, key, &data, frame);
            n += 1;
        }
        n
    }

    /// Starts the bank's decode thread (natively; og-runtime does for every
    /// engine): it runs
    /// every job as it is queued, and ends when the last clone of the bank
    /// is dropped. Once per bank: a second call is true and starts nothing.
    /// False when the thread could not start (then voices decode where they
    /// read, as without one).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn start_decode_thread(&self) -> bool {
        let d = self.decoding();
        if d.thread.swap(true, Ordering::AcqRel) {
            return true;
        }
        let worker = d.clone();
        std::thread::Builder::new()
            .name("og-audio-decode".into())
            .spawn(move || {
                while let Some((key, data, frame)) = worker.take_waiting() {
                    run(&worker, key, &data, frame);
                }
            })
            .inspect_err(|_| d.thread.store(false, Ordering::Release))
            .is_ok()
    }

    /// What the decoding has done and holds.
    pub fn decode_stats(&self) -> DecodeStats {
        self.decoding().stats()
    }
}

/// Plans the decode-ahead of the world's playheads (the `og.audio` system,
/// at the start of each tick with a sink attached).
pub(crate) fn plan(bank: &SoundBank, heads: Vec<Head>) {
    bank.decoding().plan(&heads);
}
