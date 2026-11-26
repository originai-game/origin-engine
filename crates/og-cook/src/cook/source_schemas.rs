//! Structural content inputs, not the legacy extension/planning registry.
//! Only kinds with real schemas are registered. Project/module declarations
//! and runtime records are not content factories.

use og_core::reflect::source::{AuthorSchema, Diagnostic, schemars::JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::OnceLock;

pub mod media;
pub mod procedural;
pub mod project;
pub mod reflected;
pub mod world;

/// A content identity, never a runtime handle or an instruction to evaluate
/// another entry. The project graph validates names, existence, subresources,
/// module access and target kinds. A field declares its allowed kinds with
/// `x-og-ref-kinds` schema metadata, not a second kind inside this token.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct ContentRef {
    #[serde(rename = "$ogRef")]
    #[schemars(length(min = 1))]
    pub id: String,
}

/// A domain-owned input decoder. Use `schema: || Input::schema()` to erase
/// the type while keeping the domain's typed accessor for actual cooking.
pub struct Kind {
    pub name: &'static str,
    pub schema: fn() -> &'static dyn AuthorSchema,
}

fn registry() -> &'static BTreeMap<&'static str, &'static Kind> {
    static REGISTRY: OnceLock<BTreeMap<&'static str, &'static Kind>> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut kinds = BTreeMap::new();
        for kind in procedural::KINDS
            .iter()
            .chain(world::KINDS)
            .chain(media::KINDS)
        {
            assert!(
                !kind.name.is_empty()
                    && kind.name.split('-').all(|part| {
                        !part.is_empty()
                            && part
                                .bytes()
                                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
                    }),
                "invalid author kind: {}",
                kind.name
            );
            assert!(
                kinds.insert(kind.name, kind).is_none(),
                "duplicate author kind: {}",
                kind.name
            );
        }
        kinds
    })
}

/// Exact kind lookup; unknown and not-yet-covered kinds return `None`.
pub fn get(kind: &str) -> Option<&'static Kind> {
    registry().get(kind).copied()
}

/// Covered content kinds in deterministic name order, without aliases.
pub fn all() -> impl ExactSizeIterator<Item = &'static Kind> {
    registry().values().copied()
}

/// Read a kind's document without knowing its Rust input type.
pub fn schema(kind: &str) -> Option<&'static Value> {
    get(kind).map(|kind| (kind.schema)().document())
}

/// A single default-exported content declaration. The envelope is shared by
/// every factory; schema and domain validation happen before any cooking.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct File {
    #[serde(rename = "$ogFile")]
    pub kind: String,
    pub value: Value,
}

impl File {
    /// Contextual scene/prefab/table schemas belong to the current project,
    /// never to a global registry. Their caller supplies the bound decoder.
    /// Diagnostic paths remain relative to `value`, for original-source mapping.
    pub fn check(&self, contextual: Option<&dyn AuthorSchema>) -> Result<(), Diagnostic> {
        let schema = match get(&self.kind) {
            Some(kind) => (kind.schema)(),
            None if matches!(self.kind.as_str(), "scene" | "prefab" | "table") => contextual
                .ok_or_else(|| Diagnostic {
                    path: String::new(),
                    message: format!("{} needs the project's record schemas", self.kind),
                })?,
            None => {
                return Err(Diagnostic {
                    path: String::new(),
                    message: format!("unknown author file kind {}", self.kind),
                });
            }
        };
        schema.check(&self.value)
    }
}

/// Engine-owned input shapes. Project generation augments the separate
/// ContextualFileInputs interface with its complete role-bound world schemas.
/// The same emitter serves repository drift checking and project-local types.
pub fn declarations() -> Result<String, String> {
    let mut out = String::from(
        "// Generated from the cook's Rust input schemas by `og api gen`. Do not edit.\n",
    );
    for (name, schema) in [
        ("ProjectInput", project::ProjectInput::schema().document()),
        ("ModuleInput", project::ModuleInput::schema().document()),
    ] {
        out.push_str(&og_core::reflect::source::typescript(schema, name)?);
    }
    for kind in all() {
        let name = type_name(kind.name);
        out.push_str(&og_core::reflect::source::typescript(
            (kind.schema)().document(),
            &name,
        )?);
    }
    out.push_str("export interface FileInputs {\n");
    for kind in all() {
        out.push_str(&format!(
            "readonly {:?}: {};\n",
            kind.name,
            type_name(kind.name)
        ));
    }
    out.push_str("}\n");
    Ok(out)
}

fn type_name(kind: &str) -> String {
    let mut name = String::from("Input_");
    name.push_str(&kind.replace('-', "_"));
    name
}
