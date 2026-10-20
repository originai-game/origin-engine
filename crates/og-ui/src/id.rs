//! Stable element ids and paths.
//!
//! ```text
//! id    := name | name '#' index      ui_id("Start"), ui_idi("Slot", 3) → "Slot#3"
//! path  := id ('/' id)*                "Inventory/Slot#3": nearest ancestors first to last
//! name  := any text without '/'        (a '/' is replaced by '_' with a frame warning)
//! ```
//!
//! An element without an id gets `<kind>#<n>`, `n` counting the anonymous
//! elements of that kind under the same parent (`row#0`, `label#2`), so an
//! unchanged layout keeps its anonymous ids too. Siblings must have distinct
//! ids; a repeat is renamed `<id>~<n>` and reported in `UiFrame::warnings`.

use og_core::hash::StableHasher;

/// An element id: a name, optionally indexed (`Slot#3`). The empty id asks
/// for an anonymous one.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id(String);

impl Id {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<&str> for Id {
    fn from(name: &str) -> Id {
        Id(name.to_owned())
    }
}

impl From<String> for Id {
    fn from(name: String) -> Id {
        Id(name)
    }
}

impl From<&String> for Id {
    fn from(name: &String) -> Id {
        Id(name.clone())
    }
}

impl From<&Id> for Id {
    fn from(id: &Id) -> Id {
        id.clone()
    }
}

/// The id `name`.
pub fn ui_id(name: &str) -> Id {
    Id(name.to_owned())
}

/// The id `name#index`: one of a repeated set (list rows, slots).
pub fn ui_idi(name: &str, index: u32) -> Id {
    Id(format!("{name}#{index}"))
}

/// The state key of an element: its parent's key and its own id.
pub(crate) fn child_key(parent: u64, id: &str) -> u64 {
    let mut h = StableHasher::new();
    h.write_u64(parent);
    h.write_str(id);
    // 0 means "no element" in the interaction state.
    h.finish().max(1)
}

/// The key of the element at a full path (`Settings/Tabs/Audio`, as
/// `og.ui.a11y()` and `og.ui.focus` name it): the path's ids hashed from
/// the root down, as the frame keyed them; a layer is a child of the root.
pub(crate) fn key_of_path(path: &str) -> u64 {
    path.split('/').fold(1, child_key)
}
