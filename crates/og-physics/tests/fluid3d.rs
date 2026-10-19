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

mod support;

use og_core::glam::{Vec2, Vec3};
use og_ecs::{Entity, World};
use og_physics::*;
use std::sync::Arc;
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
fn a_drop_in_zero_gravity_holds_together_and_rounds() {
    let (mut w, p) = world(Physics::default(), 1);
    let drop = w
        .spawn((
            at(0.0, 5.0, 0.0),
            Fluid3D {
                count: 512,
                size: Vec3::splat(0.8),
                gravity_scale: 0.0,
                ..Fluid3D::default()
            },
        ))
        .unwrap();
    let far = |pts: &[Vec3]| {
        let c = mean(pts);
        pts.iter().map(|q| (*q - c).length()).fold(0.0f32, f32::max)
    };
    run(&mut w, 1);
    let cube = far(&particles(&p, drop));
    let mut farthest = 0.0f32;
    for _ in 0..599 {
        run(&mut w, 1);
        farthest = farthest.max(far(&particles(&p, drop)));
    }
    let pts = particles(&p, drop);
    let c = mean(&pts);
    let mut d: Vec<f32> = pts.iter().map(|q| (*q - c).length()).collect();
    d.sort_by(f32::total_cmp);
    let (median, outer) = (d[d.len() / 2], d[d.len() * 99 / 100]);
    eprintln!(
        "a cube's corners at {cube:.3} m from its centre; over 10 s no particle beyond {farthest:.3} m; at the end the median at {median:.3} m, the 99th percentile at {outer:.3} m"
    );
    // Cohesion holds it together (nothing flies off), near pressure keeps
    // it from collapsing, and surface tension pulls the corners in: a
    // sphere of the cube's volume has a radius of 0.496 m.
    assert!(farthest < 0.9, "a particle got {farthest} m away");
    assert!(median > 0.3 && median < 0.45, "median {median} m");
    assert!(outer < 0.55, "the corners stayed out ({outer} m)");
}

#[test]
fn a_viscous_fluid_spreads_slower_than_water() {
    let spread = |viscosity: f32| {
        let (mut w, p) = world3();
        let e = w
            .spawn((
                at(0.0, 0.6, 0.0),
                Fluid3D {
                    count: 600,
                    size: Vec3::new(0.6, 1.0, 0.6),
                    viscosity,
                    ..Fluid3D::default()
                },
            ))
            .unwrap();
        run(&mut w, 60);
        let pts = particles(&p, e);
        pts.iter()
            .map(|q| Vec2::new(q.x, q.z).length())
            .sum::<f32>()
            / pts.len() as f32
    };
    let (water, honey) = (spread(0.05), spread(3.0));
    eprintln!(
        "mean distance from the centre after 1 s: viscosity 0.05 {water:.3} m, 3 {honey:.3} m"
    );
    assert!(honey < water * 0.85, "honey {honey} m, water {water} m");
}

/// A bowl 8 × 8 m: y = 0.15 (x² + z²), 33 × 33 samples.
fn bowl(p: &PhysicsSystem, w: &mut World) -> impl Fn(f32, f32) -> f32 + use<> {
    let f = |x: f32, z: f32| 0.15 * (x * x + z * z);
    let (n, size) = (33u32, 8.0f32);
    let mut heights = Vec::new();
    for r in 0..n {
        for c in 0..n {
            let x = -size * 0.5 + c as f32 * size / (n - 1) as f32;
            let z = -size * 0.5 + r as f32 * size / (n - 1) as f32;
            heights.push(f(x, z));
        }
    }
    p.set_heightfield(
        5,
        HeightfieldDesc {
            columns: n,
            rows: n,
            size: Vec2::splat(size),
            heights: Arc::from(heights),
        },
    )
    .unwrap();
    w.spawn((
        at(0.0, 0.0, 0.0),
        Collider {
            shape: SHAPE_HEIGHTFIELD,
            mesh: 5,
            ..Collider::default()
        },
    ))
    .unwrap();
    f
}

#[test]
fn fluid_poured_into_a_terrain_bowl_runs_to_its_bottom_and_never_through() {
    let (mut w, p) = world(Physics::default(), 1);
    let ground = bowl(&p, &mut w);
    let e = w
        .spawn((
            at(2.0, 2.5, 0.5),
            Fluid3D {
                count: 600,
                size: Vec3::new(0.8, 0.8, 0.8),
                ..Fluid3D::default()
            },
        ))
        .unwrap();
    let mut lowest = f32::MAX;
    for _ in 0..360 {
        run(&mut w, 1);
        for q in particles(&p, e) {
            assert!(q.is_finite());
            // The cell's triangles are flat between samples 0.25 m apart:
            // they lie at most 0.0024 m above the parabola.
            lowest = lowest.min(q.y - ground(q.x, q.z));
        }
    }
    let c = mean(&particles(&p, e));
    eprintln!(
        "after 6 s the pool's centre is at {c}; the lowest particle was {lowest:.4} m over the bowl"
    );
    assert!(
        lowest > -0.01,
        "a particle went {lowest} m into the terrain"
    );
    assert!(
        Vec2::new(c.x, c.z).length() < 0.2,
        "it ran down to the bottom, centre {c}"
    );
}

#[test]
fn fluid_poured_on_a_box_flows_around_it_and_never_inside() {
    let (mut w, p) = world3();
    spawn_static(
        &mut w,
        at(0.0, 0.4, 0.0),
        Collider::cuboid(Vec3::new(0.8, 0.8, 0.8)),
    );
    let e = w
        .spawn((
            at(0.0, 1.6, 0.0),
            Fluid3D {
                count: 500,
                size: Vec3::new(0.6, 0.6, 0.6),
                ..Fluid3D::default()
            },
        ))
        .unwrap();
    let inside = |q: &Vec3| q.x.abs() < 0.399 && q.z.abs() < 0.399 && q.y > 0.001 && q.y < 0.799;
    for _ in 0..240 {
        run(&mut w, 1);
        let pts = particles(&p, e);
        assert!(!pts.iter().any(inside), "a particle is inside the box");
    }
    let pts = particles(&p, e);
    let on_top = pts.iter().filter(|q| q.y > 0.8).count();
    let floor = pts.iter().filter(|q| q.y < 0.3).count();
    eprintln!("after 4 s: {on_top} particles on the box, {floor} on the floor beside it");
    assert!(floor > 250, "it ran off the box ({floor} on the floor)");
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

fn tank(mixing: f32) -> (World, PhysicsSystem, Entity, Entity) {
    let (mut w, p) = world3();
    let fluid = |density: f32, seed: u32| Fluid3D {
        count: 400,
        size: Vec3::new(1.0, 0.5, 1.0),
        bounds: Vec3::new(1.0, 0.0, 1.0),
        group: 1,
        density,
        mixing,
        seed,
        ..Fluid3D::default()
    };
    let heavy = w.spawn((at(0.0, 1.2, 0.0), fluid(3.0, 1))).unwrap();
    let light = w.spawn((at(0.0, 0.3, 0.0), fluid(1.0, 2))).unwrap();
    (w, p, heavy, light)
}

#[test]
fn a_heavy_fluid_poured_on_a_light_one_sinks_under_it() {
    let (mut w, p, heavy, light) = tank(0.0);
    run(&mut w, 1);
    assert!(mean(&particles(&p, heavy)).y > mean(&particles(&p, light)).y + 0.5);
    run(&mut w, 599);
    // Medians: a few light particles stay trapped on the floor under the
    // heavy fluid, as droplets do.
    let median_y = |e| {
        let mut ys: Vec<f32> = particles(&p, e).iter().map(|q| q.y).collect();
        ys.sort_by(f32::total_cmp);
        ys[ys.len() / 2]
    };
    let (h, l) = (median_y(heavy), median_y(light));
    eprintln!("mixing 0 after 10 s: heavy's median height {h:.3} m, light's {l:.3} m");
    assert!(
        h + 0.1 < l,
        "the heavy fluid sank under (heavy {h}, light {l})"
    );
    let (slot, mix) = p.fluid3d_mix(heavy).unwrap();
    assert_eq!(slot, 0);
    assert!(mix.iter().all(|m| m[0] == 1.0 && m[1] == 0.0), "unmixed");
}

#[test]
fn with_mixing_two_fluids_blend_and_keep_their_amounts() {
    let (mut w, p, heavy, light) = tank(1.0);
    run(&mut w, 600);
    let (_, hm) = p.fluid3d_mix(heavy).unwrap();
    let (slot, lm) = p.fluid3d_mix(light).unwrap();
    assert_eq!(slot, 1);
    let all: Vec<[f32; 4]> = hm.iter().chain(&lm).copied().collect();
    let share = all.iter().map(|m| m[0]).sum::<f32>() / all.len() as f32;
    let pure = all.iter().filter(|m| m[0] > 0.95 || m[0] < 0.05).count();
    eprintln!(
        "mixing 1 after 10 s: the heavy fluid's share {share:.4}, {pure} of 800 particles still pure"
    );
    assert!((share - 0.5).abs() < 1e-3, "share {share}");
    assert!(pure < 80, "{pure} still pure");
    assert!(p.fluid3d_mix(heavy).is_some());
    // A fluid alone carries no mix.
    let (mut w, p) = world3();
    let alone = w.spawn((at(0.0, 1.0, 0.0), Fluid3D::default())).unwrap();
    run(&mut w, 1);
    assert!(p.fluid3d_mix(alone).is_none());
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

#[test]
fn the_2d_profile_does_not_simulate_it() {
    let (mut w, p) = world(
        Physics {
            profile: PROFILE_2D,
            ..Physics::default()
        },
        1,
    );
    let e = w.spawn((at(0.0, 1.0, 0.0), Fluid3D::default())).unwrap();
    run(&mut w, 10);
    assert!(p.fluid3d_particles(e).is_none());
}
