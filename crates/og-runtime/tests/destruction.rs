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
    Dispatch, Fetch, Hooks, og_destruction_damage, og_destruction_info, og_engine_create,
    og_engine_frame, og_engine_probe_dump, og_engine_write_resource, og_game_game,
    og_host_begin_bundle, og_load_emitter, og_load_gltf, og_world_get, og_world_has,
    og_world_insert, og_world_instantiate, og_world_restore, og_world_snapshot, set_hooks,
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

/// A brick wall tied in at both ends, as a wall between two others is:
/// the destruction-yard fixture's, 5 × 2.4 m.
const BRICK: &str = r#"type model
version 1
---
part "wall" {
  position 0 1.2 0
  boxGeometry {
    width 5
    height 2.4
    depth 0.3
  }
}
fracture {
  style brick
  brick 0.25 0.1
  chunks 250
  seed 11
  children 32
  anchor bottom ends
}
"#;

/// A glass pane, 1.6 × 1.2 m and 1 cm thick, cracking from its middle.
const PANE: &str = r#"type model
version 1
---
part "pane" {
  position 0 0.6 0
  boxGeometry {
    width 1.6
    height 1.2
    depth 0.01
  }
}
fracture {
  style glass
  chunks 25
  rings 3
  seed 9
  children 0
  impact 0 0.6 0 0.25
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
        std::fs::write(root.join("assets/props/pane.ogt"), PANE).unwrap();
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
fn a_collapse_replays_and_restores_bit_for_bit() {
    // At the default stress budget, and at one so small that every solve
    // after a tick's first waits for a later tick: what waits replays and
    // restores the same.
    for budget in [None, Some(1)] {
        replays_and_restores(budget);
    }
}

fn replays_and_restores(budget: Option<u32>) {
    let run = |restore_at: Option<u32>| {
        let (w, wall) = start();
        if let Some(b) = budget {
            value(og_engine_write_resource(
                w,
                "og.Destruction".into(),
                json!({ "stressBudget": b }).to_string(),
            ));
        }
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
fn a_wall_tied_in_at_its_ends_blown_at_its_foot_leaves_two_piers_and_nothing_over_the_hole() {
    // The example's charge (1.5 m, full within 0.8 m) and one either side,
    // each big enough to take the wall's middle to its top and small enough
    // to spare the piers' feet: a pier steps back about half a brick a
    // course, so its height follows the width of the foot the charge
    // leaves it.
    for radius in [1.4, 1.5, 1.6] {
        let (w, wall) = start_with(
            "props/brick.ogt",
            json!({ "toughness": 800, "strength": 3000, "density": 1900, "tension": 0.1 }),
        );
        let chips = value(og_load_emitter(w, "fx/chips.ogt".into()))
            .as_f64()
            .unwrap();
        value(og_world_insert(
            w,
            wall,
            "og.Destructible".into(),
            Some(json!({ "toughness": 800, "strength": 3000, "density": 1900, "debris": chips, "impactThreshold": 400, "tension": 0.1 }).to_string()),
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
        // Where each chunk stands before the blast (every one static, at
        // its own place).
        let before = value(og_runtime::og_physics_overlap(
            w,
            Some(
                json!({ "shape": { "shape": 0, "size": { "x": 12, "y": 6, "z": 12 } }, "position": { "x": 0, "y": 2.5, "z": 0 } })
                    .to_string(),
            ),
        ));
        let rest: std::collections::BTreeMap<u64, (f64, f64)> = before
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e.as_f64().unwrap())
            .filter(|&e| og_world_has(w, e, "og.Chunk".into()))
            .map(|e| {
                let c = value(og_world_get(w, e, "og.Chunk".into()))["chunk"]
                    .as_u64()
                    .unwrap();
                let t = value(og_world_get(w, e, "og.Transform".into()));
                (
                    c,
                    (
                        t["position"]["x"].as_f64().unwrap(),
                        t["position"]["y"].as_f64().unwrap(),
                    ),
                )
            })
            .collect();
        // the destruction yard's charge: at the face of its foot, in the middle.
        // Masonry's impactThreshold (heavy blocks land at hundreds of N·s, so
        // a lower one lets every falling block knock the piers apart).
        value(og_runtime::og_destruction_explode(
            w,
            Some(
                json!({ "point": { "x": 0, "y": 0.3, "z": 0.15 }, "radius": radius, "innerRadius": 0.8, "damage": 6000, "impulse": 2500 })
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
        // A blast pushes on faces, not once per body: the outer courses are
        // nudged where the middle is thrown, and no brick lands metres past
        // the wall's ends (out of frame), nor far before or behind it.
        let all = value(og_runtime::og_physics_overlap(
            w,
            Some(
                json!({ "shape": { "shape": 0, "size": { "x": 60, "y": 20, "z": 60 } }, "position": { "x": 0, "y": 5, "z": 0 } })
                    .to_string(),
            ),
        ));
        let landed: Vec<(f64, f64)> = all
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e.as_f64().unwrap())
            .filter(|&e| {
                og_world_has(w, e, "og.Chunk".into()) && !og_world_has(w, e, "og.Parent".into())
            })
            .map(|e| {
                let t = value(og_world_get(w, e, "og.Transform".into()));
                let (x, z) = (
                    t["position"]["x"].as_f64().unwrap(),
                    t["position"]["z"].as_f64().unwrap(),
                );
                ((x.abs() - 2.5).max(0.0), (x.abs() - 2.5).max(0.0).hypot(z))
            })
            .collect();
        // A lone brick may fly (a blast's front is uneven); a spray of them
        // metres out is a push that does not taper.
        let past = landed.iter().filter(|p| p.0 > 1.5).count();
        let far = landed.iter().filter(|p| p.1 > 4.0).count();
        assert!(
            past <= 1 && far <= 2,
            "radius {radius}: {past} pieces land over 1.5 m past the wall's ends, {far} over 4 m from the wall"
        );
        // What still stands: the chunks within 10 cm of where they were
        // before the blast (a block freed whole and resting where it stood
        // stands; rubble heaped against a pier is not the pier).
        let standing: Vec<(f64, f64)> = near
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e.as_f64().unwrap())
            .filter(|&e| og_world_has(w, e, "og.Chunk".into()))
            .filter_map(|e| {
                let c = value(og_world_get(w, e, "og.Chunk".into()))["chunk"]
                    .as_u64()
                    .unwrap();
                let (x0, y0) = *rest.get(&c)?;
                let (x, y) = at(e);
                ((x - x0).hypot(y - y0) < 0.1).then_some((x, y))
            })
            .collect();
        let pier = |side: f64| -> f64 {
            standing
                .iter()
                .filter(|(x, _)| x * side > 0.9 && x * side < 2.6)
                .map(|&(_, y)| y)
                .fold(0.0, f64::max)
        };
        // Both piers stand, and one stands a third of the wall's 2.4 m.
        assert!(
            pier(-1.0) > 0.5 && pier(1.0) > 0.5 && pier(-1.0).max(pier(1.0)) > 0.8,
            "radius {radius}: a pier stands at each end, unmoved brickwork centred above 0.5 m, one above 0.8 m: {} {} of {chunks:?}",
            pier(-1.0),
            pier(1.0)
        );
        // A span is a block high over the hole with air under it; one on
        // top of the heap the wall's middle fell into rests on the pieces
        // under it.
        let over: Vec<_> = chunks
            .iter()
            .filter(|(x, y)| x.abs() < 0.25 && *y > 1.5)
            .filter(|&&(x, y)| {
                chunks
                    .iter()
                    .filter(|&&(u, v)| (u - x).abs() < 0.3 && v < y - 0.05)
                    .count()
                    < 3
            })
            .collect();
        assert!(
            over.is_empty(),
            "radius {radius}: nothing is left over the hole above its rubble (no lintel or wedged blocks span the piers): {over:?}"
        );
        // Each pier stands on brickwork at least two bricks wide: its lower
        // half, above the anchored bottom course (which stands whatever
        // the blast does), is no column a brick wide. Above that, the
        // courses tied into the wall's end may stand a block wide, as a
        // real wall's toothed end does; how high that column reaches is
        // chaotic (any change to a body's bits moves it), so a band at a
        // share of the pier's height measured that column or the
        // brickwork beside it by chance: under 0.4 m at 4 to 6 of 21
        // radii from 1.3 to 1.7 with either eigensolver (glamx's own and
        // third_party/glamx's), where the lower half stays at least
        // 0.438 m at every radius from 1.3 to 1.66 with both.
        for side in [-1.0, 1.0] {
            let h = pier(side);
            let xs: Vec<f64> = standing
                .iter()
                .filter(|&&(x, y)| x * side > 0.6 && (0.1..0.5 * h).contains(&y))
                .map(|&(x, _)| x * side)
                .collect();
            let wide = xs.iter().copied().fold(f64::MIN, f64::max)
                - xs.iter().copied().fold(f64::MAX, f64::min);
            assert!(
                wide >= 0.4,
                "radius {radius}: the pier at {side} is {wide} m wide through its lower half: {chunks:?}"
            );
        }
        // No block left standing hangs a whole brick over air, not even
        // one tied into the wall beside it: each above the bottom course
        // rests on the course below over all but half a brick of its
        // length.
        let bundle = og_cook::runtime::Bundle::open(&project().join("cooked")).unwrap();
        let bytes = bundle.read("props/brick.ogt#fracture").unwrap();
        let a = og_destruct::FractureAsset::from_bytes(
            &bundle.fracture("props/brick.ogt#fracture", &bytes).unwrap(),
        )
        .unwrap();
        let standing: Vec<(u64, f32, f32, f32)> = near
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e.as_f64().unwrap())
            .filter(|&e| og_world_has(w, e, "og.Chunk".into()))
            .map(|e| value(og_world_get(w, e, "og.Chunk".into())))
            .filter(|c| c["looseTick"].as_u64() == Some(u32::MAX as u64))
            .map(|c| {
                let i = c["chunk"].as_u64().unwrap();
                let ch = &a.chunks[i as usize];
                let x = |f: fn(f32, f32) -> f32, from: f32| {
                    ch.hull.iter().map(|v| v.x).fold(from, f) + ch.centroid.x
                };
                (
                    i,
                    x(f32::min, f32::MAX),
                    x(f32::max, f32::MIN),
                    ch.centroid.y,
                )
            })
            .collect();
        for &(i, lo, hi, y) in &standing {
            if y < 0.1 {
                continue;
            }
            let below: Vec<(f32, f32)> = standing
                .iter()
                .filter(|s| (s.3 - (y - 0.1)).abs() < 0.03)
                .map(|s| (s.1, s.2))
                .collect();
            let (mut run, mut over) = (0.0f32, 0.0f32);
            let step = 0.005;
            let mut x = lo;
            while x <= hi {
                if below.iter().any(|b| x >= b.0 - step && x <= b.1 + step) {
                    run = 0.0;
                } else {
                    run += step;
                    over = over.max(run);
                }
                x += step;
            }
            assert!(
                over <= 0.13,
                "radius {radius}: block {i} ({lo:.3} to {hi:.3}, at {y:.2} m) hangs {over:.3} m over air"
            );
        }
        let info = value(og_destruction_info(w, wall));
        assert!(
            info["islands"].as_u64().unwrap() > 1,
            "radius {radius}: {info}"
        );
    }
}

#[test]
fn a_struck_pillar_falls_to_its_stump() {
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/destruction-yard");
    let root = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("scratch-pillar");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::process::Command::new("cp")
        .arg("-R")
        .arg(src.join("assets"))
        .arg(&root)
        .status()
        .unwrap();
    let pp = root.join("assets/props/pillar.ogt");
    let t = std::fs::read_to_string(&pp)
        .unwrap()
        .replace("  chunks 44\n", "  chunks 48\n");
    std::fs::write(&pp, t).unwrap();
    let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let w = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
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
    let asset = value(og_load_gltf(w, "props/pillar.ogt".into(), None))
        .as_f64()
        .unwrap();
    let p = value(og_world_instantiate(w, asset, None))
        .as_f64()
        .unwrap();
    value(og_world_insert(w, p, "og.Destructible".into(), Some(json!({ "toughness": 2000, "strength": 20000, "density": 2400, "impactThreshold": 150 }).to_string())));
    frames(w, 10);
    value(og_destruction_damage(w, p, Some(json!({ "point": { "x": 0, "y": 1.4, "z": 0.25 }, "radius": 0.5, "innerRadius": 0.25, "amount": 8000 }).to_string())));
    frames(w, 200);
    let near = value(og_runtime::og_physics_overlap(w, Some(json!({ "shape": { "shape": 0, "size": { "x": 8, "y": 8, "z": 8 } }, "position": { "x": 0, "y": 4, "z": 0 } }).to_string())));
    // At 48 cells the pillar once cooked a sliver chunk of no mass, which
    // physics disabled mid-flight: the pieces over the blow froze where
    // they were thrown and stood as a column in the air.
    let mut high = Vec::new();
    for e in near.as_array().unwrap().iter().map(|e| e.as_f64().unwrap()) {
        if !og_world_has(w, e, "og.Chunk".into()) {
            continue;
        }
        let t = value(og_world_get(w, e, "og.Transform".into()));
        let y = t["position"]["y"].as_f64().unwrap();
        if y > 1.4 {
            high.push((
                value(og_world_get(w, e, "og.Chunk".into()))["chunk"].clone(),
                y,
            ));
        }
    }
    assert!(
        high.is_empty(),
        "chunks still standing over the blow: {high:?}"
    );
}
