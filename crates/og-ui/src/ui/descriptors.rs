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
    /// The weight each of body, display, mono, fallback, cjk and
    /// cjk_display is (a style's `fonts { body 300 "…" }`; 0: 400).
    pub weights: [u16; 6],
    /// The style's other faces: a role's other weights and italics, and the
    /// companions of other CJK locales ([`Fonts::for_styled`]).
    pub faces: Vec<FontFace>,
    /// The active locale's CJK forms (og.i18n): a companion draws Han and
    /// kana only in its own locale's forms, so Japanese never takes a
    /// Simplified Chinese face ([`Fonts::companion_styled`]).
    pub cjk_locale: CjkLocale,
}

/// Which national forms a CJK face draws Han in (`fonts { cjk ja "…" }`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CjkLocale {
    /// Simplified Chinese: the companion of every locale that is not one of
    /// the others (a Chinese name in an English menu).
    #[default]
    Hans,
    Hant,
    Ja,
    Ko,
}

impl CjkLocale {
    pub const ALL: [CjkLocale; 4] = [
        CjkLocale::Hans,
        CjkLocale::Hant,
        CjkLocale::Ja,
        CjkLocale::Ko,
    ];

    /// Its name in a style's `fonts` block: a BCP 47 tag.
    pub fn tag(self) -> &'static str {
        match self {
            CjkLocale::Hans => "zh-Hans",
            CjkLocale::Hant => "zh-Hant",
            CjkLocale::Ja => "ja",
            CjkLocale::Ko => "ko",
        }
    }

    /// The forms a locale tag reads in: `zh-Hant`, `zh-TW`, `zh-HK` and
    /// `zh-MO` traditional, `ja` Japanese, `ko` Korean, any other
    /// (Simplified Chinese, or a locale that is not CJK) simplified.
    pub fn of_locale(tag: &str) -> CjkLocale {
        let t = tag.to_ascii_lowercase().replace('_', "-");
        let mut parts = t.split('-');
        match parts.next().unwrap_or("") {
            "ja" => CjkLocale::Ja,
            "ko" => CjkLocale::Ko,
            "zh" if parts.any(|p| matches!(p, "hant" | "tw" | "hk" | "mo")) => CjkLocale::Hant,
            _ => CjkLocale::Hans,
        }
    }
}

/// One of a style's other faces: a role's real weight or italic, or a CJK
/// companion of a locale ([`Fonts::faces`]).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FontFace {
    /// Body, Display or Mono; with `cjk`, Body is the text companion and
    /// Display the display one.
    pub role: FontRole,
    pub cjk: bool,
    /// A companion's forms (Hans for a Latin face).
    pub locale: CjkLocale,
    /// 100 to 900.
    pub weight: u16,
    pub italic: bool,
    pub font: UiFont,
}

/// What [`Fonts::for_styled`] could not find a real face for: the weight
/// and slant the draw synthesises (rich text's `[b]` thickening, `[i]`
/// oblique).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Synth {
    pub bold: bool,
    pub italic: bool,
}

/// The candidate nearest `want` (CSS's rule: from 500 up a tie goes
/// heavier, below it lighter), with its weight.
fn nearest<'a>(
    candidates: impl Iterator<Item = (u16, &'a UiFont)>,
    want: u16,
) -> Option<(u16, &'a UiFont)> {
    candidates.min_by_key(|&(w, _)| {
        let d = (i32::from(w) - i32::from(want)).unsigned_abs() * 2;
        // The tie-break: half a step towards the preferred side.
        let away = if want >= 500 { w < want } else { w > want };
        d + u32::from(away)
    })
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
        self.resolve(self.get(role), self.companion(role), text)
    }

    /// [`Fonts::for_text`] at a weight (100 to 900; 0: the role's own face)
    /// and slant: the role's face nearest the weight among its own and
    /// [`Fonts::faces`], an italic one first when `italic`, with the CJK
    /// companion nearest the same weight; and what it could not find a
    /// real face for ([`Synth`]: a bold asked for 600 or more with no face
    /// within 100 of it, an italic with no italic face).
    pub fn for_styled(
        &self,
        role: FontRole,
        weight: u16,
        italic: bool,
        text: &str,
    ) -> (&UiFont, Synth) {
        let (f, got, slanted) = self.latin(role, weight, italic);
        let synth = Synth {
            bold: weight >= 600 && got + 100 < weight,
            italic: italic && !slanted,
        };
        let face = self.resolve(f, self.companion_styled(role, weight), text);
        (face, synth)
    }

    /// The weight of the `i`-th face of body, display, mono, fallback, cjk
    /// and cjk_display.
    fn weight(&self, i: usize) -> u16 {
        match self.weights[i] {
            0 => 400,
            w => w,
        }
    }

    /// The role whose faces `role` takes (display and mono fall back to
    /// body's, rounded to display's) and its index in `weights` (`None`:
    /// the game's own rounded font, which has no others).
    fn slot(&self, role: FontRole) -> (FontRole, Option<usize>) {
        match role {
            FontRole::Body => (FontRole::Body, Some(0)),
            FontRole::Display if self.display.is_some() => (FontRole::Display, Some(1)),
            FontRole::Mono if self.mono.is_some() => (FontRole::Mono, Some(2)),
            FontRole::Rounded if self.rounded.is_some() => (FontRole::Rounded, None),
            FontRole::Rounded => self.slot(FontRole::Display),
            FontRole::Display | FontRole::Mono => (FontRole::Body, Some(0)),
        }
    }

    /// The Latin face of `role` nearest `weight` (0: its own) and `italic`,
    /// with its weight and whether it is italic.
    fn latin(&self, role: FontRole, weight: u16, italic: bool) -> (&UiFont, u16, bool) {
        let base = self.get(role);
        let (slot, i) = self.slot(role);
        let own = i.map_or(400, |i| self.weight(i));
        if weight == 0 && !italic {
            return (base, own, false);
        }
        let want = if weight == 0 { own } else { weight };
        let faces = |it: bool| {
            self.faces
                .iter()
                .filter(move |f| !f.cjk && i.is_some() && f.role == slot && f.italic == it)
                .map(|f| (f.weight, &f.font))
        };
        if italic && let Some((w, f)) = nearest(faces(true), want) {
            return (f, w, true);
        }
        let (w, f) = nearest(faces(false).chain([(own, base)]), want).unwrap_or((own, base));
        (f, w, false)
    }

    /// [`Fonts::for_text`]'s order for a text of the face `f` with the CJK
    /// companion `companion`.
    fn resolve<'a>(
        &'a self,
        f: &'a UiFont,
        companion: Option<&'a UiFont>,
        text: &str,
    ) -> &'a UiFont {
        let cooked = |font: &UiFont| {
            text.chars()
                .all(|c| c.is_whitespace() || font.metrics.in_atlas(c))
        };
        if cooked(f) {
            return f;
        }
        // The style's companion draws what the role's face lacks.
        if let Some(c) = companion
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

    /// Makes each face's mixed font ([`Fonts::for_text`]): the role's font
    /// (and each of its [`Fonts::faces`]) with its companion, `chain`,
    /// `fallbacks` and `fallback` after it (and, for every role but body,
    /// the body face last), for a text that mixes scripts. [`crate::Ui`]
    /// links the fonts it is given; call it again after changing them by
    /// hand.
    pub fn link(&mut self) {
        let others: Vec<UiFont> = self
            .chain
            .iter()
            .chain(&self.fallbacks)
            .chain(self.fallback.as_ref())
            .map(UiFont::alone)
            .collect();
        // A role other than body ends its runs with the body face: a mark
        // its own face lacks (Orbitron has no middle dot) is drawn in the
        // style's body rather than missing.
        let body = self.body.alone();
        // The runs of a face of `role` at `weight`, and how many of them
        // come before the locale's (its companion).
        let runs = |role: FontRole, weight: u16| {
            let companion = self.companion_styled(role, weight);
            let last = (role != FontRole::Body).then_some(&body);
            let all: Vec<UiFont> = companion
                .map(UiFont::alone)
                .into_iter()
                .chain(others.iter().cloned())
                .chain(last.cloned())
                .collect();
            let runs: Option<std::sync::Arc<[UiFont]>> = (!all.is_empty()).then(|| all.into());
            (runs, usize::from(companion.is_some()))
        };
        // The mono role's runs end with body too, its companion is body's.
        let roles = [
            runs(FontRole::Body, 0),
            runs(FontRole::Display, 0),
            runs(FontRole::Mono, 0),
            runs(FontRole::Display, 0),
        ];
        let faces: Vec<_> = self
            .faces
            .iter()
            .map(|f| (!f.cjk).then(|| runs(f.role, f.weight)))
            .collect();
        let fonts = [
            Some(&mut self.body),
            self.display.as_mut(),
            self.mono.as_mut(),
            self.rounded.as_mut(),
        ];
        let targets = fonts.into_iter().zip(roles.iter().map(Some)).chain(
            self.faces
                .iter_mut()
                .map(|f| Some(&mut f.font))
                .zip(faces.iter().map(Option::as_ref)),
        );
        for (f, runs) in targets {
            let (Some(f), Some((runs, prefer))) = (f, runs) else {
                continue;
            };
            f.metrics.mixed = runs.as_ref().map(|o| {
                let mut m = f.alone();
                m.metrics.runs = Some(o.clone());
                m.metrics.prefer = *prefer;
                std::sync::Arc::new(m)
            });
        }
    }

    /// The face drawn as `handle`: a role's, a companion, the locale's or
    /// a fallback.
    pub fn face(&self, handle: og_render::FontHandle) -> Option<&UiFont> {
        [
            Some(&self.body),
            self.display.as_ref(),
            self.mono.as_ref(),
            self.rounded.as_ref(),
            self.fallback.as_ref(),
            self.cjk.as_ref(),
            self.cjk_display.as_ref(),
        ]
        .into_iter()
        .flatten()
        .chain(self.faces.iter().map(|f| &f.font))
        .chain(&self.chain)
        .chain(&self.fallbacks)
        .find(|f| f.handle == Some(handle))
    }

    /// The style's CJK companion of `role` in the active locale's forms
    /// ([`Fonts::companion_styled`] at the role's own weight).
    pub fn companion(&self, role: FontRole) -> Option<&UiFont> {
        self.companion_styled(role, 0)
    }

    /// The style's CJK companion of `role` at `weight` (0: the companion's
    /// own): of the active locale's ([`Fonts::cjk_locale`]) companions, the
    /// display one beside display and rounded (the text one when it has
    /// none), the text one beside body and the numeric role, nearest the
    /// weight. `cjk` and `cjk_display` are the Simplified Chinese ones; a
    /// locale the style names no companion for has none, so its text takes
    /// the locale's own faces (`chain`) and never another locale's forms.
    pub fn companion_styled(&self, role: FontRole, weight: u16) -> Option<&UiFont> {
        let loc = self.cjk_locale;
        let set = |display: bool| {
            let own = match (loc, display) {
                (CjkLocale::Hans, true) => self.cjk_display.as_ref().map(|f| (self.weight(5), f)),
                (CjkLocale::Hans, false) => self.cjk.as_ref().map(|f| (self.weight(4), f)),
                _ => None,
            };
            own.into_iter().chain(
                self.faces
                    .iter()
                    .filter(move |f| {
                        f.cjk
                            && !f.italic
                            && f.locale == loc
                            && (f.role == FontRole::Display) == display
                    })
                    .map(|f| (f.weight, &f.font)),
            )
        };
        let pick = |display: bool| {
            if weight == 0 {
                // The companion's own face first, as before weights.
                let mut s = set(display);
                let first = s.next();
                match first {
                    Some((_, f)) if loc == CjkLocale::Hans => Some(f),
                    _ => nearest(first.into_iter().chain(s), 400).map(|(_, f)| f),
                }
            } else {
                nearest(set(display), weight).map(|(_, f)| f)
            }
        };
        match role {
            FontRole::Body | FontRole::Mono => pick(false),
            FontRole::Display | FontRole::Rounded => pick(true).or_else(|| pick(false)),
        }
    }
}

impl Metrics for Fonts {
    fn metrics_for(&self, role: FontRole, text: &str) -> &FontMetrics {
        &self.for_text(role, text).metrics
    }

    fn metrics_styled(
        &self,
        role: FontRole,
        weight: u16,
        italic: bool,
        text: &str,
    ) -> &FontMetrics {
        &self.for_styled(role, weight, italic, text).0.metrics
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
    /// The atlas that draws the names this one lacks (`UiDesc.fallbackIcons`):
    /// a game's whole set under a style pack's few painted icons.
    pub fallback: Option<Box<IconAtlas>>,
}

/// Where an icon of the atlas is drawn from ([`IconAtlas::find`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IconCell {
    /// An icon set's distance-field glyph.
    Glyph(og_render::FontHandle, char),
    /// A cell of a texture grid, `[u0, v0, u1, v1]`.
    Image(TextureHandle, [f32; 4]),
}

/// A picture rich text draws inline by name (`[img=<name>]`): a texture
/// region, one em tall at `aspect` (width over height).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InlineImage {
    pub texture: TextureHandle,
    /// `[u0, v0, u1, v1]`, v down.
    pub uv: [f32; 4],
    pub aspect: f32,
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

    /// The icon named `name`: from this atlas where it draws one, else
    /// from its fallback's.
    pub fn find(&self, name: &str) -> Option<IconCell> {
        let own = self.index_of(name).and_then(|k| match self.glyph(k) {
            Some((font, ch)) => Some(IconCell::Glyph(font, ch)),
            None => self.texture.map(|t| IconCell::Image(t, self.uv(k))),
        });
        own.or_else(|| self.fallback.as_deref()?.find(name))
    }

    /// This atlas and the fallbacks under it, top first.
    pub fn layers(&self) -> impl Iterator<Item = &IconAtlas> {
        std::iter::successors(Some(self), |a| a.fallback.as_deref())
    }
}

/// One laid-out frame: what the probe reads and what the renderer draws.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiFrame {
    pub viewport: Viewport,
    /// Every visible element in draw order with its (clipped) bounds and text.
    pub elements: Vec<UiElement>,
    /// The style part each of `elements` is drawn as (`Button.base[primary]`,
    /// its variant and state flags as of this frame), index for index;
    /// `None` for one no style dresses (a layout container, a link).
    pub parts: Vec<Option<crate::dress::PartRef>>,
    /// Draw commands with their painter levels.
    pub items: Vec<DrawItem>,
    /// Problems in this frame's tree (duplicate ids, unclosed elements), each naming the fix.
    pub warnings: Vec<String>,
    /// Characters this frame's texts draw that their face lacks, in its
    /// atlas and in the file it renders from at run time: per face, in the
    /// order first drawn, each (face, character) once in the UI's life. The
    /// host names the face and warns ([`Fonts::face`] has its metrics).
    pub missing_glyphs: Vec<(og_render::FontHandle, Vec<char>)>,
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
    /// The paths of the elements whose props say dense (a dense HUD's
    /// readouts, docs/ui.md §15.2), each covering its subtree, for `og ui
    /// check`.
    pub dense: Vec<String>,
    /// The paths of the scroll containers that scroll sideways and of the
    /// tracks dragged sideways (a carousel's), for `og ui check`: a line
    /// their edge cuts across scrolls into view.
    pub sideways: Vec<String>,
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
    /// The pointer's shape over what it is on (or holds), resolved
    /// ([`crate::Cursor`]; never `Auto`): the web shell sets the canvas's
    /// CSS cursor from it.
    pub cursor: crate::Cursor,
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
    /// Scene text: `sceneInk` with the scheme's `sceneShadow` separation.
    pub on_scene: bool,
    /// Uppercase in the UI's locale, as a step with `uppercase` is.
    pub uppercase: bool,
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
    /// Auto-fit: where the text does not fit its width at `step`, it takes
    /// the ramp's smaller steps down to this one (then its ellipsis, with
    /// `ellipsis`); `None` keeps its step.
    pub fit_min: Option<TypeStep>,
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
    /// Draws no scrollbars.
    pub hide_bars: bool,
    /// How far (CSS px) the content fades into `fade_into` at an edge
    /// where more of it lies beyond; 0 draws no fade.
    pub fade: f32,
    /// The surface the content fades into; `None`: `panel`.
    pub fade_into: Option<Srgba>,
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
    /// No inset and the height of its text's line: the text sits exactly
    /// where the caller lays the field (in a box it draws, a line of a
    /// page). With or without `bare`.
    pub tight: bool,
    /// The component its style dresses it as (`Poem`, `Search[large]`):
    /// its frame `<component>.frame`, its text `<component>.text` and its
    /// placeholder `<component>.placeholder`, with its states (focused,
    /// invalid, disabled). `None`: `Field`.
    pub component: Option<String>,
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
    /// No inset and `rows` lines of its text's height ([`FieldDesc::tight`]).
    pub tight: bool,
    /// The component its style dresses it as ([`FieldDesc::component`]).
    pub component: Option<String>,
}
