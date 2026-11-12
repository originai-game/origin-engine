//! The tick's drag and drop and Pointer Events as document events
//! ([`super::super::gestures`]): fired on the element the input is at and
//! bubbled up the mounted tree to the nearest handler, as the DOM bubbles
//! them. Only the held element, the one under the pointer and a drop's
//! target are visited, never the tree.
use super::*;

impl RetainedHost {
    /// The handler of `when` on the element of `key` or its nearest
    /// mounted ancestor with one: its element and the action it names.
    fn handler(&self, key: u64, when: &When) -> Option<(MountedNode, String)> {
        let mut at = self
            .owner_of
            .get(&key)
            .or_else(|| self.by_key.get(&key))?
            .clone();
        loop {
            let e = self.elements.get(&at)?;
            if let Some(t) = e.document.nodes[e.index]
                .triggers
                .iter()
                .find(|t| &t.when == when)
                && let Act::Emit(name) = &t.act
            {
                return Some((at, name.clone()));
            }
            at = self.topology.parent(&at)?.clone();
        }
    }

    fn row_key(&self, key: u64) -> String {
        self.owner_of
            .get(&key)
            .or_else(|| self.by_key.get(&key))
            .and_then(|n| self.elements.get(n))
            .map(|e| e.context.row_key.clone())
            .unwrap_or_default()
    }

    /// Fires `when` at `key`: its handler's `emit` event, the point in the
    /// handler's box and `extra` its value.
    fn dispatch(
        &self,
        ui: &Ui,
        env: &dyn Env,
        key: u64,
        when: When,
        extra: serde_json::Map<String, Value>,
        events: &mut Vec<DocEvent>,
    ) {
        self.dispatch_at(
            ui,
            env,
            key,
            when,
            ui.interaction.input.pointer,
            extra,
            events,
        );
    }

    /// [`Self::dispatch`] with the point at `p` (screen CSS px).
    #[allow(clippy::too_many_arguments)]
    fn dispatch_at(
        &self,
        ui: &Ui,
        env: &dyn Env,
        key: u64,
        when: When,
        p: og_core::glam::Vec2,
        mut extra: serde_json::Map<String, Value>,
        events: &mut Vec<DocEvent>,
    ) {
        if key == 0 {
            return;
        }
        let Some((node, action)) = self.handler(key, &when) else {
            return;
        };
        let e = &self.elements[&node];
        // Last tick's box (this tick's is laid out after the build), and
        // the pointer mapped into it through any transform.
        let Some(hit) = ui
            .interaction
            .hit(node.key)
            .or_else(|| ui.interaction.hit(key))
        else {
            return;
        };
        let rect = hit.local;
        let (lx, ly) = hit.inv.apply(p.x, p.y);
        let (x, y) = (lx - rect.x, ly - rect.y);
        extra.insert("x".into(), json!(x));
        extra.insert("y".into(), json!(y));
        // A viewport's events carry the ray into its scene.
        if e.document.nodes[e.index].kind == NodeKind::Element("viewport")
            && let Some(r) = &e.resolved
            && let Some(view) =
                super::super::viewport::camera(&r.values, rect.width / rect.height.max(1.0))
        {
            let (o, d) = view.ray(x, y, rect.width, rect.height);
            extra.insert(
                "ray".into(),
                json!({ "origin": [o.x, o.y, o.z], "direction": [d.x, d.y, d.z] }),
            );
        }
        let mut b = e.build(env, Arc::default());
        b.event(&e.path, "emit", &action, Value::Object(extra));
        events.extend(b.events);
    }

    /// The tick's drag, drop, pointer and wheel events.
    pub(super) fn input_events(&mut self, ui: &Ui, env: &dyn Env, events: &mut Vec<DocEvent>) {
        let input = &ui.interaction.input;
        let (hot, active) = (ui.interaction.state.hot, ui.interaction.state.active);
        let moved = [input.pointer.x, input.pointer.y] != self.pointer.pointer;
        // Drag and drop: the held element's gesture against last tick's.
        let gesture = |k: u64| ui.interaction.state.motion.gestures.get(&k).copied();
        let now = (active != 0)
            .then(|| gesture(active))
            .flatten()
            .filter(|g| g.held || g.dragging)
            .map(|g| (active, g.dragging, g.over));
        let prev = self.pointer.drag;
        let keys = |dragged: u64, target: u64| {
            let mut m = serde_json::Map::new();
            m.insert("dragged".into(), json!(self.row_key(dragged)));
            m.insert("target".into(), json!(self.row_key(target)));
            m
        };
        if let Some((k, true, over)) = now {
            if !prev.is_some_and(|(p, dragging, _)| p == k && dragging) {
                self.dispatch(ui, env, k, When::DragStart, keys(k, 0), events);
            }
            let entered = !prev.is_some_and(|(p, _, o)| p == k && o == over);
            if over != 0 && (entered || moved) {
                self.dispatch(ui, env, over, When::DragOver, keys(k, over), events);
            }
        }
        if let Some((k, true, _)) = prev
            && !now.is_some_and(|(n, dragging, _)| n == k && dragging)
        {
            let target = gesture(k).map_or(0, |g| g.dropped_on);
            if target != 0 {
                self.dispatch(ui, env, target, When::Drop, keys(k, target), events);
            }
            self.dispatch(ui, env, k, When::DragEnd, keys(k, target), events);
            // Thrown off (a sheet swiped down): its element builds next
            // tick and closes.
            if gesture(k).is_some_and(|g| g.dismissed)
                && let Some(owner) = self.owner_of.get(&k).cloned()
            {
                self.emit.insert(owner);
            }
        }
        self.pointer.drag = now;
        // Pointer Events: down and wheel at the element under the pointer,
        // move at the held element (captured) or the one under it, up at
        // the element the press went down on.
        let none = serde_json::Map::new;
        if input.pressed {
            self.pointer.pressed = hot;
            self.dispatch(ui, env, hot, When::PointerDown, none(), events);
        }
        if moved {
            let at = if input.down && active != 0 {
                active
            } else {
                hot
            };
            self.dispatch(ui, env, at, When::PointerMove, none(), events);
        }
        if input.released {
            let at = std::mem::take(&mut self.pointer.pressed);
            self.dispatch(
                ui,
                env,
                if at != 0 { at } else { hot },
                When::PointerUp,
                none(),
                events,
            );
        }
        if input.wheel != og_core::glam::Vec2::ZERO {
            let mut m = none();
            m.insert("deltaX".into(), json!(input.wheel.x));
            m.insert("deltaY".into(), json!(input.wheel.y));
            self.dispatch(ui, env, hot, When::Wheel, m, events);
        }
        // A pinch: at the element the touches went down on, each tick
        // their spread changes, its scale since they went down.
        match super::super::gestures::two_touches(input, |_, _| true) {
            Some((centre, spread)) => {
                let (key, start, last) = self.pointer.pinch.unwrap_or((hot, spread, 1.0));
                let scale = spread / start;
                if (scale - last).abs() > 1e-4 {
                    let mut m = none();
                    m.insert("scale".into(), json!(scale));
                    self.dispatch_at(ui, env, key, When::Pinch, centre, m, events);
                }
                self.pointer.pinch = Some((key, start, scale));
            }
            None => self.pointer.pinch = None,
        }
        self.pointer.pointer = [input.pointer.x, input.pointer.y];
    }
}

#[cfg(test)]
mod tests {
    use crate::doc::UiDocument;
    use crate::reactive::SourceId;
    use crate::reactive::session::Command;
    use crate::{InputFrame, Ui, UiDesc, Viewport};
    use og_core::glam::Vec2;
    use serde_json::json;
    use std::collections::BTreeMap;

    const BOARD: &str = r#"
<meta name="component" content="board">
<meta name="action:pick" content="Pick">
<div id="Board" onpointerdown={press} onwheel={zoom}>
  <ul id="Bag">
    {#each $items as item (item.id)}<li id="Item" draggable="true" data-drag-type="gem" ondragstart={lift} ondragend={place}>{item.name}</li>{/each}
  </ul>
  <div id="Slot" data-accept="gem" ondragover={hover} ondrop={drop}>Slot</div>
  <div id="Wrong" data-accept="coin" ondrop={drop}>Coins</div>
</div>
<style>
#Board { width: 360px; height: 280px; padding: 0; position: absolute; top: 12px; inset-inline-start: 20px; }
#Bag { margin: 0; padding: 0; }
#Item { height: 40px; width: 200px; }
#Slot, #Wrong { height: 60px; width: 200px; }
</style>
"#;

    fn frame(
        ui: &mut Ui,
        at: Vec2,
        down: bool,
        was: bool,
        wheel: f32,
    ) -> Vec<(String, String, serde_json::Value)> {
        let input = InputFrame {
            pointer: at,
            pointer_inside: true,
            down,
            pressed: down && !was,
            released: !down && was,
            wheel: Vec2::new(0.0, wheel),
            ..Default::default()
        };
        ui.begin(input, Viewport::new(400.0, 300.0));
        ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
        ui.end();
        ui.take_doc_events()
            .into_iter()
            .filter(|e| e.kind == "emit")
            .map(|e| (e.name, e.row_key, e.value))
            .collect()
    }

    fn rect(ui: &Ui, path: &str) -> [f32; 2] {
        let e = ui
            .probe_elements()
            .into_iter()
            .find(|e| e.path == path)
            .unwrap_or_else(|| {
                panic!(
                    "{path}: {:?}",
                    ui.probe_elements()
                        .iter()
                        .map(|e| e.path.clone())
                        .collect::<Vec<_>>()
                )
            });
        [e.x, e.y]
    }

    #[test]
    fn drag_and_drop_and_pointer_events_reach_their_handlers() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(UiDocument::parse("board.svelte", BOARD).unwrap());
        let items = SourceId::new("test", "items");
        ui.docs
            .reactive
            .execute(Command::Collection {
                source: items.clone(),
                key: "id".into(),
                items: ["ruby", "jade"]
                    .iter()
                    .map(|n| json!({"id": n, "name": n}).as_object().unwrap().clone())
                    .collect(),
            })
            .unwrap();
        ui.mount_document(
            SourceId::new("test", "board"),
            "board",
            BTreeMap::from([("items".into(), items)]),
            BTreeMap::new(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let idle = Vec2::new(390.0, 290.0);
        for _ in 0..3 {
            frame(&mut ui, idle, false, false, 0.0);
        }
        let jade = rect(&ui, "test:board/Bag/Item#jade");
        let slot = rect(&ui, "test:board/Slot");
        let from = Vec2::new(jade[0] + 20.0, jade[1] + 20.0);
        let to = Vec2::new(slot[0] + 50.0, slot[1] + 30.0);
        // A press on the jade: pointerdown bubbles to the board, with the
        // point in the board's box.
        let down = frame(&mut ui, from, true, false, 0.0);
        assert_eq!(down.len(), 1, "{down:?}");
        assert_eq!(down[0].0, "press");
        // The board's box starts at (20, 12).
        assert_eq!(
            (down[0].2["x"].as_f64(), down[0].2["y"].as_f64()),
            (
                Some(f64::from(from.x) - 20.0),
                Some(f64::from(from.y) - 12.0)
            )
        );
        let mut all = Vec::new();
        for k in 1..=8 {
            let p = from + (to - from) * (k as f32 / 8.0);
            all.extend(frame(&mut ui, p, true, true, 0.0));
        }
        all.extend(frame(&mut ui, to, false, true, 0.0));
        all.extend(frame(&mut ui, to, false, false, 0.0));
        let names: Vec<&str> = all.iter().map(|(n, ..)| n.as_str()).collect();
        assert_eq!(
            names.iter().filter(|n| **n == "lift").count(),
            1,
            "{names:?}"
        );
        assert!(names.contains(&"hover"), "{names:?}");
        let drop = all.iter().find(|(n, ..)| n == "drop").expect("a drop");
        assert_eq!(drop.2["dragged"], json!("jade"), "{drop:?}");
        let place = all.iter().find(|(n, ..)| n == "place").expect("a dragend");
        assert_eq!(place.1, "jade");
        // Dropped once, and only where its type is taken.
        assert_eq!(
            names.iter().filter(|n| **n == "drop").count(),
            1,
            "{names:?}"
        );
        let order = |n: &str| names.iter().position(|m| *m == n).unwrap();
        assert!(order("lift") < order("hover") && order("hover") < order("drop"));
        assert!(order("drop") < order("place"));
        // The wheel over the board.
        let over = Vec2::new(300.0, 250.0);
        let wheel = frame(&mut ui, over, false, false, -3.0);
        assert_eq!(wheel.len(), 1, "{wheel:?}");
        assert_eq!(
            (wheel[0].0.as_str(), &wheel[0].2["deltaY"]),
            ("zoom", &json!(-3.0))
        );
        // Hovered, the board is the element under the pointer: written
        // again each tick, as any hovered control is, and nothing else.
        for _ in 0..60 {
            assert!(frame(&mut ui, over, false, false, 0.0).is_empty());
        }
        let stats = ui.frame().stats;
        assert_eq!((stats.bindings, stats.reconciled), (0, 1), "{stats:?}");
        // Away from it, once the dragged item is home: nothing at all.
        let away = Vec2::new(-20.0, -20.0);
        for _ in 0..3 {
            assert!(frame(&mut ui, away, false, false, 0.0).is_empty());
        }
        let stats = ui.frame().stats;
        assert_eq!((stats.bindings, stats.reconciled), (0, 0), "{stats:?}");
    }

    const PAD: &str = r#"
<meta name="component" content="pad">
<div id="Pad">
  <button id="Go" onclick={go}>Go</button>
  <button id="Other" onclick={go}>Other</button>
</div>
<style>
#Pad { padding: 0; }
#Go, #Other { width: 100px; }
#Go:hover, #Other:hover { width: 140px; }
#Go:active { width: 180px; }
#Go:focus-visible { width: 210px; }
</style>
"#;

    /// `og.ui.forceState(path, "interaction", …)` is how a sheet draws every
    /// state at once: a forced state reaches the element's CSS state and its
    /// widget's response as the pointer's and the focus's do, on several
    /// elements together, and never activates anything.
    #[test]
    fn forced_interaction_states_reach_css_and_never_click() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(UiDocument::parse("pad.svelte", PAD).unwrap());
        ui.mount_document(
            SourceId::new("test", "pad"),
            "pad",
            BTreeMap::new(),
            BTreeMap::new(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let away = Vec2::new(390.0, 290.0);
        let width = |ui: &Ui, id: &str| {
            let path = format!("test:pad/{id}");
            ui.probe_elements()
                .into_iter()
                .find(|e| e.path == path)
                .map(|e| e.width)
        };
        let mut clicks = 0;
        let mut run = |ui: &mut Ui, frames: usize| {
            for _ in 0..frames {
                ui.begin(
                    InputFrame {
                        pointer: away,
                        pointer_inside: true,
                        ..Default::default()
                    },
                    Viewport::new(400.0, 300.0),
                );
                ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
                ui.end();
                clicks += ui
                    .take_doc_events()
                    .iter()
                    .filter(|e| e.kind == "click")
                    .count();
            }
        };
        run(&mut ui, 2);
        assert_eq!(
            (width(&ui, "Go"), width(&ui, "Other")),
            (Some(100.0), Some(100.0))
        );
        // Two hovered at once, which the pointer alone never makes.
        ui.force_state("test:pad/Go", "interaction", Some("hover"));
        ui.force_state("test:pad/Other", "interaction", Some("hover"));
        run(&mut ui, 2);
        assert_eq!(
            (width(&ui, "Go"), width(&ui, "Other")),
            (Some(140.0), Some(140.0))
        );
        ui.force_state("test:pad/Go", "interaction", Some("pressed"));
        run(&mut ui, 2);
        assert_eq!(width(&ui, "Go"), Some(180.0));
        ui.force_state("test:pad/Go", "interaction", Some("focused"));
        run(&mut ui, 2);
        assert_eq!(width(&ui, "Go"), Some(210.0));
        // Released, each goes back to rest.
        ui.force_state("test:pad/Go", "interaction", None);
        ui.force_state("test:pad/Other", "interaction", None);
        run(&mut ui, 2);
        assert_eq!(
            (width(&ui, "Go"), width(&ui, "Other")),
            (Some(100.0), Some(100.0))
        );
        drop(run);
        assert_eq!(clicks, 0, "a forced press or focus activated a button");
    }
}
