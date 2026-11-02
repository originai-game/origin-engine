//! The pose library the engine's goals name: a few frames of each of
//! [`POSES`], in SOMA world poses at 30 Hz, carried by the bundle (og-motion-train's
//! `scripts/export_engine.py` picks them from its motion library).
//!
//! The names are fixed by the engine's version, not read from the bundle,
//! so a goal's pose is checked the same way before the weights are
//! resident and on every device. A bundle carries them as `poses.counts`
//! `[POSES.len()]` (each pose's frames, 1 to 4, in [`POSES`] order),
//! `poses.positions` `[n,30,3]` and `poses.rotations` `[n,30,3,3]` (every
//! pose's frames one after another).

use crate::bundle::{Bundle, BundleError};
use crate::contract::{JOINTS, Pose};

/// The library's poses, in the bundle's order.
pub const POSES: [&str; 7] = ["idle", "walk", "run", "jump", "crouch", "wave", "cheer"];

/// The index of pose `name` in [`POSES`].
pub fn index(name: &str) -> Option<usize> {
    POSES.iter().position(|p| *p == name)
}

/// Every pose of [`POSES`]: one to four consecutive frames each.
#[derive(Clone, Debug, PartialEq)]
pub struct Library {
    poses: Vec<Vec<Pose>>,
}

impl Library {
    /// The library a bundle carries.
    pub fn from_bundle(bundle: &Bundle) -> Result<Library, BundleError> {
        let counts = bundle.get("poses.counts", &[POSES.len()])?;
        let mut sizes = Vec::with_capacity(POSES.len());
        for (name, &c) in POSES.iter().zip(counts) {
            let c = c as usize;
            if !(1..=4).contains(&c) {
                return Err(BundleError::Header(format!(
                    "poses.counts: pose '{name}' has {c} frames, not 1 to 4"
                )));
            }
            sizes.push(c);
        }
        let n: usize = sizes.iter().sum();
        let positions = bundle.get("poses.positions", &[n, JOINTS, 3])?;
        let rotations = bundle.get("poses.rotations", &[n, JOINTS, 3, 3])?;
        let frame = |f: usize| Pose {
            positions: std::array::from_fn(|j| {
                std::array::from_fn(|k| positions[(f * JOINTS + j) * 3 + k])
            }),
            rotations: std::array::from_fn(|j| {
                std::array::from_fn(|a| {
                    std::array::from_fn(|b| rotations[((f * JOINTS + j) * 3 + a) * 3 + b])
                })
            }),
        };
        let mut at = 0;
        let poses = sizes
            .iter()
            .map(|&c| {
                let frames = (at..at + c).map(frame).collect();
                at += c;
                frames
            })
            .collect();
        Ok(Library { poses })
    }

    /// The frames of pose `index` ([`POSES`] order).
    pub fn frames(&self, index: usize) -> &[Pose] {
        &self.poses[index]
    }
}
