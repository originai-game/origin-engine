//! The intent API (`docs/design.md` §2.7): game code says what each
//! character should reach next, as a [`Goal`] (one to four target poses
//! placed in the world), and [`Motion`] fills in the motion from the frames
//! each character last generated, a span at a time, many characters in one
//! batch, chained.
//!
//! ```ignore
//! let mut motion = Motion::default();
//! let hero = motion.add(standing);                                  // its last four frames
//! motion.push(hero, Goal::placed(&walk_pose, [3.0, 4.0], 1.57));     // a pose, where, facing
//! motion.generate(&model, 30);                                       // or generate_gpu(&mut gpu, 30)
//! while let Some(frame) = motion.pop(hero) { /* 30 Hz frames: a pose and its foot contacts */ }
//! ```
//!
//! Poses are presentation (§2.4): the simulation moves the character, the
//! goals follow it, and nothing here feeds back into a tick.

use std::collections::VecDeque;

#[cfg(feature = "reference")]
use crate::Model;
use crate::contract::{
    FRAMES, LEFT_LEG, MOTION_CONTACTS, MOTION_FEATURES, Pose, RIGHT_LEG, Request, encode, heading,
    mat_mul, mat_vec, rot_y, valid_frames,
};
use crate::gpu::{Gpu, GpuError};
use crate::{Alignment, Generation, Inputs, decode_frame};

/// What a character reaches next: one to four consecutive target poses (30
/// Hz) in the world, and the token counts the generation may take (empty
/// for the model's default, 8 to 14: 1 to 1.7 s).
#[derive(Clone, Debug, PartialEq)]
pub struct Goal {
    pub targets: Vec<Pose>,
    pub token_counts: Vec<usize>,
    /// The caller's own mark, carried to every frame generated toward it
    /// ([`Frame::tag`]): which library pose it asked for, say.
    pub tag: u32,
}

impl Goal {
    /// `poses` (one to four consecutive frames, anywhere: a library's pose)
    /// moved so that the first frame's hips stand over `at` (world x, z)
    /// and it faces `heading` (radians; 0 is +Z, π/2 is +X).
    pub fn placed(poses: &[Pose], at: [f32; 2], heading_to: f32) -> Goal {
        assert!(
            (1..=4).contains(&poses.len()),
            "a goal has one to four target poses"
        );
        let first = &poses[0];
        let from = heading(first.positions[RIGHT_LEG], first.positions[LEFT_LEG]);
        let turn = rot_y(heading_to - from);
        let ground = [first.positions[0][0], 0.0, first.positions[0][2]];
        let targets = poses
            .iter()
            .map(|p| Pose {
                positions: p.positions.map(|q| {
                    let v = mat_vec(&turn, [q[0] - ground[0], q[1], q[2] - ground[2]]);
                    [v[0] + at[0], v[1], v[2] + at[1]]
                }),
                rotations: p.rotations.map(|r| mat_mul(&turn, &r)),
            })
            .collect();
        Goal {
            targets,
            token_counts: Vec::new(),
            tag: 0,
        }
    }
}

/// One generated frame of a character: its world pose and the model's foot
/// contacts (left heel, left toe, right heel, right toe; above 0.5 is on
/// the ground).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub pose: Pose,
    pub contacts: [f32; 4],
    /// The [`Goal::tag`] of the goal it was generated toward.
    pub tag: u32,
    /// Its place among that goal's frames, from 0 (30 Hz).
    pub index: u32,
}

#[derive(Clone, Debug)]
struct Character {
    /// The last four frames generated: the next generation starts from them.
    context: [Pose; 4],
    goals: VecDeque<Goal>,
    frames: VecDeque<Frame>,
    revision: u64,
}

/// The characters a model animates, their goals and their generated frames.
#[derive(Clone, Debug, Default)]
pub struct Motion {
    characters: Vec<Character>,
}

/// The characters of one generation and their requests, encoded.
#[derive(Clone, Debug)]
pub struct Batch {
    pub characters: Vec<usize>,
    pub inputs: Inputs,
    pub alignments: Vec<Alignment>,
    revisions: Vec<u64>,
}

impl Motion {
    /// A character standing in its four most recent frames (30 Hz, oldest
    /// first); answers its index.
    pub fn add(&mut self, recent: [Pose; 4]) -> usize {
        self.characters.push(Character {
            context: recent,
            goals: VecDeque::new(),
            frames: VecDeque::new(),
            revision: 0,
        });
        self.characters.len() - 1
    }

    /// Queues a goal after the character's others: each generation takes
    /// one, and a character with none holds its last frame.
    pub fn push(&mut self, character: usize, goal: Goal) {
        self.characters[character].goals.push_back(goal);
    }

    /// Drops queued goals and cancels their in-flight generation. Frames
    /// already accepted stay; a stale completion cannot consume a new goal.
    pub fn clear_goals(&mut self, character: usize) {
        let ch = &mut self.characters[character];
        ch.goals.clear();
        ch.revision = ch.revision.wrapping_add(1);
    }

    /// Goals the character has not started.
    pub fn goals(&self, character: usize) -> usize {
        self.characters[character].goals.len()
    }

    /// Generated frames the character has not shown.
    pub fn queued(&self, character: usize) -> usize {
        self.characters[character].frames.len()
    }

    /// The character's next frame to show.
    pub fn pop(&mut self, character: usize) -> Option<Frame> {
        self.characters[character].frames.pop_front()
    }

    /// The next generation: every character with a goal and fewer than
    /// `lookahead` frames queued, `None` when there is none.
    pub fn batch(&self, lookahead: usize) -> Option<Batch> {
        let characters: Vec<usize> = (0..self.characters.len())
            .filter(|&c| {
                let ch = &self.characters[c];
                !ch.goals.is_empty() && ch.frames.len() < lookahead
            })
            .collect();
        if characters.is_empty() {
            return None;
        }
        let requests: Vec<Request> = characters
            .iter()
            .map(|&c| {
                let ch = &self.characters[c];
                let goal = &ch.goals[0];
                Request {
                    context: &ch.context,
                    targets: &goal.targets,
                    token_counts: &goal.token_counts,
                }
            })
            .collect();
        let (inputs, alignments) = encode(&requests);
        Some(Batch {
            revisions: characters
                .iter()
                .map(|&c| self.characters[c].revision)
                .collect(),
            characters,
            inputs,
            alignments,
        })
    }

    /// Takes a batch's generation: each character's frames after its
    /// context are queued, its goal is done, and its next generation starts
    /// from the last four.
    pub fn accept(&mut self, batch: &Batch, generation: &Generation) {
        let per = FRAMES * MOTION_FEATURES;
        for i in 0..batch.characters.len() {
            let motion = &generation.motion[i * per..(i + 1) * per];
            self.accept_character(batch, i, generation.counts[i], motion);
        }
    }

    /// [`Motion::accept`] for the batch's character `i` alone: its token
    /// count and its `[64,366]` motion (`glue::finish_character`).
    pub fn accept_character(&mut self, batch: &Batch, i: usize, count: u32, motion: &[f32]) {
        let ch = &mut self.characters[batch.characters[i]];
        if ch.revision != batch.revisions[i] {
            return;
        }
        let tag = ch.goals.pop_front().map_or(0, |g| g.tag);
        let valid = valid_frames(count as usize);
        // The first valid frame is the context's last, already shown.
        let mut recent: Vec<Pose> = ch.context.to_vec();
        for (index, f) in (valid.start + 1..valid.end).enumerate() {
            let frame = &motion[f * MOTION_FEATURES..(f + 1) * MOTION_FEATURES];
            let pose = decode_frame(frame, &batch.alignments[i]);
            ch.frames.push_back(Frame {
                pose,
                contacts: std::array::from_fn(|k| frame[MOTION_CONTACTS.start + k]),
                tag,
                index: index as u32,
            });
            recent.push(pose);
        }
        ch.context = std::array::from_fn(|k| recent[recent.len() - 4 + k]);
    }

    /// One generation on the CPU ([`Motion::batch`], then
    /// [`Motion::accept`]); answers how many characters it moved. The
    /// reference path (the `reference` feature).
    #[cfg(feature = "reference")]
    pub fn generate(&mut self, model: &Model, lookahead: usize) -> usize {
        let Some(batch) = self.batch(lookahead) else {
            return 0;
        };
        let generation = model.generate(&batch.inputs);
        self.accept(&batch, &generation);
        batch.characters.len()
    }

    /// One generation on the GPU.
    pub async fn generate_gpu(
        &mut self,
        gpu: &mut Gpu,
        lookahead: usize,
    ) -> Result<usize, GpuError> {
        let Some(batch) = self.batch(lookahead) else {
            return Ok(0);
        };
        let generation = gpu.generate(&batch.inputs).await?;
        self.accept(&batch, &generation);
        Ok(batch.characters.len())
    }
}
