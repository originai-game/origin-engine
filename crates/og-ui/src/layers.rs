//! The layer stack (docs/ui.md §2.1): a game's UI is a stack of screens,
//! each a document of `type screen` declaring its layer (HUD, menu, modal,
//! popover, toast, tooltip, system), its input mode, its default focus and
//! focus memory, what Back does, whether it pauses the game and what
//! happens to the layers under it. Every layer is a mounted document: a
//! screen a scope mounted with its stores, or one pushed by name with a
//! viewmodel, whose values its mount holds. [`Ui::push_screen`],
//! `pop_screen`, `replace_screen` and `clear_screens` manage the stack, and
//! [`Ui::build_layers`] builds the mounted documents once per tick, before
//! the frame is laid out.
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

use crate::bind::Env;
use crate::doc::build::DocEvent;
use crate::doc::{Back, InputMode, LayerSpec, Under};
pub use crate::layer_stack::{Layer, LayerInfo, LayerOp, LayerStack, pushed};
pub(crate) use crate::layer_stack::{LayerFrame, STARTS, ends, layer_id};
use crate::layout::{Direction, Element, Float, NONE};
use crate::motion::css::CssAnimation;
use crate::ui::Ui;
use serde_json::Value;

/// Whether an animation is one of the engine's covers (`-og-iris`, …).
fn is_cover(a: &CssAnimation) -> bool {
    crate::doc::view_transition::cover(&a.name).is_some()
}

impl Ui {
    /// Pushes a screen with its viewmodel, over the layers of its kind and
    /// below those of a later kind; the active layer remembers its focus.
    /// The screen is mounted under its name ([`pushed`]) holding the
    /// viewmodel's values as its stores (its `default` fixture's when
    /// `vm` is null), so it is on the stack once. Answers the layer's
    /// serial.
    pub fn push_screen(&mut self, screen: &str, vm: Value) -> Result<u32, String> {
        self.screen_spec(screen)?;
        let mount = pushed(screen);
        if let Some(k) = self.docs.mount_layer(&mount) {
            if !self.docs.layers.layers[k].leaving {
                return Err(format!("the screen '{screen}' is already on the stack"));
            }
            // Pushed again while its cover closes over it: it goes now.
            self.docs.layers.layers.remove(k);
            self.finish_unmount(&mount);
        }
        let values = match vm {
            Value::Null => self
                .document(screen)
                .and_then(|d| d.fixture("default").cloned())
                .unwrap_or_default(),
            vm => vm,
        };
        self.mount_with_values(mount.clone(), screen, &values)?;
        let k = self.docs.mount_layer(&mount).expect("a mounted screen");
        Ok(self.docs.layers.layers[k].serial)
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
        self.push_screen(screen, vm)
    }

    /// Empties the stack (its screens are unmounted), with no transition.
    pub fn clear_screens(&mut self) {
        for l in std::mem::take(&mut self.docs.layers.layers) {
            self.finish_unmount(&l.mount);
        }
    }

    /// The viewmodel of the screen pushed by that name, as it is now: what
    /// it was pushed with and what its two-way bindings wrote since.
    pub fn screen_vm(&self, screen: &str) -> Option<Value> {
        self.mount_values(&pushed(screen))
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

    /// Builds the mounted documents, the stack's layers among them; their
    /// events wait for code ([`Ui::take_doc_events`]). Then the layer
    /// changes triggers asked for. `end` calls it (with no host) when the
    /// host did not.
    pub fn build_layers(&mut self, env: &dyn Env) {
        let tick = self.interaction.state().tick;
        if self.docs.state().built_tick == tick {
            return;
        }
        if !self.docs.state().memory.retained.is_empty()
            && let Err(error) = self.build_retained_documents(env)
        {
            self.out.warnings_mut().push(error);
        }
        self.docs.state_mut().built_tick = tick;
        // The covers still opening over layers that have left.
        let mut revealing = std::mem::take(&mut self.docs.state_mut().revealing);
        revealing.retain(|(covers, color, image, start)| {
            let t = tick.saturating_sub(*start) as f32 * 1000.0
                / self.env.ticks_per_second().max(1) as f32;
            t < ends(covers) && self.draw_covers(covers, *color, image.as_deref(), t, true)
        });
        self.docs.state_mut().revealing = revealing;
        for op in std::mem::take(&mut self.docs.state_mut().ops) {
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
                        self.finish_unmount(&l.mount);
                    }
                    Ok(())
                }
            };
            if let Err(e) = done {
                self.out.warnings_mut().push(e);
            }
        }
        // Layers draw in stack order, whoever built them first, each under
        // the floats its own elements opened (a popover, a dialog, a
        // spotlight): those stay written while their layer is opened anew.
        let order: Vec<String> = self.docs.layers.layers.iter().map(layer_id).collect();
        let floats = std::mem::take(self.tree.floating_mut());
        let owners = &self.docs.state().memory.retained.owner_of;
        let ranks: Vec<Option<(usize, bool)>> = floats
            .iter()
            .map(|&f| {
                let n = &self.tree.nodes()[f];
                if n.float == Float::Modal && self.tree.kinds()[f].0 == "layer" {
                    return order.iter().position(|o| *o == n.id).map(|p| (p, false));
                }
                let mount = &owners.get(&n.key)?.mount;
                self.docs.mount_layer(mount).map(|p| (p, true))
            })
            .collect();
        let mut ranked: Vec<(usize, (usize, bool))> = floats
            .iter()
            .zip(&ranks)
            .filter_map(|(&f, rank)| rank.map(|rank| (f, rank)))
            .collect();
        ranked.sort_by_key(|&(_, rank)| rank);
        let mut ranked = ranked.into_iter();
        *self.tree.floating_mut() = floats
            .into_iter()
            .zip(&ranks)
            .map(|(f, rank)| match rank {
                Some(_) => ranked.next().map_or(f, |(f, _)| f),
                None => f,
            })
            .collect();
    }
}

impl Ui {
    /// Removes layer `k` from the stack, unmounted, or, when its screen
    /// has a view transition, starts it leaving: the cover closes over it
    /// before it goes.
    pub(crate) fn remove_layer(&mut self, k: usize) -> Layer {
        let l = self.docs.layers.layers[k].clone();
        if self.layer_leaves(k) {
            return l;
        }
        self.docs.layers.layers.remove(k);
        self.finish_unmount(&l.mount);
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
    pub(crate) fn layer_transition(
        &mut self,
        k: usize,
        spec: &LayerSpec,
    ) -> (bool, Vec<CssAnimation>) {
        let l = &self.docs.layers.layers[k];
        let (Some(vt), true) = (&spec.transition, l.since != 0) else {
            return (!l.leaving, Vec::new());
        };
        let (mut since, leaving, serial) = (l.since, l.leaving, l.serial);
        if since == STARTS {
            since = self.interaction.state().tick;
            let screen = l.screen.clone();
            self.docs.layers.layers[k].since = since;
            self.style.start_view_transition(
                &screen,
                !leaving,
                &self.env,
                &self.paint,
                &mut self.interaction,
            );
        }
        if self.interaction.view_transition_skipped(since) {
            // Skipped (a shared element's name twice): the update happens
            // without its animations.
            if leaving {
                self.docs.state_mut().ops.push(LayerOp::Drop(serial));
                return (false, Vec::new());
            }
            self.docs.layers.layers[k].since = 0;
            return (true, Vec::new());
        }
        let t = self.interaction.state().tick.saturating_sub(since) as f32 * 1000.0
            / self.env.ticks_per_second().max(1) as f32;
        let (covers, plays): (Vec<_>, Vec<_>) = vt.old.iter().cloned().partition(is_cover);
        if leaving {
            if t < ends(&vt.old) {
                self.draw_covers(&covers, vt.color, vt.image.as_deref(), t, false);
                return (true, plays);
            }
            self.docs.state_mut().ops.push(LayerOp::Drop(serial));
            let opening: Vec<_> = vt.new.iter().filter(|a| is_cover(a)).cloned().collect();
            if !opening.is_empty() {
                self.draw_covers(&opening, vt.color, vt.image.as_deref(), 0.0, true);
                self.docs.state_mut().revealing.push((
                    opening,
                    vt.color,
                    vt.image.clone(),
                    self.interaction.state().tick,
                ));
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
        let texture = image.and_then(|p| self.docs.library.textures.get(p).copied());
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

    /// Opens layer `k`'s frame: its activation events, and, unless a layer
    /// above hides it, its float with the spec's backdrop, scope and
    /// presence, input routed to it as the stack decides. Its content
    /// follows until [`Ui::close_layer_frame`].
    pub(crate) fn open_layer_frame(&mut self, k: usize) -> (Option<LayerFrame>, Vec<DocEvent>) {
        let infos = self.layer_stack();
        let Some(info) = infos.get(k).cloned() else {
            return (None, Vec::new());
        };
        let tick = self.interaction.state().tick;
        let layer = self.docs.layers.layers[k].clone();
        self.docs.layers.layers[k].built = tick;
        self.docs.layers.layers[k].was_top = info.active;
        let mount = layer.mount.id.clone();
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
        if info.hidden || !shows || !self.docs.library.documents.contains_key(&layer.screen) {
            return (None, lead);
        }
        for a in &animations {
            if let Some(frames) = crate::doc::view_transition::builtin_keyframes(&a.name) {
                self.docs
                    .library
                    .keyframes
                    .entry(a.name.clone())
                    .or_insert_with(|| std::sync::Arc::new(frames));
            }
        }
        let spec = &info.spec;
        let t = *self.env.theme();
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
        let key = self.tree.nodes()[idx].key;
        {
            self.tree
                .capture(idx, spec.capture || dim, spec.input != InputMode::Game);
            let props = self.tree.props_mut(idx);
            props.inert = info.inert;
            props.enter = spec.enter.clone();
            props.exit = spec.exit.clone();
            props.animations = animations;
        }
        // UI actions go to the top layer that takes input; with none, to every game layer.
        let top_taker = infos.iter().rposition(|l| l.spec.input != InputMode::Game);
        let accepts = std::mem::replace(
            &mut self.docs.state_mut().accepts,
            match top_taker {
                Some(t) => t == k,
                None => true,
            },
        );
        let building = std::mem::replace(&mut self.docs.state_mut().building, layer.serial);
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
        self.docs.state_mut().building = building;
        self.docs.state_mut().accepts = accepts;
        self.close();
        let spec = &info.spec;
        let mut events = Vec::new();
        // Back: the topmost scope takes it (as a dialog does).
        if info.active && self.interaction.nav().back && self.interaction.state().scope == key {
            match &spec.back {
                Back::Pop => {
                    self.interaction.nav_mut().back = false;
                    self.docs.state_mut().ops.push(LayerOp::Pop(layer.serial));
                }
                Back::Emit(name) => {
                    self.interaction.nav_mut().back = false;
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
                *self.interaction.request_focus_mut() = f;
            }
        }
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::UiDocument;
    use crate::reactive::SourceId;
    use crate::reactive::session::Command;
    use crate::{InputFrame, UiDesc, UiKey, Viewport};
    use serde_json::json;
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
        ui.docs
            .state_mut()
            .memory
            .reactive
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
        ui.docs
            .state_mut()
            .memory
            .reactive
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
        ui.docs
            .state_mut()
            .memory
            .reactive
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
            ui.tree
                .nodes()
                .iter()
                .find(|n| n.id == "test:pause")
                .map(|n| (n.style.sampled.opacity, n.props.animations.len()))
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
        assert!(
            ui.docs
                .document_mount_inputs()
                .as_array()
                .unwrap()
                .is_empty()
        );
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
        let c = ui.front.cover().expect("the cover closes");
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
            seen.push((
                shown,
                ui.front.cover().map(|c| (c.revealing, c.kind, c.from)),
            ));
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
                ui.front.cover().map(|c| c.revealing),
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
        assert!(ui.docs.document_mount_inputs().as_array().unwrap().len() == 1);
    }

    /// The frame's solid red shapes (the shared element), on the screen.
    fn reds(ui: &Ui) -> Vec<crate::layout::Rect> {
        ui.out
            .frame()
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
                seen.push((
                    reds(&ui),
                    ui.out.frame().warnings.clone(),
                    ui.layer_stack().len(),
                ));
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
                .out
                .frame()
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
            assert!(
                ui.out.frame().warnings.is_empty(),
                "{:?}",
                ui.out.frame().warnings
            );
            let cuts: Vec<((f32, f32), (f32, f32))> = ui
                .out
                .frame()
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
