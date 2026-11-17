//! og.Season through the runtime: the date and the latitude put the sun
//! where the renderer is handed it, og.atmosphere.season answers the same
//! date, and a jump of the date plays over og.Season.change.

use og_core::glam::Vec3;
use og_render::{Background, RenderScene, RgbaImage};
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_atmosphere_season, og_engine_create, og_engine_frame,
    og_engine_write_resource, og_game_game, og_host_begin_bundle, set_hooks, set_render_backend,
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

/// One tick and its frame: where the sky's sun is drawn.
fn sun(engine: u32, seen: &Cell<Option<Vec3>>) -> Vec3 {
    let a = answer(og_engine_frame(engine, 1000.0 / 60.0));
    assert_eq!(a["ok"], json!(true), "{a}");
    seen.get().expect("a procedural sky with a sun")
}

fn elevation(v: Vec3) -> f32 {
    og_core::math::atan2(v.y, (v.x * v.x + v.z * v.z).sqrt()).to_degrees()
}

/// At 47° north the noon sun stands 66.4° up at midsummer and 19.6° at
/// midwinter, towards the heading azimuth + 90 (-Z by default); it rises
/// north of east in summer and south of east in winter, at the hours
/// og.atmosphere.season gives; south of the equator the June noon sun is
/// low in the north; `sun: false` keeps og.DayNight's own path.
#[test]
fn the_date_and_the_latitude_set_the_suns_path() {
    let (engine, seen) = engine();
    write(
        engine,
        "og.DayNight",
        json!({ "enabled": true, "start": 12, "speed": 0 }),
    );
    // No season yet: og.DayNight's path, 25° from overhead.
    let own = sun(engine, &seen);
    assert!((elevation(own) - 65.0).abs() < 0.01, "{own}");
    let fresh = og_atmosphere_season(engine);
    assert_eq!((fresh.day, fresh.season), (172.0, 1.0));

    write(
        engine,
        "og.Season",
        json!({ "enabled": true, "latitude": 47, "start": 172 }),
    );
    let summer = sun(engine, &seen);
    assert!((elevation(summer) - 66.4).abs() < 0.2, "{summer}");
    assert!(summer.x.abs() < 1e-4 && summer.z < 0.0, "{summer}");
    let info = og_atmosphere_season(engine);
    assert!((info.day_length - 15.7).abs() < 0.1, "{}", info.day_length);
    assert!((info.declination - 23.44).abs() < 0.1);
    assert_eq!(info.season, 1.0);
    // A June noon in the default climate: the season's 18.7 °C and most
    // of the day's swing above it.
    assert!(
        (info.temperature - 21.5).abs() < 0.2,
        "{}",
        info.temperature
    );

    // Sunrise: on the horizon, in the east (+X), to the north of it (+Z).
    write(engine, "og.DayNight", json!({ "start": info.sunrise }));
    let rising = sun(engine, &seen);
    assert!(rising.y.abs() < 2e-3, "{rising}");
    assert!(rising.x > 0.7 && rising.z > 0.4, "{rising}");

    write(engine, "og.Season", json!({ "start": 355 }));
    let info = og_atmosphere_season(engine);
    assert!((info.day_length - 8.3).abs() < 0.1, "{}", info.day_length);
    assert_eq!(info.season, 3.0);
    write(engine, "og.DayNight", json!({ "start": info.sunrise }));
    let rising = sun(engine, &seen);
    assert!(rising.y.abs() < 2e-3, "{rising}");
    assert!(rising.x > 0.7 && rising.z < -0.4, "{rising}");
    write(engine, "og.DayNight", json!({ "start": 12 }));
    let winter = sun(engine, &seen);
    assert!((elevation(winter) - 19.6).abs() < 0.2, "{winter}");

    // The same June day at 47° south: the noon sun low in the north.
    write(
        engine,
        "og.Season",
        json!({ "start": 172, "latitude": -47 }),
    );
    let south = sun(engine, &seen);
    assert!((elevation(south) - 19.6).abs() < 0.2, "{south}");
    assert!(south.z > 0.9, "{south}");
    assert_eq!(og_atmosphere_season(engine).season, 3.0);

    // The heading of due east turns the whole path: east in +Z, so north
    // (where this southern noon sun stands) is -X.
    write(engine, "og.DayNight", json!({ "azimuth": 0 }));
    let turned = sun(engine, &seen);
    assert!(turned.x < -0.9 && turned.z.abs() < 1e-4, "{turned}");

    write(engine, "og.DayNight", json!({ "azimuth": 90 }));
    write(engine, "og.Season", json!({ "sun": false }));
    let kept = sun(engine, &seen);
    assert!((kept - own).length() < 1e-5, "{kept} vs {own}");
}

/// A world starts on its date. A jump of a season then plays over
/// `change` seconds: the date read and the sun drawn run through the days
/// between, and never show the new date before they reach it.
#[test]
fn a_jump_of_the_date_plays_over_the_change() {
    let (engine, seen) = engine();
    write(
        engine,
        "og.DayNight",
        json!({ "enabled": true, "start": 12, "speed": 0 }),
    );
    write(
        engine,
        "og.Season",
        json!({ "enabled": true, "latitude": 47, "start": 265, "change": 6 }),
    );
    // Before the first tick the date is the calendar's.
    assert_eq!(og_atmosphere_season(engine).day, 265.0);
    let autumn = elevation(sun(engine, &seen));
    assert_eq!(og_atmosphere_season(engine).day, 265.0);

    write(engine, "og.Season", json!({ "start": 355 }));
    // The frame drawn before og.season next runs is still the old date's.
    assert_eq!(og_atmosphere_season(engine).day, 265.0);
    let mut last = autumn;
    for frame in 1..=180 {
        let e = elevation(sun(engine, &seen));
        assert!(
            e <= last + 1e-3 && last - e < 0.2,
            "frame {frame}: the noon sun sinks steadily, {last} to {e}"
        );
        last = e;
    }
    let half = og_atmosphere_season(engine);
    assert!((half.day - 310.6).abs() < 1.0, "{}", half.day);
    assert!(half.season > 2.4 && half.season < 2.6, "{}", half.season);
    for _ in 0..200 {
        sun(engine, &seen);
    }
    assert_eq!(og_atmosphere_season(engine).day, 355.0);
    assert!((elevation(sun(engine, &seen)) - 19.6).abs() < 0.2);

    // Without a change the date cuts, read at once.
    write(engine, "og.Season", json!({ "change": 0, "start": 79 }));
    assert_eq!(og_atmosphere_season(engine).day, 79.0);
    sun(engine, &seen);
    assert_eq!(og_atmosphere_season(engine).day, 79.0);
    // Turned off, the year keeps no date: it reads its start.
    write(
        engine,
        "og.Season",
        json!({ "enabled": false, "start": 10 }),
    );
    sun(engine, &seen);
    assert_eq!(og_atmosphere_season(engine).day, 10.0);
}
