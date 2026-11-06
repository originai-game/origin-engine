//! og.Fluid3D, the 3D particle fluid, through its public boundary: the
//! component in a world and `PhysicsSystem::fluid3d_particles` /
//! `fluid3d_mix` read back.
//!
//! Ways this could fail, each caught below: pressure does nothing (the
//! particles pile into one another: the pool test's spacing); there is no
//! cohesion (a drop in zero gravity flies apart) or no near pressure (it
//! collapses into a point); viscosity is ignored (a viscous fluid spreads
//! as fast as water); a particle leaks through the floor, a solid box,
//! the terrain's height field (parry never reports a point inside one) or
//! the bounds, or turns NaN; the pool never comes to rest (the solver
//! jitters or springs apart); the emitter adds the wrong number, ignores
//! `maxCount` or loses its fraction between ticks; the sink keeps what
//! enters it; two fluids of a group ignore each other or their densities
//! (no layering), mix at `mixing` 0 or do not conserve each fluid's
//! amount at 1; the state lives outside the og.physics state block (a
//! restore diverges); the 2D profile simulates it.

use crate::support;

use og_core::glam::Vec3;
use og_ecs::{Entity, World};
use og_physics::*;
use support::*;

fn world3() -> (World, PhysicsSystem) {
    let (mut w, p) = world(Physics::default(), 1);
    floor(&mut w);
    (w, p)
}

fn particles(p: &PhysicsSystem, e: Entity) -> Vec<Vec3> {
    p.fluid3d_particles(e).expect("a 3D fluid")
}

fn mean(points: &[Vec3]) -> Vec3 {
    points.iter().copied().sum::<Vec3>() / points.len() as f32
}

/// The mean speed over one tick, m/s.
fn speed(w: &mut World, p: &PhysicsSystem, e: Entity) -> f32 {
    let before = particles(p, e);
    run(w, 1);
    let after = particles(p, e);
    after
        .iter()
        .zip(&before)
        .map(|(a, b)| (*a - *b).length() * 60.0)
        .sum::<f32>()
        / after.len() as f32
}

/// Each particle's distance to its nearest neighbour, ascending.
fn spacing(points: &[Vec3]) -> Vec<f32> {
    let mut d: Vec<f32> = points
        .iter()
        .enumerate()
        .map(|(i, a)| {
            points
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, b)| (*b - *a).length())
                .fold(f32::MAX, f32::min)
        })
        .collect();
    d.sort_by(f32::total_cmp);
    d
}

#[test]
fn a_poured_block_settles_into_a_still_pool_inside_its_bounds() {
    let (mut w, p) = world3();
    let fluid = w
        .spawn((
            at(0.0, 1.0, 0.0),
            Fluid3D {
                count: 900,
                size: Vec3::new(1.2, 0.8, 1.2),
                bounds: Vec3::new(1.5, 0.0, 1.5),
                ..Fluid3D::default()
            },
        ))
        .unwrap();
    run(&mut w, 1);
    let start = particles(&p, fluid);
    assert_eq!(start.len(), 900);
    run(&mut w, 359);
    let still = speed(&mut w, &p, fluid);
    let pool = particles(&p, fluid);
    let top = pool.iter().map(|q| q.y).fold(f32::MIN, f32::max);
    let gaps = spacing(&pool);
    let (close, median) = (gaps[gaps.len() / 100], gaps[gaps.len() / 2]);
    eprintln!(
        "after 6 s: mean speed {still:.3} m/s, top {top:.3} m, mean y {:.3} m, nearest neighbour 1st percentile {close:.3} m, median {median:.3} m",
        mean(&pool).y
    );
    for q in &pool {
        assert!(q.is_finite(), "a particle went NaN");
        assert!(q.y > 0.0, "{q} under the floor");
        assert!(
            q.x.abs() <= 0.75 + 1e-4 && q.z.abs() <= 0.75 + 1e-4,
            "{q} outside the 1.5 m bounds"
        );
    }
    assert!(still < 0.15, "the pool is still moving at {still} m/s");
    assert!(top < 0.9, "it fell and spread (top {top} m)");
    // Pressure keeps the particles apart: the pool is not a pile of
    // coincident points (the radius is 0.2 m).
    assert!(close > 0.04, "1 % of the particles are within {close} m");
    assert!(median > 0.06 && median < 0.16, "median spacing {median} m");
}

#[test]
fn an_emitter_adds_its_rate_up_to_max_count_and_a_sink_takes_what_enters() {
    let (mut w, p) = world3();
    let e = w
        .spawn((
            at(0.0, 2.0, 0.0),
            Fluid3D {
                count: 0,
                max_count: 400,
                emit_rate: 270.0,
                emit_velocity: Vec3::new(1.0, 0.0, 0.0),
                ..Fluid3D::default()
            },
        ))
        .unwrap();
    run(&mut w, 1);
    // 270 a second is 4.5 a tick: the half carries over.
    assert_eq!(particles(&p, e).len(), 4);
    run(&mut w, 59);
    assert_eq!(particles(&p, e).len(), 270, "a second of 270/s");
    run(&mut w, 60);
    assert_eq!(particles(&p, e).len(), 400, "capped at maxCount");

    // A sink on the floor under a stream: what lands is removed.
    let (mut w, p) = world3();
    let e = w
        .spawn((
            at(0.0, 2.0, 0.0),
            Fluid3D {
                count: 0,
                emit_rate: 600.0,
                sink_offset: Vec3::new(0.0, -1.8, 0.0),
                sink_size: Vec3::new(3.0, 0.4, 3.0),
                ..Fluid3D::default()
            },
        ))
        .unwrap();
    let mut most = 0;
    for _ in 0..300 {
        run(&mut w, 1);
        let pts = particles(&p, e);
        most = most.max(pts.len());
        assert!(
            !pts.iter()
                .any(|q| q.x.abs() < 1.499 && q.z.abs() < 1.499 && q.y < 0.399),
            "a particle stayed in the sink"
        );
    }
    eprintln!("a 600/s stream into a sink: at most {most} particles alive over 5 s");
    assert!(most > 100 && most < 600, "{most} alive at once");
}

#[test]
fn a_restored_fluid_continues_exactly_and_the_seed_matters() {
    let build = |seed: u32| {
        let (mut w, p) = world3();
        let e = w
            .spawn((
                at(0.0, 1.0, 0.0),
                Fluid3D {
                    count: 300,
                    emit_rate: 120.0,
                    emit_offset: Vec3::new(0.0, 1.0, 0.0),
                    sink_offset: Vec3::new(0.6, -0.9, 0.0),
                    sink_size: Vec3::new(0.4, 0.4, 0.4),
                    seed,
                    ..Fluid3D::default()
                },
            ))
            .unwrap();
        (w, p, e)
    };
    let (mut a, pa, ea) = build(3);
    run(&mut a, 90);
    let half = a.snapshot();
    run(&mut a, 90);
    let (mut b, pb, eb) = build(3);
    b.restore(&half).unwrap();
    run(&mut b, 90);
    assert_eq!(particles(&pa, ea), particles(&pb, eb));
    assert_eq!(a.world_hash(), b.world_hash());
    let (mut c, pc, ec) = build(4);
    run(&mut c, 180);
    assert_ne!(particles(&pa, ea), particles(&pc, ec), "another seed");
}
