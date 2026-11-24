//! World-space and UI-node anchors, edge clamping, decluttering, and
//! tutorial spotlight cutouts.

use super::paint::{affine, radii};
use super::{El, NONE, Tree};

/// What an anchored node sits on (CONVENTIONS.md §World).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnchorTarget {
    /// An entity, by its bits.
    Entity(u64),
    Point([f32; 3]),
    /// A UI node, by its `name` (an index into the tree's names): the
    /// tree places it from that node's box after layout; the host is
    /// never asked.
    Node(u32),
}

/// What the host sees of an anchored node's target ([`Tree::place_anchors`]).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Sighting {
    /// Its surface point, CSS px, which may be off the surface; behind the
    /// camera, a point well off the surface the way the camera would turn
    /// to face it (what an edge-clamped node pins toward).
    pub at: [f32; 2],
    /// Its distance along the view axis: the nearer of two decluttering
    /// nodes of one priority keeps its place.
    pub depth: f32,
    pub behind: bool,
    /// Geometry stands between the eye and it. Asked only at the tick, of
    /// a node with `occlusion`.
    pub occluded: bool,
}

/// What an anchored node does behind geometry (`occlusion`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Occlusion {
    #[default]
    None,
    /// Drawn at [`OCCLUDED_ALPHA`], still pressable.
    Fade,
    /// Hidden as off the screen is.
    Hide,
}

/// What an anchored node does over another (`declutter`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Declutter {
    #[default]
    None,
    /// Moved straight up clear of what it overlaps (nameplates over a crowd).
    Stack,
    /// Hidden while it overlaps.
    Hide,
}

/// An occluded node's opacity under `occlusion="fade"` (Unreal's and the
/// old core's dimmed-through-walls nameplate).
pub(crate) const OCCLUDED_ALPHA: f32 = 0.35;

/// `clampMargin`'s default, CSS px (the old core's).
pub(crate) const CLAMP_MARGIN: f32 = 24.0;

/// The space two decluttered nodes keep between them, CSS px.
const DECLUTTER_GAP: f32 = 2.0;

/// An anchored node: its target, world offset and placement props as its
/// props set them, what the host last saw of its target, and its top left
/// on the surface and whether it shows as the tree last settled it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Anchor {
    /// `None`: in flow, its placement props kept for when it is anchored.
    pub(crate) target: Option<AnchorTarget>,
    pub(crate) lift: [f32; 3],
    pub(crate) occlusion: Occlusion,
    pub(crate) declutter: Declutter,
    /// Higher keeps its place over lower when decluttering.
    pub(crate) priority: f32,
    /// `clamp="edge"`: pinned inside the surface off it.
    pub(crate) clamp: bool,
    /// `clampMargin`: how far inside the surface's edge, CSS px.
    pub(crate) margin: f32,
    /// `scaleAt`: world distance where scale is 1.0 (0: disabled).
    pub(crate) scale_at: f32,
    /// `scaleMin` / `scaleMax`: distance-scaling clamp bounds.
    pub(crate) scale_min: f32,
    pub(crate) scale_max: f32,
    /// `maxDistance`: hide past this view-axis depth in world units (0: unlimited).
    pub(crate) max_distance: f32,
    /// `lod`: `[near, far]` thresholds in world units for `data-lod-{near,mid,far}`.
    pub(crate) lod: [f32; 2],
    /// The last sighting, `None` when there was none (no camera, a dead
    /// entity).
    pub(crate) seen: Option<Sighting>,
    /// Pinned to the surface's edge: its side (0 top, 1 right, 2 bottom,
    /// 3 left).
    pub(crate) edge: Option<u8>,
    /// Behind geometry at the last tick.
    pub(crate) occluded: bool,
    pub(crate) at: [f32; 2],
    pub(crate) shown: bool,
    /// Shown where the tick's end placed it ([`Tree::place_anchors`]).
    pub(crate) ticked: bool,
    /// Last distance scale applied at settle/paint, to detect scale-only changes.
    pub(crate) last_scale: f32,
}

impl El {
    /// Its anchor when it is out of flow on a target.
    pub(super) fn anchored(&self) -> Option<&Anchor> {
        self.anchor.as_deref().filter(|a| a.target.is_some())
    }
}

impl Anchor {
    pub(crate) fn new() -> Anchor {
        Anchor {
            target: None,
            lift: [0.0; 3],
            occlusion: Occlusion::None,
            declutter: Declutter::None,
            priority: 0.0,
            clamp: false,
            margin: CLAMP_MARGIN,
            scale_at: 0.0,
            scale_min: 0.5,
            scale_max: 1.5,
            max_distance: 0.0,
            lod: [0.0; 2],
            seen: None,
            edge: None,
            occluded: false,
            at: [0.0; 2],
            shown: false,
            ticked: false,
            last_scale: 1.0,
        }
    }

    /// Its opacity over what it inherits.
    pub(crate) fn alpha(&self) -> f32 {
        if self.occluded && self.occlusion == Occlusion::Fade {
            OCCLUDED_ALPHA
        } else {
            1.0
        }
    }

    /// Distance-based scale factor when `scale_at > 0` and a positive depth is known.
    pub(crate) fn dist_scale(&self) -> f32 {
        if self.scale_at > 0.0
            && let Some(s) = self.seen
            && s.depth > 1e-4
        {
            let lo = self.scale_min.max(0.01);
            let hi = self.scale_max.max(lo);
            (self.scale_at / s.depth).clamp(lo, hi)
        } else {
            1.0
        }
    }
}

impl Tree {
    /// Places each anchored node (CONVENTIONS.md §World): `project` gives
    /// what the camera sees of its target lifted by its world offset, and
    /// whether geometry stands in front of it when the last argument asks
    /// (a node with `occlusion`); `None` (no camera, a dead entity) hides
    /// it, its subtree and its accessibility nodes, as a target off the
    /// screen or behind the camera does unless it is edge-clamped. Where
    /// it lands is settled after layout ([`Tree::present`]): clamped to
    /// the edge, then decluttered. Allocates nothing unless a node's
    /// `data-clamped`, `data-edge-*` or `data-occluded` changes.
    pub fn place_anchors(
        &mut self,
        project: impl FnMut(AnchorTarget, [f32; 3], bool) -> Option<Sighting>,
    ) {
        self.place(project, true);
    }

    /// [`Tree::place_anchors`] on a frame drawn between ticks, from the
    /// drawn camera and the targets' drawn places, so an anchored node
    /// stays on what it is anchored to as that moves. Only the nodes the
    /// tick's end showed move: a dead entity's stays hidden, and what
    /// occludes them is the tick's (it is never asked).
    pub fn follow_anchors(
        &mut self,
        project: impl FnMut(AnchorTarget, [f32; 3], bool) -> Option<Sighting>,
    ) {
        self.place(project, false);
    }

    fn place(
        &mut self,
        mut project: impl FnMut(AnchorTarget, [f32; 3], bool) -> Option<Sighting>,
        tick: bool,
    ) {
        let els = &self.els;
        self.anchored
            .retain(|&e| !els[e as usize].dead && els[e as usize].anchored().is_some());
        let (w, h) = self.size;
        for k in 0..self.anchored.len() {
            let e = self.anchored[k];
            let el = &self.els[e as usize];
            let a = *el.anchor.as_deref().expect("an anchored node");
            if !tick && !a.ticked {
                continue;
            }
            let target = a.target.expect("an anchored node");
            if matches!(target, AnchorTarget::Node(_)) {
                continue;
            }
            let ask = tick && a.occlusion != Occlusion::None;
            let seen = project(target, a.lift, ask);
            let in_range = seen.is_some_and(|s| a.max_distance <= 0.0 || s.depth <= a.max_distance);
            let seen = seen.filter(|_| in_range);
            // The last layout's drawn box (none before the first), for
            // whether it stays inside the clamp's margin; the settle
            // places it with this frame's.
            let drawn = if a.clamp && self.paints > 0 {
                self.anchor_box(e)
            } else {
                [0.0; 4]
            };
            let on = seen.is_some_and(|s| {
                !s.behind && s.at[0] >= 0.0 && s.at[1] >= 0.0 && s.at[0] <= w && s.at[1] <= h
            });
            let edge = seen
                .filter(|_| a.clamp)
                .and_then(|s| clamp_edge(s, drawn, a.margin, w, h));
            let occluded = if tick {
                ask && edge.is_none() && seen.is_some_and(|s| s.occluded)
            } else {
                a.occluded
            };
            let visible_seen = seen.filter(|_| on || edge.is_some());
            let lod = a.lod;
            let (lod_near, lod_mid, lod_far) = match visible_seen {
                Some(s) if lod[0] > 0.0 && lod[1] > lod[0] => (
                    s.depth <= lod[0],
                    s.depth > lod[0] && s.depth <= lod[1],
                    s.depth > lod[1],
                ),
                Some(s) if lod[0] > 0.0 => (s.depth <= lod[0], false, s.depth > lod[0]),
                Some(s) if lod[1] > 0.0 => (false, s.depth <= lod[1], s.depth > lod[1]),
                _ => (false, false, false),
            };
            let a = self.els[e as usize].anchor.as_mut().expect("anchored");
            a.seen = visible_seen;
            a.edge = edge;
            a.occluded = occluded;
            if tick {
                a.ticked = a.seen.is_some();
            }
            self.mark(e, "clamped", edge.is_some());
            for (side, name) in ["edge-top", "edge-right", "edge-bottom", "edge-left"]
                .into_iter()
                .enumerate()
            {
                self.mark(e, name, edge == Some(side as u8));
            }
            self.mark(e, "occluded", occluded);
            self.mark(e, "lod-near", lod_near);
            self.mark(e, "lod-mid", lod_mid);
            self.mark(e, "lod-far", lod_far);
        }
        self.refresh_data();
    }

    /// Turns `e`'s engine-set `data-<name>` on or off.
    pub(super) fn mark(&mut self, e: u32, name: &str, on: bool) {
        let attrs = &mut self.els[e as usize].attrs;
        match (attrs.binary_search_by(|a| (**a).cmp(name)), on) {
            (Err(at), true) => attrs.insert(at, name.into()),
            (Ok(at), false) => drop(attrs.remove(at)),
            _ => return,
        }
        self.data_dirty = true;
    }

    /// Anchored `e`'s drawn box from its top left, as its transform
    /// (`-translate-x-1/2`, a scale) moves it: x and y offsets, width and
    /// height, CSS px.
    pub(super) fn anchor_box(&self, e: u32) -> [f32; 4] {
        let el = &self.els[e as usize];
        let b = self
            .motion
            .box_of(el.node)
            .unwrap_or_else(|| self.arena.box_of(el.node));
        let mut t = self.motion.skin(el.node, &el.record.skin).transform;
        if let Some(a) = el.anchored() {
            let ds = a.dist_scale();
            t.scale = [t.scale[0] * ds, t.scale[1] * ds];
        }
        let [a, b2, c, d, tx, ty] = affine(&t, [0.0, 0.0, b.w, b.h]);
        let mut r = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
        for [x, y] in [[0.0, 0.0], [b.w, 0.0], [0.0, b.h], [b.w, b.h]] {
            let (sx, sy) = (a * x + c * y + tx, b2 * x + d * y + ty);
            r = [r[0].min(sx), r[1].min(sy), r[2].max(sx), r[3].max(sy)];
        }
        [r[0], r[1], r[2] - r[0], r[3] - r[1]]
    }

    /// Where each anchored node lands this frame, after layout: an
    /// edge-clamped one pinned inside the margin on the line from the
    /// surface's centre toward its target, the decluttering ones taken by
    /// priority (higher first), then depth (nearer first), each moved up
    /// clear of those placed before it or hidden (MapLibre's collision
    /// pass, one direction). Repaints and moves the accessibility nodes
    /// of what changed.
    pub(super) fn settle_anchors(&mut self) {
        // A node anchored to a UI node sits where that node's box is now,
        // `lift` its x and y as fractions of that box's size.
        for k in 0..self.anchored.len() {
            let e = self.anchored[k];
            let Some(&Anchor {
                target: Some(AnchorTarget::Node(n)),
                lift,
                ..
            }) = self.els[e as usize].anchored()
            else {
                continue;
            };
            let seen = self
                .named(&self.names[n as usize])
                .filter(|&t| t != e)
                .and_then(|t| self.laid_box(t))
                .map(|[x, y, bw, bh]| Sighting {
                    at: [x + lift[0] * bw, y + lift[1] * bh],
                    ..Sighting::default()
                });
            let a = self.els[e as usize].anchor.as_mut().expect("anchored");
            (a.seen, a.edge, a.occluded, a.ticked) = (seen, None, false, seen.is_some());
        }
        let (w, h) = self.size;
        let mut order = std::mem::take(&mut self.anchor_order);
        let mut placed = std::mem::take(&mut self.anchor_placed);
        order.clear();
        placed.clear();
        order.extend(self.anchored.iter().copied().filter(|&e| {
            let el = &self.els[e as usize];
            !el.dead && el.anchored().is_some()
        }));
        let key = |e: u32| {
            let a = self.els[e as usize].anchor.as_deref().expect("anchored");
            (a.priority, a.seen.map_or(f32::MAX, |s| s.depth))
        };
        order.sort_unstable_by(|&x, &y| {
            let ((px, dx), (py, dy)) = (key(x), key(y));
            py.total_cmp(&px).then(dx.total_cmp(&dy)).then(x.cmp(&y))
        });
        for &e in &order {
            let [ox, oy, bw, bh] = self.anchor_box(e);
            let a = *self.els[e as usize].anchor.as_deref().expect("anchored");
            let mut shown = a.seen.is_some() && !(a.occluded && a.occlusion == Occlusion::Hide);
            let mut at = a.seen.map_or(a.at, |s| s.at);
            if let Some(s) = a.seen.filter(|_| a.edge.is_some()) {
                at = clamped(s, [ox, oy, bw, bh], a.margin, w, h);
            } else if shown && a.clamp && matches!(a.target, Some(AnchorTarget::Node(_))) {
                // A popup on a UI node: its drawn box shifts along each
                // axis to stay inside the surface less the margin
                // (Floating UI's `shift`); one wider than that keeps its
                // start at the margin.
                let shift =
                    |x: f32, size: f32, extent: f32| x.min(extent - a.margin - size).max(a.margin);
                at = [shift(at[0] + ox, bw, w) - ox, shift(at[1] + oy, bh, h) - oy];
            } else if shown && a.declutter != Declutter::None {
                let mut r = [at[0] + ox, at[1] + oy, bw, bh];
                for _ in 0..=placed.len() {
                    let Some(p) = placed.iter().find(|p| overlaps(p, &r, DECLUTTER_GAP)) else {
                        break;
                    };
                    if a.declutter == Declutter::Hide {
                        shown = false;
                        break;
                    }
                    r[1] = p[1] - DECLUTTER_GAP - bh;
                }
                // Nothing clear on the screen: hidden.
                if r[1] < 0.0 || placed.iter().any(|p| overlaps(p, &r, DECLUTTER_GAP)) {
                    shown = false;
                }
                if shown {
                    at[1] = r[1] - oy;
                    placed.push(r);
                }
            }
            let el = &mut self.els[e as usize];
            let a = el.anchor.as_deref_mut().expect("anchored");
            let ds = a.dist_scale();
            if shown != a.shown || at != a.at || (ds - a.last_scale).abs() > 1e-4 {
                (a.shown, a.at, a.last_scale) = (shown, at, ds);
                self.sem.place(el.node, shown.then_some(at));
                self.paint = true;
            }
        }
        self.anchor_order = order;
        self.anchor_placed = placed;
    }

    /// The anchored nodes' world targets, for the host to ask which
    /// entities they need (a node anchored to a UI node is the tree's).
    pub fn anchor_targets(&self) -> impl Iterator<Item = AnchorTarget> + '_ {
        self.anchored
            .iter()
            .filter_map(|&e| self.els[e as usize].anchored().and_then(|a| a.target))
            .filter(|t| !matches!(t, AnchorTarget::Node(_)))
    }

    /// The index of `name` among the names node anchors target.
    pub(crate) fn name_index(&mut self, name: &str) -> u32 {
        match self.names.iter().position(|n| **n == *name) {
            Some(i) => i as u32,
            None => {
                self.names.push(name.into());
                self.names.len() as u32 - 1
            }
        }
    }

    /// The live node `name` names: the last made of those that carry it.
    pub(super) fn named(&self, name: &str) -> Option<u32> {
        if name.is_empty() {
            return None;
        }
        (0..self.els.len().min(self.input.per.len()))
            .rev()
            .find(|&e| !self.els[e].dead && *self.input.per[e].name == *name)
            .map(|e| e as u32)
    }

    /// `e`'s box on the surface as laid out now (x, y, width, height: its
    /// layout transition's box, its parents' scrolls, an anchored
    /// ancestor's place; not its transform), `None` while it or an
    /// ancestor does not show.
    pub(super) fn laid_box(&self, e: u32) -> Option<[f32; 4]> {
        let size = |c: u32| {
            let n = self.els[c as usize].node;
            self.motion
                .box_of(n)
                .unwrap_or_else(|| self.arena.box_of(n))
        };
        let own = size(e);
        let (mut x, mut y) = (0.0, 0.0);
        let mut c = e;
        loop {
            let el = &self.els[c as usize];
            if el.dead
                || el.hidden > 0
                || el.record.hidden
                || el.record.layout.display() == taffy::Display::None
            {
                return None;
            }
            if let Some(a) = el.anchored() {
                a.shown.then_some(())?;
                (x, y) = (x + a.at[0], y + a.at[1]);
                break;
            }
            let b = size(c);
            (x, y) = (x + b.x, y + b.y);
            if el.arena_parent == NONE {
                break;
            }
            let [sx, sy] = self.moved_by(c);
            (x, y) = (x - sx, y - sy);
            c = el.arena_parent;
        }
        Some([x, y, own.w, own.h])
    }

    /// Turns each spotlight's `data-lit` on while the node it names shows,
    /// off otherwise: whether any changed.
    pub(super) fn mark_lit(&mut self) -> bool {
        let (els, per) = (&self.els, &self.input.per);
        self.spotlights.retain(|&e| {
            !els[e as usize].dead && per.get(e as usize).is_some_and(|p| !p.spotlight.is_empty())
        });
        let was = self.data_dirty;
        self.data_dirty = false;
        for k in 0..self.spotlights.len() {
            let e = self.spotlights[k];
            let lit = self.hole(e).is_some();
            self.mark(e, "lit", lit);
        }
        let changed = self.data_dirty;
        self.data_dirty |= was;
        changed
    }

    /// The box spotlight `e` lights (its `spotlight` node's), when it
    /// names one that shows, and that node's corner radii: the hole is its
    /// border box, rounded as it is.
    pub(super) fn hole(&self, e: u32) -> Option<([f32; 4], [f32; 4])> {
        let name = &self.input.per.get(e as usize)?.spotlight;
        let t = self.named(name).filter(|&t| t != e)?;
        let b = self.laid_box(t)?;
        let node = self.els[t as usize].node;
        let skin = self.motion.skin(node, &self.els[t as usize].record.skin);
        Some((b, radii(skin.radius, b[2], b[3])))
    }

    /// Whether (x, y) is in spotlight `e`'s hole, the rounded box its
    /// shade is drawn around (a press in a corner outside the arc is the
    /// shade's); `None` when it lights nothing.
    pub(super) fn in_hole(&self, e: u32, x: f32, y: f32) -> Option<bool> {
        let ([hx, hy, hw, hh], r) = self.hole(e)?;
        let at = [x - hx - hw / 2.0, y - hy - hh / 2.0];
        Some(og_draw::paint::round_box_distance(at, [hw / 2.0, hh / 2.0], r) < 0.0)
    }
}

/// Whether `a` and `b` (x, y, w, h) come nearer than `gap`.
fn overlaps(a: &[f32; 4], b: &[f32; 4], gap: f32) -> bool {
    a[0] < b[0] + b[2] + gap
        && b[0] < a[0] + a[2] + gap
        && a[1] < b[1] + b[3] + gap
        && b[1] < a[1] + a[3] + gap
}

/// An edge-clamped node's box centre as its sighting puts it, the screen
/// centre, the line between them and how far along it the centre may go
/// (the half room inside the margin on each axis over the line's run).
fn clamp_line(
    s: Sighting,
    drawn: [f32; 4],
    m: f32,
    w: f32,
    h: f32,
) -> ([f32; 2], [f32; 2], f32, f32) {
    let [ox, oy, bw, bh] = drawn;
    let c = [w / 2.0, h / 2.0];
    let mut d = [
        s.at[0] + ox + bw / 2.0 - c[0],
        s.at[1] + oy + bh / 2.0 - c[1],
    ];
    if d == [0.0, 0.0] {
        d = [0.0, 1.0];
    }
    let room = |half: f32, size: f32, run: f32| {
        if run == 0.0 {
            f32::INFINITY
        } else {
            (half - m - size / 2.0).max(0.0) / run.abs()
        }
    };
    (c, d, room(c[0], bw, d[0]), room(c[1], bh, d[1]))
}

/// The side of the surface (0 top, 1 right, 2 bottom, 3 left) an
/// edge-clamped node with margin `m` pins to, or `None` while its drawn
/// box stays inside the margin.
fn clamp_edge(s: Sighting, drawn: [f32; 4], m: f32, w: f32, h: f32) -> Option<u8> {
    let (_, d, tx, ty) = clamp_line(s, drawn, m, w, h);
    if !s.behind && tx >= 1.0 && ty >= 1.0 {
        return None;
    }
    Some(match tx < ty {
        true if d[0] > 0.0 => 1,
        true => 3,
        false if d[1] > 0.0 => 2,
        false => 0,
    })
}

/// An edge-clamped node's top left: its box centre on the line from the
/// surface's centre toward its target, where the line leaves the margin
/// (as a waypoint pins in Unreal's and Unity's HUD guides).
fn clamped(s: Sighting, drawn: [f32; 4], m: f32, w: f32, h: f32) -> [f32; 2] {
    let (c, d, tx, ty) = clamp_line(s, drawn, m, w, h);
    let t = if s.behind {
        tx.min(ty)
    } else {
        tx.min(ty).min(1.0)
    };
    let [ox, oy, bw, bh] = drawn;
    [
        c[0] + d[0] * t - ox - bw / 2.0,
        c[1] + d[1] * t - oy - bh / 2.0,
    ]
}
