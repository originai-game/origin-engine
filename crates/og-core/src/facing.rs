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

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::{FRAC_PI_2, PI};

    fn near(a: Vec3, b: Vec3) -> bool {
        (a - b).length() < 1e-5
    }

    #[test]
    fn right_is_forward_cross_up_in_every_row() {
        assert_eq!(FORWARD.cross(UP), RIGHT);
        assert_eq!(CAMERA_FORWARD.cross(UP), CAMERA_RIGHT);
        // On the 2D plane the up of the table is the spin axis.
        assert_eq!(FORWARD_2D.cross(SPIN_2D), Vec3::NEG_Y);
    }

    #[test]
    fn a_heading_turns_from_z_towards_x() {
        assert_eq!(heading(Quat::IDENTITY), 0.0);
        assert!(near(forward(from_heading(FRAC_PI_2)), Vec3::X));
        assert!(near(forward(from_heading(PI)), Vec3::NEG_Z));
        assert!((heading(from_heading(1.2)) - 1.2).abs() < 1e-6);
        assert!((heading_of(1.0, 0.0) - FRAC_PI_2).abs() < 1e-6);
        // An object turned to its right faces -X.
        assert!(near(
            forward(from_heading(-FRAC_PI_2)),
            right(Quat::IDENTITY)
        ));
    }

    #[test]
    fn a_2d_heading_turns_from_x_towards_y() {
        assert!(near(forward_2d(from_heading_2d(FRAC_PI_2)), Vec3::Y));
        assert!((heading_2d(Vec2::new(0.0, 1.0)) - FRAC_PI_2).abs() < 1e-6);
        // Facing a target turns +X towards it and stays in the plane.
        let (at, target) = (Vec2::new(2.0, 1.0), Vec2::new(-1.0, 5.0));
        let q = look_2d(at, target);
        assert!(near(forward_2d(q), (target - at).normalize().extend(0.0)));
        assert!(near(q * SPIN_2D, SPIN_2D));
        assert_eq!(look_2d(at, at), Quat::IDENTITY);
    }

    #[test]
    fn a_camera_looks_down_negative_z_and_an_object_faces_positive_z() {
        let (eye, target) = (Vec3::new(1.0, 2.0, 3.0), Vec3::new(-4.0, 0.5, 9.0));
        let d = (target - eye).normalize();
        let cam = camera_look(eye, target, Vec3::ZERO);
        assert!(near(camera_forward(cam), d));
        assert!(camera_right(cam).y.abs() < 1e-6, "the horizon is level");
        assert!(up(cam).y > 0.0);
        let obj = look(eye, target, Vec3::ZERO);
        assert!(near(forward(obj), d));
        assert!(up(obj).y > 0.0);
        // The two share a right-hand side: the object's is where the
        // camera's would be if it looked the same way.
        assert!(near(right(obj), camera_right(cam)));
        assert_eq!(camera_look(eye, eye, Vec3::ZERO), Quat::IDENTITY);
    }

    #[test]
    fn looking_straight_down_keeps_x_on_the_right() {
        let cam = camera_look(Vec3::Y, Vec3::ZERO, Vec3::ZERO);
        assert!(near(camera_forward(cam), Vec3::NEG_Y));
        assert!(near(camera_right(cam), Vec3::X));
    }

    #[test]
    fn forward_input_goes_into_the_screen_from_any_side() {
        let w = Vec2::new(0.0, 1.0);
        let d = Vec2::new(1.0, 0.0);
        // The default camera looks down -Z with +X on its right.
        assert!(near(move_relative(w, Quat::IDENTITY), Vec3::NEG_Z));
        assert!(near(move_relative(d, Quat::IDENTITY), Vec3::X));
        // A third-person camera behind and above a walker at the origin.
        let behind = camera_look(Vec3::new(0.0, 4.0, -8.0), Vec3::ZERO, Vec3::ZERO);
        assert!(near(move_relative(w, behind), Vec3::Z));
        assert!(near(move_relative(d, behind), Vec3::NEG_X));
        // Orbited half a turn, W still walks away from the camera.
        let front = camera_look(Vec3::new(0.0, 4.0, 8.0), Vec3::ZERO, Vec3::ZERO);
        assert!(near(move_relative(w, front), Vec3::NEG_Z));
        assert!(near(move_relative(d, front), Vec3::X));
        // Top down: the top of the screen is forward.
        let above = camera_look(Vec3::new(0.0, 10.0, 0.0), Vec3::ZERO, Vec3::ZERO);
        assert!(near(move_relative(w, above), Vec3::NEG_Z));
        // The length of a half-tilted stick is kept.
        let half = move_relative(Vec2::new(0.3, 0.4), behind);
        assert!((half.length() - 0.5).abs() < 1e-6);
        assert_eq!(half.y, 0.0);
    }

    #[test]
    fn a_camera_on_its_side_still_moves_along_its_view() {
        // Rolled a quarter turn about its view axis: its right is vertical.
        let rolled = Quat::from_rotation_z(FRAC_PI_2);
        assert!(near(camera_right(rolled), Vec3::Y));
        assert!(near(
            move_relative(Vec2::new(0.0, 1.0), rolled),
            Vec3::NEG_Z
        ));
        assert!(near(move_relative(Vec2::new(1.0, 0.0), rolled), Vec3::X));
    }
}
