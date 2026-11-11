//! The default invariants from reflection and og.assert's report: a NaN in a
//! resource, a component field outside its declared range and an Entity
//! field naming a despawned entity are each a violation with the tick, the
//! entity, the record, the field and the value; a sometimes that never held
//! is uncovered; and `assertions { take }` starts a report that inherits
//! none of it.
#![cfg(not(feature = "publish"))]

use og_runtime::*;
use serde_json::{Value, json};
use std::rc::Rc;

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["ok"], true, "{text}");
    v["value"].clone()
}

struct Game;
impl Hooks for Game {
    fn dispatch(&self, _call: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
}

fn engine() -> u32 {
    set_hooks(Rc::new(Game));
    og_host_begin_bundle();
    value(og_reflect_define_component(
        "Health".into(),
        json!([{"name": "hp", "kind": "f32", "default": 100, "options": {"min": 0, "max": 1000}}])
            .to_string(),
    ));
    value(og_reflect_define_component(
        "Fade".into(),
        json!([{"name": "depth", "kind": "f32", "default": 0.2, "options": {"min": 0.1, "max": 0.3}}])
            .to_string(),
    ));
    value(og_reflect_define_component(
        "Target".into(),
        json!([{"name": "of", "kind": "Entity", "default": 0, "options": {}}]).to_string(),
    ));
    value(og_reflect_define_resource(
        "Wind".into(),
        json!([{"name": "speed", "kind": "f32", "default": 1, "options": {}}]).to_string(),
    ));
    let game = og_game_game(Some(json!({"resources": ["Wind"]}).to_string()));
    value(og_engine_create(game, "{}".into())).as_u64().unwrap() as u32
}

fn report(engine: u32, take: bool) -> Value {
    serde_json::from_str(&og_engine_probe_assertions(engine, Some(take))).unwrap()
}

fn named<'a>(r: &'a Value, prefix: &str) -> &'a Value {
    r["violations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"].as_str().unwrap().starts_with(prefix))
        .unwrap_or_else(|| panic!("no violation '{prefix}…' in {r}"))
}

#[test]
fn nan_range_and_dangling_reference_are_violations_and_take_starts_clean() {
    let e = engine();
    let hero = value(og_runtime::host_impl::answer(og_world_spawn(e, None)))
        .as_f64()
        .unwrap();
    let foe = value(og_runtime::host_impl::answer(og_world_spawn(e, None)))
        .as_f64()
        .unwrap();
    value(og_runtime::host_impl::answer(og_world_insert(
        e,
        hero,
        "Health".into(),
        Some(json!({"hp": 50}).to_string()),
    )));
    value(og_runtime::host_impl::answer(og_world_insert(
        e,
        hero,
        "Target".into(),
        Some(json!({"of": foe}).to_string()),
    )));
    value(og_runtime::host_impl::answer(og_world_insert(
        e,
        hero,
        "og.Music".into(),
        Some(json!({"start": -8.55}).to_string()),
    )));
    value(og_engine_advance(e, 3));
    let clean = report(e, false);
    assert_eq!(clean["violations"], json!([]), "{clean}");
    assert!(
        clean["defaults"]["fields"].as_u64().unwrap() >= 3,
        "{clean}"
    );

    // A game write is stored as given: -5 is outside Health.hp's 0..1000.
    value(og_runtime::host_impl::answer(og_world_set(
        e,
        hero,
        "Health".into(),
        json!({"hp": -5}).to_string(),
    )));
    // A NaN reaches a resource through its packed bytes, as a JS handle writes it.
    let mut bytes = og_engine_read_resource(e, "Wind".into());
    bytes[..4].copy_from_slice(&f32::NAN.to_le_bytes());
    value(og_engine_write_resource_bytes(e, "Wind".into(), &bytes));
    value(og_runtime::host_impl::answer(og_world_despawn(e, foe)));
    // A sometimes that never holds, and one that does.
    let call = |name: &str, condition: bool| {
        og_assert_record(
            e,
            json!({"kind": "sometimes", "name": name, "location": "src/main.ts:9", "condition": condition, "key": "", "details": null, "tick": 3, "declared": false}).to_string(),
        )
    };
    call("boss reaches phase 2", false);
    call("hero jumps", true);
    value(og_engine_advance(e, 1));

    let r = report(e, true);
    let range = named(&r, "in range Health.hp");
    // The tick that ran (3, the sometimes calls' tick), not og.Time's next one.
    assert_eq!(range["tick"], 3, "{range}");
    assert_eq!(range["field"], "hp");
    assert_eq!(range["value"], -5.0);
    assert!(
        range["entity"].as_str().unwrap().starts_with('#'),
        "{range}"
    );
    let nan = named(&r, "finite Wind.speed");
    assert_eq!(nan["value"], "NaN", "{nan}");
    assert_eq!(nan["entity"], Value::Null);
    let dangling = named(&r, "live entity Target.of");
    assert_eq!(dangling["component"], "Target", "{dangling}");
    let uncovered: Vec<&str> = r["uncovered"]
        .as_array()
        .unwrap()
        .iter()
        .map(|u| u["name"].as_str().unwrap())
        .collect();
    assert_eq!(uncovered, ["boss reaches phase 2"], "{r}");

    // The next report inherits nothing: the sites stay, unfired.
    let next = report(e, false);
    assert_eq!(next["violations"], json!([]), "{next}");
    assert_eq!(next["epoch"], 1);
    assert_eq!(next["uncovered"].as_array().unwrap().len(), 2, "{next}");
}

/// The checks read what was written since the last one: a quiet tick reads
/// no component row, a write between ticks is read at the next, and a
/// restore reads everything again (a snapshot's bad value is found once
/// more after a fresh report).
#[test]
fn the_checks_read_only_written_columns_and_everything_after_a_restore() {
    let e = engine();
    let hero = value(og_runtime::host_impl::answer(og_world_spawn(e, None)))
        .as_f64()
        .unwrap();
    value(og_runtime::host_impl::answer(og_world_insert(
        e,
        hero,
        "Health".into(),
        Some(json!({"hp": 50}).to_string()),
    )));
    value(og_engine_advance(e, 2));
    report(e, true);
    // One tick reads what the engine's own systems wrote (input, camera)...
    value(og_engine_advance(e, 1));
    let quiet = report(e, true)["defaults"]["values"].as_u64().unwrap();
    // ...and one with Health written between ticks reads its one row more.
    value(og_runtime::host_impl::answer(og_world_set(
        e,
        hero,
        "Health".into(),
        json!({"hp": 60}).to_string(),
    )));
    value(og_engine_advance(e, 1));
    let written = report(e, true)["defaults"]["values"].as_u64().unwrap();
    assert_eq!(
        written,
        quiet + 1,
        "Health.hp is read once written, and only then"
    );

    value(og_runtime::host_impl::answer(og_world_set(
        e,
        hero,
        "Health".into(),
        json!({"hp": -5}).to_string(),
    )));
    value(og_engine_advance(e, 1));
    let save = value(og_runtime::host_impl::answer(og_world_snapshot(e, None)))
        .as_str()
        .unwrap()
        .to_owned();
    let bad = report(e, true);
    assert_eq!(named(&bad, "in range Health.hp")["tick"], 4, "{bad}");

    value(og_runtime::host_impl::answer(og_world_set(
        e,
        hero,
        "Health".into(),
        json!({"hp": 10}).to_string(),
    )));
    value(og_engine_advance(e, 2));
    assert_eq!(report(e, true)["violations"], json!([]));
    value(og_runtime::host_impl::answer(og_world_restore(
        e, save, None,
    )));
    value(og_engine_advance(e, 1));
    let again = report(e, true);
    assert_eq!(
        named(&again, "in range Health.hp")["value"],
        -5.0,
        "{again}"
    );
}

#[test]
fn a_float_field_at_its_declared_bound_is_in_range() {
    let e = engine();
    let lake = value(og_runtime::host_impl::answer(og_world_spawn(e, None)))
        .as_f64()
        .unwrap();
    // og.Water.depthFade's minimum is 0.01: its f32 holds 0.0099999998.
    value(og_runtime::host_impl::answer(og_world_insert(
        e,
        lake,
        "og.Water".into(),
        Some(json!({"depthFade": 0.01}).to_string()),
    )));
    // Fade.depth's maximum 0.3: its f32 holds 0.30000001.
    value(og_runtime::host_impl::answer(og_world_insert(
        e,
        lake,
        "Fade".into(),
        Some(json!({"depth": 0.3}).to_string()),
    )));
    value(og_engine_advance(e, 2));
    let at_bounds = report(e, true);
    assert_eq!(at_bounds["violations"], json!([]), "{at_bounds}");

    // Past a bound by more than the f32's rounding is still out of range.
    value(og_runtime::host_impl::answer(og_world_set(
        e,
        lake,
        "og.Water".into(),
        json!({"depthFade": 0.0099}).to_string(),
    )));
    value(og_engine_advance(e, 1));
    let r = report(e, true);
    let range = named(&r, "in range og.Water.depthFade");
    assert!(
        (range["value"].as_f64().unwrap() - 0.0099).abs() < 1e-6,
        "{range}"
    );
}
