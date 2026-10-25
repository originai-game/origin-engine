//! The determinism scenario: `WALKERS` characters on the walker rig, half
//! driven by the graph (a speed profile through idle, walk and run, an
//! additive wave masked to the arm and synced to the gait), half by players
//! switching clips (the mirrored walk among them) with cross-fades; all
//! with planar root motion, blendshapes, two-bone feet and a FABRIK arm,
//! hinged knees and a cone on the shoulder (joint limits) and a head
//! looking at a target that circles it, and both feet locked while planted.
//! A driver system (before `og.anim`) derives every input from the tick and
//! the walker's index, so the world is a function of its seed and tick.
#![allow(dead_code)]

use super::*;
use og_anim::*;
use og_core::glam::Vec3;
use og_core::{Reflect, math};
use og_ecs::{Entity, QueryDesc, SystemDesc, World, WorldDesc};

pub const WALKERS: u32 = 20;

/// Which walker an entity is (on skeletons and on their chains).
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "test.Walker")]
pub struct Walker {
    pub index: u32,
    /// 0 skeleton, 1 left foot, 2 right foot, 3 left hand, 4 the head's
    /// look-at, 5 a joint limit, 6 a foot lock.
    pub role: u32,
}

/// The graph's speed parameter of walker `i` at `tick`.
pub fn speed_of(i: u32, tick: u64) -> f32 {
    let t = tick as f32 / 60.0 + i as f32 * 0.37;
    let s = 2.0 + 2.2 * math::sin(t * 0.4) + 0.8 * math::sin(t * 1.3);
    s.max(0.0)
}

pub fn build(seed: u32) -> (World, AnimationSystem) {
    let mut world = World::new(&WorldDesc {
        tick_rate: 60,
        seed,
    })
    .unwrap();
    world.register::<Walker>().unwrap();
    // The driver is registered before og.anim, so it runs first each tick.
    world.register::<Transform>().unwrap();
    world.register::<AnimationGraph>().unwrap();
    world.register::<AnimationPlayer>().unwrap();
    world.register::<IkChain>().unwrap();
    world.register::<LookAt>().unwrap();
    world.register::<Blendshapes>().unwrap();
    world.register::<Skeleton>().unwrap();
    world
        .add_system(
            SystemDesc::new("test.drive")
                .read::<Walker>()
                .read::<Transform>()
                .write::<AnimationGraph>()
                .write::<AnimationPlayer>()
                .write::<Blendshapes>()
                .write::<IkChain>()
                .write::<LookAt>()
                .query(QueryDesc::new().read::<Walker>()),
            |ctx| {
                let tick = ctx.time().tick;
                let mut es: Vec<Entity> = Vec::new();
                for a in ctx.query(0)? {
                    es.extend_from_slice(a.entities());
                }
                for e in es {
                    let w: Walker = ctx.get(e)?;
                    let i = w.index;
                    if w.role == 0 {
                        if let Ok(mut g) = ctx.get::<AnimationGraph>(e) {
                            g.p0 = speed_of(i, tick);
                            g.p1 = 0.5 + 0.5 * math::sin(tick as f32 / 47.0 + i as f32);
                            ctx.set(e, g)?;
                        }
                        if let Ok(mut p) = ctx.get::<AnimationPlayer>(e) {
                            // Switch clips every 97 + i ticks.
                            let period = 97 + i as u64;
                            if tick % period == 0 {
                                p.clip = [WALK, RUN, IDLE, WALK_MIRRORED]
                                    [((tick / period + i as u64) % 4) as usize];
                                p.fade = 0.3;
                            }
                            ctx.set(e, p)?;
                        }
                        if let Ok(mut b) = ctx.get::<Blendshapes>(e)
                            && b.source == BLENDSHAPES_FROM_COMPONENT
                        {
                            b.w0 = 0.5 + 0.5 * math::sin(tick as f32 / 13.0);
                            b.w1 = (tick % 60) as f32 / 60.0;
                            ctx.set(e, b)?;
                        }
                    } else if w.role == 4 {
                        let mut l: LookAt = ctx.get(e)?;
                        let sk: Transform = ctx.get(Entity::from_bits(l.skeleton))?;
                        // Circles the walker: in front, beside, behind.
                        let a = tick as f32 / 90.0 + i as f32;
                        let local = Vec3::new(2.0 * math::sin(a), 1.4, 2.0 * math::cos(a));
                        l.target = sk.position + sk.quaternion * (sk.scale * local);
                        ctx.set(e, l)?;
                    } else if w.role < 5 {
                        let mut c: IkChain = ctx.get(e)?;
                        let sk: Transform = ctx.get(Entity::from_bits(c.skeleton))?;
                        let local = match w.role {
                            1 => Vec3::new(0.18, 0.25 + 0.05 * math::sin(tick as f32 / 9.0), 0.1),
                            2 => {
                                Vec3::new(-0.18, 0.25, -0.1 + 0.05 * math::cos(tick as f32 / 11.0))
                            }
                            _ => Vec3::new(
                                0.6,
                                1.6 + 0.2 * math::sin(tick as f32 / 17.0 + i as f32),
                                0.2,
                            ),
                        };
                        c.target = sk.position + sk.quaternion * (sk.scale * local);
                        c.pole = sk.position + sk.quaternion * Vec3::new(0.0, 0.5, 2.0);
                        ctx.set(e, c)?;
                    }
                }
                Ok(())
            },
        )
        .unwrap();
    let anim = install(&mut world).unwrap();
    register(&anim);
    for i in 0..WALKERS {
        let at = Transform {
            position: Vec3::new((i % 5) as f32 * 3.0, 0.0, (i / 5) as f32 * 3.0),
            quaternion: math::quat_from_axis_angle(Vec3::Y, i as f32 * 0.3),
            scale: Vec3::ONE,
        };
        let skel = Skeleton {
            asset: SKELETON,
            root_motion: ROOT_MOTION_PLANAR,
            ..Skeleton::default()
        };
        let e = if i % 2 == 0 {
            world
                .spawn((
                    at,
                    skel,
                    AnimationGraph::new(GRAPH),
                    Blendshapes::default(),
                    Walker { index: i, role: 0 },
                ))
                .unwrap()
        } else {
            world
                .spawn((
                    at,
                    skel,
                    AnimationPlayer::new(WALK),
                    Blendshapes {
                        source: BLENDSHAPES_FROM_CLIPS,
                        ..Blendshapes::default()
                    },
                    Walker { index: i, role: 0 },
                ))
                .unwrap()
        };
        for (role, tip, solver, bones) in [
            (1, J_FOOT_L, IK_TWO_BONE, 2),
            (2, J_FOOT_R, IK_TWO_BONE, 2),
            (3, J_HAND_L, IK_FABRIK, 2),
        ] {
            world
                .spawn((
                    IkChain {
                        skeleton: e.to_bits(),
                        tip,
                        bones,
                        solver,
                        ..IkChain::default()
                    },
                    Walker { index: i, role },
                ))
                .unwrap();
        }
        world
            .spawn((
                LookAt {
                    skeleton: e.to_bits(),
                    joint: J_HEAD,
                    bones: 2,
                    max_yaw: 1.2,
                    max_pitch: 0.5,
                    ..LookAt::default()
                },
                Walker { index: i, role: 4 },
            ))
            .unwrap();
        for foot in [J_FOOT_L, J_FOOT_R] {
            world
                .spawn((
                    FootLock {
                        skeleton: e.to_bits(),
                        foot,
                        ..FootLock::default()
                    },
                    Walker { index: i, role: 6 },
                ))
                .unwrap();
        }
        // Knees hinge forwards only (with a little play); the shoulder swings
        // within a cone.
        for limit in [
            JointLimit {
                joint: J_SHIN_L,
                axis: Vec3::X,
                swing: 0.15,
                twist_min: -0.05,
                twist_max: 2.6,
                ..JointLimit::default()
            },
            JointLimit {
                joint: J_SHIN_R,
                axis: Vec3::X,
                swing: 0.15,
                twist_min: -0.05,
                twist_max: 2.6,
                ..JointLimit::default()
            },
            JointLimit {
                joint: J_ARM_L,
                axis: Vec3::X,
                swing: 1.4,
                ..JointLimit::default()
            },
        ] {
            world
                .spawn((
                    JointLimit {
                        skeleton: e.to_bits(),
                        ..limit
                    },
                    Walker { index: i, role: 5 },
                ))
                .unwrap();
        }
    }
    (world, anim)
}
