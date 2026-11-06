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

use crate::soma::CONTACT;

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

/// The bound on a jump's drawn air pose at `phase` of the clip's own flight
/// (0 at its takeoff, 1 at its touchdown), [`air_pose`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AirPose {
    /// How far the leg further forward may swing back from straight below
    /// its hip, rad.
    pub lead: f32,
    /// The same for the other leg, rad.
    pub trail: f32,
    /// How far the body may lean from upright (the pelvis to the chest, the
    /// hips to the head), rad.
    pub lean: f32,
}

/// The phase of a flight its apex is at.
const APEX: f32 = 0.5;

/// A running jump's air pose (`docs/design.md` §2.7, "Jumps"): at takeoff a
/// leg may trail the push-off (45°) and the body lean into it (30°); toward
/// the apex both ease (smoothstep) to the landing's, the legs under the
/// hips (the lead straight below, the trail within 5°) and the body near
/// upright (10°), and hold there to touchdown. A generated flight that
/// pitches toward horizontal with both legs swept back reads as a dive,
/// not a jump; commercial running jumps lead with one leg and bring both
/// under the hips before touchdown.
pub fn air_pose(phase: f32) -> AirPose {
    let t = (phase / APEX).clamp(0.0, 1.0);
    let t = t * t * (3.0 - 2.0 * t);
    let lerp = |a: f32, b: f32| (a + (b - a) * t).to_radians();
    AirPose {
        lead: lerp(15.0, 0.0),
        trail: lerp(45.0, 5.0),
        lean: lerp(30.0, 10.0),
    }
}

/// How far a leg swings back from straight below its hip, rad: the angle in
/// the body's sagittal plane (forward +Z, up +Y) of `hip_to_ankle`, positive
/// behind.
pub fn swing_back(hip_to_ankle: [f32; 3]) -> f32 {
    og_core::math::atan2(-hip_to_ankle[2], -hip_to_ankle[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOWN: [f32; 4] = [1.0, 1.0, 0.0, 0.0];
    const UP: [f32; 4] = [0.0; 4];

    #[test]
    fn flight_is_the_clips_own_contacts() {
        let c = [DOWN, DOWN, UP, UP, UP, DOWN, DOWN];
        assert_eq!(
            clip_flight(&c, 0, 3),
            Some(ClipFlight {
                takeoff: 2,
                touchdown: Some(5)
            })
        );
        // Not yet generated, and never jumped.
        assert_eq!(clip_flight(&c[..5], 0, 3).unwrap().touchdown, None);
        assert_eq!(clip_flight(&c[..4], 0, 3), None);
        assert_eq!(clip_flight(&[DOWN; 5], 0, 3), None);
        // A run's short flights before the jump are strides, not the jump.
        let run = [DOWN, UP, DOWN, UP, UP, DOWN, DOWN, UP, UP, UP, UP, DOWN];
        assert_eq!(
            clip_flight(&run, 0, 3),
            Some(ClipFlight {
                takeoff: 7,
                touchdown: Some(11)
            })
        );
    }

    #[test]
    fn local_height_keeps_compression_and_removes_travel() {
        // Planted (the foot at its clearance or below it): the hips as generated.
        assert_eq!(local_height(0.72, 0.09, 0.09), 0.72);
        assert_eq!(local_height(0.72, 0.07, 0.09), 0.72);
        // In the air 0.4 m up: only the leg's extension above the foot is left.
        assert!((local_height(1.37, 0.49, 0.09) - 0.97).abs() < 1e-6);
    }

    #[test]
    fn air_phase_is_ballistic_time_and_runs_on_past_a_ledge() {
        let (v0, g) = (3.0_f32, 9.81_f32);
        let flight = 2.0 * v0 / g;
        for k in [0.0, 0.25, 0.5, 0.9] {
            let v = v0 - g * flight * k;
            assert!((air_phase(v, v0) - k).abs() < 1e-5);
        }
        // Falling faster than the takeoff speed: below the takeoff height.
        assert!(air_phase(-4.0, v0) > 1.0);
        // A ceiling that stopped the rise: the fall begins at half way.
        assert_eq!(air_phase(0.0, v0), 0.5);
    }

    #[test]
    fn retime_puts_the_clips_takeoff_and_touchdown_on_the_simulations() {
        // Takeoff at frame 10, touchdown at 22, 40 frames from frame 2.
        let f = ClipFlight {
            takeoff: 10,
            touchdown: Some(22),
        };
        let at = |t| retime(f, 2, 40, 30.0, t);
        assert_eq!(
            at(JumpTime::Prepare {
                elapsed: 0.0,
                prepare: 0.3
            }),
            2.0
        );
        assert_eq!(
            at(JumpTime::Prepare {
                elapsed: 0.15,
                prepare: 0.3
            }),
            6.0
        );
        assert_eq!(at(JumpTime::Air { phase: 0.0 }), 10.0);
        assert_eq!(at(JumpTime::Air { phase: 0.5 }), 16.0);
        // Still in the air at and past the clip's own landing: held before it.
        assert_eq!(at(JumpTime::Air { phase: 1.0 }), 21.0);
        assert_eq!(at(JumpTime::Air { phase: 1.6 }), 21.0);
        // An early landing jumps to touchdown; then the clip plays at its rate.
        assert_eq!(at(JumpTime::Landed { since: 0.0 }), 22.0);
        assert_eq!(at(JumpTime::Landed { since: 0.1 }), 25.0);
        assert_eq!(at(JumpTime::Landed { since: 9.0 }), 39.0);
        // Touchdown not generated yet: the latest frame, never a landing.
        let open = ClipFlight {
            takeoff: 10,
            touchdown: None,
        };
        let air = retime(open, 2, 18, 30.0, JumpTime::Air { phase: 0.9 });
        assert!((air - 16.3).abs() < 1e-5, "{air}");
        assert_eq!(
            retime(open, 2, 18, 30.0, JumpTime::Air { phase: 1.4 }),
            17.0
        );
        assert_eq!(
            retime(open, 2, 18, 30.0, JumpTime::Landed { since: 0.2 }),
            17.0
        );
    }
}
