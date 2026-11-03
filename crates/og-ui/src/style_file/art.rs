use super::*;

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

    pub(super) fn bit(self) -> u32 {
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

    /// The part a widget drawing this version 1 class is ([`Self::selector`]
    /// as a [`crate::dress::PartRef`], with its element class and level).
    pub fn part_ref(self) -> crate::dress::PartRef {
        use crate::dress::PartRef;
        use crate::tokens::{ElementClass as C, Level};
        let p = match self {
            StylePart::Panel => PartRef::new("Panel", "frame", C::Panel),
            StylePart::Card => PartRef::new("Card", "frame", C::Card).at_level(Level::Raised),
            StylePart::Button => PartRef::new("Button", "base", C::Button),
            StylePart::ButtonHover => PartRef::new("Button", "base", C::Button).flag("hover"),
            StylePart::ButtonPressed => PartRef::new("Button", "base", C::Button).flag("pressed"),
            StylePart::ButtonDisabled => PartRef::new("Button", "base", C::Button).flag("disabled"),
            StylePart::Field => PartRef::new("Field", "frame", C::Field),
            StylePart::Slot => PartRef::new("Slot", "frame", C::Slot),
            StylePart::Tooltip => {
                PartRef::new("Tooltip", "frame", C::Tooltip).at_level(Level::Floating)
            }
            StylePart::Hud => PartRef::new("Hud", "frame", C::Hud),
            StylePart::Track => PartRef::new("Meter", "track", C::Bar),
            StylePart::Fill => PartRef::new("Meter", "fill", C::Bar),
        };
        debug_assert_eq!(StylePart::from_part(&p), Some(self));
        p
    }

    /// The version 1 class a part draws its atlas art with: a part of one
    /// of the twelve selectors (its variants and states beyond a button's
    /// own taken off), else none.
    pub fn from_part(p: &crate::dress::PartRef) -> Option<StylePart> {
        let state = |s| p.has(s);
        Some(match (p.component, p.part) {
            ("Panel", "frame") => StylePart::Panel,
            ("Card", "frame") => StylePart::Card,
            ("Button", "base") if state("disabled") => StylePart::ButtonDisabled,
            ("Button", "base") if state("pressed") => StylePart::ButtonPressed,
            ("Button", "base") if state("hover") => StylePart::ButtonHover,
            ("Button", "base") => StylePart::Button,
            ("Field", "frame") => StylePart::Field,
            ("Slot", "frame") => StylePart::Slot,
            ("Tooltip", "frame") => StylePart::Tooltip,
            ("Hud", "frame") => StylePart::Hud,
            ("Meter", "track") => StylePart::Track,
            ("Meter", "fill") => StylePart::Fill,
            _ => return None,
        })
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
    pub(super) fn parse(w: &str) -> Option<Ink> {
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
    /// Moves it outwards (corners, crests, edges) or apart (flourishes), CSS px.
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
    /// How far a `glitch` entrance or toast tears (`enter glitch 0.8`; 0
    /// when the style names no glitch, which then tears at 1).
    pub glitch: f32,
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
    /// The same atlas at 2× and 3× its texels (`atlas … 2 3`): a frame
    /// draws from the one nearest at or above its device pixels per CSS
    /// px ([`Art::atlas_for`]), so painted art stays sharp on a phone and
    /// a 4K screen without changing a region or an inset.
    pub atlas_dense: [Option<TextureHandle>; 2],
    pub slices: [Option<Slice>; PARTS],
    /// A blurred white rectangle: soft shadows and glows.
    pub soft: Option<Slice>,
    pub corner: Option<Ornament>,
    /// The parts that take corner ornaments (bits).
    pub corner_parts: u32,
    pub divider: Option<Ornament>,
    pub flourish: Option<Ornament>,
    /// Drawn at a part's top centre, as authored (a dressing's `crest`).
    pub crest: Option<Ornament>,
    /// Drawn at the middle of a part's left edge as authored and of its
    /// right edge mirrored (a dressing's `edges`).
    pub edge: Option<Ornament>,
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
    /// `blend srgb`: the UI blends in sRGB space, as CSS does
    /// (og_render::Overlay::srgb_blend); `blend linear` (the default) in
    /// linear light.
    pub srgb_blend: bool,
    /// A version 1 style: a game shipped with it keeps the screens it
    /// drew (docs/ui.md §12.1): its dialogs' titles the ramp's xl in the
    /// body face, its ramp untracked, its grid's selection an underline,
    /// and the templates (`UiTheme.styleVersion`) their version 1 layouts.
    /// A version 2 style says so with `screens 1` (what og style upgrade
    /// writes).
    pub legacy: bool,
    /// A version 1 file itself (not upgraded): its sliders' knobs and
    /// switches take the control radius, as version 1 drew them. An
    /// upgraded file's are round, as games shipped them once upgraded.
    pub v1_controls: bool,
}

impl Art {
    /// The atlas to draw at `dpr` device pixels per CSS px: the nearest
    /// density at or above it (1 up to 1×, 2 up to 2×, 3 beyond), else the
    /// densest there is below it.
    pub fn atlas_for(&self, dpr: f32) -> Option<TextureHandle> {
        let base = self.atlas?;
        let [x2, x3] = self.atlas_dense;
        Some(if dpr <= 1.0 {
            base
        } else if dpr <= 2.0 {
            x2.or(x3).unwrap_or(base)
        } else {
            x3.or(x2).unwrap_or(base)
        })
    }

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
