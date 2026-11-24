//! A generated jump drawn on the simulation's flight (`docs/design.md`
//! §2.7, "Jumps"): the simulation owns the elevation, the model the pose.
//!
//! Two pure pieces, the same for every rig and for the presenter and the
//! offline checks:
//!
//! - [`local_height`]: a generated pose's hips height with its own vertical
//!   travel removed, the hips over the lowest foot's ground clearance. It
//!   is the generated height whenever a foot is down (crouch and landing
//!   compression stay), and in flight puts the lowest foot on the
//!   simulation's elevation, so the feet never pass below what the
//!   controller cleared and reach the ground on its landing tick.
//! - [`retime`]: which generated frame to draw at a moment of the
//!   simulation's jump, so the clip's own takeoff and touchdown (its
//!   contact output, [`ClipFlight`]) fall on the simulation's (distance
//!   matching): the frames before takeoff spread over the preparation, the
//!   flight frames follow the simulation's rise and fall ([`air_phase`]),
//!   held below touchdown until the simulation lands, the touchdown frames
//!   from the landing on.
//!
//! The decomposition and retiming are motion warping's (Witkin and
//! Popović, "Motion Warping", SIGGRAPH 1995): a time warp of the clip and
//! a displacement of one channel, here the hips height. Reimplemented from
//! the paper; no code is ported.

use crate::motion::soma::CONTACT;

/// A generated clip's own flight, by its contacts.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClipFlight {
    /// The first frame with no foot down.
    pub takeoff: usize,
    /// The first frame after `takeoff` with a foot down again, when the
    /// clip has generated it.
    pub touchdown: Option<usize>,
}

/// The clip's jump: its first run of at least `min_frames` frames with no
/// contact on, from `from` on (a running stride's short flights are not a
/// jump), to the next frame with one on. A run still open at the clip's end
/// counts once it is `min_frames` long. `None` when there is none (the model
/// has not jumped, or not yet).
pub fn clip_flight(contacts: &[[f32; 4]], from: usize, min_frames: usize) -> Option<ClipFlight> {
    let down = |c: &[f32; 4]| c.iter().any(|&x| x > CONTACT);
    let mut f = from;
    while f < contacts.len() {
        if down(&contacts[f]) {
            f += 1;
            continue;
        }
        let touchdown = (f + 1..contacts.len()).find(|&g| down(&contacts[g]));
        if touchdown.unwrap_or(contacts.len()) - f >= min_frames.max(1) {
            return Some(ClipFlight {
                takeoff: f,
                touchdown,
            });
        }
        f = touchdown?;
    }
    None
}

/// The hips height of a generated pose with its own vertical travel
/// removed: `hips_y` less how far the lowest foot (`lowest_foot_y`) is above
/// `clearance`, that foot's height when it stands on the ground.
pub fn local_height(hips_y: f32, lowest_foot_y: f32, clearance: f32) -> f32 {
    hips_y - (lowest_foot_y - clearance).max(0.0)
}

/// How far through its flight a ballistic body is, 0 at takeoff to 1 back
/// at its takeoff height, from its vertical speed `v` and its takeoff speed
/// `v0` (> 0): (v0 − v) / (2 v0). It is the fraction of flight time under
/// constant gravity whatever the gravity, and stays correct where time is
/// not: a ceiling that stops the rise moves it on to the fall, and a fall
/// past the takeoff height (off a ledge) runs past 1.
pub fn air_phase(v: f32, v0: f32) -> f32 {
    if v0 <= 0.0 {
        return 1.0;
    }
    ((v0 - v) / (2.0 * v0)).max(0.0)
}

/// Where the simulation is in its jump.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum JumpTime {
    /// Before takeoff: `elapsed` of `prepare` seconds.
    Prepare { elapsed: f32, prepare: f32 },
    /// In the air: [`air_phase`].
    Air { phase: f32 },
    /// `since` seconds after the landing tick.
    Landed { since: f32 },
}

/// The generated frame, fractional, to draw at `time` of the simulation's
/// jump: `flight` is the clip's, `start` its first frame of the jump, `frames`
/// how many it has, `rate` its frames a second. In the air the frame stays
/// below touchdown (or, before the touchdown is generated, at the last
/// generated frame, the flight then spread over the frames there are)
/// until the simulation lands: a flight never draws a landing the
/// simulation has not made.
pub fn retime(flight: ClipFlight, start: usize, frames: usize, rate: f32, time: JumpTime) -> f32 {
    let last = frames.saturating_sub(1) as f32;
    let takeoff = flight.takeoff as f32;
    let before = flight
        .touchdown
        .map_or(last, |t| (t as f32 - 1.0).max(takeoff));
    let frame = match time {
        JumpTime::Prepare { elapsed, prepare } => {
            let k = if prepare > 0.0 {
                (elapsed / prepare).clamp(0.0, 1.0)
            } else {
                1.0
            };
            start as f32 + (takeoff - start as f32) * k
        }
        JumpTime::Air { phase } => {
            let end = flight.touchdown.map_or(last, |t| t as f32);
            (takeoff + (end - takeoff) * phase).min(before)
        }
        JumpTime::Landed { since } => match flight.touchdown {
            Some(t) => t as f32 + since.max(0.0) * rate,
            None => last,
        },
    };
    frame.clamp(0.0, last)
}
