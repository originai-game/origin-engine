//! The 2D profile: the same components, simulated by rapier2d in the xy
//! plane.
//!
//! Ways this could fail: the 2D step reads or writes the wrong axes (the
//! circle falls along z, or its z is overwritten); rotation is lost in the
//! quaternion round trip (the circle slides instead of rolling, or its
//! quaternion leaves the z axis); a 3D-only shape is silently accepted.

mod support;

use og_core::glam::{Quat, Vec3};
use og_core::math::quat_from_axis_angle;
use og_physics::*;
use support::*;

#[test]
fn a_circle_rolls_down_a_slope() {
    let (mut w, physics) = world(Physics::profile_2d(), 1);
    // A 30 m plank tilted 20 degrees, descending towards +x.
    let tilt = quat_from_axis_angle(Vec3::Z, -20f32.to_radians());
    let slope = spawn_static(
        &mut w,
        rotated(at(0.0, 0.0, 0.0), tilt),
        Collider {
            friction: 1.0,
            ..Collider::cuboid(Vec3::new(30.0, 1.0, 1.0))
        },
    );
    // On the slope near its top: the surface at x = -10 is at
    // y = 10 tan 20° + 0.5 / cos 20°; drop the circle a little above.
    let start = Vec3::new(
        -10.0,
        10.0 * 20f32.to_radians().tan() + 0.5 / 20f32.to_radians().cos() + 0.6,
        0.7,
    );
    let ball = spawn_body(
        &mut w,
        Transform::at(start),
        RigidBody::dynamic(),
        Collider {
            friction: 1.0,
            ..Collider::sphere(0.5)
        },
    );
    run(&mut w, 120);
    let t = transform(&w, ball);
    let v = velocity(&w, ball);
    assert!(
        t.position.x > start.x + 3.0,
        "rolled downhill along +x: {t:?}"
    );
    assert!(t.position.y < start.y - 1.0, "{t:?}");
    assert_eq!(t.position.z, 0.7, "2D keeps z");
    assert_eq!(v.linear.z, 0.0);
    assert_eq!(
        (t.quaternion.x, t.quaternion.y),
        (0.0, 0.0),
        "a rotation about z only"
    );
    assert!(t.quaternion.is_normalized());
    // Rolling, not sliding: clockwise spin, and |ω| r matches the speed.
    let speed = v.linear.length();
    assert!(speed > 2.0, "{v:?}");
    assert!(v.angular.z < 0.0, "{v:?}");
    assert!((-v.angular.z * 0.5 - speed).abs() < 0.05 * speed, "{v:?}");
    // It has been touching the slope the whole time it rolled.
    let touching = w.pending_events::<Contact>().unwrap();
    assert_eq!(touching.len(), 1);
    assert!([touching[0].a, touching[0].b].contains(&slope.to_bits()));

    // Queries run in the plane too: straight down onto the slope.
    let hit = physics
        .query(&w)
        .unwrap()
        .raycast(&RaycastDesc {
            origin: Vec3::new(0.0, 5.0, 0.0),
            direction: -Vec3::Y,
            ..RaycastDesc::default()
        })
        .unwrap()
        .unwrap();
    assert_eq!(hit.entity, slope);
    assert!(
        (hit.distance - (5.0 - 0.5 / 20f32.to_radians().cos())).abs() < 1e-3,
        "{hit:?}"
    );
}

#[test]
fn the_2d_profile_refuses_3d_only_shapes() {
    let (mut w, _p) = world(Physics::profile_2d(), 2);
    let e = spawn_body(
        &mut w,
        at(0.0, 0.0, 0.0),
        RigidBody::dynamic(),
        Collider::cylinder(1.0, 1.0),
    );
    let err = w.step().unwrap_err();
    assert!(err.reason().contains("3D only"), "{}", err.reason());
    assert!(err.reason().contains(&e.to_string()), "{}", err.reason());
}

#[test]
fn a_polygon_from_a_mesh_lands_flat() {
    let (mut w, physics) = world(Physics::profile_2d(), 3);
    physics
        .set_mesh(
            1,
            MeshDesc {
                // A 2 × 1 rectangle as a polygon (z is ignored in 2D).
                positions: vec![
                    Vec3::new(-1.0, -0.5, 3.0),
                    Vec3::new(1.0, -0.5, -3.0),
                    Vec3::new(1.0, 0.5, 0.0),
                    Vec3::new(-1.0, 0.5, 0.0),
                ],
                indices: vec![],
            },
        )
        .unwrap();
    floor(&mut w);
    let plate = spawn_body(
        &mut w,
        at(0.0, 3.0, 0.0),
        RigidBody::dynamic(),
        Collider::convex(1),
    );
    run(&mut w, 180);
    let t = transform(&w, plate);
    assert!((t.position.y - 0.5).abs() < 0.02, "{t:?}");
    assert!(t.quaternion.angle_between(Quat::IDENTITY) < 1e-3, "{t:?}");
}

/// og.Physics.profile changed at run time: a 3D world plays 2D and back.
/// What could go wrong: the simulation of the old profile kept (bodies
/// stop, or step twice), the new one not built from the components (a
/// body jumps back to where it was when the profile last ran), the change
/// not reproducible (two runs differ) or lost across a restore.
#[test]
fn a_world_switches_profile_at_run_time_and_back() {
    let run = || {
        let (mut w, physics) = world(Physics::default(), 1);
        floor(&mut w);
        let ball = spawn_body(
            &mut w,
            at(0.0, 20.0, 0.4),
            RigidBody::dynamic(),
            Collider::sphere(0.5),
        );
        let mut heights = Vec::new();
        let mut snapshot = None;
        for tick in 0..200u32 {
            if tick == 20 || tick == 70 {
                let mut p = *w.resource::<Physics>().unwrap();
                p.profile = if tick == 20 { PROFILE_2D } else { PROFILE_3D };
                w.insert_resource(p).unwrap();
            }
            if tick == 50 {
                snapshot = Some(w.snapshot());
            }
            w.step().unwrap();
            heights.push(transform(&w, ball).position);
        }
        let _ = physics;
        (w, heights, snapshot.unwrap(), ball)
    };
    let (w, heights, snapshot, ball) = run();
    // It kept falling through both switches and came to rest on the floor.
    for t in [19usize, 21, 69, 71] {
        assert!(
            heights[t + 1].y < heights[t].y,
            "tick {t}: still falling ({} then {})",
            heights[t].y,
            heights[t + 1].y
        );
    }
    let end = heights[199];
    assert!((end.y - 0.5).abs() < 0.02, "at rest on the floor: {end}");
    // In 2D it kept its z.
    assert_eq!(heights[60].z, 0.4);
    // The same twice, and from a restore in the 2D stretch.
    let (w2, heights2, _, _) = run();
    assert_eq!(heights, heights2);
    assert_eq!(w.world_hash(), w2.world_hash());
    let (mut w3, physics3) = world(Physics::default(), 9);
    floor(&mut w3);
    spawn_body(
        &mut w3,
        at(0.0, 20.0, 0.4),
        RigidBody::dynamic(),
        Collider::sphere(0.5),
    );
    w3.restore(&snapshot).unwrap();
    for tick in 50..200u32 {
        if tick == 70 {
            let mut p = *w3.resource::<Physics>().unwrap();
            p.profile = PROFILE_3D;
            w3.insert_resource(p).unwrap();
        }
        w3.step().unwrap();
    }
    let _ = physics3;
    assert_eq!(transform(&w3, ball).position, end);
    assert_eq!(w3.world_hash(), w.world_hash());
}
