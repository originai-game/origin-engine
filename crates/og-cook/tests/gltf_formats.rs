//! The glTF forms commercial and optimised assets ship in, through the
//! cook: Draco and meshopt compression, quantised attributes, Basis and
//! WebP images, and the texture sources beside PNG and JPEG (WebP, TGA,
//! PSD, DDS). Each fixture under `fixtures/gltf/` is small and made as
//! its README says; the cooked data is compared with the plain source.
//!
//! Ways this could fail: a compressed file is refused (the gltf crate
//! refuses an `extensionsRequired` it does not know); a decoded accessor
//! is read with the wrong stride, normalisation or component type, so the
//! positions or uvs move; gltfpack's dequantisation (its node scale and
//! its `KHR_texture_transform`) is dropped, so the box comes out tiny or
//! its uvs stretched; an image in a KTX2, WebP, DDS or PSD decodes to the
//! wrong texels or is flipped.

use og_cook::runtime::{Bundle, CookedModel};
use og_cook::{CookOptions, cook_project};
use std::path::PathBuf;

pub fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/gltf")
        .join(name)
}

/// Cooks `files` (a name under `fixtures/gltf/` and where it goes under
/// `assets/`) as a project of their own; the bundle and every asset's
/// report notes, one a line.
pub fn cook(project: &str, files: &[(&str, &str)]) -> (Bundle, String) {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("og-cook-gltf-{project}"));
    let _ = std::fs::remove_dir_all(&root);
    for (from, to) in files {
        let to = root.join("assets").join(to);
        std::fs::create_dir_all(to.parent().unwrap()).unwrap();
        std::fs::copy(fixture(from), &to).unwrap();
    }
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{project}: {:?}", report.errors());
    let notes = report
        .assets
        .iter()
        .flat_map(|a| a.notes.iter().map(move |n| format!("{}: {n}\n", a.source)))
        .collect();
    (Bundle::open(&root.join("cooked")).unwrap(), notes)
}

/// The source box (`box.gltf`): its corners, and each vertex's position
/// with its uv.
const LO: [f32; 3] = [-1.0, 0.0, -0.5];
const HI: [f32; 3] = [2.0, 3.0, 0.5];

/// A vertex: its position and uv.
type Corner = ([f32; 3], [f32; 2]);

fn source_vertices() -> Vec<Corner> {
    let text = std::fs::read_to_string(fixture("box.gltf")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    let uri = json["buffers"][0]["uri"].as_str().unwrap();
    let b64 = uri.split_once(";base64,").unwrap().1;
    let bin = base64(b64);
    let f32s = |view: usize| -> Vec<f32> {
        let v = &json["bufferViews"][view];
        let (o, l) = (
            v["byteOffset"].as_u64().unwrap() as usize,
            v["byteLength"].as_u64().unwrap() as usize,
        );
        bin[o..o + l]
            .as_chunks::<4>()
            .0
            .iter()
            .map(|c| f32::from_le_bytes(*c))
            .collect()
    };
    let (p, uv) = (f32s(0), f32s(2));
    (0..24)
        .map(|i| {
            (
                [p[i * 3], p[i * 3 + 1], p[i * 3 + 2]],
                [uv[i * 2], uv[i * 2 + 1]],
            )
        })
        .collect()
}

fn base64(s: &str) -> Vec<u8> {
    let mut out = Vec::new();
    let (mut acc, mut bits) = (0u32, 0);
    for c in s.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => continue,
        } as u32;
        acc = acc << 6 | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    out
}

/// Every drawn vertex of the model's default scene in model space, with
/// its uv as the base colour texture samples it (the material's transform
/// applied), and the triangle count.
pub fn drawn(model: &CookedModel) -> (Vec<Corner>, usize) {
    let world = model.node_world();
    let mut out = Vec::new();
    let mut tris = 0;
    for n in model.scene_nodes(model.default_scene.unwrap()) {
        let Some(m) = model.nodes[n].mesh else {
            continue;
        };
        for p in model.meshes[m].primitives.clone() {
            let prim = &model.primitives[p];
            let t = model.materials[prim.material].uv_transforms[0];
            tris += prim.triangle_count(0);
            for v in &prim.vertices {
                let w = world[n].transform_point3(glam::Vec3::from(v.position));
                out.push((w.to_array(), t.apply(v.uv)));
            }
        }
    }
    (out, tris)
}

/// Every cooked vertex lies on a source vertex (position and uv within
/// `tol` and `uv_tol`), every source vertex is drawn, and the box has its
/// twelve triangles.
fn assert_box(name: &str, model: &CookedModel, tol: f32, uv_tol: f32) {
    let (verts, tris) = drawn(model);
    assert_eq!(tris, 12, "{name}: triangles");
    let src = source_vertices();
    let near = |a: &Corner, b: &Corner| {
        (0..3).all(|k| (a.0[k] - b.0[k]).abs() <= tol)
            && (0..2).all(|k| (a.1[k] - b.1[k]).abs() <= uv_tol)
    };
    for v in &verts {
        assert!(
            src.iter().any(|s| near(v, s)),
            "{name}: the cooked vertex {v:?} is no source vertex"
        );
    }
    for s in &src {
        assert!(
            verts.iter().any(|v| near(v, s)),
            "{name}: the source vertex {s:?} is not drawn"
        );
    }
    for k in 0..3 {
        assert!(
            (model.min[k] - LO[k]).abs() <= tol && (model.max[k] - HI[k]).abs() <= tol,
            "{name}: bounds {:?}..{:?}",
            model.min,
            model.max
        );
    }
}

/// Draco (Google's encoder, through glTF Transform), meshopt with
/// quantisation (glTF Transform), gltfpack's quantised file and its
/// meshopt-compressed one (`-cc`, with its uvs dequantised by a
/// `KHR_texture_transform`): each cooks to the source box.
#[test]
fn compressed_and_quantised_gltf_cooks_to_the_source_geometry() {
    let files = [
        ("box.gltf", "models/plain.gltf"),
        ("draco.glb", "models/draco.glb"),
        ("meshopt.glb", "models/meshopt.glb"),
        ("quantized.glb", "models/quantized.glb"),
        ("gltfpack-cc.glb", "models/gltfpack.glb"),
    ];
    let (bundle, _) = cook("compressed", &files);
    // The plain box through the cook: the measure of the cook's own
    // quantisation (16 bits over the bounds) and the tolerance below it.
    let plain = bundle.load_model("models/plain.gltf").unwrap();
    assert_box("plain", &plain, 1e-4, 1e-4);
    // Draco quantises positions to 11 bits by default (3 m / 2^11), uvs
    // to 10; gltfpack positions to 14 bits and uvs to 12.
    for (id, tol, uv_tol) in [
        ("models/draco.glb", 2e-3, 2e-3),
        ("models/meshopt.glb", 1e-3, 1e-3),
        ("models/quantized.glb", 1e-3, 1e-3),
        ("models/gltfpack.glb", 1e-3, 1e-3),
    ] {
        let model = bundle.load_model(id).unwrap();
        assert_box(id, &model, tol, uv_tol);
    }
}

/// glTF's `KHR_texture_transform` uv' = T * R * S * uv, written out.
fn gltf_transform(uv: [f32; 2], offset: [f32; 2], scale: [f32; 2], rotation: f32) -> [f32; 2] {
    let (s, c) = rotation.sin_cos();
    let (x, y) = (uv[0] * scale[0], uv[1] * scale[1]);
    [c * x + s * y + offset[0], -s * x + c * y + offset[1]]
}

/// `extensions.gltf`: a texture transform the cook bakes (one material,
/// one transform for its every texture) and one it carries (the base
/// colour transformed, the normal map not), an unlit material, three
/// punctual lights, a node instanced three times and two material
/// variants.
#[test]
fn texture_transform_unlit_lights_instancing_and_variants_are_cooked() {
    let (bundle, notes) = cook("extensions", &[("extensions.gltf", "models/x.gltf")]);
    let model = bundle.load_model("models/x.gltf").unwrap();
    let mat = |name: &str| model.materials.iter().position(|m| m.name == name).unwrap();
    let mesh_prim = |name: &str| {
        let m = model.meshes.iter().find(|m| m.name == name).unwrap();
        &model.primitives[m.primitives.start]
    };

    // Baked: the material keeps no transform, its primitive's uvs moved.
    let baked = &model.materials[mat("baked")];
    assert!(
        baked.uv_transforms.iter().all(|t| t.is_identity()),
        "{baked:?}"
    );
    let src = source_vertices();
    for v in &mesh_prim("baked").vertices {
        let s = src
            .iter()
            .find(|s| (0..3).all(|k| (s.0[k] - v.position[k]).abs() < 1e-4))
            .unwrap();
        // Several source vertices share a corner: one of them has this uv.
        let ok = src.iter().filter(|t| t.0 == s.0).any(|t| {
            let want = gltf_transform(t.1, [0.25, 0.5], [0.5, 0.25], 0.5);
            (0..2).all(|k| (want[k] - v.uv[k]).abs() < 1e-4)
        });
        assert!(ok, "a baked uv {:?} at {:?}", v.uv, v.position);
    }

    // Carried: the base colour's transform on its slot, the normal map's none,
    // and the vertices' uvs as the source's.
    let carried = &model.materials[mat("carried")];
    let t = carried.uv_transforms[og_render::UvTransform::MAP];
    assert_eq!(
        (t.offset.to_array(), t.scale.to_array(), t.rotation),
        ([0.0, 0.0], [2.0, 3.0], 0.25)
    );
    assert!(carried.uv_transforms[og_render::UvTransform::NORMAL].is_identity());
    let want = gltf_transform([0.3, 0.7], [0.0, 0.0], [2.0, 3.0], 0.25);
    let got = t.apply([0.3, 0.7]);
    assert!(
        (0..2).all(|k| (want[k] - got[k]).abs() < 1e-6),
        "{got:?} {want:?}"
    );
    for v in &mesh_prim("variant").vertices {
        assert!(
            src.iter()
                .any(|s| (0..2).all(|k| (s.1[k] - v.uv[k]).abs() < 1e-4))
        );
    }
    // The carried transform reaches the standard material a scene draws.
    let textures: Vec<_> = (0..model.textures.len())
        .map(|_| og_render::TextureData {
            width: 1,
            height: 1,
            rgba: vec![255; 4],
            color_space: og_render::ColorSpace::Srgb,
            mips: og_render::Mips::None,
        })
        .collect();
    let mut scene = og_render::RenderScene::new();
    let parts = model.add_to(&mut scene, &textures);
    let drawn_carried = parts
        .parts
        .iter()
        .map(|p| scene.standard_material(p.1).unwrap())
        .find(|m| m.mapRepeat == glam::Vec2::new(2.0, 3.0))
        .expect("the carried transform on a drawn material");
    assert_eq!(drawn_carried.mapRotation, 0.25);

    // Unlit.
    assert!(model.materials[mat("unlit")].unlit);
    assert!(!carried.unlit);
    assert!(
        parts
            .parts
            .iter()
            .any(|p| scene.standard_material(p.1).unwrap().unlit == 1)
    );

    // Lights, by node.
    assert_eq!(model.lights.len(), 3, "{:?}", model.lights);
    use og_cook::format::light;
    let by = |n: &str| model.lights.iter().find(|l| l.name == n).unwrap();
    let bulb = by("bulb");
    assert_eq!(
        (bulb.kind, bulb.color, bulb.intensity, bulb.range),
        (light::POINT, [1.0, 0.5, 0.25], 20.0, 0.0)
    );
    assert_eq!(model.nodes[bulb.node].name, "bulb");
    let cone = by("cone");
    assert_eq!(
        (cone.kind, cone.range, cone.cone),
        (light::SPOT, 5.0, [0.2, 0.6])
    );
    assert_eq!(model.nodes[cone.node].name, "cone");
    assert_eq!(by("sun").kind, light::DIRECTIONAL);

    // Instancing: three instances of one mesh, not copies.
    let crates = model.nodes.iter().position(|n| n.name == "crates").unwrap();
    let inst = &model.instances[&crates];
    assert_eq!(inst.len(), 3);
    let xs: Vec<f32> = inst.iter().map(|m| m.w_axis.x).collect();
    assert_eq!(xs, [0.0, 5.0, 10.0]);
    assert_eq!(model.meshes.len(), 3, "no copied meshes");
    let crate_parts: Vec<_> = parts
        .nodes
        .iter()
        .zip(&parts.parts)
        .filter(|(n, _)| **n == crates)
        .map(|(_, p)| p)
        .collect();
    assert_eq!(crate_parts.len(), 3);
    assert!(
        crate_parts.iter().all(|p| p.0 == crate_parts[0].0),
        "one mesh"
    );
    let at: Vec<f32> = crate_parts.iter().map(|p| p.2.w_axis.x).collect();
    assert_eq!(at, [0.0, 5.0, 10.0]);
    assert_eq!(at.len(), 3);
    assert_eq!(
        crate_parts[0].2.w_axis.z, -4.0,
        "the node's transform under each instance"
    );
    // The bounds hold the instances.
    assert!((model.max.x - 12.0).abs() < 1e-4, "{:?}", model.max);

    // Variants: the table, and the variant primitive's material under each.
    assert_eq!(model.variants, ["base", "red"]);
    assert_eq!(model.variant("red"), Some(1));
    let vp = model
        .meshes
        .iter()
        .find(|m| m.name == "variant")
        .unwrap()
        .primitives
        .start;
    assert_eq!(model.primitive_material(vp, None), mat("carried"));
    assert_eq!(model.primitive_material(vp, Some(0)), mat("carried"));
    assert_eq!(model.primitive_material(vp, Some(1)), mat("red"));
    let bp = model
        .meshes
        .iter()
        .find(|m| m.name == "baked")
        .unwrap()
        .primitives
        .start;
    assert_eq!(model.primitive_material(bp, Some(1)), mat("baked"));
    // Per variant, each part's material handle.
    assert_eq!(parts.variants.len(), 2);
    let k = parts
        .nodes
        .iter()
        .position(|&n| model.nodes[n].name == "variant")
        .unwrap();
    assert_eq!(parts.variants[0][k], parts.parts[k].1);
    let red = scene.standard_material(parts.variants[1][k]).unwrap();
    assert_eq!(red.color, glam::Vec3::new(1.0, 0.0, 0.0));
    assert!(notes.contains("KHR_lights_punctual: 3 light(s)"), "{notes}");
}
