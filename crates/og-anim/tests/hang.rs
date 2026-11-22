//! og.Hang: the skeleton entity's Transform is placed so the grip joint
//! sits at the rope point and the body's +Y runs along the rope, its front
//! towards `facing`; its limbs' chains reach for the rope at their
//! distances; weight 0 leaves the Transform alone.
//!
//! Failure modes covered: the grip placed by its rest position or the
//! entity's origin instead of the posed joint (the hand misses the point),
//! the rope direction reversed (the body hangs head down), `facing` not
//! made orthogonal to the rope, a limb's distance measured the wrong way
//! along the rope, the hang applied after IK (the chain solves against
//! the old body and misses).

use crate::support;

use og_anim::*;
use og_core::glam::Vec3;
use og_ecs::{Entity, World, WorldDesc};
use support::*;

const HAND_NODE: u32 = 12;
const HIPS_NODE: u32 = 1;
const HEAD_NODE: u32 = 3;

fn world() -> (World, AnimationSystem, Entity) {
    let mut w = World::new(&WorldDesc {
        tick_rate: 60,
        seed: 1,
    })
    .unwrap();
    let anim = install(&mut w).unwrap();
    register(&anim);
    let e = w
        .spawn((
            Transform::at(Vec3::new(-4.0, 0.0, 1.0)),
            Skeleton {
                asset: SKELETON,
                ..Skeleton::default()
            },
        ))
        .unwrap();
    (w, anim, e)
}

fn world_of(w: &World, anim: &AnimationSystem, e: Entity, node: u32) -> Vec3 {
    let t: Transform = w.get(e).unwrap();
    let local = node_position(&anim.pose(e).unwrap(), node);
    t.position + t.quaternion * (t.scale * local)
}

#[test]
fn a_hang_holds_the_grip_on_the_rope_and_lays_the_body_along_it() {
    let (mut w, anim, e) = world();
    let (anchor, point) = (Vec3::new(0.0, 10.0, 0.0), Vec3::new(3.0, 6.0, 0.0));
    let along = (anchor - point).normalize();
    let foot = w
        .spawn((IkChain {
            skeleton: e.to_bits(),
            tip: J_FOOT_L,
            ..IkChain::default()
        },))
        .unwrap();
    let h = w
        .spawn((Hang {
            skeleton: e.to_bits(),
            grip: J_HAND_L,
            point,
            anchor,
            facing: Vec3::new(0.0, -1.0, 0.0),
            limbs: og_core::List::from_slice(&[foot.to_bits()]).unwrap(),
            reach: og_core::List::from_slice(&[-1.0]).unwrap(),
            ..Hang::default()
        },))
        .unwrap();
    w.step().unwrap();
    let hang: Hang = w.get(h).unwrap();
    assert!(hang.held);
    let hand = world_of(&w, &anim, e, HAND_NODE);
    assert!(
        (hand - point).length() < 1e-4,
        "the hand is on the rope: {hand}"
    );
    let body = (world_of(&w, &anim, e, HEAD_NODE) - world_of(&w, &anim, e, HIPS_NODE)).normalize();
    assert!(
        body.dot(along) > 0.9999,
        "hips to head run up the rope: {body} vs {along}"
    );
    let t: Transform = w.get(e).unwrap();
    let front = t.quaternion * Vec3::Z;
    assert!(
        front.dot(along).abs() < 1e-4 && front.y < -0.5,
        "chest to the ground: {front}"
    );
    // The free foot reaches the rope a metre below the grip.
    let chain: IkChain = w.get(foot).unwrap();
    let target = point - along;
    assert!((chain.target - target).length() < 1e-5, "{}", chain.target);
    assert!(
        chain.reached,
        "the leg solved against the hung body: error {}",
        chain.error
    );
}

#[test]
fn lay_zero_hangs_upright_and_weight_zero_keeps_the_transform() {
    let (mut w, anim, e) = world();
    let point = Vec3::new(3.0, 6.0, 0.0);
    let h = w
        .spawn((Hang {
            skeleton: e.to_bits(),
            grip: J_HAND_L,
            point,
            anchor: Vec3::new(0.0, 10.0, 0.0),
            lay: 0.0,
            ..Hang::default()
        },))
        .unwrap();
    w.step().unwrap();
    let t: Transform = w.get(e).unwrap();
    assert!((t.quaternion * Vec3::Y - Vec3::Y).length() < 1e-5);
    assert!((world_of(&w, &anim, e, HAND_NODE) - point).length() < 1e-4);
    let before: Transform = w.get(e).unwrap();
    let moved = Hang {
        weight: 0.0,
        point: Vec3::new(9.0, 9.0, 9.0),
        ..w.get::<Hang>(h).unwrap()
    };
    w.insert(h, moved).unwrap();
    w.step().unwrap();
    let after: Transform = w.get(e).unwrap();
    assert_eq!(before, after);
    assert!(!w.get::<Hang>(h).unwrap().held);
}
