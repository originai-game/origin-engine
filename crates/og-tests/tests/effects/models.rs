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
        scene.frame.camera = camera;
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
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 3.0, 6.0),
        target: Vec3::new(0.0, 1.0, -2.0),
        ..Default::default()
    };
    let (crowd, stats) = frame(&desc, &scene);
    save("og-effects", "crowd-baked", &crowd);
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
        scene.frame.camera = Camera {
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
    save("og-effects", "deformed-vertices", &tall);
    let top = |img: &RgbaImage| {
        (0..180)
            .find(|&y| (0..320).any(|x| luma(&px_clamped(img, x, y)) > 20.0))
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

/// A model fading in (`InstanceFx::reveal`): a box over a lit ground, drawn
/// through the lean pipelines (a plain material), the full ones (a
/// textured one) and under GPU culling, at five amounts. It draws that
/// share of its pixels, its shadow darkens by that share, the pattern
/// turns with each drawn frame, and at 0 the frame is the one without the box: no
/// pass (lit, prepass, shadow, the culling's occluders) draws what the
/// dither cut.
#[test]
#[ignore = "requires GPU"]
fn a_model_fading_in_draws_its_share_of_pixels_and_of_its_shadow() {
    let camera = Camera {
        position: Vec3::new(0.0, 3.2, 6.5),
        target: Vec3::new(-0.6, 0.5, 0.0),
        ..Default::default()
    };
    let build = |textured: bool, reveal: Option<f32>, tick: u64| {
        let mut scene = RenderScene::new();
        let ground = scene.add_mesh(shapes::plane(14.0, 1.0));
        let white = scene.add_material(Material::default());
        scene.add_instance(ground, white, Mat4::IDENTITY);
        let texture = textured.then(|| scene.add_texture(shapes::checker(64, 4)));
        let red = scene.add_material(Material {
            base_color: [0.9, 0.04, 0.04, 1.0],
            base_color_texture: texture,
            ..Default::default()
        });
        let mesh = scene.add_mesh(shapes::cuboid(Vec3::splat(1.6)));
        if let Some(reveal) = reveal {
            scene.add_instance(mesh, red, Mat4::from_translation(Vec3::new(0.0, 0.8, 0.0)));
            scene.objects.instances.last_mut().expect("added").fx = og_render::InstanceFx {
                reveal,
                ..og_render::InstanceFx::NONE
            };
        }
        scene.lighting.light.direction_to_light = Vec3::new(0.7, 0.6, 0.25).normalize();
        scene.lighting.light.cast_shadows = true;
        scene.frame.camera = camera;
        scene.frame.tick = tick;
        scene
    };
    let red = |p: [u8; 3]| p[0] as i32 > p[1] as i32 + 60;
    let count = |img: &RgbaImage, keep: &dyn Fn(u32, u32) -> bool| {
        let mut n = 0u32;
        for y in 0..img.height {
            for x in 0..img.width {
                n += keep(x, y) as u32;
            }
        }
        n
    };
    for (name, textured) in [("lean", false), ("textured", true)] {
        let desc = small();
        // Two frames each (GPU culling's occluders are the frame before's),
        // or `frames`.
        let shot_after = |reveal: Option<f32>, tick: u64, frames: u32| {
            let mut scene = build(textured, reveal, tick);
            run(&desc, &mut scene, frames, |_, _| {}).0
        };
        let shot = |reveal: Option<f32>, tick: u64| shot_after(reveal, tick, 2);
        let (whole, empty) = (shot(Some(1.0), 7), shot(None, 7));
        let boxed = count(&whole, &|x, y| red(px_clamped(&whole, x, y)));
        assert!(boxed > 1500, "{name}: the box covers {boxed} pixels");
        // Its shadow on the ground: darker with the box than without, and
        // not the box itself.
        let shaded = |x: u32, y: u32| {
            !red(px_clamped(&whole, x, y))
                && luma(&px_clamped(&empty, x, y)) - luma(&px_clamped(&whole, x, y)) > 25.0
        };
        let shadow = count(&whole, &shaded);
        assert!(shadow > 400, "{name}: the shadow covers {shadow} pixels");
        let shade = |img: &RgbaImage| {
            let mut sum = 0.0;
            for y in 0..img.height {
                for x in 0..img.width {
                    if shaded(x, y) {
                        sum += luma(&px_clamped(&empty, x, y)) - luma(&px_clamped(img, x, y));
                    }
                }
            }
            sum / shadow as f32
        };
        let full_shade = shade(&whole);
        let mut last = full_shade;
        for reveal in [0.75, 0.5, 0.25] {
            let img = shot(Some(reveal), 7);
            save("og-effects", &format!("reveal-{name}-{reveal}"), &img);
            let drawn = count(&img, &|x, y| red(px_clamped(&img, x, y))) as f32 / boxed as f32;
            assert!(
                (drawn - reveal).abs() < 0.06,
                "{name}: at reveal {reveal} it draws {drawn} of its pixels"
            );
            // Where it is cut the ground shows, never black or the clear colour.
            let through = count(&img, &|x, y| {
                red(px_clamped(&whole, x, y))
                    && !red(px_clamped(&img, x, y))
                    && luma(&px_clamped(&img, x, y)) < 20.0
            });
            assert_eq!(through, 0, "{name}: {through} cut pixels are black");
            let s = shade(&img);
            // Lighter with every step, and neither whole nor gone: the
            // frame is display-encoded, so the share is not the luma's.
            assert!(
                s < last - 2.0 && s > 0.1 * full_shade,
                "{name}: at reveal {reveal} its shadow is {s} of {full_shade}, after {last}"
            );
            last = s;
        }
        // The pattern turns with the frames drawn (the TAA jitter's clock,
        // common.wgsl `dither_cut`), the share drawn does not.
        let (a, b) = (shot(Some(0.5), 7), shot_after(Some(0.5), 7, 3));
        let moved = count(&a, &|x, y| {
            red(px_clamped(&a, x, y)) != red(px_clamped(&b, x, y))
        });
        assert!(
            moved > boxed / 4,
            "{name}: {moved} of {boxed} pixels changed between two frames"
        );
        // Nothing of it is left at 0, in any pass.
        let gone = shot(Some(0.0), 7);
        save("og-effects", &format!("reveal-{name}-0"), &gone);
        assert_eq!(
            differing(&gone, &empty, 1.0),
            0,
            "{name}: at reveal 0 the frame is the one without the box"
        );
        save("og-effects", &format!("reveal-{name}-1"), &whole);
    }
}

/// A model held at reveal 0 stops no frame to compile: nothing of it is
/// drawn while its pipelines are made on the compile workers, and neither
/// the frame that starts its fade nor the one that ends it waits for one.
#[test]
#[ignore = "requires GPU"]
fn a_hidden_model_compiles_ahead_and_its_fade_waits_for_no_pipeline() {
    let mut scene = RenderScene::new();
    let ground = scene.add_mesh(shapes::plane(14.0, 1.0));
    let white = scene.add_material(Material::default());
    scene.add_instance(ground, white, Mat4::IDENTITY);
    // The model's resources are the scene's from the start: only its
    // pipelines are new when it is first drawn.
    let texture = scene.add_texture(shapes::checker(64, 4));
    let red = scene.add_material(Material {
        base_color: [0.9, 0.04, 0.04, 1.0],
        base_color_texture: Some(texture),
        ..Default::default()
    });
    let mesh = scene.add_mesh(shapes::cuboid(Vec3::splat(1.6)));
    scene.lighting.light.direction_to_light = Vec3::new(0.7, 0.6, 0.25).normalize();
    scene.lighting.light.cast_shadows = true;
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 3.2, 6.5),
        target: Vec3::new(-0.6, 0.5, 0.0),
        ..Default::default()
    };
    let mut renderer = Renderer::new_blocking(&small()).expect("renderer");
    let frame = |renderer: &mut Renderer, scene: &mut RenderScene| {
        scene.frame.tick += 1;
        renderer
            .render(scene, &RenderTarget::Offscreen)
            .expect("render")
    };
    for _ in 0..3 {
        frame(&mut renderer, &mut scene);
    }
    scene.add_instance(mesh, red, Mat4::from_translation(Vec3::new(0.0, 0.8, 0.0)));
    let model = scene.objects.instances.len() - 1;
    let red_pixels = |renderer: &mut Renderer| {
        let img = renderer.read_offscreen().expect("capture");
        (0..img.width * img.height)
            .filter(|i| {
                let p = px_clamped(&img, i % img.width, i / img.width);
                p[0] as i32 > p[1] as i32 + 60
            })
            .count()
    };
    // Each stage of its fade: no frame waits, and the pipelines the next
    // stage draws with are made before it starts.
    for (reveal, least) in [(0.0, 0), (0.5, 500), (1.0, 1500)] {
        scene.objects.instances[model].fx.reveal = reveal;
        let stats = frame(&mut renderer, &mut scene);
        assert_eq!(stats.compile_wait_us, 0, "reveal {reveal} waited");
        let drawn = red_pixels(&mut renderer);
        assert!(
            drawn >= least && (reveal > 0.0 || drawn == 0),
            "reveal {reveal} drew {drawn} pixels"
        );
        let mut compiling = stats.compiling_ahead;
        for _ in 0..6000 {
            if !compiling {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
            let stats = frame(&mut renderer, &mut scene);
            assert_eq!(stats.compile_wait_us, 0, "reveal {reveal} waited");
            compiling = stats.compiling_ahead;
        }
        assert!(!compiling, "reveal {reveal}: still compiling ahead");
    }
}
