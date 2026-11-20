//! The material style's made surface (`MaterialStyle::surface`, the IDL's
//! `MaterialStyle.surface`): off at zero, and where it is on a lit ball
//! takes its bumps (its normal bent in the mesh's own space) instead of
//! drawing as the plain material.

use glam::{Mat4, Vec3};
use og_render::{
    Camera, Material, MaterialStyle, MaterialSurface, RenderDesc, RenderScene, render_to_rgba,
    shapes,
};

fn ball(style: MaterialStyle) -> RenderScene {
    let mut scene = RenderScene::new();
    let ball = scene.add_mesh(shapes::sphere(48, 32));
    let clay = scene.add_material(Material {
        base_color: [0.7, 0.45, 0.3, 1.0],
        roughness: 0.8,
        ..Default::default()
    });
    scene.add_instance(ball, clay, Mat4::from_scale(Vec3::splat(2.0)));
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 0.4, 3.2),
        target: Vec3::ZERO,
        ..Default::default()
    };
    scene.look.stylise.material = style;
    scene
}

/// The mean absolute step between horizontal neighbours over the ball's
/// pixels (those that are not the background's): how much the shading
/// varies from one pixel to the next.
fn roughness(rgba: &[u8], width: usize, background: [u8; 4]) -> f64 {
    let px = |i: usize| &rgba[i * 4..i * 4 + 4];
    let (mut sum, mut n) = (0.0, 0usize);
    for i in 0..rgba.len() / 4 - 1 {
        if (i + 1) % width == 0 || px(i) == background || px(i + 1) == background {
            continue;
        }
        sum += (0..3)
            .map(|c| (px(i)[c] as f64 - px(i + 1)[c] as f64).abs())
            .sum::<f64>();
        n += 1;
    }
    sum / n.max(1) as f64
}

#[test]
#[ignore = "requires GPU"]
fn a_made_surface_bends_a_lit_ball_and_is_off_at_zero_strength() {
    let desc = RenderDesc {
        width: 192,
        height: 144,
        ..Default::default()
    };
    let plain = render_to_rgba(&desc, &ball(MaterialStyle::default())).expect(
        "a GPU adapter: a GPU test, run where there is one (OG_RENDERER=none takes it away)",
    );
    let background: [u8; 4] = plain.pixels[..4].try_into().unwrap();
    let clay = |strength: f32| MaterialStyle {
        surface: MaterialSurface::Clay,
        surface_strength: strength,
        ..Default::default()
    };
    // Off at zero: a surface at strength 0 is the plain frame, byte for byte.
    let zero = render_to_rgba(&desc, &ball(clay(0.0))).expect("renders");
    assert_eq!(plain.pixels, zero.pixels, "strength 0 changed the frame");
    // On: lumps and dents in the ball's own space vary the shading from
    // pixel to pixel far more than the smooth sphere does.
    let made = render_to_rgba(&desc, &ball(clay(1.0))).expect("renders");
    let (a, b) = (
        roughness(&plain.pixels, 192, background),
        roughness(&made.pixels, 192, background),
    );
    assert!(b > a * 1.5 + 0.5, "clay {b:.2} against plain {a:.2}");
}

/// A quad facing the camera whose normals lie in its own plane (what a
/// smooth-shaded mesh has at its silhouette, and a bevel's rim where the
/// faces turn away): the made surface's bump has no gradient frame there
/// (Mikkelsen's determinant is zero), so it keeps the shading normal and
/// lights the quad like the plain material instead of drawing it black.
#[test]
#[ignore = "requires GPU"]
fn a_made_surface_keeps_the_normal_where_it_lies_in_the_face() {
    let desc = RenderDesc {
        width: 96,
        height: 96,
        ..Default::default()
    };
    let scene = |surface: MaterialSurface| {
        let mut scene = RenderScene::new();
        let mut quad = shapes::plane_xy(2.0, 2.0);
        for v in &mut quad.vertices {
            v.normal = [0.0, 1.0, 0.0];
        }
        let quad = scene.add_mesh(quad);
        let paint = scene.add_material(Material {
            base_color: [0.4, 0.7, 0.3, 1.0],
            roughness: 0.6,
            ..Default::default()
        });
        scene.add_instance(quad, paint, Mat4::IDENTITY);
        scene.frame.camera = Camera {
            position: Vec3::new(0.0, 0.0, 2.0),
            target: Vec3::ZERO,
            ..Default::default()
        };
        scene.look.stylise.material = MaterialStyle {
            surface,
            surface_strength: 1.0,
            ..Default::default()
        };
        scene
    };
    let plain = render_to_rgba(&desc, &scene(MaterialSurface::None)).expect(
        "a GPU adapter: a GPU test, run where there is one (OG_RENDERER=none takes it away)",
    );
    let centre = |rgba: &[u8]| {
        let i = (48 * 96 + 48) * 4;
        [rgba[i], rgba[i + 1], rgba[i + 2]]
    };
    let lit = centre(&plain.pixels);
    assert!(
        lit.iter().any(|&c| c > 40),
        "the plain quad is lit ({lit:?})"
    );
    for surface in [
        MaterialSurface::Clay,
        MaterialSurface::Paper,
        MaterialSurface::Plastic,
    ] {
        let made = render_to_rgba(&desc, &scene(surface)).expect("renders");
        let c = centre(&made.pixels);
        let (a, b) = (
            lit.iter().map(|&v| v as i32).sum::<i32>(),
            c.iter().map(|&v| v as i32).sum::<i32>(),
        );
        assert!(
            b * 2 > a,
            "{surface:?} drew the quad {c:?} against plain {lit:?}"
        );
    }
}

/// Generated ground, blades and placed copies take the same made material
/// as models; setting its strength to zero leaves each renderer alone.
#[test]
#[ignore = "requires GPU"]
fn generated_surfaces_take_the_made_material_and_keep_the_zero_strength_frame() {
    use glam::Vec2;
    use og_render::{Grass, GrassDraw, ScatterDraw, Terrain, TerrainData, TerrainScatter};

    let desc = RenderDesc {
        width: 128,
        height: 96,
        ..Default::default()
    };
    for path in ["terrain", "grass", "scatter"] {
        let scene = |surface, strength| {
            let mut scene = RenderScene::new();
            let ground = scene.add_terrain(TerrainData::flat(Vec2::splat(8.0), 9, 9));
            let mut terrain = Terrain::new(ground);
            terrain.layers[0].color = Vec3::new(0.7, 0.45, 0.3);
            terrain.visible = path != "scatter";
            scene.world.terrains.push(terrain);
            scene.frame.debug = og_render::DebugView::Albedo;
            scene.frame.camera = Camera {
                position: Vec3::new(0.0, 1.4, 2.4),
                target: Vec3::ZERO,
                ..Default::default()
            };
            if path == "grass" {
                scene.world.grass.push(GrassDraw {
                    terrain: 0,
                    grass: Grass {
                        density: 30.0,
                        width: 0.12,
                        height: 0.7,
                        far_tint: false,
                        ..Default::default()
                    },
                });
            } else if path == "scatter" {
                let mesh = scene.add_mesh(shapes::sphere(12, 8));
                let material = scene.add_material(Material {
                    base_color: [0.7, 0.45, 0.3, 1.0],
                    ..Default::default()
                });
                scene.world.scatters.push(ScatterDraw {
                    id: 0,
                    terrain: 0,
                    mesh,
                    material,
                    lod_materials: [None; 2],
                    scatter: TerrainScatter {
                        density: 1.5,
                        scale: [0.3, 0.3],
                        sway: 0.0,
                        ..Default::default()
                    },
                    cast_shadows: false,
                    impostor: None,
                    impostor_distance: 0.0,
                    with: None,
                    lods: [None; 2],
                });
            }
            scene.look.stylise.material = MaterialStyle {
                surface,
                surface_strength: strength,
                surface_scale: 0.5,
                ..Default::default()
            };
            scene
        };
        let plain = render_to_rgba(&desc, &scene(MaterialSurface::None, 0.0)).expect("GPU renders");
        let zero = render_to_rgba(&desc, &scene(MaterialSurface::Clay, 0.0)).expect("GPU renders");
        assert_eq!(
            plain.pixels, zero.pixels,
            "{path}: zero strength changed the surface"
        );
        let made = render_to_rgba(&desc, &scene(MaterialSurface::Clay, 1.0)).expect("GPU renders");
        // Only blade-coloured pixels count for grass: a ground-only hook
        // must not pass by changing the terrain underneath the blades.
        let changed = plain
            .pixels
            .as_chunks::<4>()
            .0
            .iter()
            .zip(made.pixels.as_chunks::<4>().0)
            .any(|(a, b)| a != b && (path != "grass" || (a[1] > a[0] && a[1] > a[2])));
        assert!(changed, "{path}: ignored the made material");
        assert!(
            made.pixels
                .as_chunks::<4>()
                .0
                .iter()
                .any(|p| p[0] > 40 && p[1] > 20),
            "{path}: the made surface lost its light"
        );
    }
}

/// A smooth front face can have a shading normal just past the silhouette.
/// It must keep receiving light from above, not be flipped into the ground.
/// Turning the triangle over on a double-sided leaf must still flip its normal.
#[test]
#[ignore = "requires GPU"]
fn triangle_facing_not_the_shading_normal_selects_the_lit_side() {
    use og_render::meshlet::{Cluster, ClusterLevel, MeshClusters};
    use og_render::{MaterialShader, ShaderMode, Side, StandardMaterial};
    let desc = RenderDesc {
        width: 64,
        height: 64,
        ..Default::default()
    };
    let render = |path: &str, back: bool, side: Side, flat: bool, z: f32| {
        let mut scene = RenderScene::new();
        let mut quad = shapes::plane_xy(0.5, 0.5);
        for vertex in &mut quad.vertices {
            vertex.normal = Vec3::new(0.0, 1.0, z).normalize().to_array();
        }
        if back {
            for triangle in quad.indices.as_chunks_mut::<3>().0 {
                triangle.swap(1, 2);
            }
        }
        let clusters = MeshClusters {
            levels: vec![ClusterLevel {
                count: 6,
                clusters: 0..1,
                ..Default::default()
            }],
            clusters: vec![Cluster {
                triangles: 2,
                radius: 0.36,
                cone_cutoff: 1.0,
                ..Default::default()
            }],
            indices: quad.indices.clone(),
        };
        let mesh = scene.add_mesh(quad);
        if path == "meshlet" {
            assert!(scene.set_mesh_clusters(mesh, Some(clusters)));
        }
        let material = scene.add_standard_material(StandardMaterial {
            color: Vec3::splat(0.5),
            roughness: 1.0,
            clearcoat: if path == "physical" { 0.5 } else { 0.0 },
            side: side as u32,
            flatShading: u32::from(flat),
            ..Default::default()
        });
        if path == "custom" {
            let shader = scene.add_shader(MaterialShader {
                name: "smooth facing".into(),
                code: "fn fragment(f: FragmentInput, p: Params) -> SurfaceOutput { return SurfaceOutput(vec4<f32>(0.5, 0.5, 0.5, 1.0), f.normal, 0.0, 1.0, vec3<f32>(0.0)); }".into(),
                params: vec![],
                mode: ShaderMode::Full { lit: true },
            });
            assert!(scene.set_material_shader(material, Some(shader)));
        }
        for _ in 0..if path == "meshlet" { 65 } else { 1 } {
            scene.add_instance(mesh, material, Mat4::IDENTITY);
        }
        scene.frame.camera = Camera {
            position: Vec3::Z * 3.0,
            target: Vec3::ZERO,
            ..Default::default()
        };
        scene.lighting.light.direction_to_light = if flat {
            Vec3::Z
        } else if back {
            -Vec3::Y
        } else {
            Vec3::Y
        };
        scene.lighting.light.intensity = 3.0;
        scene.lighting.light.cast_shadows = false;
        scene.lighting.environment.intensity = 0.0;
        let image = render_to_rgba(&desc, &scene).expect("renderer");
        image.pixels[(32 * 64 + 32) * 4]
    };
    for path in ["standard", "physical", "custom", "meshlet"] {
        let control = render(path, false, Side::Front, false, 0.05);
        assert!(control > 100, "{path} lit control: {control}");
        for side in [Side::Front, Side::Double] {
            let past = render(path, false, side, false, -0.05);
            assert!(
                past.abs_diff(control) < 10,
                "{path} {side:?} silhouette: {past}, control {control}"
            );
        }
        let back = render(path, true, Side::Double, false, -0.05);
        assert!(
            back.abs_diff(control) < 10,
            "{path} back of leaf: {back}, control {control}"
        );
    }
    for back in [false, true] {
        let flat = render("standard", back, Side::Double, true, -0.05);
        assert!(flat > 100, "flat face back={back}: {flat}");
    }
}

/// A silhouette rim belongs to the underlying surface, not the wrinkles
/// of a normal map or a procedural material. With all other light off,
/// a front-facing plane stays dark; a grazing vertex normal keeps its rim.
#[test]
#[ignore = "requires GPU"]
fn rim_light_ignores_normal_maps_and_procedural_bumps() {
    use og_render::{ColorSpace, Mips, StandardMaterial, TextureData, Tonemap};
    let render = |physical: bool, mapped: bool, surface: MaterialSurface, grazing: bool| {
        let mut scene = RenderScene::new();
        let mut quad = shapes::plane_xy(2.0, 2.0);
        for vertex in &mut quad.vertices {
            vertex.normal = if grazing {
                Vec3::new(0.98, 0.0, 0.2).normalize().to_array()
            } else {
                Vec3::Z.to_array()
            };
            vertex.tangent = [1.0, 0.0, 0.0, 1.0];
        }
        let mesh = scene.add_mesh(quad);
        let normal = scene.add_texture(TextureData {
            width: 1,
            height: 1,
            rgba: vec![252, 128, 154, 255],
            color_space: ColorSpace::Linear,
            mips: Mips::Generate,
        });
        let material = scene.add_standard_material(StandardMaterial {
            color: Vec3::ZERO,
            roughness: 1.0,
            normalMap: if mapped { normal.slot() } else { 0 },
            clearcoat: if physical { 0.5 } else { 0.0 },
            rimColor: Vec3::ONE,
            rimIntensity: 1.0,
            rimPower: 2.0,
            ..Default::default()
        });
        scene.add_instance(mesh, material, Mat4::IDENTITY);
        scene.frame.camera = Camera {
            position: Vec3::Z * 3.0,
            target: Vec3::ZERO,
            ..Default::default()
        };
        scene.look.stylise.material = MaterialStyle {
            surface,
            surface_strength: 1.0,
            ..Default::default()
        };
        scene.look.post.tonemap = Tonemap::None;
        scene.lighting.light.intensity = 0.0;
        scene.lighting.environment.intensity = 0.0;
        let image = render_to_rgba(
            &RenderDesc {
                width: 64,
                height: 64,
                ..Default::default()
            },
            &scene,
        )
        .expect("GPU renders");
        (28..36)
            .flat_map(|y| (28..36).map(move |x| (y * 64 + x) * 4))
            .map(|i| image.pixels[i])
            .max()
            .expect("interior pixels")
    };
    for physical in [false, true] {
        for (mapped, surface) in [
            (false, MaterialSurface::None),
            (true, MaterialSurface::None),
            (false, MaterialSurface::Clay),
            (true, MaterialSurface::Clay),
        ] {
            let interior = render(physical, mapped, surface, false);
            assert!(
                interior < 8,
                "physical={physical}, mapped={mapped}, {surface:?}: interior rim {interior}"
            );
        }
        let edge = render(physical, false, MaterialSurface::None, true);
        assert!(edge > 128, "physical={physical}: grazing rim lost ({edge})");
    }
}
