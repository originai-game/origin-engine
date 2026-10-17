//! glam's vector, quaternion and matrix arithmetic is the same on every host.
//!
//! The workspace builds glam with `scalar-math`, so `Vec3A`, `Vec4`, `Quat`,
//! `Mat3A`, `Mat4` and `Affine3A` run one scalar implementation everywhere.
//! Each result below is compared bit for bit with an explicit reference in
//! plain `f32`, written out in a fixed operation order (IEEE 754 makes each
//! `+ - * / sqrt` exact-rounded, so the reference is the same on every
//! host). glam's SIMD backends (SSE2, NEON, wasm `simd128`) sum the lanes in
//! another order and fail this test; these are the types that diverged
//! across hosts before `scalar-math`.

use og_core::math::{Affine3A, Mat3, Mat3A, Mat4, Quat, Vec3, Vec3A, Vec4};
use og_core::Rng;

type V3 = [f32; 3];
type V4 = [f32; 4];
type M4 = [V4; 4]; // columns

fn dot3(a: V3, b: V3) -> f32 {
    ((a[0] * b[0]) + (a[1] * b[1])) + (a[2] * b[2])
}

fn dot4(a: V4, b: V4) -> f32 {
    (((a[0] * b[0]) + (a[1] * b[1])) + (a[2] * b[2])) + (a[3] * b[3])
}

fn cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - b[1] * a[2],
        a[2] * b[0] - b[2] * a[0],
        a[0] * b[1] - b[0] * a[1],
    ]
}

fn scale3(a: V3, s: f32) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn add3(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale4(a: V4, s: f32) -> V4 {
    [a[0] * s, a[1] * s, a[2] * s, a[3] * s]
}

fn add4(a: V4, b: V4) -> V4 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]]
}

fn normalize4(a: V4) -> V4 {
    scale4(a, 1.0 / dot4(a, a).sqrt())
}

fn quat_mul(a: V4, b: V4) -> V4 {
    let [x0, y0, z0, w0] = a;
    let [x1, y1, z1, w1] = b;
    [
        ((w0 * x1 + x0 * w1) + y0 * z1) - z0 * y1,
        ((w0 * y1 - x0 * z1) + y0 * w1) + z0 * x1,
        ((w0 * z1 + x0 * y1) - y0 * x1) + z0 * w1,
        ((w0 * w1 - x0 * x1) - y0 * y1) - z0 * z1,
    ]
}

/// `v·(w² - b·b) + b·(2 v·b) + (b × v)·2w`, with `b` the vector part.
fn quat_rotate(q: V4, v: V3) -> V3 {
    let b = [q[0], q[1], q[2]];
    let w = q[3];
    let t = scale3(v, w * w - dot3(b, b));
    let t = add3(t, scale3(b, dot3(v, b) * 2.0));
    add3(t, scale3(cross(b, v), w * 2.0))
}

fn mat4_mul_vec4(m: &M4, v: V4) -> V4 {
    let r = scale4(m[0], v[0]);
    let r = add4(r, scale4(m[1], v[1]));
    let r = add4(r, scale4(m[2], v[2]));
    add4(r, scale4(m[3], v[3]))
}

fn mat3_mul_vec3(m: &[V3; 3], v: V3) -> V3 {
    let r = scale3(m[0], v[0]);
    let r = add3(r, scale3(m[1], v[1]));
    add3(r, scale3(m[2], v[2]))
}

/// The rotation matrix of a unit quaternion, scaled per column, translated.
fn srt(s: V3, q: V4, t: V3) -> M4 {
    let [x, y, z, w] = q;
    let (x2, y2, z2) = (x + x, y + y, z + z);
    let (xx, xy, xz) = (x * x2, x * y2, x * z2);
    let (yy, yz, zz) = (y * y2, y * z2, z * z2);
    let (wx, wy, wz) = (w * x2, w * y2, w * z2);
    [
        scale4([1.0 - (yy + zz), xy + wz, xz - wy, 0.0], s[0]),
        scale4([xy - wz, 1.0 - (xx + zz), yz + wx, 0.0], s[1]),
        scale4([xz + wy, yz - wx, 1.0 - (xx + yy), 0.0], s[2]),
        [t[0], t[1], t[2], 1.0],
    ]
}

fn b3(v: V3) -> [u32; 3] {
    v.map(f32::to_bits)
}

fn b4(v: V4) -> [u32; 4] {
    v.map(f32::to_bits)
}

fn m4bits(m: &M4) -> [[u32; 4]; 4] {
    m.map(b4)
}

struct Inputs(Rng);

impl Inputs {
    fn f(&mut self) -> f32 {
        self.0.range_f32(-100.0, 100.0)
    }
    fn v3(&mut self) -> V3 {
        [self.f(), self.f(), self.f()]
    }
    fn v4(&mut self) -> V4 {
        [self.f(), self.f(), self.f(), self.f()]
    }
    fn unit_quat(&mut self) -> V4 {
        normalize4(self.v4())
    }
    fn m4(&mut self) -> M4 {
        [self.v4(), self.v4(), self.v4(), self.v4()]
    }
}

const SAMPLES: usize = 20_000;

#[test]
fn vec3a_and_vec4_sum_lanes_in_one_order() {
    let mut r = Inputs(Rng::new(0x5ca1a));
    for _ in 0..SAMPLES {
        let (a, b) = (r.v3(), r.v3());
        let (ga, gb) = (Vec3A::from_array(a), Vec3A::from_array(b));
        assert_eq!(
            ga.dot(gb).to_bits(),
            dot3(a, b).to_bits(),
            "Vec3A::dot {a:?} {b:?}"
        );
        assert_eq!(
            b3(ga.cross(gb).to_array()),
            b3(cross(a, b)),
            "Vec3A::cross {a:?} {b:?}"
        );
        assert_eq!(
            ga.length().to_bits(),
            dot3(a, a).sqrt().to_bits(),
            "Vec3A::length {a:?}"
        );

        let (c, d) = (r.v4(), r.v4());
        let (gc, gd) = (Vec4::from_array(c), Vec4::from_array(d));
        assert_eq!(
            gc.dot(gd).to_bits(),
            dot4(c, d).to_bits(),
            "Vec4::dot {c:?} {d:?}"
        );
        assert_eq!(
            gc.length().to_bits(),
            dot4(c, c).sqrt().to_bits(),
            "Vec4::length {c:?}"
        );
        assert_eq!(
            b4(gc.normalize().to_array()),
            b4(normalize4(c)),
            "Vec4::normalize {c:?}"
        );
    }
}

#[test]
fn quat_products_and_rotations_match_the_scalar_reference() {
    let mut r = Inputs(Rng::new(0x9a7));
    for _ in 0..SAMPLES {
        let (raw, p, q, v) = (r.v4(), r.unit_quat(), r.unit_quat(), r.v3());
        let normalized = Quat::from_array(raw).normalize();
        assert_eq!(
            b4(normalized.to_array()),
            b4(normalize4(raw)),
            "Quat::normalize {raw:?}"
        );
        let (gp, gq) = (Quat::from_array(p), Quat::from_array(q));
        assert_eq!(
            b4((gp * gq).to_array()),
            b4(quat_mul(p, q)),
            "Quat * Quat {p:?} {q:?}"
        );
        assert_eq!(
            b3((gp * Vec3::from_array(v)).to_array()),
            b3(quat_rotate(p, v)),
            "Quat * Vec3 {p:?} {v:?}"
        );
        assert_eq!(
            b3((gp * Vec3A::from_array(v)).to_array()),
            b3(quat_rotate(p, v)),
            "Quat * Vec3A {p:?} {v:?}"
        );
    }
}

#[test]
fn mat4_products_and_transforms_match_the_scalar_reference() {
    let mut r = Inputs(Rng::new(0x3a74));
    for _ in 0..SAMPLES / 4 {
        let (a, b, v) = (r.m4(), r.m4(), r.v4());
        let (ga, gb) = (Mat4::from_cols_array_2d(&a), Mat4::from_cols_array_2d(&b));
        assert_eq!(
            b4((ga * Vec4::from_array(v)).to_array()),
            b4(mat4_mul_vec4(&a, v)),
            "Mat4 * Vec4"
        );
        let product: M4 = [0, 1, 2, 3].map(|i| mat4_mul_vec4(&a, b[i]));
        assert_eq!(
            m4bits(&(ga * gb).to_cols_array_2d()),
            m4bits(&product),
            "Mat4 * Mat4"
        );

        let (s, q, t, p) = (r.v3(), r.unit_quat(), r.v3(), r.v3());
        let m = srt(s, q, t);
        let gm = Mat4::from_scale_rotation_translation(
            Vec3::from_array(s),
            Quat::from_array(q),
            Vec3::from_array(t),
        );
        assert_eq!(
            m4bits(&gm.to_cols_array_2d()),
            m4bits(&m),
            "Mat4::from_scale_rotation_translation"
        );
        let point = mat4_mul_vec4(&m, [p[0], p[1], p[2], 1.0]);
        assert_eq!(
            b3(gm.transform_point3(Vec3::from_array(p)).to_array()),
            b3([point[0], point[1], point[2]]),
            "Mat4::transform_point3"
        );
    }
}

#[test]
fn mat3a_and_affine3a_match_the_scalar_reference() {
    let mut r = Inputs(Rng::new(0xaff));
    for _ in 0..SAMPLES / 4 {
        let (cols, v, t) = ([r.v3(), r.v3(), r.v3()], r.v3(), r.v3());
        let m = Mat3A::from_cols_array_2d(&cols);
        assert_eq!(
            b3((m * Vec3A::from_array(v)).to_array()),
            b3(mat3_mul_vec3(&cols, v)),
            "Mat3A * Vec3A"
        );
        let affine =
            Affine3A::from_mat3_translation(Mat3::from_cols_array_2d(&cols), Vec3::from_array(t));
        assert_eq!(
            b3(affine.transform_point3(Vec3::from_array(v)).to_array()),
            b3(add3(mat3_mul_vec3(&cols, v), t)),
            "Affine3A::transform_point3"
        );
    }
}
