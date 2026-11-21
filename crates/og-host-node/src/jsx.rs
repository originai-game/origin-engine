//! `og build`'s JSX compile (packages/engine/src/node/build.ts's plugin,
//! crates/og-ui-next/CONVENTIONS.md §JSX compile): a game's `.tsx`/`.jsx`
//! module to JS calling the UI renderer, by nsh-jsx.

// napi registers the exports outside `cfg(test)` only.
#![cfg_attr(test, allow(dead_code))]

use napi_derive::napi;
use serde_json::json;

/// `{ ok: true, code, map, warnings }` (the source map as JSON), or
/// `{ ok: false, reason }` with a code frame.
#[napi(js_name = "og_node_jsx")]
pub fn og_node_jsx(path: String, source: String) -> String {
    match nsh_jsx::compile(&path, &source) {
        Ok(m) => json!({ "ok": true, "code": m.code, "map": m.map, "warnings": m.warnings }),
        Err(reason) => json!({ "ok": false, "reason": reason }),
    }
    .to_string()
}
