//! Cooked look values, independent of the author language and text dumps.
use super::{LookBlock, LookFile};
use bincode::Options;
use og_core::ogt::{Number, Value};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
enum Scalar {
    Number(f64),
    Word(String),
    String(String),
}

#[derive(Serialize, Deserialize)]
struct Block {
    resource: String,
    fields: Vec<(String, Vec<Scalar>)>,
}

#[derive(Serialize, Deserialize)]
struct Look {
    name: Option<String>,
    extends: Vec<String>,
    mood: Option<String>,
    variant: Option<String>,
    ui: Option<String>,
    ui_seeds: Vec<(String, String)>,
    antialias: Option<String>,
    blocks: Vec<Block>,
}

impl LookFile {
    /// Native values only; source locations belong to checked author inputs.
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        let look = Look {
            name: self.name.clone(),
            extends: self.extends.clone(),
            mood: self.mood.clone(),
            variant: self.variant.clone(),
            ui: self.ui.clone(),
            ui_seeds: self.ui_seeds.clone(),
            antialias: self.antialias.clone(),
            blocks: self
                .blocks
                .iter()
                .map(|b| Block {
                    resource: b.resource.clone(),
                    fields: b
                        .fields
                        .iter()
                        .map(|(name, values, _)| {
                            (
                                name.clone(),
                                values
                                    .iter()
                                    .map(|v| match v {
                                        Value::Number(n) => Scalar::Number(n.as_f64()),
                                        Value::Word(s) => Scalar::Word(s.clone()),
                                        Value::String(s) => Scalar::String(s.clone()),
                                    })
                                    .collect(),
                            )
                        })
                        .collect(),
                })
                .collect(),
        };
        bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .serialize(&look)
            .map_err(|e| format!("native look: {e}"))
    }

    /// Bounded by the enclosing section, with no text or legacy fallback.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let look: Look = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .with_limit(bytes.len() as u64)
            .reject_trailing_bytes()
            .deserialize(bytes)
            .map_err(|e| format!("native look: {e}"))?;
        let blocks = look
            .blocks
            .into_iter()
            .map(|b| {
                Ok(LookBlock {
                    resource: b.resource,
                    fields: b
                        .fields
                        .into_iter()
                        .map(|(name, values)| {
                            Ok((
                                name,
                                values
                                    .into_iter()
                                    .map(|v| match v {
                                        Scalar::Number(n) => Number::from_f64(n)
                                            .map(Value::Number)
                                            .map_err(|e| e.to_string()),
                                        Scalar::Word(s) => Ok(Value::Word(s)),
                                        Scalar::String(s) => Ok(Value::String(s)),
                                    })
                                    .collect::<Result<Vec<_>, String>>()?,
                                0,
                            ))
                        })
                        .collect::<Result<Vec<_>, String>>()?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let out = Self {
            name: look.name,
            extends: look.extends,
            mood: look.mood,
            variant: look.variant,
            ui: look.ui,
            ui_seeds: look.ui_seeds,
            antialias: look.antialias,
            blocks,
        };
        let mut resources = std::collections::BTreeSet::new();
        for block in &out.blocks {
            if !super::LOOK_RESOURCES.contains(&block.resource.as_str())
                || !resources.insert(&block.resource)
            {
                return Err("native look: unknown or duplicate resource".into());
            }
            let mut fields = std::collections::BTreeSet::new();
            if block
                .fields
                .iter()
                .any(|(name, _, _)| name.is_empty() || !fields.insert(name))
            {
                return Err("native look: empty or duplicate field".into());
            }
        }
        if out
            .mood
            .as_ref()
            .is_some_and(|m| !super::MOODS.contains(&m.as_str()))
            || out
                .antialias
                .as_ref()
                .is_some_and(|a| !super::ANTIALIAS.contains(&a.as_str()))
        {
            return Err("native look: invalid mood or antialias".into());
        }
        let mut seeds = std::collections::BTreeSet::new();
        if out.ui_seeds.iter().any(|(name, hex)| {
            !super::UI_SEEDS.contains(&name.as_str())
                || !seeds.insert(name)
                || !hex.starts_with('#')
                || super::hex_to_linear(hex).is_none()
        }) {
            return Err("native look: invalid UI seed".into());
        }
        Ok(out)
    }
}
