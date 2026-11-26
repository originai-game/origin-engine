//! A cooked rigid-node animation must reach the host's submitted instances,
//! not only the simulation pose queried by og.anim.
use og_render::{RenderScene, RgbaImage};
use og_runtime::*;
use serde_json::{Value, json};
use std::{cell::RefCell, rc::Rc};

struct Assets;
impl Hooks for Assets {
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

struct Recorder(Rc<RefCell<Vec<og_core::glam::Mat4>>>);
impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (320, 180)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        *self.0.borrow_mut() = scene
            .objects
            .instances
            .iter()
            .map(|i| i.transform)
            .collect();
        Ok(())
    }
    fn capture(&mut self, _: &RenderScene, _: u32, _: u32) -> Option<RgbaImage> {
        None
    }
}

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["ok"], true, "{text}");
    v["value"].clone()
}

#[test]
fn cooked_rigid_node_keys_move_the_drawn_model() {
    let root = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("rigid-node-animation-{}", std::process::id()));
    std::fs::create_dir_all(root.join("assets")).unwrap();
    std::fs::write(
        root.join("assets/panel.ogt"),
        "type model\nversion 1\n---\nconform off\npivot \"root\" {\nposition 1 3 0\n}\npart \"panel\" {\npivot \"root\"\nboxGeometry {\nwidth 1\nheight 2\ndepth 0.5\n}\n}\n",
    ).unwrap();
    std::fs::write(
        root.join("assets/panel.ogt.ogt"),
        "type import\nversion 1\n---\nanimation {\nclip \"open\" {\nchannel \"panel\" translation {\nkey 0 0 0 0\nkey 0.5 2 1 0\n}\nchannel \"panel\" rotation {\nkey 0 0 0 0 1\nkey 0.5 0 0 0.70710678 0.70710678\n}\n}\n}\n",
    ).unwrap();
    let cooked = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
    assert!(cooked.ok(), "{:?}", cooked.errors());
    set_hooks(Rc::new(Assets));
    og_host_begin_bundle();
    let game = og_game_game(Some(json!({"seed": 1}).to_string()));
    let engine = value(og_engine_create(game, json!({"root": root}).to_string()))
        .as_u64()
        .unwrap() as u32;
    let drawn = Rc::new(RefCell::new(Vec::new()));
    set_render_backend(engine, Some(Box::new(Recorder(drawn.clone())))).unwrap();
    let model = og_load_gltf(engine, "panel.ogt".into(), None).unwrap();
    let animation = og_load_animation(engine, "panel.ogt".into()).unwrap();
    let entity = og_world_spawn(
        engine,
        Some(json!({"position":{"x":5,"y":2,"z":0}}).to_string()),
    )
    .unwrap()
    .as_f64()
    .unwrap();
    for (component, fields) in [
        ("og.Model", json!({"asset": model})),
        ("og.Skeleton", json!({"asset": animation["skeleton"]})),
        (
            "og.AnimationPlayer",
            json!({"clip": animation["clips"][0]["clip"], "loop":false,"fade":0}),
        ),
    ] {
        og_world_insert(engine, entity, component.into(), Some(fields.to_string())).unwrap();
    }
    value(og_engine_advance(engine, 35));
    value(og_engine_frame(engine, 0.0));
    let matrices = drawn.borrow();
    assert_eq!(matrices.len(), 1);
    let m = matrices[0];
    assert!(
        (m.w_axis.x - 8.0).abs() < 0.001 && (m.w_axis.y - 6.0).abs() < 0.001,
        "{m:?}"
    );
    assert!(
        m.x_axis.x.abs() < 0.001 && (m.x_axis.y - 1.0).abs() < 0.001,
        "{m:?}"
    );
    assert!(og_engine_destroy(engine));
    std::fs::remove_dir_all(root).unwrap();
}

/// The animation/physics boundary: desired root travel must be reported,
/// swept, consumed once, and replayed exactly after a restore.
#[test]
fn controller_root_motion_stops_at_a_wall_and_restores() {
    use og_anim::{
        AnimationPlayer, Channel, ChannelPath, ClipAsset, NONE, ROOT_MOTION_PLANAR, Skeleton,
        SkeletonAsset, SkeletonNode, Trs,
    };
    use og_core::glam::Vec3;
    use og_ecs::transform::Transform;
    use og_ecs::{World, WorldDesc};
    use og_physics::{CharacterController, Collider, Physics};
    let mut w = World::new(&WorldDesc {
        tick_rate: 60,
        seed: 8,
    })
    .unwrap();
    let anim = og_anim::install(&mut w).unwrap();
    let physics = og_physics::install(
        &mut w,
        Physics {
            gravity: Vec3::ZERO,
            ..Default::default()
        },
    )
    .unwrap();
    connect_animation_physics(&mut w, &anim, physics).unwrap();
    anim.add_skeleton(
        1,
        SkeletonAsset {
            nodes: vec![SkeletonNode {
                name: "root".into(),
                parent: NONE,
                rest: Trs::default(),
                mesh: NONE,
            }],
            ..Default::default()
        },
    )
    .unwrap();
    anim.add_clip(
        2,
        ClipAsset {
            name: "move".into(),
            duration: 1.0,
            rate: 1.0,
            source_rate: 1.0,
            frames: 2,
            channels: vec![Channel {
                node: 0,
                node_name: "root".into(),
                path: ChannelPath::Translation,
                step: false,
                lanes: 3,
                samples: vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.2].into(),
            }],
            events: vec![],
        },
    )
    .unwrap();
    let wall = w
        .spawn((
            Transform::at(Vec3::new(0.0, 1.0, 1.5)),
            Collider::cuboid(Vec3::new(4.0, 4.0, 0.2)),
        ))
        .unwrap();
    let actor = w
        .spawn((
            Transform::default(),
            Skeleton {
                asset: 1,
                root_motion: ROOT_MOTION_PLANAR,
                root_motion_mode: 1,
                ..Default::default()
            },
            AnimationPlayer::new(2),
            Collider::capsule(0.3, 0.8),
            CharacterController {
                gravity_scale: 0.0,
                ..Default::default()
            },
        ))
        .unwrap();
    let mut requested = 0.0;
    for _ in 0..120 {
        w.step().unwrap();
        requested += w.get::<Skeleton>(actor).unwrap().root_delta.z;
        assert_eq!(
            w.get::<CharacterController>(actor).unwrap().root_delta,
            Vec3::ZERO
        );
    }
    let stopped = w.get::<Transform>(actor).unwrap().position.z;
    assert!(
        requested > 2.0 && stopped > 0.9 && stopped < 1.12,
        "requested {requested}, swept to {stopped}"
    );
    let checkpoint = w.snapshot();
    for _ in 0..30 {
        w.step().unwrap();
    }
    let end = w.snapshot();
    w.restore(&checkpoint).unwrap();
    for _ in 0..30 {
        w.step().unwrap();
    }
    assert_eq!(w.snapshot(), end);
    w.despawn(wall).unwrap();
    for _ in 0..60 {
        w.step().unwrap();
    }
    assert!(
        w.get::<Transform>(actor).unwrap().position.z > stopped + 0.8,
        "motion resumes after the obstruction is removed"
    );
}
