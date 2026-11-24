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

use crate::support;

use og_core::glam::{Vec2, Vec3};
use og_ecs::World;
use og_physics::*;
use std::sync::Arc;
use support::*;

#[test]
fn cooked_colliders_keep_scale_and_refuse_stale_geometry() {
    for kind in [BakeKind::Trimesh, BakeKind::Convex, BakeKind::Vhacd] {
        let (mut w, p) = world(
            Physics {
                gravity: Vec3::ZERO,
                ..Default::default()
            },
            13,
        );
        let mesh = support::pile::octahedron(1.0);
        let baked = bake_collider(&mesh, kind).unwrap();
        p.set_mesh(42, mesh).unwrap();
        p.set_baked_collider(42, &baked).unwrap();
        let mut tr = at(0.0, 0.0, 0.0);
        tr.scale = Vec3::new(-2.0, 3.0, 4.0);
        spawn_static(
            &mut w,
            tr,
            Collider {
                shape: if kind == BakeKind::Trimesh {
                    SHAPE_TRIMESH
                } else {
                    SHAPE_CONVEX
                },
                mesh: 42,
                ..Default::default()
            },
        );
        w.step().unwrap();
        let ray = RaycastDesc {
            origin: Vec3::new(0.5, 8.0, 0.0),
            direction: -Vec3::Y,
            max_distance: 20.0,
            ..Default::default()
        };
        let top = p.query(&w).unwrap().raycast(&ray).unwrap().unwrap().point.y;
        assert!(
            (top - 2.25).abs() < 0.1,
            "{kind:?} scales the hull, including a reflected axis: {top}"
        );
        let saved = w.snapshot();
        run(&mut w, 3);
        let end = w.snapshot();
        w.restore(&saved).unwrap();
        run(&mut w, 3);
        assert_eq!(w.snapshot(), end);
        // The handle stays the same, but the bake's content hash is stale.
        p.set_mesh(42, support::pile::octahedron(2.0)).unwrap();
        w.step().unwrap();
        let top = p.query(&w).unwrap().raycast(&ray).unwrap().unwrap().point.y;
        assert!(
            (top - 5.25).abs() < 0.001,
            "{kind:?} must not reuse a bake for other content: {top}"
        );
    }
}

#[test]
fn queries_share_the_simulation_across_restore() {
    for profile in [PROFILE_2D, PROFILE_3D] {
        let mut w = World::new(&og_ecs::WorldDesc {
            tick_rate: 60,
            seed: 1,
        })
        .unwrap();
        w.register::<Transform>().unwrap();
        let p = PhysicsSystem::register(
            &mut w,
            Physics {
                profile,
                gravity: Vec3::ZERO,
                ..Physics::default()
            },
        )
        .unwrap();
        let body = spawn_body(
            &mut w,
            at(0.0, 0.0, 0.0),
            RigidBody::kinematic(),
            Collider::sphere(0.5),
        );
        let (desc, run) = p.query_system();
        w.add_system(desc, run).unwrap();
        let pre = p.clone();
        w.add_system(
            og_ecs::SystemDesc::new("game")
                .write::<Transform>()
                .write::<RigidBody>()
                .write::<Collider>(),
            move |ctx| {
                let x = ctx.time().tick as f32 * 2.0;
                let q = pre.tick_query()?;
                assert_eq!(
                    q.overlap_point(&PointOverlapDesc {
                        point: Vec3::new(x, 0.0, 0.0),
                        ..Default::default()
                    })?,
                    vec![body]
                );
                ctx.set(body, at(x + 2.0, 0.0, 0.0))?;
                assert!(
                    q.overlap_point(&PointOverlapDesc {
                        point: Vec3::new(x + 2.0, 0.0, 0.0),
                        ..Default::default()
                    })?
                    .is_empty()
                );
                let spawned_at = Vec3::new(50.0 + x, 0.0, 0.0);
                ctx.spawn((
                    Transform::at(spawned_at),
                    RigidBody::fixed(),
                    Collider::sphere(0.25),
                ))?;
                assert!(
                    q.overlap_point(&PointOverlapDesc {
                        point: spawned_at,
                        ..Default::default()
                    })?
                    .is_empty(),
                    "a spawn is not visible before physics"
                );
                Ok(())
            },
        )
        .unwrap();
        p.schedule(&mut w).unwrap();
        let post = p.clone();
        w.add_system(og_ecs::SystemDesc::new("presentation"), move |ctx| {
            let x = (ctx.time().tick + 1) as f32 * 2.0;
            assert_eq!(
                post.tick_query()?.overlap_point(&PointOverlapDesc {
                    point: Vec3::new(x, 0.0, 0.0),
                    ..Default::default()
                })?,
                vec![body]
            );
            // A direct Transform write teleports this non-controller body;
            // querying must not invent a velocity for that teleport.
            assert_eq!(post.velocity_at_point(body, Vec3::ZERO)?.unwrap().x, 0.0);
            assert_eq!(
                post.tick_query()?
                    .overlap_point(&PointOverlapDesc {
                        point: Vec3::new(48.0 + x, 0.0, 0.0),
                        ..Default::default()
                    })?
                    .len(),
                1,
                "the same spawn is visible after physics"
            );
            Ok(())
        })
        .unwrap();
        // Initialize before any step, then keep this handle across both a
        // step and a restore: it must not retain a shadow collider world.
        let q = p.query(&w).unwrap();
        assert_eq!(
            q.overlap_point(&PointOverlapDesc::default()).unwrap(),
            vec![body]
        );
        w.step().unwrap();
        let saved = w.snapshot();
        w.step().unwrap();
        assert_eq!(
            q.overlap_point(&PointOverlapDesc {
                point: Vec3::new(4.0, 0.0, 0.0),
                ..Default::default()
            })
            .unwrap(),
            vec![body]
        );
        w.restore(&saved).unwrap();
        assert_eq!(
            q.overlap_point(&PointOverlapDesc {
                point: Vec3::new(2.0, 0.0, 0.0),
                ..Default::default()
            })
            .unwrap(),
            vec![body]
        );
        assert_eq!(
            w.snapshot(),
            saved,
            "querying a restore must not resync or mutate it"
        );
        w.step().unwrap();
    }
}

const GROUND: u64 = 7;
const HULL: u64 = 8;
const TERRAIN: u64 = 9;

/// A rolling 120 m square of 60 × 60 cells: 7,200 triangles.
// Test reference values: libm checks the result, never the tick.
#[allow(clippy::disallowed_methods)]
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
            holes: None,
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
    // The format byte is a hard boundary, including missing/unknown formats.
    let mut edit = og_ecs::SnapshotEdit::from_bytes(&saved).unwrap();
    let block = edit.block("og.physics").unwrap().to_vec();
    assert_eq!(block[0], 2);
    for bytes in [
        Vec::new(),
        [vec![1], block[1..].to_vec()].concat(),
        [vec![3], block[1..].to_vec()].concat(),
    ] {
        edit.set_block("og.physics", bytes).unwrap();
        let error = w.restore(&edit.to_bytes()).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("incompatible og.physics state format"),
            "{error}"
        );
        assert_eq!(
            w.world_hash(),
            hash,
            "a refused format leaves the world intact"
        );
    }
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

/// A 3 × 3 block of unit squares (two triangles each) without its centre,
/// its corners on whole numbers from (0, 0) to (3, 3): a ring with a hole.
fn ring() -> MeshDesc {
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    for y in 0..3 {
        for x in 0..3 {
            if (x, y) == (1, 1) {
                continue;
            }
            let a = positions.len() as u32;
            for (dx, dy) in [(0, 0), (1, 0), (1, 1), (0, 1)] {
                positions.push(Vec3::new((x + dx) as f32, (y + dy) as f32, 0.0));
            }
            indices.push([a, a + 1, a + 2]);
            indices.push([a, a + 2, a + 3]);
        }
    }
    MeshDesc { positions, indices }
}

#[test]
fn a_generated_2d_outline_collides_and_restores_without_its_mesh() {
    // G2-6: a tilemap's merged solid region is a generated mesh, a 2D
    // triangle mesh colliding as its outline, its shape saved with the
    // world. On the old code a 2D triangle mesh is refused; a restore that
    // still needed the mesh would be refused in the fresh engine below.
    let mesh = GENERATED_MESH | 5;
    let (mut w, p) = world(Physics::profile_2d(), 1);
    p.set_mesh(mesh, ring()).unwrap();
    spawn_static(&mut w, at(0.0, 0.0, 0.0), Collider::trimesh(mesh));
    // One ball dropped on the ring's top edge (y = 3), one in its hole.
    let top = spawn_body(
        &mut w,
        at(0.5, 5.0, 0.0),
        RigidBody::dynamic(),
        Collider::sphere(0.3),
    );
    let held = spawn_body(
        &mut w,
        at(1.5, 1.5, 0.0),
        RigidBody::dynamic(),
        Collider::sphere(0.3),
    );
    for _ in 0..60 {
        w.step().unwrap();
    }
    let saved = w.snapshot();
    let hash = w.world_hash();
    for _ in 0..120 {
        w.step().unwrap();
    }
    let rest = transform(&w, top).position;
    assert!(
        (rest.y - 3.3).abs() < 0.02 && (rest.x - 0.5).abs() < 0.05,
        "{rest}"
    );
    let h = transform(&w, held).position;
    assert!(
        (h.y - 1.3).abs() < 0.02 && h.x > 1.0 && h.x < 2.0,
        "the hole holds it: {h}"
    );

    // A fresh engine with no mesh registered restores and runs the same.
    let (mut r, _) = world(Physics::profile_2d(), 2);
    r.restore(&saved).unwrap();
    assert_eq!(r.world_hash(), hash);
    for _ in 0..120 {
        r.step().unwrap();
    }
    assert_eq!(r.world_hash(), w.world_hash());

    // In 3D (a region's prism) a generated triangle mesh is saved whole,
    // where an asset's is left for the restore to rebuild.
    let ground = GENERATED_MESH | 6;
    let (mut w, p) = world(Physics::default(), 1);
    p.set_mesh(ground, ground_mesh()).unwrap();
    spawn_static(&mut w, at(0.0, 0.0, 0.0), Collider::trimesh(ground));
    let ball = spawn_body(
        &mut w,
        at(3.0, 2.0, 1.0),
        RigidBody::dynamic(),
        Collider::sphere(0.5),
    );
    for _ in 0..40 {
        w.step().unwrap();
    }
    let saved = w.snapshot();
    let hash = w.world_hash();
    for _ in 0..120 {
        w.step().unwrap();
    }
    let (mut r, _) = world(Physics::default(), 2);
    r.restore(&saved).unwrap();
    assert_eq!(r.world_hash(), hash);
    for _ in 0..120 {
        r.step().unwrap();
    }
    assert_eq!(r.world_hash(), w.world_hash());
    let y = transform(&r, ball).position.y;
    assert!(y > 0.0 && y < 1.0, "the ball is at {y}");
}
