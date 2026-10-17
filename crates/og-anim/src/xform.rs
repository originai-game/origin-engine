//! Translation, rotation, scale: the pose's unit, and the few quaternion
//! operations animation needs. Only `+ - * /` and `sqrt` (glam is built with
//! `scalar-math`), so every result is bit-identical on every host.

use og_core::glam::{Mat4, Quat, Vec3};

/// A local or model-space transform: scale, then rotate, then translate
/// (glTF's TRS). Composition assumes uniform scale along a chain, as the
/// pose's model-space positions do; skinning matrices use full [`Mat4`]s.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Trs {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Trs {
    fn default() -> Self {
        Trs::IDENTITY
    }
}

impl Trs {
    pub const IDENTITY: Trs = Trs {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    pub fn new(translation: Vec3, rotation: Quat, scale: Vec3) -> Trs {
        Trs {
            translation,
            rotation,
            scale,
        }
    }

    /// `self` applied after `child` (`self` is the parent).
    pub fn mul(&self, child: &Trs) -> Trs {
        Trs {
            translation: self.translation + self.rotation * (self.scale * child.translation),
            rotation: unit(self.rotation * child.rotation),
            scale: self.scale * child.scale,
        }
    }

    /// The inverse (exact for uniform scale).
    pub fn inverse(&self) -> Trs {
        let r = self.rotation.conjugate();
        let s = Vec3::new(
            recip(self.scale.x),
            recip(self.scale.y),
            recip(self.scale.z),
        );
        Trs {
            translation: s * (r * -self.translation),
            rotation: r,
            scale: s,
        }
    }

    pub fn transform_point(&self, p: Vec3) -> Vec3 {
        self.translation + self.rotation * (self.scale * p)
    }

    /// The column-major matrix (glam's `from_scale_rotation_translation`,
    /// which is plain arithmetic).
    pub fn to_mat4(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    /// Component-wise interpolation: translation and scale linearly,
    /// rotation by normalised lerp on the shorter arc.
    pub fn lerp(&self, other: &Trs, t: f32) -> Trs {
        Trs {
            translation: self.translation + (other.translation - self.translation) * t,
            rotation: nlerp(self.rotation, other.rotation, t),
            scale: self.scale + (other.scale - self.scale) * t,
        }
    }
}

fn recip(x: f32) -> f32 {
    if x == 0.0 {
        0.0
    } else {
        1.0 / x
    }
}

/// `q` normalised; a zero or non-finite quaternion is the identity.
pub fn unit(q: Quat) -> Quat {
    let len2 = q.length_squared();
    if len2 > 1e-24 && len2.is_finite() {
        q * (1.0 / len2.sqrt())
    } else {
        Quat::IDENTITY
    }
}

/// Normalised lerp on the shorter arc.
pub fn nlerp(a: Quat, b: Quat, t: f32) -> Quat {
    let b = if a.dot(b) < 0.0 { -b } else { b };
    unit(a * (1.0 - t) + b * t)
}

/// The shortest rotation taking direction `from` to direction `to` (neither
/// need be normalised; a zero vector gives the identity).
pub fn rotation_arc(from: Vec3, to: Vec3) -> Quat {
    let (lf, lt) = (from.length(), to.length());
    if lf < 1e-12 || lt < 1e-12 {
        return Quat::IDENTITY;
    }
    let (a, b) = (from / lf, to / lt);
    let d = a.dot(b);
    if d < -0.999_999 {
        // Opposite: half a turn about any axis perpendicular to `a`.
        let axis = if a.x.abs() < 0.9 {
            Vec3::X.cross(a)
        } else {
            Vec3::Y.cross(a)
        };
        let axis = axis / axis.length();
        return Quat::from_xyzw(axis.x, axis.y, axis.z, 0.0);
    }
    let c = a.cross(b);
    unit(Quat::from_xyzw(c.x, c.y, c.z, 1.0 + d))
}

/// The twist of `q` about +Y (swing-twist decomposition): the yaw part, as a
/// quaternion, without trigonometry.
pub fn twist_y(q: Quat) -> Quat {
    unit(Quat::from_xyzw(0.0, q.y, 0.0, q.w))
}
