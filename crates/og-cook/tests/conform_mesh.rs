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
    // The model's own texture, by the id its blob names (unconformed, it is
    // the one cooked texture of its image, shared by id).
    let tex = |root: &Path| {
        let bundle = Bundle::open(&root.join("cooked")).unwrap();
        let id = bundle.load_model("models/ball.glb").unwrap().textures[0]
            .id
            .clone();
        let t = bundle.load_texture(&id).unwrap();
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
