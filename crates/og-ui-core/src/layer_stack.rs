//! The layer stack's state (docs/ui.md §2.1): its screens, the changes
//! asked of it and what the probe lists ([`crate::layers`] runs it).

use crate::doc::Back;
use crate::doc::{DocKind, InputMode, LayerSpec, Under};
use crate::motion::css::CssAnimation;
use crate::reactive::SourceId;
use crate::ui::Ui;
use serde_json::{Value, json};

/// A layer change a document's trigger (or the probe) asked for, done
/// after the build.
#[derive(Clone, Debug, PartialEq)]
pub enum LayerOp {
    Push(String, Value),
    /// Pops the layer of that serial (0: the active one).
    Pop(u32),
    Replace(String, Value),
    Clear,
    /// Removes the layer of that serial now (its leaving cover closed).
    Drop(u32),
}

/// One screen on the stack.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Layer {
    /// Unique for the UI's life: what a push answers.
    pub serial: u32,
    pub screen: String,
    /// Its viewmodel (two-way bindings write into it); unused by a mounted layer.
    pub vm: Value,
    /// The retained mount it shows: a `type screen` document a scope
    /// mounted with store inputs (`scope.mount`). It is on the stack while
    /// it is mounted, and leaves it, unmounted, when it is popped.
    pub mount: Option<SourceId>,
    /// A mounted layer was emitted last frame (a hidden one is released).
    pub shown: bool,
    /// The tick its view transition's cover started (0: none running).
    pub since: u64,
    /// It is leaving: its view transition covers it before it goes.
    pub leaving: bool,
    /// The focus it had when a layer was pushed over it.
    pub focus: u64,
    /// It was the active layer last tick.
    pub was_top: bool,
    /// The tick it was last built.
    pub built: u64,
}

/// The stack, bottom first.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LayerStack {
    pub layers: Vec<Layer>,
    pub next: u32,
    /// The game's time scale when a pausing layer was pushed (restored when none is left).
    pub held_scale: Option<f32>,
}

/// When the last of `animations` ends, ms after they start.
pub fn ends(animations: &[CssAnimation]) -> f32 {
    animations
        .iter()
        .map(|a| {
            let n = if a.iterations.is_finite() {
                a.iterations
            } else {
                1.0
            };
            a.delay_ms + a.duration_ms * n
        })
        .fold(0.0, f32::max)
}

/// An open layer frame ([`Ui::open_layer_frame`]): what closing it restores
/// and decides.
pub struct LayerFrame {
    pub key: u64,
    pub info: LayerInfo,
    pub layer: Layer,
    pub mount: String,
    pub accepts: bool,
    pub building: u32,
}

/// A layer as the probe lists it (`dump_ui:layers`).
#[derive(Clone, Debug, PartialEq)]
pub struct LayerInfo {
    pub serial: u32,
    pub screen: String,
    pub mount: Option<SourceId>,
    pub spec: LayerSpec,
    pub top: bool,
    /// The top layer that takes input (the topmost with none): it takes
    /// Back, `pop` and focus.
    pub active: bool,
    /// Not built: a layer above hides it.
    pub hidden: bool,
    /// Takes no input: a menu layer is above it.
    pub inert: bool,
}

impl LayerInfo {
    pub fn to_json(&self) -> Value {
        let s = &self.spec;
        json!({
            "serial": self.serial, "screen": self.screen, "kind": s.kind.name(),
            "mount": self.mount.as_ref().map_or(String::new(), |m| format!("{}:{}", m.scope, m.id)),
            "input": s.input.name(), "capture": s.capture, "pause": s.pause,
            "under": s.under.name(),
            "back": match &s.back { Back::Pop => "pop".to_string(), Back::None => "none".into(), Back::Emit(n) => format!("emit {n}") },
            "focus": s.focus, "remember": s.remember, "top": self.top, "active": self.active, "hidden": self.hidden, "inert": self.inert,
        })
    }
}

/// A layer's node id: its screen's name (its probe path's first segment),
/// or for a mounted layer its mount's (`scope:id`).
pub fn layer_id(l: &Layer) -> String {
    match &l.mount {
        Some(m) => format!("{}:{}", m.scope, m.id),
        None => l.screen.clone(),
    }
}

impl Ui {
    pub fn screen_spec(&self, screen: &str) -> Result<LayerSpec, String> {
        match self.document(screen) {
            Some(d) if d.kind == DocKind::Screen => Ok(d.layer.clone()),
            Some(_) => Err(format!(
                "'{screen}' is a component: push a document of type screen"
            )),
            None => Err(format!(
                "no screen '{screen}' (load its document: og.load.document; known: {})",
                self.documents().join(", ")
            )),
        }
    }

    /// The stack, bottom first.
    pub fn layer_stack(&self) -> Vec<LayerInfo> {
        let st = &self.docs.host.layers.layers;
        let specs: Vec<LayerSpec> = st
            .iter()
            .map(|l| {
                self.screen_spec(&l.screen)
                    .unwrap_or_else(|_| LayerSpec::of(Default::default()))
            })
            .collect();
        // A leaving layer is only drawn: it hides, holds and takes nothing.
        let here = |k: usize| !st[k].leaving;
        let hide_from = (0..st.len()).rposition(|k| here(k) && specs[k].under == Under::Hide);
        let menu_at = (0..st.len()).rposition(|k| here(k) && specs[k].input == InputMode::Menu);
        let active = (0..st.len())
            .rposition(|k| here(k) && specs[k].input != InputMode::Game)
            .or_else(|| (0..st.len()).rposition(here));
        st.iter()
            .enumerate()
            .map(|(k, l)| LayerInfo {
                serial: l.serial,
                screen: l.screen.clone(),
                mount: l.mount.clone(),
                spec: specs[k].clone(),
                top: k + 1 == st.len(),
                active: active == Some(k),
                hidden: hide_from.is_some_and(|h| k < h),
                inert: l.leaving || menu_at.is_some_and(|m| k < m),
            })
            .collect()
    }
}

/// A layer's `since` until its view transition's first tick.
pub const STARTS: u64 = u64::MAX;

impl Ui {
    /// Puts a layer into the stack at its kind's place.
    pub fn insert_screen(
        &mut self,
        screen: &str,
        vm: Value,
        mount: Option<SourceId>,
    ) -> Result<u32, String> {
        let spec = self.screen_spec(screen)?;
        let kind = spec.kind;
        // Its cover starts the first tick it is built.
        let since = if spec.transition.is_some() { STARTS } else { 0 };
        let vm = if mount.is_some() {
            Value::Null
        } else if vm.is_null() {
            self.document(screen)
                .and_then(|d| d.fixture("default").cloned())
                .unwrap_or(Value::Object(Default::default()))
        } else {
            vm
        };
        let at = self
            .docs
            .host
            .layers
            .layers
            .iter()
            .rposition(|l| self.screen_spec(&l.screen).is_ok_and(|s| s.kind <= kind))
            .map_or(0, |k| k + 1);
        let st = &mut self.docs.host.layers;
        st.next += 1;
        let serial = st.next;
        st.layers.insert(
            at,
            Layer {
                serial,
                screen: screen.to_string(),
                vm,
                mount,
                since,
                ..Layer::default()
            },
        );
        Ok(serial)
    }

    /// The active layer's index: the top one that takes input, else the topmost.
    pub fn active_layer(&self) -> Option<usize> {
        self.layer_stack().iter().position(|l| l.active)
    }

    /// Keeps a retained mount's layer in step with its document: a
    /// `type screen` document is a layer of the stack for as long as it is
    /// mounted (inserted at its kind's place; the active layer remembers its
    /// focus), any other document none.
    pub fn sync_mount_layer(&mut self, mount: &SourceId, document: Option<&str>) {
        let screen = document.filter(|d| self.screen_spec(d).is_ok());
        let st = &self.docs.host.layers.layers;
        let at = st.iter().position(|l| l.mount.as_ref() == Some(mount));
        if let Some(k) = at {
            if screen == Some(st[k].screen.as_str()) {
                // Mounted again while it was leaving: it stays.
                let l = &mut self.docs.host.layers.layers[k];
                if l.leaving {
                    l.leaving = false;
                    l.since = 0;
                }
                return;
            }
            self.docs.host.layers.layers.remove(k);
        }
        if let Some(screen) = screen {
            if let Some(a) = self.active_layer() {
                self.docs.host.layers.layers[a].focus = self.interaction.state.focus;
            }
            let _ = self.insert_screen(screen, Value::Null, Some(mount.clone()));
        }
    }

    /// After a restore: a mounted layer whose mount is gone leaves the
    /// stack, and a mounted screen without one joins it.
    pub(crate) fn sync_mount_layers(&mut self) {
        let all = self.docs.mounted_documents();
        self.docs.host.layers.layers.retain(|l| {
            l.mount
                .as_ref()
                .is_none_or(|m| all.iter().any(|(a, _)| a == m))
        });
        for (mount, document) in all {
            self.sync_mount_layer(&mount, Some(&document));
        }
    }

    pub(crate) fn restore_docs_state(&mut self, v: &Value) -> Result<(), String> {
        self.docs.state.restore(v)?;
        let d = &mut self.docs.host;
        let key = |v: &Value| v.as_str().and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
        d.layers.next = v["next"].as_u64().unwrap_or(0) as u32;
        d.layers.held_scale = v["heldScale"].as_f64().map(|x| x as f32);
        d.layers.layers = v["layers"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|l| Layer {
                        serial: l["serial"].as_u64().unwrap_or(0) as u32,
                        screen: l["screen"].as_str().unwrap_or_default().to_string(),
                        vm: l["vm"].clone(),
                        focus: key(&l["focus"]),
                        was_top: l["wasTop"].as_bool().unwrap_or(false),
                        mount: serde_json::from_value(l["mount"].clone()).ok(),
                        shown: false,
                        since: l["since"].as_u64().unwrap_or(0),
                        leaving: l["leaving"].as_bool().unwrap_or(false),
                        built: 0,
                    })
                    .collect()
            })
            .unwrap_or_default();
        d.states = v["states"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|s| {
                        (
                            (key(&s[0]), s[1].as_str().unwrap_or_default().to_string()),
                            s[2].as_str().unwrap_or_default().to_string(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        self.sync_mount_layers();
        Ok(())
    }
}

impl crate::ui::Docs {
    /// The stack index of a mounted layer.
    pub fn mount_layer(&self, mount: &SourceId) -> Option<usize> {
        self.host
            .layers
            .layers
            .iter()
            .position(|l| l.mount.as_ref() == Some(mount))
    }

    /// The layer state snapshots carry, as JSON.
    pub(crate) fn docs_state(&self) -> Value {
        let d = &self.host;
        let m = self.state.memory();
        json!({
            "next": d.layers.next,
            "heldScale": d.layers.held_scale,
            "stores": m.reactive.save(),
            "mounts": m.retained.save(),
            "mountRegistrations": m.document_registration_state(),
            "seen": m.seen_now.iter().map(u64::to_string).collect::<Vec<_>>(),
            "layers": d.layers.layers.iter().map(|l| json!({
                "serial": l.serial, "screen": l.screen, "vm": l.vm, "focus": l.focus.to_string(), "wasTop": l.was_top,
                "mount": l.mount, "since": l.since, "leaving": l.leaving,
            })).collect::<Vec<_>>(),
            "states": d.states.iter().map(|((k, g), s)| json!([k.to_string(), g, s])).collect::<Vec<_>>(),
            "last": m.last.iter().map(|((k, i), v)| json!([k.to_string(), i, v])).collect::<Vec<_>>(),
        })
    }
}
