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
//! The grammar is [`parse_style`]'s; the runtime loads a style's atlas,
//! fonts, icons and sounds through the bundle and hands the handles to
//! [`StyleFile::theme`].

use crate::motion::TransitionKind;
use crate::style::{Durations, Palette, Radius, Scheme, Srgba, Theme, TypeRamp, TypeStep};
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
    /// The theme with the style's palette, radii, ramp and durations; its
    /// art has no atlas or sounds yet (the loader adds the handles).
    pub theme: Theme,
    /// The atlas texture's asset id.
    pub atlas: Option<String>,
    /// Font asset ids: body, display, numeric (the mono role), fallback
    /// (characters the others lack, such as CJK).
    pub fonts: [Option<String>; 4],
    /// The icon texture's asset id, its grid and names.
    pub icons: Option<(String, u32, u32, Vec<String>)>,
    /// Sound asset ids by [`UiSound`].
    pub sounds: [Option<String>; 6],
}

impl StyleFile {
    /// Every asset id the file names (the cook checks they exist).
    pub fn references(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        out.extend(self.atlas.clone());
        out.extend(self.fonts.iter().flatten().cloned());
        out.extend(self.icons.as_ref().map(|i| i.0.clone()));
        out.extend(self.sounds.iter().flatten().cloned());
        out
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
    }

    /// The theme with the loaded atlas and sound handles.
    pub fn theme(&self, atlas: Option<TextureHandle>, sounds: [u64; 6]) -> Theme {
        let mut t = self.theme;
        t.art.atlas = atlas;
        t.art.sounds = sounds;
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

struct P<'a> {
    e: &'a Entry,
}

impl<'a> P<'a> {
    fn err(&self, what: &str) -> String {
        format!("line {}: `{}` {what}", self.e.line, self.e.key)
    }
    fn num(&self, i: usize) -> Result<f32, String> {
        self.e
            .values
            .get(i)
            .and_then(Value::as_number)
            .map(|n| n.as_f32())
            .filter(|v| v.is_finite())
            .ok_or_else(|| self.err(&format!("takes a number at position {}", i + 1)))
    }
    fn num_or(&self, i: usize, d: f32) -> Result<f32, String> {
        if self.e.values.len() > i {
            self.num(i)
        } else {
            Ok(d)
        }
    }
    fn word(&self, i: usize) -> Result<&'a str, String> {
        self.e
            .values
            .get(i)
            .and_then(Value::as_word)
            .ok_or_else(|| self.err(&format!("takes a word at position {}", i + 1)))
    }
    fn string(&self, i: usize) -> Result<&'a str, String> {
        self.e
            .values
            .get(i)
            .and_then(Value::as_str)
            .ok_or_else(|| self.err(&format!("takes a quoted string at position {}", i + 1)))
    }
    fn color(&self, i: usize) -> Result<Srgba, String> {
        let s = self.string(i)?;
        parse_color(s)
            .ok_or_else(|| self.err(&format!("takes \"#rrggbb\" or \"#rrggbbaa\", not \"{s}\"")))
    }
    fn block(&self) -> Result<&'a og_core::ogt::Block, String> {
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

/// Parses a UI style:
///
/// ```text
/// type style
/// version 1
/// ---
/// name "fantasy"
/// scheme dark                                # dark (the default) or light: what the game reads as UiTheme.scheme
/// atlas "atlas.png" 256 256                  # its asset id and size in texels
/// palette {                                  # every ThemeColor role, one per line (all required)
///   backdrop "#1a120b"
///   panel "#f2e4c4"
///   ink "#2b1d12"
///   …
/// }
/// radius {                                   # all four required
///   small 4
///   control 4
///   bar 4
///   panel 6
/// }
/// ramp {
///   displayLg 44 48                          # TypeStepName: size, line height [, CJK line height, tracking em]
/// }
/// durations {
///   base 260
/// }
/// fonts {
///   display "styles/fantasy/Cinzel-Bold.ttf"
///   body "styles/fantasy/Newsreader.ttf"
///   numeric "styles/fantasy/Newsreader.ttf"   # the mono role
///   fallback "fonts/NotoSansSC-Regular-subset.ttf"
/// }
/// icons "styles/fantasy/icons.png" 8 4 {
///   names "heart" "star"
/// }
/// slice panel 0 0 64 64 20 [20 20 20] [tint|authored] [stretch|repeat|round]
///                                             # region, corner insets in texels; how edges fill
/// soft 192 0 32 32 12                         # a blurred white rect: soft shadows and glows
/// scale 1                                     # CSS px per atlas texel for slice corners
/// corner 64 0 24 24 24 24 primary [offset]   # region, size on screen, ink (authored, primary, ink, muted, line, focus)
/// corners panel card
/// divider 64 32 96 12 96 12 primary
/// flourish 64 48 48 16 48 16 primary [gap]
/// tile 128 64 64 64 64 64 0.25 [inset] panel card
/// shadow panel 0 6 14 0 "#00000080"           # part dx dy blur spread colour
/// glow button 6 primary 0.5                   # part size ink alpha
/// textShadow display 0 2 0 "#000000a0"        # role dx dy blur colour
/// textGlow display 3 primary 0.6
/// textWeight body 0.5                         # role, thickening in device px (a pixel font's thin strokes)
/// press 1.04 0.92 2                           # sx sy dy
/// hover -2 1.02                               # dy scale (both move on springs: motion below)
/// motion {
///   enter pop                                 # dialogs and screen panels
///   stagger 40                                # ms between their controls
///   toast slideLeft
///   progress 400                              # ms progress bars ease over
///   shake 6                                   # CSS px a refused press shakes
///   pop 0.12                                  # a landing count-up's bump
///   hoverSpring 1 350                         # the hover lift's spring: damping ratio, response ms
///   pressSpring 0.55 120                      # the press squash's
/// }
/// sound press "audio/ui/press.wav"
/// ```
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
    if doc.version != 1 {
        return Err(format!(
            "style version {} (this engine reads 1)",
            doc.version
        ));
    }
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
        theme,
        atlas: None,
        fonts: Default::default(),
        icons: None,
        sounds: Default::default(),
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
    for e in &body.entries {
        let p = P { e };
        let needs_atlas = || {
            if file.atlas.is_none() {
                Err(p.err("needs an `atlas` line first"))
            } else {
                Ok(())
            }
        };
        match e.key.as_str() {
            "scheme" | "atlas" | "scale" => {}
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
                        _ => {
                            return Err(q.err("is a font role: body, display, numeric or fallback"));
                        }
                    };
                    file.fonts[i] = Some(q.string(0)?.to_string());
                }
            }
            "icons" => {
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
            "slice" | "soft" => {
                needs_atlas()?;
                let (part, at) = if e.key == "soft" {
                    (None, 0)
                } else {
                    let w = p.word(0)?;
                    (
                        Some(
                            StylePart::from_name(w)
                                .ok_or_else(|| p.err(&format!("names an unknown part '{w}'")))?,
                        ),
                        1,
                    )
                };
                let (uv, px) = region(&p, at, size)?;
                let numbers = e.values[at + 4..]
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
                        return Err(p.err(
                            "takes one corner inset or four (left top right bottom), in texels",
                        ));
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
                    return Err(
                        p.err("tiles its edges but leaves no middle between the corners to tile")
                    );
                }
                let s = Slice {
                    uv,
                    border_uv: [b[0] / size.0, b[1] / size.1, b[2] / size.0, b[3] / size.1],
                    border: [b[0] * scale, b[1] * scale, b[2] * scale, b[3] * scale],
                    tint,
                    fill,
                    middle,
                };
                match part {
                    Some(part) => theme.art.slices[part.index()] = Some(s),
                    None => theme.art.soft = Some(s),
                }
            }
            "corner" | "divider" | "flourish" => {
                needs_atlas()?;
                let (uv, _) = region(&p, 0, size)?;
                let ink = match e.values.get(6).and_then(Value::as_word) {
                    None => Ink::Authored,
                    Some(w) => Ink::parse(w).ok_or_else(|| {
                        p.err(&format!(
                            "ink '{w}' is authored, accent, fg, muted, border or focus"
                        ))
                    })?,
                };
                let o = Ornament {
                    uv,
                    w: p.num(4)?,
                    h: p.num(5)?,
                    ink,
                    offset: p.num_or(7, 0.0)?,
                };
                match e.key.as_str() {
                    "corner" => theme.art.corner = Some(o),
                    "divider" => theme.art.divider = Some(o),
                    _ => theme.art.flourish = Some(o),
                }
            }
            "corners" => theme.art.corner_parts = parts_mask(&p, 0)?,
            "tile" => {
                needs_atlas()?;
                let (uv, _) = region(&p, 0, size)?;
                let numbers = e.values[4..]
                    .iter()
                    .take_while(|v| v.as_number().is_some())
                    .count();
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
                let w = p.word(0)?;
                let part = StylePart::from_name(w)
                    .ok_or_else(|| p.err(&format!("names an unknown part '{w}'")))?;
                let s = Shadow {
                    dx: p.num(1)?,
                    dy: p.num(2)?,
                    blur: p.num(3)?.max(0.0),
                    spread: p.num(4)?,
                    color: p.color(5)?,
                };
                theme.art.shadows[part.class().index()] = Some(s);
            }
            "glow" => {
                let w = p.word(0)?;
                let part = StylePart::from_name(w)
                    .ok_or_else(|| p.err(&format!("names an unknown part '{w}'")))?;
                let iw = p.word(2)?;
                theme.art.glows[part.class().index()] = Some(Glow {
                    size: p.num(1)?.max(0.0),
                    ink: Ink::parse(iw).ok_or_else(|| {
                        p.err(&format!("ink '{iw}' is accent, fg, muted, border or focus"))
                    })?,
                    alpha: p.num(3)?.clamp(0.0, 1.0),
                });
            }
            "glass" => {
                let w = p.word(0)?;
                let part = StylePart::from_name(w)
                    .ok_or_else(|| p.err(&format!("names an unknown part '{w}'")))?;
                theme.art.glass[part.class().index()] = Some(Glass {
                    blur: p.num(1)?.max(0.0),
                    fill: p.num(2)?.clamp(0.0, 1.0),
                    tint: p.num_or(3, 0.25)?.clamp(0.0, 1.0),
                });
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
                let iw = p.word(2)?;
                theme.art.text_glows[i] = Some(Glow {
                    size: p.num(1)?.max(0.0),
                    ink: Ink::parse(iw).ok_or_else(|| {
                        p.err(&format!("ink '{iw}' is accent, fg, muted, border or focus"))
                    })?,
                    alpha: p.num(3)?.clamp(0.0, 1.0),
                });
            }
            "textWeight" => {
                let i = role_index(&p, p.word(0)?)?;
                let w = p.num(1)?;
                if !(0.0..=2.0).contains(&w) {
                    return Err(p.err("takes the thickening in device pixels, 0 to 2"));
                }
                theme.art.text_weights[i] = w;
            }
            "press" => {
                theme.art.press = Some(Press {
                    sx: p.num(0)?,
                    sy: p.num(1)?,
                    dy: p.num_or(2, 0.0)?,
                })
            }
            "hover" => {
                theme.art.hover = Some(Hover {
                    dy: p.num(0)?,
                    scale: p.num_or(1, 1.0)?,
                })
            }
            "motion" => {
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
                        "stagger" => m.stagger_ms = q.num(0)?.max(0.0),
                        "progress" => m.progress_ms = q.num(0)?.max(0.0),
                        "shake" => m.shake = q.num(0)?.max(0.0),
                        "pop" => m.pop = q.num(0)?.clamp(0.0, 1.0),
                        "hoverSpring" | "pressSpring" => {
                            let spring = Some(crate::theme::Spring {
                                damping: q.num(0)?.clamp(0.05, 4.0),
                                response_ms: q.num(1)?.clamp(16.0, 4000.0),
                            });
                            if c.key == "hoverSpring" {
                                m.hover_spring = spring;
                            } else {
                                m.press_spring = spring;
                            }
                        }
                        _ => return Err(q.err(
                            "is a motion setting: enter, stagger, toast, progress, shake, pop, hoverSpring or pressSpring",
                        )),
                    }
                }
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
            k => return Err(format!("line {}: unknown style setting '{k}'", e.line)),
        }
    }
    if file.atlas.is_some() && theme.art.slices.iter().all(Option::is_none) {
        return Err("the style has an atlas but no `slice` lines".into());
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
    file.theme = theme;
    Ok(file)
}
