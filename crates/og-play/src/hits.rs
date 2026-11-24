//! Hit volumes against hurt volumes and colliders: each active hit volume
//! is swept from where it was at the last tick to where it is, so a fast
//! blade meets what it passed through between two ticks, and each target
//! is reported once a swing.

use crate::contact::{Contact, touch};
use crate::shape::{Motion, Pose, Shape, Solid};
use og_core::glam::Vec3;
use std::collections::BTreeMap;

/// The most tests a sweep makes in one tick: past it, a volume moving
/// farther than this many of its thickness in a tick may step over a
/// target thinner than the gap.
pub const MAX_STEPS: u32 = 64;

/// A hit volume as the tick leaves it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Volume {
    pub entity: u64,
    /// Whose swing it is: what a target is reported once to.
    pub owner: u64,
    pub team: u32,
    pub swing: u32,
    pub active: bool,
    pub shape: Shape,
    /// Where it was at the last tick, when it was active then too: the
    /// sweep starts there. `None` tests where it is now alone.
    pub from: Option<Pose>,
    pub to: Pose,
    /// The collider layers it hits besides hurt volumes; 0 hits none.
    pub colliders: u32,
}

/// A hurt volume, placed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hurt {
    pub entity: u64,
    /// Who is hurt: what a hit names as its target.
    pub owner: u64,
    pub team: u32,
    pub solid: Solid,
}

/// The colliders a hit volume may strike, as the physics world answers.
pub trait Colliders {
    /// The colliders on `layers` that overlap `shape` at `pose`, each with
    /// where they meet, in collider entity order; never `exclude`.
    fn overlap(&self, shape: &Shape, pose: &Pose, layers: u32, exclude: u64)
    -> Vec<(u64, Contact)>;
}

/// No colliders: hurt volumes only.
pub struct NoColliders;

impl Colliders for NoColliders {
    fn overlap(&self, _: &Shape, _: &Pose, _: u32, _: u64) -> Vec<(u64, Contact)> {
        Vec::new()
    }
}

/// A target struck for the first time in a swing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hit {
    pub volume: u64,
    pub owner: u64,
    pub swing: u32,
    /// The hurt volume's owner, or the collider's entity.
    pub target: u64,
    /// The hurt volume struck first; 0 for a collider.
    pub hurt: u64,
    pub point: Vec3,
    pub normal: Vec3,
}

/// How many tests a sweep from `from` to `to` makes: enough that no point
/// of the shape moves farther than its thickness between two.
pub fn steps(shape: &Shape, motion: &Motion) -> u32 {
    let thickness = shape.thickness();
    if thickness <= 0.0 {
        return 1;
    }
    let n = (shape.travel(motion) / thickness).ceil();
    if n.is_nan() || n < 1.0 {
        1
    } else if n > MAX_STEPS as f32 {
        MAX_STEPS
    } else {
        n as u32
    }
}

/// The poses a volume is tested at this tick, in order.
fn poses(v: &Volume) -> Vec<Pose> {
    match v.from {
        None => vec![v.to],
        Some(from) => {
            let motion = Motion::between(from, v.to);
            let n = steps(&v.shape, &motion);
            (1..=n).map(|i| motion.at(i as f32 / n as f32)).collect()
        }
    }
}

/// The targets each owner's swing has struck: world state, saved and
/// hashed with the world. A swing is forgotten once no volume of its owner
/// holds it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Swings(BTreeMap<(u64, u32), Vec<u64>>);

impl Swings {
    /// One tick: every active volume swept, in the order given (the
    /// binding's entity order), against every hurt volume of another team
    /// and owner and the colliders on its layers. Returns each target
    /// struck for the first time in its swing, at the earliest test that
    /// met it (ties: the first hurt volume given), by volume, then test,
    /// then target.
    pub fn step(
        &mut self,
        volumes: &[Volume],
        hurts: &[Hurt],
        colliders: &dyn Colliders,
    ) -> Vec<Hit> {
        let mut out = Vec::new();
        for v in volumes.iter().filter(|v| v.active) {
            // Per target: the earliest test, the hurt volume and where.
            let mut met: BTreeMap<u64, (usize, u64, Contact)> = BTreeMap::new();
            for (i, pose) in poses(v).iter().enumerate() {
                let solid = v.shape.place(pose);
                for h in hurts {
                    if h.team == v.team || h.owner == v.owner || met.contains_key(&h.owner) {
                        continue;
                    }
                    if let Some(c) = touch(&solid, &h.solid) {
                        met.insert(h.owner, (i, h.entity, c));
                    }
                }
                if v.colliders != 0 {
                    for (e, c) in colliders.overlap(&v.shape, pose, v.colliders, v.owner) {
                        if e != v.entity && e != v.owner {
                            met.entry(e).or_insert((i, 0, c));
                        }
                    }
                }
            }
            let mut found: Vec<(usize, u64, u64, Contact)> =
                met.into_iter().map(|(t, (i, h, c))| (i, t, h, c)).collect();
            found.sort_by_key(|&(i, t, ..)| (i, t));
            let struck = self.0.entry((v.owner, v.swing)).or_default();
            for (_, target, hurt, c) in found {
                if let Err(at) = struck.binary_search(&target) {
                    struck.insert(at, target);
                    out.push(Hit {
                        volume: v.entity,
                        owner: v.owner,
                        swing: v.swing,
                        target,
                        hurt,
                        point: c.point,
                        normal: c.normal,
                    });
                }
            }
        }
        self.0.retain(|&(owner, swing), _| {
            volumes.iter().any(|v| (v.owner, v.swing) == (owner, swing))
        });
        out
    }

    /// The targets `owner`'s swing `swing` has struck, in entity order.
    pub fn struck(&self, owner: u64, swing: u32) -> &[u64] {
        self.0.get(&(owner, swing)).map_or(&[], |t| t.as_slice())
    }

    /// As bytes, for the world's state block: little-endian, in key order.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend((self.0.len() as u32).to_le_bytes());
        for (&(owner, swing), targets) in &self.0 {
            out.extend(owner.to_le_bytes());
            out.extend(swing.to_le_bytes());
            out.extend((targets.len() as u32).to_le_bytes());
            for t in targets {
                out.extend(t.to_le_bytes());
            }
        }
        out
    }

    /// From [`Swings::to_bytes`]; `None` for bytes it did not write.
    pub fn from_bytes(bytes: &[u8]) -> Option<Swings> {
        let mut at = 0usize;
        let mut take = |n: usize| -> Option<&[u8]> {
            let s = bytes.get(at..at + n)?;
            at += n;
            Some(s)
        };
        let u32_ = |s: &[u8]| u32::from_le_bytes(s.try_into().expect("four bytes"));
        let u64_ = |s: &[u8]| u64::from_le_bytes(s.try_into().expect("eight bytes"));
        let mut map = BTreeMap::new();
        for _ in 0..u32_(take(4)?) {
            let owner = u64_(take(8)?);
            let swing = u32_(take(4)?);
            let n = u32_(take(4)?) as usize;
            let mut targets = Vec::with_capacity(n.min(1024));
            for _ in 0..n {
                targets.push(u64_(take(8)?));
            }
            map.insert((owner, swing), targets);
        }
        (at == bytes.len()).then_some(Swings(map))
    }
}
