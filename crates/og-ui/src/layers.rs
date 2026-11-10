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

/// Whether an animation is one of the engine's covers (`-og-iris`, …).
fn is_cover(a: &CssAnimation) -> bool {
    crate::doc::view_transition::cover(&a.name).is_some()
}

/// When the last of `animations` ends, ms after they start.
pub(crate) fn ends(animations: &[CssAnimation]) -> f32 {
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

/// A layer's `since` until its view transition's first tick.
const STARTS: u64 = u64::MAX;

/// An open layer frame ([`Ui::open_layer_frame`]): what closing it restores
/// and decides.
pub(crate) struct LayerFrame {
    key: u64,
    info: LayerInfo,
    layer: Layer,
    mount: String,
    accepts: bool,
    building: u32,
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
        self.insert_screen(screen, vm, None)
    }

    /// Puts a layer into the stack at its kind's place.
    fn insert_screen(
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
                mount,
                since,
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
        Some(self.remove_layer(k).screen)
    }

    /// Replaces the active layer (the others keep their focus memory).
    pub fn replace_screen(&mut self, screen: &str, vm: Value) -> Result<u32, String> {
        self.screen_spec(screen)?;
        if let Some(k) = self.active_layer() {
            self.remove_layer(k);
        }
        self.insert_screen(screen, vm, None)
    }

    /// Empties the stack (mounted layers are unmounted), with no transition.
    pub fn clear_screens(&mut self) {
        for l in std::mem::take(&mut self.docs.layers.layers) {
            if let Some(mount) = &l.mount {
                self.finish_unmount(mount);
            }
        }
    }

    /// Removes layer `k` from the stack (a mounted one is unmounted), or,
    /// when its screen has a view transition, starts it leaving: the cover
    /// closes over it before it goes.
    fn remove_layer(&mut self, k: usize) -> Layer {
        let l = self.docs.layers.layers[k].clone();
        if self.layer_leaves(k) {
            return l;
        }
        self.docs.layers.layers.remove(k);
        if let Some(mount) = &l.mount {
            self.finish_unmount(mount);
        }
        l
    }

    /// Starts layer `k` leaving when its screen has a view transition;
    /// true while it is leaving.
    pub(crate) fn layer_leaves(&mut self, k: usize) -> bool {
        let transition = self
            .screen_spec(&self.docs.layers.layers[k].screen)
            .is_ok_and(|s| s.transition.is_some());
        let l = &mut self.docs.layers.layers[k];
        if !transition {
            return false;
        }
        if !l.leaving {
            l.leaving = true;
            l.since = STARTS;
        }
        true
    }

    /// Where layer `k`'s view transition is this tick: whether the layer
    /// shows, and the animations its node plays (`new` while it joins, `old`
    /// while it leaves). Covers named on `old` close first; on a join the
    /// layer shows once they have closed, and `new`'s covers open on it. A
    /// leaving layer is dropped when its `old` animations end, and `new`'s
    /// covers then open on what is under it.
    fn layer_transition(&mut self, k: usize, spec: &LayerSpec) -> (bool, Vec<CssAnimation>) {
        let l = &self.docs.layers.layers[k];
        let (Some(vt), true) = (&spec.transition, l.since != 0) else {
            return (!l.leaving, Vec::new());
        };
        let (mut since, leaving, serial) = (l.since, l.leaving, l.serial);
        if since == STARTS {
            since = self.state.tick;
            let screen = l.screen.clone();
            self.docs.layers.layers[k].since = since;
            self.start_view_transition(&screen, !leaving);
        }
        if self.view_transition_skipped(since) {
            // Skipped (a shared element's name twice): the update happens
            // without its animations.
            if leaving {
                self.docs.ops.push(LayerOp::Drop(serial));
                return (false, Vec::new());
            }
            self.docs.layers.layers[k].since = 0;
            return (true, Vec::new());
        }
        let t = self.state.tick.saturating_sub(since) as f32 * 1000.0
            / self.ticks_per_second.max(1) as f32;
        let (covers, plays): (Vec<_>, Vec<_>) = vt.old.iter().cloned().partition(is_cover);
        if leaving {
            if t < ends(&vt.old) {
                self.draw_covers(&covers, vt.color, vt.image.as_deref(), t, false);
                return (true, plays);
            }
            self.docs.ops.push(LayerOp::Drop(serial));
            let opening: Vec<_> = vt.new.iter().filter(|a| is_cover(a)).cloned().collect();
            if !opening.is_empty() {
                self.draw_covers(&opening, vt.color, vt.image.as_deref(), 0.0, true);
                self.docs
                    .revealing
                    .push((opening, vt.color, vt.image.clone(), self.state.tick));
            }
            return (false, Vec::new());
        }
        let wait = ends(&covers);
        if t < wait {
            self.draw_covers(&covers, vt.color, vt.image.as_deref(), t, false);
            return (false, Vec::new());
        }
        let t = t - wait;
        if t >= ends(&vt.new) {
            self.docs.layers.layers[k].since = 0;
            return (true, Vec::new());
        }
        let (opening, plays): (Vec<_>, Vec<_>) = vt.new.iter().cloned().partition(is_cover);
        self.draw_covers(&opening, vt.color, vt.image.as_deref(), t, true);
        (true, plays)
    }

    /// Draws the cover of the last of `covers` that has started at `t` ms:
    /// closing (0 to 1 over its timing) or opening (1 to 0), in `color`, or
    /// the texture asset `image` tinted by it. False when none is running.
    pub(crate) fn draw_covers(
        &mut self,
        covers: &[CssAnimation],
        color: Option<crate::Srgba>,
        image: Option<&str>,
        t: f32,
        opening: bool,
    ) -> bool {
        let Some(a) = covers.iter().rev().find(|a| t >= a.delay_ms || opening) else {
            return false;
        };
        let Some((kind, from)) = crate::doc::view_transition::cover(&a.name) else {
            return false;
        };
        let p = if a.duration_ms > 0.0 {
            ((t - a.delay_ms) / a.duration_ms).clamp(0.0, 1.0)
        } else {
            f32::from(u8::from(t >= a.delay_ms))
        };
        let p = a.ease.apply(p);
        let texture = image.and_then(|p| self.docs.textures.get(p).copied());
        let desc = crate::ScreenTransitionDesc {
            kind,
            from,
            color: if texture.is_some() {
                Some(color.unwrap_or(crate::Srgba::WHITE))
            } else {
                color
            },
            texture: texture.map(|t| (t, [0.0, 0.0, 1.0, 1.0])),
            ..Default::default()
        };
        self.cover(&desc, if opening { 1.0 - p } else { p }, opening);
        true
    }

    /// Keeps a retained mount's layer in step with its document: a
    /// `type screen` document is a layer of the stack for as long as it is
    /// mounted (inserted at its kind's place; the active layer remembers its
    /// focus), any other document none.
    pub(crate) fn sync_mount_layer(&mut self, mount: &SourceId, document: Option<&str>) {
        let screen = document.filter(|d| self.screen_spec(d).is_ok());
        let st = &self.docs.layers.layers;
        let at = st.iter().position(|l| l.mount.as_ref() == Some(mount));
        if let Some(k) = at {
            if screen == Some(st[k].screen.as_str()) {
                // Mounted again while it was leaving: it stays.
                let l = &mut self.docs.layers.layers[k];
                if l.leaving {
                    l.leaving = false;
                    l.since = 0;
                }
                return;
            }
            self.docs.layers.layers.remove(k);
        }
        if let Some(screen) = screen {
            if let Some(a) = self.active_layer() {
                self.docs.layers.layers[a].focus = self.state.focus;
            }
            let _ = self.insert_screen(screen, Value::Null, Some(mount.clone()));
        }
    }

    /// The stack index of a mounted layer.
    pub(crate) fn mount_layer(&self, mount: &SourceId) -> Option<usize> {
        self.docs
            .layers
            .layers
            .iter()
            .position(|l| l.mount.as_ref() == Some(mount))
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

    /// A layer's viewmodel (the top one of that screen).
    pub fn screen_vm(&self, screen: &str) -> Option<&Value> {
        self.docs
            .layers
            .layers
            .iter()
            .rev()
            .find(|l| l.screen == screen && l.mount.is_none())
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
            .find(|l| l.screen == screen && l.mount.is_none())
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
        if !self
            .docs
            .layers
            .layers
            .iter()
            .any(|l| l.screen == screen && l.mount.is_none())
            && let Err(e) = self.push_screen(screen, Value::Null)
        {
            self.warnings.push(format!("Ui::screen: {e}"));
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
            .rposition(|l| l.screen == screen && l.mount.is_none())
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
        // The covers still opening over layers that have left.
        let mut revealing = std::mem::take(&mut self.docs.revealing);
        revealing.retain(|(covers, color, image, start)| {
            let t =
                tick.saturating_sub(*start) as f32 * 1000.0 / self.ticks_per_second.max(1) as f32;
            t < ends(covers) && self.draw_covers(covers, *color, image.as_deref(), t, true)
        });
        self.docs.revealing = revealing;
        for k in 0..self.docs.layers.layers.len() {
            if self
                .docs
                .layers
                .layers
                .get(k)
                .is_some_and(|l| l.built != tick && l.mount.is_none())
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
                    if let Some(k) = self
                        .docs
                        .layers
                        .layers
                        .iter()
                        .position(|l| l.serial == serial)
                    {
                        self.remove_layer(k);
                    }
                    Ok(())
                }
                LayerOp::Clear => {
                    self.clear_screens();
                    Ok(())
                }
                LayerOp::Drop(serial) => {
                    let st = &mut self.docs.layers.layers;
                    if let Some(k) = st.iter().position(|l| l.serial == serial) {
                        let l = st.remove(k);
                        if let Some(mount) = &l.mount {
                            self.finish_unmount(mount);
                        }
                    }
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
        let (frame, lead) = self.open_layer_frame(k);
        let Some(frame) = frame else {
            return DocResponse {
                events: lead,
                ..DocResponse::default()
            };
        };
        let layer = self.docs.layers.layers[k].clone();
        let doc = self.docs.docs[&layer.screen].clone();
        let mut vm = layer.vm.clone();
        let root_id = match doc.nodes.get(doc.root) {
            Some(n) if !n.id.is_empty() => n.id.clone(),
            _ => "Root".to_string(),
        };
        let mut r = self.mount_doc(
            doc.clone(),
            layer.screen.clone(),
            &root_id,
            &mut vm,
            env,
            Default::default(),
        );
        let mut events = lead;
        events.append(&mut r.events);
        events.extend(self.close_layer_frame(frame, &r.path));
        r.events = events;
        if let Some(l) = self.docs.layers.layers.get_mut(k)
            && l.serial == layer.serial
        {
            l.vm = vm;
        }
        r
    }

    /// Opens layer `k`'s frame: its activation events, and, unless a layer
    /// above hides it, its float with the spec's backdrop, scope and
    /// presence, input routed to it as the stack decides. Its content
    /// follows until [`Ui::close_layer_frame`].
    pub(crate) fn open_layer_frame(&mut self, k: usize) -> (Option<LayerFrame>, Vec<DocEvent>) {
        let infos = self.layer_stack();
        let Some(info) = infos.get(k).cloned() else {
            return (None, Vec::new());
        };
        let tick = self.state.tick;
        let layer = self.docs.layers.layers[k].clone();
        self.docs.layers.layers[k].built = tick;
        self.docs.layers.layers[k].was_top = info.active;
        let mount = layer
            .mount
            .as_ref()
            .map_or(layer.screen.clone(), |m| m.id.clone());
        // CommonUI's OnActivated / OnDeactivated, before the layer's own events.
        let mut lead = Vec::new();
        if info.active != layer.was_top {
            lead.push(DocEvent {
                mount: mount.clone(),
                path: layer_id(&layer),
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
        let (shows, animations) = self.layer_transition(k, &info.spec);
        if info.hidden || !shows || !self.docs.docs.contains_key(&layer.screen) {
            return (None, lead);
        }
        for a in &animations {
            if let Some(frames) = crate::doc::view_transition::builtin_keyframes(&a.name) {
                self.docs
                    .keyframes
                    .entry(a.name.clone())
                    .or_insert_with(|| std::sync::Arc::new(frames));
            }
        }
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
            n.props.animations = animations;
        }
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
        (
            Some(LayerFrame {
                key,
                info,
                layer,
                mount,
                accepts,
                building,
            }),
            lead,
        )
    }

    /// Closes a layer's frame: Back, then the focus a layer that became the
    /// active one takes. `path` is its content root's path.
    pub(crate) fn close_layer_frame(&mut self, frame: LayerFrame, path: &str) -> Vec<DocEvent> {
        let LayerFrame {
            key,
            info,
            layer,
            mount,
            accepts,
            building,
        } = frame;
        self.docs.building = building;
        self.docs.accepts = accepts;
        self.close();
        let spec = &info.spec;
        let mut events = Vec::new();
        // Back: the topmost scope takes it (as a dialog does).
        if info.active && self.nav.back && self.state.scope == key {
            match &spec.back {
                Back::Pop => {
                    self.nav.back = false;
                    self.docs.ops.push(LayerOp::Pop(layer.serial));
                }
                Back::Emit(name) => {
                    self.nav.back = false;
                    events.push(DocEvent {
                        mount,
                        path: path.to_string(),
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
                .then(|| crate::id::key_of_path(&format!("{path}/{}", spec.focus)));
            if let Some(f) = remembered.or(declared) {
                self.request_focus = f;
            }
        }
        events
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
                "mount": l.mount, "since": l.since, "leaving": l.leaving,
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
        self.sync_mount_layers();
        Ok(())
    }
}

/// A layer's node id: its screen's name (its probe path's first segment),
/// or for a mounted layer its mount's (`scope:id`).
pub(crate) fn layer_id(l: &Layer) -> String {
    match &l.mount {
        Some(m) => format!("{}:{}", m.scope, m.id),
        None => l.screen.clone(),
    }
}

/// The host-less build `end` falls back on.
pub(crate) fn build_without_host(ui: &mut Ui) {
    ui.build_layers(&NoEnv);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::UiDocument;
    use crate::reactive::session::Command;
    use crate::{InputFrame, UiDesc, UiKey, Viewport};
    use std::collections::BTreeMap;

    const HUD: &str = r#"<meta name="component" content="hud">
<column id="Hud"><text id="Gold">gold</text><button id="Map">map</button></column>"#;
    const PAUSE: &str = r#"<meta name="screen" content="pause" data-layer="modal" data-pause="true" data-focus="Resume">
<column id="Menu"><text id="Vol">{$volume}</text><button id="Quit">quit</button><button id="Resume">resume</button></column>"#;

    fn ui_with(docs: &[(&str, &str)]) -> Ui {
        let mut ui = Ui::new(UiDesc::default());
        for (name, text) in docs {
            ui.define_document(UiDocument::parse(name, text).unwrap());
        }
        ui.reactive
            .execute(Command::Signal {
                source: SourceId::new("test", "volume"),
                initial: json!("half"),
                readable: false,
            })
            .unwrap();
        ui
    }

    fn mount(ui: &mut Ui, id: &str, document: &str) {
        let inputs = BTreeMap::from([("volume".into(), SourceId::new("test", "volume"))]);
        ui.reactive
            .execute(Command::Signal {
                source: SourceId::new("test", "_"),
                initial: json!(0),
                readable: false,
            })
            .ok();
        ui.register_document_mount(
            SourceId::new("test", id),
            document.into(),
            inputs,
            Default::default(),
        )
        .unwrap();
    }

    fn step(ui: &mut Ui, input: InputFrame) -> crate::LayoutStats {
        ui.begin(input, Viewport::new(640.0, 480.0));
        ui.end().stats
    }

    fn escape() -> InputFrame {
        InputFrame {
            keys: vec![UiKey::Escape],
            keys_held: vec!["Escape"],
            keys_pressed: vec!["Escape"],
            ..InputFrame::default()
        }
    }

    fn texts(ui: &Ui) -> Vec<String> {
        ui.probe_elements()
            .into_iter()
            .filter(|e| e.kind == "label")
            .map(|e| e.text)
            .collect()
    }

    /// A mounted screen document is a layer of the stack while it is
    /// mounted: its store input binds, it pauses and holds input, takes its
    /// declared focus and Back, and an idle frame does no work.
    ///
    /// Ways it could fail: the mount drawn as a plain overlay (no layer,
    /// no pause), focus left on the HUD, Back leaving the mount drawn, or
    /// the layer frame rebuilding the retained tree every tick.
    #[test]
    fn a_mounted_screen_is_a_layer_with_store_inputs() {
        let mut ui = ui_with(&[("hud.svelte", HUD), ("pause.svelte", PAUSE)]);
        mount(&mut ui, "hud", "hud");
        for _ in 0..2 {
            step(&mut ui, InputFrame::default());
        }
        assert!(ui.layer_stack().is_empty(), "a component is not a layer");
        assert!(ui.game_input());
        mount(&mut ui, "pause", "pause");
        for _ in 0..3 {
            step(&mut ui, InputFrame::default());
        }
        let stack = ui.layer_stack();
        assert_eq!(stack.len(), 1);
        assert_eq!(stack[0].mount, Some(SourceId::new("test", "pause")));
        assert!(stack[0].active);
        assert_eq!(stack[0].to_json()["mount"], "test:pause");
        assert!(ui.paused() && !ui.game_input());
        assert_eq!(ui.focus_path(0), "test:pause/Menu/Resume");
        let events = ui.take_doc_events();
        assert!(
            events
                .iter()
                .any(|e| e.kind == "activate" && e.mount == "pause"),
            "{events:?}"
        );
        assert!(texts(&ui).contains(&"half".to_string()));
        // An idle frame evaluates and writes nothing.
        // An idle frame evaluates nothing and writes only the focused
        // control (it reads the focus every frame).
        let idle = step(&mut ui, InputFrame::default());
        assert_eq!((idle.bindings, idle.reconciled), (0, 1), "{idle:?}");
        ui.reactive
            .execute(Command::Set {
                source: SourceId::new("test", "volume"),
                value: json!("full"),
                force: false,
            })
            .unwrap();
        let changed = step(&mut ui, InputFrame::default());
        assert_eq!(changed.bindings, 1, "{changed:?}");
        assert!(texts(&ui).contains(&"full".to_string()));
        // A snapshot carries the mounted layer.
        let saved = ui.save_state();
        // Back pops it: the layer and its mount go, the HUD stays.
        step(&mut ui, escape());
        step(&mut ui, InputFrame::default());
        assert!(ui.layer_stack().is_empty());
        assert!(!ui.paused() && ui.game_input());
        assert_eq!(texts(&ui), ["gold"]);
        ui.restore_state(&saved).unwrap();
        for _ in 0..2 {
            step(&mut ui, InputFrame::default());
        }
        assert_eq!(ui.layer_stack().len(), 1);
        assert!(texts(&ui).contains(&"full".to_string()), "{:?}", texts(&ui));
        // Unmounting removes the layer too.
        ui.unmount_document(&SourceId::new("test", "pause"));
        step(&mut ui, InputFrame::default());
        assert!(ui.layer_stack().is_empty());
        assert_eq!(texts(&ui), ["gold"]);
    }

    /// By default a layer cross-fades as it joins and leaves (the UA's
    /// fade-in and fade-out, 0.25s): its node's opacity rises from 0, and a
    /// leaving layer fades out before it goes.
    #[test]
    fn a_view_transition_cross_fades_a_layer_by_default() {
        let page = format!("{PAUSE}<style>@view-transition {{ navigation: auto; }}</style>");
        let mut ui = ui_with(&[("hud.svelte", HUD), ("pause.svelte", &page)]);
        mount(&mut ui, "pause", "pause");
        let opacity = |ui: &Ui| {
            ui.nodes
                .iter()
                .find(|n| n.id == "test:pause")
                .map(|n| (n.sampled.opacity, n.props.animations.len()))
        };
        let ticks = ui.ticks_for(250.0) as usize;
        let mut seen = Vec::new();
        for _ in 0..ticks + 3 {
            step(&mut ui, InputFrame::default());
            seen.push(opacity(&ui).unwrap());
        }
        assert!(seen[0].0 < 0.2 && seen[0].1 == 1, "{seen:?}");
        assert!(
            seen.windows(2).take(ticks - 1).all(|w| w[1].0 > w[0].0),
            "{seen:?}"
        );
        assert_eq!(*seen.last().unwrap(), (1.0, 0), "{seen:?}");
        assert!(texts(&ui).contains(&"half".to_string()));
        ui.unmount_document(&SourceId::new("test", "pause"));
        let mut seen = Vec::new();
        for _ in 0..ticks + 3 {
            step(&mut ui, InputFrame::default());
            seen.push((opacity(&ui), ui.layer_stack().len()));
        }
        assert!(seen[0].0.unwrap().0 > 0.8 && seen[0].1 == 1, "{seen:?}");
        assert!(seen[ticks - 2].0.unwrap().0 < 0.2, "{seen:?}");
        assert!(
            seen.last().is_some_and(|(o, n)| o.is_none() && *n == 0),
            "{seen:?}"
        );
        assert!(ui.document_mount_inputs().as_array().unwrap().is_empty());
    }

    /// `::view-transition { background-image: url("…") }`: the cover shows
    /// the texture the path is bound to, tinted white with no colour.
    ///
    /// Ways it could fail: the image dropped by the parser, an unbound path
    /// that draws a texture of nothing, the backdrop colour darkening it.
    #[test]
    fn a_view_transition_covers_with_its_texture() {
        let page = format!(
            "{PAUSE}<style>@view-transition {{ navigation: auto; }}\n::view-transition-old(root) {{ animation: 400ms linear -og-iris; }}\n::view-transition {{ background-image: url(\"ui/covers/stars.png\"); }}</style>"
        );
        let mut ui = ui_with(&[("hud.svelte", HUD), ("pause.svelte", &page)]);
        let stars = og_draw::TextureHandle::from_bits(9);
        ui.define_texture("ui/covers/stars.png", stars);
        mount(&mut ui, "hud", "hud");
        step(&mut ui, InputFrame::default());
        mount(&mut ui, "pause", "pause");
        step(&mut ui, InputFrame::default());
        step(&mut ui, InputFrame::default());
        let c = ui.cover.expect("the cover closes");
        assert_eq!(c.texture.map(|t| t.0), Some(stars));
        assert_eq!(c.color, crate::Srgba::WHITE);
    }

    /// A cover named on `old` closes over the outgoing view, and the
    /// incoming view shows once it has closed; on `new` it opens. A
    /// leaving layer stays under the closing cover, then goes.
    #[test]
    fn view_transition_covers_close_on_old_and_open_on_new() {
        let page = format!(
            "{PAUSE}<style>@view-transition {{ navigation: auto; }}\n::view-transition-old(root), ::view-transition-new(root) {{ animation: 400ms linear -og-wipe-from-right; }}\n::view-transition {{ background-color: #102030; }}</style>"
        );
        let mut ui = ui_with(&[("hud.svelte", HUD), ("pause.svelte", &page)]);
        mount(&mut ui, "hud", "hud");
        step(&mut ui, InputFrame::default());
        mount(&mut ui, "pause", "pause");
        let half = ui.ticks_for(400.0) as usize;
        assert!(half > 2);
        // (shown, (revealing, kind, colour)) per tick.
        let mut seen = Vec::new();
        for _ in 0..2 * half + 2 {
            step(&mut ui, InputFrame::default());
            let shown = texts(&ui).contains(&"half".to_string());
            seen.push((shown, ui.cover.map(|c| (c.revealing, c.kind, c.from))));
        }
        // Hidden while the cover closes (from nothing, on its first tick).
        let covering = seen.iter().take_while(|(s, _)| !s).count();
        assert_eq!(covering, half, "{seen:?}");
        assert!(
            seen[1..half].iter().all(|(_, c)| c.is_some_and(|c| !c.0)),
            "{seen:?}"
        );
        assert_eq!(
            seen[half].1,
            Some((true, crate::ScreenWipe::Wipe, crate::Side::Right)),
            "{seen:?}"
        );
        assert!(seen[half].0, "{seen:?}");
        assert!(
            seen.last().is_some_and(|(s, c)| *s && c.is_none()),
            "{seen:?}"
        );
        ui.unmount_document(&SourceId::new("test", "pause"));
        let mut seen = Vec::new();
        for _ in 0..2 * half + 2 {
            step(&mut ui, InputFrame::default());
            seen.push((
                texts(&ui).contains(&"half".to_string()),
                ui.layer_stack().len(),
                ui.cover.map(|c| c.revealing),
            ));
        }
        assert!(seen[1].0 && seen[1].2 == Some(false), "{seen:?}");
        let gone = seen.iter().position(|(s, ..)| !s).unwrap();
        assert_eq!(gone, half, "{seen:?}");
        assert!(
            seen[gone..].iter().all(|(s, n, _)| !s && *n == 0),
            "{seen:?}"
        );
        assert_eq!(seen[gone].2, Some(true));
        assert!(seen.last().unwrap().2.is_none(), "{seen:?}");
        assert!(ui.document_mount_inputs().as_array().unwrap().len() == 1);
    }

    /// The frame's solid red shapes (the shared element), on the screen.
    fn reds(ui: &Ui) -> Vec<crate::layout::Rect> {
        ui.frame
            .items
            .iter()
            .filter_map(|it| match &it.cmd {
                crate::DrawCmd::Quad { rect, color, .. }
                    if color[0] > 0.9 && color[1] < 0.05 && color[2] < 0.05 =>
                {
                    Some(*rect)
                }
                crate::DrawCmd::Shape { shape, .. } => match shape.paint {
                    og_draw::UiPaint::Solid(c) if c[0] > 0.9 && c[1] < 0.05 && c[2] < 0.05 => {
                        Some(crate::layout::Rect::new(
                            shape.center.x - shape.half.x,
                            shape.center.y - shape.half.y,
                            2.0 * shape.half.x,
                            2.0 * shape.half.y,
                        ))
                    }
                    _ => None,
                },
                _ => None,
            })
            .collect()
    }

    /// A named element morphs from its old box to its new one across a
    /// navigation: its group starts on the old element, moves by its
    /// timing (styled by class here), cross-fades the two images there,
    /// and hides both live elements until it ends. A name on two elements
    /// skips the transition, as reduced motion does.
    #[test]
    fn a_view_transition_name_morphs_one_element_into_another() {
        const LIST: &str = r#"<meta name="screen" content="list"><column id="Page"><column id="Card"></column></column>
<style>@view-transition { navigation: auto; } #Page { padding: 40px; }
#Card { width: 100px; height: 50px; background: #ff0000; view-transition-name: hero; }</style>"#;
        let detail = |twice: bool| {
            format!(
                r#"<meta name="screen" content="detail"><column id="Page"><column id="Card"></column>{}</column>
<style>@view-transition {{ navigation: auto; }} #Page {{ padding: 200px 0 0 300px; }}
#Card, #Again {{ width: 200px; height: 100px; background: #ff0000; view-transition-name: hero; view-transition-class: card; }}
::view-transition-group(*.card) {{ animation-duration: 400ms; animation-timing-function: linear; }}</style>"#,
                if twice {
                    r#"<column id="Again"></column>"#
                } else {
                    ""
                }
            )
        };
        let run = |twice: bool, reduced: bool| {
            let mut ui = ui_with(&[("list.svelte", LIST), ("detail.svelte", &detail(twice))]);
            let step = |ui: &mut Ui, _: InputFrame| {
                let mut viewport = Viewport::new(640.0, 480.0);
                viewport.reduced_motion = reduced;
                ui.begin(InputFrame::default(), viewport);
                ui.end();
            };
            ui.push_screen("list", Value::Null).unwrap();
            for _ in 0..ui.ticks_for(300.0) + 2 {
                step(&mut ui, InputFrame::default());
            }
            let before = reds(&ui);
            ui.replace_screen("detail", Value::Null).unwrap();
            let mut seen = Vec::new();
            for _ in 0..ui.ticks_for(400.0) + 3 {
                step(&mut ui, InputFrame::default());
                seen.push((reds(&ui), ui.frame.warnings.clone(), ui.layer_stack().len()));
            }
            (before, seen)
        };
        let (before, seen) = run(false, false);
        assert_eq!(before.len(), 1, "{before:?}");
        let (old, after) = (before[0], seen.last().unwrap().0[0]);
        assert_eq!(seen.last().unwrap().0.len(), 1, "{seen:?}");
        assert!(
            after.x > old.x + 100.0 && after.width > 1.5 * old.width,
            "{old:?} {after:?}"
        );
        let near = |a: &crate::layout::Rect, b: &crate::layout::Rect| {
            (a.x - b.x).abs() < 1.0
                && (a.y - b.y).abs() < 1.0
                && (a.width - b.width).abs() < 1.0
                && (a.height - b.height).abs() < 1.0
        };
        // Its first tick: the old image on the old box (the new one at 0),
        // and neither live element.
        assert!(
            seen[0].0.len() == 1 && near(&seen[0].0[0], &old),
            "{seen:?}"
        );
        // Halfway, linear: both images on the box halfway.
        let half = ui_ticks(400.0) / 2;
        let mid = crate::layout::Rect::new(
            (old.x + after.x) / 2.0,
            (old.y + after.y) / 2.0,
            (old.width + after.width) / 2.0,
            (old.height + after.height) / 2.0,
        );
        assert_eq!(seen[half].0.len(), 2, "{seen:?}");
        assert!(
            seen[half].0.iter().all(|r| near(r, &mid)),
            "{mid:?} {:?}",
            seen[half].0
        );
        assert!(seen.iter().all(|(_, w, _)| w.is_empty()), "{seen:?}");
        // A restore halfway lands on the same frames.
        let mut ui = ui_with(&[("list.svelte", LIST), ("detail.svelte", &detail(false))]);
        ui.push_screen("list", Value::Null).unwrap();
        for _ in 0..ui.ticks_for(300.0) + 2 {
            step(&mut ui, InputFrame::default());
        }
        ui.replace_screen("detail", Value::Null).unwrap();
        for _ in 0..=half {
            step(&mut ui, InputFrame::default());
        }
        let saved = ui.save_state();
        step(&mut ui, InputFrame::default());
        let next = reds(&ui);
        ui.restore_state(&saved).unwrap();
        step(&mut ui, InputFrame::default());
        assert_eq!(
            (reds(&ui), &next),
            (seen[half + 1].0.clone(), &seen[half + 1].0)
        );

        // Two elements named hero in the new view: skipped, with a warning;
        // the leaving layer goes at once and both cards show where they are.
        let (_, seen) = run(true, false);
        assert!(
            seen[0]
                .1
                .iter()
                .any(|w| w.contains("'hero' is on two elements of the new view")),
            "{seen:?}"
        );
        assert!(
            seen[2..].iter().all(|(r, _, n)| r.len() == 2 && *n == 1),
            "{seen:?}"
        );

        // Reduced motion: no morph; the new card is where it is at once
        // (the leaving layer's card where it was, until the layer goes).
        let (_, seen) = run(false, true);
        assert!(
            seen.iter().all(|(r, ..)| r.iter().any(|r| near(r, &after))
                && r.iter().all(|r| near(r, &after) || near(r, &old))),
            "{seen:?}"
        );
    }

    /// The image pair keeps each image's aspect at the group's width, so a
    /// new image taller than the group overflows it, as the UA stylesheet
    /// has it; `overflow` on the group or the image pair cuts what it
    /// holds to its box, along the axes it names.
    #[test]
    fn overflow_on_a_view_transition_group_clips_its_images() {
        const LIST: &str = r#"<meta name="screen" content="list"><column id="Page"><column id="Card"></column></column>
<style>@view-transition { navigation: auto; } #Page { padding: 40px; }
#Card { width: 100px; height: 50px; background: #ff0000; view-transition-name: hero; }</style>"#;
        let detail = |rule: &str| {
            format!(
                r#"<meta name="screen" content="detail"><column id="Page"><column id="Card"></column></column>
<style>@view-transition {{ navigation: auto; }} #Page {{ padding: 40px; }}
#Card {{ width: 100px; height: 100px; background: #ff0000; view-transition-name: hero; }}
::view-transition-group(hero) {{ animation-duration: 400ms; animation-timing-function: linear; }}
{rule}</style>"#
            )
        };
        // The cuts of the frame halfway, as (centre, half extent).
        let halfway = |rule: &str| {
            let mut ui = ui_with(&[("list.svelte", LIST), ("detail.svelte", &detail(rule))]);
            let step = |ui: &mut Ui| {
                ui.begin(InputFrame::default(), Viewport::new(640.0, 480.0));
                ui.end();
            };
            ui.push_screen("list", Value::Null).unwrap();
            for _ in 0..ui.ticks_for(300.0) + 2 {
                step(&mut ui);
            }
            ui.replace_screen("detail", Value::Null).unwrap();
            for _ in 0..=ui_ticks(400.0) / 2 {
                step(&mut ui);
            }
            let cuts: Vec<((f32, f32), (f32, f32))> = ui
                .frame
                .items
                .iter()
                .filter_map(|it| match it.cmd {
                    crate::DrawCmd::PushClip { center, half, .. } => Some((center, half)),
                    _ => None,
                })
                .collect();
            (cuts, reds(&ui))
        };
        // Unclipped: the new image (100 × 100) reaches below the group.
        let (cuts, reds) = halfway("");
        assert!(cuts.is_empty(), "{cuts:?}");
        let top = reds.iter().map(|r| r.y).fold(f32::MAX, f32::min);
        assert!(
            reds.iter().any(|r| (r.height - 100.0).abs() < 1.0),
            "{reds:?}"
        );
        // The group halfway is 100 × 75 at the cards' top left.
        let near =
            |a: (f32, f32), b: (f32, f32)| (a.0 - b.0).abs() < 1.0 && (a.1 - b.1).abs() < 1.0;
        for rule in [
            "::view-transition-group(hero) { overflow: clip; }",
            "::view-transition-image-pair(*) { overflow: hidden; }",
        ] {
            let (cuts, _) = halfway(rule);
            assert!(
                cuts.len() == 1
                    && near(cuts[0].0, (90.0, top + 37.5))
                    && near(cuts[0].1, (50.0, 37.5)),
                "{rule}: {cuts:?}"
            );
        }
        // One axis: the other is left open.
        let (cuts, _) = halfway("::view-transition-group(hero) { overflow-y: clip; }");
        assert!(
            cuts.len() == 1 && cuts[0].1.0 > 1e6 && near((0.0, cuts[0].1.1), (0.0, 37.5)),
            "{cuts:?}"
        );
    }

    /// `view-transition-group` nests a group in an ancestor's: the card's
    /// group rides inside the box's `::view-transition-group-children`,
    /// placed in the box's padding box as it moves, and the box's image
    /// leaves the card out (its own group). Without nesting the card's
    /// group is the screen's; `overflow` on the children clips them to
    /// the box's padding box.
    #[test]
    fn view_transition_group_nests_a_group_in_its_ancestors() {
        let page = |screen: &str, page: &str, side: f32, at: (f32, f32), rule: &str| {
            format!(
                r#"<meta name="screen" content="{screen}"><column id="Page"><column id="Box"><column id="Card"></column></column></column>
<style>@view-transition {{ navigation: auto; }} #Page {{ padding: {page}; }}
#Box {{ width: {side}px; height: {side}px; border: 10px solid #0000ff; view-transition-name: box; }}
#Card {{ width: 100px; height: 50px; margin: {}px 0 0 {}px; background: #ff0000; view-transition-name: hero; }}
::view-transition-group(box) {{ animation-duration: 400ms; animation-timing-function: linear; }}
::view-transition-group(hero) {{ animation-duration: 1ms; }}
{rule}</style>"#,
                at.1, at.0
            )
        };
        let halfway = |rule: &str| {
            let list = page("list", "40px", 200.0, (20.0, 20.0), rule);
            let detail = page("detail", "200px 0 0 300px", 240.0, (60.0, 100.0), rule);
            let mut ui = ui_with(&[("list.svelte", &list), ("detail.svelte", &detail)]);
            let step = |ui: &mut Ui| {
                ui.begin(InputFrame::default(), Viewport::new(640.0, 480.0));
                ui.end();
            };
            ui.push_screen("list", Value::Null).unwrap();
            for _ in 0..ui.ticks_for(300.0) + 2 {
                step(&mut ui);
            }
            ui.replace_screen("detail", Value::Null).unwrap();
            for _ in 0..=ui_ticks(400.0) / 2 {
                step(&mut ui);
            }
            assert!(ui.frame.warnings.is_empty(), "{:?}", ui.frame.warnings);
            let cuts: Vec<((f32, f32), (f32, f32))> = ui
                .frame
                .items
                .iter()
                .filter_map(|it| match it.cmd {
                    crate::DrawCmd::PushClip { center, half, .. } => Some((center, half)),
                    _ => None,
                })
                .collect();
            (reds(&ui), cuts)
        };
        let near = |r: &crate::layout::Rect, x: f32, y: f32| {
            (r.x - x).abs() < 1.0
                && (r.y - y).abs() < 1.0
                && (r.width - 100.0).abs() < 1.0
                && (r.height - 50.0).abs() < 1.0
        };
        // The box halfway is (170, 120) 220 × 220, its padding box from
        // (180, 130): nested, the card (done moving) is at its new place in
        // it, (60, 100) in; flat, at its new place on the screen. Only the
        // card's own group draws it: neither of the box's images does.
        for (rule, x, y) in [
            ("#Box { view-transition-group: contain; }", 240.0, 230.0),
            ("#Card { view-transition-group: nearest; }", 240.0, 230.0),
            ("#Card { view-transition-group: box; }", 240.0, 230.0),
            ("#Card { view-transition-group: elsewhere; }", 370.0, 310.0),
            ("", 370.0, 310.0),
        ] {
            let (reds, cuts) = halfway(rule);
            assert!(reds.len() == 1 && near(&reds[0], x, y), "{rule}: {reds:?}");
            assert!(cuts.is_empty(), "{rule}: {cuts:?}");
        }
        let (_, cuts) = halfway(
            "#Box { view-transition-group: contain; } ::view-transition-group-children(box) { overflow: clip; }",
        );
        assert!(
            cuts.len() == 1
                && (cuts[0].0.0 - 280.0).abs() < 1.0
                && (cuts[0].0.1 - 230.0).abs() < 1.0
                && (cuts[0].1.0 - 100.0).abs() < 1.0
                && (cuts[0].1.1 - 100.0).abs() < 1.0,
            "{cuts:?}"
        );
    }

    fn ui_ticks(ms: f32) -> usize {
        Ui::new(UiDesc::default()).ticks_for(ms) as usize
    }
}
