//! The layer stack (docs/ui.md §2.1): a game's UI is a stack of screens,
//! each a document of `type screen` declaring its layer (HUD, menu, modal,
//! popover, toast, tooltip, system), its input mode, its default focus and
//! focus memory, what Back does, whether it pauses the game and what
//! happens to the layers under it. [`Ui::push_screen`], `pop_screen`,
//! `replace_screen` and `clear_screens` manage it; [`Ui::screen`] builds a
//! layer where code calls it, and [`Ui::build_layers`] builds the rest once
//! per tick, in stack order, before the frame is laid out.
//!
//! The stack is ordered by layer kind, then by push order (Unreal
//! CommonUI's layer groups): a modal pushed before a menu stays over it, a
//! toast or tooltip over both. The **active** layer is the top one whose
//! input mode is `menu` or `all` (the topmost with none), CommonUI's
//! active activatable: it takes Back, `pop` and focus, keeps the focus it
//! had while one is above it, and hears `activate` and `deactivate` as it
//! becomes and stops being the active one. Input goes to it once per tick;
//! a layer whose input mode is `menu` or `all` is a focus scope; the layers
//! under a `menu` layer are inert; a capturing layer takes the pointer on
//! its gaps too. [`Ui::game_input`] tells gameplay whether input is its
//! this tick.

use crate::bind::{Env, NoEnv};
use crate::doc::build::{DocEvent, DocResponse};
use crate::doc::{Back, DocKind, InputMode, LayerSpec, Under};
use crate::layout::{Direction, Element, Float, NONE};
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
}

/// One screen on the stack.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Layer {
    /// Unique for the UI's life: what a push answers.
    pub serial: u32,
    pub screen: String,
    /// Its viewmodel (two-way bindings write into it).
    pub vm: Value,
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

/// A layer as the probe lists it (`dump_ui:layers`).
#[derive(Clone, Debug, PartialEq)]
pub struct LayerInfo {
    pub serial: u32,
    pub screen: String,
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
            "input": s.input.name(), "capture": s.capture, "pause": s.pause,
            "under": s.under.name(),
            "back": match &s.back { Back::Pop => "pop".to_string(), Back::None => "none".into(), Back::Emit(n) => format!("emit {n}") },
            "focus": s.focus, "remember": s.remember, "top": self.top, "active": self.active, "hidden": self.hidden, "inert": self.inert,
        })
    }
}

impl Ui {
    fn screen_spec(&self, screen: &str) -> Result<LayerSpec, String> {
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

    /// Pushes a screen with its viewmodel, over the layers of its kind and
    /// below those of a later kind; the active layer remembers its focus.
    /// Answers the layer's serial.
    pub fn push_screen(&mut self, screen: &str, vm: Value) -> Result<u32, String> {
        if let Some(a) = self.active_layer() {
            self.docs.layers.layers[a].focus = self.state.focus;
        }
        self.insert_screen(screen, vm)
    }

    /// Puts a layer into the stack at its kind's place.
    fn insert_screen(&mut self, screen: &str, vm: Value) -> Result<u32, String> {
        let kind = self.screen_spec(screen)?.kind;
        let vm = if vm.is_null() {
            self.document(screen)
                .and_then(|d| d.fixture("default").cloned())
                .unwrap_or(Value::Object(Default::default()))
        } else {
            vm
        };
        let at = self
            .docs
            .layers
            .layers
            .iter()
            .rposition(|l| self.screen_spec(&l.screen).is_ok_and(|s| s.kind <= kind))
            .map_or(0, |k| k + 1);
        let st = &mut self.docs.layers;
        st.next += 1;
        let serial = st.next;
        st.layers.insert(
            at,
            Layer {
                serial,
                screen: screen.to_string(),
                vm,
                ..Layer::default()
            },
        );
        Ok(serial)
    }

    /// The active layer's index: the top one that takes input, else the topmost.
    fn active_layer(&self) -> Option<usize> {
        self.layer_stack().iter().position(|l| l.active)
    }

    /// Pops the active layer; answers its screen.
    pub fn pop_screen(&mut self) -> Option<String> {
        let k = self.active_layer()?;
        Some(self.docs.layers.layers.remove(k).screen)
    }

    /// Replaces the active layer (the others keep their focus memory).
    pub fn replace_screen(&mut self, screen: &str, vm: Value) -> Result<u32, String> {
        self.screen_spec(screen)?;
        if let Some(k) = self.active_layer() {
            self.docs.layers.layers.remove(k);
        }
        self.insert_screen(screen, vm)
    }

    /// Empties the stack.
    pub fn clear_screens(&mut self) {
        self.docs.layers.layers.clear();
    }

    /// The stack, bottom first.
    pub fn layer_stack(&self) -> Vec<LayerInfo> {
        let st = &self.docs.layers.layers;
        let specs: Vec<LayerSpec> = st
            .iter()
            .map(|l| {
                self.screen_spec(&l.screen)
                    .unwrap_or_else(|_| LayerSpec::of(Default::default()))
            })
            .collect();
        let hide_from = specs.iter().rposition(|s| s.under == Under::Hide);
        let menu_at = specs.iter().rposition(|s| s.input == InputMode::Menu);
        let active = specs
            .iter()
            .rposition(|s| s.input != InputMode::Game)
            .or(st.len().checked_sub(1));
        st.iter()
            .enumerate()
            .map(|(k, l)| LayerInfo {
                serial: l.serial,
                screen: l.screen.clone(),
                spec: specs[k].clone(),
                top: k + 1 == st.len(),
                active: active == Some(k),
                hidden: hide_from.is_some_and(|h| k < h),
                inert: menu_at.is_some_and(|m| k < m),
            })
            .collect()
    }

    /// A layer's viewmodel (the top one of that screen).
    pub fn screen_vm(&self, screen: &str) -> Option<&Value> {
        self.docs
            .layers
            .layers
            .iter()
            .rev()
            .find(|l| l.screen == screen)
            .map(|l| &l.vm)
    }

    /// Sets a layer's viewmodel (the top one of that screen); false when it is not on the stack.
    pub fn set_screen_vm(&mut self, screen: &str, vm: Value) -> bool {
        match self
            .docs
            .layers
            .layers
            .iter_mut()
            .rev()
            .find(|l| l.screen == screen)
        {
            Some(l) => {
                l.vm = vm;
                true
            }
            None => false,
        }
    }

    /// Some layer pauses the game.
    pub fn paused(&self) -> bool {
        self.layer_stack().iter().any(|l| l.spec.pause)
    }

    /// The time scale the game should take now, given its current one: 0
    /// once a pausing layer is on the stack (the current one kept), the
    /// kept one once none is; `None` when nothing changes.
    pub fn pause_time_scale(&mut self, current: f32) -> Option<f32> {
        let paused = self.paused();
        let st = &mut self.docs.layers;
        match (paused, st.held_scale) {
            (true, None) => {
                st.held_scale = Some(current);
                Some(0.0)
            }
            (false, Some(s)) => {
                st.held_scale = None;
                Some(s)
            }
            _ => None,
        }
    }

    /// The input mode of the top layer that takes input (`game` with none).
    pub fn input_mode(&self) -> InputMode {
        self.layer_stack()
            .iter()
            .rev()
            .map(|l| l.spec.input)
            .find(|m| *m != InputMode::Game)
            .unwrap_or(InputMode::Game)
    }

    /// Gameplay takes input this tick: no menu layer holds it.
    pub fn game_input(&self) -> bool {
        self.input_mode() != InputMode::Menu
    }

    /// Mounts a screen as a layer (pushed when it is not on the stack),
    /// with its viewmodel for this tick, and builds it now, so code fills
    /// its slots after this call. Its events since the last call come back.
    pub fn screen(&mut self, screen: &str, vm: Value, env: &dyn Env) -> DocResponse {
        if !self.docs.layers.layers.iter().any(|l| l.screen == screen)
            && let Err(e) = self.push_screen(screen, Value::Null)
        {
            self.warnings.push(format!("og.ui.screen: {e}"));
            return DocResponse::default();
        }
        if !vm.is_null() {
            self.set_screen_vm(screen, vm);
        }
        let k = self
            .docs
            .layers
            .layers
            .iter()
            .rposition(|l| l.screen == screen)
            .unwrap_or(0);
        let mut r = self.build_layer(k, env);
        let mut queued = Vec::new();
        self.docs.queued.retain(|e| {
            if e.mount == screen {
                queued.push(e.clone());
                false
            } else {
                true
            }
        });
        queued.append(&mut r.events);
        r.events = queued;
        r
    }

    /// Builds every layer code did not build this tick (bottom first);
    /// their events wait for code ([`Ui::take_doc_events`]). Then the layer
    /// changes triggers asked for. `end` calls it (with no host) when the
    /// host did not.
    pub fn build_layers(&mut self, env: &dyn Env) {
        let tick = self.state.tick;
        if self.docs.built_tick == tick {
            return;
        }
        if !self.docs.retained.is_empty()
            && let Err(error) = self.build_retained_documents(env)
        {
            self.warnings.push(error);
        }
        self.docs.built_tick = tick;
        for k in 0..self.docs.layers.layers.len() {
            if self
                .docs
                .layers
                .layers
                .get(k)
                .is_some_and(|l| l.built != tick)
            {
                let r = self.build_layer(k, env);
                self.docs.queued.extend(r.events);
            }
        }
        for op in std::mem::take(&mut self.docs.ops) {
            let done = match op {
                LayerOp::Push(s, vm) => self.push_screen(&s, vm).map(|_| ()),
                LayerOp::Replace(s, vm) => self.replace_screen(&s, vm).map(|_| ()),
                LayerOp::Pop(0) => {
                    self.pop_screen();
                    Ok(())
                }
                LayerOp::Pop(serial) => {
                    self.docs.layers.layers.retain(|l| l.serial != serial);
                    Ok(())
                }
                LayerOp::Clear => {
                    self.clear_screens();
                    Ok(())
                }
            };
            if let Err(e) = done {
                self.warnings.push(e);
            }
        }
        // Layers draw in stack order, whoever built them first.
        let order: Vec<String> = self.docs.layers.layers.iter().map(layer_id).collect();
        let rank = |ui: &Ui, f: usize| {
            let n = &ui.nodes[f];
            if n.float == Float::Modal && ui.kinds[f].0 == "layer" {
                order.iter().position(|o| *o == n.id).map_or(0, |p| p + 1)
            } else {
                0
            }
        };
        let mut floats = std::mem::take(&mut self.floating);
        let layered: Vec<usize> = floats
            .iter()
            .copied()
            .filter(|&f| rank(self, f) > 0)
            .collect();
        if layered.len() > 1 {
            let mut sorted = layered.clone();
            sorted.sort_by_key(|&f| rank(self, f));
            let mut it = sorted.into_iter();
            for f in floats.iter_mut() {
                if layered.contains(f) {
                    *f = it.next().unwrap_or(*f);
                }
            }
        }
        self.floating = floats;
    }

    /// Builds layer `k` of the stack.
    fn build_layer(&mut self, k: usize, env: &dyn Env) -> DocResponse {
        let infos = self.layer_stack();
        let Some(info) = infos.get(k).cloned() else {
            return DocResponse::default();
        };
        let tick = self.state.tick;
        let layer = self.docs.layers.layers[k].clone();
        self.docs.layers.layers[k].built = tick;
        self.docs.layers.layers[k].was_top = info.active;
        // CommonUI's OnActivated / OnDeactivated, before the layer's own events.
        let mut lead = Vec::new();
        if info.active != layer.was_top {
            lead.push(DocEvent {
                mount: layer.screen.clone(),
                path: layer.screen.clone(),
                row_key: String::new(),
                kind: if info.active {
                    "activate"
                } else {
                    "deactivate"
                }
                .into(),
                name: String::new(),
                value: Value::Null,
            });
        }
        let doc = match self.docs.docs.get(&layer.screen) {
            Some(d) if !info.hidden => d.clone(),
            _ => {
                return DocResponse {
                    events: lead,
                    ..DocResponse::default()
                };
            }
        };
        let spec = &info.spec;
        let t = self.theme;
        let dim = matches!(spec.under, Under::Dim | Under::Blur);
        let idx = self.open_layer(
            "layer",
            crate::Id::from(layer_id(&layer).as_str()),
            Element {
                direction: Direction::Column,
                background: dim.then_some(t.palette.scrim),
                ..Element::default()
            },
            Float::Modal,
            NONE,
        );
        let key = self.nodes[idx].key;
        {
            let n = &mut self.nodes[idx];
            n.backdrop = spec.capture || dim;
            n.scope = spec.input != InputMode::Game;
            n.props.inert = info.inert;
            n.props.enter = spec.enter.clone();
            n.props.exit = spec.exit.clone();
        }
        let mut vm = layer.vm.clone();
        let root_id = match doc.nodes.get(doc.root) {
            Some(n) if !n.id.is_empty() => n.id.clone(),
            _ => "Root".to_string(),
        };
        // UI actions go to the top layer that takes input; with none, to every game layer.
        let top_taker = infos.iter().rposition(|l| l.spec.input != InputMode::Game);
        let accepts = std::mem::replace(
            &mut self.docs.accepts,
            match top_taker {
                Some(t) => t == k,
                None => true,
            },
        );
        let building = std::mem::replace(&mut self.docs.building, layer.serial);
        let mut r = self.mount_doc(
            doc.clone(),
            layer.screen.clone(),
            &root_id,
            &mut vm,
            env,
            Default::default(),
        );
        self.docs.building = building;
        self.docs.accepts = accepts;
        lead.append(&mut r.events);
        r.events = lead;
        self.close();
        // Back: the topmost scope takes it (as a dialog does).
        if info.active && self.nav.back && self.state.scope == key {
            match &spec.back {
                Back::Pop => {
                    self.nav.back = false;
                    self.docs.ops.push(LayerOp::Pop(layer.serial));
                }
                Back::Emit(name) => {
                    self.nav.back = false;
                    r.events.push(DocEvent {
                        mount: layer.screen.clone(),
                        path: r.path.clone(),
                        row_key: String::new(),
                        kind: "back".into(),
                        name: name.clone(),
                        value: Value::Null,
                    });
                }
                Back::None => {}
            }
        }
        // Focus: the layer that became the active one takes its remembered
        // focus, else its declared one (else its first control, the scope's rule).
        if info.active && !layer.was_top {
            let remembered = (spec.remember && layer.focus != 0).then_some(layer.focus);
            let declared = (!spec.focus.is_empty())
                .then(|| crate::id::key_of_path(&format!("{}/{}", r.path, spec.focus)));
            if let Some(f) = remembered.or(declared) {
                self.request_focus = f;
            }
        }
        if let Some(l) = self.docs.layers.layers.get_mut(k)
            && l.serial == layer.serial
        {
            l.vm = vm;
        }
        r
    }

    /// The layer state snapshots carry, as JSON.
    pub(crate) fn docs_state(&self) -> Value {
        let d = &self.docs;
        json!({
            "next": d.layers.next,
            "heldScale": d.layers.held_scale,
            "stores": self.reactive.save(),
            "mounts": d.retained.save(),
            "mountRegistrations": self.document_registration_state(),
            "seen": d.seen_now.iter().map(u64::to_string).collect::<Vec<_>>(),
            "layers": d.layers.layers.iter().map(|l| json!({
                "serial": l.serial, "screen": l.screen, "vm": l.vm, "focus": l.focus.to_string(), "wasTop": l.was_top,
            })).collect::<Vec<_>>(),
            "states": d.states.iter().map(|((k, g), s)| json!([k.to_string(), g, s])).collect::<Vec<_>>(),
            "last": d.last.iter().map(|((k, i), v)| json!([k.to_string(), i, v])).collect::<Vec<_>>(),
        })
    }

    pub(crate) fn restore_docs_state(&mut self, v: &Value) -> Result<(), String> {
        let retained = self.restore_document_mounts(&v["mounts"], &v["mountRegistrations"])?;
        if let Some(stores) = v.get("stores") {
            self.reactive.restore(stores)?;
        } else {
            self.reactive = Default::default();
        }
        self.reactive.graph.clear_bindings();
        let d = &mut self.docs;
        d.retained = retained;
        d.built_tick = 0;
        let key = |v: &Value| v.as_str().and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
        d.seen_now = v["seen"]
            .as_array()
            .map(|a| a.iter().map(key).collect())
            .unwrap_or_default();
        d.seen = d.seen_now.clone();
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
        d.last = v["last"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|s| {
                        (
                            (key(&s[0]), s[1].as_u64().unwrap_or(0) as usize),
                            s[2].clone(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(())
    }
}

/// A layer's node id: its screen's name (its probe path's first segment).
fn layer_id(l: &Layer) -> String {
    l.screen.clone()
}

/// The host-less build `end` falls back on.
pub(crate) fn build_without_host(ui: &mut Ui) {
    ui.build_layers(&NoEnv);
}
