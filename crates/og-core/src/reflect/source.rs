//! Structural author input schemas, alongside packed runtime reflection.
//!
//! Derive `schemars::JsonSchema` on the same Rust input types that derive
//! `serde::Deserialize`. Serde names, defaults, enum tagging and unknown-field
//! rules then have one definition. Field constraints/docs use `#[schemars]`;
//! domain validators still own relationships such as height count = rows × columns.
//! This module is behind `source` and is not linked by a runtime-only build.

use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::marker::PhantomData;

pub use schemars;

mod typescript;
pub use typescript::typescript;

/// A value location and reason. `path` is a JSON Pointer; empty means root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub path: String,
    pub message: String,
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {}",
            if self.path.is_empty() {
                "$"
            } else {
                &self.path
            },
            self.message
        )
    }
}

impl std::error::Error for Diagnostic {}

/// A tooling schema without its concrete decoder type. Structural checking
/// includes serde decoding; domain relationships still need the domain cooker.
pub trait AuthorSchema: Sync {
    fn document(&self) -> &Value;
    fn check(&self, value: &Value) -> Result<(), Diagnostic>;
}

/// A compiled schema bound to its Rust decoder. Keep one per input type in a
/// registry; tools may read `document()` without knowing the concrete type.
pub struct InputSchema<T> {
    document: Value,
    validator: jsonschema::Validator,
    input: PhantomData<fn() -> T>,
}

impl<T: JsonSchema> InputSchema<T> {
    pub fn new() -> Result<Self, String> {
        let document = schemars::generate::SchemaSettings::draft2020_12()
            .for_deserialize()
            .into_generator()
            .into_root_schema_for::<T>()
            .to_value();
        Self::from_document(document)
    }
}

impl<T> InputSchema<T> {
    /// Bind a reflected or context-expanded schema to the same Rust decoder.
    /// Local `$defs` must be included; remote schema retrieval is disabled.
    pub fn from_document(document: Value) -> Result<Self, String> {
        let validator = jsonschema::validator_for(&document).map_err(|e| e.to_string())?;
        Ok(Self {
            document,
            validator,
            input: PhantomData,
        })
    }

    /// Draft 2020-12, including local `$defs`, docs, defaults and extensions.
    pub fn document(&self) -> &Value {
        &self.document
    }

    pub fn validate(&self, value: &Value) -> Result<(), Diagnostic> {
        self.validator.validate(value).map_err(|e| Diagnostic {
            path: e.instance_path().to_string(),
            message: e.to_string(),
        })
    }
}

impl<T: DeserializeOwned> InputSchema<T> {
    /// Schema validation cannot be bypassed by a TS assertion. Decode with
    /// serde afterwards as well: a Rust-specific constraint may be stricter.
    pub fn decode(&self, value: Value) -> Result<T, Diagnostic> {
        self.validate(&value)?;
        serde_path_to_error::deserialize(value).map_err(|e| {
            let mut path = String::new();
            for segment in e.path() {
                let key = match segment {
                    serde_path_to_error::Segment::Seq { index } => index.to_string(),
                    serde_path_to_error::Segment::Map { key }
                    | serde_path_to_error::Segment::Enum { variant: key } => key.clone(),
                    serde_path_to_error::Segment::Unknown => continue,
                };
                path.push('/');
                path.push_str(&key.replace('~', "~0").replace('/', "~1"));
            }
            Diagnostic {
                path,
                message: e.inner().to_string(),
            }
        })
    }
}

impl<T: DeserializeOwned> AuthorSchema for InputSchema<T> {
    fn document(&self) -> &Value {
        self.document()
    }

    fn check(&self, value: &Value) -> Result<(), Diagnostic> {
        self.decode(value.clone()).map(|_| ())
    }
}
