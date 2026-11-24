//! The presenting host's frames (the IDL's `Present` members): every host
//! that shows the engine's frames on a display (the web Worker's canvas,
//! the desktop shell's window) hands the engine each display frame's
//! timestamp, and the engine keeps the duties a display asks for, alike on
//! every host:
//!
//! - **Pacing.** The frame cap (og.render.setDisplay, `max_fps`) draws only
//!   some display frames: 120 Hz on ProMotion and 144 or more on gaming
//!   monitors, while a cap of 60 or 30 draws every other or every fourth.
//!   The fixed tick is unchanged: a drawn frame is handed the whole time
//!   since the last drawn one, so the accumulator runs the ticks it owes.
//!   A host without a display callback waits until the answer's `nextMs`
//!   (winit's `ControlFlow::WaitUntil`).
//! - **The display's interval**, automatic quality's budget where the cap
//!   is the display's rate: the median of the display frames' intervals,
//!   unless the host read the display's refresh itself
//!   (`og_present_refresh`, a window's monitor).
//! - **The frame-time record**: a drawn frame's interval is the host's (its
//!   timestamps), which `author/perf.rs` takes.
//! - **Fullscreen**: the frame answer's `fullscreen` is the player's ask,
//!   which the host carries out and reports back (`og_present_fullscreen`).
//!
//! The locale is the host's too, named once at `og_engine_create`.

use crate::engine::engine;
use serde_json::json;

/// Display intervals the measurement keeps: about half a second at 120 Hz.
const WINDOW: usize = 64;
/// Intervals measured before the median is the display's.
const ENOUGH: usize = 8;
/// An interval this long is not the display's (a hidden page, a stall).
const LONGEST_MS: f64 = 1000.0 / 20.0;

/// Which display frames are drawn, and the display's interval.
#[derive(Clone, Debug)]
pub(crate) struct Pacer {
    intervals: [f64; WINDOW],
    measured: usize,
    /// The host timestamps of the last display frame and the last drawn
    /// one; 0 before the first (or after a resume).
    last_display: f64,
    last_drawn: f64,
    /// The median of the intervals, ms; 0 before [`ENOUGH`] of them.
    median_ms: f64,
    /// The display's interval the host read from the display, ms (0:
    /// none told).
    told_ms: f64,
}

impl Default for Pacer {
    fn default() -> Pacer {
        Pacer {
            intervals: [0.0; WINDOW],
            measured: 0,
            last_display: 0.0,
            last_drawn: 0.0,
            median_ms: 0.0,
            told_ms: 0.0,
        }
    }
}

impl Pacer {
    /// The display's interval, ms: the host's reading of the display, or
    /// the measured median; 0 while unknown.
    pub fn display_ms(&self) -> f64 {
        if self.told_ms > 0.0 {
            self.told_ms
        } else {
            self.median_ms
        }
    }

    /// The display's refresh as the host read it, Hz (0 or less: unknown).
    pub fn tell(&mut self, hz: f64) {
        self.told_ms = if hz.is_finite() && hz > 0.0 {
            1000.0 / hz
        } else {
            0.0
        };
    }

    /// The slack a drawn frame is let in early by: half a display interval,
    /// so a 60 cap on a 120 Hz display draws every other frame without
    /// drifting into a skipped pair by jitter; before the display is known,
    /// less than any display's interval.
    fn slack(&self) -> f64 {
        match self.display_ms() {
            d if d > 0.0 => d * 0.5,
            _ => 2.0,
        }
    }

    /// One display frame at `now` (the host's timestamp): under a cap of
    /// `max_fps` frames a second (0: every frame), the time since the last
    /// drawn frame when this one is drawn (0 for the first), or `None`
    /// when it is skipped. A cap at or above the display's rate draws
    /// every frame.
    pub fn frame(&mut self, now: f64, max_fps: u32) -> Option<f64> {
        if self.last_display > 0.0 {
            let d = now - self.last_display;
            if d > 0.0 && d < LONGEST_MS {
                self.intervals[self.measured % WINDOW] = d;
                self.measured += 1;
                let n = self.measured.min(WINDOW);
                if n >= ENOUGH {
                    let mut sorted = self.intervals;
                    let sorted = &mut sorted[..n];
                    sorted.sort_by(f64::total_cmp);
                    self.median_ms = sorted[n / 2];
                }
            }
        }
        self.last_display = now;
        if self.last_drawn > 0.0
            && max_fps > 0
            && now - self.last_drawn + self.slack() < 1000.0 / f64::from(max_fps)
        {
            return None;
        }
        let since = if self.last_drawn > 0.0 {
            now - self.last_drawn
        } else {
            0.0
        };
        self.last_drawn = now;
        Some(since)
    }

    /// The host time the next frame to draw is due under `max_fps`: now
    /// (the last display frame's time) without a cap.
    pub fn next_ms(&self, max_fps: u32) -> f64 {
        if max_fps == 0 || self.last_drawn == 0.0 {
            return self.last_display;
        }
        (self.last_drawn + 1000.0 / f64::from(max_fps) - self.slack()).max(self.last_display)
    }

    /// The clock stopped or went on: the next frame draws, handed no time
    /// from before, and the gap is no display interval.
    pub fn resume(&mut self) {
        self.last_drawn = 0.0;
        self.last_display = 0.0;
    }
}

/// `og_present_frame`.
pub fn frame(id: u32, now_ms: f64) -> String {
    let eng = match engine(id) {
        Ok(e) => e,
        Err(e) => return crate::refuse(e),
    };
    let (drawn, cap) = {
        let mut rs = eng.render.borrow_mut();
        // Only a display's frames are capped: a script's or a recording's
        // are drawn as the host steps them.
        let cap = if rs.display.kind.displayed() {
            super::display::max_fps(&rs)
        } else {
            0
        };
        (rs.display.pacer.frame(now_ms, cap), cap)
    };
    drop(eng);
    let mut answer = match drawn {
        Some(since) => {
            let mut a = crate::engine::frame_value(id, since, Some(since));
            a["drawn"] = json!(true);
            a
        }
        None => json!({ "ok": true, "drawn": false }),
    };
    if let Ok(eng) = engine(id) {
        answer["nextMs"] = json!(eng.render.borrow().display.pacer.next_ms(cap));
    }
    answer.to_string()
}

/// `og_present_resume`.
pub fn resume(id: u32) {
    if let Ok(eng) = engine(id)
        && let Ok(mut rs) = eng.render.try_borrow_mut()
    {
        rs.display.pacer.resume();
    }
}

/// `og_present_refresh`.
pub fn refresh(id: u32, hz: f64) {
    if let Ok(eng) = engine(id)
        && let Ok(mut rs) = eng.render.try_borrow_mut()
    {
        rs.display.pacer.tell(hz);
    }
}
