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

use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use std::collections::BTreeMap;
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
        let n = notes(&report, "models/ball.glb");
        assert!(
            n.iter()
                .any(|x| x.starts_with("conform:") && x.contains("propFaces 500")),
            "{n:?}"
        );
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
        notes(&report, "models/ball.glb")
            .iter()
            .any(|n| n.contains("normals flat"))
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
    let n = notes(&report, "models/ball.glb");
    let voxels: usize = n
        .iter()
        .find_map(|x| {
            x.split("voxelised at 0.1 m (")
                .nth(1)?
                .split(' ')
                .next()?
                .parse()
                .ok()
        })
        .unwrap_or_else(|| panic!("{n:?}"));
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

#[test]
fn textures_are_resampled_to_the_density_and_capped_at_max_size() {
    // A 2 m wide model (two nodes 3 m apart: 5 m extent), a 256 texture.
    let s = sphere(1.0, [0.0, 1.0, 0.0], 16, 8);
    for (blocks, want, pixel) in [
        // 5 m × 16 texels a metre = 80, rounded to a power of two: 64.
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
        notes(&report, "models/walker.glb")
            .iter()
            .any(|n| n.contains("stepped at 12 fps"))
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
    assert!(
        notes(&report, "models/walker.glb")
            .iter()
            .any(|n| n.contains("warning") && n.contains("maxFrames 5"))
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
    assert!(
        notes(&report, "models/ball.glb")
            .iter()
            .any(|n| n.starts_with("conform: off"))
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
