//! The TSX UI as the new core runs it: the renderer's op batches
//! ([`ops`]) make and patch its elements in an [`Arena`], each with the
//! record its classes resolve to. A frame applies what changed to the
//! text its subtree inherits, lays out what is dirty and paints again
//! only when something changed. A frame with nothing changed does no work
//! and allocates nothing.

use crate::arena::{Arena, Id, Run};
use crate::icons::{IconAtlas, IconCell};
use crate::record::{Decoration, Record, Rgba, Text, TextShadows, TextStroke};
use og_draw::glyphs::Atlas;
use og_draw::paint::{Paint, PaintEffect, PaintGroup, PaintKind, PaintQuad};
use og_i18n::Direction;
use og_text::fonts::Collection;

const NONE: u32 = u32::MAX;

/// What an op node is: a box (a View, a Button, a Scroll …), a text, an
/// image or an icon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Box,
    /// A `<Text>`: a paragraph of the runs, spans and icons in it ([`para`]).
    Text,
    Label,
    Image,
    Icon,
}

/// A `glide-<name>` element that left the screen: its name, the time,
/// the box it was last drawn in (surface space) and the ancestors it was
/// under, nearest first.
type Left = (Box<str>, f64, [f32; 4], [u32; 4]);

/// A text rolling to a new number ([`Tree::roll`]).
#[derive(Clone, Debug)]
struct Count {
    /// The text node.
    e: u32,
    from: f64,
    /// What it draws now.
    now: f64,
    to: Number,
    /// The time it started.
    since: f64,
}

/// The one number in a text (`1,250`, `-3.5`, `x12`), how it is written,
/// and where: what a `transition-count` text rolls.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Number {
    value: f64,
    /// Bytes before it and after it.
    at: [usize; 2],
    decimals: usize,
    /// The thousands separator it is grouped by, if any.
    group: Option<char>,
}

impl Number {
    /// The text's number, if it holds exactly one.
    fn of(text: &str) -> Option<Number> {
        let start = text.find(|c: char| c.is_ascii_digit())?;
        let neg = text[..start].ends_with('-');
        let mut group = None;
        let mut digits = String::new();
        let mut end = start;
        let mut decimals = None::<usize>;
        let b = text.as_bytes();
        let mut i = start;
        while i < text.len() {
            let c = text[i..].chars().next()?;
            if c.is_ascii_digit() {
                digits.push(c);
                if let Some(d) = decimals.as_mut() {
                    *d += 1;
                }
                i += 1;
                end = i;
            } else if (c == ',' || c == '\u{2009}' || c == '\u{202f}')
                && decimals.is_none()
                && text[i + c.len_utf8()..].starts_with(|d: char| d.is_ascii_digit())
            {
                group = Some(c);
                i += c.len_utf8();
            } else if c == '.' && decimals.is_none() && b.get(i + 1).is_some_and(u8::is_ascii_digit)
            {
                digits.push('.');
                decimals = Some(0);
                i += 1;
            } else {
                break;
            }
        }
        if text[end..].contains(|c: char| c.is_ascii_digit()) {
            return None;
        }
        let value: f64 = digits.parse().ok()?;
        Some(Number {
            value: if neg { -value } else { value },
            at: [if neg { start - 1 } else { start }, end],
            decimals: decimals.unwrap_or(0),
            group,
        })
    }

    /// `text`, the one this number was read from, with `v` in its place,
    /// written as it is.
    fn show(&self, v: f64, text: &str) -> String {
        let n = format!("{:.*}", self.decimals, v.abs());
        let (int, frac) = n.split_once('.').unwrap_or((&n, ""));
        let mut out = String::with_capacity(text.len() + 4);
        out.push_str(&text[..self.at[0]]);
        if v < 0.0 && n.bytes().any(|d| d != b'0' && d != b'.') {
            out.push('-');
        }
        for (k, d) in int.chars().enumerate() {
            if k > 0
                && (int.len() - k).is_multiple_of(3)
                && let Some(g) = self.group
            {
                out.push(g);
            }
            out.push(d);
        }
        if !frac.is_empty() {
            out.push('.');
            out.push_str(frac);
        }
        out.push_str(&text[self.at[1]..]);
        out
    }
}

/// The effective look an element passes to its children.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Inherit {
    size: f32,
    ink: Rgba,
    text: Text,
    opacity: f32,
    text_shadows: TextShadows,
    text_stroke: TextStroke,
    decoration: Decoration,
}

struct El {
    node: Id,
    /// Its `disabled` prop.
    disabled: bool,
    /// It answers a press: an element with a press or tap handler.
    clickable: bool,
    /// A press moves it: its own classes under `active:`, or the pack's
    /// `Press.feedback` (`restyle`).
    squash: bool,
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
    /// The paragraph it is laid out in ([`para`]), `NONE` for none; and a
    /// paragraph's own members.
    inline: u32,
    para: Option<Box<para::Para>>,
    /// It waits in [`Tree::paras`] to be laid out again.
    queued: bool,
    /// A span's `ruby`: the annotation drawn over it, its leaf's text.
    ruby: Box<str>,
}

/// Theme colour role `role` (`gold`, `icon-outline`) as a toned icon
/// paints it: the theme's in the page's scheme, or the other one's,
/// resolved once a resolver (`cache` is cleared with it).
fn tone(
    classes: &ops::Classes,
    cache: &std::cell::RefCell<std::collections::HashMap<(Box<str>, bool), Rgba>>,
    role: &str,
    other: bool,
) -> Rgba {
    let key = (Box::<str>::from(role), other);
    if let Some(c) = cache.borrow().get(&key) {
        return *c;
    }
    let c = classes.tone(role, other);
    cache.borrow_mut().insert(key, c);
    c
}

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
            (im.texture, im.size) = textures.map_or((0, [0.0; 2]), |f| f(self.handle));
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
    /// The rounded overflow clip each element's children paint in, its
    /// own or its nearest clipping ancestor's: its index + 1 in
    /// [`Paint::rounds`] (its padding box, inner corner radii and
    /// transform), 0 none.
    rounds: Vec<u32>,
    /// The elements with an anchor, in no order (a dead or unanchored one
    /// leaves at the next [`Tree::place_anchors`]).
    anchored: Vec<u32>,
    /// [`Tree::settle_anchors`]'s order and the boxes it placed, kept.
    anchor_order: Vec<u32>,
    anchor_placed: Vec<[f32; 4]>,
    /// The names node anchors target, each once ([`AnchorTarget::Node`]).
    names: Vec<Box<str>>,
    /// The elements with a `spotlight`, in no order (a dead or cleared one
    /// leaves at the next [`Tree::present`]).
    spotlights: Vec<u32>,
    /// Each text leaf's layout and glyph quads ([`crate::text`]).
    texts: crate::text::Texts,
    /// The collection's revision texts were last measured at: when it
    /// moves (a pack or a locale's font loaded), every text is again.
    faces: Option<u64>,
    /// The atlas plots the last paint's glyph quads read: marked used on
    /// each frame that keeps that paint, so they are not emptied under it.
    plots: Vec<u32>,
    /// Device pixels a CSS px of the surface ([`Tree::set_scale`]) and
    /// the player's text size ([`Tree::set_text_scale`]).
    scale: f32,
    text_scale: f32,
    /// Characters text was laid out with that no face has, each once,
    /// until [`Tree::take_missing`]: the host loads a font pack that
    /// covers them (docs/ui.md §3.4), and the text draws in it once it is
    /// in the collection.
    missing: Vec<char>,
    /// What the host logs once ([`Tree::take_warnings`]), and the nodes
    /// whose filters were drawn unfiltered for want of layers, each said
    /// once.
    warnings: Vec<String>,
    warned_layers: Vec<u32>,
    /// The screen transition cover [`Tree::set_cover`] gave this frame.
    cover: Option<(crate::cover::Cover, bool)>,
    /// The pad's virtual cursor ([`Tree::virtual_cursor`]), and its
    /// shadow, ink and focus colours (the pack's `bg-bg`, `text-fg` and
    /// `bg-focus`, read with the class resolver).
    vcursor: vcursor::State,
    cursor_colors: [Rgba; 3],
    /// Paragraphs to lay out again, and whether they are being; the ones
    /// warned of a box they cannot hold in a line.
    paras: Vec<u32>,
    composing: bool,
    warned_paras: Vec<u32>,
    /// The selection in a `select-text` text, and the press making it.
    select: Option<para::Selected>,
    select_press: Option<para::SelectPress>,
    /// Each element's transform in `drawn.transforms` (its own composed
    /// with its ancestors'; 0 none), what its children compose onto.
    xfs: Vec<u32>,
    /// Each element's mask in the last paint, its own or its nearest
    /// masked ancestor's: a gradient of [`Tree::drawn`] plus 1, 0 none.
    masks: Vec<u32>,
    /// Each element's colour matrix in the last paint, its own composed
    /// over its ancestors': one of [`Tree::drawn`]'s filters plus 1, 0 none.
    filters: Vec<u32>,
    /// The transforms of the last paint, which `xfs` index: [`Tree::drawn`]
    /// is the host's to take, and the probe reads where a text was drawn
    /// after it has.
    painted_transforms: Vec<[f32; 8]>,
    /// The host's: an image's source (a texture handle) as the slot it
    /// draws, read as the tree paints, so a replaced texture follows.
    textures: Option<Textures>,
    /// The live elements depth first, a parent before its children, made
    /// again when the structure changes; `el_of` maps
    /// an arena node to its element, `walk` is its stack.
    order: Vec<u32>,
    ordered: bool,
    el_of: Vec<u32>,
    walk: Vec<Id>,
    /// Compositing and hit order, with the reusable quad reorder buffer.
    stacking: stacking::Stacking,
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
    /// The time the cascade ranks are for, and how many children of each
    /// parent with a `stagger-*` entered then ([`ops`]'s `cascade`).
    cascade: (f64, Vec<(u32, u32)>),
    /// The texts rolling to a new number (`transition-count`).
    counts: Vec<Count>,
    /// Elements with a `glide-<name>` that left the screen ([`ops`]'s
    /// `glide_from`).
    glides: Vec<Left>,
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
    /// The theme colours toned icons have asked for, by role and whether
    /// in the other scheme, resolved once a resolver ([`Tree::tone`]).
    tone_colors: std::cell::RefCell<std::collections::HashMap<(Box<str>, bool), Rgba>>,
}

/// What an element passes down in a frame, as motion shows it: its
/// opacity times its parent's, its ink, text shadow and text outline (each
/// its own, else its parent's), and whether it is in an exit, which
/// nothing hits.
#[derive(Clone, Copy)]
struct Shown {
    alpha: f32,
    ink: Rgba,
    text_shadows: TextShadows,
    text_stroke: TextStroke,
    out: bool,
    /// A press, a hover and the cursor reach it (`pointer-events-*`).
    touch: bool,
    /// Its icons' paint (`fill-*`); `None` the ink.
    fill: Option<Rgba>,
}

mod input;
mod ops;
mod para;
mod particles;
mod stacking;
mod thumb;
mod vcursor;
pub use input::{Cue, Play, SoundTable};
pub use ops::{EVENTS, Event as OpEvent, Fired, MAGIC, REFUSED, Resolve};
pub use vcursor::{Pad as CursorPad, Pointer as CursorPointer, VirtualCursorDesc};

/// An image's source, a texture handle as the game holds it, as its
/// texture's slot (`og_draw::TextureHandle::slot`; 0 none) and its size in
/// texels (0 unknown) ([`Tree::set_textures`]).
pub type Textures = Box<dyn Fn(f64) -> (u32, [f32; 2])>;

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
                text_shadows: TextShadows::default(),
                text_stroke: TextStroke::default(),
                decoration: Decoration::default(),
            },
            els: Vec::new(),
            arena: Arena::new(),
            paint: true,
            insets: [0.0; 4],
            drawn: Paint::default(),
            sem: crate::a11y::Semantics::default(),
            abs: Vec::new(),
            cuts: Vec::new(),
            rounds: Vec::new(),
            anchored: Vec::new(),
            anchor_order: Vec::new(),
            anchor_placed: Vec::new(),
            names: Vec::new(),
            spotlights: Vec::new(),
            texts: Default::default(),
            faces: None,
            plots: Vec::new(),
            scale: 1.0,
            text_scale: 1.0,
            xfs: Vec::new(),
            masks: Vec::new(),
            filters: Vec::new(),
            painted_transforms: Vec::new(),
            textures: None,
            order: Vec::new(),
            ordered: false,
            el_of: Vec::new(),
            walk: Vec::new(),
            stacking: Default::default(),
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
            cascade: (f64::NAN, Vec::new()),
            counts: Vec::new(),
            glides: Vec::new(),
            gone: Vec::new(),
            input: input::Input::new(),
            last_pointer: (f32::NAN, f32::NAN, 0),
            paints: 0,
            particles: Default::default(),
            tone_colors: Default::default(),
            missing: Vec::new(),
            warnings: Vec::new(),
            warned_layers: Vec::new(),
            cover: None,
            vcursor: Default::default(),
            cursor_colors: [[0.0, 0.0, 0.0, 0.45], [1.0; 4], [0.9, 0.9, 0.95, 1.0]],
            paras: Vec::new(),
            composing: false,
            warned_paras: Vec::new(),
            select: None,
            select_press: None,
        }
    }

    pub fn len(&self) -> usize {
        self.els.len()
    }

    pub fn is_empty(&self) -> bool {
        self.els.is_empty()
    }

    /// `e`'s text; an image's is its source. In a `transition-count` text
    /// drawn already, a number that changes rolls there on its curve: its
    /// text (what the probe and a screen reader read) is the new one at
    /// once, and what draws counts up or down to it ([`Tree::roll`]).
    fn set_text(&mut self, e: u32, text: &str) {
        if self.els[e as usize].kind == Kind::Image {
            return self.set_src(e, text);
        }
        let el = &self.els[e as usize];
        if &*el.text == text {
            return;
        }
        let rolls = el.kind == Kind::Label
            && el.entering.is_none()
            && el.parent != NONE
            && self.els[el.parent as usize].record.count
            && !self.motion.reduced();
        // A number written short (`0`) rolls written as the long one was;
        // one whose words or unit change (`999` to `1.2K`) jumps.
        let old = Number::of(&el.text);
        let group = old.and_then(|n| n.group);
        let to = Number::of(text).map(|n| Number {
            group: n.group.or(group),
            ..n
        });
        let same = match (old, to) {
            (Some(o), Some(to)) => {
                el.text[..o.at[0]] == text[..to.at[0]] && el.text[o.at[1]..] == text[to.at[1]..]
            }
            _ => false,
        };
        let shown = self.counts.iter().position(|c| c.e == e);
        let from = match shown {
            Some(i) => Some(self.counts.swap_remove(i).now),
            None => Number::of(&el.text).map(|n| n.value),
        };
        let el = &mut self.els[e as usize];
        el.text.clear();
        el.text.push_str(text);
        match (from, to) {
            (Some(from), Some(to)) if rolls && from != to.value && same => {
                self.counts.push(Count {
                    e,
                    from,
                    now: from,
                    to,
                    since: self.time,
                })
            }
            _ => {
                if let Some(leaf) = el.leaf {
                    self.arena.set_text(leaf, text);
                }
            }
        }
        self.para_touch(e);
        self.paint = true;
    }

    /// Draws each rolling number at its place on its text's curve, and
    /// lands it on its text when the curve comes to rest.
    fn roll(&mut self) {
        let mut i = 0;
        while i < self.counts.len() {
            let c = &self.counts[i];
            let el = &self.els[c.e as usize];
            let spec = self.els[el.parent as usize].record.motion;
            let (p, rest) = crate::motion::Motion::progress(&spec, (self.time - c.since) as f32);
            let done = rest || el.dead || self.motion.reduced();
            let now = c.from + (c.to.value - c.from) * p as f64;
            let text = if done {
                el.text.clone()
            } else {
                c.to.show(now, &el.text)
            };
            if let Some(leaf) = el.leaf {
                self.arena.set_text(leaf, &text);
            }
            let e = c.e;
            self.para_touch(e);
            if done {
                self.counts.swap_remove(i);
            } else {
                self.counts[i].now = now;
                i += 1;
            }
        }
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
            if matches!(target, AnchorTarget::Node(_)) {
                continue;
            }
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
            if shown != a.shown || at != a.at {
                (a.shown, a.at) = (shown, at);
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
    fn named(&self, name: &str) -> Option<u32> {
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
    fn laid_box(&self, e: u32) -> Option<[f32; 4]> {
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
    fn mark_lit(&mut self) -> bool {
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
    fn hole(&self, e: u32) -> Option<([f32; 4], [f32; 4])> {
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
    fn in_hole(&self, e: u32, x: f32, y: f32) -> Option<bool> {
        let ([hx, hy, hw, hh], r) = self.hole(e)?;
        let at = [x - hx - hw / 2.0, y - hy - hh / 2.0];
        Some(og_draw::paint::round_box_distance(at, [hw / 2.0, hh / 2.0], r) < 0.0)
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
            size: r
                .font_size_em
                .map_or(if r.text.sized { r.font_size } else { from.size }, |em| {
                    em * from.size
                }),
            ink: if r.inked { r.skin.ink } else { from.ink },
            text: r.text.over(&from.text),
            opacity: from.opacity * r.skin.opacity,
            text_shadows: if r.shadowed {
                r.skin.text_shadows
            } else {
                from.text_shadows
            },
            text_stroke: if r.stroked {
                r.skin.text_stroke
            } else {
                from.text_stroke
            },
            decoration: if r.skin.decoration.line != 0 {
                r.skin.decoration
            } else {
                from.decoration
            },
        };
        let mut layout = r.layout_in(self.insets);
        // A `Scroll` is a scroll container on both axes, as `overflow:
        // auto` is: its automatic minimum is 0, so its content does not
        // size it. An anchored node is out of flow: the host places it.
        let scroller = self.input.per(e).is_some_and(|p| p.scroller);
        // A scroll bar's thumb is as long as the tree fitted it, and
        // hidden while its scroller has nothing to scroll.
        let thumb = self
            .input
            .per(e)
            .filter(|p| !p.thumb.is_empty())
            .map(|p| (p.thumb_len, p.thumb_horizontal));
        if el.hidden > 0 || el.anchored().is_some() || scroller || thumb.is_some() {
            let mut s = taffy::Style::from(&layout);
            if scroller {
                s.overflow = taffy::Point {
                    x: taffy::Overflow::Scroll,
                    y: taffy::Overflow::Scroll,
                };
            }
            if let Some((len, horizontal)) = thumb {
                let len = taffy::Dimension::length(len);
                if horizontal {
                    s.size.width = len;
                } else {
                    s.size.height = len;
                }
                s.flex_shrink = 0.0;
                if len == taffy::Dimension::length(0.0) {
                    s.display = taffy::Display::None;
                }
            }
            if el.hidden > 0 {
                s.display = taffy::Display::None;
            } else if el.anchored().is_some() {
                s.position = taffy::Position::Absolute;
            }
            layout = rune_taffy::CompactStyle::from(&s);
        }
        if el.inline != NONE {
            let mut s = taffy::Style::from(&layout);
            Tree::member_style(el.kind, &mut s);
            layout = rune_taffy::CompactStyle::from(&s);
        }
        let in_para = el.inline != NONE || el.para.is_some();
        let (node, leaf, space) = (el.node, el.leaf, r.space);
        self.arena.set_style(node, layout);
        self.arena.set_space(node, space);
        if let Some(leaf) = leaf {
            // The player's text size scales the size and what is a
            // length with it, before the text is laid out.
            let k = self.text_scale;
            let field = self.input.per(e).filter(|p| p.input);
            let one_line = field.is_some_and(|p| !p.multiline);
            // `rows`: a field at least that many lines tall.
            let rows = field.map_or(0, |p| p.rows);
            if field.is_some() {
                let mut s = taffy::Style::default();
                if rows > 0 {
                    let line = eff.text.line_height(eff.size) * k;
                    let line = if line < 0.0 { eff.size * k * 1.2 } else { line };
                    s.min_size.height = taffy::LengthPercentageAuto::length(f32::from(rows) * line);
                }
                self.arena
                    .set_style(leaf, rune_taffy::CompactStyle::from(&s));
            }
            // A span's leaf is its ruby: half its size, on one line, set
            // solid.
            let ruby =
                self.els[e as usize].kind == Kind::Text && self.els[e as usize].inline != NONE;
            let eff = if ruby {
                let mut r = eff;
                r.size *= RUBY;
                r.text.line = 1.0;
                r.text.line_px = false;
                r.text.nowrap = true;
                r
            } else {
                eff
            };
            self.arena.set_run(
                leaf,
                Run {
                    size: eff.size * k,
                    tracking: eff.text.tracking(eff.size) * k,
                    line: eff.text.line_height(eff.size) * k,
                    weight: eff.text.weight,
                    nowrap: eff.text.nowrap || one_line,
                    clamp: eff.text.clamp,
                    family: eff.text.family,
                    italic: eff.text.italic,
                    variant: eff.text.variant,
                    wrap: eff.text.wrap,
                    hyphens: match eff.text.hyphens {
                        nsh_tailwind::Hyphens::Manual => og_text::layout::Hyphens::Manual,
                        nsh_tailwind::Hyphens::Auto => og_text::layout::Hyphens::Auto,
                        nsh_tailwind::Hyphens::None => og_text::layout::Hyphens::None,
                    },
                    ellipsis: eff.text.overflow == 1,
                    align: eff.text.align,
                    field: one_line,
                    // A field's text stays in lines: its caret runs along them.
                    vertical: eff.text.vertical && field.is_none(),
                },
            );
        }
        let el = &self.els[e as usize];
        // An icon's or an image's size is its inline box's.
        let changed = el.eff != eff || matches!(el.kind, Kind::Icon | Kind::Image);
        self.els[e as usize].eff = eff;
        // A member's style is its paragraph's span.
        if in_para && changed {
            self.para_touch(e);
        }
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

    /// Whether the tree's motion is still on its way to rest as of the
    /// last frame ([`crate::motion::Motion::settling`]): what a capture
    /// waits out before it judges a frame.
    pub fn settling(&self) -> bool {
        self.motion.settling() || !self.counts.is_empty()
    }

    /// The characters laid-out text drew that no face has, since the last
    /// call: a frame that found none allocates nothing.
    pub fn take_missing(&mut self) -> Vec<char> {
        std::mem::take(&mut self.missing)
    }

    /// What the paint could not draw as asked since the last call (a
    /// filter past the frame's layers), for the host's log.
    pub fn take_warnings(&mut self) -> Vec<String> {
        std::mem::take(&mut self.warnings)
    }

    /// Device pixels a CSS px of the surface: glyphs raster at their
    /// drawn size, on its pixels.
    pub fn set_scale(&mut self, scale: f32) {
        let scale = if scale.is_finite() && scale > 0.0 {
            scale
        } else {
            1.0
        };
        if self.scale != scale {
            self.scale = scale;
            self.paint = true;
        }
    }

    /// The player's text size (`UiDesc.textScale`): every text's font
    /// size, and its line height and spacing with it, times `scale`. A
    /// change lays every text out again.
    pub fn set_text_scale(&mut self, scale: f32) {
        if self.text_scale == scale {
            return;
        }
        self.text_scale = scale;
        for e in 0..self.els.len() as u32 {
            if !self.els[e as usize].dead && self.els[e as usize].leaf.is_some() {
                self.inherit(e);
                // A paragraph's spans are its runs' sizes.
                if self.els[e as usize].inline != NONE {
                    self.para_touch(e);
                }
            }
        }
        self.paint = true;
    }

    /// The paragraph direction and the locale (BCP 47) texts are laid out
    /// in: the base direction of each, its `locl` forms and the faces its
    /// characters fall back to. A change lays every text out again.
    pub fn set_writing(&mut self, direction: Direction, locale: &str) {
        if self.texts.set_writing(direction, locale) {
            self.arena.remeasure();
            self.paint = true;
        }
    }

    /// Lays out what changed and paints. Texts are laid out from `fonts`
    /// and their glyphs rastered into `atlas`; `icons` draws icons.
    pub fn frame(&mut self, fonts: &mut Collection, atlas: &mut Atlas, icons: &IconAtlas) -> Frame {
        let (laid_out, painted) = self.present(fonts, atlas, icons);
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
    pub fn present(
        &mut self,
        fonts: &mut Collection,
        atlas: &mut Atlas,
        icons: &IconAtlas,
    ) -> (bool, bool) {
        for e in self.entering.drain(..) {
            self.els[e as usize].entering = None;
        }
        // Input since the last batch may have changed a group's states.
        self.refresh_data();
        let moving = self.motion.tick_at(self.time) | !self.counts.is_empty();
        self.roll();
        self.settle_exits();
        let (w, h) = self.size;
        if self.faces != Some(fonts.revision()) {
            self.faces = Some(fonts.revision());
            self.arena.remeasure();
            self.paint = true;
        }
        self.compose_paras();
        let texts = &mut self.texts;
        let laid_out = self.arena.layout(w, h, &mut |id, t, run, avail| {
            texts.measure(fonts, id, t, run, avail)
        });
        if laid_out {
            self.clamp_scrolls();
        }
        // A scroll bar's thumb fits its scroller's share shown: what that
        // changes lays out again.
        let laid_out = if laid_out && self.fit_thumbs() {
            let texts = &mut self.texts;
            self.arena.layout(w, h, &mut |id, t, run, avail| {
                texts.measure(fonts, id, t, run, avail)
            });
            self.clamp_scrolls();
            true
        } else {
            laid_out
        };
        // A box that moved slides there when it transitions its layout.
        let arena = &self.arena;
        self.motion.relayout(|n| arena.box_of(n));
        if !self.anchored.is_empty() {
            self.settle_anchors();
        }
        // A spotlight's `data-lit` follows whether its node shows; what
        // its classes and its group's read of it change lays out again.
        if !self.spotlights.is_empty() && self.mark_lit() {
            self.refresh_data();
            let texts = &mut self.texts;
            if self.arena.layout(w, h, &mut |id, t, run, avail| {
                texts.measure(fonts, id, t, run, avail)
            }) {
                self.clamp_scrolls();
            }
            let arena = &self.arena;
            self.motion.relayout(|n| arena.box_of(n));
            if !self.anchored.is_empty() {
                self.settle_anchors();
            }
            self.paint = true;
        }
        // A `<Particles>` with anything still to draw paints every frame.
        let painted = laid_out || self.paint || moving || self.particles.live;
        if painted {
            self.particles.live = false;
            self.paint = false;
            self.repaint(fonts, atlas, icons);
            self.paints += 1;
            let drawn = self.drawn_boxes();
            self.sem.set_drawn(drawn);
        } else {
            for &plot in &self.plots {
                atlas.touch(plot);
            }
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

    fn repaint(&mut self, fonts: &mut Collection, atlas: &mut Atlas, icons: &IconAtlas) {
        /// A line's stroke: its own colour faded, else the ink (already faded).
        fn s_line_color(stroke: &Option<Rgba>, ink: Rgba, alpha: f32) -> Rgba {
            stroke.map_or(ink, |mut c| {
                c[3] *= alpha;
                c
            })
        }
        let (vw, vh) = self.size;
        let screen = [0.0, 0.0, vw, vh];
        self.follow_caret();
        let p = &mut self.drawn;
        p.clear();
        (p.width, p.height) = (vw, vh);
        self.plots.clear();
        let scale = self.scale;
        self.order();
        self.stacking.begin(&self.els, &self.order);
        self.abs.clear();
        self.abs.resize(self.els.len(), (0.0, 0.0, false, [0.0; 4]));
        const ALL: [f32; 4] = [f32::MIN, f32::MIN, f32::MAX, f32::MAX];
        self.cuts.clear();
        self.cuts.resize(self.els.len(), ALL);
        self.rounds.clear();
        self.rounds.resize(self.els.len(), 0);
        self.shown.clear();
        let top = Shown {
            alpha: 1.0,
            ink: self.root.ink,
            text_shadows: self.root.text_shadows,
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
        self.xfs.resize(self.els.len(), 0);
        self.masks.clear();
        self.masks.resize(self.els.len(), 0);
        self.filters.clear();
        self.filters.resize(self.els.len(), 0);
        // The nodes painting for the first time with a `sound` to play.
        let mut sounding: Vec<u32> = Vec::new();
        // The groups open while their subtrees paint, innermost last, and
        // the layers the paint may still draw.
        let mut open: Vec<Open> = Vec::new();
        // The boxes of what the last filter group listed holds, each a
        // group it merged (`close_group`).
        let mut members: Vec<[f32; 4]> = Vec::new();
        let mut room = Room {
            layers: 0,
            area: 0.0,
            limit: og_draw::paint::GROUP_SCREENS * vw * vh,
        };
        for k in 0..self.order.len() {
            let e = self.order[k] as usize;
            // What it is not under is painted: those groups close.
            while open
                .last()
                .is_some_and(|o| !descends(&self.els, e as u32, o.e))
            {
                let o = open.pop().expect("open");
                close_group(
                    &mut self.drawn,
                    &o,
                    screen,
                    (!self.stacking.reorders()).then_some(&mut members),
                );
            }
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
                let [sx, sy] = self.moved_by(e as u32);
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
                        text_shadows: eff.text_shadows,
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
            let text_shadows = if r.shadowed {
                s.text_shadows
            } else {
                from.text_shadows
            };
            let text_stroke = if r.stroked {
                s.text_stroke
            } else {
                from.text_stroke
            };
            self.shown[e] = Shown {
                alpha,
                ink: own_ink,
                text_shadows,
                text_stroke,
                out: from.out
                    || self.motion.exiting(el.node)
                    || self.leaving.iter().any(|l| l.0 == e as u32),
                touch: r.touch.unwrap_or(from.touch),
                fill: r.fill.unwrap_or(from.fill),
            };
            let visible = pv
                && !el.dead
                && el.hidden == 0
                && !r.hidden
                && alpha > 0.0
                && r.layout.display() != taffy::Display::None;
            // An icon or an image in a paragraph is drawn where its line
            // put it, and not where a cut line dropped it.
            let at = match el.kind {
                Kind::Icon | Kind::Image if el.inline != NONE => Some(self.inline_box(e as u32)),
                Kind::Text if el.inline != NONE && !el.ruby.is_empty() => {
                    Some(self.ruby_at(e as u32))
                }
                _ => None,
            };
            // A run in a paragraph is drawn by the paragraph.
            let visible = visible
                && at.is_none_or(|a| a.is_some())
                && !(el.inline != NONE && el.kind == Kind::Label);
            let (x, y) = match at.flatten() {
                Some([ix, iy]) => (ix, iy),
                None => (px + b.x, py + b.y),
            };
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
            // The rounded clip it is drawn in, and the one it sets: CSS
            // clips overflow to the padding box with the inner radii.
            let pround = match el.arena_parent {
                _ if el.anchor.is_some() => 0,
                NONE => 0,
                p => self.rounds[p as usize],
            };
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
            self.filters[e] = if el.arena_parent == NONE {
                0
            } else {
                self.filters[el.arena_parent as usize]
            };
            if !visible {
                continue;
            }
            if !self.input.per[e].cue.is_empty() {
                sounding.push(e as u32);
            }
            // A `knob` node rides its on-screen control's knob.
            let [kx, ky] = self.knob_of(e as u32);
            // A scroll bar's thumb rides its scroller's offset.
            let [hx, hy] = self.thumb_shift(e as u32);
            let mut t = s.transform;
            t.translate = [t.translate[0] + kx + hx, t.translate[1] + ky + hy];
            let hole = self.hole(e as u32);
            // Its `blur-*` and `drop-shadow-*` draw it and its subtree
            // through a layer of their own, while the paint has room for
            // one; past it, unfiltered, said once.
            let group =
                r.effect_count > 0 && room.take(reached([x, y, b.w, b.h], r.effects(), screen));
            if r.effect_count > 0 && !group && !self.warned_layers.contains(&el.node) {
                self.warned_layers.push(el.node);
                let name = &self.input.per[e].name;
                self.warnings.push(format!(
                    "node {}{}{} draws its blur and drop shadows unfiltered: the frame's other filters already take {} layers or {} screens of them",
                    el.node,
                    if name.is_empty() { "" } else { " " },
                    name,
                    og_draw::paint::GROUP_LAYERS,
                    og_draw::paint::GROUP_SCREENS,
                ));
            }
            // Its group's composite takes its opacity, its mask and its
            // ancestors' colour filters (CSS's order: the filter, then the
            // mask, then opacity); inside the layer it and its subtree
            // draw as they are, their own colour filters first.
            let outer_alpha = alpha;
            let alpha = if group {
                self.shown[e].alpha = 1.0;
                1.0
            } else {
                alpha
            };
            // A selected text's selection, and a paragraph's members' inks
            // and decorations, by ink ([`para`]).
            let selected = self.selected_in(e as u32);
            let span_looks: Vec<(Rgba, Decoration)> = el.para.as_ref().map_or(Vec::new(), |para| {
                para.members
                    .iter()
                    .map(|&m| {
                        let m = &self.els[m as usize];
                        let mut c = m.eff.ink;
                        c[3] *= alpha;
                        (c, m.eff.decoration)
                    })
                    .collect()
            });
            let p = &mut self.drawn;
            let start = p.quads.len();
            // Its own mask replaces an ancestor's over its subtree.
            if let Some(g) = &r.mask {
                let area = [x, y, b.w, b.h];
                let gi = p.shaped_gradient(paint_gradient(g, area), g.stops().iter().copied());
                self.masks[e] = gi + 1;
            }
            let (outer_mask, outer_filter) = (self.masks[e], self.filters[e]);
            if group {
                (self.masks[e], self.filters[e]) = (0, 0);
            }
            // Its own filters, then its ancestors' over them (CSS's filter
            // on each of the nested groups).
            if let Some(own) = &r.filter {
                let m = match self.filters[e] {
                    0 => *own,
                    k => og_draw::paint::color_then(own, &p.filters[k as usize - 1]),
                };
                p.filters.push(m);
                self.filters[e] = p.filters.len() as u32;
            }
            let xf = transformed(p, pxf, &t, [x, y, b.w, b.h]);
            self.xfs[e] = xf;
            self.rounds[e] = pround;
            if cuts {
                let outer = radii(s.radius, b.w, b.h);
                let inner = [
                    outer[0] - bt.max(bl),
                    outer[1] - bt.max(br),
                    outer[2] - bb.max(br),
                    outer[3] - bb.max(bl),
                ]
                .map(|k| k.max(0.0));
                if inner.iter().any(|&k| k > 0.0) {
                    p.rounds.push(og_draw::paint::PaintRound {
                        rect: padding,
                        radius: inner,
                        xf,
                        invert: 0,
                        _pad: [0; 2],
                    });
                    self.rounds[e] = p.rounds.len() as u32;
                }
            }
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
            // Its backdrop filter: a filtered copy of what is under it,
            // drawn under its box, rounded as it is.
            let mut backdrop = None;
            if s.backdrop_blur > 0.0 || r.backdrop_filter.is_some() {
                let first = p.effects.len() as u32;
                if s.backdrop_blur > 0.0 {
                    p.effects.push(PaintEffect::Blur(s.backdrop_blur));
                }
                if let Some(m) = &r.backdrop_filter {
                    p.filters.push(*m);
                    p.effects
                        .push(PaintEffect::Matrix(p.filters.len() as u32 - 1));
                }
                let at = p.quads.len() as u32;
                p.groups.push(PaintGroup {
                    quad: at,
                    end: at + 1,
                    first,
                    count: p.effects.len() as u32 - first,
                    xf,
                    backdrop: true,
                });
                p.quads.push(quad(
                    PaintKind::Group,
                    rect,
                    clip,
                    [1.0, 1.0, 1.0, outer_alpha],
                    radius,
                    [0.0; 4],
                    p.groups.len() as u32 - 1,
                    og_draw::paint::GROUP_SHAPED,
                ));
                backdrop = Some(at as usize);
            }
            if group {
                let first = p.effects.len() as u32;
                p.effects.extend_from_slice(r.effects());
                open.push(Open {
                    e: e as u32,
                    quad: p.quads.len(),
                    first,
                    count: u32::from(r.effect_count),
                    xf,
                    composite: PaintQuad {
                        mask: outer_mask,
                        filter: outer_filter,
                        round: pround,
                        ..quad(
                            PaintKind::Group,
                            [0.0; 4],
                            clip,
                            [1.0, 1.0, 1.0, outer_alpha],
                            [0.0; 4],
                            [0.0; 4],
                            0,
                            0,
                        )
                    },
                });
                // Written when its subtree has painted (`close_group`).
                p.quads.push(open.last().expect("pushed").composite);
            }
            let faded = |mut c: Rgba| {
                c[3] *= alpha;
                c
            };
            let im = &match &el.image {
                Some(props) => props.over(&s.image, self.textures.as_ref()),
                None => s.image,
            };
            let image = og_draw::TextureHandle::from_slot(im.texture).filter(|_| im.wipe > 0.0);
            // CSS `border-image`: a nine-sliced frame is the box's edge.
            let framed = image.is_some() && im.slice != [0.0; 4];
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
            // A framed box's outline follows its frame, as a browser's
            // follows `border-radius`: the frame's silhouette in the
            // outline's colour, grown by its width (and a positive offset),
            // under the box. It is always outside, since the frame covers
            // what is inside.
            let o = &s.outline;
            if let Some(t) = image.filter(|_| framed && o.width > 0.0 && o.color[3] > 0.0) {
                let out = o.width + o.offset.max(0.0);
                let grown = [x - out, y - out, b.w + 2.0 * out, b.h + 2.0 * out];
                let width = if im.width == [0.0; 4] {
                    im.slice
                } else {
                    im.width
                };
                let fit = (grown[2] / (width[1] + width[3]))
                    .min(grown[3] / (width[0] + width[2]))
                    .min(1.0);
                p.quads.push(PaintQuad {
                    uv: im.uv,
                    ..quad(
                        PaintKind::Image,
                        grown,
                        clip,
                        faded(o.color),
                        width.map(|w| w * fit),
                        im.slice,
                        t.to_bits(),
                        og_draw::paint::IMAGE_FILL,
                    )
                });
            }
            // A fill-tinted image takes the fill's colour in its place.
            let fill_tints = image.is_some() && im.tint == crate::record::Tint::Fill;
            if let Some(([hx, hy, hw, hh], hr)) = hole.filter(|_| s.fill[3] > 0.0) {
                // A spotlight's ground covers the surface but its hole,
                // the lit node's border box rounded as it is: one quad
                // cut by the hole's round, inverted.
                p.rounds.push(og_draw::paint::PaintRound {
                    rect: [hx, hy, hx + hw, hy + hh],
                    radius: hr,
                    xf: 0,
                    invert: 1,
                    _pad: [0; 2],
                });
                p.quads.push(PaintQuad {
                    round: p.rounds.len() as u32,
                    ..quad(
                        PaintKind::Solid,
                        screen,
                        screen,
                        faded(s.fill),
                        [0.0; 4],
                        [0.0; 4],
                        0,
                        0,
                    )
                });
            } else if s.fill[3] > 0.0 && !fill_tints {
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
                // CSS border-image: insets that together overrun the box
                // shrink in proportion until they fit, so a frame's
                // corners never cross on a box smaller than its art.
                let fit = (b.w / (width[1] + width[3]))
                    .min(b.h / (width[0] + width[2]))
                    .min(1.0);
                let width = width.map(|w| w * fit);
                let color = match im.tint {
                    crate::record::Tint::Own => [1.0, 1.0, 1.0, alpha],
                    crate::record::Tint::Ink => faded(own_ink),
                    crate::record::Tint::Fill => faded(s.fill),
                };
                let flags = (u32::from(im.fill) * og_draw::paint::IMAGE_FILL)
                    | (u32::from(im.tile) * og_draw::paint::IMAGE_TILE);
                let cut = if im.wipe < 1.0 { 1.0 - im.wipe } else { 0.0 };
                // `object-*`: the source at its size in the box, placed
                // and cropped to it.
                let (rect, uv) = match r.object {
                    Some(object) if im.slice == [0.0; 4] => fitted(object, rect, im.uv, im.size),
                    _ => (rect, im.uv),
                };
                p.quads.push(PaintQuad {
                    uv,
                    wipe: [cut, im.wipe_start, f32::from(u8::from(im.wipe_ccw)), 0.0],
                    mask: 0,
                    filter: 0,
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
            // The frame takes the border's place: the border keeps only
            // its room in the layout.
            if !framed
                && (s.border_color[3] > 0.0 || sides != 0)
                && s.border.iter().any(|&w| w > 0.0)
            {
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
            if !framed && o.width > 0.0 && o.color[3] > 0.0 {
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
                    Some((IconCell::Face(face, index), _)) => {
                        // Its fields, in the glyph atlas: put back when the
                        // atlas dropped them, drawn nothing this frame when
                        // it has no room for them yet. A toned icon is a
                        // sticker: its silhouette's shadow and outline, then
                        // each tone in its colour role.
                        let mut layers: Vec<(u32, [f32; 4], Rgba, f32)> = Vec::new();
                        match face.tones.of(index) {
                            Some(tones) => {
                                let [o, dx, dy] = face.tones.sticker.map(|v| v * size);
                                let at =
                                    |dx: f32, dy: f32| [square[0] + dx, square[1] + dy, size, size];
                                let role = |r: &str| {
                                    faded(tone(&self.classes, &self.tone_colors, r, el.other))
                                };
                                layers.push((index, at(dx, dy), role("icon-shadow"), o));
                                layers.push((index, square, role("icon-outline"), o));
                                for (g, r) in tones {
                                    let c = if r.is_empty() { icon_ink } else { role(r) };
                                    layers.push((*g, square, c, 0.0));
                                }
                            }
                            None => layers.push((index, square, icon_ink, 0.0)),
                        }
                        for (glyph, at, color, grow) in layers {
                            if color[3] <= 0.0 {
                                continue;
                            }
                            if let Some((q, plot)) = face.quad(atlas, glyph, at) {
                                if !self.plots.contains(&plot) {
                                    self.plots.push(plot);
                                }
                                p.quads.push(PaintQuad {
                                    color,
                                    clip,
                                    p: [grow, 0.0, q.p[2], q.p[3]],
                                    ..q
                                });
                            }
                        }
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
            // An `Input`'s own placeholder colour, else half its ink.
            let field = self.input.per(e as u32).filter(|f| f.input);
            // A one-line field's text scrolls inside its content box, with
            // room past its right edge for the caret standing there.
            let scroll = field.filter(|f| !f.multiline).map_or(0.0, |f| f.scroll_x);
            let clip = match field {
                Some(f) if !f.multiline => {
                    let [l, _, r, _] = self.arena.box_of(el.node).inset;
                    cut_to(clip, [x + l, y, x + b.w - r + 2.0, y + b.h])
                }
                _ => clip,
            };
            let mut text_ink = ink;
            if field.is_some_and(|f| f.value.is_empty()) {
                text_ink = s.placeholder.map_or_else(
                    || {
                        let mut c = ink;
                        c[3] *= 0.5;
                        c
                    },
                    faded,
                );
            }
            // What the leaf lays out is what draws: a rolling number's
            // place on its way, not yet its text.
            if let Some(leaf) = el.leaf
                && !el.text.is_empty()
                && let Some((drawn, run)) = self.arena.text_run(leaf).as_ref()
            {
                let run = *run;
                let lb = self.arena.box_of(leaf);
                // A transform that is more than a move draws distance
                // fields; any other text sits on whole device pixels,
                // where a move puts it.
                let [ma, mb, mc, md, tx, ty, _, _] = match xf {
                    0 => og_draw::paint::IDENTITY,
                    xf => p.transforms[xf as usize],
                };
                let sdf = [ma, mb, mc, md] != [1.0, 0.0, 0.0, 1.0];
                let snap = |v: f32, t: f32| match sdf {
                    true => v,
                    false => ((v + t) * scale).round() / scale - t,
                };
                let (lx, ly) = (snap(x + lb.x - scroll, tx), snap(y + lb.y, ty));
                // `text-overflow` cuts at the box that set it.
                let mut width = lb.w;
                if run.ellipsis
                    && let Some(right) =
                        overflow_edge(&self.els, &self.abs, &self.motion, &self.arena, e as u32)
                {
                    width = width.min(right - (x + lb.x)).max(0.0);
                }
                let sh = text_shadows;
                let st = text_stroke;
                // A raster per distinct blur among the layers that draw.
                let mut blurs = [0.0f32; crate::record::TEXT_SHADOWS];
                let mut n_blurs = 0u8;
                for (l, _) in sh.each().filter(|(l, _)| l.color[3] > 0.0) {
                    if !blurs[..usize::from(n_blurs)].contains(&l.blur) {
                        blurs[usize::from(n_blurs)] = l.blur;
                        n_blurs += 1;
                    }
                }
                let look = crate::text::Look {
                    scale,
                    field: sdf,
                    shadows: blurs,
                    shadow_count: n_blurs,
                    stroke: if st.color[3] > 0.0 {
                        st.width.max(0.0)
                    } else {
                        0.0
                    },
                    ink: [text_ink[0], text_ink[1], text_ink[2]]
                        .map(og_core::color::linear_to_srgb),
                };
                let g = self.texts.paint(
                    fonts,
                    atlas,
                    leaf,
                    drawn,
                    &run,
                    // Columns break at the box's height.
                    if run.vertical { lb.h } else { width },
                    &look,
                    &mut self.missing,
                );
                // A glyph the atlas had no room for draws next frame.
                self.paint |= g.incomplete;
                for &plot in self.texts.plots(leaf) {
                    if !self.plots.contains(&plot) {
                        self.plots.push(plot);
                    }
                }
                let g = self.texts.leaf(leaf).expect("painted");
                if let Some(f) = field
                    && self.input.focus_el() == e as u32
                {
                    if let Some((cs, ce)) = f.composing_display_range() {
                        let line_h = (1.0 * scale).round().max(1.0) / scale;
                        let ucolor = faded(own_ink);
                        g.selection(cs..ce, |[sx, sy, w, h]| {
                            p.quads.push(quad(
                                PaintKind::Solid,
                                [lx + sx, ly + sy + h - line_h, w, line_h],
                                clip,
                                ucolor,
                                [0.; 4],
                                [0.; 4],
                                0,
                                0,
                            ));
                        });
                    } else if f.anchor != f.caret {
                        let (s0, s1) = f.selection_range();
                        let (d0, d1) = (f.display_offset(s0), f.display_offset(s1));
                        let color = faded(s.selection.unwrap_or([0.1, 0.3, 0.8, 0.4]));
                        g.selection(d0..d1, |[sx, sy, w, h]| {
                            p.quads.push(quad(
                                PaintKind::Solid,
                                [lx + sx, ly + sy, w, h],
                                clip,
                                color,
                                [0.; 4],
                                [0.; 4],
                                0,
                                0,
                            ));
                        });
                    }
                }
                if let Some(range) = selected {
                    let color = faded(s.selection.unwrap_or([0.1, 0.3, 0.8, 0.4]));
                    g.selection(range, |[sx, sy, w, h]| {
                        p.quads.push(quad(
                            PaintKind::Solid,
                            [lx + sx, ly + sy, w, h],
                            clip,
                            color,
                            [0.; 4],
                            [0.; 4],
                            0,
                            0,
                        ));
                    });
                }
                let placed = |q: &PaintQuad, dx: f32, dy: f32, color: Rgba| PaintQuad {
                    rect: [
                        q.rect[0] + lx + dx,
                        q.rect[1] + ly + dy,
                        q.rect[2],
                        q.rect[3],
                    ],
                    clip,
                    color,
                    ..*q
                };
                // The shadows lowest, the last layer first as CSS paints
                // them, each a whole number of device pixels off, in its
                // colour over this text's ink.
                for (l, kind) in sh.each().rev().filter(|(l, _)| l.color[3] > 0.0) {
                    let Some(j) = look.blurs().iter().position(|&b| b == l.blur) else {
                        continue;
                    };
                    let mut c = TextShadows::color(l, kind, own_ink);
                    c[3] *= alpha;
                    let [dx, dy] = l.offset.map(|d| match sdf {
                        true => d,
                        false => (d * scale).round() / scale,
                    });
                    p.quads
                        .extend(g.shadow[j].iter().map(|q| placed(q, dx, dy, c)));
                }
                // The outline under the fill: the band the stroke's width
                // wide on each glyph's outline (`-webkit-text-stroke`).
                if look.stroke > 0.0 {
                    let mut c = st.color;
                    c[3] *= alpha;
                    p.quads
                        .extend(g.stroke.iter().map(|q| placed(q, 0.0, 0.0, c)));
                }
                // Each glyph in its span's ink; a text of one run, all in its own.
                let ink_of = |i: u16| match i {
                    0 => text_ink,
                    i => span_looks.get(usize::from(i) - 1).map_or(text_ink, |l| l.0),
                };
                p.quads.extend(
                    g.fill
                        .iter()
                        .zip(g.fill_ink.iter().copied().chain(std::iter::repeat(0)))
                        .map(|(q, i)| placed(q, 0.0, 0.0, ink_of(i))),
                );
                // A paragraph's lines are decorated span by span, each as
                // its run's style says, along the stretches its ink drew.
                let own = {
                    let mut c = own_ink;
                    c[3] *= alpha;
                    c
                };
                // A stretch's decoration and ink: a paragraph's span by
                // span along its inks, any other text's line by line.
                let rich = !span_looks.is_empty();
                let look_of = |i: u16| match i {
                    0 => (el.eff.decoration, own),
                    i => span_looks
                        .get(usize::from(i) - 1)
                        .map_or((el.eff.decoration, own), |&(c, d)| (d, c)),
                };
                // A column's text has no decoration lines yet.
                let decorated = !run.vertical
                    && if rich {
                        g.inks.iter().any(|&(i, _)| look_of(i).0.line != 0)
                    } else {
                        el.eff.decoration.line != 0
                    };
                if decorated {
                    // The lines the primary font gives, each a whole
                    // number of device pixels thick, on one.
                    let face = self.texts.decorations(fonts, leaf);
                    let px = |v: f32| (v * scale).round().max(1.0) / scale;
                    use crate::record::Decoration as D;
                    let g = self.texts.leaf(leaf).expect("painted");
                    let n = if rich { g.inks.len() } else { g.lines.len() };
                    for k in 0..n {
                        let (d, ink, [x0, base, w]) = if rich {
                            let (i, l) = g.inks[k];
                            let (d, c) = look_of(i);
                            (d, c, l)
                        } else {
                            (el.eff.decoration, own, g.lines[k])
                        };
                        if d.line == 0 {
                            continue;
                        }
                        let c = if d.color[3] > 0.0 {
                            let mut c = d.color;
                            c[3] *= alpha;
                            c
                        } else {
                            ink
                        };
                        let at = [
                            (D::UNDERLINE, face.underline),
                            (D::OVERLINE, [face.ascent, face.underline[1]]),
                            (D::LINE_THROUGH, face.through),
                        ];
                        for (bit, [above, thick]) in at {
                            if d.line & bit != 0 {
                                let th = px(if d.thickness > 0.0 {
                                    d.thickness
                                } else {
                                    thick
                                });
                                let top = if let (D::UNDERLINE, Some(offset)) = (bit, d.offset) {
                                    base + offset - th / 2.0
                                } else if bit == D::OVERLINE {
                                    base - above
                                } else {
                                    base - above - th / 2.0
                                };
                                let top = (top * scale).round() / scale;
                                let rect = [lx + x0, ly + top, w, th];
                                decorate(p, rect, clip, c, d.style);
                            }
                        }
                    }
                }
            }
            // The focused `Input`'s caret: a bar its line's height before
            // the character it stands at, in the text's ink.
            if let Some(f) = field
                && self.input.focus_el() == e as u32
                && (f.anchor == f.caret || !f.composing.is_empty())
                && let Some([cx, top, bar, height]) =
                    input::caret_bar(el, f, &self.texts, &self.arena, scale, self.text_scale)
            {
                let snap = |v: f32| (v * scale).round() / scale;
                let rect = [snap(x + cx), snap(y + top), bar, height];
                p.quads.push(quad(
                    PaintKind::Solid,
                    rect,
                    clip,
                    s.caret.map_or(ink, faded),
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
            // The rounded clip it is drawn in cuts what it drew but a
            // spotlight's shade, which its own hole cuts.
            if pround != 0 {
                for q in &mut self.drawn.quads[start..] {
                    if q.round == 0 {
                        q.round = pround;
                    }
                }
            }
            let (mask, filter) = (self.masks[e], self.filters[e]);
            if mask != 0 || filter != 0 {
                for q in &mut self.drawn.quads[start..] {
                    (q.mask, q.filter) = (mask, filter);
                }
            }
            // A backdrop takes its mask, and its ancestors' colour filters
            // as what is under it would.
            if let Some(at) = backdrop {
                let q = &mut self.drawn.quads[at];
                (q.mask, q.filter) = (outer_mask, outer_filter);
            }
            self.stacking.painted(e, start..self.drawn.quads.len());
        }
        while let Some(o) = open.pop() {
            close_group(
                &mut self.drawn,
                &o,
                screen,
                (!self.stacking.reorders()).then_some(&mut members),
            );
        }
        self.stacking
            .reorder(&mut self.drawn.quads, &mut self.drawn.groups);
        for e in sounding {
            let cue = std::mem::take(&mut self.input.per[e as usize].cue);
            self.sound(e, &cue);
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
    /// parent paints them in, holds the point, the last painted in its
    /// stacking context; NONE for none.
    fn hit(&self, x: f32, y: f32) -> u32 {
        let e = self.hit_where(x, y, |el| el.clickable);
        if e != NONE || !self.input.touch {
            return e;
        }
        self.near_target(x, y)
    }

    /// On a touch screen, a press that lands on no control goes to the
    /// nearest enabled one whose box, grown to [`TOUCH_TARGET`] square
    /// round its centre, holds the point (Material's minimum interactive
    /// size, Apple's 44 pt): a small control keeps its look and still
    /// takes a finger. The nearest box wins; at a tie, the one on top.
    fn near_target(&self, x: f32, y: f32) -> u32 {
        let (mut best, mut far) = (NONE, f32::MAX);
        for &e in self.stacking.order.iter().rev() {
            let (e, el) = (e as usize, &self.els[e as usize]);
            if !el.clickable || el.disabled || el.dead {
                continue;
            }
            if self.shown.get(e).is_some_and(|s| s.out || !s.touch) {
                continue;
            }
            let Some(&(bx, by, shown, _)) = self.abs.get(e) else {
                continue;
            };
            // The clip its parent paints it in, as `topmost` reads it.
            let all = [f32::MIN, f32::MIN, f32::MAX, f32::MAX];
            let clip = match el.arena_parent {
                _ if el.anchored().is_some() => all,
                NONE => all,
                p => self.abs.get(p as usize).map_or(all, |a| a.3),
            };
            if !shown || self.in_hole(e as u32, x, y).is_some() || !self.reachable(e as u32) {
                continue;
            }
            let b = self
                .motion
                .box_of(el.node)
                .unwrap_or_else(|| self.arena.box_of(el.node));
            // What of it is drawn: a control scrolled out of view takes nothing.
            let [x0, y0, x1, y1] = [
                bx.max(clip[0]),
                by.max(clip[1]),
                (bx + b.w).min(clip[2]),
                (by + b.h).min(clip[3]),
            ];
            if x1 <= x0 || y1 <= y0 {
                continue;
            }
            let gx = ((TOUCH_TARGET - (x1 - x0)) / 2.0).max(0.0);
            let gy = ((TOUCH_TARGET - (y1 - y0)) / 2.0).max(0.0);
            if x < x0 - gx || x >= x1 + gx || y < y0 - gy || y >= y1 + gy {
                continue;
            }
            let (dx, dy) = ((x0 - x).max(x - x1).max(0.0), (y0 - y).max(y - y1).max(0.0));
            let d = dx * dx + dy * dy;
            if d < far {
                (best, far) = (e as u32, d);
            }
        }
        best
    }

    /// [`Tree::hit`] over the elements `want` accepts; a press needs them
    /// enabled too. Behind an open modal, nothing answers.
    fn hit_where(&self, x: f32, y: f32, want: impl Fn(&El) -> bool) -> u32 {
        self.topmost(x, y, want, true)
    }

    /// The topmost shown element at (x, y) that `want` accepts; a disabled
    /// control counts only when `enabled` is false (its cursor shows).
    fn topmost(&self, x: f32, y: f32, want: impl Fn(&El) -> bool, enabled: bool) -> u32 {
        for &e in self.stacking.order.iter().rev() {
            let (e, el) = (e as usize, &self.els[e as usize]);
            // An element on its way out, or in one, answers nothing.
            if el.dead || self.shown.get(e).is_some_and(|s| s.out || !s.touch) {
                continue;
            }
            let Some(&(bx, by, shown, _)) = self.abs.get(e) else {
                continue;
            };
            // A spotlight takes the surface outside its hole, whatever its
            // box: the lit node under it takes its own presses, and what
            // else is under it none.
            if let Some(in_hole) = self
                .in_hole(e as u32, x, y)
                .filter(|_| shown && self.reachable(e as u32))
            {
                if in_hole {
                    continue;
                }
                let takes = want(el) && !(enabled && el.clickable && el.disabled);
                return if takes { e as u32 } else { NONE };
            }
            if !want(el) {
                continue;
            }
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
            // A span in a paragraph is where its glyphs are.
            let inside = match self.on_span(e as u32, x, y) {
                Some(on) => on && x >= cx0 && x < cx1 && y >= cy0 && y < cy1,
                None => {
                    x >= bx.max(cx0)
                        && x < (bx + b.w).min(cx1)
                        && y >= by.max(cy0)
                        && y < (by + b.h).min(cy1)
                }
            };
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
            let moved = (was.0, was.1) != (x, y);
            self.pointer_ops(x, y, moved, pressed, released);
            self.select_pointer(x, y, moved, pressed, released);
        }
    }
}

/// A ruby annotation's size, of its base's (CSS Ruby's UA style, 50 %).
pub(crate) const RUBY: f32 = 0.5;

/// The side, CSS px, a control's box is grown to for a finger on a touch
/// screen ([`Tree::near_target`]): WCAG 2.5.8's enhanced target, Apple's
/// 44 pt and the design's `--size-touch`.
pub(crate) const TOUCH_TARGET: f32 = 44.0;

/// A filter group open while its subtree paints: its element, its
/// composite's place in the arena (written by [`close_group`]), its
/// effects in [`Paint::effects`], its element's transform, and its
/// composite's look but the rect.
struct Open {
    e: u32,
    quad: usize,
    first: u32,
    count: u32,
    xf: u32,
    composite: PaintQuad,
}

/// The layers a paint may still draw ([`og_draw::paint::GROUP_LAYERS`])
/// and the area they may cover (CSS px²).
struct Room {
    layers: usize,
    area: f32,
    limit: f32,
}

impl Room {
    /// Takes a layer of `area`, when there is room for it.
    fn take(&mut self, area: f32) -> bool {
        if self.layers >= og_draw::paint::GROUP_LAYERS || self.area + area > self.limit {
            return false;
        }
        self.layers += 1;
        self.area += area;
        true
    }
}

/// How far `effects` spread what they filter, CSS px of the space they
/// are in: three deviations a blur, and a shadow's offset.
fn reach_of(effects: &[PaintEffect]) -> f32 {
    effects
        .iter()
        .map(|f| match *f {
            PaintEffect::Blur(sigma) => 3.0 * sigma,
            PaintEffect::DropShadow { offset, sigma, .. } => {
                3.0 * sigma + (offset[0] * offset[0] + offset[1] * offset[1]).sqrt()
            }
            PaintEffect::Matrix(_) => 0.0,
        })
        .sum()
}

/// The area of the layer a box `rect` (x, y, w, h) filtered by `effects`
/// may take on `screen`: the box grown by their reach, on the screen
/// grown by it.
fn reached(rect: [f32; 4], effects: &[PaintEffect], screen: [f32; 4]) -> f32 {
    let r = reach_of(effects);
    let [x, y, w, h] = rect;
    let w = (x + w + r).min(screen[2] + r) - (x - r).max(screen[0] - r);
    let h = (y + h + r).min(screen[3] + r) - (y - r).max(screen[1] - r);
    w.max(0.0) * h.max(0.0)
}

/// Whether element `e` is drawn under element `a` (its arena descendant).
fn descends(els: &[El], e: u32, a: u32) -> bool {
    let mut c = els[e as usize].arena_parent;
    while c != NONE {
        if c == a {
            return true;
        }
        c = els[c as usize].arena_parent;
    }
    false
}

/// Ends group `o` once everything it holds has painted: its composite
/// covers the screen box of what it holds (each quad's reach, through its
/// transform, inside its clip) grown by its effects' reach (through its
/// element's transform), on what of `screen` it shows in grown as much,
/// and the group is listed. With `members` (no stacking reorder moves its
/// quads: the boxes of the groups the last one listed holds), a group
/// right after the last one listed, with the same effects, transform and
/// composite, and whose box meets none of those, joins it: one layer and
/// one filter run draw them all, as they would alone, since none's
/// effects reach another's box (a row of shadowed icons is one layer, not
/// one each).
fn close_group(p: &mut Paint, o: &Open, screen: [f32; 4], members: Option<&mut Vec<[f32; 4]>>) {
    let mut bounds = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
    for q in &p.quads[o.quad + 1..] {
        let [x, y, w, h] = q.rect;
        // A shadow reaches past its box by its offset, spread and three
        // deviations; a line by its stroke; a box's edge by a pixel.
        let g = match q.kind {
            k if k == PaintKind::Shadow as u32 => {
                q.p[0].abs().max(q.p[1].abs()) + q.p[3].max(0.0) + 1.5 * q.p[2] + 1.0
            }
            k if k == PaintKind::Line as u32 => q.p[0] + 1.0,
            k if k == PaintKind::Group as u32
                || k == PaintKind::Image as u32
                || k == PaintKind::Raster as u32 =>
            {
                0.0
            }
            _ => 1.0,
        };
        let [ma, mb, mc, md, me, mf, _, _] = match q.xf {
            0 => og_draw::paint::IDENTITY,
            xf => p.transforms[xf as usize],
        };
        let mut b = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
        for [cx, cy] in [
            [x - g, y - g],
            [x + w + g, y - g],
            [x - g, y + h + g],
            [x + w + g, y + h + g],
        ] {
            let (sx, sy) = (ma * cx + mc * cy + me, mb * cx + md * cy + mf);
            b = [b[0].min(sx), b[1].min(sy), b[2].max(sx), b[3].max(sy)];
        }
        let c = q.clip;
        b = [
            b[0].max(c[0]),
            b[1].max(c[1]),
            b[2].min(c[2]),
            b[3].min(c[3]),
        ];
        if b[0] < b[2] && b[1] < b[3] {
            bounds = [
                bounds[0].min(b[0]),
                bounds[1].min(b[1]),
                bounds[2].max(b[2]),
                bounds[3].max(b[3]),
            ];
        }
    }
    let [ma, mb, mc, md, ..] = match o.xf {
        0 => og_draw::paint::IDENTITY,
        xf => p.transforms[xf as usize],
    };
    // The reach in screen px: the longer of the space's axes.
    let scale = (ma * ma + mb * mb).max(mc * mc + md * md).sqrt();
    let effects = &p.effects[o.first as usize..(o.first + o.count) as usize];
    let r = reach_of(effects) * scale;
    let c = o.composite.clip;
    let shown = [
        c[0].max(screen[0]) - r,
        c[1].max(screen[1]) - r,
        c[2].min(screen[2]) + r,
        c[3].min(screen[3]) + r,
    ];
    let x0 = (bounds[0] - r).max(shown[0]);
    let y0 = (bounds[1] - r).max(shown[1]);
    let x1 = (bounds[2] + r).min(shown[2]);
    let y1 = (bounds[3] + r).min(shown[3]);
    let rect = [x0, y0, (x1 - x0).max(0.0), (y1 - y0).max(0.0)];
    let end = p.quads.len() as u32;
    let mut members = members;
    if let Some(&last) = p.groups.last()
        && let Some(members) = members.as_deref_mut()
        && !last.backdrop
        && last.end == o.quad as u32
        && last.xf == o.xf
        && p.effects[last.first as usize..(last.first + last.count) as usize] == *effects
    {
        let at = last.quad as usize;
        let was = p.quads[at];
        let [ax, ay, aw, ah] = was.rect;
        // Apart when the boxes overlap by no more than a pixel: each box's
        // last pixel is its effects' last tap (3 deviations out, under 1%
        // of the middle's weight) on its content's anti-aliased edge. A
        // blur alone blurs its content too, so one layer differs from two
        // laid over each other by the product of their tails: under half
        // a code value (1/512) while the gap between their contents is 3.5
        // deviations (each tail under 0.044 at its middle), so each box
        // may reach 1.25 deviations into the other's.
        let overlap = 1.0
            + match *effects {
                [PaintEffect::Blur(sigma)] => 2.0 * 1.25 * sigma * scale,
                _ => 0.0,
            };
        let apart = members.iter().all(|&[mx, my, mw, mh]| {
            x1 <= mx + overlap
                || mx + mw <= x0 + overlap
                || y1 <= my + overlap
                || my + mh <= y0 + overlap
        });
        let same = PaintQuad {
            rect: was.rect,
            a: was.a,
            ..o.composite
        } == was;
        if apart && same {
            members.push(rect);
            let (ux, uy) = (ax.min(x0), ay.min(y0));
            p.quads[at].rect = [ux, uy, (ax + aw).max(x1) - ux, (ay + ah).max(y1) - uy];
            // Its own composite draws nothing in the layer they share.
            p.quads[o.quad] = PaintQuad {
                rect: [0.0; 4],
                color: [0.0; 4],
                kind: PaintKind::Solid as u32,
                ..o.composite
            };
            if let Some(g) = p.groups.last_mut() {
                g.end = end;
            }
            return;
        }
    }
    if let Some(members) = members {
        members.clear();
        members.push(rect);
    }
    p.groups.push(PaintGroup {
        quad: o.quad as u32,
        end,
        first: o.first,
        count: o.count,
        xf: o.xf,
        backdrop: false,
    });
    p.quads[o.quad] = PaintQuad {
        rect,
        a: p.groups.len() as u32 - 1,
        ..o.composite
    };
}

/// An image of `size` texels showing `uv` of it, fitted into `rect` as
/// CSS's `object-fit` and `object-position` place it: the part of the box
/// it covers, and the region of the texture drawn there. An unknown size
/// stretches it.
fn fitted(
    (fit, at): (
        nsh_tailwind::ResizeMode,
        nsh_tailwind::style::ObjectPosition,
    ),
    rect: [f32; 4],
    uv: [f32; 4],
    size: [f32; 2],
) -> ([f32; 4], [f32; 4]) {
    use nsh_tailwind::ResizeMode as R;
    let [x, y, w, h] = rect;
    let natural = [
        size[0] * (uv[2] - uv[0]).abs(),
        size[1] * (uv[3] - uv[1]).abs(),
    ];
    if fit == R::Stretch || natural[0] <= 0.0 || natural[1] <= 0.0 || w <= 0.0 || h <= 0.0 {
        return (rect, uv);
    }
    let contain = (w / natural[0]).min(h / natural[1]);
    let scale = match fit {
        R::Contain => contain,
        R::Cover => (w / natural[0]).max(h / natural[1]),
        R::Center => 1.0,
        R::ScaleDown => contain.min(1.0),
        R::Stretch => unreachable!(),
    };
    let (dw, dh) = (natural[0] * scale, natural[1] * scale);
    let dx = at.x.resolve(nsh_tailwind::geometry::Points(w - dw)).0;
    let dy = at.y.resolve(nsh_tailwind::geometry::Points(h - dh)).0;
    // The drawn image's box, then the part of it inside the element's.
    let (ix0, iy0) = (x + dx, y + dy);
    let (x0, y0) = (ix0.max(x), iy0.max(y));
    let (x1, y1) = ((ix0 + dw).min(x + w), (iy0 + dh).min(y + h));
    let along = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let u = |px: f32| along(uv[0], uv[2], (px - ix0) / dw);
    let v = |py: f32| along(uv[1], uv[3], (py - iy0) / dh);
    (
        [x0, y0, (x1 - x0).max(0.0), (y1 - y0).max(0.0)],
        [u(x0), v(y0), u(x1), v(y1)],
    )
}

/// Paints the decoration through the existing solid/polyline primitives.
fn decorate(
    p: &mut Paint,
    rect: [f32; 4],
    clip: [f32; 4],
    color: Rgba,
    style: nsh_tailwind::style::DecorationStyle,
) {
    use nsh_tailwind::style::DecorationStyle::*;
    let [x, y, width, th] = rect;
    let solid = |p: &mut Paint, r, radius| {
        p.quads.push(quad(
            PaintKind::Solid,
            r,
            clip,
            color,
            [radius; 4],
            [0.; 4],
            0,
            0,
        ));
    };
    match style {
        Solid => solid(p, rect, 0.),
        Double => {
            solid(p, [x, y - th, width, th], 0.);
            solid(p, [x, y + th, width, th], 0.);
        }
        Dotted | Dashed => {
            let length = if style == Dotted { th } else { 3. * th };
            let mut at = 0.;
            while at < width {
                solid(
                    p,
                    [x + at, y, length.min(width - at), th],
                    if style == Dotted { th / 2. } else { 0. },
                );
                at += length + th;
            }
        }
        Wavy => {
            let first = p.points.len() as u32;
            let steps = (width / th).ceil() as u32;
            if steps == 0 {
                return;
            }
            for k in 0..=steps {
                let at = width * k as f32 / steps as f32;
                p.points.push([
                    at + th,
                    2. * th + th * og_core::math::sin(at / (4. * th) * std::f32::consts::TAU),
                ]);
            }
            p.quads.push(quad(
                PaintKind::Line,
                [x - th, y - 1.5 * th, width + 2. * th, 4. * th],
                clip,
                color,
                [0.; 4],
                [th, 0., 0., 0.],
                first,
                steps + 1,
            ));
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
        filter: 0,
        round: 0,
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

/// The right edge of the content box that set `e`'s `text-overflow`
/// (itself or the ancestor it inherits it from), where its text is cut;
/// `None` when that box was not placed this frame.
fn overflow_edge(
    els: &[El],
    abs: &[(f32, f32, bool, [f32; 4])],
    motion: &crate::motion::Motion,
    arena: &Arena,
    e: u32,
) -> Option<f32> {
    let mut a = e;
    while els[a as usize].record.text.set & Text::OVERFLOW == 0 {
        a = els[a as usize].parent;
        if a == NONE {
            return None;
        }
    }
    let &(x, _, shown, _) = abs.get(a as usize)?;
    let node = els[a as usize].node;
    let b = motion.box_of(node).unwrap_or_else(|| arena.box_of(node));
    shown.then_some(x + b.w - arena.box_of(node).inset[2])
}
