//! The form `html_controls` drives and its rig, shared with the frame test
//! (og-tests-ui includes this file from its `html_controls`).

use og_core::glam::Vec2;
use og_ui::reactive::{SourceId, wire};
use og_ui::*;
use serde_json::{Value, json};

pub(crate) const FORM: &str = r#"<meta name="component" content="form">
<column id="Form">
  <button id="Opener">Open</button>
  <details id="More" bind:open={$more}><summary>More options</summary><text id="Body">Hidden text</text></details>
  <details id="Lit"><summary>Literal</summary><text id="LitBody">Shown text</text></details>
  <meter id="Hp" value={$hp} min="0" max="100" low="25" high="75" optimum="90" />
  <dialog id="Ask" modal bind:open={$ask} aria-label="Quit?"><button id="Yes">Yes</button><button id="No">No</button></dialog>
  <dialog id="Note" bind:open={$note} closedby="closerequest" aria-label="Note"><button id="Ok">Ok</button></dialog>
  <select id="Pick" bind:value={$choice}>
    <optgroup label="Warm">
      <option value="r">Red</option>
      <option value="o" disabled>Orange</option>
    </optgroup>
    <optgroup label="Cool">
      <option value="b">Blue</option>
      <option value="c">Cyan</option>
    </optgroup>
  </select>
</column>
<style>
#Form { gap: 8px; padding: 8px; }
select { width: 260px; }
</style>"#;

pub(crate) const STORES: [(&str, Value); 5] = [
    ("choice", Value::Null),
    ("more", Value::Bool(false)),
    ("hp", Value::Null),
    ("ask", Value::Bool(false)),
    ("note", Value::Bool(false)),
];

pub(crate) struct Rig {
    pub(crate) ui: Ui,
}

impl Rig {
    pub(crate) fn new(ui: Ui, initial: &[(&str, Value)]) -> Rig {
        let mut ui = ui;
        ui.define_document(UiDocument::parse("ui/form.svelte", FORM).expect("the form parses"));
        let mut rig = Rig { ui };
        let mut inputs = std::collections::BTreeMap::new();
        for (name, value) in initial {
            let source = SourceId::new("test", name);
            rig.apply(json!({"op": "signal", "source": source, "initial": value}));
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

    pub(crate) fn store(&mut self, name: &str) -> Value {
        self.apply(json!({"op": "read", "source": SourceId::new("test", name)}))["value"].clone()
    }

    pub(crate) fn set(&mut self, name: &str, value: Value) {
        self.apply(json!({"op": "set", "source": SourceId::new("test", name), "value": value}));
        self.step(InputFrame::default());
        self.step(InputFrame::default());
    }

    pub(crate) fn step(&mut self, input: InputFrame) {
        self.step_at(input, Viewport::new(800.0, 900.0));
    }

    pub(crate) fn step_at(&mut self, input: InputFrame, vp: Viewport) {
        self.ui.begin(input, vp);
        self.ui
            .build_retained_documents(&bind::NoEnv)
            .expect("the form builds");
        self.ui.end();
    }

    pub(crate) fn el(&self, id: &str) -> Option<UiElement> {
        self.ui.element(id).cloned()
    }

    pub(crate) fn click(&mut self, id: &str) {
        let e = self.el(id).unwrap_or_else(|| panic!("no {id}"));
        let at = Vec2::new(e.x + e.width / 2.0, e.y + e.height / 2.0);
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
        self.settle();
    }

    /// Past the double-click window and every transition.
    pub(crate) fn settle(&mut self) {
        for _ in 0..40 {
            self.step(InputFrame::default());
        }
    }

    pub(crate) fn key(&mut self, key: UiKey) {
        self.step(InputFrame {
            keys: vec![key],
            ..InputFrame::default()
        });
        self.settle();
    }

    pub(crate) fn pad(&mut self, button: u32) {
        self.step(InputFrame {
            gamepad: GamepadFrame {
                pressed: 1 << button,
                ..GamepadFrame::default()
            },
            ..InputFrame::default()
        });
        self.settle();
    }

    pub(crate) fn typed(&mut self, text: &str) {
        self.step(InputFrame {
            text: text.into(),
            ..InputFrame::default()
        });
        self.settle();
    }

    pub(crate) fn focus(&self) -> String {
        self.ui.focus_path(0).to_string()
    }

    pub(crate) fn a11y(&self, path_end: &str) -> A11yNode {
        self.ui
            .a11y()
            .nodes
            .iter()
            .find(|n| n.path == path_end || n.path.ends_with(&format!("/{path_end}")))
            .cloned()
            .unwrap_or_else(|| panic!("no a11y node {path_end}"))
    }
}
