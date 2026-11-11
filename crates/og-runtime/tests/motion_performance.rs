//! Authored performance works during model startup, on the real cooked assets,
//! without a GPU. The CC0 strider and clip library are the fx-showcase fixtures.
use og_runtime::*;
use serde_json::{Value, json};
use std::{cell::RefCell, path::PathBuf, rc::Rc};

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["ok"], true, "{text}");
    v["value"].clone()
}

struct Disk;
impl Hooks for Disk {
    fn dispatch(&self, _: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        match std::fs::read(path) {
            Ok(bytes) => Fetch::Ready(bytes),
            Err(e) => Fetch::Failed(e.to_string()),
        }
    }
}

type Matrices = Vec<og_core::glam::Mat4>;
struct Recorder(Rc<RefCell<Matrices>>);
impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (320, 180)
    }
    fn render(&mut self, scene: &og_render::RenderScene) -> Result<(), String> {
        if let Some(p) = scene.objects.joint_palettes.first() {
            *self.0.borrow_mut() = p.matrices.clone();
        }
        // The garment must take the same procedural palette as the body,
        // without pre-evaluating its points on the CPU during extraction.
        assert!(
            !scene.objects.deformed_vertices.is_empty(),
            "motion agent's garment was lost"
        );
        assert_eq!(
            scene.objects.previous_deformed_vertices.len(),
            scene.objects.deformed_vertices.len(),
            "motion blur without TAA must retain garment history"
        );
        assert_eq!(
            scene.objects.previous_joint_palettes.len(),
            scene.objects.joint_palettes.len()
        );
        for instance in &scene.objects.skinned_instances {
            let Some(d) = instance.deformed else { continue };
            let draw = &scene.objects.deformed_vertices[d as usize];
            let garment = draw.garment.as_ref().expect("deferred garment pose");
            assert!(draw.positions.is_empty() && draw.normals.is_empty());
            assert_eq!(
                *garment.palette,
                scene.objects.joint_palettes[instance.palette as usize].matrices
            );
            // Pinned cloth has zero offsets: the fallback must reproduce
            // the body's normalized skin blend, including procedural turns.
            let vertices = draw.evaluated();
            for (p, &(point, _)) in vertices.positions.iter().zip(&garment.topology.vertices) {
                let i = point as usize;
                let (joints, weights) = garment.topology.skin[i];
                let total: f32 = weights.iter().sum();
                let rest = garment.topology.rest[i];
                let mut want = rest;
                if total >= 1e-6 {
                    want = og_core::glam::Vec3::ZERO;
                    for (j, w) in joints.into_iter().zip(weights) {
                        want += garment.palette[j as usize].transform_point3(rest) * (w / total);
                    }
                }
                assert!(
                    (*p - want).length() < 1e-4,
                    "garment did not follow the motion palette"
                );
            }
        }
        Ok(())
    }
    fn capture(
        &mut self,
        _: &og_render::RenderScene,
        _: u32,
        _: u32,
    ) -> Option<og_render::RgbaImage> {
        None
    }
}

fn frames(w: u32, n: usize) {
    for _ in 0..n {
        value(og_engine_frame(w, 1000.0 / 60.0));
    }
}

#[test]
fn cooked_action_and_look_compose_release_and_replay_on_tick_time() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("motion-performance-{}", std::process::id()));
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/motion/assets");
    for path in ["models/strider.glb", "anims/humanoid-clips.glb"] {
        let to = root.join("assets").join(path);
        std::fs::create_dir_all(to.parent().unwrap()).unwrap();
        std::fs::copy(source.join(path), to).unwrap();
    }
    // A licensed walk exercises the authored-clip path; it is not a brace.
    std::fs::write(root.join("assets/action.ogt"),
        "type motion-action\nversion 1\n---\nsource \"anims/humanoid-clips.glb\"\nclip \"walk\"\nholdStart 0.2\nholdEnd 0.6\nhand right\nhandTarget 0.12 0 0\n").unwrap();
    let report = og_cook::cook_project(
        &root,
        &og_cook::CookOptions {
            threads: 1,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some("{\"seed\":1}".into()));
    let w = value(og_engine_create(game, json!({"root":root}).to_string()))
        .as_u64()
        .unwrap() as u32;
    let anim = value(og_load_animation(w, "models/strider.glb".into()));
    let model = value(og_load_gltf(w, "models/strider.glb".into(), None));
    let e = value(og_world_instantiate(w, model.as_f64().unwrap(), None))
        .as_f64()
        .unwrap();
    for (name, desc) in [
        ("og.Skeleton", json!({"asset":anim["skeleton"]})),
        ("og.MotionAgent", json!({"drive":true})),
        (
            "og.Garment",
            json!({"mesh":0,"maxDistance":0,"gravityScale":0}),
        ),
    ] {
        value(og_world_insert(w, e, name.into(), Some(desc.to_string())));
    }
    value(og_engine_write_resource(
        w,
        "og.PostEffects".into(),
        json!({"motionBlur":true}).to_string(),
    ));
    let drawn = Rc::new(RefCell::new(Vec::new()));
    set_render_backend(w, Some(Box::new(Recorder(drawn.clone())))).unwrap();
    frames(w, 1);
    let rest = drawn.borrow().clone();
    assert!(!rest.is_empty());
    value(og_motion_look_at(
        w,
        e,
        Some(json!({"target":{"x":3,"y":1.6,"z":-1},"duration":2,"blend":0.1}).to_string()),
    ));
    frames(w, 15);
    let looked = drawn.borrow().clone();
    assert_ne!(
        rest, looked,
        "look must change the rendered palette, even without a model"
    );
    // The fixture's skin starts hips, spine, chest, neck, head. Hips stay
    // locomotion-owned while the upper body turns.
    assert_eq!(rest[0], looked[0]);
    assert_ne!(rest[4], looked[4]);
    for joint in 13..21 {
        assert_eq!(rest[joint], looked[joint], "look moved a leg joint");
    }
    value(og_motion_react(
        w,
        e,
        Some(json!({"action":"action.ogt","direction":{"x":1,"y":0,"z":0},"duration":0.8,"blend":0.1}).to_string()),
    ));
    frames(w, 20);
    assert_eq!(value(og_motion_status(w, e))["reacting"], true);
    assert_ne!(
        *drawn.borrow(),
        looked,
        "authored motion must reach the palette"
    );
    // The fixture's bundle declares no model: og.anim's pose is drawn and the
    // agent is never ready (failed with that reason, on the CPU path that
    // needs no GPU to try).
    let presentation = value(og_motion_presentation(w, e));
    assert_ne!(
        presentation["state"], "ready",
        "an authored fallback palette is not generated readiness"
    );
    // Reconstruct the drawn joint transforms from the fixture's inverse binds.
    // This catches a contact composed before lookAt, or metre offsets wrongly
    // multiplied by the centimetre rig's 0.01 root scale.
    let glb = std::fs::read(source.join("models/strider.glb")).unwrap();
    let n = u32::from_le_bytes(glb[12..16].try_into().unwrap()) as usize;
    let gltf: Value = serde_json::from_slice(&glb[20..20 + n]).unwrap();
    let accessor =
        &gltf["accessors"][gltf["skins"][0]["inverseBindMatrices"].as_u64().unwrap() as usize];
    let view = &gltf["bufferViews"][accessor["bufferView"].as_u64().unwrap() as usize];
    let offset = 28
        + n
        + view["byteOffset"].as_u64().unwrap_or(0) as usize
        + accessor["byteOffset"].as_u64().unwrap_or(0) as usize;
    let joint = |i: usize| {
        let values = std::array::from_fn(|k| {
            f32::from_le_bytes(
                glb[offset + i * 64 + k * 4..offset + i * 64 + k * 4 + 4]
                    .try_into()
                    .unwrap(),
            )
        });
        drawn.borrow()[i] * og_core::glam::Mat4::from_cols_array(&values).inverse()
    };
    let head = joint(4);
    let facing = joint(0).to_scale_rotation_translation().1 * og_core::glam::Vec3::Z;
    assert!(
        facing.x > 0.9,
        "reaction must face its world direction: {facing}"
    );
    let hand = joint(12).w_axis.truncate();
    let head_rotation = head.to_scale_rotation_translation().1;
    let wanted = head.w_axis.truncate()
        + head_rotation * og_core::glam::Vec3::new(0.12 * 0.60 / 0.54, 0.0, 0.0);
    assert!(
        hand.distance(wanted) < 0.025,
        "hood contact: hand {hand}, target {wanted}"
    );
    let saved = value(og_world_snapshot(w, None))
        .as_str()
        .unwrap()
        .to_string();
    frames(w, 12);
    let expected = drawn.borrow().clone();
    let expected_status = value(og_motion_status(w, e));
    value(og_engine_frame(w, 0.0));
    assert_eq!(
        *drawn.borrow(),
        expected,
        "drawing without a tick advanced the action"
    );
    assert_eq!(value(og_motion_status(w, e)), expected_status);
    value(og_world_restore(w, saved, None));
    frames(w, 12);
    assert_eq!(*drawn.borrow(), expected, "restored performance diverged");
    assert_eq!(value(og_motion_status(w, e)), expected_status);
    value(og_motion_look_at(w, e, Some("{}".into())));
    value(og_motion_react(w, e, Some("{}".into())));
    frames(w, 20);
    let status = value(og_motion_status(w, e));
    assert_eq!(status["looking"], false);
    assert_eq!(status["reacting"], false);
    assert_eq!(
        *drawn.borrow(),
        rest,
        "release must return to the base animation"
    );
    let bad: Value = serde_json::from_str(&og_motion_react(
        w,
        e,
        Some(json!({"action":"action.ogt","strength":2}).to_string()),
    ))
    .unwrap();
    assert_eq!(bad["ok"], false);
    assert_eq!(value(og_motion_status(w, e))["reacting"], false);
    value(og_motion_look_at(
        w,
        e,
        Some(
            json!({
                "target":{"x":-2,"y":2,"z":1}, "duration":0.1, "blend":0.05
            })
            .to_string(),
        ),
    ));
    value(og_motion_react(
        w,
        e,
        Some(
            json!({
                "action":"action.ogt", "duration":0.1, "blend":0.05
            })
            .to_string(),
        ),
    ));
    frames(w, 90);
    let status = value(og_motion_status(w, e));
    assert_eq!(status["looking"], false, "look did not expire");
    assert_eq!(
        status["reacting"], false,
        "reaction did not finish its exit"
    );
    assert_eq!(*drawn.borrow(), rest);
}
