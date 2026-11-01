//! The live tree for the probe, and the system that runs the UI in the tick.

use crate::input::{InputFrame, OgInput};
use crate::ui::{Ui, Viewport};
use og_ecs::{SystemContext, SystemDesc, World};
use og_probe::{UiElement, UiTree};
use std::sync::{Arc, Mutex, MutexGuard};

/// A [`Ui`] shared between the system that builds it (inside the tick), the
/// probe (between ticks) and the renderer (after the tick).
#[derive(Clone, Debug, Default)]
pub struct SharedUi(Arc<Mutex<Ui>>);

impl SharedUi {
    pub fn new(ui: Ui) -> SharedUi {
        SharedUi(Arc::new(Mutex::new(ui)))
    }

    /// The UI; a panic inside a build closure does not lock it out.
    pub fn lock(&self) -> MutexGuard<'_, Ui> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// The probe hook: `probe.set_ui(shared.probe())`.
    pub fn probe(&self) -> Box<dyn UiTree> {
        Box::new(UiProbe(self.clone()))
    }

    /// A system that builds the UI once per tick from the world's `og.Input`
    /// (declared as a read): `world.add_system(desc, run)` with the pair this
    /// returns. `viewport` gives the play surface for the tick (the host's
    /// current canvas size, DPR and safe area).
    pub fn system<V, F>(
        &self,
        name: &str,
        viewport: V,
        build: F,
    ) -> (
        SystemDesc,
        impl Fn(&mut SystemContext<'_>) -> og_core::Result<()> + Send + 'static + use<V, F>,
    )
    where
        V: Fn() -> Viewport + Send + 'static,
        F: Fn(&mut Ui, &SystemContext<'_>) -> og_core::Result<()> + Send + 'static,
    {
        let shared = self.clone();
        let desc = SystemDesc::new(name)
            .read::<OgInput>()
            .read::<og_input::TextInput>()
            .read::<og_input::A11yAction>();
        let run = move |ctx: &mut SystemContext<'_>| {
            let typed = ctx.events::<og_input::TextInput>().unwrap_or_default();
            let asked = ctx.events::<og_input::A11yAction>().unwrap_or_default();
            let input = match ctx.resource::<OgInput>() {
                Ok(i) => InputFrame::read(&*i),
                Err(_) => InputFrame::default(),
            }
            .with_text(&typed)
            .with_a11y(&asked);
            let mut ui = shared.lock();
            ui.begin(input, viewport());
            let built = build(&mut ui, ctx);
            ui.end();
            built
        };
        (desc, run)
    }
}

/// [`UiTree`] over a [`SharedUi`]: the elements of its last completed frame.
#[derive(Clone, Debug)]
pub struct UiProbe(pub SharedUi);

impl UiTree for UiProbe {
    fn elements(&self, _world: &World) -> Vec<UiElement> {
        self.0.lock().probe_elements()
    }

    fn extra(&self, _world: &World) -> Option<serde_json::Map<String, serde_json::Value>> {
        Some(self.0.lock().probe_extra())
    }

    fn surface(&self, _world: &World) -> Option<(f32, f32, f32)> {
        let v = self.0.lock().viewport();
        Some((v.width, v.height, if v.dpr > 0.0 { v.dpr } else { 1.0 }))
    }

    fn style(&self, _world: &World) -> Option<String> {
        Some(self.0.lock().style_name().to_owned())
    }
}

impl Ui {
    /// The last frame's elements with what documents, layers and motion
    /// know of each (og-probe's `UiElement::detail`, docs/ui.md §16.1).
    pub fn probe_elements(&self) -> Vec<UiElement> {
        use serde_json::{Value, json};
        let mut out = self.frame().elements.clone();
        if self.docs.built.is_empty()
            && self.frame().motion.is_empty()
            && self.docs.layers.layers.is_empty()
        {
            return out;
        }
        let layers: Vec<String> = self
            .docs
            .layers
            .layers
            .iter()
            .map(|l| l.screen.clone())
            .collect();
        let built: std::collections::BTreeMap<&str, &crate::doc::build::Built> = self
            .docs
            .built
            .iter()
            .map(|b| (b.path.as_str(), b))
            .collect();
        let motion: std::collections::BTreeMap<&str, &crate::motion::MotionDump> = self
            .frame()
            .motion
            .iter()
            .map(|m| (m.path.as_str(), m))
            .collect();
        let arts: std::collections::BTreeMap<&str, &crate::slot::SlotSpec> = self
            .docs
            .arts
            .iter()
            .map(|(p, s)| (p.as_str(), s))
            .collect();
        for e in &mut out {
            let d = &mut e.detail;
            let first = e.path.split('/').next().unwrap_or("");
            if layers.iter().any(|l| l == first) {
                d.insert("layer".into(), json!(first));
            }
            if let Some(b) = built.get(e.path.as_str()) {
                let source = self
                    .document(&b.doc)
                    .map_or(String::new(), |doc| doc.source.clone());
                d.insert("source".into(), json!(format!("{source}:{}", b.line)));
                if !b.bindings.is_empty() {
                    d.insert(
                        "bindings".into(),
                        Value::Array(
                            b.bindings
                                .iter()
                                .map(|(p, x, v)| json!({ "prop": p, "expr": x, "value": v }))
                                .collect(),
                        ),
                    );
                }
                if !b.edits.is_empty() {
                    d.insert(
                        "edits".into(),
                        Value::Object(b.edits.iter().map(|(p, v)| (p.clone(), json!(v))).collect()),
                    );
                }
                if self.docs.slots.contains_key(&e.path) {
                    d.insert("slot".into(), json!({ "kind": "code" }));
                }
            }
            if let Some(m) = motion.get(e.path.as_str()) {
                if !m.states.is_empty() {
                    d.insert(
                        "visualStates".into(),
                        Value::Object(
                            m.states
                                .iter()
                                .map(|(g, s)| (g.clone(), json!(s)))
                                .collect(),
                        ),
                    );
                }
                if !m.moving.is_empty() || !m.plays.is_empty() || m.leaving {
                    d.insert(
                        "motion".into(),
                        json!({ "moving": m.moving, "plays": m.plays, "leaving": m.leaving }),
                    );
                }
            }
            if let Some(s) = arts.get(e.path.as_str()) {
                d.insert("slot".into(), crate::doc::build::slot_json(s));
            }
        }
        out
    }

    /// The UI's state beyond its elements for `dump_ui` (docs/ui.md §16.1).
    pub fn probe_extra(&self) -> serde_json::Map<String, serde_json::Value> {
        use serde_json::{Value, json};
        let mut m = serde_json::Map::new();
        m.insert(
            "layers".into(),
            Value::Array(
                self.layer_stack()
                    .iter()
                    .map(|l| {
                        let mut v = l.to_json();
                        if l.top {
                            v["focused"] = json!(self.focus_path(0));
                        }
                        v
                    })
                    .collect(),
            ),
        );
        m.insert(
            "motion".into(),
            Value::Array(self.frame().motion.iter().map(|d| json!({
                "path": d.path,
                "states": d.states.iter().map(|(g, s)| (g.clone(), json!(s))).collect::<serde_json::Map<_, _>>(),
                "moving": d.moving, "plays": d.plays, "leaving": d.leaving,
            })).collect()),
        );
        m.insert("actions".into(), self.actions_json());
        m.insert(
            "documents".into(),
            Value::Array(self.docs.docs.values().map(|d| d.to_json()).collect()),
        );
        m.insert(
            "edits".into(),
            Value::Array(
                self.docs
                    .edits
                    .iter()
                    .map(|e| json!({ "path": e.path, "prop": e.prop, "value": e.source }))
                    .collect(),
            ),
        );
        m
    }
}
