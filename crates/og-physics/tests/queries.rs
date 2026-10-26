//! Scene queries: raycast, shape cast and overlap over the colliders the
//! world holds.
//!
//! Ways this could fail: the first-inserted collider is returned instead of
//! the nearest; the filter's mask, exclusion or sensor flag is ignored;
//! distances or normals are in the wrong space; the query sees stale
//! positions instead of the written-back ones; a zero direction panics.

use crate::support;

use og_core::glam::{Quat, Vec3};
use og_physics::*;
use support::*;

fn ray(origin: Vec3, direction: Vec3) -> RaycastDesc {
    RaycastDesc {
        origin,
        direction,
        ..RaycastDesc::default()
    }
}

#[test]
fn a_raycast_hits_the_nearest_body_and_honours_the_filter() {
    let (mut w, physics) = world(Physics::default(), 1);
    // Spawned far to near, so the nearest is the last inserted.
    let far = spawn_body(
        &mut w,
        at(9.0, 0.0, 0.0),
        RigidBody::fixed(),
        Collider::cuboid(Vec3::ONE),
    );
    let mid = spawn_body(
        &mut w,
        at(6.0, 0.0, 0.0),
        RigidBody::fixed(),
        Collider::cuboid(Vec3::ONE),
    );
    let near = spawn_static(
        &mut w,
        at(3.0, 0.0, 0.0),
        Collider::cuboid(Vec3::ONE).on_layers(2, u32::MAX),
    );
    let sensor = spawn_static(
        &mut w,
        at(1.5, 0.0, 0.0),
        Collider::sphere(0.25).as_sensor(),
    );

    let q = physics.query(&w).unwrap();
    let hit = q
        .raycast(&ray(Vec3::ZERO, Vec3::new(2.0, 0.0, 0.0)))
        .unwrap()
        .unwrap();
    assert_eq!(hit.entity, near);
    assert!((hit.distance - 2.5).abs() < 1e-4, "{hit:?}");
    assert!(
        (hit.point - Vec3::new(2.5, 0.0, 0.0)).length() < 1e-4,
        "{hit:?}"
    );
    assert!(
        (hit.normal - Vec3::new(-1.0, 0.0, 0.0)).length() < 1e-4,
        "{hit:?}"
    );

    let only_layer_1 = RaycastDesc {
        filter: QueryFilter {
            mask: 1,
            ..QueryFilter::default()
        },
        ..ray(Vec3::ZERO, Vec3::X)
    };
    assert_eq!(q.raycast(&only_layer_1).unwrap().unwrap().entity, mid);
    let skip_near = RaycastDesc {
        filter: QueryFilter {
            exclude: Some(near),
            ..QueryFilter::default()
        },
        ..ray(Vec3::ZERO, Vec3::X)
    };
    assert_eq!(q.raycast(&skip_near).unwrap().unwrap().entity, mid);
    let with_sensors = RaycastDesc {
        filter: QueryFilter {
            include_sensors: true,
            ..QueryFilter::default()
        },
        ..ray(Vec3::ZERO, Vec3::X)
    };
    assert_eq!(q.raycast(&with_sensors).unwrap().unwrap().entity, sensor);
    let short = RaycastDesc {
        max_distance: 2.0,
        ..ray(Vec3::ZERO, Vec3::X)
    };
    assert_eq!(q.raycast(&short).unwrap(), None);
    assert_eq!(q.raycast(&ray(Vec3::ZERO, -Vec3::X)).unwrap(), None);
    // From the far side the far box is the nearest.
    assert_eq!(
        raycast(&w, &physics, &ray(Vec3::new(20.0, 0.0, 0.0), -Vec3::X))
            .unwrap()
            .unwrap()
            .entity,
        far
    );
    assert!(q.raycast(&ray(Vec3::ZERO, Vec3::ZERO)).is_err());
}

#[test]
fn queries_see_where_the_step_moved_bodies() {
    let (mut w, physics) = world(Physics::default(), 2);
    floor(&mut w);
    let ball = spawn_body(
        &mut w,
        at(0.0, 10.0, 0.0),
        RigidBody::dynamic(),
        Collider::sphere(0.5),
    );
    let down = ray(Vec3::new(0.0, 20.0, 0.0), -Vec3::Y);
    let before = raycast(&w, &physics, &down).unwrap().unwrap();
    assert_eq!(before.entity, ball);
    assert!((before.distance - 9.5).abs() < 1e-4);
    run(&mut w, 30);
    let after = raycast(&w, &physics, &down).unwrap().unwrap();
    let y = transform(&w, ball).position.y;
    assert!(y < 9.0);
    assert!(
        (after.distance - (20.0 - y - 0.5)).abs() < 1e-3,
        "{after:?} at y {y}"
    );
}

#[test]
fn a_shape_cast_stops_at_the_first_surface_and_overlap_lists_what_it_touches() {
    let (mut w, physics) = world(Physics::default(), 3);
    let ground = floor(&mut w);
    let a = spawn_body(
        &mut w,
        at(-1.0, 0.5, 0.0),
        RigidBody::fixed(),
        Collider::cuboid(Vec3::ONE),
    );
    let b = spawn_body(
        &mut w,
        at(1.0, 0.5, 0.0),
        RigidBody::fixed(),
        Collider::sphere(0.5),
    );
    spawn_body(
        &mut w,
        at(8.0, 0.5, 0.0),
        RigidBody::fixed(),
        Collider::sphere(0.5),
    );
    let q = physics.query(&w).unwrap();

    let sphere = ShapeDesc {
        shape: SHAPE_SPHERE,
        radius: 0.5,
        ..ShapeDesc::default()
    };
    let hit = q
        .shape_cast(&ShapeCastDesc {
            shape: sphere,
            position: Vec3::new(4.0, 5.0, 0.0),
            direction: -Vec3::Y,
            ..ShapeCastDesc::default()
        })
        .unwrap()
        .unwrap();
    assert_eq!(hit.entity, ground);
    assert!((hit.distance - 4.5).abs() < 1e-3, "{hit:?}");
    assert!(
        (hit.point - Vec3::new(4.0, 0.0, 0.0)).length() < 1e-3,
        "{hit:?}"
    );
    assert!((hit.normal - Vec3::Y).length() < 1e-3, "{hit:?}");

    // A 3 × 0.5 × 1 box above the floor spanning both bodies.
    let slab = OverlapDesc {
        shape: ShapeDesc {
            shape: SHAPE_BOX,
            size: Vec3::new(3.0, 0.5, 1.0),
            ..ShapeDesc::default()
        },
        position: Vec3::new(0.0, 0.5, 0.0),
        rotation: Quat::IDENTITY,
        filter: QueryFilter::default(),
    };
    let mut want = vec![a, b];
    want.sort_by_key(|e| e.to_bits());
    assert_eq!(q.overlap(&slab).unwrap(), want);
    let lower = OverlapDesc {
        position: Vec3::new(0.0, 0.1, 0.0),
        ..slab
    };
    let mut want = vec![ground, a, b];
    want.sort_by_key(|e| e.to_bits());
    assert_eq!(overlap(&w, &physics, &lower).unwrap(), want);
}
