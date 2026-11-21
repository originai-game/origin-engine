//! project.ogt's export settings, as `og build` hands them to the web
//! export (`packages/engine/src/node/web-build.ts`, docs/web-export.md):
//!
//! ```text
//! publish {
//!   probe player            # player | all (a --publish build serves the player members)
//!   out "dist"
//!   keep "audio/**"         # asset roots in addition to scenes and static code references
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
//!   pipelines auto          # auto | sync | async | prewarm: which web pipelines use createRenderPipelineAsync (auto: async on a mobile GPU)
//!   loading engine          # engine | game | page: who draws the loading screen (og.loading)
//!   loading_labels {        # the splash's caption per boot stage
//!     fetch "下载中"          # fetch | transcode | prime | present
//!   }
//!   assets "https://cdn.example.com/runner/1.4.0"  # the asset base `cooked/` is served from
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

/// An asset base as the export takes it (docs/web-export.md §Asset base),
/// and a content base (guide loading §14) alike:
/// an absolute `https://` URL, or `http://` on localhost or 127.0.0.1 (a
/// test server), with no credentials, query or fragment; the trailing `/`
/// is dropped. `Err` says what is wrong, after the setting's name.
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

/// The settings JSON for `web-build.ts --settings`, from the project's manifest.
pub fn settings(manifest: &Path) -> Result<Value, String> {
    let file = manifest.display().to_string();
    let source =
        std::fs::read_to_string(manifest).map_err(|e| format!("cannot read {file}: {e}"))?;
    let doc = og_core::ogt::parse(&source).map_err(|e| format!("{file}: {}", e.reason()))?;
    og_cook::cook::assets::publish_keep(&doc)?;
    let b = &doc.body;
    let s = |k: &str| b.get(k).and_then(|e| e.string(0).ok()).map(str::to_string);
    let profile = if og_cook::look::project_2d(b).map_err(|e| format!("{file}:{e}"))? {
        "2d"
    } else {
        "3d"
    };
    let empty = Block::new();
    let p = b
        .get("publish")
        .and_then(|e| e.block.as_ref())
        .unwrap_or(&empty);
    let mut publish = serde_json::Map::new();
    for e in &p.entries {
        let v = match e.key.as_str() {
            "out" | "keep" => continue,
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
            "pipelines" => json!(word(e, &file, &["auto", "sync", "async", "prewarm"])?),
            "made_with" => json!(word(e, &file, &["true", "false"])? == "true"),
            "loading" => json!(word(e, &file, &["engine", "game", "page"])?),
            "loading_labels" => {
                let block = e.block.as_ref().ok_or_else(|| {
                    format!(
                        "{file}:{}: publish.loading_labels is a block of `<stage> \"<caption>\"` lines (fetch, transcode, prime, present)",
                        e.line
                    )
                })?;
                let mut labels = serde_json::Map::new();
                for l in &block.entries {
                    if !["fetch", "transcode", "prime", "present"].contains(&l.key.as_str()) {
                        return Err(format!(
                            "{file}:{}: publish.loading_labels has no stage `{}`; the stages are fetch, transcode, prime and present",
                            l.line, l.key
                        ));
                    }
                    let caption = l.string(0).map_err(|_| {
                        format!(
                            "{file}:{}: publish.loading_labels.{} takes the caption as a quoted string",
                            l.line, l.key
                        )
                    })?;
                    labels.insert(l.key.clone(), json!(caption));
                }
                Value::Object(labels)
            }
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
            "assets" | "content" => json!(
                asset_base(text(e, &file)?)
                    .map_err(|why| format!("{file}:{}: publish.{} {why}", e.line, e.key))?
            ),
            "short_name" => json!(text(e, &file)?),
            other => {
                return Err(format!(
                    "{file}:{}: publish has no `{other}`; it takes probe, out, isolation, fit, aspect, aspect_min, aspect_max, orientation, dpr_cap, fullscreen, gate, made_with, icon, short_name, color_textures, telemetry, pipelines, loading, loading_labels, assets and content",
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
            "loading_labels" => "loadingLabels",
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
    // The origins remote `<iframe>`s may load from: the platform sets its
    // CSP to match (docs/web-export.md §Frames).
    let frames = og_ui::doc::frames::project_origins(b).map_err(|e| format!("{file}:{e}"))?;
    Ok(json!({
        "frames": frames,
        "name": s("name").unwrap_or_else(|| "Origin Engine game".into()),
        "id": s("id").unwrap_or_else(|| "game".into()),
        "engine": s("engine").unwrap_or_else(|| crate::project::own_version().into()),
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
        "the 2D profile's publish module has no 3D world (RenderCapabilities.world3d is false: terrain, planar water, scattered copies and 3D fluids are left out of it), and these scenes hold it:\n{}\nremove them from the scenes, or make the project 3D (project.ogt: `create {{ form \"3d\" }}`)",
        hits.join("\n")
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
    fn keep_patterns_are_checked_before_export() {
        assert!(settings_of("keep", "  keep \"audio/**\" \"textures/icons/*\"\n").is_ok());
        for value in ["", "true", "\"../private\""] {
            assert!(
                settings_of("bad-keep", &format!("  keep {value}\n"))
                    .unwrap_err()
                    .contains("publish.keep")
            );
        }
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

    /// `pipelines` names how the web page makes its frame pipelines;
    /// absent, the web build's default (`auto`) applies.
    #[test]
    fn pipelines_name_the_web_mode() {
        let on = settings_of("pipelines", "  pipelines async\n").unwrap();
        assert_eq!(on["publish"]["pipelines"], "async");
        let sync = settings_of("syncpipelines", "  pipelines sync\n").unwrap();
        assert_eq!(sync["publish"]["pipelines"], "sync");
        assert!(
            settings_of("nopipelines", "  gate tap\n").unwrap()["publish"]
                .get("pipelines")
                .is_none()
        );
        let bad = settings_of("badpipelines", "  pipelines lazy\n").unwrap_err();
        assert!(
            bad.contains("publish.pipelines is `lazy`; use one of auto, sync, async, prewarm"),
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

    /// Who draws the loading screen and the splash's captions reach the
    /// web build; a stage the boot does not have is refused by name.
    #[test]
    fn the_loading_screen_and_its_labels_reach_the_web_build() {
        let s = settings_of(
            "loading",
            "  loading page\n  loading_labels {\n    fetch \"Downloading\"\n    present \"Almost\"\n  }\n",
        )
        .unwrap();
        assert_eq!(s["publish"]["loading"], "page");
        assert_eq!(
            s["publish"]["loadingLabels"],
            serde_json::json!({ "fetch": "Downloading", "present": "Almost" })
        );
        let bad = settings_of("badloading", "  loading splash\n").unwrap_err();
        assert!(bad.contains("engine, game, page"), "{bad}");
        let stage =
            settings_of("badstage", "  loading_labels {\n    unpack \"…\"\n  }\n").unwrap_err();
        assert!(stage.contains("no stage `unpack`"), "{stage}");
    }

    /// An asset base reaches the web build without its trailing `/`; one a
    /// page could not fetch safely (plain http off localhost, credentials, a
    /// query or fragment, not absolute) is refused with its line.
    #[test]
    fn an_asset_base_is_an_https_directory_or_local_http() {
        let ok = settings_of("assets", "  assets \"https://cdn.example.com/g/1.4/\"\n").unwrap();
        assert_eq!(ok["publish"]["assets"], "https://cdn.example.com/g/1.4");
        for local in ["http://localhost:8080/g", "http://127.0.0.1:9/g"] {
            assert_eq!(super::asset_base(local).as_deref(), Ok(local));
        }
        for bad in [
            "http://cdn.example.com/g",
            "http://localhost.evil.example/g",
            "https://user:pw@cdn.example.com/g",
            "https://cdn.example.com/g?v=1",
            "https://cdn.example.com/g#x",
            "cdn.example.com/g",
            "https:///g",
        ] {
            assert!(super::asset_base(bad).is_err(), "{bad}");
        }
        let line = settings_of("bad-assets", "  assets \"http://cdn.example.com\"\n").unwrap_err();
        assert!(line.contains(":7: publish.assets"), "{line}");
    }
}
