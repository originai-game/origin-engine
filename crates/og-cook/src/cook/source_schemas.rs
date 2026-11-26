//! Structural content inputs, not the legacy extension/planning registry.
//! Only kinds with real schemas are registered. Project/module declarations
//! and runtime records are not content factories.

use og_core::reflect::source::{AuthorSchema, schemars::JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::OnceLock;

pub mod media;
pub mod procedural;
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
