//! Streaming groups (phase 3) through the cook: a scene's `stream` blocks
//! become packs under `cooked/stream/`, manifest entries and `#proxy` models.
//!
//! Ways this could fail: a pack misses a grouped file, holds one of another
//! group's or the first group's, or its bytes differ from the cooked file;
//! the manifest loses a group's trigger, priority, memory or sounds on its
//! way through text; a grouped output does not name its group (the web
//! build would precache it); the proxy keeps textures or more than its
//! coarsest LOD, or lands in the group it stands in for; the memory a group
//! takes is 0 or less than its texture's pixels; a bad declaration (an
//! asset in two groups, a skinned model, an asset that is not cooked, two
//! triggers, a name taken twice) is cooked instead of refused; a group
//! dropped from the scene leaves its pack behind.

use og_cook::runtime::{Bundle, Manifest, StreamTrigger, unpack};
use og_cook::{CookOptions, cook_project};
use std::path::{Path, PathBuf};

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

const SCENE: &str = r#"type scene
version 1
---
name "Path"
stream "village" {
  asset "models/house-a.glb"
  asset "textures/stone.png"
  distance 0 0 -20 30
  priority 2
}
stream "cave" {
  asset "models/house-b.glb"
  asset "audio/blip.wav"
  volume "cave-mouth"
}
stream "boss" {
  asset "models/house-c.glb"
}
entity "cave-mouth" {
  Transform {
    position 0 0 -120
    quaternion 0 0 0 1
    scale 20 10 20
  }
}
"#;

/// A textured glTF: a wavy 48 x 48 quad sheet (enough triangles for an LOD
/// chain) whose material samples `png` as its base colour.
fn textured_glb(png: &[u8]) -> Vec<u8> {
    const N: usize = 48;
    let mut bin: Vec<u8> = Vec::new();
    let (mut pos, mut nrm, mut uv, mut idx) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for z in 0..=N {
        for x in 0..=N {
            let (u, v) = (x as f32 / N as f32, z as f32 / N as f32);
            let y = 0.3 * (u * 9.0).sin() * (v * 7.0).cos();
            pos.extend([u * 10.0 - 5.0, y, v * 10.0 - 5.0]);
            nrm.extend([0.0f32, 1.0, 0.0]);
            uv.extend([u, v]);
        }
    }
    for z in 0..N {
        for x in 0..N {
            let i = (z * (N + 1) + x) as u32;
            let r = (N + 1) as u32;
            idx.extend([i, i + r, i + 1, i + 1, i + r, i + r + 1]);
        }
    }
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
    push(&f32s(&pos), &mut bin);
    push(&f32s(&nrm), &mut bin);
    push(&f32s(&uv), &mut bin);
    push(
        &idx.iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<u8>>(),
        &mut bin,
    );
    push(png, &mut bin);
    while !bin.len().is_multiple_of(4) {
        bin.push(0);
    }
    let count = (N + 1) * (N + 1);
    let json = serde_json::json!({
        "asset": { "version": "2.0" },
        "scene": 0,
        "scenes": [{ "nodes": [0] }],
        "nodes": [{ "mesh": 0 }],
        "meshes": [{ "primitives": [{
            "attributes": { "POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2 },
            "indices": 3, "material": 0
        }]}],
        "materials": [{ "pbrMetallicRoughness": { "baseColorTexture": { "index": 0 } } }],
        "textures": [{ "source": 0 }],
        "images": [{ "bufferView": 4, "mimeType": "image/png" }],
        "accessors": [
            { "bufferView": 0, "componentType": 5126, "count": count, "type": "VEC3",
              "min": [-5.0, -0.3, -5.0], "max": [5.0, 0.3, 5.0] },
            { "bufferView": 1, "componentType": 5126, "count": count, "type": "VEC3" },
            { "bufferView": 2, "componentType": 5126, "count": count, "type": "VEC2" },
            { "bufferView": 3, "componentType": 5125, "count": idx.len(), "type": "SCALAR" }
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

fn stage(root: &Path, scene: &str) {
    let _ = std::fs::remove_dir_all(root);
    for d in ["models", "textures", "audio", "scenes"] {
        std::fs::create_dir_all(root.join("assets").join(d)).unwrap();
    }
    let png = std::fs::read(repo().join("templates/fx-showcase/assets/textures/toon.png")).unwrap();
    let sheet = textured_glb(&png);
    for n in ["house-a", "house-b", "house-c", "hut"] {
        std::fs::write(root.join(format!("assets/models/{n}.glb")), &sheet).unwrap();
    }
    std::fs::copy(
        repo().join("templates/game/assets/models/walker.glb"),
        root.join("assets/models/walker.glb"),
    )
    .unwrap();
    std::fs::copy(
        repo().join("templates/fx-showcase/assets/textures/toon.png"),
        root.join("assets/textures/stone.png"),
    )
    .unwrap();
    std::fs::copy(
        repo().join("templates/game/assets/audio/blip.wav"),
        root.join("assets/audio/blip.wav"),
    )
    .unwrap();
    std::fs::write(root.join("assets/scenes/path.ogt"), scene).unwrap();
}

#[test]
fn groups_are_packed_listed_and_stood_in_for() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-cook-stream");
    stage(&root, SCENE);
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let cooked = root.join("cooked");
    let bundle = Bundle::open(&cooked).unwrap();
    let m = bundle.manifest();

    // Three groups with their triggers and priorities, by name.
    let names: Vec<&str> = m.streams.iter().map(|g| g.name.as_str()).collect();
    assert_eq!(names, ["boss", "cave", "village"]);
    let group = |n: &str| m.streams.iter().find(|g| g.name == n).unwrap();
    assert_eq!(group("boss").trigger, StreamTrigger::Manual);
    assert_eq!(
        group("cave").trigger,
        StreamTrigger::Volume("cave-mouth".into())
    );
    assert_eq!(
        group("village").trigger,
        StreamTrigger::Distance {
            anchor: [0.0, 0.0, -20.0],
            radius: 30.0,
            unload: 37.5
        }
    );
    assert_eq!(group("village").priority, 2);
    assert_eq!(group("cave").priority, 0);
    assert_eq!(group("village").scene, "scenes/path.ogt");

    // Every grouped output names its group; the rest (and every proxy) none.
    for o in m.outputs() {
        let asset = o.id.split('#').next().unwrap();
        let expect = match asset {
            "models/house-a.glb" | "textures/stone.png" => Some("village"),
            "models/house-b.glb" | "audio/blip.wav" => Some("cave"),
            "models/house-c.glb" => Some("boss"),
            _ => None,
        };
        let expect = if o.id.ends_with("#proxy") {
            None
        } else {
            expect
        };
        assert_eq!(o.group.as_deref(), expect, "{}", o.id);
    }

    // Each pack holds exactly its group's cooked files, byte for byte.
    for g in &m.streams {
        let pack = std::fs::read(cooked.join(&g.file)).unwrap();
        assert_eq!(pack.len() as u64, g.bytes, "{}", g.name);
        let files = unpack(&pack).unwrap();
        let mut want: Vec<String> = m
            .outputs()
            .filter(|o| o.group.as_deref() == Some(g.name.as_str()))
            .map(|o| o.file.clone())
            .collect();
        want.sort();
        let got: Vec<String> = files.iter().map(|f| f.0.clone()).collect();
        assert_eq!(got, want, "{}", g.name);
        for (file, bytes) in &files {
            assert_eq!(&std::fs::read(cooked.join(file)).unwrap(), bytes, "{file}");
        }
        assert!(g.memory > 0, "{} takes memory", g.name);
    }
    let stone = bundle.load_texture("textures/stone.png").unwrap();
    let pixels = u64::from(stone.meta.width) * u64::from(stone.meta.height) * 4;
    assert!(
        group("village").memory > pixels,
        "the village holds the stone's {pixels} bytes and a model: {}",
        group("village").memory
    );

    // The cave's sound plays as silence of its own length until it loads.
    let blip = bundle.load_sound("audio/blip.wav").unwrap();
    let s = &group("cave").sounds;
    assert_eq!(s.len(), 1);
    assert_eq!(s[0].id, "audio/blip.wav");
    assert_eq!(s[0].frames, u64::from(blip.meta.frames));
    assert_eq!(s[0].sample_rate, blip.meta.sample_rate);
    assert_eq!(s[0].channels, blip.meta.channels);

    // The manifest keeps all of it through its text.
    assert_eq!(&Manifest::parse(&m.to_text()).unwrap(), m);

    // A proxy: the coarsest LOD of every primitive, no textures.
    for n in ["house-a", "house-b", "house-c"] {
        let full = bundle.load_model(&format!("models/{n}.glb")).unwrap();
        let proxy = bundle.load_model(&format!("models/{n}.glb#proxy")).unwrap();
        assert_eq!(proxy.primitives.len(), full.primitives.len());
        assert_eq!(proxy.materials.len(), full.materials.len());
        assert!(proxy.textures.is_empty(), "{n}'s proxy has no textures");
        assert!(!full.textures.is_empty(), "{n} has textures");
        for (p, f) in proxy.primitives.iter().zip(&full.primitives) {
            assert_eq!(p.lods.len(), 1);
            let coarsest = f.lods.len() - 1;
            assert!(f.lods.len() > 1, "the sheet has an LOD chain");
            assert_eq!(p.triangle_count(0), f.triangle_count(coarsest));
            assert!(p.vertices.len() < f.vertices.len());
        }
        for mat in &proxy.materials {
            assert!(
                mat.base_color_texture.is_none()
                    && mat.metallic_roughness_texture.is_none()
                    && mat.normal_texture.is_none()
                    && mat.emissive_texture.is_none(),
                "{n}'s proxy materials sample nothing"
            );
        }
    }
    // An ungrouped model has no proxy.
    assert!(bundle.output("models/hut.glb#proxy").is_err());

    // Dropping a group removes its pack and its proxy.
    std::fs::write(
        root.join("assets/scenes/path.ogt"),
        SCENE.replace("stream \"boss\" {\n  asset \"models/house-c.glb\"\n}\n", ""),
    )
    .unwrap();
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&cooked).unwrap();
    assert_eq!(bundle.streams().len(), 2);
    assert!(!cooked.join("stream/boss.ogb").exists());
    assert!(bundle.output("models/house-c.glb#proxy").is_err());
    assert_eq!(bundle.output("models/house-c.glb").unwrap().group, None);
}

#[test]
fn bad_groups_are_refused_with_the_reason() {
    let cases: [(&str, String, &str); 6] = [
        (
            "two-groups",
            SCENE.replace(
                "asset \"models/house-c.glb\"",
                "asset \"models/house-a.glb\"",
            ),
            "is in stream \"village\" already",
        ),
        (
            "skinned",
            SCENE.replace("models/house-c.glb", "models/walker.glb"),
            "a skinned or morphed model",
        ),
        (
            "not-cooked",
            SCENE.replace("models/house-c.glb", "models/missing.glb"),
            "which is not an asset under assets/",
        ),
        (
            "two-triggers",
            SCENE.replace(
                "volume \"cave-mouth\"",
                "volume \"cave-mouth\"\n  distance 0 0 0 5",
            ),
            "has both distance and volume",
        ),
        (
            "scene",
            SCENE.replace("models/house-c.glb", "scenes/path.ogt"),
            "a group holds glTF models",
        ),
        (
            "unload",
            SCENE.replace("priority 2", "priority 2\n  unload 10"),
            "unload takes one distance at least the radius",
        ),
    ];
    for (name, scene, reason) in cases {
        let root =
            PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("og-cook-stream-{name}"));
        stage(&root, &scene);
        let report = cook_project(&root, &CookOptions::default()).unwrap();
        let errors = report.errors().join("\n");
        assert!(errors.contains(reason), "{name}: {errors}");
        assert!(
            errors.contains("scenes/path.ogt"),
            "{name} names the scene: {errors}"
        );
        let packed = Bundle::open(&root.join("cooked")).map_or(0, |b| b.streams().len());
        assert_eq!(
            packed, 0,
            "{name}: a scene with a refused group packs none of its groups"
        );
    }
    // A name taken by two scenes.
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-cook-stream-taken");
    stage(&root, SCENE);
    std::fs::write(
        root.join("assets/scenes/other.ogt"),
        "type scene\nversion 1\n---\nname \"Other\"\nstream \"boss\" {\n  asset \"models/hut.glb\"\n}\n",
    )
    .unwrap();
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    let errors = report.errors().join("\n");
    assert!(errors.contains("a group's name is taken once"), "{errors}");
}
