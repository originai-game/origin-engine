//! A game's named stores, for tests that change what a document shows:
//! declared in the scope `test` over the host protocol, as a scope's
//! `signal` and `collection` are, and read by a document mounted with them.

use og_ui::reactive::{SourceId, wire};
use og_ui::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// The test store `name`.
pub fn source(name: &str) -> SourceId {
    SourceId::new("test", name)
}

/// One host operation; answers its value.
pub fn apply(ui: &mut Ui, command: Value) -> Value {
    let reply =
        wire::decode(&ui.apply_reactive(&wire::encode(&json!([command])).unwrap())).unwrap();
    assert_eq!(reply[0]["ok"], true, "{reply}");
    reply[0]["value"].clone()
}

/// Declares store `name`: a list is a keyed collection (by `id`).
pub fn store(ui: &mut Ui, name: &str, value: Value) {
    let source = source(name);
    apply(
        ui,
        match &value {
            Value::Array(items) if items.iter().all(Value::is_object) => {
                json!({"op": "collection", "source": source, "key": "id", "items": items})
            }
            _ => json!({"op": "signal", "source": source, "initial": value}),
        },
    );
}

/// A new value for scalar store `name`.
pub fn set_store(ui: &mut Ui, name: &str, value: Value) {
    apply(
        ui,
        json!({"op": "set", "source": source(name), "value": value}),
    );
}

/// Store `name`'s value now: what a control wrote back to it.
pub fn read_store(ui: &mut Ui, name: &str) -> Value {
    apply(ui, json!({"op": "read", "source": source(name)}))["value"].clone()
}

/// A keyed edit `op` (`insert`, `remove`, `move`, `patch`) of list store
/// `name`.
pub fn edit_store(ui: &mut Ui, name: &str, mut op: Value) {
    op["source"] = json!(source(name));
    apply(ui, op);
}

/// Mounts `document` as `mount` with the test stores `stores`, declared
/// here.
pub fn mount_stores(ui: &mut Ui, mount: SourceId, document: &str, stores: &[(&str, Value)]) {
    let mut inputs = BTreeMap::new();
    for (name, value) in stores {
        store(ui, name, value.clone());
        inputs.insert(name.to_string(), source(name));
    }
    ui.mount_document(mount, document, inputs, Default::default(), &bind::NoEnv)
        .unwrap_or_else(|e| panic!("mounts: {e}"));
}

/// Mounts the screen `screen` under its name, as a push does, with the
/// test stores `stores`.
pub fn mount_screen(ui: &mut Ui, screen: &str, stores: &[(&str, Value)]) {
    mount_stores(ui, layers::pushed(screen), screen, stores);
}

/// [`mount_screen`] over the host protocol, as a scope's `mount` is: the
/// screen compiles at the next build, in that build's environment (its
/// string tables).
pub fn register_screen(ui: &mut Ui, screen: &str, stores: &[(&str, Value)]) {
    let mut inputs = serde_json::Map::new();
    for (name, value) in stores {
        store(ui, name, value.clone());
        inputs.insert(name.to_string(), json!(source(name)));
    }
    apply(
        ui,
        json!({"op": "mount", "mount": layers::pushed(screen), "document": screen, "inputs": inputs}),
    );
}
