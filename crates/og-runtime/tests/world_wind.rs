//! og.Wind is the world's wind: the one resource moves what the frame
//! draws in the air. Vegetation leans by its speed, the clouds drift on it
//! (faster aloft) and rain falls slanted by it, each adding its own
//! record's `wind`; without og.Wind the air is still.
//!
//! Ways this could fail: one of the three keeps reading only its own
//! `wind` (the state before: three winds a game had to keep in step), a
//! record's own `wind` replaces the world's instead of adding to it, or
//! still air leaves a default drift behind.
//!
//! The tick moves in the same air: an og.WindMotor carries a body with
//! RigidBody.airDrag along the motor entity's forward, and not a body
//! beyond its reach. Ways this could fail: the motors never reach physics
//! (the state before: they only leaned vegetation), the push ignores the
//! entity's rotation (the body drifts along -z), or the reach is not
//! honoured (the far body moves).
//!
//! Particles are in that air too: a system's emitter with `wind` is handed
//! the air where the system is (og.Wind and the motors in reach), and one
//! the tick simulates (it sends events) is carried by the tick's air. Ways
//! this could fail: the frame never fills ParticleSystem.air, the motors
//! are left out or reach a system beyond them, or the tick's simulation
//! keeps still air (its particles stop instead of drifting downwind).

use glam::Vec3;
use og_render::RenderScene;
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame,
    og_engine_write_resource, og_game_game, og_host_begin_bundle, og_load_emitter,
    og_system_define_system, og_world_get, og_world_insert, og_world_spawn, set_hooks,
    set_render_backend,
};
use serde_json::{Value, json};
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
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

/// What the last frame's air moved.
#[derive(Clone, Copy, Debug)]
struct Air {
    /// The vegetation's wind: where it blows and its lean per metre.
    lean: Option<(Vec3, f32)>,
    clouds: Vec3,
    /// Where the rain falls (unit).
    rain: Vec3,
}

struct Recorder(Rc<Cell<Option<Air>>>);

impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "test".into()
    }
    fn size(&self) -> (u32, u32) {
        (64, 64)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        let a = &scene.atmosphere;
        self.0.set(Some(Air {
            lean: a.wind.map(|w| (w.direction, w.strength)),
            clouds: a.clouds.expect("og.Clouds is on").wind,
            rain: a.surface_weather.as_ref().expect("og.WeatherState").fall,
        }));
        Ok(())
    }
    fn capture(&mut self, _: &RenderScene, _: u32, _: u32) -> Option<og_render::RgbaImage> {
        None
    }
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

fn close(a: Vec3, b: Vec3) -> bool {
    (a - b).length() < 1e-4
}

#[test]
fn the_worlds_wind_moves_vegetation_clouds_and_rain_together() {
    set_hooks(Rc::new(NoAssets));
    og_host_begin_bundle();
    let game = og_game_game(Some("{}".into()));
    let engine = answer(og_engine_create(game, "{}".into()))["value"]
        .as_u64()
        .unwrap() as u32;
    let seen = Rc::new(Cell::new(None));
    set_render_backend(engine, Some(Box::new(Recorder(seen.clone())))).unwrap();
    let frame = || {
        answer(og_engine_frame(engine, 1000.0 / 60.0));
        seen.get().expect("the renderer was handed a frame")
    };
    write(engine, "og.Clouds", json!({ "enabled": true }));
    write(engine, "og.Weather", json!({ "kind": 1, "intensity": 1 }));
    write(engine, "og.WeatherState", json!({}));

    // Still air: nothing leans, the clouds only billow, the rain falls
    // straight.
    let still = frame();
    assert_eq!(still.lean, None);
    assert!(close(still.clouds, Vec3::new(0.0, 0.4, 0.0)), "{still:?}");
    assert!(close(still.rain, Vec3::NEG_Y), "{still:?}");

    // 5 m/s to -z, given off the axis and unnormalised.
    write(
        engine,
        "og.Wind",
        json!({ "direction": { "x": 0, "y": 3, "z": -2 }, "speed": 5 }),
    );
    let windy = frame();
    let (direction, lean) = windy.lean.expect("vegetation sways");
    assert!(close(direction, Vec3::NEG_Z), "{windy:?}");
    assert!((lean - 0.25).abs() < 1e-5, "5 m/s leans 5 cm a metre each");
    assert!(close(windy.clouds, Vec3::new(0.0, 0.4, -10.0)), "{windy:?}");
    let slant = Vec3::new(0.0, -11.0, -5.0).normalize();
    assert!(close(windy.rain, slant), "{windy:?}");

    // Their own wind adds to the world's.
    write(
        engine,
        "og.Clouds",
        json!({ "enabled": true, "wind": { "x": 3, "y": 0, "z": 4 } }),
    );
    write(
        engine,
        "og.Weather",
        json!({ "kind": 1, "intensity": 1, "wind": { "x": 0, "y": 0, "z": 5 } }),
    );
    let own = frame();
    assert!(close(own.clouds, Vec3::new(3.0, 0.0, -6.0)), "{own:?}");
    assert!(close(own.rain, Vec3::NEG_Y), "{own:?}");
}

fn spawn(engine: u32, desc: Value, parts: &[(&str, Value)]) -> f64 {
    let id = answer(og_runtime::host_impl::answer(og_world_spawn(
        engine,
        Some(desc.to_string()),
    )))["value"]
        .as_f64()
        .unwrap();
    for (name, v) in parts {
        answer(og_runtime::host_impl::answer(og_world_insert(
            engine,
            id,
            (*name).into(),
            Some(v.to_string()),
        )));
    }
    id
}

fn position(engine: u32, id: f64) -> Vec3 {
    let t = answer(og_runtime::host_impl::answer(og_world_get(
        engine,
        id,
        "og.Transform".into(),
    )));
    let p = &t["value"]["position"];
    let f = |k: &str| p[k].as_f64().unwrap() as f32;
    Vec3::new(f("x"), f("y"), f("z"))
}

#[test]
fn a_wind_motor_carries_the_body_in_its_reach_along_its_forward() {
    set_hooks(Rc::new(NoAssets));
    og_host_begin_bundle();
    let game = og_game_game(Some("{}".into()));
    let engine = answer(og_engine_create(game, "{}".into()))["value"]
        .as_u64()
        .unwrap() as u32;
    // A weightless ball the air carries, 2 m in front of the fan and 50 m
    // to its side.
    let ball = |z: f32| {
        spawn(
            engine,
            json!({ "position": { "x": 0, "y": 3, "z": z } }),
            &[
                ("og.RigidBody", json!({ "gravityScale": 0, "airDrag": 2 })),
                ("og.Collider", json!({ "shape": "sphere", "radius": 0.25 })),
            ],
        )
    };
    let near = ball(0.0);
    let far = ball(50.0);
    // A fan turned a quarter turn about y, so its forward (-z) is -x.
    let turn = std::f32::consts::FRAC_1_SQRT_2;
    spawn(
        engine,
        json!({
            "position": { "x": 2, "y": 3, "z": 0 },
            "quaternion": { "x": 0, "y": turn, "z": 0, "w": turn },
        }),
        &[(
            "og.WindMotor",
            json!({ "kind": 1, "radius": 6, "strength": 10 }),
        )],
    );
    for _ in 0..60 {
        answer(og_engine_frame(engine, 1000.0 / 60.0));
    }
    let moved = position(engine, near) - Vec3::new(0.0, 3.0, 0.0);
    assert!(moved.x < -1.0, "carried along the fan's forward: {moved:?}");
    assert!(
        moved.y.abs() < 1e-3 && moved.z.abs() < 1e-3,
        "and nowhere else: {moved:?}"
    );
    let still = position(engine, far) - Vec3::new(0.0, 3.0, 50.0);
    assert!(still.length() < 1e-6, "beyond the reach: {still:?}");
}

struct Disk;

impl Hooks for Disk {
    fn dispatch(&self, _call: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        match std::fs::read(path) {
            Ok(bytes) => Fetch::Ready(bytes),
            Err(e) => Fetch::Failed(format!("{path}: {e}")),
        }
    }
}

/// What the last frame's particle systems were handed and held.
#[derive(Default)]
struct Carried {
    /// Each system's air, by entity.
    air: Vec<(u64, Vec3)>,
    /// The velocities of the particles over 2.5 s old the tick simulates.
    settled: Vec<Vec3>,
}

struct Systems(Rc<RefCell<Carried>>);

impl RenderBackend for Systems {
    fn tier(&self) -> String {
        "test".into()
    }
    fn size(&self) -> (u32, u32) {
        (64, 64)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        let fx = &scene.effects;
        *self.0.borrow_mut() = Carried {
            air: fx.particles.iter().map(|p| (p.id, p.air)).collect(),
            settled: fx
                .hosted_particles
                .iter()
                .flat_map(|(_, cpu)| &cpu.slots)
                .filter(|s| s.alive() && s.pos_age[3] > 2.5)
                .map(|s| Vec3::from_slice(&s.vel_life[..3]))
                .collect(),
        };
        Ok(())
    }
    fn capture(&mut self, _: &RenderScene, _: u32, _: u32) -> Option<og_render::RgbaImage> {
        None
    }
}

#[test]
fn the_air_carries_the_particles_of_an_emitter_that_takes_wind() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("og-runtime-world-wind-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let asset = |path: &str, text: &str| {
        let p = root.join("assets").join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    };
    let dust = "type particles\nversion 1\n---\nrate 30\nlifetime 5\nspeed 1\n\
                gravity 0 0 0\ndrag 4\nwind 1\nspace world\n";
    asset("fx/smoke.ogt", dust);
    // The same, sending events: the tick simulates it.
    asset("fx/dust.ogt", &format!("{dust}events birth\n"));
    let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());

    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let watch = og_system_define_system(
        "Watch".into(),
        json!({ "reads": [{ "kind": "event", "name": "og.ParticleEvent" }], "writes": [] })
            .to_string(),
        1,
    );
    let game = og_game_game(Some(json!({ "seed": 3, "systems": [watch] }).to_string()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let engine = answer(og_engine_create(game, options))["value"]
        .as_u64()
        .unwrap() as u32;
    let seen = Rc::new(RefCell::new(Carried::default()));
    set_render_backend(engine, Some(Box::new(Systems(seen.clone())))).unwrap();
    answer(og_engine_frame(engine, 1000.0 / 60.0));

    let load = |path: &str| {
        answer(og_runtime::host_impl::answer(og_load_emitter(
            engine,
            path.into(),
        )))["value"]
            .clone()
    };
    let (smoke, dust) = (load("fx/smoke.ogt"), load("fx/dust.ogt"));
    let system = |x: f32, emitter: &Value, seed: u32| {
        spawn(
            engine,
            json!({ "position": { "x": x, "y": 3, "z": 0 } }),
            &[(
                "og.ParticleSystem",
                json!({ "emitter": emitter, "seed": seed, "startTick": 0 }),
            )],
        ) as u64
    };
    // Smoke 2 m in front of a fan that blows along -x, smoke and dust far
    // beyond its reach; 5 m/s of wind to -z over all of them.
    let near = system(0.0, &smoke, 1);
    let far = system(60.0, &smoke, 2);
    let hosted = system(90.0, &dust, 3);
    let turn = std::f32::consts::FRAC_1_SQRT_2;
    spawn(
        engine,
        json!({
            "position": { "x": 2, "y": 3, "z": 0 },
            "quaternion": { "x": 0, "y": turn, "z": 0, "w": turn },
        }),
        &[(
            "og.WindMotor",
            json!({ "kind": 1, "radius": 6, "strength": 10 }),
        )],
    );
    write(
        engine,
        "og.Wind",
        json!({ "direction": { "x": 0, "y": 0, "z": -1 }, "speed": 5 }),
    );
    for _ in 0..200 {
        answer(og_engine_frame(engine, 1000.0 / 60.0));
    }
    let seen = seen.borrow();
    let wind = Vec3::new(0.0, 0.0, -5.0);
    let air = |id: u64| {
        let found = seen.air.iter().find(|(e, _)| *e == id);
        found.unwrap_or_else(|| panic!("system {id} drawn")).1
    };
    for id in [far, hosted] {
        assert!(close(air(id), wind), "only the wind: {:?}", air(id));
    }
    let fanned = air(near) - wind;
    assert!(
        fanned.x < -1.0 && fanned.y.abs() < 1e-3 && fanned.z.abs() < 1e-3,
        "the fan adds its push along its forward: {fanned:?}"
    );
    assert!(
        seen.settled.len() > 20,
        "{} old particles in the tick",
        seen.settled.len()
    );
    for v in &seen.settled {
        assert!(v.distance(wind) < 0.05, "carried downwind: {v:?}");
    }
}
