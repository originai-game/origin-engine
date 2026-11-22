//! A volume's shape and where it is: a sphere, a capsule or a box, at a
//! world pose, and the same shape as a world-space solid the contact
//! tests read.

use og_core::glam::{Quat, Vec3};
use og_core::math;

/// A volume's shape in its own frame, sizes already scaled.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    Sphere {
        radius: f32,
    },
    /// Along its local y: `height` between the caps' centres, as three.js
    /// CapsuleGeometry's length.
    Capsule {
        radius: f32,
        height: f32,
    },
    /// `half`: half its extents.
    Box {
        half: Vec3,
    },
}

impl Shape {
    /// How far it may move between two tests without stepping over a
    /// target: its thinnest half (a sphere's or capsule's radius, a box's
    /// least half extent).
    pub fn thickness(&self) -> f32 {
        match *self {
            Shape::Sphere { radius } | Shape::Capsule { radius, .. } => radius,
            Shape::Box { half } => half.min_element(),
        }
    }

    /// The points that move farthest when the shape turns about its
    /// centre: a capsule's cap centres, a box's corners.
    fn extremes(&self) -> Vec<Vec3> {
        match *self {
            Shape::Sphere { .. } => vec![Vec3::ZERO],
            Shape::Capsule { height, .. } => {
                vec![Vec3::Y * (height * 0.5), Vec3::Y * (-height * 0.5)]
            }
            Shape::Box { half } => (0..8)
                .map(|i| {
                    Vec3::new(
                        if i & 1 == 0 { -half.x } else { half.x },
                        if i & 2 == 0 { -half.y } else { half.y },
                        if i & 4 == 0 { -half.z } else { half.z },
                    )
                })
                .collect(),
        }
    }

    /// At most how far any point of the shape travels along `motion`:
    /// its farthest point's straight distance, lengthened by the arc's
    /// ratio to its chord (never less than the arc).
    pub fn travel(&self, motion: &Motion) -> f32 {
        let chord = self
            .extremes()
            .into_iter()
            .map(|p| (motion.to.apply(p) - motion.from.apply(p)).length())
            .fold(0.0, f32::max);
        chord * motion.arc
    }

    /// The shape at `pose`, in world space.
    pub fn place(&self, pose: &Pose) -> Solid {
        match *self {
            Shape::Sphere { radius } => Solid::Round {
                a: pose.position,
                b: pose.position,
                radius,
            },
            Shape::Capsule { radius, height } => {
                let half = pose.rotation * (Vec3::Y * (height * 0.5));
                Solid::Round {
                    a: pose.position - half,
                    b: pose.position + half,
                    radius,
                }
            }
            Shape::Box { half } => Solid::Box {
                centre: pose.position,
                axes: [
                    pose.rotation * Vec3::X,
                    pose.rotation * Vec3::Y,
                    pose.rotation * Vec3::Z,
                ],
                half,
            },
        }
    }
}

/// Where a shape is: its centre and rotation in world space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub position: Vec3,
    pub rotation: Quat,
}

impl Pose {
    pub fn new(position: Vec3, rotation: Quat) -> Pose {
        Pose { position, rotation }
    }

    fn apply(&self, p: Vec3) -> Vec3 {
        self.position + self.rotation * p
    }

    /// Where `p`, a point in the shape's frame, is in the world.
    pub fn point(&self, p: Vec3) -> Vec3 {
        self.apply(p)
    }
}

/// A shape placed in world space: a segment swollen by a radius (a sphere
/// when its ends meet, a capsule) or an oriented box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Solid {
    Round {
        a: Vec3,
        b: Vec3,
        radius: f32,
    },
    Box {
        centre: Vec3,
        axes: [Vec3; 3],
        half: Vec3,
    },
}

impl Solid {
    /// Its centre.
    pub fn centre(&self) -> Vec3 {
        match *self {
            Solid::Round { a, b, .. } => (a + b) * 0.5,
            Solid::Box { centre, .. } => centre,
        }
    }
}

/// The rigid motion from one pose to another as a screw (Chasles): a turn
/// about one axis and a slide along it. A volume on a bone turning about a
/// joint between two ticks follows the joint's arc, which a straight line
/// between its two poses would cut. Deterministic: [`math::acos`] and
/// [`math::sin_cos`], never the host's libm.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Motion {
    pub from: Pose,
    pub to: Pose,
    /// The turn's unit axis, its angle (0 to π) and a point on its axis;
    /// `None` when the poses turn (nearly) not at all.
    turn: Option<(Vec3, f32, Vec3)>,
    /// The slide along the axis (the whole move without a turn).
    slide: Vec3,
    /// How much longer than its chord a point's arc is (1 without a turn).
    arc: f32,
}

impl Motion {
    pub fn between(from: Pose, to: Pose) -> Motion {
        let mut q = to.rotation * from.rotation.conjugate();
        if q.w < 0.0 {
            q = -q;
        }
        let v = Vec3::new(q.x, q.y, q.z);
        let sin_half = v.length();
        let moved = to.position - from.position;
        if sin_half < 1e-4 {
            return Motion {
                from,
                to,
                turn: None,
                slide: moved,
                arc: 1.0,
            };
        }
        let axis = v / sin_half;
        let angle = 2.0 * math::acos(q.w.min(1.0));
        let slide = axis * moved.dot(axis);
        let across = moved - slide;
        // The axis's point: on the bisector of the chord `across`, at
        // |across| / (2 tan(angle / 2)) from its middle.
        let pivot = from.position + across * 0.5 + axis.cross(across) * (q.w / (2.0 * sin_half));
        Motion {
            from,
            to,
            turn: Some((axis, angle, pivot)),
            slide,
            arc: (angle * 0.5 / sin_half).max(1.0),
        }
    }

    /// The pose `t` of the way along (0: `from`, 1: `to`).
    pub fn at(&self, t: f32) -> Pose {
        if t >= 1.0 {
            return self.to;
        }
        match self.turn {
            None => Pose {
                position: self.from.position + self.slide * t,
                rotation: math::quat_slerp(self.from.rotation, self.to.rotation, t),
            },
            Some((axis, angle, pivot)) => {
                let r = math::quat_from_axis_angle(axis, angle * t);
                Pose {
                    position: pivot + r * (self.from.position - pivot) + self.slide * t,
                    rotation: (r * self.from.rotation).normalize(),
                }
            }
        }
    }
}
