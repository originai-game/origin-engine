//! Determinism of physics beyond rigid bodies, in the shape of the pile
//! test: the scenario of `support/beyond.rs` (a 3D world with a character,
//! joints, a car, a rope, a cloth and a self-colliding cloth, water and a
//! ragdoll and two 3D fluids of a group in a terrain bowl; a 2D one with a platformer, a car, a fluid and a group of two,
//! a water line and a 2D ragdoll), each run 1200 ticks twice to the same
//! world hash and snapshot, and restored at tick 600 into a fresh world
//! (another seed) to end in the same world as the uninterrupted run.
//!
//! Ways this could fail, each caught here: state of a feature kept outside
//! the og.physics state block or the components (ropes, cloth, fluid,
//! water, joints, wheels, ragdoll bodies, controller timers): the restored
//! run diverges; an iteration order that is not fixed or a platform-
//! dependent function: the two runs diverge (and the host and wasm32 hashes
//! of tools/wasm-determinism would); a feature that does nothing: the
//! checks on what moved fail.

mod support;

use og_core::glam::Vec3;
use og_physics::*;
use std::time::Instant;

use support::beyond::*;

fn check(profile: u32) -> String {
    let started = Instant::now();
    let second = std::thread::spawn(move || {
        let (mut b, pb, cb) = build(7, profile);
        run(&mut b, &pb, &cb, 0, TICKS, profile);
        b
    });
    let (mut a, pa, ca) = build(7, profile);
    let start = a.world_hash();
    run(&mut a, &pa, &ca, 0, HALF, profile);
    let half = a.snapshot();
    let half_hash = a.world_hash();
    run(&mut a, &pa, &ca, HALF, TICKS, profile);
    let b = second.join().unwrap();
    assert_eq!(
        a.world_hash(),
        b.world_hash(),
        "two {profile}D runs diverged"
    );
    assert_eq!(a.snapshot(), b.snapshot());

    // A fresh world (another seed), its assets (the ragdoll rig) registered
    // again, restored at tick 600, ends where the uninterrupted run did.
    let (mut c, pc, cc) = build(99, profile);
    c.restore(&half).unwrap();
    assert_eq!(c.world_hash(), half_hash);
    run(&mut c, &pc, &cc, HALF, TICKS, profile);
    assert_eq!(
        c.world_hash(),
        a.world_hash(),
        "the restored {profile}D run diverged"
    );
    // What physics keeps beyond the components ends the same too (it is
    // in the state block, so in the hash, but these read it back).
    for (i, (ea, ec)) in a.entities().into_iter().zip(c.entities()).enumerate() {
        assert_eq!(ea, ec, "entity {i}");
        assert_eq!(pa.rope_points(ea), pc.rope_points(ec), "rope {ea}");
        assert_eq!(pa.cloth_points(ea), pc.cloth_points(ec), "cloth {ea}");
        assert_eq!(pa.fluid_particles(ea), pc.fluid_particles(ec), "fluid {ea}");
        assert_eq!(pa.fluid_mix(ea), pc.fluid_mix(ec), "fluid mix {ea}");
        assert_eq!(
            pa.fluid3d_particles(ea),
            pc.fluid3d_particles(ec),
            "3D fluid {ea}"
        );
        assert_eq!(pa.fluid3d_mix(ea), pc.fluid3d_mix(ec), "3D fluid mix {ea}");
        assert_eq!(pa.water_heights(ea), pc.water_heights(ec), "water {ea}");
        assert_eq!(pa.water_line(ea), pc.water_line(ec), "water line {ea}");
        assert_eq!(pa.vehicle_wheels(ea), pc.vehicle_wheels(ec), "wheels {ea}");
        assert_eq!(pa.ragdoll_pose(ea), pc.ragdoll_pose(ec), "ragdoll {ea}");
    }
    assert_ne!(a.world_hash(), start);

    // It all moved.
    let walked = a.get::<Transform>(ca.walker).unwrap().position;
    assert!(walked.length() > 1.0, "the controller stayed at {walked}");
    let car = a.get::<Vehicle>(ca.car).unwrap();
    let ct = a.get::<Transform>(ca.car).unwrap();
    assert!(
        car.contacts > 0,
        "{car:?} {ct:?} up {}",
        ct.quaternion * Vec3::Y
    );
    if let Some(r) = ca.ragdoll {
        assert!(pa.ragdoll_pose(r).is_some());
    }
    if let Some((heavy, light)) = ca.fluids3d {
        let (h, l) = (
            pa.fluid3d_particles(heavy).unwrap(),
            pa.fluid3d_particles(light).unwrap(),
        );
        assert!(h.len() > 200, "the emitter added none ({})", h.len());
        assert!(l.len() < 200, "the sink took none ({})", l.len());
        let (_, mix) = pa.fluid3d_mix(heavy).unwrap();
        assert!(mix.iter().any(|m| m[1] > 0.01), "the fluids never mixed");
    }
    format!(
        "{{\"profile\":{profile},\"ticks\":{TICKS},\"hash_tick_0\":\"{start:016x}\",\"hash_tick_{HALF}\":\"{half_hash:016x}\",\"hash_tick_{TICKS}\":\"{:016x}\",\"hash_tick_{TICKS}_restored\":\"{:016x}\",\"snapshot_bytes\":{},\"seconds\":{:.3}}}",
        a.world_hash(),
        c.world_hash(),
        half.len(),
        started.elapsed().as_secs_f64()
    )
}

#[test]
fn physics_beyond_rigid_bodies_is_deterministic_and_restores_mid_run() {
    let three = check(PROFILE_3D);
    let two = check(PROFILE_2D);
    let report = format!("[{three},\n{two}]\n");
    print!("{report}");
    if let Some(path) = std::env::var_os("OG_PHYSICS_BEYOND_REPORT") {
        std::fs::write(path, report).unwrap();
    }
}
