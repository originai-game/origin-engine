//! The weight bundle: one safetensors file with every number the model reads.
//!
//! Tensor names are those of og-motion-train's `scripts/export.py`:
//! `student.*` and `decoder.*` (PyTorch state-dict names), `codebook`
//! `[5,16,32]`, `stats.mean` and `stats.std` `[370]`, `capture_offsets` and
//! `standard_offsets` `[30,3,3]`. The metadata entry `config` is the JSON of
//! both architectures. Tensors are F32 or F16 on disk and F32 in memory.

use std::collections::HashMap;

use serde::Deserialize;

/// Architecture of the student pose model.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct StudentConfig {
    pub width: usize,
    pub layers: usize,
    pub heads: usize,
    pub ff: usize,
}

/// Architecture of the decoder (NVIDIA's `DoubleCondDecoder`).
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct DecoderConfig {
    pub input_emb_width: usize,
    pub output_emb_width: usize,
    pub down_t: usize,
    pub width: usize,
    pub depth: usize,
    pub dilation_growth_rate: usize,
    pub target_cond_dim: usize,
    pub external_cond_dim: usize,
    #[serde(default)]
    pub cond_fusion_last_layer: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Config {
    pub format: String,
    pub student: StudentConfig,
    pub decoder: DecoderConfig,
}

/// The bundle format this crate reads.
pub const FORMAT: &str = "og-motion/1";

#[derive(Clone, Debug)]
pub struct Tensor {
    pub shape: Vec<usize>,
    pub data: Vec<f32>,
}

#[derive(Clone, Debug)]
pub struct Bundle {
    pub config: Config,
    pub tensors: HashMap<String, Tensor>,
    /// The bundle's `attribution` metadata, carried with any redistribution.
    pub attribution: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BundleError {
    Truncated,
    Header(String),
    Dtype {
        name: String,
        dtype: String,
    },
    Missing(String),
    Shape {
        name: String,
        expected: Vec<usize>,
        found: Vec<usize>,
    },
    Format(String),
}

impl std::fmt::Display for BundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BundleError::Truncated => write!(f, "the bundle is shorter than its header says"),
            BundleError::Header(e) => write!(f, "bad safetensors header: {e}"),
            BundleError::Dtype { name, dtype } => {
                write!(f, "{name}: dtype {dtype} (F32 or F16 expected)")
            }
            BundleError::Missing(name) => write!(f, "missing tensor {name}"),
            BundleError::Shape {
                name,
                expected,
                found,
            } => {
                write!(f, "{name}: shape {found:?}, expected {expected:?}")
            }
            BundleError::Format(found) => write!(f, "bundle format {found:?}, expected {FORMAT:?}"),
        }
    }
}

impl std::error::Error for BundleError {}

#[derive(Deserialize)]
struct Entry {
    dtype: String,
    shape: Vec<usize>,
    data_offsets: [usize; 2],
}

/// IEEE half to single precision.
pub fn f16_to_f32(bits: u16) -> f32 {
    let sign = ((bits >> 15) as u32) << 31;
    let exponent = ((bits >> 10) & 0x1f) as u32;
    let mantissa = (bits & 0x3ff) as u32;
    let value = match exponent {
        0 if mantissa == 0 => sign,
        0 => {
            // Subnormal: normalise the mantissa.
            let shift = mantissa.leading_zeros() - 21;
            let mantissa = (mantissa << shift) & 0x3ff;
            sign | ((113 - shift) << 23) | (mantissa << 13)
        }
        0x1f => sign | 0x7f80_0000 | (mantissa << 13),
        _ => sign | ((exponent + 112) << 23) | (mantissa << 13),
    };
    f32::from_bits(value)
}

impl Bundle {
    /// Parses a safetensors file (F32 or F16 tensors) and its `config` metadata.
    pub fn from_safetensors(bytes: &[u8]) -> Result<Bundle, BundleError> {
        let header_len = u64::from_le_bytes(
            bytes
                .get(..8)
                .ok_or(BundleError::Truncated)?
                .try_into()
                .unwrap(),
        ) as usize;
        let header = bytes.get(8..8 + header_len).ok_or(BundleError::Truncated)?;
        let data = &bytes[8 + header_len..];
        let mut header: HashMap<String, serde_json::Value> =
            serde_json::from_slice(header).map_err(|e| BundleError::Header(e.to_string()))?;
        let metadata: HashMap<String, String> = match header.remove("__metadata__") {
            Some(m) => serde_json::from_value(m).map_err(|e| BundleError::Header(e.to_string()))?,
            None => HashMap::new(),
        };
        let config: Config = serde_json::from_str(
            metadata
                .get("config")
                .ok_or_else(|| BundleError::Header("no config metadata".into()))?,
        )
        .map_err(|e| BundleError::Header(format!("config: {e}")))?;
        if config.format != FORMAT {
            return Err(BundleError::Format(config.format));
        }
        let mut tensors = HashMap::with_capacity(header.len());
        for (name, value) in header {
            let entry: Entry = serde_json::from_value(value)
                .map_err(|e| BundleError::Header(format!("{name}: {e}")))?;
            let raw = data
                .get(entry.data_offsets[0]..entry.data_offsets[1])
                .ok_or(BundleError::Truncated)?;
            let values: Vec<f32> = match entry.dtype.as_str() {
                "F32" => raw
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .map(|c| f32::from_le_bytes(*c))
                    .collect(),
                "F16" => raw
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|c| f16_to_f32(u16::from_le_bytes(*c)))
                    .collect(),
                other => {
                    return Err(BundleError::Dtype {
                        name,
                        dtype: other.into(),
                    });
                }
            };
            if values.len() != entry.shape.iter().product::<usize>() {
                return Err(BundleError::Truncated);
            }
            tensors.insert(
                name,
                Tensor {
                    shape: entry.shape,
                    data: values,
                },
            );
        }
        Ok(Bundle {
            config,
            tensors,
            attribution: metadata.get("attribution").cloned(),
        })
    }

    /// The tensor `name`, checked against `shape`.
    pub fn get(&self, name: &str, shape: &[usize]) -> Result<&[f32], BundleError> {
        let tensor = self
            .tensors
            .get(name)
            .ok_or_else(|| BundleError::Missing(name.into()))?;
        if tensor.shape != shape {
            return Err(BundleError::Shape {
                name: name.into(),
                expected: shape.to_vec(),
                found: tensor.shape.clone(),
            });
        }
        Ok(&tensor.data)
    }

    /// Parameters of the two networks (buffers such as statistics excluded).
    pub fn parameters(&self) -> usize {
        self.tensors
            .iter()
            .filter(|(k, _)| {
                (k.starts_with("student.") || k.starts_with("decoder."))
                    && !matches!(
                        k.as_str(),
                        "student.mean" | "student.std" | "student.attn_mask"
                    )
            })
            .map(|(_, t)| t.data.len())
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::f16_to_f32;

    #[test]
    fn half_precision_values() {
        assert_eq!(f16_to_f32(0x3c00), 1.0);
        assert_eq!(f16_to_f32(0xc000), -2.0);
        assert_eq!(f16_to_f32(0x3555), 0.333_251_95);
        assert_eq!(f16_to_f32(0x0001), 5.960_464_5e-8); // smallest subnormal
        assert_eq!(f16_to_f32(0x7bff), 65504.0);
        assert!(f16_to_f32(0x7c00).is_infinite());
        assert_eq!(f16_to_f32(0x8000).to_bits(), (-0.0f32).to_bits());
    }
}
