use super::*;

/// The widest the HUD band is by default, over its height: 21:9, so a
/// 32:9 surface's HUD corners sit where a 21:9 one's do, centred
/// ([`Ui::hud_area`]).
pub const HUD_MAX_ASPECT: f32 = 21.0 / 9.0;

/// The play surface the UI lays out in, in CSS pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub width: f32,
    pub height: f32,
    /// Device pixels per CSS pixel; rectangles snap to device pixels.
    pub dpr: f32,
    /// The device's unsafe margins (notch, home indicator); the root pads by them.
    pub safe_area: Insets,
    /// A touch screen or other coarse pointer (raises controls to touch size).
    pub coarse_pointer: bool,
    /// The page prefers reduced motion (`prefers-reduced-motion: reduce`):
    /// the theme's `reduced_motion` is on whatever the game configured.
    pub reduced_motion: bool,
    /// The page prefers more contrast (`prefers-contrast: more`): the
    /// theme is its high-contrast variant whatever the game configured.
    pub high_contrast: bool,
    /// The page prefers less transparency (`prefers-reduced-transparency:
    /// reduce`): the theme's `reduced_transparency` is on whatever the game
    /// configured (glass parts draw as opaque surfaces, scrims and
    /// translucent fills flat: `Theme::to_reduced_transparency`).
    pub reduced_transparency: bool,
    /// Text controls take macOS's editing keys: Command for the shortcuts
    /// (A, C, X, Z) and with ←/→ for the line's ends (↑/↓ the text's),
    /// Option with ←/→, Backspace and Delete by words, Control+A and E the
    /// line's ends (emacs). Otherwise Control takes the shortcuts and moves
    /// by words (Alt too), Home and End go to the line's ends and Command
    /// only copies, cuts, selects all and undoes. The web shell sets it on
    /// Apple platforms (`navigator.platform`); native hosts leave it off, so
    /// a script edits the same on every OS.
    pub mac_keys: bool,
    /// Where the host's own control covers the surface (the web shell's
    /// round Back, top left of a published page), in CSS pixels; empty is
    /// none. Anchored groups keep out of it: a group it would overlap
    /// moves along the edge it sits on, past it.
    pub covered: Rect,
}

impl Default for Viewport {
    fn default() -> Viewport {
        Viewport {
            width: 1280.0,
            height: 720.0,
            dpr: 1.0,
            safe_area: Insets::default(),
            coarse_pointer: false,
            reduced_motion: false,
            high_contrast: false,
            reduced_transparency: false,
            mac_keys: false,
            covered: Rect::default(),
        }
    }
}

impl Viewport {
    pub fn new(width: f32, height: f32) -> Viewport {
        Viewport {
            width,
            height,
            ..Viewport::default()
        }
    }

    /// This surface in og-ui units of `unit` CSS px (docs/ui.md §15.2):
    /// its size, safe area and covered region over `unit`, and `unit`
    /// times as many device pixels to each, so what is laid out on it and
    /// drawn `unit` times larger lands on the same device pixels.
    pub fn zoomed(&self, unit: f32) -> Viewport {
        if unit == 1.0 {
            return *self;
        }
        let k = 1.0 / unit;
        let s = self.safe_area;
        Viewport {
            width: self.width * k,
            height: self.height * k,
            dpr: self.dpr * unit,
            safe_area: Insets {
                left: s.left * k,
                top: s.top * k,
                right: s.right * k,
                bottom: s.bottom * k,
            },
            covered: Rect::new(
                self.covered.x * k,
                self.covered.y * k,
                self.covered.width * k,
                self.covered.height * k,
            ),
            ..*self
        }
    }

    /// The size class of this surface.
    pub fn breakpoint(&self) -> Breakpoint {
        Breakpoint::of(self.width, self.height)
    }

    /// The safe area, in CSS pixels.
    pub fn safe_rect(&self) -> Rect {
        Rect::new(
            self.safe_area.left,
            self.safe_area.top,
            (self.width - self.safe_area.horizontal()).max(0.0),
            (self.height - self.safe_area.vertical()).max(0.0),
        )
    }

    /// The HUD band ([`Ui::hud_area`]): the safe area at most `max_aspect`
    /// times its height wide, centred; the whole safe area where it is no
    /// wider, or for a `max_aspect` of 0 (or not finite).
    pub fn hud_rect(&self, max_aspect: f32) -> Rect {
        let safe = self.safe_rect();
        if !(max_aspect.is_finite() && max_aspect > 0.0) {
            return safe;
        }
        let widest = max_aspect * safe.height;
        if safe.width <= widest {
            return safe;
        }
        Rect::new(
            safe.x + (safe.width - widest) / 2.0,
            safe.y,
            widest,
            safe.height,
        )
    }

    pub(crate) fn dpr(&self) -> f32 {
        if self.dpr.is_finite() && self.dpr > 0.0 {
            self.dpr
        } else {
            1.0
        }
    }
}

/// How to create a [`Ui`]. The zero value is the dark theme at 60 ticks per second.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiDesc {
    pub theme: Theme,
    /// The world's tick rate, for delays given in milliseconds (0: 60).
    pub ticks_per_second: u32,
    /// The fonts; without one, text is measured at 0.5 em per character and not drawn.
    pub fonts: Fonts,
}

/// The fonts of each role; `display` and `mono` (the numeric role) fall
/// back to `body`, `rounded` to `display`, then `body`. A text the role's
/// font cannot draw whole takes the first font of `chain` (the locale's,
/// og.i18n) that can; a text mixing what the role's font draws with what it
/// does not is laid out per run, each run from the first of the role's,
/// `chain`, `fallbacks` (the game's) and `fallback` (a style's Latin display
/// face and Chinese text) that has it ([`Fonts::for_text`]).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Fonts {
    pub body: UiFont,
    pub display: Option<UiFont>,
    pub mono: Option<UiFont>,
    /// Levels and scores (`FontRole::Rounded`).
    pub rounded: Option<UiFont>,
    pub fallback: Option<UiFont>,
    /// The style's Chinese companions: `cjk` beside body and the numeric
    /// role, `cjk_display` beside display and rounded (`cjk` when unset).
    /// A text of a role with a companion takes its Latin from the role's
    /// face and its Chinese from the companion, ahead of the locale's and
    /// the game's fonts, so Chinese matches the style's voice instead of
    /// a neutral fallback face ([`Fonts::for_text`]).
    pub cjk: Option<UiFont>,
    pub cjk_display: Option<UiFont>,
    /// The active locale's fonts, first first.
    pub chain: Vec<UiFont>,
    /// The game's fonts for other scripts (`UiDesc.fallbackFonts`), tried
    /// after the locale's: a text the role's font cannot draw whole takes
    /// the first that can.
    pub fallbacks: Vec<UiFont>,
}

impl Fonts {
    pub fn get(&self, role: FontRole) -> &UiFont {
        match role {
            FontRole::Body => &self.body,
            FontRole::Display => self.display.as_ref().unwrap_or(&self.body),
            FontRole::Mono => self.mono.as_ref().unwrap_or(&self.body),
            FontRole::Rounded => self
                .rounded
                .as_ref()
                .or(self.display.as_ref())
                .unwrap_or(&self.body),
        }
    }

    /// The font a text of `role` is measured and drawn with: the role's when
    /// its atlas has every character; else the first of `chain` (the
    /// locale's faces, which set its strings whole) whose atlas does; else,
    /// for a text the role's font draws in part (a Latin name beside CJK or
    /// Arabic), the role's font mixed with the others: each run from the
    /// first of the role's, `chain`, `fallbacks` and `fallback` that has it
    /// ([`FontMetrics::is_mixed`]; once [`Fonts::link`] made it, as
    /// [`crate::Ui`] does); else the first of `fallbacks` whose atlas has
    /// every character; else the style's `fallback` when it has a character
    /// the role's lacks; else the first font, in the same order, that can
    /// draw it with the characters its file renders at run time (`font {
    /// runtime true }`); else (no one font draws it all) the mixed font,
    /// so only the characters no font has are missing.
    pub fn for_text(&self, role: FontRole, text: &str) -> &UiFont {
        let f = self.get(role);
        let cooked = |font: &UiFont| {
            text.chars()
                .all(|c| c.is_whitespace() || font.metrics.in_atlas(c))
        };
        if cooked(f) {
            return f;
        }
        // The style's companion draws what the role's face lacks.
        if let Some(c) = self.companion(role)
            && let Some(m) = f.metrics.mixed.as_deref()
            && text
                .chars()
                .any(|ch| !ch.is_whitespace() && !f.metrics.in_atlas(ch) && c.metrics.has(ch))
        {
            return m;
        }
        if let Some(c) = self.chain.iter().find(|c| cooked(c)) {
            return c;
        }
        let mixed = f.metrics.mixed.as_deref();
        if let Some(m) = mixed
            && text
                .chars()
                .any(|c| !c.is_whitespace() && f.metrics.in_atlas(c))
        {
            return m;
        }
        let others = || self.chain.iter().chain(&self.fallbacks);
        if let Some(c) = self.fallbacks.iter().find(|c| cooked(c)) {
            return c;
        }
        if let Some(fb) = &self.fallback
            && text
                .chars()
                .any(|c| !c.is_whitespace() && !f.metrics.in_atlas(c) && fb.metrics.has(c))
        {
            return fb;
        }
        let drawable = |font: &UiFont| {
            text.chars()
                .all(|c| c.is_whitespace() || font.metrics.has(c))
        };
        if drawable(f) {
            return f;
        }
        if let Some(c) = others().find(|c| drawable(c)) {
            return c;
        }
        // No one font draws it all: each run from the font that has it, so
        // only what no font has is missing.
        mixed.unwrap_or(f)
    }

    /// Makes each role's mixed font ([`Fonts::for_text`]): the role's font
    /// with `chain`, `fallbacks` and `fallback` after it, for a text that
    /// mixes scripts. [`crate::Ui`] links the fonts it is given; call it
    /// again after changing them by hand.
    pub fn link(&mut self) {
        let others: Vec<UiFont> = self
            .chain
            .iter()
            .chain(&self.fallbacks)
            .chain(self.fallback.as_ref())
            .map(UiFont::alone)
            .collect();
        let linked = |companion: Option<&UiFont>| -> Option<std::sync::Arc<[UiFont]>> {
            let all: Vec<UiFont> = companion
                .map(UiFont::alone)
                .into_iter()
                .chain(others.iter().cloned())
                .collect();
            (!all.is_empty()).then(|| all.into())
        };
        let text_runs = linked(self.companion(FontRole::Body));
        let display_runs = linked(self.companion(FontRole::Display));
        let (text_prefer, display_prefer) = (
            usize::from(self.companion(FontRole::Body).is_some()),
            usize::from(self.companion(FontRole::Display).is_some()),
        );
        let roles = [
            (Some(&mut self.body), &text_runs, text_prefer),
            (self.display.as_mut(), &display_runs, display_prefer),
            (self.mono.as_mut(), &text_runs, text_prefer),
            (self.rounded.as_mut(), &display_runs, display_prefer),
        ];
        for (f, runs, prefer) in roles {
            let Some(f) = f else { continue };
            f.metrics.mixed = runs.as_ref().map(|o| {
                let mut m = f.alone();
                m.metrics.runs = Some(o.clone());
                m.metrics.prefer = prefer;
                std::sync::Arc::new(m)
            });
        }
    }

    /// The style's Chinese companion of `role` ([`Fonts::cjk`]).
    pub fn companion(&self, role: FontRole) -> Option<&UiFont> {
        match role {
            FontRole::Body | FontRole::Mono => self.cjk.as_ref(),
            FontRole::Display | FontRole::Rounded => {
                self.cjk_display.as_ref().or(self.cjk.as_ref())
            }
        }
    }
}

impl Metrics for Fonts {
    fn metrics_for(&self, role: FontRole, text: &str) -> &FontMetrics {
        &self.for_text(role, text).metrics
    }
}

/// The icon atlas `Ui::icon` draws from: a grid of equal cells of a
/// texture, or an icon set cooked like a font (`font`: an `.ogt` of
/// `type icons`, og-cook's `cook::icons`), whose icons are distance-field
/// glyphs drawn crisp at any size and tinted by their ink.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct IconAtlas {
    pub texture: Option<TextureHandle>,
    pub columns: u32,
    pub rows: u32,
    /// Optional names, one per cell in reading order, for `icon_named`;
    /// an icon set's names, glyph U+E000 + i being icon i.
    pub names: Vec<String>,
    /// An icon set: its MTSDF font (`texture` is then unused).
    pub font: Option<crate::text::UiFont>,
}

/// The code point of an icon set's first icon (og-cook's `ICON_PUA`).
pub const ICON_PUA: u32 = 0xE000;

impl IconAtlas {
    /// The UV rectangle of cell `index`.
    pub fn uv(&self, index: u32) -> [f32; 4] {
        let (c, r) = (self.columns.max(1), self.rows.max(1));
        let (x, y) = ((index % c) as f32, ((index / c) % r) as f32);
        let (w, h) = (1.0 / c as f32, 1.0 / r as f32);
        [x * w, y * h, (x + 1.0) * w, (y + 1.0) * h]
    }

    pub fn index_of(&self, name: &str) -> Option<u32> {
        self.names.iter().position(|n| n == name).map(|i| i as u32)
    }

    /// Whether it draws anything (a texture, or an icon set's font).
    pub fn is_drawn(&self) -> bool {
        self.texture.is_some() || self.font.as_ref().is_some_and(|f| f.handle.is_some())
    }

    /// An icon set's glyph for icon `index`: its font and character.
    pub fn glyph(&self, index: u32) -> Option<(og_render::FontHandle, char)> {
        let f = self.font.as_ref()?.handle?;
        Some((f, char::from_u32(ICON_PUA + index)?))
    }
}

/// One laid-out frame: what the probe reads and what the renderer draws.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiFrame {
    pub viewport: Viewport,
    /// Every visible element in draw order with its (clipped) bounds and text.
    pub elements: Vec<UiElement>,
    /// Draw commands with their painter levels.
    pub items: Vec<DrawItem>,
    /// Problems in this frame's tree (duplicate ids, unclosed elements), each naming the fix.
    pub warnings: Vec<String>,
    /// The path of the control each pad's focus is on (pad 0: the shared
    /// focus), or empty.
    pub focus_paths: [String; 4],
    /// The accessibility tree (`crate::a11y`).
    pub a11y: crate::a11y::A11yTree,
    /// The focused text control, for the host: where an input method's
    /// window goes, and the selection a copy takes (`None`: no text focus).
    pub text_input: Option<TextTarget>,
    /// Overlaps no element declared (docs/ui.md §6.2), for `og ui check`.
    pub overlaps: Vec<Overlap>,
    /// What motion did this frame: timeline events (sounds, haptics,
    /// callbacks), plays that ended, swipes, snaps and drops.
    pub events: Vec<UiEvent>,
    /// Elements built, rebuilt and laid out (docs/ui.md §6.3).
    pub stats: LayoutStats,
    /// Elements in visual states and the animations in flight, for `dump_ui`.
    pub motion: Vec<crate::motion::run::MotionDump>,
    /// The og-ui unit, CSS px per unit (docs/ui.md §15.2): `viewport`,
    /// `elements`, `a11y`, `text_input` and `overlaps` are in units (the
    /// game's coordinates); `items` are already in CSS px. A host that hands
    /// a rectangle to the page (an input method's window, the accessibility
    /// mirror, the probe) multiplies by it.
    pub unit: f32,
}

/// What [`Ui::edit_marks`] draws one line's marks from.
pub(crate) struct EditMarks<'a> {
    pub m: &'a crate::text::FontMetrics,
    /// The line's text as shown.
    pub text: &'a str,
    /// Where its first glyph's pen starts, and the line box's top and height.
    pub x: f32,
    pub top: f32,
    pub line_h: f32,
    pub size: f32,
    pub tracking: f32,
    /// The index of the line's first character in the whole text.
    pub first: usize,
    pub edit: &'a crate::layout::EditView,
    pub clip: Rect,
    pub alpha: f32,
    /// The selection (under the text), or the composition and caret (over it).
    pub under: bool,
    /// The caret is on this line (a caret at a soft break shows once).
    pub caret_here: bool,
}

/// The focused text control as a host sees it ([`UiFrame::text_input`]):
/// the browser shell puts its hidden text target over it (so an input
/// method's candidate window opens there) and copies `selection` on the
/// page's copy and cut.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextTarget {
    /// The element's path.
    pub path: String,
    /// Its rectangle, CSS px.
    pub rect: Rect,
    /// A multi-line text area.
    pub multiline: bool,
    /// The selected text (empty: none).
    pub selection: String,
    /// An input method is composing in it.
    pub composing: bool,
}

impl UiFrame {
    /// The element at an id or `Parent/Child` path (topmost first), as `touch_move_to` resolves it.
    pub fn element(&self, path: &str) -> Option<&UiElement> {
        find_by_id(&self.elements, path)
    }

    /// The element showing a text (`[=]text[@N]`), as `move_to_text` resolves it.
    pub fn element_by_text(&self, query: &str) -> Option<&UiElement> {
        find_by_text(&self.elements, query)
    }

    /// Painter levels used: the most sprite and text runs the frame can take.
    pub fn levels(&self) -> i32 {
        self.items.iter().map(|i| i.level + 1).max().unwrap_or(0)
    }

    /// Appends the frame's sprites, text and clip rectangles to an overlay
    /// (`RenderScene::overlay`), in CSS pixels: make it with the viewport's
    /// size and `dpr` (`og_render::Overlay::new(vp.width, vp.height, vp.dpr)`).
    pub fn emit(&self, desc: &EmitDesc, overlay: &mut Overlay) {
        emit(&self.items, desc, overlay, &|_| None);
    }

    /// [`UiFrame::emit`] with each anchored element ([`crate::WorldCmd::Anchor`])
    /// moved by `shift` of its key, CSS px: a host drawing between ticks
    /// moves a nameplate by how far its point's projection moved since the
    /// tick placed it, so it stays on its entity. `None` leaves it.
    pub fn emit_shifted(
        &self,
        desc: &EmitDesc,
        overlay: &mut Overlay,
        shift: &dyn Fn(u64) -> Option<og_core::glam::Vec2>,
    ) {
        // The shift is in units (the frame's viewport); the items in CSS px.
        let unit = self.unit.max(1.0);
        emit(&self.items, desc, overlay, &|k| shift(k).map(|d| d * unit));
    }
}

/// Button roles: how a style draws each.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    /// A clear lens: neutral fill (the default).
    #[default]
    Secondary,
    /// The primary action: one per screen.
    Primary,
    /// Text until hovered.
    Quiet,
    /// A destructive action: danger ink on the danger tint.
    Danger,
    /// An outlined button: clear inside a strong line until hovered, for
    /// a second action beside a primary that must still read as a button.
    Ghost,
}

/// Control heights from the density (`controlSm/md/lg`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ControlSize {
    Small,
    #[default]
    Medium,
    Large,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ButtonDesc {
    pub variant: ButtonVariant,
    pub size: ControlSize,
    pub disabled: bool,
    /// `None`: fit the label.
    pub width: Option<Size>,
    /// Take the focus when it appears (it was not on screen last frame)
    /// and no dialog is open: what opens takes the focus, so a title
    /// menu's first item answers Enter and the pad's A at once.
    pub autofocus: bool,
    /// A square of the control height with no side padding: an icon
    /// button, a one-glyph label (`+`). Overrides `width`.
    pub square: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LabelDesc {
    /// `None`: `text-body`.
    pub step: Option<TypeStep>,
    /// `None`: `ink`.
    pub color: Option<Srgba>,
    pub align: TextAlign,
    /// Keep on one line (break only at `\n`).
    pub nowrap: bool,
    pub width: Size,
    pub font: FontRole,
    /// A heading: a style with a flourish frames it.
    pub heading: bool,
    /// Cut with "…" where the text does not fit its width: on its one line
    /// (with `nowrap`; the label can then shrink in a row) or on its last
    /// line (`max_lines`).
    pub ellipsis: bool,
    /// The most lines a wrapping label shows (0: no limit); with `ellipsis`
    /// the last one ends in "…" when text is left out.
    pub max_lines: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PanelDesc {
    /// Lay children left to right (default: top to bottom).
    pub row: bool,
    pub width: Size,
    pub height: Size,
    /// `None`: `space.lg`.
    pub padding: Option<Insets>,
    /// `None`: `space.md`.
    pub gap: Option<f32>,
    pub align: Align,
    pub cross: Cross,
    /// `None`: `panel`.
    pub background: Option<Srgba>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScrollDesc {
    pub width: Size,
    pub height: Size,
    pub padding: Insets,
    pub gap: f32,
    /// Which ways it scrolls (vertical by default).
    pub axes: ScrollAxes,
}

/// Which ways a scroll container scrolls.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollAxes {
    #[default]
    Vertical,
    Horizontal,
    Both,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ImageDesc {
    /// `None` draws a solid rectangle of `tint`.
    pub texture: Option<TextureHandle>,
    /// `None`: the whole texture (`[u0, v0, u1, v1]`, v down).
    pub uv: Option<[f32; 4]>,
    /// `None`: white.
    pub tint: Option<Srgba>,
    pub width: Size,
    pub height: Size,
    /// Width over height (0: none): with the height left to fit, the image
    /// is as tall as its width over this (`Element::aspect`).
    pub aspect: f32,
    /// What a screen reader calls it (and the probe's text); empty is
    /// decoration.
    pub label: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ProgressDesc {
    /// `None`: grow.
    pub width: Option<Size>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SliderDesc {
    /// The range; `min == max` (the zero value) means 0 to 1.
    pub min: f32,
    pub max: f32,
    /// Snap to multiples of `step` from `min` (0: continuous).
    pub step: f32,
    /// `None`: grow.
    pub width: Option<Size>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FieldDesc {
    /// Shown, dimmed, while the field is empty.
    pub placeholder: String,
    /// `None`: grow.
    pub width: Option<Size>,
    /// The most characters it takes (0: no limit).
    pub max_chars: u32,
    /// Its text is wrong (a form's error): `Field.frame[invalid]`, the
    /// danger outline an undressed frame takes.
    pub invalid: bool,
    /// No frame and no focus ring of its own, a small inset: a component
    /// frames it (an input with an icon and a clear button).
    pub bare: bool,
}

/// How a multi-line text area looks ([`Ui::text_area`]).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextAreaDesc {
    /// Shown, dimmed, while the area is empty.
    pub placeholder: String,
    /// `None`: grow.
    pub width: Option<Size>,
    /// Lines shown at once (0: 4); more scroll.
    pub rows: u32,
    /// The most characters it takes (0: no limit).
    pub max_chars: u32,
    /// Its text is wrong: `Field.frame[invalid]`.
    pub invalid: bool,
    /// No frame and no focus ring of its own: a component frames it.
    pub bare: bool,
}
