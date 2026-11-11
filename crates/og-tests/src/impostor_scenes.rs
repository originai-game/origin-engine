//! R6's octahedral impostors (og-render's `impostor.rs`): a ball whose
//! +x half is red and −x half blue, drawn as its mesh and as its impostor,
//! and scattered over a terrain with its far copies as impostors.
//!
//! The atlas here is made analytically (each frame's texel the ball's
//! point seen along the frame's direction), so these scenes test the
//! renderer's cards apart from the cook's bake (og-cook's
//! `model/impostor.rs` has its own test).

use crate::fork_scenes::flags_terrain;
use glam::{Mat4, Vec3};
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
    let mut m = shapes::sphere(32, 20);
    for v in &mut m.vertices {
        let c = ball_color(Vec3::from(v.position));
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
    let (a, n) = ball_atlases(8, 128);
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

/// `impostors`: on a grey floor under a sun from the right, front and
/// above, the ball drawn as its mesh (left) and as its impostor (right),
/// each 2 m across and casting its shadow.
pub fn impostors_scene() -> RenderScene {
    impostors_scene_with(true)
}

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
        });
    } else {
        scene.add_instance(ball, white, at(BALL_CARD_AT));
    }
    scene.camera = Camera {
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
    });
    scene.camera = Camera {
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
        terrain: 0,
        mesh: ball,
        material: white,
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
    });
    scene.camera = Camera {
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
