//! Plants the cook grows (a `plant` block in a `type model`, og-geom's
//! `plant`) and a wood scattered from them: og-tests' assets/plants, the
//! models guide's four kinds (an oak, a pine, a flowering shrub, a daisy),
//! cooked in a project of their own, the three woody ones with their
//! impostors.
//!
//! - `grown_scene()` (`plant-grown`): the oak, the pine and the shrub
//!   standing in a row on a floor with three daisies near the camera, each drawn
//!   as its cooked parts (the wood, then the leaves and petals with the
//!   leaf's material), under a sun from the front right.
//! - `scatter_scene()` (`plant-scatter`): a 64 m hill with the three woody
//!   plants scattered whole, as og.TerrainScatter.model has og-runtime make
//!   them: one `ScatterDraw` per part, the parts after the first drawn
//!   `with` it. Oaks keep pines and shrubs out of their footprint; copies
//!   past 30 m draw as their impostors; a light wind sways each copy's
//!   wood and leaves as one plant.

use glam::{Mat4, Vec2, Vec3};
use og_cook::runtime::Bundle;
use og_render::{
    Camera, DirectionalLight, Impostor, MaterialHandle, MeshHandle, Projection, RenderScene,
    ScatterDraw, StandardMaterial, Terrain, TerrainData, TerrainLayer, TerrainScatter, Wind,
    shapes,
};
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

/// The plants cooked once per process: `(name, impostor)`.
const PLANTS: [(&str, bool); 4] = [
    ("oak", true),
    ("pine", true),
    ("bush", true),
    ("daisy", false),
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
    impostor: Option<Impostor>,
}

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
    let parts = model.add_to(scene, &textures).parts;
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
            })
        }
        None => None,
    };
    Ok(Plant { parts, impostor })
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

/// `plant-grown` ([module docs](self)).
pub fn grown_scene() -> Result<RenderScene, String> {
    let bundle = Bundle::open(&cooked()?.join("cooked")).map_err(|e| e.to_string())?;
    let mut scene = RenderScene::new();
    sun(&mut scene, Vec3::new(0.5, 0.7, 0.45), 60.0);
    let ground = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.3, 0.32, 0.28),
        roughness: 1.0,
        ..Default::default()
    });
    let floor = scene.add_mesh(shapes::plane(120.0, 1.0));
    scene.add_instance(floor, ground, Mat4::IDENTITY);
    for (name, at) in ["oak", "pine", "bush"].into_iter().zip(GROWN_AT) {
        let p = plant(&mut scene, &bundle, name)?;
        for (mesh, material, local) in p.parts {
            scene.add_instance(mesh, material, Mat4::from_translation(at) * local);
        }
    }
    let daisy = plant(&mut scene, &bundle, "daisy")?;
    for (k, x) in [2.6f32, 3.1, 3.5].into_iter().enumerate() {
        let at = Mat4::from_translation(Vec3::new(x, 0.0, 15.2 + 0.4 * k as f32))
            * Mat4::from_rotation_y(1.7 * k as f32);
        for &(mesh, material, local) in &daisy.parts {
            scene.add_instance(mesh, material, at * local);
        }
    }
    scene.frame.camera = Camera {
        position: Vec3::new(1.0, 3.2, 25.0),
        target: Vec3::new(0.0, 5.0, 0.0),
        far: 200.0,
        projection: Projection::Perspective {
            fov_y_degrees: 50.0,
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

fn hill() -> TerrainData {
    let n = 129u32;
    let at = |i: u32| (i as f32 / (n - 1) as f32 - 0.5) * 64.0;
    let heights: Vec<f32> = (0..n * n)
        .map(|i| hill_height(at(i % n), at(i / n)))
        .collect();
    TerrainData {
        heights: Arc::from(heights),
        ..TerrainData::flat(Vec2::splat(64.0), n, n)
    }
}

/// Past this the wood's copies draw as cards (m, along x and z).
pub const SCATTER_IMPOSTORS: f32 = 30.0;

/// `plant-scatter` ([module docs](self)).
pub fn scatter_scene() -> Result<RenderScene, String> {
    let bundle = Bundle::open(&cooked()?.join("cooked")).map_err(|e| e.to_string())?;
    let mut scene = RenderScene::new();
    sun(&mut scene, Vec3::new(0.5, 0.7, 0.45), 120.0);
    let h = scene.add_terrain(hill());
    scene.world.terrains.push(Terrain {
        layers: vec![TerrainLayer {
            color: Vec3::new(0.25, 0.36, 0.16),
            ..Default::default()
        }],
        fade_in: false,
        ..Terrain::new(h)
    });
    // (plant, copies a square metre, seed, priority, footprint in metres,
    // the share of the ground its noise leaves bare).
    for (name, density, seed, priority, footprint, bare) in [
        ("oak", 0.012, 3, 3, 5.0, 0.35),
        ("pine", 0.016, 5, 2, 4.0, 0.45),
        ("bush", 0.03, 9, 1, 2.0, 0.3),
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
                impostor_distance: SCATTER_IMPOSTORS,
                with: (k > 0).then_some(first),
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
        position: Vec3::new(0.0, 22.0, 44.0),
        target: Vec3::new(0.0, 4.0, 0.0),
        far: 400.0,
        projection: Projection::Perspective {
            fov_y_degrees: 50.0,
        },
        ..Default::default()
    };
    Ok(scene)
}
