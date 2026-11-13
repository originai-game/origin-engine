//! FBX and OBJ through the cook: each becomes the same model and `#anim`
//! blobs a glTF does, through the one glTF path. The fixtures under
//! `fixtures/fbx/` are made as their README says (Blender's exporters on
//! a scene its script builds, and a hand-written ASCII FBX).
//!
//! Ways this could fail: `.fbx` and `.obj` are not models at all (left
//! ignored); the skin binds with the wrong inverse binds or joints, so the
//! arm drifts from the glTF of the same scene as it plays; a clip is
//! baked at the wrong rate or offset, or a morph weight track is lost;
//! the textures and `.mtl` beside a model are cooked a second time as
//! assets of their own, or not hashed, so an edited `.mtl` never re-cooks.

use og_anim::cooked::{BundleAnim, CookedAnimation};
use og_anim::{Pose, bind, sample};
use og_cook::runtime::{Bundle, CookedModel};
use og_cook::{CookOptions, cook_project};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/fbx")
        .join(name)
}

const OBJ: &str = "mtllib quad.mtl
v 0 0 0
v 1 0 0
v 1 0 -1
v 0 0 -1
vt 0 0
vt 1 0
vt 1 1
vt 0 1
usemtl tile
f 1/1 2/2 3/3 4/4
";

fn png() -> Vec<u8> {
    let img = image::RgbaImage::from_pixel(4, 4, image::Rgba([200, 100, 50, 255]));
    let mut out = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    out
}

fn project(name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("og-cook-fbx-{name}"));
    let _ = std::fs::remove_dir_all(&root);
    let models = root.join("assets/models");
    std::fs::create_dir_all(&models).unwrap();
    for f in [
        "props.fbx",
        "props-linked.fbx",
        "crate.png",
        "arm.fbx",
        "arm.glb",
        "board.fbx",
    ] {
        std::fs::copy(fixture(f), models.join(f)).unwrap();
    }
    std::fs::write(models.join("quad.obj"), OBJ).unwrap();
    std::fs::write(
        models.join("quad.mtl"),
        "newmtl tile\nKd 1 1 1\nmap_Kd tile.png\n",
    )
    .unwrap();
    std::fs::write(models.join("tile.png"), png()).unwrap();
    root
}

fn cook(root: &Path) -> og_cook::cook::Report {
    let report = cook_project(root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    report
}

/// Every FBX and the OBJ cook to models; the textures and the `.mtl`
/// beside them are consumed (no asset of their own); editing the `.mtl`
/// re-cooks the OBJ and nothing else.
#[test]
fn fbx_and_obj_cook_to_models_and_consume_what_they_name() {
    let root = project("models");
    let report = cook(&root);
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    for id in [
        "models/props.fbx",
        "models/props-linked.fbx",
        "models/arm.fbx",
        "models/board.fbx",
        "models/quad.obj",
    ] {
        let m = bundle
            .load_model(id)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        assert!(!m.primitives.is_empty(), "{id}");
    }
    let sources: Vec<&str> = report.assets.iter().map(|a| a.source.as_str()).collect();
    for consumed in ["models/crate.png", "models/tile.png", "models/quad.mtl"] {
        assert!(
            !sources.contains(&consumed),
            "{consumed} cooked on its own: {sources:?}"
        );
        assert!(
            !report.ignored.iter().any(|i| i == consumed),
            "{consumed} ignored"
        );
    }
    // The crate is 2 m long and 0.5 m high, standing on y = 0.
    let props = bundle.load_model("models/props.fbx").unwrap();
    assert!(
        (props.min.y).abs() < 1e-3 && (props.max.y - 0.8).abs() < 1e-3,
        "{:?} {:?}",
        props.min,
        props.max
    );
    assert!((props.min.x + 1.0).abs() < 1e-3, "{:?}", props.min);
    let board = bundle.load_model("models/board.fbx").unwrap();
    assert!((board.max.x - 2.0).abs() < 1e-4 && (board.min.z + 1.0).abs() < 1e-4);

    std::fs::write(
        root.join("assets/models/quad.mtl"),
        "newmtl tile\nKd 1 0 0\nmap_Kd tile.png\n",
    )
    .unwrap();
    let again = cook(&root);
    let cooked: Vec<&str> = again
        .assets
        .iter()
        .filter(|a| a.outputs.iter().any(|o| o.cooked))
        .map(|a| a.source.as_str())
        .collect();
    assert_eq!(cooked, ["models/quad.obj"]);
}

fn anim(b: &Bundle, id: &str) -> CookedAnimation {
    b.load_animation(&format!("{id}#anim")).unwrap()
}

/// The skinned vertices of `m` posed by clip `clip` at `t`: each cooked
/// vertex with its morph deltas at the clip's weights, moved by its
/// joints' palette.
fn posed(m: &CookedModel, a: &CookedAnimation, clip: &str, t: f32) -> Vec<glam::Vec3> {
    let clip = a.clip(clip).unwrap();
    let s = &a.skin_streams[0];
    let morph = a.morphs.iter().find(|mt| mt.primitive == s.primitive);
    let mut pose = Pose::rest(&a.skeleton);
    sample(
        &a.skeleton,
        clip,
        &bind(&a.skeleton, clip),
        t,
        morph.map_or(u32::MAX, |mt| mt.mesh as u32),
        &mut pose,
    );
    let mut palette = Vec::new();
    a.skeleton.palette(0, &pose.nodes, &mut palette);
    m.primitives[s.primitive]
        .vertices
        .iter()
        .enumerate()
        .map(|(v, vert)| {
            let mut p = glam::Vec3::from(vert.position);
            if let Some(mt) = morph {
                for (k, d) in mt.positions.iter().enumerate() {
                    p += d[v] * pose.weights[k];
                }
            }
            (0..4)
                .map(|k| palette[s.joints[v][k] as usize].transform_point3(p) * s.weights[v][k])
                .sum()
        })
        .collect()
}

/// The farthest any point of `a` lies from its nearest point of `b`.
fn reach(a: &[glam::Vec3], b: &[glam::Vec3]) -> f32 {
    a.iter()
        .map(|p| b.iter().map(|q| p.distance(*q)).fold(f32::MAX, f32::min))
        .fold(0.0, f32::max)
}

/// Blender's FBX and its glTF of the same arm cook to the same posed
/// surface: at the rest pose and through both clips (the wave's bend and
/// rising bulge, the lift), every skinned vertex of one lies on the
/// other's within a millimetre, and both play the same clips for the same
/// lengths.
#[test]
fn an_fbx_plays_as_the_gltf_of_the_same_scene() {
    let root = project("parity");
    cook(&root);
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let (fm, fa) = (
        bundle.load_model("models/arm.fbx").unwrap(),
        anim(&bundle, "models/arm.fbx"),
    );
    let (gm, ga) = (
        bundle.load_model("models/arm.glb").unwrap(),
        anim(&bundle, "models/arm.glb"),
    );
    let clips = |a: &CookedAnimation| {
        let mut c: Vec<(String, i32)> = a
            .clips
            .iter()
            .map(|c| (c.name.clone(), (c.duration * 1000.0).round() as i32))
            .collect();
        c.sort();
        c
    };
    assert_eq!(clips(&fa), clips(&ga));
    assert_eq!(
        clips(&fa),
        [("lift".to_string(), 500), ("wave".to_string(), 1000)]
    );
    for (clip, t) in [
        ("wave", 0.0),
        ("wave", 0.5),
        ("wave", 1.0),
        ("lift", 0.25),
        ("lift", 0.5),
    ] {
        let f = posed(&fm, &fa, clip, t);
        let g = posed(&gm, &ga, clip, t);
        let d = reach(&f, &g).max(reach(&g, &f));
        assert!(d < 1e-3, "{clip} at {t}: the FBX is {d} m off the glTF");
    }
    // The wave bends the elbow: the tip, straight up at rest, swings a
    // quarter turn about X to point along Z at the elbow's height.
    let rest = posed(&fm, &fa, "wave", 0.0);
    let tip = (0..rest.len())
        .max_by(|&a, &b| rest[a].y.total_cmp(&rest[b].y))
        .unwrap();
    assert!((rest[tip].y - 2.0).abs() < 1e-3, "{}", rest[tip]);
    let waved = posed(&fm, &fa, "wave", 1.0)[tip];
    assert!(
        (waved.y - 1.1).abs() < 0.2 && waved.z.abs() > 0.7,
        "the tip at {waved}"
    );
}
