//! Checked TS export settings shared by the CLI and Node build tools.
use serde_json::Value;
use std::path::Path;

/// Asset and content URL bases: HTTPS, or HTTP on a local test server.
pub fn asset_base(url: &str) -> Result<String, String> {
    let (scheme, rest) = url.split_once("://").ok_or_else(|| {
        format!(
            "is \"{url}\"; give an absolute URL, such as \"https://cdn.example.com/my-game/1.0\""
        )
    })?;
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    let name = host.rsplit_once(':').map_or(host, |(h, _)| h);
    let local = matches!(name, "localhost" | "127.0.0.1" | "[::1]");
    if !(scheme == "https" || scheme == "http" && local) {
        return Err(format!(
            "is \"{url}\"; files from another origin are fetched over https (http only on localhost)"
        ));
    }
    if host.is_empty() || host.contains('@') || rest.contains(['?', '#']) {
        return Err(format!(
            "is \"{url}\"; give the base directory, without credentials, query or fragment"
        ));
    }
    Ok(url.trim_end_matches('/').to_string())
}

/// Bootstrap has checked the vocabulary/types. Validate cross-field export
/// rules here, then omit cook-only settings and absent values from host JSON.
pub fn normalize(input: &Value) -> Result<Value, String> {
    let mut publish = input.as_object().cloned().unwrap_or_default();
    publish.retain(|key, value| {
        !value.is_null() && !matches!(key.as_str(), "out" | "keep" | "colorTextures")
    });
    for key in ["assets", "content"] {
        if let Some(value) = publish.get_mut(key) {
            *value = Value::String(
                asset_base(value.as_str().expect("checked URL string"))
                    .map_err(|why| format!("project.ts publish.{key} {why}"))?,
            );
        }
    }
    let mut ratios = std::collections::BTreeMap::new();
    for key in ["aspect", "aspectMin", "aspectMax"] {
        if let Some(value) = publish.get(key) {
            let ratio = value
                .as_str()
                .and_then(|s| s.split_once(':'))
                .and_then(|(w, h)| {
                    let (w, h) = (w.parse::<f64>().ok()?, h.parse::<f64>().ok()?);
                    (w.is_finite() && h.is_finite() && w > 0.0 && h > 0.0).then_some(w / h)
                })
                .ok_or_else(|| {
                    format!("project.ts publish.{key}: write positive width:height, such as 16:9")
                })?;
            ratios.insert(key, ratio);
        }
    }
    if publish.get("fit").and_then(Value::as_str) == Some("letterbox") {
        if !ratios.contains_key("aspect") {
            return Err("project.ts publish.fit is letterbox but publish.aspect is absent".into());
        }
        if ratios.contains_key("aspectMin") || ratios.contains_key("aspectMax") {
            return Err(
                "project.ts publish.aspectMin/aspectMax bound fit fill, not letterbox".into(),
            );
        }
    }
    if let (Some(lo), Some(hi)) = (ratios.get("aspectMin"), ratios.get("aspectMax"))
        && lo > hi
    {
        return Err("project.ts publish.aspectMin is wider than aspectMax".into());
    }
    Ok(Value::Object(publish))
}

/// The same checked snapshot supplies web build settings and runtime startup.
pub fn settings(root: &Path) -> Result<Value, String> {
    crate::project::snapshot(root)
}
