//! The fonts a UI text draws with: each role's face, the style's CJK
//! companions, the locale's chain (og.i18n), the game's fallbacks and the
//! named families, and the face a text or run takes from them.

use og_text::text::UiFont;

/// The default UI's runtime-outline face, bundled for games with UI documents
/// and loaded only when no game font or style supplies the role.
pub const DEFAULT_FONT_ASSET: &str = "@styles/shared/InstrumentSans[wdth,wght].ttf";

/// Which font a text uses.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FontRole {
    #[default]
    Body,
    /// Display titles and key numbers (falls back to `Body`).
    Display,
    /// Code, ids and prices (falls back to `Body`).
    Mono,
    /// Levels and scores (falls back
    /// to `Display`, then `Body`).
    Rounded,
    /// A family a document's `@font-face` rules name (`font-family:
    /// "Name", <generic>`): its faces matched by weight, style and
    /// `unicode-range`, then its fallback role ([`Fonts::families`]).
    Family(Family),
}

/// A named font family as a text refers to it: the family's id and the
/// role a text falls back to where the family has no face for a character
/// (the list's generic family; body when it names none).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Family {
    id: u32,
    fallback: u8,
}

impl Family {
    /// `name` (compared case-insensitively, as CSS family names are)
    /// falling back to `fallback` (a role; a family's own fallback when
    /// `fallback` is a family).
    pub fn new(name: &str, fallback: FontRole) -> Family {
        Family {
            id: Family::id_of(name),
            fallback: match fallback.base() {
                FontRole::Display => 1,
                FontRole::Mono => 2,
                FontRole::Rounded => 3,
                _ => 0,
            },
        }
    }

    /// The id of the family `name`: FNV-1a over its ASCII-lowercased bytes.
    pub fn id_of(name: &str) -> u32 {
        name.bytes().fold(0x811c_9dc5, |h, b| {
            (h ^ u32::from(b.to_ascii_lowercase())).wrapping_mul(0x0100_0193)
        })
    }

    pub fn id(self) -> u32 {
        self.id
    }

    /// The four roles, in the order a family's fallback indexes them.
    pub const ROLES: [FontRole; 4] = [
        FontRole::Body,
        FontRole::Display,
        FontRole::Mono,
        FontRole::Rounded,
    ];

    /// The role a character the family has no face for is drawn in.
    pub fn fallback(self) -> FontRole {
        Family::ROLES[self.fallback_index()]
    }

    /// [`Family::fallback`]'s index in [`Family::ROLES`].
    pub fn fallback_index(self) -> usize {
        usize::from(self.fallback)
    }
}

impl FontRole {
    /// The role a style's text effects are keyed by (`body`, `display`,
    /// `numeric`): a rounded text takes the display role's.
    pub fn effect_index(self) -> usize {
        match self.base() {
            FontRole::Display | FontRole::Rounded => 1,
            FontRole::Mono => 2,
            _ => 0,
        }
    }

    /// The role itself, or a family's fallback role.
    pub fn base(self) -> FontRole {
        match self {
            FontRole::Family(f) => f.fallback(),
            r => r,
        }
    }

    /// The role an IDL name names (`body`, `display`, `mono`, `rounded`).
    pub fn from_name(name: &str) -> Option<FontRole> {
        Some(match name {
            "body" => FontRole::Body,
            "display" => FontRole::Display,
            "mono" => FontRole::Mono,
            "rounded" => FontRole::Rounded,
            _ => return None,
        })
    }
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
    /// The faces of the named families documents declare (`@font-face`),
    /// in declaration order: a text of [`FontRole::Family`] takes the
    /// family's face its weight and style match (CSS Fonts 4 §5.2), each
    /// character from the first face of that set, last declared first,
    /// whose `unicode-range` has it, then from the family's fallback role
    /// and the fonts after it ([`Fonts::for_styled`]).
    pub families: Vec<FamilyFace>,
    /// Interned CSS family lists and their resolved weight/style combinations.
    pub family_lists: FamilyLists,
    /// The face a style's named family backs each of body, display, mono
    /// and rounded with ([`Fonts::back_roles`]), where it has no font of
    /// its own.
    pub backed: [Option<og_draw::FontHandle>; 4],
}

type FamilySelection = (u32, usize, FontQuery);

/// A family list for a query, each face's synthesis, and the primary
/// face's matched weight and width.
#[derive(Clone, Debug, PartialEq)]
struct ResolvedFamily {
    font: UiFont,
    synthesis: Vec<Synth>,
    at: (u16, u16),
}

/// One entry in an ordered CSS font-family list. Quoted generic names are named families.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FontFamily {
    Named(String),
    /// A generic family every style has a face for (`serif`, `monospace`,
    /// …): the role's face, another's where it has none.
    Generic(FontRole),
    /// A generic family only a suitable face matches (`system-ui`,
    /// `ui-sans-serif`, `ui-monospace`, `ui-rounded`; CSS Fonts 4 §2.1.3):
    /// the role's own face, else none and the list goes on.
    Ui(FontRole),
}

impl FontFamily {
    pub fn css(&self) -> String {
        match self {
            Self::Named(name) => format!("\"{}\"", name.replace('\\', "\\\\").replace('"', "\\\"")),
            Self::Generic(role) => match role.base() {
                FontRole::Mono => "monospace",
                FontRole::Display => "fantasy",
                FontRole::Rounded => "ui-rounded",
                _ => "sans-serif",
            }
            .into(),
            Self::Ui(role) => match role.base() {
                FontRole::Mono => "ui-monospace",
                FontRole::Rounded => "ui-rounded",
                _ => "system-ui",
            }
            .into(),
        }
    }
}

#[derive(Clone, Default, PartialEq)]
pub struct FamilyLists {
    names: elsa::FrozenMap<u32, Box<[FontFamily]>>,
    fonts: elsa::FrozenMap<FamilySelection, Box<ResolvedFamily>>,
    /// Faces at a variable font's instance ([`Fonts::instanced`]).
    instances: elsa::FrozenMap<InstanceKey, Box<UiFont>>,
    /// Each instance's face by the handle it draws as (the first made),
    /// so [`Fonts::face`] finds text drawn in one.
    instance_faces: elsa::FrozenMap<og_draw::FontHandle, Box<UiFont>>,
    /// Mixed fonts presenting their emoji otherwise than by Unicode
    /// ([`Fonts::presented`]), by the font's handle, its runs' and file's
    /// addresses (the entry holds both) and the presentation.
    presented: elsa::FrozenMap<PresentedKey, Box<UiFont>>,
    /// Fonts with their faces at `font-size-adjust`'s sizes
    /// ([`Fonts::adjusted`]), keyed as `presented` with the presentation
    /// the font has and the adjustment.
    adjusted: elsa::FrozenMap<(PresentedKey, og_text::text::SizeAdjust), Box<UiFont>>,
}

type PresentedKey = (
    Option<og_draw::FontHandle>,
    usize,
    usize,
    og_text::text::EmojiStyle,
);

/// A face at an instance: the instance's file, the handle it is drawn
/// with once the host made it, and the face's own handle and other fonts
/// (by address; the entry holds both, so neither is freed while it is
/// keyed).
type InstanceKey = (
    usize,
    Option<og_draw::FontHandle>,
    Option<og_draw::FontHandle>,
    usize,
);

impl std::fmt::Debug for FamilyLists {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FamilyLists")
            .field("lists", &self.names.len())
            .finish()
    }
}

/// An `@font-face`'s `font-style`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FaceStyle {
    #[default]
    Normal,
    Italic,
    Oblique,
}

/// One `@font-face` of a named family ([`Fonts::families`]).
#[derive(Clone, Debug, PartialEq)]
pub struct FamilyFace {
    /// The family name as declared (matched case-insensitively).
    pub family: String,
    /// The document whose stylesheet declares it (its faces are replaced
    /// when it is defined again).
    pub document: String,
    /// `font-weight` as an inclusive range (one weight is both ends).
    pub weight: (u16, u16),
    pub style: FaceStyle,
    /// `font-stretch` (`font-width`) as an inclusive range in tenths of a
    /// percent of the normal width (1000: normal; one width is both ends).
    pub stretch: (u16, u16),
    /// Its font, limited to its `unicode-range`
    /// ([`og_text::text::FontMetrics::with_range`]).
    pub font: UiFont,
    /// The font with the runs a text it draws only in part takes, for each
    /// fallback role (body, display, mono, rounded): made by
    /// [`Fonts::link`] (leave it empty).
    pub linked: Vec<UiFont>,
}

impl Default for FamilyFace {
    fn default() -> FamilyFace {
        FamilyFace {
            family: String::new(),
            document: String::new(),
            weight: (400, 400),
            style: FaceStyle::Normal,
            stretch: (FontQuery::NORMAL_WIDTH, FontQuery::NORMAL_WIDTH),
            font: UiFont::default(),
            linked: Vec::new(),
        }
    }
}

impl FamilyFace {
    fn is(&self, family: Family) -> bool {
        Family::id_of(&self.family) == family.id()
    }
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
/// oblique). As what a text may synthesise (CSS `font-synthesis`) it also
/// says whether its small capitals and super- and subscripts may be, which
/// rich text does where the face lacks their forms (CSS Fonts 4 §6.3,
/// §6.5); `for_styled` leaves those false.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Synth {
    pub bold: bool,
    pub italic: bool,
    pub small_caps: bool,
    pub position: bool,
}

impl Synth {
    /// Everything may be synthesised (CSS `font-synthesis`' initial value).
    pub const ALL: Synth = Synth {
        bold: true,
        italic: true,
        small_caps: true,
        position: true,
    };
}

/// What a text asks of its font family besides its characters (CSS Fonts
/// 4 §5.2): its weight (0: the role's own face), whether it is italic, and
/// its width (`font-stretch`, `font-width`) in tenths of a percent of the
/// normal width; and of a variable face (§7.2) its variation settings
/// (`font-variation-settings`) and its optical size in tenths of a CSS
/// px (`font-optical-sizing: auto`, the text's size; 0: `none`); and of a
/// colour face its palette (`font-palette`, §9); how its emoji are
/// presented (`font-variant-emoji`, §6.12); and the size its faces are
/// drawn at (`font-size-adjust`, CSS Fonts 5 §2.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FontQuery {
    pub weight: u16,
    pub italic: bool,
    pub stretch: u16,
    pub variations: og_text::shape::Variations,
    pub optical: u16,
    pub palette: og_text::shape::Palette,
    pub emoji: og_text::text::EmojiStyle,
    pub size_adjust: Option<og_text::text::SizeAdjust>,
}

impl FontQuery {
    /// The normal width, 100 %, in [`FontQuery::stretch`]'s tenths.
    pub const NORMAL_WIDTH: u16 = 1000;

    /// At `weight` and slant, the normal width.
    pub const fn new(weight: u16, italic: bool) -> FontQuery {
        FontQuery {
            weight,
            italic,
            stretch: Self::NORMAL_WIDTH,
            variations: og_text::shape::Variations::NONE,
            optical: 0,
            palette: og_text::shape::Palette::NONE,
            emoji: og_text::text::EmojiStyle::Unicode,
            size_adjust: None,
        }
    }

    /// With its faces sized as `adjust` says.
    pub fn with_size_adjust(self, size_adjust: Option<og_text::text::SizeAdjust>) -> FontQuery {
        FontQuery {
            size_adjust,
            ..self
        }
    }

    /// With its emoji presented as `emoji` says.
    pub fn with_emoji(self, emoji: og_text::text::EmojiStyle) -> FontQuery {
        FontQuery { emoji, ..self }
    }

    /// In a colour face's `palette`.
    pub fn in_palette(self, palette: og_text::shape::Palette) -> FontQuery {
        FontQuery { palette, ..self }
    }

    /// With a variable face's settings, and its optical size at `size`
    /// CSS px when `optical` (`font-optical-sizing: auto`).
    pub fn with_axes(
        self,
        variations: og_text::shape::Variations,
        optical: bool,
        size: f32,
    ) -> FontQuery {
        FontQuery {
            variations,
            optical: if optical {
                (size * 10.0).round().clamp(1.0, f32::from(u16::MAX)) as u16
            } else {
                0
            },
            ..self
        }
    }

    /// The axis settings a variable face is instanced at (CSS Fonts 4
    /// §7.2): `wght` from the weight, `wdth` from the width, `opsz` from
    /// the optical size, then `font-variation-settings`, which overrides.
    pub fn axes(&self) -> Vec<og_text::shape::Variation> {
        let mut out = Vec::new();
        if self.weight != 0 {
            out.push((*b"wght", f32::from(self.weight)));
        }
        if self.stretch != Self::NORMAL_WIDTH {
            out.push((*b"wdth", f32::from(self.stretch) / 10.0));
        }
        if self.optical != 0 {
            out.push((*b"opsz", f32::from(self.optical) / 10.0));
        }
        out.extend(self.variations.settings().iter().copied());
        out
    }

    /// At a width, a percentage of the normal (`None`: normal).
    pub fn at_width(self, percent: Option<f32>) -> FontQuery {
        FontQuery {
            stretch: percent.map_or(Self::NORMAL_WIDTH, Self::width_tenths),
            ..self
        }
    }

    /// A width percentage in tenths, as [`FontQuery::stretch`] keeps it.
    pub fn width_tenths(percent: f32) -> u16 {
        (percent * 10.0).round().clamp(1.0, f32::from(u16::MAX)) as u16
    }
}

impl Default for FontQuery {
    fn default() -> FontQuery {
        FontQuery::new(0, false)
    }
}

/// Whether a text at `weight` drawn in a face of weight `got` is given a
/// synthesised bold: it asks for 600 or more and the face is lighter
/// than 600, as browsers decide (CSS Fonts 4 §7.1 leaves the line to the
/// user agent).
fn needs_bold(weight: u16, got: u16) -> bool {
    weight >= 600 && got < 600
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
    pub fn family_names(&self, family: Family) -> Vec<FontFamily> {
        self.family_lists
            .names
            .get(&family.id())
            .map(<[FontFamily]>::to_vec)
            .unwrap_or_else(|| {
                self.families
                    .iter()
                    .find(|f| f.is(family))
                    .map(|f| vec![FontFamily::Named(f.family.clone())])
                    .unwrap_or_default()
            })
    }

    /// Retain named and generic families in CSS order, then use the fallback
    /// role. A [`FontFamily::Ui`] stays only where the style has the role's
    /// own face ([`Fonts::has_own`]), as its generic.
    pub fn family_list(&self, names: &[FontFamily], fallback: FontRole) -> FontRole {
        let names: Vec<FontFamily> = names
            .iter()
            .filter_map(|n| match n {
                FontFamily::Named(name) => self
                    .has_family(name)
                    .then(|| FontFamily::Named(name.to_ascii_lowercase())),
                FontFamily::Generic(_) => Some(n.clone()),
                FontFamily::Ui(role) => self.has_own(*role).then_some(FontFamily::Generic(*role)),
            })
            .collect();
        match names.as_slice() {
            [] => fallback,
            [FontFamily::Named(name)] => FontRole::Family(Family::new(name, fallback)),
            [FontFamily::Generic(role)] => *role,
            _ => {
                let key = names
                    .iter()
                    .map(FontFamily::css)
                    .collect::<Vec<_>>()
                    .join("\0");
                let family = Family::new(&key, fallback);
                if self.family_lists.names.get(&family.id()).is_none() {
                    self.family_lists
                        .names
                        .insert(family.id(), names.into_boxed_slice());
                }
                FontRole::Family(family)
            }
        }
    }

    /// Whether the style has a face of `role`'s own, not another role's
    /// standing in (body always has one).
    pub fn has_own(&self, role: FontRole) -> bool {
        match role.base() {
            FontRole::Display => self.display.is_some(),
            FontRole::Mono => self.mono.is_some(),
            FontRole::Rounded => self.rounded.is_some(),
            _ => true,
        }
    }

    pub fn get(&self, role: FontRole) -> &UiFont {
        match role {
            FontRole::Family(f) if self.family_lists.names.get(&f.id()).is_some() => {
                self.for_styled(role, FontQuery::new(400, false), "").0
            }
            FontRole::Family(f) => match self.family_set(f, FontQuery::new(400, false)).0.first() {
                Some(face) => &face.font,
                None => self.get(f.fallback()),
            },
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
    /// its atlas or runtime file has every character; else, for a text
    /// the role's font draws in part (a Latin name beside CJK or
    /// Arabic), the role's font mixed with the others: each run from the
    /// first of the role's, `chain`, `fallbacks` and `fallback` that has it
    /// ([`FontMetrics::is_mixed`]; once [`Fonts::link`] made it, as
    /// og-ui's `Ui` does); else the first of `chain` (the locale's faces)
    /// whose atlas has every character, then the first of `fallbacks`
    /// whose atlas has every character; else the style's `fallback` when it has a character
    /// the role's lacks; else the first font, in the same order, that can
    /// draw it with the characters its file renders at run time (`font {
    /// runtime true }`); else (no one font draws it all) the mixed font,
    /// so only the characters no font has are missing.
    pub fn for_text(&self, role: FontRole, text: &str) -> &UiFont {
        if let FontRole::Family(_) = role {
            return self.for_styled(role, FontQuery::new(0, false), text).0;
        }
        self.resolve(
            self.get(role),
            self.companion(role),
            text,
            og_text::text::EmojiStyle::Unicode,
        )
    }

    /// Gives each of body, display, mono and rounded that has no font of
    /// its own the face of the first family of its list (the style's
    /// `--font-body`, `--font-display`, `--font-mono`, `--font-rounded`)
    /// with a face ([`Fonts::families`]): its normal face nearest the
    /// role's weight that draws Latin. A style that brings its faces with
    /// `@font-face` has no other font, and text a widget sets in a role
    /// then draws in the style's family, as text with no font-family of
    /// its own takes the root's in a browser. Roles backed before are
    /// backed again, so a new style or new faces replace them.
    pub fn back_roles(&mut self, lists: [Vec<FontFamily>; 4]) {
        for (i, list) in lists.iter().enumerate() {
            let slot = match i {
                0 => Some(&self.body),
                1 => self.display.as_ref(),
                2 => self.mono.as_ref(),
                _ => self.rounded.as_ref(),
            };
            let handle = slot.and_then(|f| f.handle);
            // A backed role still holding its backing face gives it up; a
            // font set over it since is the role's own.
            if let Some(backing) = std::mem::take(&mut self.backed[i])
                && handle == Some(backing)
            {
                if i < 3 {
                    self.weights[i] = 0;
                }
                match i {
                    0 => self.body = UiFont::default(),
                    1 => self.display = None,
                    2 => self.mono = None,
                    _ => self.rounded = None,
                }
            }
            let own = match i {
                0 => self.body.handle.is_some(),
                1 => self.display.is_some(),
                2 => self.mono.is_some(),
                _ => self.rounded.is_some(),
            };
            if own {
                continue;
            }
            let weight = self
                .weights
                .get(i)
                .copied()
                .filter(|&w| w > 0)
                .unwrap_or(400);
            let Some((font, got)) = list.iter().find_map(|name| {
                let FontFamily::Named(name) = name else {
                    return None;
                };
                let (set, (got, _), _) = self.family_set(
                    Family::new(name, FontRole::Body),
                    FontQuery::new(weight, false),
                );
                let face = set
                    .iter()
                    .find(|f| f.font.metrics.has('a'))
                    .or(set.first())?;
                Some((face.font.clone(), got))
            }) else {
                continue;
            };
            self.backed[i] = font.handle;
            match i {
                0 => self.body = font,
                1 => self.display = Some(font),
                2 => self.mono = Some(font),
                _ => self.rounded = Some(font),
            }
            if i < 3 {
                self.weights[i] = got;
            }
        }
    }

    /// Whether the named family `name` has a face ([`Fonts::families`]).
    pub fn has_family(&self, name: &str) -> bool {
        let id = Family::id_of(name);
        self.families.iter().any(|f| Family::id_of(&f.family) == id)
    }

    /// The faces of `family` a text asking `query` (weight 0: 400) takes
    /// (CSS Fonts 4 §5.2): of the faces of the nearest width (a width
    /// range that holds it; else, at or under normal, the narrower ones
    /// nearest first and then the wider; over normal, the wider then the
    /// narrower), those of the nearest style (italic, then oblique, then
    /// normal for an italic text; normal, oblique, italic otherwise), those
    /// of the nearest weight range, last declared first; with the weight
    /// and width they draw at (the query's, clamped to their ranges) and
    /// whether they slant.
    fn family_set(&self, family: Family, query: FontQuery) -> (Vec<&FamilyFace>, (u16, u16), bool) {
        let FontQuery {
            weight,
            italic,
            stretch,
            ..
        } = query;
        let faces: Vec<&FamilyFace> = self.families.iter().filter(|f| f.is(family)).collect();
        let narrow = stretch <= FontQuery::NORMAL_WIDTH;
        let width_key = |&(a, b): &(u16, u16)| -> (u8, u16) {
            if (a..=b).contains(&stretch) {
                (0, 0)
            } else if b < stretch {
                (if narrow { 1 } else { 2 }, stretch - b)
            } else {
                (if narrow { 2 } else { 1 }, a - stretch)
            }
        };
        let Some(width) = faces.iter().map(|f| f.stretch).min_by_key(width_key) else {
            return (Vec::new(), (0, 0), false);
        };
        let faces: Vec<&FamilyFace> = faces.into_iter().filter(|f| f.stretch == width).collect();
        let order = if italic {
            [FaceStyle::Italic, FaceStyle::Oblique, FaceStyle::Normal]
        } else {
            [FaceStyle::Normal, FaceStyle::Oblique, FaceStyle::Italic]
        };
        let Some(style) = order
            .into_iter()
            .find(|s| faces.iter().any(|f| f.style == *s))
        else {
            return (Vec::new(), (0, 0), false);
        };
        let w = if weight == 0 { 400 } else { weight };
        // Within the range first; then, from 400 to 500, the heavier ones up
        // to 500 nearest first, the lighter ones nearest first, the ones
        // heavier than 500; below 400 lighter then heavier; above 500
        // heavier then lighter.
        let key = |&(a, b): &(u16, u16)| -> (u8, u16) {
            let middle = (400..=500).contains(&w);
            if (a..=b).contains(&w) {
                (0, 0)
            } else if w < a {
                let tier = match () {
                    _ if middle && a <= 500 => 1,
                    _ if middle => 3,
                    _ if w < 400 => 2,
                    _ => 1,
                };
                (tier, a - w)
            } else {
                (if w < 400 { 1 } else { 2 }, w - b)
            }
        };
        let Some(range) = faces
            .iter()
            .filter(|f| f.style == style)
            .map(|f| f.weight)
            .min_by_key(key)
        else {
            return (Vec::new(), (0, 0), false);
        };
        let set = faces
            .into_iter()
            .rev()
            .filter(|f| f.style == style && f.weight == range)
            .collect();
        let drawn = (w.clamp(range.0, range.1), stretch.clamp(width.0, width.1));
        (set, drawn, style != FaceStyle::Normal)
    }

    /// [`Fonts::for_text`] at a weight (100 to 900; 0: the role's own face)
    /// and slant: the role's face nearest the weight among its own and
    /// [`Fonts::faces`], an italic one first when `italic`, with the CJK
    /// companion nearest the same weight; and what it could not find a
    /// real face for ([`Synth`]: a weight of 600 or more whose face is
    /// lighter than 600, an italic with no italic face).
    ///
    /// A variable face is drawn at its instance at the query's axes
    /// ([`FontQuery::axes`]) with the weight and width it is drawn at (the
    /// query's clamped to its declared ranges, CSS Fonts 4 §7.2): a named
    /// face declared 400 draws a 700 at `wght` 400, synthesised bold.
    /// Role faces instead use the file's weight range and requested width.
    pub fn for_styled(&self, role: FontRole, query: FontQuery, text: &str) -> (&UiFont, Synth) {
        let face = FontQuery {
            variations: og_text::shape::Variations::NONE,
            optical: 0,
            palette: og_text::shape::Palette::NONE,
            size_adjust: None,
            ..query
        };
        let (font, synth, (weight, stretch)) = self.face_styled(role, face, text);
        let drawn = FontQuery {
            weight,
            stretch,
            ..query
        };
        let font = self.instanced(font, drawn).unwrap_or(font);
        let font = self.presented(font, query.emoji);
        (self.adjusted(font, query.size_adjust), synth)
    }

    /// `font` with each of its faces at the size `font-size-adjust` gives
    /// it ([`og_text::text::SizeAdjust::scales`]); itself for `none`.
    fn adjusted<'a>(
        &'a self,
        font: &'a UiFont,
        adjust: Option<og_text::text::SizeAdjust>,
    ) -> &'a UiFont {
        let Some(adjust) = adjust else {
            return font;
        };
        let runs = font.metrics.runs.as_ref();
        let file = font
            .metrics
            .file
            .as_ref()
            .map_or(0, |f| std::sync::Arc::as_ptr(f) as usize);
        let key = (
            (
                font.handle,
                runs.map_or(0, |r| r.as_ptr() as usize),
                file,
                font.metrics.emoji,
            ),
            adjust,
        );
        if let Some(f) = self.family_lists.adjusted.get(&key) {
            return f;
        }
        let parts = 1 + runs.map_or(0, |r| r.len());
        let scales = adjust.scales(&font.metrics, parts);
        let mut made = font.clone();
        made.metrics = made.metrics.scaled(scales[0]);
        made.metrics.runs = runs.map(|r| {
            r.iter()
                .zip(&scales[1..])
                .map(|(f, &scale)| UiFont {
                    metrics: f.metrics.clone().scaled(scale),
                    ..f.clone()
                })
                .collect()
        });
        self.family_lists.adjusted.insert(key, Box::new(made))
    }

    /// A mixed `font` presenting its emoji as `emoji` says (CSS
    /// `font-variant-emoji`); itself for `unicode`, which every font
    /// presents by, and for a font that is not mixed.
    fn presented<'a>(&'a self, font: &'a UiFont, emoji: og_text::text::EmojiStyle) -> &'a UiFont {
        let Some(runs) = font.metrics.runs.as_ref() else {
            return font;
        };
        if emoji == font.metrics.emoji {
            return font;
        }
        let file = font
            .metrics
            .file
            .as_ref()
            .map_or(0, |f| std::sync::Arc::as_ptr(f) as usize);
        let key = (font.handle, runs.as_ptr() as usize, file, emoji);
        match self.family_lists.presented.get(&key) {
            Some(f) => f,
            None => {
                let mut made = font.clone();
                made.metrics.emoji = emoji;
                self.family_lists.presented.insert(key, Box::new(made))
            }
        }
    }

    /// `font` at its instance at `query`'s axes and palette; `None` for a
    /// static face without colour glyphs, and one at its default location
    /// in its cooked palette.
    fn instanced(&self, font: &UiFont, query: FontQuery) -> Option<&UiFont> {
        let file = font.metrics.file.as_ref()?;
        if file.axes().is_empty() && !file.is_colour() {
            return None;
        }
        let instance = file.instance(&query.axes(), query.palette);
        if !instance.is_instance() {
            return None;
        }
        let runs = font
            .metrics
            .runs
            .as_ref()
            .map_or(0, |r| r.as_ptr() as usize);
        let key = (
            std::sync::Arc::as_ptr(&instance) as usize,
            instance.handle(),
            font.handle,
            runs,
        );
        let made = match self.family_lists.instances.get(&key) {
            Some(f) => f,
            None => {
                // Until the host makes its font, it draws with the nearest
                // instance's that it has made, so an animated palette or
                // axis keeps its text on screen between frames.
                let handle = instance
                    .handle()
                    .or_else(|| file.stand_in(&instance).and_then(|f| f.handle()));
                let made = UiFont {
                    handle,
                    metrics: font.metrics.at_instance(instance),
                };
                if let Some(h) = handle
                    && self.family_lists.instance_faces.get(&h).is_none()
                {
                    self.family_lists
                        .instance_faces
                        .insert(h, Box::new(made.clone()));
                }
                self.family_lists.instances.insert(key, Box::new(made))
            }
        };
        Some(made)
    }

    /// [`Fonts::for_styled`]'s face, before any variable face is
    /// instanced, with the weight and width it is drawn at.
    fn face_styled(
        &self,
        role: FontRole,
        query: FontQuery,
        text: &str,
    ) -> (&UiFont, Synth, (u16, u16)) {
        let FontQuery { weight, italic, .. } = query;
        if let FontRole::Family(family) = role {
            if let Some(names) = self.family_lists.names.get(&family.id()) {
                let key = (family.id(), family.fallback_index(), query);
                let resolved = self.family_lists.fonts.get(&key).unwrap_or_else(|| {
                    let mut fonts = Vec::new();
                    let mut at = (0, FontQuery::NORMAL_WIDTH);
                    let mut rendered = false;
                    for name in names {
                        let (set, (got, width), slanted) = match name {
                            FontFamily::Named(name) => {
                                let (faces, got, slanted) =
                                    self.family_set(Family::new(name, family.fallback()), query);
                                (
                                    faces.into_iter().map(|f| &f.font).collect::<Vec<_>>(),
                                    got,
                                    slanted,
                                )
                            }
                            FontFamily::Generic(role) | FontFamily::Ui(role) => {
                                let (font, got, slanted) = self.latin(*role, weight, italic);
                                (
                                    std::iter::once(font)
                                        .chain(self.companion_styled(*role, weight))
                                        .collect(),
                                    (got, query.stretch),
                                    slanted,
                                )
                            }
                        };
                        let renders = set.iter().any(|f| f.handle.is_some());
                        if fonts.is_empty() || (!rendered && renders) {
                            at = (got, width);
                        }
                        rendered |= renders;
                        fonts.extend(set.into_iter().map(|font| {
                            let synth = match name {
                                FontFamily::Named(_) => Synth {
                                    bold: needs_bold(weight, got),
                                    italic: italic && !slanted,
                                    ..Synth::default()
                                },
                                _ => self.fallback_synthesis(font, query),
                            };
                            (font.alone(), synth)
                        }));
                    }
                    let generic = family.fallback();
                    if rendered {
                        fonts.retain(|(f, _)| f.handle.is_some());
                    }
                    fonts.extend(
                        std::iter::once(self.latin(generic, weight, italic).0)
                            .chain(self.companion_styled(generic, weight))
                            .chain(&self.chain)
                            .chain(&self.fallbacks)
                            .chain(self.fallback.as_ref())
                            .chain(std::iter::once(&self.body))
                            .filter(|f| !rendered || f.handle.is_some())
                            .map(|f| (f.alone(), self.fallback_synthesis(f, query))),
                    );
                    let synthesis = fonts.iter().map(|(_, s)| *s).collect();
                    let mut font = fonts.remove(0).0;
                    // Preserve CSS family order when instancing clears a
                    // variable face's default-location atlas. A later
                    // cooked face must not displace its runtime glyphs.
                    font.metrics.prefer = fonts.len();
                    font.metrics.runs = Some(fonts.into_iter().map(|(f, _)| f).collect());
                    self.family_lists.fonts.insert(
                        key,
                        Box::new(ResolvedFamily {
                            font,
                            synthesis,
                            at,
                        }),
                    )
                });
                let k = resolved
                    .font
                    .metrics
                    .font_runs(text)
                    .first()
                    .map_or(0, |(_, k)| *k);
                return (&resolved.font, resolved.synthesis[k], resolved.at);
            }
            let (set, (got, width), slanted) = self.family_set(family, query);
            let Some(first) = set.first() else {
                return self.face_styled(family.fallback(), query, text);
            };
            let drawn = || text.chars().filter(|c| !c.is_whitespace());
            // The face that sets the text whole, else the first that draws
            // some of it (its runs take the rest).
            let face = set
                .iter()
                .find(|f| drawn().all(|c| f.font.metrics.in_atlas(c)))
                .or_else(|| set.iter().find(|f| drawn().any(|c| f.font.metrics.has(c))))
                .unwrap_or(first);
            let f = face
                .linked
                .get(family.fallback_index())
                .unwrap_or(&face.font);
            let synth = Synth {
                bold: needs_bold(weight, got),
                italic: italic && !slanted,
                ..Synth::default()
            };
            let companion = self.companion_styled(family.fallback(), weight);
            return (
                self.resolve(f, companion, text, query.emoji),
                synth,
                (got, width),
            );
        }
        let (f, got, slanted) = self.latin(role, weight, italic);
        let synth = Synth {
            bold: needs_bold(weight, got),
            italic: italic && !slanted,
            ..Synth::default()
        };
        let face = self.resolve(f, self.companion_styled(role, weight), text, query.emoji);
        (face, synth, (got, query.stretch))
    }

    /// Synthesis for a role, locale or fallback face appended to a CSS
    /// family list. Named families retain their matched descriptors above;
    /// these faces instead use the style's registered weights.
    fn fallback_synthesis(&self, font: &UiFont, query: FontQuery) -> Synth {
        let same = |f: &UiFont| std::ptr::eq(f, font);
        let (weight, slanted) = self
            .faces
            .iter()
            .find(|f| same(&f.font))
            .map(|f| (f.weight, f.italic))
            .unwrap_or_else(|| {
                let slot = [
                    Some(&self.body),
                    self.display.as_ref(),
                    self.mono.as_ref(),
                    None,
                    self.cjk.as_ref(),
                    self.cjk_display.as_ref(),
                ]
                .iter()
                .position(|f| f.is_some_and(same));
                (slot.map_or(400, |i| self.weight(i)), false)
            });
        let weight = font
            .metrics
            .file
            .as_ref()
            .and_then(|file| file.axes().iter().find(|axis| axis.tag == *b"wght"))
            .map_or(weight, |axis| {
                f32::from(if query.weight == 0 {
                    weight
                } else {
                    query.weight
                })
                .clamp(axis.min, axis.max)
                .round() as u16
            });
        Synth {
            bold: needs_bold(query.weight, weight),
            italic: query.italic && !slanted,
            ..Synth::default()
        }
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
            FontRole::Family(f) => self.slot(f.fallback()),
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
        // Role faces have no CSS descriptor restricting their weight:
        // match each variable candidate at the weight its axis reaches.
        // Keep the actual clamped weight for synthetic-bold detection.
        let at_weight = |w, font: &UiFont| {
            font.metrics
                .file
                .as_ref()
                .and_then(|file| file.axes().iter().find(|a| a.tag == *b"wght"))
                .map_or(w, |axis| {
                    f32::from(want).clamp(axis.min, axis.max).round() as u16
                })
        };
        let faces = |it: bool| {
            self.faces
                .iter()
                .filter(move |f| !f.cjk && i.is_some() && f.role == slot && f.italic == it)
                .map(|f| (at_weight(f.weight, &f.font), &f.font))
        };
        if italic && let Some((w, f)) = nearest(faces(true), want) {
            return (f, w, true);
        }
        let (w, f) = nearest(faces(false).chain([(at_weight(own, base), base)]), want)
            .unwrap_or((own, base));
        (f, w, false)
    }

    /// [`Fonts::for_text`]'s order for a text of the face `f` with the CJK
    /// companion `companion`.
    fn resolve<'a>(
        &'a self,
        f: &'a UiFont,
        companion: Option<&'a UiFont>,
        text: &str,
        emoji: og_text::text::EmojiStyle,
    ) -> &'a UiFont {
        // An emoji presented as emoji is drawn by its mixed font's colour
        // face, though the role's face has a text form of it.
        if let Some(m) = f.metrics.mixed.as_deref()
            && f.metrics.wants_colour(text, emoji)
        {
            return m;
        }
        let cooked = |font: &UiFont| {
            text.chars()
                .all(|c| c.is_whitespace() || font.metrics.in_atlas(c))
        };
        let drawable = |font: &UiFont| {
            text.chars()
                .all(|c| c.is_whitespace() || font.metrics.has(c))
        };
        if drawable(f) {
            return f;
        }
        // The style's companion draws what the role's face lacks.
        if let Some(c) = companion
            && let Some(m) = f.metrics.mixed.as_deref()
            && text
                .chars()
                .any(|ch| !ch.is_whitespace() && !f.metrics.has(ch) && c.metrics.has(ch))
        {
            return m;
        }
        let mixed = f.metrics.mixed.as_deref();
        if let Some(m) = mixed
            && text.chars().any(|c| !c.is_whitespace() && f.metrics.has(c))
        {
            return m;
        }
        if let Some(c) = self.chain.iter().find(|c| cooked(c)) {
            return c;
        }
        let others = || self.chain.iter().chain(&self.fallbacks);
        if let Some(c) = self.fallbacks.iter().find(|c| cooked(c)) {
            return c;
        }
        if let Some(fb) = &self.fallback
            && text
                .chars()
                .any(|c| !c.is_whitespace() && !f.metrics.has(c) && fb.metrics.has(c))
        {
            return fb;
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
    /// the body face last), for a text that mixes scripts. og-ui's `Ui`
    /// links the fonts it is given; call it again after changing them by
    /// hand.
    pub fn link(&mut self) {
        self.family_lists.fonts = elsa::FrozenMap::new();
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
                // The role's own runtime glyphs precede a later atlas,
                // just as its cooked glyphs do; include its companion.
                m.metrics.prefer = (*prefer).max(1);
                std::sync::Arc::new(m)
            });
        }
        // A family's face, for each fallback role: the other faces of its
        // set (its weight range and style, last declared first, each drawing
        // its own unicode-range), then the role's face nearest its weight,
        // the role's companion, the others and the body face.
        let linked: Vec<Vec<UiFont>> = self
            .families
            .iter()
            .enumerate()
            .map(|(i, face)| {
                let set: Vec<UiFont> = self
                    .families
                    .iter()
                    .enumerate()
                    .rev()
                    .filter(|&(j, f)| {
                        j != i
                            && f.family.eq_ignore_ascii_case(&face.family)
                            && f.style == face.style
                            && f.weight == face.weight
                    })
                    .map(|(_, f)| f.font.alone())
                    .collect();
                let w = 400u16.clamp(face.weight.0, face.weight.1);
                let slanted = face.style != FaceStyle::Normal;
                Family::ROLES
                    .iter()
                    .map(|&role| {
                        let own = self.latin(role, w, slanted).0.alone();
                        let available =
                            |font: &UiFont| face.font.handle.is_none() || font.handle.is_some();
                        let last =
                            (own.handle != body.handle && available(&body)).then(|| body.clone());
                        let own = available(&own).then_some(own);
                        let all: Vec<UiFont> = set
                            .iter()
                            .cloned()
                            .chain(own)
                            .chain(self.companion_styled(role, w).map(UiFont::alone))
                            .chain(others.iter().cloned())
                            .chain(last)
                            .collect();
                        let mut m = face.font.alone();
                        // An explicit runtime family keeps its supported
                        // glyphs ahead of the role and locale atlases too.
                        m.metrics.prefer = set.len().max(1).min(all.len());
                        m.metrics.runs = Some(all.into());
                        let mut f = face.font.alone();
                        f.metrics.mixed = Some(std::sync::Arc::new(m));
                        f
                    })
                    .collect()
            })
            .collect();
        for (face, linked) in self.families.iter_mut().zip(linked) {
            face.linked = linked;
        }
    }

    /// The face drawn as `handle`: a role's, a companion, the locale's, a
    /// fallback, or a variable face's instance (a weight or width a text
    /// is drawn at).
    pub fn face(&self, handle: og_draw::FontHandle) -> Option<&UiFont> {
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
        .chain(self.families.iter().map(|f| &f.font))
        .chain(&self.chain)
        .chain(&self.fallbacks)
        .find(|f| f.handle == Some(handle))
        .or_else(|| self.family_lists.instance_faces.get(&handle))
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
        match role.base() {
            FontRole::Display | FontRole::Rounded => pick(true).or_else(|| pick(false)),
            _ => pick(false),
        }
    }
}
