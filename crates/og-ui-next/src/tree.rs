//! The TSX UI as the new core runs it: the renderer's op batches
//! ([`ops`]) make and patch its elements in an [`Arena`], each with the
//! record its classes resolve to. A frame applies what changed to the
//! text its subtree inherits, lays out what is dirty and paints again
//! only when something changed. A frame with nothing changed does no work
//! and allocates nothing.

use crate::arena::{Arena, Id};
use crate::icons::IconAtlas;
use crate::record::{Decoration, Record, Rgba, Text, TextShadows, TextStroke};
use og_draw::glyphs::Atlas;
use og_draw::paint::{Paint, PaintGroup, PaintQuad};
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
    /// The element it inherits from, and its direct logical children.
    parent: u32,
    children: Vec<u32>,
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
    /// Freed element indices in [`Tree::els`] awaiting reuse by [`Tree::create`].
    free_els: Vec<u32>,
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
    /// Each element's blend mode in the last paint ([`og_draw::paint::PaintBlend`]):
    /// 0 normal, 1 add, 2 screen, 3 multiply.
    blends: Vec<u8>,
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

mod anchors;
mod checkpoint;
mod count;
mod hit;
mod inherit;
mod input;
mod ops;
mod paint;
mod para;
mod particles;
mod stacking;
mod thumb;
mod vcursor;
pub(crate) use anchors::{Anchor, CLAMP_MARGIN, Declutter, Occlusion};
pub use anchors::{AnchorTarget, Sighting};
pub use checkpoint::{InputFieldState, TreeCheckpoint, VirtualCursorCheckpoint};
use count::Count;
pub(crate) use hit::TOUCH_TARGET;
pub use input::{Cue, HapticTable, Play, SoundTable};
pub use ops::{EVENTS, Event as OpEvent, Fired, MAGIC, REFUSED, Resolve};
pub(crate) use paint::{ImageProps, Line};
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
            free_els: Vec::new(),
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
            blends: Vec::new(),
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

    /// This frame's screen transition cover (`None`: none), drawn over
    /// everything; `reduced` motion draws it as a fade.
    pub fn set_cover(&mut self, cover: Option<crate::cover::Cover>, reduced: bool) {
        let cover = cover.map(|c| (c, reduced));
        if self.cover != cover {
            self.cover = cover;
            self.paint = true;
        }
    }
}

/// A ruby annotation's size, of its base's (CSS Ruby's UA style, 50 %).
pub(crate) const RUBY: f32 = 0.5;
