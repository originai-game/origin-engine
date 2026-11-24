//! The shapes that carve a navmesh (obstacles) or mark its areas: boxes,
//! boxes turned about +y, upright cylinders and convex prisms, each with
//! the cell test the tile rebuild uses and a byte form the `og.nav` state
//! block and the cooked navmesh share.

use og_core::glam::{Vec2, Vec3};

/// A volume over the navmesh's floor.
#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    /// An axis-aligned box.
    Box { min: Vec3, max: Vec3 },
    /// A box of half sizes `half` round `center`, turned `yaw` radians about +y.
    Oriented { center: Vec3, half: Vec3, yaw: f32 },
    /// An upright cylinder standing on `base`.
    Cylinder {
        base: Vec3,
        radius: f32,
        height: f32,
    },
    /// A convex prism: its outline on x and z (either winding) from `bottom` to `top`.
    Convex {
        points: Vec<Vec2>,
        bottom: f32,
        top: f32,
    },
}

impl Shape {
    /// The axis-aligned box round it.
    pub fn bounds(&self) -> (Vec3, Vec3) {
        match self {
            Shape::Box { min, max } => (*min, *max),
            Shape::Oriented { center, half, yaw } => {
                let (s, c) = og_core::math::sin_cos(*yaw);
                let ex = (half.x * c).abs() + (half.z * s).abs();
                let ez = (half.x * s).abs() + (half.z * c).abs();
                let e = Vec3::new(ex, half.y, ez);
                (*center - e, *center + e)
            }
            Shape::Cylinder {
                base,
                radius,
                height,
            } => (
                *base - Vec3::new(*radius, 0.0, *radius),
                *base + Vec3::new(*radius, *height, *radius),
            ),
            Shape::Convex {
                points,
                bottom,
                top,
            } => {
                let mut lo = Vec2::splat(f32::MAX);
                let mut hi = Vec2::splat(f32::MIN);
                for p in points {
                    lo = lo.min(*p);
                    hi = hi.max(*p);
                }
                if points.is_empty() {
                    (lo, hi) = (Vec2::ZERO, Vec2::ZERO);
                }
                (Vec3::new(lo.x, *bottom, lo.y), Vec3::new(hi.x, *top, hi.y))
            }
        }
    }

    /// Whether the point (x, z) is inside the shape grown by `grow` on x and z.
    pub fn covers(&self, x: f32, z: f32, grow: f32) -> bool {
        match self {
            Shape::Box { min, max } => {
                x >= min.x - grow && x <= max.x + grow && z >= min.z - grow && z <= max.z + grow
            }
            Shape::Oriented { center, half, yaw } => {
                let (s, c) = og_core::math::sin_cos(*yaw);
                let (dx, dz) = (x - center.x, z - center.z);
                // Into the box's frame: the inverse turn.
                let lx = dx * c - dz * s;
                let lz = dx * s + dz * c;
                lx.abs() <= half.x + grow && lz.abs() <= half.z + grow
            }
            Shape::Cylinder { base, radius, .. } => {
                let (dx, dz) = (x - base.x, z - base.z);
                let r = radius + grow;
                dx * dx + dz * dz <= r * r
            }
            Shape::Convex { points, .. } => {
                let n = points.len();
                if n < 3 {
                    return false;
                }
                // The winding from the signed area, so either order works.
                let mut area = 0.0f32;
                for i in 0..n {
                    area += points[i].perp_dot(points[(i + 1) % n]);
                }
                let sign = if area < 0.0 { -1.0 } else { 1.0 };
                let p = Vec2::new(x, z);
                (0..n).all(|i| {
                    let (a, b) = (points[i], points[(i + 1) % n]);
                    let e = b - a;
                    let len = e.length();
                    len == 0.0 || sign * e.perp_dot(p - a) / len >= -grow
                })
            }
        }
    }

    /// Appends its byte form.
    pub fn write(&self, out: &mut Vec<u8>) {
        let f = |out: &mut Vec<u8>, v: f32| out.extend_from_slice(&v.to_le_bytes());
        let v3 = |out: &mut Vec<u8>, v: Vec3| {
            for c in [v.x, v.y, v.z] {
                out.extend_from_slice(&c.to_le_bytes());
            }
        };
        match self {
            Shape::Box { min, max } => {
                out.push(0);
                v3(out, *min);
                v3(out, *max);
            }
            Shape::Oriented { center, half, yaw } => {
                out.push(1);
                v3(out, *center);
                v3(out, *half);
                f(out, *yaw);
            }
            Shape::Cylinder {
                base,
                radius,
                height,
            } => {
                out.push(2);
                v3(out, *base);
                f(out, *radius);
                f(out, *height);
            }
            Shape::Convex {
                points,
                bottom,
                top,
            } => {
                out.push(3);
                f(out, *bottom);
                f(out, *top);
                out.extend_from_slice(&(points.len() as u32).to_le_bytes());
                for p in points {
                    f(out, p.x);
                    f(out, p.y);
                }
            }
        }
    }

    /// Reads one shape off the front of `b`.
    pub fn read(b: &mut &[u8]) -> Option<Shape> {
        fn take<'a>(b: &mut &'a [u8], n: usize) -> Option<&'a [u8]> {
            if b.len() < n {
                return None;
            }
            let (h, t) = b.split_at(n);
            *b = t;
            Some(h)
        }
        let f = |b: &mut &[u8]| Some(f32::from_le_bytes(take(b, 4)?.try_into().ok()?));
        let v3 = |b: &mut &[u8]| Some(Vec3::new(f(b)?, f(b)?, f(b)?));
        Some(match take(b, 1)?[0] {
            0 => Shape::Box {
                min: v3(b)?,
                max: v3(b)?,
            },
            1 => Shape::Oriented {
                center: v3(b)?,
                half: v3(b)?,
                yaw: f(b)?,
            },
            2 => Shape::Cylinder {
                base: v3(b)?,
                radius: f(b)?,
                height: f(b)?,
            },
            3 => {
                let bottom = f(b)?;
                let top = f(b)?;
                let n = u32::from_le_bytes(take(b, 4)?.try_into().ok()?) as usize;
                if n > b.len() / 8 {
                    return None;
                }
                let mut points = Vec::with_capacity(n);
                for _ in 0..n {
                    points.push(Vec2::new(f(b)?, f(b)?));
                }
                Shape::Convex {
                    points,
                    bottom,
                    top,
                }
            }
            _ => return None,
        })
    }
}
