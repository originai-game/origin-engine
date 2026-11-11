use super::*;

/// What one retained element wrote and used as it was emitted.
#[derive(Clone, Debug, Default)]
pub(crate) struct Owned {
    /// Its own node and its widget's internal nodes, in write order.
    pub slots: Vec<usize>,
    /// The widget memories it read or wrote.
    pub memories: Vec<u64>,
    /// A memory it wrote changed: it is in motion and builds again next frame.
    pub moved: bool,
    /// Its style moves with the clock (SMIL): it resolves again next frame.
    pub restyle: bool,
}

impl Ui {
    pub(super) fn write_node(&mut self, node: Node, kind: (&'static str, bool)) -> usize {
        let identity = (node.key, node.parent);
        let i = if let Some(&i) = self.tree.render_slots.get(&identity) {
            let previous = std::mem::replace(&mut self.tree.nodes[i], node);
            self.layout.cache.prev.entry(i).or_insert(previous);
            self.tree.prev_kinds.entry(i).or_insert(self.tree.kinds[i]);
            self.tree.kinds[i] = kind;
            i
        } else if let Some(i) = self.tree.free_slots.pop() {
            self.tree.nodes[i] = node;
            self.tree.kinds[i] = kind;
            self.tree.render_slots.insert(identity, i);
            i
        } else {
            let i = self.tree.nodes.len();
            self.tree.nodes.push(node);
            self.tree.kinds.push(kind);
            self.tree.slot_present.push(false);
            self.tree.persistent.push(false);
            self.tree.render_slots.insert(identity, i);
            i
        };
        self.tree.slot_present[i] = true;
        self.tree.emitted.push(i);
        // A rewritten node sets its draw and layer again as it builds.
        self.tree.draws.remove(&i);
        self.tree.dressable.remove(&i);
        if self.tree.persistent[i] {
            self.tree.floating.retain(|&f| f != i);
        }
        self.tree.persistent[i] = match self.tree.owning.last_mut() {
            Some(owned) => {
                owned.slots.push(i);
                true
            }
            None => false,
        };
        i
    }

    /// Vacates slot `i` after the frame that last drew it: its identity is
    /// free for a new node, its slot for reuse from the next frame on.
    fn vacate(&mut self, i: usize) {
        if self.tree.kinds[i].0 == "vacant" {
            return;
        }
        self.untree(i);
        let old = &self.tree.nodes[i];
        self.tree.render_slots.remove(&(old.key, old.parent));
        // Its parent derives its children without it.
        if old.parent != NONE {
            self.tree.invalidation.rederive.push(old.parent);
        }
        self.tree.invalidation.forced.remove(&i);
        let mut vacant = Node::new(0, String::new(), NONE, Element::default());
        vacant.hidden = true;
        let old = std::mem::replace(&mut self.tree.nodes[i], vacant);
        self.layout.cache.prev.entry(i).or_insert(old);
        self.tree.prev_kinds.entry(i).or_insert(self.tree.kinds[i]);
        self.tree.kinds[i] = ("vacant", false);
        self.tree.draws.remove(&i);
        self.tree.dressable.remove(&i);
        self.tree.free_slots.push(i);
    }

    pub(super) fn retire_nodes(&mut self) {
        for k in 0..self.tree.retiring.len() {
            let i = self.tree.retiring[k];
            if self.tree.slot_present[i] || self.tree.persistent[i] {
                continue;
            }
            self.vacate(i);
        }
        self.tree.retiring.clear();
    }

    /// Retires slots a retained element owned and no longer writes: they
    /// leave this frame's tree and are vacated with the frame's other
    /// retired slots, never reused inside it.
    pub(crate) fn retire_owned(&mut self, slots: &[usize]) {
        for &i in slots {
            if !self.tree.persistent[i] {
                continue;
            }
            self.tree.persistent[i] = false;
            self.tree.slot_present[i] = false;
            self.tree.floating.retain(|&f| f != i);
            self.tree.retiring.push(i);
        }
    }

    /// The slot the node `key` under `parent` was last written to.
    pub(crate) fn render_slot(&self, key: u64, parent: usize) -> Option<usize> {
        self.tree.render_slots.get(&(key, parent)).copied()
    }

    /// Whether slot `i` still holds the retained node `key` under `parent`.
    pub(crate) fn holds(&self, i: usize, key: u64, parent: usize) -> bool {
        self.tree.persistent.get(i).copied().unwrap_or(false)
            && self.tree.nodes[i].key == key
            && self.tree.nodes[i].parent == parent
    }

    /// Puts a retained node that was not emitted again back under the open
    /// element, with its subtree as it stands.
    pub(crate) fn reattach(&mut self, i: usize) {
        let parent = self.content_parent();
        self.tree.nodes[parent].built_children.push(i);
    }

    /// Puts back a retained node the element being emitted owns and does
    /// not write again (a virtual list's row): it stays owned and keeps
    /// its subtree.
    pub(crate) fn keep_owned(&mut self, i: usize) {
        self.reattach(i);
        if let Some(owned) = self.tree.owning.last_mut() {
            owned.slots.push(i);
        }
    }

    pub(crate) fn previous_node(&self, i: usize) -> Option<&Node> {
        self.layout
            .cache
            .prev
            .get(&i)
            .or_else(|| self.tree.nodes.get(i))
    }

    pub(crate) fn previous_nodes(&self) -> impl Iterator<Item = &Node> {
        self.tree
            .retiring
            .iter()
            .filter_map(|&i| self.previous_node(i))
    }

    /// A further touch (pointers 1 to 4, `og.Pointers`) on the touch control
    /// `key`: the first pointer held, or released this tick, that started on
    /// its last-frame rectangle (`by_start`, a stick that keeps the finger
    /// that grabbed it) or is over it now (buttons, d-pad arms). Its
    /// position, and whether it is still held.
    pub(crate) fn touch(&self, key: u64, by_start: bool) -> Option<(Vec2, bool)> {
        let rect = self.hit(key)?.rect;
        let p = &self.interaction.input.pointers;
        (1..og_input::MAX_POINTERS as u32).find_map(|id| {
            let held = p.down(id);
            if !held && !p.is_released(id) {
                return None;
            }
            let at = if by_start {
                p.start(id)
            } else {
                p.position(id)
            };
            rect.contains(at.x, at.y).then(|| (p.position(id), held))
        })
    }

    pub(crate) fn hit(&self, key: u64) -> Option<&Hit> {
        if key == 0 {
            return None;
        }
        self.interaction.state.hits.iter().find(|h| h.key == key)
    }

    pub(crate) fn parent(&self) -> usize {
        self.tree.stack.last().copied().unwrap_or(0)
    }

    pub(crate) fn content_parent(&self) -> usize {
        let owner = self.parent();
        match self.tree.nodes[owner].content_slot {
            NONE => owner,
            slot => slot,
        }
    }

    pub(crate) fn write_part(
        &mut self,
        owner: usize,
        reference: crate::dress::PartRef,
        identity: (&str, usize),
        element: Element,
        content: Content,
    ) -> usize {
        let id = format!("::part({})#{}", identity.0, identity.1);
        let key = child_key(self.tree.nodes[owner].key, &id);
        let part = crate::layout::PartBox {
            owner,
            reference,
            element,
            intrinsic_props: Default::default(),
        };
        if let Some(i) = self.render_slot(key, owner)
            && self.tree.nodes[owner].built_children.contains(&i)
        {
            if !self.tree.emitted.contains(&i) {
                self.layout
                    .cache
                    .prev
                    .entry(i)
                    .or_insert_with(|| self.tree.nodes[i].clone());
            }
            self.tree.nodes[i].part_box = Some(part);
            self.tree.nodes[i].el = element;
            self.tree.nodes[i].content = content;
            self.tree.invalidation.rederive.push(owner);
            return i;
        }
        let mut node = Node::new(key, id, owner, element);
        node.part_box = Some(part);
        node.content = content;
        let i = self.write_node(node, ("part", false));
        self.tree.nodes[owner].built_children.push(i);
        i
    }

    pub(crate) fn open_part(&mut self, id: impl Into<Id>, mut element: Element) {
        let reference = element.part.take().expect("a widget part reference");
        let owner = self.tree.stack.last().copied().unwrap_or(0);
        let i = self.push("part", id.into(), element);
        self.tree.nodes[i].part_box = Some(crate::layout::PartBox {
            owner,
            reference,
            element,
            intrinsic_props: Default::default(),
        });
        self.tree.stack.push(i);
    }

    /// Adds a node under the open element and returns its index.
    pub(crate) fn push(&mut self, kind: &'static str, id: Id, el: Element) -> usize {
        if !self.tree.building {
            self.out
                .warnings
                .push("an element was added outside begin/end; call ui.begin() first".into());
            self.begin(InputFrame::default(), self.env.viewport);
        }
        let owner = self.parent();
        let parent = self.content_parent();
        let anonymous = id.is_empty();
        let parent_key = self.tree.nodes[owner].key;
        let mut local = if anonymous {
            let n = self
                .tree
                .anonymous
                .entry(child_key(parent_key, kind))
                .or_insert(0);
            *n += 1;
            format!("{kind}#{}", *n - 1)
        } else {
            let name = id.as_str();
            if name.contains('/') {
                self.out.warnings.push(format!(
                    "id '{name}' contains '/', the path separator; it is used as '{}'",
                    name.replace('/', "_")
                ));
                name.replace('/', "_")
            } else {
                name.to_owned()
            }
        };
        let mut key = child_key(parent_key, &local);
        if !self.tree.built.insert(key) {
            let mut n = 1;
            loop {
                key = child_key(parent_key, &format!("{local}~{n}"));
                if self.tree.built.insert(key) {
                    break;
                }
                n += 1;
            }
            let renamed = format!("{local}~{n}");
            self.out.warnings.push(format!(
                "duplicate id '{local}' under '{}'; this one is '{renamed}': give siblings distinct ids (ui_idi(name, index))",
                self.path_of(parent)
            ));
            local = renamed;
        }
        let in_place = self.tree.in_place == Some((key, parent));
        let index = self.write_node(Node::new(key, local, parent, el), (kind, anonymous));
        if in_place {
            self.tree.in_place = None;
        } else {
            self.tree.nodes[parent].built_children.push(index);
        }
        index
    }

    /// The full path of the element built just before (a container just
    /// closed, or a widget), as `bounds`, `gesture` and the probe name it;
    /// empty when there is none.
    pub fn last_path(&self) -> String {
        if self.tree.last_closed >= self.tree.nodes.len() || self.tree.last_closed == 0 {
            return String::new();
        }
        self.path_of(self.tree.last_closed)
    }

    pub(crate) fn path_of(&self, mut i: usize) -> String {
        let mut parts = Vec::new();
        let own = i;
        while i != NONE && i != 0 {
            if i == own
                || self.tree.nodes[i].part_box.is_none()
                || !self.tree.nodes[i].id.starts_with("::part(")
            {
                parts.push(self.tree.nodes[i].id.clone());
            }
            i = self.tree.nodes[i].parent;
        }
        if parts.is_empty() {
            return "root".into();
        }
        parts.reverse();
        parts.join("/")
    }

    /// The path of the innermost open container, as the probe, `bounds`,
    /// `visual_state` and `force_state` name it; empty at the top level.
    /// A component reads its root's path right after opening it, to name
    /// its parts in those calls.
    pub fn open_path(&self) -> String {
        match self.tree.stack.last() {
            Some(&i) if i != 0 => self.probe_path(i),
            _ => String::new(),
        }
    }

    /// The path the probe lists an element by: [`Ui::path_of`], and for a
    /// layer placed against another element (a tooltip) that element's path
    /// before its own id, as its `parent` is that element's id.
    pub(crate) fn probe_path(&self, i: usize) -> String {
        let n = &self.tree.nodes[i];
        if n.parent == NONE && n.anchor != NONE && n.anchor < self.tree.nodes.len() {
            return format!("{}/{}", self.probe_path(n.anchor), n.id);
        }
        self.path_of(i)
    }

    /// The interaction state the probe or an agent forced on the element
    /// of render key `key` (`og.ui.forceState(path, "interaction", …)`):
    /// `hover`, `pressed`, `focused` or `disabled`.
    pub(crate) fn forced_interaction(&self, key: u64) -> Option<&str> {
        let forced = &self.interaction.state.motion.forced;
        if forced.is_empty() || key == 0 {
            return None;
        }
        forced
            .get(&crate::motion::state::group_key(key, "interaction"))
            .map(String::as_str)
    }

    pub(crate) fn respond(&self, key: u64) -> Response {
        let s = &self.interaction.state;
        // A forced state is drawn and matched as the real one is, and
        // activates nothing: it never clicks.
        let forced = self.forced_interaction(key);
        let hovered = (s.hot == key && (s.active == 0 || s.active == key))
            || matches!(forced, Some("hover" | "pressed"));
        let by_pad = (key != 0)
            .then(|| {
                self.interaction
                    .nav
                    .pad_activate
                    .iter()
                    .position(|&k| k == key)
            })
            .flatten();
        let pad = match by_pad {
            Some(k) => Some(k as u32 + 1),
            None if self.interaction.nav.activate == key
                && key != 0
                && self.interaction.nav.pad0 =>
            {
                Some(0)
            }
            None => None,
        };
        let mut focused_by = u32::from(s.focus == key && key != 0);
        for (k, &f) in s.pad_focus.iter().enumerate() {
            if f == key && key != 0 && (k as u8) + 1 < s.players {
                focused_by |= 2 << k;
            }
        }
        let pressed = (s.active == key && self.interaction.input.down) || forced == Some("pressed");
        let clicked = forced.is_none()
            && ((s.active == key
                && self.interaction.input.released
                && s.hot == key
                && !s.tap.taken)
                || (self.interaction.nav.activate == key && key != 0)
                || by_pad.is_some());
        // Read only on the frames it is pressed: the hits are a list.
        let player = (pressed || clicked)
            .then(|| self.hit(key).map(|h| h.player).filter(|&p| p != 0))
            .flatten()
            .map(|p| u32::from(p) - 1);
        Response {
            hovered,
            player,
            pressed,
            clicked,
            focused: s.focus == key || forced == Some("focused"),
            pad,
            focused_by,
            tapped: key != 0 && (self.interaction.nav.tapped == key || self.tapped_by_keys(key)),
            double_tapped: key != 0 && self.interaction.nav.double_tapped == key,
            long_pressed: key != 0 && self.interaction.nav.long_pressed == key,
            long_press: self.long_press_progress(key),
            ..Response::default()
        }
    }

    /// Marks node `i` as a keyboard/gamepad focus target and returns its
    /// response; the ring shows while navigation put focus on it.
    pub(crate) fn nav_target(&mut self, i: usize, adjust: bool) -> Response {
        let key = self.tree.nodes[i].key;
        let r = self.respond(key);
        let ring = (self.interaction.state.focus == key && self.interaction.state.focus_visible)
            || self.forced_interaction(key) == Some("focused");
        let n = &mut self.tree.nodes[i];
        n.nav = true;
        n.adjust = adjust;
        let own = n.el.own_ring;
        n.ring = !own && ring;
        n.pad_rings = if own { 0 } else { (r.focused_by >> 1) as u8 };
        Response { ring, ..r }
    }

    /// Draws the element added last (a container just opened) with `draw`
    /// (`docs/ui.md` §3.3): its shape, fills, strokes, effects, opacity,
    /// blend, how it clips its children and whether its subtree draws
    /// offscreen through a material; over its background.
    pub fn set_draw(&mut self, draw: crate::paint::ElementDraw) {
        if let Some(&last) = self.tree.emitted.last() {
            if self.tree.draws.contains_key(&last) {
                let path = self.path_of(last);
                self.out.warnings.push(format!(
                    "setDraw replaced the draw set on {path} this tick: setDraw draws the element added last, \
                     so call it right after the element's open or control, before its children"
                ));
            }
            self.tree.draws.insert(last, draw);
            self.tree.dressable.remove(&last);
        }
    }

    /// [`Ui::set_draw`] as a component's default look of the part the
    /// element names (`Element::part`): where the style dresses that part,
    /// the dressing's fills, strokes, effects, material, blend and mask
    /// replace the draw's (each list the dressing sets), and its atlas art
    /// replaces the whole draw; the draw's shape stays, so a ring, a star
    /// or a segment keeps its geometry in every style (docs/ui.md §3.2).
    pub fn set_draw_dressable(&mut self, draw: crate::paint::ElementDraw) {
        if let Some(&last) = self.tree.emitted.last() {
            self.tree.draws.insert(last, draw);
            self.tree.dressable.insert(last);
        }
    }

    /// Opens a container; children follow until [`Ui::close`].
    pub fn open(&mut self, id: impl Into<Id>, el: Element) {
        let kind = match (el.scroll || el.scroll_x, el.direction) {
            (true, _) => "scroll",
            (false, Direction::Row) => "row",
            (false, Direction::Column) => "column",
        };
        let i = self.push(kind, id.into(), el);
        self.tree.stack.push(i);
    }

    /// Closes the open container. For an `interactive` element, its response.
    pub fn close(&mut self) -> Response {
        if self.tree.stack.len() <= 1 {
            self.out
                .warnings
                .push("close without open: every ui.open() takes one ui.close()".into());
            return Response::default();
        }
        let i = self.tree.stack.pop().unwrap_or(0);
        self.tree.last_closed = i;
        if self.tree.nodes[i].el.interactive && self.tree.nodes[i].el.focusable {
            let adjustable = self.tree.nodes[i].el.adjustable;
            let claim = self.tree.nodes[i].el.arrows
                | if adjustable {
                    crate::ui::ARROW_LEFT | crate::ui::ARROW_RIGHT
                } else {
                    0
                };
            let mut r = self.nav_target(i, adjustable);
            self.tree.nodes[i].claim = claim;
            if r.focused && claim & (crate::ui::ARROW_LEFT | crate::ui::ARROW_RIGHT) != 0 {
                r.adjust = self.interaction.nav.adjust;
            }
            if r.focused && claim & (crate::ui::ARROW_UP | crate::ui::ARROW_DOWN) != 0 {
                r.adjust_y = self.interaction.nav.adjust_y;
            }
            r
        } else if self.tree.nodes[i].el.interactive {
            self.respond(self.tree.nodes[i].key)
        } else {
            Response::default()
        }
    }

    /// A container built by `f`.
    pub fn container(
        &mut self,
        id: impl Into<Id>,
        el: Element,
        f: impl FnOnce(&mut Ui),
    ) -> Response {
        self.open(id, el);
        f(self);
        self.close()
    }

    /// A row built by `f`.
    pub fn row(&mut self, id: impl Into<Id>, el: Element, f: impl FnOnce(&mut Ui)) {
        self.container(
            id,
            Element {
                direction: Direction::Row,
                ..el
            },
            f,
        );
    }

    /// A column built by `f`.
    pub fn column(&mut self, id: impl Into<Id>, el: Element, f: impl FnOnce(&mut Ui)) {
        self.container(
            id,
            Element {
                direction: Direction::Column,
                ..el
            },
            f,
        );
    }

    /// A card: `panel`, `radius.panel`, padded and spaced by the style.
    pub fn panel(&mut self, id: impl Into<Id>, desc: PanelDesc, f: impl FnOnce(&mut Ui)) {
        self.open_panel(id, desc);
        f(self);
        self.close();
    }

    /// Opens a card ([`Ui::panel`] without the closure); its children follow
    /// until [`Ui::close`]. For hosts whose build code cannot pass a closure
    /// (the TypeScript binding).
    pub fn open_panel(&mut self, id: impl Into<Id>, desc: PanelDesc) {
        let t = self.env.theme;
        let el = Element {
            direction: if desc.row {
                Direction::Row
            } else {
                Direction::Column
            },
            width: desc.width,
            height: desc.height,
            padding: desc.padding.unwrap_or(Insets::all(t.space.lg)),
            gap: desc.gap.unwrap_or(t.space.md),
            align: desc.align,
            cross: desc.cross,
            background: Some(desc.background.unwrap_or(t.palette.panel)),
            radius: t.radius.panel,
            frame: Some(crate::style_file::StylePart::Card),
            ..Element::default()
        };
        let id = id.into();
        let i = self.push("panel", id, el);
        self.tree.stack.push(i);
    }

    /// A vertically scrolling, clipping column built by `f`.
    pub fn scroll(&mut self, id: impl Into<Id>, desc: ScrollDesc, f: impl FnOnce(&mut Ui)) {
        self.open_scroll(id, desc);
        f(self);
        self.close();
    }

    /// Opens a scroll column ([`Ui::scroll`] without the closure); its
    /// children follow until [`Ui::close`].
    pub fn open_scroll(&mut self, id: impl Into<Id>, desc: ScrollDesc) {
        self.open(
            id,
            Element {
                direction: Direction::Column,
                width: desc.width,
                height: desc.height,
                padding: desc.padding,
                gap: desc.gap,
                scroll: desc.axes != ScrollAxes::Horizontal,
                scroll_x: desc.axes != ScrollAxes::Vertical,
                hide_bars: desc.hide_bars,
                fade: desc.fade.max(0.0),
                fade_into: desc.fade_into,
                ..Element::default()
            },
        );
    }

    pub(crate) fn leaf(&mut self, kind: &'static str, id: Id, el: Element) -> usize {
        let i = self.push(kind, id, el);
        self.tree.last_closed = i;
        i
    }

    pub(crate) fn text_content(&self, text: &str, step: TypeStep, color: Srgba) -> TextContent {
        TextContent {
            text: text.to_owned(),
            step,
            color,
            align: TextAlign::Start,
            wrap: true,
            font: FontRole::Body,
            ellipsis: false,
            max_lines: 0,
            emphasis: 0.0,
            fit: Vec::new(),
            wrap_style: Default::default(),
            cut: None,
            vertical: None,
        }
    }
}

impl Ui {
    /// A tooltip for the element just built: shown below it (above when there
    /// is no room) once the pointer has rested on it for `motion.duration.slow`.
    /// Only interactive elements are hovered, so anchor it to a button or a control.
    pub fn tooltip(&mut self, text: &str) {
        let anchor = self.tree.last_closed;
        if anchor == NONE {
            self.out.warnings.push(
                "tooltip with no element before it: call ui.tooltip() right after the control"
                    .into(),
            );
            return;
        }
        let delay = (self.env.theme.durations_ms.slow * self.env.ticks_per_second as f32 / 1000.0)
            .ceil() as u32;
        if self.interaction.state.hot != self.tree.nodes[anchor].key
            || self.interaction.state.hover_ticks < delay
        {
            return;
        }
        let t = self.env.theme;
        let key = child_key(self.tree.nodes[anchor].key, "tooltip");
        let mut node = Node::new(
            key,
            "tooltip".into(),
            NONE,
            Element {
                padding: Insets::xy(t.space.sm, t.space.xs),
                background: Some(t.palette.panel_raised),
                radius: t.radius.control,
                frame: Some(StylePart::Tooltip),
                ..Element::default()
            },
        );
        let p = t.palette;
        let ground = self
            .part_ground(&StylePart::Tooltip.part_ref(), p.panel_raised)
            .over(p.backdrop);
        let ink = if p.ink_inverse.contrast(ground) > p.ink.contrast(ground) {
            p.ink_inverse
        } else {
            p.ink
        };
        let mut content = self.text_content(text, t.type_ramp.meta, ink);
        content.wrap = false;
        node.content = Content::Text(content);
        node.widget = Widget::Tooltip;
        node.probe_text = text.to_owned();
        node.anchor = anchor;
        node.float = Float::Tooltip;
        let index = self.write_node(node, ("tooltip", false));
        self.tree.floating.push(index);
    }

    /// A top-layer element's place in the tree (a modal dialog's, a
    /// spotlight's): a zero-size leaf `id` under the open parent, named
    /// `<id>~place` where its key would be `layer`'s, the key of the layer
    /// that draws the element (a modal dialog under the root, a spotlight's
    /// callout keyed as the element itself). One key names one node: the
    /// trees' index, motion and interaction state find the layer by it.
    pub(crate) fn layer_place(&mut self, id: &Id, layer: u64) -> usize {
        let parent = self.tree.nodes[self.parent()].key;
        let id = if child_key(parent, id.as_str()) == layer {
            Id::from(format!("{}~place", id.as_str()))
        } else {
            id.clone()
        };
        let el = Element {
            width: Size::Fixed(0.0),
            height: Size::Fixed(0.0),
            ..Element::default()
        };
        self.leaf("dialog", id, el)
    }

    /// Opens a layer outside the tree (an anchored HUD group, a dialog's
    /// backdrop, a menu, the toast stack): it lays out and draws over the
    /// tree in its rank, and its children follow until [`Ui::close`]. Its
    /// path is its own id; `anchor` is the node it is placed against.
    pub(crate) fn open_layer(
        &mut self,
        kind: &'static str,
        id: Id,
        el: Element,
        float: Float,
        anchor: usize,
    ) -> usize {
        if !self.tree.building {
            self.out
                .warnings
                .push("an element was added outside begin/end; call ui.begin() first".into());
            self.begin(InputFrame::default(), self.env.viewport);
        }
        let mut local = id.as_str().replace('/', "_");
        if local.is_empty() {
            local = format!("{kind}#{}", self.tree.floating.len());
        }
        // Taken by a layer written or kept this frame; a retained layer of
        // last frame not yet written is this one's own slot, rewritten below.
        let taken = |s: &Ui, l: &str| {
            s.tree
                .floating
                .iter()
                .any(|&f| s.tree.nodes[f].id == l && s.tree.slot_present[f])
        };
        if taken(self, &local) {
            let mut n = 1;
            while taken(self, &format!("{local}~{n}")) {
                n += 1;
            }
            let renamed = format!("{local}~{n}");
            self.out.warnings.push(format!(
                "duplicate layer id '{local}'; this one is '{renamed}': give each anchored group, dialog and menu its own id"
            ));
            local = renamed;
        }
        let key = child_key(self.tree.nodes[0].key, &local);
        self.open_layer_keyed(kind, key, local, el, float, anchor)
    }

    /// Opens a layer whose node takes `key` itself rather than one derived
    /// from its id: a document's `<anchor>` or `<panel world>` keyed by its
    /// own identity, so its children's keys (folded from it with
    /// [`child_key`]) are the ones the retained document gives them.
    pub(crate) fn open_layer_keyed(
        &mut self,
        kind: &'static str,
        key: u64,
        local: String,
        el: Element,
        float: Float,
        anchor: usize,
    ) -> usize {
        let mut node = Node::new(key, local, NONE, el);
        node.float = float;
        node.anchor = anchor;
        let index = self.write_node(node, (kind, false));
        self.tree.floating.push(index);
        self.tree.stack.push(index);
        index
    }
}
