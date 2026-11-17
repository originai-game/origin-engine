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

use og_anim::cooked::BundleAnim;
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
    let nodes = (0..nodes)
        .map(|i| serde_json::json!({ "mesh": 0, "translation": [i as f32 * 3.0, 0.0, 0.0] }))
        .collect();
    glb_nodes(mesh, nodes, color, png)
}

/// [`glb`] with the scene's root nodes as given (each draws mesh 0).
fn glb_nodes(
    mesh: &Mesh,
    nodes: Vec<serde_json::Value>,
    color: [f32; 4],
    png: Option<&[u8]>,
) -> Vec<u8> {
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
        "scenes": [{ "nodes": (0..nodes.len()).collect::<Vec<_>>() }],
        "nodes": nodes,
        "scene": 0,
    });
    if let Some(v) = img {
        json["images"] = serde_json::json!([{ "bufferView": v, "mimeType": "image/png" }]);
        json["textures"] = serde_json::json!([{ "source": 0 }]);
        material["pbrMetallicRoughness"]["baseColorTexture"] = serde_json::json!({ "index": 0 });
    }
    json["materials"] = serde_json::json!([material]);
    glb_bytes(&json, bin)
}

fn glb_bytes(json: &serde_json::Value, mut bin: Vec<u8>) -> Vec<u8> {
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
    let report =
        og_cook::cook::cook_project_with_texture_cache(root, &CookOptions::default(), None)
            .unwrap();
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

/// gltfpack-style integer positions with a tiny dequantization bind. The
/// skinned node's scale is deliberately 100x smaller: glTF ignores it.
#[test]
fn quantized_skin_bounds_and_texture_density_use_the_bind_and_uv_transform() {
    use serde_json::json;
    let mut bin = Vec::new();
    let mut views = Vec::new();
    let mut add = |bytes: Vec<u8>| {
        while !bin.len().is_multiple_of(4) {
            bin.push(0);
        }
        views.push(json!({"buffer": 0, "byteOffset": bin.len(), "byteLength": bytes.len()}));
        bin.extend(bytes);
        views.len() - 1
    };
    let pos = add([0u16, 0, 0, 32767, 0, 0, 0, 65534, 0]
        .into_iter()
        .flat_map(u16::to_le_bytes)
        .collect());
    let uv = add([0.0f32, 0.0, 0.25, 0.0, 0.0, 0.5]
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect());
    let joints = add(vec![0; 12]);
    let weights = add([1.0f32, 0.0, 0.0, 0.0]
        .repeat(3)
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect());
    let bind = add(glam::Mat4::from_scale(glam::Vec3::splat(1.82 / 65534.0))
        .to_cols_array()
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect());
    let image = add(png(1024));
    let json = json!({
        "asset": {"version": "2.0"},
        "extensionsUsed": ["KHR_mesh_quantization", "KHR_texture_transform"],
        "extensionsRequired": ["KHR_mesh_quantization"],
        "buffers": [{"byteLength": bin.len()}], "bufferViews": views,
        "accessors": [
            {"bufferView": pos, "componentType": 5123, "count": 3, "type": "VEC3", "min": [0,0,0], "max": [32767,65534,0]},
            {"bufferView": uv, "componentType": 5126, "count": 3, "type": "VEC2"},
            {"bufferView": joints, "componentType": 5121, "count": 3, "type": "VEC4"},
            {"bufferView": weights, "componentType": 5126, "count": 3, "type": "VEC4"},
            {"bufferView": bind, "componentType": 5126, "count": 1, "type": "MAT4"}
        ],
        "meshes": [{"primitives": [{"attributes": {"POSITION": 0, "TEXCOORD_0": 1, "JOINTS_0": 2, "WEIGHTS_0": 3}, "material": 0}]}],
        "nodes": [{"mesh": 0, "skin": 0, "scale": [0.0000002777142,0.0000002777142,0.0000002777142]}, {}],
        "skins": [{"joints": [1], "inverseBindMatrices": 4}],
        "scenes": [{"nodes": [0,1]}], "scene": 0,
        "materials": [{"pbrMetallicRoughness": {"baseColorTexture": {"index": 0, "extensions": {"KHR_texture_transform": {"scale": [4,2]}}}}}],
        "textures": [{"source": 0}], "images": [{"bufferView": image, "mimeType": "image/png"}]
    });
    let bytes = glb_bytes(&json, bin);
    let raw = og_cook::runtime::UncookedGltf::read("courier.glb", &bytes).unwrap();
    let primitives = (0..raw.primitive_count())
        .map(|p| raw.primitive(p).unwrap())
        .collect();
    let raw = raw.finish(primitives);
    assert!(
        (raw.max.y - raw.min.y - 1.82).abs() < 1e-5,
        "uncooked bounds {:?} {:?}",
        raw.min,
        raw.max
    );
    let root = project(
        "quantized-skin-density",
        &[
            ("project.ogt", PROJECT.as_bytes().to_vec()),
            (
                "assets/looks/main.ogt",
                look("ArtTexels {\n  density 128\n}\n").into_bytes(),
            ),
            ("assets/models/courier.glb", bytes),
        ],
    );
    let report = cook(&root);
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let model = bundle.load_model("models/courier.glb").unwrap();
    let draws = model.node_draws(&model.node_world(), 0);
    let p = &model.primitives[0];
    let height = draws[0].transform_point3(p.max).y - draws[0].transform_point3(p.min).y;
    assert!((height - 1.82).abs() < 1e-5, "height {height}");
    assert!((model.max.y - model.min.y - 1.82).abs() < 1e-5);
    // World area = .91*1.82/2, transformed UV area = 1*1/2.
    // sqrt(area ratio) = 1.287 m, *128 rounds to 128, not 512 (UV ignored)
    // or 4 (tiny bind rejected). The report also retains the physical measurement.
    assert!(
        notes(&report, "models/courier.glb")
            .iter()
            .any(|n| n.contains("1.29 m")),
        "{:?}",
        notes(&report, "models/courier.glb")
    );
    let texture = bundle.load_texture(&model.textures[0].id).unwrap();
    assert_eq!((texture.meta.width, texture.meta.height), (128, 128));
}

#[test]
fn atlas_reduction_keeps_small_islands_and_dilates_their_own_colours() {
    let colour = |col: u32, row: u32| {
        if (col + row).is_multiple_of(2) {
            [240, 80, 20, 255]
        } else {
            [20, 180, 230, 255]
        }
    };
    let image = image::RgbaImage::from_fn(256, 256, |x, y| {
        image::Rgba(
            if (9..25).contains(&(x % 64)) && (9..33).contains(&(y % 64)) {
                colour(x / 64, y / 64)
            } else {
                [180, 0, 180, 255]
            },
        )
    });
    let mut png = std::io::Cursor::new(Vec::new());
    image.write_to(&mut png, image::ImageFormat::Png).unwrap();
    let mut mesh: Mesh = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for row in 0..4 {
        for col in 0..4 {
            let base = mesh.0.len() as u32;
            for (x, y) in [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)] {
                mesh.0.push([col as f32 + x, row as f32 + y, 0.0]);
                mesh.1.push([0.0, 0.0, 1.0]);
                mesh.2.push([
                    (col as f32 * 64.0 + 9.0 + x * 16.0) / 256.0,
                    (row as f32 * 64.0 + 9.0 + y * 24.0) / 256.0,
                ]);
            }
            mesh.3
                .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }
    let root = project(
        "many-island-atlas",
        &[
            ("project.ogt", PROJECT.as_bytes().to_vec()),
            (
                "assets/looks/main.ogt",
                look("ArtTexels {\n  maxSize 8\n  filter nearest\n  mips none\n}\n").into_bytes(),
            ),
            (
                "assets/models/atlas.glb",
                glb(&mesh, 1, [1.0; 4], Some(png.get_ref())),
            ),
        ],
    );
    cook(&root);
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let model = bundle.load_model("models/atlas.glb").unwrap();
    let texture = bundle.load_texture(&model.textures[0].id).unwrap();
    // A 16-texel source island needs four texels across: 256/4 = 64.
    assert_eq!((texture.meta.width, texture.meta.height), (64, 64));
    let texture = texture.to_texture_data().unwrap();
    for row in 0..4 {
        for col in 0..4 {
            for (x, y) in [(1, 1), (1, 4), (2, 4), (3, 4), (6, 4)] {
                // Diagonal/left gutters, fractional chart boundary, interior, right gutter.
                let at = (((row * 16 + y) * 64 + col * 16 + x) * 4) as usize;
                assert_eq!(
                    &texture.rgba[at..at + 4],
                    &colour(col, row),
                    "island ({col},{row}), x {x}"
                );
            }
        }
    }
}

#[test]
fn face_budgets_simplify_uv_charts_without_losing_their_attributes() {
    let original = sphere(1.0, [0.0; 3], 64, 32);
    let mut mesh: Mesh = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    // Eight adjacent texture charts: the same surface positions have
    // different UVs on either side of each seam.
    for tri in original.3.as_chunks::<3>().0 {
        let chart =
            (tri.iter().map(|&i| original.2[i as usize][0]).sum::<f32>() / 3.0 * 8.0).floor();
        for &i in tri {
            let i = i as usize;
            mesh.3.push(mesh.0.len() as u32);
            mesh.0.push(original.0[i]);
            mesh.1.push(original.1[i]);
            mesh.2
                .push([original.2[i][0] * 8.0 - chart, original.2[i][1]]);
        }
    }
    let root = project(
        "seamed-budget",
        &[
            ("project.ogt", PROJECT.as_bytes().to_vec()),
            (
                "assets/looks/main.ogt",
                look("ArtShape {\n propFaces 1000\n silhouette 0.05\n}\n").into_bytes(),
            ),
            (
                "assets/models/ball.glb",
                glb(&mesh, 1, [1.0; 4], Some(&png(32))),
            ),
        ],
    );
    cook(&root);
    let tris = triangles(&root, "models/ball.glb");
    assert!(tris[0].len() <= 1000, "{} triangles", tris[0].len());
    for corner in tris[0].iter().flatten() {
        let p = glam::Vec3::from(corner.position);
        assert!((p.length() - 1.0).abs() < 0.001, "source surface retained");
        // Surviving corners keep the UV of the source corner at this
        // position, not an arbitrary corner welded across the chart.
        assert!(
            mesh.0.iter().zip(&mesh.2).any(|(&source, &uv)| {
                glam::Vec3::from(source).distance(p) < 0.0001
                    && glam::Vec2::from(uv).distance(glam::Vec2::from(corner.uv)) < 0.0001
            }),
            "source UV retained: {corner:?}"
        );
    }
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

#[test]
fn a_model_whose_triangles_all_cook_away_is_refused() {
    // Every triangle on one line: no area. Flat shading leaves none of
    // them, so the model would load with no mesh; the cook says so.
    let (mut p, n, uv, idx) = sphere(1.0, [0.0; 3], 8, 6);
    for v in &mut p {
        *v = [v[0], 0.0, 0.0];
    }
    let root = project(
        "no-area",
        &[
            ("project.ogt", PROJECT.as_bytes().to_vec()),
            (
                "assets/looks/main.ogt",
                look("ArtShape {\n  normals flat\n}\n").into_bytes(),
            ),
            (
                "assets/models/line.glb",
                glb(&(p, n, uv, idx), 1, [1.0; 4], None),
            ),
        ],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(
        report
            .errors()
            .iter()
            .any(|e| e.contains("models/line.glb") && e.contains("nothing left to draw")),
        "{:?}",
        report.errors()
    );
}

/// A mesh under a mirrored node (glTF: a negative determinant turns the
/// winding the renderer takes as the front) is voxelised in model space
/// and brought back under the node: its triangles stay counter-clockwise
/// from outside in the mesh's own space, as the unmirrored node's are, and
/// agree with their normals. Before, they came back wound clockwise, right
/// only under a renderer that ignored the node's mirror.
#[test]
fn voxels_under_a_mirrored_node_keep_the_mesh_s_winding() {
    use glam::Vec3;
    let volume = |name: &str, scale: [f32; 3]| {
        let root = project(
            name,
            &[
                ("project.ogt", PROJECT.as_bytes().to_vec()),
                (
                    "assets/looks/main.ogt",
                    look("ArtShape {\n  voxel 0.25\n}\n").into_bytes(),
                ),
                (
                    "assets/models/ball.glb",
                    glb_nodes(
                        &sphere(1.0, [0.5, 0.0, 0.0], 16, 12),
                        vec![serde_json::json!({ "mesh": 0, "scale": scale })],
                        [1.0; 4],
                        None,
                    ),
                ),
            ],
        );
        let report = cook(&root);
        assert!(
            notes(&report, "models/ball.glb")
                .iter()
                .any(|n| n.contains("voxel")),
            "{:?}",
            notes(&report, "models/ball.glb")
        );
        let tris = triangles(&root, "models/ball.glb");
        let (mut volume, mut against) = (0.0, 0);
        for t in tris.iter().flatten() {
            let [a, b, c] = t.map(|v| Vec3::from(v.position));
            let face = (b - a).cross(c - a);
            volume += a.dot(b.cross(c)) / 6.0;
            against += (face.dot(Vec3::from(t[0].normal)) <= 0.0) as usize;
        }
        let faces = tris.iter().flatten().count();
        assert_eq!(
            against, 0,
            "{name}: of {faces} faces, wound against their normals"
        );
        volume
    };
    let plain = volume("voxel-plain", [1.0, 1.0, 1.0]);
    let mirrored = volume("voxel-mirrored", [-1.0, 1.0, 1.0]);
    assert!(plain > 3.0, "the ball's volume {plain}");
    assert!(
        (mirrored - plain).abs() < 0.05 * plain,
        "mirrored {mirrored} against {plain}"
    );
}

/// A ground voxelised keeps its height: its cubes lie under the surface,
/// their top faces on it, so what stands on the ground is not buried.
/// Before, each surface point filled the cell it lay in, and a ground at
/// y = 0 became a slab from 0 to one edge up (a whole cube over its feet;
/// with the edge doubled past the face budget, 1.6 m over the props).
#[test]
fn a_voxelised_ground_keeps_its_height() {
    let ground: Mesh = (
        vec![
            [-2.0, 0.0, -2.0],
            [2.0, 0.0, -2.0],
            [2.0, 0.0, 2.0],
            [-2.0, 0.0, 2.0],
        ],
        vec![[0.0, 1.0, 0.0]; 4],
        vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        vec![0, 3, 2, 0, 2, 1],
    );
    let root = project(
        "voxel-ground",
        &[
            ("project.ogt", PROJECT.as_bytes().to_vec()),
            (
                "assets/looks/main.ogt",
                look("ArtShape {\n  voxel 0.25\n}\n").into_bytes(),
            ),
            (
                "assets/models/ground.glb",
                glb(&ground, 1, [0.3, 0.5, 0.2, 1.0], None),
            ),
        ],
    );
    let report = cook(&root);
    assert!(
        notes(&report, "models/ground.glb")
            .iter()
            .any(|n| n.contains("voxel")),
        "{:?}",
        notes(&report, "models/ground.glb")
    );
    let ys: Vec<f32> = triangles(&root, "models/ground.glb")
        .iter()
        .flatten()
        .flatten()
        .map(|v| v.position[1])
        .collect();
    let (lo, hi) = ys
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), &y| (a.min(y), b.max(y)));
    assert!(hi.abs() < 1e-4, "the ground's top at {hi}, not 0");
    assert!(
        (lo + 0.25).abs() < 1e-4,
        "one cube deep: its bottom at {lo}"
    );
}

/// A voxelised model past its face budget keeps the look's cube edge and
/// warns: before, the edge doubled until the budget was met (a rolling
/// ground at 0.1 m became steps 1.6 m high, walls over the scene).
#[test]
fn a_voxelised_model_past_its_budget_keeps_its_edge() {
    let root = project(
        "voxel-over",
        &[
            ("project.ogt", PROJECT.as_bytes().to_vec()),
            (
                "assets/looks/main.ogt",
                look("ArtShape {\n  propFaces 60\n  voxel 0.1\n}\n").into_bytes(),
            ),
            (
                "assets/models/ball.glb",
                glb(
                    &sphere(1.0, [0.0, 1.0, 0.0], 16, 8),
                    1,
                    [0.3, 0.5, 0.2, 1.0],
                    None,
                ),
            ),
        ],
    );
    let report = cook(&root);
    let n = notes(&report, "models/ball.glb");
    assert!(
        n.iter()
            .any(|x| x.contains("voxelised at 0.1 m") && !x.contains("coarsened")),
        "{n:?}"
    );
    assert!(
        report.warnings.iter().any(|w| w.contains("over budget")),
        "{:?}",
        report.warnings
    );
    // The cubes are 0.1 m: every vertex on the 0.1 m grid, and the sphere's
    // curve stepped finely (far more triangles than 60).
    let tris = triangles(&root, "models/ball.glb");
    let all: Vec<[f32; 3]> = tris
        .iter()
        .flatten()
        .flatten()
        .map(|v| v.position)
        .collect();
    assert!(all.len() / 3 > 60, "{}", all.len() / 3);
    for p in &all {
        for a in p {
            assert!(
                ((a * 10.0) - (a * 10.0).round()).abs() < 1e-3,
                "{p:?} off the 0.1 m grid"
            );
        }
    }
}
