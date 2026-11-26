//! Typed style-pack inputs. CSS colours, shadows and class strings remain
//! their own languages; no embedded pack JSON/OGT is accepted as configuration.
use super::super::ContentRef;
use og_core::reflect::source::{InputSchema, schemars::JsonSchema};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// One style layer. Class resolution, CSS parsing, scheme-pair semantics,
/// slot names and referenced content are checked by the pack domain validator.
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackInput {
    /// Ignored author annotation, deliberately arbitrary JSON data.
    pub about: Option<serde_json::Value>,
    pub scheme: Option<Scheme>,
    pub seeds: Option<Seeds>,
    pub theme: Option<Theme>,
    pub contrast: Option<Theme>,
    /// Component.slot[.variant] -> class string; empty declares the slot.
    #[serde(default)]
    pub variants: BTreeMap<String, String>,
    /// Ordered font faces; family, weight and style belong to each face.
    #[serde(default)]
    #[schemars(extend("x-og-ref-kinds" = ["font"]))]
    pub fonts: Vec<ContentRef>,
    #[serde(default)]
    #[schemars(extend("x-og-ref-kinds" = ["texture"]))]
    pub icons: BTreeMap<String, ContentRef>,
    #[schemars(extend("x-og-ref-kinds" = ["icons"]))]
    pub icon_set: Option<ContentRef>,
    /// Named nine-slice art; names are kebab-case.
    #[serde(default)]
    pub images: BTreeMap<String, Adaptive<NineSlice>>,
    /// An event maps to a sound; a Component.slot[.variant] maps to events.
    #[serde(default)]
    pub sounds: BTreeMap<String, SoundBinding>,
    #[serde(default)]
    pub haptics: BTreeMap<String, HapticBinding>,
}

impl PackInput {
    pub fn schema() -> &'static InputSchema<Self> {
        static SCHEMA: OnceLock<InputSchema<PackInput>> = OnceLock::new();
        SCHEMA.get_or_init(|| InputSchema::new().expect("style pack input schema"))
    }
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum Scheme {
    Light,
    Dark,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Schemes<T> {
    pub light: T,
    pub dark: T,
}

/// A shared value or an explicit complete light/dark pair.
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Adaptive<T> {
    Schemes(Schemes<T>),
    Shared(T),
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Seeds {
    pub primary: Option<Adaptive<String>>,
    pub accent: Option<Adaptive<String>>,
    pub neutral: Option<Adaptive<String>>,
    pub danger: Option<Adaptive<String>>,
    pub warn: Option<Adaptive<String>>,
    pub success: Option<Adaptive<String>>,
    pub info: Option<Adaptive<String>>,
    pub team: Option<Adaptive<String>>,
    pub hud: Option<Adaptive<String>>,
}

type Scale<T> = Option<Adaptive<BTreeMap<String, Adaptive<T>>>>;

/// NSH Tailwind theme sections, as consumed by Theme::from_json after scheme
/// splitting. Numerical scales are numbers, not arbitrary CSS/JSON values.
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Theme {
    pub colors: Scale<Palette>,
    /// Base spacing unit in points.
    #[schemars(extend("x-og-unit" = "pt"))]
    pub spacing: Option<Adaptive<f32>>,
    /// Root em in points.
    #[schemars(extend("x-og-unit" = "pt"))]
    pub rem: Option<Adaptive<f32>>,
    pub breakpoints: Scale<f32>,
    pub container: Scale<f32>,
    pub font_size: Scale<FontSize>,
    pub font_weight: Scale<f32>,
    pub font_family: Scale<String>,
    pub tracking: Scale<f32>,
    pub leading: Scale<f32>,
    pub radius: Scale<f32>,
    /// CSS box-shadow values.
    pub shadow: Scale<String>,
    pub inset_shadow: Scale<String>,
    pub drop_shadow: Scale<String>,
    /// CSS text-shadow values.
    pub text_shadow: Scale<String>,
    pub blur: Scale<f32>,
    pub perspective: Scale<f32>,
    pub aspect: Scale<f32>,
    /// Cubic Bezier control points [x1, y1, x2, y2].
    pub ease: Scale<[f32; 4]>,
    #[schemars(extend("x-og-unit" = "ms"))]
    pub duration: Scale<f32>,
    pub opacity: Scale<f32>,
    pub stroke: Scale<f32>,
    pub scale: Scale<f32>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Palette {
    /// CSS colour string.
    Color(String),
    /// Named shades; DEFAULT is the palette's unqualified colour.
    Shades(BTreeMap<String, Adaptive<String>>),
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum FontSize {
    Points(f32),
    /// Font size in points, unitless line height.
    WithLeading([f32; 2]),
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(transparent)]
pub struct Nonnegative(#[schemars(range(min = 0))] pub f32);

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Insets {
    All(Nonnegative),
    /// Top, right, bottom, left.
    Sides([Nonnegative; 4]),
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct NineSlice {
    #[schemars(extend("x-og-ref-kinds" = ["texture"]))]
    pub src: Adaptive<ContentRef>,
    #[schemars(extend("x-og-unit" = "px"))]
    pub slice: Adaptive<Insets>,
    /// Unset draws at the slice's own widths, in points.
    #[schemars(extend("x-og-unit" = "pt"))]
    pub width: Option<Adaptive<Insets>>,
    /// Unset paints the centre.
    pub fill: Option<Adaptive<bool>>,
    pub repeat: Option<Adaptive<Repeat>>,
    pub tint: Option<Adaptive<bool>>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum Repeat {
    Stretch,
    Tile,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Range {
    Fixed(f32),
    Between([f32; 2]),
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Sound {
    /// Empty clips silence an underlying layer only when volume is zero.
    #[serde(default)]
    #[schemars(extend("x-og-ref-kinds" = ["sound"]))]
    pub clips: Vec<ContentRef>,
    /// Gain or [low, high]; unset is 1.
    pub volume: Option<Range>,
    /// Playback rate or [low, high]; unset is 1.
    pub pitch: Option<Range>,
}

/// The same interaction event vocabulary for global and slot-local values.
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Events<T> {
    pub press: Option<T>,
    pub hover: Option<T>,
    pub select: Option<T>,
    pub confirm: Option<T>,
    pub cancel: Option<T>,
    pub back: Option<T>,
    pub open: Option<T>,
    pub close: Option<T>,
    pub error: Option<T>,
    pub reward: Option<T>,
    pub toggle: Option<T>,
    pub tab: Option<T>,
    pub levelup: Option<T>,
    pub purchase: Option<T>,
    pub countdown: Option<T>,
    pub tick: Option<T>,
    pub step: Option<T>,
    pub swipe: Option<T>,
    pub notify: Option<T>,
    pub unlock: Option<T>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum SoundBinding {
    Global(Sound),
    Scoped(Box<Events<Sound>>),
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum HapticBinding {
    Global(Haptic),
    Scoped(Events<Haptic>),
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum Haptic {
    Tick,
    Tap,
    Thud,
    Double,
    Buzz,
    Success,
    Warning,
    Error,
    None,
}
