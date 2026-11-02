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

fn errors(root: &Path) -> String {
    let report = cook_project(root, &CookOptions::default()).unwrap();
    assert!(!report.ok(), "the cook should refuse");
    report.errors().join("\n")
}

/// A bone's origin in the setup pose, rig pixels.
fn origin(rig: &Rig2dAsset, bone: &str) -> [f32; 2] {
    let pose = rig.pose(None, 0.0, &[]);
    pose.world[rig.bone(bone).unwrap_or_else(|| panic!("no bone {bone}"))].apply(0.0, 0.0)
}

/// The fixture's canvas pixel as rig pixels: the root is under the hips
/// (x 50) on the ground (the feet's last row, 193, ends at 194).
fn canvas(x: f32, y: f32) -> [f32; 2] {
    [x - 50.0, 194.0 - y]
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
fn layered_art_rigs_with_its_joints_where_the_parts_overlap() {
    let root = project("joints", &[("stick.ogt", &rig_text("stick.psd"))], |_| {});
    let rig = cooked(&root).load_rig2d("chars/stick.ogt").unwrap();
    // Each joint at its overlap with the parent near the joint's end: the
    // shoulder near the top of the upper arm (the whole arm overlaps the
    // torso, whose centroid would be y 64), the elbow where the forearm
    // starts, the knee, the ankle, the neck under the head.
    for (bone, x, y) in [
        ("arm-upper-r", 51.0, 53.5),
        ("arm-lower-r", 51.0, 78.0),
        ("hand-r", 51.0, 104.0),
        ("leg-lower-r", 51.0, 148.0),
        ("foot-r", 50.5, 183.5),
        ("head", 50.0, 43.0),
        ("torso", 50.0, 97.0),
    ] {
        let at = origin(&rig, bone);
        assert!(
            near(at, canvas(x, y), 1.0),
            "{bone} at {at:?}, not {:?}",
            canvas(x, y)
        );
    }
    // Upright: the head above the hips, y up.
    assert!(origin(&rig, "head")[1] > origin(&rig, "pelvis")[1] + 40.0);
    // Bones run from joint to joint: the upper arm points down at the elbow.
    let arm = &rig.bones[rig.bone("arm-upper-r").unwrap()];
    assert!((arm.length - 24.5).abs() < 1.5, "upper arm {}", arm.length);
    // Two-bone IK on each leg, its target at the ankle.
    assert_eq!(rig.iks.len(), 2);
    let ik = &rig.iks[1];
    let names: Vec<_> = ik
        .bones
        .iter()
        .map(|&b| rig.bones[b].name.as_str())
        .collect();
    assert_eq!(names, ["leg-upper-r", "leg-lower-r"]);
    assert!(near(
        origin(&rig, &rig.bones[ik.target].name.clone()),
        origin(&rig, "foot-r"),
        0.01
    ));
    // The knee bends forwards (the figure faces right) when the foot is
    // pulled in and lifted: in front of the line from the hip to the foot.
    let hip = origin(&rig, "leg-upper-r");
    let foot = [hip[0] + 20.0, hip[1] - 60.0];
    let pose = rig.pose(None, 0.0, &[(1, foot, 1.0)]);
    let knee = pose.world[rig.bone("leg-lower-r").unwrap()].apply(0.0, 0.0);
    let ankle = pose.world[rig.bone("foot-r").unwrap()].apply(0.0, 0.0);
    assert!(
        near(ankle, foot, 0.5),
        "the foot reaches {ankle:?}, not {foot:?}"
    );
    let side = (foot[0] - hip[0]) * (knee[1] - hip[1]) - (foot[1] - hip[1]) * (knee[0] - hip[0]);
    assert!(
        side > 0.0 && knee[0] > foot[0],
        "the knee at {knee:?} bends backwards"
    );
    assert_eq!(rig.texture, "chars/stick.ogt#atlas");
    // Two world units tall by default: 182 pixels of figure.
    assert!((rig.unit - 91.0).abs() < 0.01, "unit {}", rig.unit);
}

#[test]
fn the_setup_pose_draws_every_part_where_the_art_has_it() {
    let root = project("setup", &[("stick.ogt", &rig_text("stick/"))], |_| {});
    let bundle = cooked(&root);
    let rig = bundle.load_rig2d("chars/stick.ogt").unwrap();
    let atlas = bundle.load_texture("chars/stick.ogt#atlas").unwrap();
    assert_eq!([atlas.meta.width, atlas.meta.height], rig.size);
    // Each part's trimmed rectangle on the canvas (make.sh's shapes, edges inclusive).
    let rects: &[(&str, [f32; 4])] = &[
        ("arm-upper-r", [47.0, 48.0, 55.0, 80.0]),
        ("hand-l", [43.0, 102.0, 53.0, 114.0]),
        ("foot-r", [45.0, 182.0, 68.0, 194.0]),
        ("torso", [40.0, 42.0, 60.0, 100.0]),
        ("head", [34.0, 12.0, 67.0, 45.0]),
    ];
    let pose = rig.pose(None, 0.0, &[]);
    let (vertices, _) = rig.triangles(&pose);
    let atlas_size = [rig.size[0] as f32, rig.size[1] as f32];
    for (name, [l, t, r, b]) in rects {
        let slot = rig.slots.iter().position(|s| &s.name == name).unwrap();
        let drawn = slot_vertices(&rig, &pose, &vertices, slot);
        // Every vertex sits where its texel is in the art: the same
        // offset from its texel for all of them, and together they span
        // the part's rectangle.
        let positions: Vec<[f32; 2]> = drawn
            .iter()
            .map(|v| [v.position[0] * rig.unit, v.position[1] * rig.unit])
            .collect();
        let texels: Vec<[f32; 2]> = drawn
            .iter()
            .map(|v| [v.uv[0] * atlas_size[0], v.uv[1] * atlas_size[1]])
            .collect();
        let offset = |i: usize| {
            [
                positions[i][0] - texels[i][0],
                positions[i][1] + texels[i][1],
            ]
        };
        for i in 0..drawn.len() {
            assert!(
                near(offset(i), offset(0), 0.01),
                "{name}: vertex {i} is not where its texel is drawn"
            );
        }
        let lo = positions
            .iter()
            .fold([f32::MAX; 2], |a, p| [a[0].min(p[0]), a[1].min(p[1])]);
        let hi = positions
            .iter()
            .fold([f32::MIN; 2], |a, p| [a[0].max(p[0]), a[1].max(p[1])]);
        let (want_lo, want_hi) = (canvas(*l, *b), canvas(*r, *t));
        assert!(
            near(lo, want_lo, 0.01) && near(hi, want_hi, 0.01),
            "{name}: spans {lo:?}..{hi:?}, not {want_lo:?}..{want_hi:?}"
        );
        // Bending parts are meshes, the rest regions.
        let kind = &rig.attachments[rig.slots[slot].attachment.unwrap()].kind;
        let bends = matches!(*name, "arm-upper-r" | "torso");
        assert_eq!(matches!(kind, AttachmentKind::Mesh { .. }), bends, "{name}");
    }
    // Back to front as the files are numbered: the far arm first, the near hand last.
    assert_eq!(rig.slots.first().unwrap().name, "arm-upper-l");
    assert_eq!(rig.slots.last().unwrap().name, "hand-r");
}

#[test]
fn the_psd_and_the_folder_rig_alike_and_neither_cooks_as_a_texture() {
    let root = project(
        "alike",
        &[
            ("from-psd.ogt", &rig_text("stick.psd")),
            ("from-folder.ogt", &rig_text("stick/")),
        ],
        |_| {},
    );
    let bundle = cooked(&root);
    let a = bundle.load_rig2d("chars/from-psd.ogt").unwrap();
    let b = bundle.load_rig2d("chars/from-folder.ogt").unwrap();
    assert_eq!(a.bones, b.bones);
    assert_eq!(a.slots, b.slots);
    assert_eq!(a.attachments, b.attachments);
    assert_eq!(a.iks, b.iks);
    assert!(bundle.output("chars/stick.psd").is_err());
    assert!(bundle.output("chars/stick/09-head.png").is_err());
}

#[test]
fn a_pivot_line_moves_a_joint() {
    let text = rig_text("stick/") + "pivot hand-r 51 108\n";
    let root = project("pivot", &[("stick.ogt", &text)], |_| {});
    let rig = cooked(&root).load_rig2d("chars/stick.ogt").unwrap();
    assert!(near(origin(&rig, "hand-r"), canvas(51.0, 108.0), 0.01));
}

#[test]
fn art_that_does_not_fit_the_template_is_refused_naming_the_file_and_the_fix() {
    let text = rig_text("stick/");
    // A missing part: named with what it is.
    let root = project("missing", &[("stick.ogt", &text)], |d| {
        std::fs::remove_file(d.join("03-hand-l.png")).unwrap();
    });
    let e = errors(&root);
    assert!(
        e.contains("chars/stick/") && e.contains("`hand-l` (the far hand)"),
        "{e}"
    );
    // A misspelt name: the nearest part suggested, the file named.
    let root = project("misspelt", &[("stick.ogt", &text)], |d| {
        std::fs::rename(d.join("13-arm-upper-r.png"), d.join("13-arm-uper-r.png")).unwrap();
    });
    let e = errors(&root);
    assert!(
        e.contains("chars/stick/13-arm-uper-r.png") && e.contains("did you mean `arm-upper-r`"),
        "{e}"
    );
    // A hand drawn away from its forearm.
    let root = project("apart", &[("stick.ogt", &text)], |d| {
        let mut img = image::RgbaImage::new(100, 200);
        for y in 150..160 {
            for x in 80..90 {
                img.put_pixel(x, y, image::Rgba([200, 180, 120, 255]));
            }
        }
        img.save(d.join("15-hand-r.png")).unwrap();
    });
    let e = errors(&root);
    assert!(
        e.contains("chars/stick/15-hand-r.png")
            && e.contains("does not touch `arm-lower-r`")
            && e.contains("pivot hand-r"),
        "{e}"
    );
    // A part at another size than the canvas.
    let root = project("size", &[("stick.ogt", &text)], |d| {
        image::RgbaImage::from_pixel(10, 10, image::Rgba([1, 1, 1, 255]))
            .save(d.join("09-head.png"))
            .unwrap();
    });
    let e = errors(&root);
    assert!(
        e.contains("chars/stick/09-head.png") && e.contains("canvas size"),
        "{e}"
    );
    // No template, and lines the cook makes itself.
    let root = project(
        "lines",
        &[(
            "stick.ogt",
            "type rig2d\nversion 1\n---\nparts \"stick/\"\n",
        )],
        |_| {},
    );
    assert!(errors(&root).contains("names its template"));
    let root = project(
        "bones",
        &[("stick.ogt", &(text.clone() + "bone extra {\n}\n"))],
        |_| {},
    );
    assert!(errors(&root).contains("`bone` is not one of its lines"));
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
