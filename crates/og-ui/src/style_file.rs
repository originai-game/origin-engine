//! UI styles: a game's whole look, one `.ogt` of `type style`. Its values
//! (every palette role, the radii, the type ramp, durations) go into the
//! [`Theme`] fields; its art is an [`Art`], which dresses the
//! element classes ([`StylePart`]: panel, card, button and its states, field,
//! slot, tooltip, hud, progress track and fill) in nine-slice frames from
//! one atlas texture, adds ornaments (corners, divider, heading flourish), a
//! tiled panel texture, drop shadows and glows per class, text shadows and
//! glows per font role, a press squash and a hover lift, motion choices
//! (entrances, stagger, count-ups, eased progress, refused-action shake,
//! bursts) and UI sounds.
//!
//! A style is complete: it sets every palette role and every radius, so
//! nothing of another look shows through it. There is no base to fall back
//! on; before a game picks a style it has the engine's placeholder
//! `prototype` ([`crate::theme`]). [`Art::default`] dresses nothing and
//! moves nothing: every draw path checks for a slice, an ornament or a
//! motion value first, so a style without art draws flat.
//!
//! Version 2 (docs/ui.md §12) adds the tokens of [`crate::tokens`] (carried
//! in [`Theme::tokens`], not yet drawn from) and dresses parts by selector
//! (`crate::dress`): its dressings of the twelve parts above fill the same
//! [`Art`], so a style upgraded by [`upgrade_style`] draws as before.
//!
//! The grammar is [`parse_style`]'s; the runtime loads a style's atlas,
//! fonts, icons and sounds through the bundle and hands the handles to
//! [`StyleFile::theme`].

use crate::dress::{DRAW_KEYS, DressFrame, Dressing, Selector};
use crate::motion::TransitionKind;
use crate::style::{Durations, Palette, Radius, Scheme, Srgba, Theme, TypeRamp, TypeStep};
use crate::tokens::{
    self, ElementClass, Level, RadiusStep, ShapeToken, SizeStep, SoundEvent, SpaceStep,
    StrokeWidth, StyleTokens, names_of,
};
use og_core::ogt::{Document, Entry, Value};
use og_render::TextureHandle;

/// The `.ogt` type of a UI style.
pub const STYLE_TYPE: &str = "style";

/// An element class a style dresses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StylePart {
    /// Dialogs and anchored screen panels.
    Panel,
    /// Cards (`Ui::panel`), shop and level cells.
    Card,
    Button,
    ButtonHover,
    ButtonPressed,
    ButtonDisabled,
    Field,
    /// Inventory and grid cells.
    Slot,
    Tooltip,
    /// HUD elements over the game (counters, timers) and toasts.
    Hud,
    /// A progress bar's track.
    Track,
    /// A progress bar's fill.
    Fill,
}

/// How many parts there are (the size of the per-part arrays).
pub const PARTS: usize = 12;

impl StylePart {
    pub const ALL: [StylePart; PARTS] = [
        StylePart::Panel,
        StylePart::Card,
        StylePart::Button,
        StylePart::ButtonHover,
        StylePart::ButtonPressed,
        StylePart::ButtonDisabled,
        StylePart::Field,
        StylePart::Slot,
        StylePart::Tooltip,
        StylePart::Hud,
        StylePart::Track,
        StylePart::Fill,
    ];

    pub fn name(self) -> &'static str {
        match self {
            StylePart::Panel => "panel",
            StylePart::Card => "card",
            StylePart::Button => "button",
            StylePart::ButtonHover => "buttonHover",
            StylePart::ButtonPressed => "buttonPressed",
            StylePart::ButtonDisabled => "buttonDisabled",
            StylePart::Field => "field",
            StylePart::Slot => "slot",
            StylePart::Tooltip => "tooltip",
            StylePart::Hud => "hud",
            StylePart::Track => "track",
            StylePart::Fill => "fill",
        }
    }

    pub fn from_name(name: &str) -> Option<StylePart> {
        StylePart::ALL.into_iter().find(|p| p.name() == name)
    }

    pub fn index(self) -> usize {
        self as usize
    }

    /// The class whose shadow, glow, corners and texture apply (a button's
    /// states share the button's).
    pub fn class(self) -> StylePart {
        match self {
            StylePart::ButtonHover | StylePart::ButtonPressed | StylePart::ButtonDisabled => {
                StylePart::Button
            }
            p => p,
        }
    }

    fn bit(self) -> u32 {
        1 << self.class().index()
    }

    /// The part dressing (docs/ui.md §12.3) this version 1 class is:
    /// `Panel.frame`, `Card.frame`, `Button.base` and its states
    /// `[hover]`, `[pressed]`, `[disabled]`, `Field.frame`, `Slot.frame`,
    /// `Tooltip.frame`, `Hud.frame`, `Meter.track`, `Meter.fill`.
    pub fn selector(self) -> Selector {
        let (component, part, state) = match self {
            StylePart::Panel => ("Panel", "frame", None),
            StylePart::Card => ("Card", "frame", None),
            StylePart::Button => ("Button", "base", None),
            StylePart::ButtonHover => ("Button", "base", Some("hover")),
            StylePart::ButtonPressed => ("Button", "base", Some("pressed")),
            StylePart::ButtonDisabled => ("Button", "base", Some("disabled")),
            StylePart::Field => ("Field", "frame", None),
            StylePart::Slot => ("Slot", "frame", None),
            StylePart::Tooltip => ("Tooltip", "frame", None),
            StylePart::Hud => ("Hud", "frame", None),
            StylePart::Track => ("Meter", "track", None),
            StylePart::Fill => ("Meter", "fill", None),
        };
        let mut s = Selector::of(component, part);
        s.flags.extend(state.map(str::to_string));
        s
    }

    /// The version 1 class a selector is exactly, if any.
    pub fn from_selector(s: &Selector) -> Option<StylePart> {
        StylePart::ALL.into_iter().find(|p| p.selector() == *s)
    }
}

/// How a nine-slice fills its edges and centre.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SliceFill {
    /// Stretched along the edge (the default).
    #[default]
    Stretch,
    /// Tiled at the art's own size from the corner, the last tile cut.
    Repeat,
    /// Tiled a whole number of times, each tile resized to fit (CSS `round`).
    Round,
}

/// A nine-slice frame: a region of the atlas whose corners keep their size,
/// whose edges stretch (or tile) along one axis and whose centre stretches
/// (or tiles) both ways.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Slice {
    /// The region `[u0, v0, u1, v1]` (0–1, v down).
    pub uv: [f32; 4],
    /// The corner insets in the region, as UV lengths (left, top, right, bottom).
    pub border_uv: [f32; 4],
    /// The same insets on screen, CSS px.
    pub border: [f32; 4],
    /// Multiply the art by the element's fill colour (white art takes the
    /// palette); false draws the art as authored.
    pub tint: bool,
    /// How the edges and the centre fill their length.
    pub fill: SliceFill,
    /// The middle of the region on screen (its width and height between
    /// the corners), CSS px: one tile when `control` tiles.
    pub middle: [f32; 2],
    /// The mean of the art's centre in linear light, RGB and alpha (`tone
    /// <part> <grey> [alpha]` or `tone <part> <r> <g> <b> <alpha>`; `og
    /// cook` measures it from the atlas for a style that does not set it;
    /// white and opaque when unknown): what the frame really draws where its
    /// label sits ([`Slice::drawn`]), so a widget picks ink that reads on it
    /// (a neon button's dark body under a game's gold primary).
    pub tone: [f32; 4],
}

impl Slice {
    /// A tone that changes nothing: white, opaque.
    pub const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

    /// What the frame draws at its centre for `fill` (straight alpha): the
    /// fill times the art's tone when it tints, the art's own tone when it
    /// is authored; `None` when the tone is unknown (white and opaque on a
    /// tinting frame draws the fill itself).
    pub fn drawn(&self, fill: crate::Srgba) -> Option<crate::Srgba> {
        if self.tone == Self::WHITE {
            return self.tint.then_some(fill);
        }
        let t = self.tone;
        let c = if self.tint {
            let l = fill.linear();
            [l[0] * t[0], l[1] * t[1], l[2] * t[2], l[3] * t[3]]
        } else {
            t
        };
        Some(crate::Srgba::from_linear(c))
    }
}

/// Which ink an ornament takes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Ink {
    /// The art's own colours.
    #[default]
    Authored,
    Primary,
    Ink,
    Muted,
    Line,
    Focus,
}

impl Ink {
    fn parse(w: &str) -> Option<Ink> {
        Some(match w {
            "authored" => Ink::Authored,
            "primary" => Ink::Primary,
            "ink" => Ink::Ink,
            "muted" => Ink::Muted,
            "line" => Ink::Line,
            "focus" => Ink::Focus,
            _ => return None,
        })
    }

    pub fn color(self, p: &Palette) -> Srgba {
        match self {
            Ink::Authored => Srgba::WHITE,
            Ink::Primary => p.primary,
            Ink::Ink => p.ink,
            Ink::Muted => p.ink_muted,
            Ink::Line => p.line_strong,
            Ink::Focus => p.focus,
        }
    }
}

/// An ornament glyph of the atlas drawn at a fixed size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ornament {
    pub uv: [f32; 4],
    /// Size on screen, CSS px.
    pub w: f32,
    pub h: f32,
    pub ink: Ink,
    /// Moves it outwards (corners) or apart (flourishes), CSS px.
    pub offset: f32,
}

/// A tiled texture over a part's fill.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tile {
    pub uv: [f32; 4],
    /// One tile on screen, CSS px.
    pub w: f32,
    pub h: f32,
    pub alpha: f32,
    /// Inset from the element's edges, CSS px.
    pub inset: f32,
    /// The parts it covers (bits of `StylePart::index`).
    pub parts: u32,
}

/// A glass part: the frame under it blurred (og-render's glass), tinted by
/// the part's fill, under the part's own frame drawn at `control` opacity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glass {
    /// The blur's radius in CSS px.
    pub blur: f32,
    /// How opaque the part's own nine-slice stays over the glass (0–1).
    pub fill: f32,
    /// How much of the part's fill colour tints the blur (0–1).
    pub tint: f32,
}

/// A drop shadow (blur 0: hard).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shadow {
    pub dx: f32,
    pub dy: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: Srgba,
}

/// A glow around an element or text.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glow {
    pub size: f32,
    pub ink: Ink,
    pub alpha: f32,
}

/// What a press does to a button: its fill scales by `sx`, `sy` and the
/// button moves down `dy` CSS px, on the style's press spring
/// (`motion.pressSpring`): in while held, back (past rest when the spring
/// bounces) on release.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Press {
    pub sx: f32,
    pub sy: f32,
    pub dy: f32,
}

/// What hovering does to a button, moving on the style's hover spring
/// (`motion.hoverSpring`): it moves `dy` CSS px (negative lifts) and
/// scales by `scale`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hover {
    pub dy: f32,
    pub scale: f32,
}

/// The motion a style chooses. The zero value is the token kit's motion.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StyleMotion {
    /// How dialogs and anchored panels enter (`None`: dialogs scale,
    /// anchored panels appear at once).
    pub enter: Option<TransitionKind>,
    /// Delay between the entrances of a panel's controls, ms (0: together).
    pub stagger_ms: f32,
    /// How toasts arrive (`None`: slide from their edge).
    pub toast: Option<TransitionKind>,
    /// Progress bars ease to a new value over this many ms (0: at once).
    pub progress_ms: f32,
    /// A press on a disabled button shakes it this far, CSS px (0: still).
    pub shake: f32,
    /// Counters pop by this fraction when they finish (0: no pop).
    pub pop: f32,
    /// The spring the hover lift moves on (`None`: `motion.spring.default`
    /// of the style, damping 1 and 350 ms: it settles without overshoot).
    pub hover_spring: Option<crate::theme::Spring>,
    /// The spring the press squash moves on (`None`: damping 0.55 over
    /// `motion.duration.instant`: in at once, a small bounce on release).
    pub press_spring: Option<crate::theme::Spring>,
}

/// The UI events a style can sound.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UiSound {
    Hover,
    Press,
    Open,
    Close,
    Success,
    Error,
}

impl UiSound {
    pub const ALL: [UiSound; 6] = [
        UiSound::Hover,
        UiSound::Press,
        UiSound::Open,
        UiSound::Close,
        UiSound::Success,
        UiSound::Error,
    ];

    pub fn name(self) -> &'static str {
        match self {
            UiSound::Hover => "hover",
            UiSound::Press => "press",
            UiSound::Open => "open",
            UiSound::Close => "close",
            UiSound::Success => "success",
            UiSound::Error => "error",
        }
    }

    pub fn from_name(name: &str) -> Option<UiSound> {
        UiSound::ALL.into_iter().find(|s| s.name() == name)
    }

    /// The version 2 event it is.
    pub fn event(self) -> SoundEvent {
        match self {
            UiSound::Hover => SoundEvent::Hover,
            UiSound::Press => SoundEvent::Press,
            UiSound::Open => SoundEvent::Open,
            UiSound::Close => SoundEvent::Close,
            UiSound::Success => SoundEvent::Success,
            UiSound::Error => SoundEvent::Error,
        }
    }
}

/// A style's art (see the module docs). Copy: the theme
/// carries it by value.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Art {
    /// The atlas the slices, ornaments and tiles cut from; `None` dresses nothing.
    pub atlas: Option<TextureHandle>,
    pub slices: [Option<Slice>; PARTS],
    /// A blurred white rectangle: soft shadows and glows.
    pub soft: Option<Slice>,
    pub corner: Option<Ornament>,
    /// The parts that take corner ornaments (bits).
    pub corner_parts: u32,
    pub divider: Option<Ornament>,
    pub flourish: Option<Ornament>,
    pub tile: Option<Tile>,
    pub shadows: [Option<Shadow>; PARTS],
    pub glows: [Option<Glow>; PARTS],
    /// Parts drawn as glass over the blurred frame.
    pub glass: [Option<Glass>; PARTS],
    /// Per font role (body, display, mono).
    pub text_shadows: [Option<Shadow>; 3],
    pub text_glows: [Option<Glow>; 3],
    /// Per font role: thickening at the glyphs' edge in device pixels
    /// (`textWeight`), so a pixel font's one-pixel strokes, which fall
    /// between device pixels and so never reach their full ink, fill whole
    /// pixels (at a phone's 3 device pixels per CSS pixel a stroke is three
    /// wide and gains a third, not double).
    pub text_weights: [f32; 3],
    pub press: Option<Press>,
    pub hover: Option<Hover>,
    pub motion: StyleMotion,
    /// Sound handles by [`UiSound`] (0: silent).
    pub sounds: [u64; 6],
}

impl Art {
    /// The slice that draws `part`: its own, else the plain button's for a
    /// button state.
    pub fn slice(&self, part: StylePart) -> Option<Slice> {
        self.atlas?;
        self.slices[part.index()].or_else(|| {
            (part.class() == StylePart::Button)
                .then(|| self.slices[StylePart::Button.index()])
                .flatten()
        })
    }

    pub fn shadow(&self, part: StylePart) -> Option<Shadow> {
        self.atlas?;
        self.shadows[part.class().index()]
    }

    pub fn glow(&self, part: StylePart) -> Option<Glow> {
        self.atlas?;
        self.glows[part.class().index()]
    }

    pub fn glass(&self, part: StylePart) -> Option<Glass> {
        self.atlas?;
        self.glass[part.class().index()]
    }

    pub fn corners(&self, part: StylePart) -> Option<Ornament> {
        self.atlas?;
        self.corner.filter(|_| self.corner_parts & part.bit() != 0)
    }

    pub fn tile(&self, part: StylePart) -> Option<Tile> {
        self.atlas?;
        self.tile.filter(|t| t.parts & part.bit() != 0)
    }

    /// The sound of a UI event (0: none).
    pub fn sound(&self, event: UiSound) -> u64 {
        self.sounds[event as usize]
    }
}

/// A style file parsed: the theme it makes and the assets it names.
#[derive(Clone, Debug, PartialEq)]
pub struct StyleFile {
    pub name: String,
    /// The file's `version`: 1 or 2.
    pub version: u32,
    /// The theme with the style's palette, radii, ramp and durations; its
    /// art has no atlas or sounds yet (the loader adds the handles).
    pub theme: Theme,
    /// The atlas texture's asset id.
    pub atlas: Option<String>,
    /// Font asset ids: body, display, numeric (the mono role), fallback
    /// (characters the others lack, a last resort), cjk and cjkDisplay (the
    /// style's own Simplified Chinese faces beside body and display: see
    /// [`crate::Fonts::companions`]).
    pub fonts: [Option<String>; 6],
    /// The icon texture's asset id, its grid and names.
    pub icons: Option<(String, u32, u32, Vec<String>)>,
    /// Sound asset ids by [`UiSound`] (in version 2 the first of the
    /// event's set).
    pub sounds: [Option<String>; 6],
    /// Each [`SoundEvent`]'s sound set, asset ids (empty: silent).
    pub feedback_sounds: [Vec<String>; SoundEvent::COUNT],
    /// The part dressings (`crate::dress`), in file order; a version 1
    /// style's are its twelve parts' ([`StylePart::selector`]).
    pub dressings: Vec<Dressing>,
}

impl StyleFile {
    /// Every asset id the file names (the cook checks they exist).
    pub fn references(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        out.extend(self.atlas.clone());
        out.extend(self.fonts.iter().flatten().cloned());
        out.extend(self.icons.as_ref().map(|i| i.0.clone()));
        out.extend(self.sounds.iter().flatten().cloned());
        for s in self.feedback_sounds.iter().flatten() {
            if !out.contains(s) {
                out.push(s.clone());
            }
        }
        out
    }

    /// The line that sets `part`'s tone (`og cook` appends what it
    /// measured): `tone <part> …` in version 1, a `dress` block in version 2.
    pub fn tone_text(&self, part: StylePart, t: [f32; 4]) -> String {
        let v = format!("{:.3} {:.3} {:.3} {:.2}", t[0], t[1], t[2], t[3]);
        if self.version >= 2 {
            format!("dress \"{}\" {{\n  tone {v}\n}}", part.selector())
        } else {
            format!("tone {} {v}", part.name())
        }
    }

    /// Resolves the asset ids the file names against the style file's own
    /// id (`style_id`, such as `styles/fantasy/style.ogt`), so a pack copied
    /// to another directory keeps working: an id is first read relative to
    /// the style's directory (`atlas.png` is `styles/fantasy/atlas.png`,
    /// `../shared/rope.png` climbs one), and taken as written, from
    /// `assets/`, when no asset `exists` there. The cook and the runtime
    /// resolve the same way, so what the cook checked is what loads.
    pub fn resolve(&mut self, style_id: &str, exists: impl Fn(&str) -> bool) {
        let at = |r: &mut String| *r = resolve_reference(style_id, r, &exists);
        if let Some(a) = self.atlas.as_mut() {
            at(a);
        }
        for f in self.fonts.iter_mut().flatten() {
            at(f);
        }
        if let Some(i) = self.icons.as_mut() {
            at(&mut i.0);
        }
        for s in self.sounds.iter_mut().flatten() {
            at(s);
        }
        for s in self.feedback_sounds.iter_mut().flatten() {
            at(s);
        }
    }

    /// The theme with the loaded atlas and sound handles.
    pub fn theme(&self, atlas: Option<TextureHandle>, sounds: [u64; 6]) -> Theme {
        let mut t = self.theme;
        t.art.atlas = atlas;
        t.art.sounds = sounds;
        for u in UiSound::ALL {
            t.tokens.feedback[u.event().index()].sounds[0] = sounds[u as usize];
        }
        t
    }
}

/// An asset id a style file names, resolved against the style's own id (see
/// [`StyleFile::resolve`]): relative to its directory when that asset
/// exists, else as written.
pub fn resolve_reference(style_id: &str, reference: &str, exists: impl Fn(&str) -> bool) -> String {
    let dir = style_id.rsplit_once('/').map_or("", |(d, _)| d);
    let mut parts: Vec<&str> = dir.split('/').filter(|p| !p.is_empty()).collect();
    let mut inside = true;
    for p in reference.split('/') {
        match p {
            "" | "." => {}
            ".." => inside &= parts.pop().is_some(),
            p => parts.push(p),
        }
    }
    let relative = parts.join("/");
    if inside && !relative.is_empty() && exists(&relative) {
        relative
    } else {
        reference.to_string()
    }
}

/// A palette colour by its ThemeColor name (`primary`, `inkMuted`, `panelRaised`).
pub fn palette_color<'a>(p: &'a mut Palette, name: &str) -> Option<&'a mut Srgba> {
    p.get_mut(name)
}

/// A type step by its TypeStepName (`body`, `xl2`, `displayLg`).
pub fn ramp_step<'a>(r: &'a mut TypeRamp, name: &str) -> Option<&'a mut TypeStep> {
    Some(match name {
        "body" => &mut r.body,
        "xs" => &mut r.xs,
        "meta" => &mut r.meta,
        "caption" => &mut r.caption,
        "sm" => &mut r.sm,
        "reading" => &mut r.reading,
        "base" => &mut r.base,
        "lg" => &mut r.lg,
        "xl" => &mut r.xl,
        "xl2" => &mut r.xl2,
        "xl3" => &mut r.xl3,
        "xl4" => &mut r.xl4,
        "xl5" => &mut r.xl5,
        "xl6" => &mut r.xl6,
        "xl7" => &mut r.xl7,
        "xl8" => &mut r.xl8,
        "displaySm" => &mut r.display_sm,
        "displayMd" => &mut r.display_md,
        "displayLg" => &mut r.display_lg,
        "displayXl" => &mut r.display_xl,
        "displayXl2" => &mut r.display_xl2,
        _ => return None,
    })
}

/// The radius names, all of which a style sets.
const RADII: [&str; 4] = ["small", "control", "bar", "panel"];

fn radius_slot<'a>(r: &'a mut Radius, name: &str) -> Option<&'a mut f32> {
    Some(match name {
        "small" => &mut r.small,
        "control" => &mut r.control,
        "bar" => &mut r.bar,
        "panel" => &mut r.panel,
        _ => return None,
    })
}

fn duration_slot<'a>(d: &'a mut Durations, name: &str) -> Option<&'a mut f32> {
    Some(match name {
        "base" => &mut d.base,
        "bounce" => &mut d.bounce,
        "fast" => &mut d.fast,
        "instant" => &mut d.instant,
        "slow" => &mut d.slow,
        "spring" => &mut d.spring,
        _ => return None,
    })
}

fn kind_of(w: &str) -> Option<TransitionKind> {
    Some(match w {
        "fade" => TransitionKind::Fade,
        "slideUp" => TransitionKind::SlideUp,
        "slideDown" => TransitionKind::SlideDown,
        "slideLeft" => TransitionKind::SlideLeft,
        "slideRight" => TransitionKind::SlideRight,
        "scale" => TransitionKind::Scale,
        "pop" => TransitionKind::Pop,
        _ => return None,
    })
}

/// `#rrggbb` or `#rrggbbaa`.
pub fn parse_color(s: &str) -> Option<Srgba> {
    let h = s.strip_prefix('#')?;
    if !(h.len() == 6 || h.len() == 8) || !h.is_ascii() {
        return None;
    }
    let byte = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).ok();
    Some(Srgba {
        rgb: [byte(0)?, byte(2)?, byte(4)?],
        alpha: if h.len() == 8 {
            f32::from(byte(6)?) / 255.0
        } else {
            1.0
        },
    })
}

pub(crate) struct P<'a> {
    pub(crate) e: &'a Entry,
}

impl<'a> P<'a> {
    pub(crate) fn err(&self, what: &str) -> String {
        format!("line {}: `{}` {what}", self.e.line, self.e.key)
    }
    pub(crate) fn num(&self, i: usize) -> Result<f32, String> {
        self.e
            .values
            .get(i)
            .and_then(Value::as_number)
            .map(|n| n.as_f32())
            .filter(|v| v.is_finite())
            .ok_or_else(|| self.err(&format!("takes a number at position {}", i + 1)))
    }
    pub(crate) fn num_or(&self, i: usize, d: f32) -> Result<f32, String> {
        if self.e.values.len() > i {
            self.num(i)
        } else {
            Ok(d)
        }
    }
    pub(crate) fn word(&self, i: usize) -> Result<&'a str, String> {
        self.e
            .values
            .get(i)
            .and_then(Value::as_word)
            .ok_or_else(|| self.err(&format!("takes a word at position {}", i + 1)))
    }
    pub(crate) fn string(&self, i: usize) -> Result<&'a str, String> {
        self.e
            .values
            .get(i)
            .and_then(Value::as_str)
            .ok_or_else(|| self.err(&format!("takes a quoted string at position {}", i + 1)))
    }
    pub(crate) fn color(&self, i: usize) -> Result<Srgba, String> {
        let s = self.string(i)?;
        parse_color(s)
            .ok_or_else(|| self.err(&format!("takes \"#rrggbb\" or \"#rrggbbaa\", not \"{s}\"")))
    }
    pub(crate) fn block(&self) -> Result<&'a og_core::ogt::Block, String> {
        self.e
            .block
            .as_ref()
            .ok_or_else(|| self.err("takes a { } block"))
    }
}

/// The atlas region `x y w h` (texels) at `values[i..i + 4]` as UVs.
fn region(p: &P, i: usize, size: (f32, f32)) -> Result<([f32; 4], [f32; 4]), String> {
    let (x, y, w, h) = (p.num(i)?, p.num(i + 1)?, p.num(i + 2)?, p.num(i + 3)?);
    if w <= 0.0 || h <= 0.0 || x < 0.0 || y < 0.0 || x + w > size.0 || y + h > size.1 {
        return Err(p.err(&format!(
            "region {x} {y} {w} {h} is not inside the {}×{} atlas",
            size.0, size.1
        )));
    }
    Ok((
        [x / size.0, y / size.1, (x + w) / size.0, (y + h) / size.1],
        [x, y, w, h],
    ))
}

fn parts_mask(p: &P, from: usize) -> Result<u32, String> {
    let mut mask = 0;
    for i in from..p.e.values.len() {
        let w = p.word(i)?;
        let part = StylePart::from_name(w)
            .ok_or_else(|| p.err(&format!("names an unknown part '{w}'")))?;
        mask |= part.bit();
    }
    Ok(mask)
}

fn role_index(p: &P, w: &str) -> Result<usize, String> {
    match w {
        "body" => Ok(0),
        "display" => Ok(1),
        "numeric" | "mono" => Ok(2),
        _ => Err(p.err(&format!("names the role '{w}' (body, display or numeric)"))),
    }
}

/// A nine-slice at `values[at..]`: the region `x y w h` in texels, one
/// corner inset or four, then `tint` or `authored` and `stretch`, `repeat`
/// or `round`.
fn slice_at(p: &P, at: usize, size: (f32, f32), scale: f32) -> Result<Slice, String> {
    let e = p.e;
    let (uv, px) = region(p, at, size)?;
    let numbers = e.values[(at + 4).min(e.values.len())..]
        .iter()
        .take_while(|v| v.as_number().is_some())
        .count();
    let b = match numbers {
        1 => {
            let v = p.num(at + 4)?;
            [v; 4]
        }
        4 => [
            p.num(at + 4)?,
            p.num(at + 5)?,
            p.num(at + 6)?,
            p.num(at + 7)?,
        ],
        _ => {
            return Err(p.err("takes one corner inset or four (left top right bottom), in texels"));
        }
    };
    if b[0] + b[2] > px[2] || b[1] + b[3] > px[3] || b.iter().any(|v| *v < 0.0) {
        return Err(p.err("has corner insets larger than its region"));
    }
    let (mut tint, mut fill) = (true, SliceFill::Stretch);
    for v in &e.values[at + 4 + numbers..] {
        match v.as_word() {
            Some("tint") => tint = true,
            Some("authored") => tint = false,
            Some("stretch") => fill = SliceFill::Stretch,
            Some("repeat") => fill = SliceFill::Repeat,
            Some("round") => fill = SliceFill::Round,
            _ => {
                return Err(p.err(&format!(
                    "ends with tint or authored and stretch, repeat or round, not '{}'",
                    v.as_word().unwrap_or("a number")
                )));
            }
        }
    }
    let middle = [(px[2] - b[0] - b[2]) * scale, (px[3] - b[1] - b[3]) * scale];
    if fill != SliceFill::Stretch && (middle[0] < 1.0 || middle[1] < 1.0) {
        return Err(p.err("tiles its edges but leaves no middle between the corners to tile"));
    }
    Ok(Slice {
        uv,
        border_uv: [b[0] / size.0, b[1] / size.1, b[2] / size.0, b[3] / size.1],
        border: [b[0] * scale, b[1] * scale, b[2] * scale, b[3] * scale],
        tint,
        fill,
        middle,
        tone: Slice::WHITE,
    })
}

/// A slice's tone at `values[at..]`: a grey and an alpha, or r g b a (linear).
fn tone_at(p: &P, at: usize) -> Result<[f32; 4], String> {
    let tone = match p.e.values.len().saturating_sub(at) {
        1 | 2 => {
            let v = p.num(at)?;
            [v, v, v, p.num_or(at + 1, 1.0)?]
        }
        4 => [p.num(at)?, p.num(at + 1)?, p.num(at + 2)?, p.num(at + 3)?],
        _ => return Err(p.err("takes a grey and an alpha, or r g b and an alpha")),
    };
    if tone.iter().any(|v| !(0.0..=1.0).contains(v)) {
        return Err(p.err("takes values from 0 to 1 (linear light)"));
    }
    Ok(tone)
}

fn ink_at(p: &P, i: usize) -> Result<Ink, String> {
    let iw = p.word(i)?;
    Ink::parse(iw).ok_or_else(|| {
        p.err(&format!(
            "ink '{iw}' is authored, primary, ink, muted, line or focus"
        ))
    })
}

fn glow_at(p: &P, at: usize) -> Result<Glow, String> {
    Ok(Glow {
        size: p.num(at)?.max(0.0),
        ink: ink_at(p, at + 1)?,
        alpha: p.num(at + 2)?.clamp(0.0, 1.0),
    })
}

fn glass_at(p: &P, at: usize) -> Result<Glass, String> {
    Ok(Glass {
        blur: p.num(at)?.max(0.0),
        fill: p.num(at + 1)?.clamp(0.0, 1.0),
        tint: p.num_or(at + 2, 0.25)?.clamp(0.0, 1.0),
    })
}

fn press_at(p: &P) -> Result<Press, String> {
    Ok(Press {
        sx: p.num(0)?,
        sy: p.num(1)?,
        dy: p.num_or(2, 0.0)?,
    })
}

fn hover_at(p: &P) -> Result<Hover, String> {
    Ok(Hover {
        dy: p.num(0)?,
        scale: p.num_or(1, 1.0)?,
    })
}

fn part_at(p: &P, i: usize) -> Result<StylePart, String> {
    let w = p.word(i)?;
    StylePart::from_name(w).ok_or_else(|| p.err(&format!("names an unknown part '{w}'")))
}

/// The dressings a version 1 style's art makes: each of the twelve parts
/// at its [`StylePart::selector`], its class's shadow, glow, glass, corners
/// and tile on the class, the press and hover on `Button.base`.
fn v1_dressings(art: &Art) -> Vec<Dressing> {
    let mut out = Vec::new();
    for part in StylePart::ALL {
        let mut f = DressFrame {
            slice: art.slices[part.index()],
            ..DressFrame::default()
        };
        if part.class() == part {
            let i = part.index();
            f.shadow = art.shadows[i];
            f.glow = art.glows[i];
            f.glass = art.glass[i];
            f.corners = art.corner_parts & part.bit() != 0;
            f.tile = art.tile.is_some_and(|t| t.parts & part.bit() != 0);
        }
        if part == StylePart::Button {
            f.press = art.press;
            f.hover = art.hover;
        }
        if f != DressFrame::default() {
            out.push(Dressing {
                selector: part.selector(),
                frame: f,
                draw: Default::default(),
            });
        }
    }
    out
}

/// What a version 2 file's blocks give, before they are checked complete.
#[derive(Default)]
struct V2 {
    colors: Option<tokens::ColorTokens>,
    typography: Option<tokens::TypeScale>,
    space: Option<[f32; SpaceStep::COUNT]>,
    size: Option<[f32; SizeStep::COUNT]>,
    radius: Option<[f32; RadiusStep::COUNT]>,
    stroke: Option<[f32; StrokeWidth::COUNT]>,
    elevation: Option<[tokens::Elevation; Level::COUNT]>,
    shape: Option<[ShapeToken; ElementClass::COUNT]>,
    motion: Option<tokens::MotionParse>,
    springs: Vec<(&'static str, tokens::SpringToken)>,
    feedback: Option<[tokens::Feedback; SoundEvent::COUNT]>,
}

fn once<T>(p: &P, slot: &mut Option<T>, v: T) -> Result<(), String> {
    if slot.is_some() {
        return Err(p.err("is set twice"));
    }
    *slot = Some(v);
    Ok(())
}

/// Parses a UI style. Version 2 (docs/ui.md §12):
///
/// ```text
/// type style
/// version 2
/// ---
/// name "fantasy"
/// scheme dark                                # dark (the default) or light: what the game reads as UiTheme.scheme
/// atlas "atlas.png" 384 256                  # its asset id and size in texels
/// scale 1                                    # CSS px per atlas texel for slice corners
/// palette { backdrop "#1a120b" … }           # every ThemeColor role (all required)
/// colors {                                   # every token required
///   success icon "check"                     # a status: the palette's colour, its cue (icon "<name>", pattern <word>, pips <n>)
///   epic "#a335ee" pips 4                    # common … mythic: a colour and its cue, a different cue per tier
///   team1 "#3d7eff"                          # team1 … team8, then physical fire ice lightning poison holy shadow arcane: a colour [and a cue]
///   series1 "#4e79a7"                        # series1 … series8
/// }
/// typography {                               # display title heading body label caption numeric (all required)
///   display 42 48 0.02 700 display {         # size, line height, tracking em, weight, font role (body, display, numeric, rounded)
///     phonePortrait 34 40                    # per breakpoint: size, line height
///   }
/// }
/// space { xxs 2 xs 4 sm 8 md 12 lg 16 xl 24 xxl 32 xl3 48 }  # one per line, every name
/// size { xs 16 sm 24 md 32 lg 48 xl 64 xxl 96 touch 44 }
/// radius { xs 2 sm 4 md 6 lg 9 xl 12 pill 9999 }
/// stroke { hairline 1 thin 1.5 regular 2 thick 3 focus 2 }
/// elevation { raised { shadow 0 4 8 0 "#00000066" } floating … overlay … top … }   # or `<level> none`
/// shape { panel round 4 button squircle md slot notch 6 … }  # every element class: corner and radius (px or a radius name)
/// ramp { displayLg 44 48 }                   # the v1 TypeStepName ramp the widgets draw with
/// fonts { … }  icons "icons.png" 8 4 { names … }
/// soft 252 104 64 64 24  corner …  divider …  flourish …  tile 204 172 64 64 64 64 0.5 6
/// textShadow display 0 2 0 "#000000a0"  textGlow …  textWeight …
/// motion {
///   enter scale  toast slideDown  progress 420  shake 5  pop 0.1  hoverSpring 350 0
///   spring ui 180 0                          # ui panel emphasis playful: duration ms, bounce -0.75 … 0.95
///   ease enter 0.23 1 0.32 1                 # standard enter exit emphasis: control points, linear or steps <n>
///   duration base 300                        # instant fast base slow spring bounce
///   stagger 35  staggerCap 300  overshoot 0.15
/// }
/// sounds {                                   # every event of §7.8
///   press "audio/press.wav" volume 0.8 1 pitch 0.95 1.05 haptic tick
///   hover none haptic none                   # several ids: a set played without repeats
/// }
/// dress "Button.base[pressed]" {             # a part dressing (crate::dress)
///   slice 276 4 48 48 10 tint
///   tone 0.69 1
///   shadow 0 1 2 0 "#00000080"
///   fill linear 90 "#ffd34a" "#e08a1e"        # paints, strokes, effects… kept for the UI pass
/// }
/// ```
///
/// Version 1 (read until `og style upgrade` has run everywhere) has
/// `radius { small control bar panel }`, `durations`, and the per-part lines
/// `slice <part> …`, `tone <part> …`, `shadow <part> …`, `glow <part> …`,
/// `glass <part> …`, `corners <parts>`, `tile … <parts>`, `press`, `hover`,
/// `sound <event> "<id>"`, with `hoverSpring` and `pressSpring` as damping
/// ratio and response ms; its tokens are [`StyleTokens::derive`]d.
pub fn parse_style(text: &str) -> Result<StyleFile, String> {
    let doc = Document::parse(text).map_err(|e| e.to_string())?;
    if doc.ty == "skin" {
        return Err(
            "`type skin` is now `type style`: change the first line, replace `base` with `scheme`, and give the palette every role and the radius block small, control, bar and panel".into(),
        );
    }
    if doc.ty != STYLE_TYPE {
        return Err(format!(
            "a UI style is `type {STYLE_TYPE}`, not `type {}`",
            doc.ty
        ));
    }
    let v2 = match doc.version {
        1 => false,
        2 => true,
        v => return Err(format!("style version {v} (this engine reads 1 and 2)")),
    };
    let body = &doc.body;
    if let Some(e) = body.get("base") {
        return Err(P { e }.err(
            "`base` is gone: a style is complete; write `scheme dark` or `scheme light` and set every palette role",
        ));
    }
    let scheme = match body.get("scheme").map(|e| P { e }.word(0)).transpose()? {
        None | Some("dark") => Scheme::Dark,
        Some("light") => Scheme::Light,
        Some(w) => return Err(format!("scheme is dark or light, not '{w}'")),
    };
    let mut theme = Theme::new(scheme);
    let mut file = StyleFile {
        name: String::new(),
        version: doc.version,
        theme,
        atlas: None,
        fonts: Default::default(),
        icons: None,
        sounds: Default::default(),
        feedback_sounds: Default::default(),
        dressings: Vec::new(),
    };
    let mut size = (0.0f32, 0.0f32);
    if let Some(e) = body.get("atlas") {
        let p = P { e };
        file.atlas = Some(p.string(0)?.to_string());
        size = (p.num(1)?, p.num(2)?);
        if size.0 < 1.0 || size.1 < 1.0 {
            return Err(p.err("takes the atlas width and height in texels"));
        }
    }
    let mut scale = 1.0f32;
    let (mut has_palette, mut has_radius) = (false, false);
    if let Some(e) = body.get("scale") {
        scale = P { e }.num(0)?;
    }
    // The icons first: a colour's cue names one.
    if let Some(e) = body.get("icons") {
        let p = P { e };
        let names = match &e.block {
            Some(b) => b
                .get_all("names")
                .flat_map(|n| n.values.iter())
                .map(|v| {
                    v.as_str()
                        .map(str::to_string)
                        .ok_or_else(|| p.err("names takes quoted strings"))
                })
                .collect::<Result<Vec<_>, _>>()?,
            None => Vec::new(),
        };
        let (cols, rows) = (p.num(1)?, p.num(2)?);
        if cols < 1.0 || rows < 1.0 {
            return Err(p.err("takes the grid's columns and rows (at least 1)"));
        }
        file.icons = Some((p.string(0)?.to_string(), cols as u32, rows as u32, names));
    }
    let icon_names: Vec<String> = file.icons.as_ref().map(|i| i.3.clone()).unwrap_or_default();
    let mut v = V2::default();
    // The radius scale first: a shape names its steps.
    if v2 && let Some(e) = body.get("radius") {
        v.radius = Some(tokens::parse_lengths(
            &P { e },
            &names_of(RadiusStep::ALL, RadiusStep::name),
            "radius",
        )?);
    }
    let mut dressings: Vec<Dressing> = Vec::new();
    for e in &body.entries {
        let p = P { e };
        let needs_atlas = || {
            if file.atlas.is_none() {
                Err(p.err("needs an `atlas` line first"))
            } else {
                Ok(())
            }
        };
        let key = e.key.as_str();
        const V1_ONLY: [&str; 10] = [
            "slice",
            "tone",
            "shadow",
            "glow",
            "glass",
            "corners",
            "press",
            "hover",
            "sound",
            "durations",
        ];
        const V2_ONLY: [&str; 9] = [
            "colors",
            "typography",
            "space",
            "size",
            "stroke",
            "elevation",
            "shape",
            "sounds",
            "dress",
        ];
        if v2 && V1_ONLY.contains(&key) {
            return Err(p.err(match key {
                "durations" => "is a version 1 block: in version 2 the durations are `duration <name> <ms>` lines of the motion block",
                "sound" => "is a version 1 line: in version 2 every event is a line of the `sounds { }` block",
                _ => "is a version 1 part line: in version 2 a part is dressed in a `dress \"<Component.part>\" { … }` block (og style upgrade converts a version 1 style)",
            }));
        }
        if !v2 && V2_ONLY.contains(&key) {
            return Err(p.err(
                "is a version 2 block: write `version 2` and the rest of the version 2 tokens (og style upgrade converts a version 1 style)",
            ));
        }
        match key {
            "scheme" | "atlas" | "scale" | "icons" => {}
            "name" => file.name = p.string(0)?.to_string(),
            "palette" => {
                let block = p.block()?;
                for c in &block.entries {
                    let q = P { e: c };
                    let color = q.color(0)?;
                    *palette_color(&mut theme.palette, &c.key)
                        .ok_or_else(|| q.err("is not a ThemeColor name"))? = color;
                }
                let missing: Vec<&str> = Palette::NAMES
                    .iter()
                    .copied()
                    .filter(|n| block.get(n).is_none())
                    .collect();
                if !missing.is_empty() {
                    return Err(p.err(&format!(
                        "leaves out {}: a style sets every palette role",
                        missing.join(", ")
                    )));
                }
                has_palette = true;
            }
            "radius" if v2 => {}
            "radius" => {
                let block = p.block()?;
                for c in &block.entries {
                    let q = P { e: c };
                    *radius_slot(&mut theme.radius, &c.key).ok_or_else(|| {
                        q.err("is not a radius name (small, control, bar, panel)")
                    })? = q.num(0)?;
                }
                let missing: Vec<&str> = RADII
                    .iter()
                    .copied()
                    .filter(|n| block.get(n).is_none())
                    .collect();
                if !missing.is_empty() {
                    return Err(p.err(&format!(
                        "leaves out {}: a style sets every radius",
                        missing.join(", ")
                    )));
                }
                has_radius = true;
            }
            "ramp" => {
                for c in &p.block()?.entries {
                    let q = P { e: c };
                    let s = ramp_step(&mut theme.type_ramp, &c.key)
                        .ok_or_else(|| q.err("is not a TypeStepName"))?;
                    s.size = q.num(0)?;
                    s.line_height = q.num(1)?;
                    s.cjk_line_height = q.num_or(2, s.line_height.max(s.cjk_line_height))?;
                    s.tracking_em = q.num_or(3, s.tracking_em)?;
                    s.cjk_tracking_em = q.num_or(4, s.cjk_tracking_em)?;
                }
            }
            "durations" => {
                for c in &p.block()?.entries {
                    let q = P { e: c };
                    *duration_slot(&mut theme.durations_ms, &c.key)
                        .ok_or_else(|| q.err("is not a duration name"))? = q.num(0)?.max(0.0);
                }
            }
            "fonts" => {
                for c in &p.block()?.entries {
                    let q = P { e: c };
                    let i = match c.key.as_str() {
                        "body" => 0,
                        "display" => 1,
                        "numeric" | "mono" => 2,
                        "fallback" => 3,
                        "cjk" => 4,
                        "cjkDisplay" => 5,
                        _ => {
                            return Err(q.err(
                                "is a font role: body, display, numeric, fallback, cjk or cjkDisplay",
                            ));
                        }
                    };
                    file.fonts[i] = Some(q.string(0)?.to_string());
                }
            }
            "slice" => {
                needs_atlas()?;
                let part = part_at(&p, 0)?;
                theme.art.slices[part.index()] = Some(slice_at(&p, 1, size, scale)?);
            }
            "soft" => {
                needs_atlas()?;
                theme.art.soft = Some(slice_at(&p, 0, size, scale)?);
            }
            "tone" => {
                let part = part_at(&p, 0)?;
                let slice = theme.art.slices[part.index()].as_mut().ok_or_else(|| {
                    p.err(&format!(
                        "names '{}', which has no slice line before it",
                        part.name()
                    ))
                })?;
                slice.tone = tone_at(&p, 1)?;
            }
            "corner" | "divider" | "flourish" => {
                needs_atlas()?;
                let (uv, _) = region(&p, 0, size)?;
                let ink = match e.values.get(6) {
                    None => Ink::Authored,
                    Some(_) => ink_at(&p, 6)?,
                };
                let o = Ornament {
                    uv,
                    w: p.num(4)?,
                    h: p.num(5)?,
                    ink,
                    offset: p.num_or(7, 0.0)?,
                };
                match key {
                    "corner" => theme.art.corner = Some(o),
                    "divider" => theme.art.divider = Some(o),
                    _ => theme.art.flourish = Some(o),
                }
            }
            "corners" => theme.art.corner_parts = parts_mask(&p, 0)?,
            "tile" => {
                needs_atlas()?;
                let (uv, _) = region(&p, 0, size)?;
                let numbers = e.values[4.min(e.values.len())..]
                    .iter()
                    .take_while(|v| v.as_number().is_some())
                    .count();
                if v2 && e.values.len() > 4 + numbers {
                    return Err(p.err(
                        "names its parts, as version 1 did: in version 2 each part's dressing says `tile`",
                    ));
                }
                theme.art.tile = Some(Tile {
                    uv,
                    w: p.num(4)?,
                    h: p.num(5)?,
                    alpha: p.num(6)?.clamp(0.0, 1.0),
                    inset: if numbers >= 4 { p.num(7)? } else { 0.0 },
                    parts: parts_mask(&p, 4 + numbers)?,
                });
            }
            "shadow" => {
                let part = part_at(&p, 0)?;
                theme.art.shadows[part.class().index()] = Some(tokens::shadow_at(&p, 1)?);
            }
            "glow" => {
                let part = part_at(&p, 0)?;
                theme.art.glows[part.class().index()] = Some(glow_at(&p, 1)?);
            }
            "glass" => {
                let part = part_at(&p, 0)?;
                theme.art.glass[part.class().index()] = Some(glass_at(&p, 1)?);
            }
            "textShadow" => {
                let i = role_index(&p, p.word(0)?)?;
                theme.art.text_shadows[i] = Some(Shadow {
                    dx: p.num(1)?,
                    dy: p.num(2)?,
                    blur: p.num(3)?.max(0.0),
                    spread: 0.0,
                    color: p.color(4)?,
                });
            }
            "textGlow" => {
                let i = role_index(&p, p.word(0)?)?;
                theme.art.text_glows[i] = Some(glow_at(&p, 1)?);
            }
            "textWeight" => {
                let i = role_index(&p, p.word(0)?)?;
                let w = p.num(1)?;
                if !(0.0..=2.0).contains(&w) {
                    return Err(p.err("takes the thickening in device pixels, 0 to 2"));
                }
                theme.art.text_weights[i] = w;
            }
            "press" => theme.art.press = Some(press_at(&p)?),
            "hover" => theme.art.hover = Some(hover_at(&p)?),
            "motion" => {
                let mut tokens_motion = v2.then(tokens::MotionParse::new);
                if v2 && v.motion.is_some() {
                    return Err(p.err("is set twice"));
                }
                let m = &mut theme.art.motion;
                for c in &p.block()?.entries {
                    let q = P { e: c };
                    match c.key.as_str() {
                        "enter" | "toast" => {
                            let w = q.word(0)?;
                            let k = kind_of(w).ok_or_else(|| q.err(&format!("'{w}' is fade, slideUp, slideDown, slideLeft, slideRight, scale or pop")))?;
                            if c.key == "enter" {
                                m.enter = Some(k);
                            } else {
                                m.toast = Some(k);
                            }
                        }
                        "stagger" if !v2 => m.stagger_ms = q.num(0)?.max(0.0),
                        "progress" => m.progress_ms = q.num(0)?.max(0.0),
                        "shake" => m.shake = q.num(0)?.max(0.0),
                        "pop" => m.pop = q.num(0)?.clamp(0.0, 1.0),
                        "hoverSpring" | "pressSpring" => {
                            let spring = if v2 {
                                // Version 2: duration and bounce, as every spring.
                                let s = tokens::spring_at(&q, 0)?;
                                v.springs.push((
                                    if c.key == "hoverSpring" {
                                        "hoverSpring"
                                    } else {
                                        "pressSpring"
                                    },
                                    s,
                                ));
                                s.to_spring()
                            } else {
                                crate::theme::Spring {
                                    damping: q.num(0)?.clamp(0.05, 4.0),
                                    response_ms: q.num(1)?.clamp(16.0, 4000.0),
                                }
                            };
                            if c.key == "hoverSpring" {
                                m.hover_spring = Some(spring);
                            } else {
                                m.press_spring = Some(spring);
                            }
                        }
                        _ => {
                            if let Some(t) = tokens_motion.as_mut()
                                && t.line(&q)?
                            {
                                continue;
                            }
                            return Err(q.err(if v2 {
                                "is a motion setting: enter, toast, progress, shake, pop, hoverSpring, pressSpring, spring, ease, duration, stagger, staggerCap or overshoot"
                            } else {
                                "is a motion setting: enter, stagger, toast, progress, shake, pop, hoverSpring or pressSpring"
                            }));
                        }
                    }
                }
                v.motion = tokens_motion;
            }
            "sound" => {
                let w = p.word(0)?;
                let s = UiSound::from_name(w).ok_or_else(|| {
                    p.err(&format!(
                        "'{w}' is hover, press, open, close, success or error"
                    ))
                })?;
                file.sounds[s as usize] = Some(p.string(1)?.to_string());
            }
            "colors" => once(&p, &mut v.colors, tokens::parse_colors(&p, &icon_names)?)?,
            "typography" => once(&p, &mut v.typography, tokens::parse_typography(&p)?)?,
            "space" => once(
                &p,
                &mut v.space,
                tokens::parse_lengths(&p, &names_of(SpaceStep::ALL, SpaceStep::name), "space")?,
            )?,
            "size" => once(
                &p,
                &mut v.size,
                tokens::parse_lengths(&p, &names_of(SizeStep::ALL, SizeStep::name), "size")?,
            )?,
            "stroke" => once(
                &p,
                &mut v.stroke,
                tokens::parse_lengths(
                    &p,
                    &names_of(StrokeWidth::ALL, StrokeWidth::name),
                    "stroke width",
                )?,
            )?,
            "elevation" => once(&p, &mut v.elevation, tokens::parse_elevation(&p)?)?,
            "shape" => {
                let radius = v.radius.as_ref().ok_or_else(|| {
                    p.err("needs the `radius { xs sm md lg xl pill }` scale its corners name")
                })?;
                let s = tokens::parse_shape(&p, radius)?;
                once(&p, &mut v.shape, s)?
            }
            "sounds" => {
                let (f, ids) = tokens::parse_sounds(&p)?;
                once(&p, &mut v.feedback, f)?;
                file.feedback_sounds = ids;
            }
            "dress" => {
                let sel = Selector::parse(p.string(0)?).map_err(|m| p.err(&m))?;
                if e.values.len() != 1 {
                    return Err(p.err("takes one quoted selector, then a { } block"));
                }
                let block = p.block()?;
                let i = match dressings.iter().position(|d| d.selector == sel) {
                    Some(i) => i,
                    None => {
                        dressings.push(Dressing {
                            selector: sel,
                            frame: DressFrame::default(),
                            draw: Default::default(),
                        });
                        dressings.len() - 1
                    }
                };
                let d = &mut dressings[i];
                let atlas = crate::paint_entry::AtlasContext {
                    size: file.atlas.as_ref().map(|_| size),
                    scale,
                };
                let mut seen = crate::dress::DrawSeen::default();
                for c in &block.entries {
                    let q = P { e: c };
                    let bare = || {
                        if c.values.is_empty() {
                            Ok(true)
                        } else {
                            Err(q.err("takes no values"))
                        }
                    };
                    match c.key.as_str() {
                        "slice" => {
                            needs_atlas()?;
                            d.frame.slice = Some(slice_at(&q, 0, size, scale)?);
                        }
                        "tone" => {
                            let t = tone_at(&q, 0)?;
                            d.frame
                                .slice
                                .as_mut()
                                .ok_or_else(|| {
                                    q.err("sets a tone, but the dressing has no slice before it")
                                })?
                                .tone = t;
                        }
                        "shadow" => d.frame.shadow = Some(tokens::shadow_at(&q, 0)?),
                        "glow" => d.frame.glow = Some(glow_at(&q, 0)?),
                        "glass" => d.frame.glass = Some(glass_at(&q, 0)?),
                        "corners" => d.frame.corners = bare()?,
                        "tile" => d.frame.tile = bare()?,
                        "shape" => {
                            let radius = v.radius.as_ref().ok_or_else(|| {
                                q.err("needs the `radius { … }` scale its corner names")
                            })?;
                            d.frame.shape = Some(tokens::shape_at(&q, 0, radius)?);
                        }
                        "press" => d.frame.press = Some(press_at(&q)?),
                        "hover" => d.frame.hover = Some(hover_at(&q)?),
                        k if DRAW_KEYS.contains(&k) => d.draw.apply(c, &atlas, &mut seen)?,
                        _ => {
                            return Err(q.err(&format!(
                                "is not a dressing setting: slice, tone, shadow, glow, glass, corners, tile, shape, press, hover, or (for the UI pass) {}",
                                DRAW_KEYS.join(", ")
                            )));
                        }
                    }
                }
            }
            k => return Err(format!("line {}: unknown style setting '{k}'", e.line)),
        }
    }
    if v2 {
        // The dressings of the twelve version 1 parts are what the widgets draw.
        let tile_parts =
            dressings
                .iter()
                .fold(0u32, |m, d| match StylePart::from_selector(&d.selector) {
                    Some(p) if d.frame.tile && p.class() == p => m | p.bit(),
                    _ => m,
                });
        if tile_parts != 0 && theme.art.tile.is_none() {
            return Err("a dressing says `tile`, but the style has no `tile` line".into());
        }
        if let Some(t) = theme.art.tile.as_mut() {
            t.parts = tile_parts;
        }
        for d in &dressings {
            let Some(part) = StylePart::from_selector(&d.selector) else {
                continue;
            };
            let a = &mut theme.art;
            let f = &d.frame;
            if f.slice.is_some() {
                a.slices[part.index()] = f.slice;
            }
            if part.class() == part {
                let i = part.index();
                a.shadows[i] = f.shadow.or(a.shadows[i]);
                a.glows[i] = f.glow.or(a.glows[i]);
                a.glass[i] = f.glass.or(a.glass[i]);
                if f.corners {
                    a.corner_parts |= part.bit();
                }
            }
            if part == StylePart::Button {
                a.press = f.press.or(a.press);
                a.hover = f.hover.or(a.hover);
            }
        }
        let need = |what: &str| {
            format!(
                "a version 2 style needs a `{what}` block (og style upgrade writes one from a version 1 style)"
            )
        };
        let radius = v
            .radius
            .ok_or_else(|| need("radius { xs sm md lg xl pill }"))?;
        let shape = v
            .shape
            .ok_or_else(|| need("shape { <element class> <corner> <radius> }"))?;
        let space = v
            .space
            .ok_or_else(|| need("space { xxs xs sm md lg xl xxl xl3 }"))?;
        let motion = v
            .motion
            .ok_or_else(|| need("motion { spring … ease … duration … }"))?;
        let mblock = P {
            e: body.get("motion").expect("a motion block"),
        };
        let motion = motion.finish(&mblock, &v.springs)?;
        let feedback = v
            .feedback
            .ok_or_else(|| need("sounds { <event> … haptic … }"))?;
        theme.tokens = StyleTokens {
            colors: v.colors.ok_or_else(|| need("colors { … }"))?,
            typography: v.typography.ok_or_else(|| need("typography { … }"))?,
            space,
            size: v
                .size
                .ok_or_else(|| need("size { xs sm md lg xl xxl touch }"))?,
            radius,
            stroke: v
                .stroke
                .ok_or_else(|| need("stroke { hairline thin regular thick focus }"))?,
            elevation: v
                .elevation
                .ok_or_else(|| need("elevation { raised floating overlay top }"))?,
            shape,
            motion,
            feedback,
        };
        // What the widgets read today, from the tokens.
        let t = &theme.tokens;
        theme.radius = Radius {
            small: t.shape(ElementClass::Slot).radius,
            control: t.shape(ElementClass::Button).radius,
            bar: t.shape(ElementClass::Bar).radius,
            panel: t.shape(ElementClass::Panel).radius,
        };
        let s = &t.space;
        theme.space = crate::Space {
            xxs: s[0],
            xs: s[1],
            sm: s[2],
            md: s[3],
            lg: s[4],
            xl: s[5],
            xxl: s[6],
            xl3: s[7],
        };
        theme.durations_ms = t.motion.durations;
        theme.art.motion.stagger_ms = t.motion.stagger_ms;
        for u in UiSound::ALL {
            file.sounds[u as usize] = file.feedback_sounds[u.event().index()].first().cloned();
        }
        has_radius = true;
    } else {
        for u in UiSound::ALL {
            file.feedback_sounds[u.event().index()] =
                file.sounds[u as usize].iter().cloned().collect();
        }
    }
    if file.atlas.is_some() && theme.art.slices.iter().all(Option::is_none) {
        return Err(if v2 {
            "the style has an atlas but dresses no part with a `slice`".into()
        } else {
            "the style has an atlas but no `slice` lines".into()
        });
    }
    let needs_soft = theme.art.shadows.iter().flatten().any(|s| s.blur > 0.0)
        || theme.art.glows.iter().any(Option::is_some);
    if needs_soft && theme.art.soft.is_none() {
        return Err(
            "a soft shadow or a glow needs a `soft` line (a blurred white rectangle in the atlas)"
                .into(),
        );
    }
    if !has_palette {
        return Err("a style needs a `palette { … }` block with every ThemeColor role".into());
    }
    if !has_radius {
        return Err("a style needs a `radius { small control bar panel }` block".into());
    }
    if v2 {
        file.dressings = dressings;
    } else {
        theme.tokens = StyleTokens::derive(&theme, &icon_names);
        // A version 1 style's sounds are the one-sound sets of its events.
        for u in UiSound::ALL {
            if file.sounds[u as usize].is_some() {
                let f = &mut theme.tokens.feedback[u.event().index()];
                f.count = 1;
            }
        }
        file.dressings = v1_dressings(&theme.art);
    }
    // The motion code reads the style's motion tokens (docs/ui.md §7.7).
    theme.motion = crate::motion::MotionTokens::from_style(&theme.tokens);
    file.theme = theme;
    Ok(file)
}

/// `og style upgrade`: a version 1 style written as version 2. Every
/// value version 1 set keeps its value (the palette, the ramp, the fonts,
/// the art and its motion, so the style draws the same); the radii become
/// the shape language's corners, the durations motion tokens, the per-part
/// lines dressings of [`StylePart::selector`], the sounds one-sound sets;
/// the new tokens are [`StyleTokens::derive`]d from the palette and scheme.
pub fn upgrade_style(text: &str) -> Result<String, String> {
    let old = parse_style(text)?;
    if old.version != 1 {
        return Err(format!(
            "the style is version {} already: nothing to upgrade",
            old.version
        ));
    }
    let mut doc = Document::parse(text).map_err(|e| e.to_string())?;
    let body = std::mem::take(&mut doc.body);
    let icons: Vec<String> = old.icons.as_ref().map(|i| i.3.clone()).unwrap_or_default();
    let mut sounds: [Vec<String>; SoundEvent::COUNT] = Default::default();
    for e in body.get_all("sound") {
        let p = P { e };
        if let Some(u) = UiSound::from_name(p.word(0)?) {
            sounds[u.event().index()] = vec![p.string(1)?.to_string()];
        }
    }
    let (token_blocks, motion_tokens, sounds_block) = old.theme.tokens.entries(&icons, &sounds);
    let mut out = og_core::ogt::Block::new();
    let copy = |out: &mut og_core::ogt::Block, key: &str| {
        for e in body.get_all(key) {
            out.push(e.clone());
        }
    };
    for k in ["name", "scheme", "atlas", "scale", "palette"] {
        copy(&mut out, k);
    }
    out.entries.extend(token_blocks);
    for k in [
        "ramp", "fonts", "icons", "soft", "corner", "divider", "flourish",
    ] {
        copy(&mut out, k);
    }
    for e in body.get_all("tile") {
        let mut e = e.clone();
        let numbers = e.values[4.min(e.values.len())..]
            .iter()
            .take_while(|v| v.as_number().is_some())
            .count();
        e.values.truncate(4 + numbers);
        out.push(e);
    }
    for k in ["textShadow", "textGlow", "textWeight"] {
        copy(&mut out, k);
    }
    let mut motion: Vec<Entry> = Vec::new();
    if let Some(e) = body.get("motion")
        && let Some(b) = &e.block
    {
        for c in &b.entries {
            match c.key.as_str() {
                "stagger" => {}
                "hoverSpring" | "pressSpring" => {
                    let q = P { e: c };
                    let s = tokens::SpringToken::from_spring(crate::theme::Spring {
                        damping: q.num(0)?.clamp(0.05, 4.0),
                        response_ms: q.num(1)?.clamp(16.0, 4000.0),
                    });
                    let mut n = Entry::new(c.key.clone());
                    n.values = vec![
                        tokens::num_value(s.duration_ms),
                        tokens::num_value(s.bounce),
                    ];
                    motion.push(n);
                }
                _ => motion.push(c.clone()),
            }
        }
    }
    motion.extend(motion_tokens);
    out.push(Entry::new("motion").with_block(og_core::ogt::Block { entries: motion }));
    out.push(sounds_block);
    for d in &old.dressings {
        let part = StylePart::from_selector(&d.selector).expect("a version 1 part");
        let mut entries: Vec<Entry> = Vec::new();
        let rest = |e: &Entry| {
            let mut n = e.clone();
            n.values.remove(0);
            n.line = 0;
            n
        };
        let of = |e: &Entry, class: bool| {
            e.values
                .first()
                .and_then(Value::as_word)
                .and_then(StylePart::from_name)
                .is_some_and(|p| if class { p.class() == part } else { p == part })
        };
        for k in ["slice", "tone"] {
            entries.extend(body.get_all(k).filter(|e| of(e, false)).map(rest));
        }
        if part.class() == part {
            for k in ["shadow", "glow", "glass"] {
                entries.extend(body.get_all(k).filter(|e| of(e, true)).map(rest));
            }
            if d.frame.corners {
                entries.push(Entry::new("corners"));
            }
            if d.frame.tile {
                entries.push(Entry::new("tile"));
            }
        }
        if part == StylePart::Button {
            copy_into(&mut entries, &body, "press");
            copy_into(&mut entries, &body, "hover");
        }
        if !entries.is_empty() {
            out.push(
                Entry::new("dress")
                    .with(Value::String(d.selector.to_string()))
                    .with_block(og_core::ogt::Block { entries }),
            );
        }
    }
    doc.version = 2;
    doc.body = out;
    let text2 = doc.to_text().map_err(|e| e.to_string())?.replacen(
        "---\n",
        "---\n# Upgraded to version 2 by og style upgrade: its version 1 values kept, the new tokens derived from its palette and scheme (docs/ui.md §12).\n",
        1,
    );
    let new =
        parse_style(&text2).map_err(|e| format!("the upgraded style does not read back: {e}"))?;
    let (a, b) = (&old.theme, &new.theme);
    if (
        a.palette,
        a.space,
        a.radius,
        a.type_ramp,
        a.durations_ms,
        a.art,
    ) != (
        b.palette,
        b.space,
        b.radius,
        b.type_ramp,
        b.durations_ms,
        b.art,
    ) || old.sounds != new.sounds
        || old.fonts != new.fonts
    {
        return Err(
            "the upgrade would change how the style draws; this is a bug in og style upgrade"
                .into(),
        );
    }
    Ok(text2)
}

fn copy_into(out: &mut Vec<Entry>, body: &og_core::ogt::Block, key: &str) {
    for e in body.get_all(key) {
        let mut n = e.clone();
        n.line = 0;
        out.push(n);
    }
}
