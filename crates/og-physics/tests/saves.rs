//! What a snapshot carries of physics: the simulation without the geometry
//! of its mesh, convex-hull and height-field colliders, which are assets
//! (Meridian Loop's FINDINGS #11: a circuit's two triangle-mesh colliders
//! were 384 KB of a 400 KB save).
//!
//! Ways this could fail, each caught below: the mesh's triangles are still
//! in the snapshot; a restore does not rebuild the collider (the ball
//! falls through the ground) or rebuilds another (the hash differs, or the
//! run diverges); a restore without the mesh registered half-applies
//! instead of being refused with the fix; a convex hull on a dynamic body
//! or a height field is left out of either.

mod support;

use og_core::glam::{Vec2, Vec3};
use og_ecs::World;
use og_physics::*;
use std::sync::Arc;
use support::*;

const GROUND: u64 = 7;
const HULL: u64 = 8;
const TERRAIN: u64 = 9;

/// A rolling 120 m square of 60 × 60 cells: 7,200 triangles.
fn ground_mesh() -> MeshDesc {
    let n = 61;
    let mut positions = Vec::new();
    for j in 0..n {
        for i in 0..n {
            let (x, z) = (i as f32 * 2.0 - 60.0, j as f32 * 2.0 - 60.0);
            positions.push(Vec3::new(x, 0.3 * (x * 0.2).sin() * (z * 0.15).cos(), z));
        }
    }
    let mut indices = Vec::new();
    for j in 0..n - 1 {
        for i in 0..n - 1 {
            let a = (j * n + i) as u32;
            let (b, c, d) = (a + 1, a + n as u32, a + n as u32 + 1);
            indices.push([a, c, b]);
            indices.push([b, c, d]);
        }
    }
    MeshDesc { positions, indices }
}

fn register(p: &PhysicsSystem) {
    p.set_mesh(GROUND, ground_mesh()).unwrap();
    let corners = (0..8)
        .map(|i| Vec3::new((i & 1) as f32, ((i >> 1) & 1) as f32, (i >> 2) as f32) - 0.5)
        .collect();
    p.set_mesh(
        HULL,
        MeshDesc {
            positions: corners,
            indices: Vec::new(),
        },
    )
    .unwrap();
    let heights: Arc<[f32]> = (0..16 * 16).map(|i| (i % 7) as f32 * 0.1).collect();
    p.set_heightfield(
        TERRAIN,
        HeightfieldDesc {
            columns: 16,
            rows: 16,
            size: Vec2::new(30.0, 30.0),
            heights,
        },
    )
    .unwrap();
}

/// The scene; answers the ball on the triangle mesh.
fn scene(w: &mut World) -> og_ecs::Entity {
    spawn_static(w, at(0.0, 0.0, 0.0), Collider::trimesh(GROUND));
    spawn_static(
        w,
        at(200.0, 0.0, 0.0),
        Collider {
            shape: SHAPE_HEIGHTFIELD,
            mesh: TERRAIN,
            ..Collider::default()
        },
    );
    let ball = spawn_body(
        w,
        at(3.0, 2.0, 1.0),
        RigidBody::dynamic(),
        Collider::sphere(0.5),
    );
    w.spawn((
        at(-4.0, 3.0, 2.0),
        RigidBody::dynamic(),
        Collider::convex(HULL),
        Velocity {
            linear: Vec3::new(2.0, 0.0, 1.0),
            angular: Vec3::new(0.0, 3.0, 0.0),
        },
    ))
    .unwrap();
    spawn_body(
        w,
        at(200.0, 3.0, 0.0),
        RigidBody::dynamic(),
        Collider::sphere(0.4),
    );
    ball
}

#[test]
fn a_snapshot_carries_mesh_colliders_by_their_asset_and_restores_to_the_same_world() {
    let (mut w, p) = world(Physics::default(), 1);
    register(&p);
    let ball = scene(&mut w);
    for _ in 0..40 {
        w.step().unwrap();
    }
    let saved = w.snapshot();
    let hash = w.world_hash();
    // The triangles alone are 7,200 × 12 bytes of indices and 3,721 × 12
    // of positions before their BVH: the save holds none of them.
    assert!(
        saved.len() < 32 * 1024,
        "the snapshot is {} bytes",
        saved.len()
    );
    for _ in 0..120 {
        w.step().unwrap();
    }

    // A fresh engine registers its assets, then restores.
    let (mut r, rp) = world(Physics::default(), 2);
    register(&rp);
    r.restore(&saved).unwrap();
    assert_eq!(r.world_hash(), hash);
    assert_eq!(r.snapshot(), saved, "a restored world saves the same bytes");
    for _ in 0..120 {
        r.step().unwrap();
    }
    assert_eq!(r.world_hash(), w.world_hash());
    // The ball still rolls on the ground (it did not fall through).
    let y = transform(&r, ball).position.y;
    assert!(y > 0.0 && y < 1.0, "the ball is at {y}");

    // Without the mesh registered the save is refused, naming the fix,
    // and the world is left as it was.
    let (mut bare, _) = world(Physics::default(), 3);
    let before = bare.world_hash();
    let refused = bare.restore(&saved).unwrap_err();
    assert!(
        refused
            .to_string()
            .contains("register them before restoring")
            && refused.to_string().contains("og.Collider.mesh"),
        "{refused}"
    );
    assert_eq!(bare.world_hash(), before);
}
