//! One document built either way a document is: immediately
//! (`Ui::component`, a viewmodel) or mounted (`mount_document`, named
//! stores, retained). The document tests share it.

use super::{test_font_metrics, ui_with_font};
use og_core::glam::Vec2;
use og_ui::prelude::*;
use og_ui::reactive::{SourceId, wire};
use og_ui::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// One document, built either way (other document tests share it).
pub(crate) struct Doc {
    pub(crate) ui: Ui,
    name: String,
    /// Mounted with these stores; `None`: built immediately from `vm`.
    pub(crate) mounted: bool,
    vm: Value,
    /// An immediate build's events, collected tick by tick.
    pub(crate) events: Vec<DocEvent>,
    /// The viewport each step lays out in (800 × 900 unless a test resizes it).
    pub(crate) viewport: Viewport,
    /// A path the next step focuses once it is built (`Ui::set_focus`).
    pub(crate) focus: Option<String>,
}

impl Doc {
    pub(crate) fn new(text: &str, stores: &[(&str, Value)], mounted: bool) -> Doc {
        Doc::with_font(text, stores, mounted, test_font_metrics())
    }

    /// [`Doc::new`] with `font` (one with a handle draws its text).
    pub(crate) fn with_font(
        text: &str,
        stores: &[(&str, Value)],
        mounted: bool,
        font: UiFont,
    ) -> Doc {
        let mut ui = ui_with_font(font);
        let doc = UiDocument::parse("ui/t.svelte", text).unwrap_or_else(|e| panic!("{e:?}"));
        let name = doc.name.clone();
        ui.define_document(doc);
        let mut d = Doc {
            ui,
            name,
            mounted,
            events: Vec::new(),
            viewport: Viewport::new(800.0, 900.0),
            focus: None,
            // An immediate build reads the stores as `$name` from its viewmodel.
            vm: Value::Object(
                stores
                    .iter()
                    .map(|(k, v)| (format!("${k}"), v.clone()))
                    .collect(),
            ),
        };
        if mounted {
            let mut inputs = BTreeMap::new();
            for (k, v) in stores {
                let source = SourceId::new("test", k);
                d.store(k, v.clone());
                inputs.insert(k.to_string(), source);
            }
            d.ui.mount_document(
                SourceId::new("test", "t"),
                &d.name.clone(),
                inputs,
                Default::default(),
                &bind::NoEnv,
            )
            .unwrap_or_else(|e| panic!("mounts: {e}"));
        }
        d.step(InputFrame::default());
        d
    }

    fn apply(&mut self, command: Value) -> Value {
        let reply = wire::decode(
            &self
                .ui
                .apply_reactive(&wire::encode(&json!([command])).unwrap()),
        )
        .unwrap();
        assert_eq!(reply[0]["ok"], true, "{reply}");
        reply[0]["value"].clone()
    }

    /// Store `name`'s value now: what a control wrote back to it.
    pub(crate) fn read(&mut self, name: &str) -> Value {
        if self.mounted {
            self.apply(json!({"op": "read", "source": SourceId::new("test", name)}))["value"]
                .clone()
        } else {
            self.vm[format!("${name}")].clone()
        }
    }

    /// A store: a list is a keyed collection (by `id`).
    pub(crate) fn store(&mut self, name: &str, value: Value) {
        let source = SourceId::new("test", name);
        self.apply(match &value {
            Value::Array(items) => {
                json!({"op": "collection", "source": source, "key": "id", "items": items})
            }
            _ => json!({"op": "signal", "source": source, "initial": value}),
        });
    }

    /// A new value for scalar store `name`, then two steps.
    pub(crate) fn set(&mut self, name: &str, value: Value) {
        if self.mounted {
            self.apply(json!({"op": "set", "source": SourceId::new("test", name), "value": value}));
        } else {
            self.vm[format!("${name}")] = value;
        }
        self.step(InputFrame::default());
        self.step(InputFrame::default());
    }

    /// A keyed edit of list store `name`: `op` on a mounted collection,
    /// the whole list `after` in an immediate viewmodel.
    pub(crate) fn edit(&mut self, name: &str, mut op: Value, after: Value) {
        if self.mounted {
            op["source"] = json!(SourceId::new("test", name));
            self.apply(op);
        } else {
            self.vm[format!("${name}")] = after;
        }
        self.step(InputFrame::default());
        self.step(InputFrame::default());
    }

    pub(crate) fn step(&mut self, input: InputFrame) {
        self.ui.begin(input, self.viewport);
        if self.mounted {
            self.ui
                .build_retained_documents(&bind::NoEnv)
                .expect("builds");
        } else {
            let name = self.name.clone();
            let r = self.ui.component("T", &name, &mut self.vm, &bind::NoEnv);
            self.events.extend(r.events);
        }
        if let Some(path) = self.focus.take() {
            self.ui.set_focus(&path);
        }
        self.ui.end();
        self.events.extend(self.ui.take_doc_events());
    }

    pub(crate) fn el(&self, id: &str) -> Option<UiElement> {
        self.ui
            .probe_elements()
            .into_iter()
            .find(|e| e.id == id || e.path.ends_with(&format!("/{id}")))
    }

    pub(crate) fn get(&self, id: &str) -> UiElement {
        self.el(id)
            .unwrap_or_else(|| panic!("no {id} (mounted: {})", self.mounted))
    }

    pub(crate) fn a11y(&self, path_end: &str) -> A11yNode {
        self.ui
            .a11y()
            .nodes
            .iter()
            .find(|n| n.path.ends_with(&format!("/{path_end}")) || n.path == path_end)
            .cloned()
            .unwrap_or_else(|| panic!("no a11y node {path_end}"))
    }

    pub(crate) fn click_at(&mut self, at: Vec2) -> Vec<DocEvent> {
        let mut events = std::mem::take(&mut self.events);
        events.clear();
        for (pressed, released) in [(true, false), (false, true)] {
            self.step(InputFrame {
                pointer: at,
                pointer_inside: true,
                down: pressed,
                pressed,
                released,
                ..InputFrame::default()
            });
        }
        for _ in 0..30 {
            self.step(InputFrame::default());
        }
        events.append(&mut self.events);
        events
    }

    pub(crate) fn click(&mut self, id: &str) -> Vec<DocEvent> {
        let e = self.get(id);
        self.click_at(Vec2::new(e.x + e.width / 2.0, e.y + e.height / 2.0))
    }

    /// The markers drawn under `list`, top to bottom: a number's text, or
    /// the shape a bullet is drawn as (`disc`, `circle`, `square`).
    pub(crate) fn markers(&self, list: &str) -> Vec<String> {
        let elements = self.ui.probe_elements();
        let mut m: Vec<_> = elements
            .iter()
            .filter(|e| e.id == "Marker" && e.path.contains(&format!("/{list}/")))
            .collect();
        m.sort_by(|a, b| a.y.total_cmp(&b.y));
        m.into_iter()
            .map(|e| {
                if !e.text.is_empty() {
                    return e.text.clone();
                }
                elements
                    .iter()
                    .find(|c| c.path.starts_with(&format!("{}/", e.path)))
                    .map_or_else(String::new, |c| c.id.clone())
            })
            .collect()
    }
}

pub(crate) fn id_text(id: &str) -> &str {
    match id {
        "Title" => "Title",
        "Sub" => "Sub",
        "Para" => "Para",
        "Note" => "Note",
        _ => "Core",
    }
}

pub(crate) fn both(text: &str, stores: &[(&str, Value)], check: impl Fn(&mut Doc)) {
    for mounted in [false, true] {
        check(&mut Doc::new(text, stores, mounted));
    }
}
