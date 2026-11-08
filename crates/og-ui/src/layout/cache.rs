use super::*;

/// The fonts layout measures with, by role.
pub(crate) trait Metrics {
    fn metrics_for(&self, role: FontRole, text: &str) -> &FontMetrics;
    /// The metrics of the face [`crate::Fonts::for_styled`] draws `text`
    /// in at `weight` (0: the role's own) and `italic`.
    fn metrics_styled(
        &self,
        role: FontRole,
        weight: u16,
        italic: bool,
        text: &str,
    ) -> &FontMetrics {
        let _ = (weight, italic);
        self.metrics_for(role, text)
    }
}

/// What layout measures text with: the fonts, and the measurements kept
/// from the frames before ([`TextCache`]).
pub(crate) struct Measure<'a> {
    pub fonts: &'a dyn Metrics,
    pub texts: &'a mut TextCache,
}

/// Everything a text's measurement depends on besides its string and the
/// fonts: the font role, the type size and tracking, wrapping, the
/// ellipsis and its line limit, and (for its lines) the width it breaks at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TextKey {
    font: FontRole,
    size: u32,
    tracking: u32,
    cjk_tracking: u32,
    features: u8,
    weight: u16,
    wrap: bool,
    ellipsis: bool,
    max_lines: u32,
    /// The width the lines break or cut at (`None`: only at `\n`).
    width: Option<u32>,
}

impl TextKey {
    fn of(t: &TextContent, width: Option<f32>) -> TextKey {
        TextKey {
            font: t.font,
            size: t.step.size.to_bits(),
            tracking: t.step.tracking_em.to_bits(),
            cjk_tracking: t.step.cjk_tracking_em.to_bits(),
            features: t.features().bits(),
            weight: t.step.weight,
            wrap: t.wrap,
            ellipsis: t.ellipsis,
            max_lines: t.max_lines,
            width: width.map(f32::to_bits),
        }
    }
}

/// One kept measurement and the frame that last used it.
#[derive(Clone, Debug)]
struct Kept<V> {
    key: TextKey,
    value: V,
    frame: u64,
}

/// Text measurements kept across frames while a text's string, font, size
/// and width are unchanged: a HUD lays out the same texts every tick, and
/// measuring each again (its widest line and longest word, its line breaks
/// and ellipsis) was about a third of `og.ui.end`. What a frame did not use
/// goes at its [`TextCache::sweep`]; new fonts clear it
/// ([`crate::Ui::set_fonts`]), the only other input a measurement has.
#[derive(Clone, Debug, Default)]
pub(crate) struct TextCache {
    /// A text's preferred and minimum width ([`size_w`]).
    widths: std::collections::HashMap<String, Vec<Kept<(f32, f32)>>>,
    /// A text's lines at a width ([`distribute_w`]).
    lines: std::collections::HashMap<String, Vec<Kept<Vec<Line>>>>,
    frame: u64,
    /// Measurements this frame made (widths and lines each count one).
    made: u32,
    /// What the last swept frame made ([`crate::Ui::text_measurements`]).
    measured: u32,
    /// The kept strings past which the next sweep runs.
    limit: usize,
}

/// Frames a measurement no frame used is kept for.
const SWEEP_AGE: u64 = 60;
/// The fewest kept strings a sweep leaves room for.
const SWEEP_MIN: usize = 256;

/// The kept value of `text` under `key`, or `measure`'s, kept.
fn kept<V: Clone>(
    map: &mut std::collections::HashMap<String, Vec<Kept<V>>>,
    frame: u64,
    measured: &mut u32,
    text: &str,
    key: TextKey,
    measure: impl FnOnce() -> V,
) -> V {
    let measure = || {
        *measured += 1;
        measure()
    };
    if let Some(list) = map.get_mut(text) {
        if let Some(k) = list.iter_mut().find(|k| k.key == key) {
            k.frame = frame;
            return k.value.clone();
        }
        let value = measure();
        list.push(Kept {
            key,
            value: value.clone(),
            frame,
        });
        return value;
    }
    let value = measure();
    map.insert(
        text.to_owned(),
        vec![Kept {
            key,
            value: value.clone(),
            frame,
        }],
    );
    value
}

impl TextCache {
    pub(super) fn widths(
        &mut self,
        t: &TextContent,
        measure: impl FnOnce() -> (f32, f32),
    ) -> (f32, f32) {
        kept(
            &mut self.widths,
            self.frame,
            &mut self.made,
            &t.text,
            TextKey::of(t, None),
            measure,
        )
    }

    pub(super) fn lines(
        &mut self,
        t: &TextContent,
        width: Option<f32>,
        measure: impl FnOnce() -> Vec<Line>,
    ) -> Vec<Line> {
        kept(
            &mut self.lines,
            self.frame,
            &mut self.made,
            &t.text,
            TextKey::of(t, width),
            measure,
        )
    }

    /// Ends a frame. A node that keeps its layout measures nothing and
    /// reads none of this (its lines are its own), so what no frame used
    /// for a while goes, once the cache has grown to twice what it kept at
    /// its last sweep: the cost follows the measurements made.
    pub(crate) fn sweep(&mut self) {
        let frame = self.frame;
        let size = self.widths.len() + self.lines.len();
        if size > self.limit {
            for map in [&mut self.widths as &mut dyn Sweep, &mut self.lines] {
                map.sweep(frame.saturating_sub(SWEEP_AGE));
            }
            self.limit = (2 * (self.widths.len() + self.lines.len())).max(SWEEP_MIN);
        }
        self.frame += 1;
        self.measured = std::mem::take(&mut self.made);
    }

    /// The measurements the last laid-out frame made.
    pub(crate) fn measured(&self) -> u32 {
        self.measured
    }

    /// Forgets every measurement (the fonts changed).
    pub(crate) fn clear(&mut self) {
        self.widths.clear();
        self.lines.clear();
    }

    /// How many measurements are kept (tests).
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.widths.values().map(Vec::len).sum::<usize>()
            + self.lines.values().map(Vec::len).sum::<usize>()
    }
}

trait Sweep {
    fn sweep(&mut self, frame: u64);
}

impl<V> Sweep for std::collections::HashMap<String, Vec<Kept<V>>> {
    /// Drops what was last used before `frame`.
    fn sweep(&mut self, frame: u64) {
        self.retain(|_, list| {
            list.retain(|k| k.frame >= frame);
            !list.is_empty()
        });
    }
}

/// Last frame's nodes and which nodes have unchanged layout inputs. A
/// fixed-size child can isolate changed contents from its parent's inputs;
/// the child still lays out even when the parent reuses its own layout.
#[derive(Clone, Debug, Default)]
pub(crate) struct LayoutCache {
    /// Replaced or removed slots, with their previous layout.
    pub prev: std::collections::HashMap<usize, Node>,
    /// Per node: its own layout inputs and child size contributions are unchanged.
    pub clean: Vec<bool>,
    /// Fonts, theme, direction or pixel ratio changed: nothing is clean.
    pub epoch: u64,
    pub prev_epoch: u64,
    /// Nodes laid out this frame (the rest kept their layout).
    pub laid_out: u32,
    /// The theme, direction and pixel ratio last frame drew with: a change
    /// makes nothing clean.
    pub sig: Option<(crate::Theme, og_i18n::Direction, f32)>,
    /// This frame keeps nothing (`epoch` or `sig` changed).
    pub fresh: bool,
    /// Each element's own draw commands last frame, by key, reused while
    /// everything it draws from is unchanged, with how many of them are
    /// its background (its draw paints between those and the rest).
    pub cmds: crate::id::KeyMap<(usize, Vec<crate::DrawCmd>, DrawSig)>,
    pub measurements: crate::ui::settlement::Measurements,
    pub query_roots: Option<std::collections::BTreeSet<usize>>,
    /// Nodes whose laid-out size changed since they were last laid out.
    pub resized: Vec<usize>,
    /// Lay out every node this frame (a fresh frame, or the bounded
    /// walk's check): no subtree is left out.
    pub full: bool,
    /// Per node: layout reaches its children this frame (it is on a dirty
    /// path, or in a subtree laid out whole); `visiting` lists them.
    pub visit: Vec<bool>,
    pub visiting: Vec<usize>,
    /// Per node: laid out this frame (`laid` lists them, once each).
    pub laid_mark: Vec<bool>,
    pub laid: Vec<usize>,
    /// Nodes laid out whole every frame: what the frame moves after
    /// layout (a baseline row, an anchored or screen-placed element, a
    /// placement's box) is made again from their layout, never on top of
    /// last frame's.
    pub forced: std::collections::BTreeSet<usize>,
    /// Last frame's scroll containers: their offsets move their children.
    pub scrollers: Vec<usize>,
    /// Nodes whose children are derived again this frame (a child left).
    pub rederive: Vec<usize>,
    /// Nodes whose placement box was restored this frame.
    pub restored: Vec<usize>,
    /// Nodes `clean` holds false for (set true again next frame).
    pub unclean: Vec<usize>,
    /// Asks for `full` on every frame (the bounded walk's check).
    pub whole: bool,
    /// Last frame's commands of the screen and its layers, before baking,
    /// and each painted subtree's range in them: its start from its
    /// parent's (a layer's from the list's), its length and its nodes.
    pub last_cmds: Vec<(crate::DrawCmd, crate::element::Affine)>,
    pub ranges: crate::id::KeyMap<(usize, usize, u32)>,
    /// Per slot: the key and signature it was last painted with.
    pub sigs: Vec<Option<(u64, DrawSig)>>,
    /// The nodes not written this frame that paint otherwise than last
    /// frame (`None`: every node painted afresh).
    pub repainted: Option<Vec<usize>>,
    /// This frame copied every tree's commands whole from the last, and
    /// the trees' roots it drew.
    pub reused_all: bool,
    pub last_roots: Vec<u64>,
    /// What last frame drew after its trees (a cover, the HUD editor, the
    /// virtual cursor), before baking.
    pub last_tail: Vec<(crate::DrawCmd, crate::element::Affine)>,
    /// This frame's ranges copied from where they stood in last frame's
    /// commands, at the same place; last frame's commands, as `commands`
    /// leaves them (empty when taken whole), and how many there were.
    pub aligned: Vec<std::ops::Range<usize>>,
    /// How many ranges `aligned` held before a later insertion dropped
    /// some (their commands are no longer in last frame's list).
    pub aligned_kept: usize,
    pub prev_cmds: Vec<(crate::DrawCmd, crate::element::Affine)>,
    pub prev_len: usize,
    /// What last frame's levelling decided (`None`: level afresh).
    pub levels: Option<crate::draw::Levels>,
}

/// What the frame changes about a node's drawing without writing it again:
/// a retained node not written this frame draws its last commands while
/// these are as they were.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DrawSig {
    rect: Rect,
    clip: Rect,
    alpha: f32,
    world: crate::element::Affine,
    ring: bool,
    pad_rings: u8,
    scale: f32,
    sampled: crate::layout::node::SampledStyle,
}

impl DrawSig {
    /// How far a node with this signature moved from one with `old`, when
    /// moving is all that changed (its size, clip, map and style as they
    /// were, no transform): its last commands, moved as far, are its own.
    pub(crate) fn moved_from(&self, old: &DrawSig) -> Option<(f32, f32)> {
        let same = self.rect.width == old.rect.width
            && self.rect.height == old.rect.height
            && self.world.is_identity()
            && old.world.is_identity()
            && DrawSig {
                rect: old.rect,
                ..*self
            } == *old;
        same.then_some((self.rect.x - old.rect.x, self.rect.y - old.rect.y))
    }

    pub(crate) fn of(n: &Node) -> Self {
        Self {
            rect: n.rect,
            clip: n.clip,
            alpha: n.alpha,
            world: n.world,
            ring: n.ring,
            pad_rings: n.pad_rings,
            scale: n.scale,
            sampled: n.sampled,
        }
    }
}

/// Everything an element's own commands are made from is as last frame
/// (its ancestors' fills are checked by the caller, up the tree).
pub(crate) fn draw_eq(a: &Node, b: &Node) -> bool {
    a.rect == b.rect
        && a.clip == b.clip
        && a.alpha == b.alpha
        && a.world == b.world
        && a.ring == b.ring
        && a.pad_rings == b.pad_rings
        && a.heading == b.heading
        && a.disabled == b.disabled
        && a.aux == b.aux
        && a.scale == b.scale
        && a.sampled == b.sampled
        && a.el == b.el
        && a.content == b.content
        && a.widget == b.widget
        && a.shapes == b.shapes
        && a.lines == b.lines
        && a.rich_lines == b.rich_lines
}

impl LayoutCache {
    fn is_clean(&self, i: usize) -> bool {
        self.clean.get(i).copied().unwrap_or(false)
    }

    /// Whether layout reaches node `i`'s children this frame.
    pub(crate) fn visits(&self, i: usize) -> bool {
        self.full || self.visit.get(i).copied().unwrap_or(false)
    }

    /// Marks `i` and its ancestors as reached; with `whole`, its subtree too.
    pub(crate) fn reach(&mut self, nodes: &[Node], i: usize, whole: bool) {
        if self.visit.len() < nodes.len() {
            self.visit.resize(nodes.len(), false);
        }
        if whole {
            let mut stack = vec![i];
            while let Some(j) = stack.pop() {
                if !self.visit[j] {
                    self.visit[j] = true;
                    self.visiting.push(j);
                }
                stack.extend_from_slice(&nodes[j].ordered_children);
            }
        }
        let mut j = i;
        while j != NONE && j < nodes.len() {
            if self.visit[j] && j != i {
                break;
            }
            if !self.visit[j] {
                self.visit[j] = true;
                self.visiting.push(j);
            }
            j = nodes[j].parent;
        }
    }

    /// Forgets last frame's reached nodes.
    pub(crate) fn unreach(&mut self) {
        for i in std::mem::take(&mut self.visiting) {
            if let Some(v) = self.visit.get_mut(i) {
                *v = false;
            }
        }
        for i in std::mem::take(&mut self.laid) {
            if let Some(v) = self.laid_mark.get_mut(i) {
                *v = false;
            }
        }
    }

    /// The preorder of `root`'s tree that layout starts from: every node
    /// it reaches, with its children.
    pub(crate) fn walk(&self, nodes: &[Node], root: usize) -> Vec<usize> {
        let mut order = Vec::new();
        let mut stack = vec![root];
        while let Some(i) = stack.pop() {
            order.push(i);
            if self.visits(i) {
                stack.extend(nodes[i].ordered_children.iter().rev());
            }
        }
        order
    }

    /// Node `order[k]` lays out its children this frame after all (another
    /// size, place or clip than its kept layout's): they follow it in the
    /// walk, each as a subtree of its own.
    fn open(&self, nodes: &[Node], order: &mut Vec<usize>, opened: &mut Vec<usize>, k: usize) {
        let i = order[k];
        if self.visits(i) || opened.contains(&i) || nodes[i].ordered_children.is_empty() {
            return;
        }
        opened.push(i);
        order.splice(k + 1..k + 1, nodes[i].ordered_children.iter().copied());
    }

    /// Records the signature slot `i` was painted with.
    pub(crate) fn sigs_set(&mut self, i: usize, key: u64, sig: DrawSig) {
        if self.sigs.len() <= i {
            self.sigs.resize(i + 1, None);
        }
        self.sigs[i] = Some((key, sig));
    }

    /// Records that node `i` was laid out this frame.
    fn lay(&mut self, i: usize) {
        if self.laid_mark.len() <= i {
            self.laid_mark.resize(i + 1, false);
        }
        if !self.laid_mark[i] {
            self.laid_mark[i] = true;
            self.laid.push(i);
        }
    }
}

/// The box props layout reads are equal.
pub(crate) fn el_layout_eq(a: &Element, b: &Element) -> bool {
    a.direction == b.direction
        && a.width == b.width
        && a.height == b.height
        && a.padding == b.padding
        && a.gap == b.gap
        && a.align == b.align
        && a.cross == b.cross
        && a.wrap == b.wrap
        && a.scroll == b.scroll
        && a.scroll_x == b.scroll_x
        && a.aspect == b.aspect
        && a.margin_top == b.margin_top
        && a.margin_bottom == b.margin_bottom
        && a.block == b.block
}

/// The content layout measures is equal (an ink may differ).
pub(crate) fn content_layout_eq(a: &Content, b: &Content) -> bool {
    match (a, b) {
        (Content::Text(x), Content::Text(y)) => {
            x.text == y.text
                // An auto-fit text's step is layout's: its steps are its input.
                && (x.step == y.step || (!x.fit.is_empty() && x.fit == y.fit))
                && x.font == y.font
                && x.wrap == y.wrap
                && x.ellipsis == y.ellipsis
                && x.max_lines == y.max_lines
                && x.fit == y.fit
        }
        (Content::Image { .. }, Content::Image { .. }) => true,
        _ => a == b,
    }
}

/// The element-model props layout reads are equal.
pub(crate) fn props_layout_eq(a: &crate::element::Props, b: &crate::element::Props) -> bool {
    a.position == b.position
        && a.insets == b.insets
        && a.layout == b.layout
        && a.grid == b.grid
        && a.place == b.place
        && !a.variants.iter().any(|v| v.when.container)
        && !b.variants.iter().any(|v| v.when.container)
}

/// Lays out the tree under `root` inside `area`. `scroll(key)` is a scroll
/// container's offset; `dpr` snaps the final rectangles.
#[allow(clippy::too_many_arguments)]
pub(crate) fn layout(
    nodes: &mut [Node],
    root: usize,
    area: Rect,
    measure: &mut Measure<'_>,
    scroll: &dyn Fn(u64) -> (f32, f32),
    dpr: f32,
    clip: Rect,
    kept: &mut LayoutCache,
) {
    if let Some(roots) = &kept.query_roots {
        let mut at = root;
        while nodes[at].parent != NONE {
            at = nodes[at].parent;
        }
        if !roots.contains(&at) {
            return;
        }
    }
    // The nodes this call lays out: the dirty paths and their children
    // (`LayoutCache::visit`), opened further where a clean subtree is
    // given another size, moved or clipped otherwise.
    let mut order = kept.walk(nodes, root);
    let mut opened: Vec<usize> = Vec::new();
    for &i in order.iter().rev() {
        if kept.is_clean(i) {
            continue;
        }
        size_w(nodes, i, measure);
    }
    nodes[root].w = match nodes[root].el.width {
        Size::Fixed(v) => v,
        Size::Fit { .. } => nodes[root].pref_w.min(area.width).max(nodes[root].min_w),
        Size::Percent(p) => p * area.width,
        Size::Grow { .. } => {
            let (lo, hi) = nodes[root].el.width.bounds();
            area.width.clamp(lo, hi)
        }
    };
    // A clean node at the width it had keeps its children's widths, its
    // lines and its tracks.
    let same_w = |nodes: &[Node], kept: &LayoutCache, i: usize| {
        kept.is_clean(i) && nodes[i].layout_size[0] == nodes[i].w
    };
    let mut k = 0;
    while k < order.len() {
        let i = order[k];
        if same_w(nodes, kept, i) {
            // An auto-fit text keeps the step its lines were broken at.
            let step = nodes[i].layout_step;
            if let (Content::Text(a), Some(step)) = (&mut nodes[i].content, step)
                && !a.fit.is_empty()
            {
                a.step = step;
            }
        } else {
            kept.laid_out += 1;
            distribute_w(nodes, i, measure);
            kept.open(nodes, &mut order, &mut opened, k);
        }
        k += 1;
    }
    for &i in order.iter().rev() {
        if same_w(nodes, kept, i) {
            continue;
        }
        size_h(nodes, i);
    }
    nodes[root].h = match nodes[root].el.height {
        Size::Fixed(v) => v,
        Size::Fit { .. } => nodes[root].pref_h,
        Size::Percent(p) => p * area.height,
        Size::Grow { .. } => {
            let (lo, hi) = nodes[root].el.height.bounds();
            area.height.clamp(lo, hi)
        }
    };
    let same_wh = |nodes: &[Node], kept: &LayoutCache, i: usize| {
        same_w(nodes, kept, i) && nodes[i].layout_size[1] == nodes[i].h
    };
    let mut k = 0;
    while k < order.len() {
        let i = order[k];
        if !same_wh(nodes, kept, i) {
            distribute_h(nodes, i);
            kept.open(nodes, &mut order, &mut opened, k);
        }
        k += 1;
    }
    nodes[root].x = area.x;
    nodes[root].y = area.y;
    let mut k = 0;
    while k < order.len() {
        let i = order[k];
        if !nodes[i].scrolls() && same_wh(nodes, kept, i) {
            for &c in &nodes[i].ordered_children.clone() {
                nodes[c].x = nodes[i].x + nodes[c].layout_offset[0];
                nodes[c].y = nodes[i].y + nodes[c].layout_offset[1];
            }
            // Children left out keep their places while it keeps its own.
            if [nodes[i].x, nodes[i].y] != nodes[i].laid_xy {
                kept.open(nodes, &mut order, &mut opened, k);
            }
        } else {
            position(nodes, i, scroll);
            kept.open(nodes, &mut order, &mut opened, k);
        }
        k += 1;
    }
    nodes[root].clip = clip;
    let mut k = 0;
    while k < order.len() {
        let i = order[k];
        let n = &nodes[i];
        let rect = Rect::new(n.x, n.y, n.w, n.h);
        let (parent_clip, src) = if n.parent == NONE || i == root {
            (clip, NONE)
        } else {
            let p = &nodes[n.parent];
            if p.el.scroll || p.el.scroll_x {
                (p.laid_clip.intersect(&p.laid_rect), n.parent)
            } else {
                (p.laid_clip, p.clip_src)
            }
        };
        if parent_clip != nodes[i].laid_clip || src != nodes[i].clip_src {
            kept.open(nodes, &mut order, &mut opened, k);
        }
        nodes[i].clip = parent_clip;
        nodes[i].laid_clip = parent_clip;
        nodes[i].clip_src = src;
        nodes[i].rect = rect.snapped(dpr);
        nodes[i].laid_rect = nodes[i].rect;
        nodes[i].laid_xy = [nodes[i].x, nodes[i].y];
        kept.lay(i);
        if nodes[i].layout_size != [nodes[i].w, nodes[i].h] {
            kept.resized.push(i);
        }
        nodes[i].layout_size = [nodes[i].w, nodes[i].h];
        nodes[i].layout_offset = if nodes[i].parent == NONE || i == root {
            [0.0; 2]
        } else {
            let parent = &nodes[nodes[i].parent];
            [nodes[i].x - parent.x, nodes[i].y - parent.y]
        };
        nodes[i].layout_step = match &nodes[i].content {
            Content::Text(text) if !text.fit.is_empty() => Some(text.step),
            _ => None,
        };
        kept.measurements.observe(
            nodes[i].key,
            crate::ui::settlement::ContentBox {
                width: (nodes[i].w - nodes[i].el.padding.horizontal()).max(0.0),
                height: (nodes[i].h - nodes[i].el.padding.vertical()).max(0.0),
            },
        );
        k += 1;
    }
}

#[cfg(test)]
mod text_cache {
    use super::{Content, Rect, SWEEP_AGE, SWEEP_MIN};
    use crate::{Fonts, InputFrame, LabelDesc, Size, Ui, UiDesc, UiFont, Viewport};

    /// One frame of three labels (a wrapping one at `width`, one cut with
    /// an ellipsis, one unwrapped), `count` in the last: each text's lines,
    /// their widths and its rectangle.
    fn frame(ui: &mut Ui, width: f32, count: u32) -> Vec<(Vec<(String, f32)>, Rect)> {
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        ui.label(
            "wrap",
            "the quick brown fox jumps over the lazy dog",
            LabelDesc {
                width: Size::Fixed(width),
                ..LabelDesc::default()
            },
        );
        ui.label(
            "cut",
            "a title too long for its slot",
            LabelDesc {
                width: Size::Fixed(width / 2.0),
                ellipsis: true,
                nowrap: true,
                ..LabelDesc::default()
            },
        );
        ui.label("count", &format!("score {count}"), LabelDesc::default());
        ui.end();
        ui.nodes
            .iter()
            .filter(|n| matches!(n.content, Content::Text(_)))
            .map(|n| {
                let lines = n.lines.iter().map(|l| (l.text.clone(), l.width));
                (lines.collect(), n.rect)
            })
            .collect()
    }

    fn fresh() -> Ui {
        Ui::new(UiDesc {
            fonts: Fonts {
                body: UiFont::unrendered(0.55),
                ..Fonts::default()
            },
            ..UiDesc::default()
        })
    }

    #[test]
    fn a_kept_measurement_lays_out_as_a_fresh_one() {
        // Frames whose widths and counts change and come back: each laid
        // out by one Ui that keeps its measurements equals a Ui laying it
        // out for the first time.
        let mut kept = fresh();
        for (width, count) in [(120.0, 1), (120.0, 1), (200.0, 1), (120.0, 2), (90.0, 3)] {
            let laid = frame(&mut kept, width, count);
            assert_eq!(laid, frame(&mut fresh(), width, count), "width {width}");
            // The wrapping text wraps and the cut one is cut.
            assert!(laid[0].0.len() > 1, "{laid:?}");
            assert!(laid[1].0[0].0.ends_with('…') || laid[1].0[0].0.ends_with("..."));
        }
    }

    #[test]
    fn a_frame_measures_only_the_texts_that_changed() {
        // Three labels: a wrapping one, a cut one and a counter. Each text
        // is measured for its widths and for its lines.
        let mut ui = fresh();
        frame(&mut ui, 120.0, 1);
        assert_eq!(ui.text_measurements(), 6, "the first frame measures all");
        frame(&mut ui, 120.0, 1);
        assert_eq!(
            ui.text_measurements(),
            0,
            "an unchanged frame measures none"
        );
        frame(&mut ui, 120.0, 2);
        assert_eq!(
            ui.text_measurements(),
            2,
            "the new count's widths and lines"
        );
        // A new width breaks the wrapping text and cuts the cut one again;
        // their widths and the counter (which breaks only at \n) are kept.
        frame(&mut ui, 200.0, 2);
        assert_eq!(ui.text_measurements(), 2);
        let fonts = ui.fonts().clone();
        ui.set_fonts(fonts);
        frame(&mut ui, 200.0, 2);
        assert_eq!(ui.text_measurements(), 6, "new fonts measure everything");
    }

    #[test]
    fn an_unchanged_text_is_measured_once_and_gone_ones_are_dropped() {
        let mut ui = fresh();
        frame(&mut ui, 120.0, 1);
        // Each label's widths and lines.
        let first = ui.texts.len();
        assert_eq!(first, 6, "{:?}", ui.texts);
        frame(&mut ui, 120.0, 1);
        assert_eq!(ui.texts.len(), first, "the same frame keeps what it had");
        // A count that changes every frame leaves its old texts behind; they
        // go once the cache has doubled, so it stays bounded.
        for count in 2..2000 {
            frame(&mut ui, 120.0, count);
            assert_eq!(ui.text_measurements(), 2, "frame {count}");
        }
        assert!(
            ui.texts.len() <= 2 * (SWEEP_MIN + 2 * SWEEP_AGE as usize),
            "{}",
            ui.texts.len()
        );
        assert!(ui.texts.lines.contains_key("score 1999"));
        assert!(!ui.texts.lines.contains_key("score 1"));
        // New fonts forget every measurement.
        let fonts = ui.fonts().clone();
        ui.set_fonts(fonts);
        assert_eq!(ui.texts.len(), 0);
    }
}
