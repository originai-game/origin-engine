//! Native data-table rows and stable row identities. Author syntax belongs to
//! the tooling schema; the runtime never parses author text.
use std::collections::BTreeMap;

pub const DEFAULT_LOCALE: &str = "en";

#[derive(Clone, Debug)]
pub struct Table {
    pub schema: crate::Schema,
    pub locale: String,
    pub rows: Vec<Row>,
    pub strings: BTreeMap<String, String>,
    /// Row index -> typed asset relocations. Numeric row handles are packed.
    pub bindings: BTreeMap<usize, serde_json::Map<String, serde_json::Value>>,
}

#[derive(Clone, Debug)]
pub struct Row {
    pub id: String,
    pub bytes: Vec<u8>,
}

/// FNV-1a over canonical table ID, a zero byte and row ID. Low 52 bits are
/// exact as a JavaScript number; zero is reserved for no row.
pub fn row_handle(table: &str, id: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in table.bytes().chain(std::iter::once(0)).chain(id.bytes()) {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    match h & ((1 << 52) - 1) {
        0 => 1,
        h => h,
    }
}
