//! Structured pack inputs. Tooling supplies canonical content references;
//! generated design data uses strings. Both enter the same native layer.
#[cfg(feature = "source")]
use og_core::reflect::source::schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One style layer. Class resolution, CSS parsing, scheme-pair semantics,
/// slot names and referenced content are checked by the pack domain validator.
#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "source", derive(JsonSchema))]
#[cfg_attr(
    feature = "source",
    schemars(crate = "og_core::reflect::source::schemars")
)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[serde(bound(deserialize = "R: Deserialize<'de>"))]
pub struct PackInput<R = String> {
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
    #[cfg_attr(feature = "source", schemars(extend("x-og-ref-kinds" = ["font"])))]
    pub fonts: Vec<R>,
    #[serde(default)]
    #[cfg_attr(feature = "source", schemars(extend("x-og-ref-kinds" = ["texture"])))]
    pub icons: BTreeMap<String, R>,
    #[cfg_attr(feature = "source", schemars(extend("x-og-ref-kinds" = ["icons"])))]
    pub icon_set: Option<R>,
    /// Named nine-slice art; names are kebab-case.
    #[serde(default)]
    pub images: BTreeMap<String, Adaptive<NineSlice<R>>>,
    /// An event maps to a sound; a Component.slot[.variant] maps to events.
    #[serde(default)]
    pub sounds: BTreeMap<String, SoundBinding<R>>,
    #[serde(default)]
    pub haptics: BTreeMap<String, HapticBinding>,
}

#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "source", derive(JsonSchema))]
#[cfg_attr(
    feature = "source",
    schemars(crate = "og_core::reflect::source::schemars")
)]
#[serde(rename_all = "lowercase")]
pub enum Scheme {
    Light,
    Dark,
}

#[derive(Clone, Serialize)]
#[cfg_attr(feature = "source", derive(JsonSchema))]
#[cfg_attr(
    feature = "source",
    schemars(crate = "og_core::reflect::source::schemars")
)]
#[serde(deny_unknown_fields)]
pub struct Schemes<T> {
    pub light: T,
    pub dark: T,
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Schemes<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor<T>(std::marker::PhantomData<T>);
        impl<'de, T: Deserialize<'de>> serde::de::Visitor<'de> for Visitor<T> {
            type Value = Schemes<T>;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("an object with light and dark fields")
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                map: A,
            ) -> Result<Self::Value, A::Error> {
                #[derive(Deserialize)]
                #[serde(deny_unknown_fields)]
                struct Fields<T> {
                    light: T,
                    dark: T,
                }
                let Fields { light, dark } =
                    Fields::deserialize(serde::de::value::MapAccessDeserializer::new(map))?;
                Ok(Schemes { light, dark })
            }
        }
        // Derived struct readers also accept a sequence: [size, leading]
        // must remain a shared FontSize, never light=size and dark=leading.
        deserializer.deserialize_map(Visitor(std::marker::PhantomData))
    }
}

/// A shared value or an explicit complete light/dark pair.
#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "source", derive(JsonSchema))]
#[cfg_attr(
    feature = "source",
    schemars(crate = "og_core::reflect::source::schemars")
)]
#[serde(untagged)]
pub enum Adaptive<T> {
    Schemes(Schemes<T>),
    Shared(T),
}

impl<T> Adaptive<T> {
    pub fn at(&self, scheme: super::Scheme) -> &T {
        match self {
            Self::Shared(value) => value,
            Self::Schemes(sides) => match scheme {
                super::Scheme::Light => &sides.light,
                super::Scheme::Dark => &sides.dark,
            },
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "source", derive(JsonSchema))]
#[cfg_attr(
    feature = "source",
    schemars(crate = "og_core::reflect::source::schemars")
)]
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
#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "source", derive(JsonSchema))]
#[cfg_attr(
    feature = "source",
    schemars(crate = "og_core::reflect::source::schemars")
)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Theme {
    pub colors: Scale<Palette>,
    /// Base spacing unit in points.
    #[cfg_attr(feature = "source", schemars(extend("x-og-unit" = "pt")))]
    pub spacing: Option<Adaptive<f32>>,
    /// Root em in points.
    #[cfg_attr(feature = "source", schemars(extend("x-og-unit" = "pt")))]
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
    #[cfg_attr(feature = "source", schemars(extend("x-og-unit" = "ms")))]
    pub duration: Scale<f32>,
    pub opacity: Scale<f32>,
    pub stroke: Scale<f32>,
    pub scale: Scale<f32>,
}

#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "source", derive(JsonSchema))]
#[cfg_attr(
    feature = "source",
    schemars(crate = "og_core::reflect::source::schemars")
)]
#[serde(untagged)]
pub enum Palette {
    /// CSS colour string.
    Color(String),
    /// Named shades; DEFAULT is the palette's unqualified colour.
    Shades(BTreeMap<String, Adaptive<String>>),
}

#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "source", derive(JsonSchema))]
#[cfg_attr(
    feature = "source",
    schemars(crate = "og_core::reflect::source::schemars")
)]
#[serde(untagged)]
pub enum FontSize {
    Points(f32),
    /// Font size in points, unitless line height.
    WithLeading([f32; 2]),
}

#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "source", derive(JsonSchema))]
#[cfg_attr(
    feature = "source",
    schemars(crate = "og_core::reflect::source::schemars")
)]
#[serde(transparent)]
pub struct Nonnegative(#[cfg_attr(feature = "source", schemars(range(min = 0)))] pub f32);

#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "source", derive(JsonSchema))]
#[cfg_attr(
    feature = "source",
    schemars(crate = "og_core::reflect::source::schemars")
)]
#[serde(untagged)]
pub enum Insets {
    All(Nonnegative),
    /// Top, right, bottom, left.
    Sides([Nonnegative; 4]),
}

#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "source", derive(JsonSchema))]
#[cfg_attr(
    feature = "source",
    schemars(crate = "og_core::reflect::source::schemars")
)]
#[serde(deny_unknown_fields)]
pub struct NineSlice<R = String> {
    #[cfg_attr(feature = "source", schemars(extend("x-og-ref-kinds" = ["texture"])))]
    pub src: Adaptive<R>,
    #[cfg_attr(feature = "source", schemars(extend("x-og-unit" = "px")))]
    pub slice: Adaptive<Insets>,
    /// Unset draws at the slice's own widths, in points.
    #[cfg_attr(feature = "source", schemars(extend("x-og-unit" = "pt")))]
    pub width: Option<Adaptive<Insets>>,
    /// Unset paints the centre.
    pub fill: Option<Adaptive<bool>>,
    pub repeat: Option<Adaptive<Repeat>>,
    pub tint: Option<Adaptive<bool>>,
}

#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "source", derive(JsonSchema))]
#[cfg_attr(
    feature = "source",
    schemars(crate = "og_core::reflect::source::schemars")
)]
#[serde(rename_all = "lowercase")]
pub enum Repeat {
    Stretch,
    Tile,
}

#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "source", derive(JsonSchema))]
#[cfg_attr(
    feature = "source",
    schemars(crate = "og_core::reflect::source::schemars")
)]
#[serde(untagged)]
pub enum Range {
    Fixed(f32),
    Between([f32; 2]),
}

#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "source", derive(JsonSchema))]
#[cfg_attr(
    feature = "source",
    schemars(crate = "og_core::reflect::source::schemars")
)]
#[serde(deny_unknown_fields)]
#[serde(bound(deserialize = "R: Deserialize<'de>"))]
pub struct Sound<R = String> {
    /// Empty clips silence an underlying layer only when volume is zero.
    #[serde(default)]
    #[cfg_attr(feature = "source", schemars(extend("x-og-ref-kinds" = ["sound"])))]
    pub clips: Vec<R>,
    /// Gain or [low, high]; unset is 1.
    pub volume: Option<Range>,
    /// Playback rate or [low, high]; unset is 1.
    pub pitch: Option<Range>,
}

/// The same interaction event vocabulary for global and slot-local values.
#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "source", derive(JsonSchema))]
#[cfg_attr(
    feature = "source",
    schemars(crate = "og_core::reflect::source::schemars")
)]
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

#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "source", derive(JsonSchema))]
#[cfg_attr(
    feature = "source",
    schemars(crate = "og_core::reflect::source::schemars")
)]
#[serde(untagged)]
pub enum SoundBinding<R = String> {
    Global(Sound<R>),
    Scoped(Box<Events<Sound<R>>>),
}

#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "source", derive(JsonSchema))]
#[cfg_attr(
    feature = "source",
    schemars(crate = "og_core::reflect::source::schemars")
)]
#[serde(untagged)]
pub enum HapticBinding {
    Global(Haptic),
    Scoped(Events<Haptic>),
}

#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "source", derive(JsonSchema))]
#[cfg_attr(
    feature = "source",
    schemars(crate = "og_core::reflect::source::schemars")
)]
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

impl super::Layer {
    /// Build a native layer from checked fields. References are resolved by
    /// the caller, never guessed relative to a directory or a sibling file.
    pub fn from_input<R>(input: PackInput<R>, id: impl Fn(&R) -> String) -> Result<Self, String> {
        use super::{Layer, Scheme as S};
        let mut layer = Layer {
            scheme: input.scheme.map(|s| match s {
                Scheme::Light => S::Light,
                Scheme::Dark => S::Dark,
            }),
            fonts: input.fonts.iter().map(&id).collect(),
            icons: input
                .icons
                .iter()
                .map(|(k, r)| (k.clone(), id(r)))
                .collect(),
            icon_set: input.icon_set.as_ref().map(&id),
            variants: input.variants,
            ..Layer::default()
        };
        for (key, class) in &mut layer.variants {
            if !super::layer::variant_key(key) {
                return Err(format!("variants.{key}: expected Component.slot[.variant]"));
            }
            *class = class.trim().into();
        }
        if let Some(seeds) = input.seeds {
            for (name, value) in [
                ("primary", seeds.primary),
                ("accent", seeds.accent),
                ("neutral", seeds.neutral),
                ("danger", seeds.danger),
                ("warn", seeds.warn),
                ("success", seeds.success),
                ("info", seeds.info),
                ("team", seeds.team),
                ("hud", seeds.hud),
            ] {
                if let Some(value) = value {
                    layer
                        .seeds
                        .push((name.into(), S::ALL.map(|s| value.at(s).clone())));
                }
            }
        }
        for (input, output) in [
            (input.theme, &mut layer.theme),
            (input.contrast, &mut layer.contrast),
        ] {
            if let Some(input) = input {
                // NSH consumes structured values. Omitted typed sections are
                // absent, not null overrides of an underlying theme.
                let mut value = serde_json::to_value(input).map_err(|e| e.to_string())?;
                value
                    .as_object_mut()
                    .expect("typed theme")
                    .retain(|_, v| !v.is_null());
                for (out, scheme) in output.iter_mut().zip(S::ALL) {
                    *out = super::layer::split(&value, scheme, "theme")?
                        .as_object()
                        .expect("typed theme")
                        .clone();
                }
            }
        }
        for (name, image) in input.images {
            if !super::layer::kebab(&name) {
                return Err(format!("images.{name}: expected a kebab-case name"));
            }
            for (out, scheme) in layer.images.iter_mut().zip(S::ALL) {
                let image = image.at(scheme);
                let insets = |v: &Insets| -> Result<[f32; 4], String> {
                    let values = match v {
                        Insets::All(n) => [n.0; 4],
                        Insets::Sides(a) => a.each_ref().map(|n| n.0),
                    };
                    if values.iter().any(|n| !n.is_finite() || *n < 0.0) {
                        return Err(format!("images.{name}: negative or nonfinite insets"));
                    }
                    Ok(values)
                };
                out.insert(
                    name.clone(),
                    super::NineSlice {
                        src: id(image.src.at(scheme)),
                        slice: insets(image.slice.at(scheme))?,
                        width: image
                            .width
                            .as_ref()
                            .map(|v| insets(v.at(scheme)))
                            .transpose()?,
                        fill: image.fill.as_ref().is_none_or(|v| *v.at(scheme)),
                        tile: image
                            .repeat
                            .as_ref()
                            .is_some_and(|v| matches!(v.at(scheme), Repeat::Tile)),
                        tint: image.tint.as_ref().is_some_and(|v| *v.at(scheme)),
                    },
                );
            }
        }
        let sound = |s: Sound<R>| -> Result<super::Sound, String> {
            let range = |v| match v {
                None => [1.0; 2],
                Some(Range::Fixed(n)) => [n; 2],
                Some(Range::Between(r)) => r,
            };
            let volume = range(s.volume);
            if s.clips.is_empty() && volume != [0.0; 2] {
                return Err("clips: nonempty sound assets, or volume 0 for silence".into());
            }
            Ok(super::Sound {
                clips: s.clips.iter().map(&id).collect(),
                volume,
                pitch: range(s.pitch),
            })
        };
        for (key, value) in input.sounds {
            match value {
                SoundBinding::Global(value) if super::SOUND_EVENTS.contains(&key.as_str()) => {
                    layer
                        .sounds
                        .entry(String::new())
                        .or_default()
                        .insert(key, sound(value)?);
                }
                SoundBinding::Scoped(values) if super::layer::variant_key(&key) => {
                    let events = values
                        .into_events()
                        .into_iter()
                        .map(|(k, v)| Ok((k.into(), sound(v)?)))
                        .collect::<Result<_, String>>()?;
                    layer.sounds.insert(key, events);
                }
                _ => {
                    return Err(format!(
                        "sounds.{key}: expected an event or Component.slot[.variant]"
                    ));
                }
            }
        }
        let haptic = |v: Haptic| {
            serde_json::to_value(v)
                .expect("haptic name")
                .as_str()
                .expect("haptic name")
                .to_owned()
        };
        for (key, value) in input.haptics {
            match value {
                HapticBinding::Global(value) if super::SOUND_EVENTS.contains(&key.as_str()) => {
                    layer
                        .haptics
                        .entry(String::new())
                        .or_default()
                        .insert(key, haptic(value));
                }
                HapticBinding::Scoped(values) if super::layer::variant_key(&key) => {
                    layer.haptics.insert(
                        key,
                        values
                            .into_events()
                            .into_iter()
                            .map(|(k, v)| (k.into(), haptic(v)))
                            .collect(),
                    );
                }
                _ => {
                    return Err(format!(
                        "haptics.{key}: expected an event or Component.slot[.variant]"
                    ));
                }
            }
        }
        Ok(layer)
    }
}

impl<T> Events<T> {
    fn into_events(self) -> impl IntoIterator<Item = (&'static str, T)> {
        [
            ("press", self.press),
            ("hover", self.hover),
            ("select", self.select),
            ("confirm", self.confirm),
            ("cancel", self.cancel),
            ("back", self.back),
            ("open", self.open),
            ("close", self.close),
            ("error", self.error),
            ("reward", self.reward),
            ("toggle", self.toggle),
            ("tab", self.tab),
            ("levelup", self.levelup),
            ("purchase", self.purchase),
            ("countdown", self.countdown),
            ("tick", self.tick),
            ("step", self.step),
            ("swipe", self.swipe),
            ("notify", self.notify),
            ("unlock", self.unlock),
        ]
        .into_iter()
        .filter_map(|(k, v)| v.map(|v| (k, v)))
    }
}
