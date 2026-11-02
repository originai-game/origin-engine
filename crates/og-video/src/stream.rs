//! A stream's frames decoded ahead of the frame its player is at: the
//! decode plan ([`plan`], the same on every host) and, natively, a decode
//! thread per stream ([`Player`]).
//!
//! The player names the frame it wants ([`Player::want`], from the world's
//! VideoPlayer time each tick); the thread decodes from where it is, or
//! from the key frame at or before the wanted frame when that is behind it
//! or past the next key frame, and keeps up to [`AHEAD`] frames ready. An
//! executor ([`Player::take`] with `wait`) blocks until the wanted frame is
//! ready, so a capture shows exactly the frame of its tick.

use crate::ivf::Unit;

/// Frames decoded ahead of the one wanted.
pub const AHEAD: usize = 8;

/// A stream's units, frame `n` its unit `n`.
#[derive(Debug)]
pub struct Units {
    pub units: Vec<Unit>,
    /// The key frames, ascending (frame 0 first).
    pub keys: Vec<u32>,
}

impl Units {
    pub fn new(units: Vec<Unit>) -> Units {
        let keys = units
            .iter()
            .enumerate()
            .filter(|(_, u)| u.key)
            .map(|(i, _)| i as u32)
            .collect();
        Units { units, keys }
    }

    /// The key frame at or before `frame`.
    pub fn key_at(&self, frame: u32) -> u32 {
        let i = self.keys.partition_point(|&k| k <= frame);
        self.keys[i.saturating_sub(1)]
    }

    pub fn len(&self) -> u32 {
        self.units.len() as u32
    }

    pub fn is_empty(&self) -> bool {
        self.units.is_empty()
    }
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
/// `looped`: past the last frame the stream goes on at frame 0.
pub fn plan(units: &Units, pos: u32, ready: &[u32], want: u32, looped: bool) -> Step {
    let n = units.len();
    let want = want.min(n - 1);
    if ready.contains(&want) {
        if ready.len() >= AHEAD || (pos >= n && !looped) {
            return Step::Idle;
        }
        if pos >= n {
            return Step::Seek(0);
        }
        return Step::Decode;
    }
    // Behind the decoder, or a key frame between it and the wanted frame:
    // start at that key frame.
    let key = units.key_at(want);
    if want < pos || key > pos {
        return Step::Seek(key);
    }
    Step::Decode
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
            let result = decoder.decode(&units.units[frame as usize].data);
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
        Units::new(
            (0..n)
                .map(|i| Unit {
                    pts: u64::from(i),
                    key: i % every == 0,
                    data: Vec::new(),
                })
                .collect(),
        )
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
