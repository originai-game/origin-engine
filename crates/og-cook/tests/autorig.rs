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
    let order = &pose.order;
    for (name, [l, t, r, b]) in rects {
        let slot = rig.slots.iter().position(|s| &s.name == name).unwrap();
        let at = order.iter().position(|&s| s == slot).unwrap();
        let quad = &vertices[at * 4..at * 4 + 4];
        let mut got: Vec<[f32; 2]> = quad
            .iter()
            .map(|v| [v.position[0] * rig.unit, v.position[1] * rig.unit])
            .collect();
        let mut want = vec![
            canvas(*l, *t),
            canvas(*r, *t),
            canvas(*l, *b),
            canvas(*r, *b),
        ];
        let key = |p: &[f32; 2]| ((p[0] * 4.0).round() as i32, (p[1] * 4.0).round() as i32);
        got.sort_by_key(key);
        want.sort_by_key(key);
        for (g, w) in got.iter().zip(&want) {
            assert!(near(*g, *w, 0.01), "{name}: corners {got:?}, not {want:?}");
        }
        // Its texels are its own: the region's rect in the atlas is its size.
        let AttachmentKind::Region { rect, size, .. } =
            &rig.attachments[rig.slots[slot].attachment.unwrap()].kind
        else {
            panic!("{name} is a region");
        };
        assert_eq!([rect[2], rect[3]], *size);
        assert_eq!(*size, [r - l, b - t]);
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
