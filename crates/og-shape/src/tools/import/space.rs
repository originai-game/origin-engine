//! Affine transforms between a file's space and the kernel's: similarity, inverses, placement, rest poses and the axis turn.

use super::*;

/// An affine map in kernel space: `p ↦ x·p.x + y·p.y + z·p.z + t`.
/// The zero value maps everything to the origin; start from
/// [`Affine::IDENTITY`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Affine {
    /// Image of local +X.
    pub x: DVec3,
    /// Image of local +Y.
    pub y: DVec3,
    /// Image of local +Z.
    pub z: DVec3,
    /// Image of the local origin.
    pub t: DVec3,
}

impl Affine {
    /// The map that changes nothing.
    pub const IDENTITY: Affine = Affine {
        x: DVec3::X,
        y: DVec3::Y,
        z: DVec3::Z,
        t: DVec3::ZERO,
    };

    /// The linear part applied to a direction.
    pub fn rotate(&self, v: DVec3) -> DVec3 {
        self.x * v.x + self.y * v.y + self.z * v.z
    }

    /// The map applied to a point.
    pub fn apply(&self, p: DVec3) -> DVec3 {
        self.rotate(p) + self.t
    }

    /// `self ∘ other`: `other` first.
    pub fn then(&self, other: &Affine) -> Affine {
        Affine {
            x: self.rotate(other.x),
            y: self.rotate(other.y),
            z: self.rotate(other.z),
            t: self.apply(other.t),
        }
    }

    /// The determinant of the linear part: negative for a mirror.
    pub fn determinant(&self) -> f64 {
        self.x.dot(self.y.cross(self.z))
    }
}

/// `a` as a uniform scale times a rigid motion, `(scale, rigid, uneven)`,
/// if it is one to [`UNEVEN`]: `scale` is the mean axis length, `rigid`
/// the axes made exactly orthonormal (Gram–Schmidt, x first), `uneven`
/// how far `a` was from that.
pub(crate) fn similarity(a: &Affine) -> Option<(f64, Affine, f64)> {
    let l = [a.x.length(), a.y.length(), a.z.length()];
    let s = (l[0] + l[1] + l[2]) / 3.0;
    if !(s > 0.0 && s.is_finite()) || a.determinant() <= 0.0 {
        return None;
    }
    let (x, y, z) = (a.x / l[0], a.y / l[1], a.z / l[2]);
    let uneven = [
        (l[0] - s).abs() / s,
        (l[1] - s).abs() / s,
        (l[2] - s).abs() / s,
        x.dot(y).abs(),
        y.dot(z).abs(),
        x.dot(z).abs(),
    ]
    .into_iter()
    .fold(0.0, f64::max);
    if uneven > UNEVEN {
        return None;
    }
    let y = (y - x * x.dot(y)).normalize();
    let z = x.cross(y);
    Some((s, Affine { x, y, z, t: a.t }, uneven))
}

/// The inverse of an affine map, if it is not singular.
pub(crate) fn affine_inverse(a: &Affine) -> Option<Affine> {
    let det = a.determinant();
    if det == 0.0 || !det.is_finite() {
        return None;
    }
    // Rows of the inverse: the cross products of the columns over det.
    let (r0, r1, r2) = (
        a.y.cross(a.z) / det,
        a.z.cross(a.x) / det,
        a.x.cross(a.y) / det,
    );
    let l = Affine {
        x: DVec3::new(r0.x, r1.x, r2.x),
        y: DVec3::new(r0.y, r1.y, r2.y),
        z: DVec3::new(r0.z, r1.z, r2.z),
        t: DVec3::ZERO,
    };
    Some(Affine {
        t: -l.rotate(a.t),
        ..l
    })
}

/// Each vertex placed by the weighted blend of its bones' bind matrices
/// (`bind(joint)`, kernel space), and the largest distance between one of
/// those bones' placement and the blend. A weight naming no bone, a
/// mirroring bind matrix, or a vertex with no weight is an error.
pub(crate) fn place(
    positions: &[DVec3],
    raw: &[Vec<(u32, f64)>],
    bind: &dyn Fn(u32) -> Option<Affine>,
) -> Result<(Vec<DVec3>, f64), String> {
    let mut out = Vec::with_capacity(positions.len());
    let mut error: f64 = 0.0;
    for (v, (p, pairs)) in positions.iter().zip(raw).enumerate() {
        let mut each = Vec::with_capacity(4);
        let mut total = 0.0;
        for &(j, w) in pairs {
            if w > 0.0 {
                let m = bind(j)
                    .ok_or_else(|| format!("vertex {v} names joint {j}, which the skin lacks"))?;
                if m.determinant() <= 0.0 {
                    return Err(format!(
                        "joint {j}'s bind pose mirrors or flattens the mesh"
                    ));
                }
                each.push((m.apply(*p), w));
                total += w;
            }
        }
        if total <= 0.0 {
            return Err(format!("vertex {v} has no bone weight"));
        }
        let q = each
            .iter()
            .fold(DVec3::ZERO, |acc, (x, w)| acc + *x * (*w / total));
        for (x, _) in &each {
            error = error.max((*x - q).length());
        }
        out.push(q);
    }
    Ok((out, error))
}

/// The inverse of a rigid motion: its rotation transposed.
pub(crate) fn rigid_inverse(a: &Affine) -> Affine {
    let r = Affine {
        x: DVec3::new(a.x.x, a.y.x, a.z.x),
        y: DVec3::new(a.x.y, a.y.y, a.z.y),
        z: DVec3::new(a.x.z, a.y.z, a.z.z),
        t: DVec3::ZERO,
    };
    Affine {
        t: -r.rotate(a.t),
        ..r
    }
}

/// A rigid motion as a frame, and as its inverse matrix (column-major).
pub(crate) fn rest_and_inverse(r: &Affine) -> (Frame, [f64; 16]) {
    let i = rigid_inverse(r);
    (
        Frame {
            origin: r.t,
            x: r.x,
            y: r.y,
            z: r.z,
        },
        [
            i.x.x, i.x.y, i.x.z, 0.0, i.y.x, i.y.y, i.y.z, 0.0, i.z.x, i.z.y, i.z.z, 0.0, i.t.x,
            i.t.y, i.t.z, 1.0,
        ],
    )
}

/// A part turned by an axis turn of kernel space: an unskinned part's
/// transform is turned after; a skinned part's mesh and bones are turned
/// (`apply_transform` and `Mesh::placed` take it as it is), and its
/// transform conjugated, so either way it lands turned.
pub(crate) fn turn_part(p: &mut ImportedPart, c: &AxisTurn) {
    if p.solid.skin.is_some() {
        p.solid = turned(&p.solid, c);
        let back = c.inverse().after(&Affine::IDENTITY);
        p.transform = c.after(&p.transform.then(&back));
    } else {
        p.transform = c.after(&p.transform);
    }
}

/// glTF space (metres, Y up) to kernel space (Z up), unit-free:
/// `(x, y, z) ↦ (x, −z, y)`.
pub(crate) fn from_y_up(v: DVec3) -> DVec3 {
    DVec3::new(v.x, -v.z, v.y)
}

/// A glTF-space affine map, conjugated into kernel space with the unit
/// change applied to its translation.
pub(crate) fn conjugate(m: &Affine, unit: f64) -> Affine {
    // P(x, y, z) = (x, −z, y); P⁻¹ e_x = e_x, P⁻¹ e_y = −e_z, P⁻¹ e_z = e_y.
    Affine {
        x: from_y_up(m.x),
        y: -from_y_up(m.z),
        z: from_y_up(m.y),
        t: from_y_up(m.t) * unit,
    }
}
