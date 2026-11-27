//! Ordinary project identity, authored in og-project.json, never generated.
use super::source_schemas::procedural::LookBase;
use serde::Deserialize;
use serde_json::Value;
use std::{fmt, fs, path::Path};

pub const FILE: &str = "og-project.json";

#[derive(Debug)]
pub struct Error {
    pub code: &'static str,
    pub message: String,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{FILE}: {}: {}", self.code, self.message)
    }
}
impl std::error::Error for Error {}
fn invalid(message: impl Into<String>) -> Error {
    Error {
        code: "project_metadata_invalid",
        message: message.into(),
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    pub version: u32,
    pub engine: String,
    pub name: String,
    pub look: Option<LookBase>,
}

/// Concrete project pins only. Explicit engine installation also accepts the
/// historic prerelease spelling; project discovery does not need that syntax.
pub fn version(value: &str) -> bool {
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|c| c.is_ascii_digit());
    if let Some(dev) = value.strip_prefix("dev.") {
        let Some((date, n)) = dev.split_once('.') else {
            return false;
        };
        return date.len() == 8 && digits(date) && digits(n) && !n.starts_with('0');
    }
    let parts: Vec<_> = value.split('.').collect();
    parts.len() == 3 && parts.iter().all(|s| digits(s))
}

impl Metadata {
    pub fn parse(bytes: &[u8]) -> Result<Self, Error> {
        let value: Value = serde_json::from_slice(bytes).map_err(|e| invalid(e.to_string()))?;
        let object = value
            .as_object()
            .ok_or_else(|| invalid("expected an object"))?;
        if !object
            .get("engine")
            .and_then(Value::as_str)
            .is_some_and(version)
        {
            return Err(Error {
                code: "no_pin",
                message: "engine must name a concrete engine version".into(),
            });
        }
        if object.get("look").is_some_and(Value::is_null) {
            return Err(invalid("look cannot be null"));
        }
        let metadata: Self = serde_json::from_value(value).map_err(|e| invalid(e.to_string()))?;
        if metadata.version != 1 {
            return Err(invalid("version must be 1"));
        }
        if metadata.name.is_empty() {
            return Err(invalid("name must not be empty"));
        }
        if metadata.look.as_ref().is_some_and(|look| match look {
            LookBase::Preset(s) => s.is_empty(),
            LookBase::Asset(reference) => reference.id.is_empty(),
        }) {
            return Err(invalid("look must not be empty"));
        }
        Ok(metadata)
    }

    pub fn read(root: &Path) -> Result<Self, Error> {
        Self::parse(&fs::read(root.join(FILE)).map_err(|e| invalid(e.to_string()))?)
    }
}
