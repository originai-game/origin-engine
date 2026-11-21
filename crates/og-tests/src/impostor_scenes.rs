//! R6's octahedral impostors (og-render's `impostor.rs`): a ball whose
//! +x half is red and −x half blue, drawn as its mesh and as its impostor,
//! and scattered over a terrain with its far copies as impostors.
//!
//! The atlas here is made analytically (each frame's texel the ball's
//! point seen along the frame's direction), so these scenes test the
//! renderer's cards apart from the cook's bake (og-cook's
//! `model/impostor.rs` has its own test).

use crate::fork_scenes::flags_terrain;
use glam::{Mat4, Vec2, Vec3};
use og_render::impostors::{frame_basis, frame_direction};
use og_render::{
    Background, Camera, ColorSpace, DirectionalLight, Environment, Impostor, ImpostorDraw, Mips,
    ProceduralSky, Projection, RenderScene, ScatterDraw, StandardMaterial, Terrain, TerrainLayer,
    TerrainScatter, TextureData, shapes,
};

/// The ball's radius in its mesh (`shapes::sphere`).
pub const BALL: f32 = 0.5;

/// The ball's colour at a point of its surface (its mesh's space).
fn ball_color(p: Vec3) -> [u8; 3] {
    if p.x >= 0.0 {
        [230, 40, 30]
    } else {
        [30, 60, 230]
    }
}

/// The ball's mesh: vertex colours red on its +x half, blue on its −x.
pub fn ball_mesh() -> og_render::MeshData {
    painted_sphere(32, 20, &ball_color)
}

/// A sphere of radius [`BALL`] whose vertices take `color` at their point.
fn painted_sphere(
    segments: u32,
    rings: u32,
    color: &dyn Fn(Vec3) -> [u8; 3],
) -> og_render::MeshData {
    let mut m = shapes::sphere(segments, rings);
    for v in &mut m.vertices {
        let c = color(Vec3::from(v.position));
        let lin = |x: u8| {
            let v = x as f32 / 255.0;
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        v.color = [lin(c[0]), lin(c[1]), lin(c[2]), 1.0];
    }
    m
}

/// The ball's impostor atlases (`grid` frames of `frame` px): albedo and
/// coverage, and normal and depth in each frame's basis.
pub fn ball_atlases(grid: u32, frame: u32) -> (TextureData, TextureData) {
    painted_atlases(grid, frame, &ball_color)
}

/// The impostor atlases of a sphere of radius [`BALL`] painted by `color`.
fn painted_atlases(
    grid: u32,
    frame: u32,
    ball_color: &dyn Fn(Vec3) -> [u8; 3],
) -> (TextureData, TextureData) {
    let size = grid * frame;
    let mut albedo = vec![0u8; (size * size * 4) as usize];
    let mut normal = vec![0u8; (size * size * 4) as usize];
    for j in 0..grid {
        for i in 0..grid {
            let d = frame_direction(i, j, grid);
            let (right, up) = frame_basis(d);
            for y in 0..frame {
                for x in 0..frame {
                    // 4 × 4 samples a texel, as the cook's bake antialiases
                    // its frames: coverage and colour averaged, a texel
                    // nothing covers taking its nearest rim's (alpha 0).
                    const SS: u32 = 4;
                    let (mut rgb, mut nrm, mut z, mut hits) = ([0.0f32; 3], Vec3::ZERO, 0.0, 0.0);
                    let (mut rim_c, mut rim_n) = ([0.0f32; 3], Vec3::ZERO);
                    for sy in 0..SS {
                        for sx in 0..SS {
                            let s = (x as f32 + (sx as f32 + 0.5) / SS as f32) / frame as f32 * 2.0
                                - 1.0;
                            let t = 1.0
                                - (y as f32 + (sy as f32 + 0.5) / SS as f32) / frame as f32 * 2.0;
                            let r2 = s * s + t * t;
                            let k = r2.sqrt().max(1e-6);
                            let (s, t, h) = if r2 < 1.0 {
                                (s, t, (1.0 - r2).sqrt())
                            } else {
                                (s / k, t / k, 0.0)
                            };
                            let c = ball_color(right * s + up * t + d * h).map(f32::from);
                            if r2 < 1.0 {
                                for (o, v) in rgb.iter_mut().zip(c) {
                                    *o += v;
                                }
                                nrm += Vec3::new(s, t, h);
                                z += h;
                                hits += 1.0;
                            } else {
                                rim_c = c;
                                rim_n = Vec3::new(s, t, 0.0);
                            }
                        }
                    }
                    let at = ((((j * frame + y) * size) + i * frame + x) * 4) as usize;
                    let a = (hits / (SS * SS) as f32 * 255.0).round() as u8;
                    let (c, n, z) = if hits > 0.0 {
                        (rgb.map(|v| v / hits), nrm.normalize(), z / hits)
                    } else {
                        (rim_c, rim_n, 0.0)
                    };
                    albedo[at..at + 4].copy_from_slice(&[c[0] as u8, c[1] as u8, c[2] as u8, a]);
                    let q = |v: f32| ((v * 0.5 + 0.5) * 255.0).round() as u8;
                    normal[at..at + 4].copy_from_slice(&[q(n.x), q(n.y), q(n.z), q(z)]);
                }
            }
        }
    }
    let tex = |rgba, color_space| TextureData {
        width: size,
        height: size,
        rgba,
        color_space,
        mips: Mips::Generate,
    };
    (
        tex(albedo, ColorSpace::Srgb),
        tex(normal, ColorSpace::Linear),
    )
}

/// The ball's impostor in `scene` (8 × 8 frames of 128 px).
pub fn ball_impostor(scene: &mut RenderScene) -> Impostor {
    ball_impostor_with(scene, None)
}

/// The ball's impostor with, given a translucency, the cook's material
/// atlas (roughness 0.85, that translucency, metallic 0 everywhere): a
/// leaf ball.
pub fn ball_impostor_with(scene: &mut RenderScene, translucency: Option<f32>) -> Impostor {
    painted_impostor(scene, ball_atlases(8, 128), translucency)
}

/// The impostor of a painted sphere from its atlases (8 × 8 frames), with
/// the material atlas when given a translucency.
fn painted_impostor(
    scene: &mut RenderScene,
    (a, n): (TextureData, TextureData),
    translucency: Option<f32>,
) -> Impostor {
    let (a, n) = (scene.add_texture(a), scene.add_texture(n));
    let atlas = translucency.map(|t| {
        let size = 8 * 128;
        let q = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        scene.add_texture(TextureData {
            width: size,
            height: size,
            rgba: [q(0.85), q(t), 0, 255].repeat((size * size) as usize),
            color_space: ColorSpace::Linear,
            mips: Mips::Generate,
        })
    });
    let material = scene.add_standard_material(StandardMaterial {
        map: a.slot(),
        normalMap: n.slot(),
        roughnessMap: atlas.map_or(0, |t| t.slot()),
        alphaTest: 0.5,
        side: 2,
        roughness: 0.85,
        ..Default::default()
    });
    Impostor {
        material,
        grid: 8,
        center: Vec3::ZERO,
        radius: BALL,
        material_atlas: atlas.is_some(),
        below: false,
        trim: og_render::CardTrim::FULL,
    }
}

fn light(scene: &mut RenderScene) {
    let sun = Vec3::new(0.35, 0.8, 0.45).normalize();
    scene.lighting.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 3.0,
        max_shadow_distance: 40.0,
        ..Default::default()
    };
    scene.lighting.environment = Environment {
        sky: ProceduralSky {
            sun_direction: sun,
            ..Default::default()
        },
        intensity: 0.8,
        background: Background::Color(Vec3::new(0.55, 0.7, 0.9)),
        ..Environment::default()
    };
}

/// Where `impostors` stands its two balls (2 m across): the mesh left,
/// the impostor right.
pub const BALL_MESH_AT: Vec3 = Vec3::new(-1.4, 1.2, 0.0);
pub const BALL_CARD_AT: Vec3 = Vec3::new(1.4, 1.2, 0.0);

/// `impostors` with the right ball its mesh too (`card` false): what the
/// card must look like where it stands. The camera sees the two balls from
/// opposite sides, so the card is compared with this, not with the left one.
pub fn impostors_scene_with(card: bool) -> RenderScene {
    let mut scene = RenderScene::new();
    light(&mut scene);
    let floor = scene.add_mesh(shapes::cuboid(Vec3::new(14.0, 0.2, 10.0)));
    let grey = scene.add_standard_material(StandardMaterial {
        color: Vec3::splat(0.5),
        roughness: 0.9,
        ..Default::default()
    });
    scene.add_instance(
        floor,
        grey,
        Mat4::from_translation(Vec3::new(0.0, -0.1, -1.0)),
    );
    let ball = scene.add_mesh(ball_mesh());
    let white = scene.add_standard_material(StandardMaterial {
        roughness: 0.85,
        ..Default::default()
    });
    let at =
        |p: Vec3| Mat4::from_scale_rotation_translation(Vec3::splat(2.0), Default::default(), p);
    scene.add_instance(ball, white, at(BALL_MESH_AT));
    if card {
        let impostor = ball_impostor(&mut scene);
        scene.objects.impostors.push(ImpostorDraw {
            impostor,
            transform: at(BALL_CARD_AT),
            cast_shadows: true,
            sway: 0.0,
            top: 0.0,
            reveal: 1.0,
            fade_out: false,
            foliage: None,
        });
    } else {
        scene.add_instance(ball, white, at(BALL_CARD_AT));
    }
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 2.4, 6.5),
        target: Vec3::new(0.0, 1.0, 0.0),
        projection: Projection::Perspective {
            fov_y_degrees: 40.0,
        },
        ..Default::default()
    };
    scene
}

/// `impostor-backlit`: the ball's card alone, 2 m across over the grey
/// floor, seen against a low sun behind it and to the left; with a
/// translucency, the cook's material atlas says it is leaves, which let
/// the sun through to the eye.
pub fn impostor_backlit_scene(translucency: Option<f32>) -> RenderScene {
    let mut scene = RenderScene::new();
    let sun = Vec3::new(-0.3, 0.35, -0.9).normalize();
    scene.lighting.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 3.0,
        max_shadow_distance: 40.0,
        ..Default::default()
    };
    scene.lighting.environment = Environment {
        sky: ProceduralSky {
            sun_direction: sun,
            ..Default::default()
        },
        intensity: 0.8,
        background: Background::Color(Vec3::new(0.55, 0.7, 0.9)),
        ..Environment::default()
    };
    let floor = scene.add_mesh(shapes::cuboid(Vec3::new(14.0, 0.2, 10.0)));
    let grey = scene.add_standard_material(StandardMaterial {
        color: Vec3::splat(0.5),
        roughness: 0.9,
        ..Default::default()
    });
    scene.add_instance(
        floor,
        grey,
        Mat4::from_translation(Vec3::new(0.0, -0.1, -1.0)),
    );
    let impostor = ball_impostor_with(&mut scene, translucency);
    scene.objects.impostors.push(ImpostorDraw {
        impostor,
        transform: Mat4::from_scale_rotation_translation(
            Vec3::splat(2.0),
            Default::default(),
            BACKLIT_AT,
        ),
        cast_shadows: true,
        sway: 0.0,
        top: 0.0,
        reveal: 1.0,
        fade_out: false,
        foliage: None,
    });
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 1.6, 5.0),
        target: BACKLIT_AT,
        projection: Projection::Perspective {
            fov_y_degrees: 40.0,
        },
        ..Default::default()
    };
    scene
}

/// Where `impostor-backlit` stands its card.
pub const BACKLIT_AT: Vec3 = Vec3::new(0.0, 1.2, 0.0);

/// `impostor-scatter`: balls 1.2 m across scattered over a 48 m flat
/// terrain, seen from its edge; past `impostor_distance` (0: never) the
/// copies draw as the ball's impostor.
pub fn impostor_scatter_scene(impostor_distance: f32) -> RenderScene {
    let mut scene = RenderScene::new();
    light(&mut scene);
    let h = scene.add_terrain(flags_terrain());
    scene.world.terrains.push(Terrain {
        layers: vec![TerrainLayer {
            color: Vec3::new(0.35, 0.5, 0.25),
            ..Default::default()
        }],
        fade_in: false,
        ..Terrain::new(h)
    });
    let ball = scene.add_mesh(ball_mesh());
    let white = scene.add_standard_material(StandardMaterial {
        roughness: 0.85,
        ..Default::default()
    });
    let impostor = ball_impostor(&mut scene);
    scene.world.scatters.push(ScatterDraw {
        id: 0,
        terrain: 0,
        mesh: ball,
        material: white,
        lod_materials: [None; 2],
        scatter: TerrainScatter {
            density: 0.08,
            distance: 80.0,
            fade: 0.0,
            scale: [2.4, 2.4],
            seed: 9,
            sway: 0.0,
            ..TerrainScatter::default()
        },
        cast_shadows: true,
        impostor: Some(impostor),
        impostor_distance,
        with: None,
        lods: [None; 2],
    });
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 6.0, 26.0),
        target: Vec3::new(0.0, 2.0, -6.0),
        far: 200.0,
        projection: Projection::Perspective {
            fov_y_degrees: 50.0,
        },
        ..Default::default()
    };
    scene
}

// ---------------------------------------------------------------------------
// The scenes: clipped box and cushion shrubs, drawn as their meshes near
// and as their cards farther off, as a game draws its shrubs.
// ---------------------------------------------------------------------------

/// A shrub's leaves at a point of its sphere: clumps of small leaves, dark
/// in the gaps between them and lighter on their tips, and with `bloom`
/// small flowers of that colour scattered over the clumps.
fn shrub_color(p: Vec3, bloom: Option<[u8; 3]>) -> [u8; 3] {
    use crate::fx_scenes::noise2;
    let n3 = |q: Vec3, seed: u32| {
        (noise2(q.x, q.y, seed)
            + noise2(q.y + 7.1, q.z, seed + 1)
            + noise2(q.z, q.x + 3.3, seed + 2))
            / 3.0
    };
    let leaves = n3(p * 46.0, 11);
    let clumps = n3(p * 9.0, 21);
    let shade = 0.45 + 0.75 * leaves * (0.6 + 0.6 * clumps);
    // Lower down the shrub is in its own shade.
    let shade = shade * (0.78 + 0.44 * (p.y + 0.5).clamp(0.0, 1.0));
    if let Some(c) = bloom
        && n3(p * 64.0 + 5.0, 31) > 0.76 - 0.06 * clumps
    {
        return c.map(|v| (v as f32 * (0.8 + 0.3 * leaves)).min(255.0) as u8);
    }
    [62.0 * shade, 104.0 * shade, 36.0 * shade].map(|v| v.min(255.0) as u8)
}

/// A shrub `lift` metres up from its origin (a sphere of radius [`BALL`]
/// on its root, sunk by the rest): its mesh, mapped with an equirectangular
/// texture of [`shrub_color`] (its leaves sharper than vertex colours draw
/// them), its material, and its impostor (leaves letting the low sun
/// through), whose atlases paint the same leaves at the same points.
fn shrub(
    scene: &mut RenderScene,
    bloom: Option<[u8; 3]>,
    lift: f32,
) -> (og_render::MeshHandle, og_render::MaterialHandle, Impostor) {
    let (w, h) = (1024u32, 512u32);
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let theta = (y as f32 + 0.5) / h as f32 * std::f32::consts::PI;
            let phi = (x as f32 + 0.5) / w as f32 * std::f32::consts::TAU;
            let n = Vec3::new(
                theta.sin() * phi.cos(),
                theta.cos(),
                theta.sin() * phi.sin(),
            );
            let c = shrub_color(n * BALL, bloom);
            rgba.extend_from_slice(&[c[0], c[1], c[2], 255]);
        }
    }
    let map = scene.add_texture(TextureData {
        width: w,
        height: h,
        rgba,
        color_space: ColorSpace::Srgb,
        mips: Mips::Generate,
    });
    let mut sphere = shapes::sphere(64, 40);
    for v in &mut sphere.vertices {
        v.position[1] += lift;
    }
    let mesh = scene.add_mesh(sphere);
    let material = scene.add_standard_material(StandardMaterial {
        map: map.slot(),
        roughness: 0.85,
        ..Default::default()
    });
    // The atlases' points are on the unit sphere; the leaves are painted
    // at the mesh's radius.
    let atlases = painted_atlases(8, 128, &|p: Vec3| shrub_color(p * BALL, bloom));
    let impostor = Impostor {
        center: Vec3::Y * lift,
        ..painted_impostor(scene, atlases, Some(0.35))
    };
    (mesh, material, impostor)
}

/// `impostors`: the walk of a formal garden, an avenue of clipped box
/// balls 2 m across on stone plinths running to a ruined arch, pines and
/// hills beyond under the fair afternoon's sky. The front pair are the
/// same shrub drawn two ways, its mesh on the left and its impostor on the
/// right, and read alike; every ball past them is its card, as a game
/// draws the far ones.
pub fn impostors_scene() -> RenderScene {
    use crate::kit;
    use crate::terrain_scenes::{afternoon, camera_at, fair_sky};
    let mut scene = RenderScene::new();
    let meadow = kit::surface_tinted(&mut scene, "grass", [0.72, 0.98, 0.56]);
    let lawn = scene.add_mesh(kit::slab(Vec2::splat(-160.0), Vec2::splat(160.0), 0.0, 3.5));
    scene.add_instance(lawn, meadow, Mat4::IDENTITY);
    let setts = kit::surface_tinted(&mut scene, "cobbles", [0.98, 0.92, 0.8]);
    let walk = scene.add_mesh(kit::slab(
        Vec2::new(-1.9, -90.0),
        Vec2::new(1.9, 12.0),
        0.02,
        2.5,
    ));
    scene.add_instance(walk, setts, Mat4::IDENTITY);
    let stone = kit::surface_tinted(&mut scene, "cobbles", [1.0, 0.93, 0.8]);
    let plinth = scene.add_mesh(kit::block(
        Vec3::new(-0.6, 0.0, -0.6),
        Vec3::new(0.6, 0.7, 0.6),
        1.2,
    ));
    let (ball, green, impostor) = shrub(&mut scene, None, 0.0);
    let at = |x: f32, z: f32| {
        Mat4::from_scale_rotation_translation(
            Vec3::splat(2.0),
            glam::Quat::from_rotation_y(0.7 * z),
            Vec3::new(x, 0.7 + 0.95, z),
        )
    };
    for k in 0..11 {
        let z = -3.0 - 7.5 * k as f32;
        for x in [-3.4f32, 3.4] {
            scene.add_instance(plinth, stone, Mat4::from_translation(Vec3::new(x, 0.0, z)));
            if k == 0 && x < 0.0 {
                scene.add_instance(ball, green, at(x, z));
            } else {
                scene.objects.impostors.push(ImpostorDraw {
                    impostor,
                    transform: at(x, z),
                    cast_shadows: true,
                    sway: 0.0,
                    top: 0.0,
                    reveal: 1.0,
                    fade_out: false,
                    foliage: None,
                });
            }
        }
    }
    // The walk's end: an arch between columns, a low wall either side.
    kit::prop(&mut scene, "arch").place(&mut scene, Vec3::new(0.0, 0.0, -86.0), 0.0, 8.0);
    let column = kit::prop(&mut scene, "column");
    column.place(&mut scene, Vec3::new(-7.0, 0.0, -85.0), 0.2, 6.5);
    column.place(&mut scene, Vec3::new(7.0, 0.0, -85.0), 1.4, 6.5);
    for (x0, x1) in [(-40.0f32, -3.6f32), (3.6, 40.0)] {
        let wall = scene.add_mesh(kit::block(
            Vec3::new(x0, 0.0, -86.6),
            Vec3::new(x1, 1.5, -85.8),
            2.5,
        ));
        scene.add_instance(wall, stone, Mat4::IDENTITY);
    }
    let pine = kit::prop(&mut scene, "pine");
    for k in 0..240i32 {
        let h = |i: i32| crate::fx_scenes::hash2(k, i, 0x7e1);
        let (x, z) = (-120.0 + 240.0 * h(1), -92.0 - 60.0 * h(2));
        pine.place(
            &mut scene,
            Vec3::new(x, -0.2, z),
            6.3 * h(3),
            11.0 + 9.0 * h(4),
        );
    }
    kit::hills(&mut scene, None);
    afternoon(&mut scene);
    scene.lighting.light.direction_to_light = Vec3::new(-0.5, 0.9, 0.45);
    scene.lighting.light.max_shadow_distance = 140.0;
    fair_sky(&mut scene, 9.4);
    scene.frame.tick = 0;
    scene.frame.camera = Camera {
        projection: Projection::Perspective {
            fov_y_degrees: 50.0,
        },
        ..camera_at(Vec3::new(0.0, 2.2, 8.5), Vec3::new(0.0, 1.6, -30.0))
    };
    scene
}

/// `impostor-scatter`'s heath at x, z: a slope rising away from the camera
/// in low swells (m).
fn heath_height(x: f32, z: f32) -> f32 {
    use crate::fx_scenes::noise2;
    -z * 0.07
        + 2.5 * noise2(x / 40.0 + 3.0, z / 40.0 + 1.0, 0x6e)
        + 0.8 * noise2(x / 13.0, z / 13.0, 0x6f)
}

/// `impostor-scatter`: a moor of cushion shrubs on a hillside, its
/// bushes scattered over a terrain of the kit's grass with og.Grass
/// between them, the ruin of a watchtower on the rise and hills beyond;
/// bushes past `impostor_distance` (the scene: 15 m) draw as their card,
/// cross-dithered over the last tenth, and read as the near ones do.
pub fn heath_scene(impostor_distance: f32) -> RenderScene {
    use crate::kit;
    use crate::terrain_scenes::{afternoon, camera_at, fair_sky};
    use og_render::{Grass, GrassDraw, TerrainData};
    let mut scene = RenderScene::new();
    let (n, size) = (257u32, 256.0f32);
    let cell = size / (n - 1) as f32;
    let heights: Vec<f32> = (0..n * n)
        .map(|i| {
            let (x, z) = (
                (i % n) as f32 * cell - size * 0.5,
                (i / n) as f32 * cell - size * 0.5,
            );
            heath_height(x, z)
        })
        .collect();
    let h = scene.add_terrain(TerrainData {
        heights: std::sync::Arc::from(heights),
        ..TerrainData::flat(Vec2::splat(size), n, n)
    });
    let (grass, grass_normal) = kit::layer_maps(&mut scene, "grass");
    scene.world.terrains.push(Terrain {
        layers: vec![TerrainLayer {
            albedo: Some(grass),
            normal: Some(grass_normal),
            color: Vec3::new(0.8, 0.98, 0.5),
            roughness: 0.95,
            tile: 3.5,
            anti_tiling: 0.6,
            macro_variation: 0.6,
            ..Default::default()
        }],
        fade_in: false,
        ..Terrain::new(h)
    });
    scene.world.grass.push(GrassDraw {
        terrain: 0,
        grass: Grass {
            height: 0.3,
            dryness: 0.35,
            cast_shadows: true,
            ..Default::default()
        },
    });
    let (bush, material, impostor) = shrub(&mut scene, Some([226, 200, 120]), 0.22);
    scene.world.scatters.push(ScatterDraw {
        id: 0,
        terrain: 0,
        mesh: bush,
        material,
        lod_materials: [None; 2],
        scatter: TerrainScatter {
            density: 0.045,
            distance: 120.0,
            fade: 0.1,
            scale: [0.9, 2.0],
            seed: 9,
            footprint: 1.4,
            noise_scale: 18.0,
            noise_threshold: 0.35,
            sway: 0.0,
            ..TerrainScatter::default()
        },
        cast_shadows: true,
        impostor: Some(impostor),
        impostor_distance,
        with: None,
        lods: [None; 2],
    });
    let ground = |x: f32, z: f32| Vec3::new(x, heath_height(x, z), z);
    kit::prop(&mut scene, "watchtower").place(
        &mut scene,
        ground(14.0, -62.0) - Vec3::Y * 0.5,
        0.9,
        18.0,
    );
    let broken = kit::prop(&mut scene, "column_broken");
    broken.place(&mut scene, ground(4.0, -54.0) - Vec3::Y * 0.2, 1.1, 2.6);
    broken.place(&mut scene, ground(22.0, -50.0) - Vec3::Y * 0.2, 2.9, 1.9);
    let boulder = kit::prop(&mut scene, "boulder");
    for (x, z, size) in [
        (-6.0f32, -14.0f32, 1.6f32),
        (9.0, -24.0, 2.4),
        (-18.0, -40.0, 3.0),
    ] {
        boulder.place(
            &mut scene,
            ground(x, z) - Vec3::Y * 0.3 * size,
            x * 0.7,
            size,
        );
    }
    kit::hills(&mut scene, None);
    afternoon(&mut scene);
    scene.lighting.light.direction_to_light = Vec3::new(0.6, 0.45, -0.3);
    scene.lighting.light.max_shadow_distance = 160.0;
    fair_sky(&mut scene, 7.8);
    scene.frame.tick = 0;
    scene.frame.camera = Camera {
        far: 4000.0,
        ..camera_at(
            ground(-4.0, 22.0) + Vec3::Y * 3.2,
            ground(6.0, -40.0) + Vec3::Y * 5.0,
        )
    };
    scene
}
