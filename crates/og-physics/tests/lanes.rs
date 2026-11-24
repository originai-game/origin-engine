//! Lanes change where physics runs, never what it computes: the step, the
//! soft bodies stepped side by side and the fluids' per-particle passes end
//! in the same world on the lanes as off them, and physics work outside
//! the step (queries) stays on the calling thread, where an injected panic
//! is the structured crash the hosts report.
//!
//! Ways this could fail, each of which a test here would catch: a soft
//! body stepped from another one's state or in a different order of its
//! passes (the hashes differ); a fluid's neighbour pairs joined out of
//! particle order (the Jacobi gathers differ); a
//! query reaching rayon's global pool or another thread (the calling thread
//! is left in no pool); a panic in a query lost or attributed to a lane.
//!
//! Each world carries its own lanes setting (`PhysicsSystem::set_lanes`),
//! so these tests run beside any other without sharing a switch.

use crate::support;

use og_core::glam::{Vec2, Vec3};
use og_ecs::World;
use og_physics::*;
use support::{at, beyond, floor, spawn_static};

type Build = (World, PhysicsSystem, Box<dyn Fn(&mut World, u64)>);

/// `build`'s world stepped `ticks` with every pass split (`Grains::always`,
/// on the pool when this process has one), with its lanes off, and with
/// the measured grains: one hash and one snapshot. Returns the parallel
/// calls this thread started in the split run (the step decides here
/// whether it goes to the pool).
fn same_everywhere(ticks: u32, build: impl Fn() -> Build) -> u64 {
    let run = |lanes: PhysicsLanes| {
        let (mut w, physics, drive) = build();
        physics.set_lanes(lanes);
        let before = og_lanes::calls_here();
        for tick in 0..ticks as u64 {
            drive(&mut w, tick);
            w.step().unwrap();
        }
        (
            (w.world_hash(), w.snapshot()),
            og_lanes::calls_here() - before,
        )
    };
    let (split, calls) = run(PhysicsLanes {
        grains: Some(Grains::always()),
        ..PhysicsLanes::default()
    });
    let (off, off_calls) = run(PhysicsLanes {
        on: false,
        grains: None,
    });
    let (measured, _) = run(PhysicsLanes::default());
    assert_eq!(off_calls, 0, "a world with its lanes off split");
    assert_eq!(split.0, off.0, "split and off diverged");
    assert_eq!(split.1, off.1);
    assert_eq!(measured.0, off.0, "measured grains and off diverged");
    if og_lanes::report().on {
        assert!(calls > 0, "nothing split");
    } else {
        assert_eq!(calls, 0);
    }
    calls
}

/// Many ropes, cloths and fluids at once, each kind enough for several
/// lane tasks, and fluids large enough for their per-particle passes to
/// split by ranges under the measured grains too.
#[test]
fn many_soft_bodies_split_and_off_the_lanes_end_in_the_same_world() {
    let calls = same_everywhere(120, || {
        let (mut w, physics) = beyond::new_world(3, PROFILE_3D);
        floor(&mut w);
        spawn_static(
            &mut w,
            at(0.0, 1.0, 0.0),
            Collider::cuboid(Vec3::new(6.0, 0.4, 0.6)),
        );
        for i in 0..48 {
            let x = -6.0 + (i % 12) as f32;
            let z = -2.0 + (i / 12) as f32;
            let end = if i % 3 == 0 {
                w.spawn((at(x + 0.4, 2.0, z),)).unwrap()
            } else {
                w.spawn((
                    at(x + 0.4, 2.0, z),
                    RigidBody::dynamic(),
                    Collider::sphere(0.1),
                    Velocity::default(),
                ))
                .unwrap()
            };
            w.spawn((
                at(x, 3.5, z),
                Rope {
                    end: end.to_bits(),
                    coupling: true,
                    wind: Vec3::new(0.5, 0.0, 0.2),
                    ..Rope::default()
                },
            ))
            .unwrap();
        }
        for i in 0..16 {
            w.spawn((
                at(-8.0 + i as f32, 2.5, 3.0),
                Cloth {
                    bending: 2.0,
                    tether: 10000.0,
                    drag: 0.08,
                    lift: 0.04,
                    pins: if i % 2 == 0 { PIN_TOP } else { PIN_TOP_CORNERS },
                    thickness: if i % 4 == 0 { 0.05 } else { 0.0 },
                    wind: Vec3::new(0.0, 0.0, 1.0),
                    ..Cloth::default()
                },
            ))
            .unwrap();
        }
        for (i, count) in [3000u32, 1200, 1200, 600].into_iter().enumerate() {
            w.spawn((
                at(if i == 3 { 2.55 } else { -6.0 + 4.0 * i as f32 }, 1.5, -6.0),
                Fluid3D {
                    count,
                    solver_iterations: 4,
                    viscosity: 4.0,
                    vorticity: 0.02,
                    mixing: 0.4,
                    density: 1.0 + i as f32 * 0.3,
                    bounds: Vec3::new(3.0, 6.0, 3.0),
                    group: if i >= 2 { 1 } else { 0 },
                    ..Fluid3D::default()
                },
            ))
            .unwrap();
        }
        (w, physics, Box::new(|_: &mut World, _: u64| {}))
    });
    println!("{calls} parallel calls at the physics site");
    let calls = same_everywhere(120, || {
        let (mut w, physics) = beyond::new_world(4, PROFILE_2D);
        floor(&mut w);
        for i in 0..6 {
            w.spawn((
                at(if i == 5 { 8.6 } else { -12.0 + 5.0 * i as f32 }, 2.0, 0.0),
                Fluid2D {
                    solver_iterations: 4,
                    viscosity: 4.0,
                    vorticity: 0.02,
                    mixing: 0.4,
                    count: if i == 0 { 2600 } else { 700 },
                    size: Vec2::new(3.0, 3.0),
                    bounds: Vec2::new(4.5, 8.0),
                    group: if i >= 4 { 2 } else { 0 },
                    density: if i == 5 { 2.0 } else { 1.0 },
                    ..Fluid2D::default()
                },
            ))
            .unwrap();
        }
        for i in 0..12 {
            w.spawn((at(-6.0 + i as f32, 6.0, 0.0), Rope::default()))
                .unwrap();
        }
        (w, physics, Box::new(|_: &mut World, _: u64| {}))
    });
    println!("{calls} parallel calls at the physics site (2D)");
}

/// Many ragdolls in both profiles, planned by ranges of ragdolls (every
/// pass split) and alone, each following moving targets, going limp with a
/// hit at its own tick, recovering, and one rebuilt with a new rig: the
/// same world either way.
#[test]
fn many_ragdolls_split_and_off_the_lanes_end_in_the_same_world() {
    for profile in [PROFILE_3D, PROFILE_2D] {
        same_everywhere(160, || {
            let (mut w, physics) = beyond::new_world(6, profile);
            floor(&mut w);
            let rig = if profile == PROFILE_3D {
                beyond::rig()
            } else {
                beyond::rig_2d()
            };
            let mut ragdolls = Vec::new();
            for i in 0..24 {
                let (x, z) = (-12.0 + (i % 12) as f32 * 2.0, (i / 12) as f32 * 2.0);
                let e = w
                    .spawn((
                        at(x, 0.0, z),
                        Ragdoll {
                            blend_time: 0.1 + 0.05 * (i % 3) as f32,
                            ..Ragdoll::default()
                        },
                    ))
                    .unwrap();
                physics.set_ragdoll_rig(e, rig.clone()).unwrap();
                ragdolls.push((e, x, z));
            }
            let handle = physics.clone();
            let drive = move |w: &mut World, tick: u64| {
                for (k, &(e, x, z)) in ragdolls.iter().enumerate() {
                    let mut r = w.get::<Ragdoll>(e).unwrap();
                    let from = 20 + 5 * k as u64;
                    r.weight = if (from..from + 60).contains(&tick) {
                        1.0
                    } else {
                        0.0
                    };
                    r.impulse = Vec3::new(10.0 + k as f32, 6.0, 0.0);
                    w.insert(e, r).unwrap();
                    let s = tick as f32 * 0.05 + k as f32;
                    let targets = rig
                        .bones
                        .iter()
                        .map(|b| {
                            let y = b.rest.0.y;
                            (
                                Vec3::new(x + og_core::math::sin(s) * 0.2 * y, y, z),
                                b.rest.1,
                            )
                        })
                        .collect();
                    physics.set_ragdoll_targets(e, targets);
                }
                if tick == 90 {
                    // A new rig rebuilds that ragdoll.
                    let mut longer = rig.clone();
                    longer.bones[0].length *= 1.5;
                    physics.set_ragdoll_rig(ragdolls[3].0, longer).unwrap();
                }
            };
            (w, handle, Box::new(drive))
        });
    }
}
