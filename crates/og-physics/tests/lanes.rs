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

mod support;

use og_core::glam::{Vec2, Vec3};
use og_ecs::World;
use og_physics::*;
use std::sync::Mutex;
use support::{at, beyond, floor, pile, spawn_static};

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

/// The beyond scenario (ropes, cloth with self-collision, water, 3D fluids
/// of one group with an emitter and a sink, 2D fluids alone and grouped,
/// ragdolls, characters, cars) in both profiles.
#[test]
fn the_beyond_scenario_split_and_off_the_lanes_ends_in_the_same_world() {
    let _one = one();
    for profile in [PROFILE_3D, PROFILE_2D] {
        same_everywhere(300, || {
            let (w, physics, cast) = beyond::build(7, profile);
            let drive = move |w: &mut World, tick: u64| {
                beyond::drive(w, &physics, &cast, tick, profile);
            };
            (w, Box::new(drive))
        });
    }
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

/// The step's pool threshold follows the thread count, never one thread,
/// and the pen at it goes to the pool on every step with its bodies when
/// this process has a pool, one below it never.
#[test]
fn the_step_goes_to_the_pool_from_the_threshold_of_its_thread_count() {
    let _one = one();
    assert_eq!(pool_bodies(1), usize::MAX);
    assert_eq!(pool_bodies(4), 500);
    assert_eq!(pool_bodies(8), 1500);
    let mut last = 0;
    for threads in 2..=64 {
        let from = pool_bodies(threads);
        assert!(from >= last, "{threads} threads: {from} after {last}");
        last = from;
    }
    let lanes = og_lanes::report();
    if !lanes.on {
        return;
    }
    let from = pool_bodies(lanes.threads);
    for (bodies, pooled) in [(from - 1, false), (from, true)] {
        let before = physics_calls();
        let (mut w, physics) = pile::pen(bodies, 5);
        pile::run(&mut w, 10);
        let after = physics_calls();
        let stats = physics.stats();
        assert_eq!(stats.pool, pooled, "{bodies} bodies");
        assert_eq!(stats.phase(Phase::Rapier).split, pooled);
        if pooled {
            assert!(after.calls - before.calls >= 10, "{bodies} bodies");
            assert!(after.items - before.items >= 10 * bodies as u64);
        }
    }
}

/// `PhysicsSystem::stats` names every phase in step order, times them
/// within the whole system's time, and says which went to the pool: none
/// with lanes off, the step and the passes that split with every pass
/// forced to (`Grains::always`) when this process has a pool.
#[test]
fn stats_say_what_each_phase_cost_and_where_it_ran() {
    let _one = one();
    let (mut w, physics) = pile::pen(64, 3);
    assert_eq!(
        physics.stats(),
        PhysicsStats::default(),
        "before the first step"
    );
    let names: Vec<&str> = Phase::ALL.iter().map(|p| p.name()).collect();
    assert_eq!(
        names,
        [
            "gather",
            "sync",
            "joints",
            "ragdolls",
            "water",
            "rapier",
            "readout",
            "controllers",
            "soft",
            "fluids",
            "fluids3d",
            "writeBack"
        ]
    );
    og_lanes::set_enabled(false);
    pile::run(&mut w, 5);
    og_lanes::set_enabled(true);
    let off = physics.stats();
    assert_eq!(off.tick, 4, "the last step's tick");
    assert_eq!(off.threads, 1);
    assert!(!off.pool);
    assert!(off.bodies >= 64, "{} bodies", off.bodies);
    assert_eq!(off.phases.iter().map(|p| p.name).collect::<Vec<_>>(), names);
    assert!(off.phases.iter().all(|p| !p.split && p.ms >= 0.0));
    assert!(off.phase(Phase::Rapier).ms > 0.0, "{off:?}");
    let sum: f64 = off.phases.iter().map(|p| p.ms).sum();
    assert!(sum <= off.ms + 1e-6, "{sum} ms of phases in {} ms", off.ms);
    set_grains(Some(Grains::always()));
    pile::run(&mut w, 1);
    set_grains(None);
    let split = physics.stats();
    let on = og_lanes::report().on;
    assert_eq!(split.pool, on);
    for phase in [Phase::Rapier, Phase::Sync, Phase::Readout] {
        assert_eq!(split.phase(phase).split, on, "{phase:?}: {split:?}");
    }
    assert!(
        !split.phase(Phase::Fluids3d).split,
        "no fluid to split: {split:?}"
    );
}

/// A pen of `n` of the pile's bodies under a 48 × 48 triangle-mesh
/// terrain (at y = 58, give or take 0.1).
fn meshed(n: usize) -> (World, PhysicsSystem) {
    let (mut w, physics) = pile::pen(n, 9);
    let side = 49u32;
    let positions = (0..side * side)
        .map(|k| {
            let (x, z) = ((k % side) as f32, (k / side) as f32);
            Vec3::new(
                x - 24.0,
                -2.0 + 0.1 * og_core::math::sin(x * 0.7 + z),
                z - 24.0,
            )
        })
        .collect();
    let mut indices = Vec::new();
    for z in 0..side - 1 {
        for x in 0..side - 1 {
            let k = z * side + x;
            indices.push([k, k + side, k + 1]);
            indices.push([k + 1, k + side, k + side + 1]);
        }
    }
    physics
        .set_mesh(77, MeshDesc { positions, indices })
        .unwrap();
    // Above the pen's floor and its bodies.
    spawn_static(&mut w, at(0.0, 60.0, 0.0), Collider::trimesh(77));
    (w, physics)
}

fn ask(w: &World, physics: &PhysicsSystem) -> (Option<RaycastHit>, Option<ShapeCastHit>, usize) {
    let q = physics.query(w).unwrap();
    let ray = q
        .raycast(&RaycastDesc {
            origin: Vec3::new(3.0, 90.0, -20.0),
            direction: -Vec3::Y,
            ..RaycastDesc::default()
        })
        .unwrap();
    let cast = q
        .shape_cast(&ShapeCastDesc {
            shape: ShapeDesc {
                shape: SHAPE_SPHERE,
                radius: 0.5,
                ..ShapeDesc::default()
            },
            position: Vec3::new(-20.0, 70.0, 20.0),
            direction: -Vec3::Y,
            ..ShapeCastDesc::default()
        })
        .unwrap();
    let overlap = q
        .overlap(&OverlapDesc {
            shape: ShapeDesc {
                shape: SHAPE_BOX,
                size: Vec3::new(30.0, 8.0, 30.0),
                ..ShapeDesc::default()
            },
            position: Vec3::new(0.0, 58.0, 0.0),
            ..OverlapDesc::default()
        })
        .unwrap()
        .len();
    (ray, cast, overlap)
}

/// Queries outside the step, on a thread that has not stepped (as the
/// engine thread before its first tick), run on that thread inside its
/// one-thread pool: rayon reached from them runs there, never on rayon's
/// global pool; the answers are those of a thread that stepped first, with
/// lanes on or off.
#[test]
fn queries_outside_the_step_stay_on_the_calling_thread() {
    let _one = one();
    let fresh = std::thread::spawn(|| {
        let (w, physics) = meshed(64);
        assert_eq!(rayon::current_thread_index(), None, "a fresh thread");
        let answers = ask(&w, &physics);
        assert_eq!(rayon::current_thread_index(), Some(0), "in its own pool");
        assert_eq!(rayon::current_num_threads(), 1);
        answers
    })
    .join()
    .unwrap();
    let ray = fresh.0.expect("the ray meets the floor");
    assert!((ray.point.y - 58.0).abs() < 0.2, "{ray:?}");
    assert!(fresh.1.is_some() && fresh.2 > 0, "{fresh:?}");

    let (mut w, physics) = meshed(64);
    support::run(&mut w, 1);
    let (mut v, physics_v) = meshed(64);
    support::run(&mut v, 1);
    let on = ask(&w, &physics);
    og_lanes::set_enabled(false);
    let off = ask(&v, &physics_v);
    og_lanes::set_enabled(true);
    assert_eq!(on, off);
}

/// A panic in a query, with lanes on and the world's steps on the pool,
/// is the engine thread's: recorded by the hosts' hook with its message,
/// location and tick and no lane (the structured crash og-runtime reports;
/// on the web the engine Worker traps with it), and the next query and
/// step run as before.
#[test]
fn a_panic_in_a_query_is_the_structured_crash() {
    let _one = one();
    set_grains(Some(Grains::always()));
    let (mut w, physics) = pile::pen(200, 3);
    let before = physics_calls();
    support::run(&mut w, 20);
    set_grains(None);
    let on = og_lanes::report().on;
    assert_eq!(physics_calls().calls > before.calls, on, "lanes on: {on}");

    og_lanes::clear_panic();
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(|info| {
        og_lanes::record_panic_info(info, og_ecs::last_tick())
    }));
    og_lanes::inject_fault(og_lanes::Site::Physics);
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        physics
            .query(&w)
            .map(|q| q.raycast(&RaycastDesc::default()))
    }));
    std::panic::set_hook(previous);
    let payload = caught.expect_err("the query panics");
    let text = payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_default();
    assert_eq!(
        text,
        "a fault injected at the physics site (og_lanes::inject_fault)"
    );
    let report = og_lanes::panic_report().expect("recorded");
    assert_eq!(report.message, text);
    assert_eq!(report.lane, None, "the engine thread's own");
    assert_eq!(report.tick, og_ecs::last_tick());
    assert!(report.tick > 0);
    assert!(
        report.location.contains("og-lanes/src/lib.rs"),
        "{report:?}"
    );
    assert_eq!(
        report.describe(),
        format!(
            "panic at tick {}: {text} ({})",
            report.tick, report.location
        )
    );
    og_lanes::clear_panic();

    let q = physics.query(&w).unwrap();
    assert!(
        q.raycast(&RaycastDesc {
            origin: Vec3::new(0.0, 50.0, 0.0),
            direction: -Vec3::Y,
            ..RaycastDesc::default()
        })
        .unwrap()
        .is_some()
    );
    support::run(&mut w, 2);
}
