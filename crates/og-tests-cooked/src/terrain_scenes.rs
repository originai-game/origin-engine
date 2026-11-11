//! The terrain scene that cooks (og-tests' terrain_scenes has the others,
//! and the island's layers, light and camera this one reads).

use glam::{Mat4, Vec2, Vec3};
use og_render::{RenderScene, Terrain, TerrainData, TerrainLayer, Water};
use og_tests::terrain_scenes::{afternoon, camera_at, island_layers};
use std::sync::OnceLock;

/// The generated island (`terrain-generated`): a `type terrain` with a
/// `generate` block, cooked by og-cook into a project of its own and
/// loaded back as og.terrain.load would: 1 km square, noise warped into
/// an island, eroded by drops and slumped, a sea at 4 m with its shore
/// flattened, rivers carved down to it, and biome rules writing the
/// control map the island's four layers read (grass on the low flats,
/// rock on the slopes, sand on the shore below 7 m and in the rivers'
/// beds where the flow gathers, snow above 70 m). The sea's water stands at the terrain's
/// `seaLevel`, seen from 450 m out to sea at the south-east.
pub const GENERATED_TERRAIN: &str = "type terrain
version 1
---
height -20 90
size 1024 1024
generate {
  samples 257 257
  seed 11
  noise {
    octaves 6
    frequency 2
    gain 0.45
    warp 0.6
  }
  mask island 0.6
  erosion {
    drops 60000
  }
  thermal {
    talus 38
    iterations 10
  }
  sea 4
  shore 3
  rivers {
    threshold 150
    depth 2.5
    width 16
  }
  moisture {
    noise 0
  }
  biome {
    layer 0
    altitude 7 1000
    slope 0 28
  }
  biome {
    layer 1
    slope 28 90
  }
  biome {
    layer 2
    altitude -1000 7
  }
  biome {
    layer 2
    moisture 0.7 1
    slope 0 40
  }
  biome {
    layer 3
    altitude 70 1000
    slope 0 35
  }
}
layer {
  tile 6
}
layer {
  tile 14
}
layer {
  tile 5
}
layer {
  tile 8
}
";

/// The generated island's cooked field and sea level (cooked once per
/// process).
#[cfg(not(target_arch = "wasm32"))]
pub fn generated_island() -> Result<(TerrainData, f32), String> {
    static DATA: OnceLock<Result<(TerrainData, f32), String>> = OnceLock::new();
    DATA.get_or_init(|| {
        let root =
            std::env::temp_dir().join(format!("og-tests-terrain-generated-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let dir = root.join("assets/terrain");
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        std::fs::write(dir.join("island.ogt"), GENERATED_TERRAIN)
            .map_err(|e| format!("island.ogt: {e}"))?;
        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default())
            .map_err(|e| format!("cook: {e}"))?;
        if !report.ok() {
            return Err(format!("cook: {:?}", report.errors()));
        }
        let bundle =
            og_cook::runtime::Bundle::open(&root.join("cooked")).map_err(|e| e.to_string())?;
        let t = bundle
            .load_terrain("terrain/island.ogt")
            .map_err(|e| e.to_string())?;
        let sea = t.sea_level.ok_or("the generated island has no sea")?;
        let field = t.field().map_err(|e| e.to_string())?;
        let _ = std::fs::remove_dir_all(&root);
        Ok((field, sea))
    })
    .clone()
}

/// Where the `terrain-generated` camera stands and looks.
pub const GENERATED_EYE: Vec3 = Vec3::new(560.0, 300.0, 700.0);
pub const GENERATED_TARGET: Vec3 = Vec3::new(-20.0, 10.0, -40.0);

/// `terrain-generated` (see [`GENERATED_TERRAIN`]).
#[cfg(not(target_arch = "wasm32"))]
pub fn generated_scene() -> Result<RenderScene, String> {
    let (data, sea) = generated_island()?;
    let mut scene = RenderScene::new();
    let handle = scene.add_terrain(data);
    // The island's textures, every rule left to the biomes' control map.
    let layers = island_layers(&mut scene)
        .into_iter()
        .map(|l| TerrainLayer {
            slope: [0.0, 90.0],
            altitude: [-1.0e6, 1.0e6],
            ..l
        })
        .collect();
    // island_layers: grass, rock, sand, snow, the biomes' order.
    scene.world.terrains.push(Terrain {
        layers,
        height_blend: 0.4,
        ..Terrain::new(handle)
    });
    afternoon(&mut scene);
    scene.camera = camera_at(GENERATED_EYE, GENERATED_TARGET);
    scene.world.water.push(Water {
        transform: Mat4::from_translation(Vec3::new(0.0, sea, 0.0)),
        size: Vec2::splat(12000.0),
        segments: 1,
        depth_fade: 5.0,
        foam_width: 0.8,
        wave_scale: 0.08,
        ..Default::default()
    });
    Ok(scene)
}
