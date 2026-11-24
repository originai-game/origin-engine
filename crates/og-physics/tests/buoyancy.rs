//! og.Buoyancy: a swimmer and a floating ragdoll in a gale's sea run the
//! same twice and through a restore.
//!
//! Ways this could fail: buoyancy or swimming state kept outside the world,
//! or an iteration order that is not fixed: a second run or a restored run
//! diverges.

use crate::support;

use og_core::glam::{DVec3, Quat, Vec2, Vec3};
use og_ecs::{Entity, World};
use og_env::ocean::Ocean;
use og_physics::*;
use support::*;

fn world(seed: u32) -> (World, PhysicsSystem) {
    let (mut w, p) = support::world(Physics::default(), seed);
    support::enable_lazy(&mut w);
    (w, p)
}

/// A flat lake 200 m across, its surface at y 0.
fn lake() -> WaterBody {
    WaterBody {
        entity: 1 << 40,
        position: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        depth: 200.0,
        size: Vec2::splat(200.0),
        endless: false,
        sea: None,
        zero: DVec3::ZERO,
        shore: None,
        river: None,
        mesh: None,
    }
}

/// A gale's sea 6 km across (the storm frame's) centred on the absolute
/// world's zero, seen with og.WorldOrigin's zero at `zero` (the water's
/// entity moved by it, as every place in the world is).
fn sea(zero: DVec3) -> WaterBody {
    WaterBody {
        position: -zero.as_vec3(),
        sea: Ocean::new(Vec2::new(7.0, -18.7), 1.0, 1.0),
        size: Vec2::splat(6000.0),
        zero,
        ..lake()
    }
}

/// Hips, a 0.5 m spine and a 0.3 m neck, straight up from the entity.
fn rig() -> RagdollRig {
    let bone = |parent: Option<u32>, y: f32, length: f32| RagdollBone {
        parent,
        rest: (Vec3::new(0.0, y, 0.0), Quat::IDENTITY),
        length,
        axis: Vec3::Y,
    };
    RagdollRig {
        bones: vec![
            bone(None, 0.0, 0.3),
            bone(Some(0), 0.3, 0.5),
            bone(Some(1), 0.8, 0.3),
        ],
    }
}

/// A limp ragdoll dropped from `y` over the lake at `x`.
fn limp(w: &mut World, p: &PhysicsSystem, x: f32, y: f32) -> Entity {
    let ragdoll = Ragdoll {
        weight: 1.0,
        ..Ragdoll::default()
    };
    let e = w.spawn((at(x, y, 0.0), ragdoll)).unwrap();
    p.set_ragdoll_rig(e, rig()).unwrap();
    e
}

fn mean_height(p: &PhysicsSystem, e: Entity) -> f32 {
    let pose = p.ragdoll_pose(e).unwrap();
    pose.iter().map(|b| b.0.y).sum::<f32>() / pose.len() as f32
}

/// A capsule character (0.4 radius, 1.8 m tall) with its middle at `y`.
fn swimmer(w: &mut World, x: f32, y: f32, b: Option<Buoyancy>) -> Entity {
    let e = w
        .spawn((
            at(x, y, 0.0),
            Collider::capsule(0.4, 1.0),
            CharacterController {
                up: Vec3::Y,
                ..CharacterController::default()
            },
        ))
        .unwrap();
    if let Some(b) = b {
        w.insert(e, b).unwrap();
    }
    e
}

#[test]
fn a_swimmer_and_a_floating_ragdoll_run_the_same_twice_and_through_a_restore() {
    let build = || {
        let (mut w, p) = world(11);
        p.set_waters(vec![sea(DVec3::ZERO)]);
        let c = swimmer(&mut w, 2.0, 2.0, Some(Buoyancy::default()));
        let r = limp(&mut w, &p, -3.0, 2.0);
        w.insert(
            r,
            Buoyancy {
                water_density: 1000.0,
                ..Buoyancy::default()
            },
        )
        .unwrap();
        (w, p, c, r)
    };
    let state = |w: &World, p: &PhysicsSystem, c: Entity, r: Entity| {
        let t = transform(w, c).position;
        let mut bits = vec![t.x.to_bits(), t.y.to_bits(), t.z.to_bits()];
        for (at, _) in p.ragdoll_pose(r).unwrap() {
            bits.extend([at.x.to_bits(), at.y.to_bits(), at.z.to_bits()]);
        }
        bits.push(w.get::<Buoyancy>(c).unwrap().submerged.to_bits());
        bits.push(w.get::<Buoyancy>(r).unwrap().submerged.to_bits());
        bits
    };
    let (mut a, pa, c, r) = build();
    run(&mut a, 120);
    let saved = a.snapshot();
    run(&mut a, 240);
    let first = state(&a, &pa, c, r);
    // In a gale's sea they ride the waves: in the water, not flung or sunk.
    let t = transform(&a, c).position;
    assert!(t.y.abs() < 8.0, "{t}");
    assert!(mean_height(&pa, r).abs() < 8.0);
    let (mut b, pb, ..) = build();
    run(&mut b, 360);
    assert_eq!(state(&b, &pb, c, r), first);
    let (mut d, pd, ..) = build();
    d.restore(&saved).unwrap();
    run(&mut d, 240);
    assert_eq!(state(&d, &pd, c, r), first);

    // A canal cannot capture a point below its floor. Overlapping volumes
    // choose the highest *enclosing* level, not the highest in xz alone.
    let canal = WaterBody {
        entity: 2 << 40,
        position: Vec3::Y * 10.0,
        depth: 2.0,
        ..lake()
    };
    let overlap = WaterBody {
        entity: 3 << 40,
        position: Vec3::Y,
        depth: 3.0,
        ..lake()
    };
    let waters = [lake(), canal, overlap];
    assert_eq!(
        pa.water_at(&waters, &[], Vec3::new(0.0, -1.0, 0.0), 0.0)
            .unwrap()
            .water,
        3 << 40
    );
    assert_eq!(
        pa.water_at(&waters, &[], Vec3::new(0.0, 9.0, 0.0), 0.0)
            .unwrap()
            .water,
        2 << 40
    );
    assert!(
        pa.water_at(&waters, &[], Vec3::new(0.0, 5.0, 0.0), 0.0)
            .is_none()
    );
    assert!(
        pa.water_at(
            &[WaterBody {
                depth: 0.0,
                ..lake()
            }],
            &[],
            -Vec3::Y,
            0.0
        )
        .is_none()
    );
    let rotated = WaterBody {
        rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
        size: Vec2::new(8.0, 2.0),
        ..lake()
    };
    assert!(
        pa.water_at(
            std::slice::from_ref(&rotated),
            &[],
            Vec3::new(0.0, -1.0, 3.0),
            0.0
        )
        .is_some()
    );
    assert!(
        pa.water_at(&[rotated], &[], Vec3::new(3.0, -1.0, 0.0), 0.0)
            .is_none()
    );

    // A fully immersed 2×1×3 hull displaces 6 m³ regardless of its OBB's
    // rotation. Two identical pools must not double that lift. At mass
    // 3 kg, lift 9.81 N/m³ gives net acceleration +9.81 m/s².
    let pool_velocity = |copies: u32, rotation: Quat, substeps| {
        let (mut w, _) = support::world(
            Physics {
                substeps,
                ..Physics::default()
            },
            7,
        );
        support::enable_lazy(&mut w);
        for _ in 0..copies {
            w.spawn((
                at(0.0, 0.0, 0.0),
                WaterSurface {
                    depth: 10.0,
                    drag: 0.0,
                    ..WaterSurface::default()
                },
            ))
            .unwrap();
        }
        let e = w
            .spawn((
                Transform {
                    quaternion: rotation,
                    ..at(0.0, -3.0, 0.0)
                },
                RigidBody {
                    mass: 3.0,
                    ..RigidBody::dynamic()
                },
                Collider::cuboid(Vec3::new(2.0, 1.0, 3.0)),
            ))
            .unwrap();
        run(&mut w, 1);
        velocity(&w, e).linear.y
    };
    for substeps in [1, 4] {
        let single = pool_velocity(1, Quat::IDENTITY, substeps);
        assert!((single - 9.81 / 60.0).abs() < 1e-4, "{single}");
        assert_eq!(single, pool_velocity(2, Quat::IDENTITY, substeps));
        let tilted = pool_velocity(1, Quat::from_rotation_z(0.7), substeps);
        assert!((tilted - single).abs() < 1e-4, "{single} {tilted}");
    }

    // The same Buoyancy component lifts a 2D hull, not the pool's own
    // lift (explicitly disabled). Fully under: density 2 yields +g.
    let (mut w, _) = support::world(
        Physics {
            profile: PROFILE_2D,
            substeps: 4,
            ..Physics::default()
        },
        17,
    );
    support::enable_lazy(&mut w);
    w.spawn((
        at(0.0, 0.0, 0.0),
        WaterLine {
            depth: 10.0,
            buoyancy: 0.0,
            drag: 0.0,
            ..WaterLine::default()
        },
    ))
    .unwrap();
    let e = w
        .spawn((
            at(0.0, -3.0, 0.0),
            RigidBody::dynamic(),
            Collider::cuboid(Vec3::new(2.0, 1.0, 1.0)),
            Buoyancy {
                drag: 0.0,
                angular_drag: 0.0,
                ..Buoyancy::default()
            },
        ))
        .unwrap();
    run(&mut w, 1);
    assert!((velocity(&w, e).linear.y - 9.81 / 60.0).abs() < 1e-4);
    assert_eq!(w.get::<Buoyancy>(e).unwrap().submerged, 1.0);

    // The same finite pool clips at both its side and its floor. Only
    // one quarter of this collider is inside; counting its whole box or
    // clipping only at the surface gives a different acceleration.
    for profile in [PROFILE_2D, PROFILE_3D] {
        let (mut w, _) = support::world(
            Physics {
                profile,
                substeps: 1,
                ..Physics::default()
            },
            19,
        );
        support::enable_lazy(&mut w);
        if profile == PROFILE_2D {
            w.spawn((
                at(0.0, 0.0, 0.0),
                WaterLine {
                    width: 2.0,
                    depth: 1.0,
                    buoyancy: 9.81,
                    drag: 0.0,
                    ..WaterLine::default()
                },
            ))
            .unwrap();
        } else {
            w.spawn((
                at(0.0, 0.0, 0.0),
                WaterSurface {
                    size: 2.0,
                    depth: 1.0,
                    buoyancy: 9.81,
                    drag: 0.0,
                    ..WaterSurface::default()
                },
            ))
            .unwrap();
        }
        let e = w
            .spawn((
                at(1.0, -1.0, 0.0),
                RigidBody::dynamic(),
                Collider::cuboid(Vec3::splat(2.0)),
            ))
            .unwrap();
        run(&mut w, 1);
        let vy = velocity(&w, e).linear.y;
        assert!((vy + 0.75 * 9.81 / 60.0).abs() < 1e-4, "{profile}: {vy}");
    }

    // Active waves, including shallow-water momentum, survive a restore
    // exactly. A horizontal hull excites the 3D field without a crossing.
    for profile in [PROFILE_2D, PROFILE_3D] {
        let gravity = if profile == PROFILE_3D {
            Vec3::ZERO
        } else {
            Physics::default().gravity
        };
        let (mut w, p) = support::world(
            Physics {
                profile,
                gravity,
                ..Physics::default()
            },
            23,
        );
        support::enable_lazy(&mut w);
        let pool = if profile == PROFILE_2D {
            w.spawn((
                at(0.0, 0.0, 0.0),
                WaterLine {
                    width: 6.0,
                    depth: 2.0,
                    splash: 4.0,
                    buoyancy: 0.0,
                    drag: 0.0,
                    ..WaterLine::default()
                },
            ))
            .unwrap()
        } else {
            w.spawn((
                at(0.0, 0.0, 0.0),
                WaterSurface {
                    size: 6.0,
                    depth: 2.0,
                    wave_speed: 20.0,
                    splash: 4.0,
                    buoyancy: 0.0,
                    drag: 0.0,
                    ..WaterSurface::default()
                },
            ))
            .unwrap()
        };
        w.spawn((
            at(-1.0, 0.2, 0.0),
            RigidBody::dynamic(),
            Collider::cuboid(Vec3::splat(0.8)),
            Velocity {
                linear: Vec3::new(2.0, if profile == PROFILE_3D { 0.0 } else { -1.0 }, 0.0),
                angular: Vec3::ZERO,
            },
        ))
        .unwrap();
        let heights = |p: &PhysicsSystem| {
            p.water_line(pool)
                .or_else(|| p.water_heights(pool).map(|h| h.heights))
                .unwrap()
        };
        run(&mut w, 20);
        assert!(heights(&p).iter().any(|h| h.abs() > 1e-5));
        let saved = w.snapshot();
        run(&mut w, 30);
        let expected = heights(&p);
        let hash = w.world_hash();
        w.restore(&saved).unwrap();
        run(&mut w, 30);
        assert_eq!(heights(&p), expected);
        assert_eq!(w.world_hash(), hash);
    }
}
