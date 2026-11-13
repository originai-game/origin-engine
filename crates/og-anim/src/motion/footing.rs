//! A drawn character's feet: each foot's contact as the pose shows it
//! ([`Foot`]), and how well the drawn feet stand ([`Footing`], a diagnostic).

use glam::Vec3;
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// A foot is on the ground, as the pose shows it (in the model's own
/// world, never from its contact channel), when its ankle comes within
/// [`DOWN_HEIGHT`] m of its rest height over the ground moving slower than
/// [`DOWN_SPEED`] m/s, or rests within [`HELD_HEIGHT`] of it, slower than
/// [`HELD_SPEED`], for [`HELD_FRAMES`] drawn frames (a held pose's foot,
/// put down onto the ground); it lifts when it rises [`LIFT_HEIGHT`] over
/// where it touched down or moves faster than [`LIFT_SPEED`].
pub const DOWN_HEIGHT: f32 = 0.05;

pub const DOWN_SPEED: f32 = 0.6;

pub const HELD_HEIGHT: f32 = 0.2;

/// A character holding a pose in place keeps a foot down up to this high,
/// m: a held pose's shifting and fidgeting feet stand on the ground (the
/// planter steps them when they are left behind).
pub const HOLD_HEIGHT: f32 = 0.35;

pub const HELD_SPEED: f32 = 0.06;

pub const HELD_FRAMES: u32 = 6;

pub const LIFT_HEIGHT: f32 = 0.04;

pub const LIFT_SPEED: f32 = 1.0;

/// A drawn foot is on the ground, for [`Footing`], when its ankle is
/// within this of its rest height over the Transform, m (times the scale).
pub const ON_GROUND: f32 = 0.03;

/// The slide speeds [`Footing`] counts samples above, m/s.
pub const SLIDE: [f32; 2] = [0.1, 0.3];

/// How long a held foot stays up and still before [`Footing`] counts it
/// floating, s: longer than a step's rise.
pub const FLOATING: f32 = 0.1;

/// One foot's contact as the pose shows it, with hysteresis.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Foot {
    /// The ankle's height over its rest height where it touched down.
    pub touched: Option<f32>,
    /// Drawn frames it has been held still.
    pub still: u32,
}

impl Foot {
    /// Whether the foot is down at `height` m over its rest height, moving
    /// at `speed` m/s. A character `holding` a pose in place keeps every foot
    /// below [`HOLD_HEIGHT`] that is not swung fast on the ground.
    pub fn step(&mut self, height: f32, speed: f32, holding: bool) -> bool {
        self.still = if speed < HELD_SPEED {
            self.still + 1
        } else {
            0
        };
        let held = holding && height < HOLD_HEIGHT && speed < LIFT_SPEED;
        match self.touched {
            Some(_) if held => {}
            Some(t) if height > t.max(0.0) + LIFT_HEIGHT || speed > LIFT_SPEED => {
                self.touched = None;
            }
            Some(_) => {}
            None if held
                || (height < DOWN_HEIGHT && speed < DOWN_SPEED)
                || (height < HELD_HEIGHT && self.still >= HELD_FRAMES) =>
            {
                self.touched = Some(height);
            }
            None => {}
        }
        self.touched.is_some()
    }
}

/// The drawn feet as they move in the world, measured on every shown frame
/// from where they are drawn and nothing else (no contact channel, the
/// model's or the pose's): a foot on the ground in two consecutive frames
/// is a sample of its speed over the ground, counted by what the character
/// is doing; while it holds a pose, how high its feet are over the ground.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Footing {
    pub last: [Option<(Vec3, bool)>; 2],
    pub segments: BTreeMap<&'static str, Slides>,
    /// Frames of a held pose, those with a foot held over [`ON_GROUND`]
    /// without moving across the ground (floating, not stepping), and the
    /// highest a foot was over its rest height, m.
    pub held: (u32, u32, f32),
    pub floating: bool,
    /// How long each foot has been up and still, s.
    pub up: [f32; 2],
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Slides {
    pub frames: u32,
    pub samples: u32,
    pub over: [u32; 2],
    pub max: f32,
}

impl Footing {
    /// Records foot `k` drawn `at` over `rest` (its rest ankle height in
    /// the world) in `segment`, `dt` s after the last frame.
    /// A jump's clip (`clip`) is no held pose, whatever the goal.
    pub fn record(
        &mut self,
        k: usize,
        at: Vec3,
        rest: f32,
        (segment, air, clip): (&'static str, bool, bool),
        dt: f32,
    ) {
        let height = at.y - rest;
        let down = !air && height < ON_GROUND;
        let across = match self.last[k] {
            Some((was, _)) if dt > 0.0 => Vec3::new(at.x - was.x, 0.0, at.z - was.z).length() / dt,
            _ => 0.0,
        };
        let seg = self.segments.entry(segment).or_default();
        if k == 0 {
            seg.frames += 1;
        }
        if let Some((_, true)) = self.last[k]
            && down
            && dt > 0.0
        {
            let speed = across;
            seg.samples += 1;
            for (n, limit) in seg.over.iter_mut().zip(SLIDE) {
                *n += (speed > limit) as u32;
            }
            seg.max = seg.max.max(speed);
        }
        self.last[k] = Some((at, down));
        if segment == "hold" && !air && !clip {
            // A frame counts once, however many of its feet float.
            if k == 0 {
                self.held.0 += 1;
                self.floating = false;
            }
            // Up and not moving across for [`FLOATING`] s: floating (a
            // step's foot rises before it moves across, and a foot moving
            // across is stepping).
            self.up[k] = if height > ON_GROUND && across < SLIDE[0] {
                self.up[k] + dt
            } else {
                0.0
            };
            if self.up[k] >= FLOATING && !self.floating {
                self.floating = true;
                self.held.1 += 1;
            }
            self.held.2 = self.held.2.max(height);
        }
    }

    pub fn json(&self) -> Value {
        let segments: serde_json::Map<String, Value> = self
            .segments
            .iter()
            .map(|(k, s)| {
                (
                    (*k).to_string(),
                    json!({
                        "frames": s.frames,
                        "samples": s.samples,
                        "over0_1": s.over[0],
                        "over0_3": s.over[1],
                        "maxMps": s.max,
                    }),
                )
            })
            .collect();
        json!({
            "onGroundM": ON_GROUND,
            "segments": segments,
            "held": { "frames": self.held.0, "floating": self.held.1, "maxM": self.held.2 },
        })
    }
}
