//! One agent's presentation ([`Shown`]): the generated frames it plays, the
//! clip it is in and what it last asked the model for.

use super::*;

/// The bounds of a walk's or run's playback rate (stride warping by
/// time): the generated gait is played so its hips cover the ground the
/// simulation's Transform does, between these multiples of its rate.
pub(crate) const MIN_WARP: f32 = 0.5;
pub(crate) const MAX_WARP: f32 = 2.5;
/// The half-life over which the playback rate follows the speed ratio, s.
pub(crate) const WARP_HALF_LIFE: f32 = 0.08;

/// An agent as drawn.
#[derive(Clone)]
pub(crate) struct Shown {
    pub(crate) character: usize,
    pub(crate) prev: Option<Frame>,
    pub(crate) next: Option<Frame>,
    /// Where between `prev` and `next` the frame is, 0 to 1.
    pub(crate) phase: f32,
    /// The last simulation intent examined by presentation.
    pub(crate) serial: u32,
    pub(crate) requested: Option<Goal>,
    pub(crate) planter: Planter,
    pub(crate) skeleton: u64,
    pub(crate) rig: u64,
    pub(crate) target: Option<(Arc<SkeletonAsset>, HumanoidMap)>,
    pub(crate) failure: Option<String>,
    pub(crate) last_position: Vec3,
    pub(crate) velocity: Vec3,
    pub(crate) shown: u64,
    pub(crate) starved: u64,
    pub(crate) warmup: u64,
    /// The drawn pose's heading in the character's own space (its thighs'
    /// across direction): about 0 when it faces where its Transform does.
    pub(crate) facing: f32,
    /// Each leg's ankle in the world at the last frame it was planted (the
    /// model's heel contact), and how far planted ankles moved between
    /// planted frames: the summed metres and the frames (`dump_motion`'s
    /// `plantedSlide`, the foot plant's own measure).
    pub(crate) planted: [Option<Vec3>; 2],
    pub(crate) slide: (f32, u32, f32),
    /// The lean (hips to head) last drawn, rad.
    pub(crate) lean: Option<f32>,
    pub(crate) drawn_lean: Option<f32>,
    /// The rate the generated frames play at (1 is the model's own): a
    /// walk or run is played at the simulation's speed over the ground.
    pub(crate) warp: f32,
    /// The feet's contacts as the drawn pose shows them.
    pub(crate) feet: [Foot; 2],
    /// The drawn root's offset from the Transform in the world, and the
    /// generated hips (model world, over the ground) and heading it last
    /// followed: presentation only, never read by the simulation.
    pub(crate) offset: Vec3,
    pub(crate) followed: Option<(Vec3, f32)>,
    /// The drawn feet measured without any contact signal (`dump_motion`'s
    /// `footing`).
    pub(crate) footing: Footing,
    /// How far the drawn ankles are ahead of the hips along the facing, m
    /// (their mean): behind in a stride's push-off, under or ahead of the
    /// hips coming down to a landing.
    pub(crate) feet_ahead: Option<f32>,
    /// The model goals asked for, oldest first, until their frames are
    /// shown: what each frame's tag names.
    pub(crate) issued: VecDeque<Issued>,
    pub(crate) next_id: u32,
    /// While the simulation's current goal is a jump: the generated frames
    /// from the jump's start on, drawn retimed onto the simulation's flight.
    pub(crate) clip: Option<Clip>,
    /// Frames a finished jump clip had generated past the one drawn,
    /// played before the model's next.
    pub(crate) pending: VecDeque<Frame>,
}

/// A jump's generated frames and the one drawn (fractional).
#[derive(Clone)]
pub(crate) struct Clip {
    pub(crate) frames: Vec<Frame>,
    pub(crate) drawn: f32,
    /// The clip's own flight, once it has generated one.
    pub(crate) flight: Option<flight::ClipFlight>,
}

/// A model goal asked for: its id (the frames' tag), its library pose and
/// the frame clock its first frame is for (now plus the frames queued
/// before it).
#[derive(Clone)]
pub(crate) struct Issued {
    pub(crate) id: u32,
    pub(crate) pose: usize,
    pub(crate) start: f64,
}

impl Shown {
    pub(crate) fn reconcile_request(&self, motion: &mut Motion, goal: &Goal) {
        // The context remains owned by Motion until this generation lands.
        // A simulation transition alone does not change its conditioning.
        // Heading passes through Transform's quaternion between predictions;
        // allow its f32 roundoff (10 micrometres for target positions).
        let same = self.requested.as_ref().is_some_and(|old| {
            old.token_counts == goal.token_counts
                && old.targets.len() == goal.targets.len()
                && old.targets.iter().zip(&goal.targets).all(|(a, b)| {
                    a.positions
                        .iter()
                        .flatten()
                        .zip(b.positions.iter().flatten())
                        .chain(
                            a.rotations
                                .iter()
                                .flatten()
                                .flatten()
                                .zip(b.rotations.iter().flatten().flatten()),
                        )
                        .all(|(a, b)| (a - b).abs() <= 1e-5)
                })
        });
        if !same {
            motion.clear_goals(self.character);
        }
    }

    pub(crate) fn issued(&self, f: &Frame) -> Option<&Issued> {
        self.issued.iter().find(|g| g.id == f.tag)
    }

    /// Frames generated past the drawn one: the model's queue, a jump clip's
    /// frames after the drawn one and a finished clip's.
    pub(crate) fn ahead(&self, motion: &Motion) -> usize {
        let clip = self.clip.as_ref().map_or(0, |c| {
            c.frames.len().saturating_sub(c.drawn.floor() as usize + 1)
        });
        motion.queued(self.character) + clip + self.pending.len()
    }

    /// The playback rate after `elapsed` s: in a walk or run, the
    /// simulation's speed over the ground over the generated hips' (so a
    /// planted foot stays where it is in the world while the Transform
    /// carries the hips), and fast while the simulation has stopped and
    /// the model is still walking there; 1 in anything else.
    pub(crate) fn warp_for(&mut self, elapsed: f32) -> f32 {
        let gait = self.clip.is_none()
            && self
                .prev
                .and_then(|f| self.issued(&f))
                .is_some_and(|g| is_gait(g.pose));
        let to = match (self.prev, self.next) {
            (Some(a), Some(b)) if gait => {
                // The generated stride's speed: how fast the hips pass over
                // the feet on the ground (what a planted foot sees, whether
                // or not the model holds it still), else the hips' own.
                let flat = |f: &Frame, j: usize| {
                    let [x, _, z] = f.pose.positions[j];
                    Vec3::new(x, 0.0, z)
                };
                let hips = flat(&b, 0) - flat(&a, 0);
                let down: Vec<Vec3> = [LEFT_ANKLE, RIGHT_ANKLE]
                    .into_iter()
                    .zip(self.feet)
                    .filter(|(_, f)| f.touched.is_some())
                    .map(|(j, _)| hips - (flat(&b, j) - flat(&a, j)))
                    .collect();
                let stride = if down.is_empty() {
                    hips.length()
                } else {
                    down.iter().map(|d| d.length()).sum::<f32>() / down.len() as f32
                } * RATE;
                let sim = Vec3::new(self.velocity.x, 0.0, self.velocity.z).length();
                if sim < 0.1 || stride < 0.05 {
                    MAX_WARP
                } else {
                    (sim / stride).clamp(MIN_WARP, MAX_WARP)
                }
            }
            _ => 1.0,
        };
        let k = 1.0 - og_core::math::exp(-elapsed * std::f32::consts::LN_2 / WARP_HALF_LIFE);
        self.warp += (to - self.warp) * k;
        self.warp
    }

    pub(crate) fn count_wait(&mut self) {
        if self.prev.is_none() {
            self.warmup += 1;
        } else if self.next.is_none() || self.phase >= 1.0 {
            self.starved += 1;
        }
    }
}
