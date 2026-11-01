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
