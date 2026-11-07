use super::*;

impl Ctx {
    pub(super) fn err(&mut self, line: usize, what: impl std::fmt::Display) {
        self.errors.push(format!("line {line}: {what}"));
    }

    fn prop(&mut self, kind: &str, e: &Entry) -> Option<PropSet> {
        let Some((spec, p)) = schema::prop(&e.key) else {
            self.err(
                e.line,
                format!(
                    "'{}' is not a prop (the schema: og.ui.documentSchema, README §Documents)",
                    e.key
                ),
            );
            return None;
        };
        if !p.applies(kind) {
            let on = match p.on {
                schema::On::Kinds(k) => k.join(", "),
                schema::On::All => String::new(),
            };
            self.err(
                e.line,
                format!("'{}' is not a prop of {kind} (it is {on}'s)", e.key),
            );
            return None;
        }
        match parse_value(p.ty, &e.values) {
            Ok(value) => Some(PropSet {
                spec,
                value,
                line: e.line,
                source: written(&e.values),
            }),
            Err(why) => {
                self.err(e.line, format!("{}: {why}", e.key));
                None
            }
        }
    }

    fn group(&mut self, e: &Entry) -> Option<DocGroup> {
        let w = words(e);
        let name = w.first().cloned().unwrap_or_default();
        if name.is_empty() {
            self.err(e.line, "state <group> [bind <expr>] { <state> … }");
            return None;
        }
        let bind = match w.get(1).map(String::as_str) {
            Some("bind") => match w.get(2).map(|s| Expr::parse(s)) {
                Some(Ok(x)) => Some(x),
                Some(Err(why)) => {
                    self.err(e.line, why);
                    None
                }
                None => {
                    self.err(e.line, "bind needs its expression");
                    None
                }
            },
            Some(other) => {
                self.err(
                    e.line,
                    format!("'{other}': a state group is `state <group> [bind <expr>] {{`"),
                );
                None
            }
            None => None,
        };
        let mut g = DocGroup {
            name,
            bind,
            ..DocGroup::default()
        };
        for s in e.block.as_ref().map_or(&[][..], |b| &b.entries) {
            if s.key == "transition" {
                let w = words(s);
                if w.len() < 3 {
                    self.err(s.line, "transition <from|any> <to|any> <transition>");
                    continue;
                }
                match parse_transition(&s.values[2..]) {
                    // `any` is the word for every state (`*` in og-ui's StateTransition).
                    Ok(t) => g.transitions.push(StateTransition {
                        from: if w[0] == "any" {
                            "*".into()
                        } else {
                            w[0].clone()
                        },
                        to: if w[1] == "any" {
                            "*".into()
                        } else {
                            w[1].clone()
                        },
                        transition: t,
                    }),
                    Err(why) => self.err(s.line, why),
                }
                continue;
            }
            let mut st = DocState {
                name: s.key.clone(),
                ..DocState::default()
            };
            for v in s.block.as_ref().map_or(&[][..], |b| &b.entries) {
                match v.key.as_str() {
                    "enter" | "exit" => {
                        let n = v.values.first().and_then(as_name).unwrap_or_default();
                        if v.key == "enter" {
                            st.on_enter = n;
                        } else {
                            st.on_exit = n;
                        }
                    }
                    "background" | "ink" => match parse_paint(&v.values) {
                        Ok(p) => st.values.push(if v.key == "background" {
                            StateValue::Background(p)
                        } else {
                            StateValue::Ink(p)
                        }),
                        Err(why) => self.err(v.line, why),
                    },
                    k => match (
                        Prop::from_name(k),
                        v.values.first().and_then(Value::as_number),
                    ) {
                        (Some(p), Some(n)) => st.values.push(StateValue::Prop(p, n.as_f32())),
                        (Some(p), None) => match parse_lit(Ty::Num, &v.values) {
                            Ok(l @ Lit::Lengths(..)) => st.values.push(StateValue::Length(p, l)),
                            _ => self.err(v.line, format!("{k} takes a number or length token")),
                        },
                        (None, _) => self.err(
                            v.line,
                            format!(
                                "'{k}' is not a state value: {}, background, ink, enter, exit",
                                Prop::ALL
                                    .iter()
                                    .map(|p| p.name())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                        ),
                    },
                }
            }
            if g.states.iter().any(|x| x.name == st.name) {
                self.err(s.line, format!("the state '{}' is declared twice", st.name));
            }
            g.states.push(st);
        }
        if g.states.is_empty() {
            self.err(
                e.line,
                format!("the state group '{}' has no states", g.name),
            );
        }
        for t in &g.transitions {
            for s in [&t.from, &t.to] {
                if s != "*" && !g.states.iter().any(|x| &x.name == s) {
                    self.errors.push(format!(
                        "line {}: the transition names '{s}', which is not a state of '{}'",
                        e.line, g.name
                    ));
                }
            }
        }
        Some(g)
    }

    fn trigger(&mut self, e: &Entry) -> Option<Trigger> {
        let w = words(e);
        const HOW: &str = "on <click|changed|appear|tap|doubleTap|longPress|change <binding>|action <name>> <play <timeline> | state <group> <state> | emit <name> | push <screen> | pop | action <name> | set <path> <expr>>";
        let (when, rest) = match w.first().map(String::as_str) {
            Some("click") => (When::Click, &w[1..]),
            Some("changed") => (When::Changed, &w[1..]),
            Some("appear") => (When::Appear, &w[1..]),
            Some("tap") => (When::Tap, &w[1..]),
            Some("doubleTap") => (When::DoubleTap, &w[1..]),
            Some("longPress") => (When::LongPress, &w[1..]),
            Some("action") if w.len() > 1 => (When::Action(w[1].clone()), &w[2..]),
            Some("change") if w.len() > 1 => match Expr::parse(&w[1]) {
                Ok(x) => (When::Change(x), &w[2..]),
                Err(why) => {
                    self.err(e.line, why);
                    return None;
                }
            },
            _ => {
                self.err(e.line, HOW);
                return None;
            }
        };
        let arg = |i: usize| rest.get(i).cloned();
        let act = match rest.first().map(String::as_str) {
            Some("play") => arg(1).map(Act::Play),
            Some("state") => arg(1).zip(arg(2)).map(|(g, s)| Act::State(g, s)),
            Some("emit") => arg(1).map(Act::Emit),
            Some("push") => arg(1).map(Act::Push),
            Some("pop") => Some(Act::Pop),
            Some("action") => arg(1).map(Act::Action),
            Some("set") => match (arg(1), arg(2).map(|s| Expr::parse(&s))) {
                (Some(p), Some(Ok(x))) => Some(Act::Set(p, x)),
                (_, Some(Err(why))) => {
                    self.err(e.line, why);
                    return None;
                }
                _ => None,
            },
            _ => None,
        };
        match act {
            Some(act) => Some(Trigger {
                when,
                act,
                line: e.line,
            }),
            None => {
                self.err(e.line, HOW);
                None
            }
        }
    }

    fn at(&mut self, kind: &str, e: &Entry) -> Option<(AtQuery, Vec<PropSet>)> {
        use crate::element::Orientation;
        use crate::style::Breakpoint;
        let mut q = AtQuery::default();
        for w in words(e) {
            match w.as_str() {
                "phonePortrait" => q.breakpoints.push(Breakpoint::PhonePortrait),
                "phoneLandscape" => q.breakpoints.push(Breakpoint::PhoneLandscape),
                "phone" => q
                    .breakpoints
                    .extend([Breakpoint::PhonePortrait, Breakpoint::PhoneLandscape]),
                "tablet" => q.breakpoints.push(Breakpoint::Tablet),
                "desktop" => q.breakpoints.push(Breakpoint::Desktop),
                "portrait" => q.orientation = Some(Orientation::Portrait),
                "landscape" => q.orientation = Some(Orientation::Landscape),
                other => {
                    self.err(e.line, format!("'{other}' is not a breakpoint: phonePortrait, phoneLandscape, phone, tablet, desktop, portrait, landscape"));
                    return None;
                }
            }
        }
        let props = e
            .block
            .as_ref()
            .map_or(&[][..], |b| &b.entries)
            .iter()
            .filter_map(|p| self.prop(kind, p))
            .collect();
        Some((q, props))
    }

    /// A node from its entry (an element, `for`, `if` or `use`).
    pub(super) fn node(&mut self, e: &Entry, parent: usize) -> Option<usize> {
        let w = words(e);
        let (kind, id) = match e.key.as_str() {
            "for" => {
                if w.len() != 3 || w[1] != "in" {
                    self.err(e.line, "for <item> in <list> { <template> }");
                    return None;
                }
                match Expr::parse(&w[2]) {
                    Ok(list) => (
                        NodeKind::For {
                            var: w[0].clone(),
                            list,
                            key: Expr::Bin(
                                crate::bind::Op::Add,
                                Box::new(Expr::Lit(serde_json::json!(""))),
                                Box::new(Expr::Path("$index".into())),
                            ),
                            empty: Vec::new(),
                        },
                        String::new(),
                    ),
                    Err(why) => {
                        self.err(e.line, why);
                        return None;
                    }
                }
            }
            "if" => match w.first().map(|s| Expr::parse(s)) {
                Some(Ok(x)) => (NodeKind::If(x), String::new()),
                Some(Err(why)) => {
                    self.err(e.line, why);
                    return None;
                }
                None => {
                    self.err(e.line, "if <expr> { … }");
                    return None;
                }
            },
            "use" => {
                let Some(c) = w.first() else {
                    self.err(e.line, "use <Component> [id] { <prop> <value> … }");
                    return None;
                };
                (
                    NodeKind::Use(c.clone()),
                    w.get(1).cloned().unwrap_or_else(|| c.clone()),
                )
            }
            k => match schema::kind(k) {
                Some(spec) => {
                    if w.len() > 1 {
                        self.err(
                            e.line,
                            format!("{k} takes one id (its props go in its block)"),
                        );
                    }
                    (
                        NodeKind::Element(spec.name),
                        w.first().cloned().unwrap_or_default(),
                    )
                }
                None => {
                    self.err(
                        e.line,
                        format!(
                            "'{k}' is not an element: {}, or for, if, use",
                            schema::KINDS
                                .iter()
                                .map(|k| k.name)
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                    );
                    return None;
                }
            },
        };
        if id.contains('/') {
            self.err(
                e.line,
                format!("the id '{id}' has a '/' (paths join ids with it)"),
            );
        }
        let i = self.nodes.len();
        self.nodes.push(DocNode {
            kind: kind.clone(),
            id,
            line: e.line,
            source_offset: 0,
            props: Vec::new(),
            at: Vec::new(),
            rules: Vec::new(),
            groups: Vec::new(),
            triggers: Vec::new(),
            args: Vec::new(),
            children: Vec::new(),
            parent,
        });
        let kname = match &kind {
            NodeKind::Element(k) => *k,
            _ => "",
        };
        let container = schema::kind(kname).is_some_and(|k| k.container)
            || matches!(kind, NodeKind::For { .. } | NodeKind::If(_));
        for c in e.block.as_ref().map_or(&[][..], |b| &b.entries) {
            match (&kind, c.key.as_str()) {
                (NodeKind::Use(_), k)
                    if c.block.is_none()
                        && schema::kind(k).is_none()
                        && !matches!(k, "for" | "if" | "use" | "state" | "on" | "at") =>
                {
                    match schema::prop(k) {
                        // An element prop applies to the instance's root.
                        Some(_)
                            if !k.is_empty()
                                && self.nodes[i].args.iter().all(|a| a.0 != k)
                                && is_element_prop(k) =>
                        {
                            if let Some(p) = self.prop("column", c) {
                                self.nodes[i].props.push(p);
                            }
                        }
                        _ => match parse_value(Ty::Text, &c.values)
                            .or_else(|_| parse_value(Ty::Names, &c.values))
                        {
                            Ok(v) => {
                                let v = match v {
                                    PropValue::Lit(Lit::Strs(mut s)) if s.len() == 1 => {
                                        PropValue::Lit(Lit::Str(s.remove(0)))
                                    }
                                    PropValue::Lit(Lit::Str(s)) => match c.values.first() {
                                        Some(Value::Number(n)) => {
                                            PropValue::Lit(Lit::Num(n.as_f32()))
                                        }
                                        Some(Value::Word(b)) if b == "true" || b == "false" => {
                                            PropValue::Lit(Lit::Bool(b == "true"))
                                        }
                                        _ => PropValue::Lit(Lit::Str(s)),
                                    },
                                    other => other,
                                };
                                self.nodes[i].args.push((k.to_string(), v, c.line));
                            }
                            Err(why) => self.err(c.line, format!("{k}: {why}")),
                        },
                    }
                }
                (_, "state") => {
                    if let Some(g) = self.group(c) {
                        self.nodes[i].groups.push(g);
                    }
                }
                (_, "on") => {
                    if let Some(t) = self.trigger(c) {
                        self.nodes[i].triggers.push(t);
                    }
                }
                (_, "at") => {
                    if let Some(a) = self.at(if kname.is_empty() { "column" } else { kname }, c) {
                        self.nodes[i].at.push(a);
                    }
                }
                // An element (a kind name that is also one of its own props,
                // `icon coin` in an icon, is the prop).
                (_, k)
                    if (schema::kind(k).is_some() || matches!(k, "for" | "if" | "use"))
                        && (container
                            || c.block.is_some()
                            || !schema::prop(k).is_some_and(|(_, p)| p.applies(kname))) =>
                {
                    if !container {
                        self.err(c.line, format!("{kname} has no children (a container: column, row, stack, grid, panel, scroll, list, cells, slot)"));
                        continue;
                    }
                    if let Some(ch) = self.node(c, i) {
                        self.nodes[i].children.push(ch);
                    }
                }
                (NodeKind::Element(_), _) => {
                    if let Some(p) = self.prop(kname, c) {
                        self.nodes[i].props.push(p);
                    }
                }
                (_, k) => self.err(
                    c.line,
                    format!("'{k}': a `for` or `if` holds elements only"),
                ),
            }
        }
        if matches!(kind, NodeKind::Element("list" | "cells")) {
            let fors = self.nodes[i]
                .children
                .iter()
                .filter(|&&c| matches!(self.nodes[c].kind, NodeKind::For { .. }))
                .count();
            if fors != 1 || self.nodes[i].children.len() != 1 {
                self.err(e.line, format!("{kname} holds exactly one `for` (its rows are built from it, only those in view)"));
            }
        }
        // Sibling ids are unique (paths name elements by them).
        let kids = self.nodes[i].children.clone();
        let mut seen = std::collections::BTreeSet::new();
        for c in kids {
            let id = &self.nodes[c].id;
            if !id.is_empty() && !seen.insert(id.clone()) {
                let line = self.nodes[c].line;
                self.err(
                    line,
                    format!("the id '{id}' repeats among its siblings: ids name elements in paths"),
                );
            }
        }
        Some(i)
    }
}

/// A prop of the box or the element model (every kind's), which a `use`
/// instance gives its component's root.
fn is_element_prop(name: &str) -> bool {
    schema::prop(name).is_some_and(|(_, p)| p.on == schema::On::All)
}

pub(super) fn parse_timeline(ctx: &mut Ctx, e: &Entry) -> Option<Timeline> {
    let mut t = Timeline::default();
    for c in e.block.as_ref().map_or(&[][..], |b| &b.entries) {
        let w = words(c);
        match c.key.as_str() {
            "track" => {
                let (Some(target), Some(prop)) = (w.first(), w.get(1)) else {
                    ctx.err(
                        c.line,
                        "track <target|\"\"> <prop> { key <ms> <value> [ease] … }",
                    );
                    continue;
                };
                let Some(prop) = Prop::from_name(prop) else {
                    ctx.err(
                        c.line,
                        format!(
                            "'{prop}' is not an animatable prop: {}",
                            Prop::ALL
                                .iter()
                                .map(|p| p.name())
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                    );
                    continue;
                };
                let mut keys = Vec::new();
                for k in c.block.as_ref().map_or(&[][..], |b| &b.entries) {
                    if k.key != "key" {
                        ctx.err(k.line, "a track holds `key <ms> <value> [ease]` lines");
                        continue;
                    }
                    let n = |i: usize| {
                        k.values
                            .get(i)
                            .and_then(Value::as_number)
                            .map(Number::as_f32)
                    };
                    let (Some(at_ms), Some(value)) = (n(0), n(1)) else {
                        ctx.err(k.line, "key <ms> <value> [linear|out|exit|inOut|hold|spring <ms> <bounce>|bezier a b c d]");
                        continue;
                    };
                    let ease = match k.values.get(2).and_then(Value::as_word) {
                        None => KeyEase::default(),
                        Some("linear") => KeyEase::Linear,
                        Some("hold") => KeyEase::Hold,
                        Some("spring") => KeyEase::Spring(SpringSpec::new(
                            n(3).unwrap_or(300.0),
                            n(4).unwrap_or(0.0),
                        )),
                        Some("bezier") => match (n(3), n(4), n(5), n(6)) {
                            (Some(a), Some(b), Some(c2), Some(d)) => {
                                let ease = Ease::CubicBezier([a, b, c2, d]);
                                if !ease.is_valid() {
                                    ctx.err(
                                        k.line,
                                        "bezier needs finite control points and x in [0,1]",
                                    );
                                    continue;
                                }
                                KeyEase::Ease(ease)
                            }
                            _ => {
                                ctx.err(k.line, "bezier takes four numbers");
                                continue;
                            }
                        },
                        Some(e2) => match parse_ease(e2) {
                            Ok(e2) => KeyEase::Ease(e2),
                            Err(why) => {
                                ctx.err(k.line, why);
                                continue;
                            }
                        },
                    };
                    keys.push(Key { at_ms, value, ease });
                }
                if keys.windows(2).any(|w| w[1].at_ms < w[0].at_ms) {
                    ctx.err(c.line, "a track's keys are in time order");
                }
                t.tracks.push(TimelineTrack {
                    target: target.clone(),
                    prop,
                    keys,
                });
            }
            "event" => {
                let at = c
                    .values
                    .first()
                    .and_then(Value::as_number)
                    .map(Number::as_f32);
                let (Some(at_ms), Some(target), Some(kind)) = (at, w.get(1), w.get(2)) else {
                    ctx.err(
                        c.line,
                        "event <ms> <target|\"\"> <sound|haptic|burst|state|callback> …",
                    );
                    continue;
                };
                let n = |i: usize| {
                    c.values
                        .get(i)
                        .and_then(Value::as_number)
                        .map(Number::as_f32)
                };
                let kind = match kind.as_str() {
                    "sound" => EventKind::Sound {
                        name: w.get(3).cloned().unwrap_or_default(),
                        pitch: n(4).unwrap_or(1.0),
                    },
                    "haptic" => {
                        EventKind::Haptic(w.get(3).cloned().unwrap_or_else(|| "light".into()))
                    }
                    "burst" => EventKind::Burst {
                        count: n(3).unwrap_or(10.0) as u32,
                        radius: n(4).unwrap_or(40.0),
                        color: w.get(5).and_then(|s| crate::style_file::parse_color(s)),
                    },
                    "state" => match (w.get(3), w.get(4)) {
                        (Some(g), Some(s)) => EventKind::State {
                            group: g.clone(),
                            state: s.clone(),
                        },
                        _ => {
                            ctx.err(c.line, "event <ms> <target> state <group> <state>");
                            continue;
                        }
                    },
                    "callback" => EventKind::Callback(w.get(3).cloned().unwrap_or_default()),
                    other => {
                        ctx.err(
                            c.line,
                            format!(
                                "'{other}' is not an event: sound, haptic, burst, state, callback"
                            ),
                        );
                        continue;
                    }
                };
                t.events.push(TimelineEvent {
                    at_ms,
                    target: target.clone(),
                    kind,
                });
            }
            "label" => match (w.first(), c.values.get(1).and_then(Value::as_number)) {
                (Some(l), Some(ms)) => t.labels.push((l.clone(), ms.as_f32())),
                _ => ctx.err(c.line, "label <name> <ms>"),
            },
            "duration" => match c.values.first().and_then(Value::as_number) {
                Some(ms) => t.duration_ms = ms.as_f32(),
                None => ctx.err(c.line, "duration <ms>"),
            },
            k => ctx.err(
                c.line,
                format!("'{k}' is not part of a timeline: track, event, label, duration"),
            ),
        }
    }
    Some(t)
}
