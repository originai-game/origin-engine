//! Locale and font-pack inputs. ICU syntax, canonical locale matching, fallback
//! cycles, cross-locale keys/arguments and font coverage remain domain checks.
use super::super::ContentRef;
use og_core::reflect::source::{InputSchema, schemars::JsonSchema};
use serde::Deserialize;
use std::{collections::BTreeMap, sync::OnceLock};

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct StringsInput {
    /// Canonicalized as a locale tag by the localization domain.
    #[schemars(length(min = 1))]
    pub locale: String,
    #[schemars(length(min = 1))]
    pub fallback: Option<String>,
    /// Reference locale for key/argument checks; omitted is false.
    pub base: Option<bool>,
    /// Omitted: direction of locale.
    pub direction: Option<Direction>,
    /// Ordered locale font fallback chain.
    #[serde(default)]
    #[schemars(extend("x-og-ref-kinds" = ["font"]))]
    pub fonts: Vec<ContentRef>,
    /// ICU-subset source messages, not an embedded author configuration.
    pub entries: BTreeMap<String, String>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Ltr,
    Rtl,
}

impl StringsInput {
    pub fn schema() -> &'static InputSchema<Self> {
        static SCHEMA: OnceLock<InputSchema<StringsInput>> = OnceLock::new();
        SCHEMA.get_or_init(|| InputSchema::new().expect("strings input schema"))
    }
}

/// Pack membership does not change another face's own import settings or
/// charset. Coverage and script/language validity are checked from actual fonts.
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct FontExtensionInput {
    pub kind: FontKind,
    pub name: String,
    #[serde(default)]
    pub languages: Vec<String>,
    /// ISO 15924 script codes (e.g. Zsye for emoji-only packs).
    #[serde(default)]
    pub scripts: Vec<String>,
    #[schemars(length(min = 1), extend("x-og-ref-kinds" = ["font"]))]
    pub faces: Vec<ContentRef>,
    /// External <language>.hyph dictionaries, consumed only when reached by
    /// hyphens:auto and shipped locales; not independent runtime font assets.
    #[serde(default)]
    #[schemars(extend("x-og-input" = "bytes"))]
    pub hyphenation: Vec<String>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum FontKind {
    Font,
}

impl FontExtensionInput {
    pub fn schema() -> &'static InputSchema<Self> {
        static SCHEMA: OnceLock<InputSchema<FontExtensionInput>> = OnceLock::new();
        SCHEMA.get_or_init(|| InputSchema::new().expect("font-extension input schema"))
    }
}
