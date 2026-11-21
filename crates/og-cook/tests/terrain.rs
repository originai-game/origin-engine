//! Terrains through the cook and back (phase 2): a `type terrain` .ogt and
//! its 16-bit PNG (or EXR) heightmap cook into tiles the runtime loads into
//! og-render's TerrainData, with the control map and layers; the cook is
//! deterministic and incremental; and a navmesh bakes a terrain's walkable
//! surface by its slope limit.
//!
//! Ways this could fail, each caught below: tiles stitched in the wrong
//! place or order (rows and columns swapped, the last short tile lost);
//! heights quantised coarser than the source or mapped to the wrong
//! range; an EXR's metres scaled; the control map or a layer's fields
//! lost; the heightmap cooked as a texture of its own; two cooks of the
//! same sources differing, or an unchanged terrain cooked again while an
//! edited heightmap is not; a bad file accepted silently; a navmesh walking
//! up a cliff or missing a gentle slope.

use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use std::path::{Path, PathBuf};

fn project(name: &str, files: &[(&str, &[u8])]) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-cook-terrain")
        .join(name);
    let _ = std::fs::remove_dir_all(&root);
    for (path, bytes) in files {
        let p = root.join("assets").join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, bytes).unwrap();
    }
    root
}

fn png16(w: u32, h: u32, f: impl Fn(u32, u32) -> u16) -> Vec<u8> {
    let img = image::ImageBuffer::<image::Luma<u16>, Vec<u16>>::from_fn(w, h, |x, y| {
        image::Luma([f(x, y)])
    });
    let mut out = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    out
}

fn rgba(w: u32, h: u32, f: impl Fn(u32, u32) -> [u8; 4]) -> Vec<u8> {
    let img = image::RgbaImage::from_fn(w, h, |x, y| image::Rgba(f(x, y)));
    let mut out = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    out
}

/// A 300 × 200 heightmap whose samples encode their own position.
fn sample(x: u32, y: u32) -> u16 {
    ((x * 211 + y * 97) % 65536) as u16
}

const TERRAIN: &str = r#"type terrain
version 1
---
heightmap "height.png"
height -10 90
size 600 400
control "control.png"
tile 128
heightBlend 0.5
castShadows false
lod {
  spacing 1.5
  ring 96
}
layer {
  albedo "textures/grass.png"
  color 0.9 1 0.8
  roughness 0.95
  tile 6
  slope 0 34
  slopeFade 5
  altitude -1000000 60
  altitudeFade 12
}
layer {
  albedo "textures/rock.png"
  normal "textures/rock-n.png"
  tile 14
  triplanar true
  normalStrength 1.5
  slope 30 90
  projectSlope 50
  antiTiling 0.75
  heightContrast 0.5
  macroVariation 0.25
  displacement 0.125
  cellVariation 0.5
}
"#;

fn control_texel(x: u32, y: u32) -> [u8; 4] {
    [x as u8 * 4, y as u8 * 8, 7, 255 - x as u8]
}

fn rgba_pixels(w: u32, h: u32, f: impl Fn(u32, u32) -> [u8; 4]) -> Vec<u8> {
    let mut out = Vec::new();
    for y in 0..h {
        for x in 0..w {
            out.extend_from_slice(&f(x, y));
        }
    }
    out
}

fn island_files() -> Vec<(&'static str, Vec<u8>)> {
    vec![
        ("terrain/island.ogt", TERRAIN.as_bytes().to_vec()),
        ("terrain/height.png", png16(300, 200, sample)),
        ("terrain/control.png", rgba(64, 32, control_texel)),
        ("textures/grass.png", rgba(8, 8, |_, _| [40, 160, 40, 255])),
        ("textures/rock.png", rgba(8, 8, |_, _| [120, 120, 120, 255])),
        (
            "textures/rock-n.png",
            rgba(8, 8, |_, _| [128, 128, 255, 255]),
        ),
        (
            "textures/rock-n.png.ogt",
            b"type import\nversion 1\n---\ntexture {\n  kind normal\n}\n".to_vec(),
        ),
    ]
}

fn cook(root: &Path) -> og_cook::Report {
    let report = cook_project(root, &CookOptions::default()).expect("cook");
    assert!(report.ok(), "{:?}", report.errors());
    report
}

#[test]
fn a_terrain_cooks_into_tiles_and_loads_back() {
    let files = island_files();
    let refs: Vec<(&str, &[u8])> = files.iter().map(|(p, b)| (*p, b.as_slice())).collect();
    let root = project("island", &refs);
    let report = cook(&root);
    let outputs: Vec<String> = report
        .cooked_outputs()
        .iter()
        .map(|o| o.id.clone())
        .collect();
    assert!(
        outputs.iter().any(|o| o == "terrain/island.ogt"),
        "{outputs:?}"
    );
    // The heightmap and the control map are the terrain's, not textures.
    assert!(
        !outputs
            .iter()
            .any(|o| o.starts_with("terrain/height") || o.starts_with("terrain/control")),
        "{outputs:?}"
    );
    assert!(outputs.iter().any(|o| o == "textures/rock.png"));

    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let t = bundle.load_terrain("terrain/island.ogt").unwrap();
    let data = t.field().unwrap();
    assert_eq!((data.columns, data.rows), (300, 200));
    assert_eq!(t.layout.tiles(0), (3, 2), "299 × 199 quads in tiles of 128");
    assert_eq!(t.layout.levels, 3, "3 × 2, 2 × 1 and 1 × 1 tiles");
    assert_eq!(data.size, glam::Vec2::new(600.0, 400.0));
    // Every sample at its place, within the source's own 16-bit step.
    let step = 100.0 / 65535.0;
    for (x, y) in [
        (0, 0),
        (299, 0),
        (0, 199),
        (299, 199),
        (128, 64),
        (255, 130),
        (17, 191),
    ] {
        let want = -10.0 + sample(x, y) as f32 / 65535.0 * 100.0;
        let got = data.heights[(y * 300 + x) as usize];
        assert!((got - want).abs() <= step, "({x}, {y}): {got} vs {want}");
    }
    let worst = (0..200u32)
        .flat_map(|y| (0..300u32).map(move |x| (x, y)))
        .map(|(x, y)| {
            let want = -10.0 + sample(x, y) as f32 / 65535.0 * 100.0;
            (data.heights[(y * 300 + x) as usize] - want).abs()
        })
        .fold(0.0f32, f32::max);
    assert!(worst <= step, "the worst sample is off by {worst}");
    // Every tile decodes on its own, its heights its level's samples of
    // the field's pyramid (within a step: the cook filtered the source
    // before quantising, this filters the quantised field).
    let pyramid = data.height_pyramid(&t.layout);
    for index in [0, 4, t.layout.index(1, 1, 0), t.layout.index(2, 0, 0)] {
        let (tile, _) = t.tile_heights(index).unwrap();
        let want = data.tile_heights(&t.layout, index, &pyramid);
        let off = tile
            .iter()
            .zip(&want)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(off <= step * 1.01, "tile {index} is off by {off}");
        // Its horizon, cooked with it, is the field's own (from the
        // source the cook read, so within a few of its steps).
        let cooked = t.tile(index).unwrap().horizon;
        let own = data.horizon(&t.layout, index, &pyramid);
        assert_eq!(cooked.len(), t.layout.horizon_words() as usize);
        let worst = cooked
            .iter()
            .zip(&own)
            .flat_map(|(a, b)| {
                (0..2).map(move |k| ((a >> (12 * k)) & 4095).abs_diff((b >> (12 * k)) & 4095))
            })
            .max()
            .unwrap();
        assert!(worst <= 8, "tile {index}'s horizon is off by {worst} steps");
    }
    // A level-1 sample is the mean of the source under it, a centred tent
    // over its 3 × 3 finest samples: at (298, 26) the source wraps from
    // its top to its bottom (decimation kept the one sample there).
    let tile = t.tile(t.layout.index(1, 1, 0)).unwrap();
    let source = |x: u32, y: u32| -10.0 + sample(x, y) as f32 / 65535.0 * 100.0;
    let mut mean = 0.0;
    for (dy, wy) in [(0, 0.25), (1, 0.5), (2, 0.25)] {
        for (dx, wx) in [(0, 0.25), (1, 0.5), (2, 0.25)] {
            mean += wx * wy * source(297 + dx, 25 + dy);
        }
    }
    // Level 1's sample (149, 13) is tile (1, 0)'s (21, 13), past its border.
    let got = tile.heights[(13 + 1) * 131 + 21 + 1];
    assert!(
        (got - mean).abs() <= step,
        "level 1 at (149, 13): {got}, the mean {mean}"
    );
    assert!(
        (source(298, 26) - mean).abs() > 10.0,
        "the mean is not the sample"
    );
    // The control map, resampled for the tiles: what the renderer reads
    // anywhere is the source's within a step or two.
    let c = data.control.as_ref().expect("a control map");
    assert_eq!(
        (c.width, c.height),
        (63, 42),
        "27 texels per 128 quads over 299 × 199"
    );
    let source = og_render::TerrainData {
        control: Some(og_render::TerrainControl {
            width: 64,
            height: 32,
            rgba: rgba_pixels(64, 32, control_texel).into(),
        }),
        ..data.clone()
    };
    for p in [
        glam::Vec2::new(-200.0, -150.0),
        glam::Vec2::new(10.0, 33.0),
        glam::Vec2::new(250.0, 120.0),
    ] {
        let (a, b) = (data.control_at(p), source.control_at(p));
        for k in 0..4 {
            assert!((a[k] - b[k]).abs() < 0.07, "{p}: {a:?} vs {b:?}");
        }
    }
    // The layers and settings.
    assert_eq!(t.layers.len(), 2);
    assert_eq!(t.layers[0].albedo.as_deref(), Some("textures/grass.png"));
    assert_eq!(t.layers[0].normal, None);
    assert_eq!(t.layers[1].normal.as_deref(), Some("textures/rock-n.png"));
    let (g, r) = (&t.layers[0].layer, &t.layers[1].layer);
    assert_eq!(g.color, glam::Vec3::new(0.9, 1.0, 0.8));
    assert_eq!(
        (g.roughness, g.tile, g.slope, g.slope_fade),
        (0.95, 6.0, [0.0, 34.0], 5.0)
    );
    assert_eq!((g.altitude, g.altitude_fade), ([-1.0e6, 60.0], 12.0));
    assert!(r.triplanar && r.normal_strength == 1.5 && r.slope == [30.0, 90.0]);
    assert_eq!(
        (
            r.project_slope,
            r.anti_tiling,
            r.height_contrast,
            r.macro_variation,
            r.displacement,
            r.cell_variation
        ),
        (50.0, 0.75, 0.5, 0.25, 0.125, 0.5)
    );
    assert_eq!(
        (g.project_slope, g.anti_tiling, g.displacement),
        (0.0, 0.0, 0.0),
        "absent: off"
    );
    assert_eq!(t.height_blend, 0.5);
    assert_eq!((t.lod.spacing, t.lod.ring), (1.5, 96));
    assert!(!t.cast_shadows && t.collision);
    assert_eq!(t.sea_level, None, "a heightmap has no sea");
}

/// A generated island: two layers, a sea, rivers and biomes.
const GENERATED: &str = r#"type terrain
version 1
---
height -20 60
size 256 256
generate {
  samples 129 129
  seed 5
  noise {
    octaves 5
    warp 0.4
  }
  mask island 0.5
  erosion {
    drops 4000
  }
  sea 4
  shore 2
  rivers {
    threshold 60
  }
  biome {
    layer 0
    altitude 6 1000
  }
  biome {
    layer 1
    altitude -1000 6
  }
}
layer {
  albedo "textures/grass.png"
}
layer {
  albedo "textures/rock.png"
}
"#;

/// A `generate` block makes the field the heightmap would have given: the
/// same tile pyramid, its samples spanning `height`, its biomes' control
/// map and its sea level in the blob; two cooks agree byte for byte.
#[test]
fn a_generated_terrain_cooks_into_the_same_tiles() {
    let files: Vec<(&str, &[u8])> = vec![
        ("terrain/gen.ogt", GENERATED.as_bytes()),
        ("textures/grass.png", &[]),
        ("textures/rock.png", &[]),
    ];
    let grass = rgba(8, 8, |_, _| [40, 160, 40, 255]);
    let rock = rgba(8, 8, |_, _| [120, 120, 120, 255]);
    let files: Vec<(&str, &[u8])> = files
        .into_iter()
        .map(|(p, b)| match p {
            "textures/grass.png" => (p, grass.as_slice()),
            "textures/rock.png" => (p, rock.as_slice()),
            _ => (p, b),
        })
        .collect();
    let a = project("generated-a", &files);
    let b = project("generated-b", &files);
    let report = cook(&a);
    cook(&b);
    let read = |root: &Path| std::fs::read(root.join("cooked/terrain/gen.ogt.ogb")).unwrap();
    assert_eq!(read(&a), read(&b), "two cooks of the same generator");
    let notes = report.to_markdown();
    assert!(notes.contains("generated: seed 5, sea at 4 m"), "{notes}");

    let bundle = Bundle::open(&a.join("cooked")).unwrap();
    let t = bundle.load_terrain("terrain/gen.ogt").unwrap();
    assert_eq!(t.sea_level, Some(4.0));
    let data = t.field().unwrap();
    assert_eq!((data.columns, data.rows), (129, 129));
    assert_eq!(data.size, glam::Vec2::new(256.0, 256.0));
    assert_eq!(t.layout.tiles(0), (1, 1), "128 quads in one tile of 128");
    // The island's edge is the low; its top is near the high (erosion and
    // rivers only take away).
    let (lo, hi) = data.range();
    assert!((lo - -20.0).abs() < 0.01, "lowest {lo}");
    assert!(hi > 50.0 && hi <= 60.01, "highest {hi}");
    assert!(data.heights[0] < 4.0 && data.heights[64 * 129 + 64] > 4.0);
    // The biomes' control map: layer 0 on the land, layer 1 in the sea.
    let c = data.control.as_ref().expect("biomes write a control map");
    assert!(c.width > 0 && c.height > 0);
    let corner = data.control_at(glam::Vec2::new(-120.0, -120.0));
    assert!(corner[1] > 0.95 && corner[0] < 0.05, "the sea: {corner:?}");
    let (mut land, mut sea) = (0, 0);
    for r in (0..129).step_by(4) {
        for col in (0..129).step_by(4) {
            let h = data.heights[r * 129 + col];
            let p = glam::Vec2::new(col as f32 * 2.0 - 128.0, r as f32 * 2.0 - 128.0);
            let w = data.control_at(p);
            // The cooked map is resampled for the tiles: away from the
            // 6 m line (and its 2 m fade) by more than a sample's rise.
            if h > 16.0 {
                land += 1;
                assert!(w[0] > 0.8 && w[1] < 0.2, "land at {h} m: {w:?}");
            } else if h < -4.0 {
                sea += 1;
                assert!(w[1] > 0.8 && w[0] < 0.2, "sea at {h} m: {w:?}");
            }
        }
    }
    assert!(land > 30 && sea > 30, "{land} land, {sea} sea");

    // The cook report: its histogram, land above the sea and rivers, and
    // its maps as PNGs beside the report, never under assets/.
    use og_cook::cook::stats::{Budgets, cook_report_json, write_terrain_maps};
    let mut json = cook_report_json(&report, &a.join("cooked"), &Budgets::default());
    let entry = json["assets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["source"] == "terrain/gen.ogt")
        .unwrap()
        .clone();
    let g = &entry["terrain"]["generated"];
    let bins: Vec<u64> = serde_json::from_value(g["histogram"].clone()).unwrap();
    assert_eq!(bins.len(), 16);
    assert_eq!(bins.iter().sum::<u64>(), 129 * 129);
    let above = data.heights.iter().filter(|h| **h > 4.0).count() as f64 / (129.0 * 129.0);
    let land_fraction = g["landFraction"].as_f64().unwrap();
    assert!((land_fraction - above).abs() < 0.001, "{g}");
    assert!(land_fraction > 0.2 && land_fraction < 0.8, "{g}");
    assert_eq!(g["seaLevel"], 4.0);
    assert!(g["riverLength"].as_f64().unwrap() > 50.0, "{g}");
    let out = a.join("og-out");
    write_terrain_maps(&mut json, &a.join("cooked"), &out).unwrap();
    let height = image::open(out.join("terrain-maps/terrain_gen.ogt.height.png")).unwrap();
    assert_eq!((height.width(), height.height()), (129, 129));
    let control = image::open(out.join("terrain-maps/terrain_gen.ogt.control.png")).unwrap();
    assert!(control.width() > 0);
    let entry = json["assets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["source"] == "terrain/gen.ogt")
        .unwrap();
    assert_eq!(
        entry["terrain"]["generated"]["maps"],
        serde_json::json!([
            "terrain-maps/terrain_gen.ogt.height.png",
            "terrain-maps/terrain_gen.ogt.control.png"
        ])
    );
    let e = write_terrain_maps(&mut json, &a.join("cooked"), &a.join("assets/review")).unwrap_err();
    assert!(e.contains("not sources"), "{e}");
    assert!(!a.join("assets/review").exists());
}

#[test]
fn the_terrain_cook_is_deterministic_and_incremental() {
    let files = island_files();
    let refs: Vec<(&str, &[u8])> = files.iter().map(|(p, b)| (*p, b.as_slice())).collect();
    let a = project("twice-a", &refs);
    let b = project("twice-b", &refs);
    cook(&a);
    cook(&b);
    let read = |root: &Path| std::fs::read(root.join("cooked/terrain/island.ogt.ogb")).unwrap();
    assert_eq!(read(&a), read(&b), "two cooks of the same sources");
    // Again in place: nothing to do.
    let again = cook(&a);
    assert!(
        !again.cooked_outputs().iter().any(|o| o.cooked),
        "nothing changed, nothing cooked"
    );
    // A new heightmap re-cooks the terrain (and only it).
    std::fs::write(
        a.join("assets/terrain/height.png"),
        png16(300, 200, |x, y| sample(y % 200, x % 200)),
    )
    .unwrap();
    let edited = cook(&a);
    let cooked: Vec<String> = edited
        .cooked_outputs()
        .iter()
        .filter(|o| o.cooked)
        .map(|o| o.id.clone())
        .collect();
    assert_eq!(cooked, vec!["terrain/island.ogt".to_string()]);
    assert_ne!(read(&a), read(&b));
}

/// Terrains that share an edge (east and north of one): the same declared
/// `height`, the same 16-bit samples on the edge, different heights inside.
/// Each cooks its heights over the declared range, not over its own lowest
/// and highest, so the edge decodes to the same metres in both, and the
/// surface along it samples the same bits from either side, while each
/// terrain's range is still its own.
#[test]
fn adjacent_terrains_with_equal_edge_samples_meet_exactly() {
    let edge = |y: u32| (21_000 + (y * 1_237) % 9_000) as u16;
    let west_at = |x: u32, y: u32| {
        if x == 64 {
            edge(y)
        } else {
            (20_000 + (x * 37 + y * 11) % 5_000) as u16
        }
    };
    let west = png16(65, 65, west_at);
    // North of west: its first row is west's last.
    let north = png16(65, 65, |x, y| {
        if y == 0 {
            west_at(x, 64)
        } else {
            (5_000 + (x * 29 + y * 41) % 7_000) as u16
        }
    });
    let east = png16(65, 65, |x, y| {
        if x == 0 {
            edge(y)
        } else {
            (30_000 + (x * 53 + y * 7) % 9_000) as u16
        }
    });
    let ogt = |png: &str| {
        format!(
            "type terrain\nversion 1\n---\nheightmap \"{png}\"\nheight -256 768\nsize 64 64\ntile 64\n"
        )
    };
    let (w, e, n) = (ogt("west.png"), ogt("east.png"), ogt("north.png"));
    let root = project(
        "seam",
        &[
            ("terrain/west.png", &west),
            ("terrain/east.png", &east),
            ("terrain/west.ogt", w.as_bytes()),
            ("terrain/east.ogt", e.as_bytes()),
            ("terrain/north.png", &north),
            ("terrain/north.ogt", n.as_bytes()),
        ],
    );
    cook(&root);
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let load = |id: &str| bundle.load_terrain(id).unwrap();
    let (tw, te) = (load("terrain/west.ogt"), load("terrain/east.ogt"));
    let tn = load("terrain/north.ogt");
    let (fw, fe, fn_) = (
        tw.field().unwrap(),
        te.field().unwrap(),
        tn.field().unwrap(),
    );
    for y in 0..65u32 {
        let a = fw.heights[(y * 65 + 64) as usize];
        let b = fe.heights[(y * 65) as usize];
        assert_eq!(a.to_bits(), b.to_bits(), "row {y}: {a} vs {b}");
        let want = -256.0 + edge(y) as f32 / 65535.0 * 1024.0;
        assert!(
            (a - want).abs() < 1e-4,
            "row {y}: {a} vs the source's {want}"
        );
    }
    // og.terrain.sample's surface along the seam, between samples too:
    // west's +x edge is east's -x edge.
    for k in 0..=200 {
        let z = -32.0 + k as f32 * 0.32;
        let a = fw.height(glam::Vec2::new(32.0, z));
        let b = fe.height(glam::Vec2::new(-32.0, z));
        assert_eq!(a.to_bits(), b.to_bits(), "z {z}: {a} vs {b}");
        // West's +z edge is north's -z edge.
        let a = fw.height(glam::Vec2::new(z, 32.0));
        let b = fn_.height(glam::Vec2::new(z, -32.0));
        assert_eq!(a.to_bits(), b.to_bits(), "x {z}: {a} vs {b}");
    }
    // Each terrain's range is its own heights', not the declared one.
    for (t, f) in [(&tw, &fw), (&te, &fe), (&tn, &fn_)] {
        let (lo, hi) = f.range();
        assert!(lo > -200.0 && hi < 500.0, "{lo} to {hi}");
        assert_eq!(t.range, (lo, hi));
    }
}

/// A terrain with holes keeps every 16-bit step of its heightmap: its hole
/// mask travels beside the heights, a byte per cell, never in a height's
/// bits. Every sample is odd (a lowest bit a flag would take), one cell is
/// cut out, the same heights load with and without it, and the tile's mask
/// and the field's hole map name the same cell.
#[test]
fn holes_keep_every_height_step() {
    let at = |x: u32, y: u32| (2 * ((x * 131 + y * 71) % 30_000) + 1) as u16;
    let heights = png16(65, 65, at);
    let holes = rgba(64, 64, |x, y| {
        if (x, y) == (60, 60) {
            [0, 0, 0, 255]
        } else {
            [255; 4]
        }
    });
    let ogt = |holes: bool| {
        format!(
            "type terrain\nversion 1\n---\nheightmap \"h.png\"\n{}height 0 1024\nsize 64 64\ntile 64\n",
            if holes { "holes \"holes.png\"\n" } else { "" }
        )
    };
    let (with, without) = (ogt(true), ogt(false));
    let root = project(
        "holes-steps",
        &[
            ("terrain/h.png", &heights),
            ("terrain/holes.png", &holes),
            ("terrain/with.ogt", with.as_bytes()),
            ("terrain/without.ogt", without.as_bytes()),
        ],
    );
    cook(&root);
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let field = |id: &str| bundle.load_terrain(id).unwrap().field().unwrap();
    let (a, b) = (field("terrain/with.ogt"), field("terrain/without.ogt"));
    let cut = a.holes.as_deref().expect("holes");
    assert_eq!(cut.iter().filter(|h| **h != 0).count(), 1);
    assert_ne!(cut[60 * 64 + 60], 0);
    for y in 0..65u32 {
        for x in 0..65u32 {
            let i = (y * 65 + x) as usize;
            let want = at(x, y) as f32 / 65535.0 * 1024.0;
            assert!(
                (a.heights[i] - want).abs() < 1e-3,
                "({x}, {y}): {} vs the source's {want}",
                a.heights[i]
            );
            assert_eq!(a.heights[i].to_bits(), b.heights[i].to_bits(), "({x}, {y})");
        }
    }
    // The level-0 tile's mask (its border one sample wide): the cell at
    // sample (60, 60) alone, cut.
    let tile = bundle
        .load_terrain("terrain/with.ogt")
        .unwrap()
        .tile(0)
        .unwrap();
    let mask = tile.holes.as_deref().expect("a hole mask");
    let side = 67;
    assert_eq!(mask.len(), side * side);
    for (i, &m) in mask.iter().enumerate() {
        let (x, y) = (i % side, i / side);
        assert_eq!(m != 0, (x, y) == (61, 61), "sample ({x}, {y}) of the tile");
    }
    assert!(
        bundle
            .load_terrain("terrain/without.ogt")
            .unwrap()
            .tile(0)
            .unwrap()
            .holes
            .is_none()
    );
}

/// A terrain may name layer arrays another cooked bundle provides
/// (project.ogt's `cook { uses }`): one material pack for many terrain
/// packs. The terrain cooks naming them by id and packs none of them; a
/// name no bundle provides, or one of the wrong type, is still refused.
#[test]
fn layer_arrays_from_a_used_bundle() {
    let tile = rgba(8, 8, |_, _| [90, 140, 60, 255]);
    let materials = project(
        "uses-materials",
        &[
            ("tex/a.png", &tile),
            ("tex/b.png", &tile),
            (
                "tex/tiles.ogt",
                b"type texture-array\nversion 1\n---\nlayer \"a.png\"\nlayer \"b.png\"\n",
            ),
            (
                "tex/tiles-n.ogt",
                b"type texture-array\nversion 1\n---\nkind normal\nlayer \"a.png\"\nlayer \"b.png\"\n",
            ),
        ],
    );
    cook(&materials);
    let height = png16(65, 65, |x, y| (x * 300 + y * 200) as u16);
    let terrain = |layers: &str| {
        format!(
            "type terrain\nversion 1\n---\nheightmap \"land.png\"\nheight 0 20\nsize 64 64\ntile 64\n{layers}layer {{\n  albedoLayer 1\n  normalLayer 1\n}}\n"
        )
    };
    let arrays = terrain("layers \"tex/tiles.ogt\"\nlayerNormals \"tex/tiles-n.ogt\"\n");
    let parcel = |name: &str, ogt: &str, project_ogt: &str| {
        let root = project(
            name,
            &[
                ("land/land.png", &height),
                ("land/land.ogt", ogt.as_bytes()),
            ],
        );
        std::fs::write(root.join("project.ogt"), project_ogt).unwrap();
        root
    };
    let uses = "type project\nversion 1\n---\ncook {\n  uses \"../uses-materials\"\n}\n";

    let root = parcel("uses-parcel", &arrays, uses);
    cook(&root);
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let t = bundle.load_terrain("land/land.ogt").unwrap();
    assert_eq!(t.albedo_array.as_deref(), Some("tex/tiles.ogt"));
    assert_eq!(t.normal_array.as_deref(), Some("tex/tiles-n.ogt"));
    let ids: Vec<_> = bundle.manifest().outputs().map(|o| o.id.clone()).collect();
    assert!(
        !ids.iter().any(|id| id.starts_with("tex/")),
        "the used bundle's assets were packed: {ids:?}"
    );

    // Without `uses` the arrays are not the project's: refused.
    let alone = parcel("uses-none", &arrays, "type project\nversion 1\n---\n");
    let report = cook_project(&alone, &CookOptions::default()).unwrap();
    let errors = format!("{:?}", report.errors());
    assert!(
        errors.contains("tex/tiles.ogt") && errors.contains("cook { uses }"),
        "{errors}"
    );

    // An array named where a texture goes: the type is checked.
    let wrong = parcel(
        "uses-wrong",
        "type terrain\nversion 1\n---\nheightmap \"land.png\"\nheight 0 20\nsize 64 64\ntile 64\nlayer {\n  albedo \"tex/tiles.ogt\"\n}\n",
        uses,
    );
    let report = cook_project(&wrong, &CookOptions::default()).unwrap();
    let errors = format!("{:?}", report.errors());
    assert!(
        errors.contains("is og.TextureArray, not og.Texture"),
        "{errors}"
    );
}
