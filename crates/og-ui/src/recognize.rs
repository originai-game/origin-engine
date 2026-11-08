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
            let Some((s, _)) = self.state.recognizers.get(&k) else {
                continue;
            };
            if wants(s) && best.is_none_or(|(p, _)| s.priority > p) {
                best = Some((s.priority, k));
            }
        }
        best.map(|b| b.1)
    }

    fn spec(&self, key: u64) -> Option<GestureSpec> {
        self.state.recognizers.get(&key).map(|(s, _)| *s)
    }

    /// This tick's taps, double taps and long presses, from `input` and the
    /// hit under the press (`hot`, not a scrollbar grab), into the frame's
    /// navigation answers.
    pub(crate) fn recognize(&mut self, input: &InputFrame, hot: u64, grabbed: bool) {
        let now = self.state.tick;
        let tps = self.ticks_per_second;
        let ticks = |ms: f32| u64::from(crate::style::Theme::ticks(ms, tps));
        // A tap left waiting for its double: past the window, it is a tap.
        let w = self.state.tap.waiting;
        if w != 0 {
            match self.spec(w) {
                Some(s) if now - self.state.tap.waiting_tick > ticks(double_ms(&s)) => {
                    if s.tap {
                        self.nav.tapped = w;
                    }
                    self.state.tap.waiting = 0;
                }
                Some(_) => {}
                None => self.state.tap.waiting = 0,
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
                k = self.state.recognizers.get(&k).map_or(0, |r| r.1);
            }
            let t = &mut self.state.tap;
            t.chain = chain;
            t.start = p;
            t.since = now;
            t.travel = 0.0;
            t.taken = false;
        }
        if self.state.tap.chain.is_empty() {
            return;
        }
        let travel = self
            .state
            .tap
            .travel
            .max((p - self.state.tap.start).length());
        self.state.tap.travel = travel;
        let chain = self.state.tap.chain.clone();
        if (input.down || input.released) && !self.state.tap.taken {
            let held = now - self.state.tap.since;
            if let Some(w) = self.winner(&chain, |s| s.long_press) {
                let s = self.spec(w).unwrap_or_default();
                if travel <= slop(&s) && held >= ticks(long_ms(&s)).max(1) {
                    self.nav.long_pressed = w;
                    self.state.tap.taken = true;
                    self.state.tap.waiting = 0;
                }
            }
        }
        if input.released || !input.down {
            if !self.state.tap.taken
                && input.released
                && let Some(w) = self.winner(&chain, |s| s.tap || s.double_tap)
            {
                let s = self.spec(w).unwrap_or_default();
                let over = self.hit(w).is_some_and(|h| h.contains(p.x, p.y));
                if travel <= slop(&s) && over {
                    let t = &self.state.tap;
                    // Another element's waiting tap is settled by this one.
                    if t.waiting != 0 && t.waiting != w {
                        if self.spec(t.waiting).is_some_and(|o| o.tap) {
                            self.nav.tapped = t.waiting;
                        }
                        self.state.tap.waiting = 0;
                    }
                    let t = &self.state.tap;
                    if !s.double_tap {
                        self.nav.tapped = w;
                    } else if t.waiting == w
                        && now - t.waiting_tick <= ticks(double_ms(&s))
                        && (p - t.waiting_at).length() <= 2.0 * slop(&s)
                    {
                        self.nav.double_tapped = w;
                        self.state.tap.waiting = 0;
                    } else {
                        let t = &mut self.state.tap;
                        t.waiting = w;
                        t.waiting_tick = now;
                        t.waiting_at = p;
                    }
                }
            }
            self.state.tap.chain.clear();
        }
    }

    /// How far into its long press the held press is on `key` (0–1; 0:
    /// none or it is not the press's long-press winner).
    pub(crate) fn long_press_progress(&self, key: u64) -> f32 {
        let t = &self.state.tap;
        if t.taken || t.chain.is_empty() || !self.input.down {
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
        ((self.state.tick - t.since) as f32 / total).clamp(0.0, 1.0)
    }

    /// Keys and pads tap a recognising control by activating it.
    pub(crate) fn tapped_by_keys(&self, key: u64) -> bool {
        key != 0
            && (self.nav.activate == key || self.nav.pad_activate.contains(&key))
            && self.spec(key).is_some_and(|s| s.tap)
    }

    /// The focus groups, explicit focus moves and gesture recognisers of
    /// this frame's nodes for the next frame, from the nodes that have
    /// them and each node's nearest group and recogniser around it
    /// (`visit::Orders::group_of`, `gesture_of`).
    pub(crate) fn collect_input_specs(&mut self) {
        let o = &self.orders;
        // A group's parent group: its parent's (a layer's anchor's, where
        // the anchor is drawn before it).
        let around = |i: usize, of: &[u64]| {
            let n = &self.nodes[i];
            let up = if n.parent == NONE { n.anchor } else { n.parent };
            if up == NONE || up >= self.nodes.len() || o.pos[up] > o.pos[i] {
                0
            } else {
                of[up]
            }
        };
        let mut groups: IdMap<u64, GroupState> = IdMap::new();
        for &i in o.with(crate::ui::visit::feat::GROUP) {
            let n = &self.nodes[i];
            let Some(spec) = n.props.focus_group else {
                continue;
            };
            let last = self.state.groups.get(&n.key).map_or(0, |g| g.last);
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
            let n = &self.nodes[i];
            if let Some(spec) = n.props.gestures.filter(GestureSpec::any)
                && !n.inert
            {
                recognizers.insert(n.key, (spec, around(i, &o.gesture_of)));
            }
        }
        self.state.groups = groups;
        self.state.recognizers = recognizers;
        // Explicit moves: each path resolved to where it lands.
        let mut links: IdMap<u64, [u64; 6]> = IdMap::new();
        for i in self.orders.drawn(crate::ui::visit::feat::NAV_LINKS) {
            let nav = self.nodes[i].props.nav.clone();
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
                        self.warnings.push(format!(
                            "'{at}' moves focus to '{path}', which is not on screen: nav takes a full path (Hud/Bag) or a sibling's id"
                        ));
                    }
                }
            }
            links.insert(self.nodes[i].key, to);
        }
        self.state.nav_links = links;
    }

    /// Where a focus move to node `j` lands: a group (resolved when the
    /// move is made), a focusable control, or the first focusable control
    /// inside it.
    fn landing(&self, j: usize) -> u64 {
        let n = &self.nodes[j];
        if n.props.focus_group.is_some() || n.nav || n.focusable {
            return n.key;
        }
        // Its subtree in drawing order.
        let mut sub = crate::layout::preorder(&self.nodes, j);
        sub.sort_unstable_by_key(|&o| self.orders.pos[o]);
        sub.into_iter()
            .find(|&o| {
                let m: &Node = &self.nodes[o];
                (m.nav || m.focusable) && !m.inert
            })
            .map_or(0, |o| self.nodes[o].key)
    }

    /// After this frame's hits: each group's first member, and the member
    /// the focus is on remembered by every group around it.
    pub(crate) fn note_groups(&mut self) {
        if self.state.groups.is_empty() {
            return;
        }
        let hits = std::mem::take(&mut self.state.hits);
        for h in hits.iter().filter(|h| h.has(HIT_NAV | HIT_TEXT)) {
            let mut g = h.group;
            while g != 0 {
                let Some(gs) = self.state.groups.get_mut(&g) else {
                    break;
                };
                if gs.first == 0 {
                    gs.first = h.key;
                }
                if h.key == self.state.focus && gs.spec.memory {
                    gs.last = h.key;
                }
                g = gs.parent;
            }
        }
        self.state.hits = hits;
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
            inner = self.state.groups.get(&inner).map_or(0, |g| g.parent);
        }
        false
    }
}

impl Ui {
    /// This tick's taps, double taps and long presses as frame events
    /// (`tap`, `doubleTap`, `longPress`, each naming the element's path).
    pub(crate) fn note_taps(&mut self) {
        let fired = [
            ("tap", self.nav.tapped),
            ("doubleTap", self.nav.double_tapped),
            ("longPress", self.nav.long_pressed),
        ];
        // Keys and pads tap what they activate.
        let keyed = [self.nav.activate]
            .into_iter()
            .chain(self.nav.pad_activate)
            .filter(|&k| self.tapped_by_keys(k) && k != self.nav.tapped);
        let keyed: Vec<u64> = keyed.collect();
        for (kind, key) in fired
            .into_iter()
            .chain(keyed.into_iter().map(|k| ("tap", k)))
        {
            if key == 0 {
                continue;
            }
            let Some(i) = self.tree.find(key) else {
                continue;
            };
            self.events.push(crate::motion::UiEvent {
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
    /// element's hand, a link's), else the arrow.
    pub(crate) fn resolve_cursor(&self) -> crate::Cursor {
        use crate::Cursor;
        let key = if self.state.active != 0 {
            self.state.active
        } else {
            self.state.hot
        };
        // Over selectable text (or selecting it): its own cursor where one
        // is set on it or above it, else the I-beam.
        let selecting = self.hit(key).filter(|h| h.has(HIT_SELECT)).map(|h| h.key);
        if let Some(t) = selecting.or_else(|| {
            (self.state.active == 0)
                .then(|| self.select_hit_at(self.state.pointer, key))
                .flatten()
        }) && let Some(i) = self.tree.find(t)
        {
            let mut j = i;
            while j != NONE && j < self.nodes.len() {
                let c = self.nodes[j].props.cursor;
                if c != Cursor::Auto {
                    return c;
                }
                let n = &self.nodes[j];
                j = if n.parent == NONE { n.anchor } else { n.parent };
            }
            return Cursor::Text;
        }
        let Some(i) = self.tree.find(key) else {
            return Cursor::Default;
        };
        let mut j = i;
        while j != NONE && j < self.nodes.len() {
            let c = self.nodes[j].props.cursor;
            if c != Cursor::Auto {
                return c;
            }
            let n = &self.nodes[j];
            j = if n.parent == NONE { n.anchor } else { n.parent };
        }
        let n = &self.nodes[i];
        if key != n.key {
            // A rich text's link.
            return Cursor::Pointer;
        }
        if n.focusable {
            return Cursor::Text;
        }
        if n.props.drag.is_some() {
            let dragging = self
                .state
                .motion
                .gestures
                .get(&key)
                .is_some_and(|g| g.dragging || (g.held && self.input.down));
            return if dragging {
                Cursor::Grabbing
            } else {
                Cursor::Grab
            };
        }
        Cursor::Default
    }
}
