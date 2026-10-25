//! Sounds the mixer plays: a bank of generational handles over QOA files
//! (or 16-bit PCM) and the QOA frames decoded from them.
//!
//! Nothing is decoded at insert. A QOA file is indexed (its frame table)
//! and its QOA frames (5120 samples per channel, independent of each
//! other) are decoded beside the tick into the sound's frame cache
//! (`decode.rs`): every frame of a short sound, which stays decoded, and
//! the first frame of a long one; then whatever the tick's decode-ahead
//! plans from the playheads. A voice reads the cache, two frames at a
//! time, and decodes a frame nobody has decoded yet itself (a miss), to
//! the same samples, so the mix never depends on when the helper ran.
//!
//! A sound cooked in chunks (`sound { stream true }`, or a long one) is
//! not resident whole: the bank holds its first chunk and whichever others
//! the host put there ([`SoundBank::put_chunk`]), as [`chunks_wanted`]
//! asks for them from the playheads, and drops the rest
//! ([`SoundBank::keep_chunks`]). A QOA frame is self-contained, so a
//! chunk's frames decode exactly as the same frames of the whole file.

use crate::decode::Decoding;
use crate::qoa;
use og_core::IdMap;
use og_core::{Error, ErrorKind, Handle, Pool, Result};
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex, OnceLock, RwLock};

/// Samples per channel in a full QOA frame.
pub const QOA_FRAME_LEN: u64 = 5120;
const QOA_MAGIC: u32 = 0x716f_6166;

/// How a sound is held.
enum Storage {
    /// Interleaved PCM, `frames * channels` samples (`insert_pcm`).
    Resident(Vec<i16>),
    /// The QOA file and the byte range of each of its frames, decoded into
    /// the sound's frame cache.
    Qoa {
        qoa: Vec<u8>,
        frames: Vec<(usize, usize)>,
    },
    /// No samples: a streamed asset's stand-in until its group loads (the
    /// same rate and length, so the tick side times it the same). The cell
    /// receives the real sound when it arrives, and voices already playing
    /// the stand-in switch to it at the frame they are on.
    Silent(Arc<OnceLock<Arc<SoundData>>>),
    /// Cooked in chunks of `per_chunk` QOA frames: the first always
    /// resident, the others while the host keeps them there.
    Chunked(Chunked),
}

/// The QOA frames of one chunk: its bytes and each frame's byte range.
struct Chunk {
    bytes: Vec<u8>,
    frames: Vec<(usize, usize)>,
}

/// Reads chunk `n` of a chunked sound (its QOA frames), or None.
pub type ChunkLoader = Arc<dyn Fn(u32) -> Option<Vec<u8>> + Send + Sync>;

struct Chunked {
    /// QOA frames per chunk (the last chunk may hold fewer).
    per_chunk: usize,
    count: usize,
    /// Slot 0 is always the first chunk.
    slots: RwLock<Vec<Option<Arc<Chunk>>>>,
    loader: RwLock<Option<ChunkLoader>>,
}

impl Chunked {
    fn chunk(&self, c: usize) -> Option<Arc<Chunk>> {
        self.slots
            .read()
            .expect("chunk lock")
            .get(c)
            .cloned()
            .flatten()
    }
}

/// The decoded QOA frames of a QOA or chunked sound.
pub(crate) struct FrameCache {
    /// Per QOA frame, its interleaved samples once decoded.
    pub(crate) slots: Mutex<Vec<Option<Arc<[i16]>>>>,
    /// Every frame stays once decoded (a short sound, as one decoded at
    /// insert used to); otherwise only the first does, and the others while
    /// the decode-ahead wants them.
    pub(crate) pinned: bool,
    /// The bank's decoding (its queue and counters).
    pub(crate) decoding: Arc<Decoding>,
}

impl FrameCache {
    pub(crate) fn get(&self, k: usize) -> Option<Arc<[i16]>> {
        self.slots
            .lock()
            .expect("frame cache")
            .get(k)
            .cloned()
            .flatten()
    }

    /// Whether frame `k` stays decoded for good.
    pub(crate) fn pins(&self, k: usize) -> bool {
        self.pinned || k == 0
    }

    pub(crate) fn put(&self, k: usize, samples: Arc<[i16]>) {
        if let Some(slot) = self.slots.lock().expect("frame cache").get_mut(k) {
            *slot = Some(samples);
        }
    }

    pub(crate) fn evict(&self, k: usize) {
        if !self.pins(k)
            && let Some(slot) = self.slots.lock().expect("frame cache").get_mut(k)
        {
            *slot = None;
        }
    }

    fn bytes(&self) -> usize {
        self.slots
            .lock()
            .expect("frame cache")
            .iter()
            .flatten()
            .map(|s| s.len() * 2)
            .sum()
    }
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
    /// Decoded QOA frames (a QOA or chunked sound).
    pub(crate) cache: Option<FrameCache>,
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
    /// Whether it streams: a long QOA or chunked sound, whose frames are
    /// decoded as the playheads near them and dropped behind them (a short
    /// one's stay decoded once decoded).
    pub fn streamed(&self) -> bool {
        match &self.storage {
            Storage::Qoa { .. } => self.cache.as_ref().is_some_and(|c| !c.pinned),
            Storage::Chunked(_) => true,
            _ => false,
        }
    }

    /// QOA frames in the sound (0 for PCM or a silent stand-in).
    pub(crate) fn qoa_frames(&self) -> usize {
        match &self.storage {
            Storage::Qoa { .. } | Storage::Chunked(_) => {
                self.frames.div_ceil(QOA_FRAME_LEN) as usize
            }
            _ => 0,
        }
    }

    /// For a chunked sound: sample frames per chunk and the chunk count.
    pub fn chunking(&self) -> Option<(u64, u32)> {
        match &self.storage {
            Storage::Chunked(c) => Some((c.per_chunk as u64 * QOA_FRAME_LEN, c.count as u32)),
            _ => None,
        }
    }

    /// The chunks a chunked sound holds now (ascending; the first always).
    pub fn resident_chunks(&self) -> Vec<u32> {
        match &self.storage {
            Storage::Chunked(c) => c
                .slots
                .read()
                .expect("chunk lock")
                .iter()
                .enumerate()
                .filter(|(_, s)| s.is_some())
                .map(|(i, _)| i as u32)
                .collect(),
            _ => Vec::new(),
        }
    }

    /// Bytes its samples take in memory now: PCM, a QOA sound's file, a
    /// chunked sound's resident chunks, and the QOA frames decoded from
    /// them.
    pub fn resident_bytes(&self) -> usize {
        let held = match &self.storage {
            Storage::Resident(s) => s.len() * 2,
            Storage::Qoa { qoa, .. } => qoa.len(),
            Storage::Silent(_) => 0,
            Storage::Chunked(c) => c
                .slots
                .read()
                .expect("chunk lock")
                .iter()
                .flatten()
                .map(|k| k.bytes.len())
                .sum(),
        };
        held + self.cache.as_ref().map_or(0, FrameCache::bytes)
    }

    /// Length in seconds.
    pub fn duration(&self) -> f64 {
        self.frames as f64 / f64::from(self.sample_rate.max(1))
    }

    /// QOA frame `k` as a one-frame QOA file, for a decoder beside the
    /// tick; None when its chunk is not resident (nor read by its loader).
    pub(crate) fn frame_file(&self, k: usize) -> Option<Vec<u8>> {
        let chunk;
        let (qoa, &(offset, size)) = match &self.storage {
            Storage::Qoa { qoa, frames } => (qoa, frames.get(k)?),
            Storage::Chunked(c) => {
                let at = k / c.per_chunk;
                chunk = c.chunk(at).or_else(|| self.load_chunk(c, at))?;
                (&chunk.bytes, chunk.frames.get(k % c.per_chunk)?)
            }
            _ => return None,
        };
        Some(self.one_frame_file(k, &qoa[offset..offset + size]))
    }

    /// Samples per channel in QOA frame `k`.
    pub(crate) fn frame_len(&self, k: usize) -> u64 {
        (self.frames.saturating_sub(k as u64 * QOA_FRAME_LEN)).min(QOA_FRAME_LEN)
    }

    /// A QOA frame is self-contained (its own LMS state), so one frame
    /// behind a file header is a valid QOA file of its length.
    fn one_frame_file(&self, k: usize, frame: &[u8]) -> Vec<u8> {
        let len = self.frame_len(k) as u32;
        let mut file = Vec::with_capacity(8 + frame.len());
        file.extend_from_slice(&QOA_MAGIC.to_be_bytes());
        file.extend_from_slice(&len.to_be_bytes());
        file.extend_from_slice(frame);
        file
    }

    /// Decodes QOA frame `k` (interleaved samples) where it is read; None
    /// when it is not resident (a chunk the host has not put there yet).
    fn decode_frame(&self, k: usize, held: &mut Vec<(usize, Arc<Chunk>)>) -> Option<Vec<i16>> {
        let chunk;
        let (qoa, &(offset, size)) = match &self.storage {
            Storage::Qoa { qoa, frames } => (qoa, frames.get(k)?),
            Storage::Chunked(c) => {
                let at = k / c.per_chunk;
                chunk = match held.first() {
                    Some(h) if h.0 == at => h.1.clone(),
                    _ => {
                        // Entering a chunk: hold it and the next one (when
                        // resident), so a voice fading out after its
                        // playhead is gone keeps its samples while the
                        // bank drops them.
                        let this = match held.iter().find(|h| h.0 == at) {
                            Some(h) => h.1.clone(),
                            None => match c.chunk(at) {
                                Some(x) => x,
                                None => self.load_chunk(c, at)?,
                            },
                        };
                        held.clear();
                        held.push((at, this.clone()));
                        if let Some(next) = c.chunk(at + 1) {
                            held.push((at + 1, next));
                        }
                        this
                    }
                };
                (&chunk.bytes, chunk.frames.get(k % c.per_chunk)?)
            }
            _ => return None,
        };
        let file = self.one_frame_file(k, &qoa[offset..offset + size]);
        qoa::decode(&file).ok().map(|d| d.samples)
    }
}

impl SoundData {
    /// Chunk `k` of a chunked sound from its QOA frames, checked against
    /// the sound's shape.
    fn make_chunk(&self, c: &Chunked, k: usize, frames: Vec<u8>) -> Result<Arc<Chunk>> {
        if k >= c.count {
            return Err(invalid(&self.id, format!("chunk {k} of {}", c.count)));
        }
        let start = (k * c.per_chunk) as u64 * QOA_FRAME_LEN;
        let samples = (c.per_chunk as u64 * QOA_FRAME_LEN).min(self.frames - start);
        let (channels, rate, index) = index_frames(&frames, 0, samples, self.frames - start)
            .map_err(|e| invalid(&self.id, format!("chunk {k}: {e}")))?;
        if index.last().is_none_or(|&(o, n)| o + n != frames.len()) {
            return Err(invalid(
                &self.id,
                format!("chunk {k}: bytes past its frames"),
            ));
        }
        if (channels, rate) != (self.channels, self.sample_rate) {
            return Err(invalid(
                &self.id,
                format!(
                    "chunk {k} is {rate} Hz x{channels}, the sound {} Hz x{}",
                    self.sample_rate, self.channels
                ),
            ));
        }
        Ok(Arc::new(Chunk {
            bytes: frames,
            frames: index,
        }))
    }

    /// A missing chunk read through the sound's loader, and kept.
    fn load_chunk(&self, c: &Chunked, k: usize) -> Option<Arc<Chunk>> {
        let loader = c.loader.read().expect("chunk lock").clone()?;
        let made = self.make_chunk(c, k, loader(k as u32)?).ok()?;
        c.slots.write().expect("chunk lock")[k] = Some(made.clone());
        Some(made)
    }
}

/// A QOA file's frame table: channels, rate, samples per channel, and
/// (byte offset, byte size) per frame.
type QoaIndex = (usize, u32, u64, Vec<(usize, usize)>);

/// Indexes a QOA file's frames from their headers, checking each the way
/// the decoder will.
fn index_qoa(bytes: &[u8]) -> std::result::Result<QoaIndex, String> {
    let total = qoa_header(bytes)?;
    let (channels, rate, frames) = index_frames(bytes, 8, total, total)?;
    Ok((channels, rate, total, frames))
}

/// The samples per channel a QOA file's header declares.
fn qoa_header(bytes: &[u8]) -> std::result::Result<u64, String> {
    let header = bytes
        .get(0..8)
        .map(|b| u64::from_be_bytes(b.try_into().expect("8 bytes")))
        .ok_or("malformed QOA: no header")?;
    if (header >> 32) as u32 != QOA_MAGIC {
        return Err("malformed QOA: wrong magic".into());
    }
    Ok(header & 0xffff_ffff)
}

/// Channels, rate, and (byte offset, byte size) per frame.
type FrameIndex = (usize, u32, Vec<(usize, usize)>);

/// Indexes the QOA frames from byte `pos` that hold `samples` samples per
/// channel, of a file of `total` (a short frame only as the file's last).
fn index_frames(
    bytes: &[u8],
    mut pos: usize,
    samples: u64,
    total: u64,
) -> std::result::Result<FrameIndex, String> {
    let be64 = |at: usize| -> Option<u64> {
        Some(u64::from_be_bytes(bytes.get(at..at + 8)?.try_into().ok()?))
    };
    let mut seen = 0u64;
    let (mut channels, mut rate) = (0usize, 0u32);
    let mut frames = Vec::new();
    while seen < samples {
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
        if seen + len < samples && len != QOA_FRAME_LEN {
            return Err("malformed QOA: a short frame before the last".into());
        }
        frames.push((pos, size));
        pos += size;
        seen += len;
    }
    if channels == 0 {
        return Err("malformed QOA: no frames".into());
    }
    if seen != samples || (samples < total && !samples.is_multiple_of(QOA_FRAME_LEN)) {
        return Err(format!(
            "malformed QOA: frames of {seen} samples where {samples} were expected"
        ));
    }
    Ok((channels, rate, frames))
}

fn invalid(id: &str, reason: impl std::fmt::Display) -> Error {
    Error::new(ErrorKind::Invalid, format!("sound '{id}': {reason}"))
}

struct Bank {
    pool: Pool<Arc<SoundData>>,
    resident_seconds: f64,
    decoding: Arc<Decoding>,
}

impl Drop for Bank {
    fn drop(&mut self) {
        // The last clone of the bank is gone: its decode thread ends.
        self.decoding.close();
    }
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

/// Sounds up to this long stay decoded once decoded (every frame is queued
/// for the decoder at insert); longer ones stream.
pub const DEFAULT_RESIDENT_SECONDS: f64 = 6.0;

impl SoundBank {
    pub fn new() -> SoundBank {
        SoundBank::with_resident_seconds(DEFAULT_RESIDENT_SECONDS)
    }

    /// A bank that keeps sounds up to `seconds` long decoded whole and
    /// streams the rest (0 streams everything).
    pub fn with_resident_seconds(seconds: f64) -> SoundBank {
        SoundBank {
            inner: Arc::new(RwLock::new(Bank {
                pool: Pool::new(),
                resident_seconds: seconds.max(0.0),
                decoding: Arc::new(Decoding::default()),
            })),
        }
    }

    /// The bank's decoding: its queue, its in-flight jobs and counters.
    pub(crate) fn decoding(&self) -> Arc<Decoding> {
        self.inner.read().expect("sound bank lock").decoding.clone()
    }

    /// A frame cache for a QOA or chunked sound of `frames` samples.
    fn cache(&self, frames: u64, pinned: bool) -> Option<FrameCache> {
        Some(FrameCache {
            slots: Mutex::new(vec![None; frames.div_ceil(QOA_FRAME_LEN) as usize]),
            pinned,
            decoding: self.decoding(),
        })
    }

    fn put(&self, data: SoundData) -> u64 {
        let data = Arc::new(data);
        let handle = {
            let mut bank = self.inner.write().expect("sound bank lock");
            bank.pool.insert(data.clone()).to_bits()
        };
        self.decoding().pin(&data);
        handle
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
            cache: None,
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
                cache: None,
            }),
        )
    }

    fn swap(&self, handle: u64, data: Arc<SoundData>) -> bool {
        let swapped = {
            let mut bank = self.inner.write().expect("sound bank lock");
            match bank.pool.get_mut(Handle::from_bits(handle)) {
                Ok(slot) => {
                    *slot = data.clone();
                    true
                }
                Err(_) => false,
            }
        };
        if swapped {
            self.decoding().pin(&data);
        }
        swapped
    }

    /// Whether a handle's sound has samples (not a silent stand-in).
    pub fn audible(&self, handle: u64) -> bool {
        self.get(handle)
            .is_some_and(|s| !matches!(s.storage, Storage::Silent(_)))
    }

    /// A QOA file indexed, not decoded: its frames are checked against
    /// their headers here (as the decoder will read them) and decoded beside
    /// the tick (`decode.rs`).
    fn decode_qoa(&self, id: &str, bytes: &[u8]) -> Result<SoundData> {
        let (channels, sample_rate, frames, index) =
            index_qoa(bytes).map_err(|e| invalid(id, e))?;
        if sample_rate == 0 {
            return Err(invalid(id, "sample rate 0"));
        }
        let resident_seconds = self.inner.read().expect("sound bank lock").resident_seconds;
        let pinned = frames as f64 <= resident_seconds * f64::from(sample_rate);
        Ok(SoundData {
            id: id.to_string(),
            sample_rate,
            channels,
            frames,
            storage: Storage::Qoa {
                qoa: bytes.to_vec(),
                frames: index,
            },
            cache: self.cache(frames, pinned),
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
            cache: None,
        }))
    }

    /// Adds a sound cooked in chunks of `per_chunk` QOA frames (`count`
    /// chunks): `head` is a QOA file whose header declares the whole
    /// sound's length, followed by the first chunk's frames only. The
    /// other chunks arrive with [`SoundBank::put_chunk`]; until one does,
    /// its frames play as silence. Returns its handle (never 0).
    pub fn insert_chunked(&self, id: &str, head: &[u8], per_chunk: u32, count: u32) -> Result<u64> {
        let data = self.chunked_data(id, head, per_chunk, count)?;
        Ok(self.put(data))
    }

    /// Puts a sound cooked in chunks behind an existing handle (its
    /// streaming group loaded), as [`SoundBank::insert_chunked`] makes one:
    /// the head must have the handle's rate, channels and length, and
    /// voices playing the silent stand-in go on with its chunks from where
    /// they are, the ones not resident yet as silence until the host puts
    /// them ([`SoundBank::keep_chunks`] asks for them from now on). False
    /// for a handle not in the bank.
    pub fn replace_chunked(
        &self,
        handle: u64,
        id: &str,
        head: &[u8],
        per_chunk: u32,
        count: u32,
    ) -> Result<bool> {
        let Some(old) = self.get(handle) else {
            return Ok(false);
        };
        let data = self.chunked_data(id, head, per_chunk, count)?;
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

    fn chunked_data(&self, id: &str, head: &[u8], per_chunk: u32, count: u32) -> Result<SoundData> {
        let total = qoa_header(head).map_err(|e| invalid(id, e))?;
        let qoa_frames = total.div_ceil(QOA_FRAME_LEN);
        let per = u64::from(per_chunk);
        if per == 0 || u64::from(count) != qoa_frames.div_ceil(per).max(1) {
            return Err(invalid(
                id,
                format!("{count} chunks of {per_chunk} QOA frames for {qoa_frames} frames"),
            ));
        }
        let first = (per * QOA_FRAME_LEN).min(total);
        let (channels, sample_rate, frames) =
            index_frames(head, 8, first, total).map_err(|e| invalid(id, e))?;
        if sample_rate == 0 {
            return Err(invalid(id, "sample rate 0"));
        }
        let mut slots = vec![None; count as usize];
        slots[0] = Some(Arc::new(Chunk {
            bytes: head.to_vec(),
            frames,
        }));
        Ok(SoundData {
            id: id.to_string(),
            sample_rate,
            channels,
            frames: total,
            storage: Storage::Chunked(Chunked {
                per_chunk: per_chunk as usize,
                count: count as usize,
                slots: RwLock::new(slots),
                loader: RwLock::new(None),
            }),
            cache: self.cache(total, false),
        })
    }

    /// Puts chunk `chunk` of a chunked sound in place: `frames` are its QOA
    /// frames (a cooked chunk's `qoa` section). Voices reaching it decode
    /// it from then on. False for a handle not in the bank, or silent (its
    /// streaming group unloaded while the chunk was on its way); an error
    /// for a sound not chunked, a chunk past its last or frames of another
    /// shape.
    pub fn put_chunk(&self, handle: u64, chunk: u32, frames: &[u8]) -> Result<bool> {
        let Some(s) = self.get(handle) else {
            return Ok(false);
        };
        let c = match &s.storage {
            Storage::Chunked(c) => c,
            Storage::Silent(_) => return Ok(false),
            _ => return Err(invalid(&s.id, "is not chunked")),
        };
        let made = s.make_chunk(c, chunk as usize, frames.to_vec())?;
        c.slots.write().expect("chunk lock")[chunk as usize] = Some(made);
        Ok(true)
    }

    /// Gives a chunked sound a loader: a chunk a voice reaches that is not
    /// resident is read through it there and then (natively, a file read)
    /// instead of playing silence, so the mix never depends on when the
    /// host fed the bank. False for a handle that is not a chunked sound.
    pub fn set_chunk_loader(&self, handle: u64, loader: ChunkLoader) -> bool {
        match self.get(handle).as_deref().map(|s| &s.storage) {
            Some(Storage::Chunked(c)) => {
                *c.loader.write().expect("chunk lock") = Some(loader);
                true
            }
            _ => false,
        }
    }

    /// Keeps, of every chunked sound, its first chunk and the chunks
    /// `wanted` names for it ([`chunks_wanted`]), dropping the others
    /// (a voice decoding one keeps its current frame). Returns the chunks
    /// wanted but not resident, as (handle, sound id, chunk), for the host
    /// to read and [`SoundBank::put_chunk`].
    pub fn keep_chunks(&self, wanted: &IdMap<u64, BTreeSet<u32>>) -> Vec<(u64, String, u32)> {
        let sounds: Vec<(u64, Arc<SoundData>)> = {
            let bank = self.inner.read().expect("sound bank lock");
            bank.pool
                .iter()
                .filter(|(_, s)| matches!(s.storage, Storage::Chunked(_)))
                .map(|(h, s)| (h.to_bits(), s.clone()))
                .collect()
        };
        let mut missing = Vec::new();
        for (h, s) in sounds {
            let Storage::Chunked(c) = &s.storage else {
                continue;
            };
            let want = wanted.get(&h);
            let mut slots = c.slots.write().expect("chunk lock");
            for (k, slot) in slots.iter_mut().enumerate().skip(1) {
                let keep = want.is_some_and(|w| w.contains(&(k as u32)));
                if !keep {
                    if slot.take().is_some()
                        && let Some(cache) = &s.cache
                    {
                        // Its decoded frames go with it: a voice reaching
                        // them plays what a chunk not there plays.
                        for f in k * c.per_chunk..(k + 1) * c.per_chunk {
                            cache.evict(f);
                        }
                    }
                } else if slot.is_none() {
                    missing.push((h, s.id.clone(), k as u32));
                }
            }
        }
        missing
    }

    /// Bytes every sound's samples take in memory now
    /// ([`SoundData::resident_bytes`]).
    pub fn resident_bytes(&self) -> usize {
        let bank = self.inner.read().expect("sound bank lock");
        bank.pool.iter().map(|(_, s)| s.resident_bytes()).sum()
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

/// A voice's view of a sound: stereo frames by index, two QOA frames at a
/// time from the sound's frame cache, decoding one nobody decoded yet.
pub(crate) struct Reader {
    data: Arc<SoundData>,
    slots: [(usize, Arc<[i16]>); 2],
    evict: usize,
    /// A chunked sound's chunk being read and the next.
    held: Vec<(usize, Arc<Chunk>)>,
}

const EMPTY: usize = usize::MAX;
const SCALE: f32 = 1.0 / 32768.0;

impl Reader {
    pub fn new(data: Arc<SoundData>) -> Reader {
        Reader {
            data,
            slots: [(EMPTY, Arc::from([])), (EMPTY, Arc::from([]))],
            evict: 0,
            held: Vec::new(),
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
                    self.slots = [(EMPTY, Arc::from([])), (EMPTY, Arc::from([]))];
                    return self.frame(i);
                }
                None => return (0.0, 0.0),
            },
            Storage::Qoa { .. } | Storage::Chunked(_) => {
                let k = (i / QOA_FRAME_LEN) as usize;
                let off = (i % QOA_FRAME_LEN) as usize * ch;
                let slot = if self.slots[0].0 == k {
                    0
                } else if self.slots[1].0 == k {
                    1
                } else {
                    let Some(decoded) = self.load(k) else {
                        // A chunk not resident yet plays silence, and is
                        // looked for again on the next frame.
                        return (0.0, 0.0);
                    };
                    let s = self.evict;
                    self.evict ^= 1;
                    self.slots[s] = (k, decoded);
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

    /// QOA frame `k`: decoded beside the tick, or (a miss) decoded here, to
    /// the same samples; a pinned frame decoded here is kept for the
    /// others. None when its chunk is not resident.
    fn load(&mut self, k: usize) -> Option<Arc<[i16]>> {
        let cache = self.data.cache.as_ref();
        if let Some(hit) = cache.and_then(|c| c.get(k)) {
            // A chunked sound's chunk is held as a decode here would hold
            // it (a voice fading out keeps the frames after this one).
            let _ = self.hold_chunk(k);
            return Some(hit);
        }
        let decoded: Arc<[i16]> = self.data.decode_frame(k, &mut self.held)?.into();
        if let Some(c) = cache {
            c.decoding.missed();
            if c.pins(k) {
                c.put(k, decoded.clone());
            }
        }
        Some(decoded)
    }

    /// For a chunked sound: holds the chunk frame `k` is in and the next,
    /// as entering it by decoding does.
    fn hold_chunk(&mut self, k: usize) -> Option<()> {
        let Storage::Chunked(c) = &self.data.storage else {
            return None;
        };
        let at = k / c.per_chunk;
        if self.held.first().is_some_and(|h| h.0 == at) {
            return Some(());
        }
        let this = match self.held.iter().find(|h| h.0 == at) {
            Some(h) => h.1.clone(),
            None => c.chunk(at)?,
        };
        self.held.clear();
        self.held.push((at, this));
        if let Some(next) = c.chunk(at + 1) {
            self.held.push((at + 1, next));
        }
        Some(())
    }
}

/// Seconds of a chunked sound kept behind each playhead (a free-running
/// device may still be rendering them) and ahead of it beyond the chunk
/// it is in (what a host has to fetch before the playhead gets there).
pub const CHUNK_BEHIND_SECONDS: f64 = 0.5;
pub const CHUNK_AHEAD_SECONDS: f64 = 4.0;

/// The chunks each chunked sound needs now, from the world's playheads
/// (`og.AudioPlayback`): per playing voice, from [`CHUNK_BEHIND_SECONDS`]
/// behind its cursor to [`CHUNK_AHEAD_SECONDS`] ahead at its pitch, and
/// for a loop the chunk its loop start is in. The first chunk is always
/// resident and not listed. Read at a tick boundary; it never changes the
/// world. The keys of [`chunk_distances`].
pub fn chunks_wanted(world: &og_ecs::World, bank: &SoundBank) -> IdMap<u64, BTreeSet<u32>> {
    chunk_distances(world, bank)
        .into_iter()
        .map(|(h, chunks)| (h, chunks.into_keys().collect()))
        .collect()
}

/// [`chunks_wanted`] with, per chunk, how soon a playhead needs it: the
/// seconds (of the device, at the voice's pitch) until the nearest voice
/// reaches the chunk's first frame; 0 for a chunk a playhead is in or has
/// just left (a free-running device may still be rendering it); for a
/// loop's start chunk and the next, the time to the loop's wrap plus their
/// offset from the loop start. A host fetching chunks fetches the smallest
/// first (og-runtime's streaming fetches).
pub fn chunk_distances(world: &og_ecs::World, bank: &SoundBank) -> IdMap<u64, IdMap<u32, f64>> {
    let mut out: IdMap<u64, IdMap<u32, f64>> = IdMap::new();
    let Ok(rows) = world.query(&og_ecs::QueryDesc::new().read::<crate::schema::AudioPlayback>())
    else {
        return out;
    };
    let entities: Vec<og_ecs::Entity> = rows.flat_map(|a| a.entities().to_vec()).collect();
    for e in entities {
        let Ok(p) = world.get::<crate::schema::AudioPlayback>(e) else {
            continue;
        };
        let Some(s) = bank.get(p.sound) else { continue };
        let Some((per, count)) = s.chunking() else {
            continue;
        };
        let rate = f64::from(s.sample_rate.max(1));
        let pitch = f64::from(p.pitch.clamp(1.0 / 16.0, 16.0));
        // Frames of the sound a second of the device plays.
        let speed = rate * pitch;
        let lo = (p.cursor - CHUNK_BEHIND_SECONDS * speed).max(0.0);
        let hi = p.cursor.max(0.0) + CHUNK_AHEAD_SECONDS * speed;
        let last = u64::from(count - 1);
        let set = out.entry(p.sound).or_default();
        let mut want = |chunk: u64, seconds: f64| {
            let d = set.entry(chunk.min(last) as u32).or_insert(f64::INFINITY);
            *d = d.min(seconds.max(0.0));
        };
        let (a, b) = ((lo as u64 / per).min(last), (hi as u64 / per).min(last));
        for c in a..=b {
            want(c, ((c * per) as f64 - p.cursor) / speed);
        }
        if p.flags & 1 != 0 {
            // The wrap: the loop start's chunk, and the one after it when
            // the playhead is within reach of the loop's end.
            let end = if p.loop_end > 0 {
                f64::from(p.loop_end)
            } else {
                s.frames as f64
            };
            let wrap = (end - p.cursor).max(0.0) / speed;
            let start = u64::from(p.loop_start) / per;
            want(start, wrap);
            let next = ((start + 1) * per) as f64 - f64::from(p.loop_start);
            want(start + 1, wrap + next / speed);
        }
        set.remove(&0);
    }
    out
}
