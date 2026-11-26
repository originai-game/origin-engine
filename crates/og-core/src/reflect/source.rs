//! Structural author input schemas, alongside packed runtime reflection.
//!
//! Derive `schemars::JsonSchema` on the same Rust input types that derive
//! `serde::Deserialize`. Serde names, defaults, enum tagging and unknown-field
//! rules then have one definition. Field constraints/docs use `#[schemars]`;
//! domain validators still own relationships such as height count = rows × columns.
//! This module is behind `source` and is not linked by a runtime-only build.

use schemars::JsonSchema;
use schemars::transform::{RecursiveTransform, Transform};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

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
    /// Match a subschema at a document JSON Pointer, retaining root references.
    /// This is structural matching, not a substitute for `check` and its decoder.
    fn matches(&self, pointer: &str, value: &Value) -> Result<bool, String>;
}

/// A compiled schema bound to its Rust decoder. Keep one per input type in a
/// registry; tools may read `document()` without knowing the concrete type.
pub struct InputSchema<T> {
    document: Value,
    validator: jsonschema::Validator,
    branches: Mutex<BTreeMap<String, Arc<jsonschema::Validator>>>,
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
        let mut schema = schemars::Schema::try_from(document).map_err(|e| e.to_string())?;
        // Schemars marks f32 as `float` without bounds; serde_json can cast a
        // finite f64 to infinity. Intersect the Rust range, retaining narrower
        // domain bounds. Visit schema positions, never default/example data.
        RecursiveTransform(|schema: &mut schemars::Schema| {
            if schema.get("format").and_then(Value::as_str) != Some("float") {
                return;
            }
            if schema
                .get("minimum")
                .is_none_or(|v| v.as_f64().is_some_and(|n| n < f64::from(f32::MIN)))
            {
                schema.insert("minimum".into(), Value::from(f32::MIN));
            }
            if schema
                .get("maximum")
                .is_none_or(|v| v.as_f64().is_some_and(|n| n > f64::from(f32::MAX)))
            {
                schema.insert("maximum".into(), Value::from(f32::MAX));
            }
        })
        .transform(&mut schema);
        let document = schema.to_value();
        let validator = jsonschema::validator_for(&document).map_err(|e| e.to_string())?;
        Ok(Self {
            document,
            validator,
            branches: Mutex::new(BTreeMap::new()),
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

    /// Match a document subschema without detaching its local references.
    pub fn matches(&self, pointer: &str, value: &Value) -> Result<bool, String> {
        if pointer.is_empty() {
            return Ok(self.validator.is_valid(value));
        }
        let validator = {
            let mut branches = self.branches.lock().expect("author schema branch cache");
            if let Some(validator) = branches.get(pointer) {
                validator.clone()
            } else {
                if !self
                    .document
                    .pointer(pointer)
                    .is_some_and(|v| v.is_object() || v.is_boolean())
                {
                    return Err(format!("not an author subschema: {pointer}"));
                }
                // Register the original document, not a detached branch: a local
                // $ref in that branch must still see the original root's $defs.
                // Compile only requested branches, never every schema position.
                const ROOT: &str = "urn:origingame:author-schema";
                let registry = jsonschema::Registry::new()
                    .add(ROOT, &self.document)
                    .and_then(|r| r.prepare())
                    .map_err(|e| e.to_string())?;
                let fragment: String = pointer
                    .bytes()
                    .map(|b| {
                        if b.is_ascii_alphanumeric()
                            || matches!(b, b'-' | b'.' | b'_' | b'~' | b'/')
                        {
                            char::from(b).to_string()
                        } else {
                            format!("%{b:02X}")
                        }
                    })
                    .collect();
                let validator = Arc::new(
                    jsonschema::options()
                        .with_registry(&registry)
                        .build(&serde_json::json!({"$ref": format!("{ROOT}#{fragment}")}))
                        .map_err(|e| e.to_string())?,
                );
                branches.insert(pointer.to_owned(), validator.clone());
                validator
            }
        };
        Ok(validator.is_valid(value))
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

    fn matches(&self, pointer: &str, value: &Value) -> Result<bool, String> {
        self.matches(pointer, value)
    }
}
