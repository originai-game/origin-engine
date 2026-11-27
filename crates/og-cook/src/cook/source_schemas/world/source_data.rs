//! Raw source inputs, before file resolution and source-dependent defaults.
//! Existing PCM, field and icon input records are reused unchanged. The other
//! legacy parsers return normalized IR (numeric texture kinds, resolved paths,
//! source-dependent sky flags), which is deliberately not an author interface.
pub use crate::cook::formats::{audio::PcmInput, icons::IconSet};
pub use crate::cook::systems::environment::EnvironmentSource;
pub use og_bundle::field::FieldData;

use og_core::reflect::source::{InputSchema, schemars::JsonSchema};
use serde::Deserialize;
use std::sync::OnceLock;

/// An ordered array of equally-sized external images. File resolution,
/// dimensions and encoder compatibility remain texture-array domain checks.
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct TextureArrayInput {
    #[serde(default)]
    pub kind: TextureKind,
    /// Explicit canonical paths relative to the owning module (e.g.
    /// assets/tiles/grass.png), without sibling or asset-root fallback.
    #[schemars(length(min = 1, max = 256), inner(length(min = 1)), extend("x-og-input" = "bytes"))]
    pub layers: Vec<String>,
}

#[derive(Default, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum TextureKind {
    #[default]
    Color,
    Normal,
    Linear,
    Single,
    Pixel,
}

/// Exactly one SVG source. SVG syntax/element support belongs to the vector
/// domain validator. No embedded OGT or untyped configuration is accepted.
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged, deny_unknown_fields)]
pub enum VectorInput {
    File {
        /// External SVG path, resolved beside this source file.
        #[schemars(length(min = 1), extend("x-og-input" = "text"))]
        source: String,
        /// Positive flattening error in SVG view-box units; omitted uses the
        /// longer view-box side divided by 2000.
        #[schemars(extend("exclusiveMinimum" = 0, "x-og-unit" = "viewBox"))]
        tolerance: Option<f32>,
    },
    Inline {
        #[schemars(length(min = 1))]
        svg: String,
        /// Positive flattening error in SVG view-box units.
        #[schemars(extend("exclusiveMinimum" = 0, "x-og-unit" = "viewBox"))]
        tolerance: Option<f32>,
    },
}

/// Sky source and explicit overrides. Omission is retained, not replaced by
/// normalized defaults: e.g. LDR extracts the sun while HDR keeps it. Source
/// compatibility, prefilter/sample conflicts, hemisphere/ground relationships,
/// image dimensions and power-of-two size remain environment domain checks.
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EnvironmentInput {
    pub source: EnvironmentSource,
    /// Face side in texels. Omitted: 128, or the prefiltered cube's own size.
    #[schemars(range(min = og_render_data::ENVIRONMENT_MAP_MIN_SIZE, max = og_render_data::ENVIRONMENT_MAP_MAX_SIZE), extend("x-og-unit" = "px"))]
    pub size: Option<u32>,
    /// Radiance multiplier; omitted: 1.
    #[schemars(range(min = 0))]
    pub intensity: Option<f32>,
    /// Rotation about the vertical axis; omitted: 0 degrees.
    #[schemars(extend("x-og-unit" = "deg"))]
    pub rotation: Option<f32>,
    /// GGX samples per texel; omitted: 256. Forbidden for a prefiltered cube.
    #[schemars(range(min = 16, max = 4096))]
    pub samples: Option<u32>,
    /// Use the KTX2 cube's existing mip chain; omitted: false.
    pub prefiltered: Option<bool>,
    /// Omitted: extract for LDR images, keep for HDR and KTX2.
    pub sun: Option<Sun>,
    pub hemisphere: Option<Hemisphere>,
    /// Linear RGB ground tint, requires an upper-hemisphere equirectangular image.
    #[schemars(inner(range(min = 0, max = 1)))]
    pub ground: Option<[f32; 3]>,
    /// Inverse tone-mapping strength; omitted: 0.25, zero disables it.
    #[schemars(range(min = 0, max = 0.9))]
    pub expand: Option<f32>,
    /// Estimated LDR sun multiplier; omitted: 1.
    #[schemars(range(min = 0, max = 100))]
    pub sun_boost: Option<f32>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum Sun {
    Extract,
    Keep,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum Hemisphere {
    Upper,
}

impl TextureArrayInput {
    pub fn schema() -> &'static InputSchema<Self> {
        static SCHEMA: OnceLock<InputSchema<TextureArrayInput>> = OnceLock::new();
        SCHEMA.get_or_init(|| InputSchema::new().expect("texture array input schema"))
    }
}

impl VectorInput {
    pub fn schema() -> &'static InputSchema<Self> {
        static SCHEMA: OnceLock<InputSchema<VectorInput>> = OnceLock::new();
        SCHEMA.get_or_init(|| InputSchema::new().expect("vector input schema"))
    }
}

impl EnvironmentInput {
    pub fn schema() -> &'static InputSchema<Self> {
        static SCHEMA: OnceLock<InputSchema<EnvironmentInput>> = OnceLock::new();
        SCHEMA.get_or_init(|| InputSchema::new().expect("environment input schema"))
    }
}
