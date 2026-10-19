//! Retargeting at cook time: `templates/fx-showcase`'s animation library
//! (`anims/humanoid-clips.glb`: Blender names, T-pose, legs 0.45 + 0.45 m,
//! a walk made by IK with its feet planted) named by two characters'
//! import settings, `animation { library … }`, and written into their
//! `#anim` blobs: `strider.glb` (Mixamo names, an armature scaled by 0.01,
//! +Y along each bone, A-posed arms, legs 0.70 + 0.44 m; its rig file says
//! `scheme mixamo`) and `stout.glb` (Unreal names, +X along each bone, legs
//! 0.30 + 0.32 m, detected).
//!
//! Failure modes covered: the library's clips not reaching the model (the
//! clip list), markers lost, a clip bound to the wrong nodes after the
//! cook's topological reorder (the poses would not be the ones computed),
//! planted feet floating, sinking or sliding on the other proportions (the
//! contact check, per character and foot), the rig file or library not in
//! the cook stamp (an edit of either must re-cook the `#anim`), and a bad
//! rig file not named with its line.
//!
//! Evidence: `$OG_EVIDENCE_DIR/retarget-contacts.json`.

use og_anim::{ClipAsset, HumanoidMap, Pose, SkeletonAsset, bind, sample};
use og_cook::cook::{CookOptions, cook_project};
use og_cook::runtime::Bundle;
use std::path::{Path, PathBuf};

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

const STRIDER_SETTINGS: &str = "type import\nversion 1\n---\nanimation {\n  library \"anims/humanoid-clips.glb\"\n  rig \"rigs/strider.ogt\"\n}\n";
const STOUT_SETTINGS: &str =
    "type import\nversion 1\n---\nanimation {\n  library \"anims/humanoid-clips.glb\"\n}\n";
const STRIDER_RIG: &str = "type rig\nversion 1\n---\nscheme mixamo\n";

fn stage(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-cook-retarget")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    for d in ["assets/models", "assets/anims", "assets/rigs"] {
        std::fs::create_dir_all(dir.join(d)).unwrap();
    }
    let fx = repo().join("templates/fx-showcase/assets");
    for f in [
        "models/strider.glb",
        "models/stout.glb",
        "anims/humanoid-clips.glb",
    ] {
        std::fs::copy(fx.join(f), dir.join("assets").join(f)).unwrap();
    }
    std::fs::write(dir.join("assets/models/strider.glb.ogt"), STRIDER_SETTINGS).unwrap();
    std::fs::write(dir.join("assets/models/stout.glb.ogt"), STOUT_SETTINGS).unwrap();
    std::fs::write(dir.join("assets/rigs/strider.ogt"), STRIDER_RIG).unwrap();
    dir
}

fn model_pose(s: &SkeletonAsset, c: &ClipAsset, i: u32) -> Vec<og_anim::Trs> {
    let mut p = Pose::rest(s);
    let b = bind(s, c);
    sample(
        s,
        c,
        &b,
        (i as f32 / c.rate).min(c.duration),
        og_anim::NONE,
        &mut p,
    );
    s.model_space(&p.nodes)
}

struct Contacts {
    planted: usize,
    height: f32,
    slide: f32,
}

/// The target's feet over the frames where the library's are planted
/// (within 1 mm of their lowest): their height above the target's ground
/// (the library's ground scaled onto the target) and their xz travel
/// against the library foot's, scaled.
fn contacts(
    lib: &SkeletonAsset,
    walk: &ClipAsset,
    target: &SkeletonAsset,
    retargeted: &ClipAsset,
) -> (Contacts, f32) {
    let (sm, tm) = (HumanoidMap::detect(lib), HumanoidMap::detect(target));
    let s_rest = lib.model_space(&lib.rest_pose());
    let t_rest = target.model_space(&target.rest_pose());
    let height = |rest: &[og_anim::Trs], m: &HumanoidMap| {
        rest[m.node("hips").unwrap() as usize].translation.y
            - rest[m.node("leftFoot").unwrap() as usize].translation.y
    };
    let k = height(&t_rest, &tm) / height(&s_rest, &sm);
    let mut c = Contacts {
        planted: 0,
        height: 0.0,
        slide: 0.0,
    };
    for foot in ["leftFoot", "rightFoot"] {
        let (sf, tf) = (
            sm.node(foot).unwrap() as usize,
            tm.node(foot).unwrap() as usize,
        );
        let src: Vec<_> = (0..walk.frames)
            .map(|i| model_pose(lib, walk, i)[sf].translation)
            .collect();
        let dst: Vec<_> = (0..retargeted.frames)
            .map(|i| model_pose(target, retargeted, i)[tf].translation)
            .collect();
        let ground_s = src.iter().map(|p| p.y).fold(f32::MAX, f32::min);
        let ground_t = t_rest[tf].translation.y + (ground_s - s_rest[sf].translation.y) * k;
        let mut last = None;
        for i in 0..src.len() {
            if src[i].y > ground_s + 0.001 {
                last = None;
                continue;
            }
            c.planted += 1;
            c.height = c.height.max((dst[i].y - ground_t).abs());
            if let Some(j) = last {
                let (ds, dt): (og_core::glam::Vec3, og_core::glam::Vec3) =
                    ((src[i] - src[j]) * k, dst[i] - dst[j]);
                c.slide = c
                    .slide
                    .max(((dt.x - ds.x).powi(2) + (dt.z - ds.z).powi(2)).sqrt());
            }
            last = Some(i);
        }
    }
    (c, k)
}

fn cook(root: &Path) -> og_cook::cook::Report {
    let report = cook_project(root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    report
}

#[test]
fn library_clips_are_retargeted_into_each_characters_anim_with_planted_feet() {
    let root = stage("contacts");
    let report = cook(&root);
    let notes = report
        .assets
        .iter()
        .find(|a| a.source == "models/strider.glb")
        .unwrap()
        .notes
        .join("\n");
    assert!(
        notes.contains("clip 'walk' retargeted (blender -> mixamo"),
        "{notes}"
    );
    let b = Bundle::open(&root.join("cooked")).unwrap();
    let lib = b.load_animation("anims/humanoid-clips.glb").unwrap();
    let walk = lib.clip("walk").unwrap();
    let mut evidence = Vec::new();
    for (model, scheme) in [("models/strider.glb", "mixamo"), ("models/stout.glb", "ue")] {
        let a = b.load_animation(model).unwrap();
        assert_eq!(HumanoidMap::detect(&a.skeleton).scheme, scheme);
        let names: Vec<&str> = a.clips.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["walk", "run", "idle"], "{model}");
        let w = a.clip("walk").unwrap();
        assert_eq!(w.events, walk.events, "{model}: the markers came along");
        assert_eq!((w.frames, w.rate), (walk.frames, walk.rate));
        let (c, k) = contacts(&lib.skeleton, walk, &a.skeleton, w);
        eprintln!(
            "{model}: leg ratio {k:.3}, {} planted frames, height error {:.5} m, slide {:.5} m",
            c.planted, c.height, c.slide
        );
        assert!(
            c.planted >= 20,
            "{model}: the walk plants its feet ({})",
            c.planted
        );
        assert!(
            c.height <= 0.002,
            "{model}: planted feet on the ground ({} m)",
            c.height
        );
        assert!(
            c.slide <= 0.002,
            "{model}: planted feet do not slide ({} m)",
            c.slide
        );
        evidence.push(format!(
            "{{\"model\":\"{model}\",\"scheme\":\"{scheme}\",\"legRatio\":{k:.4},\"plantedFrames\":{},\"heightError\":{:.6},\"slide\":{:.6}}}",
            c.planted, c.height, c.slide
        ));
    }
    let out = std::env::var_os("OG_EVIDENCE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("evidence"));
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(
        out.join("retarget-contacts.json"),
        format!("[{}]\n", evidence.join(",\n")),
    )
    .unwrap();
}

#[test]
fn an_edit_of_the_rig_or_the_library_recooks_and_a_bad_rig_names_its_line() {
    let root = stage("stamps");
    cook(&root);
    let recooked = |r: &og_cook::cook::Report, id: &str| {
        r.assets
            .iter()
            .flat_map(|a| &a.outputs)
            .find(|o| o.id == id)
            .is_some_and(|o| o.cooked)
    };
    // Nothing changed: nothing cooks.
    let r = cook(&root);
    assert!(!recooked(&r, "models/strider.glb#anim"));
    // The rig file changes: strider's #anim cooks again, stout's does not.
    std::fs::write(
        root.join("assets/rigs/strider.ogt"),
        format!("{STRIDER_RIG}leftToes none\n"),
    )
    .unwrap();
    let r = cook(&root);
    assert!(recooked(&r, "models/strider.glb#anim"));
    assert!(!recooked(&r, "models/stout.glb#anim"));
    // The library's import settings change: both cook again.
    std::fs::write(
        root.join("assets/anims/humanoid-clips.glb.ogt"),
        "type import\nversion 1\n---\nanimation {\n  rate 60\n}\n",
    )
    .unwrap();
    let r = cook(&root);
    assert!(recooked(&r, "models/strider.glb#anim"));
    assert!(recooked(&r, "models/stout.glb#anim"));
    // A rig naming a node the model lacks fails with the file and its line.
    std::fs::write(
        root.join("assets/rigs/strider.ogt"),
        "type rig\nversion 1\n---\nscheme mixamo\nhips \"Pelvis\"\n",
    )
    .unwrap();
    let r = cook_project(&root, &CookOptions::default()).unwrap();
    let errors = r.errors().join("\n");
    assert!(
        errors.contains("rigs/strider.ogt")
            && errors.contains("line 5")
            && errors.contains("Pelvis"),
        "{errors}"
    );
}
