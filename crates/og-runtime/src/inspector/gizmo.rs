//! The inspector's transform gizmo (docs/design.md §3.6): translate,
//! rotate and scale handles on the selected entity, in world or local
//! space, at a constant size on screen, dragged by rays from the camera.
//! No UI and no engine in it: the view, the handles, what a pointer hits
//! and what a drag makes of the pose, so every rule is a unit test.
//!
//! - **Translate**: an arrow per axis (the pointer's ray against the
//!   axis's line), a square per plane (against that plane) and a square
//!   at the origin that moves in the view's plane.
//! - **Rotate**: a ring across each axis, and one facing the view. A ring
//!   seen face on turns by the pointer's angle about the origin on
//!   screen; one seen edge on by the pointer's travel along its tangent.
//! - **Scale**: always the entity's own axes (og.Transform's scale is
//!   local); an axis by the ratio of the pointer's reach along it to the
//!   reach it was grabbed at, the origin's cube uniformly by distance on
//!   screen.
//! - **Snap** (Ctrl held): a move by [`SNAP_MOVE`], a turn by
//!   [`SNAP_TURN`] degrees, a scale by [`SNAP_SCALE`] of the factor.

use glam::{Mat4, Quat, Vec2, Vec3};
use og_core::math::atan2;

/// A handle's length on screen, CSS pixels.
pub const PIXELS: f32 = 96.0;
/// How near the pointer must be to a handle, CSS pixels.
pub const REACH: f32 = 8.0;
/// Snapped steps: world units, degrees, and a scale factor's step.
pub const SNAP_MOVE: f32 = 0.5;
pub const SNAP_TURN: f32 = 15.0;
pub const SNAP_SCALE: f32 = 0.1;

/// Where an arrow's shaft starts and a plane square spans, in handle lengths.
const SHAFT: f32 = 0.18;
const SQUARE: (f32, f32) = (0.22, 0.42);
/// The view ring's radius and the origin square's half size.
const VIEW_RING: f32 = 1.2;
const CENTRE: f32 = 0.07;

/// What the gizmo does.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Translate,
    Rotate,
    Scale,
}

impl Mode {
    pub fn name(self) -> &'static str {
        match self {
            Mode::Translate => "translate",
            Mode::Rotate => "rotate",
            Mode::Scale => "scale",
        }
    }

    /// The key that picks it (`KeyboardEvent.code`): W, E and R.
    pub fn of_key(code: &str) -> Option<Mode> {
        match code {
            "KeyW" => Some(Mode::Translate),
            "KeyE" => Some(Mode::Rotate),
            "KeyR" => Some(Mode::Scale),
            _ => None,
        }
    }

    /// The og.Transform field a drag of it writes.
    pub fn field(self) -> &'static str {
        match self {
            Mode::Translate => "position",
            Mode::Rotate => "quaternion",
            Mode::Scale => "scale",
        }
    }
}

/// The axes translate and rotate follow (scale always the entity's own).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Space {
    #[default]
    World,
    Local,
}

impl Space {
    pub fn name(self) -> &'static str {
        match self {
            Space::World => "world",
            Space::Local => "local",
        }
    }
}

/// One handle of the gizmo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Handle {
    /// Along axis 0, 1 or 2 (an arrow, a scale bar).
    Axis(usize),
    /// In the plane across axis 0, 1 or 2 (translate's squares).
    Plane(usize),
    /// About axis 0, 1 or 2 (rotate's rings).
    Ring(usize),
    /// The view's own: translate in its plane, rotate about its line of
    /// sight, scale uniformly.
    View,
}

impl Handle {
    pub fn name(self) -> String {
        const AXES: [&str; 3] = ["x", "y", "z"];
        const PLANES: [&str; 3] = ["yz", "zx", "xy"];
        match self {
            Handle::Axis(i) => AXES[i].to_string(),
            Handle::Plane(i) => PLANES[i].to_string(),
            Handle::Ring(i) => format!("ring-{}", AXES[i]),
            Handle::View => "view".into(),
        }
    }
}

/// A pose: position, rotation, scale (og.Transform's, or in the world).
pub type Pose = (Vec3, Quat, Vec3);

/// The camera as the pointer sees it: projection times view on a surface
/// of `size` CSS pixels.
#[derive(Clone, Copy, Debug)]
pub struct View {
    pub view_proj: Mat4,
    pub inverse: Mat4,
    pub size: Vec2,
    pub eye: Vec3,
    pub forward: Vec3,
    pub orthographic: bool,
}

impl View {
    pub fn new(view_proj: Mat4, size: Vec2, eye: Vec3, forward: Vec3, orthographic: bool) -> View {
        View {
            view_proj,
            inverse: view_proj.inverse(),
            size: size.max(Vec2::ONE),
            eye,
            forward: forward.normalize_or(Vec3::NEG_Z),
            orthographic,
        }
    }

    /// A world point on the surface, CSS pixels from the top left; none
    /// behind the camera.
    pub fn project(&self, p: Vec3) -> Option<Vec2> {
        let c = self.view_proj * p.extend(1.0);
        if c.w <= 1e-6 {
            return None;
        }
        let ndc = c.truncate() / c.w;
        Some(Vec2::new(
            (ndc.x + 1.0) * 0.5 * self.size.x,
            (1.0 - ndc.y) * 0.5 * self.size.y,
        ))
    }

    /// The ray under a surface point: its origin on the near plane and
    /// its unit direction.
    pub fn ray(&self, at: Vec2) -> (Vec3, Vec3) {
        let nx = at.x / self.size.x * 2.0 - 1.0;
        let ny = 1.0 - at.y / self.size.y * 2.0;
        let near = self.inverse.project_point3(Vec3::new(nx, ny, 0.0));
        let far = self.inverse.project_point3(Vec3::new(nx, ny, 1.0));
        (near, (far - near).normalize_or(self.forward))
    }

    /// The direction from `p` to the viewer.
    pub fn toward_eye(&self, p: Vec3) -> Vec3 {
        if self.orthographic {
            -self.forward
        } else {
            (self.eye - p).normalize_or(-self.forward)
        }
    }

    /// The world length that spans `pixels` on screen at `p`, across the
    /// line of sight.
    pub fn length_at(&self, p: Vec3, pixels: f32) -> f32 {
        let side = self.toward_eye(p).any_orthonormal_vector();
        match (self.project(p), self.project(p + side)) {
            (Some(a), Some(b)) if a.distance(b) > 1e-6 => pixels / a.distance(b),
            _ => 1.0,
        }
    }
}

/// The gizmo on one entity this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gizmo {
    pub mode: Mode,
    pub origin: Vec3,
    /// Its three axes in the world, unit.
    pub axes: [Vec3; 3],
    /// A handle's length in world units ([`PIXELS`] on screen).
    pub length: f32,
}

impl Gizmo {
    /// The gizmo at an entity's world position and rotation.
    pub fn new(view: &View, mode: Mode, space: Space, origin: Vec3, rotation: Quat) -> Gizmo {
        let own = [rotation * Vec3::X, rotation * Vec3::Y, rotation * Vec3::Z];
        let axes = if mode == Mode::Scale || space == Space::Local {
            own.map(|a| a.normalize_or(Vec3::X))
        } else {
            [Vec3::X, Vec3::Y, Vec3::Z]
        };
        Gizmo {
            mode,
            origin,
            axes,
            length: view.length_at(origin, PIXELS),
        }
    }

    /// The handles its mode has.
    pub fn handles(&self) -> Vec<Handle> {
        match self.mode {
            Mode::Translate => (0..3)
                .map(Handle::Plane)
                .chain((0..3).map(Handle::Axis))
                .chain([Handle::View])
                .collect(),
            Mode::Rotate => (0..3).map(Handle::Ring).chain([Handle::View]).collect(),
            Mode::Scale => [Handle::View]
                .into_iter()
                .chain((0..3).map(Handle::Axis))
                .collect(),
        }
    }

    /// The two axes across axis `i`.
    fn across(&self, i: usize) -> (Vec3, Vec3) {
        (self.axes[(i + 1) % 3], self.axes[(i + 2) % 3])
    }

    /// The unit vector in a ring's plane that points across the line of
    /// sight: where the ring reaches furthest sideways on screen.
    fn ring_side(&self, view: &View, n: Vec3) -> Vec3 {
        let s = n.cross(view.toward_eye(self.origin));
        if s.length_squared() > 1e-6 {
            s.normalize()
        } else {
            n.any_orthonormal_vector()
        }
    }

    /// A point a pointer grabs the handle at (what `dump_inspector`
    /// reports, and a script presses).
    pub fn grab_point(&self, view: &View, h: Handle) -> Vec3 {
        let (o, l) = (self.origin, self.length);
        match h {
            Handle::Axis(i) => {
                o + self.axes[i] * l * if self.mode == Mode::Scale { 0.95 } else { 0.8 }
            }
            Handle::Plane(i) => {
                let (u, v) = self.across(i);
                let m = (SQUARE.0 + SQUARE.1) * 0.5 * l;
                o + (u + v) * m
            }
            Handle::Ring(i) => o + self.ring_side(view, self.axes[i]) * l,
            Handle::View => match self.mode {
                Mode::Rotate => {
                    let side = view.toward_eye(o).any_orthonormal_vector();
                    o + side * l * VIEW_RING
                }
                _ => o,
            },
        }
    }

    /// The handle under a surface point, the nearest when several are in
    /// reach (a plane's square or the origin's before the arrows over it).
    pub fn hit(&self, view: &View, at: Vec2) -> Option<Handle> {
        let o2 = view.project(self.origin)?;
        let (o, l) = (self.origin, self.length);
        let mut best: Option<(f32, Handle)> = None;
        let mut offer = |d: f32, h: Handle| {
            if d <= REACH && best.is_none_or(|(b, _)| d < b) {
                best = Some((d, h));
            }
        };
        // An axis seen end on has no length on screen to grab.
        let visible = |a: Vec3| {
            view.project(o + a * l)
                .is_some_and(|e| e.distance(o2) > PIXELS * 0.15)
        };
        match self.mode {
            Mode::Translate => {
                if at.distance(o2) <= PIXELS * CENTRE + REACH * 0.5 {
                    return Some(Handle::View);
                }
                for i in 0..3 {
                    let (u, v) = self.across(i);
                    let q = [
                        o + (u * SQUARE.0 + v * SQUARE.0) * l,
                        o + (u * SQUARE.1 + v * SQUARE.0) * l,
                        o + (u * SQUARE.1 + v * SQUARE.1) * l,
                        o + (u * SQUARE.0 + v * SQUARE.1) * l,
                    ];
                    if let [Some(a), Some(b), Some(c), Some(d)] = q.map(|p| view.project(p))
                        && inside(at, [a, b, c, d])
                        && area([a, b, c, d]) > 16.0
                    {
                        return Some(Handle::Plane(i));
                    }
                }
                for (i, &a) in self.axes.iter().enumerate() {
                    if !visible(a) {
                        continue;
                    }
                    if let (Some(p), Some(q)) =
                        (view.project(o + a * l * SHAFT), view.project(o + a * l))
                    {
                        offer(segment_distance(at, p, q), Handle::Axis(i));
                    }
                }
            }
            Mode::Rotate => {
                for (i, &axis) in self.axes.iter().enumerate() {
                    let pts = ring(o, axis, l, 64);
                    if let Some(d) = polyline_distance(view, at, &pts) {
                        offer(d, Handle::Ring(i));
                    }
                }
                let r = view
                    .project(self.grab_point(view, Handle::View))
                    .map_or(PIXELS * VIEW_RING, |p| p.distance(o2));
                offer((at.distance(o2) - r).abs(), Handle::View);
            }
            Mode::Scale => {
                if at.distance(o2) <= PIXELS * CENTRE * 1.5 + REACH * 0.5 {
                    return Some(Handle::View);
                }
                for (i, &a) in self.axes.iter().enumerate() {
                    if !visible(a) {
                        continue;
                    }
                    if let (Some(p), Some(q)) =
                        (view.project(o + a * l * SHAFT), view.project(o + a * l))
                    {
                        offer(segment_distance(at, p, q), Handle::Axis(i));
                    }
                }
            }
        }
        best.map(|(_, h)| h)
    }

    /// Its strokes in the world, each `(points as segment pairs, colour,
    /// width in CSS pixels)`: `colors` by axis then the view's, `lit` the
    /// hovered or dragged handle's.
    #[allow(clippy::needless_range_loop)]
    pub fn strokes(
        &self,
        view: &View,
        colors: [[f32; 4]; 4],
        lit: Option<Handle>,
        lit_color: [f32; 4],
    ) -> Vec<(Vec<Vec3>, [f32; 4], f32)> {
        let (o, l) = (self.origin, self.length);
        let mut out = Vec::new();
        let color = |h: Handle, c: [f32; 4]| if lit == Some(h) { lit_color } else { c };
        let width = |h: Handle, w: f32| if lit == Some(h) { w + 1.5 } else { w };
        let eye = view.toward_eye(o);
        let side = eye.any_orthonormal_vector();
        let up = eye.cross(side);
        let square = |half: f32| {
            let c = [
                o + (side + up) * half,
                o + (up - side) * half,
                o - (side + up) * half,
                o + (side - up) * half,
            ];
            (0..4)
                .flat_map(|k| [c[k], c[(k + 1) % 4]])
                .collect::<Vec<_>>()
        };
        match self.mode {
            Mode::Translate => {
                for i in 0..3 {
                    let a = self.axes[i];
                    let h = Handle::Axis(i);
                    out.push((
                        og_render::debug::arrow(o + a * l * SHAFT, o + a * l, l * 0.2),
                        color(h, colors[i]),
                        width(h, 2.5),
                    ));
                    let (u, v) = self.across(i);
                    let q = [
                        o + (u * SQUARE.0 + v * SQUARE.0) * l,
                        o + (u * SQUARE.1 + v * SQUARE.0) * l,
                        o + (u * SQUARE.1 + v * SQUARE.1) * l,
                        o + (u * SQUARE.0 + v * SQUARE.1) * l,
                    ];
                    let h = Handle::Plane(i);
                    out.push((
                        (0..4).flat_map(|k| [q[k], q[(k + 1) % 4]]).collect(),
                        color(h, colors[i]),
                        width(h, 1.5),
                    ));
                }
                out.push((
                    square(l * CENTRE),
                    color(Handle::View, colors[3]),
                    width(Handle::View, 1.5),
                ));
            }
            Mode::Rotate => {
                for i in 0..3 {
                    let h = Handle::Ring(i);
                    let pts = ring(o, self.axes[i], l, 64);
                    out.push((
                        (0..pts.len())
                            .flat_map(|k| [pts[k], pts[(k + 1) % pts.len()]])
                            .collect(),
                        color(h, colors[i]),
                        width(h, 2.0),
                    ));
                }
                let pts = ring(o, eye, l * VIEW_RING, 72);
                out.push((
                    (0..pts.len())
                        .flat_map(|k| [pts[k], pts[(k + 1) % pts.len()]])
                        .collect(),
                    color(Handle::View, colors[3]),
                    width(Handle::View, 1.5),
                ));
            }
            Mode::Scale => {
                let frame = |at: Vec3| {
                    Mat4::from_cols(
                        self.axes[0].extend(0.0),
                        self.axes[1].extend(0.0),
                        self.axes[2].extend(0.0),
                        at.extend(1.0),
                    )
                };
                for i in 0..3 {
                    let a = self.axes[i];
                    let h = Handle::Axis(i);
                    let mut pts = vec![o + a * l * SHAFT, o + a * l];
                    pts.extend(og_render::debug::box_edges(
                        frame(o + a * l),
                        Vec3::splat(l * 0.05),
                    ));
                    out.push((pts, color(h, colors[i]), width(h, 2.5)));
                }
                out.push((
                    og_render::debug::box_edges(frame(o), Vec3::splat(l * CENTRE)),
                    color(Handle::View, colors[3]),
                    width(Handle::View, 1.5),
                ));
            }
        }
        out
    }
}

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
    anchor: Anchor,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Anchor {
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
fn factor(f: f32, snap: bool) -> f32 {
    step(f, snap.then_some(SNAP_SCALE)).max(0.01)
}

/// `v` to the nearest multiple of `step`, or as it is.
fn step(v: f32, step: Option<f32>) -> f32 {
    match step {
        Some(s) if s > 0.0 => (v / s).round() * s,
        _ => v,
    }
}

/// An angle into -π..π.
fn wrap(a: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    let a = a.rem_euclid(tau);
    if a > std::f32::consts::PI { a - tau } else { a }
}

/// The parameter along the line `o + axis·t` nearest the ray; none when
/// the ray runs along it.
pub fn line_param((r0, rd): (Vec3, Vec3), o: Vec3, axis: Vec3) -> Option<f32> {
    let b = axis.dot(rd);
    let denom = 1.0 - b * b;
    if denom < 1e-5 {
        return None;
    }
    let w = o - r0;
    let d = axis.dot(w);
    let e = rd.dot(w);
    Some((b * e - d) / denom)
}

/// Where the ray meets the plane through `o` across `n`; none when it runs
/// along the plane or meets it behind the camera.
pub fn plane_hit((r0, rd): (Vec3, Vec3), o: Vec3, n: Vec3) -> Option<Vec3> {
    let den = rd.dot(n);
    if den.abs() < 1e-4 {
        return None;
    }
    let t = (o - r0).dot(n) / den;
    (t >= 0.0).then(|| r0 + rd * t)
}

/// A ring of `n` points about `o` across `axis`.
fn ring(o: Vec3, axis: Vec3, radius: f32, n: usize) -> Vec<Vec3> {
    let a = axis.normalize_or(Vec3::Y);
    let u = a.any_orthonormal_vector();
    let v = a.cross(u);
    (0..n)
        .map(|k| {
            let t = k as f32 / n as f32 * std::f32::consts::TAU;
            let (sin, cos) = og_core::math::sin_cos(t);
            o + (u * cos + v * sin) * radius
        })
        .collect()
}

/// The nearest a closed polyline comes to `at` on screen.
fn polyline_distance(view: &View, at: Vec2, pts: &[Vec3]) -> Option<f32> {
    let s: Vec<Vec2> = pts.iter().filter_map(|p| view.project(*p)).collect();
    if s.len() < 2 {
        return None;
    }
    (0..s.len())
        .map(|k| segment_distance(at, s[k], s[(k + 1) % s.len()]))
        .reduce(f32::min)
}

/// The distance from `p` to the segment `a`–`b`.
pub fn segment_distance(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let len = ab.length_squared();
    let t = if len > 1e-9 {
        ((p - a).dot(ab) / len).clamp(0.0, 1.0)
    } else {
        0.0
    };
    p.distance(a + ab * t)
}

/// Whether `p` is inside the convex quad `q` (either winding).
fn inside(p: Vec2, q: [Vec2; 4]) -> bool {
    let mut sign = 0.0;
    for k in 0..4 {
        let (a, b) = (q[k], q[(k + 1) % 4]);
        let c = (b - a).perp_dot(p - a);
        if c.abs() < 1e-6 {
            continue;
        }
        if sign == 0.0 {
            sign = c.signum();
        } else if c.signum() != sign {
            return false;
        }
    }
    true
}

/// A quad's area on screen (a square seen edge on has none to grab).
fn area(q: [Vec2; 4]) -> f32 {
    let mut a = 0.0;
    for k in 0..4 {
        a += q[k].perp_dot(q[(k + 1) % 4]);
    }
    (a * 0.5).abs()
}

/// A world pose under a parent's world pose as og.Transform holds it: the
/// inverse of og_core::math::world_pose's one step (exact for a uniform
/// parent scale).
pub fn to_local(parent: Option<Pose>, (p, q, s): Pose) -> Pose {
    let Some((pp, pq, ps)) = parent else {
        return (p, q, s);
    };
    let inv = pq.inverse();
    let safe = |v: f32| if v.abs() < 1e-9 { 1e-9 } else { v };
    let ps = Vec3::new(safe(ps.x), safe(ps.y), safe(ps.z));
    ((inv * (p - pp)) / ps, (inv * q).normalize(), s)
}

/// og.Camera's view as og.camera.worldToScreen projects it (the og.Camera
/// resource's JSON, on a `w`×`h` surface).
pub fn view_of(camera: &serde_json::Value, w: f32, h: f32) -> View {
    let v = crate::api::camera_view(camera, w, h);
    View::new(
        v.view_proj,
        Vec2::new(w, h),
        v.eye,
        v.forward,
        v.orthographic,
    )
}

/// The eight corners' box of an instance's mesh bounds, as segments.
pub fn box_lines(transform: Mat4, min: Vec3, max: Vec3) -> Vec<Vec3> {
    let centre = (min + max) * 0.5;
    og_render::debug::box_edges(
        transform * Mat4::from_translation(centre),
        (max - min) * 0.5,
    )
}

#[cfg(test)]
// Test cameras from glam's own projections.
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;

    /// A perspective camera at (0, 0, 10) looking down -z, 60° tall, on a
    /// 1000×500 surface.
    fn persp() -> View {
        let proj =
            glam::camera::rh::proj::directx::perspective(60f32.to_radians(), 2.0, 0.1, 100.0);
        let view =
            glam::camera::rh::view::look_at_mat4(Vec3::new(0.0, 0.0, 10.0), Vec3::ZERO, Vec3::Y);
        View::new(
            proj * view,
            Vec2::new(1000.0, 500.0),
            Vec3::new(0.0, 0.0, 10.0),
            Vec3::NEG_Z,
            false,
        )
    }

    /// The same camera looking from (6, 5, 8) at the origin: no axis end on.
    fn angled() -> View {
        let eye = Vec3::new(6.0, 5.0, 8.0);
        let proj =
            glam::camera::rh::proj::directx::perspective(60f32.to_radians(), 2.0, 0.1, 100.0);
        let view = glam::camera::rh::view::look_at_mat4(eye, Vec3::ZERO, Vec3::Y);
        View::new(
            proj * view,
            Vec2::new(1000.0, 500.0),
            eye,
            -eye.normalize(),
            false,
        )
    }

    fn at(view: &View, p: Vec3) -> Vec2 {
        view.project(p).unwrap()
    }

    const ID: Pose = (Vec3::ZERO, Quat::IDENTITY, Vec3::ONE);

    #[test]
    fn the_handles_keep_their_size_on_screen_at_any_distance() {
        let v = persp();
        for z in [5.0, 0.0, -20.0] {
            let g = Gizmo::new(
                &v,
                Mode::Translate,
                Space::World,
                Vec3::new(1.0, 0.5, z),
                Quat::IDENTITY,
            );
            let o = at(&v, g.origin);
            let x = at(&v, g.origin + Vec3::X * g.length);
            assert!(
                (o.distance(x) - PIXELS).abs() < PIXELS * 0.03,
                "at z {z}: {}",
                o.distance(x)
            );
        }
        // Further away is longer in the world.
        let near = Gizmo::new(
            &v,
            Mode::Translate,
            Space::World,
            Vec3::new(0.0, 0.0, 5.0),
            Quat::IDENTITY,
        );
        let far = Gizmo::new(
            &v,
            Mode::Translate,
            Space::World,
            Vec3::new(0.0, 0.0, -20.0),
            Quat::IDENTITY,
        );
        assert!(far.length > near.length * 4.0);
    }

    #[test]
    fn local_space_turns_translate_and_rotate_but_scale_is_always_local() {
        let v = angled();
        let q = Quat::from_rotation_y(90f32.to_radians());
        let world = Gizmo::new(&v, Mode::Translate, Space::World, Vec3::ZERO, q);
        assert!(world.axes[0].abs_diff_eq(Vec3::X, 1e-5));
        let local = Gizmo::new(&v, Mode::Translate, Space::Local, Vec3::ZERO, q);
        assert!(
            local.axes[0].abs_diff_eq(Vec3::NEG_Z, 1e-5),
            "{:?}",
            local.axes[0]
        );
        let scale = Gizmo::new(&v, Mode::Scale, Space::World, Vec3::ZERO, q);
        assert!(scale.axes[0].abs_diff_eq(Vec3::NEG_Z, 1e-5));
    }

    #[test]
    fn every_handles_grab_point_hits_that_handle() {
        let v = angled();
        for mode in [Mode::Translate, Mode::Rotate, Mode::Scale] {
            for space in [Space::World, Space::Local] {
                let g = Gizmo::new(
                    &v,
                    mode,
                    space,
                    Vec3::new(0.5, 0.0, -1.0),
                    Quat::from_rotation_z(0.4),
                );
                for h in g.handles() {
                    let p = at(&v, g.grab_point(&v, h));
                    assert_eq!(g.hit(&v, p), Some(h), "{mode:?} {space:?} {h:?} at {p}");
                }
                // Far from it, nothing.
                let o = at(&v, g.origin);
                assert_eq!(g.hit(&v, o + Vec2::new(300.0, 160.0)), None);
            }
        }
    }

    #[test]
    fn an_axis_seen_end_on_is_not_grabbed() {
        // The camera looks down z: the z arrow is a point on screen.
        let v = persp();
        let g = Gizmo::new(
            &v,
            Mode::Translate,
            Space::World,
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let tip = at(&v, g.origin + Vec3::Z * g.length * 0.8);
        assert_ne!(g.hit(&v, tip), Some(Handle::Axis(2)));
        assert!(Drag::begin(&v, g, Handle::Axis(2), tip, ID).is_none());
    }

    #[test]
    fn an_arrow_follows_the_pointer_along_its_axis_only() {
        let v = angled();
        let g = Gizmo::new(
            &v,
            Mode::Translate,
            Space::World,
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let grab = at(&v, g.grab_point(&v, Handle::Axis(0)));
        let d = Drag::begin(&v, g, Handle::Axis(0), grab, ID).unwrap();
        // The pointer where the world point x = 2 would be, then pulled
        // off the axis on screen: the pose moves along x alone.
        let target = at(&v, Vec3::new(2.0, 0.0, 0.0) + Vec3::X * g.length * 0.8);
        let (p, q, s) = d.pose(&v, target, false);
        assert!(p.abs_diff_eq(Vec3::new(2.0, 0.0, 0.0), 1e-3), "{p}");
        assert_eq!((q, s), (Quat::IDENTITY, Vec3::ONE));
        let (p, _, _) = d.pose(&v, target + Vec2::new(0.0, 60.0), false);
        assert!(p.y == 0.0 && p.z == 0.0 && p.x != 0.0, "{p}");
        // Snapped: whole steps of SNAP_MOVE.
        let near = at(&v, Vec3::new(1.13, 0.0, 0.0) + Vec3::X * g.length * 0.8);
        let (p, _, _) = d.pose(&v, near, true);
        assert!((p.x - 1.0).abs() < 1e-5, "{p}");
        let (p, _, _) = d.pose(&v, near, false);
        assert!((p.x - 1.13).abs() < 1e-3, "{p}");
    }

    #[test]
    fn a_plane_square_moves_in_its_plane_and_the_centre_in_the_views() {
        let v = angled();
        let g = Gizmo::new(
            &v,
            Mode::Translate,
            Space::World,
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        // The xz plane is across y (Plane(1)).
        let grab = at(&v, g.grab_point(&v, Handle::Plane(1)));
        let d = Drag::begin(&v, g, Handle::Plane(1), grab, ID).unwrap();
        let start = g.grab_point(&v, Handle::Plane(1));
        let to = at(&v, start + Vec3::new(1.5, 0.0, -0.8));
        let (p, _, _) = d.pose(&v, to, false);
        assert!(p.abs_diff_eq(Vec3::new(1.5, 0.0, -0.8), 1e-3), "{p}");
        let (p, _, _) = d.pose(&v, to, true);
        assert!(p.abs_diff_eq(Vec3::new(1.5, 0.0, -1.0), 1e-5), "{p}");
        // The centre: what it moves to stays in the plane facing the eye.
        let grab = at(&v, Vec3::ZERO);
        let d = Drag::begin(&v, g, Handle::View, grab, ID).unwrap();
        let (p, _, _) = d.pose(&v, grab + Vec2::new(50.0, -20.0), false);
        assert!(
            p.dot(v.toward_eye(Vec3::ZERO)).abs() < 1e-3 && p.length() > 0.1,
            "{p}"
        );
    }

    #[test]
    fn a_ring_face_on_turns_by_the_pointers_angle_and_snaps_to_fifteen() {
        let v = persp();
        let g = Gizmo::new(&v, Mode::Rotate, Space::World, Vec3::ZERO, Quat::IDENTITY);
        // The z ring faces the camera: from its right edge to its top is a
        // quarter turn anticlockwise to the viewer, +90° about z.
        let o = at(&v, Vec3::ZERO);
        let r = PIXELS;
        let d = Drag::begin(&v, g, Handle::Ring(2), o + Vec2::new(r, 0.0), ID).unwrap();
        let (_, q, _) = d.pose(&v, o + Vec2::new(0.0, -r), false);
        let (axis, angle) = q.to_axis_angle();
        assert!(
            axis.abs_diff_eq(Vec3::Z, 1e-4) && (angle.to_degrees() - 90.0).abs() < 0.1,
            "{axis} {}",
            angle.to_degrees()
        );
        // 20° on screen snaps to 15°.
        let a = 20f32.to_radians();
        let (_, q, _) = d.pose(&v, o + Vec2::new(a.cos(), -a.sin()) * r, true);
        assert!((q.to_axis_angle().1.to_degrees() - 15.0).abs() < 0.01);
    }

    #[test]
    fn a_ring_edge_on_turns_by_travel_along_its_tangent() {
        let v = persp();
        let g = Gizmo::new(&v, Mode::Rotate, Space::World, Vec3::ZERO, Quat::IDENTITY);
        // The y ring is seen edge on (a flat line across the screen).
        let grab = at(&v, g.grab_point(&v, Handle::Ring(1)));
        let d = Drag::begin(&v, g, Handle::Ring(1), grab, ID).unwrap();
        let Anchor::Tangent { dir, radius } = d.anchor else {
            panic!("{:?}", d.anchor);
        };
        let (_, q, _) = d.pose(&v, grab + dir * radius * 0.5, false);
        let (axis, angle) = q.to_axis_angle();
        assert!(axis.abs_diff_eq(Vec3::Y, 1e-4), "{axis}");
        assert!((angle - 0.5).abs() < 1e-3, "{angle}");
        // The other way turns back.
        let (_, back, _) = d.pose(&v, grab - dir * radius * 0.5, false);
        assert!(
            (back * q).abs_diff_eq(Quat::IDENTITY, 1e-4)
                || (back * q).abs_diff_eq(-Quat::IDENTITY, 1e-4)
        );
    }

    #[test]
    fn a_turn_keeps_what_the_entity_was_turned_by_already() {
        let v = persp();
        let q0 = Quat::from_rotation_x(0.3);
        let g = Gizmo::new(&v, Mode::Rotate, Space::World, Vec3::ZERO, q0);
        let o = at(&v, Vec3::ZERO);
        let d = Drag::begin(
            &v,
            g,
            Handle::Ring(2),
            o + Vec2::new(PIXELS, 0.0),
            (Vec3::ZERO, q0, Vec3::ONE),
        )
        .unwrap();
        let (_, q, _) = d.pose(&v, o + Vec2::new(0.0, -PIXELS), false);
        let want = Quat::from_rotation_z(90f32.to_radians()) * q0;
        assert!(q.abs_diff_eq(want, 1e-3), "{q} {want}");
    }

    #[test]
    fn scale_by_the_reach_along_an_axis_and_uniformly_by_distance() {
        let v = angled();
        let s0 = Vec3::new(2.0, 1.0, 1.0);
        let g = Gizmo::new(&v, Mode::Scale, Space::World, Vec3::ZERO, Quat::IDENTITY);
        let grab_world = g.grab_point(&v, Handle::Axis(0));
        let d = Drag::begin(
            &v,
            g,
            Handle::Axis(0),
            at(&v, grab_world),
            (Vec3::ZERO, Quat::IDENTITY, s0),
        )
        .unwrap();
        // Pulled to 1.5 times the reach it was grabbed at: x scale × 1.5.
        let (_, _, s) = d.pose(&v, at(&v, grab_world * 1.5), false);
        assert!(s.abs_diff_eq(Vec3::new(3.0, 1.0, 1.0), 1e-3), "{s}");
        let (_, _, s) = d.pose(&v, at(&v, grab_world * 1.53), true);
        assert!(s.abs_diff_eq(Vec3::new(3.0, 1.0, 1.0), 1e-4), "{s}");
        // Through the origin it stops at 1 %.
        let (_, _, s) = d.pose(&v, at(&v, -grab_world), false);
        assert!((s.x - 0.02).abs() < 1e-5, "{s}");
        // The centre: twice as far on screen doubles every axis.
        let o = at(&v, Vec3::ZERO);
        let d = Drag::begin(
            &v,
            g,
            Handle::View,
            o + Vec2::new(10.0, 0.0),
            (Vec3::ZERO, Quat::IDENTITY, s0),
        )
        .unwrap();
        let (_, _, s) = d.pose(&v, o + Vec2::new(0.0, 20.0), false);
        assert!(s.abs_diff_eq(s0 * 2.0, 1e-4), "{s}");
    }

    #[test]
    fn a_world_pose_goes_back_under_its_parent() {
        let parent: Pose = (
            Vec3::new(1.0, 2.0, 0.0),
            Quat::from_rotation_y(0.7),
            Vec3::splat(2.0),
        );
        let local: Pose = (
            Vec3::new(0.5, -1.0, 3.0),
            Quat::from_rotation_x(0.2),
            Vec3::new(1.0, 2.0, 3.0),
        );
        let world = og_core::math::world_pose(
            1u8,
            local,
            |k| (k == 1).then_some(0u8),
            |k| (k == 0).then_some(parent),
        );
        let back = to_local(Some(parent), world);
        assert!(back.0.abs_diff_eq(local.0, 1e-5), "{} {}", back.0, local.0);
        assert!(back.1.abs_diff_eq(local.1, 1e-5));
        assert_eq!(back.2, world.2);
        assert_eq!(to_local(None, world), world);
    }

    #[test]
    fn an_orthographic_view_casts_parallel_rays() {
        let half_h = 5.0;
        let proj = glam::camera::rh::proj::directx::orthographic(
            -half_h * 2.0,
            half_h * 2.0,
            -half_h,
            half_h,
            0.1,
            100.0,
        );
        let view =
            glam::camera::rh::view::look_at_mat4(Vec3::new(0.0, 0.0, 10.0), Vec3::ZERO, Vec3::Y);
        let v = View::new(
            proj * view,
            Vec2::new(1000.0, 500.0),
            Vec3::new(0.0, 0.0, 10.0),
            Vec3::NEG_Z,
            true,
        );
        let (_, a) = v.ray(Vec2::new(10.0, 10.0));
        let (_, b) = v.ray(Vec2::new(900.0, 400.0));
        assert!(a.abs_diff_eq(b, 1e-5) && a.abs_diff_eq(Vec3::NEG_Z, 1e-5));
        // 500 px is 10 units: a handle is 1.92 units long everywhere.
        let g = Gizmo::new(
            &v,
            Mode::Translate,
            Space::World,
            Vec3::new(3.0, 1.0, 0.0),
            Quat::IDENTITY,
        );
        assert!((g.length - PIXELS / 50.0).abs() < 1e-3, "{}", g.length);
        let grab = at(&v, g.grab_point(&v, Handle::Axis(1)));
        let d = Drag::begin(
            &v,
            g,
            Handle::Axis(1),
            grab,
            (g.origin, Quat::IDENTITY, Vec3::ONE),
        )
        .unwrap();
        let (p, _, _) = d.pose(&v, grab + Vec2::new(0.0, -100.0), false);
        assert!(p.abs_diff_eq(Vec3::new(3.0, 3.0, 0.0), 1e-3), "{p}");
    }
}
