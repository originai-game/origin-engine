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
    Alignment, Decorations, FontFeature, LineHeight, OverflowWrap, Style, Text, feature,
};
use og_text::strike::{Form, Strikes};

/// What a leaf breaks wider than the width it is given by, CSS px: a box
/// is its text's measured width through layout's sums, which may lose a
/// little, and the text given that width back must keep its lines.
const SLACK: f32 = 0.01;

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
    /// A text shadow's blur radius, CSS px; `None`: no shadow.
    pub shadow: Option<f32>,
    /// The outline's width (`-webkit-text-stroke`), CSS px; 0 none.
    pub stroke: f32,
    /// The text colour a colour glyph's foreground takes, sRGB.
    pub ink: [f32; 3],
}

/// What a leaf's quads were made from.
#[derive(Clone, Copy, PartialEq)]
struct Painted {
    width: u32,
    look: Look,
    /// The atlas, and how many plots it had emptied.
    atlas: (u64, u64),
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
    /// Its glyph quads from the leaf's top left (clip and colour set as
    /// they are copied): the fill, the outline under it and the shadow
    /// under both.
    pub fill: Vec<PaintQuad>,
    pub stroke: Vec<PaintQuad>,
    pub shadow: Vec<PaintQuad>,
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
}

/// Every text leaf's layout, by arena node.
pub struct Texts {
    leaves: Vec<Option<Box<Leaf>>>,
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

    /// Leaf `id` shaped from `text` and `run`, and its style's parts.
    fn shaped(&mut self, fonts: &mut Collection, id: Id, text: &str, run: &Run) -> &mut Leaf {
        if self.leaves.len() <= id as usize {
            self.leaves.resize_with(id as usize + 1, || None);
        }
        let leaf = self.leaves[id as usize].get_or_insert_with(Default::default);
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
            leaf.layout.shape(fonts, text, &s);
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
                Avail::Width(w) => Some(w + SLACK),
            };
            leaf.layout.break_at(fonts, text, &s, width);
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
        if text.is_empty() {
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
            Avail::Min => leaf.layout.content_widths().0.max(leaf.layout.width()),
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
        leaf.stroke.clear();
        leaf.shadow.clear();
        leaf.plots.clear();
        leaf.lines.clear();
        leaf.line_text.clear();
        leaf.incomplete = false;
        leaf.colored = false;
        let Leaf {
            layout,
            text: shaped,
            fill,
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
                let forms = [
                    (Some(Form::Fill), 0u8),
                    (
                        (look.stroke > 0.0).then_some(Form::Stroke(look.stroke * scale)),
                        1,
                    ),
                    (look.shadow.map(|r| Form::Shadow(r.max(0.0) * scale)), 2),
                ];
                for (form, list) in forms {
                    let Some(form) = form else { continue };
                    let quads = match list {
                        0 => &mut *fill,
                        1 => &mut *stroke,
                        _ => &mut *shadow,
                    };
                    let raster = if field {
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
                                    Form::Shadow(_) => (bold, look.shadow.unwrap_or(0.0)),
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
                    });
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
pub fn broken_words(text: &str, lines: &str) -> Vec<String> {
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
            if !opportunities.contains(&start) {
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

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::sync::Arc;

    /// A collection with the default Latin face as its sans and its
    /// fallback, as a game with no pack has.
    pub(crate) fn latin() -> Collection {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../templates/fonts/latin/NotoSans-Latin.ttf"
        );
        let mut fonts = Collection::new();
        let face = fonts.add(Arc::new(std::fs::read(path).unwrap())).unwrap();
        let family = format!("\"{}\"", fonts.family_name(face).unwrap());
        fonts.set_generic(og_text::fonts::GenericFamily::SansSerif, &family);
        fonts.set_fallback(None, &[face]);
        fonts
    }

    /// A text with no line height set takes CSS's `normal`: its font's
    /// ascent and descent, each rounded (Noto Sans at 20 px: 21 above the
    /// baseline and 6 below, from its hhea 1069 and 293 of 1000), not 1.2
    /// of its size (24). An empty text keeps that line.
    #[test]
    fn a_text_with_no_line_height_takes_its_fonts() {
        let mut fonts = latin();
        let mut texts = Texts::default();
        let run = Run {
            line: -1.0,
            ..Run::sized(20.0)
        };
        let (_, h) = texts.measure(&mut fonts, 1, "Hg", &run, Avail::Max);
        assert_eq!(h, 27.0);
        assert_eq!(
            texts.measure(&mut fonts, 2, "", &run, Avail::Max),
            (0.0, 27.0)
        );
        let fixed = Run { line: 30.0, ..run };
        assert_eq!(
            texts.measure(&mut fonts, 3, "Hg", &fixed, Avail::Max).1,
            30.0
        );
    }

    #[test]
    fn a_line_starting_inside_a_word_is_a_broken_word() {
        // Breaks at spaces, after a hyphen and at a newline are allowed.
        assert!(broken_words("Main quest log", "Main\nquest log\n").is_empty());
        assert!(broken_words("well-known", "well-\nknown\n").is_empty());
        assert!(broken_words("Leader\nboard", "Leader\nboard\n").is_empty());
        assert!(broken_words("One", "One\n").is_empty());
        // Inside a word: listed with the two sides.
        assert_eq!(
            broken_words("Leaderboard", "Leaderb\noard\n"),
            ["Leaderb|oard"]
        );
        assert_eq!(
            broken_words("The Leaderboard", "The\nLeaderb\noard\n"),
            ["Leaderb|oard"]
        );
    }

    /// CSS's intrinsic sizes: max-content is the text broken only at its
    /// newline, min-content its longest word (the whole line when it may
    /// not wrap, a letter under `anywhere`), and a width breaks it there.
    /// A word wider than the box overflows it whole unless `break-word`,
    /// and its own width given back keeps its lines.
    #[test]
    fn a_leaf_measures_by_the_width_it_may_take() {
        let mut fonts = latin();
        let mut texts = Texts::default();
        let run = Run::sized(14.0);
        let text = "Follow the oak through a season\nStay";
        let one = |texts: &mut Texts, fonts: &mut Collection, t: &str, run: &Run| {
            texts.measure(fonts, 9, t, run, Avail::Max)
        };
        let (max, h) = texts.measure(&mut fonts, 1, text, &run, Avail::Max);
        let line = one(
            &mut texts,
            &mut fonts,
            "Follow the oak through a season",
            &run,
        )
        .0;
        assert_eq!((max, h), (line, 2.0 * run.line));
        // Its width given back through layout's sums, a hair less, keeps
        // its lines.
        let back = Avail::Width(f32::from_bits(max.to_bits() - 2));
        assert_eq!(texts.measure(&mut fonts, 1, text, &run, back).1, h);
        let (min, h) = texts.measure(&mut fonts, 1, text, &run, Avail::Min);
        let word = one(&mut texts, &mut fonts, "through", &run).0;
        assert!((min - word).abs() < 0.01, "{min} vs {word}");
        // Broken at that width: "the oak" fits a line.
        assert_eq!(h, 6.0 * run.line);
        let (w, h) = texts.measure(&mut fonts, 1, text, &run, Avail::Width(120.0));
        assert!(w <= 120.0 && h == 3.0 * run.line, "{w} {h}");
        let nowrap = Run {
            nowrap: true,
            ..run
        };
        let (min, h) = texts.measure(&mut fonts, 1, text, &nowrap, Avail::Min);
        assert_eq!((min, h), (line, 2.0 * run.line));
        let anywhere = Run { wrap: 2, ..run };
        let (min, _) = texts.measure(&mut fonts, 1, "through", &anywhere, Avail::Min);
        assert!(min < word / 3.0, "{min}");
        // A box narrower than a word.
        let (w, h) = texts.measure(&mut fonts, 1, "Press any button", &run, Avail::Width(4.0));
        let button = one(&mut texts, &mut fonts, "button", &run).0;
        assert_eq!((w, h), (button, 3.0 * run.line));
        let breaks = Run { wrap: 1, ..run };
        let (w, h) = texts.measure(&mut fonts, 1, "Press", &breaks, Avail::Width(20.0));
        assert!(w <= 20.0 && h > run.line, "{w} {h}");
    }

    /// Paint makes a leaf's quads once: its glyphs on whole device pixels
    /// at their raster's size, the outline and the shadow in lists of
    /// their own, and the same leaf painted again makes nothing.
    #[test]
    fn a_leaf_keeps_its_quads() {
        let mut fonts = latin();
        let mut atlas = Atlas::default();
        let mut texts = Texts::default();
        let run = Run::sized(15.0);
        let mut look = Look {
            scale: 1.0,
            field: false,
            shadow: None,
            stroke: 0.0,
            ink: [1.0; 3],
        };
        let mut missing = Vec::new();
        let (w, _) = texts.measure(&mut fonts, 1, "Go 中", &run, Avail::Max);
        let leaf = texts.paint(
            &mut fonts,
            &mut atlas,
            1,
            "Go 中",
            &run,
            w,
            &look,
            &mut missing,
        );
        assert_eq!(leaf.fill.len(), 2, "G and o; no face has 中");
        assert_eq!(missing, ['中']);
        assert_eq!((leaf.lines.len(), &*leaf.line_text), (1, "Go 中\n"));
        for q in &leaf.fill {
            assert_eq!(q.kind, PaintKind::Raster as u32);
            assert!(q.rect.iter().all(|v| v.fract() == 0.0), "{:?}", q.rect);
            assert_eq!(
                [q.uv[2] - q.uv[0], q.uv[3] - q.uv[1]],
                [q.rect[2], q.rect[3]]
            );
        }
        let glyphs = atlas.stats().glyphs;
        let first = leaf.fill[0].rect;
        let leaf = texts.paint(
            &mut fonts,
            &mut atlas,
            1,
            "Go 中",
            &run,
            w,
            &look,
            &mut missing,
        );
        assert_eq!((leaf.fill[0].rect, atlas.stats().glyphs), (first, glyphs));
        assert_eq!(missing.len(), 1);
        // An outline and a shadow: rasters of their own, under the fill.
        look.stroke = 2.0;
        look.shadow = Some(4.0);
        let leaf = texts.paint(
            &mut fonts,
            &mut atlas,
            1,
            "Go 中",
            &run,
            w,
            &look,
            &mut missing,
        );
        assert_eq!(
            (leaf.fill.len(), leaf.stroke.len(), leaf.shadow.len()),
            (2, 2, 2)
        );
        assert!(leaf.stroke[0].rect[2] > leaf.fill[0].rect[2]);
        assert!(leaf.shadow[0].rect[2] > leaf.stroke[0].rect[2]);
        // G, o and the space (which has no ink), each in two more forms.
        assert_eq!(atlas.stats().glyphs, glyphs + 6);
        // At twice the pixels the rasters are twice the size, a CSS px
        // half a raster pixel; under a transform they are fields.
        look.scale = 2.0;
        let leaf = texts.paint(
            &mut fonts,
            &mut atlas,
            1,
            "Go 中",
            &run,
            w,
            &look,
            &mut missing,
        );
        let q = leaf.fill[0];
        assert_eq!(q.uv[2] - q.uv[0], q.rect[2] * 2.0);
        look.field = true;
        let leaf = texts.paint(
            &mut fonts,
            &mut atlas,
            1,
            "Go 中",
            &run,
            w,
            &look,
            &mut missing,
        );
        assert!(leaf.fill.iter().all(|q| q.a == GlyphTier::Field as u32));
        assert_eq!(leaf.stroke[0].p[0], 1.0, "grown by half the stroke");
    }

    /// `truncate`: a line wider than its box ends in "…" when drawn, and
    /// measures whole.
    #[test]
    fn an_ellipsis_is_drawn_and_never_measured() {
        let mut fonts = latin();
        let mut atlas = Atlas::default();
        let mut texts = Texts::default();
        let run = Run {
            nowrap: true,
            ellipsis: true,
            ..Run::sized(14.0)
        };
        let look = Look {
            scale: 1.0,
            field: false,
            shadow: None,
            stroke: 0.0,
            ink: [1.0; 3],
        };
        let name = "The Warden of the Far Northern Observatory";
        let (whole, _) = texts.measure(&mut fonts, 1, name, &run, Avail::Max);
        let leaf = texts.paint(
            &mut fonts,
            &mut atlas,
            1,
            name,
            &run,
            90.0,
            &look,
            &mut Vec::new(),
        );
        assert!(leaf.line_text.ends_with("\u{2026}\n"), "{}", leaf.line_text);
        assert!(leaf.lines[0][2] <= 90.0);
        let right = leaf
            .fill
            .iter()
            .map(|q| q.rect[0] + q.rect[2])
            .fold(0.0, f32::max);
        assert!(right <= 91.0, "{right}");
        // The caret indexes the text as drawn.
        assert!(texts.caret(1, name.len()).unwrap()[0] <= 90.0);
        assert_eq!(
            texts.measure(&mut fonts, 1, name, &run, Avail::Max).0,
            whole
        );
    }
}
