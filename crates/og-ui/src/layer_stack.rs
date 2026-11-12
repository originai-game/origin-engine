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

/// An open layer frame ([`Ui::open_layer_frame`]): what closing it restores
/// and decides.
pub(crate) struct LayerFrame {
    pub(crate) key: u64,
    pub(crate) info: LayerInfo,
    pub(crate) layer: Layer,
    pub(crate) mount: String,
    pub(crate) accepts: bool,
    pub(crate) building: u32,
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
pub(crate) fn layer_id(l: &Layer) -> String {
    match &l.mount {
        Some(m) => format!("{}:{}", m.scope, m.id),
        None => l.screen.clone(),
    }
}

impl Ui {
    pub(crate) fn screen_spec(&self, screen: &str) -> Result<LayerSpec, String> {
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
