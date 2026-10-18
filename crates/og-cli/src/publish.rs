//! project.ogt's export settings, as `og build` hands them to the web
//! export (`packages/engine/src/node/web-build.ts`, docs/web-export.md):
//!
//! ```text
//! publish {
//!   probe player            # player | all (a --publish build serves the player members)
//!   out "dist"
//!   isolation preferred     # required | preferred | off: crossOriginIsolation in og-build.json
//!   fit fill                # fill | letterbox
//!   aspect "16:9"           # letterbox: width:height
//!   orientation any         # any | landscape | portrait
//!   dpr_cap 2               # 0.5–4: the highest device pixel ratio drawn
//!   fullscreen off          # off | gesture: ask on the first gesture
//!   gate tap                # tap | none: the ready-to-interact gate
//!   made_with true          # the "Made with Origin Engine" badge
//!   icon "assets/icon.png"  # a square PNG ≥ 192 px; absent: the brand tile
//!   short_name "Runner"     # the installed app's name
//! }
//! ```
//!
//! Every value is checked here; a bad one names its line and the fix.

use og_core::ogt::{Block, Entry};
use serde_json::{Value, json};
use std::path::Path;

fn word<'a>(e: &'a Entry, file: &str, allowed: &[&str]) -> Result<&'a str, String> {
    let w = e
        .values
        .first()
        .and_then(|v| v.as_word().or_else(|| v.as_str()))
        .unwrap_or("");
    if allowed.contains(&w) {
        Ok(w)
    } else {
        Err(format!(
            "{file}:{}: publish.{} is `{w}`; use one of {}",
            e.line,
            e.key,
            allowed.join(", ")
        ))
    }
}

fn text<'a>(e: &'a Entry, file: &str) -> Result<&'a str, String> {
    e.string(0)
        .map_err(|_| format!("{file}:{}: publish.{} takes a quoted string", e.line, e.key))
}

/// The settings JSON for `web-build.ts --settings`, from the project's manifest.
pub fn settings(manifest: &Path) -> Result<Value, String> {
    let file = manifest.display().to_string();
    let source =
        std::fs::read_to_string(manifest).map_err(|e| format!("cannot read {file}: {e}"))?;
    let doc = og_core::ogt::parse(&source).map_err(|e| format!("{file}: {}", e.reason()))?;
    let b = &doc.body;
    let s = |k: &str| b.get(k).and_then(|e| e.string(0).ok()).map(str::to_string);
    let profile = s("profile").unwrap_or_else(|| "3d".into());
    if profile != "2d" && profile != "3d" {
        return Err(format!(
            "{file}: profile is `{profile}`; use \"2d\" or \"3d\""
        ));
    }
    let empty = Block::new();
    let p = b
        .get("publish")
        .and_then(|e| e.block.as_ref())
        .unwrap_or(&empty);
    let mut publish = serde_json::Map::new();
    for e in &p.entries {
        let v = match e.key.as_str() {
            "out" => continue,
            "probe" => json!(word(e, &file, &["player", "all"])?),
            "isolation" => json!(word(e, &file, &["required", "preferred", "off"])?),
            "fit" => json!(word(e, &file, &["fill", "letterbox"])?),
            "orientation" => json!(word(e, &file, &["any", "landscape", "portrait"])?),
            "fullscreen" => json!(word(e, &file, &["off", "gesture"])?),
            "gate" => json!(word(e, &file, &["tap", "none"])?),
            "made_with" => json!(word(e, &file, &["true", "false"])? == "true"),
            "dpr_cap" => {
                let n = e
                    .f32(0)
                    .map_err(|_| format!("{file}:{}: publish.dpr_cap takes a number", e.line))?;
                if !(0.5..=4.0).contains(&n) {
                    return Err(format!(
                        "{file}:{}: publish.dpr_cap is {n}; use a number from 0.5 to 4 (2 is the default)",
                        e.line
                    ));
                }
                json!(n)
            }
            "aspect" => {
                let a = text(e, &file)?;
                let ok = a.split_once(':').is_some_and(|(w, h)| {
                    let (w, h) = (w.parse::<f64>(), h.parse::<f64>());
                    matches!((w, h), (Ok(w), Ok(h)) if w > 0.0 && h > 0.0)
                });
                if !ok {
                    return Err(format!(
                        "{file}:{}: publish.aspect is \"{a}\"; write width:height, such as \"16:9\"",
                        e.line
                    ));
                }
                json!(a)
            }
            "icon" => json!(text(e, &file)?),
            "short_name" => json!(text(e, &file)?),
            other => {
                return Err(format!(
                    "{file}:{}: publish has no `{other}`; it takes probe, out, isolation, fit, aspect, orientation, dpr_cap, fullscreen, gate, made_with, icon and short_name",
                    e.line
                ));
            }
        };
        let key = match e.key.as_str() {
            "dpr_cap" => "dprCap",
            "made_with" => "madeWith",
            "short_name" => "shortName",
            k => k,
        };
        publish.insert(key.to_string(), v);
    }
    if publish.get("fit").and_then(Value::as_str) == Some("letterbox")
        && !publish.contains_key("aspect")
    {
        return Err(format!(
            "{file}: publish.fit is letterbox but there is no publish.aspect; add `aspect \"16:9\"` (width:height) or use `fit fill`"
        ));
    }
    Ok(json!({
        "name": s("name").unwrap_or_else(|| "Origin Engine game".into()),
        "id": s("id").unwrap_or_else(|| "game".into()),
        "engine": s("engine").unwrap_or_else(|| crate::VERSION.into()),
        "profile": profile,
        "publish": publish,
    }))
}
