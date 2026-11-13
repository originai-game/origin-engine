//! A drag of one handle: the pose it gives as the pointer moves.

use super::*;

/// A drag of one handle, from the pose it was grabbed at.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Drag {
    pub handle: Handle,
    pub gizmo: Gizmo,
    /// Where it was grabbed, CSS pixels.
    pub grab: Vec2,
    /// The world pose it was grabbed at.
    pub start: Pose,
    /// What the grab hit: an axis's parameter, or a plane's point.
    pub(super) anchor: Anchor,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Anchor {
    /// Along the axis line, world units from the origin.
    Line(f32),
    /// The plane's point under the grab.
    Plane(Vec3, Vec3),
    /// A ring seen face on: turned by the angle on screen about the origin
    /// (`sign` +1 when its axis faces the viewer).
    Screen { origin: Vec2, sign: f32 },
    /// A ring seen edge on: turned by travel along its tangent on screen,
    /// one radius of pixels a radian.
    Tangent { dir: Vec2, radius: f32 },
    /// Uniform scale: by distance from the origin on screen.
    Distance { origin: Vec2, from: f32 },
}

impl Drag {
    /// A drag of `handle` grabbed at `at`; none when the view cannot
    /// carry it (an axis along the line of sight).
    pub fn begin(view: &View, gizmo: Gizmo, handle: Handle, at: Vec2, start: Pose) -> Option<Drag> {
        let o = gizmo.origin;
        let ray = view.ray(at);
        let anchor = match (gizmo.mode, handle) {
            (Mode::Translate | Mode::Scale, Handle::Axis(i)) => {
                Anchor::Line(line_param(ray, o, gizmo.axes[i])?)
            }
            (Mode::Translate, Handle::Plane(i)) => {
                let n = gizmo.axes[i];
                Anchor::Plane(n, plane_hit(ray, o, n)?)
            }
            (Mode::Translate, Handle::View) => {
                let n = view.toward_eye(o);
                Anchor::Plane(n, plane_hit(ray, o, n)?)
            }
            (Mode::Rotate, Handle::Ring(i)) => {
                let n = gizmo.axes[i];
                let facing = n.dot(view.toward_eye(o));
                if facing.abs() >= 0.25 {
                    Anchor::Screen {
                        origin: view.project(o)?,
                        sign: facing.signum(),
                    }
                } else {
                    // The tangent at the ring's point nearest the grab.
                    let (r0, rd) = ray;
                    let hit = plane_hit((r0, rd), o, view.toward_eye(o)).unwrap_or(o);
                    let radial = (hit - o) - n * n.dot(hit - o);
                    let radial = radial.normalize_or(gizmo.ring_side(view, n));
                    let p = o + radial * gizmo.length;
                    let tangent = n.cross(radial);
                    let a = view.project(p)?;
                    let b = view.project(p + tangent * gizmo.length * 0.1)?;
                    let dir = (b - a).normalize_or_zero();
                    if dir == Vec2::ZERO {
                        return None;
                    }
                    let radius = view
                        .project(o)
                        .map_or(PIXELS, |c| c.distance(a).max(PIXELS * 0.5));
                    Anchor::Tangent { dir, radius }
                }
            }
            (Mode::Rotate, Handle::View) => Anchor::Screen {
                origin: view.project(o)?,
                sign: 1.0,
            },
            (Mode::Scale, Handle::View) => {
                let origin = view.project(o)?;
                Anchor::Distance {
                    origin,
                    from: at.distance(origin).max(4.0),
                }
            }
            _ => return None,
        };
        Some(Drag {
            handle,
            gizmo,
            grab: at,
            start,
            anchor,
        })
    }

    /// The world pose with the pointer at `at`, snapped with `snap`.
    pub fn pose(&self, view: &View, at: Vec2, snap: bool) -> Pose {
        let (p0, q0, s0) = self.start;
        let g = &self.gizmo;
        let ray = view.ray(at);
        match (g.mode, self.handle, self.anchor) {
            (Mode::Translate, Handle::Axis(i), Anchor::Line(t0)) => {
                let Some(t) = line_param(ray, g.origin, g.axes[i]) else {
                    return self.start;
                };
                let d = step(t - t0, snap.then_some(SNAP_MOVE));
                (p0 + g.axes[i] * d, q0, s0)
            }
            (Mode::Translate, _, Anchor::Plane(n, h0)) => {
                let Some(h) = plane_hit(ray, g.origin, n) else {
                    return self.start;
                };
                let mut d = h - h0;
                if snap {
                    // Snapped along the gizmo's axes in the plane.
                    d = g
                        .axes
                        .iter()
                        .filter(|a| a.dot(n).abs() < 0.5)
                        .map(|a| *a * step(a.dot(d), Some(SNAP_MOVE)))
                        .sum::<Vec3>();
                    if let Handle::View = self.handle {
                        d = d - n * n.dot(d);
                    }
                }
                (p0 + d, q0, s0)
            }
            (Mode::Rotate, h, anchor) => {
                let axis = match h {
                    Handle::Ring(i) => g.axes[i],
                    _ => view.toward_eye(g.origin),
                };
                let angle = match anchor {
                    Anchor::Screen { origin, sign } => {
                        let a = self.grab - origin;
                        let b = at - origin;
                        if a.length() < 1.0 || b.length() < 1.0 {
                            0.0
                        } else {
                            // Screen y points down: a turn the viewer sees
                            // anticlockwise is a negative angle on screen.
                            -sign * wrap(atan2(b.y, b.x) - atan2(a.y, a.x))
                        }
                    }
                    Anchor::Tangent { dir, radius } => (at - self.grab).dot(dir) / radius,
                    _ => 0.0,
                };
                let deg = step(angle.to_degrees(), snap.then_some(SNAP_TURN));
                let turn = og_core::math::quat_from_axis_angle(axis, deg.to_radians());
                (p0, (turn * q0).normalize(), s0)
            }
            (Mode::Scale, Handle::Axis(i), Anchor::Line(t0)) => {
                let Some(t) = line_param(ray, g.origin, g.axes[i]) else {
                    return self.start;
                };
                let f = factor(t / if t0.abs() < 1e-6 { 1e-6 } else { t0 }, snap);
                let mut s = s0;
                s[i] *= f;
                (p0, q0, s)
            }
            (Mode::Scale, Handle::View, Anchor::Distance { origin, from }) => {
                let f = factor(at.distance(origin) / from, snap);
                (p0, q0, s0 * f)
            }
            _ => self.start,
        }
    }
}

/// A factor snapped to [`SNAP_SCALE`] steps, never below 1 %.
pub(super) fn factor(f: f32, snap: bool) -> f32 {
    step(f, snap.then_some(SNAP_SCALE)).max(0.01)
}

/// `v` to the nearest multiple of `step`, or as it is.
pub(super) fn step(v: f32, step: Option<f32>) -> f32 {
    match step {
        Some(s) if s > 0.0 => (v / s).round() * s,
        _ => v,
    }
}

/// An angle into -π..π.
pub(super) fn wrap(a: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    let a = a.rem_euclid(tau);
    if a > std::f32::consts::PI { a - tau } else { a }
}
