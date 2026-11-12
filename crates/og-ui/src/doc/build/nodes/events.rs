use super::*;

pub(crate) trait NodesEventsUi {
    /// The on-screen keyboard of the text input at `path`: it opens when a
    /// pad gives the input the focus (unless `inputmode="none"`, the
    /// game's own keyboard), edits `s` in place, and Done submits; Done or
    /// Back return the focus to the input, which opens it again only once
    /// it has lost the focus. Its layout follows `type` and `inputmode`,
    /// `enterkeyhint` labels Done and `lang="ja"` offers kana.
    fn pad_keyboard(
        &mut self,
        path: &str,
        r: &Resolved,
        s: &mut String,
        resp: &mut crate::Response,
    );

    /// A `data-hold` (or `onhold`) element's hold this tick: counts the
    /// ticks it has been held (a press on it; Enter, Space or a pad's A
    /// while it has the focus; or its `data-action`'s input down) and
    /// whether the count reached its `data-hold` ms (default 500), then
    /// draws its fill (`::-og-hold`) from its start edge across the share
    /// reached, over its background and under its content. The count is
    /// widget memory: in snapshots, and the element builds each tick while
    /// it moves.
    fn hold(&mut self, key: u64, n: &DocNode, r: &Resolved, at: usize);

    /// Whether the press [`Self::hold`] counted on `key` reached its
    /// `data-hold` ms this tick (once per press).
    fn held(&mut self, key: u64, _n: &DocNode) -> bool;

    /// Fires a node's triggers for what happened to it this tick.
    fn fire(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        key: u64,
        path: &str,
        resp: crate::ui::Response,
    );
}

impl NodesEventsUi for Ui {
    fn pad_keyboard(
        &mut self,
        path: &str,
        r: &Resolved,
        s: &mut String,
        resp: &mut crate::Response,
    ) {
        use crate::widgets::KeyboardLayout as L;
        let id = format!("{path}~keyboard");
        let (open_key, off_key, field_key) = (
            format!("{id}~open"),
            format!("{id}~off"),
            format!("{id}~field"),
        );
        let was_open = self.memory(&open_key, 0.0) > 0.0;
        let off = self.memory(&off_key, 0.0) > 0.0;
        if resp.focused {
            self.remember(&field_key, f64::from_bits(self.interaction.state().focus));
        }
        let wants =
            resp.focused && self.device().is_pad() && r.str("inputmode") != Some("none") && !off;
        let open = was_open || wants;
        if !open {
            if !resp.focused {
                self.remember(&off_key, 0.0);
            }
            return;
        }
        let ty = r.str("type").unwrap_or("text");
        let mode = r.str("inputmode").unwrap_or(match ty {
            "number" => "decimal",
            "tel" => "tel",
            _ => "text",
        });
        let password = ty == "password";
        let desc = crate::widgets::KeyboardDesc {
            layout: match mode {
                "numeric" | "decimal" | "tel" => L::Numeric,
                _ => L::Letters,
            },
            kana: r
                .str("lang")
                .is_some_and(|l| l == "ja" || l.starts_with("ja-")),
            max_chars: r.num("maxChars").unwrap_or(0.0) as u32,
            title: r
                .str("label")
                .or(r.str("name"))
                .unwrap_or_default()
                .to_string(),
            placeholder: r.str("placeholder").unwrap_or_default().to_string(),
            submit_label: r
                .str("enterkeyhint")
                .map(|h| {
                    let mut c = h.chars();
                    c.next()
                        .map(|f| f.to_uppercase().chain(c).collect())
                        .unwrap_or_default()
                })
                .unwrap_or_default(),
            password,
        };
        // Open, it builds each tick: its keys take input and its caret
        // blinks.
        if let Some(owned) = self.tree.owning_mut().last_mut() {
            owned.moved = true;
        }
        let out = self.on_screen_keyboard(&id, open, s, &desc);
        resp.changed |= out.changed;
        resp.submitted |= out.submitted;
        let done = out.submitted || out.closed;
        if done {
            let back = self.memory(&field_key, 0.0).to_bits();
            if back != 0 {
                *self.interaction.request_focus_mut() = back;
            }
        }
        self.remember(&open_key, if open && !done { 1.0 } else { 0.0 });
        self.remember(&off_key, if done { 1.0 } else { f64::from(u8::from(off)) });
    }

    fn hold(&mut self, key: u64, n: &DocNode, r: &Resolved, at: usize) {
        let fill = r.has("dataHold") || n.props.iter().any(|p| p.name() == "dataHold");
        if !fill && !n.triggers.iter().any(|t| t.when == When::Hold) {
            return;
        }
        let ms = r.num("dataHold").unwrap_or(500.0);
        let need = ((ms * self.env.ticks_per_second() as f32 / 1000.0).ceil() as u64).max(1);
        let focused = self.interaction.state().focus == key
            && (self
                .interaction
                .input()
                .keys_held
                .iter()
                .any(|k| matches!(*k, "Enter" | "NumpadEnter" | "Space"))
                || self.interaction.input().gamepad.held & crate::input::GamepadFrame::A != 0);
        let pressing = (self.interaction.input().down && self.interaction.state().active == key)
            || focused
            || r.str("dataAction")
                .is_some_and(|a| self.interaction.action_down(a));
        let memory = crate::id::child_key(key, "~hold");
        let ticks = if pressing { self.anim(memory).h + 1 } else { 0 };
        if pressing || self.has_anim(memory) {
            self.set_anim(
                memory,
                crate::ui::Anim {
                    h: ticks,
                    t: need,
                    ..Default::default()
                },
            );
        }
        let frac = (ticks as f32 / need as f32).min(1.0);
        if !fill || frac <= 0.0 || at >= self.tree.nodes().len() || self.tree.nodes()[at].key != key
        {
            return;
        }
        // `::-og-hold`'s box (a pseudo-element node) colours it.
        let color = r
            .pseudo
            .iter()
            .find(|(w, ..)| *w == "hold")
            .and_then(|(_, _, p)| p.color("background"))
            .unwrap_or_else(|| self.env.theme().palette.primary.fade(0.33));
        let frame = if self.direction() == og_i18n::Direction::Rtl {
            crate::layout::Frame::left(1.0 - frac, 1.0)
        } else {
            crate::layout::Frame::left(0.0, frac)
        };
        let radius = self.tree.nodes()[at].el.radius;
        self.tree.shapes_mut(at).push(crate::layout::Shape::Rect {
            frame,
            radius,
            color,
        });
    }

    fn held(&mut self, key: u64, _n: &DocNode) -> bool {
        let a = self.anim(crate::id::child_key(key, "~hold"));
        a.h > 0 && a.h == a.t
    }

    fn fire(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        key: u64,
        path: &str,
        resp: crate::ui::Response,
    ) {
        // Paths of the events this element pushed without one.
        for e in b.events.iter_mut().filter(|e| e.path.is_empty()) {
            e.path = path.to_string();
        }
        if resp.clicked {
            b.event(path, "click", &n.id, Value::Null);
        }
        // A recogniser on a container that is not interactive answers
        // through its gesture, not its response.
        let g = self.gesture_of(key);
        let (tap, double, long) = (
            resp.tapped || g.tapped,
            resp.double_tapped || g.double_tapped,
            resp.long_pressed || g.long_pressed,
        );
        for (on, kind) in [(tap, "tap"), (double, "doubleTap"), (long, "longPress")] {
            if on {
                b.event(path, kind, &n.id, Value::Null);
            }
        }
        let appeared = !self.docs.host().seen.contains(&key);
        // `use:` in an immediate build: made the first tick it is built,
        // updated when its params change; gone is found in docs_begin. A
        // mounted element's are sent by the retained host.
        if b.cached.is_none() && !n.uses.is_empty() {
            for (k, (name, params)) in n.uses.iter().enumerate() {
                let value = params.as_ref().map_or(Value::Null, |p| b.eval(p));
                let old = self
                    .docs
                    .host_mut()
                    .last
                    .insert((key, usize::MAX - k), value.clone());
                let phase = match old {
                    None => "create",
                    Some(old) if old != value => "update",
                    Some(_) => continue,
                };
                b.event(
                    path,
                    "use",
                    name,
                    json!({ "phase": phase, "params": value }),
                );
            }
            let names = n.uses.iter().map(|(name, _)| name.clone()).collect();
            let event = DocEvent {
                mount: b.mount.clone(),
                path: path.to_string(),
                row_key: b.row_key.clone(),
                ..DocEvent::default()
            };
            self.docs.host_mut().uses.insert(key, (event, names));
        }
        let happened = std::mem::take(&mut self.docs.host_mut().happened);
        let held = n.triggers.iter().any(|t| t.when == When::Hold) && self.held(key, n);
        let context = long
            || (self.interaction.input().secondary_pressed && self.interaction.state().hot == key);
        for (k, t) in n.triggers.iter().enumerate() {
            let go = match &t.when {
                When::Click => resp.clicked,
                When::Changed => resp.changed,
                When::Appear => appeared,
                When::Tap => tap,
                When::DoubleTap => double,
                When::LongPress => long,
                When::Hold => held,
                When::ContextMenu => context,
                When::Toggle | When::Close | When::Revealed => happened.contains(&t.when),
                When::Action(a) => {
                    self.docs.host().accepts
                        && self.interaction.action_fired(a)
                        // A disabled control's shortcut activates nothing,
                        // as its press does not (a `<fieldset disabled>`'s
                        // controls included).
                        && !self.tree.index().find(key).is_some_and(|i| self.tree.nodes()[i].disabled)
                        // An exclusive `<details name>`'s own action
                        // reaches every disclosure of its name.
                        && (a.starts_with('~') || self.focus_in_row(path, &b.row_key))
                }
                When::Change(e) => {
                    let v = b.eval(e);
                    let old = self.docs.host_mut().last.insert((key, k), v.clone());
                    old.is_some_and(|o| o != v)
                }
                // The input pass fires these ([`super::pointer`]); the
                // capabilities' controls fire the rest when they land.
                _ => false,
            };
            if !go {
                continue;
            }
            match &t.act {
                Act::Play(tl) => {
                    let name = qualify(&b.doc.name, tl);
                    let id = self.play_timeline(path, &name, crate::motion::PlayDesc::default());
                    self.docs
                        .host_mut()
                        .plays
                        .insert((path.to_string(), tl.clone()), id);
                }
                Act::State(g, s) => {
                    self.docs
                        .host_mut()
                        .states
                        .insert((key, g.clone()), s.clone());
                }
                Act::Emit(name) => b.event(path, "emit", name, Value::Null),
                Act::Submit {
                    action,
                    controls,
                    row,
                } => {
                    // The first invalid control in document order takes
                    // the focus, and the form is not submitted.
                    let first = self
                        .docs
                        .host()
                        .validity
                        .iter()
                        .filter(|(_, v)| {
                            v.invalid
                                && v.mount == b.mount
                                && v.doc == b.doc.name
                                && controls.contains(&v.node)
                                && (!row || v.row_key == b.row_key)
                        })
                        .min_by(|a, b| (a.1.node, &a.1.row_key).cmp(&(b.1.node, &b.1.row_key)))
                        .map(|(k, _)| *k);
                    match first {
                        Some(k) => *self.interaction.request_focus_mut() = k,
                        None => b.event(path, "emit", action, Value::Null),
                    }
                }
                Act::Push(screen) => self
                    .docs
                    .host_mut()
                    .ops
                    .push(crate::layers::LayerOp::Push(screen.clone(), Value::Null)),
                Act::Pop => {
                    let l = self.docs.host().building;
                    self.docs
                        .host_mut()
                        .ops
                        .push(crate::layers::LayerOp::Pop(l))
                }
                Act::Action(a) => {
                    self.fire_action(a);
                    b.event(path, "action", a, Value::Null);
                }
                Act::Set(p, e) => {
                    let v = b.eval(e);
                    if let Some(w) = b.write_path(p) {
                        b.writes.push((w.clone(), v.clone()));
                        b.event(path, "set", &w, v);
                    }
                }
            }
        }
    }
}
