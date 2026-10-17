//! The two numbers every target must agree on:
//!
//! - [`math_digest`]: FNV-1a over the bits of 1.4 million results of the
//!   deterministic `og_core::math` functions and of glam vector, quaternion
//!   and matrix arithmetic (the types whose lane order used to differ per
//!   target), on inputs drawn from a fixed seed;
//! - [`bouncing_boxes_hash`]: `World::world_hash` of the bouncing-boxes
//!   world (`crates/og-ecs/tests/support/bouncing_boxes.rs`, the same file the
//!   og-ecs test runs), seed 42, after 3600 ticks.
//!
//! On wasm32 they are exported as `math_digest` and `bouncing_boxes_hash`
//! (no imports; `load.mjs` instantiates the module under node).

use og_core::hash::StableHasher;
use og_core::math::{
    atan, atan2, cos, exp, ln, pow, quat_from_axis_angle, sin, sin_cos, Mat4, Quat, Vec3, Vec3A,
    Vec4,
};
use og_core::Rng;

#[allow(dead_code)] // `Stats` fields are read only by the og-ecs test
#[path = "../../../crates/og-ecs/tests/support/bouncing_boxes.rs"]
mod bouncing_boxes;

/// Rounds of [`math_digest`]; each hashes 28 results.
pub const ROUNDS: u32 = 50_000;

fn put(h: &mut StableHasher, values: &[f32]) {
    for v in values {
        h.write_u32(v.to_bits());
    }
}

/// See the crate documentation.
pub fn math_digest() -> u64 {
    let mut rng = Rng::new(0x0d15_ea5e);
    let mut h = StableHasher::new();
    for i in 0..ROUNDS {
        // Arguments from tiny to huge: a uniform draw scaled by 2^k, k in [-20, 40).
        let scale = f32::from_bits(((127 + (i % 60) as i32 - 20) as u32) << 23);
        let x = rng.range_f32(-1.0, 1.0) * scale;
        let y = rng.range_f32(-1.0, 1.0) * scale;
        let small = rng.range_f32(-80.0, 80.0);
        let (s, c) = sin_cos(x);
        put(
            &mut h,
            &[
                sin(x),
                cos(x),
                s,
                c,
                atan(x),
                atan2(y, x),
                exp(small),
                ln(x.abs()),
                pow(x.abs(), small / 40.0),
            ],
        );

        let mut v4 = || {
            Vec4::new(
                rng.range_f32(-10.0, 10.0),
                rng.range_f32(-10.0, 10.0),
                rng.range_f32(-10.0, 10.0),
                rng.range_f32(-10.0, 10.0),
            )
        };
        let (a, b) = (v4(), v4());
        let axis = a.truncate().normalize();
        let p = quat_from_axis_angle(axis, b.x);
        let q = Quat::from_vec4(b).normalize();
        let pq = p * q;
        let m = Mat4::from_cols(
            a,
            b,
            Vec4::new(a.z, a.w, a.x, a.y),
            Vec4::new(b.w, b.z, b.y, b.x),
        );
        let rotated = q * a.truncate();
        let crossed = Vec3A::from(a.truncate()).cross(Vec3A::from(b.truncate()));
        put(
            &mut h,
            &[
                a.dot(b),
                a.length(),
                pq.x,
                pq.y,
                pq.z,
                pq.w,
                rotated.x,
                rotated.y,
                rotated.z,
                crossed.length(),
            ],
        );
        let mv = m * b;
        let mm = (m * Mat4::from_quat(q)).transform_point3(Vec3::new(b.x, b.y, b.z));
        let n = a.normalize();
        put(
            &mut h,
            &[mv.x, mv.y, mv.z, mv.w, mm.x, mm.y, mm.z, n.x, n.w],
        );
    }
    h.finish()
}

/// See the crate documentation.
pub fn bouncing_boxes_hash() -> u64 {
    let mut w = bouncing_boxes::build(42);
    bouncing_boxes::run(&mut w, 3600);
    w.world_hash()
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    #[no_mangle]
    pub extern "C" fn math_digest() -> u64 {
        super::math_digest()
    }

    #[no_mangle]
    pub extern "C" fn bouncing_boxes_hash() -> u64 {
        super::bouncing_boxes_hash()
    }
}
