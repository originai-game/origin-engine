//! G2-3: compound colliders from children, raycast-all and point
//! projection.
//!
//! Ways they could fail, and the test that catches each:
//! - a child collider stays a static collider at its own transform (the
//!   body falls through the floor as a point mass), joins the wrong body,
//!   is placed without its parents' transforms between it and the body, or
//!   does not follow the body → `child_colliders_make_one_body`
//! - a change to a child does not reach the body (a stale shape) →
//!   `child_colliders_make_one_body`
//! - the children's state is kept outside the og.physics block, or their
//!   order depends on query order: a restore mid-tumble diverges →
//!   `a_compound_restores_mid_tumble`
//! - raycast-all misses colliders behind the first, lists them in the
//!   BVH's order, ignores the cap or the filter, or names a child's body
//!   wrongly → `raycast_all_lists_every_hit_nearest_first`
//! - point projection returns a far collider, the wrong side's normal, or
//!   misreads inside and solid → `a_point_projects_onto_the_nearest_collider`

use crate::support;

use og_core::glam::{Quat, Vec3};
use og_ecs::{Entity, World};
use og_physics::*;
use support::*;

/// A dumbbell: a body with no collider of its own and two spheres of
/// radius 0.5 on child entities, at x = ±1 (the right one under an
/// intermediate child turned 90° about y, so its own offset is along +z).
fn dumbbell(w: &mut World, y: f32) -> (Entity, Entity, Entity) {
    let body = w.spawn((at(0.0, y, 0.0), RigidBody::dynamic())).unwrap();
    let left = w
        .spawn((
            at(-1.0, 0.0, 0.0),
            Collider::sphere(0.5),
            Parent {
                entity: body.to_bits(),
            },
        ))
        .unwrap();
    let mid = w
        .spawn((
            rotated(
                at(0.0, 0.0, 0.0),
                Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            ),
            Parent {
                entity: body.to_bits(),
            },
        ))
        .unwrap();
    // Turned +90° about y, local +z is world +x.
    let right = w
        .spawn((
            at(0.0, 0.0, 1.0),
            Collider::sphere(0.5),
            Parent {
                entity: mid.to_bits(),
            },
        ))
        .unwrap();
    (body, left, right)
}

fn down(x: f32) -> RaycastDesc {
    RaycastDesc {
        origin: Vec3::new(x, 10.0, 0.0),
        direction: Vec3::NEG_Y,
        ..RaycastDesc::default()
    }
}

#[test]
fn child_colliders_make_one_body() {
    let (mut w, physics) = world(Physics::default(), 1);
    floor(&mut w);
    let (body, left, right) = dumbbell(&mut w, 3.0);
    run(&mut w, 180);
    // It rests on both spheres: centre at their radius, level.
    let t = transform(&w, body);
    assert!((t.position.y - 0.5).abs() < 0.01, "{t:?}");
    assert!(t.position.x.abs() < 0.01, "{t:?}");
    let up = t.quaternion * Vec3::X;
    assert!(up.y.abs() < 0.01, "level: {up}");
    // The children are where the body carries them, and a ray finds each,
    // naming it and its body.
    let q = physics.query(&w).unwrap();
    for (x, child) in [(-1.0, left), (1.0, right)] {
        let hit = q.raycast(&down(x)).unwrap().expect("a child is hit");
        assert_eq!((hit.entity, hit.body), (child, body));
        assert!((hit.point.y - 1.0).abs() < 0.02, "{hit:?}");
    }
    // A child grows: the body is rebuilt and settles higher on it.
    let c = w.get::<Collider>(left).unwrap();
    w.insert(left, Collider { radius: 1.0, ..c }).unwrap();
    run(&mut w, 240);
    let t = transform(&w, body);
    let tilt = (t.quaternion * Vec3::X).y;
    assert!(tilt < -0.1, "the bigger left sphere lifts its side: {tilt}");
    // Each child moves with the body: the left hit is now about a radius
    // of 1 above the floor.
    let q = physics.query(&w).unwrap();
    let left_x = (t.position + t.quaternion * Vec3::new(-1.0, 0.0, 0.0)).x;
    let hit = q.raycast(&down(left_x)).unwrap().unwrap();
    assert_eq!(hit.entity, left);
    assert!((hit.point.y - 2.0).abs() < 0.05, "{hit:?}");
}

#[test]
fn a_compound_restores_mid_tumble() {
    let build = |seed| {
        let (mut w, physics) = world(Physics::default(), seed);
        floor(&mut w);
        let (body, ..) = dumbbell(&mut w, 2.0);
        let t = transform(&w, body);
        w.insert(
            body,
            Transform {
                quaternion: Quat::from_rotation_z(0.6) * Quat::from_rotation_x(0.3),
                ..t
            },
        )
        .unwrap();
        w.insert(
            body,
            Velocity {
                linear: Vec3::new(1.5, 0.0, 0.5),
                angular: Vec3::new(0.0, 2.0, 3.0),
            },
        )
        .unwrap();
        (w, physics, body)
    };
    let (mut a, _pa, body) = build(1);
    run(&mut a, 40);
    let half = a.snapshot();
    let half_hash = a.world_hash();
    run(&mut a, 80);
    let moved = transform(&a, body);
    // It tumbled: turned off its start and still on the floor.
    assert!(
        moved.position.y > 0.4 && moved.position.y < 1.6,
        "{moved:?}"
    );

    let (mut b, _pb, _) = build(9);
    b.restore(&half).unwrap();
    assert_eq!(b.world_hash(), half_hash);
    run(&mut b, 80);
    assert_eq!(b.world_hash(), a.world_hash(), "the restored run diverged");
    assert_eq!(transform(&b, body), moved);
}

#[test]
fn raycast_all_lists_every_hit_nearest_first() {
    let (mut w, physics) = world(Physics::default(), 1);
    // Spawned far to near, and two at the same distance.
    let far = spawn_static(&mut w, at(9.0, 0.0, 0.0), Collider::cuboid(Vec3::ONE));
    let tie_b = spawn_static(&mut w, at(6.0, 0.0, 0.0), Collider::cuboid(Vec3::ONE));
    let tie_a = spawn_static(&mut w, at(6.0, 0.0, 0.0), Collider::sphere(0.5));
    let near = spawn_static(
        &mut w,
        at(3.0, 0.0, 0.0),
        Collider::cuboid(Vec3::ONE).on_layers(2, u32::MAX),
    );
    let (body, left, _) = dumbbell(&mut w, 0.0);
    let t = transform(&w, body);
    w.insert(
        body,
        Transform {
            position: Vec3::new(12.0, 0.0, 0.0),
            ..t
        },
    )
    .unwrap();
    w.insert(body, RigidBody::fixed()).unwrap();
    let q = physics.query(&w).unwrap();
    let ray = RaycastDesc {
        origin: Vec3::ZERO,
        direction: Vec3::X,
        ..RaycastDesc::default()
    };
    let all = q.raycast_all(&RaycastAllDesc { ray, max_hits: 0 }).unwrap();
    let mut ties = [tie_a, tie_b];
    ties.sort_by_key(|e| e.to_bits());
    let names: Vec<Entity> = all.iter().map(|h| h.entity).collect();
    assert_eq!(names[..4], [near, ties[0], ties[1], far]);
    // The dumbbell: its left sphere first (x 10.5), then the right (12.5).
    assert_eq!(all.len(), 6, "{all:?}");
    assert_eq!((all[4].entity, all[4].body), (left, body));
    assert!((all[4].distance - 10.5).abs() < 1e-3);
    assert_eq!(all[5].body, body);
    assert!(all.windows(2).all(|p| p[0].distance <= p[1].distance));
    // The first hit is raycast's.
    let first = q.raycast(&ray).unwrap().unwrap();
    assert_eq!(first, all[0]);
    // The cap and the filter.
    let two = q.raycast_all(&RaycastAllDesc { ray, max_hits: 2 }).unwrap();
    assert_eq!(two, all[..2]);
    let masked = q
        .raycast_all(&RaycastAllDesc {
            ray: RaycastDesc {
                filter: QueryFilter {
                    mask: 1,
                    ..QueryFilter::default()
                },
                max_distance: 10.0,
                ..ray
            },
            max_hits: 0,
        })
        .unwrap();
    let masked: Vec<Entity> = masked.iter().map(|h| h.entity).collect();
    assert_eq!(
        masked,
        [ties[0], ties[1], far],
        "layer 2 and past 10 m left out"
    );
    // A zero direction is refused.
    let zero = RaycastDesc {
        direction: Vec3::ZERO,
        ..ray
    };
    assert!(
        q.raycast_all(&RaycastAllDesc {
            ray: zero,
            max_hits: 0
        })
        .is_err()
    );
}

#[test]
fn a_point_projects_onto_the_nearest_collider() {
    for profile in [3u32, 2] {
        let settings = if profile == 3 {
            Physics::default()
        } else {
            Physics::profile_2d()
        };
        let (mut w, physics) = world(settings, 1);
        let near = spawn_static(&mut w, at(3.0, 0.0, 0.0), Collider::cuboid(Vec3::ONE));
        let far = spawn_static(&mut w, at(-6.0, 0.0, 0.0), Collider::sphere(1.0));
        let q = physics.query(&w).unwrap();
        let project = |point: Vec3, solid: bool, max: f32| {
            q.project_point(&ProjectPointDesc {
                point,
                solid,
                max_distance: max,
                ..ProjectPointDesc::default()
            })
            .unwrap()
        };
        // Outside: onto the box's face at x 2.5, normal towards the point.
        let p = project(Vec3::new(0.5, 0.25, 0.0), true, 0.0).expect("{profile}D");
        assert_eq!(p.entity, near);
        assert_eq!(p.body, near);
        assert!(!p.inside);
        assert!(
            (p.point - Vec3::new(2.5, 0.25, 0.0)).length() < 1e-4,
            "{p:?}"
        );
        assert!((p.distance - 2.0).abs() < 1e-4);
        assert!((p.normal - Vec3::NEG_X).length() < 1e-4, "{p:?}");
        // Nearer the sphere: the sphere.
        let p = project(Vec3::new(-3.0, 0.0, 0.0), true, 0.0).unwrap();
        assert_eq!(p.entity, far);
        assert!((p.point.x + 5.0).abs() < 1e-4 && (p.distance - 2.0).abs() < 1e-4);
        // Inside the box, solid: itself; not solid: onto the nearest face,
        // the normal out of the box (from the face towards the inside point,
        // reversed).
        let inside = Vec3::new(3.3, 0.0, 0.0);
        let p = project(inside, true, 0.0).unwrap();
        assert!(p.inside && p.distance == 0.0 && p.point == inside, "{p:?}");
        let p = project(inside, false, 0.0).unwrap();
        assert!(p.inside, "{p:?}");
        assert!(
            (p.point.x - 3.5).abs() < 1e-4 && (p.distance - 0.2).abs() < 1e-4,
            "{p:?}"
        );
        assert!((p.normal - Vec3::X).length() < 1e-4, "{p:?}");
        // Out of reach.
        assert!(project(Vec3::new(0.5, 0.0, 0.0), true, 1.0).is_none());
    }
}
