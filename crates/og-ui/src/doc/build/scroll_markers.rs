//! CSS Overflow 5 §3's scroll navigation controls: a scroll container's
//! `::scroll-marker-group`, holding the `::scroll-marker` of each element
//! inside it, and its `::scroll-button()`s, built as its siblings when it
//! closes; and `scroll-target-group`'s `<a href="#id">` markers. A press
//! scrolls once the frame is laid out. Which marker is current (§3.1.8)
//! and which buttons are disabled is decided at the offsets just laid
//! out, and reaches the selectors (`:target-current`, `:target-before`,
//! `:target-after`, `:disabled`) through [`Ui::set_selector_state`] from
//! the next frame on. A multi-column element's `::column::scroll-marker`
//! (CSS Multicol 2 §4.8) gives each of its columns a marker that scrolls
//! to that column.
use super::*;
use crate::bind::InteractionState::{self, Disabled, TargetAfter, TargetBefore, TargetCurrent};
use crate::doc::generated::BUTTONS;
use og_core::glam::Vec2;

/// One element's `::scroll-marker`, as the element was last built.
#[derive(Clone)]
pub struct Marker {
    /// Its label: its `content`, evaluated where its element stands.
    text: String,
    /// The pseudo-element's key: what its selector states are set on.
    key: u64,
    /// Its label's text style.
    desc: LabelDesc,
    /// The document node its box rules resolved on, those values and
    /// their props (`None`: no rule styles its box).
    styled: Option<(Arc<UiDocument>, usize, ResolvedNode, Props)>,
}

/// A `::column` marker's target key ([`column_target`]), its element's
/// key and the column's index, and the marker's instance key.
type ColumnTarget = (u64, (u64, usize), u64);

/// An instanced marker's styles by the states an instance holds, the base
/// (no state) first.
pub type Variants = Vec<(BTreeSet<InteractionState>, Arc<Marker>)>;

impl std::fmt::Debug for Marker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Marker({:?}, {:x})", self.text, self.key)
    }
}

/// The controls of one scroll container, as it was last built.
#[derive(Clone, Debug, Default)]
pub struct Group {
    /// Each marker's key (its state's) and its target's render key, in
    /// tree order.
    pub markers: Vec<(u64, u64)>,
    /// The keys of its buttons up, down, left and right (0: not generated).
    pub buttons: [u64; 4],
}

/// A scroll control pressed while the frame was built.
#[derive(Clone, Copy, Debug)]
pub enum Press {
    /// A marker: its target, by render key, scrolls to the container's start.
    Marker(u64),
    /// A button, by [`BUTTONS`] index: a page in its direction.
    Page(usize),
}

/// Each button's direction, as [`BUTTONS`] orders them.
const DIRECTIONS: [Vec2; 4] = [
    Vec2::new(0.0, -1.0),
    Vec2::new(0.0, 1.0),
    Vec2::new(-1.0, 0.0),
    Vec2::new(1.0, 0.0),
];

/// How much of the scrollport a button's page is (§3.2: "usually about
/// 85%", as PgUp and PgDn).
const PAGE: f32 = 0.85;

/// Positions closer than this (CSS px) are one position: a laid-out box
/// stands on the device pixel grid.
const SAME: f32 = 0.5;

impl Ui {
    /// The sets of states the instances of element `owner`'s instanced
    /// pseudo-element hold, each once, the empty set left out (its base
    /// cascade is that): its cascade resolves once for each.
    pub(crate) fn instance_states_of(&self, owner: u64) -> Vec<BTreeSet<InteractionState>> {
        let mut sets: Vec<BTreeSet<InteractionState>> = Vec::new();
        for (o, states) in self.docs.state().instances.values() {
            if *o == owner && !states.is_empty() && !sets.contains(states) {
                sets.push(states.clone());
            }
        }
        sets
    }

    /// Notes what element `key` (`r`, at `path`) gives the scroll controls
    /// as it is built: its `::scroll-marker`, the `#id` its `href` names,
    /// and whether it is a `scroll-target-group`.
    pub(crate) fn note_scroll_controls(
        &mut self,
        b: &Build<'_>,
        n: &DocNode,
        r: &Resolved,
        path: &str,
        key: u64,
    ) {
        let g = r.generated.as_ref();
        // A column's marker as each set of states its instances hold styles
        // it, the base (none) first.
        let column = g.and_then(|g| g.column_marker.clone()).and_then(|m| {
            let base = self.marker(b, n, r, path, "column-scroll-marker", m.clone(), None)?;
            let mut variants = vec![(BTreeSet::new(), base)];
            for (states, values) in &r.instances {
                let v = self.marker(
                    b,
                    n,
                    r,
                    path,
                    "column-scroll-marker",
                    m.clone(),
                    Some(values),
                );
                variants.extend(v.map(|v| (states.clone(), v)));
            }
            Some(variants)
        });
        let marker = g
            .and_then(|g| g.scroll_marker.clone())
            .and_then(|m| self.marker(b, n, r, path, "scroll-marker", m, None));
        let href = r
            .str("href")
            .and_then(|h| h.strip_prefix('#'))
            .filter(|id| !id.is_empty())
            .map(str::to_string);
        let group = r
            .str("scrollTargetGroup")
            .is_some_and(|v| v.eq_ignore_ascii_case("auto"));
        let s = self.docs.state_mut();
        match marker {
            Some(m) => s.scroll_markers.insert(key, m),
            None => s.scroll_markers.remove(&key),
        };
        match column {
            Some(m) => s.column_markers.insert(key, m),
            None => s.column_markers.remove(&key),
        };
        match href {
            Some(id) => s.scroll_anchors.insert(key, id),
            None => s.scroll_anchors.remove(&key),
        };
        if group {
            s.target_groups.insert(key);
        } else {
            s.target_groups.remove(&key);
        }
    }

    /// Element `r`'s (`n`, at `path`) marker pseudo-element `which` with
    /// content `m`: its label where the element stands, its box's style
    /// (`instance`: an instance's values, [`Resolved::instances`]).
    #[allow(clippy::too_many_arguments)]
    fn marker(
        &mut self,
        b: &Build<'_>,
        n: &DocNode,
        r: &Resolved,
        path: &str,
        which: &str,
        m: crate::doc::generated::PseudoBox,
        instance: Option<&Arc<Resolved>>,
    ) -> Option<Arc<Marker>> {
        {
            let text = self.docs.pseudo_text(b, n, r, &m)?;
            let styled = self
                .style
                .pseudo_node(b, r, path, which)
                .map(|(at, mut node)| {
                    if let Some(values) = instance {
                        node.values = (**values).clone();
                    }
                    (at, node)
                });
            let values = styled.as_ref().map_or(r, |(_, s)| &s.values);
            let desc = values.label_desc(self.env.theme(), self.fonts(), values.own_flow().rtl);
            let key = styled.as_ref().map_or_else(
                || crate::id::key_of_path(&format!("{path}/::{which}")),
                |(_, s)| s.key,
            );
            let styled = styled.map(|(at, node)| {
                let mut built = node.probe.clone();
                let props = self.props_of(b, &b.doc.nodes[at], node.key, &node.values, &mut built);
                (b.doc.clone(), at, node, props)
            });
            Some(Arc::new(Marker {
                text,
                key,
                desc,
                styled,
            }))
        }
    }

    /// After element `r` (`n`, at `path`), built as node `own`, closed: if
    /// it is a scroll container, its `::scroll-button()`s and its
    /// `::scroll-marker-group`, built as its siblings in §3's order (the
    /// group before or after it, the buttons just before it). They are
    /// built again every frame: the markers are other elements'.
    pub(crate) fn build_scroll_controls(
        &mut self,
        b: &Build<'_>,
        n: &DocNode,
        r: &Resolved,
        path: &str,
        own: usize,
    ) {
        let node = &self.tree.nodes()[own];
        let key = node.key;
        let scrolls = node.el.scroll || node.el.scroll_x;
        let place = r
            .str("scrollMarkerGroup")
            .and_then(|v| crate::css::scroll_css::marker_group(v).ok())
            .flatten()
            .filter(|_| scrolls);
        let buttons = r
            .generated
            .as_ref()
            .map(|g| g.scroll_buttons.clone())
            .filter(|_| scrolls)
            .unwrap_or_default();
        if place.is_none() && buttons.iter().all(Option::is_none) {
            self.docs.state_mut().scroll_groups.remove(&key);
            return;
        }
        if let Some(owned) = self.tree.owning_mut().last_mut() {
            owned.moved = true;
        }
        let (theme, rtl) = (*self.env.theme(), r.own_flow().rtl);
        let mut group = Group::default();
        let mut pressed = Vec::new();
        // block-start, inline-start, inline-end, block-end.
        for k in [0, 2, 3, 1] {
            let Some(which) = &buttons[k] else { continue };
            let Some(text) = self.docs.pseudo_text(b, n, r, which) else {
                continue;
            };
            let name = BUTTONS[k];
            let styled = self.style.pseudo_node(b, r, path, name);
            let values = styled.as_ref().map_or(r, |(_, s)| &s.values);
            let desc = values.label_desc(&theme, self.fonts(), rtl);
            self.open(format!("::{name}~{key:x}"), control());
            let at = self.tree.parent();
            if let Some(s) = &styled {
                self.style_pseudo_box(b, at, s);
            }
            self.label("label", &text, desc);
            let resp = self.close();
            let state = styled.map_or_else(
                || crate::id::key_of_path(&format!("{path}/::{name}")),
                |(_, s)| s.key,
            );
            let disabled = self.docs.state().button_states.contains(&state);
            if resp.clicked && !disabled {
                pressed.push((key, Press::Page(k)));
            }
            group.buttons[k] = state;
            self.tree.place_beside(at, own, true);
        }
        if let Some(before) = place {
            let (markers, columns) = self.markers_in(own);
            let s = self.docs.state_mut();
            for (target, (owner, k), instance) in columns {
                s.column_targets.insert(target, (owner, k));
                s.instances
                    .entry(instance)
                    .or_insert_with(|| (owner, BTreeSet::new()));
            }
            self.open(format!("::scroll-marker-group~{key:x}"), Element::default());
            let at = self.tree.parent();
            if let Some(s) = self.style.pseudo_node(b, r, path, "scroll-marker-group") {
                self.style_pseudo_box(b, at, &s);
            }
            for (target, m) in markers {
                self.open(format!("::scroll-marker~{target:x}"), control());
                let i = self.tree.parent();
                if let Some((doc, node_at, node, props)) = &m.styled {
                    self.apply_pseudo_box(doc, i, *node_at, node, props.clone());
                }
                self.label("label", &m.text, m.desc);
                if self.close().clicked {
                    pressed.push((key, Press::Marker(target)));
                }
                group.markers.push((m.key, target));
            }
            self.close();
            // Before: ahead of the buttons, which stand just before it.
            let first = group.buttons.iter().filter(|&&k| k != 0).count();
            let anchor = if before {
                let siblings = &self.tree.nodes()[self.tree.nodes()[own].parent].built_children;
                siblings
                    .iter()
                    .position(|&c| c == own)
                    .and_then(|p| p.checked_sub(first))
                    .map_or(own, |p| siblings[p])
            } else {
                own
            };
            self.tree.place_beside(at, anchor, before);
        }
        let s = self.docs.state_mut();
        s.scroll_presses.extend(pressed);
        s.scroll_groups.insert(key, group);
    }

    /// The markers of the elements inside scroll container `own`, in tree
    /// order, by their target's key: those whose nearest scroll container
    /// it is (a scroll container inside it keeps its own). A multi-column
    /// element's `::column`s, the container's own among them, are its
    /// children (CSS Multicol 2 §4.8): their markers follow its own, one
    /// for each column it was last laid out with, each with its target's
    /// column ([`column_target`]).
    fn markers_in(&self, own: usize) -> (Vec<(u64, Arc<Marker>)>, Vec<ColumnTarget>) {
        let state = self.docs.state();
        let (nodes, kept) = (self.tree.nodes(), &state.scroll_markers);
        let mut out = Vec::new();
        let mut targets = Vec::new();
        let mut columns = |i: usize, out: &mut Vec<(u64, Arc<Marker>)>| {
            let key = nodes[i].key;
            let Some(variants) = state.column_markers.get(&key) else {
                return;
            };
            let base = &variants[0].1;
            // The node is being built: its columns are the last frame's.
            let laid = self.tree.previous_node(i).filter(|p| p.key == key);
            for k in 0..laid.map_or(0, |p| p.layout.columns.len()) {
                let target = column_target(key, k);
                // Each column's marker is an instance of its own: its key,
                // and the values for the states it holds.
                let instance = crate::id::key_of_path(&format!("{:x}/::column({k})", base.key));
                let held = state.instances.get(&instance).map(|(_, s)| s);
                let m = variants
                    .iter()
                    .find(|(s, _)| held.is_some_and(|h| h == s))
                    .map_or(base, |(_, m)| m);
                let mut styled = m.styled.clone();
                if let Some((_, _, node, _)) = &mut styled {
                    node.key = instance;
                }
                out.push((
                    target,
                    Arc::new(Marker {
                        key: instance,
                        styled,
                        ..(**m).clone()
                    }),
                ));
                targets.push((target, (key, k), instance));
            }
        };
        columns(own, &mut out);
        let mut stack: Vec<usize> = nodes[own].built_children.iter().rev().copied().collect();
        while let Some(i) = stack.pop() {
            let n = &nodes[i];
            if let Some(m) = kept.get(&n.key) {
                out.push((n.key, m.clone()));
            }
            if !(n.el.scroll || n.el.scroll_x) {
                columns(i, &mut out);
                stack.extend(
                    n.built_children
                        .iter()
                        .rev()
                        .filter(|&&c| nodes[c].parent == i),
                );
            }
        }
        (out, targets)
    }

    /// How far scroll container `c` scrolls from where it is (unclamped)
    /// to bring marker target `target` to its view's start: an element's
    /// box, or a `::column` ([`column_target`]); `None` when it is gone.
    fn target_start(&self, c: usize, target: u64) -> Option<Vec2> {
        use crate::interaction::scroll::{into_view_start, into_view_start_of};
        if let Some(&(owner, k)) = self.docs.state().column_targets.get(&target) {
            let n = &self.tree.nodes()[self.tree.index().find(owner)?];
            let margin = n.layout.used.multicol.map_or([0.0; 4], |m| m.snap.margin);
            return Some(into_view_start_of(
                &self.tree,
                c,
                *n.layout.columns.get(k)?,
                margin,
            ));
        }
        Some(into_view_start(
            &self.tree,
            c,
            self.tree.index().find(target)?,
        ))
    }

    /// Once the frame is laid out: the scrolls of the controls pressed as
    /// it was built, then each group's current marker and each button's
    /// `:disabled`, for the next frame. Whether an offset moved (the frame
    /// is laid out again).
    pub(crate) fn settle_scroll_controls(&mut self) -> bool {
        let mut moved = false;
        for (container, press) in std::mem::take(&mut self.docs.state_mut().scroll_presses) {
            let Some(c) = self.tree.index().find(container) else {
                continue;
            };
            let cn = &self.tree.nodes()[c];
            let range = Vec2::new(cn.max_scroll_x(), cn.max_scroll());
            let at = self.interaction.offset(container);
            let to = match press {
                Press::Marker(target) => {
                    let Some(d) = self.target_start(c, target) else {
                        continue;
                    };
                    let to = (at + d).clamp(Vec2::ZERO, range.max(Vec2::ZERO));
                    // Its target is the current one while the offset
                    // stays where it aimed (§3.1.8's current scroll target).
                    self.docs
                        .state_mut()
                        .scroll_pins
                        .insert(container, (target, to));
                    to
                }
                Press::Page(k) => {
                    self.eventual(container, range) + DIRECTIONS[k] * scrollport(cn) * PAGE
                }
            };
            self.scroll_to(container, to);
            moved |= self.interaction.offset(container) != at;
        }
        let fling = self.interaction.state().fling.map(|(k, _)| k);
        let pins = std::mem::take(&mut self.docs.state_mut().scroll_pins);
        self.docs.state_mut().scroll_pins = pins
            .into_iter()
            .filter(|(c, (_, aim))| {
                fling == Some(*c) || (self.interaction.offset(*c) - *aim).abs().max_element() < SAME
            })
            .collect();
        let mut markers = BTreeMap::new();
        let mut disabled = BTreeSet::new();
        let groups: Vec<(u64, Group)> = self
            .docs
            .state()
            .scroll_groups
            .iter()
            .map(|(k, g)| (*k, g.clone()))
            .collect();
        for (container, group) in groups {
            let Some(c) = self.tree.index().find(container) else {
                continue;
            };
            let cn = &self.tree.nodes()[c];
            let (at, range) = (
                self.interaction.offset(container),
                Vec2::new(cn.max_scroll_x(), cn.max_scroll()),
            );
            for (k, &key) in group.buttons.iter().enumerate() {
                let d = DIRECTIONS[k];
                let room = (d.x > 0.0 && at.x < range.x - SAME)
                    || (d.x < 0.0 && at.x > SAME)
                    || (d.y > 0.0 && at.y < range.y - SAME)
                    || (d.y < 0.0 && at.y > SAME);
                if key != 0 && !room {
                    disabled.insert(key);
                }
            }
            self.mark_current(container, c, &group.markers, &mut markers);
        }
        // `scroll-target-group`: its `#id` links, in tree order, mark the
        // elements they name in those elements' nearest scroll container.
        let groups: Vec<u64> = self.docs.state().target_groups.iter().copied().collect();
        for key in groups {
            let Some(g) = self.tree.index().find(key) else {
                continue;
            };
            let state = self.docs.state();
            let links: Vec<(u64, u64)> = crate::layout::preorder(self.tree.nodes(), g)
                .into_iter()
                .filter_map(|i| {
                    let link = self.tree.nodes()[i].key;
                    let id = state.scroll_anchors.get(&link)?;
                    let target = state
                        .memory
                        .retained
                        .elements
                        .iter()
                        .find(|(_, e)| e.id == *id || e.document.nodes[e.index].id == *id)
                        .map_or(0, |(node, _)| node.key);
                    Some((link, target))
                })
                .collect();
            let scroller = links.iter().find_map(|&(_, t)| {
                let nodes = self.tree.nodes();
                let mut at = nodes.get(self.tree.index().find(t)?)?.parent;
                while at != NONE && !nodes[at].scrolls() {
                    at = nodes[at].parent;
                }
                (at != NONE).then_some(at)
            });
            if let Some(c) = scroller {
                let container = self.tree.nodes()[c].key;
                self.mark_current(container, c, &links, &mut markers);
            }
        }
        let s = self.docs.state_mut();
        let (old_markers, old_disabled) = (
            std::mem::replace(&mut s.marker_states, markers.clone()),
            std::mem::replace(&mut s.button_states, disabled.clone()),
        );
        for (k, state) in &old_markers {
            if markers.get(k) != Some(state) {
                self.set_selector_state(*k, *state, false);
            }
        }
        for (k, state) in &markers {
            if old_markers.get(k) != Some(state) {
                self.set_selector_state(*k, *state, true);
            }
        }
        for k in old_disabled.difference(&disabled) {
            self.set_selector_state(*k, Disabled, false);
        }
        for k in disabled.difference(&old_disabled) {
            self.set_selector_state(*k, Disabled, true);
        }
        moved
    }

    /// The states of a group's `markers` (`(state key, target key)` in
    /// tree order) in scroll container `container` (node `c`): the
    /// current one's, and before and after it.
    fn mark_current(
        &self,
        container: u64,
        c: usize,
        markers: &[(u64, u64)],
        out: &mut BTreeMap<u64, InteractionState>,
    ) {
        let targets: Vec<(usize, Vec2)> = markers
            .iter()
            .enumerate()
            .filter_map(|(k, &(_, t))| Some((k, self.target_start(c, t)?)))
            .collect();
        let Some(current) = self.current_target(container, c, markers, &targets) else {
            return;
        };
        for (k, &(key, _)) in markers.iter().enumerate() {
            let state = match k.cmp(&current) {
                std::cmp::Ordering::Less => TargetBefore,
                std::cmp::Ordering::Equal => TargetCurrent,
                std::cmp::Ordering::Greater => TargetAfter,
            };
            out.insert(key, state);
        }
    }

    /// CSS Overflow 5 §3.1.8's example algorithm: of `targets` (a marker's
    /// index and how far its target is from the view's start, in `markers`
    /// order) in scroll container `container` (node `c`), the marker whose
    /// target is current at its eventual offset; the target it was last
    /// scrolled to by a marker while it stays there.
    fn current_target(
        &self,
        container: u64,
        c: usize,
        markers: &[(u64, u64)],
        targets: &[(usize, Vec2)],
    ) -> Option<usize> {
        let nodes = self.tree.nodes();
        if let Some(&(pinned, _)) = self.docs.state().scroll_pins.get(&container)
            && let Some(&(k, _)) = targets.iter().find(|(k, _)| markers[*k].1 == pinned)
        {
            return Some(k);
        }
        let cn = &nodes[c];
        let (port, range) = (
            scrollport(cn),
            Vec2::new(cn.max_scroll_x(), cn.max_scroll()),
        );
        let position = self.eventual(container, range);
        let at = self.interaction.offset(container);
        let mut set: Vec<(usize, Vec2)> = targets.to_vec();
        // The block axis (horizontal-tb), then the inline axis.
        for axis in [1, 0] {
            let mut pos: Vec<f32> = set.iter().map(|&(_, d)| (at + d)[axis]).collect();
            redistribute(&mut pos, port[axis], range[axis]);
            let chosen = select(&pos, position[axis], port[axis])?;
            set = set
                .into_iter()
                .zip(pos)
                .filter(|(_, p)| (p - chosen).abs() < SAME)
                .map(|(t, _)| t)
                .collect();
        }
        set.first().map(|&(k, _)| k)
    }

    /// Where scroll container `container` comes to rest: its offset, or
    /// where its glide (a smooth scroll, a fling) lands.
    fn eventual(&self, container: u64, range: Vec2) -> Vec2 {
        let at = self.interaction.offset(container);
        match self.interaction.state().fling {
            Some((k, v)) if k == container => {
                (at + v / self.fling_keep()).clamp(Vec2::ZERO, range.max(Vec2::ZERO))
            }
            _ => at,
        }
    }
}

impl crate::doc::build::state::DocState {
    /// What a snapshot keeps of the scroll controls: the states last set
    /// and the current targets markers scrolled to; `None` when none.
    pub(crate) fn save_scroll_controls(&self) -> Option<Value> {
        if self.marker_states.is_empty()
            && self.button_states.is_empty()
            && self.scroll_pins.is_empty()
        {
            return None;
        }
        let state = |s: &InteractionState| match s {
            TargetBefore => 0,
            TargetCurrent => 1,
            _ => 2,
        };
        Some(json!({
            "markers": self.marker_states.iter().map(|(k, s)| json!([k.to_string(), state(s)])).collect::<Vec<_>>(),
            "disabled": self.button_states.iter().map(u64::to_string).collect::<Vec<_>>(),
            "pins": self.scroll_pins.iter().map(|(c, (t, at))| json!([c.to_string(), t.to_string(), at.x, at.y])).collect::<Vec<_>>(),
        }))
    }

    /// Takes the scroll controls a snapshot kept (`None`: none), and sets
    /// their states again on the elements built from it.
    pub(crate) fn restore_scroll_controls(&mut self, v: Option<&Value>) -> Result<(), String> {
        let bad = || "malformed scroll controls".to_string();
        let key = |v: &Value| {
            v.as_str()
                .and_then(|s| s.parse::<u64>().ok())
                .ok_or_else(bad)
        };
        let list = |name: &str| {
            v.and_then(|v| v[name].as_array().cloned())
                .unwrap_or_default()
        };
        self.marker_states.clear();
        self.button_states.clear();
        self.scroll_pins.clear();
        for m in list("markers") {
            let state = match m[1].as_u64() {
                Some(0) => TargetBefore,
                Some(1) => TargetCurrent,
                Some(2) => TargetAfter,
                _ => return Err(bad()),
            };
            let k = key(&m[0])?;
            self.marker_states.insert(k, state);
            self.selector_states.push((k, state, true));
        }
        for d in list("disabled") {
            let k = key(&d)?;
            self.button_states.insert(k);
            self.selector_states.push((k, Disabled, true));
        }
        for p in list("pins") {
            let at = Vec2::new(
                p[2].as_f64().ok_or_else(bad)? as f32,
                p[3].as_f64().ok_or_else(bad)? as f32,
            );
            self.scroll_pins.insert(key(&p[0])?, (key(&p[1])?, at));
        }
        Ok(())
    }
}

/// A focusable box a press activates: a marker, a button.
fn control() -> Element {
    Element {
        interactive: true,
        focusable: true,
        ..Element::default()
    }
}

/// Scroll container `c`'s scrollport: its box inside its padding.
fn scrollport(c: &crate::layout::Node) -> Vec2 {
    let pad = c.layout.used.padding;
    Vec2::new(
        (c.placed.rect.width - pad.horizontal()).max(0.0),
        (c.placed.rect.height - pad.vertical()).max(0.0),
    )
}

/// §3.1.8 step 5: target positions the offset cannot reach spread over
/// the first and the last `min(port / 8, range / 2)` of the scroll range,
/// so each can be current.
fn redistribute(pos: &mut [f32], port: f32, range: f32) {
    if range <= 0.0 {
        return;
    }
    let d = (port / 8.0).min(range / 2.0);
    let before: Vec<bool> = pos.iter().map(|&p| p < d).collect();
    let after: Vec<bool> = pos.iter().map(|&p| p > range - d).collect();
    let min = pos
        .iter()
        .zip(&before)
        .filter(|(_, b)| **b)
        .map(|(p, _)| *p)
        .fold(f32::INFINITY, f32::min);
    let max = pos
        .iter()
        .zip(&after)
        .filter(|(_, a)| **a)
        .map(|(p, _)| *p)
        .fold(f32::NEG_INFINITY, f32::max);
    for (k, p) in pos.iter_mut().enumerate() {
        if before[k] {
            *p = (*p - min) / (d - min) * d;
        } else if after[k] {
            *p = (*p - (range - d)) / (max - (range - d)) * d + (range - d);
        }
    }
}

/// §3.1.8 steps 6 and 7: the largest target position at or before
/// `position`, or one in the near half of the scrollport whose next
/// smaller position is in the far half before it; else the first.
fn select(pos: &[f32], position: f32, port: f32) -> Option<f32> {
    let mut best: Option<f32> = None;
    for &p in pos {
        let smaller = pos
            .iter()
            .copied()
            .filter(|&q| q < p - SAME)
            .fold(None, |m: Option<f32>, q| Some(m.map_or(q, |m| m.max(q))));
        let shown = p <= position + SAME
            || smaller.is_some_and(|q| q < position - port / 2.0) && p < position + port / 2.0;
        if shown && best.is_none_or(|b| p > b) {
            best = Some(p);
        }
    }
    best.or_else(|| pos.first().copied())
}

/// The key a group names column `k` of multi-column element `owner` by,
/// as a `::column::scroll-marker`'s target.
fn column_target(owner: u64, k: usize) -> u64 {
    crate::id::key_of_path(&format!("{owner:x}/::column({k})"))
}
