//! Line boxes among floats (CSS 2 §9.5, §10.8). A text's lines and a
//! block's lines of inline-level boxes are broken one line at a time into
//! the room the floats of their block formatting context leave, which
//! Taffy places (`BlockContext`): a line no word or box fits moves down
//! past them. A block's floats among its lines are placed where they
//! come, at the top of their line when they fit beside what it holds, else
//! at the next line's (§9.5.1 rules 6 and 8). Each line of boxes is laid
//! out as a single flex line by Taffy, as the block's lines are when no
//! float reaches them; a line holding a `vertical-align` is then set as
//! its line box holds it.
//!
//! Taffy's float context holds rectangles, and `shape-outside` narrows
//! lines by its contour (CSS Shapes 1 §3): a context holding such a float
//! lays out again with its floats where the pass before put them
//! ([`Tree::settle_shapes`]), until they stay.

use super::*;
use crate::layout::float::{Placed, line_room};
use std::collections::HashMap;

/// A block formatting context being laid out.
#[derive(Default)]
pub(super) struct Bfc {
    pub root: usize,
    /// Its floats laid out so far this pass, in document order.
    pub floats: Vec<usize>,
    /// The boxes whose lines its floats shortened this pass.
    pub wrappers: Vec<usize>,
}

/// A context with a shaped float, as its last pass laid it out: its
/// floats' margin boxes and contours, and its line boxes' border-box
/// origins, in the root's border box.
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct Shaped {
    floats: Vec<(usize, Placed)>,
    at: HashMap<usize, [f32; 2]>,
}

/// The floats beside a box's lines.
pub(super) enum Around<'c, 'b> {
    /// Taffy's float context: margin boxes only.
    Taffy(&'c tf::BlockContext<'b>),
    /// The context's floats where its last pass put them, in the box's
    /// border box, each with its id: only those laid out so far count.
    Shapes(Vec<(usize, Placed)>, Vec<usize>),
}

impl Around<'_, '_> {
    /// The room beside a line from `y` as tall as `h` between `x0` and
    /// `x1` (the box's border box): its left and right edges, and the
    /// highest bottom of the floats that narrow it (`None`: none does).
    pub fn room(&self, y: f32, h: f32, x0: f32, x1: f32) -> (f32, f32, Option<f32>) {
        let end = y + h.max(0.01);
        match self {
            Around::Shapes(all, known) => {
                let floats: Vec<Placed> = all
                    .iter()
                    .filter(|(id, _)| known.contains(id))
                    .map(|(_, p)| p.clone())
                    .collect();
                line_room(&floats, y, end, x0, x1)
            }
            Around::Taffy(ctx) => {
                let (mut l, mut r, mut next) = (x0, x1, None::<f32>);
                let (mut q, mut after) = (y, None);
                loop {
                    let s = ctx.find_content_slot(q, tf::Clear::None, after);
                    let Some(seg) = s.segment_id else { break };
                    if s.y >= end {
                        break;
                    }
                    // Its bottom: the next segment's top, or the floats'.
                    let n = ctx.find_content_slot(s.y, tf::Clear::None, Some(seg));
                    let bottom = match n.segment_id {
                        Some(_) => n.y,
                        None => ctx.cleared_threshold(tf::Clear::Both).unwrap_or(s.y),
                    };
                    let (sl, sr) = (s.x.max(x0), (s.x + s.width).min(x1));
                    if sl > x0 + 0.001 || sr < x1 - 0.001 {
                        (l, r) = (l.max(sl), r.min(sr));
                        next = Some(next.map_or(bottom, |b| b.min(bottom)));
                    }
                    if n.segment_id.is_none() || bottom >= end || bottom <= q {
                        break;
                    }
                    (q, after) = (bottom, Some(seg));
                }
                (l, r, next)
            }
        }
    }
}

/// A text whose lines floats shorten (an auto-fit, cut or path text keeps
/// its lines).
fn wraps(n: &Node) -> bool {
    matches!(&*n.content, Content::Text(t) if t.fit.is_empty() && !t.ellipsis && t.max_lines == 0)
        && n.props.text_path.is_none()
        && n.widget == Widget::None
}

/// A rich text whose lines floats shorten.
fn rich_wraps(n: &Node) -> bool {
    matches!(&*n.content, Content::Rich(rc) if rc.wrap)
        && n.props.text_path.is_none()
        && n.widget == Widget::None
}

/// A box whose `vertical-align` moves it in its line against a parent of
/// dominant baseline `dominant`: `top` and `bottom` take `align_self`.
fn shifted(n: &Node, dominant: crate::layout::Baseline) -> bool {
    use crate::layout::BaselineShift as S;
    let v = n.layout.used.valign;
    !matches!(v.shift, S::Top | S::Bottom) && !v.is_baseline(dominant)
}

fn margins(nodes: &[Node], c: usize, basis: f32) -> tf::Rect<f32> {
    use tf::{CoreStyle, ResolveOrZero};
    View::item(nodes, c)
        .margin()
        .resolve_or_zero(Some(basis), |p, b| calc_of(nodes, p, b))
}

impl Tree<'_, '_> {
    /// Box `c`'s size, measured as Taffy's `measure_child_size` does.
    fn size_of(
        &mut self,
        c: tf::NodeId,
        known: tf::Size<Option<f32>>,
        parent: tf::Size<Option<f32>>,
        avail: tf::Size<tf::AvailableSpace>,
        sizing_mode: tf::SizingMode,
        collapsible: tf::Line<bool>,
    ) -> tf::Size<f32> {
        self.compute_child_layout(
            c,
            tf::LayoutInput {
                run_mode: tf::RunMode::ComputeSize,
                sizing_mode,
                axis: tf::RequestedAxis::Both,
                known_dimensions: known,
                known_dimensions_are_definite: known.map(|k| k.is_some()),
                parent_size: parent,
                available_space: avail,
                vertical_margins_are_collapsible: collapsible,
            },
        )
        .size
    }

    /// Box `c`'s width, measured as Taffy's `measure_child_size` does.
    #[allow(clippy::too_many_arguments)]
    fn width_of(
        &mut self,
        c: tf::NodeId,
        known: tf::Size<Option<f32>>,
        parent: tf::Size<Option<f32>>,
        avail: tf::Size<tf::AvailableSpace>,
        sizing_mode: tf::SizingMode,
        _axis: tf::AbsoluteAxis,
        collapsible: tf::Line<bool>,
    ) -> f32 {
        self.compute_child_layout(
            c,
            tf::LayoutInput {
                run_mode: tf::RunMode::ComputeSize,
                sizing_mode,
                axis: tf::RequestedAxis::Horizontal,
                known_dimensions: known,
                known_dimensions_are_definite: known.map(|k| k.is_some()),
                parent_size: parent,
                available_space: avail,
                vertical_margins_are_collapsible: collapsible,
            },
        )
        .size
        .width
    }

    /// The floats beside box `i`'s lines, where its context has a float a
    /// contour narrows lines by and its last pass placed `i`.
    fn shapes<'c, 'b>(&self, i: usize) -> Option<Around<'c, 'b>> {
        let bfc = self.bfcs.last()?;
        let s = self.shaped.get(&bfc.root)?;
        let [ox, oy] = *s.at.get(&i)?;
        let floats = s
            .floats
            .iter()
            .map(|(id, p)| {
                let mut p = p.clone();
                (p.x, p.y) = (p.x - ox, p.y - oy);
                if let Some(c) = &mut p.contour {
                    for e in c.iter_mut().flatten() {
                        (e[0], e[1]) = (e[0] - ox, e[1] - oy);
                    }
                }
                (*id, p)
            })
            .collect();
        Some(Around::Shapes(floats, bfc.floats.clone()))
    }

    /// Notes that box `i` lays its lines out beside its context's floats.
    fn among_floats(&mut self, i: usize, perform: bool) {
        if perform
            && let Some(b) = self.bfcs.last_mut()
            && !b.wrappers.contains(&i)
        {
            b.wrappers.push(i);
        }
    }

    /// Floats narrowed or moved a line of box `i`.
    fn beside_floats(&mut self, i: usize, perform: bool) {
        if perform {
            self.nodes[i].layout.beside = true;
        }
    }

    /// Text leaf `i` among floats: its lines broken one at a time at the
    /// room the floats beside each leave, one no word fits moved down past
    /// them. `None` where no float shortens or moves a line of it (it lays
    /// out as a text alone does).
    pub(super) fn text_around(
        &mut self,
        i: usize,
        input: tf::LayoutInput,
        ctx: &mut tf::BlockContext<'_>,
    ) -> Option<tf::LayoutOutput> {
        let n = &self.nodes[i];
        let w = input.known_dimensions.width?;
        if !(wraps(n) || rich_wraps(n)) {
            return None;
        }
        let pad = n.layout.used.padding;
        ctx.apply_content_box_inset([pad.left, pad.right]);
        self.among_floats(i, input.run_mode == tf::RunMode::PerformLayout);
        let n = &self.nodes[i];
        let around = self.shapes(i).unwrap_or(Around::Taffy(ctx));
        let (x0, x1) = (pad.left, (w - pad.right).max(pad.left));
        let full = x1 - x0;
        let lowest = match &around {
            Around::Taffy(c) => c.cleared_threshold(tf::Clear::Both),
            Around::Shapes(f, known) => f
                .iter()
                .filter(|(id, _)| known.contains(id))
                .map(|(_, p)| p.y + p.h)
                .reduce(f32::max),
        }
        .unwrap_or(f32::NEG_INFINITY)
            - pad.top;
        // The room beside a line at `y` (its content box's) as tall as
        // `h`: its left, its width, the floats' next bottom.
        let room_at = |y: f32, h: f32| {
            if y >= lowest {
                return (0.0, full, None);
            }
            let (l, r, next) = around.room(pad.top + y, h, x0, x1);
            (l - x0, (r - l).max(0.0), next.map(|b| b - pad.top))
        };
        let fonts = self.measure.fonts;
        let (h, baseline, lines, shifts) = match &*n.content {
            Content::Text(_) => {
                let t = n.text()?;
                let lh = t.line_height();
                let m = t.metrics(fonts);
                let at = |s: &str, w: f32| {
                    let part = TextContent {
                        text: s.to_string(),
                        ..t.content().clone()
                    };
                    crate::layout::text::text_lines(part.at(t.step), m, t.wrap.then_some(w + 0.5))
                };
                let (mut lines, mut shifts) = (Vec::new(), Vec::new());
                let (mut narrowed, mut drop) = (false, 0.0f32);
                let mut rest: &str = &t.text;
                loop {
                    let y = lines.len() as f32 * lh + drop;
                    if y >= lowest {
                        for l in at(rest, full) {
                            shifts.push([0.0, drop, full]);
                            lines.push(l);
                        }
                        break;
                    }
                    let (l, avail, next) = room_at(y, lh);
                    let first = at(rest, avail).into_iter().next().unwrap_or(Line {
                        text: String::new(),
                        width: 0.0,
                    });
                    if let Some(b) = next
                        && first.width > avail + 0.5
                        && b > y
                    {
                        drop += b - y;
                        continue;
                    }
                    narrowed |= next.is_some();
                    let p = rest.find(first.text.as_str()).unwrap_or(0);
                    rest = rest[p + first.text.len()..].trim_start_matches([' ', '\t']);
                    let newline = rest.starts_with('\n') || rest.starts_with("\r\n");
                    rest = rest
                        .strip_prefix("\r\n")
                        .or_else(|| rest.strip_prefix('\n'))
                        .unwrap_or(rest);
                    let moved = !first.text.is_empty() || newline;
                    shifts.push([l, drop, avail]);
                    lines.push(first);
                    if rest.is_empty() || !moved {
                        if newline {
                            shifts.push([0.0, drop, full]);
                            lines.push(Line {
                                text: String::new(),
                                width: 0.0,
                            });
                        }
                        break;
                    }
                }
                if !narrowed && drop == 0.0 {
                    return None;
                }
                let h = lines.len().max(1) as f32 * lh + drop;
                let b = lines
                    .first()
                    .map(|l| m.baseline(&l.text, 0.0, lh, t.step.size) + shifts[0][1]);
                (h, b, super::text::Lines::Text { lines, step: None }, shifts)
            }
            Content::Rich(rc) => {
                let v = n.rich_vertical();
                let (lines, shifts, narrowed, drop) =
                    crate::layout::float::rich_lines_around(rc, fonts, lowest, v, room_at);
                if !narrowed && drop == 0.0 {
                    return None;
                }
                let h = lines.iter().map(|l| l.height).sum::<f32>() + drop;
                let b = lines.first().map(|l| l.baseline + shifts[0][1]);
                (h, b, super::text::Lines::Rich(lines), shifts)
            }
            _ => return None,
        };
        let height = input.known_dimensions.height.unwrap_or(h + pad.vertical());
        let perform = input.run_mode == tf::RunMode::PerformLayout;
        if perform {
            let n = &mut self.nodes[i];
            lines.store(n);
            n.layout.fit_step = None;
            n.layout.wrap_lines = shifts;
        }
        self.beside_floats(i, perform);
        let mut out = tf::LayoutOutput::from_outer_size(tf::Size { width: w, height });
        out.baselines.first = baseline.map(|b| b + pad.top);
        Some(out)
    }

    /// Node `i`'s floats, the floats of its context reaching its content,
    /// or its boxes' `vertical-align`: its lines laid out one at a time
    /// ([`Tree::line_boxes`]). Otherwise `None`: it lays out as a
    /// wrapping flex row.
    pub(super) fn lines(
        &mut self,
        i: usize,
        input: tf::LayoutInput,
        ctx: Option<&mut tf::BlockContext<'_>>,
    ) -> Option<tf::LayoutOutput> {
        let n = &self.nodes[i];
        let near = ctx.as_ref().is_some_and(|c| c.has_active_floats(0.0));
        let shaped = self.shapes(i).is_some();
        if n.floats.is_empty()
            && !near
            && !shaped
            && !n
                .children
                .iter()
                .any(|&c| shifted(&self.nodes[c], n.layout.used.dominant))
        {
            return None;
        }
        // Its width: given, else as its wrapping row sizes it.
        let w = match input.known_dimensions.width {
            Some(w) => w,
            None if input.available_space.width.is_definite() => {
                let mut m = input;
                m.run_mode = tf::RunMode::ComputeSize;
                m.axis = tf::RequestedAxis::Horizontal;
                self.flex(i, m).size.width
            }
            None => return None,
        };
        Some(match ctx {
            Some(c) => self.line_boxes(i, input, w, c, false),
            None => {
                // It starts a context of its own.
                let mut bfc = tf::BlockFormattingContext::new();
                let mut c = bfc.root_block_context();
                c.set_width(w);
                self.line_boxes(i, input, w, &mut c, true)
            }
        })
    }

    fn line_boxes(
        &mut self,
        i: usize,
        input: tf::LayoutInput,
        w: f32,
        ctx: &mut tf::BlockContext<'_>,
        root: bool,
    ) -> tf::LayoutOutput {
        let perform = input.run_mode == tf::RunMode::PerformLayout;
        self.among_floats(i, perform);
        let n = &self.nodes[i];
        let el = n.layout.used;
        let pad = el.padding;
        ctx.apply_content_box_inset([pad.left, pad.right]);
        let (x0, x1) = (pad.left, (w - pad.right).max(pad.left));
        let inner = x1 - x0;
        let (gap, line_gap) = (el.main_gap(), el.line_gap());
        let kids = n.children.clone();
        // Each float with the number of boxes before it in the content.
        let mut floats: std::collections::VecDeque<(usize, usize)> = n
            .floats
            .iter()
            .filter(|&&f| !self.nodes[f].props.out_of_flow())
            .map(|&f| {
                let order = &n.ordered_children;
                let before = order.iter().position(|&k| k == f).map_or(0, |p| {
                    order[..p].iter().filter(|k| kids.contains(k)).count()
                });
                (before, f)
            })
            .collect();
        let inline: Vec<bool> = kids
            .iter()
            .map(|&c| style::inline_level(&self.nodes[c]))
            .collect();
        // Each box's outer size as a flex item of the line would hold it.
        let measured: Vec<tf::Size<f32>> = kids
            .iter()
            .map(|&c| {
                let m = margins(self.nodes, c, inner);
                let s = self.size_of(
                    id(c),
                    tf::Size::NONE,
                    tf::Size {
                        width: Some(inner),
                        height: None,
                    },
                    tf::Size {
                        width: tf::AvailableSpace::Definite(inner),
                        height: tf::AvailableSpace::MaxContent,
                    },
                    tf::SizingMode::InherentSize,
                    tf::Line::FALSE,
                );
                let basis = {
                    use tf::FlexboxItemStyle;
                    View::item(self.nodes, c).flex_basis()
                };
                let width = basis.into_option().unwrap_or(s.width);
                tf::Size {
                    width: width + m.left + m.right,
                    height: s.height + m.top + m.bottom,
                }
            })
            .collect();
        let mut shapes = self.shapes(i);
        let mut placed = Vec::new();
        let (mut rows, mut held) = (Vec::new(), Vec::new());
        let (mut s, mut y, mut narrowed) = (0, 0.0f32, false);
        let (mut first_baseline, mut last_baseline) = (None, None);
        let mut later: Vec<usize> = Vec::new();
        while s < kids.len() {
            for f in std::mem::take(&mut later) {
                self.place_float(
                    f,
                    pad.top + y,
                    inner,
                    ctx,
                    perform,
                    &mut shapes,
                    &mut placed,
                );
            }
            while let Some(&(b, f)) = floats.front()
                && b <= s
            {
                floats.pop_front();
                self.place_float(
                    f,
                    pad.top + y,
                    inner,
                    ctx,
                    perform,
                    &mut shapes,
                    &mut placed,
                );
            }
            if !inline[s] {
                let (h, b) = self.block_in_lines(kids[s], pad.top + y, x0, inner, w, ctx, input);
                if s == 0 {
                    first_baseline = b.map(|b| b + pad.top + y);
                }
                // A block's last baseline is not known here.
                last_baseline = None;
                y = h - pad.top;
                rows.push((s, s + 1));
                held.push(None);
                y += line_gap;
                s += 1;
                continue;
            }
            // The room beside the line as tall as the boxes it holds; a
            // float that comes on it and fits beside what it holds so far
            // is placed at its top, and the line filled again.
            let mut h = measured[s].height;
            let (e, l, avail, next) = 'fit: loop {
                let taffy = Around::Taffy(ctx);
                let (l, r, next) = shapes
                    .as_ref()
                    .unwrap_or(&taffy)
                    .room(pad.top + y, h, x0, x1);
                let avail = (r - l).max(0.0);
                let (mut e, mut used) = (s + 1, measured[s].width);
                loop {
                    if let Some(&(b, f)) = floats.front()
                        && b == e
                    {
                        floats.pop_front();
                        let fw = self.float_size(f, inner).0.width;
                        if used + fw <= avail + 0.5 {
                            self.place_float(
                                f,
                                pad.top + y,
                                inner,
                                ctx,
                                perform,
                                &mut shapes,
                                &mut placed,
                            );
                            continue 'fit;
                        }
                        later.push(f);
                        continue;
                    }
                    if e < kids.len() && inline[e] && used + gap + measured[e].width <= avail + 0.5
                    {
                        used += gap + measured[e].width;
                        e += 1;
                    } else {
                        break;
                    }
                }
                let tall = (s..e).map(|k| measured[k].height).fold(0.0, f32::max);
                if tall <= h + 0.01 {
                    break (e, l, avail, next);
                }
                h = tall;
            };
            if let Some(b) = next
                && measured[s].width > avail + 0.5
                && b > pad.top + y
            {
                y = b - pad.top;
                continue;
            }
            narrowed |= next.is_some();
            let (lh, base, line_held) = self.line(i, s, e, l, pad.top + y, avail, input.run_mode);
            if rows.is_empty() {
                first_baseline = base.map(|b| b + pad.top + y);
            }
            last_baseline = base.map(|b| b + pad.top + y);
            rows.push((s, e));
            held.push(line_held.then_some(lh));
            y += lh + line_gap;
            s = e;
        }
        if !rows.is_empty() {
            y -= line_gap;
        }
        // What comes after the last box: at the last line's top when it
        // fit there (placed above), else below it.
        for f in later.into_iter().chain(floats.into_iter().map(|(_, f)| f)) {
            self.place_float(
                f,
                pad.top + y,
                inner,
                ctx,
                perform,
                &mut shapes,
                &mut placed,
            );
        }
        let mut content = pad.top + y;
        if root && let Some(b) = ctx.cleared_threshold(tf::Clear::Both) {
            content = content.max(b);
        }
        let height = input
            .known_dimensions
            .height
            .unwrap_or(content + pad.bottom);
        if perform {
            let n = &mut self.nodes[i];
            n.layout.rows = rows;
            n.layout.inline_rows = held;
            n.layout.wrap_lines.clear();
        }
        if narrowed {
            self.beside_floats(i, perform);
        }
        let mut out = tf::LayoutOutput::from_outer_size(tf::Size { width: w, height });
        out.baselines.first = first_baseline;
        out.baselines.last = last_baseline;
        out
    }

    /// A float's border-box size and margins in a line box `inner` wide:
    /// its width, else its content's, shrunk to fit (CSS 2 §10.3.5).
    fn float_size(&mut self, f: usize, inner: f32) -> (tf::Size<f32>, tf::Rect<f32>) {
        let m = margins(self.nodes, f, inner);
        let parent = tf::Size {
            width: Some(inner),
            height: None,
        };
        let room = (inner - m.left - m.right).max(0.0);
        let at = |t: &mut Self, space: tf::AvailableSpace| {
            t.width_of(
                id(f),
                tf::Size::NONE,
                parent,
                tf::Size {
                    width: space,
                    height: tf::AvailableSpace::MaxContent,
                },
                tf::SizingMode::InherentSize,
                tf::AbsoluteAxis::Horizontal,
                tf::Line::FALSE,
            )
        };
        let most = at(self, tf::AvailableSpace::MaxContent);
        let least = at(self, tf::AvailableSpace::MinContent);
        let width = most.min(room).max(least);
        let size = self.size_of(
            id(f),
            tf::Size {
                width: Some(width),
                height: None,
            },
            parent,
            tf::Size {
                width: tf::AvailableSpace::Definite(room),
                height: tf::AvailableSpace::MaxContent,
            },
            tf::SizingMode::InherentSize,
            tf::Line::FALSE,
        );
        (
            tf::Size {
                width: size.width + m.left + m.right,
                height: size.height,
            },
            m,
        )
    }

    /// Float `f` placed at or below `y` (its line box's border box).
    #[allow(clippy::too_many_arguments)]
    fn place_float(
        &mut self,
        f: usize,
        y: f32,
        inner: f32,
        ctx: &mut tf::BlockContext<'_>,
        perform: bool,
        shapes: &mut Option<Around<'_, '_>>,
        placed: &mut Vec<usize>,
    ) {
        let (outer, m) = self.float_size(f, inner);
        let width = outer.width - m.left - m.right;
        let mode = if perform {
            tf::RunMode::PerformLayout
        } else {
            tf::RunMode::ComputeSize
        };
        let out = self.lay(
            f,
            tf::LayoutInput {
                run_mode: mode,
                sizing_mode: tf::SizingMode::InherentSize,
                axis: tf::RequestedAxis::Both,
                known_dimensions: tf::Size {
                    width: Some(width),
                    height: None,
                },
                known_dimensions_are_definite: tf::Size {
                    width: true,
                    height: false,
                },
                parent_size: tf::Size {
                    width: Some(inner),
                    height: None,
                },
                available_space: tf::Size {
                    width: tf::AvailableSpace::Definite(inner),
                    height: tf::AvailableSpace::MaxContent,
                },
                vertical_margins_are_collapsible: tf::Line::FALSE,
            },
        );
        let (side, clear) = {
            use tf::BlockItemStyle;
            let v = View::item(self.nodes, f);
            (v.float(), v.clear())
        };
        let dir = match side {
            tf::Float::Right => tf::FloatDirection::Right,
            _ => tf::FloatDirection::Left,
        };
        let size = out.size;
        let margin_box = tf::Size {
            width: size.width + m.left + m.right,
            height: size.height + m.top + m.bottom,
        };
        let at = ctx.place_floated_box(margin_box, y, dir, clear, false);
        if let Some(Around::Shapes(_, known)) = shapes {
            known.push(f);
        }
        placed.push(f);
        if perform {
            let mut l = tf::Layout::with_order(0);
            l.location = tf::Point {
                x: at.x + m.left,
                y: at.y + m.top,
            };
            l.size = size;
            l.margin = m;
            self.set_unrounded_layout(id(f), &l);
        }
    }

    /// A block-level box `c` among lines, its border box's top at or below
    /// `top`: a block of its own spanning the line box, its content around
    /// the floats; one that starts a context of its own is fitted beside
    /// them (CSS 2 §9.5). Gives its margin box's bottom and its first
    /// baseline below its top.
    #[allow(clippy::too_many_arguments)]
    fn block_in_lines(
        &mut self,
        c: usize,
        top: f32,
        x0: f32,
        inner: f32,
        w: f32,
        ctx: &mut tf::BlockContext<'_>,
        input: tf::LayoutInput,
    ) -> (f32, Option<f32>) {
        use tf::{BlockItemStyle, CoreStyle};
        let perform = input.run_mode == tf::RunMode::PerformLayout;
        let m = margins(self.nodes, c, inner);
        let v = View::item(self.nodes, c);
        let clear = v.clear();
        let joins = v.is_block() && !v.contain().establishes_independent_formatting_context();
        let mut top = top;
        if let Some(b) = ctx.cleared_threshold(clear) {
            top = top.max(b - m.top);
        }
        let cw = (inner - m.left - m.right).max(0.0);
        let at = |run_mode, width: f32| tf::LayoutInput {
            run_mode,
            sizing_mode: tf::SizingMode::InherentSize,
            axis: tf::RequestedAxis::Both,
            known_dimensions: tf::Size {
                width: Some(width),
                height: None,
            },
            known_dimensions_are_definite: tf::Size {
                width: true,
                height: false,
            },
            parent_size: tf::Size {
                width: Some(inner),
                height: None,
            },
            available_space: tf::Size {
                width: tf::AvailableSpace::Definite(width),
                height: tf::AvailableSpace::MaxContent,
            },
            vertical_margins_are_collapsible: tf::Line::FALSE,
        };
        let (x, y, out) = if joins {
            let y = top + m.top;
            let x = x0 + m.left;
            let mut sub = ctx.sub_context(y, [x, (w - x - cw).max(0.0)]);
            let out = self.lay_in(id(c), at(input.run_mode, cw), Some(&mut sub));
            (x, y, out)
        } else {
            // Its own width, else the line box's; narrowed to the room
            // beside the floats down to its least, moved below them where
            // that does not fit.
            let fills = v.size().width.is_auto();
            let own = if fills {
                cw
            } else {
                self.width_of(
                    id(c),
                    tf::Size::NONE,
                    tf::Size {
                        width: Some(inner),
                        height: None,
                    },
                    tf::Size {
                        width: tf::AvailableSpace::Definite(cw),
                        height: tf::AvailableSpace::MaxContent,
                    },
                    tf::SizingMode::InherentSize,
                    tf::AbsoluteAxis::Horizontal,
                    tf::Line::FALSE,
                )
            };
            let least = if fills {
                self.width_of(
                    id(c),
                    tf::Size::NONE,
                    tf::Size {
                        width: Some(inner),
                        height: None,
                    },
                    tf::Size {
                        width: tf::AvailableSpace::MinContent,
                        height: tf::AvailableSpace::MaxContent,
                    },
                    tf::SizingMode::InherentSize,
                    tf::AbsoluteAxis::Horizontal,
                    tf::Line::FALSE,
                )
            } else {
                own
            };
            let mut q = top + m.top;
            let (x, y, width) = loop {
                let slot = ctx.find_bfc_slot(
                    q,
                    [m.left, m.right],
                    tf::Direction::Ltr,
                    tf::Clear::None,
                    None,
                );
                let Some(seg) = slot.segment_id else {
                    break (x0 + m.left, slot.y, own);
                };
                if slot.border_width >= least - 0.01 {
                    let width = if fills {
                        slot.stretch_width.min(own)
                    } else {
                        own
                    };
                    break (slot.x, slot.y, width);
                }
                let n = ctx.find_content_slot(slot.y, tf::Clear::None, Some(seg));
                let next = match n.segment_id {
                    Some(_) => n.y,
                    None => ctx.cleared_threshold(tf::Clear::Both).unwrap_or(q),
                };
                if next <= q {
                    break (x0 + m.left, q, own);
                }
                q = next;
            };
            let out = self.lay(c, at(input.run_mode, width));
            (x, y, out)
        };
        if perform {
            let mut l = tf::Layout::with_order(0);
            l.location = tf::Point { x, y };
            l.size = out.size;
            l.margin = m;
            self.set_unrounded_layout(id(c), &l);
        }
        (
            y + out.size.height + m.bottom,
            out.baselines.first.map(|b| b + y - top),
        )
    }

    /// The boxes `s..e` of node `i` laid out as one flex line `avail` wide
    /// from (`x`, `y`) in its border box: the line's height, its first
    /// baseline, and whether it is set as its line box holds a
    /// `vertical-align` (CSS 2.1 §10.8).
    #[allow(clippy::too_many_arguments)]
    fn line(
        &mut self,
        i: usize,
        s: usize,
        e: usize,
        x: f32,
        y: f32,
        avail: f32,
        mode: tf::RunMode,
    ) -> (f32, Option<f32>, bool) {
        let was = self.line.replace((i, s, e));
        let out = tf::compute_flexbox_layout(
            self,
            id(i),
            tf::LayoutInput {
                run_mode: mode,
                sizing_mode: tf::SizingMode::InherentSize,
                axis: tf::RequestedAxis::Both,
                known_dimensions: tf::Size {
                    width: Some(avail),
                    height: None,
                },
                known_dimensions_are_definite: tf::Size {
                    width: true,
                    height: false,
                },
                parent_size: tf::Size {
                    width: Some(avail),
                    height: None,
                },
                available_space: tf::Size {
                    width: tf::AvailableSpace::Definite(avail),
                    height: tf::AvailableSpace::MaxContent,
                },
                vertical_margins_are_collapsible: tf::Line::FALSE,
            },
        );
        self.line = was;
        let kids: Vec<usize> = self.nodes[i].children[s..e].to_vec();
        let perform = mode == tf::RunMode::PerformLayout;
        if perform {
            for &c in &kids {
                let l = &mut self.nodes[c].layout.css.location;
                l.x += x;
                l.y += y;
            }
        }
        let dominant = self.nodes[i].layout.used.dominant;
        if !kids.iter().any(|&c| shifted(&self.nodes[c], dominant)) {
            return (out.size.height, out.baselines.first, false);
        }
        let (height, base) = self.line_box(i, &kids, y, avail, perform);
        (height, Some(base), true)
    }

    /// A line of boxes `kids` (laid out from `y`) as its line box holds
    /// them (CSS 2.1 §10.8): as tall as its strut, its baseline-aligned
    /// boxes and its shifted boxes reach, each box placed in it. Gives its
    /// height and its baseline from its top.
    fn line_box(
        &mut self,
        i: usize,
        kids: &[usize],
        y: f32,
        avail: f32,
        perform: bool,
    ) -> (f32, f32) {
        use crate::layout::BaselineShift as S;
        let fonts = self.measure.fonts;
        let n = &self.nodes[i];
        // The parent's text: the block's first text (an anonymous inline
        // box carries the block's font), else the body step.
        let (step, font) = n
            .children
            .iter()
            .find_map(|&c| match &*self.nodes[c].content {
                Content::Text(t) => Some((self.nodes[c].used_text(t).step, t.font)),
                Content::Rich(rc) => Some(rc.base),
                _ => None,
            })
            .unwrap_or((self.measure.body, FontRole::Body));
        let m = fonts.metrics_styled(font, crate::FontQuery::new(step.weight, false), "");
        let strut = crate::layout::Strut::of(m, step.size, n.layout.used.dominant);
        let half = (step.line_height - strut.ascent - strut.descent) / 2.0;
        let mut base = (half + strut.ascent, step.line_height - half - strut.ascent);
        let mut items = Vec::with_capacity(kids.len());
        let mut tops = Vec::with_capacity(kids.len());
        for &c in kids {
            let mg = margins(self.nodes, c, avail);
            let size = if perform {
                self.nodes[c].layout.css.size
            } else {
                self.size_of(
                    id(c),
                    tf::Size::NONE,
                    tf::Size {
                        width: Some(avail),
                        height: None,
                    },
                    tf::Size {
                        width: tf::AvailableSpace::Definite(avail),
                        height: tf::AvailableSpace::MaxContent,
                    },
                    tf::SizingMode::InherentSize,
                    tf::Line::FALSE,
                )
            };
            let baselines = self
                .compute_child_layout(
                    id(c),
                    tf::LayoutInput {
                        run_mode: tf::RunMode::ComputeSize,
                        sizing_mode: tf::SizingMode::InherentSize,
                        axis: tf::RequestedAxis::Both,
                        // Its height left to it: a leaf of known size answers
                        // no baseline.
                        known_dimensions: tf::Size {
                            width: Some(size.width),
                            height: None,
                        },
                        known_dimensions_are_definite: tf::Size {
                            width: true,
                            height: false,
                        },
                        parent_size: tf::Size {
                            width: Some(avail),
                            height: None,
                        },
                        available_space: size.map(tf::AvailableSpace::Definite),
                        vertical_margins_are_collapsible: tf::Line::FALSE,
                    },
                )
                .baselines;
            let va = self.nodes[c].layout.used.valign;
            // Its baseline (the last for `baseline-source: last`, as
            // `lay_in` answers it); a box with none aligns its bottom
            // margin edge (CSS Inline 3 A.3).
            let b = baselines.first.unwrap_or(size.height);
            let (above, below) = (mg.top + b, size.height - b + mg.bottom);
            let place = match va.shift {
                S::Top => crate::layout::InlinePlace::Top,
                S::Bottom => crate::layout::InlinePlace::Bottom,
                _ => va.place(
                    &strut,
                    &og_text::baselines::Baselines::of_box(above, -below),
                    step.size,
                    step.line_height,
                    above,
                    below,
                ),
            };
            if place == crate::layout::InlinePlace::Raised(0.0) {
                base = (base.0.max(above), base.1.max(below));
            }
            items.push((place, above, below));
            tops.push(mg.top);
        }
        let (height, top, at) = crate::layout::line_box(base, &items);
        if perform {
            for ((&c, &(_, above, _)), (at, mt)) in
                kids.iter().zip(&items).zip(at.into_iter().zip(tops))
            {
                self.nodes[c].layout.css.location.y = y + at - above + mt;
            }
        }
        (height, top)
    }

    /// After context root `i` laid out: where it holds a float whose
    /// contour narrows lines, its floats' and line boxes' places, so the
    /// next pass lays its lines out beside the contours. Whether they
    /// moved since the pass before (it lays out again).
    pub(super) fn settle_shapes(&mut self, i: usize) -> bool {
        let Some(bfc) = self.bfcs.last() else {
            return false;
        };
        if bfc.root != i
            || !bfc
                .floats
                .iter()
                .any(|&f| self.nodes[f].shape_outside.is_some())
        {
            return false;
        }
        // A box's border box in the root's.
        let origin = |nodes: &[Node], mut k: usize| {
            let (mut x, mut y) = (0.0, 0.0);
            while k != i && k != NONE {
                let l = nodes[k].layout.css.location;
                (x, y) = (x + l.x, y + l.y);
                k = nodes[k].up;
            }
            [x, y]
        };
        let mut floats = Vec::new();
        for &f in &bfc.floats {
            let n = &self.nodes[f];
            let l = n.layout.css;
            let [x, y] = origin(self.nodes, f);
            let m = l.margin;
            let up = n.up;
            let cb = if up == NONE {
                0.0
            } else {
                let p = &self.nodes[up];
                (p.layout.css.size.width - p.layout.used.padding.horizontal()).max(0.0)
            };
            let margin = n.shape_outside.as_ref().map_or(0.0, |s| s.margin.at(cb));
            // Its side and contour as Taffy placed it: physical, where a
            // flipping parent's mirror no longer turns them.
            let flips = View::item(self.nodes, f).parent_flips();
            let contour = n.shape_outside.as_ref().map(|s| {
                let mut s = s.clone();
                s.rtl ^= flips;
                s.edges(
                    n,
                    [l.size.width, l.size.height],
                    [m.top, m.right, m.bottom, m.left],
                    [x, y],
                    margin,
                )
            });
            let left = {
                use tf::BlockItemStyle;
                View::item(self.nodes, f).float() != tf::Float::Right
            };
            floats.push((
                f,
                Placed {
                    x: x - m.left,
                    y: y - m.top,
                    w: l.size.width + m.left + m.right,
                    h: l.size.height + m.top + m.bottom,
                    left,
                    contour,
                    margin,
                },
            ));
        }
        let at = bfc
            .wrappers
            .iter()
            .map(|&k| (k, origin(self.nodes, k)))
            .collect();
        let now = Shaped { floats, at };
        if self.shaped.get(&i) == Some(&now) {
            return false;
        }
        self.shaped.insert(i, now);
        true
    }
}
