//! An FBX model's transform: Euler orders, pivots, pre- and post-rotation, geometric offsets and the file's axes.

use super::*;

/// A turn about one axis, by `degrees`.
pub(crate) fn turn(axis: usize, degrees: f64) -> Affine {
    let (s, c) = double::sin_cos(double::to_radians(degrees));
    let mut m = Affine::IDENTITY;
    match axis {
        0 => {
            m.y = DVec3::new(0.0, c, s);
            m.z = DVec3::new(0.0, -s, c);
        }
        1 => {
            m.x = DVec3::new(c, 0.0, -s);
            m.z = DVec3::new(s, 0.0, c);
        }
        _ => {
            m.x = DVec3::new(c, s, 0.0);
            m.y = DVec3::new(-s, c, 0.0);
        }
    }
    m
}

/// Euler angles (degrees) applied in `order` (FBX's `RotationOrder`: 0
/// XYZ, 1 XZY, 2 YZX, 3 YXZ, 4 ZXY, 5 ZYX; the first letter turns
/// first).
pub(crate) fn euler(r: DVec3, order: i64) -> Affine {
    let seq: [usize; 3] = match order {
        1 => [0, 2, 1],
        2 => [1, 2, 0],
        3 => [1, 0, 2],
        4 => [2, 0, 1],
        5 => [2, 1, 0],
        _ => [0, 1, 2],
    };
    let a = [r.x, r.y, r.z];
    let mut m = Affine::IDENTITY;
    for k in seq {
        m = turn(k, a[k]).then(&m);
    }
    m
}

pub(crate) fn translate(t: DVec3) -> Affine {
    Affine {
        t,
        ..Affine::IDENTITY
    }
}

pub(crate) fn scale(s: DVec3) -> Affine {
    Affine {
        x: DVec3::X * s.x,
        y: DVec3::Y * s.y,
        z: DVec3::Z * s.z,
        t: DVec3::ZERO,
    }
}

pub(crate) fn transpose(m: &Affine) -> Affine {
    Affine {
        x: DVec3::new(m.x.x, m.y.x, m.z.x),
        y: DVec3::new(m.x.y, m.y.y, m.z.y),
        z: DVec3::new(m.x.z, m.y.z, m.z.z),
        t: DVec3::ZERO,
    }
}

/// FBX's local transform of a model:
/// `T · Roff · Rp · Rpre · R · Rpost⁻¹ · Rp⁻¹ · Soff · Sp · S · Sp⁻¹`.
pub(crate) fn local(m: &Node) -> Affine {
    local_with(
        m,
        m.p70_vec("Lcl Translation", DVec3::ZERO),
        m.p70_vec("Lcl Rotation", DVec3::ZERO),
        m.p70_vec("Lcl Scaling", DVec3::ONE),
    )
}

/// [`local`] with the model's translation, rotation (Euler degrees, in
/// its own order) and scaling given: what an animation curve changes.
pub(crate) fn local_with(m: &Node, lcl_t: DVec3, lcl_r: DVec3, lcl_s: DVec3) -> Affine {
    local_turned(m, lcl_t, &euler(lcl_r, rotation_order(m)), lcl_s)
}

/// A model's `RotationOrder`.
pub(crate) fn rotation_order(m: &Node) -> i64 {
    m.p70("RotationOrder")
        .and_then(|p| p.first())
        .and_then(Prop::int)
        .unwrap_or(0)
}

/// [`local_with`], the rotation given as a matrix (what layers composed
/// leave).
pub(crate) fn local_turned(m: &Node, lcl_t: DVec3, lcl_r: &Affine, lcl_s: DVec3) -> Affine {
    let v = |k: &str| m.p70_vec(k, DVec3::ZERO);
    let (rp, sp) = (v("RotationPivot"), v("ScalingPivot"));
    let chain = [
        translate(lcl_t),
        translate(v("RotationOffset")),
        translate(rp),
        euler(v("PreRotation"), 0),
        *lcl_r,
        transpose(&euler(v("PostRotation"), 0)),
        translate(-rp),
        translate(v("ScalingOffset")),
        translate(sp),
        scale(lcl_s),
        translate(-sp),
    ];
    chain
        .iter()
        .rev()
        .fold(Affine::IDENTITY, |acc, step| step.then(&acc))
}

/// The geometric transform: moves the mesh, not the model's children.
pub(crate) fn geometric(m: &Node) -> Affine {
    let t = translate(m.p70_vec("GeometricTranslation", DVec3::ZERO));
    let r = euler(m.p70_vec("GeometricRotation", DVec3::ZERO), 0);
    let s = scale(m.p70_vec("GeometricScaling", DVec3::ONE));
    t.then(&r).then(&s)
}

/// The file's axes as a signed permutation onto glTF's (+X right, +Y up,
/// +Z front), from `GlobalSettings`.
pub(crate) fn axes(root: &Node) -> [(usize, f64); 3] {
    let g = root.child("GlobalSettings");
    let get = |k: &str, d: i64| {
        g.and_then(|g| g.p70(k))
            .and_then(|p| p.first())
            .and_then(Prop::int)
            .unwrap_or(d)
    };
    let sign = |v: i64| if v < 0 { -1.0 } else { 1.0 };
    let (c, u, f) = (get("CoordAxis", 0), get("UpAxis", 1), get("FrontAxis", 2));
    let ok = [c, u, f].iter().all(|a| (0..3).contains(a)) && c != u && u != f && c != f;
    if !ok {
        return [(0, 1.0), (1, 1.0), (2, 1.0)];
    }
    [
        (c as usize, sign(get("CoordAxisSign", 1))),
        (u as usize, sign(get("UpAxisSign", 1))),
        (f as usize, sign(get("FrontAxisSign", 1))),
    ]
}

/// A rotation matrix taken `w` of its way from no turn (`w` in 0..=1):
/// the same axis, the angle scaled.
pub(crate) fn part_turn(r: &Affine, w: f64) -> Affine {
    if w >= 1.0 {
        return *r;
    }
    let trace = r.x.x + r.y.y + r.z.z;
    let angle = double::acos(((trace - 1.0) * 0.5).clamp(-1.0, 1.0));
    // The axis: the matrix's skew part, or by its diagonal at a half turn.
    let mut axis = DVec3::new(r.y.z - r.z.y, r.z.x - r.x.z, r.x.y - r.y.x);
    if axis.length() < 1e-9 {
        if angle < 1.0 {
            return Affine::IDENTITY;
        }
        axis = DVec3::new(
            ((r.x.x + 1.0) * 0.5).max(0.0).sqrt(),
            ((r.y.y + 1.0) * 0.5).max(0.0).sqrt(),
            ((r.z.z + 1.0) * 0.5).max(0.0).sqrt(),
        );
        if r.x.y + r.y.x < 0.0 {
            axis.y = -axis.y;
        }
        if r.x.z + r.z.x < 0.0 {
            axis.z = -axis.z;
        }
    }
    let Some(n) = axis.try_normalize() else {
        return Affine::IDENTITY;
    };
    // Rodrigues: each basis vector turned about the axis.
    let (s, c) = double::sin_cos(angle * w);
    let turn = |v: DVec3| v * c + n.cross(v) * s + n * (n.dot(v) * (1.0 - c));
    Affine {
        x: turn(DVec3::X),
        y: turn(DVec3::Y),
        z: turn(DVec3::Z),
        t: DVec3::ZERO,
    }
}
