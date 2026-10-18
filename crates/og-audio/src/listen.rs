//! What the mixer rendered last, for the probe's `listen`: a ring of the
//! newest stereo frames (two seconds at the mixer's rate) that the sink's
//! mixer writes after every render.

use std::sync::{Arc, Mutex};

/// Seconds the ring keeps.
pub const LISTEN_SECONDS: u32 = 2;

#[derive(Default)]
struct Ring {
    rate: u32,
    /// Interleaved stereo, `capacity * 2` samples.
    pcm: Vec<f32>,
    /// Frames written, ever.
    written: u64,
}

/// The ring an engine's mixer records into. Cloning shares it.
#[derive(Clone, Default)]
pub struct Listen(Arc<Mutex<Ring>>);

impl std::fmt::Debug for Listen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Listen")
    }
}

impl Listen {
    /// Appends rendered interleaved stereo frames at `rate`.
    pub fn record(&self, pcm: &[f32], rate: u32) {
        let mut r = self.0.lock().unwrap_or_else(|e| e.into_inner());
        let cap = (rate * LISTEN_SECONDS) as usize;
        if r.rate != rate || r.pcm.len() != cap * 2 {
            r.rate = rate;
            r.pcm = vec![0.0; cap * 2];
            r.written = 0;
        }
        for f in pcm.as_chunks::<2>().0 {
            let at = (r.written % cap as u64) as usize * 2;
            r.pcm[at] = f[0];
            r.pcm[at + 1] = f[1];
            r.written += 1;
        }
    }

    /// The newest `seconds` (at most what was recorded): interleaved stereo
    /// frames, oldest first, and the rate; `None` before anything was recorded.
    pub fn last(&self, seconds: f32) -> Option<(Vec<f32>, u32)> {
        let r = self.0.lock().unwrap_or_else(|e| e.into_inner());
        if r.rate == 0 || r.written == 0 {
            return None;
        }
        let cap = r.pcm.len() / 2;
        let want = ((seconds.clamp(0.0, LISTEN_SECONDS as f32) * r.rate as f32) as u64)
            .min(r.written)
            .min(cap as u64);
        let mut out = Vec::with_capacity(want as usize * 2);
        for k in r.written - want..r.written {
            let at = (k % cap as u64) as usize * 2;
            out.push(r.pcm[at]);
            out.push(r.pcm[at + 1]);
        }
        Some((out, r.rate))
    }

    /// Frames recorded so far.
    pub fn frames(&self) -> u64 {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).written
    }
}
