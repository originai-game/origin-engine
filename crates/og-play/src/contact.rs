//! Whether two placed solids touch, and where: the closest points between
//! segments, a segment and a box, and the separating axes of two boxes.
//! Plain f32 arithmetic and sqrt only, in a fixed order, so every host
//! finds the same contact.

use crate::shape::Solid;
use og_core::glam::Vec3;

/// Where a hitter meets its target: a point on the target's surface (or
/// inside it, when the hitter's core is) and the direction from the
/// target towards the hitter there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Contact {
    pub point: Vec3,
    pub normal: Vec3,
}

/// The contact of `hitter` with `target`, or `None` when they do not
/// overlap (touching surfaces do not).
pub fn touch(hitter: &Solid, target: &Solid) -> Option<Contact> {
    match (*hitter, *target) {
        (
            Solid::Round { a, b, radius: rh },
            Solid::Round {
                a: c,
                b: d,
                radius: rt,
            },
        ) => {
            let (ph, pt) = closest_segments(a, b, c, d);
            let gap = ph - pt;
            let dist = gap.length();
            if dist >= rh + rt {
                return None;
            }
            let normal = if dist > 0.0 { gap / dist } else { Vec3::Y };
            Some(Contact {
                point: pt + normal * rt.min(dist),
                normal,
            })
        }
        (Solid::Round { a, b, radius }, Solid::Box { centre, axes, half }) => {
            let bx = Frame { centre, axes, half };
            let p = lerp(a, b, segment_box_t(&bx, a, b));
            let lp = bx.local(p);
            let q = lp.clamp(-half, half);
            let dist = (lp - q).length();
            if dist > 0.0 {
                if dist >= radius {
                    return None;
                }
                return Some(Contact {
                    point: bx.world(q),
                    normal: bx.world_dir((lp - q) / dist),
                });
            }
            let (point, normal) = bx.face(lp);
            Some(Contact {
                point: bx.world(point),
                normal: bx.world_dir(normal),
            })
        }
        (Solid::Box { centre, axes, half }, Solid::Round { a, b, radius }) => {
            let bx = Frame { centre, axes, half };
            let p = lerp(a, b, segment_box_t(&bx, a, b));
            let lp = bx.local(p);
            let q = lp.clamp(-half, half);
            let dist = (lp - q).length();
            if dist >= radius {
                return None;
            }
            let normal = if dist > 0.0 {
                bx.world_dir((q - lp) / dist)
            } else {
                // The target's core is inside the hitter: towards the
                // hitter's centre.
                let to = centre - p;
                let len = to.length();
                if len > 0.0 { to / len } else { Vec3::Y }
            };
            Some(Contact {
                point: p + normal * radius,
                normal,
            })
        }
        (
            Solid::Box {
                centre: ca,
                axes: aa,
                half: ha,
            },
            Solid::Box {
                centre: cb,
                axes: ab,
                half: hb,
            },
        ) => {
            let normal = boxes(
                &Frame {
                    centre: ca,
                    axes: aa,
                    half: ha,
                },
                &Frame {
                    centre: cb,
                    axes: ab,
                    half: hb,
                },
            )?;
            let target = Frame {
                centre: cb,
                axes: ab,
                half: hb,
            };
            Some(Contact {
                point: target.world(target.local(ca).clamp(-hb, hb)),
                normal,
            })
        }
    }
}

fn lerp(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    a + (b - a) * t
}

/// The closest points of segments `a`–`b` and `c`–`d` (Ericson,
/// Real-Time Collision Detection §5.1.9).
fn closest_segments(a: Vec3, b: Vec3, c: Vec3, d: Vec3) -> (Vec3, Vec3) {
    let (d1, d2, r) = (b - a, d - c, a - c);
    let (aa, ee, f) = (d1.dot(d1), d2.dot(d2), d2.dot(r));
    let (s, t);
    if aa <= f32::EPSILON && ee <= f32::EPSILON {
        return (a, c);
    }
    if aa <= f32::EPSILON {
        s = 0.0;
        t = (f / ee).clamp(0.0, 1.0);
    } else {
        let cc = d1.dot(r);
        if ee <= f32::EPSILON {
            t = 0.0;
            s = (-cc / aa).clamp(0.0, 1.0);
        } else {
            let bb = d1.dot(d2);
            let denom = aa * ee - bb * bb;
            let s0 = if denom > 0.0 {
                ((bb * f - cc * ee) / denom).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let t0 = (bb * s0 + f) / ee;
            if t0 < 0.0 {
                t = 0.0;
                s = (-cc / aa).clamp(0.0, 1.0);
            } else if t0 > 1.0 {
                t = 1.0;
                s = ((bb - cc) / aa).clamp(0.0, 1.0);
            } else {
                t = t0;
                s = s0;
            }
        }
    }
    (a + d1 * s, c + d2 * t)
}

/// An oriented box: its centre, unit axes and half extents.
struct Frame {
    centre: Vec3,
    axes: [Vec3; 3],
    half: Vec3,
}

impl Frame {
    fn local(&self, p: Vec3) -> Vec3 {
        let d = p - self.centre;
        Vec3::new(
            d.dot(self.axes[0]),
            d.dot(self.axes[1]),
            d.dot(self.axes[2]),
        )
    }

    fn world(&self, p: Vec3) -> Vec3 {
        self.centre + self.world_dir(p)
    }

    fn world_dir(&self, v: Vec3) -> Vec3 {
        self.axes[0] * v.x + self.axes[1] * v.y + self.axes[2] * v.z
    }

    /// For a local point inside: the nearest face's point and its outward
    /// normal (ties to the lower axis).
    fn face(&self, lp: Vec3) -> (Vec3, Vec3) {
        let depth = self.half - lp.abs();
        let i = if depth.x <= depth.y && depth.x <= depth.z {
            0
        } else if depth.y <= depth.z {
            1
        } else {
            2
        };
        let sign = if lp[i] < 0.0 { -1.0 } else { 1.0 };
        let mut point = lp;
        point[i] = sign * self.half[i];
        let mut normal = Vec3::ZERO;
        normal[i] = sign;
        (point, normal)
    }
}

/// Where on segment `a`–`b` (0 to 1) it comes nearest the box: the
/// distance to a box is convex along a segment, so a golden-section search
/// of a fixed number of steps finds it, the same on every host.
fn segment_box_t(bx: &Frame, a: Vec3, b: Vec3) -> f32 {
    let (la, lb) = (bx.local(a), bx.local(b));
    let gap = |t: f32| {
        let p = la + (lb - la) * t;
        (p - p.clamp(-bx.half, bx.half)).length_squared()
    };
    if (lb - la).length_squared() <= f32::EPSILON {
        return 0.0;
    }
    const GOLDEN: f32 = 0.618_034;
    let (mut lo, mut hi) = (0.0f32, 1.0f32);
    let mut x1 = hi - GOLDEN * (hi - lo);
    let mut x2 = lo + GOLDEN * (hi - lo);
    let (mut f1, mut f2) = (gap(x1), gap(x2));
    for _ in 0..40 {
        if f1 <= f2 {
            hi = x2;
            x2 = x1;
            f2 = f1;
            x1 = hi - GOLDEN * (hi - lo);
            f1 = gap(x1);
        } else {
            lo = x1;
            x1 = x2;
            f1 = f2;
            x2 = lo + GOLDEN * (hi - lo);
            f2 = gap(x2);
        }
    }
    (lo + hi) * 0.5
}

/// Two boxes' separating-axis test (the 15 axes: each box's three and
/// their nine cross products): `None` when an axis separates them, else
/// the axis of least overlap, pointing from `target` towards `hitter`.
fn boxes(hitter: &Frame, target: &Frame) -> Option<Vec3> {
    let mut axes: Vec<Vec3> = hitter.axes.iter().chain(&target.axes).copied().collect();
    for a in &hitter.axes {
        for b in &target.axes {
            let c = a.cross(*b);
            let len = c.length();
            if len > 1e-6 {
                axes.push(c / len);
            }
        }
    }
    let between = hitter.centre - target.centre;
    let reach = |f: &Frame, l: Vec3| {
        f.half.x * f.axes[0].dot(l).abs()
            + f.half.y * f.axes[1].dot(l).abs()
            + f.half.z * f.axes[2].dot(l).abs()
    };
    let mut best: Option<(f32, Vec3)> = None;
    for l in axes {
        let d = between.dot(l);
        let overlap = reach(hitter, l) + reach(target, l) - d.abs();
        if overlap <= 0.0 {
            return None;
        }
        if best.is_none_or(|(o, _)| overlap < o) {
            best = Some((overlap, if d < 0.0 { -l } else { l }));
        }
    }
    best.map(|(_, l)| l)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape::{Pose, Shape};
    use og_core::glam::Quat;

    fn at(shape: Shape, x: f32, y: f32, z: f32, rotation: Quat) -> Solid {
        shape.place(&Pose::new(Vec3::new(x, y, z), rotation))
    }

    #[test]
    fn spheres_touch_by_their_radii_and_say_which_way() {
        let s = Shape::Sphere { radius: 0.5 };
        let hit = touch(
            &at(s, 0.9, 0.0, 0.0, Quat::IDENTITY),
            &at(s, 0.0, 0.0, 0.0, Quat::IDENTITY),
        )
        .unwrap();
        assert_eq!(hit.normal, Vec3::X);
        assert!((hit.point - Vec3::new(0.5, 0.0, 0.0)).length() < 1e-6);
        assert!(
            touch(
                &at(s, 1.0, 0.0, 0.0, Quat::IDENTITY),
                &at(s, 0.0, 0.0, 0.0, Quat::IDENTITY)
            )
            .is_none()
        );
    }

    /// A capsule's ends, not its centre: crossed capsules far apart at
    /// their centres' line still meet where their segments cross.
    #[test]
    fn crossed_capsules_meet_along_their_segments() {
        let c = Shape::Capsule {
            radius: 0.1,
            height: 4.0,
        };
        let lying = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
        let a = at(c, 0.0, 1.5, 0.15, Quat::IDENTITY);
        let b = at(c, 0.0, 0.0, 0.0, lying);
        assert!(touch(&a, &b).is_some());
        assert!(touch(&at(c, 0.0, 1.5, 0.25, Quat::IDENTITY), &b).is_none());
    }

    /// A turned box: its faces, not its axis-aligned bounds.
    #[test]
    fn a_sphere_and_a_turned_box() {
        let bx = at(
            Shape::Box { half: Vec3::ONE },
            0.0,
            0.0,
            0.0,
            Quat::from_rotation_y(std::f32::consts::FRAC_PI_4),
        );
        let s = Shape::Sphere { radius: 0.2 };
        // Inside the turned box's √2 reach on x and on z, yet 1.84 out
        // along its own z.
        assert!(touch(&at(s, 1.3, 0.0, 1.3, Quat::IDENTITY), &bx).is_none());
        let hit = touch(&at(s, 1.5, 0.0, 0.0, Quat::IDENTITY), &bx).unwrap();
        assert!((hit.point.x - std::f32::consts::SQRT_2).abs() < 1e-3);
        // The sphere's core inside the box: the nearest face, outward.
        let inside = touch(&at(s, 0.0, 0.9, 0.0, Quat::IDENTITY), &bx).unwrap();
        assert_eq!(inside.normal, Vec3::Y);
    }

    #[test]
    fn a_segment_finds_its_nearest_point_to_a_box_away_from_its_ends() {
        // A capsule lying across the box's top, both ends far outside.
        let bx = at(
            Shape::Box { half: Vec3::ONE },
            0.0,
            0.0,
            0.0,
            Quat::IDENTITY,
        );
        let lying = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
        let rod = |y| {
            at(
                Shape::Capsule {
                    radius: 0.1,
                    height: 10.0,
                },
                0.0,
                y,
                0.0,
                lying,
            )
        };
        assert!(touch(&rod(1.05), &bx).is_some());
        assert!(touch(&rod(1.15), &bx).is_none());
    }

    #[test]
    fn turned_boxes() {
        // Turned on different axes and apart on a diagonal: their bounds
        // overlap on x, y and z, their faces do not.
        let b = Shape::Box { half: Vec3::ONE };
        let a = at(
            b,
            0.0,
            0.0,
            0.0,
            Quat::from_rotation_x(std::f32::consts::FRAC_PI_4),
        );
        let c = at(
            b,
            2.2,
            2.2,
            0.0,
            Quat::from_rotation_z(std::f32::consts::FRAC_PI_4),
        );
        assert!(touch(&c, &a).is_none());
        let d = at(b, 1.5, 0.0, 0.0, Quat::IDENTITY);
        assert_eq!(touch(&d, &a).unwrap().normal, Vec3::X);
    }
}
