//! Lanes change where physics runs, never what it computes: the step, the
//! soft bodies stepped side by side and the fluids' per-particle passes end
//! in the same world on the lanes as off them, and physics work outside
//! the step (queries) stays on the calling thread, where an injected panic
//! is the structured crash the hosts report.
//!
//! Ways this could fail, each of which a test here would catch: a soft
//! body stepped from another one's state or in a different order of its
//! passes (the hashes differ); a fluid's neighbour pairs joined out of
//! particle order (the relaxation, Gauss-Seidel over them, differs); a
//! query reaching rayon's global pool or another thread (the calling thread
//! is left in no pool); a panic in a query lost or attributed to a lane.
//!
//! The lanes switch, the grains and an injected fault are the process's:
//! the tests here take turns (`ONE`).

use crate::support;

use og_core::glam::{Vec2, Vec3};
use og_ecs::World;
use og_physics::*;
use std::sync::Mutex;
use support::{at, beyond, floor, spawn_static};

static ONE: Mutex<()> = Mutex::new(());

fn one() -> std::sync::MutexGuard<'static, ()> {
    ONE.lock().unwrap_or_else(|e| e.into_inner())
}

fn physics_calls() -> og_lanes::SiteCount {
    og_lanes::report().sites[og_lanes::Site::Physics as usize]
}

/// `build`'s world stepped `ticks` with every pass split (`Grains::always`,
/// on the pool when this process has one), with lanes switched off, and
/// with the measured grains: one hash and one snapshot. Returns the
/// physics site's parallel calls of the split run.
fn same_everywhere(ticks: u32, build: impl Fn() -> (World, Box<dyn Fn(&mut World, u64)>)) -> u64 {
    let run = || {
        let (mut w, drive) = build();
        for tick in 0..ticks as u64 {
            drive(&mut w, tick);
            w.step().unwrap();
        }
        (w.world_hash(), w.snapshot())
    };
    set_grains(Some(Grains::always()));
    let before = physics_calls();
    let split = run();
    let calls = physics_calls().calls - before.calls;
    set_grains(None);
    og_lanes::set_enabled(false);
    let off = run();
    og_lanes::set_enabled(true);
    let measured = run();
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
    let _one = one();
    let calls = same_everywhere(120, || {
        let (mut w, _physics) = beyond::new_world(3, PROFILE_3D);
        floor(&mut w);
        spawn_static(
            &mut w,
            at(0.0, 1.0, 0.0),
            Collider::cuboid(Vec3::new(6.0, 0.4, 0.6)),
        );
        for i in 0..48 {
            let x = -6.0 + (i % 12) as f32;
            let z = -2.0 + (i / 12) as f32;
            w.spawn((
                at(x, 3.5, z),
                Rope {
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
                at(-6.0 + 4.0 * i as f32, 1.5, -6.0),
                Fluid3D {
                    count,
                    bounds: Vec3::new(3.0, 6.0, 3.0),
                    group: if i >= 2 { 1 } else { 0 },
                    ..Fluid3D::default()
                },
            ))
            .unwrap();
        }
        (w, Box::new(|_: &mut World, _: u64| {}))
    });
    println!("{calls} parallel calls at the physics site");
    let calls = same_everywhere(120, || {
        let (mut w, _physics) = beyond::new_world(4, PROFILE_2D);
        floor(&mut w);
        for i in 0..6 {
            w.spawn((
                at(-12.0 + 5.0 * i as f32, 2.0, 0.0),
                Fluid2D {
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
        (w, Box::new(|_: &mut World, _: u64| {}))
    });
    println!("{calls} parallel calls at the physics site (2D)");
}

/// Many ragdolls in both profiles, planned by ranges of ragdolls (every
/// pass split) and alone, each following moving targets, going limp with a
/// hit at its own tick, recovering, and one rebuilt with a new rig: the
/// same world either way.
#[test]
fn many_ragdolls_split_and_off_the_lanes_end_in_the_same_world() {
    let _one = one();
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
            (w, Box::new(drive))
        });
    }
}
