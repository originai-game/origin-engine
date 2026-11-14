//! The form `a11y` reads the tree of, shared with the wire test
//! (og-tests-ui includes this file from its `a11y`): a document mounted
//! as `test:form` over the test stores below.

use crate::support::{mount_stores, read_store};
use og_ui::*;
use serde_json::{Value, json};

/// Every kind of control, named and stated. `Player` is a settings row:
/// the label names the field, the placeholder describes it (lantern-vale
/// FINDINGS #23 heard "Your name").
pub(crate) const FORM: &str = r#"<meta name="component" content="form">
<div id="Root">
<column id="Form">
  <h1 id="Title">Settings</h1>
  <text id="Note">Changes save at once.</text>
  <button id="Save">Save</button>
  <label id="Music"><input type="checkbox" switch bind:checked={$on} /> Music</label>
  <label id="Hints"><input type="checkbox" bind:checked={$check} /> Show hints</label>
  <row id="Gain">
    <text id="Label">Master</text>
    <input type="range" id="Slider" min="0" max="1" step="any" value="0.5" />
  </row>
  <input type="range" id="Volume" min="0" max="100" step="5" bind:value={$volume} />
  <input id="Name" placeholder="Your name" bind:value={$name} />
  <row id="Player">
    <text id="Caption">Lamplighter</text>
    <input id="Field" placeholder="Your name" bind:value={$nick} />
    <textarea id="Memo" placeholder="A note" bind:value={$memo} />
  </row>
  <tabs id="Mode" items="Easy Hard" bind:selected={$tab}></tabs>
  <select id="Quality" bind:value={$choice}>
    <option value="low">Low</option>
    <option value="high">High</option>
  </select>
  <input type="number" id="Count" min="0" max="9" bind:value={$count} />
  <div id="Size" role="radiogroup">
    <label id="Small"><input type="radio" value="small" bind:group={$radio} /> Small</label>
    <label id="Large"><input type="radio" value="large" bind:group={$radio} /> Large</label>
  </div>
  <progress id="Load" value="0.4"></progress>
  <p id="Help">Read the <a href="rules">rules</a>.</p>
  <dialog id="Confirm" modal bind:open={$dialog} aria-label="Delete this save?">
    <button id="Cancel">Cancel</button>
    <button id="Delete">Delete</button>
  </dialog>
</column>
</div>"#;

/// The form's stores, as a game would set them.
pub(crate) struct Form {
    pub(crate) on: bool,
    pub(crate) check: bool,
    pub(crate) volume: f32,
    pub(crate) name: &'static str,
    pub(crate) nick: &'static str,
    pub(crate) tab: usize,
    pub(crate) choice: &'static str,
    pub(crate) count: &'static str,
    pub(crate) radio: &'static str,
    pub(crate) dialog: bool,
}

impl Default for Form {
    fn default() -> Form {
        Form {
            on: false,
            check: false,
            volume: 0.0,
            name: "",
            nick: "",
            tab: 0,
            choice: "low",
            count: "",
            radio: "small",
            dialog: false,
        }
    }
}

/// The mount the form is under.
pub(crate) fn mount() -> og_ui::reactive::SourceId {
    og_ui::reactive::SourceId::new("test", "form")
}

/// `ui` with the form defined and mounted over `f`'s stores.
pub(crate) fn mount_form(ui: &mut Ui, f: &Form) {
    ui.define_document(UiDocument::parse("ui/form.svelte", FORM).expect("the form parses"));
    let stores: [(&str, Value); 11] = [
        ("on", json!(f.on)),
        ("check", json!(f.check)),
        ("volume", json!(f.volume)),
        ("name", json!(f.name)),
        ("nick", json!(f.nick)),
        ("memo", json!("")),
        ("tab", json!(f.tab)),
        ("choice", json!(f.choice)),
        ("count", json!(f.count)),
        ("radio", json!(f.radio)),
        ("dialog", json!(f.dialog)),
    ];
    mount_stores(ui, mount(), "form", &stores);
}

/// Store `name`'s value now: what a control wrote back to it.
#[allow(dead_code)]
pub(crate) fn read(ui: &mut Ui, name: &str) -> Value {
    read_store(ui, name)
}

pub(crate) fn tick(ui: &mut Ui, input: InputFrame) {
    ui.begin(input, Viewport::new(800.0, 1000.0));
    ui.build_retained_documents(&bind::NoEnv)
        .expect("the form builds");
    ui.end();
}

/// The node at `path` under the form's mount (`Form/Volume/Track` is
/// `test:form/Form/Volume/Track`).
pub(crate) fn node<'a>(tree: &'a A11yTree, path: &str) -> &'a A11yNode {
    let full = format!("test:form/{path}");
    tree.nodes
        .iter()
        .find(|n| n.path == full)
        .unwrap_or_else(|| panic!("no node {full} in {:#?}", tree.nodes))
}
