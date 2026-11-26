//! Authored scalar/vector lattices: x fastest, then y, then z, with
//! interleaved channels. No normalization, resampling or image encoding.
use crate::grammar::{Line, find};
use og_core::ogt::{Document, Entry, Number};
use serde::{Deserialize, Serialize};

pub const LINES: &[Line] = &[
    Line::new(
        "dimensions",
        "<x> <y> <z>",
        "required",
        "Positive sample counts; x varies fastest, then y, then z.",
        "",
    ),
    Line::new(
        "channels",
        "<1..4>",
        "required",
        "Interleaved scalar channels per cell; the author defines their units and meaning.",
        "",
    ),
    Line::new(
        "origin",
        "<x> <y> <z>",
        "0 0 0",
        "Position of the first lattice sample in authored coordinate units.",
        "",
    ),
    Line::new(
        "spacing",
        "<x> <y> <z>",
        "1 1 1",
        "Positive distance between samples on each axis; no implicit resampling.",
        "",
    ),
    Line::new(
        "values",
        "<float> ...",
        "required",
        "Finite f32 samples, dimensions product times channels; no normalization or quantization.",
        "",
    ),
];

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

    pub fn parse(text: &str) -> Result<Self, String> {
        let doc = Document::parse(text).map_err(|e| e.to_string())?;
        if doc.ty != "field" || doc.version != 1 {
            return Err("a field is type field version 1".into());
        }
        let mut object = serde_json::Map::new();
        for e in &doc.body.entries {
            if find(LINES, &e.key, &[]).is_none()
                || e.block.is_some()
                || object.contains_key(&e.key)
            {
                return Err(format!(
                    "line {}: unknown, duplicate or non-scalar field '{}'",
                    e.line, e.key
                ));
            }
            let values = e
                .values
                .iter()
                .map(|v| {
                    let n = v
                        .as_number()
                        .ok_or_else(|| format!("line {}: field values are numbers", e.line))?;
                    if matches!(e.key.as_str(), "dimensions" | "channels") {
                        n.as_u64()
                            .map(serde_json::Value::from)
                            .map_err(|e| e.to_string())
                    } else {
                        serde_json::to_value(n.as_f64()).map_err(|e| e.to_string())
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            let value = if e.key == "channels" {
                if values.len() != 1 {
                    return Err("channels takes one integer".into());
                }
                values.into_iter().next().unwrap()
            } else {
                serde_json::Value::Array(values)
            };
            object.insert(e.key.clone(), value);
        }
        let data: Self =
            serde_json::from_value(serde_json::Value::Object(object)).map_err(|e| e.to_string())?;
        data.validate()?;
        Ok(data)
    }

    pub fn to_text(&self) -> Result<String, String> {
        self.validate()?;
        let mut doc = Document::new("field", 1);
        let mut dimensions = Entry::new("dimensions");
        dimensions
            .values
            .extend(self.dimensions.map(|v| Number::from(v).into()));
        doc.body.push(dimensions);
        doc.body
            .push(Entry::new("channels").with(Number::from(self.channels)));
        for (key, values) in [
            ("origin", self.origin.as_slice()),
            ("spacing", self.spacing.as_slice()),
            ("values", self.values.as_slice()),
        ] {
            let mut entry = Entry::new(key);
            for &v in values {
                entry.values.push(
                    Number::from_f64(v as f64)
                        .map_err(|e| e.to_string())?
                        .into(),
                );
            }
            doc.body.push(entry);
        }
        doc.to_text().map_err(|e| e.to_string())
    }
}
