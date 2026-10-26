//! The two numbers every target must agree on:
//!
//! - [`math_digest`]: FNV-1a over the bits of 1.4 million results of the
//!   deterministic `og_core::math` functions and of glam vector, quaternion
//!   and matrix arithmetic (the types whose lane order used to differ per
//!   target), on inputs drawn from a fixed seed;
//! - [`bouncing_boxes_hash`]: `World::world_hash` of the bouncing-boxes
//!   world (`crates/og-ecs/tests/support/bouncing_boxes.rs`, the same file the
//!   og-ecs test runs), seed 42, after 3600 ticks;
//! - [`physics3d_hash`]: the world hashes of og-physics' beyond-rigid-bodies
//!   scenario (`crates/og-physics/tests/support/beyond.rs`: joints,
//!   controllers, cars, ropes, cloth with self-collision, water, fluids and
//!   a fluid group, ragdolls) in 3D after 600 ticks, and of a pen of 1,000
//!   of the pile's bodies (`tests/support/pile.rs`) after 300, enough bodies
//!   for the step to go to four lanes (og-physics's `pool_bodies`: 500 on
//!   four), so the host's lanes run steps it on rapier's parallel build;
//! - [`physics2d_hash`]: the same scenario in 2D after 600 ticks. A build
//!   with one physics profile (the `physics2d` or `physics3d` feature alone,
//!   as og-host-web's per-profile modules are built) has only its own;
//! - [`anim_hash`]: the world hash of og-anim's walk scenario
//!   (`crates/og-anim/tests/support/walk.rs`: graphs, players, root motion,
//!   blendshapes, IK) after 600 ticks, with every sample of the walk
//!   retargeted onto the tall Mixamo rig (`og_anim::retarget`).
//!
//! On wasm32 they are exported under the same names (no imports; `load.mjs`
//! instantiates the module under node).

use og_core::Rng;
use og_core::hash::StableHasher;
use og_core::math::{
    Mat4, Quat, Vec3, Vec3A, Vec4, atan, atan2, cos, exp, ln, pow, quat_from_axis_angle, sin,
    sin_cos,
};

#[allow(dead_code)] // `Stats` fields are read only by the og-ecs test
#[path = "../../../crates/og-ecs/tests/support/bouncing_boxes.rs"]
mod bouncing_boxes;

#[path = "../../../crates/og-physics/tests/support/mod.rs"]
mod physics_support;
use physics_support::beyond;
#[cfg(feature = "physics3d")]
use physics_support::pile;

#[allow(dead_code)]
#[path = "../../../crates/og-anim/tests/support/mod.rs"]
mod anim_support;

/// Rounds of [`math_digest`]; each hashes 28 results.
pub const ROUNDS: u32 = 50_000;

fn put(h: &mut StableHasher, values: &[f32]) {
    for v in values {
        h.write_u32(v.to_bits());
    }
}

/// A channel's frames as held ([`og_anim::Samples`]): a tag for the form,
/// then every number in it, so a clip that changed form hashes differently.
fn put_samples(h: &mut StableHasher, samples: &og_anim::Samples) {
    match samples {
        og_anim::Samples::F32(v) => {
            h.write_u32(0);
            put(h, v);
        }
        og_anim::Samples::Constant(v) => {
            h.write_u32(1);
            put(h, v);
        }
        og_anim::Samples::Quantized(q) => {
            h.write_u32(2);
            for f in &q.frames {
                h.write_u32(u32::from(*f));
            }
            for w in &q.words {
                h.write_u32(u32::from(*w));
            }
            put(h, &q.range);
        }
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

fn beyond_hash(profile: u32) -> u64 {
    let (mut w, p, cast) = beyond::build(7, profile);
    beyond::run(&mut w, &p, &cast, 0, 600, profile);
    w.world_hash()
}

/// See the crate documentation.
#[cfg(feature = "physics3d")]
pub fn physics3d_hash() -> u64 {
    let mut h = StableHasher::new();
    h.write_u64(beyond_hash(og_physics::PROFILE_3D));
    let (mut w, _p) = pile::pen(1000, 11);
    pile::run(&mut w, 300);
    h.write_u64(w.world_hash());
    h.finish()
}

/// See the crate documentation.
#[cfg(feature = "physics2d")]
pub fn physics2d_hash() -> u64 {
    beyond_hash(og_physics::PROFILE_2D)
}

/// See the crate documentation.
pub fn anim_hash() -> u64 {
    let mut h = StableHasher::new();
    let (mut w, _anim) = anim_support::walk::build(3);
    for _ in 0..600 {
        w.step().expect("the walk steps");
    }
    h.write_u64(w.world_hash());
    let source = anim_support::skeleton();
    let target = anim_support::tall();
    let walk = anim_support::clips()
        .into_iter()
        .map(|(_, c)| c)
        .find(|c| c.name == "walk")
        .expect("the walk clip");
    let out = og_anim::retarget(
        &source,
        &og_anim::HumanoidMap::detect(&source),
        &target,
        &og_anim::HumanoidMap::detect(&target),
        &walk,
        &og_anim::RetargetOptions::default(),
    )
    .expect("the walk retargets");
    for c in &out.clip.channels {
        h.write_u32(c.node);
        put_samples(&mut h, &c.samples);
    }
    h.finish()
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    #[unsafe(no_mangle)]
    pub extern "C" fn math_digest() -> u64 {
        super::math_digest()
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn bouncing_boxes_hash() -> u64 {
        super::bouncing_boxes_hash()
    }

    #[cfg(feature = "physics3d")]
    #[unsafe(no_mangle)]
    pub extern "C" fn physics3d_hash() -> u64 {
        super::physics3d_hash()
    }

    #[cfg(feature = "physics2d")]
    #[unsafe(no_mangle)]
    pub extern "C" fn physics2d_hash() -> u64 {
        super::physics2d_hash()
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn anim_hash() -> u64 {
        super::anim_hash()
    }
}
