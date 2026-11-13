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

mod drag;
mod rays;

pub use drag::*;
pub use rays::*;

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
    let v = crate::host::api::camera_view(camera, w, h);
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
mod tests;
