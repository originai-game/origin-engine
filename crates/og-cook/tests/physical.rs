//! glTF's physical material extensions (`KHR_materials_clearcoat`,
//! `_sheen`, `_transmission`, `_volume`, `_ior`, `_iridescence`,
//! `_anisotropy`) through the cook and the runtime glTF loader: their
//! factors and textures reach the cooked model and the material it adds
//! to a scene, as three.js's GLTFLoader maps them.
//!
//! Ways this could fail: an extension's factor or texture is dropped on
//! the way through the material row; a texture is cooked with the wrong
//! kind (the sheen colour must be sRGB, the coat's normal map a normal
//! map); `attenuationDistance` absent is not "no absorption"; a material
//! without extensions stops being added in its glTF shape.

use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use og_render::{PhysicalExtensions, RenderScene};
use std::path::PathBuf;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A one-quad glTF with two materials: `glass` with every physical
/// extension (all textures the same image), `plain` with none.
fn physical_glb(png: &[u8]) -> Vec<u8> {
    let mut bin: Vec<u8> = Vec::new();
    let mut views = Vec::new();
    let mut push = |bytes: &[u8], bin: &mut Vec<u8>| {
        while !bin.len().is_multiple_of(4) {
            bin.push(0);
        }
        views.push(
            serde_json::json!({ "buffer": 0, "byteOffset": bin.len(), "byteLength": bytes.len() }),
        );
        bin.extend_from_slice(bytes);
    };
    let f32s = |v: &[f32]| v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>();
    push(
        &f32s(&[0., 0., 0., 1., 0., 0., 0., 0., 1., 1., 0., 1.]),
        &mut bin,
    );
    push(
        &f32s(&[0., 1., 0., 0., 1., 0., 0., 1., 0., 0., 1., 0.]),
        &mut bin,
    );
    push(&f32s(&[0., 0., 1., 0., 0., 1., 1., 1.]), &mut bin);
    push(
        &[0u32, 2, 1, 1, 2, 3]
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<u8>>(),
        &mut bin,
    );
    push(png, &mut bin);
    while !bin.len().is_multiple_of(4) {
        bin.push(0);
    }
    let t = serde_json::json!({ "index": 0 });
    let json = serde_json::json!({
        "asset": { "version": "2.0" },
        "extensionsUsed": [
            "KHR_materials_clearcoat", "KHR_materials_sheen", "KHR_materials_transmission",
            "KHR_materials_volume", "KHR_materials_ior", "KHR_materials_iridescence",
            "KHR_materials_anisotropy"
        ],
        "scene": 0,
        "scenes": [{ "nodes": [0, 1] }],
        "nodes": [{ "mesh": 0 }, { "mesh": 1 }],
        "meshes": [
            { "primitives": [{ "attributes": { "POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2 }, "indices": 3, "material": 0 }] },
            { "primitives": [{ "attributes": { "POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2 }, "indices": 3, "material": 1 }] }
        ],
        "materials": [
            {
                "name": "glass",
                "extensions": {
                    "KHR_materials_clearcoat": {
                        "clearcoatFactor": 0.8, "clearcoatTexture": t,
                        "clearcoatRoughnessFactor": 0.2, "clearcoatRoughnessTexture": t,
                        "clearcoatNormalTexture": { "index": 0, "scale": 0.5 }
                    },
                    "KHR_materials_sheen": {
                        "sheenColorFactor": [0.9, 0.4, 0.6], "sheenColorTexture": t,
                        "sheenRoughnessFactor": 0.3, "sheenRoughnessTexture": t
                    },
                    "KHR_materials_transmission": { "transmissionFactor": 0.9, "transmissionTexture": t },
                    "KHR_materials_volume": {
                        "thicknessFactor": 0.25, "thicknessTexture": t,
                        "attenuationColor": [0.9, 0.5, 0.1]
                    },
                    "KHR_materials_ior": { "ior": 1.33 },
                    "KHR_materials_iridescence": {
                        "iridescenceFactor": 0.7, "iridescenceTexture": t, "iridescenceIor": 1.4,
                        "iridescenceThicknessMinimum": 120.0, "iridescenceThicknessMaximum": 500.0,
                        "iridescenceThicknessTexture": t
                    },
                    "KHR_materials_anisotropy": {
                        "anisotropyStrength": 0.6, "anisotropyRotation": 1.25, "anisotropyTexture": t
                    }
                }
            },
            { "name": "plain", "pbrMetallicRoughness": { "roughnessFactor": 0.4 } }
        ],
        "textures": [{ "source": 0 }],
        "images": [{ "bufferView": 4, "mimeType": "image/png" }],
        "accessors": [
            { "bufferView": 0, "componentType": 5126, "count": 4, "type": "VEC3",
              "min": [0.0, 0.0, 0.0], "max": [1.0, 0.0, 1.0] },
            { "bufferView": 1, "componentType": 5126, "count": 4, "type": "VEC3" },
            { "bufferView": 2, "componentType": 5126, "count": 4, "type": "VEC2" },
            { "bufferView": 3, "componentType": 5125, "count": 6, "type": "SCALAR" }
        ],
        "bufferViews": views,
        "buffers": [{ "byteLength": bin.len() }]
    });
    let mut text = serde_json::to_vec(&json).unwrap();
    while !text.len().is_multiple_of(4) {
        text.push(b' ');
    }
    let mut glb = Vec::new();
    glb.extend_from_slice(b"glTF");
    glb.extend_from_slice(&2u32.to_le_bytes());
    glb.extend_from_slice(&((12 + 8 + text.len() + 8 + bin.len()) as u32).to_le_bytes());
    glb.extend_from_slice(&(text.len() as u32).to_le_bytes());
    glb.extend_from_slice(b"JSON");
    glb.extend_from_slice(&text);
    glb.extend_from_slice(&(bin.len() as u32).to_le_bytes());
    glb.extend_from_slice(b"BIN\0");
    glb.extend_from_slice(&bin);
    glb
}

/// What the glass material's extensions say, in glTF's terms.
fn expect_glass<T: Copy + std::fmt::Debug>(p: &PhysicalExtensions<T>) {
    type E = PhysicalExtensions<()>;
    assert_eq!(p.present, 0b111_1111, "all seven extensions");
    assert_eq!(p.present & E::VOLUME, E::VOLUME);
    assert_eq!(
        (p.clearcoat, p.clearcoat_roughness, p.clearcoat_normal_scale),
        (0.8, 0.2, 0.5)
    );
    assert_eq!((p.sheen_color, p.sheen_roughness), ([0.9, 0.4, 0.6], 0.3));
    assert_eq!((p.transmission, p.thickness), (0.9, 0.25));
    assert_eq!(p.attenuation_distance, 0.0, "absent: no absorption");
    assert_eq!(p.attenuation_color, [0.9, 0.5, 0.1]);
    assert_eq!(p.ior, 1.33);
    assert_eq!(
        (p.iridescence, p.iridescence_ior, p.iridescence_thickness),
        (0.7, 1.4, [120.0, 500.0])
    );
    assert_eq!((p.anisotropy, p.anisotropy_rotation), (0.6, 1.25));
    for t in [
        p.clearcoat_texture,
        p.clearcoat_roughness_texture,
        p.clearcoat_normal_texture,
        p.sheen_color_texture,
        p.sheen_roughness_texture,
        p.transmission_texture,
        p.thickness_texture,
        p.iridescence_texture,
        p.iridescence_thickness_texture,
        p.anisotropy_texture,
    ] {
        assert!(t.is_some(), "every texture kept: {p:?}");
    }
}

/// The standard material a scene draws for the glass.
fn expect_standard(m: &og_render::StandardMaterial) {
    assert!(m.is_physical() && m.is_transmissive(), "{m:?}");
    assert_eq!((m.clearcoat, m.clearcoatRoughness), (0.8, 0.2));
    assert_eq!(m.clearcoatNormalScale, glam::Vec2::splat(0.5));
    assert_eq!(
        m.sheen, 1.0,
        "the sheen extension turns sheen on, as three.js"
    );
    assert_eq!(m.sheenColor, glam::Vec3::new(0.9, 0.4, 0.6));
    assert_eq!((m.transmission, m.thickness, m.ior), (0.9, 0.25, 1.33));
    assert_eq!(m.iridescenceThicknessRange, glam::Vec2::new(120.0, 500.0));
    assert_eq!((m.anisotropy, m.anisotropyRotation), (0.6, 1.25));
    for slot in [
        m.clearcoatMap,
        m.clearcoatRoughnessMap,
        m.clearcoatNormalMap,
        m.sheenColorMap,
        m.sheenRoughnessMap,
        m.transmissionMap,
        m.thicknessMap,
        m.iridescenceMap,
        m.iridescenceThicknessMap,
        m.anisotropyMap,
    ] {
        assert_ne!(slot, 0);
    }
    // The sheen colour is its own (sRGB) texture; the scalar maps share
    // the linear one.
    assert_ne!(m.sheenColorMap, m.sheenRoughnessMap);
    assert_eq!(m.clearcoatMap, m.anisotropyMap);
}

/// The glass and the plain material among a model's parts (the plain one
/// is the rougher).
fn split(
    scene: &RenderScene,
    parts: impl Iterator<Item = og_render::MaterialHandle>,
) -> (og_render::MaterialHandle, og_render::MaterialHandle) {
    let mut handles: Vec<_> = parts.collect();
    handles.sort_by(|a, b| {
        let r = |h| scene.standard_material(h).unwrap().roughness;
        r(*a).total_cmp(&r(*b))
    });
    handles.dedup();
    assert_eq!(handles.len(), 2, "two materials");
    let plain = handles
        .iter()
        .copied()
        .find(|&h| scene.standard_material(h).unwrap().roughness == 0.4)
        .expect("the plain material");
    let glass = handles.iter().copied().find(|&h| h != plain).unwrap();
    (glass, plain)
}

#[test]
fn physical_extensions_are_cooked_and_loaded() {
    let png = std::fs::read(repo().join("templates/fx-showcase/assets/textures/toon.png")).unwrap();
    let glb = physical_glb(&png);

    // The runtime glTF loader (og-render's bridge).
    let model = og_render::gltf::load_glb(&glb).expect("loads");
    expect_glass(&model.physical[0]);
    assert_eq!(model.physical[1].present, 0);
    let mut scene = RenderScene::new();
    let parts = model.add_to(&mut scene);
    let (glass, plain) = split(&scene, parts.parts.iter().map(|p| p.1));
    expect_standard(scene.standard_material(glass).unwrap());
    assert!(
        scene.material(glass).is_none(),
        "added as a standard material"
    );
    assert!(scene.material(plain).is_some(), "added in its glTF shape");

    // The cook.
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-cook-physical");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets/models")).unwrap();
    std::fs::write(root.join("assets/models/glass.glb"), &glb).unwrap();
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let (cooked, textures) = bundle
        .load_model_with_textures("models/glass.glb")
        .expect("decodes");
    expect_glass(&cooked.materials[0].physical);
    assert_eq!(cooked.materials[1].physical.present, 0);
    let kinds: Vec<u32> = cooked.textures.iter().map(|t| t.kind).collect();
    use og_cook::format::kind;
    for k in [kind::COLOR, kind::NORMAL, kind::LINEAR] {
        assert!(kinds.contains(&k), "a {k} texture among {kinds:?}");
    }
    let sheen = cooked.materials[0].physical.sheen_color_texture.unwrap();
    assert_eq!(
        cooked.textures[sheen].kind,
        kind::COLOR,
        "the sheen colour is sRGB"
    );
    let normal = cooked.materials[0]
        .physical
        .clearcoat_normal_texture
        .unwrap();
    assert_eq!(cooked.textures[normal].kind, kind::NORMAL);
    let mut scene = RenderScene::new();
    let parts = cooked.add_to(&mut scene, &textures);
    let (glass, plain) = split(&scene, parts.parts.iter().map(|p| p.1));
    expect_standard(scene.standard_material(glass).unwrap());
    assert!(scene.material(plain).is_some());
}
