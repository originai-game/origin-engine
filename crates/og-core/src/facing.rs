//! Which way things face: the engine's one statement of its axes.
//!
//! The world is right-handed with +Y up. What faces which way:
//!
//! | what | forward | right | up |
//! |---|---|---|---|
//! | an object (a character, a vehicle, an agent, a sound source) | local +Z | local -X | local +Y |
//! | a camera or a light | local -Z | local +X | local +Y |
//! | anything on the 2D plane (x right, y up) | local +X | local -Y | spins about +Z |
//!
//! In every row `right = forward × up`. An object faces +Z as a glTF model
//! and three.js `Object3D.lookAt` do; a camera or a light looks down -Z as
//! three.js cameras and lights do. A heading is the angle about +Y from +Z
//! towards +X in 3D, and about +Z from +X towards +Y in 2D.
//!
//! Every system takes its axes from here instead of restating them. The
//! trigonometry is og-core's deterministic [`math`](crate::math), so a
//! heading is the same bits on every host.

use glam::{Mat3, Quat, Vec2, Vec3};

use crate::math::{atan2, sin_cos};

/// The world's up.
pub const UP: Vec3 = Vec3::Y;
/// An object's forward in its own space.
pub const FORWARD: Vec3 = Vec3::Z;
/// An object's right in its own space.
pub const RIGHT: Vec3 = Vec3::NEG_X;
/// A camera's or a light's forward in its own space.
pub const CAMERA_FORWARD: Vec3 = Vec3::NEG_Z;
/// A camera's or a light's right in its own space.
pub const CAMERA_RIGHT: Vec3 = Vec3::X;
/// Forward on the 2D plane, in an entity's own space.
pub const FORWARD_2D: Vec3 = Vec3::X;
/// The axis a 2D entity spins about.
pub const SPIN_2D: Vec3 = Vec3::Z;

/// The way an object with rotation `q` faces.
pub fn forward(q: Quat) -> Vec3 {
    q * FORWARD
}

/// An object's right-hand side.
pub fn right(q: Quat) -> Vec3 {
    q * RIGHT
}

/// The up of anything with rotation `q`.
pub fn up(q: Quat) -> Vec3 {
    q * UP
}

/// The way a camera or a light with rotation `q` looks.
pub fn camera_forward(q: Quat) -> Vec3 {
    q * CAMERA_FORWARD
}

/// A camera's right: the right of the screen.
pub fn camera_right(q: Quat) -> Vec3 {
    q * CAMERA_RIGHT
}

/// The way a 2D entity with rotation `q` faces.
pub fn forward_2d(q: Quat) -> Vec3 {
    q * FORWARD_2D
}

/// The heading of the horizontal direction (`x`, `z`): 0 along +Z, π/2
/// along +X.
pub fn heading_of(x: f32, z: f32) -> f32 {
    atan2(x, z)
}

/// The heading an object with rotation `q` faces.
pub fn heading(q: Quat) -> f32 {
    let f = forward(q);
    heading_of(f.x, f.z)
}

/// The rotation of an upright object facing heading `h`.
pub fn from_heading(h: f32) -> Quat {
    let (s, c) = sin_cos(h * 0.5);
    Quat::from_xyzw(0.0, s, 0.0, c)
}

/// The 2D heading of the direction `d`: 0 along +X, π/2 along +Y.
pub fn heading_2d(d: Vec2) -> f32 {
    atan2(d.y, d.x)
}

/// The rotation of a 2D entity facing heading `h`.
pub fn from_heading_2d(h: f32) -> Quat {
    let (s, c) = sin_cos(h * 0.5);
    Quat::from_xyzw(0.0, 0.0, s, c)
}

/// The axes of a camera looking along the unit `forward` with `up` above
/// it: its right, its up and its back, as columns. Looking straight along
/// `up`, the right is +X.
pub fn camera_axes(forward: Vec3, up: Vec3) -> Mat3 {
    let z = -forward;
    let x = up.cross(z).normalize_or(Vec3::X);
    let y = z.cross(x);
    Mat3::from_cols(x, y, z)
}

/// The axes of an object facing along `forward` with `up` roughly up: its
/// columns are the object's local +X (its left), +Y and +Z (`forward`).
pub fn axes(forward: Vec3, up: Vec3) -> Mat3 {
    camera_axes(-forward, up)
}

/// The rotation of a camera or a light at `eye` looking at `target`: its
/// -Z towards the target. A zero `up` is +Y; the identity when the two
/// points are the same.
pub fn camera_look(eye: Vec3, target: Vec3, up: Vec3) -> Quat {
    let up = if up.length_squared() == 0.0 { UP } else { up };
    let forward = (target - eye).normalize_or_zero();
    if forward == Vec3::ZERO {
        return Quat::IDENTITY;
    }
    Quat::from_mat3(&camera_axes(forward, up))
}

/// The rotation of an object at `position` facing `target`: its +Z towards
/// the target. A zero `up` is +Y; the identity when the two points are the
/// same.
pub fn look(position: Vec3, target: Vec3, up: Vec3) -> Quat {
    camera_look(target, position, up)
}

/// The rotation of a 2D entity at `position` facing `target`: about +Z,
/// its +X towards the target. The identity when the two points are the
/// same.
pub fn look_2d(position: Vec2, target: Vec2) -> Quat {
    from_heading_2d(heading_2d(target - position))
}

/// A two-axis input (`x` right, `y` forward) as a horizontal world vector
/// seen from a camera with rotation `camera`: `y` goes into the screen and
/// `x` to its right, whatever the camera's pitch, and the length is kept.
/// A camera looking straight down still has a right, so the top of the
/// screen is forward; one rolled onto its side falls back to its own
/// forward.
pub fn move_relative(input: Vec2, camera: Quat) -> Vec3 {
    let level = |v: Vec3| Vec3::new(v.x, 0.0, v.z).normalize_or_zero();
    let mut right = level(camera_right(camera));
    let forward = if right == Vec3::ZERO {
        let forward = level(camera_forward(camera));
        right = forward.cross(UP);
        forward
    } else {
        UP.cross(right)
    };
    right * input.x + forward * input.y
}
