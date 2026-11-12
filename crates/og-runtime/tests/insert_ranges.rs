//! Insert refuses reflected ranges before mutating or queuing a component.
use og_runtime::*;
use serde_json::{Value, json};
use std::{cell::Cell, rc::Rc};

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["ok"], true, "{text}");
    v["value"].clone()
}

fn refuse(engine: u32, entity: f64) {
    for (component, fields, field, bound) in [
        (
            "og.TerrainScatter",
            json!({"fade": 12}),
            "fade",
            "maximum 1",
        ),
        (
            "og.TerrainScatter",
            json!({"fade": -0.1}),
            "fade",
            "minimum 0",
        ),
        (
            "Bounded",
            json!({"amount": 2.00000001}),
            "amount",
            "maximum 2",
        ),
        (
            "Bounded",
            json!({"direction": {"x": 0, "y": -3, "z": 1}}),
            "direction",
            "minimum -2",
        ),
    ] {
        let text = og_runtime::host_impl::answer(og_world_insert(
            engine,
            entity,
            component.into(),
            Some(fields.to_string()),
        ));
        let result: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(result["ok"], false, "{text}");
        assert!(text.contains(field) && text.contains(bound), "{text}");
    }
}

struct Game {
    entity: Cell<f64>,
    called: Cell<bool>,
}
impl Hooks for Game {
    fn dispatch(&self, call: &Dispatch) -> Result<String, String> {
        if call.kind == 0 && call.id == 1 {
            refuse(call.engine, self.entity.get());
            self.called.set(true);
        }
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        Fetch::Failed(format!("no asset {path}"))
    }
}

#[test]
fn insert_checks_engine_and_game_ranges_inside_and_outside_systems() {
    let hooks = Rc::new(Game {
        entity: Cell::new(0.0),
        called: Cell::new(false),
    });
    set_hooks(hooks.clone());
    og_host_begin_bundle();
    value(og_runtime::host_impl::answer(og_reflect_define_component("Bounded".into(), json!([
        {"name": "amount", "kind": "f32", "default": 0, "options": {"min": -2, "max": 2}},
        {"name": "direction", "kind": "Vector3", "default": {"x": 0, "y": 0, "z": 0}, "options": {"min": -2, "max": 2}}
    ]).to_string())));
    let system = og_system_define_system(
        "Insert".into(),
        json!({"writes": [
            {"kind": "component", "name": "Bounded"},
            {"kind": "component", "name": "og.TerrainScatter"}
        ]})
        .to_string(),
        1,
    );
    let game = og_game_game(Some(json!({"systems": [system]}).to_string()));
    let engine = value(og_engine_create(game, "{}".into())).as_u64().unwrap() as u32;
    let entity = value(og_runtime::host_impl::answer(og_world_spawn(engine, None)))
        .as_f64()
        .unwrap();
    hooks.entity.set(entity);
    for fade in [0.0, 1.0] {
        value(og_runtime::host_impl::answer(og_world_insert(
            engine,
            entity,
            "og.TerrainScatter".into(),
            Some(json!({"fade": fade}).to_string()),
        )));
    }
    value(og_runtime::host_impl::answer(og_world_insert(
        engine,
        entity,
        "Bounded".into(),
        Some(json!({"amount": -2, "direction": {"x": 2, "y": -2, "z": 0}}).to_string()),
    )));
    refuse(engine, entity);
    value(og_engine_frame(engine, 1000.0 / 60.0));
    assert!(hooks.called.get());
    let scatter = value(og_runtime::host_impl::answer(og_world_get(
        engine,
        entity,
        "og.TerrainScatter".into(),
    )));
    assert_eq!(scatter["fade"], 1.0, "{scatter}");
}
