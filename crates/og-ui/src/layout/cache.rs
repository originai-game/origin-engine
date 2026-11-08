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
}

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

    /// Keeps `t`'s measurements through this frame's sweep: a node that
    /// kept its layout (docs/ui.md §2) did not measure, but still uses them.
    fn touch(&mut self, t: &TextContent) {
        let frame = self.frame;
        if let Some(list) = self.widths.get_mut(&t.text) {
            list.iter_mut().for_each(|k| k.frame = frame);
        }
        if let Some(list) = self.lines.get_mut(&t.text) {
            list.iter_mut().for_each(|k| k.frame = frame);
        }
    }

    /// Ends a frame: what it did not measure goes.
    pub(crate) fn sweep(&mut self) {
        let frame = self.frame;
        for map in [&mut self.widths as &mut dyn Sweep, &mut self.lines] {
            map.sweep(frame);
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
    fn sweep(&mut self, frame: u64) {
        self.retain(|_, list| {
            list.retain(|k| k.frame == frame);
            !list.is_empty()
        });
    }
}

/// Last frame's nodes and which nodes have unchanged layout inputs. A
/// fixed-size child can isolate changed contents from its parent's inputs;
/// the child still lays out even when the parent reuses its own layout.
#[derive(Clone, Debug, Default)]
pub(crate) struct LayoutCache {
    /// Last frame's nodes, with their layout.
    pub prev: Vec<Node>,
    /// Per node of this frame: its twin in `prev` (NONE: new).
    pub twin: Vec<usize>,
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
    pub cmds: crate::id::KeyMap<(usize, Vec<crate::DrawCmd>)>,
    pub measurements: crate::ui::settlement::Measurements,
    pub query_roots: Option<std::collections::BTreeSet<usize>>,
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
    let order = preorder(nodes, root);
    for &i in order.iter().rev() {
        if kept.is_clean(i) {
            if let Content::Text(t) = &nodes[i].content {
                measure.texts.touch(t);
            }
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
    let kids_of = |n: &Node| all_children(n);
    for &i in &order {
        if same_w(nodes, kept, i) {
            // An auto-fit text keeps the step its lines were broken at.
            let step = nodes[i].layout_step;
            if let (Content::Text(a), Some(step)) = (&mut nodes[i].content, step)
                && !a.fit.is_empty()
            {
                a.step = step;
            }
            continue;
        }
        kept.laid_out += 1;
        distribute_w(nodes, i, measure);
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
    for &i in &order {
        if same_wh(nodes, kept, i) {
            continue;
        }
        distribute_h(nodes, i);
    }
    nodes[root].x = area.x;
    nodes[root].y = area.y;
    for &i in &order {
        if !nodes[i].scrolls() && same_wh(nodes, kept, i) {
            for c in kids_of(&nodes[i]) {
                nodes[c].x = nodes[i].x + nodes[c].layout_offset[0];
                nodes[c].y = nodes[i].y + nodes[c].layout_offset[1];
            }
            continue;
        }
        position(nodes, i, scroll);
    }
    nodes[root].clip = clip;
    for &i in &order {
        let n = &nodes[i];
        let rect = Rect::new(n.x, n.y, n.w, n.h);
        let (parent_clip, src) = if n.parent == NONE || i == root {
            (clip, NONE)
        } else {
            let p = &nodes[n.parent];
            if p.el.scroll || p.el.scroll_x {
                (p.clip.intersect(&p.rect), n.parent)
            } else {
                (p.clip, p.clip_src)
            }
        };
        nodes[i].clip = parent_clip;
        nodes[i].clip_src = src;
        nodes[i].rect = rect.snapped(dpr);
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
    }
}

#[cfg(test)]
mod text_cache {
    use super::{Content, Rect};
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
    fn an_unchanged_text_is_measured_once_and_a_gone_one_is_dropped() {
        let mut ui = fresh();
        frame(&mut ui, 120.0, 1);
        // Each label's widths and lines.
        let first = ui.texts.len();
        assert_eq!(first, 6, "{:?}", ui.texts);
        frame(&mut ui, 120.0, 1);
        assert_eq!(ui.texts.len(), first, "the same frame keeps what it had");
        // A new count measures the new text; the old one goes at the sweep.
        frame(&mut ui, 120.0, 2);
        assert_eq!(ui.texts.len(), first);
        assert!(ui.texts.lines.contains_key("score 2"));
        assert!(!ui.texts.lines.contains_key("score 1"));
        // New fonts forget every measurement.
        let fonts = ui.fonts().clone();
        ui.set_fonts(fonts);
        assert_eq!(ui.texts.len(), 0);
    }
}
