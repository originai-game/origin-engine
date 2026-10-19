//! The terrain and water scenes of phase 2 (`crates/og-render/README.md`
//! §Terrain, §Water): a 2 km island (hills, a ridge of mountains to the
//! north, a lake, beaches into the sea) drawn as a clipmap with four
//! layers (grass, rock on slopes, sand by the water, snow on the peaks)
//! seen from three distances, grass scattered over its meadow, and the
//! water's planar reflection, its shadows and the view from under the
//! lake. Everything is generated here.

use crate::fx_scenes::{noise2, smooth, tiling_fbm};
use crate::scenes::texture;
use glam::{Mat4, Vec2, Vec3};
use og_render::{
    Background, Camera, ColorSpace, DirectionalLight, Environment, Exposure, Fog, Material,
    MeshData, ProceduralSky, Projection, RenderScene, Terrain, TerrainControl, TerrainData,
    TerrainLayer, TerrainLod, TerrainScatter, Vertex, Water, shapes,
};
use std::sync::{Arc, OnceLock};

/// The island's extent (m) and samples per side (a 2 m grid).
pub const ISLAND_SIZE: f32 = 2048.0;
pub const ISLAND_SAMPLES: u32 = 1025;
/// Where the lake is (its centre's xz) and its surface's height.
pub const LAKE: Vec2 = Vec2::new(180.0, 260.0);
pub const LAKE_LEVEL: f32 = 6.0;

fn fbm(x: f32, y: f32, octaves: u32, seed: u32) -> f32 {
    let (mut v, mut amp, mut f) = (0.0, 0.5, 1.0);
    for o in 0..octaves {
        v += (noise2(x * f, y * f, seed + o) - 0.5) * amp;
        f *= 2.03;
        amp *= 0.5;
    }
    v
}

fn ridged(x: f32, y: f32, octaves: u32, seed: u32) -> f32 {
    let (mut v, mut amp, mut f) = (0.0, 0.5, 1.0);
    for o in 0..octaves {
        let n = 1.0 - (noise2(x * f, y * f, seed + o) * 2.0 - 1.0).abs();
        v += n * n * amp;
        f *= 2.1;
        amp *= 0.5;
    }
    v
}

/// The island's height at world xz (the terrain is centred on the origin).
pub fn island_height(x: f32, z: f32) -> f32 {
    let r = Vec2::new(x, z * 1.15).length() / (ISLAND_SIZE * 0.5);
    // The coast: land inside about 0.78 of the half size, sea floor beyond.
    let land = 1.0 - smooth(0.62, 0.92, r);
    let hills = fbm(x / 420.0, z / 420.0, 6, 11) * 70.0 + 22.0;
    // A ridge of mountains to the north (-z), highest towards the middle.
    let north = smooth(-60.0, -520.0, z) * (1.0 - smooth(380.0, 820.0, x.abs()));
    let peaks = ridged(x / 360.0, z / 360.0, 5, 23) * 330.0 * north;
    let mut h = (hills + peaks) * land - 30.0 * (1.0 - land);
    // The lake: a smooth basin below its level.
    let d = (Vec2::new(x, z) - LAKE).length() / 170.0;
    let basin = 1.0 - smooth(0.55, 1.15, d);
    h = h * (1.0 - basin) + (LAKE_LEVEL - 9.0 + d * d * 6.0) * basin;
    h
}

/// The island's height field (made once per process).
pub fn island() -> TerrainData {
    static DATA: OnceLock<TerrainData> = OnceLock::new();
    DATA.get_or_init(|| {
        let n = ISLAND_SAMPLES;
        let cell = ISLAND_SIZE / (n - 1) as f32;
        let mut heights = Vec::with_capacity((n * n) as usize);
        for r in 0..n {
            for c in 0..n {
                let x = -ISLAND_SIZE * 0.5 + c as f32 * cell;
                let z = -ISLAND_SIZE * 0.5 + r as f32 * cell;
                heights.push(island_height(x, z));
            }
        }
        // The control map, painted like a splat map: sand (blue) on the
        // beaches, the lake's bed and a track from the lake towards the
        // south coast, grass (red) elsewhere; rock (green) and snow
        // (alpha) everywhere, left to their slope and altitude rules.
        let m = 256u32;
        let mut rgba = Vec::with_capacity((m * m * 4) as usize);
        for y in 0..m {
            for x in 0..m {
                let wx = -ISLAND_SIZE * 0.5 + (x as f32 + 0.5) / m as f32 * ISLAND_SIZE;
                let wz = -ISLAND_SIZE * 0.5 + (y as f32 + 0.5) / m as f32 * ISLAND_SIZE;
                let track_x = LAKE.x - 40.0 + (wz - LAKE.y) * 0.25 + (wz * 0.01).sin() * 30.0;
                let track = if wz > LAKE.y {
                    1.0 - smooth(6.0, 12.0, (wx - track_x).abs())
                } else {
                    0.0
                };
                let beach = 1.0 - smooth(LAKE_LEVEL, LAKE_LEVEL + 2.5, island_height(wx, wz));
                let sand = track.max(beach);
                let b = (sand * 255.0) as u8;
                rgba.extend_from_slice(&[255 - b, 255, b, 255]);
            }
        }
        TerrainData {
            size: Vec2::splat(ISLAND_SIZE),
            columns: n,
            rows: n,
            heights: Arc::from(heights),
            control: Some(TerrainControl {
                width: m,
                height: m,
                rgba: Arc::from(rgba),
            }),
        }
    })
    .clone()
}

/// The island's four layers, their textures made here.
pub fn island_layers(scene: &mut RenderScene) -> Vec<TerrainLayer> {
    let grass = scene.add_texture(texture(256, ColorSpace::Srgb, |u, v| {
        let blades = tiling_fbm(u, v, 32, 5);
        let patches = tiling_fbm(u, v, 4, 7);
        let g = 0.65 + 0.5 * blades;
        let c = [
            (58.0 + 30.0 * patches) * g,
            (92.0 + 26.0 * patches) * g,
            (36.0 + 8.0 * patches) * g,
        ];
        // Alpha: the layer's height (tufts stand out over sand).
        [
            c[0] as u8,
            c[1] as u8,
            c[2] as u8,
            (120.0 + 120.0 * blades) as u8,
        ]
    }));
    let rock = scene.add_texture(texture(256, ColorSpace::Srgb, |u, v| {
        let strata = (v * 40.0 + tiling_fbm(u, v, 8, 3) * 6.0).sin() * 0.5 + 0.5;
        let grain = tiling_fbm(u, v, 32, 9);
        let g = 0.55 + 0.35 * grain + 0.15 * strata;
        [
            (118.0 * g) as u8,
            (110.0 * g) as u8,
            (102.0 * g) as u8,
            (90.0 + 160.0 * grain) as u8,
        ]
    }));
    let rock_normal = scene.add_texture(texture(256, ColorSpace::Linear, |u, v| {
        let e = 1.0 / 256.0;
        let h = |u: f32, v: f32| tiling_fbm(u, v, 16, 21) + 0.4 * tiling_fbm(u, v, 64, 22);
        let dx = (h(u + e, v) - h(u - e, v)) * 18.0;
        let dy = (h(u, v + e) - h(u, v - e)) * 18.0;
        let n = Vec3::new(-dx, -dy, 1.0).normalize();
        [
            ((n.x * 0.5 + 0.5) * 255.0) as u8,
            ((n.y * 0.5 + 0.5) * 255.0) as u8,
            ((n.z * 0.5 + 0.5) * 255.0) as u8,
            255,
        ]
    }));
    let sand = scene.add_texture(texture(256, ColorSpace::Srgb, |u, v| {
        let ripples = ((u + tiling_fbm(u, v, 4, 31) * 0.2) * 48.0 * std::f32::consts::TAU).sin();
        let g = 0.88 + 0.06 * ripples + 0.1 * tiling_fbm(u, v, 64, 33);
        [(196.0 * g) as u8, (178.0 * g) as u8, (136.0 * g) as u8, 60]
    }));
    let snow = scene.add_texture(texture(256, ColorSpace::Srgb, |u, v| {
        let g = 0.92 + 0.08 * tiling_fbm(u, v, 16, 41);
        [(236.0 * g) as u8, (240.0 * g) as u8, (248.0 * g) as u8, 200]
    }));
    vec![
        TerrainLayer {
            albedo: Some(grass),
            color: Vec3::ONE,
            roughness: 0.95,
            tile: 6.0,
            slope: [0.0, 34.0],
            slope_fade: 6.0,
            altitude: [-1.0e6, 185.0],
            altitude_fade: 25.0,
            ..Default::default()
        },
        TerrainLayer {
            albedo: Some(rock),
            normal: Some(rock_normal),
            color: Vec3::ONE,
            roughness: 0.85,
            tile: 14.0,
            triplanar: true,
            slope: [30.0, 90.0],
            slope_fade: 6.0,
            ..Default::default()
        },
        TerrainLayer {
            albedo: Some(sand),
            color: Vec3::ONE,
            roughness: 0.9,
            tile: 5.0,
            slope: [0.0, 32.0],
            slope_fade: 6.0,
            ..Default::default()
        },
        TerrainLayer {
            albedo: Some(snow),
            color: Vec3::ONE,
            roughness: 0.6,
            tile: 8.0,
            slope: [0.0, 40.0],
            slope_fade: 8.0,
            altitude: [175.0, 1.0e6],
            altitude_fade: 25.0,
            ..Default::default()
        },
    ]
}

/// The island with its layers, the afternoon sun and sky, and a few
/// boulders on the meadow (shadow casters at the near camera).
pub fn island_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    let data = island();
    let handle = scene.add_terrain(data);
    let layers = island_layers(&mut scene);
    scene.terrains.push(Terrain {
        layers,
        height_blend: 0.6,
        ..Terrain::new(handle)
    });
    let boulder = scene.add_mesh(shapes::sphere(18, 12));
    let stone = scene.add_material(Material {
        base_color: [0.26, 0.25, 0.24, 1.0],
        roughness: 0.9,
        ..Default::default()
    });
    for (i, (x, z)) in [
        (-96.0f32, 120.0f32),
        (-104.0, 128.0),
        (-88.0, 133.0),
        (-110.0, 112.0),
    ]
    .iter()
    .enumerate()
    {
        let r = 1.2 + i as f32 * 0.5;
        let y = island_height(*x, *z);
        scene.add_instance(
            boulder,
            stone,
            Mat4::from_translation(Vec3::new(*x, y + r * 0.4, *z))
                * Mat4::from_scale(Vec3::new(r * 2.2, r * 1.6, r * 2.0)),
        );
    }
    let sun = Vec3::new(0.62, 0.45, 0.4);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        color: Vec3::new(1.0, 0.93, 0.82),
        intensity: 3.2,
        max_shadow_distance: 400.0,
        ..Default::default()
    };
    scene.environment = Environment {
        sky: ProceduralSky {
            sun_direction: sun,
            sun_color: Vec3::new(1.0, 0.9, 0.76),
            look: None,
        },
        intensity: 0.7,
        background: Background::Sky,
    };
    scene.fog = Some(Fog {
        color: Vec3::new(0.62, 0.7, 0.8),
        density: 0.00018,
        start: 150.0,
        height_density: 0.0006,
        height: 0.0,
        falloff: 0.01,
        sky: 0.2,
        ..Default::default()
    });
    scene
}

fn camera_at(position: Vec3, target: Vec3) -> Camera {
    Camera {
        position,
        target,
        near: 0.3,
        far: 12000.0,
        projection: Projection::Perspective {
            fov_y_degrees: 55.0,
        },
        exposure: Exposure::linear(0.9),
        ..Default::default()
    }
}

/// The island from three distances: on the meadow at eye height looking
/// at the boulders and the mountains (`near`), from 150 m over a valley
/// (`mid`), and from 700 m out at sea across the whole island (`far`).
pub fn island_view(distance: &str) -> RenderScene {
    let mut scene = island_scene();
    let ground = |x: f32, z: f32| island_height(x, z);
    scene.camera = match distance {
        "near" => {
            let (x, z) = (-80.0, 150.0);
            camera_at(
                Vec3::new(x, ground(x, z) + 1.8, z),
                Vec3::new(-110.0, ground(-110.0, 60.0) + 10.0, 60.0),
            )
        }
        "mid" => {
            let (x, z) = (-260.0, 260.0);
            camera_at(
                Vec3::new(x, ground(x, z) + 150.0, z),
                Vec3::new(0.0, 60.0, -220.0),
            )
        }
        _ => camera_at(
            Vec3::new(240.0, 700.0, 1350.0),
            Vec3::new(0.0, 40.0, -150.0),
        ),
    };
    // The sea around the island (flat, the sky's cubemap reflected).
    scene.water.push(Water {
        transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        size: Vec2::splat(24000.0),
        segments: 1,
        depth_fade: 6.0,
        foam_width: 0.6,
        wave_scale: 0.08,
        ..Default::default()
    });
    scene
}

/// The lake's surface (its own water, above the sea's level).
fn lake(scene: &mut RenderScene, w: Water) {
    scene.water.push(Water {
        transform: Mat4::from_translation(Vec3::new(LAKE.x, LAKE_LEVEL, LAKE.y)),
        size: Vec2::splat(360.0),
        segments: 1,
        shallow: Vec3::new(0.5, 0.78, 0.74),
        deep: Vec3::new(0.02, 0.08, 0.1),
        depth_fade: 3.0,
        wave_scale: 0.35,
        wave_strength: 0.25,
        ..w
    });
}

/// The lake from its south shore: the mountains and the sky mirrored by
/// the planar reflection (half resolution), posts at the water's edge.
pub fn water_planar_scene() -> RenderScene {
    let mut scene = island_scene();
    lake(
        &mut scene,
        Water {
            planar_reflection: 0.5,
            shadows: 1.0,
            ..Default::default()
        },
    );
    posts(&mut scene);
    let eye = Vec3::new(LAKE.x - 20.0, LAKE_LEVEL + 3.0, LAKE.y + 150.0);
    scene.camera = camera_at(
        eye,
        Vec3::new(LAKE.x - 40.0, LAKE_LEVEL + 40.0, LAKE.y - 300.0),
    );
    scene
}

/// A row of tall posts standing in the lake's shallows.
fn posts(scene: &mut RenderScene) {
    let post = scene.add_mesh(shapes::cuboid(Vec3::new(0.6, 9.0, 0.6)));
    let wood = scene.add_material(Material {
        base_color: [0.4, 0.27, 0.15, 1.0],
        roughness: 0.75,
        ..Default::default()
    });
    for i in 0..6 {
        let x = LAKE.x - 60.0 + i as f32 * 14.0;
        let z = LAKE.y + 110.0 - i as f32 * 6.0;
        scene.add_instance(
            post,
            wood,
            Mat4::from_translation(Vec3::new(x, LAKE_LEVEL + 1.5, z)),
        );
    }
}

/// The lake under a low sun from the west: the posts' and the shore's
/// shadows across the surface (the screen-space and cubemap reflections).
pub fn water_shadows_scene() -> RenderScene {
    let mut scene = island_scene();
    let sun = Vec3::new(-0.8, 0.3, 0.35);
    scene.light.direction_to_light = sun;
    scene.environment.sky.sun_direction = sun;
    lake(
        &mut scene,
        Water {
            shadows: 1.0,
            deep: Vec3::new(0.09, 0.22, 0.22),
            opacity: 1.0,
            reflectivity: 0.5,
            wave_strength: 0.12,
            ..Default::default()
        },
    );
    posts(&mut scene);
    // A pier on stilts: its deck's shadow falls on the water.
    let deck = scene.add_mesh(shapes::cuboid(Vec3::new(3.0, 0.3, 46.0)));
    let wood = scene.add_material(Material {
        base_color: [0.45, 0.32, 0.2, 1.0],
        roughness: 0.8,
        ..Default::default()
    });
    scene.add_instance(
        deck,
        wood,
        Mat4::from_translation(Vec3::new(LAKE.x - 8.0, LAKE_LEVEL + 2.2, LAKE.y + 100.0)),
    );
    let eye = Vec3::new(LAKE.x - 30.0, LAKE_LEVEL + 22.0, LAKE.y + 140.0);
    scene.camera = camera_at(eye, Vec3::new(LAKE.x - 4.0, LAKE_LEVEL, LAKE.y + 92.0));
    scene
}

/// Under the lake, looking up and along its bed: the water's fog, the
/// caustics on the bed, the surface seen from below (the sky through
/// Snell's window, the water's colour reflected outside it).
pub fn water_underwater_scene() -> RenderScene {
    let mut scene = island_scene();
    lake(
        &mut scene,
        Water {
            caustics: 1.0,
            caustics_scale: 0.7,
            underwater_density: 0.06,
            ..Default::default()
        },
    );
    posts(&mut scene);
    let eye = Vec3::new(LAKE.x - 40.0, LAKE_LEVEL - 3.0, LAKE.y + 70.0);
    scene.camera = camera_at(
        eye,
        Vec3::new(LAKE.x - 48.0, LAKE_LEVEL + 6.0, LAKE.y + 90.0),
    );
    scene.tick = 90;
    scene
}

/// A tuft of five grass blades, 0.5 m tall: each a tapered quad drawn on
/// both sides, darker at the root, its normals leaning up so the tuft
/// shades like the meadow under it.
pub fn grass_tuft() -> MeshData {
    let mut m = MeshData::default();
    for b in 0..5u32 {
        let yaw = b as f32 * 2.4 + 0.3;
        let (s, c) = yaw.sin_cos();
        let lean = Vec3::new(c, 0.0, s) * (0.08 + 0.03 * b as f32);
        let side = Vec3::new(-s, 0.0, c) * 0.035;
        let root = Vec3::new(c, 0.0, s) * 0.04;
        let tall = 0.38 + 0.05 * (b % 3) as f32;
        let tip = root + lean + Vec3::Y * tall;
        let n = (Vec3::Y * 2.0 + Vec3::new(c, 0.0, s)).normalize();
        for face in [1.0f32, -1.0] {
            let at = m.vertices.len() as u32;
            for (p, u, v, shade) in [
                (root - side, 0.0, 0.0, 0.45),
                (root + side, 1.0, 0.0, 0.45),
                (tip, 0.5, 1.0, 1.0),
            ] {
                m.vertices.push(Vertex {
                    position: p.into(),
                    normal: (n * face.max(0.6)).normalize().into(),
                    uv: [u, v],
                    color: [shade, shade, shade, 1.0],
                });
            }
            if face > 0.0 {
                m.indices.extend([at, at + 2, at + 1]);
            } else {
                m.indices.extend([at, at + 1, at + 2]);
            }
        }
    }
    m
}

/// The near view with grass on the meadow (the grass layer, 4 tufts a
/// square metre within 60 m, shrinking away over its last third), placed
/// by `Terrain::scatter`.
pub fn grass_scene() -> RenderScene {
    let mut scene = island_view("near");
    let tuft = scene.add_mesh(grass_tuft());
    let grass = scene.add_material(Material {
        base_color: [0.2, 0.33, 0.08, 1.0],
        roughness: 0.85,
        ..Default::default()
    });
    let data = island();
    let terrain = scene.terrains[0].clone();
    let eye = scene.camera.position;
    for (_, m) in terrain.scatter(&data, &grass_scatter(), eye) {
        scene.add_instance(tuft, grass, m);
    }
    scene
}

/// The grass scene's grass: layer 0 (grass).
pub fn grass_scatter() -> TerrainScatter {
    TerrainScatter {
        layer: 0,
        density: 4.0,
        distance: 60.0,
        fade: 0.35,
        scale: [0.8, 1.4],
        align: 0.3,
        seed: 3,
    }
}

/// Stones on layer 1 (rock), leaning with the slope (a test's scatter).
pub fn stone_scatter() -> TerrainScatter {
    TerrainScatter {
        layer: 1,
        density: 0.4,
        distance: 120.0,
        fade: 0.2,
        scale: [0.6, 2.2],
        align: 1.0,
        seed: 9,
    }
}

/// The island's terrain at the lod a scene asks for (a test helper).
pub fn with_lod(mut scene: RenderScene, lod: TerrainLod) -> RenderScene {
    for t in &mut scene.terrains {
        t.lod = lod;
    }
    scene
}
