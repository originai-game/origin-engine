//! 2D rigs through the cook: the hand-written text, and a DragonBones
//! export converted.
//!
//! Ways this could fail: the texture's size is not filled in (every uv is
//! wrong); DragonBones' y-down coordinates and clockwise angles are taken
//! as they are (the rig draws upside down and turns the wrong way); a
//! weighted vertex is left in the setup pose's space instead of each
//! bone's (the mesh flies off when a bone moves); frame durations are read
//! as seconds; a deform's offset or its y signs are ignored; a stepped key
//! (`tweenEasing: null`) interpolates; events in `events` or `actions` are
//! lost.

use og_anim::{AttachmentKind, Rig2dAsset};
use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use std::path::PathBuf;

fn project(name: &str, files: &[(&str, Vec<u8>)]) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-cook-rig2d")
        .join(name);
    let _ = std::fs::remove_dir_all(&root);
    for (path, bytes) in files {
        let p = root.join("assets").join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, bytes).unwrap();
    }
    root
}

fn png(w: u32, h: u32) -> Vec<u8> {
    let img = image::RgbaImage::from_pixel(w, h, image::Rgba([200, 100, 50, 255]));
    let mut out = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    out
}

fn cooked(root: &std::path::Path) -> Bundle {
    let report = cook_project(root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    Bundle::open(&root.join("cooked")).unwrap()
}

const HAND: &str = r#"type rig2d
version 1
---
texture "parts.png"
unit 16
bone root {
}
slot body {
  bone root
  attachment body
}
region body {
  slot body
  rect 0 0 8 8
}
clip idle {
  duration 1
}
"#;

const SKE: &str = r#"{ "name": "hero", "version": "5.5", "frameRate": 10, "armature": [{
  "name": "hero",
  "bone": [
    { "name": "root" },
    { "name": "arm", "parent": "root", "length": 40, "transform": { "x": 10, "y": -20, "skX": 30, "skY": 30 } }
  ],
  "slot": [ { "name": "s-arm", "parent": "arm", "color": { "aM": 50 } } ],
  "skin": [{ "slot": [{ "name": "s-arm", "display": [
    { "name": "arm", "transform": { "x": 20 } },
    { "name": "cloth", "type": "mesh",
      "vertices": [0, 0, 40, 0, 40, 10], "uvs": [0, 0, 1, 0, 1, 1], "triangles": [0, 1, 2],
      "weights": [1, 0, 1, 2, 0, 0.5, 1, 0.5, 1, 1, 1], "slotPose": [1, 0, 0, 1, 0, 0] },
    { "name": "box", "type": "boundingBox" }
  ] }] }],
  "ik": [{ "name": "aim", "bone": "arm", "target": "root", "bendPositive": false }],
  "animation": [{ "name": "walk", "duration": 10, "playTimes": 0,
    "frame": [{ "duration": 3 }, { "duration": 7, "events": [{ "name": "step" }] }],
    "bone": [{ "name": "arm",
      "translateFrame": [{ "duration": 5, "x": 2, "y": 4 }, { "duration": 5, "tweenEasing": null, "x": 0, "y": 0 }],
      "rotateFrame": [{ "duration": 10, "rotate": 15 }] }],
    "slot": [{ "name": "s-arm", "displayFrame": [{ "duration": 5, "value": 0 }, { "duration": 5, "value": 1 }] }],
    "ffd": [{ "name": "cloth", "slot": "s-arm", "frame": [{ "duration": 10, "offset": 1, "vertices": [3, 4] }] }],
    "zOrder": { "frame": [] }
  }]
}] }"#;

const ATLAS: &str = r#"{ "imagePath": "hero_tex.png", "width": 32, "height": 16, "SubTexture": [
  { "name": "arm", "x": 0, "y": 0, "width": 16, "height": 8 },
  { "name": "cloth", "x": 16, "y": 0, "width": 16, "height": 16 }
] }"#;

const IMPORT: &str =
    "type rig2d\nversion 1\n---\ndragonbones \"hero_ske.json\"\natlas \"hero_tex.json\"\nunit 20\n";

#[test]
fn a_hand_written_rig_cooks_with_its_texture_size() {
    let root = project(
        "hand",
        &[
            ("rigs/parts.png", png(24, 12)),
            ("rigs/body.ogt", HAND.as_bytes().to_vec()),
        ],
    );
    let rig: Rig2dAsset = cooked(&root).load_rig2d("rigs/body.ogt").unwrap();
    assert_eq!(rig.texture, "rigs/parts.png");
    assert_eq!(rig.size, [24, 12]);
    assert_eq!(rig.unit, 16.0);
    assert_eq!(rig.clip("idle"), Some(0));
}

#[test]
fn a_dragonbones_export_becomes_a_rig_with_y_up() {
    let root = project(
        "dragonbones",
        &[
            ("rigs/hero_tex.png", png(32, 16)),
            ("rigs/hero_ske.json", SKE.as_bytes().to_vec()),
            ("rigs/hero_tex.json", ATLAS.as_bytes().to_vec()),
            ("rigs/hero.ogt", IMPORT.as_bytes().to_vec()),
        ],
    );
    let rig = cooked(&root).load_rig2d("rigs/hero.ogt").unwrap();
    assert_eq!(
        (rig.texture.as_str(), rig.size, rig.unit),
        ("rigs/hero_tex.png", [32, 16], 20.0)
    );
    // y and angles change sign.
    let arm = &rig.bones[1];
    assert_eq!(
        (arm.x, arm.y, arm.rotation, arm.length),
        (10.0, 20.0, -30.0, 40.0)
    );
    assert_eq!(rig.slots[0].color, [1.0, 1.0, 1.0, 0.5]);
    assert_eq!(rig.slots[0].attachment, Some(0), "displayIndex 0");
    assert_eq!(rig.attachments.len(), 2, "the bounding box is not imported");
    match &rig.attachments[0].kind {
        AttachmentKind::Region { rect, at, .. } => {
            assert_eq!(*rect, [0.0, 0.0, 16.0, 8.0]);
            assert_eq!(*at, [20.0, 0.0]);
        }
        k => panic!("{k:?}"),
    }
    // The weighted mesh: each influence in its bone's space, so the setup
    // pose puts the vertex back where the export had it (40, 10 down).
    let setup = rig.pose(None, 0.0, &[(0, [0.0, 0.0], 0.0)]);
    let AttachmentKind::Mesh { vertices, .. } = &rig.attachments[1].kind else {
        panic!("a mesh")
    };
    let v = &vertices[2];
    assert_eq!(v.uv, [32.0, 16.0]);
    assert_eq!(v.influences[0].bone, 1);
    let p = setup.world[1].apply(v.influences[0].x, v.influences[0].y);
    assert!(
        (p[0] - 40.0).abs() < 1e-3 && (p[1] + 10.0).abs() < 1e-3,
        "{p:?}"
    );
    assert!(
        (vertices[1].influences[0].x - 40.0).abs() < 1e-4,
        "{:?}",
        vertices[1]
    );
    assert!(rig.iks[0].counterclockwise);
    // The clip: frames of frameRate 10, keys relative, y and angles flipped.
    let walk = &rig.clips[0];
    assert_eq!((walk.duration, walk.looping), (1.0, true));
    let bone = &walk.bones[0];
    assert_eq!(bone.translate[0].value, [2.0, -4.0]);
    assert_eq!(
        (bone.translate[1].time, bone.translate[1].step),
        (0.5, true)
    );
    assert!(!bone.translate[0].step);
    assert_eq!(bone.rotate[0].value, -15.0);
    assert_eq!(walk.slots[0].attachment, [(0.0, Some(0)), (0.5, Some(1))]);
    assert_eq!(
        walk.deforms[0].keys[0].value,
        [0.0, -3.0, 4.0, 0.0, 0.0, 0.0, 0.0, 0.0]
    );
    assert_eq!(walk.events, [(0.3, "step".to_string())]);
}
