//! The terrain and water scenes of phase 2 (`crates/og-render/README.md`
//! §Terrain, §Water): a 2 km island (hills, a ridge of mountains to the
//! north, a lake, beaches into the sea) drawn as a clipmap with four
//! layers (grass, rock on slopes, sand by the water, snow on the peaks)
//! seen from three distances, grass scattered over its meadow, and the
//! water's planar reflection, its shadows and the view from under the
//! lake. The land and its layers are generated here; what stands on the
//! meadow (boulders, a ruined gate, a watchtower, pines), the meadow's
//! grass and the mountains' rock are the kit's (`crate::kit`).

use crate::fx_scenes::{hash2, noise2, smooth, tiling_fbm};
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

/// The island's terrain with its layers, alone in an empty scene.
fn island_terrain() -> RenderScene {
    let mut scene = RenderScene::new();
    let data = island();
    let handle = scene.add_terrain(data);
    let layers = island_layers(&mut scene);
    scene.world.terrains.push(Terrain {
        layers,
        height_blend: 0.6,
        ..Terrain::new(handle)
    });
    scene
}

/// Where the near camera stands on the meadow (xz) and the way it looks
/// (a unit bearing in xz): what [`meadow_set`] is laid out from.
pub const MEADOW_EYE: Vec2 = Vec2::new(-80.0, 150.0);
const MEADOW_AHEAD: Vec2 = Vec2::new(-0.316_228, -0.948_683);

/// The meadow's boulders (xz): shadow casters at the near camera.
const MEADOW_BOULDERS: [(f32, f32); 4] = [
    (-96.0, 120.0),
    (-104.0, 128.0),
    (-88.0, 133.0),
    (-110.0, 112.0),
];

/// The meadow's set (the kit's models, `crate::kit`), laid out for the
/// near camera and standing for every other view of the island: the
/// boulders in front of it, a ruined gate with its columns across the way
/// to the mountains, a watchtower on the rise behind the gate, and stands
/// of pines either side of the view and up to the mountains' foot.
pub fn meadow_set(scene: &mut RenderScene) {
    use crate::kit;
    let ground = |p: Vec2| Vec3::new(p.x, island_height(p.x, p.y), p.y);
    // `ahead` metres along the near camera's view, `right` metres to its right.
    let right = Vec2::new(-MEADOW_AHEAD.y, MEADOW_AHEAD.x);
    let at = |ahead: f32, to_right: f32| MEADOW_EYE + MEADOW_AHEAD * ahead + right * to_right;
    let turn = |i: u32| hash2(i as i32, 7, 0x3ead) * std::f32::consts::TAU;
    let facing = MEADOW_AHEAD.x.atan2(MEADOW_AHEAD.y);

    let boulder = kit::prop(scene, "boulder");
    for (i, &(x, z)) in MEADOW_BOULDERS.iter().enumerate() {
        let r = 1.2 + i as f32 * 0.5;
        let foot = ground(Vec2::new(x, z)) - Vec3::Y * 0.3 * r;
        boulder.place(scene, foot, turn(i as u32), r * 1.7);
    }
    // Smaller stones about the gate and the tower's rise.
    for (i, (ahead, to_right, h)) in [
        (52.0, 30.0, 1.3),
        (70.0, -8.0, 1.0),
        (96.0, 22.0, 1.8),
        (128.0, 52.0, 2.4),
        (24.0, 17.0, 0.9),
    ]
    .into_iter()
    .enumerate()
    {
        let foot = ground(at(ahead, to_right)) - Vec3::Y * 0.25 * h;
        boulder.place(scene, foot, turn(i as u32 + 10), h);
    }

    // The gate: an arch, a standing and a broken column either side.
    let arch = kit::prop(scene, "arch");
    let column = kit::prop(scene, "column");
    let broken = kit::prop(scene, "column_broken");
    arch.place(scene, ground(at(62.0, 13.0)), facing, 9.5);
    column.place(scene, ground(at(62.0, 4.5)), turn(20), 7.0);
    column.place(scene, ground(at(62.0, 21.5)), turn(21), 7.0);
    broken.place(scene, ground(at(61.0, -2.5)), turn(22), 2.8);
    broken.place(scene, ground(at(63.0, 28.5)), turn(23), 2.8);

    // The watchtower on the highest ground near the rise behind the gate.
    let rise = {
        let c = at(150.0, 44.0);
        let mut best = c;
        for dz in -8..=8 {
            for dx in -8..=8 {
                let p = c + Vec2::new(dx as f32, dz as f32) * 4.0;
                if island_height(p.x, p.y) > island_height(best.x, best.y) {
                    best = p;
                }
            }
        }
        best
    };
    let tower = kit::prop(scene, "watchtower");
    tower.place(scene, ground(rise) - Vec3::Y * 0.4, 2.2, 22.0);

    // The pines: one by the boulders, then stands (where along the view,
    // how far to its right, their radius and how many), on ground that is
    // neither shore nor steep.
    let pine = kit::prop(scene, "pine");
    pine.place(scene, ground(at(36.0, -21.0)), turn(30), 15.0);
    for (s, (ahead, to_right, radius, count)) in [
        (78.0, -52.0, 22.0, 13),
        (56.0, 50.0, 15.0, 8),
        (104.0, 78.0, 28.0, 16),
        (120.0, -24.0, 12.0, 5),
        (180.0, -80.0, 42.0, 26),
        (205.0, 96.0, 40.0, 22),
        (270.0, 4.0, 55.0, 30),
        (340.0, -130.0, 60.0, 30),
        (345.0, 130.0, 60.0, 30),
        (-60.0, -90.0, 40.0, 18),
        (-40.0, 110.0, 45.0, 20),
    ]
    .into_iter()
    .chain(woods())
    .enumerate()
    {
        for k in 0..count {
            let h = |n: i32| hash2(k, n, 0x91e0 + s as u32);
            // Denser towards the stand's middle.
            let d = radius * h(0).sqrt() * (0.35 + 0.65 * h(3));
            let a = h(1) * std::f32::consts::TAU;
            let p = at(ahead, to_right) + Vec2::new(a.cos(), a.sin()) * d;
            let y = island_height(p.x, p.y);
            let steep = (island_height(p.x + 3.0, p.y) - y)
                .abs()
                .max((island_height(p.x, p.y + 3.0) - y).abs())
                > 2.2;
            if y < LAKE_LEVEL + 4.0 || steep || p.distance(rise) < 9.0 {
                continue;
            }
            let foot = Vec3::new(p.x, y - 0.3, p.y);
            pine.place(scene, foot, a * 3.0, 9.5 + 6.0 * h(2));
        }
    }
}

/// The island's other woods, for the views from above and from the sea:
/// a stand (as [`meadow_set`]'s: along the near view, to its right, its
/// radius, its pines) in under half the cells of a 120 m grid, none where
/// the near camera's view runs to the mountains.
fn woods() -> impl Iterator<Item = (f32, f32, f32, i32)> {
    (-6..=6).flat_map(|r: i32| {
        (-6..=6).filter_map(move |c: i32| {
            let h = |n: i32| hash2(c * 31 + n, r, 0x700d);
            let ahead = r as f32 * 120.0 + (h(1) - 0.5) * 70.0;
            let to_right = c as f32 * 120.0 + (h(2) - 0.5) * 70.0;
            let in_view = ahead > -90.0 && ahead < 420.0 && to_right.abs() < 190.0;
            (h(0) < 0.45 && !in_view)
                .then(|| (ahead, to_right, 28.0 + 30.0 * h(3), 8 + (h(4) * 9.0) as i32))
        })
    })
}

/// The lake scenes' island: the terrain, the afternoon and four plain
/// boulders on the meadow. They keep it until the water family dresses
/// them; every other view of the island is [`island_meadow`].
pub fn island_scene() -> RenderScene {
    let mut scene = island_terrain();
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

/// The island with its meadow's set ([`meadow_set`]) on a fair afternoon
/// ([`fair_afternoon`]). Its grass layer is the kit's mossy grass (colour
/// and normal map), so the ground holds up at eye height.
pub fn island_meadow() -> RenderScene {
    let mut scene = island_terrain();
    let (color, normal) = crate::kit::layer_maps(&mut scene, "grass");
    let grass = &mut scene.world.terrains[0].layers[0];
    grass.albedo = Some(color);
    grass.normal = Some(normal);
    grass.color = Vec3::new(0.72, 1.05, 0.5);
    grass.tile = 3.5;
    grass.anti_tiling = 0.5;
    let (color, normal) = crate::kit::layer_maps(&mut scene, "rock");
    let rock = &mut scene.world.terrains[0].layers[1];
    rock.albedo = Some(color);
    rock.normal = Some(normal);
    rock.color = Vec3::new(0.86, 0.84, 0.82);
    rock.tile = 18.0;
    meadow_set(&mut scene);
    fair_afternoon(&mut scene);
    scene
}

/// [`afternoon`] under the physical sky: the same sun (so every shadow
/// falls where it did) 31° up, as [`fair_sky`] colours it.
pub fn fair_afternoon(scene: &mut RenderScene) {
    afternoon(scene);
    fair_sky(scene, 8.07);
}

/// The physical sky over the scene's sun, left where it stands: its light
/// and the sky's the colours Earth's air gives the sun at `hours` (6 at
/// the horizon, 15° higher each hour), the scene's haze in the horizon's
/// colour, and fair-weather cumulus with their shadows on the land. The
/// sky is drawn per pixel, which the clouds need.
pub fn fair_sky(scene: &mut RenderScene, hours: f32) {
    let sun = scene.lighting.light.direction_to_light;
    // A path with no tilt: only the look's colours are taken.
    let look = og_render::day_night(hours, 0.0, 57.3)
        .with_atmosphere(&og_render::PhysicalAtmosphere::EARTH);
    scene.lighting.light.color = look.light_color;
    scene.lighting.light.intensity = look.light_intensity;
    scene.lighting.environment = Environment {
        sky: ProceduralSky {
            sun_direction: sun,
            sun_color: look.sun_color,
            look: Some(look.sky),
        },
        intensity: look.ambient,
        background: Background::Procedural {
            sun_direction: sun,
            sun_color: look.sun_color,
        },
        ..Environment::default()
    };
    scene.atmosphere.adaptation = look.adaptation;
    if let Some(fog) = &mut scene.atmosphere.fog {
        fog.color = look.fog_color;
        fog.sun_color = look.fog_sun.unwrap_or(fog.sun_color);
    }
    let clouds = og_render::Clouds {
        coverage: 0.4,
        layer: 0.15,
        scale: 6000.0,
        detail: 0.55,
        ..og_render::Clouds::default()
    };
    clouds.light_scene(scene);
    scene.atmosphere.clouds = Some(clouds);
    // Their shapes some minutes into the wind's drift.
    scene.frame.tick = 9000;
}

/// The island's afternoon: the sun, the sky lighting the scene and a
/// light haze.
pub fn afternoon(scene: &mut RenderScene) {
    let sun = Vec3::new(0.62, 0.45, 0.4);
    scene.lighting.light = DirectionalLight {
        direction_to_light: sun,
        color: Vec3::new(1.0, 0.93, 0.82),
        intensity: 3.2,
        max_shadow_distance: 400.0,
        ..Default::default()
    };
    scene.lighting.environment = Environment {
        sky: ProceduralSky {
            sun_direction: sun,
            sun_color: Vec3::new(1.0, 0.9, 0.76),
            look: None,
        },
        intensity: 0.7,
        background: Background::Sky,
        ..Environment::default()
    };
    scene.atmosphere.fog = Some(Fog {
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
    let mut scene = island_meadow();
    let ground = |x: f32, z: f32| island_height(x, z);
    scene.frame.camera = match distance {
        "near" => {
            let (x, z) = (MEADOW_EYE.x, MEADOW_EYE.y);
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
        // From 700 m up the horizon is 95 km off: the view and the sea
        // reach well past it, so the sea meets the sky in one line.
        _ => Camera {
            far: 1.0e6,
            ..camera_at(
                Vec3::new(240.0, 700.0, 1350.0),
                Vec3::new(0.0, 40.0, -150.0),
            )
        },
    };
    // The sea around the island, out to the horizon (flat, the sky's
    // cubemap reflected).
    scene.world.water.push(Water {
        transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        size: Vec2::splat(4.0e6),
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
    scene.world.water.push(Water {
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
    scene.frame.camera = camera_at(
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
    scene.lighting.light.direction_to_light = sun;
    scene.lighting.environment.sky.sun_direction = sun;
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
    scene.frame.camera = camera_at(eye, Vec3::new(LAKE.x - 4.0, LAKE_LEVEL, LAKE.y + 92.0));
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
    scene.frame.camera = camera_at(
        eye,
        Vec3::new(LAKE.x - 48.0, LAKE_LEVEL + 6.0, LAKE.y + 90.0),
    );
    scene.frame.tick = 90;
    scene
}

/// Under the lake, looking up towards the sun: its light coming down in
/// shafts from the bright side of Snell's window, the water darker below
/// them than above, the caustics where they reach the bed.
pub fn water_underwater_shafts_scene() -> RenderScene {
    let mut scene = water_underwater_scene();
    let sun = scene.lighting.light.direction_to_light.normalize();
    let eye = Vec3::new(LAKE.x - 40.0, LAKE_LEVEL - 3.2, LAKE.y + 70.0);
    scene.frame.camera = camera_at(
        eye,
        eye + Vec3::new(sun.x, 0.0, sun.z).normalize() * 20.0 + Vec3::Y * 5.0,
    );
    scene
}

/// A clump of fourteen grass blades up to 0.55 m tall, their roots spread
/// over a disc 0.2 m in radius (so neighbouring clumps close into turf):
/// each a thin strip of two quads and a tip, curving outwards as it rises,
/// drawn on both sides, dark at the root and lighter to the tip, one in
/// three a drier tone, its normals leaning up so the clump shades like the
/// meadow under it.
pub fn grass_tuft() -> MeshData {
    const BLADES: u32 = 14;
    let mut m = MeshData::default();
    for b in 0..BLADES {
        let yaw = b as f32 * 2.4 + 0.3;
        let (s, c) = yaw.sin_cos();
        let out = Vec3::new(c, 0.0, s);
        let side = Vec3::new(-s, 0.0, c);
        let root = out * (0.2 * ((b as f32 + 0.5) / BLADES as f32).sqrt());
        let tall = 0.34 + 0.07 * (b % 4) as f32;
        let reach = 0.1 + 0.035 * (b % 3) as f32;
        let tone = [[0.86, 1.0, 0.7], [0.78, 0.92, 0.62], [1.0, 0.96, 0.52]][(b % 3) as usize];
        let n = (Vec3::Y * 2.0 + out).normalize();
        // The blade's middle line and its half width at `t` of its height.
        let at = |t: f32| root + out * (reach * t * t) + Vec3::Y * (tall * t);
        let half = |t: f32| side * (0.028 * (1.0 - 0.75 * t));
        for face in [1.0f32, -1.0] {
            let first = m.vertices.len() as u32;
            for (p, u, v, shade) in [
                (at(0.0) - half(0.0), 0.0, 0.0, 0.4),
                (at(0.0) + half(0.0), 1.0, 0.0, 0.4),
                (at(0.55) - half(0.55), 0.0, 0.55, 0.78),
                (at(0.55) + half(0.55), 1.0, 0.55, 0.78),
                (at(1.0), 0.5, 1.0, 1.0),
            ] {
                m.vertices.push(Vertex {
                    position: p.into(),
                    normal: (n * face.max(0.6)).normalize().into(),
                    uv: [u, v],
                    color: [shade * tone[0], shade * tone[1], shade * tone[2], 1.0],
                    tangent: [0.0; 4],
                });
            }
            for [i, j, k] in [[0, 2, 1], [1, 2, 3], [2, 4, 3]] {
                if face > 0.0 {
                    m.indices.extend([first + i, first + j, first + k]);
                } else {
                    m.indices.extend([first + i, first + k, first + j]);
                }
            }
        }
    }
    m
}

/// The near view with grass on the meadow (the grass layer, 6 clumps a
/// square metre within 60 m, fading out over its last third), placed and
/// drawn by the renderer (`RenderScene::scatters`).
pub fn grass_scene() -> RenderScene {
    let mut scene = island_view("near");
    let tuft = scene.add_mesh(grass_tuft());
    let grass = scene.add_material(Material {
        base_color: [0.24, 0.42, 0.1, 1.0],
        roughness: 0.85,
        ..Default::default()
    });
    scene.world.scatters.push(ScatterDraw {
        terrain: 0,
        mesh: tuft,
        material: grass,
        lod_materials: [None; 2],
        scatter: grass_scatter(),
        cast_shadows: true,
        impostor: None,
        impostor_distance: 0.0,
        with: None,
        lods: [None; 2],
    });
    scene
}

/// The grass scene's grass: layer 0 (grass).
pub fn grass_scatter() -> TerrainScatter {
    TerrainScatter {
        layer: 0,
        density: 6.0,
        distance: 60.0,
        fade: 0.35,
        scale: [0.8, 1.4],
        align: 0.3,
        seed: 3,
        sway: 1.0,
        ..Default::default()
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
        ..Default::default()
    }
}

/// The island's terrain at the lod a scene asks for (a test helper).
pub fn with_lod(mut scene: RenderScene, lod: TerrainLod) -> RenderScene {
    for t in &mut scene.world.terrains {
        t.lod = lod;
    }
    scene
}

/// `scene` with every terrain's layer textures moved into two texture
/// arrays (`Terrain::layer_arrays`): the albedos as layers of one, the
/// normal maps of another, each layer naming its layers. The same images
/// (an array's layers share one size, so each at its array's smallest),
/// so the same frame within mip filtering (an array's mips are
/// box-filtered on the CPU, a texture's on the GPU).
pub fn with_layer_arrays(mut scene: RenderScene) -> RenderScene {
    use og_render::{TerrainLayerArrays, TextureArrayData, TextureData, TextureSource};
    for ti in 0..scene.world.terrains.len() {
        let layers = scene.world.terrains[ti].layers.clone();
        let mut albedos = Vec::new();
        let mut normals = Vec::new();
        let mut picked = Vec::new();
        for l in &layers {
            let take = |h: Option<og_render::TextureHandle>, list: &mut Vec<TextureData>| {
                h.map(|h| {
                    list.push(scene.texture(h).expect("a texture").clone());
                    list.len() as u32 - 1
                })
            };
            picked.push((take(l.albedo, &mut albedos), take(l.normal, &mut normals)));
        }
        let mut array = |list: Vec<TextureData>| {
            let size = list.iter().map(|t| t.width.min(t.height)).min();
            let list: Vec<TextureSource> = list
                .into_iter()
                .map(|t| TextureSource::Rgba(shrunk(t, size.unwrap_or(0))))
                .collect();
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
        let t = &mut scene.world.terrains[ti];
        t.layer_arrays = arrays;
        for (l, (albedo, normal)) in t.layers.iter_mut().zip(picked) {
            l.albedo_layer = albedo;
            l.normal_layer = normal;
        }
    }
    scene
}

/// `t` as a square of `size` texels (itself when it is one already).
fn shrunk(t: og_render::TextureData, size: u32) -> og_render::TextureData {
    if t.width == size && t.height == size {
        return t;
    }
    let from =
        image::RgbaImage::from_raw(t.width, t.height, t.rgba.to_vec()).expect("a texture's texels");
    let to = image::imageops::resize(&from, size, size, image::imageops::FilterType::Triangle);
    og_render::TextureData {
        width: size,
        height: size,
        rgba: to.into_raw(),
        ..t
    }
}

/// The pit cut out of the island's meadow in `terrain-holes`: its world
/// x and z ranges (whole 2 m cells), and the cave floor's height under it.
pub const PIT_X: [f32; 2] = [-60.0, -44.0];
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

/// The pit's corners on the surface: their lowest and highest heights.
fn pit_rim() -> (f32, f32) {
    [PIT_X[0], PIT_X[1]]
        .iter()
        .flat_map(|&x| [PIT_Z[0], PIT_Z[1]].map(|z| island_height(x, z)))
        .fold((f32::MAX, f32::MIN), |(lo, hi), h| (lo.min(h), hi.max(h)))
}

/// The island's meadow with a 16 × 12 m pit cut out of it
/// ([`TerrainData::holes`]): a shaft of the ruins, walled and floored in
/// cobbles, a kerb round its mouth, a ladder down its north wall, a
/// fallen column and rubble on its floor, the sun nearly overhead, seen
/// from over its corner 12 m above the meadow. The floor shows through the hole with
/// the sunlight that falls through it (the hole casts no shadow; only the
/// walls shade their foot), the meadow round it unbroken.
pub fn holes_scene() -> RenderScene {
    use crate::kit;
    let mut scene = island_meadow();
    let handle = scene.world.terrains[0].terrain;
    let mut data = scene.terrain(handle).expect("the island").clone();
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
    assert!(scene.set_terrain(handle, data));

    let c = pit_center();
    let floor = pit_floor();
    let kerb = pit_rim().1 + 0.45;
    let stone = kit::surface(&mut scene, "cobbles");
    let paving = scene.add_mesh(kit::slab(
        Vec2::new(PIT_X[0], PIT_Z[0]),
        Vec2::new(PIT_X[1], PIT_Z[1]),
        floor,
        3.0,
    ));
    scene.add_instance(paving, stone, Mat4::IDENTITY);
    // The walls stand outside the hole, 1.2 m thick, the corners closed
    // by the two along x.
    let thick = 1.2;
    for (min, max) in [
        (
            Vec2::new(PIT_X[0] - thick, PIT_Z[0] - thick),
            Vec2::new(PIT_X[1] + thick, PIT_Z[0]),
        ),
        (
            Vec2::new(PIT_X[0] - thick, PIT_Z[1]),
            Vec2::new(PIT_X[1] + thick, PIT_Z[1] + thick),
        ),
        (
            Vec2::new(PIT_X[0] - thick, PIT_Z[0]),
            Vec2::new(PIT_X[0], PIT_Z[1]),
        ),
        (
            Vec2::new(PIT_X[1], PIT_Z[0]),
            Vec2::new(PIT_X[1] + thick, PIT_Z[1]),
        ),
    ] {
        let wall = scene.add_mesh(kit::block(
            Vec3::new(min.x, floor - 0.5, min.y),
            Vec3::new(max.x, kerb, max.y),
            3.0,
        ));
        scene.add_instance(wall, stone, Mat4::IDENTITY);
    }
    let broken = kit::prop(&mut scene, "column_broken");
    broken.place(&mut scene, Vec3::new(c.x - 3.5, floor, c.z - 2.0), 0.8, 2.8);
    let boulder = kit::prop(&mut scene, "boulder");
    boulder.place(
        &mut scene,
        Vec3::new(c.x + 2.5, floor - 0.4, c.z + 1.5),
        2.3,
        2.4,
    );
    // Rubble fallen into the north-west corner, and a ladder up the north
    // wall: four lengths end to end, leaning 12° onto the kerb.
    for (i, (dx, dz, r)) in [
        (1.6, 1.4, 1.5),
        (3.4, 1.2, 1.0),
        (1.3, 3.3, 1.1),
        (2.6, 2.6, 0.7),
        (4.8, 1.0, 0.6),
        (1.0, 5.0, 0.6),
    ]
    .into_iter()
    .enumerate()
    {
        boulder.place(
            &mut scene,
            Vec3::new(PIT_X[0] + dx, floor - 0.25 * r, PIT_Z[0] + dz),
            1.7 * i as f32,
            r * 1.3,
        );
    }
    let ladder = kit::prop(&mut scene, "ladder");
    let length = 2.6;
    let lean = Mat4::from_translation(Vec3::new(c.x + 3.0, floor, PIT_Z[0] + 2.2))
        * Mat4::from_rotation_x(-12f32.to_radians());
    for i in 0..4 {
        let along = Mat4::from_translation(Vec3::Y * (i as f32 * length));
        ladder.place_by(
            &mut scene,
            lean * along * ladder.transform(Vec3::ZERO, 0.0, length),
        );
    }
    let column = kit::prop(&mut scene, "column");
    let rim = |x: f32, z: f32| Vec3::new(x, island_height(x, z), z);
    column.place(&mut scene, rim(PIT_X[0] - 3.5, PIT_Z[0] - 3.0), 0.3, 7.0);
    broken.place(&mut scene, rim(PIT_X[1] + 3.5, PIT_Z[0] - 3.0), 1.9, 2.8);

    let sun = Vec3::new(0.2, 0.95, 0.15).normalize();
    scene.lighting.light.direction_to_light = sun;
    // 75° up.
    fair_sky(&mut scene, 11.0);
    scene.frame.camera = camera_at(
        c + Vec3::new(13.0, 12.0, 12.0),
        Vec3::new(c.x - 2.0, floor + 1.5, c.z - 2.0),
    );
    scene
}

/// `terrain-layers`' flowers: clumps of blossoms of colour `petal` (sRGB)
/// over half of a dark turf, the blossoms the layer's height.
fn flowers(scene: &mut RenderScene, petal: [f32; 3], seed: u32) -> og_render::TextureHandle {
    scene.add_texture(texture(256, ColorSpace::Srgb, |u, v| {
        let clump = tiling_fbm(u, v, 5, seed) + 0.3 * tiling_fbm(u, v, 14, seed + 1);
        let speck = tiling_fbm(u, v, 56, seed + 2);
        let turf = 0.7 + 0.5 * tiling_fbm(u, v, 32, seed + 3);
        if clump > 0.6 && speck > 0.42 {
            let g = 0.75 + 0.5 * speck;
            [
                (petal[0] * g).min(255.0) as u8,
                (petal[1] * g).min(255.0) as u8,
                (petal[2] * g).min(255.0) as u8,
                235,
            ]
        } else {
            [
                (52.0 * turf) as u8,
                (84.0 * turf) as u8,
                (34.0 * turf) as u8,
                110,
            ]
        }
    }))
}

/// A clump of seven flowering stems up to 0.55 m, their roots over a
/// disc 0.22 m in radius: each a thin green blade leaning outwards with a
/// blossom of colour `petal` (linear) at its tip, drawn on both sides: a
/// `spike` (heather: two crossed upright diamonds over the stem's upper
/// half) or a flat head 0.14 m across facing up (a buttercup's).
pub fn flower_clump(petal: [f32; 3], spike: bool) -> MeshData {
    const STEMS: u32 = 7;
    let mut m = MeshData::default();
    let leaf = [0.2, 0.36, 0.1];
    // A fan of triangles about its first point, seen from both sides.
    let mut fan = |points: &[(Vec3, [f32; 3], f32)], normal: Vec3| {
        for face in [1.0f32, -1.0] {
            let first = m.vertices.len() as u32;
            for &(p, tone, shade) in points {
                m.vertices.push(Vertex {
                    position: p.into(),
                    normal: (normal * face.max(0.6)).normalize().into(),
                    uv: [0.5, 0.5],
                    color: [shade * tone[0], shade * tone[1], shade * tone[2], 1.0],
                    tangent: [0.0; 4],
                });
            }
            for i in 1..points.len() as u32 - 1 {
                if face > 0.0 {
                    m.indices.extend([first, first + i, first + i + 1]);
                } else {
                    m.indices.extend([first, first + i + 1, first + i]);
                }
            }
        }
    };
    for b in 0..STEMS {
        let yaw = b as f32 * 2.4 + 0.7;
        let (sn, cs) = yaw.sin_cos();
        let out = Vec3::new(cs, 0.0, sn);
        let side = Vec3::new(-sn, 0.0, cs);
        let root = out * (0.22 * ((b as f32 + 0.5) / STEMS as f32).sqrt());
        let tall = 0.36 + 0.06 * (b % 4) as f32;
        let tip = root + out * 0.07 + Vec3::Y * tall;
        let up = (Vec3::Y * 2.0 + out).normalize();
        fan(
            &[
                (tip, leaf, 0.9),
                (root - side * 0.012, leaf, 0.45),
                (root + side * 0.012, leaf, 0.45),
            ],
            up,
        );
        let tone = 0.8 + 0.1 * (b % 3) as f32;
        if spike {
            let (low, high) = (root.lerp(tip, 0.45), tip + Vec3::Y * 0.06);
            let waist = low.lerp(high, 0.4);
            for across in [side, out] {
                fan(
                    &[
                        (low, petal, 0.6 * tone),
                        (waist + across * 0.05, petal, tone),
                        (high, petal, 1.1 * tone),
                        (waist - across * 0.05, petal, tone),
                    ],
                    up,
                );
            }
        } else {
            let mut head = vec![(tip, [1.0, 0.72, 0.1], 0.9)];
            for k in 0..=6 {
                let a = k as f32 * std::f32::consts::TAU / 6.0;
                head.push((
                    tip + (side * a.cos() + out * a.sin()) * 0.07 + Vec3::Y * 0.015,
                    petal,
                    tone,
                ));
            }
            fan(&head, Vec3::Y);
        }
    }
    m
}

/// The weights of `terrain-layers`' four extra layers at world `p` (xz),
/// their sum at most 1: the path from behind the near camera to the gate
/// (2.4 m wide), the ruin's floor round the gate, and drifts of heather
/// and of gold flowers over the meadow within 150 m of the gate, none
/// within 5 m of the path.
fn layer_paint(p: Vec2) -> [f32; 4] {
    let right = Vec2::new(-MEADOW_AHEAD.y, MEADOW_AHEAD.x);
    let rel = p - MEADOW_EYE;
    let (ahead, to_right) = (rel.dot(MEADOW_AHEAD), rel.dot(right));
    let gate = Vec2::new(62.0, 13.0);
    // The path wanders a few metres about its line, straight into the gate.
    let line = 9.0
        + 4.0 * ((ahead - 20.0) / 42.0).clamp(-1.0, 1.0)
        + 9.0 * fbm(ahead / 45.0, 0.5, 2, 61) * smooth(4.0, 30.0, gate.x - ahead);
    let off = if ahead < gate.x - 4.0 && ahead > -60.0 {
        (to_right - line).abs()
    } else {
        f32::MAX
    };
    let edge = 1.2 * fbm(p.x / 2.5, p.y / 2.5, 2, 62);
    let mud = 1.0 - smooth(0.5, 1.9, off + edge);
    let from_gate = (Vec2::new(ahead, to_right) - gate).length();
    let clay =
        (1.0 - smooth(
            9.0,
            14.0,
            from_gate + 5.0 * fbm(p.x / 9.0, p.y / 9.0, 3, 63),
        )) * (1.0 - mud);
    let open = smooth(5.0, 9.0, off) * (1.0 - clay) * (1.0 - smooth(110.0, 150.0, from_gate));
    // The drifts thin out over metres, their edges ragged.
    let fine = 0.14 * fbm(p.x / 2.2, p.y / 2.2, 2, 66);
    let drift = |scale: f32, seed: u32| {
        smooth(0.0, 0.14, fbm(p.x / scale, p.y / scale, 3, seed) + fine) * open
    };
    let heather = drift(34.0, 64);
    let gold = drift(27.0, 65) * (1.0 - heather);
    [mud, clay, heather, gold]
}

/// The island's meadow with eight layers, their maps in texture arrays,
/// seen from 7 m up behind the near camera: its four, and four more
/// (layers 4 to 7) on the control map's second page, painted
/// ([`layer_paint`]) where the first page then gives the grass nothing: a
/// mud path to the gate and the clay floor of the ruin (the island's rock
/// and sand, tinted), heather and gold flowers in drifts (their ground
/// [`flowers`], the plants [`flower_clump`]s scattered on the two
/// layers). `painted` false leaves the second page 0: the extra layers
/// show nowhere, and no flower grows.
pub fn layers_scene(painted: bool) -> RenderScene {
    let mut scene = island_view("near");
    let heather = flowers(&mut scene, [150.0, 72.0, 176.0], 201);
    let gold = flowers(&mut scene, [236.0, 190.0, 44.0], 211);
    let t = &mut scene.world.terrains[0];
    let (grass, rock, sand) = (t.layers[0], t.layers[1], t.layers[2]);
    for (albedo, tint, tile) in [
        (rock.albedo, [0.86, 0.72, 0.6], 5.0),
        (sand.albedo, [0.66, 0.5, 0.4], 5.0),
        (Some(heather), [1.0; 3], 6.0),
        (Some(gold), [1.0; 3], 6.0),
    ] {
        t.layers.push(TerrainLayer {
            albedo,
            normal: None,
            color: Vec3::from(tint),
            roughness: 0.9,
            tile,
            macro_variation: 0.5,
            ..grass
        });
    }
    let mut scene = with_layer_arrays(scene);
    // The flowers themselves: plants scattered on the two flower layers.
    let bloom = scene.add_material(Material {
        base_color: [1.0; 4],
        roughness: 0.8,
        ..Default::default()
    });
    for (layer, petal, spike, seed) in [
        (6, [0.42, 0.12, 0.52], true, 31),
        (7, [0.95, 0.66, 0.05], false, 37),
    ] {
        let mesh = scene.add_mesh(flower_clump(petal, spike));
        scene.world.scatters.push(ScatterDraw {
            terrain: 0,
            mesh,
            material: bloom,
            lod_materials: [None; 2],
            scatter: TerrainScatter {
                layer,
                density: 7.0,
                distance: 120.0,
                fade: 0.3,
                scale: [0.8, 1.4],
                align: 0.2,
                seed,
                sway: 0.7,
                ..Default::default()
            },
            cast_shadows: false,
            impostor: None,
            impostor_distance: 0.0,
            with: None,
            lods: [None; 2],
        });
    }
    let eye = MEADOW_EYE - MEADOW_AHEAD * 14.0;
    let to = MEADOW_EYE + MEADOW_AHEAD * 58.0;
    scene.frame.camera = camera_at(
        Vec3::new(eye.x, island_height(eye.x, eye.y) + 7.0, eye.y),
        Vec3::new(to.x, island_height(to.x, to.y) + 3.0, to.y),
    );
    let handle = scene.world.terrains[0].terrain;
    let mut data = scene.terrain(handle).expect("the island").clone();
    // Both pages at 2048 texels (1 m): the first resampled from the
    // island's, its weights scaled down under the second's by as much as
    // those take.
    let m = 2048u32;
    let page = (m * m * 4) as usize;
    let mut rgba = vec![0u8; page * 2];
    for y in 0..m {
        for x in 0..m {
            let p =
                (Vec2::new(x as f32, y as f32) + 0.5) / m as f32 * ISLAND_SIZE - ISLAND_SIZE * 0.5;
            let c = data.control_at(p);
            let at = ((y * m + x) * 4) as usize;
            let mut keep = 1.0;
            // Only the meadow about the view is painted.
            if painted && (p - MEADOW_EYE).length() < 260.0 {
                for (i, w) in layer_paint(p).into_iter().enumerate() {
                    rgba[page + at + i] = (w * 255.0).round() as u8;
                    keep -= w;
                }
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
    scene
}

/// A soft sun from high up and a plain sky: the material v2 scenes'
/// light, so their frames measure the texture, not the shading.
fn plain_light(scene: &mut RenderScene) {
    let sun = Vec3::new(0.3, 1.0, 0.2);
    scene.lighting.light = DirectionalLight {
        direction_to_light: sun,
        color: Vec3::ONE,
        intensity: 2.5,
        max_shadow_distance: 200.0,
        ..Default::default()
    };
    scene.lighting.environment = Environment {
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
/// The stony plateau's swell (m): level within 24 m of the ruin at its
/// middle, rolling a few metres past it.
fn plateau_height(x: f32, z: f32) -> f32 {
    let r = (x * x + z * z).sqrt();
    5.0 * fbm(x / 140.0, z / 140.0, 3, 171) * smooth(24.0, 90.0, r)
}

/// R7: a stony plateau of one layer, the kit's rock repeating every 4 m,
/// seen from 36 m up as a strategy game looks down on its map: the ruin
/// of a round colonnade about a court of setts (a second layer, painted
/// by the control map), a watchtower, fallen stones, pines and tussocks
/// of dry grass, their shadows long across 150 m of ground. With `anti_tiling`
/// 0 every 4 m shows the same cracks, a grid across the whole frame; with
/// 1, hexagons of the rock turned and moved, and no period shows.
pub fn tiling_scene(anti_tiling: f32) -> RenderScene {
    use crate::kit;
    let mut scene = RenderScene::new();
    let (n, size) = (257u32, 768.0f32);
    let cell = size / (n - 1) as f32;
    let heights: Vec<f32> = (0..n * n)
        .map(|i| {
            let (c, r) = ((i % n) as f32, (i / n) as f32);
            plateau_height(c * cell - size * 0.5, r * cell - size * 0.5)
        })
        .collect();
    // The control map (0.75 m a texel): the court's setts (layer 1) in a
    // ragged disc of 12.5 m about the middle, the rock everywhere else.
    let m = 1024u32;
    let mut rgba = Vec::with_capacity((m * m * 4) as usize);
    for y in 0..m {
        for x in 0..m {
            let at = (Vec2::new(x as f32, y as f32) + 0.5) / m as f32 * size - size * 0.5;
            let edge = at.length() + 1.6 * fbm(at.x / 3.0, at.y / 3.0, 2, 173);
            let paved = ((1.0 - smooth(11.6, 13.2, edge)) * 255.0) as u8;
            rgba.extend_from_slice(&[255 - paved, paved, 0, 0]);
        }
    }
    let handle = scene.add_terrain(TerrainData {
        heights: heights.into(),
        control: Some(TerrainControl {
            width: m,
            height: m,
            rgba: Arc::from(rgba),
        }),
        ..TerrainData::flat(Vec2::splat(size), n, n)
    });
    let (rock, rock_normal) = kit::layer_maps(&mut scene, "rock");
    let (setts, setts_normal) = kit::layer_maps(&mut scene, "cobbles");
    scene.world.terrains.push(Terrain {
        layers: vec![
            TerrainLayer {
                albedo: Some(rock),
                normal: Some(rock_normal),
                // Sandstone.
                color: Vec3::new(1.12, 0.98, 0.8),
                roughness: 0.9,
                tile: V2_TILE,
                anti_tiling,
                macro_variation: 0.5,
                ..Default::default()
            },
            TerrainLayer {
                albedo: Some(setts),
                normal: Some(setts_normal),
                color: Vec3::new(1.25, 1.1, 0.9),
                roughness: 0.85,
                tile: 3.5,
                ..Default::default()
            },
        ],
        height_blend: 0.5,
        ..Terrain::new(handle)
    });
    let ground = |x: f32, z: f32| Vec3::new(x, plateau_height(x, z), z);
    // The court and its colonnade: an arch across from the camera, columns
    // standing and broken round a circle of 15 m.
    let arch = kit::prop(&mut scene, "arch");
    arch.place(&mut scene, ground(0.0, -15.0), 0.0, 8.5);
    let column = kit::prop(&mut scene, "column");
    let broken = kit::prop(&mut scene, "column_broken");
    for k in 1..12 {
        let a = k as f32 / 12.0 * std::f32::consts::TAU;
        let at = ground(15.0 * a.sin(), -15.0 * a.cos());
        // Two in three still stand.
        if hash2(k, 0, 0x7111) < 0.66 {
            column.place(&mut scene, at, a, 6.5);
        } else {
            broken.place(&mut scene, at, 6.3 * hash2(k, 1, 0x7111), 2.4);
        }
    }
    let tower = kit::prop(&mut scene, "watchtower");
    tower.place(&mut scene, ground(-34.0, -30.0) - Vec3::Y * 0.4, 0.8, 19.0);
    // Stones fallen about the plateau, the larger ones few.
    let boulder = kit::prop(&mut scene, "boulder");
    for k in 0..70 {
        let h = |i: i32| hash2(k, i, 0x57a1);
        let (d, a) = (20.0 + 90.0 * h(0), 6.3 * h(1));
        let (x, z) = (d * a.sin(), -8.0 + d * a.cos());
        let height = 0.5 + 3.0 * h(2) * h(2) * h(2);
        boulder.place(
            &mut scene,
            ground(x, z) - Vec3::Y * 0.25 * height,
            6.3 * h(3),
            height,
        );
    }
    // Stands of pines where the plateau begins to roll, and dry grass in
    // patches between the stones.
    let pine = kit::prop(&mut scene, "pine");
    for (s, (x, z, count)) in [(-58.0f32, 6.0f32, 9), (52.0, -44.0, 11), (30.0, 34.0, 5)]
        .into_iter()
        .enumerate()
    {
        for k in 0..count {
            let h = |i: i32| hash2(k, i, 0x91e5 + s as u32);
            let (d, a) = (16.0 * h(0).sqrt(), 6.3 * h(1));
            pine.place(
                &mut scene,
                ground(x + d * a.sin(), z + d * a.cos()) - Vec3::Y * 0.3,
                6.3 * h(2),
                8.0 + 7.0 * h(3),
            );
        }
    }
    let tuft = scene.add_mesh(grass_tuft());
    let dry = scene.add_material(Material {
        base_color: [0.5, 0.45, 0.19, 1.0],
        roughness: 0.9,
        ..Default::default()
    });
    scene.world.scatters.push(ScatterDraw {
        terrain: 0,
        mesh: tuft,
        material: dry,
        lod_materials: [None; 2],
        scatter: TerrainScatter {
            layer: 0,
            density: 5.0,
            distance: 170.0,
            fade: 0.2,
            scale: [0.8, 1.6],
            seed: 9,
            sway: 0.0,
            noise_scale: 14.0,
            noise_threshold: 0.6,
            ..Default::default()
        },
        cast_shadows: true,
        impostor: None,
        impostor_distance: 0.0,
        with: None,
        lods: [None; 2],
    });
    afternoon(&mut scene);
    // A low sun from the camera's left: every stone's shadow lies long.
    let sun = Vec3::new(-0.72, 0.42, 0.3).normalize();
    scene.lighting.light.direction_to_light = sun;
    scene.lighting.light.max_shadow_distance = 260.0;
    scene.lighting.environment.sky.sun_direction = sun;
    scene.frame.camera = Camera {
        projection: Projection::Perspective {
            fov_y_degrees: 42.0,
        },
        ..camera_at(Vec3::new(8.0, 36.0, 52.0), Vec3::new(-4.0, 1.0, -10.0))
    };
    scene
}

/// Where `steep_scene`'s cliff stands at z 0 (its foot along x) and its
/// height above the valley.
pub const STEEP_FOOT: f32 = 0.0;
pub const STEEP_HEIGHT: f32 = 60.0;

/// Where `steep_scene`'s camera stands along the cliff (z).
const STEEP_EYE_Z: f32 = 100.0;

/// `steep_scene`'s cliff line: where the wall's foot stands along x at
/// `z`. It bends round the valley (1 m further out every 16 m along z at
/// 64 m, so the cliff closes the view either way) and stands out in
/// buttresses 20 m proud between bays.
fn steep_foot(z: f32) -> f32 {
    STEEP_FOOT
        + z * z / 2048.0
        + 56.0 * fbm(z / 130.0, 0.3, 2, 77)
        + 12.0 * fbm(z / 30.0, 1.7, 2, 78)
}

/// The plateau's height above the valley at `z` (45 to 75 m).
fn steep_top(z: f32) -> f32 {
    STEEP_HEIGHT + 40.0 * fbm(z / 110.0, 4.4, 2, 79)
}

/// The scree's height against the wall at `z`: fans up to 20 m in some
/// bays, none in others.
fn steep_fan(z: f32) -> f32 {
    (9.0 + 30.0 * fbm(z / 55.0, 7.3, 2, 84)).max(0.0)
}

/// `steep_scene`'s ground: a valley for x past the cliff's foot, scree
/// fans against the wall falling at 32°, and an 85° wall up to the
/// plateau in three steps, the two ledges between them at heights and of
/// widths that change along it (each there over about half its length).
fn steep_height(x: f32, z: f32) -> f32 {
    let top = steep_top(z);
    let valley = 1.6 * fbm(x / 45.0, z / 45.0, 3, 83);
    // Metres into the wall from its foot.
    let t = steep_foot(z) - x;
    let scree = (steep_fan(z) - 0.62 * (-t).max(0.0)).max(0.0);
    let rubble = 1.4 * fbm(x / 4.0, z / 4.0, 2, 87) * smooth(0.0, 3.0, scree);
    let rise = 85f32.to_radians().tan();
    let ledges = [
        (
            0.34 + 0.3 * fbm(z / 50.0, 2.2, 2, 86),
            (16.0 * fbm(z / 45.0, 5.1, 2, 85) + 1.5).max(0.0),
        ),
        (
            0.7 + 0.22 * fbm(z / 38.0, 8.2, 2, 88),
            (14.0 * fbm(z / 52.0, 9.1, 2, 89) + 1.0).max(0.0),
        ),
    ];
    let (mut wall, mut left) = (0.0, t.max(0.0));
    for (at, width) in ledges {
        let up = (at * top - wall).max(0.0);
        if left <= up / rise {
            break;
        }
        wall += up;
        left -= up / rise;
        let on = left.min(width);
        wall += on * 0.3;
        left -= on;
    }
    wall += left * rise;
    valley + (scree + rubble + wall).min(top.max(scree))
}

/// Where `steep_scene`'s wall stands along x at `z`, `height` metres above
/// the valley (the first half metre in from its foot that reaches it).
fn steep_face(z: f32, height: f32) -> f32 {
    let foot = steep_foot(z);
    (0..160)
        .map(|i| foot - i as f32 * 0.5)
        .find(|&x| steep_height(x, z) >= height)
        .unwrap_or(foot - 80.0)
}

/// R7: a cliff 45 to 75 m high round a valley (`steep_height`: 85° walls
/// in steps, their face towards +x), seen along its foot from the valley
/// floor: buttresses and bays, ledges, scree fans with their stones, and
/// crags of rock standing out of the wall and over its edge (terrain
/// stamps, shaded by the terrain's layers), pines below for its size. Meadow (og.Grass) on the flat, scree on the
/// fans, rock on the walls. Sampled from above (`project_slope` 0) the rock
/// stretches down the walls in vertical streaks; past `project_slope`
/// the two projections facing a wall keep its cracks the shape they have
/// on a boulder.
pub fn steep_scene(project_slope: f32) -> RenderScene {
    use crate::kit;
    let mut scene = RenderScene::new();
    let n = 1025u32;
    let size = 512.0;
    let cell = size / (n - 1) as f32;
    let mut heights = Vec::with_capacity((n * n) as usize);
    for r in 0..n {
        for c in 0..n {
            heights.push(steep_height(
                c as f32 * cell - size * 0.5,
                r as f32 * cell - size * 0.5,
            ));
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
    let (grass, grass_normal) = kit::layer_maps(&mut scene, "grass");
    let (rock, rock_normal) = kit::layer_maps(&mut scene, "rock");
    let stone = TerrainLayer {
        albedo: Some(rock),
        normal: Some(rock_normal),
        roughness: 0.85,
        anti_tiling: 0.6,
        macro_variation: 0.5,
        slope_fade: 6.0,
        project_slope,
        displacement: 0.35,
        ..Default::default()
    };
    scene.world.terrains.push(Terrain {
        layers: vec![
            TerrainLayer {
                albedo: Some(grass),
                normal: Some(grass_normal),
                color: Vec3::new(0.72, 1.05, 0.5),
                roughness: 0.95,
                tile: 3.5,
                anti_tiling: 0.5,
                slope: [0.0, 24.0],
                slope_fade: 6.0,
                ..Default::default()
            },
            // The walls, and the undersides of the crags that overhang
            // them (a face turned down is past 90° from flat).
            TerrainLayer {
                color: Vec3::new(0.9, 0.86, 0.82),
                tile: V2_TILE * 2.5,
                slope: [44.0, 180.0],
                ..stone
            },
            // The scree and the ledges' edges: the same rock broken small.
            TerrainLayer {
                color: Vec3::new(1.05, 1.0, 0.96),
                tile: V2_TILE,
                slope: [24.0, 44.0],
                ..stone
            },
        ],
        height_blend: 0.6,
        displacement_distance: 70.0,
        ..Terrain::new(handle)
    });
    let ground = |x: f32, z: f32| Vec3::new(x, steep_height(x, z), z);
    let boulder = kit::prop(&mut scene, "boulder");
    let (rock_mesh, rock_material) = boulder.parts()[0];
    let unit = boulder.max - boulder.min;
    let mid = (boulder.min + boulder.max) * 0.5;
    // Crags: the boulder's shape as a terrain stamp `size` metres along
    // x, y and z, its middle at `at`, turned and tipped, taking the
    // ground's look over its lowest `blend` metres.
    let crag = |scene: &mut RenderScene, at: Vec3, size: Vec3, yaw: f32, tip: f32, blend: f32| {
        scene
            .world
            .terrain_stamps
            .push(og_render::TerrainStampDraw {
                terrain: 0,
                mesh: rock_mesh,
                transform: Mat4::from_translation(at)
                    * Mat4::from_rotation_y(yaw)
                    * Mat4::from_rotation_z(tip)
                    * Mat4::from_scale(size / unit)
                    * Mat4::from_translation(-mid),
                blend,
                cast_shadows: true,
            });
    };
    for k in 0..26 {
        let h = |i: i32| hash2(k, i, 0xc7a9);
        let z = -170.0 + 300.0 * (k as f32 + h(0)) / 26.0;
        let top = steep_top(z);
        // A third over the plateau's edge, the rest out of the walls.
        let level = if k % 3 == 0 { 0.96 } else { 0.18 + 0.66 * h(1) };
        let across = 13.0 + 20.0 * h(2);
        let size = Vec3::new(across * 0.7, 10.0 + 16.0 * h(3), across);
        let base = steep_height(steep_foot(z) + 30.0, z);
        // Its middle a fifth of its depth inside the wall.
        let at = Vec3::new(
            steep_face(z, base + level * top) - size.x * 0.2,
            base + level * top,
            z,
        );
        crag(&mut scene, at, size, 6.3 * h(4), 0.5 * (h(5) - 0.5), 1.5);
    }
    // Blocks fallen onto the fans and the valley's edge, half sunk.
    for k in 0..22 {
        let h = |i: i32| hash2(k, i, 0xb10c);
        let z = -150.0 + 270.0 * h(0);
        let x = steep_foot(z) + 2.0 + 30.0 * h(1) * h(1);
        let across = 2.5 + 7.0 * h(2) * h(2);
        let size = Vec3::new(
            across,
            across * (0.6 + 0.5 * h(3)),
            across * (0.8 + 0.5 * h(4)),
        );
        // Clear of where the camera stands.
        if (z - STEEP_EYE_Z).abs() < 16.0 && x - steep_foot(z) > 24.0 {
            continue;
        }
        let at = ground(x, z) + Vec3::Y * size.y * 0.18;
        crag(&mut scene, at, size, 6.3 * h(5), 0.6 * (h(6) - 0.5), 0.4);
    }
    // The scree's stones.
    scene.world.scatters.push(ScatterDraw {
        terrain: 0,
        mesh: rock_mesh,
        material: rock_material,
        lod_materials: [None; 2],
        scatter: TerrainScatter {
            layer: 2,
            density: 0.3,
            distance: 170.0,
            fade: 0.2,
            scale: [0.25 / unit.y, 1.3 / unit.y],
            align: 0.7,
            seed: 21,
            sway: 0.0,
            ..Default::default()
        },
        cast_shadows: true,
        impostor: None,
        impostor_distance: 0.0,
        with: None,
        lods: [None; 2],
    });
    // The valley: pines in stands clear of the fans.
    let pine = kit::prop(&mut scene, "pine");
    for k in 0..40 {
        let h = |i: i32| hash2(k, i, 0x57ee);
        let z = -160.0 + 260.0 * h(0);
        let x = steep_foot(z) + steep_fan(z) / 0.62 + 6.0 + 60.0 * h(1) * h(1);
        // Clear of the camera and its line along the wall.
        if (z - STEEP_EYE_Z).abs() < 34.0 && x - steep_foot(z) < 70.0 {
            continue;
        }
        pine.place(
            &mut scene,
            ground(x, z) - Vec3::Y * 0.3,
            h(3) * 6.3,
            10.0 + 7.0 * h(2),
        );
    }
    // The valley's meadow.
    scene.world.grass.push(og_render::GrassDraw {
        terrain: 0,
        grass: og_render::Grass {
            cast_shadows: true,
            ..Default::default()
        },
    });
    fair_afternoon(&mut scene);
    scene.frame.camera = camera_at(
        ground(steep_foot(STEEP_EYE_Z) + 42.0, STEEP_EYE_Z) + Vec3::Y * 1.8,
        Vec3::new(steep_foot(-40.0) - 4.0, STEEP_HEIGHT * 0.8, -40.0),
    );
    scene
}

/// `horizon_scene`'s ridge: where it stands along x, its height at its
/// highest and half-width (m), and the sun's elevation (degrees) from +x
/// behind it.
pub const HORIZON_RIDGE: f32 = 2000.0;
pub const HORIZON_RIDGE_HEIGHT: f32 = 300.0;
pub const HORIZON_RIDGE_WIDTH: f32 = 160.0;
pub const HORIZON_SUN: f32 = 8.0;
/// The hill `horizon_scene` is seen from: its top (x, z) and its height.
const HORIZON_HILL: (f32, f32, f32) = (-1500.0, 0.0, 300.0);

/// `horizon_scene`'s ground: a rolling plain, the ridge along z (its
/// crest between half its height and all of it, its flanks furrowed) and
/// the hill.
fn horizon_height(x: f32, z: f32) -> f32 {
    let d = (x - HORIZON_RIDGE) / HORIZON_RIDGE_WIDTH;
    let crest = 0.5 + 0.53 * ridged(z / 800.0, 0.7, 4, 91).min(0.94);
    let furrows = 0.8 + 0.4 * ridged(x / 260.0, z / 260.0, 3, 95);
    let (kx, kz, kh) = HORIZON_HILL;
    let k = Vec2::new(x - kx, z - kz).length() / 520.0;
    14.0 * fbm(x / 500.0, z / 500.0, 4, 93)
        + HORIZON_RIDGE_HEIGHT * (crest * furrows).min(1.0) * (-d * d).exp()
        + kh * (-k * k).exp()
}

/// T5: an 8 km plain with one ridge about 300 m high running along z at
/// x 2 km, the sun 8° up behind it (from +x), so the plain in front of it
/// lies in its shade for up to 2 km, the crest's outline along the
/// shade's edge; seen at sunset through a long lens from a hill as high
/// 3.5 km west of it, a ruined gate and pines on the hill's brow, the
/// plain and the ridge far past the sun's 400 m of shadow cascades. With the terrain's
/// horizon shadows the shade shows; without them only the cascades
/// shade, and the far plain is lit to the ridge's foot.
pub fn horizon_scene(horizon: bool) -> RenderScene {
    use crate::kit;
    let mut scene = RenderScene::new();
    let (n, size) = (1025u32, 8192.0f32);
    let cell = size / (n - 1) as f32;
    let mut heights = Vec::with_capacity((n * n) as usize);
    for r in 0..n {
        for c in 0..n {
            heights.push(horizon_height(
                c as f32 * cell - size * 0.5,
                r as f32 * cell - size * 0.5,
            ));
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
    let (grass, grass_normal) = kit::layer_maps(&mut scene, "grass");
    let (rock, rock_normal) = kit::layer_maps(&mut scene, "rock");
    scene.world.terrains.push(Terrain {
        layers: vec![
            TerrainLayer {
                albedo: Some(grass),
                normal: Some(grass_normal),
                color: Vec3::new(0.82, 1.0, 0.5),
                roughness: 0.95,
                tile: 3.5,
                anti_tiling: 0.5,
                macro_variation: 0.6,
                slope: [0.0, 30.0],
                slope_fade: 8.0,
                ..Default::default()
            },
            TerrainLayer {
                albedo: Some(rock),
                normal: Some(rock_normal),
                color: Vec3::new(0.86, 0.84, 0.82),
                roughness: 0.85,
                tile: 18.0,
                triplanar: true,
                anti_tiling: 0.5,
                macro_variation: 0.4,
                slope: [24.0, 90.0],
                slope_fade: 8.0,
                ..Default::default()
            },
        ],
        height_blend: 0.6,
        horizon_shadows: horizon,
        ..Terrain::new(handle)
    });
    // The brow's set, below the camera: the gate left of the view, pines
    // either side of it down the slope.
    let (kx, kz, _) = HORIZON_HILL;
    let ground = |ahead: f32, right: f32| {
        let (x, z) = (kx + ahead, kz + right);
        Vec3::new(x, horizon_height(x, z), z)
    };
    let arch = kit::prop(&mut scene, "arch");
    let column = kit::prop(&mut scene, "column");
    let broken = kit::prop(&mut scene, "column_broken");
    arch.place(&mut scene, ground(104.0, -15.0), 1.2, 9.5);
    column.place(&mut scene, ground(98.0, -23.0), 0.4, 7.0);
    broken.place(&mut scene, ground(110.0, -8.0), 2.1, 2.8);
    let boulder = kit::prop(&mut scene, "boulder");
    boulder.place(&mut scene, ground(92.0, 13.0) - Vec3::Y * 0.6, 0.9, 3.4);
    boulder.place(&mut scene, ground(124.0, 20.0) - Vec3::Y * 0.4, 2.6, 2.2);
    let pine = kit::prop(&mut scene, "pine");
    for k in 0..60 {
        let h = |i: i32| hash2(k, i, 0x4071);
        let side = if k % 2 == 0 { -1.0 } else { 1.0 };
        let ahead = 90.0 + 420.0 * h(0);
        let right = side * (22.0 + (ahead - 60.0) * 0.12 + 150.0 * h(1));
        pine.place(
            &mut scene,
            ground(ahead, right) - Vec3::Y * 0.3,
            h(2) * 6.3,
            10.0 + 7.0 * h(3),
        );
    }
    // The hill's meadow: its blades against the sun.
    scene.world.grass.push(og_render::GrassDraw {
        terrain: 0,
        grass: og_render::Grass {
            cast_shadows: true,
            ..Default::default()
        },
    });
    afternoon(&mut scene);
    let e = HORIZON_SUN.to_radians();
    scene.lighting.light.direction_to_light = Vec3::new(e.cos(), e.sin(), 0.0);
    fair_sky(&mut scene, 6.0 + HORIZON_SUN / 15.0);
    // A clear evening: thin haze, and no cloud's shadow on the plain.
    if let Some(fog) = &mut scene.atmosphere.fog {
        fog.density = 0.00003;
        fog.height_density = 0.0001;
    }
    if let Some(clouds) = &mut scene.atmosphere.clouds {
        clouds.coverage = 0.3;
        clouds.shadows = 0.0;
    }
    scene.frame.camera = Camera {
        projection: Projection::Perspective {
            fov_y_degrees: 40.0,
        },
        ..camera_at(
            ground(62.0, 0.0) + Vec3::Y * 1.8,
            Vec3::new(HORIZON_RIDGE, 0.0, 0.0),
        )
    };
    scene
}

/// `cliff_scene`'s cliff: its height, the rock's half-width along x (its
/// face stands out from x = CLIFF_ROCK towards +x by up to
/// [`CLIFF_RELIEF`]) and half-length along z (the terrain's).
pub const CLIFF_HEIGHT: f32 = 160.0;
pub const CLIFF_ROCK: f32 = 12.0;
pub const CLIFF_LENGTH: f32 = 512.0;
pub const CLIFF_RELIEF: f32 = 30.0;

/// How far `cliff_rock`'s face stands out of its block at height `y` and
/// `z` along it (4 to [`CLIFF_RELIEF`] m): the wall's line weaves in bays
/// and headlands, buttresses stand out between gullies and thin out up
/// the wall, and the rock breaks into blocks along joints 16 m apart.
fn cliff_relief(y: f32, z: f32) -> f32 {
    let bay = 10.0 * (fbm(z / 220.0, 0.2, 2, 129) + 0.5).clamp(0.0, 1.0);
    let fin = ridged(z / 70.0, y / 300.0, 3, 131);
    let buttress = 12.0 * fin * fin * (1.0 - 0.5 * y / CLIFF_HEIGHT);
    let bed = (y / 30.0 + 0.9 * fbm(z / 110.0, 0.4, 2, 133)).floor();
    let joint = (z / 16.0 + bed * 0.37).floor() as i32;
    let block = 4.0 * hash2(joint, bed as i32, 139);
    (4.0 + bay + buttress + block).clamp(0.0, CLIFF_RELIEF)
}

/// `cliff_scene`'s rock: a block 24 m thick, 1 km long and 163 m high
/// (from 1 m under the valley), its two long faces standing out by
/// [`cliff_relief`], the same on both, so the block is its own mirror
/// image across x. Its top and its ends are flat.
pub fn cliff_rock() -> MeshData {
    let (columns, rows) = (256u32, 54u32);
    let (low, high) = (-1.0, CLIFF_HEIGHT + 2.0);
    let at = |side: f32, c: u32, r: u32| {
        let z = -CLIFF_LENGTH + 2.0 * CLIFF_LENGTH * c as f32 / columns as f32;
        let y = low + (high - low) * r as f32 / rows as f32;
        Vec3::new(side * (CLIFF_ROCK + cliff_relief(y, z)), y, z)
    };
    let mut m = MeshData::default();
    // A triangle wound to face `out`, its corners new vertices or shared.
    let face = |m: &mut MeshData, ids: [u32; 3], out: Vec3| {
        let p = ids.map(|i| Vec3::from(m.vertices[i as usize].position));
        let n = (p[1] - p[0]).cross(p[2] - p[0]);
        let ids = if n.dot(out) >= 0.0 {
            ids
        } else {
            [ids[0], ids[2], ids[1]]
        };
        let n = n * n.dot(out).signum();
        for i in ids {
            let v = &mut m.vertices[i as usize];
            v.normal = (Vec3::from(v.normal) + n).into();
        }
        m.indices.extend(ids);
    };
    let vertex = |p: Vec3| Vertex {
        position: p.into(),
        normal: [0.0; 3],
        uv: [p.z / 8.0, p.y / 8.0],
        color: [1.0; 4],
        tangent: [0.0; 4],
    };
    for side in [1.0f32, -1.0] {
        let first = m.vertices.len() as u32;
        for r in 0..=rows {
            for c in 0..=columns {
                m.vertices.push(vertex(at(side, c, r)));
            }
        }
        let id = |c: u32, r: u32| first + r * (columns + 1) + c;
        for r in 0..rows {
            for c in 0..columns {
                let q = [id(c, r), id(c + 1, r), id(c + 1, r + 1), id(c, r + 1)];
                face(&mut m, [q[0], q[1], q[2]], Vec3::X * side);
                face(&mut m, [q[0], q[2], q[3]], Vec3::X * side);
            }
        }
    }
    // The top and the two ends: strips between the faces' edges.
    let strip = |m: &mut MeshData, edge: &dyn Fn(u32) -> (Vec3, Vec3), n: u32, out: Vec3| {
        let first = m.vertices.len() as u32;
        for i in 0..=n {
            let (a, b) = edge(i);
            m.vertices.push(vertex(a));
            m.vertices.push(vertex(b));
        }
        for i in 0..n {
            let q = first + i * 2;
            face(m, [q, q + 1, q + 3], out);
            face(m, [q, q + 3, q + 2], out);
        }
    };
    strip(
        &mut m,
        &|c| (at(1.0, c, rows), at(-1.0, c, rows)),
        columns,
        Vec3::Y,
    );
    strip(
        &mut m,
        &|r| (at(1.0, 0, r), at(-1.0, 0, r)),
        rows,
        Vec3::NEG_Z,
    );
    strip(
        &mut m,
        &|r| (at(1.0, columns, r), at(-1.0, columns, r)),
        rows,
        Vec3::Z,
    );
    for v in &mut m.vertices {
        v.normal = Vec3::from(v.normal).normalize_or(Vec3::Y).into();
    }
    m
}

/// The scree's height against `cliff_scene`'s wall at `z`: fans up to
/// 24 m in some bays, none in others.
fn cliff_fan(z: f32) -> f32 {
    (6.0 + 44.0 * fbm(z / 80.0, 3.1, 2, 143)).max(0.0)
}

/// `cliff_scene`'s ground: a plateau for x < 0 over a valley, one sample
/// apart (a height field's sawtooth wall), scree fans falling at 32°
/// from the rock's foot and the valley rolling a few metres past them.
fn cliff_ground(x: f32, z: f32) -> f32 {
    if x < 0.0 {
        return CLIFF_HEIGHT;
    }
    let foot = CLIFF_ROCK + cliff_relief(0.0, z);
    let scree = (cliff_fan(z) - 0.62 * (x - foot).max(0.0)).max(0.0);
    let valley = 5.0 * fbm(x / 90.0, z / 90.0, 3, 141) * smooth(foot + 10.0, foot + 80.0, x);
    scree + valley
}

/// T3: a 160 m cliff over a valley: a plateau above a meadow
/// ([`cliff_ground`]), and standing over the step a wall of rock as
/// terrain stamps: a block 24 m thick ([`cliff_rock`], stamp 0: bays,
/// buttresses and jointed blocks, its foot in the valley floor and its top
/// 2 m above the plateau) and crags of rock standing out of it, over its
/// edge and at its foot, what a height field cannot make. The ground under
/// the block is carved as og.terrain.stamp carves it (to the block's
/// underside less 0.25 m, on its edge to the underside). Every stamp is
/// shaded by the terrain's layers by its own normal (grass where it is
/// flat, rock on its faces and under its overhangs, broken rock between)
/// and, within `blend` metres above the ground it meets, takes the
/// ground's look. Seen from `view`: `wall`, from the valley 300 m off, the
/// whole height over the pines; `foot`, at eye height 30 m from where a
/// crag meets the meadow.
pub fn cliff_scene(blend: f32, view: &str) -> RenderScene {
    use crate::kit;
    let mut scene = RenderScene::new();
    let (n, size) = (513u32, 1024.0f32);
    let cell = size / (n - 1) as f32;
    let mut heights = Vec::with_capacity((n * n) as usize);
    for r in 0..n {
        for c in 0..n {
            let (x, z) = (c as f32 * cell - size * 0.5, r as f32 * cell - size * 0.5);
            let ground = cliff_ground(x, z);
            // As og.terrain.stamp carves: 0.25 m under the block's
            // underside inside it, the underside itself on its edge.
            heights.push(if x.abs() < CLIFF_ROCK {
                ground.min(-0.25)
            } else if x.abs() <= CLIFF_ROCK {
                ground.min(0.0)
            } else {
                ground
            });
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
    let (grass, grass_normal) = kit::layer_maps(&mut scene, "grass");
    let (rock, rock_normal) = kit::layer_maps(&mut scene, "rock");
    let stone = TerrainLayer {
        albedo: Some(rock),
        normal: Some(rock_normal),
        roughness: 0.85,
        anti_tiling: 0.6,
        macro_variation: 0.5,
        slope_fade: 6.0,
        project_slope: 45.0,
        ..Default::default()
    };
    scene.world.terrains.push(Terrain {
        layers: vec![
            TerrainLayer {
                albedo: Some(grass),
                normal: Some(grass_normal),
                color: Vec3::new(0.72, 1.05, 0.5),
                roughness: 0.95,
                tile: 3.5,
                anti_tiling: 0.5,
                macro_variation: 0.5,
                slope: [0.0, 26.0],
                slope_fade: 6.0,
                ..Default::default()
            },
            // The faces, and the undersides of what overhangs them (a
            // face turned down is past 90° from flat).
            TerrainLayer {
                color: Vec3::new(0.9, 0.86, 0.82),
                tile: 22.0,
                slope: [44.0, 180.0],
                ..stone
            },
            // The scree and the ledges' edges: the same rock broken small.
            TerrainLayer {
                color: Vec3::new(1.05, 1.0, 0.96),
                tile: 5.0,
                slope: [26.0, 44.0],
                ..stone
            },
        ],
        height_blend: 0.6,
        ..Terrain::new(handle)
    });
    let wall = scene.add_mesh(cliff_rock());
    scene
        .world
        .terrain_stamps
        .push(og_render::TerrainStampDraw {
            terrain: 0,
            mesh: wall,
            transform: Mat4::IDENTITY,
            blend,
            cast_shadows: true,
        });
    let ground = |x: f32, z: f32| Vec3::new(x, cliff_ground(x, z), z);
    let boulder = kit::prop(&mut scene, "boulder");
    let (rock_mesh, _) = boulder.parts()[0];
    let unit = boulder.max - boulder.min;
    let (mid, base) = ((boulder.min + boulder.max) * 0.5, boulder.min.y);
    // Crags: the boulder's shape as a terrain stamp `size` metres along
    // x, y and z, its flat base's middle at `at`, turned and tipped (by
    // π: hanging from its base).
    let crag = |scene: &mut RenderScene, at: Vec3, size: Vec3, yaw: f32, tip: f32| {
        scene
            .world
            .terrain_stamps
            .push(og_render::TerrainStampDraw {
                terrain: 0,
                mesh: rock_mesh,
                transform: Mat4::from_translation(at)
                    * Mat4::from_rotation_y(yaw)
                    * Mat4::from_rotation_z(tip)
                    * Mat4::from_scale(size / unit)
                    * Mat4::from_translation(Vec3::new(-mid.x, -base, -mid.z)),
                blend,
                cast_shadows: true,
            });
    };
    for k in 0..84 {
        let h = |i: i32| hash2(k, i, 0xc1a6);
        let z = -480.0 + 860.0 * (k as f32 + h(0)) / 84.0;
        // A fifth crown the plateau's edge; the rest stand out of the
        // wall, more of them low and the lower the larger.
        let level = if k % 5 == 0 { 1.0 } else { 0.85 * h(1) * h(1) };
        let across = 30.0 + 46.0 * h(2) * (1.0 - 0.5 * level);
        let size = Vec3::new(across * 0.6, across * (0.7 + 0.9 * h(3)), across);
        let y = level * CLIFF_HEIGHT;
        // Its middle a tenth of its depth out of the wall. The boulder's
        // base is flat: a crag is two of them base to base, the lower
        // hanging, so it is rock all round.
        let at = Vec3::new(CLIFF_ROCK + cliff_relief(y, z) + size.x * 0.1, y, z);
        let (yaw, tip) = (6.3 * h(4), 0.5 * (h(5) - 0.5));
        crag(&mut scene, at, size * Vec3::new(1.0, 0.6, 1.0), yaw, tip);
        let hung = size * Vec3::new(1.0, 0.4, 1.0);
        crag(&mut scene, at, hung, yaw, tip + std::f32::consts::PI);
    }
    // Where the camera stands: on the meadow, or by a crag's foot.
    let (eye, target) = match view {
        "foot" => (Vec2::new(77.0, -200.0), Vec3::new(40.0, 14.0, -290.0)),
        _ => (Vec2::new(300.0, 190.0), Vec3::new(30.0, 78.0, -70.0)),
    };
    // Blocks fallen onto the fans and the meadow's edge, half sunk, and
    // pines beyond them, clear of the camera.
    let pine = kit::prop(&mut scene, "pine");
    for k in 0..260 {
        let h = |i: i32| hash2(k, i, 0xb10c);
        let z = -470.0 + 800.0 * h(0);
        let foot = CLIFF_ROCK + cliff_relief(0.0, z);
        let fallen = k % 5 == 0;
        let x = foot
            + if fallen {
                4.0 + 50.0 * h(1) * h(1)
            } else {
                cliff_fan(z) / 0.62 + 12.0 + 330.0 * h(1) * h(1)
            };
        if Vec2::new(x, z).distance(eye) < 16.0 {
            continue;
        }
        if fallen {
            let across = 3.0 + 9.0 * h(2) * h(2);
            let size = Vec3::new(
                across,
                across * (0.6 + 0.5 * h(3)),
                across * (0.8 + 0.5 * h(4)),
            );
            crag(
                &mut scene,
                ground(x, z) - Vec3::Y * size.y * 0.32,
                size,
                6.3 * h(5),
                0.6 * (h(6) - 0.5),
            );
        } else {
            pine.place(
                &mut scene,
                ground(x, z) - Vec3::Y * 0.3,
                6.3 * h(3),
                10.0 + 7.0 * h(2),
            );
        }
    }
    // The valley's meadow, up to the rock's foot.
    scene.world.grass.push(og_render::GrassDraw {
        terrain: 0,
        grass: og_render::Grass {
            cast_shadows: true,
            ..Default::default()
        },
    });
    // The sun over the camera's shoulder, raking the wall: each buttress
    // and crag throws its shadow into the bay behind it.
    afternoon(&mut scene);
    scene.lighting.light.direction_to_light = Vec3::new(0.5, 0.5, 0.7).normalize();
    scene.lighting.light.max_shadow_distance = 700.0;
    fair_sky(&mut scene, 8.0);
    scene.frame.camera = camera_at(ground(eye.x, eye.y) + Vec3::Y * 1.8, target);
    scene
}

/// `cliff_block_scene`'s step and block: the plateau's height and the
/// block's half-length along z (its half-width is [`CLIFF_ROCK`], its face
/// towards +x flat at x = CLIFF_ROCK).
pub const BLOCK_HEIGHT: f32 = 300.0;
pub const BLOCK_LENGTH: f32 = 60.0;

/// The stamp tests' fixture: a 300 m step (a plateau for x < 0 over a
/// valley at 0, a height field's sawtooth wall) with a plain block 24 m
/// across standing over it as a terrain stamp, its face flat at x =
/// [`CLIFF_ROCK`], its foot at the valley floor and its top 2 m above the
/// plateau, the ground under it carved as og.terrain.stamp carves it. Two
/// flat-coloured layers: grass (0.16, 0.38, 0.08) to 40° of slope, rock
/// (0.42, 0.4, 0.37) past it, biplanar past 45°. Within `blend` metres
/// above the ground it meets the block takes the ground's look. No scene
/// draws it: what a test reads off its face needs the face where it is.
pub fn cliff_block_scene(blend: f32) -> RenderScene {
    let mut scene = RenderScene::new();
    let (n, size) = (513u32, 512.0f32);
    let cell = size / (n - 1) as f32;
    let mut heights = Vec::with_capacity((n * n) as usize);
    for r in 0..n {
        for c in 0..n {
            let (x, z) = (c as f32 * cell - size * 0.5, r as f32 * cell - size * 0.5);
            let ground = if x < 0.0 { BLOCK_HEIGHT } else { 0.0 };
            let on = x.abs() <= CLIFF_ROCK && z.abs() <= BLOCK_LENGTH;
            let inside = x.abs() < CLIFF_ROCK && z.abs() < BLOCK_LENGTH;
            heights.push(match (on, inside) {
                (_, true) => ground.min(-0.25),
                (true, false) => ground.min(0.0),
                _ => ground,
            });
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
    scene.world.terrains.push(Terrain {
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
        BLOCK_HEIGHT + 2.0,
        BLOCK_LENGTH * 2.0,
    )));
    scene
        .world
        .terrain_stamps
        .push(og_render::TerrainStampDraw {
            terrain: 0,
            mesh: rock,
            transform: Mat4::from_translation(Vec3::new(0.0, (BLOCK_HEIGHT + 2.0) * 0.5, 0.0)),
            blend,
            cast_shadows: true,
        });
    plain_light(&mut scene);
    // The sun from the valley's side, so the face and the ground are lit alike.
    scene.lighting.light.direction_to_light = Vec3::new(0.7, 0.65, 0.3).normalize();
    scene.lighting.environment.sky.sun_direction = scene.lighting.light.direction_to_light;
    scene.frame.camera = camera_at(
        Vec3::new(CLIFF_ROCK + 22.0, 5.0, 9.0),
        Vec3::new(CLIFF_ROCK, 0.6, 0.0),
    );
    scene
}

/// `displaced_scene`'s crest: where it runs along x (its z) and how fast
/// the ground falls away from it (m per m², a parabola).
pub const DISPLACED_CREST: f32 = -12.0;
pub const DISPLACED_FALL: f32 = 0.04;
/// The setts' displacement in `displaced_scene` (m).
pub const DISPLACED_RELIEF: f32 = 0.1;

/// The ground's height in `displaced_scene` at z.
pub fn displaced_height(z: f32) -> f32 {
    -DISPLACED_FALL * (z - DISPLACED_CREST) * (z - DISPLACED_CREST)
}

/// `displaced_scene`'s setts: metres per repeat of the texture, courses
/// per repeat (20 cm each), stones per course (25 cm), texels a side (a
/// millimetre each, so an arris and a pick's pit are drawn), and in
/// courses (2 cm each): the joint's width, the arris's chamfer and the
/// stone's wall (a texel: the side stands vertical).
const COBBLE_TILE: f32 = 1.0;
const COBBLE_COURSES: i32 = 5;
const COBBLE_PER_COURSE: i32 = 4;
const COBBLE_TEXELS: u32 = 1024;
const COBBLE_JOINT: f32 = 0.1;
const COBBLE_ARRIS: f32 = 0.025;
const COBBLE_WALL: f32 = 0.005;
/// The heights (0 to 1 of the displacement) of the mortar's fill, the
/// lowest and highest stone's top, and how far the arris's chamfer drops
/// below a top (45° over `COBBLE_ARRIS`: 5 mm of 10 cm).
const COBBLE_MORTAR: f32 = 0.3;
const COBBLE_TOP: [f32; 2] = [0.55, 0.75];
const COBBLE_CHAMFER: f32 = 0.05;

/// A point of the sett texture: its height (the mortar's fill in the
/// joints, each stone flat on top at its own height and tilt, pitted by
/// the dressing pick, its arris chamfered over a vertical side), how
/// much of it is stone, how deep a pit it lies in (0 to 1) and its
/// stone's id (0 to 1).
#[derive(Clone, Copy)]
struct Cobble {
    height: f32,
    stone: f32,
    pit: f32,
    id: f32,
}

/// The setts at texture coordinates (u, v), repeating: dressed stones
/// laid in courses along u, each course's joints moved along by its own
/// hash (so they never line up course to course) and each joint by a
/// little more, each stone a rounded rectangle (its signed distance,
/// Quílez's 2D box with rounded corners) whose edges a little noise
/// breaks, as hand-dressed edges are. A dressed sett's top is flat and
/// worked with a pick: pits a millimetre or two deep and 2 to 5 mm across
/// over a fine grain; its arris is a narrow chamfer that catches the light
/// over a near-vertical side, and the pits chip it here and there.
fn cobble(u: f32, v: f32) -> Cobble {
    let (n, k) = (COBBLE_COURSES, COBBLE_PER_COURSE);
    // Course units: a course is 1 tall; along it, k stones fill n units.
    let (x, y) = (u * n as f32, v * n as f32);
    let row = (y.floor() as i32).rem_euclid(n);
    let span = n as f32 / k as f32;
    let shift = hash2(row, 0, 91) * span;
    // The joints of this course, each moved by up to a fifth of a stone.
    let joint = |j: i32| {
        let w = j.rem_euclid(k);
        let wrap = (j - w) / k;
        (w as f32 + 0.4 * (hash2(row, w, 93) - 0.5)) * span + shift + (wrap * n) as f32
    };
    let mut j = ((x - shift) / span).floor() as i32 - 1;
    while joint(j + 1) <= x {
        j += 1;
    }
    while joint(j) > x {
        j -= 1;
    }
    let (x0, x1) = (joint(j), joint(j + 1));
    let stone_key = j.rem_euclid(k);
    let id = hash2(row, stone_key, 79);
    // The stone's rounded rectangle, the joint's half taken off each side.
    let centre = Vec2::new((x0 + x1) * 0.5, y.floor() + 0.5);
    let half = Vec2::new((x1 - x0) * 0.5, 0.5) - COBBLE_JOINT * 0.5;
    let r = 0.12;
    let q = (Vec2::new(x, y) - centre).abs() - half + r;
    let rough = 0.02 * (tiling_fbm(u, v, 32, 95) - 0.5);
    let d = q.max(Vec2::ZERO).length() + q.max_element().min(0.0) - r + rough;
    let inside = (-d).max(0.0);
    let tilt = Vec2::new(hash2(row, stone_key, 97), hash2(row, stone_key, 99)) - 0.5;
    let lean = 0.08 * tilt.dot((Vec2::new(x, y) - centre) / half.max(Vec2::splat(0.1)));
    let top = COBBLE_TOP[0] + (COBBLE_TOP[1] - COBBLE_TOP[0]) * id + lean;
    // The pick's pits: one possible pit per 6 mm cell (0.03 courses), each
    // at its own place and size, about one in seven struck (a third read as
    // pumice).
    let pit_cell = 0.03;
    let (px, py) = (x / pit_cell, y / pit_cell);
    let mut pit: f32 = 0.0;
    for dy in -1..=1 {
        for dx in -1..=1 {
            let (cx, cy) = (px.floor() as i32 + dx, py.floor() as i32 + dy);
            // (The cells wrap with the texture: n / pit_cell cells a repeat.)
            let period = (n as f32 / pit_cell).round() as i32;
            let (wx, wy) = (cx.rem_euclid(period), cy.rem_euclid(period));
            if hash2(wx, wy, 211) > 0.15 {
                continue;
            }
            let at = Vec2::new(
                cx as f32 + hash2(wx, wy, 223),
                cy as f32 + hash2(wx, wy, 227),
            );
            let radius = 0.15 + 0.2 * hash2(wx, wy, 229);
            let t = (Vec2::new(px, py) - at).length() / radius;
            if t < 1.0 {
                pit = pit.max((1.0 - t * t) * (0.5 + 0.5 * hash2(wx, wy, 233)));
            }
        }
    }
    // A fine grain over the worked face (a few tenths of a millimetre).
    let grain = tiling_fbm(u, v, 160, 241) - 0.5;
    // Flat on top, the arris a 45° chamfer (rounded a little where it meets
    // the top) over a side that stands vertical down to the mortar.
    let t = (inside / COBBLE_ARRIS).min(1.0);
    let chamfer = COBBLE_CHAMFER * (1.0 - t) * (1.0 - t * 0.35);
    let face = top - chamfer - 0.014 * pit + 0.006 * grain;
    // The mortar: a sandy fill lying a little higher against each stone.
    let edge = (COBBLE_JOINT * 0.5 - d.max(0.0)).max(0.0) / (COBBLE_JOINT * 0.5);
    let mortar = COBBLE_MORTAR - 0.03 * edge + 0.01 * (tiling_fbm(u, v, 96, 251) - 0.5);
    let wall = (inside / COBBLE_WALL).min(1.0);
    Cobble {
        height: mortar + (face - mortar) * wall,
        stone: wall,
        pit,
        id,
    }
}

/// `cobble` at every texel of the texture, row by row (made once).
fn cobbles() -> &'static [Cobble] {
    static GRID: OnceLock<Vec<Cobble>> = OnceLock::new();
    GRID.get_or_init(|| {
        let n = COBBLE_TEXELS;
        (0..n * n)
            .map(|i| {
                let (x, y) = (i % n, i / n);
                cobble((x as f32 + 0.5) / n as f32, (y as f32 + 0.5) / n as f32)
            })
            .collect()
    })
}

/// The sett texture's texel at (u, v) (texel centres, wrapping).
fn cobble_at(u: f32, v: f32) -> Cobble {
    let n = COBBLE_TEXELS as i32;
    let x = ((u * n as f32).floor() as i32).rem_euclid(n);
    let y = ((v * n as f32).floor() as i32).rem_euclid(n);
    cobbles()[(y * n + x) as usize]
}

/// T4: a 128 m terrain sampled every 50 cm, a gentle crest running along
/// x at z -12 (the ground falls away from it on both sides), one layer of
/// dressed setts 25 × 20 cm laid in courses with mortar joints (`cobble`):
/// its albedo,
/// its height in the albedo's alpha, `displacement` metres of it, and the
/// normal map of that relief; seen from 4 m before the crest,
/// 30 cm over the ground (34 cm under the crest), under a low sun from the
/// side: the crest is the skyline, and the setts throw shadows towards
/// the camera's right.
pub fn displaced_scene(displacement: f32) -> RenderScene {
    let mut scene = RenderScene::new();
    let (n, size) = (257u32, 128.0f32);
    let cell = size / (n - 1) as f32;
    let mut heights = Vec::with_capacity((n * n) as usize);
    for r in 0..n {
        let z = r as f32 * cell - size * 0.5;
        for _ in 0..n {
            heights.push(displaced_height(z));
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
    let albedo = scene.add_texture(texture(COBBLE_TEXELS, ColorSpace::Srgb, |u, v| {
        let c = cobble_at(u, v);
        // Each stone its own granite, grey to warm grey, speckled as granite
        // is at a millimetre (dark mica, pale quartz and feldspar), not
        // clouded; the pits hold a little grime. The mortar a sandy grey.
        let tone = [
            Vec3::new(0.29, 0.26, 0.23),
            Vec3::new(0.24, 0.235, 0.23),
            Vec3::new(0.31, 0.27, 0.235),
            Vec3::new(0.26, 0.24, 0.22),
            Vec3::new(0.21, 0.21, 0.215),
        ][(c.id * 5.0) as usize % 5];
        let n = COBBLE_TEXELS as f32;
        let fleck = noise2(u * n * 0.6, v * n * 0.6, 263);
        let speck = if fleck < 0.18 {
            0.55
        } else if fleck > 0.8 {
            1.35
        } else {
            0.92 + 0.16 * noise2(u * n * 0.25, v * n * 0.25, 269)
        };
        let worn = 0.95 + 0.1 * tiling_fbm(u, v, 8, 271);
        let stone = tone * speck * worn * (1.0 - 0.2 * c.pit);
        let sand = 0.85 + 0.3 * noise2(u * n * 0.4, v * n * 0.4, 277);
        let joint = Vec3::new(0.13, 0.115, 0.095) * sand;
        let col = joint.lerp(stone, c.stone);
        let srgb = |x: f32| (x.max(0.0).powf(1.0 / 2.2) * 255.0).min(255.0) as u8;
        [
            srgb(col.x),
            srgb(col.y),
            srgb(col.z),
            (c.height.clamp(0.0, 1.0) * 255.0) as u8,
        ]
    }));
    // The same relief as the height in alpha, for `displacement` metres of
    // it, differenced over a texel; its alpha the stone's id (the joint's
    // the stone it runs beside), for the layer's `cell_variation`.
    let normal = scene.add_texture(texture(COBBLE_TEXELS, ColorSpace::Linear, |u, v| {
        let e = 1.0 / COBBLE_TEXELS as f32;
        let per_metre = displacement.max(DISPLACED_RELIEF) / (2.0 * e * COBBLE_TILE);
        let dx = (cobble_at(u + e, v).height - cobble_at(u - e, v).height) * per_metre;
        let dy = (cobble_at(u, v + e).height - cobble_at(u, v - e).height) * per_metre;
        let n = Vec3::new(-dx, -dy, 1.0).normalize();
        [
            ((n.x * 0.5 + 0.5) * 255.0) as u8,
            ((n.y * 0.5 + 0.5) * 255.0) as u8,
            ((n.z * 0.5 + 0.5) * 255.0) as u8,
            (cobble_at(u, v).id * 255.0) as u8,
        ]
    }));
    scene.world.terrains.push(Terrain {
        layers: vec![TerrainLayer {
            albedo: Some(albedo),
            normal: Some(normal),
            color: Vec3::ONE,
            roughness: 0.85,
            tile: COBBLE_TILE,
            displacement,
            cell_variation: 1.0,
            ..Default::default()
        }],
        ..Terrain::new(handle)
    });
    plain_light(&mut scene);
    let sun = Vec3::new(-0.9, 0.35, 0.25);
    scene.lighting.light.direction_to_light = sun;
    scene.lighting.light.max_shadow_distance = 60.0;
    scene.lighting.environment.sky.sun_direction = sun;
    let eye_z = DISPLACED_CREST + 4.0;
    scene.frame.camera = camera_at(
        Vec3::new(0.0, displaced_height(eye_z) + 0.3, eye_z),
        Vec3::new(
            0.0,
            displaced_height(DISPLACED_CREST) - 0.2,
            DISPLACED_CREST,
        ),
    );
    scene
}

/// [`displaced_scene`]'s crest as a place: an old paved way over a hill
/// in the morning, the stumps and columns of a colonnade either side of
/// it on the crest, and past the crest (the ground falls away at once) the
/// top of a watchtower, the tips of pines and far hills. The setts before
/// the camera and the skyline they make are left clear: nothing stands or
/// throws its shadow there.
pub fn displaced_way() -> RenderScene {
    use crate::kit;
    let mut scene = displaced_scene(DISPLACED_RELIEF);
    let ground = |x: f32, z: f32| Vec3::new(x, displaced_height(z), z);
    let broken = kit::prop(&mut scene, "column_broken");
    let column = kit::prop(&mut scene, "column");
    let crest = DISPLACED_CREST;
    for (x, dz, height, standing) in [
        (-2.7f32, 0.6f32, 1.25f32, false),
        (3.0, 0.2, 1.0, false),
        (-6.4, -0.9, 3.3, true),
        (7.2, -1.4, 3.3, true),
        (-11.5, -2.2, 1.6, false),
        (12.4, -2.6, 3.3, true),
    ] {
        let prop = if standing { &column } else { &broken };
        prop.place(
            &mut scene,
            ground(x, crest + dz) - Vec3::Y * 0.05,
            x,
            height,
        );
    }
    let boulder = kit::prop(&mut scene, "boulder");
    boulder.place(
        &mut scene,
        ground(-4.6, crest - 2.4) - Vec3::Y * 0.2,
        1.3,
        0.9,
    );
    boulder.place(
        &mut scene,
        ground(5.3, crest - 3.4) - Vec3::Y * 0.2,
        4.1,
        0.7,
    );
    // Past the crest: what stands tall enough shows its top over it.
    let tower = kit::prop(&mut scene, "watchtower");
    tower.place(
        &mut scene,
        ground(4.5, crest - 16.0) - Vec3::Y * 0.5,
        0.6,
        18.0,
    );
    let pine = kit::prop(&mut scene, "pine");
    for k in 0..34 {
        let h = |i: i32| hash2(k, i, 0xd15b);
        let x = -40.0 + 80.0 * h(0);
        let z = crest - 9.0 - 12.0 * h(1);
        // The way's own line stays open to the hills.
        if x > -5.0 && x < 10.0 {
            continue;
        }
        pine.place(
            &mut scene,
            ground(x, z) - Vec3::Y * 0.3,
            6.3 * h(2),
            8.0 + 6.0 * h(3),
        );
    }
    kit::hills(&mut scene, None);
    // The sun stays where the bare crest has it, 20° up; the clouds stand
    // where their shadows leave the crest in the sun.
    fair_sky(&mut scene, 7.37);
    scene.frame.tick = 0;
    scene
}

/// `terrain-snow-track`'s footprints: one pressed each tick (a tick a
/// second), its frames.
pub const SNOW_TRACK_STEPS: u32 = 20;

/// The snowfield's gentle swell (m).
fn snowfield_height(x: f32, z: f32) -> f32 {
    0.35 * (x * 0.17).sin() * (z * 0.13 + 0.6).cos() + 0.2 * (x * 0.05 - z * 0.09).sin()
}

/// Where `terrain-snow-track`'s foot comes down on step `k`: left and right
/// of a line that weaves from the ruined gate towards the camera, 0.75 m a
/// stride.
fn snow_track_foot(k: u32) -> Vec3 {
    let z = -12.6 + 0.75 * k as f32;
    let x = 0.5 * (z * 0.21).sin() + if k % 2 == 0 { 0.2 } else { -0.2 };
    Vec3::new(x, snowfield_height(x, z), z)
}

/// A snowfield under a low morning sun after a night's fall (lying snow,
/// og.Weather's, on the ground, the stones, the ruin and the pines), and
/// the walker's own footprints behind them: from a ruined gate to where
/// the camera stands, an og.GrassPusher stepping once a tick
/// ([`snow_track_update`]), each footprint packed darker with its sides
/// shaded as a hollow, the first ones by the gate already filling in.
/// A watchtower and snowed hills beyond.
pub fn snow_track_scene() -> RenderScene {
    use crate::kit;
    let (size, samples) = (512.0f32, 257u32);
    let cell = size / (samples - 1) as f32;
    let heights: Vec<f32> = (0..samples * samples)
        .map(|i| {
            let (c, r) = ((i % samples) as f32, (i / samples) as f32);
            snowfield_height(-size / 2.0 + c * cell, -size / 2.0 + r * cell)
        })
        .collect();
    let mut scene = RenderScene::new();
    crate::sky_scenes::physical_light(&mut scene, 8.0, og_render::PhysicalAtmosphere::EARTH);
    let h = scene.add_terrain(TerrainData {
        heights: heights.into(),
        ..TerrainData::flat(Vec2::splat(size), samples, samples)
    });
    scene.world.terrains.push(Terrain {
        layers: vec![TerrainLayer {
            color: Vec3::new(0.32, 0.3, 0.27),
            roughness: 0.9,
            ..Default::default()
        }],
        fade_in: false,
        ..Terrain::new(h)
    });
    let ground = |x: f32, z: f32| Vec3::new(x, snowfield_height(x, z), z);
    // Stones by the trail, half under the snow.
    let boulder = kit::prop(&mut scene, "boulder");
    for (x, z, height, yaw) in [
        (-2.9f32, -1.6f32, 1.1f32, 0.4f32),
        (3.7, -5.2, 1.7, 2.1),
        (-5.6, -8.4, 2.3, 4.0),
        (2.2, 2.4, 0.7, 5.2),
    ] {
        boulder.place(
            &mut scene,
            ground(x, z) - Vec3::Y * 0.25 * height,
            yaw,
            height,
        );
    }
    // The ruined gate the trail comes through, and what is left of its
    // colonnade.
    let arch = kit::prop(&mut scene, "arch");
    arch.place(&mut scene, ground(0.3, -16.0), std::f32::consts::PI, 6.5);
    let column = kit::prop(&mut scene, "column");
    let broken = kit::prop(&mut scene, "column_broken");
    column.place(&mut scene, ground(-7.5, -18.5), 0.7, 5.2);
    broken.place(&mut scene, ground(-4.6, -15.2), 1.9, 2.0);
    broken.place(&mut scene, ground(6.2, -17.4), 3.3, 2.3);
    column.place(&mut scene, ground(9.5, -20.0), 5.0, 5.2);
    let tower = kit::prop(&mut scene, "watchtower");
    tower.place(&mut scene, ground(34.0, -78.0) - Vec3::Y * 0.4, 2.2, 22.0);
    // Pines either side of the trail's line and out to the hills.
    let pine = kit::prop(&mut scene, "pine");
    for k in 0..110 {
        let h = |i: i32| hash2(k, i, 0x5a0e);
        let (d, a) = (14.0 + 190.0 * h(0) * h(0), 6.3 * h(1));
        let (x, z) = (d * a.cos(), -20.0 + d * a.sin());
        // The trail's line and the view along it stay open, and so does
        // the sun's side of it (the camera's right): the footprints lie
        // in the light.
        let open = x.abs() < 5.0 + 0.12 * z.abs() && z < 12.0;
        let sunward = x > -4.0 && z > -14.0 && Vec2::new(x, z + 4.0).length() < 60.0;
        if open || sunward {
            continue;
        }
        pine.place(
            &mut scene,
            ground(x, z) - Vec3::Y * 0.3,
            6.3 * h(2),
            8.0 + 8.0 * h(3),
        );
    }
    kit::hills(&mut scene, None);
    scene.atmosphere.surface_weather = Some(og_render::SurfaceWeather {
        snow: 1.0,
        snow_depth: 0.1,
        shelter: 0.0,
        ..Default::default()
    });
    scene.frame.tick_rate = 1.0;
    scene.frame.camera = Camera {
        position: ground(0.9, 4.8) + Vec3::Y * 1.55,
        target: ground(0.0, -7.0) + Vec3::Y * 0.5,
        exposure: Exposure::Manual { ev: -0.3 },
        ..Default::default()
    };
    scene
}

/// `terrain-snow-track` at `frame`: tick `frame + 1`, the foot down on
/// step `frame`.
pub fn snow_track_update(scene: &mut RenderScene, frame: u32) {
    scene.frame.tick = u64::from(frame) + 1;
    scene.atmosphere.grass_pushers = vec![og_render::GrassPusher {
        position: snow_track_foot(frame),
        radius: 0.32,
        height: 1.8,
        strength: 1.0,
    }];
}

/// `terrain-scatter-unshadowed`: the island's meadow in the late
/// afternoon, the sun from the right, a field of heather, gold flowers and
/// long grass scattered on its grass layer with `cast_shadows` as given.
/// The gate, the boulders and the pines shadow the field; with
/// `cast_shadows` false (the scene) the plants throw none, so the ground
/// between them stays lit to their roots, as a game draws its undergrowth
/// (with it true the meadow's floor darkens under the blades).
pub fn evening_meadow_scene(cast_shadows: bool) -> RenderScene {
    let mut scene = island_view("near");
    scene.lighting.light.direction_to_light = Vec3::new(0.85, 0.42, -0.25);
    fair_sky(&mut scene, 7.6);
    scene.frame.tick = 0;
    let plant = scene.add_material(Material {
        base_color: [1.0; 4],
        roughness: 0.85,
        ..Default::default()
    });
    let green = scene.add_material(Material {
        base_color: [0.42, 0.46, 0.14, 1.0],
        roughness: 0.85,
        ..Default::default()
    });
    let tuft = scene.add_mesh(grass_tuft());
    let heather = scene.add_mesh(flower_clump([0.42, 0.12, 0.52], true));
    let gold = scene.add_mesh(flower_clump([0.95, 0.66, 0.05], false));
    for (mesh, material, density, scale, seed) in [
        (tuft, green, 3.0, [0.8, 1.5], 5),
        (heather, plant, 0.7, [0.9, 1.7], 41),
        (gold, plant, 0.9, [0.9, 1.4], 43),
    ] {
        scene.world.scatters.push(ScatterDraw {
            terrain: 0,
            mesh,
            material,
            lod_materials: [None; 2],
            scatter: TerrainScatter {
                layer: 0,
                density,
                distance: 90.0,
                fade: 0.3,
                scale,
                align: 0.3,
                seed,
                sway: 0.6,
                noise_scale: 9.0,
                noise_threshold: 0.35,
                ..Default::default()
            },
            cast_shadows,
            impostor: None,
            impostor_distance: 0.0,
            with: None,
            lods: [None; 2],
        });
    }
    scene
}
