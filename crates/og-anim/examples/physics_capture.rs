//! Asset-free WS-5 visual review, not a reference or a quality test.
//! Build: cargo build -p og-anim --example physics_capture
//! NVIDIA L4: OG_GFX_BACKEND=vulkan physics_capture OUTPUT_DIRECTORY
//! CPU scenario only: physics_capture OUTPUT_DIRECTORY --cpu
//! Frames: animated, activating, falling, settled (two contacting ragdolls
//! at left; a Parent-transformed SpringBone chain and its collider at right).

#[cfg(target_arch = "wasm32")]
fn main() {}
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    native::run();
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use og_anim::{NONE, Skeleton, SkeletonAsset, SkeletonNode, SpringBone, SpringCollider, Trs};
    use og_core::List;
    use og_core::glam::{Mat4, Quat, Vec3};
    use og_ecs::transform::{Parent, Transform};
    use og_ecs::{Entity, World, WorldDesc};
    use og_physics::{Collider, Physics, Ragdoll, RagdollBone, RagdollRig};
    use og_render::{
        Backend, Background, Camera, RenderDesc, RenderScene, RenderTarget, Renderer,
        StandardMaterial, geometry, shapes,
    };

    fn rig() -> RagdollRig {
        // Deliberately authored indices and axes, not humanoid inference.
        let definitions = [
            (None, [0.0, 1.1, 0.0], Vec3::Y, 0.35, 10.0),
            (Some(0), [0.0, 1.45, 0.0], Vec3::Y, 0.45, 8.0),
            (Some(1), [0.0, 1.9, 0.0], Vec3::Y, 0.3, 4.0),
            (Some(1), [-0.2, 1.72, 0.0], -Vec3::X, 0.45, 2.0),
            (Some(3), [-0.65, 1.72, 0.0], -Vec3::X, 0.4, 1.0),
            (Some(1), [0.2, 1.72, 0.0], Vec3::X, 0.45, 2.0),
            (Some(5), [0.65, 1.72, 0.0], Vec3::X, 0.4, 1.0),
            (Some(0), [-0.15, 1.1, 0.0], -Vec3::Y, 0.55, 6.0),
            (Some(7), [-0.15, 0.55, 0.0], -Vec3::Y, 0.5, 3.0),
            (Some(0), [0.15, 1.1, 0.0], -Vec3::Y, 0.55, 6.0),
            (Some(9), [0.15, 0.55, 0.0], -Vec3::Y, 0.5, 3.0),
        ];
        RagdollRig {
            bones: definitions
                .into_iter()
                .enumerate()
                .map(|(i, (parent, p, axis, length, mass))| {
                    let hinge = [4, 6, 8, 10].contains(&i);
                    RagdollBone {
                        parent,
                        rest: (Vec3::from_array(p), Quat::IDENTITY),
                        axis,
                        length,
                        mass,
                        shape: 0,
                        joint_kind: u32::from(hinge),
                        hinge_axis: if hinge { Vec3::Z } else { Vec3::ZERO },
                        swing: 0.8,
                        twist_min: if hinge { 0.0 } else { -0.4 },
                        twist_max: if hinge { 2.3 } else { 0.4 },
                    }
                })
                .collect(),
        }
    }

    pub fn run() {
        let out = std::env::args()
            .nth(1)
            .expect("physics_capture OUTPUT_DIRECTORY [--cpu]");
        let cpu = std::env::args().any(|a| a == "--cpu");
        std::fs::create_dir_all(&out).unwrap();
        let mut world = World::new(&WorldDesc {
            tick_rate: 60,
            seed: 5,
        })
        .unwrap();
        let anim = og_anim::install(&mut world).unwrap();
        let physics = og_physics::install(&mut world, Physics::default()).unwrap();
        anim.set_physics(physics.clone());
        world
            .spawn((
                Transform::at(Vec3::new(0.0, -0.5, 0.0)),
                Collider::cuboid(Vec3::new(20.0, 1.0, 12.0)),
            ))
            .unwrap();
        let rig = rig();
        let mut ragdolls: Vec<Entity> = Vec::new();
        for offset in [Vec3::new(-2.5, 0.25, 0.0), Vec3::new(-2.1, 0.35, -0.55)] {
            let e = world
                .spawn((
                    Transform::at(offset),
                    Ragdoll {
                        bone_mask: List::from_slice(
                            &(0..rig.bones.len() as u32).collect::<Vec<_>>(),
                        )
                        .unwrap(),
                        layers: 1,
                        mask: 1,
                        thickness: 0.22,
                        drive_transform: true,
                        ..Default::default()
                    },
                ))
                .unwrap();
            physics.set_ragdoll_rig(e, rig.clone()).unwrap();
            ragdolls.push(e);
        }
        let skeleton = SkeletonAsset {
            nodes: (0..5)
                .map(|i| SkeletonNode {
                    name: format!("segment-{i}"),
                    parent: if i == 0 { NONE } else { i - 1 },
                    rest: Trs::new(
                        if i == 0 { Vec3::ZERO } else { Vec3::X * 0.6 },
                        Quat::IDENTITY,
                        Vec3::ONE,
                    ),
                    mesh: NONE,
                })
                .collect(),
            ..Default::default()
        };
        anim.add_skeleton(1, skeleton.clone()).unwrap();
        let parent = world
            .spawn((Transform {
                position: Vec3::new(0.5, 2.2, 0.0),
                quaternion: og_core::math::quat_from_axis_angle(Vec3::Z, 0.2),
                scale: Vec3::new(1.2, 0.8, 1.0),
            },))
            .unwrap();
        let chain = world
            .spawn((
                Transform::at(Vec3::X * 0.5),
                Parent {
                    entity: parent.to_bits(),
                },
                Skeleton {
                    asset: 1,
                    ..Default::default()
                },
            ))
            .unwrap();
        world
            .spawn((SpringBone {
                skeleton: chain.to_bits(),
                joint: 3,
                bones: 3,
                stiffness: 25.0,
                drag: 6.0,
                tail_offset: Vec3::X * 0.6,
                gravity: -Vec3::Y * 9.8,
                radius: 0.035,
                teleport_distance: 2.0,
                max_speed: 6.0,
                world_colliders: 1,
                ..Default::default()
            },))
            .unwrap();
        world
            .spawn((SpringCollider {
                skeleton: chain.to_bits(),
                joint: 0,
                offset: Vec3::new(1.0, -0.4, 0.0),
                radius: 0.3,
                ..Default::default()
            },))
            .unwrap();
        physics.query(&world).unwrap();
        let mut renderer = (!cpu).then(|| {
            let r = Renderer::new_blocking(&RenderDesc {
                width: 1280,
                height: 720,
                backend: Backend::Vulkan,
                ..Default::default()
            })
            .expect("NVIDIA renderer");
            assert!(
                r.info().adapter.contains("NVIDIA"),
                "visual review requires an NVIDIA GPU"
            );
            println!("renderer=vulkan adapter={}", r.info().adapter);
            r
        });
        for tick in 0..=360 {
            let mut t = world.get::<Transform>(parent).unwrap();
            t.position.x = 0.5 + og_core::math::sin(tick as f32 * 0.04) * 0.3;
            world.insert(parent, t).unwrap();
            if tick == 60 {
                for (i, &e) in ragdolls.iter().enumerate() {
                    let mut r = world.get::<Ragdoll>(e).unwrap();
                    r.weight = 1.0;
                    r.impulse = Vec3::new(8.0, 1.0, if i == 0 { -3.0 } else { 3.0 });
                    world.insert(e, r).unwrap();
                }
            }
            world.step().unwrap();
            let Some(label) = (match tick {
                30 => Some("animated"),
                61 => Some("activating"),
                90 => Some("falling"),
                360 => Some("settled"),
                _ => None,
            }) else {
                continue;
            };
            let poses: Vec<_> = ragdolls
                .iter()
                .map(|&e| physics.ragdoll_pose(e).unwrap())
                .collect();
            assert!(
                poses
                    .iter()
                    .flatten()
                    .all(|(p, q)| p.is_finite() && q.is_finite())
            );
            let nodes = skeleton.model_matrices(&anim.pose(chain).unwrap());
            let chain_frame = world.get::<Transform>(parent).unwrap().matrix()
                * world.get::<Transform>(chain).unwrap().matrix();
            let mut scene = RenderScene::new();
            scene.frame.camera = Camera {
                position: Vec3::new(1.0, 3.0, 6.8),
                target: Vec3::new(0.0, 1.0, 0.0),
                projection: og_render::Projection::Perspective {
                    fov_y_degrees: 40.0,
                },
                ..Default::default()
            };
            scene.lighting.environment.background =
                Background::Color(Vec3::new(0.055, 0.075, 0.11));
            scene.lighting.light.direction_to_light = Vec3::new(-3.0, 5.0, 4.0);
            scene.lighting.light.intensity = 3.0;
            let floor = scene.add_mesh(shapes::cuboid(Vec3::new(20.0, 1.0, 12.0)));
            let grey = scene.add_standard_material(StandardMaterial {
                color: Vec3::splat(0.22),
                ..Default::default()
            });
            scene.add_instance(floor, grey, Mat4::from_translation(-Vec3::Y * 0.5));
            for (i, pose) in poses.iter().enumerate() {
                let color = if i == 0 {
                    Vec3::new(0.08, 0.5, 0.9)
                } else {
                    Vec3::new(0.9, 0.3, 0.08)
                };
                let material = scene.add_standard_material(StandardMaterial {
                    color,
                    ..Default::default()
                });
                for (b, &(p, q)) in rig.bones.iter().zip(pose) {
                    let radius = b.length * 0.22;
                    let mesh = scene.add_mesh(geometry::primitives::capsule(
                        radius,
                        (b.length - radius * 2.0).max(0.0),
                        8,
                        16,
                        1,
                    ));
                    scene.add_instance(
                        mesh,
                        material,
                        Mat4::from_rotation_translation(
                            q * og_anim::rotation_arc(Vec3::Y, b.axis),
                            p + q * b.axis * (b.length * 0.5),
                        ),
                    );
                }
            }
            let green = scene.add_standard_material(StandardMaterial {
                color: Vec3::new(0.1, 0.8, 0.35),
                ..Default::default()
            });
            for node in nodes.iter().take(4).skip(1) {
                let a = (chain_frame * *node).transform_point3(Vec3::ZERO);
                let b = (chain_frame * *node).transform_point3(Vec3::X * 0.6);
                let mesh = scene.add_mesh(geometry::primitives::capsule(
                    0.035,
                    (a.distance(b) - 0.07).max(0.0),
                    8,
                    16,
                    1,
                ));
                scene.add_instance(
                    mesh,
                    green,
                    Mat4::from_rotation_translation(
                        og_anim::rotation_arc(Vec3::Y, b - a),
                        (a + b) * 0.5,
                    ),
                );
            }
            let ball = scene.add_mesh(shapes::sphere(24, 16));
            scene.add_instance(
                ball,
                grey,
                chain_frame
                    * Mat4::from_scale_rotation_translation(
                        Vec3::splat(0.6),
                        Quat::IDENTITY,
                        Vec3::new(1.0, -0.4, 0.0),
                    ),
            );
            if let Some(r) = renderer.as_mut() {
                r.render(&scene, &RenderTarget::Offscreen).unwrap();
                std::fs::write(
                    format!("{out}/{label}.png"),
                    og_render::png(&r.read_offscreen().unwrap()),
                )
                .unwrap();
            }
            println!(
                "tick={tick} state={label} finite=true root={:?} spring_tip={:?}",
                poses[0][0].0,
                (chain_frame * nodes[3]).transform_point3(Vec3::X * 0.6)
            );
        }
    }
}
