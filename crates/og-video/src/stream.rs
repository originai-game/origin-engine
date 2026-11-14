//! A stream's frames decoded ahead of the frame its player is at: the
//! decode plan ([`plan`], the same on every host) and, natively, a decode
//! thread per stream ([`Player`]).
//!
//! The player names the frame it wants ([`Player::want`], from the world's
//! VideoPlayer time each tick); the thread decodes from where it is, or
//! from the key frame at or before the wanted frame when that is behind it
//! or past the next key frame, and keeps up to [`AHEAD`] frames ready. An
//! executor ([`Player::take`] with `wait`) blocks until the wanted frame is
//! ready, so a capture shows exactly the frame of its tick. A stream's
//! units are held by chunk ([`Units`]): [`chunks_wanted`] names the chunks
//! a player needs, the same on every host, and the plan waits for a unit
//! whose chunk has not landed.

use crate::ivf::Unit;

/// Frames decoded ahead of the one wanted.
pub const AHEAD: usize = 8;

/// The largest stream the web's software fallback (rav1d in wasm, one
/// thread) decodes in real time at 30 fps: measured in Safari 27 on Apple
/// silicon, 720p held 46 fps on worst-case footage and 1080p 19.
pub const FALLBACK_MAX: (u32, u32) = (1280, 720);

/// The size of the rendition the cook adds beside a stream larger than
/// [`FALLBACK_MAX`] in either orientation (a portrait 720x1280 fits: the
/// decoder's cost is its pixels), its aspect kept, even sides, at least 64:
/// what the web's fallback decoder plays instead. `None` for a stream that
/// fits. The cook and the runtime both derive it from the stream's size.
pub fn fallback_size((w, h): (u32, u32)) -> Option<(u32, u32)> {
    let (mw, mh) = if w >= h {
        FALLBACK_MAX
    } else {
        (FALLBACK_MAX.1, FALLBACK_MAX.0)
    };
    if w <= mw && h <= mh {
        return None;
    }
    let scale = (f64::from(mw) / f64::from(w)).min(f64::from(mh) / f64::from(h));
    let side = |v: u32| (((f64::from(v) * scale).round() as u32) & !1).max(64);
    Some((side(w), side(h)))
}

/// Seconds of footage a playing player's next chunks are fetched ahead of
/// it (as a chunked sound's are, og-audio's `chunks_wanted`).
pub const LEAD: f64 = 4.0;

/// One chunk's units, its first frame's first.
type Chunk = std::sync::Arc<Vec<Vec<u8>>>;

/// A stream's units, frame `n` its unit `n`, held by chunk: chunk `c` is
/// frames `c · chunk_frames ..` and starts on a key frame. Its chunks land
/// and go while it plays ([`Units::put`], [`Units::drop_chunk`]); the decode
/// plan waits for a unit whose chunk is not here.
#[derive(Debug)]
pub struct Units {
    /// Per frame: whether it is a key frame.
    key: Vec<bool>,
    /// The key frames, ascending (frame 0 first).
    pub keys: Vec<u32>,
    /// Frames in each chunk but the last.
    pub chunk_frames: u32,
    /// Per chunk, its frames' units while it is here.
    chunks: std::sync::Mutex<Vec<Option<Chunk>>>,
}

impl Units {
    /// A stream's shape, no chunk here yet: `key` per frame, its chunks
    /// `chunk_frames` frames each (every one starting on a key frame).
    pub fn new(key: Vec<bool>, chunk_frames: u32) -> Units {
        let keys = key
            .iter()
            .enumerate()
            .filter(|(_, k)| **k)
            .map(|(i, _)| i as u32)
            .collect();
        let chunk_frames = chunk_frames.max(1);
        let n = (key.len() as u32).div_ceil(chunk_frames) as usize;
        Units {
            key,
            keys,
            chunk_frames,
            chunks: std::sync::Mutex::new(vec![None; n]),
        }
    }

    /// Every unit at once, as one chunk (a stream read whole: tests, tools).
    pub fn whole(units: Vec<Unit>) -> Units {
        let u = Units::new(units.iter().map(|u| u.key).collect(), units.len() as u32);
        if !units.is_empty() {
            u.put(0, units.into_iter().map(|u| u.data).collect());
        }
        u
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Vec<Option<Chunk>>> {
        self.chunks.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// The key frame at or before `frame`.
    pub fn key_at(&self, frame: u32) -> u32 {
        let i = self.keys.partition_point(|&k| k <= frame);
        self.keys[i.saturating_sub(1)]
    }

    pub fn len(&self) -> u32 {
        self.key.len() as u32
    }

    pub fn is_empty(&self) -> bool {
        self.key.is_empty()
    }

    /// Its chunks.
    pub fn chunks(&self) -> u32 {
        self.len().div_ceil(self.chunk_frames)
    }

    /// The chunk that holds `frame`.
    pub fn chunk_of(&self, frame: u32) -> u32 {
        frame / self.chunk_frames
    }

    /// Chunk `n` landed: its frames' units, in order.
    pub fn put(&self, n: u32, units: Vec<Vec<u8>>) {
        if let Some(c) = self.lock().get_mut(n as usize) {
            *c = Some(std::sync::Arc::new(units));
        }
    }

    /// Chunk `n` is let go (no player wants it).
    pub fn drop_chunk(&self, n: u32) {
        if let Some(c) = self.lock().get_mut(n as usize) {
            *c = None;
        }
    }

    /// Whether chunk `n` is here.
    pub fn resident(&self, n: u32) -> bool {
        self.lock().get(n as usize).is_some_and(Option::is_some)
    }

    /// Whether `frame`'s unit is here.
    pub fn has(&self, frame: u32) -> bool {
        frame < self.len() && self.resident(self.chunk_of(frame))
    }

    /// `frame`'s unit while its chunk is here: the chunk and its place in it.
    pub fn unit(&self, frame: u32) -> Option<(std::sync::Arc<Vec<Vec<u8>>>, usize)> {
        let c = self.chunk_of(frame);
        let chunk = self.lock().get(c as usize)?.clone()?;
        let at = (frame - c * self.chunk_frames) as usize;
        (at < chunk.len()).then_some((chunk, at))
    }
}

/// The chunks a player at `frame` needs, each with the seconds until it
/// does (0: now): its own chunk, and while it plays the next ones that
/// start within [`LEAD`] seconds of it (past the last frame, a looping
/// player's go on from chunk 0). A paused player needs only its own: a
/// player paused at a segment's start is how its first frame is decoded
/// before it plays.
pub fn chunks_wanted(
    units: &Units,
    fps: u32,
    frame: u32,
    playing: bool,
    speed: f32,
    looped: bool,
) -> Vec<(u32, f64)> {
    let n = units.len();
    if n == 0 {
        return Vec::new();
    }
    let frame = frame.min(n - 1);
    let mut out = vec![(units.chunk_of(frame), 0.0)];
    if !playing || speed <= 0.0 || fps == 0 {
        return out;
    }
    let rate = f64::from(fps) * f64::from(speed);
    let mut start = (units.chunk_of(frame) + 1) * units.chunk_frames;
    let mut offset = 0u32;
    while out.len() < units.chunks() as usize {
        if start >= n {
            if !looped {
                break;
            }
            offset += n;
            start = 0;
        }
        let seconds = f64::from(start + offset - frame) / rate;
        if seconds > LEAD {
            break;
        }
        let c = units.chunk_of(start);
        if out.iter().any(|(k, _)| *k == c) {
            break;
        }
        out.push((c, seconds));
        start = (c + 1) * units.chunk_frames;
    }
    out
}

/// What the decoder does next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// Decode unit `pos` and go on.
    Decode,
    /// Flush and start again at this key frame.
    Seek(u32),
    /// Nothing until the wanted frame moves.
    Idle,
}

/// The next step for a decoder whose next unit is `pos`, holding `ready`
/// frames from `first` on (in decode order), when `want` is wanted.
/// `looped`: past the last frame the stream goes on at frame 0. A step
/// whose unit's chunk has not landed waits ([`Step::Idle`]).
pub fn plan(units: &Units, pos: u32, ready: &[u32], want: u32, looped: bool) -> Step {
    let n = units.len();
    let want = want.min(n - 1);
    if ready.contains(&want) {
        if ready.len() >= AHEAD || (pos >= n && !looped) {
            return Step::Idle;
        }
        if pos >= n {
            return wait_for(units, 0, Step::Seek(0));
        }
        return wait_for(units, pos, Step::Decode);
    }
    // Behind the decoder, or a key frame between it and the wanted frame:
    // start at that key frame.
    let key = units.key_at(want);
    if want < pos || key > pos {
        return wait_for(units, key, Step::Seek(key));
    }
    wait_for(units, pos, Step::Decode)
}

/// `step`, or [`Step::Idle`] while unit `frame`'s chunk has not landed.
fn wait_for(units: &Units, frame: u32, step: Step) -> Step {
    if units.has(frame) { step } else { Step::Idle }
}

#[cfg(all(feature = "decode", not(target_arch = "wasm32")))]
pub use native::Player;

#[cfg(all(feature = "decode", not(target_arch = "wasm32")))]
mod native {
    use super::*;
    use crate::decode::{Decoder, Picture};
    use std::collections::VecDeque;
    use std::sync::{Arc, Condvar, Mutex};

    struct Shared {
        want: u32,
        looped: bool,
        /// Decoded frames in decode order.
        ready: VecDeque<(u32, Arc<Picture>)>,
        /// Bumped by every `want` that dropped the ready frames, so a
        /// frame decoded for an older plan is not kept.
        generation: u64,
        error: Option<String>,
        quit: bool,
    }

    /// A stream's decode thread.
    pub struct Player {
        shared: Arc<(Mutex<Shared>, Condvar)>,
        thread: Option<std::thread::JoinHandle<()>>,
    }

    impl Player {
        /// Starts decoding `units` from frame 0 on `threads` decoder
        /// threads (0: rav1d's choice).
        pub fn new(units: Arc<Units>, threads: u32) -> Result<Player, String> {
            if units.is_empty() {
                return Err("a stream with no frames".into());
            }
            let decoder = Decoder::new(threads)?;
            let shared = Arc::new((
                Mutex::new(Shared {
                    want: 0,
                    looped: false,
                    ready: VecDeque::new(),
                    generation: 0,
                    error: None,
                    quit: false,
                }),
                Condvar::new(),
            ));
            let s = shared.clone();
            let thread = std::thread::Builder::new()
                .name("og-video".into())
                .spawn(move || run(&units, decoder, &s))
                .map_err(|e| format!("cannot start a video decode thread: {e}"))?;
            Ok(Player {
                shared,
                thread: Some(thread),
            })
        }

        /// A chunk landed: a decoder waiting for it goes on.
        pub fn wake(&self) {
            let (m, cv) = &*self.shared;
            drop(m.lock().unwrap_or_else(|e| e.into_inner()));
            cv.notify_all();
        }

        /// The frame the player is at (each tick); `looped` decodes on into
        /// frame 0 past the last.
        pub fn want(&self, frame: u32, looped: bool) {
            let (m, cv) = &*self.shared;
            let mut s = m.lock().unwrap_or_else(|e| e.into_inner());
            if s.want == frame && s.looped == looped {
                return;
            }
            s.want = frame;
            s.looped = looped;
            match s.ready.iter().position(|(f, _)| *f == frame) {
                Some(i) => {
                    s.ready.drain(..i);
                }
                None => {
                    s.ready.clear();
                    s.generation += 1;
                }
            }
            cv.notify_all();
        }

        /// The wanted frame's picture once decoded; `wait` blocks until it
        /// is (an executor), otherwise `None` while it is not (the picture
        /// shown stays).
        pub fn take(&self, frame: u32, wait: bool) -> Result<Option<Arc<Picture>>, String> {
            let (m, cv) = &*self.shared;
            let mut s = m.lock().unwrap_or_else(|e| e.into_inner());
            loop {
                if let Some(e) = &s.error {
                    return Err(e.clone());
                }
                if let Some((_, p)) = s.ready.iter().find(|(f, _)| *f == frame) {
                    return Ok(Some(p.clone()));
                }
                if !wait || s.want != frame {
                    return Ok(None);
                }
                s = cv.wait(s).unwrap_or_else(|e| e.into_inner());
            }
        }
    }

    impl Drop for Player {
        fn drop(&mut self) {
            {
                let (m, cv) = &*self.shared;
                m.lock().unwrap_or_else(|e| e.into_inner()).quit = true;
                cv.notify_all();
            }
            if let Some(t) = self.thread.take() {
                let _ = t.join();
            }
        }
    }

    fn run(units: &Units, mut decoder: Decoder, shared: &(Mutex<Shared>, Condvar)) {
        let (m, cv) = shared;
        let mut pos = 0u32;
        // Frames decoded before this one are not kept (a seek's lead-in).
        let mut keep_from = 0u32;
        let mut generation = 0u64;
        loop {
            let (step, want, gen_now) = {
                let mut s = m.lock().unwrap_or_else(|e| e.into_inner());
                loop {
                    if s.quit {
                        return;
                    }
                    if s.error.is_none() {
                        let ready: Vec<u32> = s.ready.iter().map(|(f, _)| *f).collect();
                        let step = plan(units, pos, &ready, s.want, s.looped);
                        if step != Step::Idle {
                            break (step, s.want.min(units.len() - 1), s.generation);
                        }
                    }
                    s = cv.wait(s).unwrap_or_else(|e| e.into_inner());
                }
            };
            if gen_now != generation {
                generation = gen_now;
                keep_from = want;
            }
            if let Step::Seek(k) = step {
                // A loop's wrap keeps everything from frame 0 on; a seek
                // only from the wanted frame on.
                keep_from = if pos >= units.len() { 0 } else { want };
                decoder.flush();
                pos = k;
            }
            let frame = pos;
            // Let go of between the plan and here: wait for it again.
            let Some((chunk, at)) = units.unit(frame) else {
                continue;
            };
            let result = decoder.decode(&chunk[at]);
            pos += 1;
            let mut s = m.lock().unwrap_or_else(|e| e.into_inner());
            match result {
                Err(e) => s.error = Some(format!("frame {frame}: {e}")),
                Ok(Some(p)) if s.generation == generation && frame >= keep_from => {
                    s.ready.push_back((frame, Arc::new(p)));
                }
                Ok(_) => {}
            }
            cv.notify_all();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn units(n: u32, every: u32) -> Units {
        Units::whole(
            (0..n)
                .map(|i| Unit {
                    pts: u64::from(i),
                    key: i % every == 0,
                    data: Vec::new(),
                })
                .collect(),
        )
    }

    /// 300 frames at 30 fps, a key frame every 30, chunks of 60 (2 s),
    /// none landed.
    fn chunked() -> Units {
        Units::new((0..300).map(|i| i % 30 == 0).collect(), 60)
    }

    #[test]
    fn the_plan_waits_for_a_units_chunk_and_goes_on_once_it_lands() {
        let u = chunked();
        u.put(0, vec![vec![1]; 60]);
        // Inside the landed chunk: decode, or seek to its key frame.
        assert_eq!(plan(&u, 12, &[], 20, false), Step::Decode);
        assert_eq!(plan(&u, 50, &[], 40, false), Step::Seek(30));
        // The key frame to seek to is in chunk 1, not landed: wait.
        assert_eq!(plan(&u, 12, &[], 75, false), Step::Idle);
        // Decoding on into chunk 1 from the last frame of chunk 0: wait.
        assert_eq!(plan(&u, 60, &[59], 59, false), Step::Idle);
        u.put(1, vec![vec![1]; 60]);
        assert_eq!(plan(&u, 12, &[], 75, false), Step::Seek(60));
        assert_eq!(plan(&u, 60, &[59], 59, false), Step::Decode);
        // A chunk let go is waited for again; a looping wrap waits for chunk 0.
        u.drop_chunk(0);
        assert_eq!(plan(&u, 50, &[], 40, false), Step::Idle);
        assert!(u.unit(10).is_none() && u.unit(70).is_some_and(|(_, at)| at == 10));
    }

    #[test]
    fn a_player_wants_its_chunk_and_those_starting_within_the_lead() {
        let u = chunked();
        let at = |w: Vec<(u32, f64)>| -> Vec<(u32, i64)> {
            w.into_iter()
                .map(|(c, s)| (c, (s * 1000.0).round() as i64))
                .collect()
        };
        // Frame 50 at 1x: chunk 1 in 10 frames, chunk 2 in 70; chunk 3 is
        // 130 frames (4.33 s) away, past the lead.
        assert_eq!(
            at(chunks_wanted(&u, 30, 50, true, 1.0, false)),
            vec![(0, 0), (1, 333), (2, 2333)]
        );
        // At 2x the same footage is half as far: up to the last chunk.
        assert_eq!(
            at(chunks_wanted(&u, 30, 50, true, 2.0, false)),
            vec![(0, 0), (1, 167), (2, 1167), (3, 2167), (4, 3167)]
        );
        // Near the end a looping player goes on from chunk 0; one that
        // does not loop wants nothing past its last.
        assert_eq!(
            at(chunks_wanted(&u, 30, 290, true, 1.0, true)),
            vec![(4, 0), (0, 333), (1, 2333)]
        );
        assert_eq!(
            at(chunks_wanted(&u, 30, 290, true, 1.0, false)),
            vec![(4, 0)]
        );
        // Paused (or at speed 0): only its own, the frame it shows.
        assert_eq!(
            at(chunks_wanted(&u, 30, 130, false, 1.0, true)),
            vec![(2, 0)]
        );
        assert_eq!(
            at(chunks_wanted(&u, 30, 130, true, 0.0, false)),
            vec![(2, 0)]
        );
        // A time past the end wants the last frame's chunk.
        assert_eq!(
            at(chunks_wanted(&u, 30, 900, false, 1.0, false)),
            vec![(4, 0)]
        );
    }

    #[test]
    fn a_rendition_fits_the_fallback_keeping_the_aspect() {
        assert_eq!(fallback_size((1280, 720)), None);
        assert_eq!(fallback_size((1920, 1080)), Some((1280, 720)));
        assert_eq!(fallback_size((3840, 1600)), Some((1280, 532)));
        assert_eq!(fallback_size((1080, 1920)), Some((720, 1280)));
        assert_eq!(fallback_size((720, 1280)), None);
        assert_eq!(fallback_size((1282, 720)), Some((1280, 718)));
        assert_eq!(fallback_size((960, 960)), Some((720, 720)));
    }

    #[test]
    fn the_plan_decodes_on_seeks_to_key_frames_and_waits_when_ahead() {
        let u = units(100, 30);
        assert_eq!(u.key_at(59), 30);
        assert_eq!(u.key_at(60), 60);
        // From the start: decode.
        assert_eq!(plan(&u, 0, &[], 0, false), Step::Decode);
        // The wanted frame is ready and fewer than AHEAD are: decode on.
        assert_eq!(plan(&u, 5, &[3, 4], 3, false), Step::Decode);
        // AHEAD ready: idle.
        let full: Vec<u32> = (3..3 + AHEAD as u32).collect();
        assert_eq!(plan(&u, 11, &full, 3, false), Step::Idle);
        // Behind the decoder: the key frame at or before it.
        assert_eq!(plan(&u, 50, &[], 40, false), Step::Seek(30));
        // A key frame between the decoder and the wanted frame: skip to it.
        assert_eq!(plan(&u, 12, &[], 75, false), Step::Seek(60));
        // Ahead but before the next key frame: decode through.
        assert_eq!(plan(&u, 12, &[], 20, false), Step::Decode);
        // Past the end: idle without loop, wrap to 0 with it.
        assert_eq!(plan(&u, 100, &[99], 99, false), Step::Idle);
        assert_eq!(plan(&u, 100, &[99], 99, true), Step::Seek(0));
        // A time past the last frame wants the last.
        assert_eq!(plan(&u, 100, &[99], 500, false), Step::Idle);
    }
}
