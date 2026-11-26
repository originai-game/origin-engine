//! Op node creation, release/exit transitions, FLIP layout glides, and
//! cascade staggers.

use super::*;

impl Tree {
    /// `top` went on screen: the elements under it not drawn yet, built
    /// before it was put there, enter with it from their `starting:`
    /// records, as CSS's `@starting-style` applies to every element first
    /// drawn (a screen inside the `<Show>` branch or the mount that brings
    /// it).
    pub(super) fn enter_subtree(&mut self, top: u32, wait: f32) {
        for i in 0..self.entering.len() {
            let d = self.entering[i];
            if d == top || self.els[d as usize].entering != Some(false) {
                continue;
            }
            if !self.is_under(d, top) {
                continue;
            }
            self.els[d as usize].entering = Some(true);
            let wait = wait + self.cascade(d);
            let el = &self.els[d as usize];
            let from = self.classes.presence(el.class, 0);
            // A child its list staggers enters late even with no
            // `starting:` of its own: its `animate-*` starts then.
            if from.is_some() || wait > 0.0 {
                self.motion.enter_after(el.node, &el.record, from, wait);
            }
        }
    }

    /// `e`'s ancestors, nearest first, four at most (`NONE` past the root).
    fn ancestors(&self, e: u32) -> [u32; 4] {
        let mut out = [NONE; 4];
        let mut a = self.els[e as usize].parent;
        for o in &mut out {
            if a == NONE {
                break;
            }
            *o = a;
            a = self.els[a as usize].parent;
        }
        out
    }

    /// The box `e` was last drawn in, in surface space.
    fn drawn_rect(&self, e: u32) -> Option<[f32; 4]> {
        let a = self.abs.get(e as usize).filter(|a| a.2)?;
        let n = self.els[e as usize].node;
        let b = self
            .motion
            .box_of(n)
            .unwrap_or_else(|| self.arena.box_of(n));
        Some([a.0, a.1, b.w, b.h])
    }

    /// A `glide-<name>` element `e` is taken off the screen: where it was
    /// is kept a moment for the one that replaces it.
    pub(super) fn left_glide(&mut self, e: u32) {
        let t = self.time;
        self.glides.retain(|g| t - g.1 < 0.5);
        if let Some(rect) = self.drawn_rect(e) {
            let name = self.els[e as usize].record.glide.clone().expect("a glide");
            let anc = self.ancestors(e);
            self.glides.push((name, t, rect, anc));
        }
    }

    /// A `glide-<name>` element `child` went on screen: it slides from
    /// the nearest element of its name, by their common ancestor, that
    /// shows (the selection it replaces, removed after it) or left it in
    /// the last half second (removed before it), as Framer Motion's
    /// `layoutId` does. Nothing near: it enters as it is.
    pub(super) fn glide_from(&mut self, child: u32) {
        let name = self.els[child as usize]
            .record
            .glide
            .clone()
            .expect("a glide");
        let mine = self.ancestors(child);
        let near = |anc: &[u32; 4]| mine.iter().position(|&a| a != NONE && anc.contains(&a));
        let mut best: Option<(usize, [f32; 4])> = None;
        let mut take = |level: Option<usize>, rect: [f32; 4]| {
            if let Some(l) = level
                && best.is_none_or(|b| l < b.0)
            {
                best = Some((l, rect));
            }
        };
        for d in 0..self.els.len() as u32 {
            let el = &self.els[d as usize];
            if d == child || el.dead || el.record.glide.as_deref() != Some(&*name) {
                continue;
            }
            if let Some(rect) = self.drawn_rect(d) {
                take(near(&self.ancestors(d)), rect);
            }
        }
        let t = self.time;
        for g in self
            .glides
            .iter()
            .filter(|g| *g.0 == *name && t - g.1 < 0.5)
        {
            take(near(&g.3), g.2);
        }
        let Some((_, [x, y, w, h])) = best else {
            return;
        };
        let p = self.els[child as usize].arena_parent;
        let (px, py) = match self.abs.get(p as usize) {
            Some(a) => {
                let [sx, sy] = self.scroll_of(p);
                (a.0 - sx, a.1 - sy)
            }
            None => (0.0, 0.0),
        };
        let node = self.els[child as usize].node;
        let mut from = self.arena.box_of(node);
        (from.x, from.y, from.w, from.h) = (x - px, y - py, w, h);
        self.motion.glide(node, from);
    }

    /// How long `child`, put on screen now, waits for its turn in its
    /// parent's cascade (`stagger-<step>/<cap>`): the n-th child entering
    /// at this time waits n steps, never past the cap. 0 under a parent
    /// with none.
    pub(super) fn cascade(&mut self, child: u32) -> f32 {
        let p = self.els[child as usize].parent;
        let Some([step, cap]) = (p != NONE)
            .then(|| self.els[p as usize].record.stagger)
            .flatten()
        else {
            return 0.0;
        };
        if self.cascade.0 != self.time {
            self.cascade = (self.time, Vec::new());
        }
        let ranks = &mut self.cascade.1;
        let n = match ranks.iter_mut().find(|r| r.0 == p) {
            Some(r) => {
                r.1 += 1;
                r.1 - 1
            }
            None => {
                ranks.push((p, 1));
                0
            }
        };
        (n as f32 * step).min(cap)
    }

    /// Longest remaining exit time of `e` and all descendants under it.
    fn max_leaves_in(&self, e: u32) -> f32 {
        let el = &self.els[e as usize];
        if el.dead {
            return 0.0;
        }
        let mut wait = self.motion.leaves_in(el.node);
        for &c in &el.children {
            wait = wait.max(self.max_leaves_in(c));
        }
        wait
    }

    /// How long `child`, put on screen beside an absolutely placed sibling
    /// on its way out, holds its entrance: until what leaves there has
    /// left, so a screen that replaces one (a `<Show>` branch, a mount)
    /// enters after it instead of over it. 0 for anything else: a list's
    /// rows, which flow, come and go together.
    pub(super) fn handoff(&self, child: u32) -> f32 {
        let abs =
            |e: u32| self.els[e as usize].record.layout.position() == taffy::Position::Absolute;
        if !abs(child) {
            return 0.0;
        }
        let parent = self.els[child as usize].parent;
        let mut wait = 0.0f32;
        if parent != NONE {
            for &s in &self.els[parent as usize].children {
                let el = &self.els[s as usize];
                if s == child || el.dead || !abs(s) {
                    continue;
                }
                if !self.motion.exiting(el.node) && !self.leaving.iter().any(|l| l.0 == s) {
                    continue;
                }
                wait = wait.max(self.max_leaves_in(s));
            }
        } else {
            for s in 0..self.els.len() as u32 {
                let el = &self.els[s as usize];
                if s == child || el.dead || el.parent != NONE || !abs(s) {
                    continue;
                }
                if !self.motion.exiting(el.node) && !self.leaving.iter().any(|l| l.0 == s) {
                    continue;
                }
                wait = wait.max(self.max_leaves_in(s));
            }
        }
        wait
    }

    /// Whether op element `e` covers over a third of the surface: a press
    /// catcher (a screen's "tap to start", a backdrop), not a control.
    pub(in crate::tree) fn catcher(&self, e: u32) -> bool {
        let b = self.arena.box_of(self.els[e as usize].node);
        b.w * b.h * 3.0 > self.size.0 * self.size.1
    }

    /// The element op node `id` is.
    pub(super) fn op(&self, id: u32) -> Result<u32, String> {
        match self.by_js.get(id as usize) {
            Some(&e) if e != NONE => Ok(e),
            _ => Err(format!("og.ui ops: no node {id}")),
        }
    }

    pub(super) fn create(&mut self, id: u32, kind: Kind, text: bool) -> Result<u32, String> {
        if id == 0 || self.by_js.get(id as usize).is_some_and(|&e| e != NONE) {
            return Err(format!("og.ui ops: node {id} exists"));
        }
        let class = self.classes.get("", "");
        self.classes.acquire(class);
        let record = self.classes.records[class as usize].clone();
        let node = self.arena.element(NONE, record.layout.clone());
        let leaf = text.then(|| self.arena.text(node, "", 16.0));
        let fresh = El {
            node,
            disabled: false,
            clickable: false,
            squash: false,
            js: id,
            listens: 0,
            class,
            arena_parent: NONE,
            parent: NONE,
            children: Vec::new(),
            kind,
            leaf,
            text: String::new(),
            hidden: 0,
            dead: false,
            record,
            line: None,
            particles: None,
            anchor: None,
            attrs: Vec::new(),
            data: 0,
            other: false,
            image: None,
            material: None,
            entering: Some(false),
            eff: self.root,
            inline: NONE,
            para: None,
            queued: false,
            ruby: Box::default(),
        };
        let e = if let Some(slot) = self.free_els.pop() {
            self.els[slot as usize] = fresh;
            if let Some(a) = self.abs.get_mut(slot as usize) {
                a.2 = false;
            }
            if let Some(x) = self.xfs.get_mut(slot as usize) {
                *x = 0;
            }
            slot
        } else {
            let slot = self.els.len() as u32;
            self.els.push(fresh);
            slot
        };
        if self.by_js.len() <= id as usize {
            self.by_js.resize(id as usize + 1, NONE);
        }
        self.by_js[id as usize] = e;
        if self.input.per.len() <= e as usize {
            self.input.per.resize(e as usize + 1, input::Per::default());
        } else {
            self.input.per[e as usize] = input::Per::default();
        }
        self.input.per[e as usize].tab = input::NO_TAB;
        self.entering.push(e);
        self.inherit(e);
        Ok(e)
    }

    /// Frees element `e` and every element under it. Inside an element on
    /// its way out, it is freed when that exit settles; its id is free now.
    pub(super) fn release(&mut self, e: u32) {
        if self.leaving_in(e) {
            self.unbind(e);
            match self.leaving.iter_mut().find(|l| l.0 == e) {
                Some(l) => l.1 = true,
                None => self.leaving.push((e, true)),
            }
            return;
        }
        self.free(e);
    }

    /// Whether `e` or an element it is in is on its way out: exiting, or
    /// removed and held while what is under it exits.
    pub(super) fn leaving_in(&self, mut e: u32) -> bool {
        while e != NONE {
            if self.motion.exiting(self.els[e as usize].node)
                || self.leaving.iter().any(|l| l.0 == e)
            {
                return true;
            }
            e = self.els[e as usize].parent;
        }
        false
    }

    /// `top` was removed with no `exit:` record of its own: the elements
    /// under it with one leave to it (a screen inside the `<Show>` branch
    /// or the mount that goes), as a removed element with one does. True
    /// when any animates out: hold `top` drawn, not hit, until they settle.
    pub(super) fn exit_under(&mut self, top: u32) -> bool {
        let mut any = false;
        let mut stack: Vec<u32> = self.els[top as usize].children.clone();
        while let Some(d) = stack.pop() {
            let el = &self.els[d as usize];
            if el.dead {
                continue;
            }
            stack.extend_from_slice(&el.children);
            if el.js == NONE {
                continue;
            }
            let to = self.classes.presence(el.class, 1);
            any |= to.is_some() && self.motion.exit(el.node, &el.record, to);
        }
        any
    }

    /// Whether an element under `top` is on its way out.
    fn exiting_under(&self, top: u32) -> bool {
        fn check(tree: &Tree, e: u32) -> bool {
            for &c in &tree.els[e as usize].children {
                let el = &tree.els[c as usize];
                if el.dead {
                    continue;
                }
                if tree.motion.exiting(el.node) || check(tree, c) {
                    return true;
                }
            }
            false
        }
        check(self, top)
    }

    /// Whether element `d` is below `top` (not `top` itself).
    fn is_under(&self, d: u32, top: u32) -> bool {
        let mut a = self.els[d as usize].parent;
        while a != NONE && a != top {
            a = self.els[a as usize].parent;
        }
        a == top && d != top
    }

    /// The ids of `e` and what is under it go back to the renderer.
    fn unbind(&mut self, e: u32) {
        let js = self.els[e as usize].js;
        // What is on its way out is no longer in the accessibility tree.
        self.sem.exiting(self.els[e as usize].node, true);
        if let Some(slot) = self.by_js.get_mut(js as usize).filter(|s| **s == e) {
            *slot = NONE;
        }
        for k in 0..self.els[e as usize].children.len() {
            let c = self.els[e as usize].children[k];
            if !self.els[c as usize].dead {
                self.unbind(c);
            }
        }
    }

    /// Frees `e` and what is under it now, recycling both element and arena slots.
    pub(super) fn free(&mut self, e: u32) {
        if self.els[e as usize].dead {
            return;
        }
        let kids = std::mem::take(&mut self.els[e as usize].children);
        for c in kids {
            if self.els[c as usize].parent == e {
                self.els[c as usize].parent = NONE;
            }
            if !self.els[c as usize].dead {
                self.free(c);
            }
        }
        self.forget_el(e, self.els[e as usize].node);
        self.set_parent(e, NONE);
        let el = &mut self.els[e as usize];
        el.dead = true;
        let (node, leaf, js, class) = (el.node, el.leaf.take(), el.js, el.class);
        el.js = NONE;
        el.arena_parent = NONE;
        el.text.clear();
        el.attrs.clear();
        el.line = None;
        el.particles = None;
        el.anchor = None;
        el.image = None;
        el.para = None;
        self.classes.release(class);
        if let Some(leaf) = leaf {
            self.texts.release(leaf);
            self.arena.release(leaf);
        }
        self.arena.release(node);
        self.motion.forget(node);
        self.sem.release(node);
        self.anchored.retain(|&a| a != e);
        self.spotlights.retain(|&s| s != e);
        self.leaving.retain(|l| l.0 != e);
        self.entering.retain(|&x| x != e);
        self.counts.retain(|c| c.e != e);
        self.paras.retain(|&p| p != e);
        self.warned_layers.retain(|&n| n != node);
        self.warned_paras.retain(|&x| x != e);
        self.glides.retain(|g| !g.3.contains(&e));
        self.cascade.1.retain(|r| r.0 != e);
        if self.select.as_ref().is_some_and(|s| s.e == e) {
            self.select = None;
        }
        if self.select_press.as_ref().is_some_and(|p| p.e == e) {
            self.select_press = None;
        }
        if let Some(a) = self.abs.get_mut(e as usize) {
            a.2 = false;
        }
        if let Some(x) = self.xfs.get_mut(e as usize) {
            *x = 0;
        }
        if let Some(slot) = self.by_js.get_mut(js as usize).filter(|s| **s == e) {
            *slot = NONE;
        }
        self.ordered = false;
        self.free_els.push(e);
    }

    /// The exits that settled in this frame's motion tick: a removed
    /// element leaves its parent, a released one is freed, and what was
    /// released inside it goes with it.
    pub(in crate::tree) fn settle_exits(&mut self) {
        let mut gone = std::mem::take(&mut self.gone);
        gone.extend(self.motion.take_exited());
        for &node in &gone {
            let Some(k) = self
                .leaving
                .iter()
                .position(|l| self.els[l.0 as usize].node == node)
            else {
                continue;
            };
            let (e, release) = self.leaving.swap_remove(k);
            if release {
                self.free(e);
            } else {
                self.arena.detach(node);
                self.set_parent(e, NONE);
                self.els[e as usize].arena_parent = NONE;
            }
            self.ordered = false;
            self.paint = true;
        }
        gone.clear();
        self.gone = gone;
        // What the renderer released inside an exit that ended, and a
        // removed element held while what is under it exited.
        let mut k = 0;
        while k < self.leaving.len() {
            let (e, release) = self.leaving[k];
            let node = self.els[e as usize].node;
            let held = self.motion.exiting(node)
                || self.exiting_under(e)
                || self.leaving_in(self.els[e as usize].parent);
            if held {
                k += 1;
            } else if release {
                self.free(e);
            } else {
                self.leaving.swap_remove(k);
                self.arena.detach(node);
                self.set_parent(e, NONE);
                self.els[e as usize].arena_parent = NONE;
                self.ordered = false;
                self.paint = true;
            }
        }
    }
}
