//! Small readers over `og_core::ogt::Value` for the IDL loader, validator and
//! generators. `og-core` owns the `.ogt` grammar; `true` and `false` are words
//! there, read with `as_bool()`.

use og_core::ogt::Value;

/// How a value is shown in diagnostics and generated text: strings quoted,
/// numbers as written, words bare.
pub(crate) fn show(v: &Value) -> String {
    match v {
        Value::String(s) => format!("{s:?}"),
        Value::Number(n) => n.text().to_string(),
        Value::Word(w) => w.clone(),
    }
}

pub(crate) fn as_f64(v: &Value) -> Option<f64> {
    v.as_number().map(|n| n.as_f64())
}

pub(crate) fn is_number(v: &Value) -> bool {
    v.as_number().is_some()
}
