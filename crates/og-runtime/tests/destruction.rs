//! og.destruction (docs/design.md §2.7): a wall cooked with a `fracture`
//! block stands on its anchored chunks; shot out at its foot, the stress
//! solver brings down what it held up, the loose chunks fall as bodies, a
//! snapshot taken mid-collapse restores into a run that hashes as the
//! uninterrupted one, and the budgets despawn the oldest loose chunks.
//!
//! Ways this could fail: the bonds' health or the chunks' bodies are not
//! world state (a restore loses them, the hashes part); the stress solver
//! is not run (the wall stands on air above its hole); islands become
//! bodies in an order that depends on something other than the world (the
//! replay diverges); the budget counts the wrong chunks or none.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_destruction_capabilities, og_destruction_damage,
    og_destruction_info, og_engine_create, og_engine_frame, og_engine_probe_dump,
    og_engine_write_resource, og_game_game, og_host_begin_bundle, og_load_emitter, og_load_gltf,
    og_world_events, og_world_get, og_world_has, og_world_insert, og_world_instantiate,
    og_world_restore, og_world_snapshot, set_hooks,
};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::rc::Rc;

const WALL: &str = r#"type model
version 1
---
material "brick" {
  color 0.6 0.3 0.2
  roughness 0.9
}
part "wall" {
  material "brick"
  position 0 1 0
  boxGeometry {
    width 4
    height 2
    depth 0.3
  }
}
fracture {
  chunks 24
  seed 3
  children 4
}
"#;

/// A brick wall tied in at both ends, as a wall between two others is.
const BRICK: &str = r#"type model
version 1
---
part "wall" {
  position 0 1.2 0
  boxGeometry {
    width 4
    height 2.4
    depth 0.3
  }
}
fracture {
  style brick
  brick 0.25 0.1
  chunks 80
  seed 11
  children 32
  anchor bottom ends
}
"#;

/// Fine debris: tumbling mesh chips, a one-shot burst.
const CHIPS: &str = r#"type particles
version 1
---
maxParticles 256
shape sphere 0.2
spread 180
rate 0
burst 0 40
duration 0.5
looping false
lifetime 2 3
speed 1 3
size 0.04 0.08
gravity 0 -9.8 0
render mesh box
lit true
collision 0.3 0.6
"#;

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

/// A project with the wall, cooked once.
fn project() -> &'static std::path::Path {
    static ROOT: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    ROOT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-destruction-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("assets/props")).unwrap();
        std::fs::write(root.join("assets/props/wall.ogt"), WALL).unwrap();
        std::fs::write(root.join("assets/props/brick.ogt"), BRICK).unwrap();
        std::fs::create_dir_all(root.join("assets/fx")).unwrap();
        std::fs::write(root.join("assets/fx/chips.ogt"), CHIPS).unwrap();
        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        root
    })
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

/// An engine with the wall standing at the origin, as an og.Destructible.
fn start() -> (u32, f64) {
    // No friction: a Voronoi wall's joints run every way, and its middle
    // comes down above a hole.
    start_with(
        "props/wall.ogt",
        json!({ "toughness": 400, "strength": 4000, "friction": 0 }),
    )
}

fn start_with(model: &str, destructible: Value) -> (u32, f64) {
    let root = project();
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let w = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let asset = value(og_load_gltf(w, model.into(), None)).as_f64().unwrap();
    let wall = value(og_world_instantiate(w, asset, None))
        .as_f64()
        .unwrap();
    value(og_world_insert(
        w,
        wall,
        "og.Destructible".into(),
        Some(destructible.to_string()),
    ));
    (w, wall)
}

fn frames(w: u32, n: u32) {
    for _ in 0..n {
        value(og_engine_frame(w, 1000.0 / 60.0));
    }
}

fn hash(w: u32) -> Value {
    let dump: Value = serde_json::from_str(&og_engine_probe_dump(w, None)).unwrap();
    dump["hash"].clone()
}

fn destructible(w: u32, e: f64) -> Value {
    value(og_world_get(w, e, "og.Destructible".into()))
}

/// A blow at the wall's foot, in its middle.
fn shoot_foot(w: u32, wall: f64) {
    value(og_destruction_damage(
        w,
        wall,
        Some(
            json!({ "point": { "x": 0, "y": 0.1, "z": 0 }, "radius": 0.9, "innerRadius": 0.5, "amount": 4000 })
                .to_string(),
        ),
    ));
}

#[test]
fn a_wall_stands_until_shot_then_its_middle_comes_down() {
    let (w, wall) = start();
    frames(w, 30);
    let d = destructible(w, wall);
    assert_eq!(d["fractured"], false, "it stands under its own weight: {d}");
    let info = value(og_destruction_info(w, wall));
    assert_eq!(info["chunks"], 24);
    assert_eq!(info["islands"], 1);
    assert_eq!(info["health"], 1.0);
    assert!(info["anchored"].as_u64().unwrap() >= 3, "{info}");

    shoot_foot(w, wall);
    frames(w, 1);
    let events: Value = serde_json::from_str(&og_world_events(w, "og.Fractured".into())).unwrap();
    let events = events.as_array().unwrap();
    assert!(!events.is_empty(), "the shot breaks something loose");
    assert_eq!(events[0]["entity"].as_f64(), Some(wall));
    // og.destruction.damage, or the chunks it shattered.
    assert!(
        events
            .iter()
            .all(|e| [1, 3, 4].contains(&e["cause"].as_u64().unwrap())),
        "{events:?}"
    );
    let shot = destructible(w, wall)["loose"].as_u64().unwrap();
    assert!(shot > 0);

    // Over the next seconds the stress solver brings down what stood on
    // the chunks that were shot away.
    frames(w, 120);
    let d = destructible(w, wall);
    let info = value(og_destruction_info(w, wall));
    assert_eq!(d["fractured"], true);
    assert!(
        d["loose"].as_u64().unwrap() > shot,
        "more fell than the shot broke ({shot}): {d} {info}"
    );
    assert!(info["health"].as_f64().unwrap() < 1.0);
}

#[test]
fn a_collapse_replays_and_restores_bit_for_bit() {
    let run = |restore_at: Option<u32>| {
        let (w, wall) = start();
        frames(w, 5);
        shoot_foot(w, wall);
        frames(w, 12);
        let mid = value(og_world_snapshot(w, None))
            .as_str()
            .unwrap()
            .to_string();
        frames(w, 40);
        let end = hash(w);
        if let Some(n) = restore_at {
            // Restored mid-collapse, it runs on as the uninterrupted world.
            value(og_world_restore(w, mid, None));
            frames(w, n);
            assert_eq!(hash(w), end, "restored mid-collapse");
        }
        (end, destructible(w, wall))
    };
    let (a, da) = run(None);
    let (b, db) = run(Some(40));
    assert_eq!(a, b, "the same shot replays to the same world");
    assert_eq!(da, db);
    assert_eq!(da["fractured"], true);
}

#[test]
fn the_chunk_budget_despawns_the_oldest_loose_chunks() {
    let (w, wall) = start();
    value(og_engine_write_resource(
        w,
        "og.Destruction".into(),
        json!({ "maxChunks": 3 }).to_string(),
    ));
    frames(w, 2);
    shoot_foot(w, wall);
    frames(w, 90);
    let d = destructible(w, wall);
    assert_eq!(d["fractured"], true);
    let loose = d["loose"].as_u64().unwrap();
    assert!(loose <= 3, "at most maxChunks loose chunks remain: {d}");
}

#[test]
fn pieces_given_to_debris_are_never_bodies() {
    let (w, wall) = start();
    let chips = value(og_load_emitter(w, "fx/chips.ogt".into()))
        .as_f64()
        .unwrap();
    value(og_world_insert(
        w,
        wall,
        "og.Destructible".into(),
        Some(json!({ "toughness": 400, "strength": 4000, "debris": chips }).to_string()),
    ));
    // Every piece of a shattered chunk is debris here.
    value(og_engine_write_resource(
        w,
        "og.Destruction".into(),
        json!({ "debrisVolume": 1.0 }).to_string(),
    ));
    frames(w, 2);
    shoot_foot(w, wall);
    frames(w, 1);
    let events: Value = serde_json::from_str(&og_world_events(w, "og.Fractured".into())).unwrap();
    let shattered: Vec<&Value> = events
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["cause"] == 4)
        .collect();
    assert!(!shattered.is_empty(), "the shot shatters chunks: {events}");
    for e in shattered {
        assert_eq!(e["chunks"], 0, "no piece became a body: {e}");
        assert_eq!(e["body"], 0);
        assert!(e["volume"].as_f64().unwrap() > 0.0);
    }
    // No renderer here: the bursts are not drawn, the world is the same.
    let caps: Value = serde_json::from_str(&og_destruction_capabilities(w)).unwrap();
    assert_eq!(caps["visualDebris"], false);
}

#[test]
fn a_prop_dropped_on_the_ground_breaks_where_it_lands() {
    // A fast body's contact opens speculatively, so the event's impulse is
    // a few N·s; the hit is weighed by its closing speed and mass instead.
    let (w, _wall) = start();
    let g = value(og_runtime::og_world_spawn(
        w,
        Some(json!({ "position": { "x": 10, "y": -0.25, "z": 0 } }).to_string()),
    ))
    .as_f64()
    .unwrap();
    value(og_world_insert(
        w,
        g,
        "og.RigidBody".into(),
        Some(json!({ "kind": 1 }).to_string()),
    ));
    value(og_world_insert(
        w,
        g,
        "og.Collider".into(),
        Some(json!({ "shape": 0, "size": { "x": 6, "y": 0.5, "z": 6 } }).to_string()),
    ));
    let asset = value(og_load_gltf(w, "props/wall.ogt".into(), None))
        .as_f64()
        .unwrap();
    let c = value(og_world_instantiate(
        w,
        asset,
        Some(json!({ "position": { "x": 10, "y": 4, "z": 0 } }).to_string()),
    ))
    .as_f64()
    .unwrap();
    value(og_world_insert(
        w,
        c,
        "og.Destructible".into(),
        Some(json!({ "toughness": 300, "anchored": false, "impactThreshold": 40 }).to_string()),
    ));
    value(og_world_insert(
        w,
        c,
        "og.RigidBody".into(),
        Some(json!({ "kind": 0 }).to_string()),
    ));
    value(og_world_insert(
        w,
        c,
        "og.Collider".into(),
        Some(json!({ "shape": 0, "size": { "x": 4, "y": 2, "z": 0.3 }, "offset": { "x": 0, "y": 1, "z": 0 } }).to_string()),
    ));
    let mut impact = false;
    for _ in 0..80 {
        frames(w, 1);
        let ev: Value = serde_json::from_str(&og_world_events(w, "og.Fractured".into())).unwrap();
        impact |= ev.as_array().unwrap().iter().any(|e| e["cause"] == 0);
    }
    let d = destructible(w, c);
    assert_eq!(d["fractured"], true, "a 4 m fall breaks it: {d}");
    assert!(impact, "broken by its impact");
}

#[test]
fn a_wall_tied_in_at_its_ends_blown_at_its_foot_leaves_two_piers_and_nothing_over_the_hole() {
    // Blast radii either side of the example's: loose blocks jammed between
    // the piers would hold a flat arch over the hole at some of them.
    for radius in [1.3, 1.4, 1.5] {
        let (w, wall) = start_with(
            "props/brick.ogt",
            json!({ "toughness": 800, "strength": 3000, "density": 1900 }),
        );
        let chips = value(og_load_emitter(w, "fx/chips.ogt".into()))
            .as_f64()
            .unwrap();
        value(og_world_insert(
            w,
            wall,
            "og.Destructible".into(),
            Some(json!({ "toughness": 800, "strength": 3000, "density": 1900, "debris": chips, "impactThreshold": 400 }).to_string()),
        ));
        // A floor for what falls.
        let g = value(og_runtime::og_world_spawn(
            w,
            Some(json!({ "position": { "x": 0, "y": -0.25, "z": 0 } }).to_string()),
        ))
        .as_f64()
        .unwrap();
        value(og_world_insert(
            w,
            g,
            "og.RigidBody".into(),
            Some(json!({ "kind": 1 }).to_string()),
        ));
        value(og_world_insert(
            w,
            g,
            "og.Collider".into(),
            Some(json!({ "shape": 0, "size": { "x": 20, "y": 0.5, "z": 20 } }).to_string()),
        ));
        frames(w, 10);
        // demolition-yard's charge: at the face of its foot, in the middle.
        // Masonry's impactThreshold (heavy blocks land at hundreds of N·s, so
        // a lower one lets every falling block knock the piers apart).
        value(og_runtime::og_destruction_explode(
            w,
            Some(
                json!({ "point": { "x": 0, "y": 0.3, "z": 0.15 }, "radius": radius, "innerRadius": 0.8, "damage": 6000, "impulse": 300 })
                    .to_string(),
            ),
        ));
        frames(w, 300);
        // Every chunk where it ended, static or loose (a compound child's
        // place through its body's).
        let near = value(og_runtime::og_physics_overlap(
            w,
            Some(
                json!({ "shape": { "shape": 0, "size": { "x": 12, "y": 6, "z": 12 } }, "position": { "x": 0, "y": 2.5, "z": 0 } })
                    .to_string(),
            ),
        ));
        let at = |e: f64| -> (f64, f64) {
            let t = value(og_world_get(w, e, "og.Transform".into()));
            let (x, y) = (
                t["position"]["x"].as_f64().unwrap(),
                t["position"]["y"].as_f64().unwrap(),
            );
            if !og_world_has(w, e, "og.Parent".into()) {
                return (x, y);
            }
            let b = value(og_world_get(w, e, "og.Parent".into()))["entity"]
                .as_f64()
                .unwrap();
            let bt = value(og_world_get(w, b, "og.Transform".into()));
            let q = &bt["quaternion"];
            let q = glam::Quat::from_xyzw(
                q["x"].as_f64().unwrap() as f32,
                q["y"].as_f64().unwrap() as f32,
                q["z"].as_f64().unwrap() as f32,
                q["w"].as_f64().unwrap() as f32,
            );
            let p = q * glam::Vec3::new(
                x as f32,
                y as f32,
                t["position"]["z"].as_f64().unwrap() as f32,
            );
            (
                bt["position"]["x"].as_f64().unwrap() + p.x as f64,
                bt["position"]["y"].as_f64().unwrap() + p.y as f64,
            )
        };
        let chunks: Vec<(f64, f64)> = near
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e.as_f64().unwrap())
            .filter(|&e| og_world_has(w, e, "og.Chunk".into()))
            .map(at)
            .collect();
        let pier = |side: f64| -> f64 {
            chunks
                .iter()
                .filter(|(x, _)| x * side > 0.9 && x * side < 2.1)
                .map(|&(_, y)| y)
                .fold(0.0, f64::max)
        };
        assert!(
            pier(-1.0) > 0.8 && pier(1.0) > 0.8,
            "radius {radius}: a pier stands at each end, a chunk of it centred above 0.8 m: {} {} of {chunks:?}",
            pier(-1.0),
            pier(1.0)
        );
        let over: Vec<_> = chunks
            .iter()
            .filter(|(x, y)| x.abs() < 0.4 && *y > 1.5)
            .collect();
        assert!(
            over.is_empty(),
            "radius {radius}: nothing is left over the hole above its rubble (no lintel or wedged blocks span the piers): {over:?}"
        );
        let info = value(og_destruction_info(w, wall));
        assert!(
            info["islands"].as_u64().unwrap() > 1,
            "radius {radius}: {info}"
        );
    }
}
