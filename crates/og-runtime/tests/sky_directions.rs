//! Where the sky's sun stands, as the renderer is handed it: og.DayNight's
//! azimuth is a heading (about +Y from +Z towards +X), so 90 rises in +X;
//! og.Sky's sun follows the DirectionalLight unless a direction is given.

use og_core::glam::Vec3;
use og_render::{Background, RenderScene, RgbaImage};
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame,
    og_engine_write_resource, og_game_game, og_host_begin_bundle, og_world_insert, og_world_spawn,
    set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::cell::Cell;
use std::rc::Rc;

struct NoAssets;

impl Hooks for NoAssets {
    fn dispatch(&self, _call: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        Fetch::Failed(format!("{path}: no assets in this test"))
    }
}

/// A renderer that keeps where the last frame's background has its sun.
struct Recorder(Rc<Cell<Option<Vec3>>>);

impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "test".into()
    }
    fn size(&self) -> (u32, u32) {
        (64, 64)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        self.0.set(match scene.lighting.environment.background {
            Background::Procedural { sun_direction, .. } => Some(sun_direction),
            _ => None,
        });
        Ok(())
    }
    fn capture(&mut self, _: &RenderScene, _: u32, _: u32) -> Option<RgbaImage> {
        None
    }
}

fn answer(text: String) -> Value {
    serde_json::from_str(&text).expect("wire JSON")
}

/// A world with a recording renderer.
fn engine() -> (u32, Rc<Cell<Option<Vec3>>>) {
    set_hooks(Rc::new(NoAssets));
    og_host_begin_bundle();
    let game = og_game_game(Some("{}".into()));
    let engine = answer(og_engine_create(game, "{}".into()))["value"]
        .as_u64()
        .unwrap() as u32;
    let seen = Rc::new(Cell::new(None));
    set_render_backend(engine, Some(Box::new(Recorder(seen.clone())))).unwrap();
    (engine, seen)
}

fn write(engine: u32, resource: &str, value: Value) {
    let a = answer(og_engine_write_resource(
        engine,
        resource.into(),
        value.to_string(),
    ));
    assert_eq!(a["ok"], json!(true), "{a}");
}

fn sun(engine: u32, seen: &Cell<Option<Vec3>>) -> Vec3 {
    let a = answer(og_engine_frame(engine, 1000.0 / 60.0));
    assert_eq!(a["ok"], json!(true), "{a}");
    seen.get().expect("a procedural sky with a sun")
}

#[test]
fn a_day_night_azimuth_of_90_puts_the_rising_sun_towards_positive_x() {
    let (engine, seen) = engine();
    // Sunrise, the clock held there. The default azimuth is 90: the sun
    // rises in +X, where it rose when the default was 0 from +X.
    write(
        engine,
        "og.DayNight",
        json!({ "enabled": true, "start": 6, "speed": 0 }),
    );
    let rising = sun(engine, &seen);
    assert!((rising - Vec3::X).length() < 1e-3, "the default: {rising}");
    let dawn = |azimuth: f32| json!({ "azimuth": azimuth });
    // A heading: 0 is +Z, and it turns from +Z towards +X.
    write(engine, "og.DayNight", dawn(0.0));
    let rising = sun(engine, &seen);
    assert!((rising - Vec3::Z).length() < 1e-3, "azimuth 0: {rising}");
    write(engine, "og.DayNight", dawn(45.0));
    let rising = sun(engine, &seen);
    assert!(rising.x > 0.7 && rising.z > 0.7, "azimuth 45: {rising}");
    write(engine, "og.DayNight", dawn(90.0));
    let rising = sun(engine, &seen);
    assert!((rising - Vec3::X).length() < 1e-3, "azimuth 90: {rising}");
}

#[test]
fn a_sky_with_no_sun_direction_draws_its_sun_where_the_light_comes_from() {
    let (engine, seen) = engine();
    // No DirectionalLight yet: the frame is lit by the built-in sun, and
    // the sky's sun stands there.
    write(engine, "og.Sky", json!({ "enabled": true }));
    let built_in = og_render::DirectionalLight::default()
        .direction_to_light
        .normalize();
    let drawn = sun(engine, &seen);
    assert!((drawn - built_in).length() < 1e-4, "no light: {drawn}");

    // A light standing at `from` and aimed at the origin shines along
    // -from: the direction towards it is `from`.
    let from = Vec3::new(-3.0, 2.0, 5.0);
    let aim = og_core::facing::camera_look(from, Vec3::ZERO, Vec3::ZERO);
    let light = answer(og_runtime::host_impl::answer(og_world_spawn(
        engine,
        Some("{}".into()),
    )))["value"]
        .as_f64()
        .unwrap();
    for (name, fields) in [
        (
            "og.Transform",
            json!({ "quaternion": { "x": aim.x, "y": aim.y, "z": aim.z, "w": aim.w } }),
        ),
        ("og.DirectionalLight", json!({ "intensity": 2 })),
    ] {
        let a = answer(og_runtime::host_impl::answer(og_world_insert(
            engine,
            light,
            name.into(),
            Some(fields.to_string()),
        )));
        assert_eq!(a["ok"], json!(true), "{a}");
    }
    let drawn = sun(engine, &seen);
    assert!(
        (drawn - from.normalize()).length() < 1e-4,
        "towards the light {}: {drawn}",
        from.normalize()
    );
    assert!(
        (drawn - built_in).length() > 0.5,
        "the light points elsewhere than the built-in sun"
    );

    // A direction given is used as given, whatever the light does.
    write(
        engine,
        "og.Sky",
        json!({ "sunDirection": { "x": 0, "y": 0, "z": -2 } }),
    );
    let drawn = sun(engine, &seen);
    assert!((drawn - Vec3::NEG_Z).length() < 1e-4, "as given: {drawn}");
    // Zero again: back with the light.
    write(
        engine,
        "og.Sky",
        json!({ "sunDirection": { "x": 0, "y": 0, "z": 0 } }),
    );
    let drawn = sun(engine, &seen);
    assert!((drawn - from.normalize()).length() < 1e-4, "{drawn}");
}
