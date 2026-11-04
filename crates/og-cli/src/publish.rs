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
//!   aspect_min "16:10"      # fill: the narrowest aspect drawn (taller screens get bars)
//!   aspect_max "21:9"       # fill: the widest aspect drawn (wider screens get bars)
//!   orientation any         # any | landscape | portrait
//!   dpr_cap 2               # 0.5–4: the highest device pixel ratio drawn
//!   fullscreen off          # off | gesture: ask on the first gesture
//!   gate tap                # tap | none: the ready-to-interact gate
//!   made_with true          # the "Made with Origin Engine" badge
//!   icon "assets/icon.png"  # a square PNG ≥ 192 px; absent: the brand tile
//!   short_name "Runner"     # the installed app's name
//!   color_textures etc1s    # etc1s | uastc: over `cook { color_textures }` for this export's cook
//!   telemetry on            # on | off: field telemetry (crash details, frame times, load stages)
//!   pipelines sync          # sync | async: the web frame pipelines with createRenderPipelineAsync
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
            // The export's cook reads it (`cook::texture_options`).
            "color_textures" => {
                word(e, &file, &["etc1s", "uastc"])?;
                continue;
            }
            "probe" => json!(word(e, &file, &["player", "all"])?),
            "isolation" => json!(word(e, &file, &["required", "preferred", "off"])?),
            "fit" => json!(word(e, &file, &["fill", "letterbox"])?),
            "orientation" => json!(word(e, &file, &["any", "landscape", "portrait"])?),
            "fullscreen" => json!(word(e, &file, &["off", "gesture"])?),
            "gate" => json!(word(e, &file, &["tap", "none"])?),
            "telemetry" => json!(word(e, &file, &["on", "off"])?),
            "pipelines" => json!(word(e, &file, &["sync", "async"])?),
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
            "aspect" | "aspect_min" | "aspect_max" => {
                let a = text(e, &file)?;
                let ok = a.split_once(':').is_some_and(|(w, h)| {
                    let (w, h) = (w.parse::<f64>(), h.parse::<f64>());
                    matches!((w, h), (Ok(w), Ok(h)) if w > 0.0 && h > 0.0)
                });
                if !ok {
                    return Err(format!(
                        "{file}:{}: publish.{} is \"{a}\"; write width:height, such as \"16:9\"",
                        e.line, e.key
                    ));
                }
                json!(a)
            }
            "icon" => json!(text(e, &file)?),
            "short_name" => json!(text(e, &file)?),
            other => {
                return Err(format!(
                    "{file}:{}: publish has no `{other}`; it takes probe, out, isolation, fit, aspect, aspect_min, aspect_max, orientation, dpr_cap, fullscreen, gate, made_with, icon, short_name, color_textures, telemetry and pipelines",
                    e.line
                ));
            }
        };
        let key = match e.key.as_str() {
            "dpr_cap" => "dprCap",
            "aspect_min" => "aspectMin",
            "aspect_max" => "aspectMax",
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
    let ratio = |k: &str| {
        publish.get(k).and_then(Value::as_str).and_then(|a| {
            let (w, h) = a.split_once(':')?;
            Some(w.parse::<f64>().ok()? / h.parse::<f64>().ok()?)
        })
    };
    if publish.get("fit").and_then(Value::as_str) == Some("letterbox")
        && (publish.contains_key("aspectMin") || publish.contains_key("aspectMax"))
    {
        return Err(format!(
            "{file}: publish.aspect_min and aspect_max bound `fit fill`; a letterbox is always its `aspect`: remove them or use `fit fill`"
        ));
    }
    if let (Some(lo), Some(hi)) = (ratio("aspectMin"), ratio("aspectMax"))
        && lo > hi
    {
        return Err(format!(
            "{file}: publish.aspect_min is wider than aspect_max; the narrowest comes first (aspect_min \"16:10\", aspect_max \"21:9\")"
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

/// The components the 2D profile's publish module leaves out
/// (`RenderCapabilities.world3d` is false there: og-render's `profile2d`),
/// by the names scenes give them, each with what it is.
const WORLD_3D: [(&str, &str); 5] = [
    ("Terrain", "a terrain"),
    ("TerrainScatter", "scattered copies"),
    ("Water", "planar water"),
    ("WaterSurface", "planar water's height field"),
    ("Fluid3D", "a 3D fluid"),
];

/// A publish build of the 2D profile (`og build --publish`, `og run
/// --device`) refuses a project whose scenes hold the 3D world: its module
/// has none of it, so it would neither draw nor load there. `Err` names
/// every such component by scene file and line. A 3D project, and a 2D
/// one without them, passes.
pub fn refuse_3d_world(root: &Path, settings: &Value) -> Result<(), String> {
    if settings["profile"] != "2d" {
        return Ok(());
    }
    fn files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        let Ok(read) = std::fs::read_dir(dir) else {
            return;
        };
        for e in read.flatten() {
            let p = e.path();
            if p.is_dir() {
                files(&p, out);
            } else if p.extension().is_some_and(|x| x == "ogt") {
                out.push(p);
            }
        }
    }
    fn walk(b: &Block, file: &str, hits: &mut Vec<String>) {
        for e in &b.entries {
            let Some(block) = &e.block else { continue };
            if e.key == "entity" {
                walk(block, file, hits);
            } else if let Some((name, what)) = WORLD_3D.iter().find(|(n, _)| *n == e.key) {
                hits.push(format!("  {file}:{}: {name} ({what})", e.line));
            }
        }
    }
    let mut scenes = Vec::new();
    files(&root.join("assets"), &mut scenes);
    scenes.sort();
    let mut hits = Vec::new();
    for path in scenes {
        // A scene that does not parse is the cook's to report.
        let Ok(doc) = std::fs::read_to_string(&path)
            .map_err(|_| ())
            .and_then(|t| og_core::ogt::parse(&t).map_err(|_| ()))
        else {
            continue;
        };
        if doc.ty != "scene" {
            continue;
        }
        let file = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        for e in doc.body.get_all("entity") {
            if let Some(b) = &e.block {
                walk(b, &file, &mut hits);
            }
        }
    }
    if hits.is_empty() {
        return Ok(());
    }
    Err(format!(
        "the 2D profile's publish module has no 3D world (RenderCapabilities.world3d is false: terrain, planar water, scattered copies and 3D fluids are left out of it), and these scenes hold it:\n{}\nremove them from the scenes, or make the project 3D (project.ogt: profile \"3d\")",
        hits.join("\n")
    ))
}

/// `og build --publish`'s refusal of a game still in the default style
/// (docs/ui.md §12.1): the default style is origin-agent's and a
/// prototype's, and every game ships its own. A game picks one by a look
/// whose `ui` line names it (its own, or a catalogue preset's through
/// `extends`; the cooked look has the chain flattened) or by loading one
/// in its code (`og.load.style`, for `og.ui.configure({ style })`). `Err`
/// names the looks that pick none and the two ways out. Run after the cook.
pub fn refuse_default_style(root: &Path) -> Result<(), String> {
    fn loads_style(dir: &Path) -> bool {
        let Ok(read) = std::fs::read_dir(dir) else {
            return false;
        };
        read.flatten().any(|e| {
            let p = e.path();
            if p.is_dir() {
                return e.file_name() != "node_modules" && loads_style(&p);
            }
            p.extension()
                .is_some_and(|x| ["ts", "tsx", "js", "mjs"].iter().any(|k| x == *k))
                && std::fs::read_to_string(&p).is_ok_and(|t| t.contains("og.load.style("))
        })
    }
    if loads_style(&root.join("src")) {
        return Ok(());
    }
    let mut bare = Vec::new();
    if let Ok(bundle) = og_cook::runtime::Bundle::open(&root.join("cooked")) {
        for out in bundle.manifest().outputs().filter(|o| o.ty == "og.Look") {
            let look = bundle
                .read(&out.id)
                .map_err(|e| e.to_string())
                .and_then(|b| bundle.look(&out.id, &b).map_err(|e| e.to_string()))
                .map_err(|e| format!("{}: {e}", out.id))?;
            if look.ui.is_some() {
                return Ok(());
            }
            bare.push(format!(
                "  assets/{} (no `ui` line in it or its bases)",
                out.id
            ));
        }
    }
    let looks = if bare.is_empty() {
        "the project has no look and its code loads no style".to_string()
    } else {
        format!(
            "no look picks a style and the code loads none:\n{}",
            bare.join("\n")
        )
    };
    Err(format!(
        "the game still draws its UI in the default style, which is for prototypes only (docs/ui.md §12.1): {looks}\ngive the game's look a style (`ui \"styles/<name>/style.ogt\"`, `og style new`, or a built-in such as `ui \"@styles/paper/style.ogt\"`), or load one in code (og.load.style, then og.ui.configure({{ style }}))"
    ))
}

#[cfg(test)]
mod tests {
    /// `settings` of a project.ogt whose publish block is `publish`.
    fn settings_of(name: &str, publish: &str) -> Result<serde_json::Value, String> {
        let dir = std::env::temp_dir().join(format!("og-publish-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("project.ogt");
        std::fs::write(
            &file,
            format!(
                "type project\nversion 1\n---\nname \"T\"\nid \"t\"\npublish {{\n{publish}}}\n"
            ),
        )
        .unwrap();
        let out = super::settings(&file);
        std::fs::remove_dir_all(&dir).ok();
        out
    }

    #[test]
    fn telemetry_off_reaches_the_web_build_and_a_bad_value_names_its_line() {
        let off = settings_of("off", "  telemetry off\n").unwrap();
        assert_eq!(off["publish"]["telemetry"], "off");
        // Absent: the web build's default (`on`) applies, so nothing is written.
        let absent = settings_of("absent", "  gate tap\n").unwrap();
        assert!(absent["publish"].get("telemetry").is_none());
        let bad = settings_of("bad", "  telemetry maybe\n").unwrap_err();
        assert!(
            bad.contains(":7: publish.telemetry is `maybe`; use one of on, off"),
            "{bad}"
        );
    }

    /// `pipelines async` opts the web page into asynchronous frame pipelines;
    /// absent, the web build's default (`sync`) applies.
    #[test]
    fn async_pipelines_are_opt_in() {
        let on = settings_of("pipelines", "  pipelines async\n").unwrap();
        assert_eq!(on["publish"]["pipelines"], "async");
        assert!(
            settings_of("nopipelines", "  gate tap\n").unwrap()["publish"]
                .get("pipelines")
                .is_none()
        );
        let bad = settings_of("badpipelines", "  pipelines lazy\n").unwrap_err();
        assert!(
            bad.contains("publish.pipelines is `lazy`; use one of sync, async"),
            "{bad}"
        );
    }

    /// The aspect range a `fill` canvas is held within reaches the web
    /// build camelCased; a reversed range, a range on a letterbox and a
    /// malformed bound name their fix.
    #[test]
    fn an_aspect_range_bounds_fill_only_narrowest_first() {
        let ok = settings_of(
            "range",
            "  fit fill\n  aspect_min \"16:10\"\n  aspect_max \"21:9\"\n",
        )
        .unwrap();
        assert_eq!(ok["publish"]["aspectMin"], "16:10");
        assert_eq!(ok["publish"]["aspectMax"], "21:9");
        let reversed =
            settings_of("reversed", "  aspect_min \"21:9\"\n  aspect_max \"4:3\"\n").unwrap_err();
        assert!(
            reversed.contains("aspect_min is wider than aspect_max"),
            "{reversed}"
        );
        let boxed = settings_of(
            "boxed",
            "  fit letterbox\n  aspect \"16:9\"\n  aspect_max \"21:9\"\n",
        )
        .unwrap_err();
        assert!(boxed.contains("bound `fit fill`"), "{boxed}");
        let bad = settings_of("badmax", "  aspect_max \"wide\"\n").unwrap_err();
        assert!(bad.contains("publish.aspect_max is \"wide\""), "{bad}");
    }
}
