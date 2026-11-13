//! A jump's clip, fitted to the simulation's takeoff, flight and landing.

use super::*;

/// The library's jump pose.
pub(crate) const JUMP: usize = 3;
/// A jump clip's drawn frame catches up with its retimed one (an early
/// landing, frames that arrived late) at most this many times the clip's rate.
pub(crate) const CATCH_UP: f32 = 3.0;

/// A jump's shortest flight, as a share of the simulation's: a clip's
/// shorter runs with no foot down are strides, not the jump.
pub(crate) const MIN_FLIGHT: f32 = 0.4;

/// While the simulation's current goal is a jump (`docs/design.md` §2.7,
/// "Jumps"): the generated frames from the jump's start on are held in the
/// agent's clip, and the frame drawn is the one [`flight::retime`] puts at
/// the simulation's moment of the jump, reached at most [`CATCH_UP`] times
/// the clip's rate. Until the clip has jumped it plays at its own rate. When
/// the goal ends, the frames past the drawn one play on from `pending`.
pub(crate) fn jump_clip(
    motion: &mut Motion,
    s: &mut Shown,
    agent: Option<&super::Agent>,
    elapsed: f32,
) {
    let Some(c) = agent
        .and_then(|a| a.current)
        .filter(|c| c.goal.kind == Kind::Jump)
    else {
        if let Some(clip) = s.clip.take() {
            let k = clip.drawn.floor() as usize;
            s.prev = clip.frames.get(k).copied().or(s.prev);
            s.next = clip.frames.get(k + 1).copied();
            s.phase = clip.drawn - k as f32;
            let rest: Vec<Frame> = clip.frames.into_iter().skip(k + 2).collect();
            for f in rest.into_iter().rev() {
                s.pending.push_front(f);
            }
        }
        return;
    };
    let started = s.clip.is_none();
    if started {
        let Some(prev) = s.prev else { return };
        s.clip = Some(Clip {
            frames: std::iter::once(prev).chain(s.next).collect(),
            drawn: s.phase.min(if s.next.is_some() { 1.0 } else { 0.0 }),
            flight: None,
        });
    }
    let clip = s.clip.as_mut().expect("set above");
    clip.frames.extend(s.pending.drain(..));
    while let Some(f) = motion.pop(s.character) {
        clip.frames.push(f);
    }
    // Frames generated toward other goals past the jump's start (a walk
    // the model took longer over) never jump: unless a jump was asked for,
    // the clip keeps a frame past the drawn one and the model starts
    // again from them, so the next request is the jump.
    let asked = s
        .issued
        .iter()
        .any(|g| g.pose == JUMP && g.id >= clip.frames[0].tag);
    if started && !asked {
        let k = (clip.drawn.floor() as usize + 2).min(clip.frames.len() - 1);
        clip.frames.truncate(k + 1);
        let recent = std::array::from_fn(|i| clip.frames[(k + i).saturating_sub(3)].pose);
        motion.restart(s.character, recent);
        s.requested = None;
    }
    let natural = clip.drawn + elapsed * RATE;
    let flight_frames = c.jump.map(|j| 2.0 * j.v0 / j.gravity.max(1e-3) * RATE);
    let min = ((MIN_FLIGHT * flight_frames.unwrap_or(0.0)) as usize).max(2);
    // A clip that lands and takes off again (the model hopping on: a
    // second jump nobody asked for) is cut at that takeoff, and the model
    // starts again from the landed frames before it: whatever follows the
    // jump, in the clip or after it, stands on the ground the simulation
    // landed on instead of carrying the hop's elevation.
    let contacts: Vec<[f32; 4]> = clip.frames.iter().map(|f| f.contacts).collect();
    if c.jump.is_some()
        && let Some(flight::ClipFlight {
            touchdown: Some(touchdown),
            ..
        }) = flight::clip_flight(&contacts, 0, min)
        && let Some(again) = flight::clip_flight(&contacts, touchdown, min)
    {
        // The landing's absorption ends where the hips are lowest between
        // touchdown and the hop: the frames after it push off for the hop,
        // and the model restarts from those before.
        let lowest = (touchdown..again.takeoff)
            .min_by(|&i, &j| {
                let y = |k: usize| clip.frames[k].pose.positions[0][1];
                y(i).total_cmp(&y(j))
            })
            .unwrap_or(again.takeoff);
        let cut = (lowest + 1).max(clip.drawn.floor() as usize + 2);
        if cut < clip.frames.len() {
            clip.frames.truncate(cut);
            let recent = std::array::from_fn(|i| clip.frames[(cut + i).saturating_sub(4)].pose);
            motion.restart(s.character, recent);
            s.requested = None;
        }
    }
    let target = c.jump.and_then(|j| {
        let contacts: Vec<[f32; 4]> = clip.frames.iter().map(|f| f.contacts).collect();
        let fl = flight::clip_flight(&contacts, 0, min)?;
        clip.flight = Some(fl);
        let time = match j.stage {
            JumpStage::Prepare => flight::JumpTime::Prepare {
                elapsed: c.elapsed,
                prepare: c.goal.prepare,
            },
            // Through the flight the tick predicts: to its landing, on a
            // ledge or below a drop, not back at the takeoff height.
            JumpStage::Air => flight::JumpTime::Air {
                phase: match j.eta {
                    Some(eta) if j.air + eta > 0.0 => j.air / (j.air + eta),
                    _ => flight::air_phase(j.velocity, j.v0),
                },
            },
            JumpStage::Landed => flight::JumpTime::Landed { since: j.landed },
        };
        Some(flight::retime(fl, 0, clip.frames.len(), RATE, time))
    });
    let step = CATCH_UP * RATE * elapsed;
    let to = target.unwrap_or(natural);
    let last = clip.frames.len().saturating_sub(1) as f32;
    clip.drawn = (clip.drawn + (to - clip.drawn).clamp(-step, step)).clamp(0.0, last);
    let k = clip.drawn.floor() as usize;
    s.prev = clip.frames.get(k).copied();
    s.next = clip.frames.get(k + 1).copied();
    s.phase = clip.drawn - k as f32;
}
