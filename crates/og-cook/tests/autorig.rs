//! Layered 2D art auto-rigged by the cook (`type rig2d` with `parts`):
//! the stick figure in `fixtures/autorig/` (`make.sh`), as a PSD and as a
//! folder of PNG parts.
//!
//! Ways this could fail: a joint is put at the part's middle, or at the
//! overlap's centroid over the whole part (a shoulder halfway down the
//! upper arm, because the arm lies over the torso); the canvas's y-down
//! is kept (the rig stands on its head); the setup pose does not draw
//! the parts where the art has them (a region not counter-rotated
//! against its bone, or placed at its corner); the PSD's layers and the
//! folder's files rig differently; the parts cook as stray textures of
//! their own; a refusal names neither the layer nor the file.

use og_anim::{AttachmentKind, Rig2dAsset};
use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use std::path::{Path, PathBuf};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/autorig")
}

/// A project with the fixture's PSD and PNG folder under `assets/chars/`,
/// `edit` applied to the folder, and `files` written.
fn project(name: &str, files: &[(&str, &str)], edit: impl Fn(&Path)) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-cook-autorig")
        .join(name);
    let _ = std::fs::remove_dir_all(&root);
    let chars = root.join("assets/chars");
    std::fs::create_dir_all(chars.join("stick")).unwrap();
    std::fs::copy(fixtures().join("stick.psd"), chars.join("stick.psd")).unwrap();
    for e in std::fs::read_dir(fixtures().join("stick")).unwrap() {
        let e = e.unwrap();
        std::fs::copy(e.path(), chars.join("stick").join(e.file_name())).unwrap();
    }
    edit(&chars.join("stick"));
    for (path, text) in files {
        std::fs::write(chars.join(path), text).unwrap();
    }
    root
}

fn rig_text(parts: &str) -> String {
    format!("type rig2d\nversion 1\n---\nparts \"{parts}\"\ntemplate humanoid-side\n")
}

fn cooked(root: &Path) -> Bundle {
    let report = cook_project(root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    Bundle::open(&root.join("cooked")).unwrap()
}

/// A bone's origin in the setup pose, rig pixels.
fn origin(rig: &Rig2dAsset, bone: &str) -> [f32; 2] {
    let pose = rig.pose(None, 0.0, &[]);
    pose.world[rig.bone(bone).unwrap_or_else(|| panic!("no bone {bone}"))].apply(0.0, 0.0)
}

/// The vertices `triangles` drew for one slot of the pose.
fn slot_vertices<'a>(
    rig: &Rig2dAsset,
    pose: &og_anim::Pose2d,
    vertices: &'a [og_anim::RigVertex],
    slot: usize,
) -> &'a [og_anim::RigVertex] {
    let mut at = 0;
    for &s in &pose.order {
        let n = match pose.slots[s].map(|a| &rig.attachments[a].kind) {
            Some(AttachmentKind::Region { .. }) => 4,
            Some(AttachmentKind::Mesh { vertices, .. }) => vertices.len(),
            _ => 0,
        };
        if s == slot {
            return &vertices[at..at + n];
        }
        at += n;
    }
    panic!("slot {slot} is not drawn")
}

fn near(a: [f32; 2], b: [f32; 2], tolerance: f32) -> bool {
    (a[0] - b[0]).abs() <= tolerance && (a[1] - b[1]).abs() <= tolerance
}

#[test]
fn an_elbow_bends_through_its_joint_instead_of_hinging() {
    use og_anim::{BoneTimeline, Clip2d, Ease, Key, Rotation, Turn};
    let root = project("bend", &[("stick.ogt", &rig_text("stick/"))], |_| {});
    let rig = cooked(&root).load_rig2d("chars/stick.ogt").unwrap();
    let forearm = rig.bone("arm-lower-r").unwrap();
    let clip = Clip2d {
        name: "bend".into(),
        duration: 1.0,
        looping: false,
        bones: vec![BoneTimeline {
            bone: forearm,
            rotate: vec![Key {
                time: 0.0,
                value: Rotation {
                    degrees: 90.0,
                    turn: Turn::Shorter,
                },
                ease: Ease::Linear,
            }],
            ..Default::default()
        }],
        slots: vec![],
        deforms: vec![],
        order: vec![],
        iks: vec![],
        events: vec![],
    };
    let setup = rig.pose(None, 0.0, &[]);
    let bent = rig.pose(Some(&clip), 0.0, &[]);
    let (before, _) = rig.triangles(&setup);
    let (after, _) = rig.triangles(&bent);
    let elbow = origin(&rig, "arm-lower-r");
    // How far each vertex moved, as a share of how far a hard hinge
    // (everything turned with the forearm about the elbow) moves it.
    let shares = |part: &str| -> Vec<([f32; 2], f32)> {
        let slot = rig.slots.iter().position(|s| s.name == part).unwrap();
        let a = slot_vertices(&rig, &setup, &before, slot);
        let b = slot_vertices(&rig, &bent, &after, slot);
        a.iter()
            .zip(b)
            .map(|(a, b)| {
                let p = [a.position[0] * rig.unit, a.position[1] * rig.unit];
                let q = [b.position[0] * rig.unit, b.position[1] * rig.unit];
                let v = [p[0] - elbow[0], p[1] - elbow[1]];
                let hinge = [elbow[0] - v[1], elbow[1] + v[0]];
                let full = (hinge[0] - p[0]).hypot(hinge[1] - p[1]);
                (p, (q[0] - p[0]).hypot(q[1] - p[1]) / full.max(1e-3))
            })
            .collect()
    };
    let lower = shares("arm-lower-r");
    let top = lower.iter().map(|v| v.0[1]).fold(f32::MIN, f32::max);
    let mid = elbow[1] - 12.0;
    for (p, share) in &lower {
        // The forearm's top, inside the upper arm, stays mostly with it.
        if (p[1] - top).abs() < 0.5 {
            assert!(
                *share < 0.4,
                "the forearm's top at {p:?} moved {share} of a hinge"
            );
        }
        // Its middle turns with it.
        if (p[1] - mid).abs() < 4.0 {
            assert!(
                (share - 1.0).abs() < 0.05,
                "the forearm's middle at {p:?} moved {share} of a hinge"
            );
        }
    }
    // The pixels at the joint turn about it rather than being pinched
    // towards it (blending a quarter turn halves into 0.71 of their reach).
    let slot = rig
        .slots
        .iter()
        .position(|s| s.name == "arm-lower-r")
        .unwrap();
    let a = slot_vertices(&rig, &setup, &before, slot);
    let b = slot_vertices(&rig, &bent, &after, slot);
    let reach = |v: &og_anim::RigVertex| {
        (v.position[0] * rig.unit - elbow[0]).hypot(v.position[1] * rig.unit - elbow[1])
    };
    let mut seen = 0;
    for (a, b) in a.iter().zip(b) {
        if (a.position[1] * rig.unit - elbow[1]).abs() <= 4.0 && reach(a) > 2.0 {
            seen += 1;
            assert!(
                reach(b) > 0.9 * reach(a),
                "a joint vertex is pinched from {} to {}",
                reach(a),
                reach(b)
            );
        }
    }
    assert!(seen >= 4, "{seen} vertices at the joint");
    // The upper arm's end, over the forearm, follows part of the way.
    let upper = shares("arm-upper-r");
    let bottom = upper.iter().map(|v| v.0[1]).fold(f32::MAX, f32::min);
    let end: Vec<f32> = upper
        .iter()
        .filter(|v| (v.0[1] - bottom).abs() < 0.5)
        .map(|v| v.1)
        .collect();
    assert!(
        !end.is_empty() && end.iter().all(|s| *s > 0.3 && *s < 1.0),
        "the upper arm's end moved {end:?} of a hinge"
    );
}

/// Every drawn vertex of a pose, rig pixels.
fn drawn(rig: &Rig2dAsset, pose: &og_anim::Pose2d) -> Vec<[f32; 2]> {
    rig.triangles(pose)
        .0
        .iter()
        .map(|v| [v.position[0] * rig.unit, v.position[1] * rig.unit])
        .collect()
}

/// The other templates' fixtures: a folder of parts each and its template.
const TEMPLATES: [(&str, &str); 3] = [
    ("front", "humanoid-front"),
    ("dog", "quadruped"),
    ("blob", "creature"),
];

/// A project `name` with fixture folder `dir` under `assets/chars/` and
/// `chars/<dir>.ogt` rigging it on `template`.
fn fixture(name: &str, dir: &str, template: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-cook-autorig")
        .join(format!("{name}-{dir}"));
    let _ = std::fs::remove_dir_all(&root);
    let chars = root.join("assets/chars");
    std::fs::create_dir_all(chars.join(dir)).unwrap();
    for e in std::fs::read_dir(fixtures().join(dir)).unwrap() {
        let e = e.unwrap();
        std::fs::copy(e.path(), chars.join(dir).join(e.file_name())).unwrap();
    }
    std::fs::write(
        chars.join(format!("{dir}.ogt")),
        format!("type rig2d\nversion 1\n---\nparts \"{dir}/\"\ntemplate {template}\n"),
    )
    .unwrap();
    root
}

/// Whether a looping clip ends where it starts.
fn loops(rig: &Rig2dAsset) {
    for c in rig.clips.iter().filter(|c| c.looping) {
        let (a, b) = (
            rig.pose(Some(c), 0.0, &[]),
            rig.pose(Some(c), c.duration, &[]),
        );
        for (p, q) in drawn(rig, &a).iter().zip(drawn(rig, &b)) {
            assert!(
                near(*p, q, 0.05),
                "{}: {} does not loop: {p:?} then {q:?}",
                rig.texture,
                c.name
            );
        }
    }
}

/// `bone`'s origin at `fraction` of `clip`, rig pixels.
fn at(rig: &Rig2dAsset, clip: &str, fraction: f32, bone: &str) -> [f32; 2] {
    let c = &rig.clips[rig.clip(clip).unwrap() as usize];
    let pose = rig.pose(Some(c), fraction * c.duration, &[]);
    pose.world[rig.bone(bone).unwrap()].apply(0.0, 0.0)
}

/// Each `(foot, phase)` is on the ground (its setup height) and its IK
/// reaches it from its contact at `phase` for `stance` of `clip`'s cycle.
fn planted(rig: &Rig2dAsset, clip: &str, stance: f32, feet: &[(&str, f32)]) {
    let c = &rig.clips[rig.clip(clip).unwrap() as usize];
    for i in 0..=48 {
        let f = i as f32 / 48.0;
        let pose = rig.pose(Some(c), f * c.duration, &[]);
        assert!(
            pose.ik_error.iter().all(|e| *e < 0.5),
            "{} {clip} at {f}: a foot misses its target by {:?}",
            rig.texture,
            pose.ik_error
        );
        for (foot, phase) in feet {
            if (f - phase).rem_euclid(1.0) <= stance {
                let y = pose.world[rig.bone(foot).unwrap()].apply(0.0, 0.0)[1];
                let ground = origin(rig, foot)[1];
                assert!(
                    (y - ground).abs() < 0.5,
                    "{} {clip} at {f}: planted `{foot}` is at {y}, not {ground}",
                    rig.texture
                );
            }
        }
    }
}

#[test]
fn every_template_plays_its_library_with_its_feet_on_the_ground() {
    let rigs: Vec<Rig2dAsset> = TEMPLATES
        .iter()
        .map(|(dir, template)| {
            cooked(&fixture("library", dir, template))
                .load_rig2d(&format!("chars/{dir}.ogt"))
                .unwrap()
        })
        .collect();
    let [front, dog, blob] = &rigs[..] else {
        unreachable!()
    };
    let names = |r: &Rig2dAsset| r.clips.iter().map(|c| c.name.clone()).collect::<Vec<_>>();
    let common = ["idle", "walk", "run", "jump", "attack", "hit", "die"];
    for (rig, emotes) in [
        (front, ["wave", "cheer", "shrug"]),
        (dog, ["howl", "play", "sniff"]),
        (blob, ["happy", "wiggle", "look"]),
    ] {
        assert_eq!(
            names(rig),
            [&common[..], &emotes[..]].concat(),
            "{}",
            rig.texture
        );
        loops(rig);
    }
    // Seen from the front, a planted foot stays put while the other lifts,
    // and straight knees bend outwards, never across each other.
    planted(front, "walk", 0.55, &[("foot-r", 0.0), ("foot-l", 0.5)]);
    planted(front, "run", 0.35, &[("foot-r", 0.0), ("foot-l", 0.5)]);
    for side in ["l", "r"] {
        let (hip, knee, ankle) = (
            at(front, "jump", 0.18, &format!("leg-upper-{side}")),
            at(front, "jump", 0.18, &format!("leg-lower-{side}")),
            at(front, "jump", 0.18, &format!("foot-{side}")),
        );
        let line = hip[0] + (ankle[0] - hip[0]) * (knee[1] - hip[1]) / (ankle[1] - hip[1]);
        let out = if side == "l" {
            line - knee[0]
        } else {
            knee[0] - line
        };
        assert!(out > 2.0, "the {side} knee bends {out} outwards");
    }
    // A dog walks its four paws one after another and bounds on pairs,
    // each planted through its stance.
    planted(
        dog,
        "walk",
        0.66,
        &[
            ("paw-back-l", 0.0),
            ("paw-front-l", 0.25),
            ("paw-back-r", 0.5),
            ("paw-front-r", 0.75),
        ],
    );
    planted(
        dog,
        "run",
        0.34,
        &[
            ("paw-back-l", 0.0),
            ("paw-back-r", 0.07),
            ("paw-front-l", 0.48),
            ("paw-front-r", 0.55),
        ],
    );
    // Dead, it lies with its belly on the ground and its paws on it.
    let die = &dog.clips[dog.clip("die").unwrap() as usize];
    let lying = drawn(dog, &dog.pose(Some(die), die.duration, &[]));
    let low = lying.iter().map(|p| p[1]).fold(f32::MAX, f32::min);
    assert!(
        low > -1.0 && low < 3.0,
        "the dog's lowest point is at {low}"
    );
    let pose = dog.pose(Some(die), die.duration, &[]);
    let body = dog.slots.iter().position(|s| s.name == "body").unwrap();
    let belly = slot_vertices(dog, &pose, &dog.triangles(&pose).0, body)
        .iter()
        .map(|v| v.position[1] * dog.unit)
        .fold(f32::MAX, f32::min);
    assert!(belly.abs() < 1.5, "the belly is {belly} off the ground");
    // A blob squashes and stretches about its bottom: breathing, landing
    // from a jump and slumping dead (upright, where its mesh's lowest
    // vertices are its drawn bottom), its body's bottom stays on the
    // ground, and its eyes stay round (the rigid check above).
    let body = blob.slots.iter().position(|s| s.name == "body").unwrap();
    let bottom = |clip: &str, f: f32| {
        let c = &blob.clips[blob.clip(clip).unwrap() as usize];
        let pose = blob.pose(Some(c), f * c.duration, &[]);
        let v = blob.triangles(&pose).0;
        slot_vertices(blob, &pose, &v, body)
            .iter()
            .map(|v| v.position[1] * blob.unit)
            .fold(f32::MAX, f32::min)
    };
    let rest = bottom("idle", 0.0);
    for (clip, f) in [
        ("idle", 0.45),
        ("idle", 0.95),
        ("jump", 29.0 / 36.0),
        ("die", 1.0),
    ] {
        let b = bottom(clip, f);
        assert!(
            (b - rest).abs() < 1.0,
            "the blob's bottom in {clip} at {f} is at {b}, not {rest}"
        );
    }
    let c = &blob.clips[blob.clip("jump").unwrap() as usize];
    let s = |f: f32| {
        let pose = blob.pose(Some(c), f * c.duration, &[]);
        let w = pose.world[blob.bone("body").unwrap()];
        [
            (w.a * w.a + w.b * w.b).sqrt(),
            (w.c * w.c + w.d * w.d).sqrt(),
        ]
    };
    let (crouch, launch) = (s(0.2), s(0.3));
    assert!(
        crouch[0] < 0.8 && crouch[1] > 1.1,
        "the crouch is {crouch:?}"
    );
    assert!(
        launch[0] > 1.2 && launch[1] < 0.9,
        "the launch is {launch:?}"
    );
}
