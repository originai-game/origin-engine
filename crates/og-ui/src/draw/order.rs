use super::{Batch, DrawCmd, DrawItem, Ground, LayerRole, mapped_bounds, union, zoom_items};
use crate::element::Affine;
use crate::layout::Rect;
use crate::world::WorldCmd;
use og_render::UiBlend;

/// One command of a level space, or a layer: its start and end in the
/// space, its own items between.
enum Entry {
    Cmd(DrawCmd, Affine),
    /// A clip bracket: drawn where it stands, never levelled.
    Bracket(DrawCmd, Affine),
    Layer {
        begin: Box<(DrawCmd, Affine)>,
        items: Vec<DrawItem>,
        end: Box<(DrawCmd, Affine)>,
        bounds: Rect,
    },
    /// A world panel's items drawn into its target, or a ground's laid
    /// onto its quad: a space of their own that levels nothing in this one.
    Panel {
        begin: Box<(DrawCmd, Affine)>,
        items: Vec<DrawItem>,
        end: Box<(DrawCmd, Affine)>,
    },
}

/// Collects one space's entries from `cmds` until its `EndLayer` (`None`:
/// the frame's end), levelling each layer's own space on the way.
fn collect(
    cmds: &mut std::vec::IntoIter<(DrawCmd, Affine)>,
    layers: &mut u32,
) -> (Vec<Entry>, Option<(DrawCmd, Affine)>) {
    let mut entries = Vec::new();
    while let Some((cmd, m)) = cmds.next() {
        match cmd {
            DrawCmd::PushClip { .. } | DrawCmd::PopClip => entries.push(Entry::Bracket(cmd, m)),
            DrawCmd::World(ref w) if matches!(**w, WorldCmd::BeginPanel { .. }) => {
                // A world panel: its own space of levels, off the screen
                // (it levels nothing there).
                let (inner, end) = collect(cmds, layers);
                let items = levelled(inner, layers, None, None);
                let end = end.unwrap_or((
                    DrawCmd::World(Box::new(WorldCmd::EndPanel)),
                    Affine::IDENTITY,
                ));
                entries.push(Entry::Panel {
                    begin: Box::new((cmd, m)),
                    items,
                    end: Box::new(end),
                });
            }
            DrawCmd::World(ref w) if matches!(**w, WorldCmd::EndPanel) => {
                return (entries, Some((cmd, m)));
            }
            DrawCmd::World(ref w) if !matches!(**w, WorldCmd::Mass { .. }) => {
                entries.push(Entry::Bracket(cmd, m))
            }
            DrawCmd::BeginLayer { .. } => {
                let (inner, end) = collect(cmds, layers);
                let mut bounds = Rect::default();
                let items = levelled(inner, layers, Some(&mut bounds), None);
                let mut begin = cmd;
                let end = end.unwrap_or((
                    DrawCmd::EndLayer {
                        geometry: None,
                        rect: bounds,
                        opacity: 1.0,
                        blend: UiBlend::Normal,
                        clip: Rect::new(-1e9, -1e9, 2e9, 2e9),
                        ground: None,
                    },
                    Affine::IDENTITY,
                ));
                // The layer's rectangle is on the screen: the union of what
                // its items draw there, whatever their maps.
                if let DrawCmd::BeginLayer {
                    rect,
                    expand,
                    filter,
                    mask,
                    ..
                } = &mut begin
                {
                    let r = match filter {
                        // The filter region, under the element's map.
                        Some(f) => {
                            crate::svg::filter::map(&mut f.1, &m);
                            f.0 = m.map_rect(&f.0);
                            f.0
                        }
                        // The box its mask is fitted to: what the mask
                        // leaves of the rest is cut.
                        None if mask.is_some() => m.map_rect(rect),
                        None => bounds.outset(*expand),
                    };
                    *rect = r;
                    bounds = r;
                }
                let image = matches!(
                    begin,
                    DrawCmd::BeginLayer {
                        role: LayerRole::Image(_),
                        ..
                    }
                );
                if image
                    || matches!(
                        &end,
                        (
                            DrawCmd::EndLayer {
                                ground: Some(Ground::Panel(_)),
                                ..
                            },
                            _,
                        )
                    )
                {
                    // Laid onto a quad that moves with the camera, in the
                    // grounds' band under the screen's levels (`emit`): it
                    // levels nothing here, so a moving camera levels nothing.
                    // An element image composites nowhere and levels
                    // nothing either.
                    entries.push(Entry::Panel {
                        begin: Box::new((begin, m)),
                        items,
                        end: Box::new(end),
                    });
                    continue;
                }
                if let (DrawCmd::EndLayer { clip, ground, .. }, _) = &end {
                    // A plane is seen anywhere its map takes it: over
                    // everything before it in its space, under what follows.
                    bounds = if matches!(ground, Some(Ground::Plane(_))) {
                        *clip
                    } else {
                        bounds.intersect(clip)
                    };
                }
                entries.push(Entry::Layer {
                    begin: Box::new((begin, m)),
                    items,
                    end: Box::new(end),
                    bounds,
                });
            }
            DrawCmd::EndLayer { .. } => return (entries, Some((cmd, m))),
            other => entries.push(Entry::Cmd(other, m)),
        }
    }
    (entries, None)
}

/// A text with visible characters (plain or styled) that measured no
/// width: its face lacks every glyph of it. A text clipped out of view
/// measured its width and is not one; it is dropped like any command that
/// draws nothing.
fn unmeasured_text(cmd: &DrawCmd) -> bool {
    match cmd {
        DrawCmd::StyledText { text, .. } => unmeasured_text(text),
        DrawCmd::Text { text, bounds, .. } => {
            bounds.is_empty() && text.chars().any(|c| !c.is_whitespace())
        }
        _ => false,
    }
}

/// Levels a space's entries and flattens them; `drawn` gets the union of
/// what they draw, and `top` what the next frame's [`Levels::update`]
/// needs.
fn levelled(
    entries: Vec<Entry>,
    layers: &mut u32,
    drawn: Option<&mut Rect>,
    top: Option<&mut Levels>,
) -> Vec<DrawItem> {
    let keyed: Vec<(Rect, Batch)> = entries
        .iter()
        .map(|e| match e {
            Entry::Cmd(c, m) => (mapped_bounds(c, m), c.batch()),
            Entry::Bracket(..) | Entry::Panel { .. } => (Rect::default(), Batch::Ui(0, None)),
            Entry::Layer { bounds, .. } => {
                *layers += 1;
                (*bounds, Batch::Composite(*layers))
            }
        })
        .collect();
    let (levels, grid, cells) = assign(&keyed);
    if let Some(u) = drawn {
        let mut acc: Option<Rect> = None;
        for (r, _) in &keyed {
            if !r.is_empty() {
                acc = Some(acc.map_or(*r, |a| union(&a, r)));
            }
        }
        *u = acc.unwrap_or_default();
    }
    let mut out = Vec::with_capacity(entries.len());
    let mut spans = Vec::with_capacity(if top.is_some() { entries.len() } else { 0 });
    for (entry, &level) in entries.into_iter().zip(&levels) {
        let first = out.len() as u32;
        match (entry, level) {
            (Entry::Bracket(cmd, transform), _) => out.push(DrawItem {
                level: 0,
                cmd,
                transform,
            }),
            (Entry::Panel { begin, items, end }, _) => {
                out.push(DrawItem {
                    level: 0,
                    cmd: begin.0,
                    transform: begin.1,
                });
                out.extend(items);
                out.push(DrawItem {
                    level: 0,
                    cmd: end.0,
                    transform: end.1,
                });
            }
            (Entry::Cmd(cmd, transform), Some(level)) => out.push(DrawItem {
                level,
                cmd,
                transform,
            }),
            // A text whose every glyph its face lacks measures no width (a
            // missing glyph takes no space), so it has no bounds. It draws
            // nothing, but it goes on so the renderer counts its missing
            // glyphs (`FrameStats::missing_glyphs`) on every frame, not only
            // when a drawn character shares its line.
            (Entry::Cmd(cmd, transform), None) if unmeasured_text(&cmd) => {
                out.push(DrawItem {
                    level: 0,
                    cmd,
                    transform,
                });
            }
            (
                Entry::Layer {
                    begin, items, end, ..
                },
                Some(level),
            ) => {
                out.push(DrawItem {
                    level,
                    cmd: begin.0,
                    transform: begin.1,
                });
                out.extend(items);
                out.push(DrawItem {
                    level,
                    cmd: end.0,
                    transform: end.1,
                });
            }
            // Nothing to draw.
            (_, None) => {}
        }
        if top.is_some() {
            let end = out.len() as u32;
            spans.push((end > first).then(|| (first, end - 1)));
        }
    }
    if let Some(top) = top {
        *top = Levels {
            at: Vec::new(),
            keyed,
            levels,
            items: spans,
            grid,
            cells,
        };
    }
    out
}

/// [`assign_levels`], with what it decided about the top level space for
/// the next frame's [`Levels::update`].
pub(crate) fn assign_levels_kept(cmds: Vec<(DrawCmd, Affine)>) -> (Vec<DrawItem>, Levels) {
    let at = top_entries(&cmds);
    let mut layers = 0;
    let mut it = cmds.into_iter();
    let mut entries = Vec::new();
    loop {
        let (more, end) = collect(&mut it, &mut layers);
        entries.extend(more);
        // A stray end closes nothing.
        if end.is_none() {
            break;
        }
    }
    let mut top = Levels::default();
    let items = levelled(entries, &mut layers, None, Some(&mut top));
    top.at = at;
    (items, top)
}

/// The levels of `keyed` (bounds and batch) in submission order: a command
/// goes above every earlier one it overlaps (on the same level when they
/// share a batch); `None` for empty bounds. Each placed rectangle is filed
/// in the cells of a coarse grid it touches, and a command tests only the
/// rectangles of its own cells: a screen transition's dissolve is 1,620
/// quads over the frame, and testing each against every earlier command was
/// 7 to 17 ms of a dev tick (Lantern Vale's playtests, the former test sample
fn assign(keyed: &[(Rect, Batch)]) -> (Vec<Option<i32>>, Grid, Vec<Vec<u32>>) {
    let bounds: Vec<Rect> = keyed.iter().map(|(r, _)| *r).collect();
    let grid = Grid::over(&bounds);
    let mut cells: Vec<Vec<u32>> = vec![Vec::new(); grid.cols * grid.rows];
    let mut out: Vec<Option<i32>> = Vec::with_capacity(keyed.len());
    for (e, &(bounds, _)) in keyed.iter().enumerate() {
        if bounds.is_empty() {
            out.push(None);
            continue;
        }
        let level = grid.level(keyed, &out, &cells, e);
        grid.file(&mut cells, &bounds, e as u32);
        out.push(Some(level));
    }
    (out, grid, cells)
}

/// What [`assign_levels`] decided about a frame's top level space: each
/// entry's bounds, batch and level, the grid they are filed in, and where
/// each submitted command and entry stand. A next frame that replaces a few
/// commands there levels again only what they overlap ([`Levels::update`]).
#[derive(Clone, Debug, Default)]
pub(crate) struct Levels {
    /// Per submitted command: its top level entry when it is one levelled
    /// command (`u32::MAX`: a bracket, a layer's or panel's, an end).
    at: Vec<u32>,
    keyed: Vec<(Rect, Batch)>,
    levels: Vec<Option<i32>>,
    /// Per entry: its first and last item (a layer's begin and end), if
    /// it has any.
    items: Vec<Option<(u32, u32)>>,
    grid: Grid,
    cells: Vec<Vec<u32>>,
}

impl Levels {
    /// Replaces the items of the commands at `changed` (submitted
    /// position, baked command and map) and levels again what they
    /// overlapped or now overlap, as [`assign_levels`] would. `false`,
    /// with nothing changed, when one is not a top level command or starts
    /// or stops drawing (the item list changes): level the frame afresh.
    pub(crate) fn update(
        &mut self,
        items: &mut [DrawItem],
        changed: Vec<(usize, DrawCmd, Affine)>,
        unit: f32,
    ) -> bool {
        let mut keys = Vec::with_capacity(changed.len());
        for (p, cmd, m) in &changed {
            let Some(&e) = self.at.get(*p).filter(|&&e| e != u32::MAX) else {
                return false;
            };
            let key = (mapped_bounds(cmd, m), cmd.batch());
            let shown = !key.0.is_empty() || unmeasured_text(cmd);
            if key.0.is_empty() != self.keyed[e as usize].0.is_empty()
                || shown != self.items[e as usize].is_some()
            {
                return false;
            }
            keys.push((e as usize, key));
        }
        let mut work = std::collections::BTreeSet::new();
        let mut moved: std::collections::BTreeMap<usize, Rect> = Default::default();
        for ((e, key), (_, cmd, transform)) in keys.into_iter().zip(changed) {
            let Some((at, _)) = self.items[e] else {
                continue;
            };
            let at = at as usize;
            items[at] = zoom_items(
                vec![DrawItem {
                    level: items[at].level,
                    cmd,
                    transform,
                }],
                unit,
            )
            .pop()
            .expect("one item");
            // Drawing nothing (a text of missing glyphs): on level 0, levelling nothing.
            if key.0.is_empty() {
                continue;
            }
            let old = self.keyed[e];
            if old != key {
                self.grid.unfile(&mut self.cells, &old.0, e as u32);
                self.grid.file(&mut self.cells, &key.0, e as u32);
                self.keyed[e] = key;
                moved.entry(e).or_insert(old.0);
            }
            work.insert(e);
        }
        // In submission order: an entry is levelled after every earlier
        // one it reads, and what it overlaps is levelled after it.
        while let Some(e) = work.pop_first() {
            let level = self.grid.level(&self.keyed, &self.levels, &self.cells, e);
            let old = moved.get(&e).copied();
            if self.levels[e] == Some(level) && old.is_none() {
                continue;
            }
            self.levels[e] = Some(level);
            let (first, last) = self.items[e].expect("a levelled entry has items");
            items[first as usize].level = level;
            items[last as usize].level = level;
            for r in std::iter::once(self.keyed[e].0).chain(old) {
                let (cols, rows) = self.grid.span(&r);
                for row in rows {
                    for col in cols.clone() {
                        for &q in &self.cells[row * self.grid.cols + col] {
                            if q as usize > e && self.keyed[q as usize].0.overlaps(&r) {
                                work.insert(q as usize);
                            }
                        }
                    }
                }
            }
        }
        true
    }

    /// The next frame's items when its commands are last frame's moved,
    /// dropped and joined by new ones ([`Spliced`], in submission order),
    /// as [`assign_levels`] would level them. A command that moved with
    /// the others by one offset (a scrolled row) keeps its level: what
    /// overlapped among them still does, exactly, and so does what covers
    /// all of them or none (`kept`). What is new, what was clipped
    /// otherwise, what lies across the moved commands' edge and what a
    /// dropped command overlapped are levelled again, with what follows
    /// them. `None` for a frame with a layer or world panel, or whose
    /// commands do not keep last frame's order: level it afresh.
    pub(crate) fn splice(
        &mut self,
        old: Vec<DrawItem>,
        next: Vec<Spliced>,
        unit: f32,
    ) -> Option<Vec<DrawItem>> {
        // One entry per command: no layer or panel gathers several.
        if self.keyed.len() != self.at.len() {
            return None;
        }
        let gone = (Rect::default(), Batch::Ui(0, None));
        let mut old: Vec<Option<DrawItem>> = old.into_iter().map(Some).collect();
        let n = next.len();
        let mut keyed = Vec::with_capacity(n);
        let mut levels = Vec::with_capacity(n);
        let mut at = Vec::with_capacity(n);
        let mut out: Vec<Option<DrawItem>> = Vec::with_capacity(n);
        // Per entry: last frame's, when its overlaps there hold here.
        let mut kept: Vec<Option<u32>> = Vec::with_capacity(n);
        let mut moved: Vec<bool> = Vec::with_capacity(n);
        let mut carried = vec![u32::MAX; self.at.len()];
        let mut last = None::<usize>;
        let mut offset = None::<(f32, f32)>;
        // The moved commands' extent, over where they were and are.
        let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for (e, s) in next.into_iter().enumerate() {
            if let Some(o) = s.from {
                if last.is_some_and(|l| o <= l) || o >= self.at.len() {
                    return None;
                }
                last = Some(o);
            }
            let Some((cmd, m)) = s.cmd else {
                // Last frame's item as it stood.
                let o = s.from?;
                if s.delta != (0.0, 0.0) {
                    return None;
                }
                keyed.push(self.keyed[o]);
                levels.push(self.levels[o]);
                at.push(if self.at[o] == u32::MAX {
                    u32::MAX
                } else {
                    e as u32
                });
                out.push(self.items[o].and_then(|(i, _)| old[i as usize].take()));
                kept.push(Some(o as u32));
                moved.push(false);
                carried[o] = e as u32;
                continue;
            };
            let bracket = match &cmd {
                DrawCmd::BeginLayer { .. } | DrawCmd::EndLayer { .. } => return None,
                DrawCmd::World(w) => match &**w {
                    WorldCmd::BeginPanel { .. } | WorldCmd::EndPanel => return None,
                    WorldCmd::Mass { .. } => false,
                    _ => true,
                },
                DrawCmd::PushClip { .. } | DrawCmd::PopClip => true,
                _ => false,
            };
            let key = if bracket {
                gone
            } else {
                (mapped_bounds(&cmd, &m), cmd.batch())
            };
            let shown = bracket || !key.0.is_empty() || unmeasured_text(&cmd);
            let item = shown.then(|| {
                zoom_items(
                    vec![DrawItem {
                        level: 0,
                        cmd,
                        transform: m,
                    }],
                    unit,
                )
                .pop()
                .expect("one item")
            });
            // Moved as one with the others, exactly (`exact`).
            let same = s.from.filter(|&o| {
                let was = self.keyed[o];
                (self.at[o] == u32::MAX) == bracket
                    && was.1 == key.1
                    && (bracket
                        || (was.0.is_empty() && key.0.is_empty())
                        || (s.delta == (0.0, 0.0) && was.0 == key.0)
                        || (offset.is_none_or(|d| d == s.delta) && exact(&was.0, &key.0, s.delta)))
            });
            if let Some(o) = same {
                carried[o] = e as u32;
                let shifted = !bracket && s.delta != (0.0, 0.0) && !key.0.is_empty();
                if shifted {
                    offset = Some(s.delta);
                    let was = self.keyed[o].0;
                    for r in [was, key.0] {
                        (x0, y0) = (x0.min(r.x), y0.min(r.y));
                        (x1, y1) = (x1.max(r.right()), y1.max(r.bottom()));
                    }
                }
                moved.push(shifted);
                levels.push(self.levels[o]);
            } else {
                moved.push(false);
                levels.push(None);
            }
            kept.push(same.map(|o| o as u32));
            keyed.push(key);
            at.push(if bracket { u32::MAX } else { e as u32 });
            out.push(item);
        }
        let bounds: Vec<Rect> = keyed.iter().map(|(r, _)| *r).collect();
        let grid = Grid::over(&bounds);
        let mut cells: Vec<Vec<u32>> = vec![Vec::new(); grid.cols * grid.rows];
        for (e, r) in bounds.iter().enumerate() {
            if !r.is_empty() {
                grid.file(&mut cells, r, e as u32);
            }
        }
        let near = |grid: &Grid, cells: &[Vec<u32>], r: &Rect| {
            let (cols, rows) = grid.span(r);
            rows.flat_map(move |row| cols.clone().map(move |col| row * grid.cols + col))
                .flat_map(|c| cells[c].iter().copied())
                .collect::<Vec<u32>>()
        };
        let mut work = std::collections::BTreeSet::new();
        // What is new or changed: levelled, and what follows it after.
        for e in 0..n {
            if kept[e].is_none() && !keyed[e].0.is_empty() {
                work.insert(e);
            }
        }
        // What a dropped or changed command overlapped, where it was.
        for (o, &now) in carried.iter().enumerate() {
            let r = self.keyed[o].0;
            if now != u32::MAX && kept[now as usize].is_some() || r.is_empty() {
                continue;
            }
            for q in near(&self.grid, &self.cells, &r) {
                let q = q as usize;
                if q > o && carried[q] != u32::MAX && self.keyed[q].0.overlaps(&r) {
                    work.insert(carried[q] as usize);
                }
            }
        }
        // A kept command partly over the moved ones may overlap others of
        // them than last frame; one over all or none of them overlaps the
        // same ones.
        if offset.is_some() {
            for s in 0..n {
                let r = keyed[s].0;
                if moved[s] || kept[s].is_none() || r.is_empty() {
                    continue;
                }
                let all = r.x <= x0 && r.y <= y0 && r.right() >= x1 && r.bottom() >= y1;
                let none = r.right() <= x0 || x1 <= r.x || r.bottom() <= y0 || y1 <= r.y;
                if all || none {
                    continue;
                }
                work.insert(s);
                for q in near(&grid, &cells, &r) {
                    let q = q as usize;
                    if q > s && moved[q] && keyed[q].0.overlaps(&r) {
                        work.insert(q);
                    }
                }
                let o = kept[s].expect("kept") as usize;
                for q in near(&self.grid, &self.cells, &r) {
                    let q = q as usize;
                    let now = carried[q] as usize;
                    if q > o && carried[q] != u32::MAX && moved[now] && self.keyed[q].0.overlaps(&r)
                    {
                        work.insert(now);
                    }
                }
            }
        }
        while let Some(e) = work.pop_first() {
            let level = Some(grid.level(&keyed, &levels, &cells, e));
            if levels[e] == level && kept[e].is_some() {
                continue;
            }
            levels[e] = level;
            for q in near(&grid, &cells, &keyed[e].0) {
                if q as usize > e && keyed[q as usize].0.overlaps(&keyed[e].0) {
                    work.insert(q as usize);
                }
            }
        }
        let mut items = Vec::with_capacity(n);
        let mut spans = Vec::with_capacity(n);
        for (e, item) in out.into_iter().enumerate() {
            spans.push(item.map(|mut item| {
                item.level = levels[e].unwrap_or(0);
                items.push(item);
                (items.len() as u32 - 1, items.len() as u32 - 1)
            }));
        }
        *self = Levels {
            at,
            keyed,
            levels,
            items: spans,
            grid,
            cells,
        };
        Some(items)
    }
}

/// One command of the frame [`Levels::splice`] levels: last frame's
/// command `from`, moved by `delta`, or a new one; `cmd`, baked, unless it
/// is last frame's item as it stood.
pub(crate) struct Spliced {
    pub from: Option<usize>,
    pub delta: (f32, f32),
    pub cmd: Option<(DrawCmd, Affine)>,
}

/// `now` is `was` moved by `d`, and every sum of their edges and `d` is
/// exact in `f32` (multiples of 1/256 px within 32,768): what overlapped
/// before still overlaps, edge for edge, and nothing else does.
fn exact(was: &Rect, now: &Rect, d: (f32, f32)) -> bool {
    let fine = |v: f32| v.abs() < 32768.0 && (v * 256.0).fract() == 0.0;
    [was.x, was.y, was.width, was.height, d.0, d.1]
        .into_iter()
        .all(fine)
        && *now == Rect::new(was.x + d.0, was.y + d.1, was.width, was.height)
}

/// Per submitted command, its top level entry when [`collect`] makes it
/// one levelled command ([`Levels::at`]); the entries counted as
/// [`assign_levels`] makes them (a stray end makes none).
fn top_entries(cmds: &[(DrawCmd, Affine)]) -> Vec<u32> {
    let mut depth = 0u32;
    let mut entry = 0u32;
    cmds.iter()
        .map(|(c, _)| {
            let (opens, closes, plain) = match c {
                DrawCmd::BeginLayer { .. } => (true, false, false),
                DrawCmd::EndLayer { .. } => (false, true, false),
                DrawCmd::PushClip { .. } | DrawCmd::PopClip => (false, false, false),
                DrawCmd::World(w) => match &**w {
                    WorldCmd::BeginPanel { .. } => (true, false, false),
                    WorldCmd::EndPanel => (false, true, false),
                    WorldCmd::Mass { .. } => (false, false, true),
                    _ => (false, false, false),
                },
                _ => (false, false, true),
            };
            let at = if depth == 0 && !closes {
                entry += 1;
                if plain { entry - 1 } else { u32::MAX }
            } else {
                u32::MAX
            };
            if opens {
                depth += 1;
            } else if closes {
                depth = depth.saturating_sub(1);
            }
            at
        })
        .collect()
}

/// [`assign`]'s grid: about one cell for every four commands, at
/// most 64 by 64 cells of at least 32 CSS px, over the commands' bounds. A rectangle is filed in every cell its closed
/// extent touches (clamped to the grid), so two rectangles that overlap
/// share a cell.
#[derive(Clone, Debug, Default)]
struct Grid {
    x: f32,
    y: f32,
    cell_w: f32,
    cell_h: f32,
    cols: usize,
    rows: usize,
}

impl Grid {
    fn over(bounds: &[Rect]) -> Grid {
        let (mut x0, mut y0) = (f32::MAX, f32::MAX);
        let (mut x1, mut y1) = (f32::MIN, f32::MIN);
        for r in bounds.iter().filter(|r| !r.is_empty()) {
            if r.x.is_finite() && r.right().is_finite() {
                x0 = x0.min(r.x);
                x1 = x1.max(r.right());
            }
            if r.y.is_finite() && r.bottom().is_finite() {
                y0 = y0.min(r.y);
                y1 = y1.max(r.bottom());
            }
        }
        // About one cell for every four commands, each at least 32 CSS px
        // across: a grid of the frame's 920 cells cost more to fill (a
        // backdrop is filed in every cell) than a HUD of a few dozen
        // commands cost to test pair by pair (the former test sample
        let cells = (bounds.len() / 4).clamp(1, 64 * 64) as f32;
        let area = ((x1 - x0).max(1.0) * (y1 - y0).max(1.0)).max(1.0);
        let side = (area / cells).sqrt().max(32.0);
        let axis = |lo: f32, hi: f32| {
            if hi <= lo {
                return (0.0, 1.0, 1);
            }
            let n = ((hi - lo) / side).ceil().clamp(1.0, 64.0);
            (lo, (hi - lo) / n, n as usize)
        };
        let (x, cell_w, cols) = axis(x0, x1);
        let (y, cell_h, rows) = axis(y0, y1);
        Grid {
            x,
            y,
            cell_w,
            cell_h,
            cols,
            rows,
        }
    }

    /// Entry `e`'s level: above every earlier entry it overlaps, on the
    /// same level as one of its own batch.
    fn level(
        &self,
        keyed: &[(Rect, Batch)],
        levels: &[Option<i32>],
        cells: &[Vec<u32>],
        e: usize,
    ) -> i32 {
        let (bounds, batch) = keyed[e];
        let (cols, rows) = self.span(&bounds);
        let mut level = 0;
        for row in rows {
            for col in cols.clone() {
                for &p in &cells[row * self.cols + col] {
                    let p = p as usize;
                    if p < e
                        && let Some(l) = levels[p]
                        && keyed[p].0.overlaps(&bounds)
                    {
                        level = level.max(if keyed[p].1 == batch { l } else { l + 1 });
                    }
                }
            }
        }
        level
    }

    /// Files entry `e` in the cells `r` touches.
    fn file(&self, cells: &mut [Vec<u32>], r: &Rect, e: u32) {
        let (cols, rows) = self.span(r);
        for row in rows {
            for col in cols.clone() {
                cells[row * self.cols + col].push(e);
            }
        }
    }

    /// Takes entry `e` out of the cells `r` touches.
    fn unfile(&self, cells: &mut [Vec<u32>], r: &Rect, e: u32) {
        if r.is_empty() {
            return;
        }
        let (cols, rows) = self.span(r);
        for row in rows {
            for col in cols.clone() {
                cells[row * self.cols + col].retain(|&q| q != e);
            }
        }
    }

    /// The columns and rows `r` touches.
    fn span(
        &self,
        r: &Rect,
    ) -> (
        std::ops::RangeInclusive<usize>,
        std::ops::RangeInclusive<usize>,
    ) {
        // `as` saturates (NaN is 0), and the clamp keeps every index in the grid.
        let at = |v: f32, o: f32, size: f32, n: usize| {
            (((v - o) / size).floor() as i64).clamp(0, n as i64 - 1) as usize
        };
        (
            at(r.x, self.x, self.cell_w, self.cols)..=at(r.right(), self.x, self.cell_w, self.cols),
            at(r.y, self.y, self.cell_h, self.rows)
                ..=at(r.bottom(), self.y, self.cell_h, self.rows),
        )
    }
}
