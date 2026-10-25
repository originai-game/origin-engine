//! 2D rigs through the cook: the hand-written text, and a DragonBones
//! export converted.
//!
//! Ways this could fail: the texture's size is not filled in (every uv is
//! wrong); DragonBones' y-down coordinates and clockwise angles are taken
//! as they are (the rig draws upside down and turns the wrong way); a
//! weighted vertex is left in the setup pose's space instead of each
//! bone's (the mesh flies off when a bone moves); frame durations are read
//! as seconds; a deform's offset or its y signs are ignored; a stepped key
//! (`tweenEasing: null`, or none at all) interpolates; events in `events`
//! or `actions` are lost; skew is folded into the rotation; a colour's
//! offsets, a zOrder or IK timeline, an armature or bounding-box display,
//! or a Bézier `curve` is dropped or played as a line; a child armature
//! plays the wrong clip or at its parent's frame rate.

use og_anim::{AttachmentKind, Ease, Rig2dAsset, Turn};
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
    { "name": "arm", "parent": "root", "length": 40, "transform": { "x": 10, "y": -20, "skX": 30, "skY": 30 } },
    { "name": "lean", "parent": "root", "transform": { "skX": 40, "skY": 10, "scX": 2 } },
    { "name": "tail1", "parent": "root", "length": 10 },
    { "name": "tail2", "parent": "tail1", "length": 10, "transform": { "x": 10 } },
    { "name": "tail3", "parent": "tail2", "length": 10, "transform": { "x": 10 } }
  ],
  "slot": [
    { "name": "s-arm", "parent": "arm", "color": { "aM": 50, "rO": 51, "bO": -255 } },
    { "name": "s-hat", "parent": "lean", "actions": [{ "gotoAndPlay": "spin" }] }
  ],
  "skin": [{ "slot": [
    { "name": "s-arm", "display": [
      { "name": "arm", "transform": { "x": 20 } },
      { "name": "cloth", "type": "mesh",
        "vertices": [0, 0, 40, 0, 40, 10], "uvs": [0, 0, 1, 0, 1, 1], "triangles": [0, 1, 2],
        "weights": [1, 0, 1, 2, 0, 0.5, 1, 0.5, 1, 1, 1], "slotPose": [1, 0, 0, 1, 0, 0] },
      { "name": "box", "type": "boundingBox", "subType": "polygon", "vertices": [0, 0, 10, 0, 10, -5] },
      { "name": "frame", "type": "boundingBox", "subType": "rectangle", "width": 4, "height": 2, "transform": { "x": 1, "skX": 90, "skY": 90 } },
      { "name": "trail", "type": "path" }
    ] },
    { "name": "s-hat", "display": [
      { "name": "hat", "type": "armature", "transform": { "x": 5, "y": -6 } }
    ] }
  ] }],
  "ik": [
    { "name": "aim", "bone": "arm", "target": "root", "bendPositive": false },
    { "name": "wag", "bone": "tail3", "chain": 2, "target": "lean" }
  ],
  "animation": [{ "name": "walk", "duration": 10, "playTimes": 0,
    "frame": [{ "duration": 3 }, { "duration": 7, "events": [{ "name": "step" }] }],
    "bone": [
      { "name": "arm",
        "translateFrame": [{ "duration": 5, "tweenEasing": 0, "x": 2, "y": 4 }, { "duration": 5, "tweenEasing": null, "x": 0, "y": 0 }],
        "rotateFrame": [{ "duration": 5, "tweenEasing": 0, "rotate": 15, "skew": 5, "curve": [0.42, 0, 1, 1] }, { "duration": 5, "rotate": 25, "skew": 0 }],
        "scaleFrame": [{ "duration": 5, "x": 2 }, { "duration": 5, "x": 1 }] }
    ],
    "slot": [{ "name": "s-arm",
      "displayFrame": [{ "duration": 5, "value": 0 }, { "duration": 5, "value": 1 }],
      "colorFrame": [{ "duration": 5, "tweenEasing": -1, "color": { "rM": 50, "gO": 102 } }, { "duration": 5, "color": {} }] }],
    "ffd": [{ "name": "cloth", "slot": "s-arm", "frame": [{ "duration": 10, "offset": 1, "vertices": [3, 4] }] }],
    "zOrder": { "frame": [{ "duration": 5 }, { "duration": 5, "zOrder": [0, 1] }] },
    "ik": [{ "name": "aim", "frame": [
      { "duration": 5, "tweenEasing": 0, "weight": 1, "bendPositive": false },
      { "duration": 5, "weight": 0.25, "bendPositive": true } ] }]
  }]
}, {
  "name": "hat", "frameRate": 20,
  "bone": [{ "name": "crown" }],
  "slot": [{ "name": "s-crown", "parent": "crown" }],
  "skin": [{ "slot": [{ "name": "s-crown", "display": [{ "name": "arm" }] }] }],
  "animation": [
    { "name": "spin", "duration": 20, "playTimes": 0, "bone": [{ "name": "crown", "rotateFrame": [{ "duration": 20, "tweenEasing": 0, "rotate": 0 }, { "duration": 0, "rotate": 90 }] }] },
    { "name": "walk", "duration": 10 }
  ]
}, { "name": "spare", "bone": [{ "name": "b" }] }] }"#;

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
        (arm.x, arm.y, arm.rotation, arm.skew, arm.length),
        (10.0, 20.0, -30.0, 0.0, 40.0)
    );
    assert_eq!(rig.slots[0].color, [1.0, 1.0, 1.0, 0.5]);
    assert_eq!(rig.slots[0].attachment, Some(0), "displayIndex 0");
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
        (bone.translate[1].time, &bone.translate[1].ease),
        (0.5, &Ease::Step)
    );
    assert_eq!(bone.translate[0].ease, Ease::Linear);
    assert_eq!(bone.rotate[0].value.degrees, -15.0);
    assert_eq!(walk.slots[0].attachment, [(0.0, Some(0)), (0.5, Some(1))]);
    assert_eq!(
        walk.deforms[0].keys[0].value,
        [0.0, -3.0, 4.0, 0.0, 0.0, 0.0, 0.0, 0.0]
    );
    assert_eq!(walk.events, [(0.3, "step".to_string())]);
}

/// What the import used to drop (the notes said "skew dropped", "timelines
/// not imported: walk.zOrder, walk.ik", "displays not imported", Bézier
/// curves played linearly): every one of them is in the rig now.
#[test]
fn a_dragonbones_export_keeps_skew_offsets_order_ik_keys_children_boxes_and_curves() {
    let root = project(
        "dragonbones-full",
        &[
            ("rigs/hero_tex.png", png(32, 16)),
            ("rigs/hero_ske.json", SKE.as_bytes().to_vec()),
            ("rigs/hero_tex.json", ATLAS.as_bytes().to_vec()),
            ("rigs/hero.ogt", IMPORT.as_bytes().to_vec()),
        ],
    );
    let rig = cooked(&root).load_rig2d("rigs/hero.ogt").unwrap();
    // Skew: skY 10 is the rotation (-10 here), skX 40 turns the y axis 30
    // further clockwise with y down: -30 here.
    let lean = &rig.bones[2];
    assert_eq!(
        (lean.rotation, lean.skew, lean.scale),
        (-10.0, -30.0, [2.0, 1.0])
    );
    let m = rig.pose(None, 0.0, &[(0, [0.0, 0.0], 0.0)]).world[2];
    // The y axis turned -40 here: (-sin -40, cos -40) = (sin 40, cos 40).
    let (s40, c40) = (40f32.to_radians().sin(), 40f32.to_radians().cos());
    assert!(
        (m.c - s40).abs() < 1e-5 && (m.d - c40).abs() < 1e-5,
        "{m:?}"
    );
    // Colour offsets: rO 51 and bO -255 of 255.
    assert_eq!(rig.slots[0].offset, [0.2, 0.0, -1.0, 0.0]);
    // Boxes: the polygon (y flipped), the rectangle turned 90 (clockwise on
    // screen, -90 here) about its display's origin at x 1.
    let shape = |n: &str| match &rig.attachments.iter().find(|a| a.name == n).unwrap().kind {
        AttachmentKind::Box { polygon } => polygon.clone(),
        k => panic!("{n}: {k:?}"),
    };
    assert_eq!(shape("box"), [[0.0, 0.0], [10.0, 0.0], [10.0, 5.0]]);
    let frame = shape("frame");
    for (got, want) in frame
        .iter()
        .zip([[0.0, 2.0], [0.0, -2.0], [2.0, -2.0], [2.0, 2.0]])
    {
        assert!(
            (got[0] - want[0]).abs() < 1e-5 && (got[1] - want[1]).abs() < 1e-5,
            "{frame:?}"
        );
    }
    // The armature display: the hat armature imported as a child, placed
    // at (5, 6), playing the slot's action `spin`; the unused armature is not.
    assert_eq!(rig.armatures.len(), 1);
    let hat = &rig.armatures[0];
    assert_eq!((hat.name.as_str(), hat.clips.len()), ("hat", 2));
    assert_eq!(
        hat.clips[0].duration, 1.0,
        "20 frames at the hat's own 20 per second"
    );
    let lamp = rig.attachments.iter().find(|a| a.name == "hat").unwrap();
    match &lamp.kind {
        AttachmentKind::Child {
            armature, clip, at, ..
        } => {
            assert_eq!(
                (*armature, clip.as_deref(), *at),
                (0, Some("spin"), [5.0, 6.0])
            );
        }
        k => panic!("{k:?}"),
    }
    assert_eq!(
        rig.slots[1].attachment,
        Some(
            rig.attachments
                .iter()
                .position(|a| a.name == "hat")
                .unwrap()
        )
    );
    let walk = &rig.clips[0];
    // The parent plays walk: the hat has a walk of its own and plays it
    // (DragonBones' inherited animation), held at its start.
    let p = rig.pose(Some(walk), 0.5, &[(0, [0.0, 0.0], 0.0)]);
    let crown = &p.children[0].1.world[0];
    assert!((crown.a - 1.0).abs() < 1e-6, "{crown:?}");
    // Without a clip of the parent's name it plays spin: 0.5 s of 1 s, 45 degrees.
    let p = rig.pose(None, 0.5, &[(0, [0.0, 0.0], 0.0)]);
    let crown = &p.children[0].1.world[0];
    assert!(
        (crown.b + 45f32.to_radians().sin()).abs() < 1e-5,
        "{crown:?}"
    );
    // zOrder: the setup order until 0.5, then slot 0 (the arm) moved one
    // place up: the hat first, the arm drawn over it.
    assert_eq!(walk.order, [(0.0, vec![]), (0.5, vec![1, 0])]);
    // IK keys: weight 1 counter-clockwise eased linearly to 0.25 clockwise.
    let ik = &walk.iks[0];
    assert_eq!(
        (ik.keys[0].value.weight, ik.keys[0].value.counterclockwise),
        (1.0, true)
    );
    assert_eq!(
        (
            ik.keys[1].time,
            ik.keys[1].value.weight,
            ik.keys[1].value.counterclockwise
        ),
        (0.5, 0.25, false)
    );
    assert_eq!(
        ik.keys[1].ease,
        Ease::Step,
        "the last frame has no tweenEasing"
    );
    // Rotation keys carry the skew; the curve is a Bézier; no tweenEasing is a step.
    let arm = &walk.bones[0];
    assert_eq!(arm.rotate[0].ease, Ease::Bezier(vec![0.42, 0.0, 1.0, 1.0]));
    assert_eq!((arm.skew[0].value, arm.skew[1].value), (-5.0, 0.0));
    assert_eq!(arm.scale[0].ease, Ease::Step);
    let p = rig.pose(Some(walk), 0.25, &[(0, [0.0, 0.0], 0.0)]);
    let turned = f64::from(p.world[1].b)
        .atan2(f64::from(p.world[1].a))
        .to_degrees();
    // -30 setup, then -15 eased in to -25: half time is 0.3153568 of the way.
    let want = -30.0 - 15.0 - 10.0 * 0.315_356_81;
    assert!((turned - want).abs() < 1e-3, "{turned} {want}");
    // Colour keys: multipliers and offsets, eased by tweenEasing -1 (quadratic in).
    let slot = &walk.slots[0];
    assert_eq!(slot.color[0].value, [0.5, 1.0, 1.0, 1.0]);
    assert_eq!(slot.offset[0].value, [0.0, 0.4, 0.0, 0.0]);
    let p = rig.pose(Some(walk), 0.25, &[(0, [0.0, 0.0], 0.0)]);
    // s² at half: a quarter of the way from 0.4 to 0.
    assert!((p.offsets[0][1] - 0.3).abs() < 1e-4, "{:?}", p.offsets[0]);
}

/// A DragonBones `chain` of 2 is three bones (the old import refused any
/// chain past 1), solved as one constraint.
#[test]
fn a_dragonbones_ik_chain_of_any_length_is_imported_root_first() {
    let root = project(
        "dragonbones-chain",
        &[
            ("rigs/hero_tex.png", png(32, 16)),
            ("rigs/hero_ske.json", SKE.as_bytes().to_vec()),
            ("rigs/hero_tex.json", ATLAS.as_bytes().to_vec()),
            ("rigs/hero.ogt", IMPORT.as_bytes().to_vec()),
        ],
    );
    let rig = cooked(&root).load_rig2d("rigs/hero.ogt").unwrap();
    let wag = &rig.iks[1];
    let names: Vec<&str> = wag
        .bones
        .iter()
        .map(|b| rig.bones[*b].name.as_str())
        .collect();
    assert_eq!(
        (wag.name.as_str(), names, wag.counterclockwise),
        ("wag", vec!["tail1", "tail2", "tail3"], false)
    );
    // Solved: the tail's tip at (12, 9) px, inside its 30 px of reach.
    let p = rig.pose(None, 0.0, &[(0, [0.0, 0.0], 0.0), (1, [12.0, 9.0], 1.0)]);
    let tip = p.world[wag.bones[2]].apply(10.0, 0.0);
    assert!(
        (tip[0] - 12.0).abs() < 1e-3 && (tip[1] - 9.0).abs() < 1e-3,
        "{tip:?}"
    );
    assert!(p.ik_error[1] < 1e-3);
}

/// The notes of the rig at `rigs/hero.ogt` of a cooked project.
fn notes_of(root: &std::path::Path) -> Vec<String> {
    let report = cook_project(root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    report.assets.iter().flat_map(|a| a.notes.clone()).collect()
}

/// A one-bone export at 10 frames per second with `spin`'s rotate frames.
fn spin_export(frames: &str) -> String {
    format!(
        r#"{{ "name": "spin", "version": "5.5", "frameRate": 10, "armature": [{{
  "name": "spin",
  "bone": [{{ "name": "root" }}, {{ "name": "spin", "parent": "root", "length": 10 }}],
  "slot": [{{ "name": "s", "parent": "spin" }}],
  "skin": [{{ "slot": [{{ "name": "s", "display": [{{ "name": "arm" }}] }}] }}],
  "animation": [{{ "name": "go", "duration": 20, "bone": [{{ "name": "spin", "rotateFrame": {frames} }}] }}]
}}] }}"#
    )
}

fn spin_rig(name: &str, frames: &str) -> (Rig2dAsset, Vec<String>) {
    let root = project(
        name,
        &[
            ("rigs/hero_tex.png", png(32, 16)),
            ("rigs/hero_ske.json", spin_export(frames).into_bytes()),
            ("rigs/hero_tex.json", ATLAS.as_bytes().to_vec()),
            ("rigs/hero.ogt", IMPORT.as_bytes().to_vec()),
        ],
    );
    let notes = notes_of(&root);
    (cooked(&root).load_rig2d("rigs/hero.ogt").unwrap(), notes)
}

/// The bone `spin`'s angle (degrees, counter-clockwise, y up) at `t`.
fn spin_angle(rig: &Rig2dAsset, t: f32) -> f64 {
    let p = rig.pose(Some(&rig.clips[0]), t, &[]);
    f64::from(p.world[1].b)
        .atan2(f64::from(p.world[1].a))
        .to_degrees()
}

/// DragonBones' `clockwise` turns more than half a turn between frames;
/// the old import played the shorter way and noted it.
#[test]
fn a_dragonbones_clockwise_turn_plays_its_whole_turns() {
    // DragonBones (y down, clockwise positive): 0 with clockwise 2, then 30
    // with clockwise -1, then -60. Its parser makes the second angle the
    // first at or past 0 going up, plus one more turn: 390 (390 degrees
    // clockwise on screen); the third the first at or below 390 going
    // down: -60 (450 degrees counter-clockwise). Here (y up) the bone turns
    // -390 in the first second and +450 in the next.
    let (rig, notes) = spin_rig(
        "dragonbones-clockwise",
        r#"[{ "duration": 10, "tweenEasing": 0, "rotate": 0, "clockwise": 2 },
            { "duration": 10, "tweenEasing": 0, "rotate": 30, "clockwise": -1 },
            { "duration": 0, "rotate": -60 }]"#,
    );
    let keys = &rig.clips[0].bones[0].rotate;
    assert_eq!(
        keys.iter()
            .map(|k| (k.value.degrees, k.value.turn))
            .collect::<Vec<_>>(),
        [
            (0.0, Turn::Clockwise(1)),
            (-30.0, Turn::Counterclockwise(1)),
            (60.0, Turn::Shorter)
        ]
    );
    let same = |a: f64, b: f64| (a - b).rem_euclid(360.0).min((b - a).rem_euclid(360.0)) < 1e-3;
    for (t, want) in [
        (0.25, -97.5),
        (0.5, -195.0),
        (0.9, -351.0),
        (1.5, -390.0 + 225.0),
        (1.8, -390.0 + 360.0),
    ] {
        let got = spin_angle(&rig, t);
        assert!(same(got, want), "at {t}: {got}, not {want}");
    }
    assert!(
        !notes
            .iter()
            .any(|n| n.contains("half a turn") || n.contains("clockwise")),
        "{notes:?}"
    );
}

/// `tweenEasing` above 1 is DragonBones' sine in-out blended with the line
/// by `tweenEasing - 1`; the old import played smoothstep (within 0.011 of
/// it, a degree of a 90-degree turn) and noted it.
#[test]
fn a_dragonbones_sine_ease_plays_the_sine_at_every_tick() {
    let (rig, notes) = spin_rig(
        "dragonbones-sine",
        r#"[{ "duration": 10, "tweenEasing": 2, "rotate": 0 }, { "duration": 10, "tweenEasing": 2.5, "rotate": 90 }, { "duration": 0, "rotate": 100 }]"#,
    );
    // DragonBones' _getEasingValue for QuadInOut, written out here.
    let sine = |s: f64, a: f64| s + a * (0.5 * (1.0 - (std::f64::consts::PI * s).cos()) - s);
    let mut worst = 0.0f64;
    for tick in 0..=120 {
        let t = tick as f64 / 60.0;
        // Clockwise on screen with y down is negative here.
        let want = if t <= 1.0 {
            -90.0 * sine(t, 1.0)
        } else {
            -90.0 - 10.0 * sine(t - 1.0, 1.5)
        };
        let got = spin_angle(&rig, t as f32);
        worst = worst.max((got - want).abs());
    }
    assert!(worst < 1e-4, "the sine is missed by {worst} degrees");
    assert!(
        !notes.iter().any(|n| n.contains("tweenEasing")),
        "{notes:?}"
    );
}

/// A DragonBones path display and path constraint (the old import noted
/// the display as not imported and had no path constraint at all).
#[test]
fn a_dragonbones_path_constraint_places_its_bones_on_the_path_display() {
    // y down there: the curve from (0, 0) through (0, -10) and (10, -10)
    // to (10, 0) is, here, the one through (0, 10) and (10, 10).
    let ske = r#"{ "name": "rail", "version": "5.5", "frameRate": 10, "armature": [{
  "name": "rail",
  "bone": [{ "name": "root" },
    { "name": "a", "parent": "root", "length": 8, "transform": { "x": -40, "y": -3, "skY": 30 } },
    { "name": "b", "parent": "a", "length": 8, "transform": { "x": 8 } }],
  "slot": [{ "name": "rail", "parent": "root" }],
  "skin": [{ "slot": [{ "name": "rail", "display": [
    { "name": "rail", "type": "path", "closed": false, "constantSpeed": false, "lengths": [20],
      "vertices": [0, 10, 0, 0, 0, -10, 10, -10, 10, 0, 10, 10] }] }] }],
  "path": [{ "name": "follow", "target": "rail", "bones": ["a", "b"],
    "positionMode": "percent", "spacingMode": "percent", "rotateMode": "tangent", "position": 0, "spacing": 0.25 }],
  "animation": [{ "name": "still", "duration": 1 }]
}] }"#;
    let root = project(
        "dragonbones-path",
        &[
            ("rigs/hero_tex.png", png(32, 16)),
            ("rigs/hero_ske.json", ske.as_bytes().to_vec()),
            ("rigs/hero_tex.json", ATLAS.as_bytes().to_vec()),
            ("rigs/hero.ogt", IMPORT.as_bytes().to_vec()),
        ],
    );
    let notes = notes_of(&root);
    assert!(
        !notes.iter().any(|n| n.contains("not imported")),
        "{notes:?}"
    );
    let rig = cooked(&root).load_rig2d("rigs/hero.ogt").unwrap();
    assert_eq!(rig.paths.len(), 1);
    let p = rig.pose(None, 0.0, &[]);
    // Worked by hand (Bernstein form): a at t 0.25, (1.5625, 5.625), along
    // the tangent (3.75, 5), 53.130102 degrees; b at t 0.5, (5, 7.5), level.
    let angle = |m: &og_anim::Affine2| f64::from(m.b).atan2(f64::from(m.a)).to_degrees();
    let (a, b) = (p.world[1], p.world[2]);
    assert!(
        (a.tx - 1.5625).abs() < 1e-4 && (a.ty - 5.625).abs() < 1e-4,
        "{a:?}"
    );
    assert!((angle(&a) - 53.130_102).abs() < 1e-3, "{}", angle(&a));
    assert!(
        (b.tx - 5.0).abs() < 1e-4 && (b.ty - 7.5).abs() < 1e-4,
        "{b:?}"
    );
    assert!(angle(&b).abs() < 1e-3, "{}", angle(&b));
}

/// Every skin of an export (the old import read the first only): the one
/// named `default` is the rig's own, each other one a skin of swaps.
#[test]
fn every_dragonbones_skin_is_imported_and_the_default_comes_first() {
    let ske = r#"{ "name": "kit", "version": "5.5", "frameRate": 10, "armature": [{
  "name": "kit",
  "bone": [{ "name": "root" }],
  "slot": [{ "name": "body", "parent": "root", "displayIndex": 0 }, { "name": "hat", "parent": "root" }],
  "skin": [
    { "name": "armor", "slot": [
      { "name": "body", "display": [{ "name": "plate", "path": "cloth" }, null] },
      { "name": "hat", "display": [null] } ] },
    { "name": "default", "slot": [
      { "name": "body", "display": [{ "name": "shirt", "path": "arm" }, { "name": "vest", "path": "arm" }] },
      { "name": "hat", "display": [{ "name": "cap", "path": "arm" }] } ] }
  ],
  "animation": [{ "name": "swap", "duration": 10, "slot": [{ "name": "body",
    "displayFrame": [{ "duration": 5, "value": 0 }, { "duration": 5, "value": 1 }] }] }]
}] }"#;
    let root = project(
        "dragonbones-skins",
        &[
            ("rigs/hero_tex.png", png(32, 16)),
            ("rigs/hero_ske.json", ske.as_bytes().to_vec()),
            ("rigs/hero_tex.json", ATLAS.as_bytes().to_vec()),
            ("rigs/hero.ogt", IMPORT.as_bytes().to_vec()),
        ],
    );
    let rig = cooked(&root).load_rig2d("rigs/hero.ogt").unwrap();
    let id = |n: &str| {
        rig.attachments
            .iter()
            .position(|a| a.name == n)
            .unwrap_or_else(|| panic!("{n}"))
    };
    // The default skin (named so, though second in the file) is the rig's own.
    assert_eq!(rig.slots[0].attachment, Some(id("shirt")));
    assert_eq!(rig.skins.len(), 1);
    assert_eq!(rig.skins[0].name, "armor");
    assert_eq!(
        rig.skins[0].swaps,
        [
            (id("shirt"), Some(id("armor/plate"))),
            (id("vest"), None),
            (id("cap"), None)
        ]
    );
    let walk = &rig.clips[0];
    let shows = |skin: u32, t: f32| rig.pose_in_skin(Some(walk), t, &[], skin).slots;
    assert_eq!(shows(0, 0.2), [Some(id("shirt")), Some(id("cap"))]);
    assert_eq!(shows(0, 0.7), [Some(id("vest")), Some(id("cap"))]);
    // Armor: the plate (the cloth sub-texture) for the shirt, nothing for the vest or the cap.
    assert_eq!(shows(1, 0.2), [Some(id("armor/plate")), None]);
    assert_eq!(shows(1, 0.7), [None, None]);
    match &rig.attachments[id("armor/plate")].kind {
        AttachmentKind::Region { rect, .. } => assert_eq!(*rect, [16.0, 0.0, 16.0, 16.0]),
        k => panic!("{k:?}"),
    }
}

/// A child armature's skins (the old import noted them and dropped them):
/// the flame's `armor` swaps its fire for the cloth, worn while the rig
/// wears its own `armor`; in the default it shows its fire.
#[test]
fn a_child_armatures_skins_are_imported_and_worn_by_name() {
    let ske = r#"{ "name": "kit", "version": "5.5", "frameRate": 10, "armature": [{
  "name": "kit",
  "bone": [{ "name": "root" }],
  "slot": [{ "name": "body", "parent": "root", "displayIndex": 0 }, { "name": "lamp", "parent": "root", "displayIndex": 0 }],
  "skin": [
    { "name": "default", "slot": [
      { "name": "body", "display": [{ "name": "shirt", "path": "arm" }] },
      { "name": "lamp", "display": [{ "name": "flame", "type": "armature" }] } ] },
    { "name": "armor", "slot": [
      { "name": "body", "display": [{ "name": "plate", "path": "cloth" }] } ] }
  ],
  "animation": [{ "name": "idle", "duration": 10 }]
}, {
  "name": "flame",
  "bone": [{ "name": "base" }],
  "slot": [{ "name": "fire", "parent": "base", "displayIndex": 0 }],
  "skin": [
    { "name": "default", "slot": [{ "name": "fire", "display": [{ "name": "fire", "path": "arm" }] }] },
    { "name": "armor", "slot": [{ "name": "fire", "display": [{ "name": "ember", "path": "cloth" }] }] }
  ],
  "animation": [{ "name": "idle", "duration": 10 }]
}] }"#;
    let root = project(
        "dragonbones-child-skins",
        &[
            ("rigs/hero_tex.png", png(32, 16)),
            ("rigs/hero_ske.json", ske.as_bytes().to_vec()),
            ("rigs/hero_tex.json", ATLAS.as_bytes().to_vec()),
            ("rigs/hero.ogt", IMPORT.as_bytes().to_vec()),
        ],
    );
    let rig = cooked(&root).load_rig2d("rigs/hero.ogt").unwrap();
    assert_eq!(rig.armatures.len(), 1);
    let flame = &rig.armatures[0];
    let id = |n: &str| {
        flame
            .attachments
            .iter()
            .position(|a| a.name == n)
            .unwrap_or_else(|| panic!("{n}"))
    };
    assert_eq!(flame.skins.len(), 1);
    assert_eq!(flame.skins[0].name, "armor");
    assert_eq!(
        flame.skins[0].swaps,
        [(id("fire"), Some(id("armor/ember")))]
    );
    let fire = |skin: u32| {
        let p = rig.pose_in_skin(Some(&rig.clips[0]), 0.0, &[], skin);
        p.children[0].1.slots[0]
    };
    assert_eq!(fire(0), Some(id("fire")));
    assert_eq!(fire(1), Some(id("armor/ember")));
}
