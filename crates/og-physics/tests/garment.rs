//! R9's garments on skinned characters (og.Garment), through the world: the
//! host registers a rig and hands in the skin's pose before every step, as
//! og-runtime's garment system does.
//!
//! Ways this could fail, each caught below: the points are not world
//! state, so a restore mid-swing goes on differently; two runs differ; a
//! garment with `radius` passes through a solid; a garment given no pose
//! moves anyway; the 2D profile steps it.

use crate::support;

use og_core::glam::Vec3;
use og_ecs::{Entity, World};
use og_physics::*;
use support::*;

/// A cape of 7 × 8 points 0.1 apart in the xy plane at z -0.2, from y 1
/// down, its normals away from the body (-z), every point free.
fn cape() -> (GarmentRig, Vec<Vec3>) {
    let (columns, rows) = (7u32, 8u32);
    let mut rest = Vec::new();
    for r in 0..rows {
        for c in 0..columns {
            rest.push(Vec3::new(c as f32 * 0.1, 1.0 - r as f32 * 0.1, -0.2));
        }
    }
    let mut triangles = Vec::new();
    for r in 0..rows - 1 {
        for c in 0..columns - 1 {
            let a = r * columns + c;
            triangles.push([a, a + columns, a + 1]);
            triangles.push([a + 1, a + columns, a + columns + 1]);
        }
    }
    let n = rest.len();
    (
        GarmentRig {
            rest: rest.clone(),
            triangles,
            freedom: vec![1.0; n],
        },
        rest,
    )
}

/// The skin's pose at `tick`: the cape on a body that sways along z.
fn pose(rest: &[Vec3], tick: u64, at: Vec3) -> GarmentPose {
    let sway = Vec3::new(0.0, 0.0, og_core::math::sin(tick as f32 * 0.15) * 0.6);
    GarmentPose {
        targets: rest.iter().map(|p| *p + at + sway).collect(),
        normals: vec![Vec3::NEG_Z; rest.len()],
        scale: 1.0,
    }
}

fn world(settings: Physics) -> (World, PhysicsSystem) {
    let (mut w, p) = support::world(settings, 5);
    support::enable_lazy(&mut w);
    (w, p)
}

/// Steps `ticks` ticks from the world's tick, handing in each one's pose.
fn walk(w: &mut World, p: &PhysicsSystem, e: Entity, rest: &[Vec3], at: Vec3, ticks: u64) {
    for _ in 0..ticks {
        let tick = w.time().tick;
        p.set_garment_pose(e, pose(rest, tick, at));
        w.step().unwrap();
    }
}

#[test]
fn garments_are_world_state_identical_across_runs_and_after_a_restore() {
    let (rig, rest) = cape();
    let start = || {
        let (mut w, p) = world(Physics::default());
        let e = w.spawn((Garment::default(),)).unwrap();
        p.set_garment_rig(e, 42, rig.clone()).unwrap();
        (w, p, e)
    };
    let (mut w1, p1, e) = start();
    walk(&mut w1, &p1, e, &rest, Vec3::ZERO, 40);
    let mid = w1.snapshot();
    walk(&mut w1, &p1, e, &rest, Vec3::ZERO, 60);
    let end = p1.garment_points(e).unwrap();
    // It moved off the skin: the hem swings.
    let off = end
        .points
        .iter()
        .zip(&end.targets)
        .fold(0.0f32, |m, (p, t)| m.max((*p - *t).length()));
    assert!(off > 0.02, "the cape never left the skin ({off})");
    let (mut w2, p2, e2) = start();
    walk(&mut w2, &p2, e2, &rest, Vec3::ZERO, 100);
    assert_eq!(p2.garment_points(e2).unwrap(), end, "two runs differ");
    assert_eq!(w1.world_hash(), w2.world_hash());
    // Restored into a fresh world: the host registers the rig again (an
    // asset), the points come from the save.
    let (mut w3, p3, _) = start();
    w3.restore(&mid).unwrap();
    p3.set_garment_rig(e, 42, rig.clone()).unwrap();
    walk(&mut w3, &p3, e, &rest, Vec3::ZERO, 60);
    assert_eq!(p3.garment_points(e).unwrap(), end, "a restored run differs");
    assert_eq!(w3.world_hash(), w1.world_hash());
    // A tick with no pose keeps the points.
    w1.step().unwrap();
    assert_eq!(p1.garment_points(e).unwrap().points, end.points);
}

#[test]
fn a_garment_with_a_radius_drapes_over_a_solid_instead_of_through_it() {
    let (rig, rest) = cape();
    let (mut w, p) = world(Physics::default());
    // A box just behind the cape's lower half (away from the body).
    let wall = Vec3::new(0.3, 0.45, -0.32);
    spawn_static(
        &mut w,
        at(wall.x, wall.y, wall.z),
        Collider::cuboid(Vec3::new(1.0, 0.4, 0.1)),
    );
    let e = w
        .spawn((Garment {
            radius: 0.02,
            max_distance: 0.4,
            ..Garment::default()
        },))
        .unwrap();
    p.set_garment_rig(e, 1, rig).unwrap();
    // The body backs into the box: the skin puts the cape's lower half
    // inside it (z -0.35), so the points must be pushed out of it.
    let still = GarmentPose {
        targets: rest
            .iter()
            .map(|q| *q + Vec3::new(0.0, 0.0, -0.15))
            .collect(),
        normals: vec![Vec3::NEG_Z; rest.len()],
        scale: 1.0,
    };
    for _ in 0..60 {
        p.set_garment_pose(e, still.clone());
        w.step().unwrap();
    }
    let g = p.garment_points(e).unwrap();
    assert!(
        g.targets
            .iter()
            .any(|t| (t.y - wall.y).abs() < 0.2 && (t.z - wall.z).abs() < 0.05),
        "the skin puts no point inside the box: the test proves nothing"
    );
    for (i, q) in g.points.iter().enumerate() {
        let inside = (q.x - wall.x).abs() < 0.5
            && (q.y - wall.y).abs() < 0.2
            && (q.z - wall.z).abs() < 0.05 + 0.02 - 1e-3;
        assert!(!inside, "point {i} at {q} is inside the box");
    }
}

#[test]
fn the_2d_profile_leaves_garments_alone() {
    let (rig, rest) = cape();
    let (mut w, p) = world(Physics {
        profile: PROFILE_2D,
        ..Physics::default()
    });
    let e = w.spawn((Garment::default(),)).unwrap();
    p.set_garment_rig(e, 1, rig).unwrap();
    walk(&mut w, &p, e, &rest, Vec3::ZERO, 10);
    assert_eq!(p.garment_points(e), None);
}
