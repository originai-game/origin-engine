//! The public sun answer uses the same rotation and strength as the sky.
use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_destroy, og_game_game,
    og_host_begin_bundle, og_load_environment, og_load_environment_sun, set_hooks,
};
use serde_json::json;
use std::rc::Rc;

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

#[test]
fn sunlight_answers_world_direction_and_directional_light_units() {
    let root = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("environment-sun-{}", std::process::id()));
    std::fs::create_dir_all(root.join("assets")).unwrap();
    image::RgbImage::from_fn(256, 128, |x, y| {
        let (dx, dy) = (x as f32 - 154.0, y as f32 - 34.0);
        image::Rgb(if dx * dx + dy * dy < 4.0 {
            [255, 255, 255]
        } else {
            [70, 110, 160]
        })
    })
    .save(root.join("assets/sky.png"))
    .unwrap();
    std::fs::write(root.join("assets/sky.ogt"),
        "type environment\nversion 1\n---\nsource \"sky.png\"\nsize 16\nsamples 16\nrotation 30\nintensity 2\n").unwrap();
    let report = og_cook::cook_project(&root, &Default::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let cooked = og_cook::runtime::Bundle::open(&root.join("cooked"))
        .unwrap()
        .load_environment("sky.ogt")
        .unwrap();
    assert!(cooked.sun.intensity > 0.0);
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some("{}".into()));
    let answer: serde_json::Value =
        serde_json::from_str(&og_engine_create(game, json!({"root": root}).to_string())).unwrap();
    let world = answer["value"].as_u64().unwrap() as u32;
    let map = og_load_environment(world, "sky.ogt".into())
        .unwrap()
        .as_f64()
        .unwrap();
    let sun = og_load_environment_sun(world, map, std::f32::consts::FRAC_PI_2, 0.3).unwrap();
    let [x, y, z] = cooked.sun.direction;
    // A +90° turn sends +X to -Z, not +Z; cooked rotation is already baked.
    for (key, expected) in [("x", z), ("y", y), ("z", -x)] {
        assert!((sun["direction"][key].as_f64().unwrap() - expected as f64).abs() < 1e-5);
    }
    assert!(
        (sun["intensity"].as_f64().unwrap() - (cooked.sun.intensity * 0.3) as f64).abs() < 1e-5
    );
    assert_eq!(sun["angularSize"], json!(cooked.sun.angular_size));
    assert_eq!(
        og_load_environment_sun(world, map, 0.0, 0.0).unwrap()["intensity"],
        0.0
    );
    og_engine_destroy(world);
    std::fs::remove_dir_all(root).unwrap();
}
