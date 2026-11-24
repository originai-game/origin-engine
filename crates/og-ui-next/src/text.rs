//! Text leaves laid out with og-text, one Parley layout a leaf
//! ([`og_text::layout::Text`]): shaped when the leaf's text, its [`Run`] or
//! the faces change, broken at each width layout asks, and read again by
//! paint, the caret, the decoration lines and the probe. A measured text
//! and a drawn one are the same object, so a text never breaks inside the
//! box it sized.
//!
//! Paint turns a leaf's glyph runs into quads of the glyph atlas
//! ([`og_draw::glyphs::Atlas`]) through og-text's strikes: coverage at the
//! drawn size on whole device pixels, a colour glyph in its own colours,
//! or a distance field for a text under a transform that is not a
//! translation. A leaf keeps its quads while what they were made from
//! stays, so a steady frame copies them and looks nothing up.

use crate::arena::{Avail, Id, Run};
use og_draw::glyphs::{Atlas, GlyphSlot, GlyphTier, NO_PLOT};
use og_draw::paint::{PaintKind, PaintQuad};
use og_i18n::Direction;
use og_text::fonts::Collection;
use og_text::layout::{
    Alignment, Decorations, FontFeature, Inline, LineHeight, OverflowWrap, Span, Style, Text,
    feature,
};
use og_text::strike::{Form, Strikes};

/// Above this many device pixels an em a glyph draws from its distance
/// field: one raster would take a plot of its own.
const FIELD_OVER: f32 = 128.0;

/// How a leaf's glyphs draw, beside its layout.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Look {
    /// Device pixels a CSS px.
    pub scale: f32,
    /// Under a transform that is not a translation: distance fields.
    pub field: bool,
    /// The text shadow layers' distinct blur radii, CSS px, the first
    /// `shadow_count` of them (the rest 0): a raster each.
    pub shadows: [f32; crate::record::TEXT_SHADOWS],
    pub shadow_count: u8,
    /// The outline's width (`-webkit-text-stroke`), CSS px; 0 none.
    pub stroke: f32,
    /// The text colour a colour glyph's foreground takes, sRGB.
    pub ink: [f32; 3],
}

impl Look {
    /// The shadow rasters' blur radii, CSS px.
    pub fn blurs(&self) -> &[f32] {
        &self.shadows[..usize::from(self.shadow_count)]
    }
}

/// What a leaf's quads were made from.
#[derive(Clone, Copy, PartialEq)]
struct Painted {
    width: u32,
    look: Look,
    /// The atlas, and how many plots it had emptied.
    atlas: (u64, u64),
}

/// A paragraph's styled runs and inline boxes, over its leaf's own
/// [`Run`]: what a `<Text>` holding more than one run lays out as one
/// text (`crate::tree`'s paragraphs).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Rich {
    pub spans: Vec<RichSpan>,
    pub boxes: Vec<RichBox>,
}

/// Bytes `start..end` of a paragraph's text, run as `run` (its size,
/// spacing, weight, slant and family), its glyphs drawn in `ink`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RichSpan {
    pub start: u32,
    pub end: u32,
    pub run: Run,
    pub ink: u16,
}

/// A box `w` by `h` CSS px laid out before byte `at`, as a character.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RichBox {
    pub at: u32,
    pub w: f32,
    pub h: f32,
    pub id: u32,
}

/// Calls `f` with `rich`'s spans and boxes as og-text takes them.
fn with_rich<R>(rich: &Rich, f: impl FnOnce(&[Span<'_>], &[Inline]) -> R) -> R {
    if rich.spans.is_empty() && rich.boxes.is_empty() {
        return f(&[], &[]);
    }
    let names: Vec<Option<std::sync::Arc<str>>> = rich
        .spans
        .iter()
        .map(|s| s.run.family.map(|f| f.name()))
        .collect();
    let spans: Vec<Span<'_>> = rich
        .spans
        .iter()
        .zip(&names)
        .map(|(s, name)| Span {
            start: s.start as usize,
            end: s.end as usize,
            family: name.as_deref().unwrap_or(SANS),
            size: s.run.size,
            weight: f32::from(s.run.weight),
            italic: s.run.italic,
            letter_spacing: s.run.tracking,
            line_height: if s.run.line < 0.0 {
                LineHeight::NORMAL
            } else {
                LineHeight::Absolute(s.run.line)
            },
            ink: s.ink,
        })
        .collect();
    let boxes: Vec<Inline> = rich
        .boxes
        .iter()
        .map(|b| Inline {
            at: b.at as usize,
            width: b.w,
            height: b.h,
            id: u64::from(b.id),
        })
        .collect();
    f(&spans, &boxes)
}

/// One text leaf: its layout and its quads.
#[derive(Default)]
pub struct Leaf {
    /// What the layout was shaped from.
    pub text: String,
    run: Option<Run>,
    /// The collection's and the writing's revisions it was shaped at.
    shaped: (u64, u64),
    layout: Text,
    /// The width the lines are broken at ([`Avail::key`]) and whether
    /// with the ellipsis; `None` after a shape.
    broken: Option<(u32, bool)>,
    painted: Option<Painted>,
    /// Its styled runs and inline boxes, when it is a paragraph's.
    rich: Rich,
    /// Its glyph quads from the leaf's top left (clip and colour set as
    /// they are copied): the fill, the outline under it and the shadow
    /// under both.
    pub fill: Vec<PaintQuad>,
    /// Each fill quad's ink ([`RichSpan::ink`]; 0 the leaf's own).
    pub fill_ink: Vec<u16>,
    /// The stretches of each line in one ink (og-text's `Text::inks`): a
    /// span's decoration lines lie along them.
    pub inks: Vec<(u16, [f32; 3])>,
    /// Its inline boxes as painted: id and rectangle from its top left.
    pub boxes: Vec<(u32, [f32; 4])>,
    pub stroke: Vec<PaintQuad>,
    /// One list per blur of [`Look::blurs`], in its order.
    pub shadow: Vec<Vec<PaintQuad>>,
    /// The atlas plots the quads read.
    plots: Vec<u32>,
    /// A glyph found no room in the atlas this frame: painted again next.
    pub incomplete: bool,
    /// Whether a colour glyph is among them (its quads follow the ink).
    colored: bool,
    /// Each line's start x, baseline and width, for its decoration.
    pub lines: Vec<[f32; 3]>,
    /// The lines' texts as laid out, each ended by `\n`: what the probe's
    /// dump checks for a word broken where UAX #14 has no break.
    pub line_text: String,
    decorations: Option<Decorations>,
}

impl Leaf {
    /// The font size it was laid out at, CSS px.
    pub fn size(&self) -> f32 {
        self.run.map_or(0.0, |r| r.size)
    }

    pub fn broken_words(&self) -> Vec<String> {
        broken_words(&self.text, &self.line_text, |at| {
            self.layout.is_discretionary_break(at)
        })
    }

    pub fn selection(&self, range: std::ops::Range<usize>, f: impl FnMut([f32; 4])) {
        self.layout.selection(range, f);
    }

    /// The caret offset (a byte of its laid-out text) nearest (x, y) from
    /// its left and top, the word there, and the offset `lines` lines from
    /// `at` (og-text's [`Text::hit`], [`Text::word_at`], [`Text::line_step`]).
    pub fn hit(&self, x: f32, y: f32) -> usize {
        self.layout.hit(x, y)
    }
    /// Its widest line, CSS px.
    pub fn width(&self) -> f32 {
        self.layout.width()
    }
    pub fn word_at(&self, x: f32, y: f32) -> std::ops::Range<usize> {
        self.layout.word_at(x, y)
    }
    /// The text it laid out: its text, or that cut to an ellipsis.
    pub fn laid(&self) -> &str {
        self.layout.laid(&self.text)
    }
    pub fn line_step(&self, at: usize, lines: isize) -> usize {
        self.layout.line_step(at, lines)
    }
}

/// Every text leaf's layout, by arena node.
pub struct Texts {
    leaves: Vec<Option<Box<Leaf>>>,
    /// Freed nodes' leaves, which new leaves take with their buffers' room
    /// (a HUD's spawning numbers shape and paint into kept vectors). Boxed
    /// as `leaves` holds them, so a leaf moves between the two unallocated.
    #[allow(clippy::vec_box)]
    spare: Vec<Box<Leaf>>,
    strikes: Strikes,
    /// The paragraph direction and the locale (BCP 47) texts are laid out
    /// in, and a count of their changes.
    direction: Direction,
    locale: String,
    writing: u64,
    features: Vec<FontFeature>,
}

impl Default for Texts {
    fn default() -> Self {
        Texts {
            leaves: Vec::new(),
            spare: Vec::new(),
            strikes: Strikes::new(),
            direction: Direction::Ltr,
            locale: String::new(),
            writing: 0,
            features: Vec::new(),
        }
    }
}

/// `run`'s CSS style over `family` (its list) and `features`.
fn style<'a>(
    run: &Run,
    family: &'a str,
    features: &'a [FontFeature],
    direction: Direction,
    locale: &'a str,
    ellipsis: bool,
) -> Style<'a> {
    Style {
        family,
        size: run.size,
        weight: f32::from(run.weight),
        italic: run.italic,
        letter_spacing: run.tracking,
        line_height: if run.line < 0.0 {
            LineHeight::NORMAL
        } else {
            LineHeight::Absolute(run.line)
        },
        features,
        locale: (!locale.is_empty()).then_some(locale),
        direction,
        wrap: !run.nowrap,
        hyphens: run.hyphens,
        overflow_wrap: match run.wrap {
            1 => OverflowWrap::BreakWord,
            2 => OverflowWrap::Anywhere,
            _ => OverflowWrap::Normal,
        },
        align: match run.align {
            1 => Alignment::Center,
            2 => Alignment::Right,
            3 => Alignment::Left,
            4 => Alignment::Justify,
            _ => Alignment::Start,
        },
        ellipsis,
        max_lines: run.clamp,
        vertical: run.vertical,
        ..Default::default()
    }
}

/// NSH's `font-variant` bits as OpenType features.
fn features_of(variant: u8, out: &mut Vec<FontFeature>) {
    use nsh_tailwind::style::FontVariant as V;
    out.clear();
    for (bit, tag) in [
        (V::SMALL_CAPS, b"smcp"),
        (V::OLDSTYLE_NUMS, b"onum"),
        (V::LINING_NUMS, b"lnum"),
        (V::TABULAR_NUMS, b"tnum"),
        (V::PROPORTIONAL_NUMS, b"pnum"),
    ] {
        if variant & bit != 0 {
            out.push(feature(tag, 1));
        }
    }
}

/// The theme's sans, what a text with no `font-family` runs in.
const SANS: &str = "sans-serif";

impl Texts {
    /// The paragraph direction and locale every text is laid out in from
    /// now on: whether they changed (every leaf then measures again).
    pub fn set_writing(&mut self, direction: Direction, locale: &str) -> bool {
        if self.direction == direction && self.locale == locale {
            return false;
        }
        self.direction = direction;
        self.locale.clear();
        self.locale.push_str(locale);
        self.writing += 1;
        true
    }

    /// A leaf's layout as paint left it.
    pub fn leaf(&self, id: Id) -> Option<&Leaf> {
        self.leaves.get(id as usize)?.as_deref()
    }

    /// Leaf `id`'s node is freed: its leaf goes to the spare ones, shaped
    /// and painted again by the next leaf that takes it.
    pub fn release(&mut self, id: Id) {
        if let Some(mut leaf) = self.leaves.get_mut(id as usize).and_then(Option::take) {
            leaf.run = None;
            leaf.painted = None;
            leaf.rich = Rich::default();
            self.spare.push(leaf);
        }
    }

    /// Sets leaf `id`'s styled runs and inline boxes (none: one style):
    /// whether they changed, when it is shaped again.
    pub fn set_rich(&mut self, id: Id, rich: &Rich) -> bool {
        if self.leaves.len() <= id as usize {
            self.leaves.resize_with(id as usize + 1, || None);
        }
        let spare = &mut self.spare;
        let leaf = self.leaves[id as usize].get_or_insert_with(|| spare.pop().unwrap_or_default());
        if leaf.rich == *rich {
            return false;
        }
        leaf.rich.clone_from(rich);
        leaf.run = None;
        true
    }

    /// Leaf `id` shaped from `text` and `run`, and its style's parts.
    fn shaped(&mut self, fonts: &mut Collection, id: Id, text: &str, run: &Run) -> &mut Leaf {
        if self.leaves.len() <= id as usize {
            self.leaves.resize_with(id as usize + 1, || None);
        }
        let spare = &mut self.spare;
        let leaf = self.leaves[id as usize].get_or_insert_with(|| spare.pop().unwrap_or_default());
        let at = (fonts.revision(), self.writing);
        if leaf.run != Some(*run) || leaf.shaped != at || *leaf.text != *text {
            leaf.text.clear();
            leaf.text.push_str(text);
            leaf.run = Some(*run);
            leaf.shaped = at;
            let family = run.family.map(|f| f.name());
            features_of(run.variant, &mut self.features);
            let s = style(
                run,
                family.as_deref().unwrap_or(SANS),
                &self.features,
                self.direction,
                &self.locale,
                false,
            );
            let layout = &mut leaf.layout;
            with_rich(&leaf.rich, |spans, boxes| {
                layout.shape(fonts, text, &Style { spans, boxes, ..s })
            });
            leaf.broken = None;
            leaf.painted = None;
            leaf.decorations = None;
        }
        leaf
    }

    /// Breaks leaf `id`'s lines at `avail` (with the ellipsis or not)
    /// unless they are.
    fn broken(
        &mut self,
        fonts: &mut Collection,
        id: Id,
        text: &str,
        run: &Run,
        avail: Avail,
        ellipsis: bool,
    ) -> &mut Leaf {
        self.shaped(fonts, id, text, run);
        let (direction, writing) = (self.direction, &self.locale);
        let leaf = self.leaves[id as usize].as_deref_mut().expect("shaped");
        let key = (avail.key(), ellipsis);
        if leaf.broken != Some(key) {
            let family = run.family.map(|f| f.name());
            features_of(run.variant, &mut self.features);
            let s = style(
                run,
                family.as_deref().unwrap_or(SANS),
                &self.features,
                direction,
                writing,
                ellipsis,
            );
            let width = match avail {
                Avail::Max => None,
                Avail::Min => Some(leaf.layout.content_widths().0),
                // A box is its text's measured width through layout's sums,
                // which may lose a little: the text given that width back
                // keeps its lines, as a flex line keeps its items.
                Avail::Width(w) => Some(w + taffy::LAYOUT_SLACK),
            };
            let layout = &mut leaf.layout;
            with_rich(&leaf.rich, |spans, boxes| {
                layout.break_at(fonts, text, &Style { spans, boxes, ..s }, width)
            });
            leaf.broken = Some(key);
            leaf.painted = None;
        }
        leaf
    }

    /// The size of leaf `id`'s `text` run as `run`, at the width it may
    /// take (CSS Sizing 3 §4): its widest line and its line boxes. A
    /// text's `text-overflow` never sizes it.
    pub fn measure(
        &mut self,
        fonts: &mut Collection,
        id: Id,
        text: &str,
        run: &Run,
        avail: Avail,
    ) -> (f32, f32) {
        // An empty text keeps a line, so an empty `Input` keeps its height:
        // its line height, or under `normal` what a space's line takes.
        let boxed = self.leaf(id).is_some_and(|l| !l.rich.boxes.is_empty());
        if text.is_empty() && !boxed {
            if run.line >= 0.0 {
                return (0.0, run.line);
            }
            return (
                0.0,
                self.broken(fonts, id, " ", run, avail, false)
                    .layout
                    .height(),
            );
        }
        let leaf = self.broken(fonts, id, text, run, avail, false);
        let width = match avail {
            Avail::Min if !run.vertical => leaf.layout.content_widths().0.max(leaf.layout.width()),
            _ => leaf.layout.width(),
        };
        (width, leaf.layout.height())
    }

    /// Where the caret before byte `at` of leaf `id`'s text stands, as
    /// paint laid it out: x from the leaf's left, its top and its height.
    pub fn caret(&self, id: Id, at: usize) -> Option<[f32; 3]> {
        let leaf = self.leaf(id)?;
        let laid = leaf.layout.laid(&leaf.text);
        let mut at = at.min(laid.len());
        while !laid.is_char_boundary(at) {
            at -= 1;
        }
        Some(leaf.layout.caret(at))
    }

    /// Leaf `id`'s decoration lines, from its primary font.
    pub fn decorations(&mut self, fonts: &mut Collection, id: Id) -> Decorations {
        let (direction, locale) = (self.direction, &self.locale);
        let Some(leaf) = self
            .leaves
            .get_mut(id as usize)
            .and_then(Option::as_deref_mut)
        else {
            return Decorations::default();
        };
        let Some(run) = leaf.run else {
            return Decorations::default();
        };
        *leaf.decorations.get_or_insert_with(|| {
            let family = run.family.map(|f| f.name());
            let s = style(
                &run,
                family.as_deref().unwrap_or(SANS),
                &[],
                direction,
                locale,
                false,
            );
            og_text::layout::decorations(fonts, &s)
        })
    }

    /// Leaf `id` laid out in a box `width` wide and its glyphs made into
    /// quads as `look` draws them, unless it holds them already. Marks the
    /// atlas plots its quads read used this frame. A character no face
    /// draws joins `missing` once.
    #[allow(clippy::too_many_arguments)]
    pub fn paint(
        &mut self,
        fonts: &mut Collection,
        atlas: &mut Atlas,
        id: Id,
        text: &str,
        run: &Run,
        width: f32,
        look: &Look,
        missing: &mut Vec<char>,
    ) -> &Leaf {
        let avail = Avail::Width(width);
        self.broken(fonts, id, text, run, avail, run.ellipsis);
        let leaf = self.leaves[id as usize].as_deref_mut().expect("broken");
        let mut painted = Painted {
            width: avail.key(),
            look: *look,
            atlas: (atlas.id(), atlas.stats().evicted),
        };
        // Only a colour glyph's quads follow the ink.
        if !leaf.colored
            && let Some(p) = &leaf.painted
        {
            painted.look.ink = p.look.ink;
        }
        if leaf.painted == Some(painted) && !leaf.incomplete {
            for &plot in &leaf.plots {
                atlas.touch(plot);
            }
            return leaf;
        }
        painted.look.ink = look.ink;
        leaf.fill.clear();
        leaf.fill_ink.clear();
        leaf.inks.clear();
        leaf.boxes.clear();
        leaf.stroke.clear();
        leaf.shadow.iter_mut().for_each(Vec::clear);
        leaf.shadow.resize_with(look.blurs().len(), Vec::new);
        leaf.plots.clear();
        leaf.lines.clear();
        leaf.line_text.clear();
        leaf.incomplete = false;
        leaf.colored = false;
        leaf.layout.inks(|ink, at| leaf.inks.push((ink, at)));
        leaf.layout.boxes(|id, at| leaf.boxes.push((id as u32, at)));
        let Leaf {
            layout,
            text: shaped,
            fill,
            fill_ink,
            stroke,
            shadow,
            plots,
            incomplete,
            colored,
            lines,
            line_text,
            ..
        } = leaf;
        let laid = layout.laid(shaped);
        layout.lines(|at, range| {
            lines.push(at);
            line_text.push_str(laid[range].trim_end_matches(['\n', '\r']));
            line_text.push('\n');
        });
        layout.missing(laid, |c| {
            if !missing.contains(&c) {
                missing.push(c);
            }
        });
        let scale = look.scale;
        let strikes = &mut self.strikes;
        let fonts = &*fonts;
        layout.runs(fonts, |run| {
            let strike = strikes.strike(atlas, &run, scale, !look.field);
            let field = look.field || strike.size() > FIELD_OVER;
            // Field pixels to CSS px, and what a field's edge moves out by
            // for a bold the face lacks.
            let em = run.size / og_text::sdf::EM_PX;
            let bold = 0.5 * strike.embolden() / scale;
            // A character no face draws is drawn by none (the missing
            // glyph is never shown): a font pack that has it is on its way.
            for g in run.glyphs.iter().filter(|g| g.id != 0) {
                // The pen on the device grid: whole pixels at scale 1, a
                // quarter of one above it; the baseline on a whole pixel.
                let x = g.x * scale;
                let (pen, bin) = if (scale - 1.0).abs() < 1e-3 {
                    (x.round(), 0u8)
                } else {
                    let q = (x * 4.0 + 0.5).floor();
                    let pen = (q / 4.0).floor();
                    (pen, (q - pen * 4.0) as u8)
                };
                let base = (g.y * scale).round();
                let stroke_form = (look.stroke > 0.0).then_some(Form::Stroke(look.stroke * scale));
                let shadow_forms = look
                    .blurs()
                    .iter()
                    .enumerate()
                    .map(|(j, &r)| (Some(Form::Shadow(r.max(0.0) * scale)), 2 + j));
                let forms = [(Some(Form::Fill), 0usize), (stroke_form, 1)]
                    .into_iter()
                    .chain(shadow_forms);
                for (form, list) in forms {
                    let Some(form) = form else { continue };
                    let quads = match list {
                        0 => &mut *fill,
                        1 => &mut *stroke,
                        j => &mut shadow[j - 2],
                    };
                    let use_color = field
                        && matches!(form, Form::Fill)
                        && strikes.is_color(fonts, &strike, g.id);
                    let raster = if field && !use_color {
                        None
                    } else {
                        strikes.glyph(fonts, atlas, &strike, run.coords, g.id, bin, form, look.ink)
                    };
                    let slot: GlyphSlot;
                    let (rect, tier, p) = match raster {
                        Some((tier, s)) => {
                            slot = s;
                            *colored |= tier == GlyphTier::Color;
                            let rect = [
                                (pen + f32::from(s.left)) / scale,
                                (base - f32::from(s.top)) / scale,
                                f32::from(s.width) / scale,
                                f32::from(s.height) / scale,
                            ];
                            (rect, tier, [0.0; 4])
                        }
                        // A text under a transform, a glyph too large for
                        // a plot: its distance field, grown and blurred in
                        // the shader.
                        None => match strikes.field(fonts, atlas, &strike, run.coords, g.id) {
                            Some(s) => {
                                slot = s;
                                let rect = [
                                    g.x + f32::from(s.left) * em,
                                    g.y - f32::from(s.top) * em,
                                    f32::from(s.width) * em,
                                    f32::from(s.height) * em,
                                ];
                                let (grow, blur) = match form {
                                    Form::Fill => (bold, 0.0),
                                    Form::Stroke(_) => (bold + 0.5 * look.stroke, 0.0),
                                    Form::Shadow(r) => (bold, r / scale),
                                };
                                let p = [grow, blur, og_text::sdf::RANGE, 0.0];
                                (rect, GlyphTier::Field, p)
                            }
                            None => {
                                *incomplete = true;
                                continue;
                            }
                        },
                    };
                    if slot.plot == NO_PLOT {
                        continue;
                    }
                    if !plots.contains(&slot.plot) {
                        plots.push(slot.plot);
                    }
                    let (x0, y0) = (f32::from(slot.x), f32::from(slot.y));
                    quads.push(PaintQuad {
                        rect,
                        clip: [0.0; 4],
                        color: [0.0; 4],
                        radius: [0.0; 4],
                        p,
                        kind: PaintKind::Raster as u32,
                        a: tier as u32,
                        b: u32::from(slot.page),
                        xf: 0,
                        uv: [
                            x0,
                            y0,
                            x0 + f32::from(slot.width),
                            y0 + f32::from(slot.height),
                        ],
                        wipe: [0.0; 4],
                        mask: 0,
                        filter: 0,
                        round: 0,
                        blend: 0,
                    });
                    if list == 0 {
                        fill_ink.push(run.ink);
                    }
                }
            }
        });
        leaf.painted = Some(painted);
        leaf
    }

    /// The atlas plots leaf `id`'s quads read.
    pub fn plots(&self, id: Id) -> &[u32] {
        self.leaf(id).map_or(&[], |l| &l.plots)
    }
}

/// Where `lines` (each ended by `\n`, as `text` was laid out) start a
/// line inside a word: at a place UAX #14 gives no break opportunity and
/// no `\n` of `text` ends the line before. Each as `…end|start…` of the
/// two lines, a few characters of each.
pub fn broken_words(text: &str, lines: &str, discretionary: impl Fn(usize) -> bool) -> Vec<String> {
    let mut out = Vec::new();
    let mut parts = lines.split_terminator('\n');
    let Some(mut before) = parts.next() else {
        return out;
    };
    let mut breaks = None;
    let mut at = text.find(before).map_or(0, |i| i + before.len());
    for line in parts {
        let Some(start) = text[at..].find(line).map(|i| at + i) else {
            break;
        };
        if !line.is_empty() && !text[..start].ends_with('\n') {
            let opportunities: &Vec<usize> = breaks.get_or_insert_with(|| {
                og_i18n::segmenter::line(Default::default())
                    .segment_str(text)
                    .collect()
            });
            if !opportunities.contains(&start) && !discretionary(start) {
                let tail: String = {
                    let c: Vec<char> = before.chars().rev().take(12).collect();
                    c.into_iter().rev().collect()
                };
                let head: String = line.chars().take(12).collect();
                out.push(format!("{tail}|{head}"));
            }
        }
        at = start + line.len();
        before = line;
    }
    out
}
