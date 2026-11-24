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
        w.insert(r, Buoyancy::default()).unwrap();
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
}
