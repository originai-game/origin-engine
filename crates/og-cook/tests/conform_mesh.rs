//! The art contract's 3D conformance through the cook (docs/art.md §2.3,
//! `cook::conform::mesh`): models held to ArtShape, their textures to
//! ArtTexels, clips to ArtMotion. The expected values are derived here from
//! the decoded geometry and the contract's numbers, not from the cook.
//!
//! Ways this could fail: a project without a look (or whose look leaves the
//! art blocks at zero) cooks other bytes than before; the budget counts one
//! primitive and not the nodes drawing it; flat normals averaged across a
//! welded corner; voxels off the grid by half a cell; a stepped clip
//! interpolating between its poses; `conform off` ignored.

use og_cook::cook::{ConformAction, ConformEntry};
use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::support;

/// Positions, normals, uvs and triangles.
type Mesh = (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>);

/// A UV sphere of radius `r` at `c`: smooth normals, uvs, `seg` × `rings`.
fn sphere(r: f32, c: [f32; 3], seg: u32, rings: u32) -> Mesh {
    let (mut p, mut n, mut uv, mut idx) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for j in 0..=rings {
        let th = std::f32::consts::PI * j as f32 / rings as f32;
        for i in 0..=seg {
            let ph = std::f32::consts::TAU * i as f32 / seg as f32;
            let d = [th.sin() * ph.cos(), th.cos(), th.sin() * ph.sin()];
            p.push([c[0] + r * d[0], c[1] + r * d[1], c[2] + r * d[2]]);
            n.push(d);
            uv.push([i as f32 / seg as f32, j as f32 / rings as f32]);
        }
    }
    let w = seg + 1;
    for j in 0..rings {
        for i in 0..seg {
            let a = j * w + i;
            if j > 0 {
                idx.extend([a, a + 1, a + w]);
            }
            if j + 1 < rings {
                idx.extend([a + 1, a + w + 1, a + w]);
            }
        }
    }
    (p, n, uv, idx)
}

/// A glb of one mesh drawn by `nodes` nodes (side by side), with a
/// material of base colour `color` and, with `png`, that image as its
/// base colour texture.
fn glb(mesh: &Mesh, nodes: usize, color: [f32; 4], png: Option<&[u8]>) -> Vec<u8> {
    let (p, n, uv, idx) = mesh;
    let mut bin: Vec<u8> = Vec::new();
    let mut views = Vec::new();
    let mut add = |bin: &mut Vec<u8>, bytes: Vec<u8>| {
        while !bin.len().is_multiple_of(4) {
            bin.push(0);
        }
        views.push(
            serde_json::json!({ "buffer": 0, "byteOffset": bin.len(), "byteLength": bytes.len() }),
        );
        bin.extend(bytes);
        views.len() - 1
    };
    let f = |v: &[f32]| v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>();
    let pv = add(&mut bin, f(&p.concat()));
    let nv = add(&mut bin, f(&n.concat()));
    let uvv = add(&mut bin, f(&uv.concat()));
    let iv = add(&mut bin, idx.iter().flat_map(|x| x.to_le_bytes()).collect());
    let img = png.map(|b| add(&mut bin, b.to_vec()));
    let (lo, hi) = p
        .iter()
        .fold(([f32::MAX; 3], [f32::MIN; 3]), |(lo, hi), v| {
            (
                [0, 1, 2].map(|i| lo[i].min(v[i])),
                [0, 1, 2].map(|i| hi[i].max(v[i])),
            )
        });
    let mut material = serde_json::json!({ "pbrMetallicRoughness": { "baseColorFactor": color } });
    let mut json = serde_json::json!({
        "asset": { "version": "2.0" },
        "buffers": [{ "byteLength": bin.len() }],
        "bufferViews": views,
        "accessors": [
            { "bufferView": pv, "componentType": 5126, "count": p.len(), "type": "VEC3", "min": lo, "max": hi },
            { "bufferView": nv, "componentType": 5126, "count": n.len(), "type": "VEC3" },
            { "bufferView": uvv, "componentType": 5126, "count": uv.len(), "type": "VEC2" },
            { "bufferView": iv, "componentType": 5125, "count": idx.len(), "type": "SCALAR" },
        ],
        "meshes": [{ "primitives": [{ "attributes": { "POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2 }, "indices": 3, "material": 0 }] }],
        "nodes": (0..nodes).map(|i| serde_json::json!({ "mesh": 0, "translation": [i as f32 * 3.0, 0.0, 0.0] })).collect::<Vec<_>>(),
        "scenes": [{ "nodes": (0..nodes).collect::<Vec<_>>() }],
        "scene": 0,
    });
    if let Some(v) = img {
        json["images"] = serde_json::json!([{ "bufferView": v, "mimeType": "image/png" }]);
        json["textures"] = serde_json::json!([{ "source": 0 }]);
        material["pbrMetallicRoughness"]["baseColorTexture"] = serde_json::json!({ "index": 0 });
    }
    json["materials"] = serde_json::json!([material]);
    let mut text = serde_json::to_vec(&json).unwrap();
    while !text.len().is_multiple_of(4) {
        text.push(b' ');
    }
    while !bin.len().is_multiple_of(4) {
        bin.push(0);
    }
    let mut out = Vec::new();
    out.extend_from_slice(b"glTF");
    out.extend_from_slice(&2u32.to_le_bytes());
    out.extend_from_slice(&((28 + text.len() + bin.len()) as u32).to_le_bytes());
    out.extend_from_slice(&(text.len() as u32).to_le_bytes());
    out.extend_from_slice(b"JSON");
    out.extend_from_slice(&text);
    out.extend_from_slice(&(bin.len() as u32).to_le_bytes());
    out.extend_from_slice(b"BIN\0");
    out.extend_from_slice(&bin);
    out
}

/// A `size` × `size` PNG of four coloured quadrants.
fn png(size: u32) -> Vec<u8> {
    let img = image::RgbaImage::from_fn(size, size, |x, y| match (x * 2 / size, y * 2 / size) {
        (0, 0) => image::Rgba([200, 40, 30, 255]),
        (1, 0) => image::Rgba([40, 160, 60, 255]),
        (0, 1) => image::Rgba([30, 60, 200, 255]),
        _ => image::Rgba([230, 200, 60, 255]),
    });
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png).unwrap();
    out.into_inner()
}

const PROJECT: &str = "type project\nversion 1\n---\nlook \"looks/main.ogt\"\n";

fn look(blocks: &str) -> String {
    format!("type look\nversion 1\n---\n{blocks}")
}

fn project(name: &str, files: &[(&str, Vec<u8>)]) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-cook-conform")
        .join(name);
    let _ = std::fs::remove_dir_all(&root);
    for (path, bytes) in files {
        let p = root.join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, bytes).unwrap();
    }
    root
}

fn cook(root: &Path) -> og_cook::cook::Report {
    let report = cook_project(root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    report
}

/// Every file under `cooked/`, by path.
fn cooked_files(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(dir: &Path, base: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for e in std::fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                walk(&p, base, out);
            } else if p.ends_with(".cook-times.ogt") {
                // Wall-clock times, never the same twice.
            } else {
                out.insert(
                    p.strip_prefix(base).unwrap().display().to_string(),
                    std::fs::read(&p).unwrap(),
                );
            }
        }
    }
    let mut out = BTreeMap::new();
    let dir = root.join("cooked");
    walk(&dir, &dir, &mut out);
    out
}

fn notes(report: &og_cook::cook::Report, source: &str) -> Vec<String> {
    report
        .assets
        .iter()
        .find(|a| a.source == source)
        .map(|a| a.notes.clone())
        .unwrap_or_default()
}

/// The art contract's entry for `source` in the report (one per asset).
fn entry<'a>(report: &'a og_cook::cook::Report, source: &str) -> &'a ConformEntry {
    let mut it = report.conform.iter().filter(|e| e.source == source);
    let e = it
        .next()
        .unwrap_or_else(|| panic!("no entry for {source}: {:?}", report.conform));
    assert!(
        it.next().is_none(),
        "one entry for {source}: {:?}",
        report.conform
    );
    e
}

/// Per primitive of the model: its LOD 0 triangles as three vertices each.
fn triangles(root: &Path, id: &str) -> Vec<Vec<[og_render::Vertex; 3]>> {
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let (model, _) = bundle.load_model_with_textures(id).expect("decodes");
    model
        .primitives
        .iter()
        .map(|p| {
            let l = p.lods[0];
            p.indices[l.first..l.first + l.count]
                .as_chunks::<3>()
                .0
                .iter()
                .map(|t| [0, 1, 2].map(|k| p.vertices[t[k] as usize]))
                .collect()
        })
        .collect()
}

fn walker() -> Vec<u8> {
    std::fs::read(support::template_model("walker.glb")).unwrap()
}

#[test]
fn without_a_look_or_with_zero_art_blocks_the_cook_is_byte_identical() {
    let ball = glb(
        &sphere(0.5, [0.0, 0.5, 0.0], 32, 16),
        1,
        [0.8, 0.3, 0.2, 1.0],
        Some(&png(256)),
    );
    let assets = |look_text: &str| {
        vec![
            ("assets/models/ball.glb", ball.clone()),
            ("assets/models/walker.glb", walker()),
            // A texture no model claims: a mesh's in a 3D project.
            ("assets/textures/stone.png", png(256)),
            ("assets/looks/main.ogt", look(look_text).into_bytes()),
        ]
    };
    let frame_only = "Stylise {\n  posterize 4\n}\n";
    let zero = "ArtShape {\n  propFaces 0\n  characterFaces 0\n  voxel 0\n  bevel 0\n  normals any\n}\nArtTexels {\n  density 0\n  maxSize 0\n}\nArtMotion {\n  stepFps 0\n  maxFrames 0\n}\nArtLine {\n  width 0\n}\nArtPalette {\n  maxColors 16\n}\n";
    for (name, text) in [("frame-only", frame_only), ("zero", zero)] {
        // Before: no project.ogt; after: project.ogt names the look.
        let before = project(&format!("same-before-{name}"), &assets(text));
        let mut with = assets(text);
        with.push(("project.ogt", PROJECT.as_bytes().to_vec()));
        let after = project(&format!("same-after-{name}"), &with);
        cook(&before);
        let report = cook(&after);
        assert!(
            notes(&report, "models/ball.glb")
                .iter()
                .all(|n| !n.starts_with("conform"))
        );
        assert!(report.conform.is_empty(), "{:?}", report.conform);
        let (a, b) = (cooked_files(&before), cooked_files(&after));
        assert_eq!(
            a.keys().collect::<Vec<_>>(),
            b.keys().collect::<Vec<_>>(),
            "{name}"
        );
        for (k, v) in &a {
            assert!(v == &b[k], "{name}: cooked/{k} differs");
        }
    }
}

#[test]
fn a_model_past_the_face_budget_is_decimated_counting_every_node_that_draws_it() {
    let s = sphere(0.5, [0.0, 0.5, 0.0], 48, 24);
    let source_tris = s.3.len() / 3;
    assert!(source_tris > 2000);
    let shape = "ArtShape {\n  propFaces 500\n}\n";
    for nodes in [1usize, 2] {
        let root = project(
            &format!("budget-{nodes}"),
            &[
                ("project.ogt", PROJECT.as_bytes().to_vec()),
                ("assets/looks/main.ogt", look(shape).into_bytes()),
                ("assets/models/ball.glb", glb(&s, nodes, [1.0; 4], None)),
            ],
        );
        let report = cook(&root);
        let tris: usize = triangles(&root, "models/ball.glb")
            .iter()
            .map(|p| p.len())
            .sum();
        // Drawn by `nodes` nodes: the budget holds for all of them.
        assert!(
            tris * nodes <= 500,
            "{nodes} node(s): {tris} triangles each"
        );
        assert!(tris * nodes > 250, "decimated far below the budget: {tris}");
        let e = entry(&report, "models/ball.glb");
        assert_eq!(e.action, ConformAction::Changed);
        assert!(e.what.contains("propFaces 500"), "{e:?}");
    }
}

/// The share of triangle corners whose normal is within a degree of its
/// face's normal (from the positions).
fn flat_share(tris: &[[og_render::Vertex; 3]]) -> f64 {
    let v = |p: [f32; 3]| glam::Vec3::from(p);
    let mut flat = 0;
    for t in tris {
        let face = (v(t[1].position) - v(t[0].position))
            .cross(v(t[2].position) - v(t[0].position))
            .normalize();
        flat += t
            .iter()
            .filter(|c| v(c.normal).normalize().dot(face) > 1f32.to_radians().cos())
            .count();
    }
    flat as f64 / (tris.len() * 3) as f64
}

#[test]
fn normals_flat_shades_every_face_with_its_own_normal_and_bakes_its_colour() {
    let s = sphere(0.5, [0.0, 0.5, 0.0], 24, 12);
    let files = |l: &str| {
        vec![
            ("project.ogt", PROJECT.as_bytes().to_vec()),
            ("assets/looks/main.ogt", look(l).into_bytes()),
            (
                "assets/models/ball.glb",
                glb(&s, 1, [1.0; 4], Some(&png(64))),
            ),
        ]
    };
    // The control: the smooth source keeps almost no corner on its face normal.
    let plain = project("flat-control", &files("Stylise {\n  posterize 4\n}\n"));
    cook(&plain);
    let smooth = flat_share(&triangles(&plain, "models/ball.glb")[0]);
    assert!(smooth < 0.1, "the source is smooth: {smooth}");
    let root = project(
        "flat",
        &files("ArtShape {\n  normals flat\n}\nArtPalette {\n  maxColors 4\n}\n"),
    );
    let report = cook(&root);
    let tris = &triangles(&root, "models/ball.glb")[0];
    assert_eq!(flat_share(tris), 1.0);
    // Untextured: one colour per face, at most maxColors of them, and the
    // material white without its texture.
    let mut colours = std::collections::BTreeSet::new();
    for t in tris {
        assert_eq!(t[0].color, t[1].color);
        assert_eq!(t[1].color, t[2].color);
        colours.insert(t[0].color.map(|c| (c * 255.0).round() as u8));
    }
    assert!(colours.len() > 1 && colours.len() <= 4, "{colours:?}");
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let (model, _) = bundle.load_model_with_textures("models/ball.glb").unwrap();
    assert_eq!(model.materials[0].base_color, [1.0; 4]);
    assert_eq!(model.materials[0].base_color_texture, None);
    assert!(
        entry(&report, "models/ball.glb")
            .what
            .contains("normals flat")
    );
}

#[test]
fn voxels_sit_on_the_grid_of_their_edge() {
    // Off the grid on purpose: the voxels are not.
    let s = sphere(0.47, [0.033, 0.51, -0.017], 32, 16);
    let edge = 0.1f32;
    let root = project(
        "voxel",
        &[
            ("project.ogt", PROJECT.as_bytes().to_vec()),
            ("assets/looks/main.ogt", look("ArtShape {\n  normals flat\n  voxel 0.1\n}\nArtPalette {\n  colors \"@palettes/dusk16\"\n}\n").into_bytes()),
            ("assets/models/ball.glb", glb(&s, 1, [0.9, 0.4, 0.2, 1.0], None)),
        ],
    );
    let report = cook(&root);
    let tris = &triangles(&root, "models/ball.glb")[0];
    assert!(!tris.is_empty());
    // 16-bit positions over about 1 m: a grid point decodes within 1e-4.
    for c in tris.iter().flatten() {
        for x in c.position {
            let off = (x / edge - (x / edge).round()).abs() * edge;
            assert!(off < 1e-4, "{:?} is off the {edge} m grid", c.position);
        }
        let axes = c.normal.iter().filter(|n| n.abs() > 0.999).count();
        assert_eq!(axes, 1, "a voxel face is axis-aligned: {:?}", c.normal);
    }
    // The ball fills its grid: at least the cells whose centre is inside
    // it, at most the cells that touch it.
    let what = &entry(&report, "models/ball.glb").what;
    let voxels: usize = what
        .split("voxelised at 0.1 m (")
        .nth(1)
        .and_then(|x| x.split(' ').next()?.parse().ok())
        .unwrap_or_else(|| panic!("{what}"));
    let (c, r) = ([0.033f64, 0.51, -0.017], 0.47f64);
    let (mut inside, mut touching) = (0, 0);
    for x in -10..10 {
        for y in -5..15 {
            for z in -10..10 {
                let lo = [x, y, z].map(|k| k as f64 * 0.1);
                let centre: f64 = (0..3).map(|a| (lo[a] + 0.05 - c[a]).powi(2)).sum();
                let near: f64 = (0..3)
                    .map(|a| (c[a].clamp(lo[a], lo[a] + 0.1) - c[a]).powi(2))
                    .sum();
                inside += usize::from(centre <= r * r);
                touching += usize::from(near <= r * r);
            }
        }
    }
    assert!(
        (inside..=touching).contains(&voxels),
        "{voxels} voxels, {inside} to {touching} expected"
    );
    // Every colour is one of dusk16's.
    let dusk: Vec<[u8; 3]> = include_str!("../../../templates/looks/assets/palettes/dusk16.hex")
        .lines()
        .filter_map(|l| {
            let h = l.trim().trim_start_matches('#');
            (h.len() == 6).then(|| [0, 2, 4].map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap()))
        })
        .collect();
    let srgb = |v: f32| {
        let s = if v <= 0.0031308 {
            v * 12.92
        } else {
            1.055 * v.powf(1.0 / 2.4) - 0.055
        };
        (s * 255.0).round() as i32
    };
    for c in tris.iter().flatten() {
        let got = [0, 1, 2].map(|i| srgb(c.color[i]));
        assert!(
            dusk.iter()
                .any(|d| (0..3).all(|i| (d[i] as i32 - got[i]).abs() <= 1)),
            "{got:?} is not a dusk16 colour"
        );
    }
}

/// og-model §5's generators under `normals flat`, as the art samples' hero
/// is made: lathed parts with poles (sphere, capsule, cone) and a part
/// turned 45° about z. Every decoded corner sits on its face's normal:
/// a pole's zero-area triangle decodes as a sliver facing any way, and a
/// turned part's voxel normals came back turned twice (90° off).
#[test]
fn flat_normals_hold_on_poles_and_on_turned_voxelised_parts() {
    const MODEL: &str = "type model\nversion 1\n---\nmaterial \"m\" {\n  color 0.5 0.4 0.3\n}\npart \"head\" {\n  material \"m\"\n  position 0 1.55 0\n  sphereGeometry {\n    radius 0.17\n    widthSegments 16\n    heightSegments 12\n  }\n}\npart \"hood\" {\n  material \"m\"\n  position 0 1.8 0\n  coneGeometry {\n    radius 0.25\n    height 0.55\n    radialSegments 16\n  }\n}\npart \"arm\" {\n  material \"m\"\n  position 0.33 1.12 0.12\n  quaternion 0 0 0.3827 0.9239\n  capsuleGeometry {\n    radius 0.07\n    height 0.4\n    radialSegments 10\n  }\n}\npart \"lamp\" {\n  material \"m\"\n  position 0.6 0.8 0\n  quaternion 0 0 0.3827 0.9239\n  boxGeometry {\n    width 0.4\n    height 0.2\n    depth 0.2\n  }\n}\n";
    for (name, shape) in [
        ("flat-poles", "ArtShape {\n  normals flat\n}\n"),
        (
            "flat-voxels",
            "ArtShape {\n  normals flat\n  voxel 0.1\n}\n",
        ),
    ] {
        let root = project(
            name,
            &[
                ("project.ogt", PROJECT.as_bytes().to_vec()),
                ("assets/looks/main.ogt", look(shape).into_bytes()),
                ("assets/models/hero.ogt", MODEL.as_bytes().to_vec()),
            ],
        );
        cook(&root);
        let tris: Vec<[og_render::Vertex; 3]> = triangles(&root, "models/hero.ogt").concat();
        assert!(!tris.is_empty());
        assert_eq!(flat_share(&tris), 1.0, "{name}");
    }
}

#[test]
fn a_model_within_its_budget_still_holds_its_voxels_to_it() {
    // A 6 m ball of 48 triangles is far under propFaces 300, so no
    // decimation is planned; at 0.1 m its shell alone is about
    // 4 pi 6^2 / 0.01 = 45,000 cell faces, so the edge must double until
    // the ball meets 300 faces (or the edge reaches 16 times, 1.6 m).
    let s = sphere(6.0, [0.0, 6.0, 0.0], 8, 4);
    let root = project(
        "voxel-budget",
        &[
            ("project.ogt", PROJECT.as_bytes().to_vec()),
            (
                "assets/looks/main.ogt",
                look("ArtShape {\n  propFaces 300\n  normals flat\n  voxel 0.1\n}\n").into_bytes(),
            ),
            (
                "assets/models/hill.glb",
                glb(&s, 1, [0.3, 0.6, 0.3, 1.0], None),
            ),
        ],
    );
    let report = cook(&root);
    let what = &entry(&report, "models/hill.glb").what;
    let edge: f32 = what
        .split("voxelised at ")
        .nth(1)
        .and_then(|x| x.split(' ').next()?.parse().ok())
        .unwrap_or_else(|| panic!("{what}"));
    let faces: usize = triangles(&root, "models/hill.glb")
        .iter()
        .map(Vec::len)
        .sum();
    assert!(
        edge > 0.1,
        "the edge stayed at {edge} m with {faces} faces: {what}"
    );
    assert!(
        faces <= 300 || edge >= 1.6 - 1e-4,
        "{faces} faces at {edge} m: {what}"
    );
}

#[test]
fn textures_are_resampled_to_the_density_and_capped_at_max_size() {
    // A ball of radius 1 m (two nodes 3 m apart), a 256 texture laid once
    // over it: one side of the texture spans about 4.1 m of its surface
    // (the area-weighted median of sqrt(2 pi^2 sin theta) over its bands).
    let s = sphere(1.0, [0.0, 1.0, 0.0], 16, 8);
    for (blocks, want, pixel) in [
        // 4.1 m × 16 texels a metre = 66, rounded to a power of two: 64.
        (
            "ArtTexels {\n  density 16\n}\nArtShape {\n  normals smooth\n}\n",
            64u32,
            false,
        ),
        (
            "ArtTexels {\n  maxSize 32\n  filter nearest\n}\nArtShape {\n  normals smooth\n}\n",
            32,
            true,
        ),
    ] {
        let root = project(
            &format!("texels-{want}"),
            &[
                ("project.ogt", PROJECT.as_bytes().to_vec()),
                ("assets/looks/main.ogt", look(blocks).into_bytes()),
                (
                    "assets/models/ball.glb",
                    glb(&s, 2, [1.0; 4], Some(&png(256))),
                ),
            ],
        );
        cook(&root);
        let bundle = Bundle::open(&root.join("cooked")).unwrap();
        let tex = bundle.load_texture("models/ball.glb#tex0").unwrap();
        assert_eq!((tex.meta.width, tex.meta.height), (want, want));
        assert_eq!(tex.is_pixel_art(), pixel);
        // The model names it as pixel art, so its loader samples it nearest.
        let (model, _) = bundle.load_model_with_textures("models/ball.glb").unwrap();
        assert_eq!(
            model.textures[0].kind == og_cook::format::kind::PIXEL,
            pixel
        );
    }
}

/// A `side` m square on the ground whose UVs run from 0 to `uv` across it.
fn plane(side: f32, uv: f32) -> Mesh {
    let h = side / 2.0;
    (
        vec![[-h, 0.0, -h], [h, 0.0, -h], [h, 0.0, h], [-h, 0.0, h]],
        vec![[0.0, 1.0, 0.0]; 4],
        vec![[0.0, 0.0], [uv, 0.0], [uv, uv], [0.0, uv]],
        vec![0, 2, 1, 0, 3, 2],
    )
}

#[test]
fn a_texture_is_sized_by_the_surface_its_uvs_lay_it_over_not_the_model_extent() {
    // ArtTexels.density counts texels per metre of the surface, as og
    // style check measures them through the UVs. A 4 m ground repeating
    // its 256 texture every metre draws it over 1 m: 16 texels a metre is
    // a 16 texture. A 4 m plane that shows a quarter of its texture (an
    // atlas: a crate's face) spreads one texture side over 16 m: 16 a
    // metre is 256, the texture kept. The model's 4 m extent gives 64 to
    // both, four times too dense on the ground and a quarter too sparse
    // on the atlas.
    for (uv, want) in [(4.0, 16u32), (0.25, 256)] {
        let root = project(
            &format!("texel-span-{want}"),
            &[
                ("project.ogt", PROJECT.as_bytes().to_vec()),
                (
                    "assets/looks/main.ogt",
                    look("ArtTexels {\n  density 16\n}\nArtShape {\n  normals smooth\n}\n")
                        .into_bytes(),
                ),
                (
                    "assets/models/ground.glb",
                    glb(&plane(4.0, uv), 1, [1.0; 4], Some(&png(256))),
                ),
            ],
        );
        cook(&root);
        let bundle = Bundle::open(&root.join("cooked")).unwrap();
        let tex = bundle.load_texture("models/ground.glb#tex0").unwrap();
        assert_eq!(
            (tex.meta.width, tex.meta.height),
            (want, want),
            "uv 0 to {uv}"
        );
    }
}

#[test]
fn a_stepped_clip_holds_each_pose_at_the_step_rate_and_drops_frames_past_the_limit() {
    let files = |l: &str| {
        vec![
            ("project.ogt", PROJECT.as_bytes().to_vec()),
            ("assets/looks/main.ogt", look(l).into_bytes()),
            ("assets/models/walker.glb", walker()),
        ]
    };
    let root = project("motion", &files("ArtMotion {\n  stepFps 12\n}\n"));
    let report = cook(&root);
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let anim = bundle.load_animation("models/walker.glb").unwrap();
    assert!(!anim.clips.is_empty());
    for c in &anim.clips {
        assert_eq!(c.rate, 12.0, "{}", c.name);
        // Keys at i / 12 from 0 to the clip's end, the last at its end.
        let expected = (c.duration as f64 * 12.0 - 1e-4).ceil() as u32 + 1;
        assert_eq!(c.frames, expected, "{}", c.name);
        assert!(
            c.channels.iter().all(|ch| ch.step),
            "{}: every channel holds",
            c.name
        );
    }
    assert!(
        entry(&report, "models/walker.glb")
            .what
            .contains("stepped at 12 fps")
    );
    let root = project(
        "motion-max",
        &files("ArtMotion {\n  stepFps 12\n  maxFrames 5\n}\n"),
    );
    let report = cook(&root);
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    for c in &bundle.load_animation("models/walker.glb").unwrap().clips {
        assert!(c.frames <= 5, "{}: {} frames", c.name, c.frames);
    }
    // Frames dropped are a warning of the cook, and in the entry.
    assert!(
        report
            .warnings
            .iter()
            .any(|w| w.starts_with("assets/models/walker.glb: warning: ")
                && w.contains("maxFrames 5")),
        "{:?}",
        report.warnings
    );
    assert!(
        entry(&report, "models/walker.glb")
            .what
            .contains("maxFrames 5")
    );
}

#[test]
fn conform_off_keeps_a_model_as_made_and_the_report_says_so() {
    let ball = glb(&sphere(0.5, [0.0, 0.5, 0.0], 32, 16), 1, [1.0; 4], None);
    let off = b"type import\nversion 1\n---\nconform off\n".to_vec();
    let a = project(
        "off",
        &[
            ("project.ogt", PROJECT.as_bytes().to_vec()),
            (
                "assets/looks/main.ogt",
                look("ArtShape {\n  propFaces 100\n  normals flat\n}\n").into_bytes(),
            ),
            ("assets/models/ball.glb", ball.clone()),
            ("assets/models/ball.glb.ogt", off.clone()),
        ],
    );
    let b = project(
        "off-control",
        &[
            (
                "assets/looks/main.ogt",
                look("ArtShape {\n  propFaces 100\n  normals flat\n}\n").into_bytes(),
            ),
            ("assets/models/ball.glb", ball),
            ("assets/models/ball.glb.ogt", off),
        ],
    );
    let report = cook(&a);
    cook(&b);
    let (fa, fb) = (cooked_files(&a), cooked_files(&b));
    let model = fa.keys().find(|k| k.contains("ball")).unwrap();
    assert!(fa[model] == fb[model]);
    let e = entry(&report, "models/ball.glb");
    assert_eq!(e.action, ConformAction::Off);
    assert!(
        e.what.contains("conform off (assets/models/ball.glb.ogt)"),
        "{e:?}"
    );
    assert!(
        notes(&report, "models/ball.glb")
            .iter()
            .any(|n| n.starts_with("art contract: off"))
    );
}

#[test]
fn conformed_output_is_deterministic() {
    let files = vec![
        ("project.ogt", PROJECT.as_bytes().to_vec()),
        ("assets/looks/main.ogt", look("ArtShape {\n  propFaces 300\n  normals flat\n  bevel 0.02\n}\nArtLine {\n  width 2\n}\nArtPalette {\n  maxColors 8\n}\n").into_bytes()),
        ("assets/models/ball.glb", glb(&sphere(0.5, [0.0, 0.5, 0.0], 32, 16), 1, [1.0; 4], Some(&png(64)))),
    ];
    let (a, b) = (project("det-a", &files), project("det-b", &files));
    cook(&a);
    cook(&b);
    assert!(cooked_files(&a) == cooked_files(&b));
}

/// project.ogt with the look and `conform` lines.
fn project_with(conform: &str) -> Vec<u8> {
    format!("{PROJECT}conform {{\n{conform}}}\n").into_bytes()
}

/// A flipbook of three frames at 8 fps: 0.125 s each, which 12 fps steps
/// hold for two steps (1/6 s).
fn coin() -> [(&'static str, Vec<u8>); 2] {
    [
        ("assets/sprites/coin.png", png(24)),
        (
            "assets/sprites/coin.ogt",
            b"type flipbook\nversion 1\n---\ntexture \"coin.png\"\ngrid 8 8 3 3\nclip spin {\n  frames 0 1 2\n  fps 8\n}\n".to_vec(),
        ),
    ]
}

fn coin_durations(root: &Path) -> Vec<f32> {
    let b = Bundle::open(&root.join("cooked")).unwrap();
    let bytes = b.read("sprites/coin.ogt").unwrap();
    b.flipbook("sprites/coin.ogt", bytes.as_slice())
        .unwrap()
        .clips[0]
        .durations
        .clone()
}

#[test]
fn project_conform_off_keeps_models_their_textures_clips_and_flipbooks_as_made() {
    const LOOK: &str = "ArtShape {\n  propFaces 100\n  normals flat\n}\nArtTexels {\n  maxSize 32\n}\nArtMotion {\n  stepFps 12\n}\n";
    let ball = glb(
        &sphere(0.5, [0.0, 0.5, 0.0], 32, 16),
        1,
        [1.0; 4],
        Some(&png(128)),
    );
    let files = |project: Vec<u8>| {
        let mut f = vec![
            ("project.ogt", project),
            ("assets/looks/main.ogt", look(LOOK).into_bytes()),
            ("assets/models/ball.glb", ball.clone()),
            ("assets/models/walker.glb", walker()),
        ];
        f.extend(coin());
        f
    };
    // The control: no look at all, so nothing is conformed.
    let control = project(
        "project-off-control",
        &files(b"type project\nversion 1\n---\n".to_vec()),
    );
    cook(&control);
    // Conformed: all of it moves.
    let on = project("project-off-on", &files(PROJECT.as_bytes().to_vec()));
    let report_on = cook(&on);
    let (fc, fo) = (cooked_files(&control), cooked_files(&on));
    for id in ["models/ball.glb", "models/walker.glb"] {
        assert_eq!(entry(&report_on, id).action, ConformAction::Changed, "{id}");
    }
    // Simplified (within the shape's error, so not necessarily to 100:
    // its uv seam and poles hold).
    assert!(
        triangles(&on, "models/ball.glb")[0].len()
            < triangles(&control, "models/ball.glb")[0].len()
    );
    let tex = |root: &Path| {
        let t = Bundle::open(&root.join("cooked"))
            .unwrap()
            .load_texture("models/ball.glb#tex0")
            .unwrap();
        (t.meta.width, t.meta.height)
    };
    assert_eq!(tex(&on), (32, 32));
    let held = 2.0f32 / 12.0;
    assert!(coin_durations(&on).iter().all(|d| (d - held).abs() < 1e-5));
    assert_eq!(
        entry(&report_on, "sprites/coin.ogt").action,
        ConformAction::Changed
    );

    // Off by pattern: models/ by a folder, the flipbook by its id.
    let off = project(
        "project-off",
        &files(project_with(
            "  off \"models/\"\n  off \"sprites/coin.ogt\"\n",
        )),
    );
    let report = cook(&off);
    let fx = cooked_files(&off);
    // The model, its texture and its clips: the control's bytes.
    let mut same = 0;
    for (k, v) in &fc {
        if k.contains("ball") || k.contains("walker") {
            assert!(fx.get(k) == Some(v), "{k} differs from the control");
            same += 1;
        }
    }
    assert!(same >= 2, "{:?}", fc.keys());
    assert_eq!(tex(&off), (128, 128));
    // ...where conforming moves every one of them.
    for k in fc
        .keys()
        .filter(|k| k.contains("ball") || k.contains("walker"))
    {
        assert!(fo.get(k) != fc.get(k), "{k}: conformed when on");
    }
    // The flipbook keeps 8 fps.
    assert!(
        coin_durations(&off)
            .iter()
            .all(|d| (d - 0.125).abs() < 1e-5)
    );
    for (id, pattern) in [
        ("models/ball.glb", "off \"models/\""),
        ("models/walker.glb", "off \"models/\""),
        ("sprites/coin.ogt", "off \"sprites/coin.ogt\""),
    ] {
        let e = entry(&report, id);
        assert_eq!(e.action, ConformAction::Off, "{id}");
        assert!(
            e.what.contains("project.ogt") && e.what.contains(pattern),
            "{id}: {e:?}"
        );
    }
    assert!(report.warnings.is_empty(), "{:?}", report.warnings);
}

#[test]
fn the_report_lists_the_mesh_pass_beside_the_texture_pass_and_keeps_it_when_up_to_date() {
    let files = vec![
        ("project.ogt", PROJECT.as_bytes().to_vec()),
        (
            "assets/looks/main.ogt",
            look("ArtShape {\n  propFaces 200\n}\nArtMotion {\n  stepFps 12\n  maxFrames 5\n}\n")
                .into_bytes(),
        ),
        // Past the budget: decimated.
        (
            "assets/models/ball.glb",
            glb(&sphere(0.5, [0.0, 0.5, 0.0], 32, 16), 1, [1.0; 4], None),
        ),
        // Inside it, smooth as asked for nothing: as made.
        (
            "assets/models/cube.glb",
            glb(&sphere(0.5, [0.0, 0.5, 0.0], 4, 2), 1, [1.0; 4], None),
        ),
        ("assets/models/walker.glb", walker()),
    ];
    let root = project("report", &files);
    let first = cook(&root);
    let ball = entry(&first, "models/ball.glb");
    assert_eq!(ball.action, ConformAction::Changed);
    assert!(ball.what.contains("propFaces 200"), "{ball:?}");
    let cube = entry(&first, "models/cube.glb");
    assert_eq!(cube.action, ConformAction::Unchanged, "{cube:?}");
    // No `conform:` line is left among the notes: the entry is the record.
    for a in &first.assets {
        assert!(
            a.notes.iter().all(|n| !n.starts_with("conform:")),
            "{:?}",
            a.notes
        );
    }
    assert!(first.warnings.iter().any(|w| w.contains("maxFrames 5")));
    let md = first.to_markdown();
    assert!(
        md.contains("`models/ball.glb`: art contract: changed"),
        "{md}"
    );
    let json = first.to_json();
    assert!(
        json.contains("\"conform\"") && json.contains("models/cube.glb"),
        "{json}"
    );
    // Up to date: nothing cooks again, the same entries and warnings.
    let second = cook(&root);
    assert!(
        second
            .assets
            .iter()
            .flat_map(|a| &a.outputs)
            .all(|o| !o.cooked),
        "nothing cooks twice"
    );
    assert_eq!(
        format!("{:?}", second.conform),
        format!("{:?}", first.conform)
    );
    assert_eq!(second.warnings, first.warnings);
}

/// A 3D project's look holding textures to 64 texels, nearest, no mips,
/// and the dusk16 palette strictly.
const TEXELS: &str = "ArtTexels {\n  maxSize 64\n  filter nearest\n  mips none\n}\nArtPalette {\n  colors \"@palettes/dusk16\"\n  strict true\n}\n";

fn stone(name: &str, profile: &str, import: Option<&str>) -> PathBuf {
    let mut files = vec![
        ("project.ogt", format!("{PROJECT}{profile}").into_bytes()),
        ("assets/looks/main.ogt", look(TEXELS).into_bytes()),
        ("assets/textures/stone.png", png(256)),
    ];
    if let Some(i) = import {
        files.push((
            "assets/textures/stone.png.ogt",
            format!("type import\nversion 1\n---\n{i}").into_bytes(),
        ));
    }
    project(name, &files)
}

#[test]
fn a_texture_no_model_claims_is_a_mesh_texture_in_3d_unless_its_own_line_says_otherwise() {
    use og_cook::cook::conform::{TextureRole, texture_roles};
    let texture = |root: &Path| {
        let t = Bundle::open(&root.join("cooked"))
            .unwrap()
            .load_texture("textures/stone.png")
            .unwrap();
        (t.meta.width, t.meta.height, t.is_pixel_art(), t.meta.levels)
    };
    let role = |root: &Path| texture_roles(root).unwrap()["textures/stone.png"];

    // Unclaimed in a 3D project: 256 → maxSize 64, nearest, one level.
    let mesh = stone("stone-mesh", "", None);
    let report = cook(&mesh);
    assert_eq!(texture(&mesh), (64, 64, true, 1));
    assert_eq!(role(&mesh), TextureRole::Mesh);
    let e = entry(&report, "textures/stone.png");
    assert_eq!(e.action, ConformAction::Changed);
    assert!(
        e.what
            .starts_with(og_cook::cook::conform::mesh::DEFAULT_MESH_TEXTURE)
            && e.what.contains("256x256 → 64x64")
            && e.what.contains("ArtPalette.colors"),
        "{e:?}"
    );

    // Not strict: a lossless (nearest) texture holds to maxColors.
    let gradient = image::RgbaImage::from_fn(64, 64, |x, y| {
        image::Rgba([(x * 4) as u8, (y * 4) as u8, ((x + y) * 2) as u8, 255])
    });
    let mut bytes = std::io::Cursor::new(Vec::new());
    gradient
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap();
    let few = project(
        "stone-few",
        &[
            ("project.ogt", PROJECT.as_bytes().to_vec()),
            (
                "assets/looks/main.ogt",
                look("ArtTexels {\n  filter nearest\n}\nArtPalette {\n  maxColors 8\n}\n")
                    .into_bytes(),
            ),
            ("assets/textures/stone.png", bytes.into_inner()),
        ],
    );
    let report = cook(&few);
    let t = Bundle::open(&few.join("cooked"))
        .unwrap()
        .load_texture("textures/stone.png")
        .unwrap()
        .to_texture_data()
        .unwrap();
    let distinct: std::collections::BTreeSet<&[u8]> = t.rgba.chunks(4).map(|p| &p[..3]).collect();
    assert!(
        distinct.len() > 1 && distinct.len() <= 8,
        "{} colours",
        distinct.len()
    );
    assert!(
        entry(&report, "textures/stone.png")
            .what
            .contains("4096 → 8 colours (ArtPalette.maxColors 8)"),
        "{:?}",
        report.conform
    );

    // `conform ui`: the 2D pass's fixed layout (palette, size kept).
    let ui = stone("stone-ui", "", Some("conform ui\n"));
    let report = cook(&ui);
    let (w, h, ..) = texture(&ui);
    assert_eq!((w, h), (256, 256));
    assert_eq!(role(&ui), TextureRole::Fixed);
    let e = entry(&report, "textures/stone.png");
    assert!(
        e.what.contains("conform ui") && !e.what.contains("mesh texture"),
        "{e:?}"
    );

    // `conform off`: the bytes of a project without a look, listed off.
    let off = stone("stone-off", "", Some("conform off\n"));
    let report = cook(&off);
    let control = project(
        "stone-control",
        &[
            ("assets/textures/stone.png", png(256)),
            (
                "assets/textures/stone.png.ogt",
                b"type import\nversion 1\n---\nconform off\n".to_vec(),
            ),
        ],
    );
    cook(&control);
    let (fa, fb) = (cooked_files(&off), cooked_files(&control));
    let k = fa
        .keys()
        .find(|k| k.contains("stone"))
        .unwrap_or_else(|| panic!("{:?}", fa.keys()));
    assert!(fa[k] == fb[k], "{k} differs from the control");
    let e = entry(&report, "textures/stone.png");
    assert_eq!(e.action, ConformAction::Off);
    assert!(
        e.what
            .contains("conform off (assets/textures/stone.png.ogt)"),
        "{e:?}"
    );

    // project.ogt's pattern does the same.
    let pattern = stone("stone-pattern", "conform {\n  off \"textures/\"\n}\n", None);
    let report = cook(&pattern);
    assert_eq!(texture(&pattern).0, 256);
    let e = entry(&report, "textures/stone.png");
    assert_eq!(e.action, ConformAction::Off);
    assert!(e.what.contains("off \"textures/\""), "{e:?}");

    // A 2D project: a sprite, the 2D pass's; the mesh pass leaves it.
    let two_d = stone("stone-2d", "profile \"2d\"\n", None);
    let report = cook(&two_d);
    assert_eq!(role(&two_d), TextureRole::Sprite);
    for e in report
        .conform
        .iter()
        .filter(|e| e.source == "textures/stone.png")
    {
        assert!(!e.what.contains("mesh texture"), "{e:?}");
    }
    assert!(
        notes(&report, "textures/stone.png")
            .iter()
            .all(|n| !n.contains("as a mesh's texture"))
    );
}

/// A glb of one mesh per part, each its own material, drawn once each.
fn glb_parts(parts: &[Mesh]) -> Vec<u8> {
    let mut bin: Vec<u8> = Vec::new();
    let (mut views, mut accessors, mut meshes) = (Vec::new(), Vec::new(), Vec::new());
    let mut add = |bin: &mut Vec<u8>, bytes: Vec<u8>| {
        views.push(
            serde_json::json!({ "buffer": 0, "byteOffset": bin.len(), "byteLength": bytes.len() }),
        );
        bin.extend(bytes);
        views.len() - 1
    };
    let f = |v: &[f32]| v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>();
    for (k, (p, n, uv, idx)) in parts.iter().enumerate() {
        let (lo, hi) = p
            .iter()
            .fold(([f32::MAX; 3], [f32::MIN; 3]), |(lo, hi), v| {
                (
                    [0, 1, 2].map(|i| lo[i].min(v[i])),
                    [0, 1, 2].map(|i| hi[i].max(v[i])),
                )
            });
        let a = accessors.len();
        accessors.push(serde_json::json!({ "bufferView": add(&mut bin, f(&p.concat())), "componentType": 5126, "count": p.len(), "type": "VEC3", "min": lo, "max": hi }));
        accessors.push(serde_json::json!({ "bufferView": add(&mut bin, f(&n.concat())), "componentType": 5126, "count": n.len(), "type": "VEC3" }));
        accessors.push(serde_json::json!({ "bufferView": add(&mut bin, f(&uv.concat())), "componentType": 5126, "count": uv.len(), "type": "VEC2" }));
        accessors.push(serde_json::json!({ "bufferView": add(&mut bin, idx.iter().flat_map(|x| x.to_le_bytes()).collect()), "componentType": 5125, "count": idx.len(), "type": "SCALAR" }));
        meshes.push(serde_json::json!({ "primitives": [{ "attributes": { "POSITION": a, "NORMAL": a + 1, "TEXCOORD_0": a + 2 }, "indices": a + 3, "material": k }] }));
    }
    let json = serde_json::json!({
        "asset": { "version": "2.0" },
        "buffers": [{ "byteLength": bin.len() }],
        "bufferViews": views,
        "accessors": accessors,
        "meshes": meshes,
        "materials": (0..parts.len()).map(|k| serde_json::json!({ "pbrMetallicRoughness": { "baseColorFactor": [0.2 + 0.05 * k as f32, 0.5, 0.7, 1.0] } })).collect::<Vec<_>>(),
        "nodes": (0..parts.len()).map(|k| serde_json::json!({ "mesh": k })).collect::<Vec<_>>(),
        "scenes": [{ "nodes": (0..parts.len()).collect::<Vec<_>>() }],
        "scene": 0,
    });
    let mut text = serde_json::to_vec(&json).unwrap();
    while !text.len().is_multiple_of(4) {
        text.push(b' ');
    }
    let mut out = Vec::new();
    out.extend_from_slice(b"glTF");
    out.extend_from_slice(&2u32.to_le_bytes());
    out.extend_from_slice(&((28 + text.len() + bin.len()) as u32).to_le_bytes());
    out.extend_from_slice(&(text.len() as u32).to_le_bytes());
    out.extend_from_slice(b"JSON");
    out.extend_from_slice(&text);
    out.extend_from_slice(&(bin.len() as u32).to_le_bytes());
    out.extend_from_slice(b"BIN\0");
    out.extend_from_slice(&bin);
    out
}

/// A triangle soup's edges used by one triangle only, and its connected
/// pieces, both by position (a flat-shaded or seamed mesh splits its
/// corners; the surface is what counts).
fn open_edges_and_pieces(tris: &[[[f32; 3]; 3]]) -> (usize, usize) {
    // On a 0.1 mm grid: the fixture's seam and poles meet to within float
    // error, the cooked positions to within their 16-bit quantisation.
    let key = |p: [f32; 3]| p.map(|x| (x * 1e4).round() as i64);
    let mut ids: BTreeMap<[i64; 3], usize> = BTreeMap::new();
    let mut id = |p: [f32; 3]| {
        let n = ids.len();
        *ids.entry(key(p)).or_insert(n)
    };
    let t: Vec<[usize; 3]> = tris.iter().map(|t| t.map(&mut id)).collect();
    let mut edges: BTreeMap<(usize, usize), usize> = BTreeMap::new();
    let mut parent: Vec<usize> = (0..ids.len()).collect();
    fn find(p: &mut [usize], x: usize) -> usize {
        let mut r = x;
        while p[r] != r {
            r = p[r];
        }
        p[x] = r;
        r
    }
    for f in &t {
        for k in 0..3 {
            let (a, b) = (f[k], f[(k + 1) % 3]);
            if a != b {
                *edges.entry((a.min(b), a.max(b))).or_default() += 1;
            }
            let (ra, rb) = (find(&mut parent, a), find(&mut parent, b));
            parent[ra] = rb;
        }
    }
    let used: BTreeSet<usize> = t.iter().flatten().copied().collect();
    let pieces: BTreeSet<usize> = used.iter().map(|&v| find(&mut parent, v)).collect();
    (edges.values().filter(|&&n| n == 1).count(), pieces.len())
}

fn positions(tris: &[[og_render::Vertex; 3]]) -> Vec<[[f32; 3]; 3]> {
    tris.iter().map(|t| t.map(|v| v.position)).collect()
}

fn source_tris(m: &Mesh) -> Vec<[[f32; 3]; 3]> {
    m.3.chunks(3)
        .map(|t| [0, 1, 2].map(|k| m.0[t[k] as usize]))
        .collect()
}

#[test]
fn decimation_keeps_every_part_closed_and_whole_and_lands_at_the_budget() {
    // A hull and ten small closed parts, each its own mesh and material
    // (a dropship's panels and lights): proportional shares give a small
    // part a handful of triangles, which only a destructive simplifier
    // reaches.
    let mut parts = vec![sphere(0.5, [0.0, 0.5, 0.0], 48, 24)];
    for k in 0..10 {
        let a = k as f32 * std::f32::consts::TAU / 10.0;
        parts.push(sphere(0.06, [0.6 * a.cos(), 0.5, 0.6 * a.sin()], 12, 6));
    }
    let drawn: usize = parts.iter().map(|m| m.3.len() / 3).sum();
    assert!(drawn > 2600);
    for m in &parts {
        // The fixture: every part closed and one piece.
        assert_eq!(open_edges_and_pieces(&source_tris(m)), (0, 1));
    }
    let root = project(
        "closed",
        &[
            ("project.ogt", PROJECT.as_bytes().to_vec()),
            (
                "assets/looks/main.ogt",
                look("ArtShape {\n  propFaces 500\n  normals flat\n}\n").into_bytes(),
            ),
            ("assets/models/parts.glb", glb_parts(&parts)),
        ],
    );
    let report = cook(&root);
    let cooked = triangles(&root, "models/parts.glb");
    assert_eq!(cooked.len(), parts.len(), "no part is lost");
    let total: usize = cooked.iter().map(Vec::len).sum();
    for (i, p) in cooked.iter().enumerate() {
        assert_eq!(
            open_edges_and_pieces(&positions(p)),
            (0, 1),
            "part {i} ({} triangles) is no longer one closed piece",
            p.len()
        );
    }
    // At the budget, not far below it.
    assert!(total <= 500 && total > 450, "{total} triangles");
    let e = entry(&report, "models/parts.glb");
    assert!(
        e.what.contains("decimated") && !e.what.contains("over budget"),
        "{e:?}"
    );
    assert!(report.warnings.is_empty(), "{:?}", report.warnings);
}

#[test]
fn a_model_that_cannot_meet_the_budget_within_its_silhouette_is_kept_and_warned_about() {
    let (r, c) = (0.5f32, [0.0f32, 0.5, 0.0]);
    let s = sphere(r, c, 48, 24);
    let root = project(
        "over",
        &[
            ("project.ogt", PROJECT.as_bytes().to_vec()),
            (
                "assets/looks/main.ogt",
                look("ArtShape {\n  propFaces 20\n  silhouette 0.002\n  normals flat\n}\n")
                    .into_bytes(),
            ),
            ("assets/models/ball.glb", glb(&s, 1, [1.0; 4], None)),
        ],
    );
    let report = cook(&root);
    let tris = &triangles(&root, "models/ball.glb")[0];
    assert!(tris.len() > 20, "{} triangles", tris.len());
    assert!(tris.len() < s.3.len() / 3, "simplified as far as it may");
    assert_eq!(open_edges_and_pieces(&positions(tris)), (0, 1));
    // The surface moved at most the silhouette: each face's centre lies
    // within it of the source's faces, which sit at most one ring step's
    // sag inside the sphere (r (1 - cos 7.5°)).
    let sag = r * (1.0 - (std::f32::consts::PI / 24.0).cos());
    for t in tris {
        let m = [0, 1, 2].map(|k| (t[0].position[k] + t[1].position[k] + t[2].position[k]) / 3.0);
        let d = (0..3).map(|k| (m[k] - c[k]).powi(2)).sum::<f32>().sqrt();
        assert!(
            d <= r + 1e-3 && d >= r - sag - 0.002 - 1e-3,
            "a face {:.4} m from the centre of a {r} m sphere",
            d
        );
    }
    let e = entry(&report, "models/ball.glb");
    assert!(e.what.contains("over budget"), "{e:?}");
    assert!(
        report.warnings.iter().any(|w| w
            .starts_with("assets/models/ball.glb: warning: over budget")
            && w.contains("propFaces 20")
            && w.contains("ArtShape.silhouette")),
        "{:?}",
        report.warnings
    );
}
