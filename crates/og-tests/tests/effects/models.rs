//! Stylised materials, skinning and blendshape frame tests.

use super::*;

// ---------------------------------------------------------------------------
// Stylised: outlines, toon, rim, dissolve, flash.
// ---------------------------------------------------------------------------

fn sphere_scene(m: StandardMaterial) -> RenderScene {
    let mut scene = RenderScene::new();
    let sphere = scene.add_mesh(shapes::sphere(48, 32));
    let mat = scene.add_standard_material(m);
    scene.add_instance(sphere, mat, Mat4::from_scale(Vec3::splat(2.0)));
    scene.environment.background = Background::Color(Vec3::new(0.6, 0.7, 0.9));
    scene.camera = Camera {
        position: Vec3::new(0.0, 0.0, 4.0),
        target: Vec3::ZERO,
        exposure: Exposure::linear(0.8),
        ..Default::default()
    };
    scene
}

fn red() -> StandardMaterial {
    StandardMaterial {
        color: Vec3::new(0.8, 0.2, 0.15),
        roughness: 0.5,
        ..Default::default()
    }
}

/// A material outline draws a dark ring inside the silhouette of its width.
#[test]
fn outlines_follow_silhouettes() {
    let (plain, _) = frame(&small(), &sphere_scene(red()));
    let (lined, stats) = frame(
        &small(),
        &sphere_scene(StandardMaterial {
            outlineWidth: 3.0,
            outlineColor: Vec3::ZERO,
            ..red()
        }),
    );
    assert!(stats.prepass_draws > 0);
    // The rim at the sphere's right edge (radius 1 at distance 4).
    let row = 90;
    let edge = (160..320)
        .rev()
        .find(|&x| px(&plain, x, row) != px(&plain, 319, row))
        .expect("edge");
    let inside = edge - 1;
    println!(
        "edge at {edge}: plain {:?} outlined {:?}",
        px(&plain, inside, row),
        px(&lined, inside, row)
    );
    assert!(luma(px(&lined, inside, row)) < 15.0);
    assert!(
        luma(px(&lined, 160, 90)) > 30.0,
        "the outline covered the middle"
    );
    // The scene-wide outline alone works too.
    let mut global = sphere_scene(red());
    global.post.outline = Some(Outline::default());
    let (g, _) = frame(&small(), &global);
    println!(
        "scene-wide outline around the edge: {:?}",
        (edge - 4..=edge + 1)
            .map(|x| px(&g, x, row))
            .collect::<Vec<_>>()
    );
    assert!(luma(px(&g, edge, row)) < 30.0 && luma(px(&g, inside, row)) < 30.0);
}

/// Toon shading quantizes the light: far fewer distinct shades across the
/// lit sphere than PBR; rim light brightens the edge; flash and dissolve.
#[test]
fn toon_rim_flash_and_dissolve() {
    let shades = |img: &RgbaImage| {
        let mut set = std::collections::BTreeSet::new();
        for x in 100..220 {
            set.insert(luma(px(img, x, 90)) as u32 / 3);
        }
        set.len()
    };
    let (pbr, _) = frame(&small(), &sphere_scene(red()));
    let (toon, _) = frame(
        &small(),
        &sphere_scene(StandardMaterial { toon: 1, ..red() }),
    );
    println!(
        "distinct shades: pbr {}, toon {}",
        shades(&pbr),
        shades(&toon)
    );
    assert!(shades(&toon) * 2 < shades(&pbr));
    let (rim, _) = frame(
        &small(),
        &sphere_scene(StandardMaterial {
            rimIntensity: 2.0,
            ..red()
        }),
    );
    let edge = (160..320)
        .rev()
        .find(|&x| px(&pbr, x, 90) != px(&pbr, 319, 90))
        .expect("edge")
        - 2;
    assert!(luma(px(&rim, edge, 90)) > luma(px(&pbr, edge, 90)) + 20.0);
    let (flash, _) = frame(
        &small(),
        &sphere_scene(StandardMaterial {
            flash: 1.0,
            ..red()
        }),
    );
    assert!(px(&flash, 160, 90).iter().all(|&c| c > 220));
    // Dissolve: 1 is gone (the background), 0.5 has holes and a glowing edge.
    let (gone, _) = frame(
        &small(),
        &sphere_scene(StandardMaterial {
            dissolve: 1.0,
            ..red()
        }),
    );
    assert_eq!(px(&gone, 160, 90), px(&gone, 2, 2));
    let (half, _) = frame(
        &small(),
        &sphere_scene(StandardMaterial {
            dissolve: 0.5,
            ..red()
        }),
    );
    let disc: Vec<[u8; 3]> = (130..190)
        .flat_map(|x| (60..120).map(move |y| (x, y)))
        .map(|(x, y)| px(&half, x, y))
        .collect();
    let holes = disc.iter().filter(|&&p| p == px(&half, 2, 2)).count();
    let glowing = disc.iter().filter(|p| p[0] > 240 && p[1] > 120).count();
    println!("dissolve 0.5: {holes} background pixels, {glowing} glowing");
    assert!(holes > 300 && glowing > 20);
}

/// A dissolved pixel is gone from every pass, not only the lit one: with the
/// depth-normal prepass on (outlines, ambient occlusion, TAA) a fully
/// dissolved sphere leaves the background, and half-dissolved holes show
/// the background too (on Metal they showed black: the prepass kept them).
#[test]
fn dissolved_pixels_leave_no_trace_in_the_prepass() {
    let desc = RenderDesc {
        antialias: og_render::Antialias::Taa,
        ..small()
    };
    let with_prepass = |dissolve: f32| {
        let mut s = sphere_scene(StandardMaterial { dissolve, ..red() });
        s.post.outline = Some(Outline::default());
        s.post.ao = Some(AmbientOcclusion::default());
        let (img, stats, _) = run(&desc, &mut s, 4, |_, _| {});
        assert!(stats.prepass_draws > 0, "the prepass ran");
        img
    };
    let gone = with_prepass(1.0);
    let bg = px(&gone, 2, 2);
    let left: usize = (130..190)
        .flat_map(|x| (60..120).map(move |y| (x, y)))
        .filter(|&(x, y)| px(&gone, x, y) != bg)
        .count();
    println!("dissolve 1 with the prepass: {left} pixels differ from the background {bg:?}");
    assert_eq!(left, 0, "a fully dissolved sphere left {left} pixels");
    let half = with_prepass(0.5);
    let dark = (130..190)
        .flat_map(|x| (60..120).map(move |y| (x, y)))
        .filter(|&(x, y)| luma(px(&half, x, y)) < 20.0)
        .count();
    println!("dissolve 0.5 with the prepass: {dark} near-black pixels");
    assert!(
        dark < 20,
        "{dark} holes drew black instead of the background"
    );
}

// ---------------------------------------------------------------------------
// Skinning and blendshapes.
// ---------------------------------------------------------------------------

/// The procedurally skinned cylinder: an identity palette draws the bind
/// pose (the same pixels as the unskinned mesh), a bent palette moves it,
/// shadows follow, out-of-view instances are culled with widened bounds.
#[test]
fn skinned_cylinder_bends_with_its_palette() {
    let desc = small();
    let (mesh, skin) = fx::skinned_cylinder(0.3, 24, 24);
    let build = |palette: JointPalette, skinned: bool| {
        let mut scene = RenderScene::new();
        let ground = scene.add_mesh(shapes::plane(10.0, 1.0));
        let white = scene.add_material(Material::default());
        scene.add_instance(ground, white, Mat4::IDENTITY);
        let m = scene.add_material(Material {
            base_color: [0.8, 0.3, 0.5, 1.0],
            ..Default::default()
        });
        if skinned {
            let h = scene.add_skinned_mesh(mesh.clone(), skin.clone());
            scene.joint_palettes.push(palette);
            scene
                .skinned_instances
                .push(SkinnedInstance::new(h, m, Mat4::IDENTITY, 0));
        } else {
            let h = scene.add_mesh(mesh.clone());
            scene.add_instance(h, m, Mat4::IDENTITY);
        }
        scene.camera = Camera {
            position: Vec3::new(0.0, 1.4, 5.0),
            target: Vec3::new(0.0, 1.2, 0.0),
            ..Default::default()
        };
        scene
    };
    let identity = JointPalette::from(vec![Mat4::IDENTITY; fx::CYLINDER_BONES]);
    let (rest, stats) = frame(&desc, &build(identity.clone(), true));
    let (plain, _) = frame(&desc, &build(identity, false));
    assert_eq!(
        (stats.skinned_instances, stats.joints),
        (1, fx::CYLINDER_BONES as u32)
    );
    let d = differing(&rest, &plain, 4.0);
    println!("bind pose vs unskinned: {d} pixels differ");
    assert!(d < 40, "the bind pose is not the mesh ({d} pixels)");
    let (bent, _) = frame(&desc, &build(fx::bend_palette(0.5, 0.0), true));
    save("skinned-bent", &bent);
    let moved = differing(&rest, &bent, 8.0);
    assert!(moved > 1_500, "bending moved {moved} pixels");
    // Bending to the left (+z rotation): the top leans towards -x.
    // The cylinder's pink (red and blue above green): its mean x above the
    // base moves left when bent (+z rotation leans towards -x).
    let mean_x = |img: &RgbaImage| {
        let (mut sum, mut n) = (0.0f32, 0.0f32);
        for y in 0..180 {
            for x in 0..320 {
                let [r, g, b] = px(img, x, y).map(i32::from);
                if r > g + 30 && b > g + 10 && y < 120 {
                    sum += x as f32;
                    n += 1.0;
                }
            }
        }
        sum / n.max(1.0)
    };
    let (rx, bx) = (mean_x(&rest), mean_x(&bent));
    println!("mean x of the cylinder: rest {rx}, bent {bx}");
    assert!(bx + 10.0 < rx);
    // Far outside the view: culled.
    let mut away = build(fx::bend_palette(0.3, 0.0), true);
    away.skinned_instances[0].transform = Mat4::from_translation(Vec3::new(60.0, 0.0, 0.0));
    let (_, s) = frame(&desc, &away);
    assert_eq!(s.culled, 1);
}

/// Baked clips (GPU crowds): an instance posed by a baked clip at a whole
/// frame draws as the same pose through a palette, and between frames as
/// the palette the clip blends on the CPU (`BakedClip::palette`), so the
/// shader's blend is the CPU's. Then a crowd of 35 cylinders playing one
/// shared clip at their own frames.
#[test]
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
            Ok(p) => scene.joint_palettes.push(p),
            Err(frame) => {
                scene.baked_clips.push(clip.clone());
                inst.baked = Some(og_render::BakedPose {
                    clip: 0,
                    frame,
                    previous_clip: 0,
                    previous_frame: frame,
                    previous_transform: Mat4::IDENTITY,
                });
            }
        }
        scene.skinned_instances.push(inst);
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
        resident.skinned_instances[0].baked.as_mut().unwrap().frame = at;
        cached.render(&resident, &RenderTarget::Offscreen).unwrap();
        let baked = cached.read_offscreen().unwrap();
        let (posed, _) = frame(&desc, &one(Ok(JointPalette::from(clip.palette(at)))));
        assert!(differing(&baked, &posed, 4.0) < 40, "cached frame {at}");
    }
    resident.baked_clips[0].frames = [0.5, 0.25, 0.0]
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
    bad.baked_clips.clear();
    let mut r = Renderer::new_blocking(&desc).expect("renderer");
    assert!(r.render(&bad, &RenderTarget::Offscreen).is_err());
    // The crowd: 7 x 5 on a ground, one clip, each at its own frame.
    let mut scene = RenderScene::new();
    let ground = scene.add_mesh(shapes::plane(20.0, 1.0));
    let white = scene.add_material(Material::default());
    scene.add_instance(ground, white, Mat4::IDENTITY);
    let m = scene.add_material(pink);
    let h = scene.add_skinned_mesh(mesh.clone(), skin.clone());
    scene.baked_clips.push(clip.clone());
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
            scene.skinned_instances.push(inst);
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
                scene.joint_palettes.push(JointPalette::default());
                scene.deformed_vertices.push(og_render::DeformedVertices {
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
                scene.skinned_instances.push(inst);
            }
        }
        scene.environment.background = Background::Color(Vec3::ZERO);
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
        scene.post.motion_blur = Some(MotionBlur::default());
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
            scene.skinned_instances[0].transform = world;
            scene.previous_joint_palettes = scene.joint_palettes.clone();
            scene.previous_deformed_vertices = vec![before.evaluate()];
            scene.deformed_vertices[0] = og_render::DeformedVertices {
                garment: Some(garment.clone()),
                ..Default::default()
            };
            let stats = gpu
                .render(&scene, &RenderTarget::Offscreen)
                .expect("compute garment frame");
            assert!(stats.prepass_draws > 0);
            let computed = gpu.read_offscreen().unwrap();
            scene.deformed_vertices[0] = garment.evaluate();
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
    bad.deformed_vertices[0].positions.pop();
    let mut r = Renderer::new_blocking(&desc).expect("renderer");
    assert!(r.render(&bad, &RenderTarget::Offscreen).is_err());
}

/// A visual garment (SIMULATION_VISUAL) is moved on the GPU from its
/// skinned pose: a cylinder skirt held at its top ring and blown along +x
/// for 40 ticks flares out on its lee side past its skin (its windward
/// side is held by the backstop, as a body inside would), while the same
/// garment the tick simulates (`visual: None`) draws at its points.
#[test]
fn a_visual_garment_is_simulated_on_the_gpu_from_its_skin() {
    let desc = small();
    let (mesh, _) = fx::skinned_cylinder(0.3, 24, 24);
    let n = mesh.vertices.len();
    let rest: Vec<Vec3> = mesh
        .vertices
        .iter()
        .map(|v| Vec3::from(v.position))
        .collect();
    let topology = std::sync::Arc::new(og_render::GarmentTopology {
        rest: rest.clone(),
        normals: mesh.vertices.iter().map(|v| Vec3::from(v.normal)).collect(),
        skin: vec![([0, 0, 0, 0], [1., 0., 0., 0.]); n],
        triangles: mesh.indices.as_chunks::<3>().0.to_vec(),
        vertices: (0..n).map(|i| (i as u32, 1.)).collect(),
    });
    let top = fx::CYLINDER_HEIGHT - 0.05;
    let reach: Vec<f32> = rest
        .iter()
        .map(|p| if p.y >= top { 0.0 } else { 0.8 })
        .collect();
    let body = og_render::VisualBody {
        pinned: (0..n as u32)
            .filter(|&i| reach[i as usize] <= 0.0)
            .collect(),
        reach: reach.into(),
        backstop: 0.05,
        substeps: 24,
        wind: Vec3::new(14.0, 0.0, 0.0),
        stiffness: 0.9,
        garment: Some(og_render::VisualGarment {
            rest: std::sync::Arc::new(rest.clone()),
            normals: topology.normals.clone().into(),
        }),
        ..Default::default()
    };
    let draw = |visual: bool| {
        let mut scene = RenderScene::new();
        let m = scene.add_material(Material {
            base_color: [0.8, 0.3, 0.5, 1.0],
            ..Default::default()
        });
        let h = scene.add_skinned_mesh(
            mesh.clone(),
            og_render::SkinData {
                joints: vec![[0; 4]; n],
                weights: vec![[0.0; 4]; n],
            },
        );
        scene
            .joint_palettes
            .push(JointPalette::from(vec![Mat4::IDENTITY]));
        scene.deformed_vertices.push(og_render::DeformedVertices {
            garment: Some(og_render::GarmentDeformation {
                key: (7, 0),
                topology: topology.clone(),
                palette: std::sync::Arc::new(vec![Mat4::IDENTITY]),
                points: std::sync::Arc::new(rest.clone()),
                targets: std::sync::Arc::new(rest.clone()),
                world: Mat4::IDENTITY,
                output_transform: Mat4::IDENTITY,
                visual: visual.then(|| std::sync::Arc::new(body.clone())),
            }),
            ..Default::default()
        });
        let mut inst = SkinnedInstance::new(h, m, Mat4::IDENTITY, 0);
        inst.deformed = Some(0);
        scene.skinned_instances.push(inst);
        scene.environment.background = Background::Color(Vec3::ZERO);
        scene.camera = Camera {
            position: Vec3::new(0.0, 1.2, 6.0),
            target: Vec3::new(0.0, 1.2, 0.0),
            ..Default::default()
        };
        scene.tick_rate = 60.0;
        let (img, _, _) = run(&desc, &mut scene, 40, |s, f| s.tick = u64::from(f));
        img
    };
    let right = |img: &RgbaImage, rows: std::ops::Range<u32>| {
        rows.filter_map(|y| (0..320).rev().find(|&x| luma(px(img, x, y)) > 20.0))
            .max()
            .unwrap_or(0)
    };
    let skin = draw(false);
    let blown = draw(true);
    save("visual-garment", &blown);
    // The bottom half of the frame holds the skirt's hem.
    let (a, b) = (right(&skin, 90..170), right(&blown, 90..170));
    println!("hem's right edge: skin {a}, blown {b}");
    assert!(a > 0, "the skin is drawn");
    assert!(b >= a + 4, "the hem stayed on the skin ({a} -> {b})");
    // The top ring is pinned: it stays where the skin draws it.
    let (a, b) = (right(&skin, 20..40), right(&blown, 20..40));
    assert!(a.abs_diff(b) <= 3, "the pinned top moved ({a} -> {b})");
}

/// Blendshapes: a target that pushes the cylinder's top up draws it taller
/// at weight 1 and unchanged at weight 0 (also on a mesh without a skin).
#[test]
fn blendshapes_move_vertices_by_their_weights() {
    let desc = small();
    let (mesh, _) = fx::skinned_cylinder(0.3, 24, 24);
    let deltas: Vec<Vec3> = mesh
        .vertices
        .iter()
        .map(|v| Vec3::new(0.0, v.position[1] * 0.4, 0.0))
        .collect();
    let build = |w: f32| {
        let mut scene = RenderScene::new();
        let m = scene.add_material(Material {
            base_color: [0.8, 0.3, 0.5, 1.0],
            ..Default::default()
        });
        let h = scene.add_mesh(mesh.clone());
        assert!(scene.set_morph_targets(
            h,
            MorphTargets {
                positions: vec![deltas.clone()],
                normals: None,
            }
        ));
        let mut inst = SkinnedInstance::new(h, m, Mat4::IDENTITY, 0);
        inst.morph_weights[0] = w;
        scene.joint_palettes.push(JointPalette::default());
        scene.skinned_instances.push(inst);
        scene.environment.background = Background::Color(Vec3::ZERO);
        scene.camera = Camera {
            position: Vec3::new(0.0, 1.6, 6.0),
            target: Vec3::new(0.0, 1.6, 0.0),
            ..Default::default()
        };
        scene
    };
    let top = |img: &RgbaImage| {
        (0..180)
            .find(|&y| (0..320).any(|x| luma(px(img, x, y)) > 20.0))
            .expect("drawn")
    };
    let (rest, _) = frame(&desc, &build(0.0));
    let (tall, _) = frame(&desc, &build(1.0));
    println!("top row: rest {}, weight 1 {}", top(&rest), top(&tall));
    assert!(top(&tall) + 10 < top(&rest));
    // 64 targets (the old cap was eight): the same push split into 64
    // equal targets, every one at weight 1, draws the single target's
    // frame; and the last target alone, carrying the whole push, too.
    let many = |weights: &dyn Fn(usize) -> f32, last_whole: bool| {
        let mut scene = build(0.0);
        let positions = (0..64)
            .map(|t| {
                let k = if last_whole {
                    if t == 63 { 1.0 } else { 0.0 }
                } else {
                    1.0 / 64.0
                };
                deltas.iter().map(|d| *d * k).collect()
            })
            .collect();
        let h = scene.skinned_instances[0].mesh;
        assert!(scene.set_morph_targets(
            h,
            MorphTargets {
                positions,
                normals: None,
            }
        ));
        for t in 0..64 {
            scene.skinned_instances[0].morph_weights[t] = weights(t);
        }
        scene
    };
    let (split, _) = frame(&desc, &many(&|_| 1.0, false));
    save("blendshapes-64", &split);
    let d = differing(&split, &tall, 8.0);
    println!("64 split targets vs one: {d} pixels differ");
    assert!(d < 40, "64 targets summed differ from one ({d} pixels)");
    let (last, _) = frame(&desc, &many(&|t| if t == 63 { 1.0 } else { 0.0 }, true));
    let d = differing(&last, &tall, 8.0);
    println!("target 64 alone vs one: {d} pixels differ");
    assert!(
        d < 40,
        "the 64th target differs from the first ({d} pixels)"
    );
    // A bad target count is refused.
    let mut bad = RenderScene::new();
    let h = bad.add_mesh(mesh.clone());
    bad.set_morph_targets(
        h,
        MorphTargets {
            positions: vec![vec![Vec3::ZERO; 3]],
            normals: None,
        },
    );
    let mut r = Renderer::new_blocking(&desc).expect("renderer");
    assert!(r.render(&bad, &RenderTarget::Offscreen).is_err());
}
