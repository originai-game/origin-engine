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
}
"#;

fn island_files() -> Vec<(&'static str, Vec<u8>)> {
    vec![
        ("terrain/island.ogt", TERRAIN.as_bytes().to_vec()),
        ("terrain/height.png", png16(300, 200, sample)),
        (
            "terrain/control.png",
            rgba(64, 32, |x, y| [x as u8 * 4, y as u8 * 8, 7, 255 - x as u8]),
        ),
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
    assert_eq!((t.data.columns, t.data.rows), (300, 200));
    assert_eq!(t.tiles, (3, 2), "300 × 200 in tiles of 128");
    assert_eq!(t.data.size, glam::Vec2::new(600.0, 400.0));
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
        let got = t.data.heights[(y * 300 + x) as usize];
        assert!((got - want).abs() <= step, "({x}, {y}): {got} vs {want}");
    }
    let worst = (0..200u32)
        .flat_map(|y| (0..300u32).map(move |x| (x, y)))
        .map(|(x, y)| {
            let want = -10.0 + sample(x, y) as f32 / 65535.0 * 100.0;
            (t.data.heights[(y * 300 + x) as usize] - want).abs()
        })
        .fold(0.0f32, f32::max);
    assert!(worst <= step, "the worst sample is off by {worst}");
    // The control map, byte for byte.
    let c = t.data.control.as_ref().expect("a control map");
    assert_eq!((c.width, c.height), (64, 32));
    assert_eq!(
        &c.rgba[(5 * 64 + 9) * 4..(5 * 64 + 9) * 4 + 4],
        &[36, 40, 7, 246]
    );
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
    assert_eq!(t.height_blend, 0.5);
    assert_eq!((t.lod.spacing, t.lod.ring), (1.5, 96));
    assert!(!t.cast_shadows && t.collision);
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

#[test]
fn an_exr_heightmap_is_metres_as_stored() {
    let img = image::Rgb32FImage::from_fn(33, 17, |x, y| {
        let h = x as f32 * 0.75 - y as f32 * 2.5 + 0.125;
        image::Rgb([h, 0.0, 0.0])
    });
    let mut exr = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut exr),
        image::ImageFormat::OpenExr,
    )
    .unwrap();
    let text = "type terrain\nversion 1\n---\nheightmap \"h.exr\"\nsize 64 32\n";
    let root = project("exr", &[("t/flat.ogt", text.as_bytes()), ("t/h.exr", &exr)]);
    cook(&root);
    let t = Bundle::open(&root.join("cooked"))
        .unwrap()
        .load_terrain("t/flat.ogt")
        .unwrap();
    assert_eq!((t.data.columns, t.data.rows), (33, 17));
    let span = 24.0 + 40.0;
    for (x, y) in [(0u32, 0u32), (32, 0), (5, 16), (20, 9)] {
        let want = x as f32 * 0.75 - y as f32 * 2.5 + 0.125;
        let got = t.data.heights[(y * 33 + x) as usize];
        assert!(
            (got - want).abs() < span / 65535.0,
            "({x}, {y}): {got} vs {want}"
        );
    }
}

#[test]
fn refusals_name_the_fix() {
    let cook_err = |name: &str, text: &str, extra: &[(&str, &[u8])]| -> String {
        let mut files: Vec<(&str, &[u8])> = vec![("t/x.ogt", text.as_bytes())];
        files.extend_from_slice(extra);
        let root = project(name, &files);
        let report = cook_project(&root, &CookOptions::default()).expect("cook runs");
        report.errors().join("\n")
    };
    let height = png16(4, 4, |_, _| 0);
    let e = cook_err(
        "missing",
        "type terrain\nversion 1\n---\nheightmap \"none.png\"\n",
        &[],
    );
    assert!(e.contains("heightmap 'none.png' does not exist"), "{e}");
    let e = cook_err("noname", "type terrain\nversion 1\n---\nsize 4 4\n", &[]);
    assert!(e.contains("names its heightmap"), "{e}");
    let e = cook_err(
        "key",
        "type terrain\nversion 1\n---\nheightmap \"h.png\"\nwidth 4\n",
        &[("t/h.png", &height)],
    );
    assert!(e.contains("line 5: 'width' is not a terrain entry"), "{e}");
    let five = format!(
        "type terrain\nversion 1\n---\nheightmap \"h.png\"\n{}",
        "layer {\n  tile 2\n}\n".repeat(5)
    );
    let e = cook_err("five", &five, &[("t/h.png", &height)]);
    assert!(e.contains("at most 4 layers"), "{e}");
    let e = cook_err(
        "texture",
        "type terrain\nversion 1\n---\nheightmap \"h.png\"\nlayer {\n  albedo \"textures/none.png\"\n}\n",
        &[("t/h.png", &height)],
    );
    assert!(e.contains("not under assets/: textures/none.png"), "{e}");
}

/// A terrain with a gentle ramp (about 17 degrees) beside a cliff (about
/// 63 degrees): its navmesh, baked with a 40 degree limit, walks the ramp
/// and the plateau above it but not the cliff's face.
#[test]
fn a_navmesh_bakes_a_terrains_walkable_surface() {
    // 81 × 41 samples over 80 × 40 m; x 0..39 ramps 0 → 12 m, the cliff
    // from x 50 rises 2 m per metre to a plateau at 20 m.
    let h = |x: u32| -> f32 {
        let x = x as f32;
        if x < 40.0 {
            x * 0.3
        } else if x < 50.0 {
            12.0
        } else {
            (12.0 + (x - 50.0) * 2.0).min(20.0)
        }
    };
    let height = png16(81, 41, |x, _| (h(x) / 32.0 * 65535.0).round() as u16);
    let terrain = "type terrain\nversion 1\n---\nheightmap \"h.png\"\nheight 0 32\nsize 80 40\n";
    let navmesh = r#"type navmesh
version 1
---
terrain "t/ramp.ogt" {
  position 0 0 0
}
agent {
  radius 0.4
  height 1.8
  climb 0.4
  slope 40
}
cellSize 0.5
cellHeight 0.2
"#;
    let root = project(
        "nav",
        &[
            ("t/ramp.ogt", terrain.as_bytes()),
            ("t/h.png", &height),
            ("nav/ramp.nav.ogt", navmesh.as_bytes()),
        ],
    );
    let report = cook(&root);
    let notes: Vec<String> = report.assets.iter().flat_map(|a| a.notes.clone()).collect();
    assert!(
        notes.iter().any(|n| n.contains("terrain 't/ramp.ogt'")),
        "{notes:?}"
    );
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let (field, tiles) = bundle
        .navmesh(
            "nav/ramp.nav.ogt",
            &bundle.read("nav/ramp.nav.ogt").unwrap(),
        )
        .unwrap();
    let mesh = og_nav::NavMesh::from_tiles(field, tiles).expect("tiles link");
    let ground = |x: f32| -> f32 { h((x + 40.0).round() as u32) };
    let near = |x: f32, z: f32| {
        mesh.nearest(
            glam::Vec3::new(x, ground(x), z),
            glam::Vec3::new(0.3, 1.0, 0.3),
        )
    };
    // On the ramp and on the ledge: walkable, at the surface's height.
    for x in [-30.0f32, -12.5, 5.0] {
        let (_, p) = near(x, 0.0).unwrap_or_else(|| panic!("walkable at x {x}"));
        assert!((p.y - ground(x)).abs() < 0.6, "x {x}: {p} vs {}", ground(x));
    }
    // Up the cliff's face (x 10.5..14 in world): not walkable.
    for x in [11.0f32, 12.5] {
        assert!(near(x, 0.0).is_none(), "the cliff at x {x} is not walkable");
    }
    // A path from the ramp's foot reaches the ledge but never the plateau.
    let start = glam::Vec3::new(-38.0, ground(-38.0), 0.0);
    let ledge = glam::Vec3::new(5.0, ground(5.0), 0.0);
    assert!(mesh.find_path(start, ledge, glam::Vec3::ONE).is_some());
    let plateau = glam::Vec3::new(35.0, 20.0, 0.0);
    let to_plateau = mesh.find_path(start, plateau, glam::Vec3::ONE);
    assert!(
        to_plateau
            .as_ref()
            .is_none_or(|p| p.points.last().is_some_and(|q| q.y < 13.0)),
        "the plateau is cut off: {to_plateau:?}"
    );
}
