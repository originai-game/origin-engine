//! Stylised materials, skinning and blendshape frame tests.

use super::*;

/// Baked clips (GPU crowds): an instance posed by a baked clip at a whole
/// frame draws as the same pose through a palette, and between frames as
/// the palette the clip blends on the CPU (`BakedClip::palette`), so the
/// shader's blend is the CPU's. Then a crowd of 35 cylinders playing one
/// shared clip at their own frames.
#[test]
#[ignore = "requires GPU"]
fn baked_clips_pose_a_crowd_like_their_palettes() {
    let desc = small();
    let (mesh, skin) = fx::skinned_cylinder(0.3, 24, 24);
    let clip = og_render::BakedClip {
        joints: fx::CYLINDER_BONES as u32,
        frames: [0.0, 0.25, 0.5]
            .iter()
            .flat_map(|&b| fx::bend_palette(b, 0.0).matrices)
            .collect(),
    };
    let pink = Material {
        base_color: [0.8, 0.3, 0.5, 1.0],
        ..Default::default()
    };
    let camera = Camera {
        position: Vec3::new(0.0, 1.4, 5.0),
        target: Vec3::new(0.0, 1.2, 0.0),
        ..Default::default()
    };
    // One cylinder, by a palette or at `frame` of the clip.
    let one = |pose: Result<JointPalette, f32>| {
        let mut scene = RenderScene::new();
        let ground = scene.add_mesh(shapes::plane(10.0, 1.0));
        let white = scene.add_material(Material::default());
        scene.add_instance(ground, white, Mat4::IDENTITY);
        let m = scene.add_material(pink.clone());
        let h = scene.add_skinned_mesh(mesh.clone(), skin.clone());
        let mut inst = SkinnedInstance::new(h, m, Mat4::IDENTITY, 0);
        match pose {
            Ok(p) => scene.objects.joint_palettes.push(p),
            Err(frame) => {
                scene.objects.baked_clips.push(clip.clone());
                inst.baked = Some(og_render::BakedPose {
                    clip: 0,
                    frame,
                    previous_clip: 0,
                    previous_frame: frame,
                    previous_transform: Mat4::IDENTITY,
                });
            }
        }
        scene.objects.skinned_instances.push(inst);
        scene.camera = camera;
        scene
    };
    for (at, palette) in [
        (1.0, fx::bend_palette(0.25, 0.0)),
        (1.5, JointPalette::from(clip.palette(1.5))),
        (2.0, fx::bend_palette(0.5, 0.0)),
    ] {
        let (baked, stats) = frame(&desc, &one(Err(at)));
        let (posed, _) = frame(&desc, &one(Ok(palette)));
        assert_eq!(stats.skinned_instances, 1);
        let d = differing(&baked, &posed, 4.0);
        println!("baked at frame {at} vs its palette: {d} pixels differ");
        assert!(d < 40, "frame {at}: {d} pixels differ");
    }
    // The frames differ from each other (the clip plays).
    let (a, _) = frame(&desc, &one(Err(0.0)));
    let (b, _) = frame(&desc, &one(Err(2.0)));
    assert!(differing(&a, &b, 8.0) > 1_500);
    // Playback and a same-index replacement must both use the current bake
    // when drawing repeatedly through one renderer's persistent cache.
    let mut resident = one(Err(0.0));
    let mut cached = Renderer::new_blocking(&desc).expect("renderer");
    for at in [0.0, 1.5, 2.0, 0.0] {
        resident.objects.skinned_instances[0]
            .baked
            .as_mut()
            .unwrap()
            .frame = at;
        cached.render(&resident, &RenderTarget::Offscreen).unwrap();
        let baked = cached.read_offscreen().unwrap();
        let (posed, _) = frame(&desc, &one(Ok(JointPalette::from(clip.palette(at)))));
        assert!(differing(&baked, &posed, 4.0) < 40, "cached frame {at}");
    }
    resident.objects.baked_clips[0].frames = [0.5, 0.25, 0.0]
        .iter()
        .flat_map(|&b| fx::bend_palette(b, 0.0).matrices)
        .collect();
    cached.render(&resident, &RenderTarget::Offscreen).unwrap();
    let (posed, _) = frame(&desc, &one(Ok(fx::bend_palette(0.5, 0.0))));
    assert!(
        differing(&cached.read_offscreen().unwrap(), &posed, 4.0) < 40,
        "replaced bake"
    );
    // A clip that is not in the scene is refused.
    let mut bad = one(Err(1.0));
    bad.objects.baked_clips.clear();
    let mut r = Renderer::new_blocking(&desc).expect("renderer");
    assert!(r.render(&bad, &RenderTarget::Offscreen).is_err());
    // The crowd: 7 x 5 on a ground, one clip, each at its own frame.
    let mut scene = RenderScene::new();
    let ground = scene.add_mesh(shapes::plane(20.0, 1.0));
    let white = scene.add_material(Material::default());
    scene.add_instance(ground, white, Mat4::IDENTITY);
    let m = scene.add_material(pink);
    let h = scene.add_skinned_mesh(mesh.clone(), skin.clone());
    scene.objects.baked_clips.push(clip.clone());
    for z in 0..5 {
        for x in 0..7 {
            let k = z * 7 + x;
            let f = (k as f32 * 0.37) % 2.0;
            let at = Mat4::from_translation(Vec3::new(x as f32 - 3.0, 0.0, -(z as f32) * 1.2));
            let mut inst = SkinnedInstance::new(h, m, at, 0);
            inst.baked = Some(og_render::BakedPose {
                clip: 0,
                frame: f,
                previous_clip: 0,
                previous_frame: f,
                previous_transform: at,
            });
            scene.objects.skinned_instances.push(inst);
        }
    }
    scene.camera = Camera {
        position: Vec3::new(0.0, 3.0, 6.0),
        target: Vec3::new(0.0, 1.0, -2.0),
        ..Default::default()
    };
    let (crowd, stats) = frame(&desc, &scene);
    save("crowd-baked", &crowd);
    assert_eq!(stats.skinned_instances, 35);
}

/// Deformed vertices (a simulated garment's): a skinned instance with its
/// own positions draws them instead of its mesh's. The bind positions draw
/// the plain mesh; the positions of the blendshape test's taller cylinder
/// draw that cylinder, so the shader reads every vertex, odd and even.
#[test]
#[ignore = "requires GPU"]
fn deformed_vertices_replace_the_mesh() {
    let desc = small();
    let (mesh, _) = fx::skinned_cylinder(0.3, 24, 24);
    let n = mesh.vertices.len();
    let build = |stretch: Option<f32>| {
        let mut scene = RenderScene::new();
        let m = scene.add_material(Material {
            base_color: [0.8, 0.3, 0.5, 1.0],
            ..Default::default()
        });
        match stretch {
            None => {
                let h = scene.add_mesh(mesh.clone());
                scene.add_instance(h, m, Mat4::IDENTITY);
            }
            Some(k) => {
                // Zero skin weights: the palette leaves the vertices alone.
                let h = scene.add_skinned_mesh(
                    mesh.clone(),
                    og_render::SkinData {
                        joints: vec![[0; 4]; n],
                        weights: vec![[0.0; 4]; n],
                    },
                );
                scene.objects.joint_palettes.push(JointPalette::default());
                scene
                    .objects
                    .deformed_vertices
                    .push(og_render::DeformedVertices {
                        positions: mesh
                            .vertices
                            .iter()
                            .map(|v| {
                                let p = Vec3::from(v.position);
                                p + Vec3::new(0.0, p.y * k, 0.0)
                            })
                            .collect(),
                        normals: mesh.vertices.iter().map(|v| Vec3::from(v.normal)).collect(),
                        garment: None,
                    });
                let mut inst = SkinnedInstance::new(h, m, Mat4::IDENTITY, 0);
                inst.deformed = Some(0);
                scene.objects.skinned_instances.push(inst);
            }
        }
        scene.lighting.environment.background = Background::Color(Vec3::ZERO);
        scene.camera = Camera {
            position: Vec3::new(0.0, 1.6, 6.0),
            target: Vec3::new(0.0, 1.6, 0.0),
            ..Default::default()
        };
        scene
    };
    let (plain, _) = frame(&desc, &build(None));
    let (rest, stats) = frame(&desc, &build(Some(0.0)));
    assert_eq!(stats.skinned_instances, 1);
    let d = differing(&rest, &plain, 4.0);
    println!("deformed at the bind positions vs the plain mesh: {d} pixels differ");
    assert!(d < 40, "the bind positions are not the mesh ({d} pixels)");
    let (tall, _) = frame(&desc, &build(Some(0.4)));
    save("deformed-vertices", &tall);
    let top = |img: &RgbaImage| {
        (0..180)
            .find(|&y| (0..320).any(|x| luma(px(img, x, y)) > 20.0))
            .expect("drawn")
    };
    println!("top row: rest {}, stretched {}", top(&rest), top(&tall));
    assert!(top(&tall) + 10 < top(&rest));
    // The compute path must draw the same skin, offsets and gathered normals
    // as the CPU fallback, including saved positions on the second TAA frame.
    let topology = std::sync::Arc::new(og_render::GarmentTopology {
        rest: mesh
            .vertices
            .iter()
            .map(|v| Vec3::from(v.position))
            .collect(),
        normals: mesh.vertices.iter().map(|v| Vec3::from(v.normal)).collect(),
        skin: vec![([0, 1, 0, 0], [0.25, 0.75, 0., 0.]); n],
        triangles: mesh.indices.as_chunks::<3>().0.to_vec(),
        vertices: (0..n).map(|i| (i as u32, 1.)).collect(),
    });
    for antialias in [og_render::Antialias::Taa, og_render::Antialias::Off] {
        let mut taa = desc;
        taa.antialias = antialias;
        let mut gpu = Renderer::new_blocking(&taa).expect("compute garment");
        let mut cpu = Renderer::new_blocking(&taa).expect("reference garment");
        let mut scene = build(Some(0.));
        scene.look.post.motion_blur = Some(MotionBlur::default());
        let mut previous = None::<og_render::GarmentDeformation>;
        for step in 0..3 {
            let world = Mat4::from_translation(Vec3::new(step as f32 * 0.2, 0., 0.));
            let garment = og_render::GarmentDeformation {
                key: (1, 0),
                topology: topology.clone(),
                palette: std::sync::Arc::new(vec![
                    Mat4::IDENTITY,
                    Mat4::from_rotation_z(0.1 * step as f32),
                ]),
                points: std::sync::Arc::new(
                    mesh.vertices
                        .iter()
                        .map(|v| Vec3::new(1., v.position[1] * 0.15 + 2., 0.03 * step as f32 - 3.))
                        .collect(),
                ),
                targets: std::sync::Arc::new(vec![Vec3::new(1., 2., -3.); n]),
                world,
                output_transform: Mat4::IDENTITY,
                visual: None,
            };
            let mut before = previous.clone().unwrap_or_else(|| garment.clone());
            before.output_transform = world.inverse() * before.world;
            scene.objects.skinned_instances[0].transform = world;
            scene.objects.previous_joint_palettes = scene.objects.joint_palettes.clone();
            scene.objects.previous_deformed_vertices = vec![before.evaluate()];
            scene.objects.deformed_vertices[0] = og_render::DeformedVertices {
                garment: Some(garment.clone()),
                ..Default::default()
            };
            let stats = gpu
                .render(&scene, &RenderTarget::Offscreen)
                .expect("compute garment frame");
            assert!(stats.prepass_draws > 0);
            let computed = gpu.read_offscreen().unwrap();
            scene.objects.deformed_vertices[0] = garment.evaluate();
            cpu.render(&scene, &RenderTarget::Offscreen)
                .expect("fallback reference frame");
            let reference = cpu.read_offscreen().unwrap();
            let d = differing(&computed, &reference, 4.);
            assert!(
                d < 40,
                "garment frame {step}: compute differs from fallback at {d} pixels"
            );
            previous = Some(garment);
        }
    }
    // A count that does not match the mesh is refused.
    let mut bad = build(Some(0.0));
    bad.objects.deformed_vertices[0].positions.pop();
    let mut r = Renderer::new_blocking(&desc).expect("renderer");
    assert!(r.render(&bad, &RenderTarget::Offscreen).is_err());
}
