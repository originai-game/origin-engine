//! Development-only cross-host digests, using the engine's built modules.
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
//!   four), so the host's lanes run steps it on rapier's parallel build,
//!   and of 60 compound bodies of convex hulls (`pile::compounds`, built as
//!   destruction's loose islands are) after 300: a hull's and a compound's
//!   mass properties are diagonalised by glamx's eigensolver, which went
//!   through the host's libm until `third_party/glamx` and the two hosts
//!   drifted apart;
//! - [`physics2d_hash`]: the same scenario in 2D after 600 ticks;
//! - [`anim_hash`]: the world hash of og-anim's walk scenario
//!   (`crates/og-anim/tests/support/walk.rs`: graphs, players, root motion,
//!   blendshapes, IK) after 600 ticks, with every sample of the walk
//!   retargeted onto the tall Mixamo rig (`og_anim::retarget`).
//!
//! `dump_determinism` returns hex digests without losing u64 bits through JSON.
//! Published modules exclude these scenarios and are compared through snapshots.

use og_core::Rng;
use og_core::hash::StableHasher;
use og_core::math::{
    Mat4, Quat, Vec3, Vec3A, Vec4, atan, atan2, cos, exp, ln, pow, quat_from_axis_angle, sin,
    sin_cos,
};

#[allow(dead_code)] // `Stats` fields are read only by the og-ecs test
#[path = "../../../og-ecs/tests/support/bouncing_boxes.rs"]
mod bouncing_boxes;

#[path = "../../../og-physics/tests/support/mod.rs"]
mod physics_support;
use physics_support::beyond;
use physics_support::pile;

#[path = "../../../og-anim/tests/support/mod.rs"]
mod anim_support;

/// Rounds of [`math_digest`]; each hashes 28 results.
pub const ROUNDS: u32 = 50_000;

pub(crate) fn register(probe: &mut og_probe::Probe) {
    probe
        .register_dump(
            "dump_determinism",
            og_probe::CommandDesc {
                usage: "dump_determinism[:path]".into(),
                doc: "cross-host math, ECS, physics, animation and text-layout digests; development builds only".into(),
            },
            0,
            |_, _| {
                let before = og_lanes::calls_here();
                let mut digests = serde_json::json!({
                    "math": format!("{:016x}", math_digest()),
                    "boxes": format!("{:016x}", bouncing_boxes_hash()),
                    "anim": format!("{:016x}", anim_hash()),
                    "text": format!("{:016x}", text_layout_hash()),
                });
                digests["physics3d"] = format!("{:016x}", physics3d_hash()).into();
                digests["physics2d"] = format!("{:016x}", physics2d_hash()).into();
                Ok(serde_json::json!({ "digests": digests, "parallelCalls": og_lanes::calls_here() - before }))
            },
        )
        .expect("unique engine dump name");
}

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
pub fn physics3d_hash() -> u64 {
    let mut h = StableHasher::new();
    h.write_u64(beyond_hash(og_physics::PROFILE_3D));
    let (mut w, _p) = pile::pen(1000, 11);
    pile::run(&mut w, 300);
    h.write_u64(w.world_hash());
    let (mut w, _p) = pile::compounds(60, 13);
    pile::run(&mut w, 300);
    h.write_u64(w.world_hash());
    h.finish()
}

/// See the crate documentation.
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

/// Actual line breaks, shaped glyph positions and script fallback, with
/// the same OpenType bytes on both hosts: a paragraph that breaks (combining
/// marks, a tab, a no-break space, a soft hyphen, a zero-width space) and
/// one whose Arabic and Devanagari fall back to the game's fonts, each at
/// three widths, through the UI's Parley layout. No renderer or system
/// fonts.
pub fn text_layout_hash() -> u64 {
    use og_text::fonts::Collection;
    use og_text::layout::{Style, Text};
    use std::sync::Arc;
    let mut fonts = Collection::new();
    for bytes in [
        &include_bytes!("../../../../tests/fixtures/ui-text/assets/fonts/Inter-Display.ttf")[..],
        include_bytes!("../../../../tests/fixtures/ui-text/assets/fonts/NotoSansArabic-subset.ttf"),
        include_bytes!(
            "../../../../tests/fixtures/ui-text/assets/fonts/NotoSansDevanagari-subset.ttf"
        ),
    ] {
        fonts.add(Arc::new(bytes.to_vec())).expect("a fixture face");
    }
    let style = Style {
        family: "Inter, \"Noto Sans Arabic\", \"Noto Sans Devanagari\"",
        size: 16.0,
        ..Style::default()
    };
    let mut h = StableHasher::new();
    let mut layout = Text::new();
    for text in [
        "office cafe\u{301}\tA\u{a0}B a/b-c\u{200b}def ab\u{ad}cdef",
        "مرحبا 42 नमस्ते",
    ] {
        layout.shape(&mut fonts, text, &style);
        for width in [53.25f32, 91.75, 231.5] {
            layout.break_at(&mut fonts, text, &style, Some(width));
            layout.lines(|m, range| {
                h.write_u32(range.start as u32);
                h.write_u32(range.end as u32);
                for v in m {
                    h.write_u32(v.to_bits());
                }
            });
            layout.runs(&fonts, |run| {
                h.write_u32(run.face.0);
                for g in run.glyphs {
                    h.write_u32(g.id);
                    for v in [g.x, g.y, g.advance] {
                        h.write_u32(v.to_bits());
                    }
                }
            });
        }
    }
    h.finish()
}
