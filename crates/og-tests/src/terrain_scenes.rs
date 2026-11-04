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
    MeshData, ProceduralSky, Projection, RenderScene, ScatterDraw, Terrain, TerrainControl,
    TerrainData, TerrainLayer, TerrainLod, TerrainScatter, Vertex, Water, shapes,
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
            holes: None,
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
            anti_tiling: 1.0,
            macro_variation: 0.6,
            height_contrast: 0.4,
            ..Default::default()
        },
        TerrainLayer {
            albedo: Some(rock),
            normal: Some(rock_normal),
            color: Vec3::ONE,
            roughness: 0.85,
            tile: 14.0,
            triplanar: true,
            project_slope: 45.0,
            anti_tiling: 0.5,
            macro_variation: 0.4,
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
            anti_tiling: 1.0,
            macro_variation: 0.4,
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
            height_contrast: 0.6,
            anti_tiling: 1.0,
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
    afternoon(&mut scene);
    scene
}

/// The island's afternoon: the sun, the sky lighting the scene and a
/// light haze.
pub fn afternoon(scene: &mut RenderScene) {
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
        ..Environment::default()
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
}

pub fn camera_at(position: Vec3, target: Vec3) -> Camera {
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
                    tangent: [0.0; 4],
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
/// square metre within 60 m, fading out over its last third), placed and
/// drawn by the renderer (`RenderScene::scatters`).
pub fn grass_scene() -> RenderScene {
    let mut scene = island_view("near");
    let tuft = scene.add_mesh(grass_tuft());
    let grass = scene.add_material(Material {
        base_color: [0.2, 0.33, 0.08, 1.0],
        roughness: 0.85,
        ..Default::default()
    });
    scene.scatters.push(ScatterDraw {
        terrain: 0,
        mesh: tuft,
        material: grass,
        scatter: grass_scatter(),
        cast_shadows: true,
        impostor: None,
        impostor_distance: 0.0,
    });
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
        sway: 1.0,
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
        sway: 0.0,
    }
}

/// The island's terrain at the lod a scene asks for (a test helper).
pub fn with_lod(mut scene: RenderScene, lod: TerrainLod) -> RenderScene {
    for t in &mut scene.terrains {
        t.lod = lod;
    }
    scene
}

/// `scene` with every terrain's layer textures moved into two texture
/// arrays (`Terrain::layer_arrays`): the albedos as layers of one, the
/// normal maps of another, each layer naming its layers. The same images,
/// so the same frame within mip filtering (an array's mips are box-filtered
/// on the CPU, a texture's on the GPU).
pub fn with_layer_arrays(mut scene: RenderScene) -> RenderScene {
    use og_render::{TerrainLayerArrays, TextureArrayData, TextureSource};
    for ti in 0..scene.terrains.len() {
        let layers = scene.terrains[ti].layers.clone();
        let mut albedos = Vec::new();
        let mut normals = Vec::new();
        let mut picked = Vec::new();
        for l in &layers {
            let take = |h: Option<og_render::TextureHandle>, list: &mut Vec<TextureSource>| {
                h.map(|h| {
                    list.push(TextureSource::Rgba(
                        scene.texture(h).expect("a texture").clone(),
                    ));
                    list.len() as u32 - 1
                })
            };
            picked.push((take(l.albedo, &mut albedos), take(l.normal, &mut normals)));
        }
        let mut array = |list: Vec<TextureSource>| {
            (!list.is_empty()).then(|| {
                scene
                    .add_texture_array(TextureArrayData { layers: list })
                    .expect("same-sized layers")
            })
        };
        let arrays = TerrainLayerArrays {
            albedo: array(albedos),
            normal: array(normals),
        };
        let t = &mut scene.terrains[ti];
        t.layer_arrays = arrays;
        for (l, (albedo, normal)) in t.layers.iter_mut().zip(picked) {
            l.albedo_layer = albedo;
            l.normal_layer = normal;
        }
    }
    scene
}

/// The pit cut out of the island's meadow in `terrain-holes`: its world
/// x and z ranges (whole 2 m cells), and the cave floor's height under it.
pub const PIT_X: [f32; 2] = [-76.0, -60.0];
pub const PIT_Z: [f32; 2] = [110.0, 122.0];

/// The pit's centre on the surface it was cut from.
pub fn pit_center() -> Vec3 {
    let (x, z) = ((PIT_X[0] + PIT_X[1]) * 0.5, (PIT_Z[0] + PIT_Z[1]) * 0.5);
    Vec3::new(x, island_height(x, z), z)
}

/// The cave floor under the pit: 8 m below the pit's lowest corner.
pub fn pit_floor() -> f32 {
    [PIT_X[0], PIT_X[1]]
        .iter()
        .flat_map(|&x| [PIT_Z[0], PIT_Z[1]].map(|z| island_height(x, z)))
        .fold(f32::MAX, f32::min)
        - 8.0
}

/// The island with a 16 × 12 m pit cut out of its meadow
/// ([`TerrainData::holes`]) over an orange cave floor, the sun nearly
/// overhead, seen from 30 m above and behind the pit: the floor shows
/// through the hole with the sunlight that falls through it (the hole
/// casts no shadow), the meadow round it unbroken.
pub fn holes_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    let mut data = island();
    let (cx, cz) = data.cells();
    let cell = ISLAND_SIZE / (ISLAND_SAMPLES - 1) as f32;
    let mut holes = vec![0u8; (cx * cz) as usize];
    let cells = |r: [f32; 2]| {
        let a = ((r[0] + ISLAND_SIZE * 0.5) / cell) as u32;
        let b = ((r[1] + ISLAND_SIZE * 0.5) / cell) as u32;
        a..b
    };
    for r in cells(PIT_Z) {
        for c in cells(PIT_X) {
            holes[(r * cx + c) as usize] = 1;
        }
    }
    data.holes = Some(Arc::from(holes));
    let handle = scene.add_terrain(data);
    let layers = island_layers(&mut scene);
    scene.terrains.push(Terrain {
        layers,
        height_blend: 0.6,
        ..Terrain::new(handle)
    });
    let floor = scene.add_mesh(shapes::cuboid(Vec3::new(60.0, 1.0, 60.0)));
    let rock = scene.add_material(Material {
        // A deep red-orange: it stays saturated after the tonemap under the overhead sun.
        base_color: [0.5, 0.08, 0.01, 1.0],
        roughness: 0.8,
        ..Default::default()
    });
    let c = pit_center();
    scene.add_instance(
        floor,
        rock,
        Mat4::from_translation(Vec3::new(c.x, pit_floor() - 0.5, c.z)),
    );
    afternoon(&mut scene);
    let sun = Vec3::new(0.2, 0.95, 0.15).normalize();
    scene.light.direction_to_light = sun;
    scene.environment.sky.sun_direction = sun;
    scene.camera = camera_at(c + Vec3::new(0.0, 30.0, 24.0), c);
    scene
}

/// Where `terrain-layers` paints its four extra layers (world x, z), each
/// a disc of [`LAYER_DISC`] metres (seven control texels) clear of the
/// boulders, their shadows and each other, and their colours: clay,
/// heather, mud and gold flowers.
pub const LAYER_SPOTS: [(f32, f32); 4] = [
    (-134.0, 92.0),
    (-66.0, 92.0),
    (-134.0, 132.0),
    (-66.0, 132.0),
];
pub const LAYER_DISC: f32 = 14.0;
pub const LAYER_TINTS: [[f32; 3]; 4] = [
    [0.62, 0.2, 0.08],
    [0.4, 0.14, 0.5],
    [0.06, 0.045, 0.03],
    [0.75, 0.58, 0.06],
];
/// A meadow point of `terrain-layers` clear of the discs and the boulders.
pub const LAYER_MEADOW: (f32, f32) = (-100.0, 100.0);

/// The island's meadow from 70 m above with eight layers: its four with
/// their maps in texture arrays, and four more (layers 4 to 7, plain clay,
/// heather, mud and gold) on the control map's second page, each painted
/// in a disc on the meadow ([`LAYER_SPOTS`]) where the first page gives
/// the grass nothing. `painted` false leaves the second page 0: the extra
/// layers show nowhere.
pub fn layers_scene(painted: bool) -> RenderScene {
    let mut scene = with_layer_arrays(island_view("near"));
    let (x, z) = (-98.0, 114.0);
    let below = Vec3::new(x, island_height(x, z), z);
    scene.camera = camera_at(below + Vec3::new(0.0, 70.0, 26.0), below);
    let handle = scene.terrains[0].terrain;
    let mut data = scene.terrain(handle).expect("the island").clone();
    // Both pages at 1024 texels (2 m): the first resampled from the
    // island's. Each disc's layer fades from its full weight at 60 % of
    // the radius to none at the rim, the first page's weights scaled down
    // under it by as much.
    let m = 1024u32;
    let page = (m * m * 4) as usize;
    let mut rgba = vec![0u8; page * 2];
    for y in 0..m {
        for x in 0..m {
            let p =
                (Vec2::new(x as f32, y as f32) + 0.5) / m as f32 * ISLAND_SIZE - ISLAND_SIZE * 0.5;
            let c = data.control_at(p);
            let at = ((y * m + x) * 4) as usize;
            let mut keep = 1.0;
            for (i, s) in LAYER_SPOTS.iter().enumerate().filter(|_| painted) {
                let d = (p - Vec2::new(s.0, s.1)).length() / LAYER_DISC;
                let t = ((1.0 - d) / 0.4).clamp(0.0, 1.0);
                let w = t * t * (3.0 - 2.0 * t);
                rgba[page + at + i] = (w * 255.0).round() as u8;
                keep -= w;
            }
            for k in 0..4 {
                rgba[at + k] = (c[k] * keep.max(0.0) * 255.0).round() as u8;
            }
        }
    }
    data.control = Some(TerrainControl {
        width: m,
        height: m,
        rgba: Arc::from(rgba),
    });
    assert!(scene.set_terrain(handle, data));
    let t = &mut scene.terrains[0];
    let grass = t.layers[0];
    for tint in LAYER_TINTS {
        t.layers.push(TerrainLayer {
            color: Vec3::from(tint),
            roughness: 0.9,
            albedo_layer: None,
            normal_layer: None,
            ..grass
        });
    }
    scene
}

/// A soft sun from high up and a plain sky: the material v2 scenes'
/// light, so their frames measure the texture, not the shading.
fn plain_light(scene: &mut RenderScene) {
    let sun = Vec3::new(0.3, 1.0, 0.2);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        color: Vec3::ONE,
        intensity: 2.5,
        max_shadow_distance: 200.0,
        ..Default::default()
    };
    scene.environment = Environment {
        sky: ProceduralSky {
            sun_direction: sun,
            sun_color: Vec3::ONE,
            look: None,
        },
        intensity: 0.6,
        background: Background::Sky,
        ..Environment::default()
    };
}

/// The tile of the material v2 scenes' layer, in metres.
pub const V2_TILE: f32 = 4.0;
/// `tiling_scene`'s orthographic view: metres across its height.
pub const TILING_VIEW: f32 = 64.0;

/// R7: a flat 256 m terrain of one layer, a high-contrast texture of
/// dark blobs on pale ground repeating every 4 m, seen straight down
/// through an orthographic camera 64 m tall: with `anti_tiling` 0 every
/// 4 m shows the same blobs, with 1 hexagons of it turned and moved.
pub fn tiling_scene(anti_tiling: f32) -> RenderScene {
    let mut scene = RenderScene::new();
    let handle = scene.add_terrain(TerrainData::flat(Vec2::splat(256.0), 129, 129));
    let blobs = scene.add_texture(texture(256, ColorSpace::Srgb, |u, v| {
        // Both octaves lie in 0..1 (mean about 0.47): blobs over the
        // upper part of their sum, a little under half the ground.
        let n = tiling_fbm(u, v, 4, 51) + 0.35 * tiling_fbm(u, v, 16, 53);
        let g = if n > 0.65 { 40 } else { 215 };
        [g, g, g, 255]
    }));
    scene.terrains.push(Terrain {
        layers: vec![TerrainLayer {
            albedo: Some(blobs),
            color: Vec3::ONE,
            roughness: 0.9,
            tile: V2_TILE,
            anti_tiling,
            ..Default::default()
        }],
        ..Terrain::new(handle)
    });
    plain_light(&mut scene);
    scene.camera = Camera {
        position: Vec3::new(0.3, 150.0, 0.7),
        target: Vec3::new(0.3, 0.0, 0.7),
        up: Vec3::NEG_Z,
        near: 1.0,
        far: 400.0,
        projection: Projection::Orthographic {
            height: TILING_VIEW,
        },
        exposure: Exposure::linear(0.9),
    };
    scene
}

/// Where `steep_scene`'s wall starts (its foot along x) and its height.
pub const STEEP_FOOT: f32 = 0.0;
pub const STEEP_HEIGHT: f32 = 60.0;

/// R7: a 85° wall 60 m high across a 128 m terrain (its foot at x 0, its
/// face towards -x), one layer of a black and white checker of 4 m cells
/// over everything, seen square-on from 30 m away at mid-height: from
/// above (`project_slope` 0) the wall's cells stretch into vertical
/// streaks; past `project_slope` the two projections facing it keep
/// them square.
pub fn steep_scene(project_slope: f32) -> RenderScene {
    let mut scene = RenderScene::new();
    let n = 257u32;
    let size = 128.0;
    let cell = size / (n - 1) as f32;
    let rise = 85f32.to_radians().tan();
    let mut heights = Vec::with_capacity((n * n) as usize);
    for _ in 0..n {
        for c in 0..n {
            let x = c as f32 * cell - size * 0.5;
            heights.push(((x - STEEP_FOOT) * rise).clamp(0.0, STEEP_HEIGHT));
        }
    }
    let handle = scene.add_terrain(TerrainData {
        size: Vec2::splat(size),
        columns: n,
        rows: n,
        heights: Arc::from(heights),
        control: None,
        holes: None,
    });
    let checker = scene.add_texture(texture(256, ColorSpace::Srgb, |u, v| {
        let on = ((u * 2.0) as u32 + (v * 2.0) as u32).is_multiple_of(2);
        let g = if on { 230 } else { 25 };
        [g, g, g, 255]
    }));
    scene.terrains.push(Terrain {
        layers: vec![TerrainLayer {
            albedo: Some(checker),
            color: Vec3::ONE,
            roughness: 0.9,
            tile: V2_TILE * 2.0,
            project_slope,
            ..Default::default()
        }],
        ..Terrain::new(handle)
    });
    plain_light(&mut scene);
    // The sun from the camera's side, so the face is lit.
    scene.light.direction_to_light = Vec3::new(-0.8, 0.5, 0.3);
    scene.camera = camera_at(
        Vec3::new(STEEP_FOOT - 30.0, STEEP_HEIGHT * 0.5, 0.0),
        Vec3::new(STEEP_FOOT, STEEP_HEIGHT * 0.5, 0.0),
    );
    scene
}

/// `cliff_scene`'s cliff: its height, the rock's half-width along x (its
/// face towards +x at x = CLIFF_ROCK) and half-length along z.
pub const CLIFF_HEIGHT: f32 = 300.0;
pub const CLIFF_ROCK: f32 = 12.0;
pub const CLIFF_LENGTH: f32 = 60.0;
/// Where `cliff_scene`'s camera stands and what it looks at: the foot of
/// the rock's face, where the stamp meets the valley's ground.
pub const CLIFF_EYE: Vec3 = Vec3::new(CLIFF_ROCK + 22.0, 5.0, 9.0);
pub const CLIFF_FOOT: Vec3 = Vec3::new(CLIFF_ROCK, 0.6, 0.0);

/// T3: a 300 m cliff: a plateau for x < 0 over a valley at 0, one sample
/// apart on a 1 m grid (a height field's sawtooth wall), with a block of
/// rock 24 m across standing over the step as a terrain stamp, its foot
/// at the valley floor and its top 2 m above the plateau. The ground
/// under the block is carved as og.terrain.stamp carves it (to the
/// block's underside less 0.25 m). Two layers: grass up to 40° of slope,
/// rock past it, biplanar past 45°. The stamp is shaded by those layers
/// by its own normal (rock on its face, grass on its top) and, within
/// `blend` metres above the ground it meets, takes the ground's look.
pub fn cliff_scene(blend: f32) -> RenderScene {
    let mut scene = RenderScene::new();
    let (n, size) = (513u32, 512.0f32);
    let cell = size / (n - 1) as f32;
    let mut heights = Vec::with_capacity((n * n) as usize);
    for r in 0..n {
        for c in 0..n {
            let (x, z) = (c as f32 * cell - size * 0.5, r as f32 * cell - size * 0.5);
            let ground = if x < 0.0 { CLIFF_HEIGHT } else { 0.0 };
            let under = x.abs() <= CLIFF_ROCK && z.abs() <= CLIFF_LENGTH;
            heights.push(if under { ground.min(-0.25) } else { ground });
        }
    }
    let handle = scene.add_terrain(TerrainData {
        size: Vec2::splat(size),
        columns: n,
        rows: n,
        heights: Arc::from(heights),
        control: None,
        holes: None,
    });
    scene.terrains.push(Terrain {
        layers: vec![
            TerrainLayer {
                color: Vec3::new(0.16, 0.38, 0.08),
                roughness: 0.95,
                slope: [0.0, 40.0],
                slope_fade: 4.0,
                ..Default::default()
            },
            TerrainLayer {
                color: Vec3::new(0.42, 0.4, 0.37),
                roughness: 0.8,
                slope: [40.0, 90.0],
                slope_fade: 4.0,
                project_slope: 45.0,
                ..Default::default()
            },
        ],
        ..Terrain::new(handle)
    });
    let rock = scene.add_mesh(shapes::cuboid(Vec3::new(
        CLIFF_ROCK * 2.0,
        CLIFF_HEIGHT + 2.0,
        CLIFF_LENGTH * 2.0,
    )));
    scene.terrain_stamps.push(og_render::TerrainStampDraw {
        terrain: 0,
        mesh: rock,
        transform: Mat4::from_translation(Vec3::new(0.0, (CLIFF_HEIGHT + 2.0) * 0.5, 0.0)),
        blend,
        cast_shadows: true,
    });
    plain_light(&mut scene);
    // The sun from the valley's side, so the face and the ground are lit alike.
    scene.light.direction_to_light = Vec3::new(0.7, 0.65, 0.3).normalize();
    scene.environment.sky.sun_direction = scene.light.direction_to_light;
    scene.camera = camera_at(CLIFF_EYE, CLIFF_FOOT);
    scene
}
