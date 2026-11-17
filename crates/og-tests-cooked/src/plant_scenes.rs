//! Plants the cook grows (a `plant` block in a `type model`, og-geom's
//! `plant`) and a wood scattered from them: og-tests' assets/plants, the
//! models guide's kinds (each asset a `kind` and nothing else, two with a
//! colour of their own), cooked in a project of their own, the oak, the
//! pine and the shrub with their impostors.
//!
//! - `grown_scene()` (`plant-grown`): the oak, the pine and the shrub
//!   standing in a glade with a patch of daisies before the camera, each
//!   drawn as its cooked parts (the wood, then the leaves and petals with
//!   the leaf's material), a ruin's columns and boulders in the grass, a
//!   wood of the same cooked oaks and pines round it and the kit's hills
//!   beyond, under the fair afternoon's sky and a sun from the front right.
//! - `kinds_scene()` (`plant-kinds`): the kinds `plant-grown` does not
//!   show, planted in the garden of a ruin (a path of setts to its arch,
//!   a low wall, columns, the cooked wood beyond) under the same sky: a palm, a birch, a willow, three bamboo
//!   canes and a cactus in a row with an autumn birch behind (`kind
//!   birch` with its leaves' colours), and near the camera two ferns, two
//!   reeds and red and yellow tulips.
//! - `scatter_scene()` (`plant-scatter`): a hill 6 m high in rolling
//!   grassland (384 m of terrain, the kit's hills beyond, the fair
//!   afternoon's sky) with the three woody
//!   plants scattered whole, as og.TerrainScatter.model has og-runtime make
//!   them: one `ScatterDraw` per part, the parts after the first drawn
//!   `with` it. Oaks keep pines and shrubs out of their footprint; a
//!   plant's copies draw as its impostor past [`card_distance`] (the
//!   shrubs from 28 m, the pines from 81 m, no oak on a hill this small);
//!   a light wind sways each copy's wood and leaves as one plant.
//! - `switch_scene()` (`plant-scatter-switch`): oaks and pines on a flat
//!   strip 224 m long, seen along it from 23 m to 247 m, so both kinds
//!   cross from mesh to card in the frame (the pines at 81 m, the oaks at
//!   113 m). A card that is lit, shadowed or coloured unlike its mesh
//!   shows as a band across the strip.
//! - `toon_scene()` (`plant-scatter-toon`): `plant-scatter` under the
//!   anime cel look's `MaterialStyle` (two bands, a cool shadow tint, a
//!   2.5 px line outside the shape): the wood's leaves, cards and ground
//!   take the look's bands and its line as masses, as its meshes do.

use glam::{Mat4, Vec2, Vec3};
use og_cook::runtime::Bundle;
use og_render::{
    Camera, DirectionalLight, Grass, GrassDraw, Impostor, MaterialHandle, MaterialStyle,
    MeshHandle, OutlinePlace, Projection, RenderScene, ScatterDraw, StandardMaterial, Terrain,
    TerrainControl, TerrainData, TerrainLayer, TerrainScatter, TextureSource, Wind,
};
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

/// The plants cooked once per process: `(name, impostor)`.
const PLANTS: [(&str, bool); 14] = [
    ("oak", true),
    ("pine", true),
    ("bush", true),
    ("daisy", false),
    ("palm", false),
    ("birch", false),
    ("willow", false),
    ("bamboo", false),
    ("cactus", false),
    ("fern", false),
    ("reed", false),
    ("tulip", false),
    ("tulip-yellow", false),
    ("birch-autumn", false),
];

fn cooked() -> Result<&'static PathBuf, String> {
    static ROOT: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = std::env::temp_dir().join(format!("og-tests-plants-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let dir = root.join("assets/plants");
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        for (name, impostor) in PLANTS {
            let file = format!("{name}.ogt");
            std::fs::write(
                dir.join(&file),
                crate::scenes::asset(&format!("plants/{file}"))?,
            )
            .map_err(|e| format!("{file}: {e}"))?;
            if impostor {
                std::fs::write(
                    dir.join(format!("{file}.ogt")),
                    "type import\nversion 1\n---\nmodel {\n  impostor true\n}\n",
                )
                .map_err(|e| format!("{file}.ogt: {e}"))?;
            }
        }
        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default())
            .map_err(|e| format!("cook: {e}"))?;
        if !report.ok() {
            return Err(format!("cook: {:?}", report.errors()));
        }
        Ok(root)
    })
    .as_ref()
    .map_err(Clone::clone)
}

/// A cooked plant in a scene: its parts (the wood first) and its impostor
/// when it was cooked with one.
struct Plant {
    parts: Vec<(MeshHandle, MaterialHandle, Mat4)>,
    /// Per part, its mesh's coarser levels ([`ScatterDraw::lods`]): the
    /// cook's thinned leaves and simplified wood.
    levels: Vec<Levels>,
    impostor: Option<Impostor>,
}

type Levels = [Option<(MeshHandle, f32)>; 2];

fn plant(scene: &mut RenderScene, bundle: &Bundle, name: &str) -> Result<Plant, String> {
    let id = format!("plants/{name}.ogt");
    let model = bundle.load_model(&id).map_err(|e| format!("{id}: {e}"))?;
    let texture = |t: &str| {
        bundle
            .load_texture(t)
            .and_then(|t| t.to_texture_data())
            .map_err(|e| format!("{t}: {e}"))
    };
    let textures: Vec<_> = model
        .textures
        .iter()
        .map(|t| texture(&t.id))
        .collect::<Result<_, _>>()?;
    let sources: Vec<_> = textures.into_iter().map(TextureSource::Rgba).collect();
    let (added, meshes) = model.add_to_shared(scene, &sources, None);
    let parts = added.parts;
    // Each part's own levels, as og-runtime gives a scattered model whose
    // parts stand as made (terrain.rs `extract`).
    let coarser = model.add_lod_levels(scene);
    let levels = parts
        .iter()
        .map(|&(mesh, ..)| {
            let mut levels = [None; 2];
            let own = meshes.iter().position(|&m| m == mesh);
            for (to, &level) in levels.iter_mut().zip(own.iter().flat_map(|&p| &coarser[p])) {
                *to = Some(level);
            }
            levels
        })
        .collect();
    // The card, as og-runtime makes it (assets.rs `insert_model`).
    let impostor = match &model.impostor {
        Some(i) => {
            let albedo = scene.add_texture(texture(&i.albedo)?);
            let normal = scene.add_texture(texture(&i.normal)?);
            let atlas = match &i.material {
                Some(m) => Some(scene.add_texture(texture(m)?)),
                None => None,
            };
            let material = scene.add_standard_material(StandardMaterial {
                map: albedo.slot(),
                normalMap: normal.slot(),
                roughnessMap: atlas.map_or(0, |a| a.slot()),
                alphaTest: 0.5,
                side: 2,
                roughness: 0.85,
                ..Default::default()
            });
            Some(Impostor {
                material,
                grid: i.grid.clamp(2, og_render::MAX_IMPOSTOR_GRID),
                center: i.center,
                radius: i.radius,
                material_atlas: atlas.is_some(),
                below: i.below,
            })
        }
        None => None,
    };
    Ok(Plant {
        parts,
        levels,
        impostor,
    })
}

/// Where a card's texel covers one pixel of the capture (720 rows, 50
/// degrees of them, the cook's 128 texel frame across the sphere): the
/// nearest a card is as sharp as its mesh, 12 radii here (the models
/// guide's rule for impostorDistance).
fn card_distance(impostor: Impostor) -> f32 {
    const ROWS: f32 = 720.0;
    const FRAME: f32 = 128.0;
    impostor.radius * ROWS / (FRAME * 25f32.to_radians().tan())
}

fn sun(scene: &mut RenderScene, direction: Vec3, max_shadow_distance: f32) {
    let direction = direction.normalize();
    scene.lighting.light = DirectionalLight {
        direction_to_light: direction,
        intensity: 3.0,
        max_shadow_distance,
        ..Default::default()
    };
    scene.lighting.environment.sky.sun_direction = direction;
    scene.lighting.environment.intensity = 0.7;
}

/// Where `plant-grown` stands its oak, pine and shrub (their roots).
pub const GROWN_AT: [Vec3; 3] = [
    Vec3::new(-7.5, 0.0, 0.0),
    Vec3::new(1.5, 0.0, -1.0),
    Vec3::new(7.5, 0.0, 2.0),
];

/// A glade's ground: a terrain 384 m square of the kit's grass, level
/// within 60 m of the middle and easing down 2.5 m by its edge to meet the
/// kit's hills, short og.Grass on it, under the fair afternoon's physical
/// sky with its sun turned to `sun` (toward the light). With `path` its
/// control map lays a path of setts 2.8 m wide along z from z = -16 to the
/// camera, where no grass grows.
fn glade(scene: &mut RenderScene, sun: Vec3, max_shadow_distance: f32, path: bool) {
    use og_tests::kit;
    let (n, size) = (193u32, 384.0f32);
    let cell = size / (n - 1) as f32;
    let height = |x: f32, z: f32| {
        let d = (x * x + z * z).sqrt();
        let t = ((d - 60.0) / 130.0).clamp(0.0, 1.0);
        -2.5 * t * t * (3.0 - 2.0 * t) + 0.25 * (x * 0.07).sin() * (z * 0.05).cos() * t
    };
    let heights: Vec<f32> = (0..n * n)
        .map(|i| {
            let (c, r) = ((i % n) as f32, (i / n) as f32);
            height(c * cell - size * 0.5, r * cell - size * 0.5)
        })
        .collect();
    // The path's control map (0.375 m a texel), its edges ragged.
    let control = path.then(|| {
        let m = 1024u32;
        let mut rgba = Vec::with_capacity((m * m * 4) as usize);
        for y in 0..m {
            for x in 0..m {
                let at = (Vec2::new(x as f32, y as f32) + 0.5) / m as f32 * size - size * 0.5;
                let ragged = 0.35 * (at.y * 1.7).sin() * (at.y * 0.63 + 1.0).cos();
                let across = ((1.6 - at.x.abs() + ragged) / 0.5).clamp(0.0, 1.0);
                let along = ((at.y + 16.0) / 0.8).clamp(0.0, 1.0);
                let paved = (across * along * 255.0) as u8;
                rgba.extend_from_slice(&[255 - paved, paved, 0, 0]);
            }
        }
        TerrainControl {
            width: m,
            height: m,
            rgba: Arc::from(rgba),
        }
    });
    let h = scene.add_terrain(TerrainData {
        heights: Arc::from(heights),
        control,
        ..TerrainData::flat(Vec2::splat(size), n, n)
    });
    let (grass, grass_normal) = kit::layer_maps(scene, "grass");
    let mut layers = vec![TerrainLayer {
        albedo: Some(grass),
        normal: Some(grass_normal),
        color: Vec3::new(0.72, 1.05, 0.5),
        roughness: 0.95,
        tile: 3.5,
        anti_tiling: 0.5,
        macro_variation: 0.5,
        ..Default::default()
    }];
    if path {
        let (setts, setts_normal) = kit::layer_maps(scene, "cobbles");
        layers.push(TerrainLayer {
            albedo: Some(setts),
            normal: Some(setts_normal),
            color: Vec3::new(1.15, 1.08, 0.92),
            roughness: 0.85,
            tile: 2.5,
            ..Default::default()
        });
    }
    scene.world.terrains.push(Terrain {
        layers,
        height_blend: if path { 0.5 } else { 0.0 },
        fade_in: false,
        ..Terrain::new(h)
    });
    scene.world.grass.push(GrassDraw {
        terrain: 0,
        grass: Grass {
            height: 0.28,
            cast_shadows: true,
            ..Default::default()
        },
    });
    kit::hills(scene, None);
    og_tests::terrain_scenes::afternoon(scene);
    scene.lighting.light.direction_to_light = sun.normalize();
    scene.lighting.light.max_shadow_distance = max_shadow_distance;
    og_tests::terrain_scenes::fair_sky(scene, 8.6);
    scene.frame.tick = 0;
}

/// A number in 0..1 for `k` and `i`, the same on every run.
fn hash(k: u32, i: u32) -> f32 {
    let mut h = k.wrapping_mul(0x8da6_b343) ^ i.wrapping_mul(0xd816_3841) ^ 0x51ed;
    h = (h ^ (h >> 13)).wrapping_mul(0x85eb_ca6b);
    h ^= h >> 16;
    (h & 0xff_ffff) as f32 / 16_777_216.0
}

/// `plant-grown` ([module docs](self)).
pub fn grown_scene() -> Result<RenderScene, String> {
    use og_tests::kit;
    let bundle = Bundle::open(&cooked()?.join("cooked")).map_err(|e| e.to_string())?;
    let mut scene = RenderScene::new();
    glade(&mut scene, Vec3::new(0.5, 0.7, 0.45), 60.0, false);
    for (name, at) in ["oak", "pine", "bush"].into_iter().zip(GROWN_AT) {
        let p = plant(&mut scene, &bundle, name)?;
        for (mesh, material, local) in p.parts {
            scene.add_instance(mesh, material, Mat4::from_translation(at) * local);
        }
    }
    // The daisies: a patch in the grass before the camera.
    let daisy = plant(&mut scene, &bundle, "daisy")?;
    for k in 0..46 {
        let (d, a) = (2.2 * hash(k, 1).sqrt(), 6.3 * hash(k, 2));
        let at = Mat4::from_translation(Vec3::new(3.0 + d * a.cos(), 0.0, 15.5 + d * a.sin()))
            * Mat4::from_rotation_y(6.3 * hash(k, 3))
            * Mat4::from_scale(Vec3::splat(1.2 + 0.5 * hash(k, 4)));
        for &(mesh, material, local) in &daisy.parts {
            scene.add_instance(mesh, material, at * local);
        }
    }
    // A ruin at the glade's edge behind the trees, boulders in the grass
    // and a dark wood closing it in.
    let column = kit::prop(&mut scene, "column");
    let broken = kit::prop(&mut scene, "column_broken");
    column.place(&mut scene, Vec3::new(-16.0, -0.1, -14.0), 0.3, 6.5);
    broken.place(&mut scene, Vec3::new(-11.5, -0.1, -16.0), 1.9, 2.6);
    column.place(&mut scene, Vec3::new(14.0, -0.1, -13.0), 2.2, 6.5);
    let boulder = kit::prop(&mut scene, "boulder");
    for (at, yaw, size) in [
        (Vec3::new(-3.0, -0.3, 6.0), 0.4, 1.3),
        (Vec3::new(10.5, -0.4, 6.5), 2.1, 1.8),
        (Vec3::new(-12.0, -0.3, 3.0), 4.0, 1.1),
    ] {
        boulder.place(&mut scene, at, yaw, size);
    }
    // The wood round the glade: the cook's oaks and pines, in clumps.
    let wood = [
        plant(&mut scene, &bundle, "oak")?,
        plant(&mut scene, &bundle, "pine")?,
    ];
    for k in 0..130 {
        let a = -3.0 + 2.8 * hash(k, 1);
        let d = 34.0 + 50.0 * hash(k, 2) * hash(k, 2);
        let at = Mat4::from_translation(Vec3::new(d * a.cos(), -0.1, d * a.sin()))
            * Mat4::from_rotation_y(6.3 * hash(k, 3))
            * Mat4::from_scale(Vec3::splat(0.8 + 0.5 * hash(k, 4)));
        for &(mesh, material, local) in &wood[(hash(k, 5) < 0.55) as usize].parts {
            scene.add_instance(mesh, material, at * local);
        }
    }
    scene.frame.camera = Camera {
        position: Vec3::new(1.0, 2.2, 25.0),
        target: Vec3::new(0.0, 5.2, 0.0),
        far: 4000.0,
        projection: Projection::Perspective {
            fov_y_degrees: 50.0,
        },
        exposure: og_render::Exposure::linear(0.9),
        ..Default::default()
    };
    Ok(scene)
}

/// `plant-kinds` ([module docs](self)).
pub fn kinds_scene() -> Result<RenderScene, String> {
    let bundle = Bundle::open(&cooked()?.join("cooked")).map_err(|e| e.to_string())?;
    use og_tests::kit;
    let mut scene = RenderScene::new();
    // The garden of a ruin: a path of setts through its beds to the arch,
    // a low wall of stone behind, columns either side.
    glade(&mut scene, Vec3::new(0.5, 0.7, 0.45), 60.0, true);
    let stone = kit::surface_tinted(&mut scene, "cobbles", [1.0, 0.93, 0.8]);
    for (x0, x1) in [(-26.0f32, -3.0f32), (3.0, 24.0)] {
        let wall = scene.add_mesh(kit::block(
            Vec3::new(x0, -0.3, -15.0),
            Vec3::new(x1, 1.4, -14.2),
            2.5,
        ));
        scene.add_instance(wall, stone, Mat4::IDENTITY);
    }
    kit::prop(&mut scene, "arch").place(&mut scene, Vec3::new(0.0, -0.1, -14.6), 0.0, 7.0);
    let column = kit::prop(&mut scene, "column");
    let broken = kit::prop(&mut scene, "column_broken");
    column.place(&mut scene, Vec3::new(-15.5, -0.1, -4.0), 0.4, 6.0);
    broken.place(&mut scene, Vec3::new(14.5, -0.1, -3.0), 2.0, 2.3);
    column.place(&mut scene, Vec3::new(16.0, -0.1, 9.0), 1.1, 6.0);
    let boulder = kit::prop(&mut scene, "boulder");
    for (at, yaw, size) in [
        (Vec3::new(-6.5, -0.2, 13.5), 0.7, 0.9),
        (Vec3::new(9.5, -0.3, 11.0), 2.5, 1.4),
    ] {
        boulder.place(&mut scene, at, yaw, size);
    }
    // The wood beyond the wall.
    let wood = [
        plant(&mut scene, &bundle, "oak")?,
        plant(&mut scene, &bundle, "pine")?,
    ];
    for k in 0..120 {
        let a = -3.0 + 2.8 * hash(k, 11);
        let d = 36.0 + 50.0 * hash(k, 12) * hash(k, 12);
        let at = Mat4::from_translation(Vec3::new(d * a.cos(), -0.1, d * a.sin()))
            * Mat4::from_rotation_y(6.3 * hash(k, 13))
            * Mat4::from_scale(Vec3::splat(0.8 + 0.5 * hash(k, 14)));
        for &(mesh, material, local) in &wood[(hash(k, 15) < 0.5) as usize].parts {
            scene.add_instance(mesh, material, at * local);
        }
    }
    let stand: [(&str, Vec3); 15] = [
        ("palm", Vec3::new(-11.0, 0.0, 0.0)),
        ("birch", Vec3::new(-5.5, 0.0, -1.0)),
        ("willow", Vec3::new(1.5, 0.0, -1.0)),
        ("bamboo", Vec3::new(7.0, 0.0, 2.0)),
        ("bamboo", Vec3::new(7.8, 0.0, 1.2)),
        ("bamboo", Vec3::new(8.4, 0.0, 2.4)),
        ("cactus", Vec3::new(11.5, 0.0, 6.0)),
        ("fern", Vec3::new(-3.7, 0.0, 16.2)),
        ("fern", Vec3::new(-3.0, 0.0, 15.0)),
        ("reed", Vec3::new(-6.2, 0.0, 15.2)),
        ("reed", Vec3::new(-5.2, 0.0, 14.2)),
        ("tulip", Vec3::new(2.2, 0.0, 16.4)),
        ("tulip-yellow", Vec3::new(3.9, 0.0, 16.2)),
        ("tulip-yellow", Vec3::new(4.3, 0.0, 15.7)),
        ("birch-autumn", Vec3::new(5.0, 0.0, -9.0)),
    ];
    for (k, (name, at)) in stand.into_iter().enumerate() {
        let p = plant(&mut scene, &bundle, name)?;
        let at = Mat4::from_translation(at) * Mat4::from_rotation_y(1.3 * k as f32);
        for (mesh, material, local) in p.parts {
            scene.add_instance(mesh, material, at * local);
        }
    }
    // A bed of red and yellow tulips beside the path.
    let tulips = [
        plant(&mut scene, &bundle, "tulip")?,
        plant(&mut scene, &bundle, "tulip-yellow")?,
    ];
    for k in 0..60 {
        let at = Mat4::from_translation(Vec3::new(
            2.4 + 3.2 * hash(k, 21),
            0.0,
            12.5 + 4.5 * hash(k, 22),
        )) * Mat4::from_rotation_y(6.3 * hash(k, 23));
        for &(mesh, material, local) in &tulips[(hash(k, 24) < 0.4) as usize].parts {
            scene.add_instance(mesh, material, at * local);
        }
    }
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 2.2, 21.0),
        target: Vec3::new(0.0, 3.6, 0.0),
        far: 4000.0,
        exposure: og_render::Exposure::linear(0.9),
        projection: Projection::Perspective {
            fov_y_degrees: 58.0,
        },
        ..Default::default()
    };
    Ok(scene)
}

/// `plant-scatter`'s ground: 64 m square, a hill 6 m high on a slope that
/// rises 2 m to the north.
pub fn hill_height(x: f32, z: f32) -> f32 {
    1.0 - z / 32.0 + 6.0 * (-(x * x + z * z) / 320.0).exp()
}

/// The hill and the land round it, 384 m square, out to where the kit's
/// hills take over.
fn hill() -> TerrainData {
    let (n, size) = (385u32, 384.0f32);
    let at = |i: u32| (i as f32 / (n - 1) as f32 - 0.5) * size;
    let heights: Vec<f32> = (0..n * n)
        .map(|i| hill_height(at(i % n), at(i / n)))
        .collect();
    // The control map (0.75 m a texel): the meadow (layer 1, where nothing
    // is scattered) south of a ragged line 40 m from the hill's top, the
    // wood's ground (layer 0) north of it.
    let m = 512u32;
    let mut rgba = Vec::with_capacity((m * m * 4) as usize);
    for y in 0..m {
        for x in 0..m {
            let p = (Vec2::new(x as f32, y as f32) + 0.5) / m as f32 * size - size * 0.5;
            let edge = 40.0 + 6.0 * (p.x * 0.11).sin() + 3.0 * (p.x * 0.37 + 1.0).cos();
            let meadow = ((p.y - edge) / 3.0).clamp(0.0, 1.0);
            let w = (meadow * 255.0) as u8;
            rgba.extend_from_slice(&[255 - w, w, 0, 0]);
        }
    }
    TerrainData {
        heights: Arc::from(heights),
        control: Some(TerrainControl {
            width: m,
            height: m,
            rgba: Arc::from(rgba),
        }),
        ..TerrainData::flat(Vec2::splat(size), n, n)
    }
}

/// `plant-scatter` ([module docs](self)).
pub fn scatter_scene() -> Result<RenderScene, String> {
    let bundle = Bundle::open(&cooked()?.join("cooked")).map_err(|e| e.to_string())?;
    use og_tests::kit;
    let mut scene = RenderScene::new();
    let h = scene.add_terrain(hill());
    let (grass, grass_normal) = kit::layer_maps(&mut scene, "grass");
    let wood_floor = TerrainLayer {
        albedo: Some(grass),
        normal: Some(grass_normal),
        color: Vec3::new(0.62, 0.9, 0.42),
        roughness: 0.95,
        tile: 3.5,
        anti_tiling: 0.5,
        macro_variation: 0.5,
        ..Default::default()
    };
    scene.world.terrains.push(Terrain {
        layers: vec![
            wood_floor,
            TerrainLayer {
                color: Vec3::new(0.78, 1.05, 0.52),
                ..wood_floor
            },
        ],
        fade_in: false,
        ..Terrain::new(h)
    });
    scene.world.grass.push(GrassDraw {
        terrain: 0,
        grass: Grass {
            layer: 1,
            height: 0.4,
            dryness: 0.1,
            cast_shadows: true,
            ..Default::default()
        },
    });
    kit::hills(&mut scene, None);
    og_tests::terrain_scenes::afternoon(&mut scene);
    scene.lighting.light.direction_to_light = Vec3::new(0.5, 0.7, 0.45).normalize();
    scene.lighting.light.max_shadow_distance = 160.0;
    og_tests::terrain_scenes::fair_sky(&mut scene, 8.6);
    // (plant, copies a square metre, seed, priority, footprint in metres,
    // the share of the ground its noise leaves bare).
    for (name, density, seed, priority, footprint, bare) in [
        ("oak", 0.012, 3, 3, 5.0, 0.5),
        ("pine", 0.016, 5, 2, 4.0, 0.55),
        ("bush", 0.03, 9, 1, 2.0, 0.4),
    ] {
        let p = plant(&mut scene, &bundle, name)?;
        let scatter = TerrainScatter {
            density,
            distance: 200.0,
            fade: 0.1,
            scale: [0.7, 1.15],
            seed,
            priority,
            footprint,
            noise_scale: 14.0,
            noise_threshold: bare,
            sway: 1.0,
            ..TerrainScatter::default()
        };
        // A model's parts are one copy: the first places it, the rest
        // draw with it (og-runtime's terrain.rs `extract`).
        let first = scene.world.scatters.len();
        for (k, (mesh, material, local)) in p.parts.into_iter().enumerate() {
            // A scatter draws a part as it is made: a grown plant's stand
            // at its origin (og-runtime skips a model whose nodes move
            // its parts and that has no impostor).
            if local != Mat4::IDENTITY {
                return Err(format!("{name}: a part is moved by its node"));
            }
            scene.world.scatters.push(ScatterDraw {
                terrain: 0,
                mesh,
                material,
                scatter,
                cast_shadows: true,
                impostor: p.impostor,
                impostor_distance: p.impostor.map_or(0.0, card_distance),
                with: (k > 0).then_some(first),
                lods: p.levels[k],
            });
        }
    }
    scene.atmosphere.wind = Some(Wind {
        strength: 0.06,
        direction: Vec3::new(1.0, 0.0, 0.3).normalize(),
        ..Wind::default()
    });
    scene.frame.tick = 90;
    scene.frame.camera = Camera {
        position: Vec3::new(-14.0, 4.5, 92.0),
        target: Vec3::new(0.0, 8.0, 0.0),
        far: 4000.0,
        exposure: og_render::Exposure::linear(0.9),
        projection: Projection::Perspective {
            fov_y_degrees: 50.0,
        },
        ..Default::default()
    };
    Ok(scene)
}

/// `plant-scatter-toon` ([module docs](self)).
pub fn toon_scene() -> Result<RenderScene, String> {
    let mut scene = scatter_scene()?;
    // templates/looks/anime-cel-3d.ogt, its colours linear.
    scene.look.stylise.material = MaterialStyle {
        toon: true,
        shadow_tint: [0.0232, 0.0296, 0.117],
        shadow_tint_amount: 0.7,
        outline_color: [0.007, 0.0052, 0.0212],
        outline_width: 2.5,
        outline_distance: 10.0,
        outline_place: OutlinePlace::Outside,
        ..Default::default()
    };
    Ok(scene)
}

/// `plant-scatter-switch` ([module docs](self)).
pub fn switch_scene() -> Result<RenderScene, String> {
    let bundle = Bundle::open(&cooked()?.join("cooked")).map_err(|e| e.to_string())?;
    let mut scene = RenderScene::new();
    sun(&mut scene, Vec3::new(0.5, 0.7, 0.45), 260.0);
    let h = scene.add_terrain(TerrainData::flat(Vec2::new(64.0, 224.0), 33, 113));
    scene.world.terrains.push(Terrain {
        layers: vec![TerrainLayer {
            color: Vec3::new(0.25, 0.36, 0.16),
            ..Default::default()
        }],
        fade_in: false,
        ..Terrain::new(h)
    });
    for (name, density, seed, priority, footprint) in
        [("oak", 0.005, 3, 3, 6.0), ("pine", 0.006, 5, 2, 4.5)]
    {
        let p = plant(&mut scene, &bundle, name)?;
        let scatter = TerrainScatter {
            density,
            distance: 400.0,
            fade: 0.1,
            scale: [0.85, 1.1],
            seed,
            priority,
            footprint,
            sway: 1.0,
            ..TerrainScatter::default()
        };
        let first = scene.world.scatters.len();
        for (k, (mesh, material, _)) in p.parts.into_iter().enumerate() {
            scene.world.scatters.push(ScatterDraw {
                terrain: 0,
                mesh,
                material,
                scatter,
                cast_shadows: true,
                impostor: p.impostor,
                impostor_distance: p.impostor.map_or(0.0, card_distance),
                with: (k > 0).then_some(first),
                lods: p.levels[k],
            });
        }
    }
    scene.frame.tick = 90;
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 20.0, 135.0),
        target: Vec3::new(0.0, 2.0, 40.0),
        far: 600.0,
        projection: Projection::Perspective {
            fov_y_degrees: 50.0,
        },
        ..Default::default()
    };
    Ok(scene)
}
