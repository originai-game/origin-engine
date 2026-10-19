//! Sounds the mixer plays: a bank of generational handles over decoded
//! (resident) or QOA-streamed PCM.
//!
//! A short sound is decoded once at insert and shared by every voice that
//! plays it; a long one (music, ambience) keeps its QOA bytes and each voice
//! decodes the QOA frames (5120 samples per channel, independent of each
//! other) it reaches, two at a time, so a loop's wrap and a frame boundary
//! never decode twice.

use crate::qoa;
use og_core::{Error, ErrorKind, Handle, Pool, Result};
use std::sync::{Arc, OnceLock, RwLock};

/// Samples per channel in a full QOA frame.
pub const QOA_FRAME_LEN: u64 = 5120;
const QOA_MAGIC: u32 = 0x716f_6166;

/// How a sound is held.
enum Storage {
    /// Interleaved PCM, `frames * channels` samples.
    Resident(Vec<i16>),
    /// The QOA file and the byte range and length of each of its frames.
    Streamed {
        qoa: Vec<u8>,
        frames: Vec<(usize, usize)>,
    },
    /// No samples: a streamed asset's stand-in until its group loads (the
    /// same rate and length, so the tick side times it the same). The cell
    /// receives the real sound when it arrives, and voices already playing
    /// the stand-in switch to it at the frame they are on.
    Silent(Arc<OnceLock<Arc<SoundData>>>),
}

/// A sound in the bank.
pub struct SoundData {
    /// Where it came from (the asset id).
    pub id: String,
    pub sample_rate: u32,
    /// Channels in the source; the mixer reads the first two.
    pub channels: usize,
    /// Samples per channel.
    pub frames: u64,
    storage: Storage,
}

impl std::fmt::Debug for SoundData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SoundData")
            .field("id", &self.id)
            .field("sample_rate", &self.sample_rate)
            .field("channels", &self.channels)
            .field("frames", &self.frames)
            .field("streamed", &self.streamed())
            .finish()
    }
}

impl SoundData {
    /// Whether voices decode it QOA frame by QOA frame.
    pub fn streamed(&self) -> bool {
        matches!(self.storage, Storage::Streamed { .. })
    }

    /// Length in seconds.
    pub fn duration(&self) -> f64 {
        self.frames as f64 / f64::from(self.sample_rate.max(1))
    }

    /// Decodes QOA frame `k` (interleaved samples).
    fn decode_frame(&self, k: usize) -> Option<Vec<i16>> {
        let Storage::Streamed { qoa, frames } = &self.storage else {
            return None;
        };
        let &(offset, size) = frames.get(k)?;
        let len = (self.frames - k as u64 * QOA_FRAME_LEN).min(QOA_FRAME_LEN) as u32;
        // A QOA frame is self-contained (its own LMS state), so one frame
        // behind a file header is a valid QOA file of `len` samples.
        let mut file = Vec::with_capacity(8 + size);
        file.extend_from_slice(&QOA_MAGIC.to_be_bytes());
        file.extend_from_slice(&len.to_be_bytes());
        file.extend_from_slice(&qoa[offset..offset + size]);
        qoa::decode(&file).ok().map(|d| d.samples)
    }
}

/// A QOA file's frame table: channels, rate, samples per channel, and
/// (byte offset, byte size) per frame.
type QoaIndex = (usize, u32, u64, Vec<(usize, usize)>);

/// Indexes a QOA file's frames from their headers, checking each the way
/// the decoder will.
fn index_qoa(bytes: &[u8]) -> std::result::Result<QoaIndex, String> {
    let be64 = |at: usize| -> Option<u64> {
        Some(u64::from_be_bytes(bytes.get(at..at + 8)?.try_into().ok()?))
    };
    let header = be64(0).ok_or("malformed QOA: no header")?;
    if (header >> 32) as u32 != QOA_MAGIC {
        return Err("malformed QOA: wrong magic".into());
    }
    let total = header & 0xffff_ffff;
    let mut pos = 8;
    let mut seen = 0u64;
    let (mut channels, mut rate) = (0usize, 0u32);
    let mut frames = Vec::new();
    while seen < total {
        let fh = be64(pos).ok_or("malformed QOA: truncated frame header")?;
        let ch = (fh >> 56) as usize;
        let fr = ((fh >> 32) & 0xff_ffff) as u32;
        let len = (fh >> 16) & 0xffff;
        let size = (fh & 0xffff) as usize;
        if ch == 0 || ch > qoa::MAX_CHANNELS || len == 0 || len > QOA_FRAME_LEN {
            return Err("malformed QOA: invalid frame header".into());
        }
        if channels == 0 {
            (channels, rate) = (ch, fr);
        } else if ch != channels || fr != rate {
            return Err("malformed QOA: channel count or rate changes between frames".into());
        }
        let slices = (len as usize).div_ceil(20);
        if size != 8 + 16 * ch + 8 * slices * ch || pos + size > bytes.len() {
            return Err("malformed QOA: frame size does not match its header".into());
        }
        if seen + len < total && len != QOA_FRAME_LEN {
            return Err("malformed QOA: a short frame before the last".into());
        }
        frames.push((pos, size));
        pos += size;
        seen += len;
    }
    if channels == 0 {
        return Err("malformed QOA: no frames".into());
    }
    Ok((channels, rate, total, frames))
}

fn invalid(id: &str, reason: impl std::fmt::Display) -> Error {
    Error::new(ErrorKind::Invalid, format!("sound '{id}': {reason}"))
}

struct Bank {
    pool: Pool<Arc<SoundData>>,
    resident_seconds: f64,
}

/// The sounds of one engine, shared by its tick system (lengths and rates)
/// and its mixer (samples). Cloning shares the bank.
#[derive(Clone)]
pub struct SoundBank {
    inner: Arc<RwLock<Bank>>,
}

impl Default for SoundBank {
    fn default() -> Self {
        SoundBank::new()
    }
}

impl std::fmt::Debug for SoundBank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SoundBank({} sounds)", self.len())
    }
}

/// Sounds up to this long are decoded at insert; longer ones stream.
pub const DEFAULT_RESIDENT_SECONDS: f64 = 6.0;

impl SoundBank {
    pub fn new() -> SoundBank {
        SoundBank::with_resident_seconds(DEFAULT_RESIDENT_SECONDS)
    }

    /// A bank that decodes sounds up to `seconds` long at insert and streams
    /// the rest (0 streams everything).
    pub fn with_resident_seconds(seconds: f64) -> SoundBank {
        SoundBank {
            inner: Arc::new(RwLock::new(Bank {
                pool: Pool::new(),
                resident_seconds: seconds.max(0.0),
            })),
        }
    }

    fn put(&self, data: SoundData) -> u64 {
        let mut bank = self.inner.write().expect("sound bank lock");
        bank.pool.insert(Arc::new(data)).to_bits()
    }

    /// Adds a QOA file: a cooked sound's `qoa` section
    /// (`og_cook::runtime::CookedSound { id, qoa, .. }` →
    /// `insert_qoa(&cooked.id, &cooked.qoa)`). Returns its handle (never 0).
    pub fn insert_qoa(&self, id: &str, bytes: &[u8]) -> Result<u64> {
        let data = self.decode_qoa(id, bytes)?;
        Ok(self.put(data))
    }

    /// A silent sound of a given shape (a streamed sound before its group
    /// is resident): voices play it as silence and the tick side reads the
    /// same rate and length the real one has.
    pub fn insert_silent(
        &self,
        id: &str,
        sample_rate: u32,
        channels: usize,
        frames: u64,
    ) -> Result<u64> {
        if channels == 0 || sample_rate == 0 {
            return Err(invalid(id, "needs 1+ channels and a rate"));
        }
        Ok(self.put(SoundData {
            id: id.to_string(),
            sample_rate,
            channels,
            frames,
            storage: Storage::Silent(Arc::default()),
        }))
    }

    /// Puts a QOA file's samples behind an existing handle (its group
    /// loaded). The file must have the handle's rate, channels and length,
    /// so nothing the tick side reads changes; voices playing the silent
    /// stand-in continue with the real samples from where they are. False
    /// for a handle not in the bank.
    pub fn replace_qoa(&self, handle: u64, id: &str, bytes: &[u8]) -> Result<bool> {
        let Some(old) = self.get(handle) else {
            return Ok(false);
        };
        let data = self.decode_qoa(id, bytes)?;
        if (data.sample_rate, data.channels, data.frames)
            != (old.sample_rate, old.channels, old.frames)
        {
            return Err(invalid(
                id,
                format!(
                    "{} Hz x{} for {} frames replaces {} Hz x{} for {}",
                    data.sample_rate,
                    data.channels,
                    data.frames,
                    old.sample_rate,
                    old.channels,
                    old.frames
                ),
            ));
        }
        let data = Arc::new(data);
        if let Storage::Silent(cell) = &old.storage {
            let _ = cell.set(data.clone());
        }
        Ok(self.swap(handle, data))
    }

    /// Makes a sound silent again, keeping its handle and shape (its group
    /// unloaded); voices already playing it finish with its samples. False
    /// for a handle not in the bank.
    pub fn silence(&self, handle: u64) -> bool {
        let Some(old) = self.get(handle) else {
            return false;
        };
        self.swap(
            handle,
            Arc::new(SoundData {
                id: old.id.clone(),
                sample_rate: old.sample_rate,
                channels: old.channels,
                frames: old.frames,
                storage: Storage::Silent(Arc::default()),
            }),
        )
    }

    fn swap(&self, handle: u64, data: Arc<SoundData>) -> bool {
        let mut bank = self.inner.write().expect("sound bank lock");
        match bank.pool.get_mut(Handle::from_bits(handle)) {
            Ok(slot) => {
                *slot = data;
                true
            }
            Err(_) => false,
        }
    }

    /// Whether a handle's sound has samples (not a silent stand-in).
    pub fn audible(&self, handle: u64) -> bool {
        self.get(handle)
            .is_some_and(|s| !matches!(s.storage, Storage::Silent(_)))
    }

    fn decode_qoa(&self, id: &str, bytes: &[u8]) -> Result<SoundData> {
        let (channels, sample_rate, frames, index) =
            index_qoa(bytes).map_err(|e| invalid(id, e))?;
        if sample_rate == 0 {
            return Err(invalid(id, "sample rate 0"));
        }
        let resident_seconds = self.inner.read().expect("sound bank lock").resident_seconds;
        let storage = if frames as f64 <= resident_seconds * f64::from(sample_rate) {
            let decoded = qoa::decode(bytes).map_err(|e| invalid(id, e))?;
            Storage::Resident(decoded.samples)
        } else {
            Storage::Streamed {
                qoa: bytes.to_vec(),
                frames: index,
            }
        };
        Ok(SoundData {
            id: id.to_string(),
            sample_rate,
            channels,
            frames,
            storage,
        })
    }

    /// Adds interleaved 16-bit PCM, always resident (generated tones, tests).
    pub fn insert_pcm(
        &self,
        id: &str,
        sample_rate: u32,
        channels: usize,
        samples: &[i16],
    ) -> Result<u64> {
        if channels == 0 || sample_rate == 0 || !samples.len().is_multiple_of(channels) {
            return Err(invalid(id, "needs 1+ channels, a rate and whole frames"));
        }
        Ok(self.put(SoundData {
            id: id.to_string(),
            sample_rate,
            channels,
            frames: (samples.len() / channels) as u64,
            storage: Storage::Resident(samples.to_vec()),
        }))
    }

    /// The sound of a handle, while it is in the bank.
    pub fn get(&self, handle: u64) -> Option<Arc<SoundData>> {
        if handle == 0 {
            return None;
        }
        let bank = self.inner.read().expect("sound bank lock");
        bank.pool.get(Handle::from_bits(handle)).ok().cloned()
    }

    /// (sample rate, frames) of a handle: what the tick side needs.
    pub fn info(&self, handle: u64) -> Option<(u32, u64)> {
        self.get(handle).map(|s| (s.sample_rate, s.frames))
    }

    /// Drops a sound; voices playing it keep their reference until they end.
    pub fn remove(&self, handle: u64) -> bool {
        let mut bank = self.inner.write().expect("sound bank lock");
        bank.pool.remove(Handle::from_bits(handle)).is_ok()
    }

    pub fn len(&self) -> usize {
        self.inner.read().expect("sound bank lock").pool.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A voice's view of a sound: stereo frames by index, decoding QOA frames
/// on demand into two slots.
pub(crate) struct Reader {
    data: Arc<SoundData>,
    slots: [(usize, Vec<i16>); 2],
    evict: usize,
}

const EMPTY: usize = usize::MAX;
const SCALE: f32 = 1.0 / 32768.0;

impl Reader {
    pub fn new(data: Arc<SoundData>) -> Reader {
        Reader {
            data,
            slots: [(EMPTY, Vec::new()), (EMPTY, Vec::new())],
            evict: 0,
        }
    }

    /// Frame `i` as (left, right); past the end is silence.
    #[inline]
    pub fn frame(&mut self, i: u64) -> (f32, f32) {
        let ch = self.data.channels;
        if i >= self.data.frames {
            return (0.0, 0.0);
        }
        let (buf, at): (&[i16], usize) = match &self.data.storage {
            Storage::Resident(s) => (s, i as usize * ch),
            Storage::Silent(cell) => match cell.get() {
                Some(real) => {
                    self.data = real.clone();
                    self.slots = [(EMPTY, Vec::new()), (EMPTY, Vec::new())];
                    return self.frame(i);
                }
                None => return (0.0, 0.0),
            },
            Storage::Streamed { .. } => {
                let k = (i / QOA_FRAME_LEN) as usize;
                let off = (i % QOA_FRAME_LEN) as usize * ch;
                let slot = if self.slots[0].0 == k {
                    0
                } else if self.slots[1].0 == k {
                    1
                } else {
                    let s = self.evict;
                    self.evict ^= 1;
                    self.slots[s] = (k, self.data.decode_frame(k).unwrap_or_default());
                    s
                };
                // The slot the next miss evicts is the one not just used.
                self.evict = slot ^ 1;
                (&self.slots[slot].1, off)
            }
        };
        let l = buf.get(at).copied().unwrap_or(0);
        let r = if ch > 1 {
            buf.get(at + 1).copied().unwrap_or(0)
        } else {
            l
        };
        (f32::from(l) * SCALE, f32::from(r) * SCALE)
    }
}
