//! The form `input_types` drives and its rig, shared with the frame test
//! (og-tests-ui includes this file from its `input_types`).

use crate::support;
use og_core::glam::Vec2;
use og_ui::reactive::{SourceId, wire};
use og_ui::*;
use serde_json::{Value, json};
use support::*;

pub(crate) const FORM: &str = r#"<meta name="component" content="form">
<column id="Form">
  <input id="Pin" type="password" autocomplete="current-password" bind:value={$pin} />
  <input id="Age" type="number" step="any" bind:value={$age} />
  <input id="Mail" type="email" inputmode="text" bind:value={$mail} />
  <input id="Site" type="url" bind:value={$site} />
</column>
<style>
#Form { gap: 8px; padding: 8px; }
input { width: 300px; }
input:invalid { width: 277px; }
</style>"#;

pub(crate) const STORES: [&str; 4] = ["pin", "age", "mail", "site"];

/// The form mounted retained over four writable stores.
pub(crate) struct Rig {
    pub(crate) ui: Ui,
}

impl Rig {
    pub(crate) fn new() -> Rig {
        Rig::with(FORM, &STORES)
    }

    /// Document `doc` (component `form`) mounted over empty stores `stores`.
    pub(crate) fn with(doc: &str, stores: &[&str]) -> Rig {
        let mut ui = ui_with_font(test_font_metrics());
        ui.define_document(UiDocument::parse("ui/form.svelte", doc).expect("the form parses"));
        let mut rig = Rig { ui };
        let mut inputs = std::collections::BTreeMap::new();
        for &name in stores {
            let source = SourceId::new("test", name);
            rig.apply(json!({"op": "signal", "source": source, "initial": ""}));
            inputs.insert(name.to_string(), source);
        }
        rig.ui
            .mount_document(
                SourceId::new("test", "form"),
                "form",
                inputs,
                Default::default(),
                &bind::NoEnv,
            )
            .expect("the form mounts");
        rig.step(InputFrame::default());
        rig
    }

    pub(crate) fn apply(&mut self, command: Value) -> Value {
        let reply = wire::decode(
            &self
                .ui
                .apply_reactive(&wire::encode(&json!([command])).unwrap()),
        )
        .unwrap();
        assert_eq!(reply[0]["ok"], true, "{reply}");
        reply[0]["value"].clone()
    }

    /// A store's value.
    pub(crate) fn store(&mut self, name: &str) -> Value {
        self.apply(json!({"op": "read", "source": SourceId::new("test", name)}))["value"].clone()
    }

    pub(crate) fn set(&mut self, name: &str, value: Value) {
        self.apply(json!({"op": "set", "source": SourceId::new("test", name), "value": value}));
    }

    pub(crate) fn step(&mut self, input: InputFrame) {
        self.ui.begin(input, Viewport::new(800.0, 600.0));
        self.ui
            .build_retained_documents(&bind::NoEnv)
            .expect("the form builds");
        self.ui.end();
    }

    pub(crate) fn el(&self, id: &str) -> UiElement {
        self.ui
            .element(id)
            .cloned()
            .unwrap_or_else(|| panic!("{id}"))
    }

    pub(crate) fn centre(&self, id: &str) -> Vec2 {
        let e = self.el(id);
        Vec2::new(e.x + e.width / 2.0, e.y + e.height / 2.0)
    }

    pub(crate) fn click(&mut self, at: Vec2) {
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
        // Past the double-click window.
        for _ in 0..40 {
            self.step(InputFrame::default());
        }
    }

    /// Focuses field `id` by a press near its start.
    pub(crate) fn focus(&mut self, id: &str) {
        let e = self.el(id);
        self.click(Vec2::new(e.x + 6.0, e.y + e.height / 2.0));
    }

    pub(crate) fn typed(&mut self, text: &str) {
        let events = og_input::TextInput::split(text, false);
        self.step(InputFrame::default().with_text(&events));
        self.step(InputFrame::default());
    }

    pub(crate) fn chord(&mut self, key: UiKey) {
        self.step(InputFrame {
            keys: vec![key],
            ctrl: true,
            pointer_inside: true,
            ..InputFrame::default()
        });
    }

    pub(crate) fn a11y(&self, path_end: &str) -> A11yNode {
        self.ui
            .a11y()
            .nodes
            .iter()
            .find(|n| n.path.ends_with(path_end))
            .cloned()
            .unwrap_or_else(|| panic!("no a11y node {path_end}"))
    }
}
