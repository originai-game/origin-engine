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

#[cfg(test)]
mod tests {
    use super::*;
    use og_core::glam::Quat;

    const BLADE: Shape = Shape::Capsule {
        radius: 0.05,
        height: 1.0,
    };

    /// A blade on an arm of length `reach` around the origin, `angle`
    /// radians from +x towards +z, lying along the arm.
    fn arm(angle: f32, reach: f32) -> Pose {
        let (s, c) = og_core::math::sin_cos(angle);
        Pose::new(
            Vec3::new(c, 0.0, s) * reach,
            // Its y (the capsule's axis) along the arm.
            Quat::from_rotation_y(-angle) * Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2),
        )
    }

    fn volume(from: Option<Pose>, to: Pose, swing: u32) -> Volume {
        Volume {
            entity: 10,
            owner: 1,
            team: 1,
            swing,
            active: true,
            shape: BLADE,
            from,
            to,
            colliders: 0,
        }
    }

    /// A thin target on the arc at 90°.
    fn dummy(owner: u64, team: u32) -> Hurt {
        Hurt {
            entity: owner + 100,
            owner,
            team,
            solid: Shape::Box {
                half: Vec3::new(0.3, 0.5, 0.01),
            }
            .place(&Pose::new(Vec3::new(0.0, 0.0, 1.5), Quat::IDENTITY)),
        }
    }

    /// The whole blade passes through the thin dummy between two ticks: a
    /// test of where it was and where it is misses, the sweep meets it.
    #[test]
    fn a_blade_that_passes_through_between_ticks_is_swept_into_it() {
        let (from, to) = (arm(0.4, 1.5), arm(2.7, 1.5));
        let solid = |p: &Pose| BLADE.place(p);
        let target = dummy(2, 2);
        assert!(touch(&solid(&from), &target.solid).is_none());
        assert!(touch(&solid(&to), &target.solid).is_none());
        let hits = Swings::default().step(&[volume(Some(from), to, 1)], &[target], &NoColliders);
        assert_eq!(hits.len(), 1);
        assert_eq!((hits[0].target, hits[0].hurt), (2, 102));
        // Met on the dummy's face, the blade on the near side of +z.
        assert!((hits[0].point.z - 1.5).abs() < 0.05, "{:?}", hits[0]);
        // Without the sweep (first active tick) it is not met.
        let none = Swings::default().step(&[volume(None, to, 1)], &[target], &NoColliders);
        assert!(none.is_empty());
    }

    /// Ten ticks of one swing over the dummy: one hit; the next swing
    /// strikes it again.
    #[test]
    fn a_swing_strikes_a_target_once_and_the_next_swing_again() {
        let mut swings = Swings::default();
        let target = dummy(2, 2);
        let tick = |swings: &mut Swings, i: u32, swing: u32| {
            let (a, b) = (1.2 + 0.05 * i as f32, 1.2 + 0.05 * (i + 1) as f32);
            swings.step(
                &[volume(Some(arm(a, 1.5)), arm(b, 1.5), swing)],
                &[target],
                &NoColliders,
            )
        };
        let n: usize = (0..10).map(|i| tick(&mut swings, i, 7).len()).sum();
        assert_eq!(n, 1);
        assert_eq!(swings.struck(1, 7), &[2]);
        assert_eq!(tick(&mut swings, 3, 8).len(), 1);
        // Swing 7 is forgotten once no volume holds it.
        assert!(swings.struck(1, 7).is_empty());
    }

    #[test]
    fn a_team_and_an_owner_are_not_struck_by_their_own() {
        let mut swings = Swings::default();
        let v = volume(None, arm(std::f32::consts::FRAC_PI_2, 1.5), 1);
        let ally = dummy(2, 1);
        let mut own = dummy(1, 2);
        own.entity = 50;
        assert!(swings.step(&[v], &[ally, own], &NoColliders).is_empty());
        assert_eq!(swings.step(&[v], &[dummy(3, 0)], &NoColliders).len(), 1);
    }

    /// Two hurt volumes of one target: one hit, at the one met first; an
    /// inactive volume strikes nothing.
    #[test]
    fn one_hit_per_target_at_its_earliest_volume() {
        let mut near = dummy(2, 2);
        near.entity = 90;
        near.solid =
            Shape::Sphere { radius: 0.2 }.place(&Pose::new(arm(1.0, 1.5).position, Quat::IDENTITY));
        let far = dummy(2, 2);
        let v = volume(Some(arm(0.8, 1.5)), arm(2.0, 1.5), 1);
        let hits = Swings::default().step(&[v], &[far, near], &NoColliders);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].hurt, 90);
        let idle = Volume { active: false, ..v };
        assert!(
            Swings::default()
                .step(&[idle], &[far], &NoColliders)
                .is_empty()
        );
    }

    struct Wall;
    impl Colliders for Wall {
        fn overlap(&self, _: &Shape, pose: &Pose, layers: u32, _: u64) -> Vec<(u64, Contact)> {
            // A wall at x = 0.3 on layer 2.
            if layers & 2 != 0 && pose.position.x < 0.3 {
                vec![(
                    77,
                    Contact {
                        point: Vec3::X * 0.3,
                        normal: Vec3::X,
                    },
                )]
            } else {
                Vec::new()
            }
        }
    }

    #[test]
    fn colliders_on_its_layers_are_struck_once() {
        let mut v = volume(Some(arm(0.0, 1.5)), arm(std::f32::consts::PI, 1.5), 1);
        assert!(Swings::default().step(&[v], &[], &Wall).is_empty());
        v.colliders = 2;
        let mut swings = Swings::default();
        let hits = swings.step(&[v], &[], &Wall);
        assert_eq!((hits.len(), hits[0].target, hits[0].hurt), (1, 77, 0));
        assert!(swings.step(&[v], &[], &Wall).is_empty());
    }

    #[test]
    fn steps_follow_the_farthest_point() {
        // Turning in place moves the blade's ends, not its centre.
        let from = Pose::new(Vec3::ZERO, Quat::IDENTITY);
        let to = Pose::new(
            Vec3::ZERO,
            Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        );
        // Each end's arc is a quarter turn of radius 0.5 (0.785): a
        // radius of 0.05 is 16 steps; its chord alone (0.707) would be 15.
        assert_eq!(steps(&BLADE, &Motion::between(from, to)), 16);
        assert_eq!(steps(&BLADE, &Motion::between(from, from)), 1);
        let far = Pose::new(Vec3::X * 100.0, Quat::IDENTITY);
        assert_eq!(steps(&BLADE, &Motion::between(from, far)), MAX_STEPS);
    }

    /// A pose turning about a pivot it does not sit on keeps its distance
    /// from the pivot all the way: the arc, not the chord.
    #[test]
    fn a_motion_follows_the_arc_about_its_pivot() {
        let (from, to) = (arm(0.0, 1.5), arm(2.5, 1.5));
        let m = Motion::between(from, to);
        for i in 0..=8 {
            let p = m.at(i as f32 / 8.0);
            assert!((p.position.length() - 1.5).abs() < 1e-4, "{i}: {p:?}");
            // Still lying along its arm.
            let along = p.rotation * Vec3::Y;
            assert!(along.dot(p.position / 1.5) > 0.9999, "{i}");
        }
        assert!((m.at(0.5).position - arm(1.25, 1.5).position).length() < 1e-4);
    }

    #[test]
    fn swings_round_trip_through_bytes() {
        let mut swings = Swings::default();
        let v = volume(None, arm(std::f32::consts::FRAC_PI_2, 1.5), 4);
        swings.step(&[v], &[dummy(2, 2), dummy(5, 2)], &NoColliders);
        let bytes = swings.to_bytes();
        assert_eq!(Swings::from_bytes(&bytes), Some(swings));
        assert_eq!(Swings::from_bytes(&bytes[..bytes.len() - 1]), None);
    }
}
