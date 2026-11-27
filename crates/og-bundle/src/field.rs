//! Authored scalar/vector lattices: x fastest, then y, then z, with
//! interleaved channels. No normalization, resampling or image encoding.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(
    feature = "source",
    derive(og_core::reflect::source::schemars::JsonSchema)
)]
#[cfg_attr(
    feature = "source",
    schemars(crate = "og_core::reflect::source::schemars")
)]
#[serde(deny_unknown_fields)]
pub struct FieldData {
    /// Positive sample counts; x varies fastest, then y, then z.
    #[cfg_attr(feature = "source", schemars(inner(range(min = 1))))]
    pub dimensions: [u32; 3],
    /// Interleaved scalar channels per cell; their meaning belongs to the author.
    #[cfg_attr(feature = "source", schemars(range(min = 1, max = 4)))]
    pub channels: u32,
    /// First sample position in authored coordinate units.
    #[serde(default)]
    pub origin: [f32; 3],
    /// Positive sample spacing in authored coordinate units.
    #[serde(default = "spacing")]
    #[cfg_attr(feature = "source", schemars(extend("items" = {"type":"number", "format":"float", "exclusiveMinimum":0})))]
    pub spacing: [f32; 3],
    /// Finite f32 samples; dimensions product times channels, without normalization.
    #[cfg_attr(feature = "source", schemars(length(min = 1, max = 16_777_216)))]
    pub values: Vec<f32>,
}

fn spacing() -> [f32; 3] {
    [1.0; 3]
}

impl FieldData {
    #[cfg(feature = "source")]
    pub fn schema() -> &'static og_core::reflect::source::InputSchema<Self> {
        static SCHEMA: std::sync::OnceLock<og_core::reflect::source::InputSchema<FieldData>> =
            std::sync::OnceLock::new();
        SCHEMA.get_or_init(|| og_core::reflect::source::InputSchema::new().expect("field schema"))
    }

    pub fn validate(&self) -> Result<(), String> {
        let count = self
            .dimensions
            .iter()
            .try_fold(u64::from(self.channels), |n, &d| {
                n.checked_mul(u64::from(d))
            });
        if self.dimensions.contains(&0)
            || !(1..=4).contains(&self.channels)
            || !count.is_some_and(|n| n <= 16_777_216 && n == self.values.len() as u64 && n > 0)
        {
            return Err("field needs positive dimensions, 1..4 channels, and exactly their product in values (at most 16,777,216 scalars)".into());
        }
        if self.origin.iter().any(|v| !v.is_finite())
            || self.spacing.iter().any(|v| !v.is_finite() || *v <= 0.0)
            || self.values.iter().any(|v| !v.is_finite())
        {
            return Err(
                "field origin and values must be finite f32, spacing finite and positive".into(),
            );
        }
        Ok(())
    }
}
