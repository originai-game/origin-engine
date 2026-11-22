//! The TSX UI as the new core runs it: the renderer's op batches
//! ([`ops`]) make and patch its elements in an [`Arena`], each with the
//! record its classes resolve to. A frame applies what changed to the
//! text its subtree inherits, lays out what is dirty and paints again
//! only when something changed. A frame with nothing changed does no work
//! and allocates nothing.

use crate::arena::{Arena, Id, Run};
use crate::icons::{IconAtlas, IconCell};
use crate::record::{Record, Rgba, Shadow, Text, TextStroke};
use og_draw::paint::{Paint, PaintKind, PaintQuad};

const NONE: u32 = u32::MAX;

/// What an op node is: a box (a View, a Button, a Scroll …), a text, an
/// image or an icon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Box,
    Label,
    Image,
    Icon,
}

/// The effective look an element passes to its children.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Inherit {
    size: f32,
    ink: Rgba,
    text: Text,
    opacity: f32,
    text_shadow: Shadow,
    text_stroke: TextStroke,
}

struct El {
    node: Id,
    /// Its `disabled` prop.
    disabled: bool,
    /// It answers a press: an element with a press or tap handler.
    clickable: bool,
    /// The renderer's id of the op node ([`ops`]).
    js: u32,
    /// An op node's events (bits by [`ops::Event`]) and its class entry.
    listens: u32,
    class: u32,
    /// The element its arena node is a child of (a fixed one: none).
    arena_parent: u32,
    /// The element it inherits from.
    parent: u32,
    kind: Kind,
    leaf: Option<Id>,
    /// Its text, a text leaf's or an icon's name: rewritten in place, so a
    /// new text no longer than its capacity allocates nothing.
    text: String,
    hidden: u16,
    dead: bool,
    record: Record,
    /// Not drawn since it was made or inserted: `Some(true)` inserted into
    /// a parent on screen. Its style is not drawn yet, so a restyle enters
    /// it again instead of running a transition (CSS never transitions an
    /// element's first style).
    entering: Option<bool>,
    eff: Inherit,
    /// A `<Line>`'s points and how they map onto its box; `None` for any
    /// other node.
    line: Option<Box<Line>>,
    /// A `<Particles>`'s props and its running system; `None` for any
    /// other node.
    particles: Option<Box<particles::Particles>>,
    /// Where an op node with `anchor` sits in the world, how it is placed
    /// there and where it was last placed; `None` for one that sets no
    /// placement prop ([`El::anchored`] for whether it is anchored).
    anchor: Option<Box<Anchor>>,
    /// An op node's own image props over its classes' box image; `None`
    /// for a node that sets none.
    image: Option<Box<ImageProps>>,
    /// An op node's `data-*` props that are on (names without `data-`,
    /// sorted), and its data context's entry ([`ops::Classes::data_id`]):
    /// these and its nearest groups', which its classes' `data-*:` and
    /// `group-data-*:` variants read; 0 none.
    attrs: Vec<Box<str>>,
    data: u32,
    /// It is drawn in the other scheme than the page's: its nearest
    /// `scheme-dark` or `scheme-light`, its own or an ancestor's, names
    /// it ([`crate::classes::OTHER_SCHEME`]).
    other: bool,
}

/// What an anchored node sits on (CONVENTIONS.md §World).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnchorTarget {
    /// An entity, by its bits.
    Entity(u64),
    Point([f32; 3]),
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
}

impl El {
    /// Its anchor when it is out of flow on a target.
    fn anchored(&self) -> Option<&Anchor> {
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
            seen: None,
            edge: None,
            occluded: false,
            at: [0.0; 2],
            shown: false,
            ticked: false,
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
}

/// A `<Line>`'s own props (CONVENTIONS.md §Ops).
#[derive(Clone, Debug, Default)]
pub(crate) struct Line {
    /// x, y pairs: CSS px from the box's top left, or units of `view`.
    pub(crate) points: Vec<f32>,
    pub(crate) closed: bool,
    /// `viewBox` as SVG's min-x, min-y, width and height: the points'
    /// space, which is mapped onto the laid-out box as it paints; `None` px.
    pub(crate) view: Option<[f32; 4]>,
    /// `fit="contain"`: the view scales uniformly and is centred in the box
    /// (SVG's `meet`); otherwise it stretches to fill it.
    pub(crate) contain: bool,
}

impl Line {
    /// The points in the box's px, for a box `w` by `h`.
    fn mapped(&self, w: f32, h: f32) -> impl Iterator<Item = [f32; 2]> + '_ {
        let (s, o) = match self.view {
            Some([vx, vy, vw, vh]) if vw > 0.0 && vh > 0.0 => {
                let (sx, sy) = (w / vw, h / vh);
                let (s, pad) = if self.contain {
                    let k = sx.min(sy);
                    ([k, k], [(w - vw * k) / 2.0, (h - vh * k) / 2.0])
                } else {
                    ([sx, sy], [0.0, 0.0])
                };
                (s, [pad[0] - vx * s[0], pad[1] - vy * s[1]])
            }
            _ => ([1.0, 1.0], [0.0, 0.0]),
        };
        self.points
            .as_chunks::<2>()
            .0
            .iter()
            .map(move |&[x, y]| [o[0] + x * s[0], o[1] + y * s[1]])
    }
}

/// What an element's props set of its box image (CONVENTIONS.md §Images):
/// its source (TSX `src`, .svelte `<img texture>`) and the radial wipe
/// bound per tick, so a moving share never makes a class string.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ImageProps {
    /// The source: the texture handle as the game holds it, which
    /// [`Tree::set_textures`] resolves as it paints; NaN none.
    pub(crate) handle: f64,
    pub(crate) wipe: Option<f32>,
    pub(crate) wipe_start: Option<f32>,
    pub(crate) wipe_ccw: Option<bool>,
}

impl ImageProps {
    /// The box image `base` (its classes') with these props over it, the
    /// source's slot from `textures` (none: 0).
    fn over(
        &self,
        base: &crate::record::Slice,
        textures: Option<&Textures>,
    ) -> crate::record::Slice {
        let mut im = *base;
        if !self.handle.is_nan() {
            im.texture = textures.map_or(0, |f| f(self.handle));
        }
        im.wipe = self.wipe.unwrap_or(im.wipe);
        im.wipe_start = self.wipe_start.unwrap_or(im.wipe_start);
        im.wipe_ccw = self.wipe_ccw.unwrap_or(im.wipe_ccw);
        im
    }
}

/// What a frame did: whether it laid out and painted.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Frame {
    pub laid_out: bool,
    pub painted: bool,
}

pub struct Tree {
    /// The surface's size in CSS px ([`Tree::resize`]).
    size: (f32, f32),
    els: Vec<El>,
    arena: Arena,
    /// What the root passes down: the theme's body text and ink
    /// ([`Tree::set_resolver`]).
    root: Inherit,
    paint: bool,
    /// The HUD's safe-area insets (top, right, bottom, left) a record's
    /// `env(hud-inset-*)` adds at layout ([`Tree::set_insets`]).
    insets: [f32; 4],
    /// What it paints (og-draw's arena, drawn by og-render's paint
    /// pass), refilled in place: its generation moves each repaint.
    pub drawn: Paint,
    /// Each arena node's semantics for the accessibility tree, written by
    /// the op applier ([`crate::a11y`]).
    pub sem: crate::a11y::Semantics,
    /// Each element's place on screen, whether it shows, and the clip its
    /// children paint in (x0, y0, x1, y1).
    abs: Vec<(f32, f32, bool, [f32; 4])>,
    /// The part of the screen each element's children show in as its
    /// scrollers and overflow boxes cut it (x0, y0, x1, y1), the surface
    /// not counted: what the probe calls clipped.
    cuts: Vec<[f32; 4]>,
    /// The elements with an anchor, in no order (a dead or unanchored one
    /// leaves at the next [`Tree::place_anchors`]).
    anchored: Vec<u32>,
    /// [`Tree::settle_anchors`]'s order and the boxes it placed, kept.
    anchor_order: Vec<u32>,
    anchor_placed: Vec<[f32; 4]>,
    /// Each element's text laid out as glyph quads from its leaf's top
    /// left, kept while its text, width and run stay.
    glyphs: Vec<Glyphs>,
    /// The faces text was measured and laid out in (the body's, then its
    /// fallbacks'): when a locale links others, every text is again.
    faces: Vec<Option<og_draw::FontHandle>>,
    /// Characters text was laid out with that no face has, each once,
    /// until [`Tree::take_missing`]: the host loads a font pack that
    /// covers them (docs/ui.md §3.4), and the text draws in it once linked.
    missing: Vec<char>,
    /// The lines [`lay_out`] breaks a text into, kept between texts.
    breaks: Vec<(std::ops::Range<usize>, f32)>,
    /// The screen transition cover [`Tree::set_cover`] gave this frame.
    cover: Option<(crate::cover::Cover, bool)>,
    /// The pad's virtual cursor ([`Tree::virtual_cursor`]), and its
    /// shadow, ink and focus colours (the pack's `bg-bg`, `text-fg` and
    /// `bg-focus`, read with the class resolver).
    vcursor: vcursor::State,
    cursor_colors: [Rgba; 3],
    /// Each element's transform in `drawn.transforms` (its own composed
    /// with its ancestors'; 0 none), what its children compose onto.
    xfs: Vec<u32>,
    /// Each element's mask in the last paint, its own or its nearest
    /// masked ancestor's: a gradient of [`Tree::drawn`] plus 1, 0 none.
    masks: Vec<u32>,
    /// The transforms of the last paint, which `xfs` index: [`Tree::drawn`]
    /// is the host's to take, and the probe reads where a text was drawn
    /// after it has.
    painted_transforms: Vec<[f32; 8]>,
    /// The host's: an image's source (a texture handle) as the slot it
    /// draws, read as the tree paints, so a replaced texture follows.
    textures: Option<Textures>,
    /// The live elements depth first, a parent before its children (in
    /// paint order), made again when the structure changes; `el_of` maps
    /// an arena node to its element, `walk` is its stack.
    order: Vec<u32>,
    ordered: bool,
    el_of: Vec<u32>,
    walk: Vec<Id>,
    /// The op nodes: renderer id to element (NONE: free), their classes'
    /// records once per distinct class and style, and the events since the
    /// renderer last took them ([`ops`]).
    by_js: Vec<u32>,
    classes: ops::Classes,
    pub events: Vec<ops::Fired>,
    /// A batch changed a `data-*` prop, a class or the op tree's shape:
    /// the data contexts are worked out again after it.
    data_dirty: bool,
    /// The scheme the page's records are resolved in ([`Tree::set_resolver`]).
    page_dark: bool,
    /// The string tables op batches named (op 12), for the host to
    /// register ([`Tree::take_strings`]).
    strings: Vec<String>,
    /// What moves, keyed by arena node ([`crate::motion`]), at the UI
    /// tick's `time`; `shown` is what each element passes down this frame
    /// as motion shows it ([`Shown`]).
    motion: crate::motion::Motion,
    time: f64,
    shown: Vec<Shown>,
    /// Op nodes kept drawn while their exit runs: removed (false) or
    /// released too (true); `gone` is the settled exits' scratch list.
    leaving: Vec<(u32, bool)>,
    /// The elements whose `entering` is set, cleared by [`Tree::present`].
    entering: Vec<u32>,
    gone: Vec<Id>,
    /// Op nodes' hover, press, focus and modals ([`input`]), and where
    /// the pointer was in the last frame and how many frames had painted
    /// then (`paints` counts them).
    input: input::Input,
    last_pointer: (f32, f32, u64),
    paints: u64,
    /// The `<Particles>` nodes' tick and which wait for their emitter
    /// ([`particles`]).
    particles: particles::Clock,
}

/// What an element passes down in a frame, as motion shows it: its
/// opacity times its parent's, its ink, text shadow and text outline (each
/// its own, else its parent's), and whether it is in an exit, which
/// nothing hits.
#[derive(Clone, Copy)]
struct Shown {
    alpha: f32,
    ink: Rgba,
    text_shadow: Shadow,
    text_stroke: TextStroke,
    out: bool,
    /// A press, a hover and the cursor reach it (`pointer-events-*`).
    touch: bool,
    /// Its icons' paint (`fill-*`); `None` the ink.
    fill: Option<Rgba>,
}

/// An element's text as glyph quads (from its leaf's top left; colour and
/// clip set as they are copied), and what they were laid out from.
#[derive(Default)]
struct Glyphs {
    text: String,
    key: [u32; 5],
    quads: Vec<PaintQuad>,
    /// Each line's start x, baseline and width, for its decoration.
    lines: Vec<[f32; 3]>,
    /// The lines' texts as laid out, each ended by `\n`: what the probe's
    /// dump checks for a word broken where UAX #14 has no break.
    line_text: String,
}

mod input;
mod ops;
mod particles;
mod vcursor;
pub use input::{Cue, Play, SoundTable};
pub use ops::{EVENTS, Event as OpEvent, Fired, MAGIC, REFUSED, Resolve};
pub use vcursor::{Pad as CursorPad, Pointer as CursorPointer, VirtualCursorDesc};

/// An image's source, a texture handle as the game holds it, as its
/// texture's slot (`og_draw::TextureHandle::slot`; 0 none)
/// ([`Tree::set_textures`]).
pub type Textures = Box<dyn Fn(f64) -> u32>;

impl Default for Tree {
    fn default() -> Self {
        Tree::new()
    }
}

impl Tree {
    /// How an image's source resolves to its texture (the host's assets).
    pub fn set_textures(&mut self, textures: Textures) {
        self.textures = Some(textures);
        self.paint = true;
    }

    /// The textures [`Tree::set_textures`] resolves against changed: the
    /// next frame paints its images again.
    pub fn repaint_images(&mut self) {
        self.paint = true;
    }

    /// An empty tree. Its root passes down the default body text until a
    /// class resolver gives it the theme's ([`Tree::set_resolver`]).
    pub fn new() -> Tree {
        let root = Record::default();
        Tree {
            size: (0.0, 0.0),
            root: Inherit {
                size: root.font_size,
                ink: [1.0; 4],
                text: root.text,
                opacity: 1.0,
                text_shadow: Shadow::default(),
                text_stroke: TextStroke::default(),
            },
            els: Vec::new(),
            arena: Arena::new(),
            paint: true,
            insets: [0.0; 4],
            drawn: Paint::default(),
            sem: crate::a11y::Semantics::default(),
            abs: Vec::new(),
            cuts: Vec::new(),
            anchored: Vec::new(),
            anchor_order: Vec::new(),
            anchor_placed: Vec::new(),
            glyphs: Vec::new(),
            xfs: Vec::new(),
            masks: Vec::new(),
            painted_transforms: Vec::new(),
            textures: None,
            order: Vec::new(),
            ordered: false,
            el_of: Vec::new(),
            walk: Vec::new(),
            by_js: Vec::new(),
            classes: ops::Classes::default(),
            events: Vec::new(),
            data_dirty: false,
            page_dark: false,
            strings: Vec::new(),
            motion: crate::motion::Motion::new(),
            time: 0.0,
            shown: Vec::new(),
            leaving: Vec::new(),
            entering: Vec::new(),
            gone: Vec::new(),
            input: input::Input::new(),
            last_pointer: (f32::NAN, f32::NAN, 0),
            paints: 0,
            particles: Default::default(),
            faces: Vec::new(),
            missing: Vec::new(),
            breaks: Vec::new(),
            cover: None,
            vcursor: Default::default(),
            cursor_colors: [[0.0, 0.0, 0.0, 0.45], [1.0; 4], [0.9, 0.9, 0.95, 1.0]],
        }
    }

    pub fn len(&self) -> usize {
        self.els.len()
    }

    pub fn is_empty(&self) -> bool {
        self.els.is_empty()
    }

    /// `e`'s text; an image's is its source.
    fn set_text(&mut self, e: u32, text: &str) {
        if self.els[e as usize].kind == Kind::Image {
            return self.set_src(e, text);
        }
        let el = &mut self.els[e as usize];
        if &*el.text == text {
            return;
        }
        el.text.clear();
        el.text.push_str(text);
        if let Some(leaf) = el.leaf {
            self.arena.set_text(leaf, text);
        }
        self.paint = true;
    }

    /// `e`'s box image's source: a texture handle's number ("" none).
    fn set_src(&mut self, e: u32, src: &str) {
        let props = self.els[e as usize].image.get_or_insert_with(|| {
            Box::new(ImageProps {
                handle: f64::NAN,
                ..Default::default()
            })
        });
        props.handle = src.trim().parse::<f64>().unwrap_or(f64::NAN);
        self.paint = true;
    }

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
            let ask = tick && a.occlusion != Occlusion::None;
            let seen = project(target, a.lift, ask);
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
            let a = self.els[e as usize].anchor.as_mut().expect("anchored");
            a.seen = seen.filter(|_| on || edge.is_some());
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
        }
        self.refresh_data();
    }

    /// Turns `e`'s engine-set `data-<name>` on or off.
    fn mark(&mut self, e: u32, name: &str, on: bool) {
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
    fn anchor_box(&self, e: u32) -> [f32; 4] {
        let el = &self.els[e as usize];
        let b = self
            .motion
            .box_of(el.node)
            .unwrap_or_else(|| self.arena.box_of(el.node));
        let t = self.motion.skin(el.node, &el.record.skin).transform;
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
    fn settle_anchors(&mut self) {
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
            if shown != a.shown || at != a.at {
                (a.shown, a.at) = (shown, at);
                self.sem.place(el.node, shown.then_some(at));
                self.paint = true;
            }
        }
        self.anchor_order = order;
        self.anchor_placed = placed;
    }

    /// The anchored nodes, for the host to ask which entities they need.
    pub fn anchor_targets(&self) -> impl Iterator<Item = AnchorTarget> + '_ {
        self.anchored
            .iter()
            .filter_map(|&e| self.els[e as usize].anchored().and_then(|a| a.target))
    }

    /// Applies `e`'s record over what it inherits, then its children's.
    fn inherit(&mut self, e: u32) {
        let parent = self.els[e as usize].parent;
        let from = if parent == NONE {
            self.root
        } else {
            self.els[parent as usize].eff
        };
        let el = &self.els[e as usize];
        let r = &el.record;
        let eff = Inherit {
            size: if r.text.sized { r.font_size } else { from.size },
            ink: if r.inked { r.skin.ink } else { from.ink },
            text: r.text.over(&from.text),
            opacity: from.opacity * r.skin.opacity,
            text_shadow: if r.shadowed {
                r.skin.text_shadow
            } else {
                from.text_shadow
            },
            text_stroke: if r.stroked {
                r.skin.text_stroke
            } else {
                from.text_stroke
            },
        };
        let mut layout = r.layout_in(self.insets);
        // A `Scroll` is a scroll container on both axes, as `overflow:
        // auto` is: its automatic minimum is 0, so its content does not
        // size it. An anchored node is out of flow: the host places it.
        let scroller = self.input.per(e).is_some_and(|p| p.scroller);
        if el.hidden > 0 || el.anchored().is_some() || scroller {
            let mut s = taffy::Style::from(&layout);
            if scroller {
                s.overflow = taffy::Point {
                    x: taffy::Overflow::Scroll,
                    y: taffy::Overflow::Scroll,
                };
            }
            if el.hidden > 0 {
                s.display = taffy::Display::None;
            } else if el.anchored().is_some() {
                s.position = taffy::Position::Absolute;
            }
            layout = rune_taffy::CompactStyle::from(&s);
        }
        let (node, leaf) = (el.node, el.leaf);
        self.arena.set_style(node, layout);
        if let Some(leaf) = leaf {
            self.arena.set_run(
                leaf,
                Run {
                    size: eff.size,
                    tracking: eff.text.tracking(eff.size),
                    line: eff.text.line_height(eff.size),
                    weight: eff.text.weight,
                    nowrap: eff.text.nowrap,
                    clamp: eff.text.clamp,
                },
            );
        }
        self.els[e as usize].eff = eff;
    }

    /// Re-applies `e` and every descendant whose inheritance changed.
    fn reapply(&mut self, e: u32) {
        let before = self.els[e as usize].eff;
        self.inherit(e);
        if self.els[e as usize].eff == before {
            return;
        }
        for c in 0..self.els.len() as u32 {
            if self.els[c as usize].parent == e && !self.els[c as usize].dead {
                self.reapply(c);
            }
        }
    }

    /// The surface's size changed: the next frame lays out and paints
    /// again. A record that depends on the size class is the resolver's,
    /// which the host replaces when the class moves.
    pub fn resize(&mut self, width: f32, height: f32) {
        if self.size != (width, height) {
            self.size = (width, height);
            self.paint = true;
        }
    }

    /// The HUD's safe-area insets on the tree's surface (top, right,
    /// bottom, left): the viewport's safe area with the host's covered
    /// region folded in, which `env(hud-inset-*)` reads. A change lays the
    /// elements that read them out again.
    pub fn set_insets(&mut self, insets: [f32; 4]) {
        if self.insets == insets {
            return;
        }
        self.insets = insets;
        for e in 0..self.els.len() as u32 {
            let el = &self.els[e as usize];
            if !el.dead && el.record.safe != [0; 2] {
                self.inherit(e);
            }
        }
        self.paint = true;
    }

    /// The characters laid-out text drew that no face has, since the last
    /// call: a frame that found none allocates nothing.
    pub fn take_missing(&mut self) -> Vec<char> {
        std::mem::take(&mut self.missing)
    }

    /// Lays out what changed and paints. The
    /// fonts measure text; `icons` draws icons.
    pub fn frame(&mut self, fonts: &crate::fonts::Fonts, icons: &IconAtlas) -> Frame {
        let (laid_out, painted) = self.present(fonts, icons);
        Frame { laid_out, painted }
    }

    /// The UI tick's time in seconds (unscaled: a paused game's menus
    /// still move) and the player's reduced-motion setting, which the next
    /// [`Tree::present`] samples motion at. Never a wall clock.
    pub fn set_time(&mut self, t: f64, reduced_motion: bool) {
        self.time = t;
        self.motion.set_reduced_motion(reduced_motion);
        self.particles.reduced = reduced_motion;
    }

    /// Samples motion at the tick's time, lays out what is dirty and paints
    /// again when anything changed or moves: whether it laid out, and
    /// whether it painted.
    pub fn present(&mut self, fonts: &crate::fonts::Fonts, icons: &IconAtlas) -> (bool, bool) {
        for e in self.entering.drain(..) {
            self.els[e as usize].entering = None;
        }
        let moving = self.motion.tick_at(self.time);
        self.settle_exits();
        let (w, h) = self.size;
        let tf = text_font(fonts);
        let faces = || {
            std::iter::once(tf.handle).chain(
                tf.metrics
                    .runs
                    .iter()
                    .flat_map(|r| r.iter().map(|f| f.handle)),
            )
        };
        if !faces().eq(self.faces.iter().copied()) {
            self.faces = faces().collect();
            self.arena.remeasure();
            for g in &mut self.glyphs {
                g.key = [u32::MAX; 5];
            }
            self.paint = true;
        }
        let m = &tf.metrics;
        let laid_out = self.arena.layout(w, h, &mut |t, run, width| {
            crate::text::measure(m, t, run, width)
        });
        if laid_out {
            self.clamp_scrolls();
        }
        // A box that moved slides there when it transitions its layout.
        let arena = &self.arena;
        self.motion.relayout(|n| arena.box_of(n));
        if !self.anchored.is_empty() {
            self.settle_anchors();
        }
        // A `<Particles>` with anything still to draw paints every frame.
        let painted = laid_out || self.paint || moving || self.particles.live;
        if painted {
            self.particles.live = false;
            self.repaint(fonts, icons);
            self.paint = false;
            self.paints += 1;
            let drawn = self.drawn_boxes();
            self.sem.set_drawn(drawn);
        }
        (laid_out, painted)
    }

    /// The box each element under a transform is drawn in (its painted
    /// box's corners through the transform, bounded), by arena node: the
    /// accessibility tree's bounds for it.
    fn drawn_boxes(&self) -> rustc_hash::FxHashMap<Id, [f32; 4]> {
        let mut out = rustc_hash::FxHashMap::default();
        for (e, el) in self.els.iter().enumerate() {
            let xf = self.xfs.get(e).copied().unwrap_or(0);
            let Some(&(x, y, true, _)) = self.abs.get(e) else {
                continue;
            };
            if xf == 0 || el.dead {
                continue;
            }
            let Some(&[a, b, c, d, tx, ty, _, _]) = self.painted_transforms.get(xf as usize) else {
                continue;
            };
            let bx = self
                .motion
                .box_of(el.node)
                .unwrap_or_else(|| self.arena.box_of(el.node));
            let corners = [(x, y), (x + bx.w, y), (x, y + bx.h), (x + bx.w, y + bx.h)]
                .map(|(px, py)| (a * px + c * py + tx, b * px + d * py + ty));
            let x0 = corners.iter().map(|p| p.0).fold(f32::INFINITY, f32::min);
            let y0 = corners.iter().map(|p| p.1).fold(f32::INFINITY, f32::min);
            let x1 = corners
                .iter()
                .map(|p| p.0)
                .fold(f32::NEG_INFINITY, f32::max);
            let y1 = corners
                .iter()
                .map(|p| p.1)
                .fold(f32::NEG_INFINITY, f32::max);
            out.insert(el.node, [x0, y0, x1 - x0, y1 - y0]);
        }
        out
    }

    /// Makes `order` again when the structure changed: the elements under
    /// the arena's root depth first, in their arena order.
    fn order(&mut self) {
        if self.ordered {
            return;
        }
        self.ordered = true;
        self.el_of.clear();
        self.el_of.resize(self.arena.len(), NONE);
        for (e, el) in self.els.iter().enumerate() {
            if !el.dead {
                self.el_of[el.node as usize] = e as u32;
            }
        }
        self.order.clear();
        self.walk.clear();
        self.walk.extend(self.arena.children(0).iter().rev());
        while let Some(n) = self.walk.pop() {
            let e = self.el_of.get(n as usize).copied().unwrap_or(NONE);
            if e != NONE {
                self.order.push(e);
            }
            self.walk.extend(self.arena.children(n).iter().rev());
        }
    }

    fn repaint(&mut self, fonts: &crate::fonts::Fonts, icons: &IconAtlas) {
        /// A line's stroke: its own colour faded, else the ink (already faded).
        fn s_line_color(stroke: &Option<Rgba>, ink: Rgba, alpha: f32) -> Rgba {
            stroke.map_or(ink, |mut c| {
                c[3] *= alpha;
                c
            })
        }
        let (vw, vh) = self.size;
        let screen = [0.0, 0.0, vw, vh];
        let p = &mut self.drawn;
        p.clear();
        (p.width, p.height) = (vw, vh);
        // Font slots: the body's (0), the icon set's (1), then the faces
        // a character the body lacks falls back to (its mixed font's runs:
        // the CJK companion, the locale's, the game's, the fallback), one
        // each from 2; the host adds each one's other pages.
        let tf = text_font(fonts);
        let icon_font = icons.font.as_ref().and_then(|f| f.handle);
        let runs = tf.metrics.runs.as_deref().unwrap_or(&[]);
        let firsts = || {
            [tf.handle, icon_font]
                .into_iter()
                .chain(runs.iter().map(|f| f.handle))
        };
        if p.fonts.len() != 2 + runs.len()
            || firsts()
                .zip(&p.fonts)
                .any(|(h, slot)| slot.first().copied() != h)
        {
            p.fonts = firsts().map(|h| h.iter().copied().collect()).collect();
        }
        let body = tf.handle;
        self.order();
        self.abs.clear();
        self.abs.resize(self.els.len(), (0.0, 0.0, false, [0.0; 4]));
        const ALL: [f32; 4] = [f32::MIN, f32::MIN, f32::MAX, f32::MAX];
        self.cuts.clear();
        self.cuts.resize(self.els.len(), ALL);
        self.shown.clear();
        let top = Shown {
            alpha: 1.0,
            ink: self.root.ink,
            text_shadow: self.root.text_shadow,
            text_stroke: self.root.text_stroke,
            out: false,
            touch: true,
            fill: None,
        };
        // NaN: not reached yet this frame.
        let unseen = Shown {
            alpha: f32::NAN,
            ..top
        };
        self.shown.resize(self.els.len(), unseen);
        self.glyphs.resize_with(self.els.len(), Glyphs::default);
        self.xfs.resize(self.els.len(), 0);
        self.masks.clear();
        self.masks.resize(self.els.len(), 0);
        let m = &tf.metrics;
        for k in 0..self.order.len() {
            let e = self.order[k] as usize;
            let el = &self.els[e];
            // What motion shows: the box a layout transition slides, the
            // skin a transition or keyframes move.
            let b = self
                .motion
                .box_of(el.node)
                .unwrap_or_else(|| self.arena.box_of(el.node));
            let (px, py, pv, pclip) = if let Some(a) = el.anchored() {
                // Its box's top left at its placed point, over the screen;
                // it shows when its target is in view and its parent shows.
                let pv = el.arena_parent == NONE || self.abs[el.arena_parent as usize].2;
                (a.at[0] - b.x, a.at[1] - b.y, pv && a.shown, screen)
            } else if el.arena_parent == NONE {
                (0.0, 0.0, true, screen)
            } else {
                // A scroller's children move by its scroll.
                let [sx, sy] = self.scroll_of(el.arena_parent);
                let (px, py, pv, pclip) = self.abs[el.arena_parent as usize];
                (px - sx, py - sy, pv, pclip)
            };
            // What it inherits comes from the element it inherits from,
            // which a fixed one is not the arena child of: one not reached
            // yet gives what its record folds to.
            let from = match el.parent {
                NONE => top,
                p if !self.shown[p as usize].alpha.is_nan() => self.shown[p as usize],
                p => {
                    let eff = &self.els[p as usize].eff;
                    Shown {
                        alpha: eff.opacity,
                        ink: eff.ink,
                        text_shadow: eff.text_shadow,
                        text_stroke: eff.text_stroke,
                        out: false,
                        touch: true,
                        fill: None,
                    }
                }
            };
            // `display: none` leaves a zero box at its parent's corner:
            // nothing of it or under it paints.
            let r = &el.record;
            let s = self.motion.skin(el.node, &r.skin);
            let alpha = from.alpha * s.opacity * el.anchored().map_or(1.0, Anchor::alpha);
            let own_ink = if r.inked { s.ink } else { from.ink };
            let text_shadow = if r.shadowed {
                s.text_shadow
            } else {
                from.text_shadow
            };
            let text_stroke = if r.stroked {
                s.text_stroke
            } else {
                from.text_stroke
            };
            self.shown[e] = Shown {
                alpha,
                ink: own_ink,
                text_shadow,
                text_stroke,
                out: from.out || self.motion.exiting(el.node),
                touch: r.touch.unwrap_or(from.touch),
                fill: r.fill.unwrap_or(from.fill),
            };
            let visible = pv
                && !el.dead
                && el.hidden == 0
                && !r.hidden
                && alpha > 0.0
                && r.layout.display() != taffy::Display::None;
            let (x, y) = (px + b.x, py + b.y);
            let clip = pclip;
            let pcut = match el.arena_parent {
                _ if el.anchor.is_some() => ALL,
                NONE => ALL,
                p => self.cuts[p as usize],
            };
            // What overflows a box that does not show it is cut to it.
            let o = taffy::CoreStyle::overflow(&r.layout);
            let scroller = self.scroll_axes(e as u32) != [false; 2];
            // CSS clips overflow at the padding box: what scrolls or runs
            // over never covers the box's own border.
            let [bt, br, bb, bl] = s.border;
            let cuts =
                scroller || o.x != taffy::Overflow::Visible || o.y != taffy::Overflow::Visible;
            let cut_to = |c: [f32; 4], [x0, y0, x1, y1]: [f32; 4]| {
                [c[0].max(x0), c[1].max(y0), c[2].min(x1), c[3].min(y1)]
            };
            let padding = [x + bl, y + bt, x + b.w - br, y + b.h - bb];
            let inner = if cuts { cut_to(clip, padding) } else { clip };
            self.abs[e] = (x, y, visible, inner);
            self.cuts[e] = if cuts { cut_to(pcut, padding) } else { pcut };
            let pxf = if el.arena_parent == NONE {
                0
            } else {
                self.xfs[el.arena_parent as usize]
            };
            self.xfs[e] = pxf;
            self.masks[e] = if el.arena_parent == NONE {
                0
            } else {
                self.masks[el.arena_parent as usize]
            };
            if !visible {
                continue;
            }
            // A `knob` node rides its on-screen control's knob.
            let [kx, ky] = self.knob_of(e as u32);
            let mut t = s.transform;
            t.translate = [t.translate[0] + kx, t.translate[1] + ky];
            let p = &mut self.drawn;
            let start = p.quads.len();
            // Its own mask replaces an ancestor's over its subtree.
            if let Some(g) = &r.mask {
                let area = [x, y, b.w, b.h];
                let gi = p.shaped_gradient(paint_gradient(g, area), g.stops().iter().copied());
                self.masks[e] = gi + 1;
            }
            let xf = transformed(p, pxf, &t, [x, y, b.w, b.h]);
            self.xfs[e] = xf;
            // The clip it sets for what it holds is drawn through its
            // transform (an ancestor's or its own): the box that bounds its
            // padding box there, against the clip it is drawn in.
            if xf != 0 && cuts {
                let [ma, mb, mc, md, me, mf, _, _] = p.transforms[xf as usize];
                let [x0, y0, x1, y1] = padding;
                let mut bound = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
                for [cx, cy] in [[x0, y0], [x1, y0], [x0, y1], [x1, y1]] {
                    let (sx, sy) = (ma * cx + mc * cy + me, mb * cx + md * cy + mf);
                    bound = [
                        bound[0].min(sx),
                        bound[1].min(sy),
                        bound[2].max(sx),
                        bound[3].max(sy),
                    ];
                }
                self.abs[e].3 = cut_to(clip, bound);
                self.cuts[e] = cut_to(pcut, bound);
            }
            let rect = [x, y, b.w, b.h];
            let radius = radii(s.radius, b.w, b.h);
            let faded = |mut c: Rgba| {
                c[3] *= alpha;
                c
            };
            // Outer shadows under the box, the last listed lowest.
            let shadows = &s.shadows[..usize::from(s.shadow_count)];
            for sh in shadows
                .iter()
                .rev()
                .filter(|sh| !sh.inset && sh.color[3] > 0.0)
            {
                let p4 = [sh.offset[0], sh.offset[1], sh.blur, sh.spread];
                p.quads.push(quad(
                    PaintKind::Shadow,
                    rect,
                    clip,
                    faded(sh.color),
                    radius,
                    p4,
                    0,
                    0,
                ));
            }
            let im = &match &el.image {
                Some(props) => props.over(&s.image, self.textures.as_ref()),
                None => s.image,
            };
            let image = og_draw::TextureHandle::from_slot(im.texture).filter(|_| im.wipe > 0.0);
            // A fill-tinted image takes the fill's colour in its place.
            let fill_tints = image.is_some() && im.tint == crate::record::Tint::Fill;
            if s.fill[3] > 0.0 && !fill_tints {
                p.quads.push(quad(
                    PaintKind::Solid,
                    rect,
                    clip,
                    faded(s.fill),
                    radius,
                    [0.0; 4],
                    0,
                    0,
                ));
            }
            if let Some(g) = &s.gradient {
                let gi = p.shaped_gradient(
                    paint_gradient(g, [x, y, b.w, b.h]),
                    g.stops().iter().copied(),
                );
                p.quads.push(quad(
                    PaintKind::Gradient,
                    rect,
                    clip,
                    [1.0, 1.0, 1.0, alpha],
                    radius,
                    [0.0; 4],
                    gi,
                    0,
                ));
            }
            // Its image over the fill, nine-sliced, radially wiped.
            if let Some(t) = image {
                let width = if im.width == [0.0; 4] {
                    im.slice
                } else {
                    im.width
                };
                let color = match im.tint {
                    crate::record::Tint::Own => [1.0, 1.0, 1.0, alpha],
                    crate::record::Tint::Ink => faded(own_ink),
                    crate::record::Tint::Fill => faded(s.fill),
                };
                let flags = (u32::from(im.fill) * og_draw::paint::IMAGE_FILL)
                    | (u32::from(im.tile) * og_draw::paint::IMAGE_TILE);
                let cut = if im.wipe < 1.0 { 1.0 - im.wipe } else { 0.0 };
                p.quads.push(PaintQuad {
                    uv: im.uv,
                    wipe: [cut, im.wipe_start, f32::from(u8::from(im.wipe_ccw)), 0.0],
                    mask: 0,
                    ..quad(
                        PaintKind::Image,
                        rect,
                        clip,
                        color,
                        width,
                        im.slice,
                        t.to_bits(),
                        flags,
                    )
                });
            }
            for sh in shadows
                .iter()
                .rev()
                .filter(|sh| sh.inset && sh.color[3] > 0.0)
            {
                let p4 = [sh.offset[0], sh.offset[1], sh.blur, sh.spread];
                p.quads.push(quad(
                    PaintKind::InsetShadow,
                    rect,
                    clip,
                    faded(sh.color),
                    radius,
                    p4,
                    0,
                    0,
                ));
            }
            let sides = if s.border_side[3] > 0.0 {
                s.border_sides
            } else {
                0
            };
            if (s.border_color[3] > 0.0 || sides != 0) && s.border.iter().any(|&w| w > 0.0) {
                p.quads.push(PaintQuad {
                    uv: faded(s.border_side),
                    ..quad(
                        PaintKind::Border,
                        rect,
                        clip,
                        faded(s.border_color),
                        radius,
                        s.border,
                        0,
                        u32::from(sides),
                    )
                });
            }
            let o = &s.outline;
            if o.width > 0.0 && o.color[3] > 0.0 {
                // The ring of the box grown by the offset and the width; its
                // corners follow the box's, as browsers draw them.
                let out = o.offset + o.width;
                let grown = [x - out, y - out, b.w + 2.0 * out, b.h + 2.0 * out];
                let rr = radius.map(|k| if k > 0.0 { (k + out).max(0.0) } else { 0.0 });
                p.quads.push(quad(
                    PaintKind::Border,
                    grown,
                    clip,
                    faded(o.color),
                    rr,
                    [o.width; 4],
                    0,
                    0,
                ));
            }
            let mut ink = own_ink;
            ink[3] *= alpha;
            // An icon is painted in its fill, the ink unless `fill-*` says.
            let icon_ink = self.shown[e].fill.map_or(ink, faded);
            if let Some(line) = &el.line {
                let stroke = s_line_color(&r.stroke, ink, alpha);
                let n = line.points.len() / 2;
                if r.stroke_width >= 1.0 && n >= 2 && stroke[3] > 0.0 {
                    let first = p.points.len() as u32;
                    p.points.extend(line.mapped(b.w, b.h));
                    p.quads.push(quad(
                        PaintKind::Line,
                        rect,
                        clip,
                        stroke,
                        [0.0; 4],
                        [r.stroke_width, f32::from(u8::from(line.closed)), 0.0, 0.0],
                        first,
                        n as u32,
                    ));
                }
            }
            if el.kind == Kind::Icon && !el.text.is_empty() {
                let size = b.w.min(b.h).max(1.0);
                let square = [x + (b.w - size) / 2.0, y + (b.h - size) / 2.0, size, size];
                match icons.find_drawn(&el.text) {
                    Some((IconCell::Glyph(_, ch), _)) => {
                        let (asc, desc) = icons
                            .font
                            .as_ref()
                            .map_or((0.8, -0.2), |f| (f.metrics.ascender, f.metrics.descender));
                        let top = y + (b.h - (asc - desc) * size) / 2.0;
                        p.quads.push(quad(
                            PaintKind::Glyph,
                            [square[0], top + asc * size, size, 0.0],
                            clip,
                            icon_ink,
                            [0.0; 4],
                            [0.0; 4],
                            ch as u32,
                            1,
                        ));
                    }
                    // A texture grid's cell: a picture in its own colours, a
                    // mask in the ink.
                    Some((IconCell::Image(t, uv), picture)) => {
                        let color = if picture {
                            [1.0, 1.0, 1.0, alpha]
                        } else {
                            icon_ink
                        };
                        p.quads.push(PaintQuad {
                            uv,
                            ..quad(
                                PaintKind::Image,
                                square,
                                clip,
                                color,
                                [0.0; 4],
                                [0.0; 4],
                                t.to_bits(),
                                og_draw::paint::IMAGE_FILL,
                            )
                        });
                    }
                    Some((IconCell::Builtin(name), _)) => {
                        let set = crate::icons::icons();
                        if p.icons.is_none() {
                            p.icons = Some(set.segments.clone());
                        }
                        if let Some((first, count)) = set.find(name) {
                            p.quads.push(quad(
                                PaintKind::Icon,
                                square,
                                clip,
                                icon_ink,
                                [0.0; 4],
                                [0.0; 4],
                                first,
                                count,
                            ));
                        }
                    }
                    None => {}
                }
            }
            // A glyph with the pack's art for its binding draws it in its
            // text's place: an ink mask one em square, centred on the leaf
            // (which holds an em space, so it lays out as one).
            if let Some(t) = self
                .input
                .per(e as u32)
                .and_then(|g| og_draw::TextureHandle::from_slot(g.art))
                && let Some(leaf) = el.leaf
            {
                let lb = self.arena.box_of(leaf);
                let size = el.eff.size;
                let square = [
                    x + lb.x + (lb.w - size) / 2.0,
                    y + lb.y + (lb.h - size) / 2.0,
                    size,
                    size,
                ];
                p.quads.push(PaintQuad {
                    uv: [0.0, 0.0, 1.0, 1.0],
                    ..quad(
                        PaintKind::Image,
                        square,
                        clip,
                        ink,
                        [0.0; 4],
                        [0.0; 4],
                        t.to_bits(),
                        og_draw::paint::IMAGE_FILL,
                    )
                });
            }
            // An `Input` showing its placeholder draws it at half its ink.
            let field = self.input.per(e as u32).filter(|f| f.input);
            let mut text_ink = ink;
            if field.is_some_and(|f| f.value.is_empty()) {
                text_ink[3] *= 0.5;
            }
            if let (Some(leaf), Some(_)) = (el.leaf, body)
                && !el.text.is_empty()
            {
                let lb = self.arena.box_of(leaf);
                let g = &mut self.glyphs[e];
                lay_out(
                    g,
                    &el.text,
                    &el.eff,
                    lb.w,
                    m,
                    &mut self.missing,
                    &mut self.breaks,
                );
                let (lx, ly) = (x + lb.x, y + lb.y);
                let sh = text_shadow;
                if sh.color[3] > 0.0 {
                    let mut c = sh.color;
                    c[3] *= alpha;
                    let [dx, dy] = sh.offset;
                    p.quads.extend(g.quads.iter().map(|q| PaintQuad {
                        rect: [
                            q.rect[0] + lx + dx,
                            q.rect[1] + ly + dy,
                            q.rect[2],
                            q.rect[3],
                        ],
                        clip,
                        color: c,
                        p: [q.p[0], sh.blur, 0.0, 0.0],
                        ..*q
                    }));
                }
                // The outline under the fill: each glyph thickened by half
                // the stroke (its field's reach bounds it, in the shader).
                let st = text_stroke;
                if st.width > 0.0 && st.color[3] > 0.0 {
                    let mut c = st.color;
                    c[3] *= alpha;
                    p.quads.extend(g.quads.iter().map(|q| PaintQuad {
                        rect: [q.rect[0] + lx, q.rect[1] + ly, q.rect[2], q.rect[3]],
                        clip,
                        color: c,
                        p: [q.p[0] + 0.5 * st.width, 0.0, 0.0, 0.0],
                        ..*q
                    }));
                }
                p.quads.extend(g.quads.iter().map(|q| PaintQuad {
                    rect: [q.rect[0] + lx, q.rect[1] + ly, q.rect[2], q.rect[3]],
                    clip,
                    color: text_ink,
                    ..*q
                }));
                let d = s.decoration;
                if d.line != 0 {
                    let mut c = if d.color[3] > 0.0 { d.color } else { own_ink };
                    c[3] *= alpha;
                    let size = el.eff.size;
                    let th = if d.thickness > 0.0 {
                        d.thickness
                    } else {
                        (size / 14.0).max(1.0)
                    };
                    use crate::record::Decoration as D;
                    for &[x0, base, w] in &g.lines {
                        let at = [
                            (D::UNDERLINE, base + 0.12 * size),
                            (D::OVERLINE, base - m.ascender * size),
                            (D::LINE_THROUGH, base - 0.3 * size),
                        ];
                        for (bit, cy) in at {
                            if d.line & bit != 0 {
                                let rect = [lx + x0, ly + cy - th / 2.0, w, th];
                                p.quads.push(quad(
                                    PaintKind::Solid,
                                    rect,
                                    clip,
                                    c,
                                    [0.0; 4],
                                    [0.0; 4],
                                    0,
                                    0,
                                ));
                            }
                        }
                    }
                }
            }
            // The focused `Input`'s caret: a bar the text's height before
            // the character it stands at, in the text's ink.
            if let (Some(leaf), Some(f)) = (el.leaf, field)
                && self.input.focus_el() == e as u32
            {
                let t = &el.eff.text;
                let size = el.eff.size;
                let typed = !f.value.is_empty();
                let before = if typed {
                    &f.value[..f.caret.min(f.value.len())]
                } else {
                    ""
                };
                let features = og_text::shape::Features::NONE;
                let track = t.tracking(size) / size.max(f32::MIN_POSITIVE);
                let pen = og_text::text::widest_line(before, m, size, track, features);
                let dx = if typed {
                    self.glyphs[e].lines.first().map_or(0.0, |l| l[0])
                } else {
                    0.0
                };
                let glyph = (m.ascender - m.descender) * size;
                let top = (t.line_height(size) - glyph) / 2.0;
                let lb = self.arena.box_of(leaf);
                let rect = [
                    x + lb.x + dx + pen,
                    y + lb.y + top,
                    (size / 14.0).max(1.0),
                    glyph,
                ];
                p.quads.push(quad(
                    PaintKind::Solid,
                    rect,
                    clip,
                    ink,
                    [0.0; 4],
                    [0.0; 4],
                    0,
                    0,
                ));
            }
            if xf != 0 {
                for q in &mut self.drawn.quads[start..] {
                    q.xf = xf;
                }
            }
            // A `<Particles>`' particles over its box, each under the
            // node's transform with its own turn.
            if self.els[e].particles.is_some() {
                self.paint_particles(e, rect, ink, clip, xf);
            }
            let mask = self.masks[e];
            if mask != 0 {
                for q in &mut self.drawn.quads[start..] {
                    q.mask = mask;
                }
            }
        }
        self.painted_transforms.clone_from(&self.drawn.transforms);
        let mut quads = std::mem::take(&mut self.drawn.quads);
        self.paint_virtual_cursor(&mut quads);
        self.drawn.quads = quads;
        // A screen transition's cover, over everything.
        if let Some((cover, reduced)) = self.cover {
            cover.paint(vw, vh, reduced, &mut self.drawn.quads);
        }
        self.drawn.generation += 1;
    }

    /// This frame's screen transition cover (`None`: none), drawn over
    /// everything; `reduced` motion draws it as a fade.
    pub fn set_cover(&mut self, cover: Option<crate::cover::Cover>, reduced: bool) {
        let cover = cover.map(|c| (c, reduced));
        if self.cover != cover {
            self.cover = cover;
            self.paint = true;
        }
    }

    /// The element a press at (x, y) reaches: of the shown, enabled
    /// elements that answer a press and whose box, inside the clip their
    /// parent paints them in, holds the point, the last made (a descendant
    /// comes after its ancestors and paints over them); NONE for none.
    fn hit(&self, x: f32, y: f32) -> u32 {
        self.hit_where(x, y, |el| el.clickable)
    }

    /// [`Tree::hit`] over the elements `want` accepts; a press needs them
    /// enabled too. Behind an open modal, nothing answers.
    fn hit_where(&self, x: f32, y: f32, want: impl Fn(&El) -> bool) -> u32 {
        self.topmost(x, y, want, true)
    }

    /// The topmost shown element at (x, y) that `want` accepts; a disabled
    /// control counts only when `enabled` is false (its cursor shows).
    fn topmost(&self, x: f32, y: f32, want: impl Fn(&El) -> bool, enabled: bool) -> u32 {
        for &e in self.order.iter().rev() {
            let (e, el) = (e as usize, &self.els[e as usize]);
            // An element on its way out, or in one, answers nothing.
            if el.dead || !want(el) || self.shown.get(e).is_some_and(|s| s.out || !s.touch) {
                continue;
            }
            let Some(&(bx, by, shown, _)) = self.abs.get(e) else {
                continue;
            };
            let all = [f32::MIN, f32::MIN, f32::MAX, f32::MAX];
            let [cx0, cy0, cx1, cy1] = match el.arena_parent {
                _ if el.anchored().is_some() => all,
                NONE => all,
                p => self.abs.get(p as usize).map_or(all, |a| a.3),
            };
            let b = self
                .motion
                .box_of(el.node)
                .unwrap_or_else(|| self.arena.box_of(el.node));
            let inside = x >= bx.max(cx0)
                && x < (bx + b.w).min(cx1)
                && y >= by.max(cy0)
                && y < (by + b.h).min(cy1);
            if !shown || !inside || !self.reachable(e as u32) {
                continue;
            }
            if enabled && el.clickable && el.disabled {
                continue;
            }
            return e as u32;
        }
        NONE
    }

    /// The pointer this frame, at (x, y) on the tree's surface. The op
    /// nodes take it when it moved, a frame painted since (a box may have
    /// come under it), or a button changed.
    pub fn pointer(&mut self, x: f32, y: f32, pressed: bool, released: bool) {
        let was = std::mem::replace(&mut self.last_pointer, (x, y, self.paints));
        if self.has_ops() && (was != self.last_pointer || pressed || released) {
            self.pointer_ops(x, y, (was.0, was.1) != (x, y), pressed, released);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn quad(
    kind: PaintKind,
    rect: [f32; 4],
    clip: [f32; 4],
    color: Rgba,
    radius: [f32; 4],
    p: [f32; 4],
    a: u32,
    b: u32,
) -> PaintQuad {
    PaintQuad {
        rect,
        clip,
        color,
        radius,
        p,
        kind: kind as u32,
        a,
        b,
        xf: 0,
        uv: [0.0; 4],
        wipe: [0.0; 4],
        mask: 0,
    }
}

/// Adds the transform of a box `rect` (x, y, w, h) composed onto its
/// parent's `parent` to `p.transforms`: its index, or `parent` when its own
/// is the identity. CSS's order: move to the origin, translate (its
/// percentages of the box's own size), rotate, scale, move back.
fn transformed(p: &mut Paint, parent: u32, t: &crate::record::Transform, rect: [f32; 4]) -> u32 {
    if t.is_identity() {
        return parent;
    }
    if p.transforms.is_empty() {
        p.transforms.push(og_draw::paint::IDENTITY);
    }
    let [a, b, c, d, e, f] = affine(t, rect);
    let [pa, pb, pc, pd, pe, pf, _, _] = p.transforms[parent as usize];
    p.transforms.push([
        pa * a + pc * b,
        pb * a + pd * b,
        pa * c + pc * d,
        pb * c + pd * d,
        pa * e + pc * f + pe,
        pb * e + pd * f + pf,
        0.0,
        0.0,
    ]);
    p.transforms.len() as u32 - 1
}

/// `t` on a box at `rect` as a 2D affine map (a, b, c, d, e, f): a point
/// (x, y) goes to (a x + c y + e, b x + d y + f).
fn affine(t: &crate::record::Transform, rect: [f32; 4]) -> [f32; 6] {
    let (sin, cos) = og_core::math::sin_cos(t.rotate);
    let [sx, sy] = t.scale;
    let (a, b, c, d) = (cos * sx, sin * sx, -sin * sy, cos * sy);
    let ox = rect[0] + t.origin[0] * rect[2];
    let oy = rect[1] + t.origin[1] * rect[3];
    let e = ox + t.translate[0] + t.percent[0] * rect[2] - (a * ox + c * oy);
    let f = oy + t.translate[1] + t.percent[1] * rect[3] - (b * ox + d * oy);
    [a, b, c, d, e, f]
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

/// Corner radii (top left, top right, bottom right, bottom left) on a
/// `w`×`h` box, all scaled down by one factor when two on a side would
/// overlap, as CSS scales them.
/// Gradient `g` over the box `area` (x y w h, CSS px) as the paint list's:
/// a linear one at its angle (a `to <corner>` one aimed at the box's
/// corner), a radial one's ending shape sized by CSS Images 3's keywords, a
/// conic one from its angle; centres in CSS px from the box's centre.
fn paint_gradient(g: &crate::record::Gradient, area: [f32; 4]) -> og_draw::paint::PaintGradient {
    use crate::record::Shape;
    use nsh_tailwind::effects::GradientSize as Size;
    let [_, _, w, h] = area;
    // A `to <corner>` line runs perpendicular to the diagonal between the
    // other two corners, so its angle is the box's.
    let angle = match g.corner {
        [0, _] | [_, 0] => g.angle,
        [cx, cy] => og_core::math::atan2(f32::from(cx) * h, -f32::from(cy) * w),
    };
    let (cx, cy) = (g.center[0] * w, g.center[1] * h);
    let center = [cx - w / 2.0, cy - h / 2.0];
    // The distances from the centre to the nearer and the farther side.
    let (near, far) = (
        [cx.min(w - cx).abs(), cy.min(h - cy).abs()],
        [cx.max(w - cx).abs(), cy.max(h - cy).abs()],
    );
    let (shape, radii) = match g.shape {
        Shape::Linear => (og_draw::paint::GRADIENT_LINEAR, [0.0; 2]),
        Shape::Conic => (og_draw::paint::GRADIENT_CONIC, [0.0; 2]),
        Shape::Radial { circle, size } => {
            let radii = match (circle, size) {
                (true, Size::ClosestSide) => [near[0].min(near[1]); 2],
                (true, Size::FarthestSide) => [far[0].max(far[1]); 2],
                (true, Size::ClosestCorner) => [(near[0] * near[0] + near[1] * near[1]).sqrt(); 2],
                (true, Size::FarthestCorner) => [(far[0] * far[0] + far[1] * far[1]).sqrt(); 2],
                (false, Size::ClosestSide) => near,
                (false, Size::FarthestSide) => far,
                // The ellipse of the sides' proportions through the corner.
                (false, Size::ClosestCorner) => near.map(|r| r * std::f32::consts::SQRT_2),
                (false, Size::FarthestCorner) => far.map(|r| r * std::f32::consts::SQRT_2),
            };
            (og_draw::paint::GRADIENT_RADIAL, radii)
        }
    };
    og_draw::paint::PaintGradient {
        angle,
        first: 0,
        count: 0,
        shape,
        center,
        radii,
        area,
    }
}

fn radii(r: [f32; 4], w: f32, h: f32) -> [f32; 4] {
    let [tl, tr, br, bl] = r.map(|k| k.max(0.0));
    let fit = |side: f32, sum: f32| if sum > side { side / sum } else { 1.0 };
    let f = fit(w, tl + tr)
        .min(fit(w, bl + br))
        .min(fit(h, tl + bl))
        .min(fit(h, tr + br));
    [tl * f, tr * f, br * f, bl * f]
}

/// The font a text draws in: the body's mixed font (the body face, then
/// the faces it falls back to for what it lacks: `crate::fonts::Fonts::link`),
/// else the body alone.
fn text_font(fonts: &crate::fonts::Fonts) -> &og_text::text::UiFont {
    fonts.body.metrics.mixed.as_deref().unwrap_or(&fonts.body)
}

/// The paint font slot of face `k` of a text's mixed font: the body's 0,
/// a fallback face's from 2 (1 is the icons').
fn slot_of(k: u8) -> u32 {
    match k {
        0 => 0,
        k => u32::from(k) + 1,
    }
}

/// Lays `text` out as glyph quads from its leaf's top left (a quad's x the
/// pen, y the baseline, w the size; `b` the font slot of the face that
/// has the character, [`slot_of`]), when it, the width or the run changed
/// since `g` was laid out (a change of faces clears `g.key`: `present`).
fn lay_out(
    g: &mut Glyphs,
    text: &str,
    eff: &Inherit,
    width: f32,
    m: &og_text::text::FontMetrics,
    missing: &mut Vec<char>,
    breaks: &mut Vec<(std::ops::Range<usize>, f32)>,
) {
    let t = &eff.text;
    let size = eff.size;
    let tracking = t.tracking(size);
    let line = t.line_height(size);
    let bold = t.weight >= 600;
    let key = [
        width.to_bits(),
        size.to_bits(),
        tracking.to_bits(),
        line.to_bits(),
        u32::from(t.align)
            | u32::from(t.nowrap) << 2
            | u32::from(bold) << 3
            | u32::from(t.overflow) << 4
            | u32::from(t.clamp) << 8,
    ];
    if g.key == key && *g.text == *text {
        return;
    }
    g.key = key;
    if *g.text != *text {
        g.text.clear();
        g.text.push_str(text);
    }
    g.quads.clear();
    g.lines.clear();
    g.line_text.clear();
    let glyph = (m.ascender - m.descender) * size;
    let weight = if bold { size * 0.03 } else { 0.0 };
    let track = if size > 0.0 { tracking / size } else { 0.0 };
    let features = og_text::shape::Features::NONE;
    let mut place =
        |k: usize, l: &str, w: f32, quads: &mut Vec<PaintQuad>, lines: &mut Vec<[f32; 3]>| {
            let top = k as f32 * line + (line - glyph) / 2.0;
            let dx = match t.align {
                1 => (width - w) / 2.0,
                2 => width - w,
                _ => 0.0,
            };
            let baseline = top + m.ascender * size;
            lines.push([dx, baseline, w]);
            let mut pen = 0.0f32;
            let mut prev: Option<char> = None;
            for ch in l.chars() {
                if !m.has(ch) {
                    if !ch.is_whitespace()
                        && !ch.is_control()
                        && !og_text::text::invisible(ch)
                        && !missing.contains(&ch)
                    {
                        missing.push(ch);
                    }
                    prev = None;
                    continue;
                }
                if let Some(p) = prev {
                    pen += m.kern(p, ch);
                }
                if !ch.is_whitespace() {
                    // The face that has it, drawn at its own scale.
                    let k = m.face_of(ch.encode_utf8(&mut [0; 4]));
                    let scale = m.part(usize::from(k)).map_or(1.0, |f| f.scale);
                    quads.push(quad(
                        PaintKind::Glyph,
                        [dx + pen * size, baseline, size * scale, 0.0],
                        [0.0; 4],
                        [0.0; 4],
                        [0.0; 4],
                        [weight, 0.0, 0.0, 0.0],
                        ch as u32,
                        slot_of(k),
                    ));
                }
                pen += m.advance(ch) + track;
                prev = Some(ch);
            }
        };
    // `line-clamp-*` shows its first lines, the last cut to fit an ellipsis
    // when more follow; `truncate` (one line, `text-ellipsis`) cuts a line
    // wider than the box the same way.
    let shown = |n: usize| match t.clamp {
        0 => n,
        c => n.min(usize::from(c)),
    };
    let ellipsis = if m.has('\u{2026}') { "\u{2026}" } else { "..." };
    let wide = |l: &str| og_text::text::widest_line(l, m, size, track, features);
    let cut = |l: &str, w: f32, more: bool| -> Option<(String, f32)> {
        if !more && (t.overflow != 1 || w <= width + 0.5) {
            return None;
        }
        let mut s = l.trim_end().to_string();
        loop {
            let with = format!("{s}{ellipsis}");
            let w = wide(&with);
            if w <= width + 0.5 || s.is_empty() {
                return Some((with, w));
            }
            s.pop();
            s.truncate(s.trim_end().len());
        }
    };
    let mut lines: Vec<(std::borrow::Cow<str>, f32)> = Vec::new();
    // Broken as the measure broke it (text.rs), by the same shaped layout
    // at the same width, so a text never breaks inside the box it sized.
    let wrap = (!t.nowrap && size > 0.0).then_some((width + 0.5) / size);
    if og_text::text::paragraph::lines_into(m, text, track, features, wrap, breaks) {
        lines.extend(breaks.iter().map(|(r, w)| {
            let l = text[r.clone()].trim_end_matches(['\n', '\r']);
            (l.into(), w * size)
        }));
    } else if t.nowrap || (wide(text) <= width + 0.5 && !text.contains('\n')) {
        lines.extend(text.split('\n').map(|l| (l.into(), wide(l))));
    } else {
        let broken = og_text::text::break_lines_with_overflow(
            text,
            m,
            size,
            track,
            features,
            Some(width + 0.5),
            false,
        );
        lines.extend(broken.into_iter().map(|l| (l.text.into(), l.width)));
    }
    let n = shown(lines.len());
    for (k, (l, w)) in lines.iter().take(n).enumerate() {
        let (l, w) = match cut(l, *w, k + 1 == n && n < lines.len()) {
            Some((l, w)) => (l.into(), w),
            None => (std::borrow::Cow::Borrowed(&**l), *w),
        };
        place(k, &l, w, &mut g.quads, &mut g.lines);
        g.line_text.push_str(&l);
        g.line_text.push('\n');
    }
}
