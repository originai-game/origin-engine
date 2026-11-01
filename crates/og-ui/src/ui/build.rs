use super::*;

impl Ui {
    /// A further touch (pointers 1 to 4, `og.Pointers`) on the touch control
    /// `key`: the first pointer held, or released this tick, that started on
    /// its last-frame rectangle (`by_start`, a stick that keeps the finger
    /// that grabbed it) or is over it now (buttons, d-pad arms). Its
    /// position, and whether it is still held.
    pub(crate) fn touch(&self, key: u64, by_start: bool) -> Option<(Vec2, bool)> {
        let rect = self.hit(key)?.rect;
        let p = &self.input.pointers;
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
        self.state.hits.iter().find(|h| h.key == key)
    }

    pub(crate) fn parent(&self) -> usize {
        self.stack.last().copied().unwrap_or(0)
    }

    /// Adds a node under the open element and returns its index.
    pub(crate) fn push(&mut self, kind: &'static str, id: Id, el: Element) -> usize {
        if !self.building {
            self.warnings
                .push("an element was added outside begin/end; call ui.begin() first".into());
            self.begin(InputFrame::default(), self.viewport);
        }
        let parent = self.parent();
        let anonymous = id.is_empty();
        let mut local = if anonymous {
            let n = self.nodes[parent]
                .children
                .iter()
                .filter(|&&c| self.kinds[c] == (kind, true))
                .count();
            format!("{kind}#{n}")
        } else {
            let name = id.as_str();
            if name.contains('/') {
                self.warnings.push(format!(
                    "id '{name}' contains '/', the path separator; it is used as '{}'",
                    name.replace('/', "_")
                ));
                name.replace('/', "_")
            } else {
                name.to_owned()
            }
        };
        if self.nodes[parent]
            .children
            .iter()
            .any(|&c| self.nodes[c].id == local)
        {
            let mut n = 1;
            while self.nodes[parent]
                .children
                .iter()
                .any(|&c| self.nodes[c].id == format!("{local}~{n}"))
            {
                n += 1;
            }
            let renamed = format!("{local}~{n}");
            self.warnings.push(format!(
                "duplicate id '{local}' under '{}'; this one is '{renamed}': give siblings distinct ids (ui_idi(name, index))",
                self.path_of(parent)
            ));
            local = renamed;
        }
        let key = child_key(self.nodes[parent].key, &local);
        let index = self.nodes.len();
        self.nodes.push(Node::new(key, local, parent, el));
        self.kinds.push((kind, anonymous));
        self.nodes[parent].children.push(index);
        index
    }

    /// The full path of the element built just before (a container just
    /// closed, or a widget), as `bounds`, `gesture` and the probe name it;
    /// empty when there is none.
    pub fn last_path(&self) -> String {
        if self.last_closed >= self.nodes.len() || self.last_closed == 0 {
            return String::new();
        }
        self.path_of(self.last_closed)
    }

    pub(crate) fn path_of(&self, mut i: usize) -> String {
        let mut parts = Vec::new();
        while i != NONE && i != 0 {
            parts.push(self.nodes[i].id.clone());
            i = self.nodes[i].parent;
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
        match self.stack.last() {
            Some(&i) if i != 0 => self.probe_path(i),
            _ => String::new(),
        }
    }

    /// The path the probe lists an element by: [`Ui::path_of`], and for a
    /// layer placed against another element (a tooltip) that element's path
    /// before its own id, as its `parent` is that element's id.
    pub(crate) fn probe_path(&self, i: usize) -> String {
        let n = &self.nodes[i];
        if n.parent == NONE && n.anchor != NONE && n.anchor < self.nodes.len() {
            return format!("{}/{}", self.probe_path(n.anchor), n.id);
        }
        self.path_of(i)
    }

    pub(crate) fn respond(&self, key: u64) -> Response {
        let s = &self.state;
        let hovered = s.hot == key && (s.active == 0 || s.active == key);
        let by_pad = (key != 0)
            .then(|| self.nav.pad_activate.iter().position(|&k| k == key))
            .flatten();
        let pad = match by_pad {
            Some(k) => Some(k as u32 + 1),
            None if self.nav.activate == key && key != 0 && self.nav.pad0 => Some(0),
            None => None,
        };
        let mut focused_by = u32::from(s.focus == key && key != 0);
        for (k, &f) in s.pad_focus.iter().enumerate() {
            if f == key && key != 0 && (k as u8) + 1 < s.players {
                focused_by |= 2 << k;
            }
        }
        Response {
            hovered,
            pressed: s.active == key && self.input.down,
            clicked: (s.active == key && self.input.released && s.hot == key)
                || (self.nav.activate == key && key != 0)
                || by_pad.is_some(),
            focused: s.focus == key,
            pad,
            focused_by,
            ..Response::default()
        }
    }

    /// Marks node `i` as a keyboard/gamepad focus target and returns its
    /// response; the ring shows while navigation put focus on it.
    pub(crate) fn nav_target(&mut self, i: usize, adjust: bool) -> Response {
        let key = self.nodes[i].key;
        let r = self.respond(key);
        let n = &mut self.nodes[i];
        n.nav = true;
        n.adjust = adjust;
        n.ring = self.state.focus == key && self.state.focus_visible;
        n.pad_rings = (r.focused_by >> 1) as u8;
        r
    }

    /// Draws the element added last (a container just opened) with `draw`
    /// (`docs/ui.md` §5): its shape, fills, strokes, effects, opacity,
    /// blend, how it clips its children and whether its subtree draws
    /// offscreen through a material; over its background.
    pub fn set_draw(&mut self, draw: crate::paint::ElementDraw) {
        if let Some(last) = self.nodes.len().checked_sub(1) {
            self.draws.insert(last, draw);
            self.dressable.remove(&last);
        }
    }

    /// [`Ui::set_draw`] as a component's default look of the part the
    /// element names (`Element::part`): where the style dresses that part,
    /// the dressing's fills, strokes, effects, material, blend and mask
    /// replace the draw's (each list the dressing sets), and its atlas art
    /// replaces the whole draw; the draw's shape stays, so a ring, a star
    /// or a segment keeps its geometry in every style (docs/ui.md §12.3).
    pub fn set_draw_dressable(&mut self, draw: crate::paint::ElementDraw) {
        if let Some(last) = self.nodes.len().checked_sub(1) {
            self.draws.insert(last, draw);
            self.dressable.insert(last);
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
        self.stack.push(i);
    }

    /// Closes the open container. For an `interactive` element, its response.
    pub fn close(&mut self) -> Response {
        if self.stack.len() <= 1 {
            self.warnings
                .push("close without open: every ui.open() takes one ui.close()".into());
            return Response::default();
        }
        let i = self.stack.pop().unwrap_or(0);
        self.last_closed = i;
        if self.nodes[i].el.interactive && self.nodes[i].el.focusable {
            let adjustable = self.nodes[i].el.adjustable;
            let mut r = self.nav_target(i, adjustable);
            if adjustable && r.focused {
                r.adjust = self.nav.adjust;
            }
            r
        } else if self.nodes[i].el.interactive {
            self.respond(self.nodes[i].key)
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
        let t = self.theme;
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
        self.stack.push(i);
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
                ..Element::default()
            },
        );
    }

    pub(crate) fn leaf(&mut self, kind: &'static str, id: Id, el: Element) -> usize {
        let i = self.push(kind, id, el);
        self.last_closed = i;
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
        }
    }
}

impl Ui {
    /// A tooltip for the element just built: shown below it (above when there
    /// is no room) once the pointer has rested on it for `motion.duration.slow`.
    /// Only interactive elements are hovered, so anchor it to a button or a control.
    pub fn tooltip(&mut self, text: &str) {
        let anchor = self.last_closed;
        if anchor == NONE {
            self.warnings.push(
                "tooltip with no element before it: call ui.tooltip() right after the control"
                    .into(),
            );
            return;
        }
        let delay =
            (self.theme.durations_ms.slow * self.ticks_per_second as f32 / 1000.0).ceil() as u32;
        if self.state.hot != self.nodes[anchor].key || self.state.hover_ticks < delay {
            return;
        }
        let t = self.theme;
        let key = child_key(self.nodes[anchor].key, "tooltip");
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
        let mut content = self.text_content(text, t.type_ramp.meta, t.palette.ink);
        content.wrap = false;
        node.content = Content::Text(content);
        node.widget = Widget::Tooltip;
        node.probe_text = text.to_owned();
        node.anchor = anchor;
        node.float = Float::Tooltip;
        let index = self.nodes.len();
        self.nodes.push(node);
        self.kinds.push(("tooltip", false));
        self.floating.push(index);
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
        if !self.building {
            self.warnings
                .push("an element was added outside begin/end; call ui.begin() first".into());
            self.begin(InputFrame::default(), self.viewport);
        }
        let mut local = id.as_str().replace('/', "_");
        if local.is_empty() {
            local = format!("{kind}#{}", self.floating.len());
        }
        let taken = |s: &Ui, l: &str| s.floating.iter().any(|&f| s.nodes[f].id == l);
        if taken(self, &local) {
            let mut n = 1;
            while taken(self, &format!("{local}~{n}")) {
                n += 1;
            }
            let renamed = format!("{local}~{n}");
            self.warnings.push(format!(
                "duplicate layer id '{local}'; this one is '{renamed}': give each anchored group, dialog and menu its own id"
            ));
            local = renamed;
        }
        let key = child_key(self.nodes[0].key, &local);
        let mut node = Node::new(key, local, NONE, el);
        node.float = float;
        node.anchor = anchor;
        let index = self.nodes.len();
        self.nodes.push(node);
        self.kinds.push((kind, false));
        self.floating.push(index);
        self.stack.push(index);
        index
    }
}
