//! og.Weather switches with a ramp, not a cut: a game that writes a new
//! kind sees the old weather thin out as the new one builds over
//! og.Weather.change, its fog and particles with it, and a game that starts
//! in rain draws rain from the first frame.
//!
//! Ways this could fail: the frame keeps drawing the game's `kind` (the
//! state before: the fog, the sky and the particles swapped in one frame),
//! the two weathers share one emitter (the rain's drops in the air vanish
//! at the switch), or the drawn weather starts from clear and builds even
//! when nothing was switched (every rainy scene opens dry).

use og_render::RenderScene;
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame,
    og_engine_write_resource, og_game_game, og_host_begin_bundle, set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::cell::RefCell;
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

/// What the last frame drew of the weather: its fog's density, how many
/// particle systems fell, and the ground under it (og.WeatherState's).
#[derive(Clone, Copy, Debug, Default)]
struct Drawn {
    fog: f32,
    systems: usize,
    puddles: f32,
    snow_depth: f32,
}

struct Recorder(Rc<RefCell<Drawn>>);

impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "test".into()
    }
    fn size(&self) -> (u32, u32) {
        (64, 64)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        let ground = scene.atmosphere.surface_weather.unwrap_or_default();
        *self.0.borrow_mut() = Drawn {
            fog: scene.atmosphere.fog.map_or(0.0, |f| f.density),
            systems: scene.effects.particles.len(),
            puddles: ground.puddles,
            snow_depth: ground.snow_depth,
        };
        Ok(())
    }
    fn capture(&mut self, _: &RenderScene, _: u32, _: u32) -> Option<og_render::RgbaImage> {
        None
    }
}

/// Rain (fog density 0.03) switched to snow (0.018) at og.Weather's
/// default 20 s: the first frame is the rain; 10 s on, the fog is half of
/// each and both kinds' particles fall; 30 s on, the snow alone. The ground
/// follows what falls: a minute of rain soaks it and starts the puddles,
/// which go on filling while the rain thins out (by 0.0625 in the first
/// 10 s, at puddleTime's 120 s), and the snow lies as it builds, 10
/// seconds' worth over the ramp and then 1 cm a minute: 4.8 cm five
/// minutes after the switch.
#[test]
fn a_switch_of_weather_ramps_from_the_first_kind_to_the_next() {
    set_hooks(Rc::new(NoAssets));
    og_host_begin_bundle();
    let game = og_game_game(Some("{}".into()));
    let engine = answer(og_engine_create(game, "{}".into()))["value"]
        .as_u64()
        .unwrap() as u32;
    let seen = Rc::new(RefCell::new(Drawn::default()));
    set_render_backend(engine, Some(Box::new(Recorder(seen.clone())))).unwrap();
    let frames = |n: u32| {
        for _ in 0..n {
            answer(og_engine_frame(engine, 1000.0 / 60.0));
        }
        *seen.borrow()
    };
    write(engine, "og.Weather", json!({ "kind": 1, "intensity": 1 }));
    write(engine, "og.WeatherState", json!({}));
    let rain = frames(1);
    assert!(
        (rain.fog - 0.03).abs() < 1e-5 && rain.systems == 1,
        "{rain:?}"
    );
    let soaked = frames(3599);
    write(engine, "og.Weather", json!({ "kind": 2 }));
    let half = frames(600);
    assert!(
        (half.fog - 0.024).abs() < 2e-4 && half.systems == 2,
        "half way: {half:?}"
    );
    assert!(
        (half.puddles - soaked.puddles - 0.0625).abs() < 2e-3 && half.snow_depth > 0.0,
        "the rain still falling fills the puddles and the snow starts to lie: {soaked:?} {half:?}"
    );
    let snow = frames(1200);
    assert!(
        (snow.fog - 0.018).abs() < 1e-5 && snow.systems == 1,
        "{snow:?}"
    );
    let lying = frames(16200);
    assert!(
        (lying.snow_depth - 0.0483).abs() < 1e-3,
        "five minutes after the switch: {lying:?}"
    );
    println!("the switch: soaked {soaked:?}, half way {half:?}, five minutes on {lying:?}");
}

fn answer(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v
}

fn write(engine: u32, resource: &str, value: Value) {
    answer(og_engine_write_resource(
        engine,
        resource.into(),
        value.to_string(),
    ));
}
