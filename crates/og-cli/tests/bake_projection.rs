//! `og bake projection` as a user drives it: init → plan → prepare → project
//! → export on a small level, with a flat colour standing in for the model.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Value, json};

const PAINT: [u8; 3] = [200, 60, 30];

fn og(run: &Path, args: &[&str]) -> Value {
    let out = Command::new(env!("CARGO_BIN_EXE_og"))
        .args(["bake", "projection"])
        .args(args)
        .args(["--run", run.to_str().unwrap(), "--threads", "2"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "og bake projection {args:?}: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).unwrap()
}

/// A 20 m floor (uv left half) and a 6 × 3 m wall behind it (uv right half),
/// as a binary glTF with one material each; without `TEXCOORD_0` when
/// `uvs` is false.
fn level_glb(uvs: bool) -> Vec<u8> {
    let pos: [[f32; 3]; 8] = [
        [-10.0, 0.0, -10.0],
        [10.0, 0.0, -10.0],
        [10.0, 0.0, 10.0],
        [-10.0, 0.0, 10.0],
        [-3.0, 0.0, -4.0],
        [3.0, 0.0, -4.0],
        [3.0, 3.0, -4.0],
        [-3.0, 3.0, -4.0],
    ];
    let uv: [[f32; 2]; 8] = [
        [0.02, 0.02],
        [0.48, 0.02],
        [0.48, 0.98],
        [0.02, 0.98],
        [0.52, 0.98],
        [0.98, 0.98],
        [0.98, 0.02],
        [0.52, 0.02],
    ];
    let floor: [u16; 6] = [0, 2, 1, 0, 3, 2];
    let wall: [u16; 6] = [4, 5, 6, 4, 6, 7];
    let mut bin = Vec::new();
    pos.iter()
        .flatten()
        .for_each(|f| bin.extend(f.to_le_bytes()));
    uv.iter()
        .flatten()
        .for_each(|f| bin.extend(f.to_le_bytes()));
    floor
        .iter()
        .chain(&wall)
        .for_each(|i| bin.extend(i.to_le_bytes()));
    let mut doc = json!({
        "asset": {"version": "2.0"},
        "scene": 0, "scenes": [{"nodes": [0]}], "nodes": [{"mesh": 0}],
        "meshes": [{"primitives": [
            {"attributes": {"POSITION": 0, "TEXCOORD_0": 1}, "indices": 2, "material": 0},
            {"attributes": {"POSITION": 0, "TEXCOORD_0": 1}, "indices": 3, "material": 1}]}],
        "materials": [{"name": "floor"}, {"name": "wall"}],
        "buffers": [{"byteLength": bin.len()}],
        "bufferViews": [
            {"buffer": 0, "byteOffset": 0, "byteLength": 96},
            {"buffer": 0, "byteOffset": 96, "byteLength": 64},
            {"buffer": 0, "byteOffset": 160, "byteLength": 24}],
        "accessors": [
            {"bufferView": 0, "componentType": 5126, "count": 8, "type": "VEC3",
             "min": [-10.0, 0.0, -10.0], "max": [10.0, 3.0, 10.0]},
            {"bufferView": 1, "componentType": 5126, "count": 8, "type": "VEC2"},
            {"bufferView": 2, "byteOffset": 0, "componentType": 5123, "count": 6, "type": "SCALAR"},
            {"bufferView": 2, "byteOffset": 12, "componentType": 5123, "count": 6, "type": "SCALAR"}]
    });
    if !uvs {
        for p in doc["meshes"][0]["primitives"].as_array_mut().unwrap() {
            p["attributes"]
                .as_object_mut()
                .unwrap()
                .remove("TEXCOORD_0");
        }
    }
    let mut js = serde_json::to_vec(&doc).unwrap();
    while !js.len().is_multiple_of(4) {
        js.push(b' ');
    }
    let mut glb = Vec::new();
    glb.extend(b"glTF");
    glb.extend(2u32.to_le_bytes());
    glb.extend(((12 + 8 + js.len() + 8 + bin.len()) as u32).to_le_bytes());
    glb.extend((js.len() as u32).to_le_bytes());
    glb.extend(b"JSON");
    glb.extend(js);
    glb.extend((bin.len() as u32).to_le_bytes());
    glb.extend(b"BIN\0");
    glb.extend(bin);
    glb
}

fn scratch(name: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("og-bake-projection-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn a_painted_view_ends_up_in_the_exported_level_texture() {
    let (dir, init, glb) = paint_and_export("uvs", true);
    assert_eq!(init["run"]["unwrapped"], json!(false));
    // The level's own uvs are kept: the floor stays in the left half.
    let (doc, buffers, _) = gltf::import(&glb).unwrap();
    let floor = doc.meshes().next().unwrap().primitives().next().unwrap();
    let r = floor.reader(|b| Some(&buffers[b.index()].0[..]));
    let uv: Vec<[f32; 2]> = r.read_tex_coords(0).unwrap().into_f32().collect();
    let used: Vec<[f32; 2]> = r
        .read_indices()
        .unwrap()
        .into_u32()
        .map(|i| uv[i as usize])
        .collect();
    assert!(used.iter().all(|t| t[0] <= 0.5), "{used:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_level_without_uvs_is_charted_and_painted_like_one_with_them() {
    let (dir, init, glb) = paint_and_export("no-uvs", false);
    assert_eq!(init["run"]["unwrapped"], json!(true));
    // The charts: every surface at one texel density, inside the page,
    // with the level where it was.
    let (doc, buffers, _) = gltf::import(&glb).unwrap();
    let mut density = Vec::new();
    let mut uv_total = 0.0;
    let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
    for prim in doc
        .meshes()
        .flat_map(|m| m.primitives().collect::<Vec<_>>())
    {
        let r = prim.reader(|b| Some(&buffers[b.index()].0[..]));
        let pos: Vec<[f32; 3]> = r.read_positions().unwrap().collect();
        let uv: Vec<[f32; 2]> = r
            .read_tex_coords(0)
            .expect("charted uvs")
            .into_f32()
            .collect();
        let idx: Vec<u32> = r.read_indices().unwrap().into_u32().collect();
        assert!(
            uv.iter().flatten().all(|c| (0.0..=1.0).contains(c)),
            "uv outside the page"
        );
        for p in &pos {
            for c in 0..3 {
                lo[c] = lo[c].min(p[c]);
                hi[c] = hi[c].max(p[c]);
            }
        }
        let (mut world, mut tex) = (0.0f64, 0.0f64);
        for t in idx.chunks(3) {
            let [a, b, c] = [t[0], t[1], t[2]].map(|i| pos[i as usize]);
            let e = |p: [f32; 3], q: [f32; 3]| {
                [
                    (q[0] - p[0]) as f64,
                    (q[1] - p[1]) as f64,
                    (q[2] - p[2]) as f64,
                ]
            };
            let (u, v) = (e(a, b), e(a, c));
            let n = [
                u[1] * v[2] - u[2] * v[1],
                u[2] * v[0] - u[0] * v[2],
                u[0] * v[1] - u[1] * v[0],
            ];
            world += 0.5 * (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            let [p, q, s] = [t[0], t[1], t[2]].map(|i| uv[i as usize]);
            tex += 0.5
                * (((q[0] - p[0]) * (s[1] - p[1]) - (q[1] - p[1]) * (s[0] - p[0])) as f64).abs();
        }
        density.push(tex / world);
        uv_total += tex;
    }
    assert_eq!(density.len(), 2, "one primitive per material");
    // The floor (400 m²) and the wall (18 m²) get the same texels per m².
    assert!(
        (density[0] / density[1] - 1.0).abs() < 0.1,
        "densities {density:?}"
    );
    assert!(uv_total > 0.2 && uv_total <= 1.0, "uv area {uv_total}");
    assert_eq!((lo, hi), ([-10.0, 0.0, -10.0], [10.0, 3.0, 10.0]));
    let _ = std::fs::remove_dir_all(&dir);
}

/// init → plan → prepare → project a flat colour → export, checking that
/// exactly the projected texels carry it and every material samples the
/// texture; the run's init output and the exported glb.
fn paint_and_export(name: &str, uvs: bool) -> (PathBuf, Value, PathBuf) {
    let dir = scratch(name);
    let src = dir.join("level.src.glb");
    std::fs::write(&src, level_glb(uvs)).unwrap();
    let run = dir.join("run");

    let init = og(
        &run,
        &[
            "init",
            src.to_str().unwrap(),
            "--size",
            "256",
            "--view-size",
            "256",
        ],
    );
    assert_eq!(init["run"]["labels"], json!(["floor", "wall"]));
    let plan = og(&run, &["plan", "--views", "2"]);
    assert!(plan["views"].as_u64().unwrap() >= 1, "{plan}");

    let prep = og(&run, &["prepare", "--view", "0"]);
    assert_eq!(prep["skip"], json!(false), "{prep}");
    // The stand-in model paints every pixel one colour.
    let painted = dir.join("painted.png");
    image::RgbaImage::from_pixel(256, 256, image::Rgba([PAINT[0], PAINT[1], PAINT[2], 255]))
        .save(&painted)
        .unwrap();
    let proj = og(
        &run,
        &[
            "project",
            "--view",
            "0",
            "--image",
            painted.to_str().unwrap(),
        ],
    );
    let coverage = proj["coverage"].as_f64().unwrap();
    assert!(
        proj["written"].as_u64().unwrap() > 0 && coverage > 0.05,
        "{proj}"
    );

    let export = og(&run, &["export", "--padding", "0"]);
    let png = std::fs::read(export["texture"].as_str().unwrap()).unwrap();
    let tex = image::load_from_memory(&png).unwrap().to_rgb8();
    let texels = init["run"]["texels"].as_u64().unwrap() as f64;
    let painted_texels = tex
        .pixels()
        .filter(|p| p.0.iter().zip(PAINT).all(|(a, b)| a.abs_diff(b) <= 2))
        .count() as f64;
    // Without padding, exactly the projected texels carry the paint.
    assert!(
        (painted_texels / texels - coverage).abs() < 0.01,
        "{painted_texels} of {texels} texels vs coverage {coverage}"
    );

    // Every material of the exported level samples that texture.
    let (doc, _, images) = gltf::import(export["glb"].as_str().unwrap()).unwrap();
    assert_eq!(doc.materials().len(), 2);
    for m in doc.materials() {
        let t = m
            .pbr_metallic_roughness()
            .base_color_texture()
            .expect("material without base colour texture");
        let img = &images[t.texture().source().index()];
        assert_eq!((img.width, img.height), (tex.width(), tex.height()));
        assert_eq!(img.pixels[..], tex.as_raw()[..]);
    }
    (dir, init, PathBuf::from(export["glb"].as_str().unwrap()))
}
