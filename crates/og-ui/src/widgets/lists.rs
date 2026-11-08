use super::*;
use crate::virtualize::Order;

impl Ui {
    // ── lists and grids ────────────────────────────────────────────────────

    /// The height of a scroll container last frame, or a guess.
    fn view_height(&self, key: u64, size: Option<Size>) -> f32 {
        // Last frame's whole height: what is visible of it would build too
        // few rows as it scrolls into view, and one that is out of view has
        // no hit (a screenful of rows was built for it: a grid below a
        // phone page's fold built ten rows).
        if let Some(h) = self.hit(key) {
            return h.full.height;
        }
        if let Some(r) = self.prev_rect(key)
            && r.height > 0.0
        {
            return r.height;
        }
        match size {
            Some(Size::Fixed(v)) => v,
            _ => self.viewport.height,
        }
    }

    /// Where element `key` was laid out last frame, whether or not any of
    /// it was on screen (one out of view has no hit).
    fn prev_rect(&self, key: u64) -> Option<crate::Rect> {
        if key == 0 {
            return None;
        }
        self.previous_child(key)
            .map(|i| self.nodes[i].rect)
            .or_else(|| self.previous_nodes().find(|n| n.key == key).map(|n| n.rect))
    }

    /// The node `key` laid out last frame under the open element, before
    /// it is written again: a retained list keeps its slot and its rows'.
    fn previous_child(&self, key: u64) -> Option<usize> {
        let parent = self.parent();
        self.nodes[parent]
            .built_children
            .iter()
            .copied()
            .find(|&c| self.nodes[c].key == key && self.nodes[c].parent == parent)
            .or_else(|| {
                let i = self.render_slot(key, parent)?;
                (self.nodes[i].key == key && self.kinds[i].0 != "vacant").then_some(i)
            })
    }

    /// Opens a virtualised list (a scroll column) and answers which rows to
    /// build: call [`Ui::list_item`] for each of `first..last`, then
    /// [`Ui::close`]. Fillers keep the scroll range of all `count` rows.
    pub fn open_list(&mut self, id: impl Into<Id>, desc: ListDesc) -> ListRange {
        self.open_list_inner(id.into(), desc, None)
    }

    /// Notify a keyed container of patched rows before opening it. Order
    /// changes arrive through open_keyed_list/grid's authoritative key slice.
    pub fn invalidate_keyed_items(&mut self, id: impl Into<Id>, keys: &[String]) {
        let key = self.key_for(id.into().as_str());
        if let Some(v) = self.state.virtual_lists.get_mut(&key) {
            v.invalidate(keys);
        }
    }

    /// Measured rows with persistent collection keys. `row_height` estimates
    /// unbuilt rows; keys, not `count`, determine their number and identity.
    pub fn open_keyed_list(
        &mut self,
        id: impl Into<Id>,
        keys: &[String],
        desc: ListDesc,
    ) -> ListRange {
        self.open_list_inner(id.into(), desc, Some(Order::Untracked(keys)))
    }

    pub(crate) fn open_list_inner(
        &mut self,
        id: Id,
        desc: ListDesc,
        keys: Option<Order<'_>>,
    ) -> ListRange {
        let row = if desc.row_height > 0.0 {
            desc.row_height
        } else {
            self.sizes().row
        };
        let key = self.key_for(id.as_str());
        let view = self.view_height(key, desc.height) - desc.padding.vertical();
        let off = self.state.scroll.get(&key).copied().unwrap_or(0.0);
        let stride = row + desc.gap;
        let count = desc.count;
        let first = ((off / stride).floor().max(0.0) as u32)
            .saturating_sub(1)
            .min(count);
        let last = ((((off + view) / stride).ceil().max(0.0)) as u32 + 1).min(count);
        let (first, last, top, rest) = if let Some(keys) = keys {
            let width = self
                .hit(key)
                .map(|h| h.full)
                .or_else(|| self.prev_rect(key))
                .map_or(self.viewport.width, |r| r.width);
            let (range, top, rest) =
                self.keyed_range(key, keys, 1, row, desc.gap, width, view, false);
            (range.first, range.last, top, rest)
        } else {
            self.state.virtual_lists.remove(&key);
            (
                first,
                last,
                first as f32 * stride - desc.gap,
                if count > last {
                    (count - last) as f32 * stride - desc.gap
                } else {
                    -1.0
                },
            )
        };
        self.open(
            id,
            Element {
                direction: Direction::Column,
                width: desc.width,
                height: desc.height.unwrap_or(Size::grow()),
                padding: desc.padding,
                gap: desc.gap,
                cross: Cross::Stretch,
                scroll: true,
                ..Element::default()
            },
        );
        let list = self.parent();
        self.nodes[list].aux = [row, 0.0, -1.0];
        if first > 0 {
            self.filler(top, Size::grow());
        }
        // The bottom filler goes in when the list closes.
        self.nodes[list].aux[2] = rest;
        ListRange { first, last }
    }

    #[allow(clippy::too_many_arguments)]
    fn keyed_range(
        &mut self,
        container: u64,
        keys: Order<'_>,
        columns: usize,
        estimate: f32,
        gap: f32,
        width: f32,
        view: f32,
        grid: bool,
    ) -> (ListRange, f32, f32) {
        let prefix = if grid { "Cell#" } else { "Row#" };
        let row = |n: &crate::layout::Node| {
            let key =
                n.id.strip_prefix(prefix)?
                    .replace("%23", "#")
                    .replace("%2F", "/")
                    .replace("%25", "%");
            Some((key, n.rect.height))
        };
        // Its rows as laid out last frame: the container's own children,
        // which it still holds before it is written again.
        let measurements: Vec<_> = match self.previous_child(container) {
            Some(list) => self.nodes[list]
                .ordered_children
                .iter()
                .filter_map(|&c| row(&self.nodes[c]))
                .collect(),
            None => self
                .previous_nodes()
                .filter_map(|n| {
                    let parent = self.previous_node(n.parent)?;
                    if parent.key != container {
                        return None;
                    }
                    row(n)
                })
                .collect(),
        };
        let old = self.state.scroll.get(&container).copied().unwrap_or(0.0);
        let v = self.state.virtual_lists.entry(container).or_default();
        v.grid = grid;
        let off = v.update(
            keys,
            &measurements,
            columns,
            estimate,
            gap.max(0.0),
            width,
            view.max(0.0),
            old,
        );
        self.state.scroll.insert(container, off);
        let range = v.range(off);
        let top = v.offset(range.first as usize) - gap;
        let rest = if (range.last as usize) < keys.keys().len() {
            v.total() - v.offset(range.last as usize)
        } else {
            -1.0
        };
        (range, top, rest)
    }

    fn filler(&mut self, h: f32, w: Size) {
        let i = self.leaf(
            "spacer",
            Id::default(),
            Element {
                width: w,
                height: Size::Fixed(h.max(0.0)),
                ..Element::default()
            },
        );
        self.nodes[i].widget = Widget::Spacer;
    }

    /// Closes a list or grid opened with [`Ui::open_list`] / [`Ui::open_grid`].
    pub fn close_list(&mut self) -> Response {
        let list = self.parent();
        let rest = self.nodes[list].aux[2];
        if rest >= 0.0 {
            let w = if self.nodes[list].el.wrap {
                Size::Percent(1.0)
            } else {
                Size::grow()
            };
            self.filler(rest, w);
        }
        self.close()
    }

    /// Opens row `index` of the open list (`Row#index`): a selectable,
    /// focusable row; its content follows until [`Ui::close`].
    pub fn list_item(&mut self, index: u32, selected: bool) -> Response {
        let t = self.theme;
        let list = self.parent();
        let h = self.nodes[list].aux[0];
        let keyed = self
            .state
            .virtual_lists
            .get(&self.nodes[list].key)
            .and_then(|v| v.keys.get(index as usize))
            .map(|k| crate::virtualize::item_id(k, false));
        let height = if keyed.is_some() {
            Size::fit()
        } else {
            Size::Fixed(h)
        };
        let i = self.push(
            "row",
            keyed.map_or_else(|| crate::ui_idi("Row", index), Id::from),
            Element {
                direction: Direction::Row,
                width: Size::grow(),
                height,
                padding: Insets::xy(t.space.md, 0.0),
                gap: t.space.sm,
                cross: Cross::Center,
                interactive: true,
                radius: t.radius.control,
                ..Element::default()
            },
        );
        self.stack.push(i);
        self.nodes[i].a11y.role = Some("option");
        self.nodes[i].a11y.selected = Some(selected);
        let mut r = self.nav_target(i, false);
        r.dropped = self.drag_and_drop(list, i, index);
        let p = &t.palette;
        // Clear at rest (the control's hue at no alpha, so a fade passes no
        // grey), `control` under hover and focus, `controlStrong` selected and
        // pressed, a step past it pressed while selected.
        let clear = p.control.fade(0.0);
        let strong = crate::ui::mix_srgba(p.selection, p.ink, 0.12);
        self.option_states(
            i,
            selected,
            [clear, p.selection],
            [p.control, p.selection],
            [p.control_strong, strong],
        );
        r
    }

    /// An option's (a list row's, a grid cell's) background through §7.3
    /// state groups, so the style's motion moves it: `selection` (`off`,
    /// `on`, from `selected`) and `interaction` (`normal`, `hover`,
    /// `pressed`, `focused` from input, focus drawn as hover). Each pair is
    /// (unselected, selected): at rest, under hover or focus, pressed.
    fn option_states(
        &mut self,
        i: usize,
        selected: bool,
        rest: [Srgba; 2],
        hover: [Srgba; 2],
        pressed: [Srgba; 2],
    ) {
        use crate::motion::{StateGroup, VisualState};
        let bg = |c: Srgba| -> crate::element::Values {
            let f = |v: u8| f32::from(v) / 255.0;
            vec![
                (crate::element::Prop::BackgroundR, f(c.rgb[0])),
                (crate::element::Prop::BackgroundG, f(c.rgb[1])),
                (crate::element::Prop::BackgroundB, f(c.rgb[2])),
                (crate::element::Prop::BackgroundA, c.alpha),
            ]
        };
        let state = |name: &str, c: Srgba| VisualState {
            name: name.to_owned(),
            values: bg(c),
            ..VisualState::default()
        };
        let k = usize::from(selected);
        let n = &mut self.nodes[i];
        n.el.background = Some(rest[k]);
        n.props.states = vec![
            StateGroup {
                name: "selection".into(),
                states: vec![state("off", rest[0]), state("on", rest[1])],
                transitions: Vec::new(),
            },
            StateGroup {
                name: "interaction".into(),
                states: vec![
                    state("normal", rest[k]),
                    state("hover", hover[k]),
                    state("pressed", pressed[k]),
                    state("focused", hover[k]),
                    state("disabled", rest[k]),
                ],
                transitions: Vec::new(),
            },
        ];
        n.props.state = vec![(
            "selection".into(),
            if selected { "on" } else { "off" }.into(),
        )];
    }

    /// Drag and drop between the items of one list or grid: a press on an
    /// item marks it the source (in the container's memory); a release over
    /// another item answers the source's index there.
    fn drag_and_drop(&mut self, container: usize, item: usize, index: u32) -> Option<u32> {
        let ckey = self.nodes[container].key;
        let key = self.nodes[item].key;
        let mut a = self.anim(ckey);
        let mut dropped = None;
        if self.input.pressed && self.state.active == key {
            a.h = u64::from(index) + 1;
        } else if self.input.released && a.h > 0 {
            if self.state.hot == key && self.state.active != key && self.state.active != 0 {
                dropped = Some((a.h - 1) as u32);
            }
        } else if !self.input.down && !self.input.released {
            a.h = 0;
        }
        self.set_anim(ckey, a);
        dropped
    }

    /// Whether a drag from another item of `container` is over `item` now.
    fn drop_target(&self, container: usize, item: usize) -> bool {
        let ckey = self.nodes[container].key;
        let key = self.nodes[item].key;
        let a = self.state.anims.get(&ckey).copied().unwrap_or_default();
        a.h > 0
            && self.input.down
            && self.state.hot == key
            && self.state.active != key
            && self.state.active != 0
    }

    /// The index of the list or grid item a held press started on (a drag
    /// in progress), for drawing what is dragged under the pointer.
    pub fn dragging(&mut self, id: impl Into<Id>) -> Option<u32> {
        let id = id.into();
        let key = self.key_for(id.as_str());
        let a = self.state.anims.get(&key).copied().unwrap_or_default();
        (self.input.down && a.h > 0 && self.state.active != 0).then(|| (a.h - 1) as u32)
    }

    /// A virtualised list built by `row(ui, index)` for the rows in view;
    /// answers the row clicked this tick.
    pub fn list(
        &mut self,
        id: impl Into<Id>,
        desc: ListDesc,
        selected: Option<u32>,
        mut row: impl FnMut(&mut Ui, u32),
    ) -> Option<u32> {
        let range = self.open_list(id, desc);
        let mut clicked = None;
        for k in range.first..range.last {
            if self.list_item(k, selected == Some(k)).clicked {
                clicked = Some(k);
            }
            row(self, k);
            self.close();
        }
        self.close_list();
        clicked
    }

    /// Opens a virtualised grid (a wrapping, scrolling row of equal cells):
    /// call [`Ui::grid_item`] for each of `first..last`, then [`Ui::close_list`].
    pub fn open_grid(&mut self, id: impl Into<Id>, desc: GridDesc) -> ListRange {
        self.open_grid_inner(id.into(), desc, None)
    }

    /// Measured grid cells, keyed independently of their position. Each row
    /// occupies its tallest cell; `cell_height` estimates unbuilt cells.
    pub fn open_keyed_grid(
        &mut self,
        id: impl Into<Id>,
        keys: &[String],
        desc: GridDesc,
    ) -> ListRange {
        self.open_grid_inner(id.into(), desc, Some(Order::Untracked(keys)))
    }

    pub(crate) fn open_grid_inner(
        &mut self,
        id: Id,
        desc: GridDesc,
        keys: Option<Order<'_>>,
    ) -> ListRange {
        let t = self.theme;
        let floor = self.min_target();
        let asked = if desc.cell_width > 0.0 {
            desc.cell_width
        } else {
            64.0
        }
        .max(floor);
        let ch = if desc.cell_height > 0.0 {
            desc.cell_height
        } else {
            asked
        }
        .max(floor);
        let gap = if desc.gap > 0.0 { desc.gap } else { t.space.sm };
        let key = self.key_for(id.as_str());
        // Its bar's gutter while its rows ran past it.
        let gutter = self.bar_gutter(key);
        // The width the cells had last frame (the grid's, inside its padding).
        // Last frame's whole box, on screen or not: out of view a grid
        // guessed the viewport's width, built a dozen columns of rows and
        // laid them out four across, three times its height.
        let known_w = self
            .hit(key)
            .map(|h| h.full.width)
            .or_else(|| self.prev_rect(key).map(|r| r.width))
            .map(|w| w - desc.padding.horizontal() - gutter);
        // `columns` across, exactly: a fit grid is that many cells of the
        // asked width; any other width is shared by them (wider cells, or
        // narrower down to the touch target), rounded down to 1/64 px so a
        // row never spills into the next. Else as many as fit.
        let (columns, cw) = if desc.columns > 0 {
            let c = desc.columns;
            let cw = match known_w {
                Some(w) if !matches!(desc.width, Size::Fit { .. }) => {
                    (((w + gap) / c as f32 - gap) * 64.0).floor() / 64.0
                }
                _ => asked,
            };
            (c, cw.max(floor))
        } else {
            let inner_w = known_w.unwrap_or(self.viewport.width - desc.padding.horizontal());
            (
                (((inner_w + gap) / (asked + gap)).floor() as u32).max(1),
                asked,
            )
        };
        let view = self.view_height(key, desc.height) - desc.padding.vertical();
        let off = self.state.scroll.get(&key).copied().unwrap_or(0.0);
        let stride = ch + gap;
        let rows = desc.count.div_ceil(columns);
        let first_row = ((off / stride).floor().max(0.0) as u32)
            .saturating_sub(1)
            .min(rows);
        let last_row = ((((off + view) / stride).ceil().max(0.0)) as u32 + 1).min(rows);
        let width = if desc.columns > 0 && matches!(desc.width, Size::Fit { .. }) {
            Size::Fixed(columns as f32 * (cw + gap) - gap + desc.padding.horizontal() + gutter)
        } else {
            desc.width
        };
        let (first, last, top, rest) = if let Some(keys) = keys {
            let (range, top, rest) =
                self.keyed_range(key, keys, columns as usize, ch, gap, cw, view, true);
            (range.first, range.last, top, rest)
        } else {
            self.state.virtual_lists.remove(&key);
            (
                (first_row * columns).min(desc.count),
                (last_row * columns).min(desc.count),
                first_row as f32 * stride - gap,
                if rows > last_row {
                    (rows - last_row) as f32 * stride - gap
                } else {
                    -1.0
                },
            )
        };
        self.open(
            id,
            Element {
                direction: Direction::Row,
                wrap: true,
                scroll: true,
                width,
                height: desc.height.unwrap_or(Size::grow()),
                padding: desc.padding,
                gap,
                ..Element::default()
            },
        );
        let grid = self.parent();
        self.nodes[grid].aux = [cw, ch, -1.0];
        if first > 0 {
            self.filler(top, Size::Percent(1.0));
        }
        self.nodes[grid].aux[2] = rest;
        ListRange { first, last }
    }

    /// Opens cell `index` of the open grid (`Cell#index`): a slot on `control`
    /// (a step above `controlStrong` and the style's rim when selected, a
    /// pack's `controlStrong` and its accent hairline: one mark, no second
    /// underline), focusable; its
    /// content follows until [`Ui::close`].
    pub fn grid_item(&mut self, index: u32, selected: bool) -> Response {
        let t = self.theme;
        let grid = self.parent();
        let [cw, ch, _] = self.nodes[grid].aux;
        let keyed = self
            .state
            .virtual_lists
            .get(&self.nodes[grid].key)
            .and_then(|v| v.keys.get(index as usize))
            .map(|k| crate::virtualize::item_id(k, true));
        let height = if keyed.is_some() {
            Size::fit()
        } else {
            Size::Fixed(ch)
        };
        let i = self.push(
            "cell",
            keyed.map_or_else(|| crate::ui_idi("Cell", index), Id::from),
            Element {
                direction: Direction::Column,
                width: Size::Fixed(cw),
                height,
                align: Align::Center,
                cross: Cross::Center,
                padding: Insets::all(t.space.xs),
                interactive: true,
                radius: t.radius.small,
                part: Some(
                    crate::dress::PartRef::new("Slot", "frame", crate::tokens::ElementClass::Slot)
                        .flag_if(selected && !t.art.legacy, "selected"),
                ),
                ..Element::default()
            },
        );
        self.stack.push(i);
        self.nodes[i].a11y.role = Some("option");
        self.nodes[i].a11y.selected = Some(selected);
        let mut r = self.nav_target(i, false);
        r.dropped = self.drag_and_drop(grid, i, index);
        let p = &t.palette;
        let amounts = self.state_amounts(i, &r);
        let hover = p.panel_hover.fade(0.6).over(p.control.over(p.panel));
        // A selected tile is lifted a step above a raised one: told apart
        // by tone and lift, not only by its rim (docs/ui.md §6.3); a
        // pack's is one step up, on `controlStrong`, with the accent
        // hairline (dress::pack_grammar, the grammar of chips and tabs).
        let lifted = |k: f32| crate::ui::mix_srgba(p.selection, p.ink, k);
        if t.art.legacy {
            // A version 1 style's grammar, as its game shipped: the selected
            // cell on `controlStrong` with a `primary` underline.
            self.option_states(
                i,
                selected,
                [p.control, p.control_strong],
                [hover, p.control_strong],
                [p.control_strong, p.control_strong],
            );
            if selected {
                let w = t.space.xxs;
                self.nodes[i].shapes.push(Shape::DressedRect {
                    part: crate::dress::PartRef::of("Slot", "indicator", None).flag("selected"),
                    frame: Frame {
                        fw: 1.0,
                        h: w,
                        fy: 1.0,
                        y: -w - t.space.xs,
                        x: t.space.md,
                        w: -2.0 * t.space.md,
                        ..Frame::default()
                    },
                    radius: w,
                    color: p.primary,
                });
            }
        } else {
            let step = if crate::dress::pack_grammar(&t) {
                0.0
            } else {
                0.12
            };
            self.option_states(
                i,
                selected,
                [p.control, lifted(step)],
                [hover, lifted(step + 0.06)],
                [p.control_strong, p.selection],
            );
        }
        if let Some(part) = self.nodes[i].el.part.as_mut() {
            *part = part
                .flag_if(r.pressed, "pressed")
                .flag_if(r.hovered && !r.pressed, "hover")
                .flag_if(r.focused, "focused");
            part.hover = amounts.0;
            part.press = amounts.1;
        }
        if r.clicked {
            self.feedback(crate::tokens::SoundEvent::Select);
        }
        if self.drop_target(grid, i) {
            self.nodes[i].ring = true;
        }
        r
    }

    /// A virtualised grid built by `cell(ui, index)`; answers the cell clicked.
    pub fn grid(
        &mut self,
        id: impl Into<Id>,
        desc: GridDesc,
        selected: Option<u32>,
        mut cell: impl FnMut(&mut Ui, u32),
    ) -> Option<u32> {
        let range = self.open_grid(id, desc);
        let mut clicked = None;
        for k in range.first..range.last {
            if self.grid_item(k, selected == Some(k)).clicked {
                clicked = Some(k);
            }
            cell(self, k);
            self.close();
        }
        self.close_list();
        clicked
    }
}
