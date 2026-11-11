//! Gesture recognisers and focus groups (docs/ui.md §2.1, §2.1).
//!
//! **Tap, double tap and long press** are recognised at the start of a
//! frame against the last frame's recognisers ([`crate::GestureSpec`] on
//! an element, collected at `end`), from the press, how far it travelled
//! and how many ticks it was held, all in the UI state. One press is an
//! arena: every element at or around the pressed one that declares a
//! gesture takes part, and for each gesture the highest priority wins it
//! (ties: the innermost). A tap waits for a double tap only where the
//! winner declares one, so a plain tap answers on the release; a long press
//! that fires takes the press (its release clicks nothing and taps
//! nothing); travel past the winner's slop fails both.
//!
//! **Focus groups** ([`crate::FocusGroup`]) keep, between frames, their
//! spec, the group around them and the member they last had, which focus
//! coming in from outside returns to.

use crate::element::{FocusGroup, GestureSpec};
use crate::input::InputFrame;
use crate::layout::{NONE, Node};
use crate::ui::{HIT_NAV, HIT_SELECT, HIT_TEXT, Ui};
use og_core::IdMap;
use og_core::glam::Vec2;

/// A focus group as the last frame had it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct GroupState {
    pub spec: FocusGroup,
    /// The group around it (0: none).
    pub parent: u64,
    /// The member the focus was last on inside it (0: none yet).
    pub last: u64,
    /// Its first member in reading order (0: none).
    pub first: u64,
}

/// The press being recognised and a tap waiting for its double.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct TapState {
    /// The recognising elements the held press started on, innermost first
    /// (empty: no press is being recognised).
    pub chain: Vec<u64>,
    /// Where the press started and the tick it did.
    pub start: Vec2,
    pub since: u64,
    /// The furthest the pointer travelled from `start`, CSS px.
    pub travel: f32,
    /// Its long press fired: the press is taken (its release clicks nothing).
    pub taken: bool,
    /// A tap waiting to see whether a second one makes it a double tap:
    /// the element (0: none), the tick of the first tap and where it was.
    pub waiting: u64,
    pub waiting_tick: u64,
    pub waiting_at: Vec2,
}

/// The defaults of a spec's zero values.
fn long_ms(s: &GestureSpec) -> f32 {
    if s.long_press_ms > 0.0 {
        s.long_press_ms
    } else {
        500.0
    }
}

fn double_ms(s: &GestureSpec) -> f32 {
    if s.double_tap_ms > 0.0 {
        s.double_tap_ms
    } else {
        300.0
    }
}

fn slop(s: &GestureSpec) -> f32 {
    if s.slop > 0.0 { s.slop } else { 10.0 }
}

impl Ui {
    /// The element of `chain` that wins a gesture `wants`: the highest
    /// priority, the innermost of equals.
    fn winner(&self, chain: &[u64], wants: impl Fn(&GestureSpec) -> bool) -> Option<u64> {
        let mut best: Option<(i32, u64)> = None;
        for &k in chain {
            let Some((s, _)) = self.interaction.state.recognizers.get(&k) else {
                continue;
            };
            if wants(s) && best.is_none_or(|(p, _)| s.priority > p) {
                best = Some((s.priority, k));
            }
        }
        best.map(|b| b.1)
    }

    fn spec(&self, key: u64) -> Option<GestureSpec> {
        self.interaction
            .state
            .recognizers
            .get(&key)
            .map(|(s, _)| *s)
    }

    /// This tick's taps, double taps and long presses, from `input` and the
    /// hit under the press (`hot`, not a scrollbar grab), into the frame's
    /// navigation answers.
    pub(crate) fn recognize(&mut self, input: &InputFrame, hot: u64, grabbed: bool) {
        let now = self.interaction.state.tick;
        let tps = self.env.ticks_per_second;
        let ticks = |ms: f32| u64::from(crate::style::Theme::ticks(ms, tps));
        // A tap left waiting for its double: past the window, it is a tap.
        let w = self.interaction.state.tap.waiting;
        if w != 0 {
            match self.spec(w) {
                Some(s) if now - self.interaction.state.tap.waiting_tick > ticks(double_ms(&s)) => {
                    if s.tap {
                        self.interaction.nav.tapped = w;
                    }
                    self.interaction.state.tap.waiting = 0;
                }
                Some(_) => {}
                None => self.interaction.state.tap.waiting = 0,
            }
        }
        let p = input.pointer;
        if input.pressed {
            let mut chain = Vec::new();
            let mut k = if grabbed {
                0
            } else {
                self.hit(hot).map_or(0, |h| h.gesture)
            };
            while k != 0 && chain.len() < 16 {
                chain.push(k);
                k = self
                    .interaction
                    .state
                    .recognizers
                    .get(&k)
                    .map_or(0, |r| r.1);
            }
            let t = &mut self.interaction.state.tap;
            t.chain = chain;
            t.start = p;
            t.since = now;
            t.travel = 0.0;
            t.taken = false;
        }
        if self.interaction.state.tap.chain.is_empty() {
            return;
        }
        let travel = self
            .interaction
            .state
            .tap
            .travel
            .max((p - self.interaction.state.tap.start).length());
        self.interaction.state.tap.travel = travel;
        let chain = self.interaction.state.tap.chain.clone();
        if (input.down || input.released) && !self.interaction.state.tap.taken {
            let held = now - self.interaction.state.tap.since;
            if let Some(w) = self.winner(&chain, |s| s.long_press) {
                let s = self.spec(w).unwrap_or_default();
                if travel <= slop(&s) && held >= ticks(long_ms(&s)).max(1) {
                    self.interaction.nav.long_pressed = w;
                    self.interaction.state.tap.taken = true;
                    self.interaction.state.tap.waiting = 0;
                }
            }
        }
        if input.released || !input.down {
            if !self.interaction.state.tap.taken
                && input.released
                && let Some(w) = self.winner(&chain, |s| s.tap || s.double_tap)
            {
                let s = self.spec(w).unwrap_or_default();
                let over = self.hit(w).is_some_and(|h| h.contains(p.x, p.y));
                if travel <= slop(&s) && over {
                    let t = &self.interaction.state.tap;
                    // Another element's waiting tap is settled by this one.
                    if t.waiting != 0 && t.waiting != w {
                        if self.spec(t.waiting).is_some_and(|o| o.tap) {
                            self.interaction.nav.tapped = t.waiting;
                        }
                        self.interaction.state.tap.waiting = 0;
                    }
                    let t = &self.interaction.state.tap;
                    if !s.double_tap {
                        self.interaction.nav.tapped = w;
                    } else if t.waiting == w
                        && now - t.waiting_tick <= ticks(double_ms(&s))
                        && (p - t.waiting_at).length() <= 2.0 * slop(&s)
                    {
                        self.interaction.nav.double_tapped = w;
                        self.interaction.state.tap.waiting = 0;
                    } else {
                        let t = &mut self.interaction.state.tap;
                        t.waiting = w;
                        t.waiting_tick = now;
                        t.waiting_at = p;
                    }
                }
            }
            self.interaction.state.tap.chain.clear();
        }
    }

    /// How far into its long press the held press is on `key` (0–1; 0:
    /// none or it is not the press's long-press winner).
    pub(crate) fn long_press_progress(&self, key: u64) -> f32 {
        let t = &self.interaction.state.tap;
        if t.taken || t.chain.is_empty() || !self.interaction.input.down {
            return 0.0;
        }
        if self.winner(&t.chain, |s| s.long_press) != Some(key) {
            return 0.0;
        }
        let s = self.spec(key).unwrap_or_default();
        if t.travel > slop(&s) {
            return 0.0;
        }
        let total = self.ticks_for(long_ms(&s)).max(1) as f32;
        ((self.interaction.state.tick - t.since) as f32 / total).clamp(0.0, 1.0)
    }

    /// Keys and pads tap a recognising control by activating it.
    pub(crate) fn tapped_by_keys(&self, key: u64) -> bool {
        key != 0
            && (self.interaction.nav.activate == key
                || self.interaction.nav.pad_activate.contains(&key))
            && self.spec(key).is_some_and(|s| s.tap)
    }

    /// The focus groups, explicit focus moves and gesture recognisers of
    /// this frame's nodes for the next frame, from the nodes that have
    /// them and each node's nearest group and recogniser around it
    /// (`visit::Orders::group_of`, `gesture_of`).
    pub(crate) fn collect_input_specs(&mut self) {
        let o = &self.tree.orders;
        // A group's parent group: its parent's (a layer's anchor's, where
        // the anchor is drawn before it).
        let around = |i: usize, of: &[u64]| {
            let n = &self.tree.nodes[i];
            let up = if n.parent == NONE { n.anchor } else { n.parent };
            if up == NONE || up >= self.tree.nodes.len() || o.pos[up] > o.pos[i] {
                0
            } else {
                of[up]
            }
        };
        let mut groups: IdMap<u64, GroupState> = IdMap::new();
        for &i in o.with(crate::ui::visit::feat::GROUP) {
            let n = &self.tree.nodes[i];
            let Some(spec) = n.props.focus_group else {
                continue;
            };
            let last = self
                .interaction
                .state
                .groups
                .get(&n.key)
                .map_or(0, |g| g.last);
            groups.insert(
                n.key,
                GroupState {
                    spec,
                    parent: around(i, &o.group_of),
                    last,
                    first: 0,
                },
            );
        }
        let mut recognizers: IdMap<u64, (GestureSpec, u64)> = IdMap::new();
        for &i in o.with(crate::ui::visit::feat::RECOGNIZER) {
            let n = &self.tree.nodes[i];
            if let Some(spec) = n.props.gestures.filter(GestureSpec::any)
                && !n.inert
            {
                recognizers.insert(n.key, (spec, around(i, &o.gesture_of)));
            }
        }
        self.interaction.state.groups = groups;
        self.interaction.state.recognizers = recognizers;
        // Explicit moves: each path resolved to where it lands.
        let mut links: IdMap<u64, [u64; 6]> = IdMap::new();
        for i in self.tree.orders.drawn(crate::ui::visit::feat::NAV_LINKS) {
            let nav = self.tree.nodes[i].props.nav.clone();
            let mut to = [0u64; 6];
            for (k, path) in [
                &nav.up, &nav.down, &nav.left, &nav.right, &nav.next, &nav.prev,
            ]
            .into_iter()
            .enumerate()
            {
                if path.is_empty() {
                    continue;
                }
                match self.node_at(path, i) {
                    Some(j) => to[k] = self.landing(j),
                    None => {
                        let at = self.path_of(i);
                        self.out.warnings.push(format!(
                            "'{at}' moves focus to '{path}', which is not on screen: nav takes a full path (Hud/Bag) or a sibling's id"
                        ));
                    }
                }
            }
            links.insert(self.tree.nodes[i].key, to);
        }
        self.interaction.state.nav_links = links;
    }

    /// Where a focus move to node `j` lands: a group (resolved when the
    /// move is made), a focusable control, or the first focusable control
    /// inside it.
    fn landing(&self, j: usize) -> u64 {
        let n = &self.tree.nodes[j];
        if n.props.focus_group.is_some() || n.nav || n.focusable {
            return n.key;
        }
        // Its subtree in drawing order.
        let mut sub = crate::layout::preorder(&self.tree.nodes, j);
        sub.sort_unstable_by_key(|&o| self.tree.orders.pos[o]);
        sub.into_iter()
            .find(|&o| {
                let m: &Node = &self.tree.nodes[o];
                (m.nav || m.focusable) && !m.inert
            })
            .map_or(0, |o| self.tree.nodes[o].key)
    }

    /// After this frame's hits: each group's first member, and the member
    /// the focus is on remembered by every group around it.
    pub(crate) fn note_groups(&mut self) {
        if self.interaction.state.groups.is_empty() {
            return;
        }
        let hits = std::mem::take(&mut self.interaction.state.hits);
        for h in hits.iter().filter(|h| h.has(HIT_NAV | HIT_TEXT)) {
            let mut g = h.group;
            while g != 0 {
                let Some(gs) = self.interaction.state.groups.get_mut(&g) else {
                    break;
                };
                if gs.first == 0 {
                    gs.first = h.key;
                }
                if h.key == self.interaction.state.focus && gs.spec.memory {
                    gs.last = h.key;
                }
                g = gs.parent;
            }
        }
        self.interaction.state.hits = hits;
    }

    /// Whether group `inner` is `outer` or inside it.
    pub(crate) fn in_group(&self, mut inner: u64, outer: u64) -> bool {
        if outer == 0 {
            return true;
        }
        while inner != 0 {
            if inner == outer {
                return true;
            }
            inner = self
                .interaction
                .state
                .groups
                .get(&inner)
                .map_or(0, |g| g.parent);
        }
        false
    }
}

impl Ui {
    /// This tick's taps, double taps and long presses as frame events
    /// (`tap`, `doubleTap`, `longPress`, each naming the element's path).
    pub(crate) fn note_taps(&mut self) {
        let fired = [
            ("tap", self.interaction.nav.tapped),
            ("doubleTap", self.interaction.nav.double_tapped),
            ("longPress", self.interaction.nav.long_pressed),
        ];
        // Keys and pads tap what they activate.
        let keyed = [self.interaction.nav.activate]
            .into_iter()
            .chain(self.interaction.nav.pad_activate)
            .filter(|&k| self.tapped_by_keys(k) && k != self.interaction.nav.tapped);
        let keyed: Vec<u64> = keyed.collect();
        for (kind, key) in fired
            .into_iter()
            .chain(keyed.into_iter().map(|k| ("tap", k)))
        {
            if key == 0 {
                continue;
            }
            let Some(i) = self.tree.index.find(key) else {
                continue;
            };
            self.out.events.push(crate::motion::UiEvent {
                kind: kind.into(),
                target: self.path_of(i),
                pitch: 1.0,
                ..crate::motion::UiEvent::default()
            });
        }
    }

    /// The pointer's shape this frame: over the element the press holds,
    /// else the one under the pointer, the first cursor set on it or around
    /// it, else its own kind's (a text control's I-beam, a draggable
    /// element's hand, a link's), else the arrow. With it, the `url()`
    /// images declared beside the keyword that won (CSS UI 4 §5.1: the
    /// property inherits whole, images and keyword together).
    pub(crate) fn resolve_cursor(
        &self,
        order: &[usize],
    ) -> (crate::Cursor, Vec<crate::CursorImage>) {
        let (cursor, at) = self.cursor_at(order);
        let images = if at == NONE {
            Vec::new()
        } else {
            self.tree.nodes[at].props.cursor_images.clone()
        };
        (cursor, images)
    }

    /// The resolved keyword and the node that declared it (`NONE`: a
    /// default of the element's kind).
    fn cursor_at(&self, order: &[usize]) -> (crate::Cursor, usize) {
        use crate::Cursor;
        let mut key = if self.interaction.state.active != 0 {
            self.interaction.state.active
        } else {
            self.interaction.state.hot
        };
        let hovered = if self.interaction.state.active == 0 {
            let Some(i) = self.cursor_node(order) else {
                return (Cursor::Default, NONE);
            };
            // Keep a control's link, resize handle or native cursor when
            // its noninteractive child is on top, but not through a sibling.
            let hot = self.tree.index.find(key);
            let mut ancestor = i;
            loop {
                if Some(ancestor) == hot
                    || crate::ui::resize::grip_key(self.tree.nodes[ancestor].key) == key
                {
                    break;
                }
                ancestor = self.tree.nodes[ancestor].parent;
                if ancestor == NONE {
                    key = self.tree.nodes[i].key;
                    break;
                }
            }
            Some(i)
        } else {
            None
        };
        // Over selectable text (or selecting it): its own cursor where one
        // is set on it or above it, else the I-beam.
        let selecting = self.hit(key).filter(|h| h.has(HIT_SELECT)).map(|h| h.key);
        if let Some(t) = selecting.or_else(|| {
            hovered
                .and_then(|i| self.hit(self.tree.nodes[i].key))
                .filter(|h| h.has(HIT_SELECT))
                .map(|h| h.key)
        }) && let Some(i) = self.tree.index.find(t)
        {
            return match self.declared_cursor(i) {
                (Cursor::Auto, at) => (Cursor::Text, at),
                c => c,
            };
        }
        let Some(i) = self.tree.index.find(key) else {
            // A resize handle: the resize cursor of its axes.
            return match self
                .tree
                .orders
                .with(crate::ui::visit::feat::RESIZE)
                .iter()
                .map(|&i| &self.tree.nodes[i])
                .find(|n| crate::ui::resize::grip_key(n.key) == key)
                .map(|n| n.el.resize)
            {
                Some(1) => (Cursor::ResizeEw, NONE),
                Some(2) => (Cursor::ResizeNs, NONE),
                Some(_) if self.env.direction == og_i18n::Direction::Rtl => {
                    (Cursor::ResizeNesw, NONE)
                }
                Some(_) => (Cursor::ResizeNwse, NONE),
                None => (Cursor::Default, NONE),
            };
        };
        let (c, at) = self.declared_cursor(hovered.unwrap_or(i));
        if c != Cursor::Auto {
            return (c, at);
        }
        let n = &self.tree.nodes[i];
        if key != n.key {
            // A rich text's link.
            return (Cursor::Pointer, at);
        }
        if n.focusable {
            return (Cursor::Text, at);
        }
        if n.props.drag.is_some() {
            let dragging = self
                .interaction
                .state
                .motion
                .gestures
                .get(&key)
                .is_some_and(|g| g.dragging || (g.held && self.interaction.input.down));
            return if dragging {
                (Cursor::Grabbing, at)
            } else {
                (Cursor::Grab, at)
            };
        }
        (Cursor::Default, at)
    }

    /// Cursor targeting includes plain boxes without giving them input hits.
    fn cursor_node(&self, order: &[usize]) -> Option<usize> {
        use crate::element::{PointerEvents, SvgPointer};
        let p = self.interaction.state.pointer;
        order.iter().rev().copied().find(|&i| {
            let n = &self.tree.nodes[i];
            let pointer = self.tree.orders.pointer_of[i];
            let svg = match pointer {
                PointerEvents::Svg(bits)
                    if self.draw_of(i).is_some_and(|d| d.view_box.is_some()) =>
                {
                    Some(bits.0)
                }
                _ => None,
            };
            if i == 0
                || n.inert
                || pointer == PointerEvents::None
                || self.tree.orders.skip_of[i] != 0
                || (self.tree.orders.hidden_of[i]
                    && !svg.is_some_and(|s| s & SvgPointer::VISIBLE == 0))
                || !n.clip.contains(p.x, p.y)
            {
                return false;
            }
            let Some(inv) = n.world.invert() else {
                return false;
            };
            let (x, y) = inv.apply(p.x, p.y);
            if !n.rect.contains(x, y) {
                return false;
            }
            let mut parent = n.parent;
            while parent != NONE {
                let ancestor = &self.tree.nodes[parent];
                if ancestor.decor.as_ref().is_some_and(|d| d.clip) {
                    let Some(inv) = ancestor.world.invert() else {
                        return false;
                    };
                    let (x, y) = inv.apply(p.x, p.y);
                    if !ancestor.rect.contains(x, y) {
                        return false;
                    }
                }
                parent = ancestor.parent;
            }
            svg.is_none_or(|bits| {
                bits & SvgPointer::BOX != 0
                    || self.draw_of(i).is_some_and(|d| {
                        d.covers(
                            [n.rect.width, n.rect.height],
                            Vec2::new(x - n.rect.x, y - n.rect.y),
                            bits,
                        )
                    })
            })
        })
    }

    /// The `cursor` declared on node `i` or the nearest element around it
    /// that declares one (a keyword other than `auto`, or images over
    /// `auto`), and that element (`NONE`: none does).
    fn declared_cursor(&self, i: usize) -> (crate::Cursor, usize) {
        let mut j = i;
        while j != NONE && j < self.tree.nodes.len() {
            let n = &self.tree.nodes[j];
            if n.props.cursor != crate::Cursor::Auto || !n.props.cursor_images.is_empty() {
                return (n.props.cursor, j);
            }
            j = if n.parent == NONE { n.anchor } else { n.parent };
        }
        (crate::Cursor::Auto, NONE)
    }
}
